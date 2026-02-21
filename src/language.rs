use std::path::Path;

/// Source language being compiled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    C,
    Cpp,
    Java,
}

impl Language {
    /// Detect language from file extension.
    pub fn from_path(path: &str) -> Option<Self> {
        let ext = Path::new(path).extension()?.to_str()?;
        match ext {
            "c" => Some(Language::C),
            "cpp" | "cc" | "cxx" | "C" | "c++" => Some(Language::Cpp),
            "h" | "hpp" | "hxx" => Some(Language::Cpp),
            "java" => Some(Language::Java),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Java => "Java",
        }
    }
}
