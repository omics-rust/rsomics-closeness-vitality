use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use serde::Serialize;

use rsomics_common::{run, CommonFlags, Result, RsomicsError, ToolMeta};

use rsomics_closeness_vitality::{closeness_vitality, closeness_vitality_node, io, is_connected};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

/// Closeness vitality of nodes in an undirected, unweighted graph.
///
/// Reads an edge list (one `u v` per line) from FILE or stdin (`-`). Comment
/// lines (`#`) and blank lines are ignored. Self-loops are dropped; duplicate
/// edges collapse to a simple graph (`nx.Graph` semantics).
///
/// Closeness vitality of node v = Wiener(G) − Wiener(G − v), where the Wiener
/// index is the sum of shortest-path distances over all unordered node pairs.
/// If removing v disconnects the graph, its vitality is −inf. Value-exact to
/// networkx 3.6.1 `nx.closeness_vitality`.
///
/// Output: one `label value` per line sorted by label (all nodes), or a single
/// value with `--node LABEL`. Exact-integer vitalities are printed without
/// decimals; −inf/inf are printed as `-inf`/`inf`.
#[derive(Parser, Debug)]
#[command(name = "rsomics-closeness-vitality", version, about, long_about = None)]
pub struct Cli {
    /// Only compute and print the vitality for this node label.
    #[arg(long = "node", value_name = "LABEL")]
    pub node: Option<String>,

    /// Edge list file (`-` or omitted reads stdin).
    #[arg(value_name = "EDGELIST")]
    pub edgelist: Option<PathBuf>,

    #[command(flatten)]
    pub common: CommonFlags,
}

struct Out {
    /// Vitality entries: single (label, value) for `--node`, all nodes sorted otherwise.
    values: Vec<(String, f64)>,
}

fn fmt_vitality(v: f64) -> String {
    if v.is_infinite() {
        if v < 0.0 {
            "-inf".to_owned()
        } else {
            "inf".to_owned()
        }
    } else {
        // Connected unweighted graphs always yield exact integers; no decimal needed.
        format!("{}", v as i64)
    }
}

impl Cli {
    pub fn run(self) -> ExitCode {
        let common = self.common.clone();
        run(&common, META, || self.execute(&common))
    }

    fn execute(self, common: &CommonFlags) -> Result<Out> {
        let g = io::read_edgelist(self.edgelist.as_deref())?;

        if !is_connected(&g) {
            return Err(RsomicsError::InvalidInput(
                "graph is disconnected; closeness vitality is defined only for a connected graph \
                 (networkx yields nan/±inf, and an edge list cannot represent isolated nodes)"
                    .to_owned(),
            ));
        }

        let values: Vec<(String, f64)> = if let Some(ref label) = self.node {
            let v = closeness_vitality_node(&g, label).ok_or_else(|| {
                RsomicsError::InvalidInput(format!("node {label:?} is not present in the graph"))
            })?;
            vec![(label.clone(), v)]
        } else {
            let mut map: Vec<(String, f64)> = closeness_vitality(&g).into_iter().collect();
            map.sort_by(|a, b| a.0.cmp(&b.0));
            map
        };

        if !common.json {
            for (label, v) in &values {
                println!("{label} {}", fmt_vitality(*v));
            }
        }

        Ok(Out { values })
    }
}

impl Serialize for Out {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut m = serializer.serialize_map(Some(self.values.len()))?;
        for (label, v) in &self.values {
            if v.is_infinite() {
                m.serialize_entry(label, &fmt_vitality(*v))?;
            } else {
                m.serialize_entry(label, v)?;
            }
        }
        m.end()
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        super::Cli::command().debug_assert();
    }
}
