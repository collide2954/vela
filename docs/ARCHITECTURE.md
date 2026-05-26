# Architecture

This document describes how the Vela compiler and runtime are organized.
The source of truth for language behavior is `SPEC.md`. This document
describes the *implementation*: which crate owns what, how data flows
between them, and where the planned bytecode VM and JIT fit.

## Crate layout

    crates/
        vela-lexer/    # source bytes → tokens
        vela-parser/   # tokens → AST
        vela-check/    # AST → typed AST + diagnostics
        vela-eval/     # AST → values (tree-walking interpreter)
        vela-fmt/      # AST → canonical source text
        vela-diag/     # diagnostic data type + rustc-style rendering
        vela-pkg/      # vela.toml manifests and project scaffolding
        vela-repl/     # interactive read-eval-print loop
        vela-cli/      # the `vela` binary, dispatches subcommands

Each crate has a single, narrow responsibility. The CLI is a thin shell
that wires the other crates together.

## Pipeline

    source bytes
        │
        ▼     vela-lexer
    token stream  (indent-aware)
        │
        ▼     vela-parser
    AST
        │
        ▼     vela-check
    typed AST + warnings  ─────►  vela-fmt  ─────►  formatted source
        │
        ▼     vela-eval
    runtime values

The CLI maps subcommands onto stages:

| subcommand   | stages exercised                              |
|--------------|-----------------------------------------------|
| `vela check` | lex → parse → check                           |
| `vela fmt`   | lex → parse → fmt                             |
| `vela run`   | lex → parse → check → eval                    |
| `vela test`  | lex → parse → check → eval (tests blocks)     |
| `vela`       | lex → parse → check → eval (stateful, REPL)   |
| `vela new`   | scaffold project; no source pipeline involved |

Diagnostics flow from any stage to `vela-diag` for rendering.

## Sessions

`vela-check::Session` and `vela-eval::Session` retain their environment
across calls. The REPL builds on those; future tooling that re-checks
the same project incrementally (LSP, watch mode) will use them too. The
one-shot `check_program` and `run` helpers are convenience wrappers
that throw away the session at the end.

## Runtime today

`vela-eval` is a tree-walking interpreter over the AST. Closures hold
an `Env` with `Rc<RefCell>`-shared frames so `let rec` and `var`-with-
capture work. The `?` operator short-circuits via a `RuntimeError`
variant the closure boundary catches. Pattern matching uses a single
`match_pat` function shared by `let`-destructuring, `for`-loop
bindings, function parameters, and `match` arms.

This is enough to be a correct reference implementation. It is not
fast.

## Runtime tomorrow: bytecode and JIT

The spec commits to a register-based bytecode VM with a sampling
profiler and a JIT for hot functions. The planned crates:

    crates/
        vela-bytecode/  # IR: instructions, constant pool, function tables
        vela-vm/        # bytecode interpreter (the baseline tier)
        vela-jit/       # Cranelift-based native code emitter

### Compile path

    typed AST  ──►  vela-bytecode (compiler)  ──►  Chunk
    Chunk      ──►  vela-vm                   ──►  Value
    hot Chunk  ──►  vela-jit                  ──►  native fn pointer

The typed AST from `vela-check` is the input. The compiler emits a
`Chunk` per function: an instruction stream, a constant pool, a list of
upvalue descriptors, and source-span debug info. Chunks reference each
other by `FunctionId`.

### Instruction set (sketch)

The VM is register-based, three-address. Each function has a virtual
register window; calls allocate a new window. Instructions look like:

    LoadConst   rd, kidx
    Move        rd, rs
    Add | Sub | Mul | Div | Mod | Pow   rd, ra, rb
    Eq  | Ne  | Lt  | Le  | Gt | Ge     rd, ra, rb
    Not | Neg                           rd, ra
    Jump        offset
    JumpIfFalse rcond, offset
    Call        rd, rfn, base, n_args
    Return      rs
    MkClosure   rd, fid, n_upvals
    GetUpval    rd, idx
    SetUpval    idx, rs
    GetGlobal   rd, name_idx
    SetGlobal   name_idx, rs
    MkTuple     rd, base, n
    MkRecord    rd, base, n
    MkCons      rd, ctor_idx, base, n
    GetField    rd, robj, field_idx
    GetIndex    rd, rseries, ridx
    Match       rscrut, table_idx
    ?           rd, rres                # Result short-circuit; same
                                        # boundary semantics as today

The exact encoding (16-bit operands? variable-length?) is deferred
until measured. The shape above is what the compiler emits and the VM
consumes; everything else is an implementation detail.

### Closures and captures

Captured variables become *upvalues*: indirections that may point to a
stack slot of an enclosing frame while the frame is live, then "close"
to an owned cell when that frame returns. This is the Lua 5/Wren model
and survives `let rec` and `var`-with-capture as the tree-walker's
`Rc<RefCell>` frames do today.

### Pattern matching

`match` lowers to a decision tree the compiler builds from the patterns
and the scrutinee's type. The compiler reuses `vela-check`'s
exhaustiveness analysis to detect dead arms and missing cases at
compile time, before any code is emitted. At runtime the VM runs a
small dispatcher (`Match` instruction) that walks the decision tree.

### Profiling and JIT trigger

Each `Chunk` carries a counter. Hot edges (back-branches, function
entries) increment the counter. When it crosses a threshold the VM
hands the chunk to `vela-jit`. The JIT compiles to native code via
Cranelift, returns a function pointer, and the VM patches future calls
to dispatch through it.

The JIT is *optional*: every program that runs under the VM also runs
without the JIT, just slower. There is no JIT-only feature.

### Reproducibility constraints

`SPEC.md` §10 requires byte-identical output across runs. Three rules
follow for the runtime:

1. **Float reductions use a fixed tree.** The compiler emits a
   deterministic reduction shape; auto-parallelizing operations
   pre-split into equally sized chunks so the order of partial sums
   does not depend on thread count.
2. **No implicit randomness.** `Rand` flows through values; the VM
   does not own a PRNG; the JIT does not introduce any.
3. **No timing-dependent dispatch.** The JIT trigger is based on a
   counter (deterministic), not wall-clock time.

### Tier and deopt strategy

There are exactly two tiers: VM (baseline) and JIT (optimized). There
is no interpreter-only mode beyond the VM, and no separate IR between
VM bytecode and JIT input — the JIT consumes the same `Chunk`. If the
JIT ever needs to bail out (type mismatch, hot loop in cold code, GC
pressure) it returns control to the VM at the same instruction
boundary. There is no on-stack replacement in 1.0.

## Tooling roadmap

Implemented and stable:

- `vela check FILE`
- `vela run FILE`
- `vela test FILE`
- `vela fmt FILE...`     `vela fmt --check`
- `vela new NAME`        `vela new --lib NAME`
- `vela`                  (REPL)
- `vela explain CODE`

Planned, in roughly priority order:

- `vela build`           project-wide build, .vela/cache/
- `vela add NAME`        edit vela.toml's `[deps]`
- `vela update`          rewrite vela.lock
- `vela vendor`          materialize deps under `vendor/`
- `vela doc`             extract doc comments → HTML
- `vela bench`           benchmark runner
- `vela lsp`             language server (stdio); reuses the Sessions
- `vela history` / `vela diff-runs`   over `.vela/runs/*.json`
- `vela profile FILE`    sampling profiler
- `vela kernel`          Jupyter kernel
- `vela notebook`        notebook UI
- `vela app new|serve|build`   reactive-app scaffolding and bundling

Every subcommand in this list is described in `SPEC.md` §12.1.
