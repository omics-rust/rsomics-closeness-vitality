pub mod io;

use std::collections::HashMap;

use io::Graph;

/// Wiener index of `g`: sum of shortest-path distances over all UNORDERED pairs
/// (u, v) with u < v, i.e. each pair counted once.
///
/// Matches `nx.wiener_index(G, weight=None)` on undirected graphs:
///   total = Σ_{u,v ordered} d(u,v)  then  return total / 2
///
/// Returns `f64::INFINITY` if the graph is disconnected (any pair unreachable).
fn wiener_index(g: &Graph) -> f64 {
    let n = g.n();
    if n <= 1 {
        return 0.0;
    }
    let mut total: u64 = 0;
    let mut dist = vec![u32::MAX; n];
    let mut queue = std::collections::VecDeque::with_capacity(n);

    for src in 0..n {
        // BFS from src
        dist.iter_mut().for_each(|d| *d = u32::MAX);
        dist[src] = 0;
        queue.clear();
        queue.push_back(src as u32);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[u as usize];
            for &v in &g.adj[u as usize] {
                if dist[v as usize] == u32::MAX {
                    dist[v as usize] = d_u + 1;
                    queue.push_back(v);
                }
            }
        }
        // Sum distances to all nodes reachable from src (including itself = 0)
        for d in &dist {
            if *d == u32::MAX {
                return f64::INFINITY;
            }
            total += *d as u64;
        }
    }
    // total is the sum over ALL ordered pairs; divide by 2 for unordered pairs
    (total / 2) as f64
}

/// Whether every node of `g` is reachable from node 0 (a single connected
/// component). An empty graph is trivially connected.
///
/// Closeness vitality is only well defined on a connected graph: on a
/// disconnected graph `wiener_index(G)` is infinite, so networkx's per-node
/// `wiener(G) − wiener(G − v)` collapses to a mix of `nan`/`±inf`. An edge list
/// also cannot carry isolated nodes, so a disconnected input can never be
/// reconstructed faithfully — the caller bails rather than emit a plausible
/// wrong value.
pub fn is_connected(g: &Graph) -> bool {
    let n = g.n();
    if n <= 1 {
        return true;
    }
    let mut seen = vec![false; n];
    let mut queue = std::collections::VecDeque::with_capacity(n);
    seen[0] = true;
    queue.push_back(0u32);
    let mut count = 1usize;
    while let Some(u) = queue.pop_front() {
        for &v in &g.adj[u as usize] {
            if !seen[v as usize] {
                seen[v as usize] = true;
                count += 1;
                queue.push_back(v);
            }
        }
    }
    count == n
}

/// Closeness vitality of every node in `g`.
///
/// vitality(v) = wiener_index(G) − wiener_index(G − v)
///
/// Matches `nx.closeness_vitality(G)` exactly for connected unweighted graphs:
/// - Values are exact integers stored as f64 (all distances are integers).
/// - If removing v disconnects G, wiener(G − v) = +inf, so vitality(v) = −inf.
///
/// The caller is expected to reject a disconnected `g` (see [`is_connected`])
/// before calling this; on a disconnected graph the results are not
/// reconstructible from an edge list.
///
/// Returns a `HashMap<String, f64>` keyed by node label, matching nx's dict.
pub fn closeness_vitality(g: &Graph) -> HashMap<String, f64> {
    let w_g = wiener_index(g);
    let n = g.n();
    let mut result = HashMap::with_capacity(n);

    // Reuse buffers across nodes for BFS on the subgraph G − v.
    let mut dist = vec![u32::MAX; n];
    let mut queue = std::collections::VecDeque::with_capacity(n);

    for v in 0..n {
        let w_after = wiener_subgraph_minus(g, v, &mut dist, &mut queue);
        let vitality = w_g - w_after;
        result.insert(g.labels[v].clone(), vitality);
    }
    result
}

/// Closeness vitality for a single node `v` (by label).
///
/// Returns `None` if the label is not in the graph.
pub fn closeness_vitality_node(g: &Graph, label: &str) -> Option<f64> {
    let v = *g.index.get(label)?;
    let w_g = wiener_index(g);
    let mut dist = vec![u32::MAX; g.n()];
    let mut queue = std::collections::VecDeque::with_capacity(g.n());
    let w_after = wiener_subgraph_minus(g, v as usize, &mut dist, &mut queue);
    Some(w_g - w_after)
}

/// Wiener index of G − v (the subgraph induced by all nodes except `v`).
///
/// BFS skips node `v` entirely. Returns +inf if the subgraph is disconnected.
fn wiener_subgraph_minus(
    g: &Graph,
    v: usize,
    dist: &mut [u32],
    queue: &mut std::collections::VecDeque<u32>,
) -> f64 {
    let n = g.n();

    if n - 1 <= 1 {
        return 0.0;
    }

    let mut total: u64 = 0;

    for src in 0..n {
        if src == v {
            continue;
        }
        // BFS from src in G − v
        dist.iter_mut().for_each(|d| *d = u32::MAX);
        dist[src] = 0;
        dist[v] = 0; // sentinel: treat v as already-seen so BFS never visits it
        queue.clear();
        queue.push_back(src as u32);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[u as usize];
            for &w in &g.adj[u as usize] {
                if dist[w as usize] == u32::MAX {
                    dist[w as usize] = d_u + 1;
                    queue.push_back(w);
                }
            }
        }
        for (u, &d) in dist.iter().enumerate() {
            if u == v {
                continue;
            }
            if d == u32::MAX {
                return f64::INFINITY;
            }
            total += d as u64;
        }
    }
    (total / 2) as f64
}
