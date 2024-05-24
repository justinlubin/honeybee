mod analysis;
mod backend;
mod derivation;
mod egglog_adapter;
mod ir;
mod syntax;
mod synthesis;

use chumsky::Parser;

fn main() {
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

    if !egglog_adapter::check_possible(&lib, &prog) {
        println!(">>> Not possible <<<");
        return;
    }

    let mut s = synthesis::Synthesizer::new(&lib, &prog);

    loop {
        println!("{}", s.tree);
        let options = s.options();
        if options.is_empty() {
            break;
        }
        let choice = analysis::manual(options);
        s.step(&choice);
    }

    println!("{}", backend::Python::new(&s.tree).emit().nbformat(&imp));
}
