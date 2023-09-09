/// Generates a template component for a simple website
use std::sync::Arc;

use miette::NamedSource;
use pretty_assertions::assert_eq;
use template_compiler::{gen_component, parse_file, Config as CompilerConfig, TemplateGenerator, Params};

use anyhow::Result;

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};

mod bindings {
    use wasmtime_component_macro::bindgen;

    bindgen!({
        inline: "
            package template:website

            world website {
                record params {
                    content: string,
                    title: string,
                    include-footer: bool,
                }

                export apply: func(param: params) -> string
            }
        ",

    });
}

const TEMPLATE: &'static str = "
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
</head>
<body>
    <h1>{{title }}</h1>
    {{ content }}

    {% if include_footer %}
    Thanks!!
    {% endif %}
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

    let params = Params::new(&file_data.contents);
    let template = TemplateGenerator::new(params, &file_data);
    let component = gen_component(&compiler_config, &template);
    let component_bytes = component.finish();

    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let component = Component::new(&engine, component_bytes)?;

    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let (website, _) = bindings::Website::instantiate(&mut store, &component, &linker)?;

    let title = "What is WebAssembly (Wasm)?";
    let content =
        "WebAssembly, commonly abreviated as Wasm, is a secure, portable, and fast compile target";
    let expected = format!(
        "
<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
</head>
<body>
    <h1>{}</h1>
    {}

    
    Thanks!!
    
</body>
</html>
",
        title, title, content
    );
    let params = bindings::Params { title, content, include_footer: true };
    let result = website.call_apply(&mut store, params)?;

    assert_eq!(result, expected);

    Ok(())
}
