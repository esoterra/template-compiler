/// Generates a template component for lorem-ipsum

const LOREM_IPSUM: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In egestas dapibus diam, vitae commodo diam rhoncus ut. Donec at urna in nisl aliquet mattis. Sed semper nisi sed blandit egestas. In rutrum libero vel accumsan euismod. Phasellus ante leo, gravida ac consequat at, mattis tincidunt lorem. Suspendisse tincidunt ligula nulla, sed laoreet quam vehicula ac. Etiam sodales augue ut nisi mollis, ac dictum urna consequat.

Quisque suscipit viverra turpis, at semper risus tristique eget. Aenean massa sapien, auctor vitae orci in, pellentesque rhoncus augue. Maecenas fringilla mi sit amet posuere consectetur. Curabitur sed molestie tellus. Curabitur lacus sapien, sagittis non efficitur non, consectetur non quam. Nullam volutpat nec nisi quis fringilla. Suspendisse aliquam, elit sit amet pharetra facilisis, magna augue semper erat, a vestibulum mi purus quis nisl. In hendrerit, velit eget condimentum finibus, orci diam porttitor ipsum, nec condimentum nisi ante quis leo. Quisque finibus dolor urna, eu dapibus metus viverra vitae. Etiam iaculis laoreet tempor. Quisque vitae facilisis dolor. Donec porta quam sit amet risus dapibus, at sodales sapien blandit. Phasellus ac arcu eget diam rhoncus gravida quis et arcu.

Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Mauris vulputate, nisi at imperdiet interdum, est nisl facilisis ligula, sed mattis tellus felis non enim. Praesent iaculis tellus non quam auctor, et faucibus tortor finibus. Nulla diam diam, pulvinar id libero non, vulputate laoreet ante. Nullam quis neque nec nunc hendrerit efficitur eget feugiat nisi. Nam non lacus vel enim sollicitudin auctor ac pellentesque ipsum. Nullam tempor ullamcorper libero, ac cursus neque feugiat quis. Nullam quis velit diam. Integer posuere hendrerit ex, a ultricies lorem. Vestibulum congue tortor libero, a accumsan dolor finibus eget. Integer sed tempus metus. Vivamus facilisis lobortis elementum. Cras placerat nibh sed turpis lacinia imperdiet in non ex. Phasellus enim ex, sodales ac rhoncus id, consectetur nec magna.

Praesent facilisis dolor diam, lacinia egestas diam vulputate eu. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Duis luctus porttitor massa a malesuada. Vestibulum imperdiet erat sed condimentum pellentesque. Ut fermentum sollicitudin commodo. Mauris vulputate elit ultrices, ornare ipsum vel, placerat elit. Quisque scelerisque vitae purus ac semper. Phasellus auctor, augue ut egestas consequat, lorem dolor posuere mauris, eget tempus ligula leo ut sapien. Morbi nec sapien sed enim dictum scelerisque quis at dolor. Vivamus pharetra felis non mauris bibendum auctor. Vivamus efficitur consectetur mi nec vestibulum. Etiam tempus, leo dapibus sodales tincidunt, risus lacus rhoncus velit, ac convallis magna leo in mi. Etiam sodales nunc at diam maximus, quis ornare neque aliquam. Sed auctor tincidunt augue.

Suspendisse odio dui, finibus ut suscipit non, pulvinar ut purus. Phasellus quis gravida ligula, vel pharetra mauris. Proin vel arcu laoreet, cursus nisl ac, varius erat. Donec vel tempus tortor, at semper orci. Nam tincidunt egestas augue, vitae porta tellus porttitor ut. Aenean at est sit amet magna pellentesque pharetra eget eu massa. Vestibulum imperdiet lacus nec purus mollis sodales. Morbi placerat mauris a nunc pharetra, in faucibus quam ornare. Cras nec sapien eget mi vestibulum tempor vel sit amet risus. Curabitur nec dolor sed ligula efficitur ullamcorper. ";

use template_compiler::{ Config as CompilerConfig, FileData, gen_component };

use anyhow::Result;

use wasmtime::{Engine, Config, Store, component::{Component, Linker, TypedFunc}};

#[test]
fn test_lorem_ipsum() -> Result<()> {
    let compiler_config = CompilerConfig {
        export_func_name: "apply".into(),
        export_mem_name: "mem".into()
    };
    let file_data = FileData {
        name: "lorem-ipsum.txt".into(),
        contents: LOREM_IPSUM.into()
    };
    let component = gen_component(&compiler_config, &file_data);
    let component_bytes = component.finish();
    // println!("Output: {}", wasmprinter::print_bytes(&component_bytes).unwrap());

    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let component = Component::new(&engine, component_bytes)?;

    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;

    let apply: TypedFunc<(), (String,)> = instance.get_typed_func(&mut store, "apply")?;
    let result = apply.call(&mut store, ())?.0;

    assert_eq!(result, LOREM_IPSUM);

    Ok(())
}