use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use template_compiler::{Config, Params, TemplateGenerator, gen_component, parse_template};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,

    // The export name for the template function
    #[arg(short, long)]
    export_name: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config {
        export_func_name: args.export_name.unwrap_or("apply".into()),
    };

    let name: String = args
        .input
        .file_name()
        .context("No file name found")?
        .to_str()
        .context("File name was not valid utf-8")?
        .into();

    let text = fs::read_to_string(args.input)?;

    let ast = parse_template(&name, &text)?;
    let params = Params::new(&ast);
    let template = TemplateGenerator::new(params, &ast);
    let component = gen_component(&config, &template);
    fs::write(args.output, component.finish().as_slice())?;

    Ok(())
}
