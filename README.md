# Mini Rust Compiler

A minimalist compiler for a small language with Rust-like syntax that generates
RISC-V assembly code.  This project is based on the concepts from *Modern
Compiler Implementation in ML* by Andrew Appel, particularly the clean
separation of compiler phases and the use of an Intermediate Representation
(IR).

---

## Project Structure

* `src/parse` – Parser that produces the AST
* `src/ast` – Definition of AST nodes
* `examples/` – Small test programs
* `Dockerfile` – Environment with Rust and RISC-V toolchain

---

## Requirements

* Docker (for reproducible builds & tests)
* Rust toolchain (only for the compiler itself, not the target programs)
* RISC-V toolchain (included in the Docker container)

---

## Quickstart

All Docker-related commands are run via the `do` script:

```bash
./do docker-rebuild
./do compile examples/example.mrs
```

The compiled assembly and binaries are placed in `tmp/` within the project
directory.
