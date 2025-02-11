use crate::core::*;
use crate::top_down;

fn python_value(v: &Value) -> String {
    match v {
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::Int(i) => i.to_string(),
        Value::Str(s) => format!("\"{}\"", s).to_owned(),
    }
}

pub fn python_multi(e: &Exp, current_indent: usize) -> String {
    match e {
        top_down::Sketch::Hole(h) => top_down::pretty_hole_string(*h),
        top_down::Sketch::App(f, args) => {
            let new_indent = current_indent + 1;
            format!(
                "{}({}\n{}_metadata={{{}}}\n{})",
                f.name.0,
                args.iter()
                    .map(|(fp, arg)| format!(
                        "\n{}{}={},",
                        "  ".repeat(new_indent),
                        fp.0,
                        python_multi(arg, new_indent)
                    ))
                    .collect::<Vec<_>>()
                    .join(""),
                "  ".repeat(new_indent),
                f.metadata
                    .iter()
                    .map(|(mp, v)| format!("{}={}", mp.0, python_value(v)))
                    .collect::<Vec<_>>()
                    .join(", "),
                "  ".repeat(current_indent)
            )
        }
    }
}

pub fn python_single(e: &Exp) -> String {
    match e {
        top_down::Sketch::Hole(h) => top_down::pretty_hole_string(*h),
        top_down::Sketch::App(f, args) => {
            format!(
                "{}({}{}_metadata={{{}}})",
                f.name.0,
                args.iter()
                    .map(|(fp, arg)| format!("{}={}", fp.0, python_single(arg)))
                    .collect::<Vec<_>>()
                    .join(", "),
                if args.is_empty() { "" } else { ", " },
                f.metadata
                    .iter()
                    .map(|(mp, v)| format!("{}={}", mp.0, python_value(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

// func(hello = g(goodbye = 3))
// func(
//   hello = g(
//     goodbye = 3
//   ),
// )
