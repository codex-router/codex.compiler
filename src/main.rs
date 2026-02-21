#![allow(dead_code)]
mod error;
mod language;
mod lexer;
mod parser;
mod token;

use clap::Parser as ClapParser;
use colored::Colorize;
use error::{DiagnosticBag, FileResult, Severity};
use language::Language;
use lexer::Lexer;
use parser::{c_parser, java_parser, Parser};
use rayon::prelude::*;
use std::{fs, time::Instant};

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(ClapParser, Debug)]
#[command(
    name = "tcc",
    version = "0.1.0",
    about = "Codex Compiler – fast grammar verification for C / C++ / Java",
    long_about = None
)]
struct Cli {
    /// Source files to verify (*.c, *.cpp, *.cc, *.h, *.hpp, *.java)
    #[arg(required = true)]
    files: Vec<String>,

    /// Stop after the first error in each file
    #[arg(short = 'f', long = "fast-fail")]
    fast_fail: bool,

    /// Maximum errors to report per file (0 = unlimited)
    #[arg(short = 'e', long = "error-limit", default_value = "0")]
    error_limit: usize,

    /// Print timings
    #[arg(short = 't', long = "timings")]
    timings: bool,

    /// Show full token stream before parsing (debug)
    #[arg(long = "dump-tokens")]
    dump_tokens: bool,

    /// Do not use coloured output
    #[arg(long = "no-color")]
    no_color: bool,
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    println!(
        "{} – grammar checker for C / C++ / Java",
        "tcc".bold().cyan()
    );
    println!();

    let wall_start = Instant::now();

    // Collect (path, lang) pairs – report unknown extensions immediately
    let jobs: Vec<(String, Language)> = cli
        .files
        .iter()
        .filter_map(|f| match Language::from_path(f) {
            Some(lang) => Some((f.clone(), lang)),
            None => {
                eprintln!(
                    "{} {}: unknown file extension, skipping",
                    "warn".yellow().bold(),
                    f
                );
                None
            }
        })
        .collect();

    if jobs.is_empty() {
        eprintln!("{}", "No valid input files.".red());
        std::process::exit(1);
    }

    // Process files in parallel
    let results: Vec<FileResult> = jobs
        .par_iter()
        .map(|(path, lang)| compile_file(path, *lang, cli.error_limit, cli.fast_fail, cli.dump_tokens))
        .collect();

    let wall_elapsed = wall_start.elapsed();

    // ── Print results ─────────────────────────────────────────────────
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;
    let mut files_ok = 0usize;

    for result in &results {
        let ec = result.diags.error_count();
        let wc = result.diags.warning_count();
        total_errors += ec;
        total_warnings += wc;

        // Print diagnostics
        for diag in &result.diags.items {
            let prefix = match diag.severity {
                Severity::Error => "error".red().bold(),
                Severity::Warning => "warning".yellow().bold(),
            };
            println!(
                "{}:{}:{}: {}: {}",
                result.path,
                diag.span.line,
                diag.span.col,
                prefix,
                diag.message
            );
        }

        // File summary
        if ec == 0 {
            files_ok += 1;
            println!(
                "{} {} ({}  {} lines)  {}",
                "ok".green().bold(),
                result.path,
                result.diags.items.len()
                    .to_string()
                    .dimmed(),
                result.lines,
                Language::from_path(&result.path)
                    .map(|l| l.name())
                    .unwrap_or("?")
                    .dimmed()
            );
        } else {
            println!(
                "{} {} – {} error(s), {} warning(s)  [{} lines]",
                "FAIL".red().bold(),
                result.path,
                ec.to_string().red(),
                wc.to_string().yellow(),
                result.lines
            );
        }
    }

    // Overall summary
    println!();
    println!(
        "Files: {}/{} ok  |  Errors: {}  |  Warnings: {}",
        files_ok.to_string().green(),
        results.len(),
        if total_errors > 0 { total_errors.to_string().red() } else { total_errors.to_string().green() },
        if total_warnings > 0 { total_warnings.to_string().yellow() } else { total_warnings.to_string().green() },
    );

    if cli.timings {
        println!(
            "Time: {:.2}ms  (wall, {} thread(s))",
            wall_elapsed.as_secs_f64() * 1000.0,
            rayon::current_num_threads()
        );
    }

    std::process::exit(if total_errors > 0 { 1 } else { 0 });
}

// ── Per-file compilation ──────────────────────────────────────────────────────

fn compile_file(
    path: &str,
    lang: Language,
    error_limit: usize,
    fast_fail: bool,
    dump_tokens: bool,
) -> FileResult {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            let mut diags = DiagnosticBag::new(error_limit);
            diags.error(crate::token::Span::new(0, 0), format!("cannot read file: {e}"));
            return FileResult { path: path.to_string(), diags, lines: 0 };
        }
    };

    let lines = src.lines().count();

    // Lex
    let mut lex_diags = DiagnosticBag::new(error_limit);
    let tokens = Lexer::new(&src, lang).tokenize(&mut lex_diags);

    if dump_tokens {
        eprintln!("=== TOKEN DUMP: {} ===", path);
        for tok in &tokens {
            eprintln!("  {:?}", tok);
        }
    }

    // Parse
    let mut p = Parser::new(&tokens, error_limit, fast_fail);
    // Merge lex errors first
    p.diags.items.extend(lex_diags.items);

    match lang {
        Language::C => c_parser::parse(&mut p, Language::C),
        Language::Cpp => c_parser::parse(&mut p, Language::Cpp),
        Language::Java => java_parser::parse(&mut p),
    }

    // Sort by position
    p.diags.items.sort_by_key(|d| (d.span.line, d.span.col));

    FileResult {
        path: path.to_string(),
        diags: p.diags,
        lines,
    }
}

