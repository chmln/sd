pub(crate) type Result<T> = std::result::Result<T, crate::Error>;

pub(crate) fn regex_case_sensitive(pattern: &str) -> bool {
    use regex_syntax::Parser;
    Parser::new()
        .parse(pattern)
        .map(|hir| hir_has_uppercase_char(&hir))
        .unwrap_or_else(|_| false)
}

// Check if regex HIR has any uppercase chars
// snippet borrowed from @sharkdp
fn hir_has_uppercase_char(hir: &regex_syntax::hir::Hir) -> bool {
    use regex_syntax::hir::*;

    match hir.kind() {
        HirKind::Literal(Literal::Unicode(c)) => c.is_uppercase(),
        HirKind::Class(Class::Unicode(ref ranges)) => ranges
            .iter()
            .any(|r| r.start().is_uppercase() || r.end().is_uppercase()),
        HirKind::Group(Group { ref hir, .. })
        | HirKind::Repetition(Repetition { ref hir, .. }) => {
            hir_has_uppercase_char(hir)
        },
        HirKind::Concat(ref hirs) | HirKind::Alternation(ref hirs) => {
            hirs.iter().any(hir_has_uppercase_char)
        },
        _ => false,
    }
}
