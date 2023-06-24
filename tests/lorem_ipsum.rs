/// Generates a template component for lorem-ipsum

const LOREM_IPSUM: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In egestas dapibus diam, vitae commodo diam rhoncus ut. Donec at urna in nisl aliquet mattis. Sed semper nisi sed blandit egestas. In rutrum libero vel accumsan euismod. Phasellus ante leo, gravida ac consequat at, mattis tincidunt lorem. Suspendisse tincidunt ligula nulla, sed laoreet quam vehicula ac. Etiam sodales augue ut nisi mollis, ac dictum urna consequat.";

use std::sync::Arc;

use miette::{NamedSource, SourceSpan};
use template_compiler::{gen_component, Config as CompilerConfig, FileData, Node};

use anyhow::Result;

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};

use wasmtime_component_macro::bindgen;

bindgen!({
    inline: "
        package template:lorem-ipsum

        world template {
            record params {}

            export apply: func(param: params) -> string
        }
    "
});

#[test]
fn test_lorem_ipsum() -> Result<()> {
    let compiler_config = CompilerConfig {
        export_func_name: "apply".into(),
    };
    let span = SourceSpan::from((0, LOREM_IPSUM.len()));
    let file_data = FileData {
        source: Arc::new(NamedSource::new("lorem-ipsum.txt", LOREM_IPSUM)),
        contents: vec![(span, Node::Text { text: LOREM_IPSUM })],
    };

    let component = gen_component(&compiler_config, &file_data);
    let component_bytes = component.finish();

    // let component_ast = wasmprinter::print_bytes(&component_bytes).unwrap();
    // println!("{}", component_ast);

    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let component = Component::new(&engine, component_bytes)?;

    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let (lorem_ipsum, _) = Template::instantiate(&mut store, &component, &linker)?;

    let params = Params {};
    let result = lorem_ipsum.call_apply(&mut store, params)?;

    assert_eq!(result, LOREM_IPSUM);

    Ok(())
}
