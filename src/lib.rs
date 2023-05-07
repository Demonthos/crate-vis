use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::str::FromStr;

use krates::Krates;
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::*;
use layout::std_shapes::shapes::*;
use layout::topo::layout::VisualGraph;

use clap::Parser;

/// Simple program to greet a person
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

#[test]
fn run() {
    let args = Args {
        workspace_color: Rgba {
            r: 0,
            g: 255,
            b: 0,
            a: 255,
        },
        duplicate_color: Rgba {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        },
        only_workspace: false,
        ..Default::default()
    };

    generate_graph(args).unwrap();
}

pub fn generate_graph(args: Args) -> Result<(), krates::Error> {
    use krates::{cm, Builder, Cmd};
    dbg!(&args);

    let mut cmd = Cmd::new();
    cmd.manifest_path(args.manifest_path.unwrap_or("./Cargo.toml".to_string()));
    cmd.features(args.features.iter().flat_map(|i| i.iter().cloned()));
    if args.all_features {
        cmd.all_features();
    }

    let mut builder = Builder::new();

    builder.include_targets(
        args.targets
            .iter()
            .map(|s| (s.as_str(), Default::default())),
    );

    let krates: Krates = builder.build(cmd, |_: cm::Package| {})?;

    let mut vg = petgraph_to_graph_vis(
        krates,
        args.only_workspace,
        args.workspace_color.to_u32(),
        args.duplicate_color.to_u32(),
        args.exclude
            .iter()
            .flat_map(|v| v.iter().map(|s| s.as_str()))
            .collect(),
    );

    let mut svg = layout::backends::svg::SVGWriter::new();
    vg.do_it(false, false, false, &mut svg);

    let _ = layout::core::utils::save_to_file(
        args.output.as_deref().unwrap_or("./graph.svg"),
        &svg.finalize(),
    );

    Ok(())
}

fn petgraph_to_graph_vis(
    krates: Krates,
    workspace_only: bool,
    workspace_color: u32,
    duplicate_color: u32,
    exclude: HashSet<&str>,
) -> VisualGraph {
    let in_workspace: HashSet<_> = krates
        .workspace_members()
        .filter_map(|id| match id {
            krates::Node::Krate { id, .. } => Some(id.clone()),
            _ => None,
        })
        .collect();

    let mut vg = VisualGraph::new(Orientation::TopToBottom);

    let mut seen: HashMap<&String, HashSet<&krates::semver::Version>> = HashMap::new();
    let mut duplicates = HashSet::new();
    for krate in krates.krates() {
        let name = &krate.name;
        let version = &krate.version;
        if let Some(set) = seen.get_mut(name) {
            let already_contained = set.insert(version);
            if !already_contained {
                duplicates.insert(name);
            }
        } else {
            let mut set = HashSet::new();
            set.insert(version);
            seen.insert(name, set);
        }
    }

    let sz = Point::new(100., 100.);
    let mut nodes = HashMap::new();
    for krate in krates.krates() {
        let (name, version) = (&krate.name, &krate.version);
        let sp = ShapeKind::new_box(&format!("{}-{}", name, version));
        let mut look = StyleAttr::simple();
        if duplicates.contains(name) {
            look.fill_color = Some(layout::core::color::Color::new(duplicate_color));
        }
        let is_in_workspace = in_workspace.contains(&krate.id);
        if is_in_workspace {
            look.fill_color = Some(layout::core::color::Color::new(workspace_color));
        }
        let node = Element::create(sp, look, Orientation::TopToBottom, sz);

        if workspace_only && !is_in_workspace {
            continue;
        }
        if exclude.contains(name.as_str()) {
            continue;
        }
        let handle = vg.add_node(node);
        nodes.insert(krate.id.clone(), handle);
    }

    for krate in krates.krates() {
        let is_in_workspace = in_workspace.contains(&krate.id);
        if workspace_only && !is_in_workspace {
            continue;
        }
        if exclude.contains(krate.name.as_str()) {
            continue;
        }

        let id = krates.nid_for_kid(&krate.id).unwrap();

        let handle0 = nodes[&krate.id];
        for dep in krates.direct_dependencies(id) {
            let is_in_workspace = in_workspace.contains(&dep.krate.id);
            if workspace_only && !is_in_workspace {
                continue;
            }
            if exclude.contains(dep.krate.name.as_str()) {
                continue;
            }
            let handle1 = nodes[&dep.krate.id];
            let arrow = Arrow::simple("");
            vg.add_edge(arrow, handle0, handle1);
        }
    }

    vg
}

#[derive(Debug, Clone, Default)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Display for Rgba {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{},{}", self.r, self.g, self.b, self.a)
    }
}

impl Rgba {
    fn to_u32(&self) -> u32 {
        rgba(self.r, self.g, self.b, self.a)
    }
}

impl FromStr for Rgba {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',');
        let r = parts
            .next()
            .ok_or_else(|| "missing r".to_string())?
            .parse()
            .map_err(|e| format!("{}", e))?;
        let g = parts
            .next()
            .ok_or_else(|| "missing g".to_string())?
            .parse()
            .map_err(|e| format!("{}", e))?;
        let b = parts
            .next()
            .ok_or_else(|| "missing b".to_string())?
            .parse()
            .map_err(|e| format!("{}", e))?;
        let a = parts
            .next()
            .unwrap_or("255")
            .parse()
            .map_err(|e| format!("{}", e))?;
        Ok(Rgba { r, g, b, a })
    }
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}
