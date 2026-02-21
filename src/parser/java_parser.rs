/// Recursive-descent grammar checker for Java.
use crate::{
    language::Language,
    parser::{c_parser, Parser},
    token::TokenKind,
};

// ── Public entry point ────────────────────────────────────────────────────────

pub fn parse(p: &mut Parser) {
    parse_compilation_unit(p);
}

// ── Compilation unit ──────────────────────────────────────────────────────────

fn parse_compilation_unit(p: &mut Parser) {
    // package declaration
    if matches!(p.peek_kind(), TokenKind::KwPackage) {
        p.advance();
        parse_qualified_name(p);
        p.expect(&TokenKind::Semi, "';'");
    }

    // import declarations
    while matches!(p.peek_kind(), TokenKind::KwImport) {
        p.advance();
        // optional static import
        if matches!(p.peek_kind(), TokenKind::KwStatic) { p.advance(); }
        parse_qualified_name(p);
        // optional  .*
        if p.check(&TokenKind::Dot) {
            p.advance();
            p.eat(&TokenKind::Star);
        }
        p.expect(&TokenKind::Semi, "';'");
    }

    // type declarations
    while !p.at_eof() && !p.should_abort() {
        // skip stray semicolons
        if p.eat(&TokenKind::Semi) { continue; }
        parse_type_decl(p);
    }
}

// ── Type declarations ─────────────────────────────────────────────────────────

fn parse_type_decl(p: &mut Parser) {
    skip_annotations(p);
    parse_modifiers(p);
    match p.peek_kind().clone() {
        TokenKind::KwClass => parse_class_decl(p),
        TokenKind::KwInterface => parse_interface_decl(p),
        TokenKind::KwEnum => parse_enum_decl(p),
        TokenKind::Ident(ref s) if s == "@" => {
            // @interface (annotation type)
            p.advance(); // @
            if matches!(p.peek_kind(), TokenKind::KwInterface) { p.advance(); }
            parse_class_body(p); // annotation body similar to interface
        }
        _ => {
            p.diags.error(p.span(), format!("expected class/interface/enum declaration, got {:?}", p.peek_kind()));
            // recovery: skip to next '{'  or EOF
            while !p.at_eof() && !p.check(&TokenKind::LBrace) && !p.check(&TokenKind::Semi) {
                p.advance();
            }
            if p.check(&TokenKind::LBrace) { skip_balanced_braces(p); }
        }
    }
}

// ── Class ─────────────────────────────────────────────────────────────────────

fn parse_class_decl(p: &mut Parser) {
    p.advance(); // 'class'
    if p.is_ident() { p.advance(); } else {
        p.diags.error(p.span(), "expected class name");
    }
    // type parameters  <T, U>
    if p.check(&TokenKind::Lt) { parse_type_params(p); }
    // extends
    if matches!(p.peek_kind(), TokenKind::KwExtends) {
        p.advance();
        parse_type_ref(p);
    }
    // implements
    if matches!(p.peek_kind(), TokenKind::KwImplements) {
        p.advance();
        loop {
            parse_type_ref(p);
            if !p.eat(&TokenKind::Comma) { break; }
        }
    }
    parse_class_body(p);
}

fn parse_interface_decl(p: &mut Parser) {
    p.advance(); // 'interface'
    if p.is_ident() { p.advance(); }
    if p.check(&TokenKind::Lt) { parse_type_params(p); }
    if matches!(p.peek_kind(), TokenKind::KwExtends) {
        p.advance();
        loop { parse_type_ref(p); if !p.eat(&TokenKind::Comma) { break; } }
    }
    parse_class_body(p);
}

fn parse_enum_decl(p: &mut Parser) {
    p.advance(); // 'enum'
    if p.is_ident() { p.advance(); }
    if matches!(p.peek_kind(), TokenKind::KwImplements) {
        p.advance();
        loop { parse_type_ref(p); if !p.eat(&TokenKind::Comma) { break; } }
    }
    p.expect(&TokenKind::LBrace, "'{'");
    // enum constants
    loop {
        if p.check(&TokenKind::RBrace) || p.check(&TokenKind::Semi) { break; }
        skip_annotations(p);
        if p.is_ident() { p.advance(); } else { break; }
        if p.check(&TokenKind::LParen) { parse_call_args(p); }
        if p.check(&TokenKind::LBrace) { parse_class_body(p); }
        if !p.eat(&TokenKind::Comma) { break; }
    }
    // optional body members after ';'
    if p.eat(&TokenKind::Semi) {
        while !p.at_eof() && !p.check(&TokenKind::RBrace) {
            if p.should_abort() { break; }
            let pos_before = p.pos;
            parse_class_member(p);
            if p.pos == pos_before && !p.at_eof() && !p.check(&TokenKind::RBrace) { p.advance(); }
        }
    }
    p.expect(&TokenKind::RBrace, "'}'");
}

fn parse_class_body(p: &mut Parser) {
    p.expect(&TokenKind::LBrace, "'{'");
    while !p.at_eof() && !p.check(&TokenKind::RBrace) {
        if p.should_abort() { break; }
        if p.eat(&TokenKind::Semi) { continue; }
        let pos_before = p.pos;
        parse_class_member(p);
        if p.pos == pos_before && !p.at_eof() && !p.check(&TokenKind::RBrace) { p.advance(); }
    }
    p.expect(&TokenKind::RBrace, "'}'");
}

fn parse_class_member(p: &mut Parser) {
    skip_annotations(p);
    parse_modifiers(p);

    match p.peek_kind().clone() {
        // Nested type
        TokenKind::KwClass => parse_class_decl(p),
        TokenKind::KwInterface => parse_interface_decl(p),
        TokenKind::KwEnum => parse_enum_decl(p),

        // Static / instance initializer block
        TokenKind::LBrace => {
            skip_balanced_braces(p);
        }

        // Everything else: field or method
        _ => {
            // Method-level type parameters: <T extends Comparable<T>>
            if p.check(&TokenKind::Lt) {
                parse_type_params(p);
            }
            // Type + name [params | = init]
            if parse_type_ref_opt(p) {
                // might be a constructor: ClassName(...)
                let is_ident = p.is_ident();
                if is_ident {
                    p.advance(); // name
                }
                match p.peek_kind() {
                    TokenKind::LParen => {
                        // method or constructor
                        parse_formal_params(p);
                        if matches!(p.peek_kind(), TokenKind::KwThrows) {
                            p.advance();
                            loop { parse_type_ref(p); if !p.eat(&TokenKind::Comma) { break; } }
                        }
                        if p.check(&TokenKind::LBrace) {
                            parse_method_body(p);
                        } else if p.check(&TokenKind::Semi) {
                            p.advance(); // abstract method or interface method
                        } else {
                            p.expect(&TokenKind::Semi, "';'");
                        }
                    }
                    TokenKind::LBracket => {
                        // field with array suffix
                        while p.eat(&TokenKind::LBracket) { p.expect(&TokenKind::RBracket, "']'"); }
                        parse_field_rest(p);
                    }
                    TokenKind::Eq | TokenKind::Comma | TokenKind::Semi => {
                        parse_field_rest(p);
                    }
                    _ => {
                        p.eat(&TokenKind::Semi);
                    }
                }
            } else {
                p.diags.error(p.span(), format!("unexpected token in class member: {:?}", p.peek_kind()));
                p.advance();
            }
        }
    }
}

fn parse_field_rest(p: &mut Parser) {
    // optional initializer(s)
    while p.eat(&TokenKind::Comma) {
        if p.is_ident() { p.advance(); }
        if p.eat(&TokenKind::Eq) {
            parse_var_initializer(p);
        }
    }
    if p.eat(&TokenKind::Eq) { parse_var_initializer(p); }
    p.expect(&TokenKind::Semi, "';'");
}

fn parse_var_initializer(p: &mut Parser) {
    if p.check(&TokenKind::LBrace) {
        p.advance(); // '{'
        while !p.at_eof() && !p.check(&TokenKind::RBrace) {
            parse_var_initializer(p);
            if !p.eat(&TokenKind::Comma) { break; }
        }
        p.expect(&TokenKind::RBrace, "'}'");
    } else {
        parse_assign_expr(p);
    }
}

fn parse_method_body(p: &mut Parser) {
    p.advance(); // '{'
    while !p.at_eof() && !p.check(&TokenKind::RBrace) {
        if p.should_abort() { break; }
        let pos_before = p.pos;
        c_parser::parse_statement(p, Language::Java);
        if p.pos == pos_before && !p.at_eof() && !p.check(&TokenKind::RBrace) { p.advance(); }
    }
    p.expect(&TokenKind::RBrace, "'}'");
}

// ── Formal parameters ─────────────────────────────────────────────────────────

fn parse_formal_params(p: &mut Parser) {
    p.advance(); // '('
    if p.check(&TokenKind::RParen) { p.advance(); return; }
    loop {
        skip_annotations(p);
        // optional 'final'
        if matches!(p.peek_kind(), TokenKind::KwFinal) { p.advance(); }
        skip_annotations(p);
        parse_type_ref(p);
        // vararg
        p.eat(&TokenKind::Ellipsis);
        if p.is_ident() { p.advance(); }
        // array dimensions
        while p.eat(&TokenKind::LBracket) { p.expect(&TokenKind::RBracket, "']'"); }
        if !p.eat(&TokenKind::Comma) { break; }
        if p.check(&TokenKind::RParen) { break; }
    }
    p.expect(&TokenKind::RParen, "')'");
}

fn parse_call_args(p: &mut Parser) {
    p.advance(); // '('
    if p.check(&TokenKind::RParen) { p.advance(); return; }
    loop {
        parse_assign_expr(p);
        if !p.eat(&TokenKind::Comma) { break; }
        if p.check(&TokenKind::RParen) { break; }
    }
    p.expect(&TokenKind::RParen, "')'");
}

fn parse_assign_expr(p: &mut Parser) {
    c_parser::parse_expr(p, Language::Java);
}

// ── Type references ───────────────────────────────────────────────────────────

/// Returns true if a type name was consumed.
fn parse_type_ref_opt(p: &mut Parser) -> bool {
    match p.peek_kind() {
        k if k.is_type_start() => { parse_type_ref(p); true }
        _ => false,
    }
}

fn parse_type_ref(p: &mut Parser) {
    match p.peek_kind() {
        TokenKind::KwVoid
        | TokenKind::KwInt
        | TokenKind::KwLong
        | TokenKind::KwShort
        | TokenKind::KwByte
        | TokenKind::KwChar
        | TokenKind::KwFloat
        | TokenKind::KwDouble
        | TokenKind::KwBool => { p.advance(); }
        TokenKind::Ident(_) => {
            parse_qualified_name(p);
            // type arguments
            if p.check(&TokenKind::Lt) { parse_type_args(p); }
        }
        _ => {}
    }
    // array dimensions
    while p.check(&TokenKind::LBracket) {
        p.advance();
        p.expect(&TokenKind::RBracket, "']'");
    }
}

fn parse_type_params(p: &mut Parser) {
    p.advance(); // '<'
    let mut depth = 1u32;
    while !p.at_eof() && depth > 0 {
        match p.peek_kind() {
            TokenKind::Lt => { depth += 1; p.advance(); }
            TokenKind::Gt => { depth -= 1; p.advance(); }
            TokenKind::GtGt => { if depth >= 2 { depth -= 2; } else { depth -= 1; } p.advance(); }
            _ => { p.advance(); }
        }
    }
}

fn parse_type_args(p: &mut Parser) {
    parse_type_params(p);
}

fn parse_qualified_name(p: &mut Parser) {
    if p.is_ident() { p.advance(); }
    while p.check(&TokenKind::Dot) {
        p.advance();
        if p.is_ident() { p.advance(); }
    }
}

// ── Modifiers ─────────────────────────────────────────────────────────────────

fn parse_modifiers(p: &mut Parser) {
    loop {
        match p.peek_kind() {
            TokenKind::KwPublic
            | TokenKind::KwProtected
            | TokenKind::KwPrivate
            | TokenKind::KwStatic
            | TokenKind::KwAbstract
            | TokenKind::KwFinal
            | TokenKind::KwNative
            | TokenKind::KwSynchronized
            | TokenKind::KwTransient
            | TokenKind::KwVolatile
            | TokenKind::KwStrictfp
            | TokenKind::KwDefault
            | TokenKind::KwInline => { p.advance(); }
            _ => break,
        }
    }
}

// ── Annotations ───────────────────────────────────────────────────────────────

/// Skip zero or more @annotation clauses.
fn skip_annotations(p: &mut Parser) {
    while p.check(&TokenKind::Ident(String::new())) {
        if let TokenKind::Ident(ref s) = p.peek_kind().clone() {
            if s == "@" {
                p.advance(); // @
                if p.is_ident() { p.advance(); }
                if p.check(&TokenKind::LParen) {
                    p.advance();
                    let mut depth = 1u32;
                    while !p.at_eof() && depth > 0 {
                        match p.peek_kind() {
                            TokenKind::LParen => { depth += 1; p.advance(); }
                            TokenKind::RParen => { depth -= 1; p.advance(); }
                            _ => { p.advance(); }
                        }
                    }
                }
                continue;
            }
        }
        break;
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Skip a balanced { … } block  (used for recovery / static initializers).
fn skip_balanced_braces(p: &mut Parser) {
    if !p.check(&TokenKind::LBrace) { return; }
    p.advance(); // '{'
    let mut depth = 1u32;
    while !p.at_eof() && depth > 0 {
        match p.peek_kind() {
            TokenKind::LBrace => { depth += 1; p.advance(); }
            TokenKind::RBrace => { depth -= 1; p.advance(); }
            _ => { p.advance(); }
        }
    }
}
