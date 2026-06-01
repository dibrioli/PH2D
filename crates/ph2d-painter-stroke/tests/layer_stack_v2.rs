//! `.ph2d-painter` v2 layer-stack format (ADR-0046-amendment-1) — migration,
//! validation bounds, and round-trip. Pairs with the type-level discriminant
//! pins in `device.rs` and the byte-order pins in `forward_compat_pins.rs`.

use ph2d_painter_stroke::{
    CanvasInfo, LAYER_FLAG_ACTIVE, LAYER_FLAG_VISIBLE, LayerId, LayerNode, LayerNodeKind,
    LayerStackEntry, LoadError, MAX_GROUP_DEPTH, MAX_LAYER_NAME_BYTES, MAX_LAYERS, PaintProject,
    load, save,
};

fn canvas(w: u32, h: u32) -> CanvasInfo {
    CanvasInfo { width: w, height: h, ..CanvasInfo::default() }
}

fn raster(id: u32, name: &str) -> LayerNode {
    LayerNode {
        id: LayerId(id),
        name: name.to_string(),
        kind: LayerNodeKind::Raster {
            width: 16,
            height: 16,
        },
        blend_mode: 0,
        opacity: 1.0,
        modifiers: LAYER_FLAG_VISIBLE,
        mask: None,
    }
}

/// Forge an on-disk **v1** file: `new` produces v2, so we downgrade to the v1
/// stub shape (`Reserved`) and re-lock the checksum over the v1 bytes.
fn v1_bytes(w: u32, h: u32) -> Vec<u8> {
    let mut p = PaintProject::new(canvas(w, h));
    p.version = 1;
    p.layer_stack.layers = vec![LayerStackEntry::Reserved(Vec::new())];
    p.recompute_checksum();
    postcard::to_allocvec(&p).expect("serialize v1")
}

#[test]
fn load_migrates_v1_to_v2_with_default_layer() {
    let bytes = v1_bytes(100, 80);
    let p = load(&bytes).expect("load + migrate v1");
    assert_eq!(p.version, 2, "v1 file must migrate to v2");
    assert_eq!(
        p.layer_stack.layers.len(),
        1,
        "one default layer for v1 content"
    );
    match &p.layer_stack.layers[0] {
        LayerStackEntry::Node(n) => {
            assert_eq!(
                n.id,
                LayerId(0),
                "default layer id 0 — matches v1 stroke layer_target"
            );
            assert_ne!(
                n.modifiers & LAYER_FLAG_ACTIVE,
                0,
                "default layer is active"
            );
            assert_ne!(
                n.modifiers & LAYER_FLAG_VISIBLE,
                0,
                "default layer is visible"
            );
            assert_eq!(
                n.kind,
                LayerNodeKind::Raster {
                    width: 100,
                    height: 80
                },
                "default raster takes canvas dims",
            );
        }
        other => panic!("expected migrated Node, got {other:?}"),
    }
    // The migrated project re-locks its checksum (re-save → load is clean).
    let resaved = save(&p).expect("re-save migrated");
    assert_eq!(load(&resaved).expect("reload").version, 2);
}

#[test]
fn v2_layer_tree_roundtrips_through_save_load() {
    let mut p = PaintProject::new(canvas(64, 64));
    // bottom raster, then a collapsed group holding one masked raster, active.
    let mut top = raster(2, "Ink");
    top.modifiers |= LAYER_FLAG_ACTIVE;
    top.blend_mode = 11; // HardLight
    top.opacity = 0.7;
    top.mask = Some(Box::new(LayerNode {
        id: LayerId(3),
        name: "Ink mask".to_string(),
        kind: LayerNodeKind::Mask {
            width: 64,
            height: 64,
            inverted: false,
        },
        blend_mode: 0,
        opacity: 1.0,
        modifiers: LAYER_FLAG_VISIBLE,
        mask: None,
    }));
    let group = LayerNode {
        id: LayerId(4),
        name: "Inking".to_string(),
        kind: LayerNodeKind::Group {
            children: vec![top],
            collapsed: true,
        },
        blend_mode: 6, // Screen
        opacity: 0.9,
        modifiers: LAYER_FLAG_VISIBLE,
        mask: None,
    };
    p.layer_stack.layers = vec![
        LayerStackEntry::Node(group),
        LayerStackEntry::Node(raster(1, "Background")),
    ];

    let bytes = save(&p).expect("save");
    let back = load(&bytes).expect("load");
    assert_eq!(
        back.layer_stack, p.layer_stack,
        "layer tree must round-trip losslessly"
    );
    assert_eq!(back.version, 2);
}

#[test]
fn load_rejects_groups_nested_past_max_depth() {
    let mut p = PaintProject::new(canvas(8, 8));
    // MAX_GROUP_DEPTH + 1 nested groups → the deepest trips the depth guard.
    let mut node = LayerNode {
        id: LayerId(0),
        name: "g".to_string(),
        kind: LayerNodeKind::Group {
            children: vec![],
            collapsed: false,
        },
        blend_mode: 0,
        opacity: 1.0,
        modifiers: LAYER_FLAG_VISIBLE,
        mask: None,
    };
    for i in 0..MAX_GROUP_DEPTH {
        node = LayerNode {
            id: LayerId(i as u32 + 1),
            name: "g".to_string(),
            kind: LayerNodeKind::Group {
                children: vec![node],
                collapsed: false,
            },
            blend_mode: 0,
            opacity: 1.0,
            modifiers: LAYER_FLAG_VISIBLE,
            mask: None,
        };
    }
    p.layer_stack.layers = vec![LayerStackEntry::Node(node)];
    p.recompute_checksum();
    let bytes = postcard::to_allocvec(&p).unwrap();
    assert!(
        matches!(
            load(&bytes),
            Err(LoadError::CapExceeded {
                kind: "layer_node.group_depth",
                ..
            })
        ),
        "must reject groups nested past MAX_GROUP_DEPTH"
    );
}

#[test]
fn load_rejects_overlong_layer_name() {
    let mut p = PaintProject::new(canvas(8, 8));
    let mut node = raster(0, "x");
    node.name = "n".repeat(MAX_LAYER_NAME_BYTES + 1);
    p.layer_stack.layers = vec![LayerStackEntry::Node(node)];
    p.recompute_checksum();
    let bytes = postcard::to_allocvec(&p).unwrap();
    assert!(
        matches!(
            load(&bytes),
            Err(LoadError::CapExceeded {
                kind: "layer_node.name",
                ..
            })
        ),
        "must reject an overlong layer name"
    );
}

#[test]
fn load_rejects_more_total_nodes_than_max_layers() {
    let mut p = PaintProject::new(canvas(4, 4));
    // One group with MAX_LAYERS children → total = 1 + MAX_LAYERS > MAX_LAYERS.
    let children: Vec<LayerNode> = (0..MAX_LAYERS as u32).map(|i| raster(i + 1, "c")).collect();
    let group = LayerNode {
        id: LayerId(0),
        name: "root".to_string(),
        kind: LayerNodeKind::Group {
            children,
            collapsed: false,
        },
        blend_mode: 0,
        opacity: 1.0,
        modifiers: LAYER_FLAG_VISIBLE,
        mask: None,
    };
    p.layer_stack.layers = vec![LayerStackEntry::Node(group)];
    p.recompute_checksum();
    let bytes = postcard::to_allocvec(&p).unwrap();
    assert!(
        matches!(
            load(&bytes),
            Err(LoadError::CapExceeded {
                kind: "layer_stack.nodes",
                ..
            })
        ),
        "must count nested nodes toward MAX_LAYERS"
    );
}
