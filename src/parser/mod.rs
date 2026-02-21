pub mod c_parser;
pub mod java_parser;

use crate::{
    error::DiagnosticBag,
    token::{Span, Token, TokenKind},
};

/// Shared recursive-descent cursor used by all language parsers.
pub struct Parser<'t> {
    tokens: &'t [Token],
    pos: usize,
    pub diags: DiagnosticBag,
    pub fast_fail: bool,
}

impl<'t> Parser<'t> {
    pub fn new(tokens: &'t [Token], error_limit: usize, fast_fail: bool) -> Self {
        Self { tokens, pos: 0, diags: DiagnosticBag::new(error_limit), fast_fail }
    }

    // ── Token cursor helpers ──────────────────────────────────────────

    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    pub fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    pub fn peek2_kind(&self) -> &TokenKind {
        let idx = (self.pos + 1).min(self.tokens.len() - 1);
        &self.tokens[idx].kind
    }

    /// Current cursor position (token index).
    pub fn pos(&self) -> usize { self.pos }

    /// Read-only access to the full token slice (for lookahead heuristics).
    pub fn tokens(&self) -> &[Token] { self.tokens }

    pub fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    pub fn span(&self) -> Span {
        self.peek().span
    }

    pub fn at_eof(&self) -> bool {
        self.peek_kind() == &TokenKind::Eof
    }

    pub fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind)
    }

    pub fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume a specific token or emit an error and recover.
    pub fn expect(&mut self, kind: &TokenKind, desc: &str) -> bool {
        if self.eat(kind) {
            true
        } else {
            if !self.diags.too_many_errors() {
                self.diags.error(
                    self.span(),
                    format!("expected {}, got {:?}", desc, self.peek_kind()),
                );
            }
            false
        }
    }

    /// Check whether the current token is an identifier (any value).
    pub fn is_ident(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Ident(_))
    }

    /// Consume the current identifier token and return its text.
    pub fn expect_ident(&mut self) -> Option<String> {
        if let TokenKind::Ident(s) = self.peek_kind().clone() {
            self.advance();
            Some(s)
        } else {
            if !self.diags.too_many_errors() {
                self.diags.error(self.span(), format!("expected identifier, got {:?}", self.peek_kind()));
            }
            None
        }
    }

    /// Skip tokens until we reach one of the synchronisation points.
    pub fn sync_to(&mut self, sync: &[TokenKind]) {
        while !self.at_eof() {
            for s in sync {
                if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(s) {
                    return;
                }
            }
            self.advance();
        }
    }

    /// Skip to end of statement (next ';' or '}').
    pub fn skip_to_stmt_end(&mut self) {
        self.sync_to(&[TokenKind::Semi, TokenKind::RBrace]);
        self.eat(&TokenKind::Semi);
    }

    pub fn should_abort(&self) -> bool {
        self.fast_fail && self.diags.too_many_errors()
    }
}
