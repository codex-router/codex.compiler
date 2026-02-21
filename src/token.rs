/// Source position (1-based line and column)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: u32,
    pub col: u32,
}

impl Span {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// A single token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, line: u32, col: u32) -> Self {
        Self { kind, span: Span::new(line, col) }
    }
}

/// All token kinds shared across C, C++, and Java.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Literals ──────────────────────────────────────────────────────
    IntLit(i64),
    FloatLit(f64),
    CharLit(char),
    StringLit(String),
    BoolLit(bool),   // true / false (Java)
    NullLit,         // null (Java) / NULL (C/C++ – treated as ident)

    // ── Identifier / Keywords ─────────────────────────────────────────
    Ident(String),

    // C / C++ keywords
    KwAuto,
    KwBreak,
    KwCase,
    KwChar,
    KwConst,
    KwContinue,
    KwDefault,
    KwDo,
    KwDouble,
    KwElse,
    KwEnum,
    KwExtern,
    KwFloat,
    KwFor,
    KwGoto,
    KwIf,
    KwInt,
    KwLong,
    KwRegister,
    KwReturn,
    KwShort,
    KwSigned,
    KwSizeof,
    KwStatic,
    KwStruct,
    KwSwitch,
    KwTypedef,
    KwUnion,
    KwUnsigned,
    KwVoid,
    KwVolatile,
    KwWhile,
    // C++ extras
    KwClass,
    KwNew,
    KwDelete,
    KwThis,
    KwNamespace,
    KwUsing,
    KwPublic,
    KwProtected,
    KwPrivate,
    KwVirtual,
    KwOverride,
    KwFinal,
    KwTemplate,
    KwTypename,
    KwTry,
    KwCatch,
    KwThrow,
    KwBool,
    KwConstexpr,
    KwInline,
    KwExplicit,
    KwFriend,
    KwOperator,
    // Java keywords
    KwAbstract,
    KwAssert,
    KwByte,
    KwExtends,
    KwFinally,
    KwImplements,
    KwImport,
    KwInstanceof,
    KwInterface,
    KwNative,
    KwPackage,
    KwStrictfp,
    KwSuper,
    KwSynchronized,
    KwThrows,
    KwTransient,

    // ── Delimiters ────────────────────────────────────────────────────
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    Semi,      // ;
    Colon,     // :
    DoubleColon, // ::
    Comma,     // ,
    Dot,       // .
    Arrow,     // ->
    Ellipsis,  // ...

    // ── Operators ─────────────────────────────────────────────────────
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Percent,   // %
    Amp,       // &
    Pipe,      // |
    Caret,     // ^
    Tilde,     // ~
    Bang,      // !
    Eq,        // =
    Lt,        // <
    Gt,        // >
    Question,  // ?
    Hash,      // # (preprocessor)

    PlusPlus,   // ++
    MinusMinus, // --
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    PercentEq,  // %=
    AmpEq,      // &=
    PipeEq,     // |=
    CaretEq,    // ^=
    LtLt,       // <<
    GtGt,       // >>
    LtLtEq,     // <<=
    GtGtEq,     // >>=
    EqEq,       // ==
    BangEq,     // !=
    LtEq,       // <=
    GtEq,       // >=
    AmpAmp,     // &&
    PipePipe,   // ||
    DotStar,    // .*   (C++)
    ArrowStar,  // ->*  (C++)
    // Java-specific triple shift
    GtGtGt,     // >>>
    GtGtGtEq,   // >>>=

    // ── Preprocessor line (C/C++) ─────────────────────────────────────
    PreprocLine(String),

    // ── End of file ───────────────────────────────────────────────────
    Eof,
}

impl TokenKind {
    /// Map a string to a C keyword token, if applicable.
    pub fn c_keyword(s: &str) -> Option<TokenKind> {
        match s {
            "auto" => Some(TokenKind::KwAuto),
            "break" => Some(TokenKind::KwBreak),
            "case" => Some(TokenKind::KwCase),
            "char" => Some(TokenKind::KwChar),
            "const" => Some(TokenKind::KwConst),
            "continue" => Some(TokenKind::KwContinue),
            "default" => Some(TokenKind::KwDefault),
            "do" => Some(TokenKind::KwDo),
            "double" => Some(TokenKind::KwDouble),
            "else" => Some(TokenKind::KwElse),
            "enum" => Some(TokenKind::KwEnum),
            "extern" => Some(TokenKind::KwExtern),
            "float" => Some(TokenKind::KwFloat),
            "for" => Some(TokenKind::KwFor),
            "goto" => Some(TokenKind::KwGoto),
            "if" => Some(TokenKind::KwIf),
            "int" => Some(TokenKind::KwInt),
            "long" => Some(TokenKind::KwLong),
            "register" => Some(TokenKind::KwRegister),
            "return" => Some(TokenKind::KwReturn),
            "short" => Some(TokenKind::KwShort),
            "signed" => Some(TokenKind::KwSigned),
            "sizeof" => Some(TokenKind::KwSizeof),
            "static" => Some(TokenKind::KwStatic),
            "struct" => Some(TokenKind::KwStruct),
            "switch" => Some(TokenKind::KwSwitch),
            "typedef" => Some(TokenKind::KwTypedef),
            "union" => Some(TokenKind::KwUnion),
            "unsigned" => Some(TokenKind::KwUnsigned),
            "void" => Some(TokenKind::KwVoid),
            "volatile" => Some(TokenKind::KwVolatile),
            "while" => Some(TokenKind::KwWhile),
            _ => None,
        }
    }

    /// Additional C++ keywords (superset of C).
    pub fn cpp_keyword(s: &str) -> Option<TokenKind> {
        if let Some(k) = Self::c_keyword(s) {
            return Some(k);
        }
        match s {
            "class" => Some(TokenKind::KwClass),
            "new" => Some(TokenKind::KwNew),
            "delete" => Some(TokenKind::KwDelete),
            "this" => Some(TokenKind::KwThis),
            "namespace" => Some(TokenKind::KwNamespace),
            "using" => Some(TokenKind::KwUsing),
            "public" => Some(TokenKind::KwPublic),
            "protected" => Some(TokenKind::KwProtected),
            "private" => Some(TokenKind::KwPrivate),
            "virtual" => Some(TokenKind::KwVirtual),
            "override" => Some(TokenKind::KwOverride),
            "final" => Some(TokenKind::KwFinal),
            "template" => Some(TokenKind::KwTemplate),
            "typename" => Some(TokenKind::KwTypename),
            "try" => Some(TokenKind::KwTry),
            "catch" => Some(TokenKind::KwCatch),
            "throw" => Some(TokenKind::KwThrow),
            "bool" => Some(TokenKind::KwBool),
            "true" => Some(TokenKind::BoolLit(true)),
            "false" => Some(TokenKind::BoolLit(false)),
            "nullptr" => Some(TokenKind::NullLit),
            "constexpr" => Some(TokenKind::KwConstexpr),
            "inline" => Some(TokenKind::KwInline),
            "explicit" => Some(TokenKind::KwExplicit),
            "friend" => Some(TokenKind::KwFriend),
            "operator" => Some(TokenKind::KwOperator),
            _ => None,
        }
    }

    /// Java keywords.
    pub fn java_keyword(s: &str) -> Option<TokenKind> {
        match s {
            "abstract" => Some(TokenKind::KwAbstract),
            "assert" => Some(TokenKind::KwAssert),
            "boolean" => Some(TokenKind::KwBool),
            "break" => Some(TokenKind::KwBreak),
            "byte" => Some(TokenKind::KwByte),
            "case" => Some(TokenKind::KwCase),
            "catch" => Some(TokenKind::KwCatch),
            "char" => Some(TokenKind::KwChar),
            "class" => Some(TokenKind::KwClass),
            "const" => Some(TokenKind::KwConst),
            "continue" => Some(TokenKind::KwContinue),
            "default" => Some(TokenKind::KwDefault),
            "do" => Some(TokenKind::KwDo),
            "double" => Some(TokenKind::KwDouble),
            "else" => Some(TokenKind::KwElse),
            "enum" => Some(TokenKind::KwEnum),
            "extends" => Some(TokenKind::KwExtends),
            "final" => Some(TokenKind::KwFinal),
            "finally" => Some(TokenKind::KwFinally),
            "float" => Some(TokenKind::KwFloat),
            "for" => Some(TokenKind::KwFor),
            "goto" => Some(TokenKind::KwGoto),
            "if" => Some(TokenKind::KwIf),
            "implements" => Some(TokenKind::KwImplements),
            "import" => Some(TokenKind::KwImport),
            "instanceof" => Some(TokenKind::KwInstanceof),
            "int" => Some(TokenKind::KwInt),
            "interface" => Some(TokenKind::KwInterface),
            "long" => Some(TokenKind::KwLong),
            "native" => Some(TokenKind::KwNative),
            "new" => Some(TokenKind::KwNew),
            "package" => Some(TokenKind::KwPackage),
            "private" => Some(TokenKind::KwPrivate),
            "protected" => Some(TokenKind::KwProtected),
            "public" => Some(TokenKind::KwPublic),
            "return" => Some(TokenKind::KwReturn),
            "short" => Some(TokenKind::KwShort),
            "static" => Some(TokenKind::KwStatic),
            "strictfp" => Some(TokenKind::KwStrictfp),
            "super" => Some(TokenKind::KwSuper),
            "switch" => Some(TokenKind::KwSwitch),
            "synchronized" => Some(TokenKind::KwSynchronized),
            "this" => Some(TokenKind::KwThis),
            "throw" => Some(TokenKind::KwThrow),
            "throws" => Some(TokenKind::KwThrows),
            "transient" => Some(TokenKind::KwTransient),
            "try" => Some(TokenKind::KwTry),
            "void" => Some(TokenKind::KwVoid),
            "volatile" => Some(TokenKind::KwVolatile),
            "while" => Some(TokenKind::KwWhile),
            "true" => Some(TokenKind::BoolLit(true)),
            "false" => Some(TokenKind::BoolLit(false)),
            "null" => Some(TokenKind::NullLit),
            _ => None,
        }
    }

    pub fn is_type_start(&self) -> bool {
        matches!(
            self,
            TokenKind::KwInt
                | TokenKind::KwChar
                | TokenKind::KwFloat
                | TokenKind::KwDouble
                | TokenKind::KwLong
                | TokenKind::KwShort
                | TokenKind::KwUnsigned
                | TokenKind::KwSigned
                | TokenKind::KwVoid
                | TokenKind::KwBool
                | TokenKind::KwByte
                | TokenKind::KwConst
                | TokenKind::KwStatic
                | TokenKind::KwExtern
                | TokenKind::KwInline
                | TokenKind::KwVolatile
                | TokenKind::KwConstexpr
                | TokenKind::KwVirtual
                | TokenKind::KwAbstract
                | TokenKind::KwSynchronized
                | TokenKind::KwNative
                | TokenKind::KwTransient
                | TokenKind::KwStrictfp
                | TokenKind::KwFinal
                | TokenKind::KwPublic
                | TokenKind::KwProtected
                | TokenKind::KwPrivate
                | TokenKind::KwStruct
                | TokenKind::KwUnion
                | TokenKind::KwEnum
                | TokenKind::KwClass
                | TokenKind::Ident(_)
        )
    }
}
