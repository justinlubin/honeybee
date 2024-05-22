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

    let lib_src =
        std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let prog_src =
        std::fs::read_to_string(std::env::args().nth(2).unwrap()).unwrap();

    match syntax::parse::library().parse(lib_src) {
        Ok(lib) => {
            log::debug!("Library:\n{}", syntax::unparse::library(&lib));
            match syntax::parse::program().parse(prog_src) {
                Ok(prog) => {
                    log::debug!(
                        "Program:\n{}",
                        syntax::unparse::program(&prog)
                    );

                    if egglog_adapter::check_possible(&lib, &prog) {
                        println!(">>> Possible! <<<");
                    } else {
                        println!(">>> Not possible <<<");
                        return;
                    }

                    let mut s = synthesis::Synthesizer::new(&lib, &prog);

                    loop {
                        let options = s.options();
                        if options.is_empty() {
                            break;
                        }
                        let choice = analysis::auto(options);
                        s.step(&choice);
                    }

                    println!("{}\n", s.tree);

                    println!("{}\n", backend::Python::new(&s.tree))
                }
                Err(errs) => errs
                    .iter()
                    .for_each(|e| println!("Program parse error: {}", e)),
            };
        }
        Err(errs) => errs
            .iter()
            .for_each(|e| println!("Library parse error: {}", e)),
    }
}
