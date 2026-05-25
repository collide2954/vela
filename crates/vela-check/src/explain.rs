pub fn explain(code: &str) -> Option<&'static str> {
    match code {
        "E0001" => Some(
            "Parse error.\n\nThe source could not be parsed. The accompanying message\nidentifies the unexpected token or missing form. Common causes:\nstray punctuation, missing `=` or `->`, or a body that does not\nstart with a valid expression.",
        ),
        "E0100" => Some(
            "Type error.\n\nThe program is well-formed syntactically but its types do not\nfit together. The accompanying message names the two types that\nfailed to unify.",
        ),
        "E0110" => Some(
            "Unbound name.\n\nA name was used that is not defined in the current scope and is\nnot exported by any imported module. Check spelling, that the\ndefinition appears before its use at top level, and that any\nrequired `import` is present.",
        ),
        "E0130" => Some(
            "Non-exhaustive match.\n\nThe match expression does not cover every possible value of the\nscrutinee type. Add the missing arms or a wildcard `_ -> ...`.",
        ),
        _ => None,
    }
}
