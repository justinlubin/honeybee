mod ir;
mod parse;

fn main() {
    use chumsky::prelude::*;

    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    match parse::library().parse(src) {
        Ok(ast) => println!("{:?}", ast),
        Err(errs) => errs
            .into_iter()
            .for_each(|e| println!("Parse error: {}", e)),
    }
}
