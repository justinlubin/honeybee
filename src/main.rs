mod analysis;
mod backend;
mod derivation;
mod egglog_adapter;
mod ir;
mod syntax;
mod synthesis;

use chumsky::Parser;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    env_logger::init();

    let lib = syntax::parse::library()
        .parse(
            std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap(),
        )
        .unwrap();

    let imp =
        std::fs::read_to_string(std::env::args().nth(2).unwrap()).unwrap();

    let prog = syntax::parse::program()
        .parse(
            std::fs::read_to_string(std::env::args().nth(3).unwrap()).unwrap(),
        )
        .unwrap();

    let output_filename = std::env::args().nth(4).unwrap();

    if !egglog_adapter::check_possible(&lib, &prog) {
        println!(">>> Not possible <<<");
        return Ok(());
    }

    let mut s = synthesis::Synthesizer::new(&lib, &prog);

    loop {
        print!("{}", s.tree.pretty());
        let options = s.options();
        if options.is_empty() {
            break;
        }
        println!();
        let choice = analysis::fast_forward(options);
        s.step(&choice);
    }

    let nb = backend::Python::new(&s.tree).emit().nbformat(&imp);

    let mut output_file = File::create(output_filename)?;
    write!(output_file, "{}", nb)?;

    Ok(())
}
