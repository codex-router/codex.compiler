use crate::token::Span;

#[allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A single diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn error(span: Span, msg: impl Into<String>) -> Self {
        Self { severity: Severity::Error, span, message: msg.into() }
    }
    pub fn warning(span: Span, msg: impl Into<String>) -> Self {
        Self { severity: Severity::Warning, span, message: msg.into() }
    }
}

/// Collection of diagnostics for one file.
#[derive(Debug, Default)]
pub struct DiagnosticBag {
    pub items: Vec<Diagnostic>,
    /// Stop collecting after this many errors (0 = unlimited)
    pub error_limit: usize,
}

impl DiagnosticBag {
    pub fn new(error_limit: usize) -> Self {
        Self { items: Vec::new(), error_limit }
    }

    pub fn push(&mut self, d: Diagnostic) {
        self.items.push(d);
    }

    pub fn error(&mut self, span: Span, msg: impl Into<String>) {
        self.push(Diagnostic::error(span, msg));
    }

    pub fn warning(&mut self, span: Span, msg: impl Into<String>) {
        self.push(Diagnostic::warning(span, msg));
    }

    pub fn error_count(&self) -> usize {
        self.items.iter().filter(|d| d.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.items.iter().filter(|d| d.severity == Severity::Warning).count()
    }

    /// Returns true when we have already collected too many errors.
    pub fn too_many_errors(&self) -> bool {
        self.error_limit > 0 && self.error_count() >= self.error_limit
    }
}

/// Final result for one compiled file.
pub struct FileResult {
    pub path: String,
    pub diags: DiagnosticBag,
    pub lines: usize,
}

impl FileResult {
    pub fn ok(&self) -> bool {
        self.diags.error_count() == 0
    }
}
