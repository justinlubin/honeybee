use crate::core::*;

pub fn python_value(v: &Value) -> String {
    match v {
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::Int(i) => i.to_string(),
        Value::Str(s) => format!("\"{}\"", s).to_owned(),
    }
}

pub fn python(e: &Exp) -> String {
    match e {
        crate::top_down::Sketch::Hole(h) => format!("?{}", h),
        crate::top_down::Sketch::App(f, args) => {
            format!(
                "{}({}{}_metadata={{{}}})",
                f.name.0,
                if args.is_empty() { "" } else { ", " },
                args.iter()
                    .map(|(fp, arg)| format!("{}={}", fp.0, python(arg)))
                    .collect::<Vec<_>>()
                    .join(", "),
                f.metadata
                    .iter()
                    .map(|(mp, v)| format!("{}={}", mp.0, python_value(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}
