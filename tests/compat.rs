//! Value-exactness against networkx 3.6.1.
//!
//! Goldens are hardcoded full-precision constants captured from
//! `networkx.closeness_vitality` (version 3.6.1); graph fixtures live in
//! `tests/golden/*.txt` via `include_str!`. No subprocess or Python is
//! invoked at test time.
//!
//! Convention confirmed from nx source:
//!   wiener_index(G, unweighted) = (Σ_{all ordered pairs} d(u,v)) / 2
//! Removing a node that disconnects the graph → wiener(G−v) = +inf → vitality = −inf.

use rsomics_closeness_vitality::{closeness_vitality, closeness_vitality_node, io, is_connected};

fn graph_from_str(edges: &str) -> io::Graph {
    io::read_edgelist_str(edges).expect("parse edge list")
}

fn check(got: f64, want: f64, label: &str) {
    if want.is_infinite() {
        assert!(
            got.is_infinite() && got.signum() == want.signum(),
            "{label}: got {got}, want {want}"
        );
    } else {
        // Integer-valued vitalities: bit-exact (they come from integer BFS sums).
        assert_eq!(
            got,
            want,
            "{label}: got {got}, want {want} (diff {})",
            (got - want).abs()
        );
    }
}

// ─── Cycle(3) — cycle_graph(3), all vitalities = 2.0 ───────────────────────
const CYCLE3: &str = "0 1\n1 2\n2 0\n";

#[test]
fn cycle3_all() {
    let g = graph_from_str(CYCLE3);
    let cv = closeness_vitality(&g);
    check(cv["0"], 2.0, "cycle3 node 0");
    check(cv["1"], 2.0, "cycle3 node 1");
    check(cv["2"], 2.0, "cycle3 node 2");
}

// ─── Path(5) — endpoints retain 10.0, interior nodes disconnect → −inf ─────
// Wiener(P5) = 20.0 (sum of all unordered pair distances)
// Nodes 1,2,3 are cut vertices; 0 and 4 are leaves.
const PATH5: &str = "0 1\n1 2\n2 3\n3 4\n";

#[test]
fn path5_all() {
    let g = graph_from_str(PATH5);
    let cv = closeness_vitality(&g);
    check(cv["0"], 10.0, "path5 node 0");
    check(cv["1"], f64::NEG_INFINITY, "path5 node 1");
    check(cv["2"], f64::NEG_INFINITY, "path5 node 2");
    check(cv["3"], f64::NEG_INFINITY, "path5 node 3");
    check(cv["4"], 10.0, "path5 node 4");
}

// ─── Cycle(6) — all vitalities = 7.0 ────────────────────────────────────────
// Wiener(C6) = 27.0
const CYCLE6: &str = "0 1\n1 2\n2 3\n3 4\n4 5\n5 0\n";

#[test]
fn cycle6_all() {
    let g = graph_from_str(CYCLE6);
    let cv = closeness_vitality(&g);
    for node in ["0", "1", "2", "3", "4", "5"] {
        check(cv[node], 7.0, &format!("cycle6 node {node}"));
    }
}

// ─── Cut-vertex graph: 0-1, 0-2, 0-3, 1-4, 1-5 ──────────────────────────────
// Wiener(G) = 29.0; nodes 0 and 1 are cut vertices → vitality = −inf.
// Leaves 2,3,4,5 → vitality = 11.0.
const CUT_VERTEX: &str = include_str!("golden/cut_vertex_edges.txt");

#[test]
fn cut_vertex_graph() {
    let g = graph_from_str(CUT_VERTEX);
    let cv = closeness_vitality(&g);
    check(cv["0"], f64::NEG_INFINITY, "cut_vertex node 0");
    check(cv["1"], f64::NEG_INFINITY, "cut_vertex node 1");
    check(cv["2"], 11.0, "cut_vertex node 2");
    check(cv["3"], 11.0, "cut_vertex node 3");
    check(cv["4"], 11.0, "cut_vertex node 4");
    check(cv["5"], 11.0, "cut_vertex node 5");
}

// ─── Single-node query via closeness_vitality_node ──────────────────────────
#[test]
fn single_node_query_cycle3() {
    let g = graph_from_str(CYCLE3);
    let v = closeness_vitality_node(&g, "0").expect("node 0 present");
    check(v, 2.0, "cycle3 single node 0");
}

#[test]
fn single_node_query_absent_returns_none() {
    let g = graph_from_str(CYCLE3);
    assert!(closeness_vitality_node(&g, "99").is_none());
}

// ─── Karate club (networkx 3.6.1 golden values) ─────────────────────────────
// Wiener(karate) = 1351.0
// Node 0 is a cut vertex → −inf; node 33 = −20.0 (removing it actually shortens
// some paths because its removal forces traffic through shorter routes — unusual
// but correct).
const KARATE: &str = include_str!("golden/karate_edges.txt");

// Golden: nx.closeness_vitality(nx.karate_club_graph())
// (node labels relabelled to string integers)
const KARATE_CV: &[(u32, f64)] = &[
    (0, f64::NEG_INFINITY),
    (1, 64.0),
    (2, 19.0),
    (3, 71.0),
    (4, 87.0),
    (5, 85.0),
    (6, 85.0),
    (7, 75.0),
    (8, 64.0),
    (9, 76.0),
    (10, 87.0),
    (11, 90.0),
    (12, 89.0),
    (13, 62.0),
    (14, 89.0),
    (15, 89.0),
    (16, 116.0),
    (17, 88.0),
    (18, 89.0),
    (19, 66.0),
    (20, 89.0),
    (21, 88.0),
    (22, 89.0),
    (23, 83.0),
    (24, 88.0),
    (25, 88.0),
    (26, 91.0),
    (27, 71.0),
    (28, 73.0),
    (29, 86.0),
    (30, 72.0),
    (31, 25.0),
    (32, 52.0),
    (33, -20.0),
];

#[test]
fn karate_all_nodes() {
    let g = graph_from_str(KARATE);
    let cv = closeness_vitality(&g);
    for &(node, want) in KARATE_CV {
        let label = node.to_string();
        let got = cv[&label];
        check(got, want, &format!("karate node {node}"));
    }
}

// ─── gnm(40, 100, seed=7) — connected random graph ─────────────────────────
const GNM40: &str = include_str!("golden/gnm40_edges.txt");

// Golden: nx.closeness_vitality on gnm_random_graph(40, 100, seed=7)
// Wiener(G) = 1839.0
const GNM40_CV: &[(u32, f64)] = &[
    (0, 95.0),
    (1, 88.0),
    (2, 101.0),
    (3, 40.0),
    (4, 23.0),
    (5, 52.0),
    (6, 84.0),
    (7, 77.0),
    (8, 110.0),
    (9, 38.0),
    (10, 79.0),
    (11, 91.0),
    (12, 100.0),
    (13, 54.0),
    (14, 100.0),
    (15, 89.0),
    (16, 77.0),
    (17, 111.0),
    (18, 87.0),
    (19, 75.0),
    (20, 91.0),
    (21, 82.0),
    (22, 97.0),
    (23, 86.0),
    (24, 105.0),
    (25, 86.0),
    (26, 77.0),
    (27, 84.0),
    (28, 83.0),
    (29, 74.0),
    (30, 93.0),
    (31, 63.0),
    (32, 97.0),
    (33, 100.0),
    (34, 63.0),
    (35, 72.0),
    (36, 42.0),
    (37, 70.0),
    (38, 97.0),
    (39, 58.0),
];

#[test]
fn gnm40_all_nodes() {
    let g = graph_from_str(GNM40);
    let cv = closeness_vitality(&g);
    for &(node, want) in GNM40_CV {
        let label = node.to_string();
        let got = cv[&label];
        check(got, want, &format!("gnm40 node {node}"));
    }
}

// ─── gnm(25, 70, seed=42) — connected random graph ─────────────────────────
const GNM25: &str = include_str!("golden/gnm25_edges.txt");

// Golden: nx.closeness_vitality on gnm_random_graph(25, 70, seed=42)
// Wiener(G) = 582.0
const GNM25_CV: &[(u32, f64)] = &[
    (0, 44.0),
    (1, 45.0),
    (2, 29.0),
    (3, 39.0),
    (4, 47.0),
    (5, 41.0),
    (6, 43.0),
    (7, 20.0),
    (8, 31.0),
    (9, 56.0),
    (10, 50.0),
    (11, 52.0),
    (12, 20.0),
    (13, 43.0),
    (14, 40.0),
    (15, 60.0),
    (16, 51.0),
    (17, 44.0),
    (18, 48.0),
    (19, 39.0),
    (20, 30.0),
    (21, 40.0),
    (22, 43.0),
    (23, 44.0),
    (24, 43.0),
];

#[test]
fn gnm25_all_nodes() {
    let g = graph_from_str(GNM25);
    let cv = closeness_vitality(&g);
    for &(node, want) in GNM25_CV {
        let label = node.to_string();
        let got = cv[&label];
        check(got, want, &format!("gnm25 node {node}"));
    }
}

#[test]
fn inline_hash_comment_matches_uncommented() {
    // A `#` starts a comment anywhere in a line (nx.parse_edgelist), so a `#`
    // fused to a token and a whole-line comment must not alter the graph.
    let with_comments = graph_from_str("0 1\n1 2#c\n# whole-line comment\n2 3\n");
    let plain = graph_from_str("0 1\n1 2\n2 3\n");
    assert_eq!(with_comments.labels, plain.labels);
    assert_eq!(with_comments.adj, plain.adj);
}

#[test]
fn connectivity_detection() {
    // A connected graph (path) and a two-component graph. Closeness vitality is
    // only defined on the connected one; the CLI rejects the disconnected case
    // because networkx there collapses to nan/±inf and an edge list cannot carry
    // isolated nodes.
    assert!(is_connected(&graph_from_str("0 1\n1 2\n2 3\n")));
    assert!(!is_connected(&graph_from_str("0 1\n2 3\n")));
}
