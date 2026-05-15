//! Generic A* pathfinding over [`GridMath`].
//!
//! # Cost model
//!
//! Edge cost = 1 per step (uniform). Heuristic = `grid.distance` —
//! always admissible for the grid kinds we ship (each grid's
//! `distance` returns the minimum number of edge steps, which is
//! exactly what a uniform-cost A* converges to). Weighted edges
//! (diagonals at √2, terrain cost, etc.) are out of scope for v1;
//! gameplay callers needing them should fork this module.
//!
//! # Determinism (HR-5)
//!
//! `BTreeMap` for `came_from` and `g_score`; tie-breaking in the
//! open set falls back to insertion order via a stable secondary
//! key. Identical inputs produce identical paths.
//!
//! # Allocation
//!
//! Allocates internal `BinaryHeap`, `BTreeMap`s, and a buffer for
//! [`GridMath::neighbors`]. Not zero-alloc; intended for one-off
//! editor calls and game-side path requests batched outside the
//! frame's hot path. A pool-aware variant can be layered on later.

use crate::GridMath;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};

/// Pathfinding result. `cost` is the total step count; `out_path`
/// (populated by [`astar`]) holds cells from `start` to `goal`
/// inclusive.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AStarResult {
    pub cost: u32,
}

/// Open-set entry, ordered by f-score (g + h), then by insertion
/// sequence for deterministic tie-breaking.
struct OpenEntry<C> {
    f: u32,
    seq: u32,
    cell: C,
}

impl<C: Eq> Eq for OpenEntry<C> {}
impl<C: Eq> PartialEq for OpenEntry<C> {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.seq == other.seq
    }
}
impl<C: Eq> Ord for OpenEntry<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is max-heap; reverse so smaller f pops first.
        // Tie-break: smaller seq pops first (insertion-order stable).
        other.f.cmp(&self.f).then(other.seq.cmp(&self.seq))
    }
}
impl<C: Eq> PartialOrd for OpenEntry<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Find the shortest path from `start` to `goal` over `grid`,
/// avoiding cells where `is_blocked` returns `true`. Path is
/// written to `out_path` (cleared first) on success.
///
/// Returns `Some(AStarResult { cost })` on success, `None` if no
/// path exists or `max_iterations` is exceeded.
pub fn astar<G, F>(
    grid: &G,
    start: G::Cell,
    goal: G::Cell,
    is_blocked: F,
    max_iterations: u32,
    out_path: &mut Vec<G::Cell>,
) -> Option<AStarResult>
where
    G: GridMath,
    F: Fn(G::Cell) -> bool,
{
    out_path.clear();
    if is_blocked(start) || is_blocked(goal) {
        return None;
    }
    if start == goal {
        out_path.push(start);
        return Some(AStarResult { cost: 0 });
    }

    let mut open: BinaryHeap<OpenEntry<G::Cell>> = BinaryHeap::new();
    let mut g_score: BTreeMap<G::Cell, u32> = BTreeMap::new();
    let mut came_from: BTreeMap<G::Cell, G::Cell> = BTreeMap::new();
    let mut seq: u32 = 0;

    g_score.insert(start, 0);
    let h_start = grid.distance(start, goal);
    open.push(OpenEntry {
        f: h_start,
        seq,
        cell: start,
    });
    seq += 1;

    let mut nbuf: Vec<G::Cell> = Vec::new();
    let mut iters: u32 = 0;
    while let Some(current) = open.pop() {
        iters += 1;
        if iters > max_iterations {
            return None;
        }
        if current.cell == goal {
            reconstruct_path(&came_from, goal, out_path);
            let cost = g_score[&goal];
            return Some(AStarResult { cost });
        }
        let g_cur = g_score[&current.cell];
        grid.neighbors(current.cell, &mut nbuf);
        for n in &nbuf {
            if is_blocked(*n) {
                continue;
            }
            let tentative_g = g_cur + 1;
            let better = match g_score.get(n) {
                Some(&existing) => tentative_g < existing,
                None => true,
            };
            if better {
                came_from.insert(*n, current.cell);
                g_score.insert(*n, tentative_g);
                let h = grid.distance(*n, goal);
                open.push(OpenEntry {
                    f: tentative_g + h,
                    seq,
                    cell: *n,
                });
                seq += 1;
            }
        }
    }
    None
}

fn reconstruct_path<C: Copy + Ord>(came_from: &BTreeMap<C, C>, goal: C, out: &mut Vec<C>) {
    out.push(goal);
    let mut cur = goal;
    while let Some(&prev) = came_from.get(&cur) {
        out.push(prev);
        cur = prev;
    }
    out.reverse();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::{SquareGrid, SquareNeighborhood};

    #[test]
    fn open_path_cost_matches_manhattan() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut path = Vec::new();
        let r = astar(&g, (0, 0), (3, 4), |_| false, 1_000, &mut path).unwrap();
        // Manhattan distance under Von4.
        assert_eq!(r.cost, 7);
        assert_eq!(path.first(), Some(&(0, 0)));
        assert_eq!(path.last(), Some(&(3, 4)));
        assert_eq!(path.len(), 8); // cost + 1
    }

    #[test]
    fn moore8_uses_chebyshev_so_diagonal_costs_one_step() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Moore8);
        let mut path = Vec::new();
        let r = astar(&g, (0, 0), (3, 3), |_| false, 1_000, &mut path).unwrap();
        // Chebyshev distance is 3 (move diagonally three times).
        assert_eq!(r.cost, 3);
    }

    #[test]
    fn wall_forces_detour() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        // Wall blocks all cells with x = 2 except y = 4.
        let is_blocked = |c: (i32, i32)| c.0 == 2 && c.1 != 4;
        let mut path = Vec::new();
        let r = astar(&g, (0, 0), (4, 0), is_blocked, 10_000, &mut path).unwrap();
        // Manhattan would be 4; detour adds 8 (go up to y=4, across, back down).
        assert_eq!(r.cost, 12);
    }

    #[test]
    fn blocked_goal_returns_none() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut path = Vec::new();
        let r = astar(&g, (0, 0), (3, 3), |c| c == (3, 3), 1_000, &mut path);
        assert!(r.is_none());
    }

    #[test]
    fn start_equals_goal_is_zero_cost() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut path = Vec::new();
        let r = astar(&g, (5, 5), (5, 5), |_| false, 100, &mut path).unwrap();
        assert_eq!(r.cost, 0);
        assert_eq!(path, vec![(5, 5)]);
    }

    #[test]
    fn unreachable_returns_none() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        // Box the goal in completely (3x3 wall around (5, 5) except (5, 5)
        // itself; surrounded on all 4 sides).
        let is_blocked =
            |(x, y): (i32, i32)| (x == 5 && (y == 4 || y == 6)) || (y == 5 && (x == 4 || x == 6));
        let mut path = Vec::new();
        let r = astar(&g, (0, 0), (5, 5), is_blocked, 100_000, &mut path);
        assert!(r.is_none(), "goal is unreachable, expected None");
    }

    #[test]
    fn max_iterations_aborts() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut path = Vec::new();
        // Cap iterations very low; expect None.
        let r = astar(&g, (0, 0), (100, 100), |_| false, 5, &mut path);
        assert!(r.is_none());
    }

    #[test]
    fn deterministic_path_for_identical_inputs() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Moore8);
        let mut p1 = Vec::new();
        let mut p2 = Vec::new();
        astar(&g, (0, 0), (5, 5), |_| false, 1_000, &mut p1).unwrap();
        astar(&g, (0, 0), (5, 5), |_| false, 1_000, &mut p2).unwrap();
        assert_eq!(p1, p2);
    }
}
