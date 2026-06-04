use crate::{core, top_down, unparse, util};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeciderMessage {
    GetWorkingExpression,
    Provide,
    Decide { index: usize },
    Quit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderMessage {
    WorkingExpression(String),
    Steps(Vec<String>),
}

use DeciderMessage::*;
use ProviderMessage::*;

pub fn interact(
    mut controller: pbn::Controller<
        util::Timer,
        top_down::TopDownStep<core::ParameterizedFunction>,
    >,
) -> Result<(), String> {
    println!("{}", serde_json::to_string(&Decide { index: 3 }).unwrap());
    println!("{}", serde_json::to_string(&Provide).unwrap());
    loop {
        let mut options = controller.provide().unwrap();
        let exp = controller.working_expression();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let request: DeciderMessage = serde_json::from_str(&input).unwrap();

        let response = match request {
            DeciderMessage::GetWorkingExpression => Some(
                ProviderMessage::WorkingExpression(unparse::exp(exp).unwrap()),
            ),
            DeciderMessage::Provide => Some(ProviderMessage::Steps(
                options.iter().map(|_| "test".to_owned()).collect(),
            )),
            DeciderMessage::Decide { index } => {
                controller.decide(options.swap_remove(index));
                None
            }
            DeciderMessage::Quit => std::process::exit(0),
        };

        match response {
            Some(msg) => println!("{}", serde_json::to_string(&msg).unwrap()),
            None => (),
        }
    }
}
