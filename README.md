A small library for creating a visualization of the crate dependency graph.

## Usage

```rust
use crate_vis::*;

let cfg = VisualizationCfg{
    workspace_only: true,
    manifest_path: "path/to/Cargo.toml".into(),
    ..Default::default()
};

generate_graph(&cfg).unwrap();
```

![graph](cli/dependency_graph.svg)
