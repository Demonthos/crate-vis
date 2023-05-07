use clap::Parser;
use crate_vis::*;

fn main() {
    let args = Args::parse();

    generate_graph(args).unwrap();
}
