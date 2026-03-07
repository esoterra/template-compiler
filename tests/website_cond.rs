/// Generates a template component for a simple website
use pretty_assertions::assert_eq;
use template_compiler::{
    Config as CompilerConfig, Params, TemplateGenerator, gen_component, parse_template,
};

use anyhow::Result;

use wasmtime::{
    Config, Engine, Store,
    component::{Component, Linker},
};

mod bindings {
    use wasmtime::component::bindgen;

    bindgen!({
        inline: "
            package template:website;

            world website {
                record params {
                    content: string,
                    title: string,
                    include-footer: bool,
                }

                export apply: func(param: params) -> string;
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
    <h1>{{ title }}</h1>
    {{ content }}

    {% if include-footer %}
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
    let ast = parse_template("website-cond.html", TEMPLATE).unwrap();

    let params = Params::new(&ast);
    let template = TemplateGenerator::new(params, &ast);
    let component = gen_component(&compiler_config, &template);
    let component_bytes = component.finish();

    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let component = Component::new(&engine, component_bytes)?;

    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let website = bindings::Website::instantiate(&mut store, &component, &linker)?;

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
    let title = title.to_owned();
    let content = content.to_owned();
    let params = bindings::Params {
        title,
        content,
        include_footer: true,
    };
    let result = website.call_apply(&mut store, &params)?;

    assert_eq!(result, expected);

    Ok(())
}
