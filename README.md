# Wasm Template Compiler

This project compiles template files based on Nunjucks to WebAssembly (Wasm) Components.

It is very much a work in progress and the current compiler simply generates a component
that prints out the entire template input.

Features will be progressively added, tentatively in this order:

1. Parameter interpolation
2. Conditional rendering
3. Repeated Rendering
4. Filters
5. Async/streams?

## Try it out

Invoke the compiler like this to generate a component for a template.

```sh
cargo run -- -i <input-path> -o <destination-path>
```
