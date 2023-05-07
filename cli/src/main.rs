use std::path::PathBuf;

use clap::Parser;
use crate_vis::*;

#[derive(Parser, Debug, Default)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, default_value_t = Default::default())]
    workspace_color: Rgba,

    #[arg(short, long, default_value_t = Default::default())]
    duplicate_color: Rgba,

    #[arg(short, long)]
    exclude: Option<Vec<String>>,

    #[arg(short, long, default_value_t = Default::default())]
    only_workspace: bool,

    #[arg(short, long)]
    targets: Vec<String>,

    #[arg(short, long)]
    manifest_path: Option<String>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    features: Option<Vec<String>>,

    #[arg(short, long, default_value_t = Default::default())]
    all_features: bool,
}

impl From<Args> for VisualizationCfg {
    fn from(val: Args) -> Self {
        VisualizationCfg {
            workspace_color: val.workspace_color,
            duplicate_color: val.duplicate_color,
            exclude: val.exclude.unwrap_or_default().into_iter().collect(),
            only_workspace: val.only_workspace,
            targets: val.targets,
            manifest_path: PathBuf::from(val.manifest_path.unwrap_or("./Cargo.toml".to_string())),
            output: PathBuf::from(val.output.unwrap_or("./graph.svg".to_string())),
            features: val.features.unwrap_or_default(),
            all_features: val.all_features,
        }
    }
}

fn main() {
    let args = Args::parse();

    let cfg = args.into();

    generate_graph(cfg).unwrap();
}
