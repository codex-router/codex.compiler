use crate::{
    error::DiagnosticBag,
    language::Language,
    token::{Span, Token, TokenKind},
};

#[allow(dead_code)]

pub struct Lexer<'src> {
    src: &'src [u8],
    pos: usize,
    line: u32,
    col: u32,
    lang: Language,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str, lang: Language) -> Self {
        Self { src: src.as_bytes(), pos: 0, line: 1, col: 1, lang }
    }

    // ── Source navigation ─────────────────────────────────────────────

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<u8> {
        self.src.get(self.pos + 1).copied()
    }

    fn peek3(&self) -> Option<u8> {
        self.src.get(self.pos + 2).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let ch = self.src.get(self.pos).copied()?;
        self.pos += 1;
        if ch == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn eat(&mut self, ch: u8) -> bool {
        if self.peek() == Some(ch) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ── Skip whitespace and comments ──────────────────────────────────

    fn skip_whitespace_and_comments(&mut self, diags: &mut DiagnosticBag) {
        loop {
            match self.peek() {
                Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n') => {
                    self.advance();
                }
                Some(b'/') if self.peek2() == Some(b'/') => {
                    // Line comment
                    while self.peek().is_some_and(|c| c != b'\n') {
                        self.advance();
                    }
                }
                Some(b'/') if self.peek2() == Some(b'*') => {
                    // Block comment
                    let sp = self.span();
                    self.advance(); // /
                    self.advance(); // *
                    loop {
                        match self.peek() {
                            None => {
                                diags.error(sp, "unterminated block comment");
                                return;
                            }
                            Some(b'*') if self.peek2() == Some(b'/') => {
                                self.advance();
                                self.advance();
                                break;
                            }
                            _ => {
                                self.advance();
                            }
                        }
                    }
                }
                _ => break,
            }
        }
    }

    // ── Tokenise one token ────────────────────────────────────────────

    fn next_token(&mut self, diags: &mut DiagnosticBag) -> Token {
        self.skip_whitespace_and_comments(diags);
        let sp = self.span();

        let ch = match self.peek() {
            None => return Token::new(TokenKind::Eof, sp.line, sp.col),
            Some(c) => c,
        };

        // Preprocessor line  (#...)
        if ch == b'#' && (self.lang == Language::C || self.lang == Language::Cpp) {
            self.advance(); // consume '#'
            let mut line_text = String::new();
            // Collect the rest of the logical line (handle line-continuation \<nl>)
            loop {
                match self.peek() {
                    None | Some(b'\n') => break,
                    Some(b'\\') if self.peek2() == Some(b'\n') => {
                        self.advance();
                        self.advance();
                        line_text.push(' ');
                    }
                    Some(c) => {
                        line_text.push(c as char);
                        self.advance();
                    }
                }
            }
            return Token::new(TokenKind::PreprocLine(line_text.trim().to_string()), sp.line, sp.col);
        }

        // Identifiers and keywords
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let mut s = String::new();
            while self.peek().is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_') {
                s.push(self.advance().unwrap() as char);
            }
            let kind = match self.lang {
                Language::C => TokenKind::c_keyword(&s),
                Language::Cpp => TokenKind::cpp_keyword(&s),
                Language::Java => TokenKind::java_keyword(&s),
            }
            .unwrap_or(TokenKind::Ident(s));
            return Token::new(kind, sp.line, sp.col);
        }

        // Numbers
        if ch.is_ascii_digit() || (ch == b'.' && self.peek2().is_some_and(|c| c.is_ascii_digit())) {
            return self.lex_number(sp, diags);
        }

        // String literal
        if ch == b'"' {
            return self.lex_string(sp, diags);
        }

        // Char literal
        if ch == b'\'' {
            return self.lex_char(sp, diags);
        }

        // Operators and delimiters
        self.advance();
        let kind = match ch {
            b'(' => TokenKind::LParen,
            b')' => TokenKind::RParen,
            b'{' => TokenKind::LBrace,
            b'}' => TokenKind::RBrace,
            b'[' => TokenKind::LBracket,
            b']' => TokenKind::RBracket,
            b';' => TokenKind::Semi,
            b',' => TokenKind::Comma,
            b'~' => TokenKind::Tilde,
            b'?' => TokenKind::Question,
            b'@' => TokenKind::Ident("@".to_string()), // Java annotations
            b':' => {
                if self.eat(b':') { TokenKind::DoubleColon } else { TokenKind::Colon }
            }
            b'.' => {
                if self.peek() == Some(b'.') && self.peek2() == Some(b'.') {
                    self.advance(); self.advance();
                    TokenKind::Ellipsis
                } else if self.peek() == Some(b'*') {
                    self.advance();
                    TokenKind::DotStar
                } else {
                    TokenKind::Dot
                }
            }
            b'+' => {
                if self.eat(b'+') { TokenKind::PlusPlus }
                else if self.eat(b'=') { TokenKind::PlusEq }
                else { TokenKind::Plus }
            }
            b'-' => {
                if self.eat(b'-') { TokenKind::MinusMinus }
                else if self.eat(b'=') { TokenKind::MinusEq }
                else if self.peek() == Some(b'>') && self.peek2() == Some(b'*') {
                    self.advance(); self.advance();
                    TokenKind::ArrowStar
                } else if self.eat(b'>') { TokenKind::Arrow }
                else { TokenKind::Minus }
            }
            b'*' => {
                if self.eat(b'=') { TokenKind::StarEq } else { TokenKind::Star }
            }
            b'/' => {
                if self.eat(b'=') { TokenKind::SlashEq } else { TokenKind::Slash }
            }
            b'%' => {
                if self.eat(b'=') { TokenKind::PercentEq } else { TokenKind::Percent }
            }
            b'&' => {
                if self.eat(b'&') { TokenKind::AmpAmp }
                else if self.eat(b'=') { TokenKind::AmpEq }
                else { TokenKind::Amp }
            }
            b'|' => {
                if self.eat(b'|') { TokenKind::PipePipe }
                else if self.eat(b'=') { TokenKind::PipeEq }
                else { TokenKind::Pipe }
            }
            b'^' => {
                if self.eat(b'=') { TokenKind::CaretEq } else { TokenKind::Caret }
            }
            b'!' => {
                if self.eat(b'=') { TokenKind::BangEq } else { TokenKind::Bang }
            }
            b'=' => {
                if self.eat(b'=') { TokenKind::EqEq } else { TokenKind::Eq }
            }
            b'<' => {
                if self.eat(b'<') {
                    if self.eat(b'=') { TokenKind::LtLtEq } else { TokenKind::LtLt }
                } else if self.eat(b'=') { TokenKind::LtEq }
                else { TokenKind::Lt }
            }
            b'>' => {
                if self.eat(b'>') {
                    if self.lang == Language::Java && self.eat(b'>') {
                        if self.eat(b'=') { TokenKind::GtGtGtEq } else { TokenKind::GtGtGt }
                    } else if self.eat(b'=') { TokenKind::GtGtEq }
                    else { TokenKind::GtGt }
                } else if self.eat(b'=') { TokenKind::GtEq }
                else { TokenKind::Gt }
            }
            other => {
                diags.error(sp, format!("unexpected character '{}'", other as char));
                TokenKind::Ident(format!("{}", other as char))
            }
        };

        Token::new(kind, sp.line, sp.col)
    }

    fn lex_number(&mut self, sp: Span, _diags: &mut DiagnosticBag) -> Token {
        let start = self.pos;
        let mut is_float = false;

        // Hex
        if self.peek() == Some(b'0')
            && (self.peek2() == Some(b'x') || self.peek2() == Some(b'X'))
        {
            self.advance(); self.advance(); // 0x
            while self.peek().is_some_and(|c| c.is_ascii_hexdigit() || c == b'_') {
                self.advance();
            }
            let text = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("0");
            let val = i64::from_str_radix(&text[2..].replace('_', ""), 16).unwrap_or(0);
            self.lex_int_suffix();
            return Token::new(TokenKind::IntLit(val), sp.line, sp.col);
        }

        // Binary (C++ / Java)
        if self.peek() == Some(b'0')
            && (self.peek2() == Some(b'b') || self.peek2() == Some(b'B'))
        {
            self.advance(); self.advance();
            while self.peek().is_some_and(|c| c == b'0' || c == b'1' || c == b'_') {
                self.advance();
            }
            let text = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("0");
            let val = i64::from_str_radix(&text[2..].replace('_', ""), 2).unwrap_or(0);
            self.lex_int_suffix();
            return Token::new(TokenKind::IntLit(val), sp.line, sp.col);
        }

        // Decimal / float
        while self.peek().is_some_and(|c| c.is_ascii_digit() || c == b'_') {
            self.advance();
        }
        if self.peek() == Some(b'.') && self.peek2().is_some_and(|c| c.is_ascii_digit() || c.is_ascii_digit()) {
            is_float = true;
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit() || c == b'_') {
                self.advance();
            }
        }
        // exponent
        if self.peek().is_some_and(|c| c == b'e' || c == b'E') {
            is_float = true;
            self.advance();
            if self.peek().is_some_and(|c| c == b'+' || c == b'-') {
                self.advance();
            }
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        // suffix
        while self.peek().is_some_and(|c| matches!(c, b'f' | b'F' | b'l' | b'L' | b'u' | b'U' | b'd' | b'D')) {
            if matches!(self.peek(), Some(b'f') | Some(b'F') | Some(b'd') | Some(b'D')) {
                is_float = true;
            }
            self.advance();
        }

        let raw = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("0");
        let clean = raw.replace('_', "");

        if is_float {
            let val: f64 = clean.trim_end_matches(|c| matches!(c, 'f' | 'F' | 'l' | 'L' | 'd' | 'D'))
                .parse().unwrap_or(0.0);
            Token::new(TokenKind::FloatLit(val), sp.line, sp.col)
        } else {
            let val: i64 = clean.trim_end_matches(|c| matches!(c, 'l' | 'L' | 'u' | 'U'))
                .parse().unwrap_or(0);
            Token::new(TokenKind::IntLit(val), sp.line, sp.col)
        }
    }

    fn lex_int_suffix(&mut self) {
        while self.peek().is_some_and(|c| matches!(c, b'u' | b'U' | b'l' | b'L')) {
            self.advance();
        }
    }

    fn lex_string(&mut self, sp: Span, diags: &mut DiagnosticBag) -> Token {
        self.advance(); // opening "
        let mut s = String::new();
        loop {
            match self.peek() {
                None | Some(b'\n') => {
                    diags.error(sp, "unterminated string literal");
                    break;
                }
                Some(b'"') => {
                    self.advance();
                    break;
                }
                Some(b'\\') => {
                    self.advance();
                    if let Some(esc) = self.advance() {
                        s.push(unescape(esc));
                    }
                }
                Some(c) => {
                    s.push(c as char);
                    self.advance();
                }
            }
        }
        Token::new(TokenKind::StringLit(s), sp.line, sp.col)
    }

    fn lex_char(&mut self, sp: Span, diags: &mut DiagnosticBag) -> Token {
        self.advance(); // opening '
        let ch = match self.advance() {
            None => {
                diags.error(sp, "unterminated char literal");
                return Token::new(TokenKind::CharLit('\0'), sp.line, sp.col);
            }
            Some(b'\\') => {
                let esc = self.advance().unwrap_or(b'?');
                unescape(esc)
            }
            Some(c) => c as char,
        };
        if !self.eat(b'\'') {
            diags.error(sp, "unterminated char literal");
        }
        Token::new(TokenKind::CharLit(ch), sp.line, sp.col)
    }

    /// Tokenise the entire source into a `Vec<Token>`.
    pub fn tokenize(&mut self, diags: &mut DiagnosticBag) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token(diags);
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

fn unescape(ch: u8) -> char {
    match ch {
        b'n' => '\n',
        b't' => '\t',
        b'r' => '\r',
        b'0' => '\0',
        b'\\' => '\\',
        b'\'' => '\'',
        b'"' => '"',
        b'a' => '\x07',
        b'b' => '\x08',
        b'f' => '\x0C',
        b'v' => '\x0B',
        other => other as char,
    }
}
