# Mini Rust Compiler

A minimalist compiler for a small subset of Rust that generates RISC-V assembly
code.  This project is based on the concepts from *Modern Compiler
Implementation in ML* by Andrew Appel, particularly the use of an Intermediate
Representation Trees. It was developed as part of the Compiler Construction
course at the University of Freiburg.

It builds on two official course codebases:

- The [Compiler Construction template repository](https://github.com/Compiler-Construction-Uni-Freiburg/compiler-construction-uni-freiburg-2025-compiler-2025-template-2025), which provides the project structure, build system, and RISC-V/QEMU-based execution environment.
- The [MiniRust reference implementation](https://github.com/Proglang-Uni-Freiburg/minirust), which provided the initial parser and type-checking infrastructure used as a starting point for the frontend.

The compiler implementation was extended beyond these foundations, including the full pipeline for a Rust-like language and code generation to RISC-V.

---

## Project Structure

* `src/parse` – Parser that produces the AST
* `src/ast` – AST definitions, formatting, and error handling
* `src/semant` – Semantic analysis (type checking, environments, scope resolution)
* `src/ir` – Intermediate Representation and lowering infrastructure
* `src/codegen` – RISC-V code generation 
* `runtime/` – Minimal C runtime used by generated programs
* `examples/` – Small `.mrs` test programs
* `tests/` – Compiler test suite and test harness
* `tmp/` – Output directory (generated assembly & binaries)
* `do` – Build/run helper script (Docker + local workflows)
* `Dockerfile` – Reproducible environment with Rust, GCC, RISC-V toolchain, QEMU

---

## Requirements

* Docker (for reproducible builds & tests)

All required toolchains are provided inside the Docker container, including:
- Rust (nightly toolchain for box patterns)
- RISC-V toolchain

---

## Quickstart

All Docker-related commands are run via the `do` script:

```bash
./do compile examples/example.mrs
./do run examples/example.mrs

./do test
./do docker-rebuild
```

The compiled assembly and binaries are placed in `tmp/` within the project
directory.
