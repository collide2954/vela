# The Vela Language Specification

Status: draft, version 0.5.
Target: Vela 1.0.
License: Apache-2.0.

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
2. Vela is tuned for numerical, tabular, and statistical work. GUIs
   and game engines are out of scope for the standard library. Web
   servers and interactive data applications are in scope, because
   sharing analyses is part of doing analysis.
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

Documentation comments use `///` immediately above the item they
document and `//!` at the top of a file to document the module:

    //! Descriptive statistics for Series and DataFrame columns.

    /// Compute the arithmetic mean of a Series of Float.
    ///
    /// ```
    /// assert mean [1.0, 2.0, 3.0] == Ok 2.0
    /// ```
    pub let mean xs = ...

Doc comments are Markdown. Fenced code blocks are extracted and run
as doctests by `vela test`.

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
    42u         # UInt (u64); suffix `u`
    10n         # BigInt (arbitrary precision); suffix `n`
    1.50d       # Decimal (arbitrary precision base-10); suffix `d`

Unsuffixed numeric literals are polymorphic at the call site. An
integer literal unifies with `Int`, `UInt`, `BigInt`, `Float`, or
`Decimal`; a decimal literal unifies with `Float` or `Decimal`. In
isolation a literal defaults to `Int` or `Float`. Once a literal is
bound to a name, the name has a concrete type; `let n = 1` gives
`n : Int`, and `n + 1.0` is a type error. Suffixed literals never
overflow at parse time; `10n ^ 100` is well-formed and exact.

When an arithmetic expression leaves operand types unconstrained
(for example `fn x y -> x + y`), the inferred type defaults to
`Int`. Vela does not have numeric polymorphism via a type class; a
function meant to work on multiple numeric types must annotate its
parameters or be written with explicit conversions.

String literals are double-quoted and support the usual escapes:

    "hello"
    "line\nbreak"

Symbol literals begin with a colon and name a column or key:

    :species

Boolean literals are `true` and `false`. The unit value is written
`()`.

Reserved words: `let`, `rec`, `var`, `fn`, `if`, `then`, `else`,
`match`, `with`, `when`, `type`, `trait`, `impl`, `for`, `in`,
`pub`, `module`, `import`, `as`, `where`, `scope`, `spawn`,
`extern`, `open`, `app`, `input`, `output`, `tests`, `test`,
`prop`, `true`, `false`, `and`, `or`, `not`.

## 5. Syntax

Vela is expression-oriented. Every construct produces a value; blocks
return the value of their last expression. A block whose final
statement is not an expression (for example, ends in `let` or
`for`) evaluates to `()`.

### 5.1 Bindings

    let x = 1                  # immutable binding
    var counter = 0            # mutable binding
    counter <- counter + 1     # mutation uses left-arrow

`let` introduces an immutable binding. `var` introduces a mutable
binding. The `<-` operator is the only way to mutate a `var`. Plain
`=` is binding, never assignment.

`let` is not recursive: the bound name is not in scope in its own
right-hand side. To define a recursive function, use `let rec`:

    let rec factorial n =
        if n <= 1 then 1
        else n * factorial (n - 1)

Within a `let rec ... and ... and ...` block, every name is in
scope in every right-hand side, enabling mutual recursion.

Shadowing is permitted: a later `let x = ...` in the same scope
rebinds `x`. The compiler emits a warning (`W0050`) on each
shadowing binding; pass `--allow shadow` to silence it locally.

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

Function parameters may be any irrefutable pattern. The common
cases are an identifier, a typed identifier `(name : T)`, the unit
pattern `()`, a tuple pattern, or a record pattern:

    let dist (a, b) (c, d) = (c - a) + (d - b)
    let area { width = w, height = h } = w * h
    let thunk () = 42

A refutable pattern (constructor or list pattern) is accepted as a
parameter only when the parameter's type makes the match
exhaustive. Otherwise the compiler emits an error.

Both lambda bodies (`fn x -> body`) and `let f x = body` definitions
accept a block body: a newline followed by an indented sequence of
statements ending in an expression. The block's value is its
trailing expression (section 5).

    let f x =
        let y = x + 1
        let z = y * 2
        z

    let g = fn x ->
        let y = x + 1
        y * 2

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
error. Exhaustiveness checking considers nested patterns: a match
on `(Int, Option[Int])` requires arms that together cover
`(_, None)` as well as `(_, Some _)`. Arms with guards do not
contribute to coverage.

Match arms accept the same block body syntax as functions: `| pat ->`
followed by a newline and an indented block whose trailing
expression is the arm's value.

Patterns include:

- Literals (`0`, `"abc"`, `true`).
- Variable bindings (`x`) and the wildcard `_`.
- Constructor patterns (`Some x`, `Circle r`, `Point { x, y }`).
- Tuple patterns (`(a, b, c)`).
- List patterns: `[]`, `[x]`, `[x, y]`, and `[x, ..rest]` for
  head-and-tail destructuring.
- Range patterns: `1..=10` for inclusive numeric ranges.
- Or-patterns: `Red | Blue -> "primary"`.
- As-bindings: `Circle r as c when r > 0.0 -> area c`.
- Guards: `pattern when expr -> body`.

Example:

    match shape with
    | Circle r as c when r > 0.0 -> area c
    | Square s | Rect { width = s, height = s } -> s * s
    | _                                          -> 0.0

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
    |> plot (aes { x = :species, y = :mu }) ++ bar ()

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

    let m = lm { formula = y ~ x1 + x2 * x3, data = df }

A formula is a syntactic object, not a string. The compiler parses
it, the standard library interprets it. Functions that take many
named parameters (like `lm`) accept a record; this is the canonical
way to express named arguments in Vela. The full grammar of
formulas is given in section 11.4.

### 5.9 Iteration

Transformations are written as pipelines of pure operations:

    let positives = xs |> filter (x > 0)
    let total     = xs |> sum

For effectful iteration (printing, accumulating into a `var`, calling
an IO function for each element), Vela provides a `for` form:

    for x in xs:
        println x

    var total = 0
    for x in xs:
        total <- total + x

The binding in `for binding in iter:` is an irrefutable pattern
just like a function parameter, so destructuring is natural:

    for (key, value) in pairs:
        println (format "{} = {}" key value)

`for` requires its body to evaluate to `()`; it is for side effects.
Anything that returns a value should be expressed with `map`,
`filter`, `fold`, or `summarize`. Vela has no `while` or `loop`
forms; unbounded iteration is expressed with `Stream.unfold` or
explicit recursion.

### 5.10 Closures

Anonymous functions capture their enclosing bindings by value. Because
`let` is immutable, a captured binding cannot be observed to change
after capture. A closure that needs to read a mutable slot must
capture a `var`, in which case the closure holds the slot itself and
the mutation is visible to both the closure and the enclosing scope.

    let x = 10
    let f = fn () -> x + 1
    # f is a closure over x by value; x cannot be rebound

    var counter = 0
    let inc = fn () -> counter <- counter + 1
    inc ()
    inc ()
    # counter is now 2

There are no capture lists, no reference/value markers, and no
lifetimes. Closures are values and can be returned, stored, and
passed freely.

### 5.11 Operator precedence

Precedence runs from tightest (top) to loosest (bottom).
Associativity is left unless noted.

    1. Field access `.`, function application (juxtaposition)
    2. Postfix `?`
    3. Prefix `-`, prefix `not`
    4. `^`                                       (right-assoc)
    5. `*`  `/`  `%`
    6. `+`  `-`
    7. `++`                                      (concatenation: strings, series, arrays, plots)
    8. `==`  `!=`  `<`  `<=`  `>`  `>=`
    9. `and`
   10. `or`
   11. `~`                                       (formula)
   12. `|>`

There are no user-defined operators. The standard library may not add
operator symbols; it adds functions instead. New operator symbols
are a language change and require a specification update.

### 5.12 Modules and imports

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

    T ::= Int | UInt | BigInt | Float | Decimal
        | Bool | String | Symbol | ()
        | [T]                       # Series of T
        | Array[T, n]               # n-dimensional array
        | DataFrame                 # heterogeneous tabular
        | { l1 : T1, ..., ln : Tn } # record
        | (T1, ..., Tn)             # tuple
        | T1 -> T2                  # function
        | Option[T] | Result[T, E]
        | C[T1, ..., Tn]            # user-declared constructor
        | 'a                        # type variable

Numeric types do not implicitly convert. `1 + 1.0` is a type error;
the conversion is written `Float.of_int 1 + 1.0` or, more commonly,
written as `1.0 + 1.0` at the source. `BigInt` and `Decimal` are
exact; `Float` is `f64`. The standard library defines explicit
conversions between every pair of numeric types and documents the
rounding mode for each one that is not exact.

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

In version 1.0 multiple dispatch is supported only inside `trait`
and `impl` blocks (section 6.2). Free functions are single-dispatch:
a name binds to at most one definition in a given scope. Free-
function multi-dispatch is a planned 1.x feature.

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

    let print_all (xs : [t]) where Show t =
        xs |> each (fn x -> println (show x))

A `where` clause introduces a constraint that the type checker
records as part of the function's inferred type. At each call site
the checker verifies that the concrete type substituted for the
constrained variable has the required `impl`.

### 6.3 Option and Result

Missing values are `Option[T]`. Recoverable errors are `Result[T,
E]`. There is no `null`, no `nil`, no `NA`. The `?` postfix operator
short-circuits a `Result`:

    let load path =
        let raw = read_file path?
        let df  = parse_csv raw?
        Ok df

`?` requires the enclosing function to return a `Result`, and the
error type of the operand must unify with the error type of the
enclosing function's return type. The checker tracks this expected
return type through the body of every function definition.

### 6.4 No exceptions, no panics in user code

User code cannot raise. The runtime may abort on unrecoverable
internal conditions (out-of-memory, stack overflow), and the standard
library functions that assert invariants document those aborts.
Aborts terminate the process; they are not catchable.

### 6.5 Type declarations

User-defined types are introduced with `type`. The same form covers
sum types (algebraic data types), nominal records, and newtypes:

    type Shape =
        | Circle Float
        | Square Float
        | Rect   { width : Float, height : Float }

    type Point = { x : Float, y : Float }

    type Email = Email String

A `type` with `|` alternatives is a sum type; each alternative is a
constructor that takes positional or record arguments. A `type` with
a single record body is a nominal record (distinct from structural
records of the same shape). A `type` with a single named
constructor wrapping one value is the idiomatic newtype.

Each constructor is exposed in the enclosing module's value scope as
a function from its arguments to the declared type. A nullary
constructor (`Leaf`) is a value of that type; a constructor with
arguments (`Circle Float`) is a function (`Float -> Shape`). Pattern
matching uses the same constructors.

A nominal record type is constructed by writing a bare record
literal in a context where the nominal type is expected:

    type Point = { x : Float, y : Float }

    let p : Point = { x = 1.0, y = 2.0 }   # nominal Point
    let q       = { x = 1.0, y = 2.0 }     # structural record

The literal's fields must match the nominal type's fields exactly
(no extras, no missing). A structural record literal in a position
expecting `Point` is rewritten to the nominal `Point` at
elaboration time and is otherwise distinct from `Point` thereafter.

Parametric types are written with type-variable parameters:

    type Tree 'a =
        | Leaf
        | Node (Tree 'a) 'a (Tree 'a)

Vela does not have type aliases distinct from `type`. A name binding
to an existing type is written as a newtype; opaque-vs-transparent is
controlled by whether the constructor is `pub`.

### 6.6 Equality, ordering, hashing, and display

Every type automatically implements `Eq`, `Ord`, `Hash`, and `Show`.
These implementations are derived from the structure of the type:

- `Eq` is structural equality. Records compare field by field; sums
  compare constructor and then payload; collections compare
  element-wise.
- `Ord` is the lexicographic order induced by field/constructor
  order in the declaration.
- `Hash` is consistent with `Eq`.
- `Show` produces the canonical Vela textual form. For built-in
  types the form parses back to an equal value; for user types it is
  the constructor name followed by arguments.

The autoderivations may be overridden with an explicit `impl`:

    type Person = { name : String, age : Int }

    impl Show Person =
        fn show p = format "{} ({})" p.name p.age

`Float` equality follows IEEE 754: `NaN != NaN`. `Float` ordering
puts `NaN` last. These choices are part of the spec and do not vary
across platforms.

`DataFrame` equality is structural over columns and rows; column
order matters. `==` on two `DataFrame` values larger than a runtime
threshold is permitted to use parallel comparison subject to the
determinism rules in section 9.

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

A scope's value is the tuple of its spawned tasks' return values, in
spawn order. If a spawned task returns a `Result`, that `Result`
appears as-is in the tuple; the surrounding code propagates errors
with `?` like any other `Result`. The runtime guarantees that all
spawned tasks complete or are cancelled before the scope returns.

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
   `.vela/runs/<timestamp>.json`. A SQLite index at
   `.vela/runs.db` is built and updated on demand by `vela history`
   and `vela diff-runs`; the JSON files are the source of truth and
   the index can be rebuilt from them.
5. The IANA time zone database is vendored with the toolchain.
   `DateTime` operations on the same input yield identical results
   regardless of the host's `/usr/share/zoneinfo`. The bundled
   tzdata version is part of `vela --version` output.

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

Primitive types and operations: `Int`, `UInt`, `BigInt`, `Float`,
`Decimal`, `Bool`, `String`, `Symbol`, `Option`, `Result`,
`Ordering`, conversions, and the prelude that is imported by default
into every module.

### 11.2 Collections (`std.collection`)

`Series` (written `[T]` in type expressions), `Array[T, n]`,
immutable `Map`, immutable `Set`, `Range`, and `Stream` for unbounded
or lazy sequences. Pipeline-friendly operations: `map`, `filter`,
`fold`, `each`, `group_by`, `sort_by`, `zip`, `chunk`, `window`,
`head`, `tail`. `Stream.unfold` is the canonical way to build an
unbounded sequence.

### 11.3 DataFrame (`std.frame`)

`DataFrame`, columnar storage backed by Apache Arrow arrays.
Operations: `select`, `filter`, `mutate`, `group_by`, `summarize`,
`join`, `pivot`, `unpivot`, `read_csv`, `write_csv`, `read_parquet`,
`write_parquet`, `read_arrow`, `write_arrow`. Columns are
`Series[Option[T]]`; nullability is a property of the column, not a
sentinel value, and maps directly onto Arrow's validity bitmap.

A DataFrame's type may be schema-erased (`DataFrame`) or carry a
static row schema (`DataFrame[{ x : Float, y : Int }]`). When the
schema is known, `df.x` is checked at compile time and returns the
exact column type `[Option[T]]`. When the schema is erased, `df.x`
returns `[Option['a]]` with `'a` left to inference; `df.col :x`
returns `Result[[Option['a]], ColumnError]` for the runtime
lookup.

    let p : DataFrame[{ x : Float, y : Int }] = ...
    let xs : [Option[Float]] = p.x                # static, checked

    let q : DataFrame = read_csv "x.csv"?         # schema erased
    let xs = q.col :x?                            # dynamic lookup

Because columns are Arrow arrays in memory, every DataFrame can be
handed to a Rust extension or an Arrow-aware runtime without copying.
The `to_arrow` and `from_arrow` constructors expose this directly.

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

Missing values are never skipped silently. A statistical function
that operates on `Series[Option[T]]` does not compile unless the
caller has either filtered the `None` values out (`filter_some`),
converted them to a default, or used a `*_skip_none` variant that
documents the policy explicitly.

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
"naive" datetime. The IANA time zone database is vendored with the
toolchain so that `DateTime` arithmetic is reproducible across hosts.
The bundled tzdata version is recorded in `vela --version` and in
the run manifest of any program that uses `std.time`.

### 11.10 Plot (`std.plot`)

A grammar-of-graphics plotting system. A plot is constructed by
applying `plot` to data and an aesthetic mapping, then concatenating
layers, scales, and facets with `++`:

    plot df (aes { x = :species, y = :petal_length })
      ++ point ()
      ++ smooth ()
      ++ facet_wrap :site

Layers: `point`, `line`, `bar`, `box`, `hist`, `density`, `smooth`,
`errorbar`, `ribbon`, `tile`, `text`. Scales: `scale_x_log`,
`scale_color_brewer`, and so on. Facets: `facet_wrap`, `facet_grid`.
The `++` operator is the same concatenation operator used for
strings, series, and arrays (section 5.11); composing a plot is
concatenating its layers. Plots render to SVG, PNG, and the native
notebook. Rendered output is bit-deterministic given the same data
and theme.

### 11.11 HTTP (`std.http`)

An HTTP client and server with a small router and middleware system.
Submodules:

- `std.http.client` for outbound requests (`get`, `post`, etc.,
  returning `Result[Response, HttpError]`).
- `std.http.server` for inbound requests: a `router` value built up
  with `get`, `post`, `put`, `delete`, `patch`, and `head`; a
  `serve` function that binds it to a port.
- `std.http.middleware` for cross-cutting wraps: `compress`,
  `log`, `cors`, `static_files`, `signed_cookies`.

Path parameters are bound by `:name` segments. JSON parsing is
provided by `std.json` and integrates via `Request.json` and
`Response.json`.

    import std.http as h

    let app =
        h.router
        |> h.get  "/users/:id"   (fn req -> ...)
        |> h.post "/users"       (fn req -> ...)
        |> h.middleware h.compress

    h.serve app { port = 8080 }

### 11.12 Apps (`std.app`)

A reactive application framework. See section 17 for the full
description; this module is the API surface used by app authors.

### 11.13 Print (`std.print`)

`print`, `println`, `eprint`, `eprintln`, and `format` consume any
value implementing `Show` (see section 6.6) and produce text. The
formatter syntax is positional with `{}` placeholders only; each
`{}` consumes the next argument and renders it via the argument's
`Show` implementation. Width, precision, alignment, and indexed
placeholders are not supported in 1.0; callers can build them
explicitly with `Float.to_string`, `Int.to_string`, and string
concatenation.

    format "x = {}, y = {}" x y

These functions are re-exported from the prelude.

### 11.14 Datasets (`std.data`)

A small set of well-known datasets is shipped with the standard
library so that newcomers can experiment, examples in the
documentation are self-contained, and `vela test` doctests do not
need network access or filesystem fixtures. Every dataset is a
`DataFrame` value embedded in the binary as Arrow data; loading is
zero-copy and the bytes are pinned by the toolchain version, so
results are bit-identical across machines.

The 1.0 set:

- `iris` — Fisher's irises (150 × 5).
- `penguins` — Palmer Station penguins, the modern replacement for
  `iris` (344 × 8).
- `mtcars` — Motor Trend automobiles (32 × 11).
- `diamonds` — ggplot2's diamonds (~54k × 10), large enough to
  demonstrate group-by and join performance.
- `airquality` — New York City air quality, 1973 (153 × 6).
- `USArrests` — violent-crime rates by US state (50 × 4).
- `anscombe` — Anscombe's quartet (11 × 8), four datasets with
  identical summary statistics and different shapes.
- `titanic` — passenger survival on the RMS Titanic (891 × 12).
- `faithful` — Old Faithful geyser eruptions (272 × 2).
- `trees` — black cherry trees (31 × 3).

    import std.data

    let df = data.iris
    let stats =
        df
        |> group_by :species
        |> summarize { mu = mean (col :petal_length) }

Each dataset's name, schema, source, and licensing are listed in
`vela doc std.data`. Datasets are immutable; mutating operations
return new `DataFrame` values.

## 12. Tooling

The `vela` binary is the only tool.

### 12.1 Subcommands

    vela                  # REPL
    vela run FILE         # compile and execute
    vela build            # build the current project
    vela test             # run tests
    vela fmt              # format
    vela check            # lint and type-check
    vela explain CODE     # show the long form of a diagnostic
    vela add NAME         # add a dependency
    vela update           # update lockfile
    vela vendor           # copy resolved dependencies into vendor/
    vela doc              # generate documentation
    vela bench            # run benchmarks
    vela notebook         # serve the notebook UI
    vela app new NAME     # scaffold a reactive app
    vela app serve        # serve the app in dev mode (hot reload)
    vela app build        # bundle the app as a single binary
    vela lsp              # language server (stdio)
    vela kernel           # Jupyter kernel
    vela profile FILE     # sampling profiler
    vela history          # list past runs from .vela/runs/
    vela diff-runs A B    # compare two recorded runs
    vela cache prune      # remove old build-cache entries
    vela new NAME         # scaffold a new project

There are no flags for stylistic choices. `vela fmt` has no options.
`vela check` has no configuration file. The toolchain is opinionated
by design.

### 12.2 Output conventions

All `vela` subcommands write machine-parseable output to stdout when
invoked with `--json` and human-oriented output otherwise. Colors are
on when stdout is a TTY and disabled otherwise; `NO_COLOR` is
respected. Exit codes are documented per subcommand and stable
across patch versions.

### 12.3 Formatter rules

`vela fmt` is opinionated and has no options. The rules are:

- Four-space indentation. Tabs are an error.
- 100-column soft limit; lines exceeding the limit are wrapped at
  the lowest-precedence operator.
- One blank line between top-level items; no blank lines within an
  expression block.
- Trailing commas on every multi-line list, record, and DataFrame
  literal.
- Pipelines break before `|>` and align continuations to the value
  position.
- Match arms align the `|` and `->` symbols within a single match.
- Imports are grouped (stdlib, dependencies, local) and sorted
  alphabetically within each group.

### 12.4 Build cache

Each compilation unit is written to `.vela/cache/<hh>/<hash>`,
where the hash is computed over the canonical source bytes, the
direct and transitive dependency hashes, and the toolchain version.
Reusing a cache entry is safe across machines for the same
toolchain. `vela build --clean` removes the cache for the current
project; `vela cache prune` removes cache entries older than a
configurable age.

## 13. Diagnostics

Diagnostics are a first-class part of the language. Vela's
benchmark is the diagnostic quality of `rustc`: every error message
identifies the precise source span, names the rule that was
violated, shows the surrounding source with carets, and, where
possible, suggests a fix.

A diagnostic has the following structure:

    error[E0123]: expected `Float`, found `String`
      --> analysis.vela:14:18
       |
    14 |     let m = mean "iris"
       |                  ^^^^^^ expected a Series of Float here
       |
       = note: `mean` has type [Float] -> Float
       = help: did you mean `mean (df.col :iris)`?

### 13.1 Levels

Diagnostics have four levels: `error`, `warning`, `note`, and
`help`. `error` is fatal to the current command; `warning` is not.
`vela check` treats warnings as errors when `--deny-warnings` is
passed; the default is to surface them and continue.

Every diagnostic carries:

- A stable code (`E0123`, `W0042`) that identifies the rule.
- A primary span and zero or more labelled secondary spans.
- A short title, a long explanation, and zero or more suggestions.

The full text of each diagnostic, indexed by code, is shipped in the
binary and viewable with `vela explain E0123`.

### 13.2 Spans across multiple files

When a diagnostic touches more than one file (for example, a type
mismatch between a call site and a definition), both files are
shown with their own gutters and the spans are connected by a
labelled note.

### 13.3 Suggestions

Suggestions that mechanically transform the source are emitted as
patches in the JSON output and rendered inline in human output.
`vela check --fix` applies all unambiguous suggestions.

### 13.4 No untraceable errors

Every error condition in the compiler, runtime, and standard
library is reachable through a documented code. There are no
anonymous panics, no `unreachable!` paths surfaced to the user, and
no errors that lack a span. When the runtime aborts, it produces a
crash report containing the stack trace, the active task, and a
bug-report URL.

### 13.5 Internationalization

Diagnostic strings are English in 1.0. The infrastructure
(structured codes, machine-readable output, externalized strings)
permits translation, but no translations ship in 1.0.

## 14. Packages

A project is a directory containing `vela.toml`. The conventional
layout is:

    my_pkg/
        vela.toml
        vela.lock
        src/
            lib.vela       # library entry point (if a library)
            main.vela      # binary entry point (if a binary)
            other.vela     # additional modules
            sub/
                module.vela
        tests/
            integration.vela
        examples/
            usage.vela

A package may be a library (`src/lib.vela` only), a binary
(`src/main.vela` only), or both. Other `.vela` files under `src/`
are private modules of the library; a module becomes part of the
library's public surface by being re-exported from `lib.vela` with
`pub import`.

The manifest:

    [package]
    name    = "my_analysis"
    version = "0.1.0"
    edition = "2026"

    [deps]
    stats     = { git = "https://github.com/vela-lang/stats", tag = "v1.2.0" }
    local_lib = { path = "../local_lib" }

    [dev-deps]
    quickcheck = { git = "https://github.com/vela-lang/quickcheck", tag = "v0.4.0" }

    [reproducibility]
    offline = false        # true requires vendor/ for all deps

A package containing Rust code adds a `rust/` subdirectory with its
own `Cargo.toml`. The Rust crate is built and linked automatically
by `vela build`; no separate command is required. Pure-Vela packages
have no `rust/` directory and require no Rust toolchain to build.

The lockfile `vela.lock` is committed and is the source of truth for
reproducible builds.

### 14.1 Dependency sources

Dependencies are resolved from one of three sources:

- `path = "..."` for local packages on disk.
- `git = "...", tag = "..."` (or `rev`, or `branch`) for git
  repositories. Tags and branches are resolved to commit SHAs and
  recorded in `vela.lock`; subsequent builds pin to the SHA.
- The built-in `std` package, which ships with the toolchain.

There is no central package registry in 1.0. Discoverability is the
responsibility of curated lists and the community; reproducibility
is the responsibility of `vela.lock`. A registry may be added in a
future release as a fourth source without breaking existing
manifests.

### 14.2 Versioning and resolution

Git tags that match `vN.N.N` are treated as semver versions for the
purpose of selecting between multiple version constraints in the
dependency graph. Tags that do not match a semver shape may still be
used, but only as exact pins (`tag = "..."` with no constraint
resolution).

### 14.3 Vendoring

`vela vendor` copies every resolved dependency into `vendor/`,
allowing fully offline reproducible builds. The vendored
directory is the only acceptable form of dependency for projects
that declare `reproducibility.offline = true` in `vela.toml`.

## 15. Interop

Rust is Vela's primary interop language. A Vela package may be
written in pure Vela, in a mix of Vela and Rust, or (less commonly)
embedded inside a Rust application.

### 15.1 Mixed Vela and Rust packages

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

### 15.2 The Rust binding surface

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

### 15.3 Calling Vela from Rust

A Rust crate that embeds the Vela runtime depends on `vela-runtime`
and constructs a `Runtime` value:

    let rt = vela_runtime::Runtime::new()?;
    let module = rt.load("my_pkg")?;
    let result: f64 = module.call("dot", (a, b))?;

This path is used by tools, editors, and any host application that
wants to embed Vela.

### 15.4 C FFI

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

### 15.5 Reproducibility considerations

Both Rust and C dependencies are pinned by content hash in
`vela.lock` (see section 14). Rust crates pulled in through `cargo`
are locked through `cargo`'s lockfile, which `vela.lock` references
by hash. System libraries linked through `extern "C"` remain
non-pinned in 1.0 and trigger a `vela check` warning when present in
a reproducible build.

### 15.6 Data interchange

Apache Arrow is Vela's in-memory columnar layout (see section 11.3),
which makes zero-copy interchange with any Arrow-aware runtime free
within a single process. Arrow IPC is also supported as an on-disk
and on-wire format in `std.io` for crossing process boundaries.

## 16. Notebook

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

## 17. Apps

`std.app` is Vela's reactive application framework, the equivalent
of Shiny for R. Apps are written in pure Vela, rendered on the
server, and delivered to the browser through a thin runtime that
swaps DOM nodes over a websocket. The app author writes no
JavaScript and no HTML templates.

### 17.1 Reactive cells

An app is an `app = ...` block containing three kinds of cells.

- `input name = widget { ... }` declares a piece of user state
  bound to a widget (slider, text box, file picker, dropdown).
- `let name = expr` declares an intermediate derivation.
- `output name = expr` declares a value to render in the page.

Each cell is a node in a dependency graph. The runtime tracks which
cells read which inputs and re-evaluates only the cells whose inputs
have changed. Cells without dependents are pruned.

    app =
        input n        = slider     { min = 1, max = 1000, default = 100 }
        input dataset  = file_picker { accept = [".csv"] }

        let df         = read_csv dataset?
        let sample     = df |> head n

        output table   = sample
        output hist    = plot sample (aes { x = :x }) ++ hist ()
        output summary = format "rows = {}, cols = {}" sample.rows sample.cols

The cell language is the rest of Vela; an app cell is an expression
of any type that has a `Show` implementation or a built-in renderer
(`DataFrame`, `Plot`, `String`, primitives).

### 17.2 Determinism and reproducibility

Cells are pure functions of their inputs. The runtime guarantees
that two evaluations with the same input values produce the same
output bytes. This is the determinism rule from section 9 applied
to the app: a recorded session can be replayed and the rendered
output diffed against the recording.

### 17.3 Sessions and auth

`std.http.middleware.signed_cookies` provides cookie-backed
sessions; the cookie value is HMAC-signed with a per-app secret.
`std.app.session` exposes the current session to cells. OAuth2
callback handling is provided by `std.app.oauth2` for a single
provider per app; multiple-provider flows are an application
concern.

### 17.4 Dev mode and deployment

`vela app serve` serves the app in development mode: on every save
the runtime re-typechecks, replaces the affected cells, and pushes
the new state to connected clients without a full reload. State
that did not change is preserved.

`vela app build` produces a single self-contained binary that
embeds the runtime, the cached bytecode, and the static assets. The
binary takes the same configuration as `vela app serve` at runtime.
There is no separate "build for production" step beyond this.

### 17.5 Limits

The app framework is for interactive analyses and internal tools.
It is not a general-purpose web framework: it has no routing beyond
mounting an app at a path, no template language, and no client-side
state outside of widget values. Authors who need those use
`std.http` directly.

## 18. Testing

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

## 19. Versioning

Vela follows semantic versioning at the level of language and
standard library. The bytecode format and JIT are implementation
details and may change between any two versions.

Each `vela.toml` declares an `edition`. Editions group breaking
language changes; the toolchain supports building any supported
edition from any compatible toolchain version.

## 20. Bootstrap

Version 0.x ships the compiler, runtime, garbage collector, JIT,
and core data structures in Rust. The standard library beyond the
core is written in Vela; this includes all of `std.stats`,
`std.plot`, `std.formula`, the high-level API of `std.frame`,
`std.app` reactivity, and the `std.http` router and middleware
layer. The low-level HTTP socket handling, Arrow column storage,
and the app runtime's DOM-diff layer remain in Rust through their
respective `vela-sdk` crates.

A subset of the language sufficient to bootstrap the standard
library is defined as Vela-core and lives in section 5 of this
document. Vela-core has no formulas, no notebook syntax, no
plotting layer, and no `app =` block; it is what the Rust
front-end accepts.

## 21. Open questions

No open questions remain at the level of this document. Decisions
made during implementation that warrant a specification change are
proposed as amendments to this file with a version bump.
