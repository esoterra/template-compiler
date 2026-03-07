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
The template does not have to be valid HTML. It can be any text you want.
The components the compiler generates export a single function that takes in parameters and returns the template output string.

It currently supports parameter interpolation (via `{{ param }}` syntax) and conditional rendering (via `{% if param %}...{% endif %}` syntax).

I don't currently have time to dedicate to the project, but I would like to add more features eventually like loops/repeated rendering, dotted/nested parameter names, filters, and async streams.

## Basic Example

1. Write a template in the nunjucks-like style template-compiler supports like so.

```njk
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
</head>
<body>
    <h1>{{ title }}</h1>
    {{ content }}

    {% if include-footer %}
    Thanks!!
    {% endif %}
</body>
</html>
```

2. The compiler will generate a Wasm Component with an inferred component type like the following.

```wit
package template:website;

world website {
    record params {
        content: string,
        title: string,
        include-footer: bool,
    }

    export apply: func(param: params) -> string;
}
```

3. The component can now be provided to a Wasm Component runtime (like [Wasmtime](https://wasmtime.dev/)) and its exported `apply` function called. 

The full code required to invoke the compiler and run the component in this example is available in the ["website_cond" test](https://github.com/esoterra/template-compiler/blob/main/tests/website_cond.rs).

## Compiler CLI

The CLI is extremely simple and has no subcommands. There are named parameters for input path and output path. The `--help` flag is available.

```sh
cargo run -- -i <input-path> -o <destination-path>
```
