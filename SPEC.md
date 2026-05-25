# The Vela Language Specification

Status: draft, version 0.1.
Target: Vela 1.0.

Vela is a programming language for data science, statistics, and analysis.
It is built around three commitments: correctness, reproducibility, and
ergonomic safety. It draws its type discipline from the ML family
(OCaml, F#, Haskell) and its tooling discipline from Rust, and it presents
that discipline through a surface syntax that a working analyst can read
and write without prior training in type theory.

This document describes the language and runtime to be implemented. It is
intentionally complete enough to drive implementation and intentionally
narrow enough to permit only one obvious way to express most ideas.

## 1. Goals

1. A statistical language that is correct by default. Programs that
   compile produce the same results on every supported platform from the
   same inputs.
2. A surface syntax that a Python or R user can read on first contact.
3. One opinionated way to do common things: one formatter, one test
   harness, one package layout, one plotting grammar, one project
   manifest.
4. A single binary, `vela`, that ships every tool a working analyst needs.
5. A standard library written in Vela itself, with the bootstrap
   compiler, runtime, and core data structures written in Rust.

## 2. Non-goals

1. Vela is not a systems language. It does not expose manual memory
   management, raw pointers, or unsafe casts.
2. Vela is not a general-purpose application platform. It is tuned for
   numerical, tabular, and statistical work. Web servers, GUIs, and
   game engines are out of scope for the standard library.
3. Vela does not attempt to be backwards-compatible with R, Python,
   Julia, MATLAB, or Octave at the language level. Interop with their
   ecosystems is provided through data formats and the C ABI, not
   language emulation.

## 3. Design principles

1. Reproducibility is a language feature, not a discipline. The
   compiler, runtime, and standard library together guarantee that
   identical source plus identical lockfile plus identical inputs
   produces identical outputs, bit for bit.
2. Inference over annotation. Type signatures are optional almost
   everywhere. Public APIs in libraries are annotated by convention and
   enforced by `vela check`.
3. Errors are values. Vela has no exceptions, no panics in user code,
   and no silently propagating `NA`.
4. Immutability is the default. Mutation is explicit and local.
5. Parallelism is implicit and deterministic. The compiler is
   permitted to parallelize collection operations only when doing so
   yields bit-identical results to a serial execution.
6. Data is first-class. `DataFrame`, `Series`, and `Array` are language
   constructs with literal syntax, not library types bolted onto a
   general-purpose core.
7. The standard library is the language. Most user-visible features
   are defined in `.vela` files and are readable by users.

## 4. Lexical structure

Source files are UTF-8 encoded and have the extension `.vela`. Line
endings are normalized to `\n` by the lexer.

Indentation is significant. Blocks are introduced by a header line
ending in `=` (for definitions) or `:` (for control flow) and consist
of the lines indented further than the header. Mixed tabs and spaces
in the same file are a lexical error. The canonical indent, enforced
by `vela fmt`, is four spaces.

Comments begin with `#` and run to the end of the line. There are no
block comments.

Identifiers begin with a Unicode letter or underscore and continue
with letters, digits, or underscore. Identifiers are case-sensitive.
By convention values and functions are `snake_case`, types and
constructors are `UpperCamelCase`.

Numeric literals:

    42          # Int (i64)
    3.14        # Float (f64)
    1_000_000   # underscores as digit separators
    0xff, 0b10  # hex and binary integer literals
    1e-3        # scientific Float
    NaN, Inf    # named Float constants

String literals are double-quoted and support the usual escapes:

    "hello"
    "line\nbreak"

Symbol literals begin with a colon and name a column or key:

    :species

Boolean literals are `true` and `false`. The unit value is written
`()`.

Reserved words: `let`, `var`, `fn`, `if`, `else`, `match`, `with`,
`type`, `trait`, `impl`, `for`, `in`, `return`, `pub`, `module`,
`import`, `as`, `where`, `scope`, `spawn`, `true`, `false`, `and`,
`or`, `not`.

## 5. Syntax

Vela is expression-oriented. Every construct produces a value; blocks
return the value of their last expression.

### 5.1 Bindings

    let x = 1                  # immutable binding
    var counter = 0            # mutable binding
    counter <- counter + 1     # mutation uses left-arrow

`let` introduces an immutable binding. `var` introduces a mutable
binding. The `<-` operator is the only way to mutate a `var`. Plain
`=` is binding, never assignment.

### 5.2 Functions

    let standardize xs =
        let m = mean xs
        let s = std xs
        xs |> map (fn x -> (x - m) / s)

`let` doubles as the function-definition form when followed by
parameters. Anonymous functions use `fn ... -> ...`. Function
application is space-separated; parentheses group expressions, they
do not introduce calls. Within an argument position, parentheses
disambiguate complex arguments.

Functions are curried. `let add x y = x + y` has type
`Int -> Int -> Int` and `add 1` is a function of one argument.

### 5.3 Type annotations

Annotations are optional and may appear on any binding:

    let standardize (xs : [Float]) : [Float] = ...
    let m : Float = mean xs

The compiler infers all unannotated types via Hindley-Milner with
extensions for multiple dispatch and row polymorphism on records.

### 5.4 Conditionals and pattern matching

    if x > 0 then x else -x

    match result with
    | Ok v  -> v
    | Err _ -> 0.0

`match` is exhaustive; non-exhaustive matches are a compile-time
error. The wildcard pattern is `_`. Patterns may bind variables,
destructure records and tuples, and guard with `when`:

    match shape with
    | Circle r when r > 0.0 -> area_circle r
    | Square s              -> s * s
    | _                     -> 0.0

### 5.5 Records and tuples

    let point = { x = 1.0, y = 2.0 }
    let p2    = { point with x = 3.0 }
    let pair  = (1, "a")
    let (a, _) = pair

Records are structurally typed with row polymorphism: a function that
reads `.x` accepts any record with an `x` field of the right type.

### 5.6 Pipelines

The pipe operator `|>` threads a value through a series of functions:

    df
    |> filter (col :x > 0)
    |> group_by :species
    |> summarize { mu = mean (col :petal_length) }
    |> plot (aes x: species, y: mu) + bar ()

`a |> f` is equivalent to `f a`. `|>` is left-associative and has
lower precedence than function application. Vela has exactly one pipe
operator; there are no variants.

### 5.7 Data literals

Series, DataFrame, and Array have literal syntax.

    let s = [1.0, 2.0, 3.0]                # Series[Float]

    let df = {|
        name : ["a", "b", "c"],
        x    : [1.0, 2.0, 3.0],
        y    : [10,  20,  30 ],
    |}                                     # DataFrame

    let m = [| 1.0, 2.0 ; 3.0, 4.0 |]      # Array[Float, 2]

DataFrame literals (`{| ... |}`) and Array literals (`[| ... |]`)
distinguish the constructors lexically so type inference is local.

### 5.8 Formulas

The `~` operator is built into the grammar and produces a value of
type `Formula`:

    let m = lm (y ~ x1 + x2 * x3, data = df)

A formula is a syntactic object, not a string. The compiler parses
it, the standard library interprets it. The full grammar of formulas
is given in section 11.4.

### 5.9 Modules and imports

A file is a module. The path of the file (relative to `src/`)
determines its module path. Modules export only items marked `pub`.

    pub let mean xs = sum xs / Float.of_int (length xs)

Imports name the module path and optionally a list of items:

    import stats
    import stats.dist (Normal, T)
    import frame as f

There is no `from ... import *`. Star imports are not part of the
language.

## 6. Type system

Vela has a sound, static type system with full inference. The type
language is:

    T ::= Int | Float | Bool | String | Symbol | ()
        | [T]                       # Series of T
        | Array[T, n]               # n-dimensional array
        | DataFrame                 # heterogeneous tabular
        | { l1 : T1, ..., ln : Tn } # record
        | (T1, ..., Tn)             # tuple
        | T1 -> T2                  # function
        | Option[T] | Result[T, E]
        | C[T1, ..., Tn]            # user-declared constructor
        | 'a                        # type variable

Type schemes generalize free variables at let-bindings, in the usual
HM style. Annotations are checked, not inferred-around.

### 6.1 Multiple dispatch

Functions may have multiple implementations specialized on argument
types. The most specific applicable implementation is chosen at the
call site by the compiler, statically when possible and at runtime
otherwise.

    fn add (a : Matrix, b : Matrix) : Matrix = ...
    fn add (a : Matrix, b : Float ) : Matrix = ...
    fn add (a : Float , b : Matrix) : Matrix = ...

Dispatch is closed within a module unless the function is declared
`open`. Open functions may be extended in downstream modules,
subject to the coherence rule: at most one most-specific
implementation must exist for any concrete call.

### 6.2 Traits

Traits group related operations and are dispatched the same way as
free functions. They are syntactic sugar for grouped multiple
dispatch.

    trait Show t =
        fn show (x : t) : String

    impl Show Float =
        fn show x = Float.to_string x

There is no inheritance. Trait constraints appear in `where`
clauses:

    let print_all xs where Show t = xs |> each (fn x -> println (show x))

### 6.3 Option and Result

Missing values are `Option[T]`. Recoverable errors are `Result[T,
E]`. There is no `null`, no `nil`, no `NA`. The `?` postfix operator
short-circuits a `Result`:

    let load path =
        let raw = read_file path?
        let df  = parse_csv raw?
        Ok df

`?` requires the enclosing function to return a compatible
`Result`.

### 6.4 No exceptions, no panics in user code

User code cannot raise. The runtime may abort on unrecoverable
internal conditions (out-of-memory, stack overflow), and the standard
library functions that assert invariants document those aborts.
Aborts terminate the process; they are not catchable.

## 7. Evaluation and execution

Vela source is compiled to a register-based bytecode and executed by
a virtual machine written in Rust. Hot bytecode functions are
identified by a sampling profiler and lowered to native code by a
just-in-time compiler. The JIT is part of the runtime and ships in
the same binary.

The bytecode format and the VM ABI are private to the implementation
and may change between minor versions. Source-level semantics are
stable.

Evaluation is strict (call-by-value). There is no laziness in the
language; iterators in the standard library expose lazy streaming
semantics where useful, but those are library constructs.

## 8. Memory

Vela values live on a managed heap. The runtime uses a generational,
compacting garbage collector with separate nurseries per VM thread
and a shared mature space. Collection is incremental; pause budgets
are configurable but bounded by default.

The collector preserves object identity only where it is
observable. DataFrames, Series, and Arrays are values: equality is
structural, sharing is a runtime detail. The compiler is free to
share storage between bindings that the type system proves
non-aliasing.

## 9. Concurrency and determinism

Vela has structured concurrency. Tasks are spawned inside a `scope`
block and the block does not return until every spawned task
completes:

    scope =
        spawn (load "a.csv")
        spawn (load "b.csv")

A scope's value is the tuple of its spawned tasks' results, in
spawn order. Failures inside a scope propagate to the scope's
result.

Collection operations on Series, DataFrame, and Array
auto-parallelize when the implementation can prove the operation
preserves bit-identical results. The proof obligation is on the
implementation; the user is given a single guarantee: a program's
output does not depend on the number of cores used to execute it.

Operations that cannot be made deterministic in parallel run
serially. There is no opt-out and no fast-but-non-deterministic
variant in the standard library.

## 10. Reproducibility

Reproducibility is the central commitment of Vela. The following
guarantees hold for any Vela program built and executed with a
matching toolchain version:

1. The same source, lockfile, and inputs produce byte-identical
   outputs on any supported platform (x86_64 and aarch64 Linux, macOS,
   and Windows).
2. All floating-point reductions use a fixed-tree associativity
   order, independent of thread count.
3. All randomness flows from a seeded, splittable PRNG. There is no
   implicit access to system entropy. A program that uses
   `Rand.global` without seeding it produces a deterministic error,
   not a non-deterministic result.
4. The build, run, and notebook tools record a manifest of input
   file hashes, RNG seeds, and toolchain version in
   `.vela/runs/<timestamp>.json`.

The lockfile `vela.lock` pins:

- The toolchain version (`vela` itself).
- The version and content hash of every direct and transitive
  Vela dependency.
- The hash of every `.vela` file in the dependency closure.
- The content hashes of any Rust crates pulled in by mixed Vela/Rust
  packages, by reference to `cargo`'s lockfile.

System libraries linked through `extern "C"` are not pinned by
`vela.lock` in version 1.0; such calls are explicitly marked
non-reproducible and a `vela check` lint warns when reproducible
builds depend on them.

## 11. Standard library

The standard library is written in Vela and lives in `std/`. It is
versioned with the compiler. The major sections are:

### 11.1 Core (`std.core`)

Primitive types and operations: `Int`, `Float`, `Bool`, `String`,
`Symbol`, `Option`, `Result`, `Ordering`, conversions, and the
prelude that is imported by default into every module.

### 11.2 Collections (`std.collection`)

`Series[T]`, `Array[T, n]`, immutable `Map`, immutable `Set`,
`Range`. Pipeline-friendly operations: `map`, `filter`, `fold`,
`group_by`, `sort_by`, `zip`, `chunk`, `window`.

### 11.3 DataFrame (`std.frame`)

`DataFrame`, columnar storage backed by typed buffers. Operations:
`select`, `filter`, `mutate`, `group_by`, `summarize`, `join`,
`pivot`, `unpivot`, `read_csv`, `write_csv`, `read_parquet`,
`write_parquet`. Columns are `Series[Option[T]]`; nullability is a
property of the column, not a sentinel value.

### 11.4 Formulas (`std.formula`)

The formula grammar:

    formula ::= term ~ term
    term    ::= name
              | term + term         # additive
              | term - term         # remove
              | term * term         # main effects + interaction
              | term : term         # interaction only
              | term / term         # nesting
              | I(expr)             # identity escape
              | 1 | 0               # intercept toggle

Formulas evaluate to a model matrix when given a `DataFrame`.

### 11.5 Statistics (`std.stats`)

Descriptive statistics, hypothesis tests, distributions, regression,
generalized linear models, mixed-effects models, survival analysis,
time series, Bayesian inference. Submodules:

- `std.stats.descr`
- `std.stats.test`
- `std.stats.dist`
- `std.stats.lm`
- `std.stats.glm`
- `std.stats.mixed`
- `std.stats.survival`
- `std.stats.ts`
- `std.stats.bayes`

Each submodule defines a single canonical API. Variants and
alternative parameterizations are not duplicated.

### 11.6 Optimization (`std.optim`)

Unconstrained and constrained optimizers, root finding, automatic
differentiation. Reverse-mode AD is the default; forward-mode is
exposed for small input dimensions.

### 11.7 Random (`std.rand`)

A splittable, seeded PRNG (counter-based, Philox-family).
`Rand.global` must be seeded explicitly or a program is in error.
Distributions in `std.stats.dist` consume `Rand` values.

### 11.8 IO (`std.io`)

File and stream IO, returning `Result`. No exceptions are raised. CSV,
TSV, JSON, JSONL, Parquet, and Arrow IPC are supported in 1.0.

### 11.9 Time (`std.time`)

`Instant`, `Duration`, `Date`, `DateTime`, `TimeZone`. All
operations on `DateTime` require an explicit time zone; there is no
"naive" datetime.

### 11.10 Plot (`std.plot`)

A grammar-of-graphics plotting system. The plot grammar:

    plot(data, aes(...)) + layer(...) + scale(...) + facet(...)

Layers: `point`, `line`, `bar`, `box`, `hist`, `density`, `smooth`,
`errorbar`, `ribbon`, `tile`, `text`. Scales: `scale_x_log`,
`scale_color_brewer`, and so on. Facets: `facet_wrap`, `facet_grid`.
Plots render to SVG, PNG, and the native notebook. Rendered output is
bit-deterministic given the same data and theme.

## 12. Tooling

The `vela` binary is the only tool. Subcommands:

    vela                # REPL
    vela run FILE       # compile and execute
    vela build          # build the current project
    vela test           # run tests
    vela fmt            # format
    vela check          # lint and type-check
    vela add NAME       # add a dependency
    vela update         # update lockfile
    vela doc            # generate documentation
    vela bench          # run benchmarks
    vela notebook       # serve the notebook UI
    vela lsp            # language server (stdio)
    vela kernel         # Jupyter kernel
    vela profile FILE   # sampling profiler
    vela publish        # publish a package
    vela new NAME       # scaffold a new project

There are no flags for stylistic choices. `vela fmt` has no options.
`vela check` has no configuration file. The toolchain is opinionated
by design.

## 13. Packages

A project is a directory containing `vela.toml`. The manifest:

    [package]
    name    = "my_analysis"
    version = "0.1.0"
    edition = "2026"

    [deps]
    polars-compat = "1.2"

    [dev-deps]
    quickcheck = "0.4"

A package containing Rust code adds a `rust/` subdirectory with its
own `Cargo.toml`. The Rust crate is built and linked automatically
by `vela build`; no separate command is required. Pure-Vela packages
have no `rust/` directory and require no Rust toolchain to build.

The lockfile `vela.lock` is committed and is the source of truth for
reproducible builds.

The central registry is `vela.pkg` (the registry URL and protocol are
defined separately). Packages are content-addressed; the registry
stores immutable artifacts.

## 14. Interop

Rust is Vela's primary interop language. A Vela package may be
written in pure Vela, in a mix of Vela and Rust, or (less commonly)
embedded inside a Rust application.

### 14.1 Mixed Vela and Rust packages

A package's layout is:

    my_pkg/
        vela.toml
        src/
            lib.vela
            ...
        rust/
            Cargo.toml
            src/lib.rs

The presence of `rust/Cargo.toml` triggers a Rust build during
`vela build`. The Rust crate is compiled by `cargo` (vendored by the
toolchain), produces a static library, and is linked into the Vela
runtime image for the package. The `rust/` crate must be a
`cdylib` or `staticlib` crate and must depend on the `vela-sdk`
crate, which provides the binding macros and types.

### 14.2 The Rust binding surface

Functions exported from Rust to Vela are written with the
`#[vela::export]` attribute:

    use vela_sdk::prelude::*;

    #[vela::export]
    pub fn dot(a: Series<f64>, b: Series<f64>) -> Result<f64> {
        if a.len() != b.len() {
            return Err(Error::msg("length mismatch"));
        }
        Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum())
    }

The macro generates the binding stub, the Vela type signature, and
the marshaling code. Types crossing the boundary use the `vela-sdk`
representations of `Int`, `Float`, `String`, `Series`, `Array`,
`DataFrame`, `Option`, and `Result`; these share representation with
the Vela runtime, so the common cases are zero-copy.

Vela calls into Rust through the same dispatch machinery as Vela-to-
Vela calls. There is no separate `extern` syntax for Rust exports;
they look like ordinary Vela functions at the call site.

### 14.3 Calling Vela from Rust

A Rust crate that embeds the Vela runtime depends on `vela-runtime`
and constructs a `Runtime` value:

    let rt = vela_runtime::Runtime::new()?;
    let module = rt.load("my_pkg")?;
    let result: f64 = module.call("dot", (a, b))?;

This path is used by tools, editors, and any host application that
wants to embed Vela.

### 14.4 C FFI

The C ABI is available through `extern "C"`:

    extern "C" =
        fn cblas_dgemm (
            layout : Int, transa : Int, transb : Int,
            m : Int, n : Int, k : Int,
            alpha : Float, a : Ptr[Float], lda : Int,
            b : Ptr[Float], ldb : Int, beta : Float,
            c : Ptr[Float], ldc : Int,
        ) : ()

The `Ptr` type is only constructible inside `extern` declarations
and standard library wrappers around them. C FFI is the fallback for
libraries with no Rust binding; new bindings should prefer a thin
Rust crate using `vela-sdk` over a direct `extern "C"`.

### 14.5 Reproducibility considerations

Both Rust and C dependencies are pinned by content hash in
`vela.lock` (see section 13). Rust crates pulled in through `cargo`
are locked through `cargo`'s lockfile, which `vela.lock` references
by hash. System libraries linked through `extern "C"` remain
non-pinned in 1.0 and trigger a `vela check` warning when present in
a reproducible build.

### 14.6 Data interchange

Apache Arrow IPC is supported as an on-disk and on-wire data
interchange format in `std.io`. Zero-copy in-process interop with
other Arrow-aware runtimes is a 1.x goal, not a 1.0 guarantee.

## 15. Notebook

`vela notebook` serves a browser-based notebook UI. Notebook files are
plain Vela files annotated with cell boundaries:

    #%% cell
    let df = read_csv "iris.csv"
    df

    #%% cell
    df |> summarize { mean = mean (col :petal_length) }

Notebooks evaluate cells in source order. Out-of-order execution is
not supported; the notebook UI hides the question by re-running
downstream cells when a cell changes. This is the only mode and there
are no settings to change it.

Notebook output cells store the bit-identical result of the last
execution. A notebook can be re-evaluated and the output diffed
against the stored result to detect reproducibility regressions.

## 16. Testing

Tests live alongside source in a `tests` block:

    pub let mean xs = sum xs / Float.of_int (length xs)

    tests =
        test "mean of empty is Err" =
            assert (mean [] == Err EmptyInput)
        test "mean of [1, 2, 3] is 2" =
            assert (mean [1.0, 2.0, 3.0] == Ok 2.0)

`vela test` discovers every `tests` block in the project. Property
tests use the `prop` form:

    prop "mean is between min and max" (xs : [Float]) when length xs > 0 =
        let m = mean xs |> Result.unwrap
        min xs <= m and m <= max xs

Doctests in `///` documentation comments above public items are
executed by `vela test` as well.

## 17. Versioning

Vela follows semantic versioning at the level of language and
standard library. The bytecode format and JIT are implementation
details and may change between any two versions.

Each `vela.toml` declares an `edition`. Editions group breaking
language changes; the toolchain supports building any supported
edition from any compatible toolchain version.

## 18. Bootstrap

Version 0.x ships the compiler, runtime, and core data structures in
Rust. The standard library beyond the core (everything in `std.stats`,
`std.plot`, `std.formula`, and most of `std.frame`) is written in
Vela.

A subset of the language sufficient to bootstrap the standard
library is defined as Vela-core and lives in section 5 of this
document. Vela-core has no formulas, no notebook syntax, and no
plotting layer; it is what the Rust front-end accepts.

## 19. Open questions

The following questions remain open and will be resolved before 1.0:

1. The set of supported number types beyond `Int` and `Float`
   (`UInt`, `BigInt`, `Decimal`).
2. The exact layout of the run manifest in `.vela/runs/`.
3. Whether traits can have default methods and, if so, the dispatch
   rule when a default and an `impl` both apply.
4. The Arrow IPC integration boundary for 1.0 versus 1.x.
5. The package registry protocol and authentication model.
6. Time zone database packaging (system vs vendored).

These questions are tracked in `docs/open-questions/`.
