/// Recursive-descent grammar checker for C and C++.
use crate::{
    language::Language,
    parser::Parser,
    token::TokenKind,
};

// ── Public entry point ────────────────────────────────────────────────────────

/// Parse a token stream as C or C++.  Diagnostics are stored inside `p`.
pub fn parse(p: &mut Parser, lang: Language) {
    parse_translation_unit(p, lang);
}

// ── Translation unit ──────────────────────────────────────────────────────────

fn parse_translation_unit(p: &mut Parser, lang: Language) {
    while !p.at_eof() && !p.should_abort() {
        let pos_before = p.pos;
        parse_external_decl(p, lang);
        // Safety: if no progress was made, force advance to avoid infinite loop
        if p.pos == pos_before && !p.at_eof() {
            p.diags.error(p.span(), format!("unexpected token {:?}", p.peek_kind()));
            p.advance();
        }
    }
}

fn parse_external_decl(p: &mut Parser, lang: Language) {
    // Preprocessor directives – skip entirely
    if matches!(p.peek_kind(), TokenKind::PreprocLine(_)) {
        p.advance();
        return;
    }
    // Stray ';'
    if p.eat(&TokenKind::Semi) {
        return;
    }
    // extern "C" / extern "C++"  linkage spec
    if matches!(p.peek_kind(), TokenKind::KwExtern) {
        if let TokenKind::StringLit(_) = p.peek2_kind().clone() {
            p.advance(); // extern
            p.advance(); // string
            if p.check(&TokenKind::LBrace) {
                parse_compound_body(p, lang);
            } else {
                parse_external_decl(p, lang);
            }
            return;
        }
    }
    // namespace  (C++)
    if lang == Language::Cpp && matches!(p.peek_kind(), TokenKind::KwNamespace) {
        parse_namespace(p, lang);
        return;
    }
    // using declaration / directive  (C++)
    if lang == Language::Cpp && matches!(p.peek_kind(), TokenKind::KwUsing) {
        parse_using(p);
        return;
    }
    // template  (C++)
    if lang == Language::Cpp && matches!(p.peek_kind(), TokenKind::KwTemplate) {
        parse_template(p, lang);
        return;
    }

    // Everything else: a declaration or function definition
    parse_declaration_or_function(p, lang);
}

// ── Namespace ─────────────────────────────────────────────────────────────────

fn parse_namespace(p: &mut Parser, lang: Language) {
    p.advance(); // 'namespace'
    // optional name or inline
    if p.is_ident() { p.advance(); }
    if p.check(&TokenKind::LBrace) {
        p.advance(); // '{'
        while !p.at_eof() && !p.check(&TokenKind::RBrace) {
            if p.should_abort() { break; }
            let pos_before = p.pos;
            parse_external_decl(p, lang);
            if p.pos == pos_before && !p.at_eof() && !p.check(&TokenKind::RBrace) { p.advance(); }
        }
        p.expect(&TokenKind::RBrace, "'}'");
    } else {
        p.diags.error(p.span(), "expected '{' after namespace declaration");
        p.skip_to_stmt_end();
    }
}

fn parse_using(p: &mut Parser) {
    p.advance(); // 'using'
    if matches!(p.peek_kind(), TokenKind::KwNamespace) { p.advance(); }
    parse_qualified_name(p);
    p.expect(&TokenKind::Semi, "';'");
}

// ── Template ──────────────────────────────────────────────────────────────────

fn parse_template(p: &mut Parser, lang: Language) {
    p.advance(); // 'template'
    if p.check(&TokenKind::Lt) {
        p.advance(); // '<'
        skip_angle_brackets_content(p);
    }
    parse_external_decl(p, lang);
}

/// Skip everything inside  < … >  (nesting-aware)
fn skip_angle_brackets_content(p: &mut Parser) {
    let mut depth = 1u32;
    while !p.at_eof() && depth > 0 {
        match p.peek_kind() {
            TokenKind::Lt | TokenKind::LtLt => { depth += 1; p.advance(); }
            TokenKind::Gt => { depth -= 1; p.advance(); }
            TokenKind::GtGt => {
                // >> can close two levels
                if depth >= 2 { depth -= 2; } else { depth -= 1; }
                p.advance();
            }
            TokenKind::GtEq | TokenKind::GtGtEq => { depth -= 1; p.advance(); }
            _ => { p.advance(); }
        }
    }
}

// ── Declaration / function definition ────────────────────────────────────────

/// Attempt to parse a declaration or function definition.
fn parse_declaration_or_function(p: &mut Parser, lang: Language) {
    // Parse leading declaration specifiers
    if !parse_decl_specifiers(p, lang) {
        // Nothing recognised → skip token and try again
        p.diags.error(p.span(), format!("unexpected token {:?} in declaration", p.peek_kind()));
        p.advance();
        return;
    }

    // Constructor/destructor shorthand: Ident '(' …  with no specifiers before
    // (handled implicitly because Ident is a valid type-name specifier)

    // Declarator list (may be a function definition or variable declaration)
    parse_init_declarator_list(p, lang);
}

/// Returns `true` if at least one specifier was consumed.
///
/// Strategy: consume storage/qualifier keywords freely, but only consume ONE
/// user-type identifier (e.g. `MyType`) if no primitive type keyword has been
/// seen yet.  This prevents eating the declarator name as a type.
fn parse_decl_specifiers(p: &mut Parser, lang: Language) -> bool {
    let mut saw_any = false;
    // Whether we have already consumed a concrete type (int, char, struct…)
    // After that, an identifier is the declarator, not another type specifier.
    let mut saw_type = false;

    loop {
        match p.peek_kind() {
            // Storage class / function specifiers
            TokenKind::KwAuto
            | TokenKind::KwRegister
            | TokenKind::KwExtern
            | TokenKind::KwTypedef
            | TokenKind::KwInline
            | TokenKind::KwVirtual
            | TokenKind::KwExplicit
            | TokenKind::KwFriend
            | TokenKind::KwConstexpr
            | TokenKind::KwAbstract
            | TokenKind::KwSynchronized
            | TokenKind::KwNative
            | TokenKind::KwTransient
            | TokenKind::KwStrictfp => {
                saw_any = true;
                p.advance();
            }
            // Static is also a storage class but doubles as a type-adjacent
            TokenKind::KwStatic => {
                saw_any = true;
                p.advance();
            }
            // Type qualifiers
            TokenKind::KwConst | TokenKind::KwVolatile => {
                saw_any = true;
                p.advance();
            }
            // Primitive type specifiers – can appear multiple times (unsigned long int)
            TokenKind::KwVoid
            | TokenKind::KwChar
            | TokenKind::KwShort
            | TokenKind::KwInt
            | TokenKind::KwLong
            | TokenKind::KwFloat
            | TokenKind::KwDouble
            | TokenKind::KwSigned
            | TokenKind::KwUnsigned
            | TokenKind::KwBool
            | TokenKind::KwByte => {
                saw_any = true;
                saw_type = true;
                p.advance();
            }
            // Access specifiers (C++ / Java)
            TokenKind::KwPublic | TokenKind::KwProtected | TokenKind::KwPrivate => {
                saw_any = true;
                p.advance();
                // Could be followed by ':' (class body access label)
                p.eat(&TokenKind::Colon);
            }
            // struct / union
            TokenKind::KwStruct | TokenKind::KwUnion => {
                saw_any = true;
                saw_type = true;
                p.advance();
                parse_struct_or_union_body(p, lang);
                break;
            }
            // enum
            TokenKind::KwEnum => {
                saw_any = true;
                saw_type = true;
                p.advance();
                // optional 'class' (C++11 enum class)
                if lang == Language::Cpp { p.eat(&TokenKind::KwClass); }
                if p.is_ident() { p.advance(); }
                if p.check(&TokenKind::LBrace) { parse_enum_body(p); }
                // might have base type after ':'
                if p.eat(&TokenKind::Colon) { parse_type_name(p, lang); }
                break;
            }
            // class (C++ / Java)
            TokenKind::KwClass if !saw_type => {
                saw_any = true;
                saw_type = true;
                p.advance();
                parse_struct_or_union_body(p, lang);
                break;
            }
            // User-defined type name (typedef / class name) –
            // ONLY consume if we haven't seen a primitive type keyword yet.
            TokenKind::Ident(_) if !saw_type => {
                saw_any = true;
                saw_type = true;
                parse_qualified_name(p);

                // Handle template arguments e.g. vector<int> – only in C++
                if lang == Language::Cpp && p.check(&TokenKind::Lt) {
                    let peek_after = p.peek2_kind().clone();
                    let looks_like_type = peek_after.is_type_start()
                        || matches!(peek_after, TokenKind::Star | TokenKind::Amp | TokenKind::Gt);
                    if looks_like_type {
                        let save = p.pos;
                        let dsave = p.diags.items.len();
                        p.advance();
                        skip_angle_brackets_content(p);
                        // Should be followed by pointer/ref/ident/paren/brace/semi
                        if !p.peek_kind().is_type_start()
                            && !p.check(&TokenKind::Star)
                            && !p.check(&TokenKind::Amp)
                            && !p.check(&TokenKind::AmpAmp)
                            && !p.is_ident()
                            && !p.check(&TokenKind::LParen)
                            && !p.check(&TokenKind::LBrace)
                            && !p.check(&TokenKind::Semi)
                            && !p.check(&TokenKind::Comma)
                            && !p.check(&TokenKind::RBrace)
                            && !p.check(&TokenKind::Tilde)
                        {
                            p.pos = save;
                            p.diags.items.truncate(dsave);
                        }
                    }
                }
                break; // one user-defined type name is enough
            }
            // Destructor tilde
            TokenKind::Tilde if !saw_type => {
                saw_any = true;
                p.advance();
                if p.is_ident() { p.advance(); }
                break;
            }
            TokenKind::KwOperator if lang == Language::Cpp => {
                saw_any = true;
                p.advance();
                parse_operator_overload_type(p);
                break;
            }
            _ => break,
        }
    }
    saw_any
}

fn parse_type_name(p: &mut Parser, lang: Language) {
    parse_decl_specifiers(p, lang);
    // skip optional pointer/ref
    while p.eat(&TokenKind::Star) || p.eat(&TokenKind::Amp) {}
}

fn parse_struct_or_union_body(p: &mut Parser, lang: Language) {
    // optional name
    if p.is_ident() { p.advance(); }
    // optional base (C++ struct : Base)
    if lang == Language::Cpp && p.eat(&TokenKind::Colon) {
        loop {
            // access specifier
            match p.peek_kind() {
                TokenKind::KwPublic | TokenKind::KwProtected | TokenKind::KwPrivate => { p.advance(); }
                _ => {}
            }
            parse_type_name(p, lang);
            if !p.eat(&TokenKind::Comma) { break; }
        }
    }
    if p.check(&TokenKind::LBrace) {
        parse_class_body(p, lang);
    }
}

fn parse_operator_overload_type(p: &mut Parser) {
    // Skip until '(' – operator types can be very complex
    while !p.at_eof() && !p.check(&TokenKind::LParen) && !p.check(&TokenKind::Semi) {
        p.advance();
    }
}

fn parse_enum_body(p: &mut Parser) {
    p.advance(); // '{'
    while !p.at_eof() && !p.check(&TokenKind::RBrace) {
        if p.is_ident() { p.advance(); } else { p.advance(); }
        if p.eat(&TokenKind::Eq) {
            parse_const_expr(p);
        }
        if !p.eat(&TokenKind::Comma) { break; }
    }
    p.expect(&TokenKind::RBrace, "'}'");
}

fn parse_const_expr(p: &mut Parser) {
    // Simple: consume until comma, brace, or semicolon
    while !p.at_eof() {
        match p.peek_kind() {
            TokenKind::Comma | TokenKind::RBrace | TokenKind::Semi => break,
            _ => { p.advance(); }
        }
    }
}

/// A.B.C or A::B::C (mixed, Java vs C++)
fn parse_qualified_name(p: &mut Parser) {
    if p.is_ident() { p.advance(); }
    loop {
        if p.check(&TokenKind::DoubleColon) || p.check(&TokenKind::Dot) {
            p.advance();
            if p.is_ident() { p.advance(); }
        } else {
            break;
        }
    }
}

fn parse_init_declarator_list(p: &mut Parser, lang: Language) {
    parse_init_declarator(p, lang);
    while p.eat(&TokenKind::Comma) {
        parse_init_declarator(p, lang);
    }
}

fn parse_init_declarator(p: &mut Parser, lang: Language) {
    parse_declarator(p, lang);
    match p.peek_kind() {
        TokenKind::LBrace => {
            // function definition body
            parse_compound_body(p, lang);
        }
        TokenKind::Colon => {
            // bit-field or constructor init list
            p.advance();
            parse_expr(p, lang);
            if p.check(&TokenKind::LBrace) { parse_compound_body(p, lang); }
            // Try to end the declaration
            p.eat(&TokenKind::Semi);
        }
        TokenKind::Eq => {
            p.advance(); // '='
            if p.check(&TokenKind::LBrace) {
                parse_brace_initializer(p, lang);
            } else {
                parse_assign_expr(p, lang);
            }
            p.eat(&TokenKind::Semi);
        }
        TokenKind::Semi => { p.advance(); }
        _ => {
            // May be inline definition without body (prototype) – just require ';'
            p.eat(&TokenKind::Semi);
        }
    }
}

fn parse_brace_initializer(p: &mut Parser, lang: Language) {
    p.advance(); // '{'
    while !p.at_eof() && !p.check(&TokenKind::RBrace) {
        if p.check(&TokenKind::LBrace) {
            parse_brace_initializer(p, lang);
        } else {
            parse_assign_expr(p, lang);
        }
        if !p.eat(&TokenKind::Comma) { break; }
    }
    p.expect(&TokenKind::RBrace, "'}'");
}

/// Parse a declarator: optional pointer chain, then direct declarator.
fn parse_declarator(p: &mut Parser, lang: Language) {
    // Pointer / reference chain
    loop {
        match p.peek_kind() {
            TokenKind::Star | TokenKind::Amp => { p.advance(); }
            TokenKind::AmpAmp if lang == Language::Cpp => { p.advance(); } // rvalue-ref
            _ => break,
        }
        // optional cv-quals
        while matches!(p.peek_kind(), TokenKind::KwConst | TokenKind::KwVolatile) { p.advance(); }
    }
    parse_direct_declarator(p, lang);
}

fn parse_direct_declarator(p: &mut Parser, lang: Language) {
    // Grouped declarator: ( declarator )
    // Distinguish from function-style constructor call:
    // If '(' is followed by a type or just ')', treat as param list directly.
    // If '(' is followed by '*' or '&', it's a grouped declarator like (*fp).
    if p.check(&TokenKind::LParen) {
        let is_grouped = matches!(p.peek2_kind(), TokenKind::Star | TokenKind::Amp | TokenKind::AmpAmp);
        if is_grouped {
            p.advance(); // '('
            parse_declarator(p, lang);
            p.expect(&TokenKind::RParen, "')'");
        }
        // else fall through to suffix loop which handles '(' as param list
    } else if p.is_ident() {
        p.advance();
        // Qualified  Class::method
        while p.check(&TokenKind::DoubleColon) {
            p.advance();
            if p.peek_kind() == &TokenKind::Tilde { p.advance(); } // destructor
            if p.is_ident() { p.advance(); }
        }
    } else if lang == Language::Cpp && p.check(&TokenKind::KwOperator) {
        // operator overload name: operator+, operator[], operator==, etc.
        p.advance(); // 'operator'
        parse_operator_overload_type(p);
    } else if lang == Language::Cpp && p.check(&TokenKind::Tilde) {
        // destructor at declaration level
        p.advance(); // '~'
        if p.is_ident() { p.advance(); }
    }
    // Suffixes: [] and ()
    loop {
        match p.peek_kind() {
            TokenKind::LBracket => {
                p.advance();
                if !p.check(&TokenKind::RBracket) { parse_assign_expr(p, lang); }
                p.expect(&TokenKind::RBracket, "']'");
            }
            TokenKind::LParen => {
                // Decide: function parameter list (types) or constructor call (args)?
                // Heuristic: if content is a type-start or empty → param list
                //             if content is expr (IntLit, etc.) → constructor call args
                let looks_like_params = {
                    let inner = p.peek2_kind().clone();
                    inner.is_type_start()
                        || matches!(inner, TokenKind::RParen | TokenKind::Ellipsis | TokenKind::KwVoid)
                };
                if looks_like_params {
                    parse_param_list(p, lang);
                    // optional cv-qualifiers, noexcept, override, final, pure-virtual '= 0'
                    while matches!(p.peek_kind(),
                        TokenKind::KwConst | TokenKind::KwVolatile | TokenKind::KwOverride
                        | TokenKind::KwFinal) {
                        p.advance();
                    }
                    // noexcept (treated as an identifier in older parsers)
                    if matches!(p.peek_kind(), TokenKind::Ident(_)) {
                        if let TokenKind::Ident(ref s) = p.peek_kind().clone() {
                            if s == "noexcept" || s == "__attribute__" {
                                p.advance();
                                if p.check(&TokenKind::LParen) { parse_call_args(p, lang); }
                            }
                        }
                    }
                    // pure virtual: = 0  OR  = default  OR  = delete
                    if p.check(&TokenKind::Eq) {
                        p.advance();
                        p.advance(); // consume 0 / default / delete
                    }
                    // trailing return type  -> Type
                    if p.check(&TokenKind::Arrow) {
                        p.advance();
                        parse_type_name(p, lang);
                    }
                } else {
                    // constructor/uniform init call: consume as call args
                    parse_call_args(p, lang);
                }
                break;
            }
            _ => break,
        }
    }
    // Constructor initializer list  :  Base(…), mem(…), …  (handled in parse_init_declarator)
}

fn parse_param_list(p: &mut Parser, lang: Language) {
    p.advance(); // '('
    if p.check(&TokenKind::RParen) { p.advance(); return; }
    loop {
        if p.eat(&TokenKind::Ellipsis) { break; }
        parse_type_name(p, lang);
        // optional declarator
        if !p.check(&TokenKind::Comma) && !p.check(&TokenKind::RParen) {
            parse_declarator(p, lang);
        }
        // optional default
        if p.eat(&TokenKind::Eq) {
            parse_assign_expr(p, lang);
        }
        if !p.eat(&TokenKind::Comma) { break; }
        if p.check(&TokenKind::RParen) { break; }
    }
    p.expect(&TokenKind::RParen, "')'");
}

// ── Class body (C++) ──────────────────────────────────────────────────────────

fn parse_class_body(p: &mut Parser, lang: Language) {
    p.advance(); // '{'
    while !p.at_eof() && !p.check(&TokenKind::RBrace) {
        if p.should_abort() { break; }
        // Access specifier label
        if matches!(p.peek_kind(), TokenKind::KwPublic | TokenKind::KwProtected | TokenKind::KwPrivate) {
            p.advance();
            if p.eat(&TokenKind::Colon) { continue; }
        }
        if p.eat(&TokenKind::Semi) { continue; }
        let pos_before = p.pos;
        parse_declaration_or_function(p, lang);
        if p.pos == pos_before && !p.at_eof() && !p.check(&TokenKind::RBrace) { p.advance(); }
    }
    p.expect(&TokenKind::RBrace, "'}'");
    p.eat(&TokenKind::Semi);
}

// ── Statements ────────────────────────────────────────────────────────────────

fn parse_compound_body(p: &mut Parser, lang: Language) {
    p.advance(); // '{'
    while !p.at_eof() && !p.check(&TokenKind::RBrace) {
        if p.should_abort() { break; }
        let pos_before = p.pos;
        parse_statement(p, lang);
        // Safety: if no progress was made, force advance to avoid infinite loop
        if p.pos == pos_before && !p.at_eof() && !p.check(&TokenKind::RBrace) {
            p.advance();
        }
    }
    p.expect(&TokenKind::RBrace, "'}'");
}

pub fn parse_statement(p: &mut Parser, lang: Language) {
    if p.eat(&TokenKind::Semi) { return; } // empty statement

    match p.peek_kind().clone() {
        TokenKind::LBrace => parse_compound_body(p, lang),
        TokenKind::PreprocLine(_) => { p.advance(); }

        TokenKind::KwIf => parse_if(p, lang),
        TokenKind::KwWhile => parse_while(p, lang),
        TokenKind::KwFor => parse_for(p, lang),
        TokenKind::KwDo => parse_do_while(p, lang),
        TokenKind::KwSwitch => parse_switch(p, lang),
        TokenKind::KwReturn => parse_return(p, lang),
        TokenKind::KwBreak => { p.advance(); p.expect(&TokenKind::Semi, "';'"); }
        TokenKind::KwContinue => { p.advance(); p.expect(&TokenKind::Semi, "';'"); }
        TokenKind::KwGoto => { p.advance(); p.expect_ident(); p.expect(&TokenKind::Semi, "';'"); }
        TokenKind::KwTry => parse_try(p, lang),
        TokenKind::KwThrow => {
            p.advance();
            if !p.check(&TokenKind::Semi) { parse_assign_expr(p, lang); }
            p.expect(&TokenKind::Semi, "';'");
        }
        // labeled statement: ident ':'  (or case / default)
        TokenKind::KwCase => {
            p.advance();
            parse_expr(p, lang);
            p.expect(&TokenKind::Colon, "':'");
        }
        TokenKind::KwDefault => {
            p.advance();
            p.expect(&TokenKind::Colon, "':'");
        }
        // Java assert statement
        TokenKind::KwAssert => {
            p.advance();
            parse_expr(p, lang);
            if p.eat(&TokenKind::Colon) { parse_expr(p, lang); }
            p.expect(&TokenKind::Semi, "';'");
        }
        TokenKind::Ident(_) if matches!(p.peek2_kind(), TokenKind::Colon) => {
            p.advance(); // label name
            p.advance(); // ':'
        }
        _ => {
            // A declaration or expression statement
            if is_decl_start(p) || qualified_type_decl_heuristic(p) {
                parse_declaration_or_function(p, lang);
            } else {
                parse_expr(p, lang);
                p.expect(&TokenKind::Semi, "';'");
            }
        }
    }
}

fn is_decl_start(p: &Parser) -> bool {
    // Keyword type-specifier or storage class → definitely a declaration
    if !matches!(p.peek_kind(), TokenKind::Ident(_)) {
        return matches!(
            p.peek_kind(),
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
                | TokenKind::KwAuto
                | TokenKind::KwRegister
                | TokenKind::KwTypedef
                | TokenKind::KwStruct
                | TokenKind::KwUnion
                | TokenKind::KwEnum
                | TokenKind::KwClass
        );
    }
    // Ident … → heuristic: check what follows
    // "TypeName varName"  (Ident Ident) or "TypeName *varName" (Ident Star)
    // or "TypeName &ref"  (Ident Amp)  → declaration
    // "funcCall(...)"     (Ident LParen) or "x = ..."  → expression
    matches!(
        p.peek2_kind(),
        TokenKind::Ident(_) | TokenKind::Star | TokenKind::Amp | TokenKind::AmpAmp
    )
}

/// Heuristic: current token is Ident and peek2 is `::` — scan forward over the
/// qualified name (Ident :: Ident :: ...) and optional <...> template args,
/// then check if the *next* token looks like a declarator name or ptr (`*`, `&`).
/// This handles:  `geometry::Vector2D<int> v1(1,2);`  →  declaration
/// But NOT:       `std::cout << "hello";`              →  expression  (`<<` is not a decl-start)
fn qualified_type_decl_heuristic(p: &Parser) -> bool {
    if !matches!(p.peek_kind(), TokenKind::Ident(_)) { return false; }
    if !matches!(p.peek2_kind(), TokenKind::DoubleColon) { return false; }

    // Walk forward through the token stream speculatively
    let tokens = p.tokens();
    let mut i = p.pos(); // current position (the first Ident)

    // consume   (Ident ::)+  Ident
    loop {
        // expect Ident
        match tokens.get(i).map(|t| &t.kind) {
            Some(TokenKind::Ident(_)) => i += 1,
            _ => return false,
        }
        // next: :: or end-of-qualified-name
        match tokens.get(i).map(|t| &t.kind) {
            Some(TokenKind::DoubleColon) => i += 1, // keep going
            _ => break, // end of qualified name
        }
    }

    // optional template args:  < ... >
    if matches!(tokens.get(i).map(|t| &t.kind), Some(TokenKind::Lt)) {
        // skip until matching >  (track nesting)
        i += 1;
        let mut depth = 1usize;
        while i < tokens.len() && depth > 0 {
            match &tokens[i].kind {
                TokenKind::Lt => { depth += 1; i += 1; }
                TokenKind::Gt | TokenKind::GtGt => { depth -= 1; i += 1; }
                TokenKind::Semi | TokenKind::LBrace => return false, // something wrong
                _ => i += 1,
            }
        }
    }

    // optional pointer/reference qualifiers
    while matches!(
        tokens.get(i).map(|t| &t.kind),
        Some(TokenKind::Star) | Some(TokenKind::Amp) | Some(TokenKind::AmpAmp)
            | Some(TokenKind::KwConst)
    ) {
        i += 1;
    }

    // Now: if the next token is an Ident → declarator name → it's a declaration
    matches!(tokens.get(i).map(|t| &t.kind), Some(TokenKind::Ident(_)))
}

fn parse_if(p: &mut Parser, lang: Language) {
    p.advance(); // 'if'
    p.expect(&TokenKind::LParen, "'('");
    parse_expr(p, lang);
    p.expect(&TokenKind::RParen, "')'");
    parse_statement(p, lang);
    if matches!(p.peek_kind(), TokenKind::KwElse) {
        p.advance();
        parse_statement(p, lang);
    }
}

fn parse_while(p: &mut Parser, lang: Language) {
    p.advance(); // 'while'
    p.expect(&TokenKind::LParen, "'('");
    parse_expr(p, lang);
    p.expect(&TokenKind::RParen, "')'");
    parse_statement(p, lang);
}

fn parse_for(p: &mut Parser, lang: Language) {
    p.advance(); // 'for'
    p.expect(&TokenKind::LParen, "'('");
    // init
    if !p.check(&TokenKind::Semi) {
        if is_decl_start(p) {
            parse_decl_specifiers(p, lang);
            parse_declarator(p, lang);
            // for-each colon (Java / C++ range-for)
            if p.eat(&TokenKind::Colon) {
                parse_expr(p, lang);
                p.expect(&TokenKind::RParen, "')'");
                parse_statement(p, lang);
                return;
            }
            if p.eat(&TokenKind::Eq) {
                parse_assign_expr(p, lang);
            }
            while p.eat(&TokenKind::Comma) {
                parse_init_declarator(p, lang);
            }
        } else {
            parse_expr(p, lang);
        }
    }
    p.expect(&TokenKind::Semi, "';'");
    // condition
    if !p.check(&TokenKind::Semi) { parse_expr(p, lang); }
    p.expect(&TokenKind::Semi, "';'");
    // increment
    if !p.check(&TokenKind::RParen) { parse_expr(p, lang); }
    p.expect(&TokenKind::RParen, "')'");
    parse_statement(p, lang);
}


fn parse_do_while(p: &mut Parser, lang: Language) {
    p.advance(); // 'do'
    parse_statement(p, lang);
    p.expect(&TokenKind::KwWhile, "'while'");
    p.expect(&TokenKind::LParen, "'('");
    parse_expr(p, lang);
    p.expect(&TokenKind::RParen, "')'");
    p.expect(&TokenKind::Semi, "';'");
}

fn parse_switch(p: &mut Parser, lang: Language) {
    p.advance(); // 'switch'
    p.expect(&TokenKind::LParen, "'('");
    parse_expr(p, lang);
    p.expect(&TokenKind::RParen, "')'");
    parse_statement(p, lang);
}

fn parse_return(p: &mut Parser, lang: Language) {
    p.advance(); // 'return'
    if !p.check(&TokenKind::Semi) {
        if p.check(&TokenKind::LBrace) {
            parse_brace_initializer(p, lang);
        } else {
            parse_expr(p, lang);
        }
    }
    p.expect(&TokenKind::Semi, "';'");
}

fn parse_try(p: &mut Parser, lang: Language) {
    p.advance(); // 'try'
    parse_compound_body(p, lang);
    while matches!(p.peek_kind(), TokenKind::KwCatch) {
        p.advance(); // 'catch'
        p.expect(&TokenKind::LParen, "'('");
        if !p.eat(&TokenKind::Ellipsis) {
            parse_type_name(p, lang);
            if p.is_ident() { p.advance(); }
        }
        p.expect(&TokenKind::RParen, "')'");
        parse_compound_body(p, lang);
    }
    // finally (Java)
    if matches!(p.peek_kind(), TokenKind::KwFinally) {
        p.advance();
        parse_compound_body(p, lang);
    }
}

// ── Expressions (Pratt / precedence-climbing) ─────────────────────────────────

/// Top-level expression: comma-separated.
pub fn parse_expr(p: &mut Parser, lang: Language) {
    parse_assign_expr(p, lang);
    while p.eat(&TokenKind::Comma) {
        parse_assign_expr(p, lang);
    }
}

fn parse_assign_expr(p: &mut Parser, lang: Language) {
    parse_ternary_expr(p, lang);
    match p.peek_kind() {
        TokenKind::Eq
        | TokenKind::PlusEq
        | TokenKind::MinusEq
        | TokenKind::StarEq
        | TokenKind::SlashEq
        | TokenKind::PercentEq
        | TokenKind::AmpEq
        | TokenKind::PipeEq
        | TokenKind::CaretEq
        | TokenKind::LtLtEq
        | TokenKind::GtGtEq
        | TokenKind::GtGtGtEq => {
            p.advance();
            parse_assign_expr(p, lang);
        }
        _ => {}
    }
}

fn parse_ternary_expr(p: &mut Parser, lang: Language) {
    parse_or_expr(p, lang);
    if p.eat(&TokenKind::Question) {
        parse_expr(p, lang);
        p.expect(&TokenKind::Colon, "':'");
        parse_assign_expr(p, lang);
    }
}

macro_rules! left_assoc {
    ($fn_name:ident, $next:ident, $($op:pat),+) => {
        fn $fn_name(p: &mut Parser, lang: Language) {
            $next(p, lang);
            while matches!(p.peek_kind(), $($op)|+) {
                p.advance();
                $next(p, lang);
            }
        }
    };
}

left_assoc!(parse_or_expr, parse_and_expr, TokenKind::PipePipe);
left_assoc!(parse_and_expr, parse_bit_or_expr, TokenKind::AmpAmp);
left_assoc!(parse_bit_or_expr, parse_bit_xor_expr, TokenKind::Pipe);
left_assoc!(parse_bit_xor_expr, parse_bit_and_expr, TokenKind::Caret);
left_assoc!(parse_bit_and_expr, parse_eq_expr, TokenKind::Amp);
left_assoc!(parse_eq_expr, parse_rel_expr, TokenKind::EqEq, TokenKind::BangEq);
left_assoc!(parse_rel_expr, parse_shift_expr,
    TokenKind::Lt, TokenKind::Gt, TokenKind::LtEq, TokenKind::GtEq,
    TokenKind::KwInstanceof);
left_assoc!(parse_shift_expr, parse_add_expr,
    TokenKind::LtLt, TokenKind::GtGt, TokenKind::GtGtGt);
left_assoc!(parse_add_expr, parse_mul_expr, TokenKind::Plus, TokenKind::Minus);
left_assoc!(parse_mul_expr, parse_cast_expr,
    TokenKind::Star, TokenKind::Slash, TokenKind::Percent);

fn parse_cast_expr(p: &mut Parser, lang: Language) {
    // Heuristic: if we see '(' followed by a type keyword, treat as cast
    if p.check(&TokenKind::LParen) {
        let pos_save = p.pos;
        let diag_save = p.diags.items.len();
        p.advance(); // '('
        if p.peek_kind().is_type_start() {
            parse_type_name(p, lang);
            if p.check(&TokenKind::RParen) {
                p.advance(); // ')'
                parse_cast_expr(p, lang);
                return;
            }
        }
        // Restore and fall through
        p.pos = pos_save;
        p.diags.items.truncate(diag_save);
    }
    parse_unary_expr(p, lang);
}

fn parse_unary_expr(p: &mut Parser, lang: Language) {
    match p.peek_kind() {
        TokenKind::PlusPlus
        | TokenKind::MinusMinus
        | TokenKind::Amp
        | TokenKind::Star
        | TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Tilde
        | TokenKind::Bang => {
            p.advance();
            parse_cast_expr(p, lang);
        }
        TokenKind::KwSizeof => {
            p.advance();
            if p.eat(&TokenKind::LParen) {
                parse_type_name(p, lang);
                p.expect(&TokenKind::RParen, "')'");
            } else {
                parse_unary_expr(p, lang);
            }
        }
        TokenKind::KwNew => {
            p.advance();
            // optional placement new
            if p.check(&TokenKind::LParen) { parse_call_args(p, lang); }
            parse_type_name(p, lang);
            // array new: new int[3][3] or new Type[][]
            while p.check(&TokenKind::LBracket) {
                p.advance();
                if !p.check(&TokenKind::RBracket) { parse_assign_expr(p, lang); }
                p.expect(&TokenKind::RBracket, "']'");
            }
            // initialiser
            if p.check(&TokenKind::LParen) { parse_call_args(p, lang); }
            else if p.check(&TokenKind::LBrace) { parse_brace_initializer(p, lang); }
        }
        TokenKind::KwDelete => {
            p.advance();
            p.eat(&TokenKind::LBracket);
            p.eat(&TokenKind::RBracket);
            parse_cast_expr(p, lang);
        }
        _ => parse_postfix_expr(p, lang),
    }
}

fn parse_postfix_expr(p: &mut Parser, lang: Language) {
    parse_primary_expr(p, lang);
    loop {
        match p.peek_kind() {
            TokenKind::LBracket => {
                p.advance();
                parse_expr(p, lang);
                p.expect(&TokenKind::RBracket, "']'");
            }
            TokenKind::LParen => {
                parse_call_args(p, lang);
            }
            TokenKind::Dot | TokenKind::Arrow => {
                p.advance();
                if matches!(p.peek_kind(), TokenKind::Tilde) { p.advance(); }
                if p.is_ident() { p.advance(); }
                // Template method call e.g. ptr->template method<T>()
                if p.check(&TokenKind::KwTemplate) { p.advance(); }
                if p.check(&TokenKind::Lt) {
                    p.advance();
                    skip_angle_brackets_content(p);
                }
            }
            TokenKind::PlusPlus | TokenKind::MinusMinus => { p.advance(); }
            _ => break,
        }
    }
}

fn parse_call_args(p: &mut Parser, lang: Language) {
    p.advance(); // '('
    if p.check(&TokenKind::RParen) { p.advance(); return; }
    loop {
        parse_assign_expr(p, lang);
        if !p.eat(&TokenKind::Comma) { break; }
        if p.check(&TokenKind::RParen) { break; }
    }
    p.expect(&TokenKind::RParen, "')'");
}

fn parse_primary_expr(p: &mut Parser, lang: Language) {
    match p.peek_kind().clone() {
        TokenKind::IntLit(_)
        | TokenKind::FloatLit(_)
        | TokenKind::CharLit(_)
        | TokenKind::StringLit(_)
        | TokenKind::BoolLit(_)
        | TokenKind::NullLit => { p.advance(); }
        TokenKind::Ident(_) => {
            p.advance();
            // Qualified name  ::  .  ->
            while p.check(&TokenKind::DoubleColon) {
                p.advance();
                if p.is_ident() { p.advance(); }
            }
            // Template args: ONLY attempt in C++ AND only when the `<` is
            // followed by a type-like token (keyword or ident), to avoid
            // consuming comparison operators like `i < 3`.
            if lang == Language::Cpp && p.check(&TokenKind::Lt) {
                let peek_after = p.peek2_kind().clone();
                let looks_like_type = peek_after.is_type_start()
                    || matches!(peek_after, TokenKind::Star | TokenKind::Amp | TokenKind::Gt);
                if looks_like_type {
                    let save = p.pos;
                    let dsave = p.diags.items.len();
                    p.advance(); // '<'
                    skip_angle_brackets_content(p);
                    // Sanity-check: should be followed by pointer/ref/ident/paren/semi
                    let ok = p.check(&TokenKind::LParen)
                        || p.check(&TokenKind::LBrace)
                        || p.check(&TokenKind::Semi)
                        || p.check(&TokenKind::Comma)
                        || p.check(&TokenKind::RParen)
                        || p.check(&TokenKind::Eof);
                    if !ok {
                        p.pos = save;
                        p.diags.items.truncate(dsave);
                    }
                }
            }
        }
        TokenKind::KwThis | TokenKind::KwSuper => { p.advance(); }
        TokenKind::LParen => {
            p.advance();
            // lambda or grouped expr; handle lambda init list
            parse_expr(p, lang);
            p.expect(&TokenKind::RParen, "')'");
        }
        // C++ lambda
        TokenKind::LBracket if lang == Language::Cpp => {
            p.advance(); // '['
            while !p.at_eof() && !p.check(&TokenKind::RBracket) { p.advance(); }
            p.expect(&TokenKind::RBracket, "']'");
            // params
            if p.check(&TokenKind::LParen) { parse_param_list(p, lang); }
            // trailing return
            if p.check(&TokenKind::Arrow) { p.advance(); parse_type_name(p, lang); }
            // body
            if p.check(&TokenKind::LBrace) { parse_compound_body(p, lang); }
        }
        _ => {
            // Don't emit an error here; many callers recover
        }
    }
}
