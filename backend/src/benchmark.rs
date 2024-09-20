use std::path::PathBuf;

// Directory format:
// - suite_directory/
//   - _suite.hblib
//   - _suite.py
//   - some_benchmark_name.hb
//   - another_benchmark.hb
//   - yet_another.hb
//   - ...
pub fn run(suite_directory: &PathBuf) -> Result<(), String> {
    assert!(suite_directory.is_dir());
    for entry in glob::glob(suite_directory.join("*.hb").to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
    {
        println!("{}", entry.display());
    }
    Ok(())
}
