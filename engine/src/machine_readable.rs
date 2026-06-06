use crate::{cellgen, core, top_down, unparse, util};

use jsonrpcmsg::{Error, Id, Params, Request, Response};
use serde::Serialize;
use serde_json::json;

////////////////////////////////////////////////////////////////////////////////
// Message types and interaction handling

#[derive(Debug, Clone)]
pub enum DeciderMessage {
    WorkingExpression,
    Provide,
    Decide { index: usize },
    Quit,
}

#[derive(Debug, Clone, Serialize)]
pub enum ProviderMessage {
    WorkingExpression(String),
    Steps(Vec<cellgen::FunctionChoice>),
    AckDecide,
    AckQuit,
}

fn out_of_time() -> Error {
    Error::new(1, "Allocated time expired (early cutoff)".to_owned())
}

fn no_more_steps() -> Error {
    Error::new(2, "No more steps".to_owned())
}

fn handle(
    library: &core::Library,
    controller: &mut pbn::Controller<
        util::Timer,
        top_down::TopDownStep<core::ParameterizedFunction>,
    >,
    decider_message: &DeciderMessage,
) -> Result<ProviderMessage, Error> {
    match decider_message {
        DeciderMessage::WorkingExpression => {
            Ok(ProviderMessage::WorkingExpression(
                unparse::exp(controller.working_expression()).unwrap(),
            ))
        }
        DeciderMessage::Provide => {
            let options = controller.provide().map_err(|_| out_of_time())?;
            let function_choices = cellgen::fill(
                library,
                &options,
                cellgen::exp(&library, &controller.working_expression()),
            )
            .unwrap()
            .into_iter()
            .find_map(|c| match c {
                cellgen::Cell::Choice {
                    function_choices, ..
                } => Some(function_choices),
                _ => None,
            })
            .ok_or_else(|| no_more_steps())?;
            Ok(ProviderMessage::Steps(function_choices))
        }
        DeciderMessage::Decide { index } => {
            let mut options =
                controller.provide().map_err(|_| out_of_time())?;
            controller.decide(options.swap_remove(*index));
            Ok(ProviderMessage::AckDecide)
        }
        DeciderMessage::Quit => Ok(ProviderMessage::AckQuit),
    }
}

////////////////////////////////////////////////////////////////////////////////
// Parsing/unparsing

fn request_to_message(r: &Request) -> Result<DeciderMessage, Error> {
    match r.method.as_str() {
        "working_expression" => Ok(DeciderMessage::WorkingExpression),
        "provide" => Ok(DeciderMessage::Provide),
        "decide" => {
            match r.params.as_ref().ok_or_else(|| Error::invalid_params())? {
                Params::Array(values) => {
                    if values.len() == 1 {
                        let index = values[0]
                            .as_u64()
                            .and_then(|v| usize::try_from(v).ok())
                            .ok_or_else(|| Error::invalid_params())?;
                        Ok(DeciderMessage::Decide { index })
                    } else {
                        Err(Error::invalid_params())
                    }
                }
                Params::Object(_) => Err(Error::invalid_params()),
            }
        }
        "quit" => Ok(DeciderMessage::Quit),
        _ => Err(Error::method_not_found()),
    }
}

fn message_to_response(
    provider_message: &ProviderMessage,
) -> serde_json::Value {
    match provider_message {
        ProviderMessage::WorkingExpression(e) => json!(e),
        ProviderMessage::Steps(function_choices) => {
            serde_json::to_value(function_choices).unwrap()
        }
        ProviderMessage::AckDecide => json!("ack_decide"),
        ProviderMessage::AckQuit => json!("ack_quit"),
    }
}

////////////////////////////////////////////////////////////////////////////////
// IO

fn parse_input() -> Result<Request, Error> {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|_| Error::parse_error())?;
    let json =
        serde_json::from_str(&input).map_err(|_| Error::parse_error())?;
    jsonrpcmsg::deserialize::from_request_value(json)
        .map_err(|_| Error::invalid_request())
}

fn definitely_respond_error(e: Error, id: Option<Id>) {
    let res =
        jsonrpcmsg::serialize::to_response_string(&Response::error(e, id))
            .unwrap();

    println!("{}", res);
}

fn maybe_respond_error(e: Error, id: Option<Id>) {
    match id {
        Some(id) => definitely_respond_error(e, Some(id)),
        None => (),
    }
}

fn maybe_respond_success(v: serde_json::Value, id: Option<Id>) {
    let id = match id {
        Some(id) => id,
        None => return,
    };

    let res = jsonrpcmsg::serialize::to_response_string(&Response::success(
        v,
        Some(id),
    ))
    .unwrap();

    println!("{}", res);
}

////////////////////////////////////////////////////////////////////////////////
// Main

pub fn interact(
    library: &core::Library,
    controller: &mut pbn::Controller<
        util::Timer,
        top_down::TopDownStep<core::ParameterizedFunction>,
    >,
) -> Result<(), String> {
    loop {
        let request = match parse_input() {
            Ok(r) => r,
            Err(e) => {
                definitely_respond_error(e, None);
                continue;
            }
        };

        let decider_message = match request_to_message(&request) {
            Ok(dm) => dm,
            Err(e) => {
                maybe_respond_error(e, request.id);
                continue;
            }
        };

        let provider_message =
            match handle(library, controller, &decider_message) {
                Ok(pm) => pm,
                Err(e) => {
                    maybe_respond_error(e, request.id);
                    continue;
                }
            };

        let response = message_to_response(&provider_message);
        maybe_respond_success(response, request.id);

        match provider_message {
            ProviderMessage::AckQuit => break,
            _ => (),
        }
    }

    Ok(())
}
