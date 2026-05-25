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

A function with annotated arguments and a return type:

```vela
let standardize (xs : [Float]) : [Float] =
    let m = mean xs
    let s = std xs
    xs |> map (fn x -> (x - m) / s)
```

Algebraic data types, pattern matching, and constructors as values:

```vela
type Shape =
    | Circle Float
    | Square Float
    | Rect   { width : Float, height : Float }

let area shape =
    match shape with
    | Circle r                          -> 3.14159 * r * r
    | Square s | Rect { width = s, height = s } -> s * s
    | Rect { width = w, height = h }    -> w * h
```

Errors as values, propagated with `?`:

```vela
let load path =
    let raw = read_file path?
    let df  = parse_csv raw?
    Ok df
```

DataFrames are a first-class language construct, not a library:

```vela
import std.data

let df = data.iris
df
|> group_by :species
|> summarize { mu = mean (col :petal_length) }
|> plot (aes { x = :species, y = :mu })
    ++ bar ()
```

A reactive app shares a one-page analysis without any frontend code:

```vela
let dashboard = app =
    input n        = slider { min = 1, max = 1000, default = 100 }
    input dataset  = file_picker { accept = [".csv"] }

    let df         = read_csv dataset?
    let sample     = df |> head n

    output table   = sample
    output hist    = plot sample (aes { x = :x }) ++ hist ()
    output summary = format "rows = {}, cols = {}" sample.rows sample.cols
```

Test cases and property tests live alongside the code they cover:

```vela
pub let mean xs = sum xs / Float.of_int (length xs)

tests =
    test "mean of [1, 2, 3] is 2" =
        assert (mean [1.0, 2.0, 3.0] == Ok 2.0)

    prop "mean is between min and max" (xs : [Float]) when length xs > 0 =
        let m = mean xs |> Result.unwrap
        min xs <= m and m <= max xs
```

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
