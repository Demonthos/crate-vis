use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use krates::Krates;
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::*;
use layout::std_shapes::shapes::*;
use layout::topo::layout::VisualGraph;

#[derive(Default)]
pub struct VisualizationCfg {
    pub workspace_color: Rgba,
    pub duplicate_color: Rgba,
    pub exclude: HashSet<String>,
    pub only_workspace: bool,
    pub targets: Vec<String>,
    pub manifest_path: PathBuf,
    pub output: PathBuf,
    pub features: Vec<String>,
    pub all_features: bool,
}

impl VisualizationCfg {
    fn should_include(&self, name: &str, in_workspace: bool) -> bool {
        !self.exclude.contains(name) && (!self.only_workspace || in_workspace)
    }
}

#[test]
fn run() {
    let args = VisualizationCfg::default();

    generate_graph(args).unwrap();
}

pub fn generate_graph(cfg: VisualizationCfg) -> Result<(), krates::Error> {
    use krates::{cm, Builder, Cmd};

    let mut cmd = Cmd::new();
    cmd.manifest_path(cfg.manifest_path.clone());
    cmd.features(cfg.features.clone());
    if cfg.all_features {
        cmd.all_features();
    }

    let mut builder = Builder::new();

    builder.include_targets(cfg.targets.iter().map(|s| (s.as_str(), Default::default())));

    let krates: Krates = builder.build(cmd, |_: cm::Package| {})?;

    let mut vg = krates_to_graph_vis(krates, &cfg);

    let mut svg = layout::backends::svg::SVGWriter::new();
    vg.do_it(false, false, false, &mut svg);

    let _ = layout::core::utils::save_to_file(&cfg.output.to_string_lossy(), &svg.finalize());

    Ok(())
}

fn krates_to_graph_vis(krates: Krates, cfg: &VisualizationCfg) -> VisualGraph {
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
            look.fill_color = Some(layout::core::color::Color::new(
                cfg.duplicate_color.to_u32(),
            ));
        }
        let is_in_workspace = in_workspace.contains(&krate.id);
        if is_in_workspace {
            look.fill_color = Some(layout::core::color::Color::new(
                cfg.workspace_color.to_u32(),
            ));
        }
        let node = Element::create(sp, look, Orientation::TopToBottom, sz);

        if !cfg.should_include(name.as_str(), is_in_workspace) {
            continue;
        }
        let handle = vg.add_node(node);
        nodes.insert(krate.id.clone(), handle);
    }

    for krate in krates.krates() {
        let is_in_workspace = in_workspace.contains(&krate.id);
        if !cfg.should_include(krate.name.as_str(), is_in_workspace) {
            continue;
        }

        let id = krates.nid_for_kid(&krate.id).unwrap();

        let handle0 = nodes[&krate.id];
        for dep in krates.direct_dependencies(id) {
            let is_in_workspace = in_workspace.contains(&dep.krate.id);
            if !cfg.should_include(dep.krate.name.as_str(), is_in_workspace) {
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
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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
