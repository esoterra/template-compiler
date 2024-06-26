<div align="center">
  <h1>Wasm Template Compiler</h1>

  <p>
    <strong>A compiler from templates to minimal Wasm Components</strong>
  </p>

  <p>
    <a href="https://crates.io/crates/template-compiler"><img src="https://img.shields.io/crates/v/template-compiler.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/template-compiler"><img src="https://img.shields.io/crates/d/template-compiler.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/claw-cli"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
  </p>

  <p>
    <a href="https://techforpalestine.org/learn-more"><img src="https://badge.techforpalestine.org/default" alt="build status" /></a>
  </p>
</div>

This project compiles template files based on Nunjucks to WebAssembly (Wasm) Components.

It is very much a work in progress and the current compiler simply generates a component
with a single export function that takes in parameters and returns the resulting template output string.

Features will be progressively added, tentatively in this order:

- [x] Parameter interpolation
- [x] Conditional rendering
- [ ] Repeated rendering
- [ ] Dotted/nested parameter names
- [ ] Filters
- [ ] Async/streams?

## Try it out

Invoke the compiler like this to generate a component for a template.

```sh
cargo run -- -i <input-path> -o <destination-path>
```

## Examples

The best examples are currently the [runtime tests](https://github.com/esoterra/template-compiler/tree/main/tests),
with the most interesting one being the ["website" test](https://github.com/esoterra/template-compiler/blob/main/tests/website.rs).
