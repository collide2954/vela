# Vela

A programming language for data science, statistics, and analysis.

Vela is built around three commitments: correctness, reproducibility, and
ergonomic safety. It draws its type discipline from the ML family
(OCaml, F#, Haskell) and its tooling discipline from Rust, and it presents
that discipline through a surface syntax that a working analyst can read
and write without prior training in type theory.

The full specification lives in [SPEC.md](SPEC.md).

## Status

Pre-alpha. The lexer and parser are largely complete; the type checker,
runtime, and standard library are in progress. Nothing here is stable
yet.

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

A working Rust toolchain (1.85 or later) is required.

## License

Apache-2.0. See [LICENSE](LICENSE).
