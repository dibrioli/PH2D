//! SONDA (`--ignored`): o Contour da cena `=25` (a estrela do Enio) produz anéis SÃOS ou
//! geometria à deriva? Reproduz o que o `cook_piece` do shell faz — `offset_ring` no sinal da
//! distância, com fallback no `offset_path` — sobre a estrela EXATA da cena e inspeciona cada anel.
//!
//! Rodar: `cargo test -p ph2d-vec-boolean --test probe_contour_demo_star -- --ignored --nocapture`

use ph2d_vec_scene::{LineJoin, OffsetSide, ShapeKind, VecPath, cook};

/// A estrela pelada da cena `=25` (`contour_smoke.rs`): `ShapeKind::Star`, bbox `[-3.6,-1.4]` a
/// `[-0.8,1.4]`, args `[5 pontas, 0.45 interno, 0 rotação]`.
fn demo_star() -> VecPath {
    cook(
        ShapeKind::Star,
        [-3.6, -1.4],
        [-0.8, 1.4],
        &[5.0, 0.45, 0.0],
    )
}

/// bbox das ÂNCORAS de um caminho (primário + subpaths), e a maior coordenada absoluta.
fn stats(p: &VecPath) -> (usize, [f64; 4], f64) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let (mut n, mut worst) = (0usize, 0.0f64);
    let mut scan = |v: &[ph2d_vec_scene::VecVertex]| {
        for vv in v {
            n += 1;
            for k in 0..2 {
                lo[k] = lo[k].min(vv.anchor[k]);
                hi[k] = hi[k].max(vv.anchor[k]);
                worst = worst.max(vv.anchor[k].abs());
                worst = worst.max(vv.in_handle[k].abs());
                worst = worst.max(vv.out_handle[k].abs());
            }
        }
    };
    scan(&p.verts);
    for c in &p.subpaths {
        scan(&c.verts);
    }
    (n, [lo[0], lo[1], hi[0], hi[1]], worst)
}

/// Reproduz o `cook_piece`: `offset_ring` (grow-only), senão `offset_path`.
fn cook_piece(world: &VecPath, dist: f64, join: LineJoin) -> Option<Vec<VecPath>> {
    match ph2d_vec_boolean::offset_ring(world, dist, join, OffsetSide::Outer) {
        Some(g) => Some(g),
        None => {
            let g = ph2d_vec_boolean::offset_path(world, dist, join, OffsetSide::Outer);
            if g.is_empty() { None } else { Some(g) }
        }
    }
}

#[test]
#[ignore = "sonda visual"]
fn probe_contour_demo_star() {
    let star = demo_star();
    let (sn, sb, _) = stats(&star);
    println!(
        "\nESTRELA fonte: {sn} verts, bbox [{:.2},{:.2}]..[{:.2},{:.2}] (largura ~2.8)",
        sb[0], sb[1], sb[2], sb[3]
    );

    // A cena =25: DEMO_STEPS=6, DEMO_D=0.12, accel=1 (linear ⇒ k·d). Side default = 0 (Outer ⇒
    // distância POSITIVA), join default = 1 (Round). Testo Round E Miter (o roteiro avisa do
    // buraco do Miter/Bevel).
    for (jname, join) in [("Round", LineJoin::Round), ("Miter", LineJoin::Miter)] {
        // E os DOIS sinais: Outer (grow, +) e Inner (shrink, −) — o modelo de direção que eu mexi.
        for (sname, sign) in [("Outer +", 1.0), ("Inner -", -1.0)] {
            println!("\n=== join {jname}, side {sname} ===");
            for k in 1..=6u16 {
                let dist = sign * f64::from(k) * 0.12;
                match cook_piece(&star, dist, join) {
                    Some(g) if g.is_empty() => {
                        println!("  anel {k} (d={dist:+.2}): VAZIO (aniquilado/buraco do sweep)");
                    }
                    Some(g) => {
                        for (i, ring) in g.iter().enumerate() {
                            let (n, b, worst) = stats(ring);
                            // Um anel são cerca a fonte (~[-3.6,-0.8]×[-1.4,1.4]); worst ~4.
                            // "À DERIVA" = uma coord absurda (>50) ou bbox gigante ⇒ handle/ponto
                            // solto = exatamente as "alças bugadas + linhas retas" da foto.
                            let bbox_w = b[2] - b[0];
                            let drift = worst > 50.0 || bbox_w > 20.0 || !worst.is_finite();
                            println!(
                                "  anel {k}.{i} (d={dist:+.2}): {n} verts, bbox \
                                 [{:.2},{:.2}]..[{:.2},{:.2}], |max|={worst:.2}{}",
                                b[0],
                                b[1],
                                b[2],
                                b[3],
                                if drift { "  <<<<< A DERIVA" } else { "" }
                            );
                        }
                    }
                    None => println!("  anel {k} (d={dist:+.2}): None (booleana falhou)"),
                }
            }
        }
    }
}
