use std::{fs, path::PathBuf};

use clap::Parser;

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

fn main() {
    let args = Args::parse();
    let config = Config {
        export_func_name: args.export_name.unwrap_or("apply".into()),
        export_mem_name: "mem".into(),
    };

    let file_data = parse_file(args.input);
    let component = gen_component(&config, &file_data);
    fs::write(args.output, component.finish().as_slice()).unwrap();
}
