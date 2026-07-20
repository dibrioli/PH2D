//! **O offset roda POR FRAME** (o arrasto do slider re-offseta a cena a cada frame), então
//! nenhuma quina pode custar uma ordem acima das outras.
//!
//! O bug que este gate pina (2026-07-20, *"severa queda de FPS"*): a quina **Round** produz
//! uma banda de ARCOS, e o sweep sobre cúbicas custava **19–43 ms** na rosquinha do smoke
//! contra ~1 ms do Miter/Bevel — o arrasto caía a ~12 fps. O `flat_lines` achata a banda na
//! tolerância RELATIVA da forma antes de todo sweep (a mesma moeda do power stroke), e o
//! Round voltou à mesma ordem dos irmãos.
//!
//! **RAZÃO, não wall-clock**: `ci-test` compila em `opt-level=1` e uma barra absoluta mede
//! o perfil da máquina, não o código ([[feedback_frozen_bar_check_the_arithmetic]]).

use ph2d_vec_boolean::offset_path;
use ph2d_vec_scene::{Contour, FillRule, LineJoin, OffsetSide, VecPath, VecVertex};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex::corner([x, y])
}

/// A rosquinha do smoke (`PH2D_BUILD_SMOKE=17/18`) — a forma em que a queda foi vista.
fn smoke_donut() -> VecPath {
    let hole = 0.7;
    VecPath {
        verts: vec![v(2.8, -1.2), v(5.2, -1.2), v(5.2, 1.2), v(2.8, 1.2)],
        closed: true,
        subpaths: vec![Contour::new_closed(vec![
            v(4.0 - hole, -hole),
            v(4.0 + hole, -hole),
            v(4.0 + hole, hole),
            v(4.0 - hole, hole),
        ])],
        fill_rule: FillRule::EvenOdd,
        ..VecPath::default()
    }
}

/// Mediana de 5 corridas do offset a `d = 1.0` (o fim do slider — o pior caso do arrasto).
fn median_cost(join: LineJoin) -> std::time::Duration {
    let mut runs: Vec<std::time::Duration> = (0..5)
        .map(|_| {
            let src = smoke_donut();
            let t0 = std::time::Instant::now();
            let out = offset_path(&src, 1.0, join, OffsetSide::Both);
            assert!(!out.is_empty(), "o offset da fixture não pode sumir");
            t0.elapsed()
        })
        .collect();
    runs.sort_unstable();
    runs[2]
}

#[test]
fn a_round_live_offset_costs_like_the_other_joins() {
    let round = median_cost(LineJoin::Round);
    // Piso no divisor: numa máquina rápida o Bevel pode medir microssegundos e a razão
    // viraria ruído puro.
    let bevel = median_cost(LineJoin::Bevel).max(std::time::Duration::from_micros(200));
    let ratio = round.as_secs_f64() / bevel.as_secs_f64();
    // Medido pós-fix: ~7× (debug) — os arcos ainda geram mais segmentos, e isso é honesto.
    // O bug dava ~30×. A barra em 15× separa os dois regimes com folga dos dois lados.
    assert!(
        ratio < 15.0,
        "Round custou {ratio:.1}× o Bevel ({round:?} vs {bevel:?}) — o sweep voltou a \
         mastigar CÚBICAS? (o `flat_lines` do `loop_region` achata a banda antes de todo \
         sweep; sem ele o arrasto do slider cai a ~12 fps)"
    );
}
