//! The Motion vertical, end to end and HEADLESS (W2 proof): a real graph
//! `motion.grid → motion.transform → motion.clone`, cooked through the real
//! `NodeRegistry`, validated, and lowered to `RenderInstance`s — all without a
//! GPU. The visual smoke (uploading these instances and seeing them on screen
//! via `./play.command`) is the human step (W2.T3); this proves the data path.

use ph2d_eval_motion::{evaluate_motion, lower_to_instances};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_transform::register(&mut reg).unwrap();
    ph2d_node_motion_clone::register(&mut reg).unwrap();
    reg
}

#[test]
fn grid_transform_clone_lowers_to_render_instances() {
    let reg = registry();

    // grid (3×3 = 9) → transform (identity defaults) → clone (×3, step_x 2).
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let xf = g.add_node("motion.transform");
    let clone = g.add_node("motion.clone");
    g.connect(Edge {
        from: (grid, 0),
        to: (xf, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (xf, 0),
        to: (clone, 0),
        delayed: false,
    })
    .unwrap();

    // The graph is well-typed (all Instances/Vec2/Frame; all Pure → no membrane
    // violation). validate proves it before cooking.
    g.validate(&reg).expect("motion graph is well-typed");

    let mut cook = Cook::new();
    let instances = evaluate_motion(&mut cook, &g, &reg, clone, 0.0).unwrap();

    // 9 grid points × 3 clone copies = 27 render instances.
    assert_eq!(instances.len(), 27);

    // copy 0 = the grid itself (transform is identity at defaults):
    //   grid[0] = (0,0), grid[8] = (2,2).
    assert_eq!(instances[0].world_pos, [0.0, 0.0]);
    assert_eq!(instances[8].world_pos, [2.0, 2.0]);
    // copy 1 = grid + (step_x=2, 0): instances[9] = grid[0] + (2,0).
    assert_eq!(instances[9].world_pos, [2.0, 0.0]);
    // copy 2 = grid + (4, 0): instances[18] = grid[0] + (4,0).
    assert_eq!(instances[18].world_pos, [4.0, 0.0]);
    // defaults stamped by the lowering:
    assert_eq!(instances[0].size, [1.0, 1.0]);
    assert_eq!(instances[0].tint, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn incremental_recook_is_stable() {
    // Cooking the same static graph twice (same Cook) yields identical output —
    // the memoized cook path is stable for a combinational motion graph. This
    // graph is purely combinational (no `pre` edge), so `advance_tick` snapshots
    // nothing; the point is that re-cooking a memoized graph reproduces its
    // output exactly (the live-preview re-cook every frame must not drift).
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let clone = g.add_node("motion.clone");
    g.connect(Edge {
        from: (grid, 0),
        to: (clone, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let a = evaluate_motion(&mut cook, &g, &reg, clone, 0.0).unwrap();
    cook.advance_tick(&g, &reg, 0.0).unwrap();
    let b = evaluate_motion(&mut cook, &g, &reg, clone, 0.0).unwrap();
    assert_eq!(a.len(), b.len());
    assert_eq!(
        a.first().map(|i| i.world_pos),
        b.first().map(|i| i.world_pos)
    );
}

#[test]
fn lower_to_instances_is_public_api() {
    // sanity: the lowering is reusable on its own.
    use ph2d_nodegraph::attr::{Column, Stream};
    let s = Stream::new(1).with("P", Column::Vec2(vec![[7.0, 8.0]]));
    assert_eq!(lower_to_instances(&s)[0].world_pos, [7.0, 8.0]);
}
