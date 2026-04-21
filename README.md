# Mini Rust Compiler

A minimalist compiler for a small of Rust that generates RISC-V assembly code.
This project is based on the concepts from *Modern Compiler Implementation in
ML* by Andrew Appel, particularly the use of an Intermediate Representation Trees.

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
* Rust toolchain (only for the compiler itself, not the target programs)
* RISC-V toolchain (included in the Docker container)

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
