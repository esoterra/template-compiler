/// Generates a template component for a simple website

use std::sync::Arc;

use pretty_assertions::assert_eq;
use miette::NamedSource;
use template_compiler::{gen_component, Config as CompilerConfig, parse_file};

use anyhow::Result;

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};

use wasmtime_component_macro::bindgen;

bindgen!({
    inline: "
        default world website {
            record params {
                content: string,
                title: string,
            }
        
            export apply: func(param: params) -> string
        }
    "
});

const TEMPLATE: &'static str = "
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
</head>
<body>
    <h1>{{title}}</h1>
    {{ content }}
</body>
</html>
";

#[test]
fn test_website() -> Result<()> {
    let compiler_config = CompilerConfig {
        export_func_name: "apply".into(),
    };
    let source = Arc::new(NamedSource::new("website.html", TEMPLATE));
    let file_data = parse_file(source, TEMPLATE).unwrap();

    let component = gen_component(&compiler_config, &file_data);
    let component_bytes = component.finish();

    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let component = Component::new(&engine, component_bytes)?;

    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let (website, _) = Website::instantiate(&mut store, &component, &linker)?;

    let title = "What is WebAssembly (Wasm)?";
    let content = "WebAssembly, commonly abreviated as Wasm, is a secure, portable, and fast compile target";
    let expected = format!("
<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>
", title, title, content);
    let params = Params { title, content };
    let result = website.call_apply(&mut store, params)?;

    assert_eq!(result, expected);

    Ok(())
}
