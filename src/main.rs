use std::{fs, path::PathBuf, sync::Arc};

use anyhow::{Result, Context};
use clap::Parser;
use miette::NamedSource;

use template_compiler::{gen_component, parse_file, Config};

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

    let source = Arc::new(NamedSource::new(name, text.clone()));

    let file_data = parse_file(source, &text)?;
    let component = gen_component(&config, &file_data);
    fs::write(args.output, component.finish().as_slice())?;

    Ok(())
}
