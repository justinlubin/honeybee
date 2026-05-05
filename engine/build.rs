use std::process::Command;

fn main() {
    println!(
        "cargo::rustc-env=HONEYBEE_VERSION={}+{}",
        env!("CARGO_PKG_VERSION"),
        String::from_utf8(
            Command::new("git")
                .arg("rev-parse")
                .arg("--short")
                .arg("HEAD")
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
    );
}
