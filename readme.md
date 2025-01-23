# `cargo pup`
## aka, Pretty Useful Pup

In the tradition of [ArchUnit](https://www.archunit.org/) and [ArchUnitNet](https://github.com/TNG/ArchUnitNET), **Pretty Useful Pup** (_pup_) lets you write assertions about the architecture of your Rust project, allowing you to continuously validate them locally and in your CI pipelines. Perhaps more significantly, it also introduces a fresh new naming convention for architectural linting tools. 

Pup lets you enforce your mental model of how your system should be structured, ensuring that consistency is maintained automatically rather than relying on all of the people submitting PRs to share a perfectly consistent understanding of the system. As systems and the number of contributors grow and it becomes increasingly hard to manually police consistency across a codebase. Lack of architectural consistency increases the cognitive load required to do work, and everyone is worse off for it.

While Rust allows us to model all sorts of structural constraints within its type system, much remains that contributes to an overall sense of consistency 
that cannot be enforced simply through traits: 

* Every implementation of `MyTrait` should be named `.*MyTrait` - Enforce naming consistency
* Every implementation of `MyTrait` should be `private` - Enforce visibility 
* All code beneath `myproj.public.myapi` should not import `reqwest` or `sqlx` - Enforce layering 

## Pretty Useful Pup Tenets

* **Not [clippy](https://github.com/rust-lang/rust-clippy)** - pup isn't interested in code style and common-mistake style linting. We already have a great tool for this!
* **Simple to use** - pup should be easy to drop onto a developer's desktop or into a CI pipeline and work seamlessly as a `cargo` extension
* **Simple to configure** - in the spirit of similar static analysis tools, pup reads from `pup.yaml` dropped into the root of a project
* **Easy to integrate** - TODO - reference that standard for exporting linting syntax. 


## Usage
**Pretty Useful Pup** is installed as a [cargo](TODO) subcommand. This simply means that it needs to be in your `$PATH`, 
optimally, in your `~/.local/bin` directory (following the so-called [XDG basedir](https://specifications.freedesktop.org/basedir-spec/latest/) specification).

**Step 1**: 
Install [rustup](https://rustup.rs/) to manage your local rust installs and provide the tooling required for Pretty Useful Pup, if you haven't already.

**Step 2**:
Download `cargo-tt` and `tt-driver` from the [releases](TODO) page for your platform and drop them in your `~/.local/bin` directory. 
Run `cargo tt` to make sure everythings working; you should get a typical help page back. 

Alternatively, run [tt-up.sh](TODO) to download the latest release and install it automatically. As with any scripts run blindly from
the internet, have a quick read through first!

**Step 3**: 







# Compilation
We want something that runs without having to have exactly the same nightly rustup'd version available on the `rpath`.
The rustc nightly's themselves do not contain static libraries for `librustc_driver`, only `dylib`s. Likewise, if you
build rust from the rust repo, by default you only get dylibs. 

Here are the options:

* Include the `dylib`s from the compiler in the distribution package, in the same dir (TODO - check if this works)
The `dylib` is about 145 MiB.
This works if you set `DYLD_LIBRARY_PATH` to contain the bin dir. 
We could probably also do it by using `install_tool`: 

<!-- this doesn't work !! --> 
`install_name_tool -add_rpath `.` target/debug/cargo-gsr`

<!-- this does work! --> 
install_name_tool -add_rpath @executable_path target/debug/cargo-gsr

* Build rustc with `compiler/rust_driver/Cargo.toml` modified to set `cratetype` to `staticlib` (note - the lib is about 400 MiB) 
* Somehow make the project link dynamically against the `librustc_driver.dylib` available in the user's distribution

# Code Processing

Type definitions for HIR, MIR, and THIR are in the [rustc_middle](https://doc.rust-lang.org/stable/nightly-rustc/rustc_middle/) crate.
[intermediate representations summary](https://rustc-dev-guide.rust-lang.org/overview.html#intermediate-representations)

[TyCtx](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html)
"The central data structure of the compiler. Stores references to the various _arenas_ and 
houses the results of the various _queries_. 

`TyKind` is a big enum with variants to represent many different rust types (primitives, references, algebraic data types, generics, lifetimes, ...)

The `<'tcx>` everywhere is the lifetime of the arena the type is stored in. We can basically just ignore it.

[the ty module - representing types](https://rustc-dev-guide.rust-lang.org/ty.html)
`rustc_hir::ty` reflects what the user wrote, and not really the underlying type itself.
`rustc_middle::ty::Ty` reflects the actual types themselves and their semantics.
There's also some stuff in here on comparing types - which will become important later!


This seems important:

_One other thing to note is that many values in the compiler are interned. This
is a performance and memory optimization in which we allocate the values in a
special allocator called an arena. Then, we pass around references to the values
allocated in the arena. This allows us to make sure that identical values (e.g.
types in your program) are only allocated once and can be compared cheaply by
comparing pointers. Many of the intermediate representations are interned._

The structure of the compiler is driven by queries, rather than blocks of work. That means
that you don't do all of the stages below, one by one, but rather some query "pulls on" the 
optimized MIR of a function, which then pulls on the THIR, which pulls on the HIR, and so on.

There are exceptions to this but this is the general structural principal.

## Generating MIR/HIR/THIR/etc

```bash
# X one of
# * hir
# * hir,typed
# * hir-tree
# * thir-tree
# * thir-flat
# * mir 
# * stable-mir
# * mir-cfg
cargo rustc -- -Z unpretty={X}
```


## 1. Tokenization / Lexxing / Parsing

[rustc_lexer](https://github.com/rust-lang/rust/tree/master/compiler/rustc_lexer)
[rustc_parse](https://github.com/rust-lang/rust/tree/master/compiler/rustc_parse)

Happens in `rustc_lexer`, then `rustc_parse`.

Output is an abstract syntax tree (AST).

## 2. High-Level Intermediate Representation (HIR)

[rustc_hir](https://github.com/rust-lang/rust/tree/master/compiler/rustc_hir)
[compiler guide docs](https://rustc-dev-guide.rust-lang.org/hir.html)

The AST is _lowered_ into HIR. This involves lots of desugaring - 
of loops, async fn, and so on.

Using this representation we do:

* Type inference 
* Trait solving (which impl is used for each reference to a trait)
* Type checking - here we convert HIR types to compiler-internal types - `hir::Ty` --> `Ty<'tcx>`

"The top level data structure in the HIR is the Crate".

* A `DefId` refers to a definition any _any_ crate
* A `LocalDefId` refers to a definition in the _current_ crate
* A `HirId` refers to _any node_ in the HIR


## 3. Typed HIR (THIR)
Lowered from HIR. An even more desugared HIR used for pattern and exhaustiveness checking.

### 4. MIR 

[MIR section from compiler guide](https://rustc-dev-guide.rust-lang.org/mir/index.html)

Lowered from THIR, used for optimizations.
"This is basically a Control-Flow Graph"
This is also used for monomorphization - replacing generic code with concrete-typed code. We collect
the information needed to do this at the MIR level, but the process is actually done when we convert
MIR to LLVM-IR.

### 5. Codegen / LLVM-IR
First we convert MIR to LLVM-IR, then we kick it over the fence to LLVM.
