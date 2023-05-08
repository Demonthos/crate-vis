use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Crate {
    pub name: String,
    pub size: usize,
}

#[derive(Debug, Deserialize)]
struct BloatResult {
    crates: Vec<Crate>,
}

pub fn get_retained_sizes(crate_path: PathBuf) -> Vec<Crate> {
    insure_cargo_bloat_installed();

    let output = std::process::Command::new("cargo")
        .arg("bloat")
        .arg("--release")
        .arg("--crates")
        .arg("--message-format")
        .arg("json")
        .arg("-n")
        .arg("0")
        .current_dir(crate_path)
        .output()
        .expect("failed to execute cargo bloat");

    let stdout = String::from_utf8(output.stdout).expect("failed to parse stdout");

    let deserialize_result: Result<BloatResult, serde_json::Error> = serde_json::from_str(&stdout);

    let bloat_result = deserialize_result.expect("failed to deserialize bloat result");

    bloat_result.crates
}

#[test]
fn test_get_retained_sizes() {
    get_retained_sizes(PathBuf::from("../cli"));
}

fn insure_cargo_bloat_installed() {
    println!("checking if cargo bloat is installed...");
    use std::process::Command;
    let installed = Command::new("cargo")
        .arg("bloat")
        .arg("-h")
        .output()
        .is_ok();

    if !installed {
        println!("cargo bloat is not installed, installing...");
        Command::new("cargo")
            .arg("install")
            .arg("cargo-bloat")
            .output()
            .expect("failed to install cargo bloat");
    }
}

// use twiggy_analyze as analyze;
// use twiggy_opt::{self as opt, CommonCliOptions};
// use twiggy_parser as parser;

// fn run(opts: &opt::Options) {
//     let mut items = parser::read_and_parse(opts.input(), opts.parse_mode())?;

//     let data = match opts {
//         opt::Options::Top(ref top) => analyze::top(&mut items, top)?,
//         opt::Options::Dominators(ref doms) => analyze::dominators(&mut items, doms)?,
//         opt::Options::Paths(ref paths) => analyze::paths(&mut items, paths)?,
//         opt::Options::Monos(ref monos) => analyze::monos(&mut items, monos)?,
//         opt::Options::Garbage(ref garbo) => analyze::garbage(&items, garbo)?,
//         opt::Options::Diff(ref diff) => {
//             let mut new_items = parser::read_and_parse(diff.new_input(), opts.parse_mode())?;
//             analyze::diff(&mut items, &mut new_items, diff)?
//         }
//     };

//     let mut dest = opts.output_destination().open()?;

//     data.emit(&items, &mut *dest, opts.output_format())
// }
