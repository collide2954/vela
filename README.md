# Vela

A programming language for data science, statistics, and analysis.

Vela is built around three commitments: correctness, reproducibility, and
ergonomic safety. It draws its type discipline from the ML family
(OCaml, F#, Haskell) and its tooling discipline from Rust, and it presents
that discipline through a surface syntax that a working analyst can read
and write without prior training in type theory.

The full language specification lives in [SPEC.md](SPEC.md).

## Status

Pre-alpha. The lexer, parser, and a substantial type checker (Hindley-
Milner with let polymorphism, user types, Option/Result, row-polymorphic
records, pattern matching with guards) are in place. The runtime,
standard library, formatter, and CLI subcommands are still to come.
Nothing here is stable.

## A taste

    let standardize (xs : [Float]) : [Float] =
        let m = mean xs
        let s = std xs
        xs |> map (fn x -> (x - m) / s)

    type Shape =
        | Circle Float
        | Square Float

    let area shape =
        match shape with
        | Circle r -> 3.14159 * r * r
        | Square s -> s * s

## Layout

    crates/
        vela-cli/      # the `vela` binary
        vela-lexer/    # lexical analysis
        vela-parser/   # syntactic analysis
        vela-check/    # type checking and inference

The standard library will live under `std/` once the bootstrap compiler
can run it.

## Building

    cargo build
    cargo test

Requires Rust 1.95 or later. There are no other dependencies.

## Contributing

The project is too young to take outside contributions usefully. If you
have ideas, open an issue.

## License

Apache-2.0. See [LICENSE](LICENSE).
