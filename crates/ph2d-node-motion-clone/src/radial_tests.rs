//! **O LEQUE, pelo `eval`** (doc 89 folha 04 — a célula *Setor* do `motion.kaleidoscope`).
//!
//! ⚠️ Os gates da aritmética vivem no [`super::super::radial`]; estes correm a costura — o
//! `mode` sai de um `ctx.param`, o passo sai da CONTAGEM já orçamentada, e o `copy_rank` entra
//! nos dois modos pelo mesmo sítio. Uma lei certa com um `eval` que não a chama é a causa nº 1
//! da semana perdida no Painter.

use super::*;
use crate::radial::{MODE_LINEAR, MODE_RADIAL};
use crate::tests::{Ops, clone_p};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph};

/// O ângulo de um ponto em torno da origem, em graus `[0, 360)`.
fn bearing(p: [f32; 2]) -> f32 {
    let d = p[1].atan2(p[0]).to_degrees();
    if d < 0.0 { d + 360.0 } else { d }
}

/// O raio de um ponto em torno de `pivot`.
fn radius(p: [f32; 2], pivot: [f32; 2]) -> f32 {
    (p[0] - pivot[0]).hypot(p[1] - pivot[1])
}

/// ⚠️ **A trig desta casa é a parábola corrigida do HR-5** (~0,09% fora da trig verdadeira),
/// e estes gates medem com o `atan2` REAL da `std`. Meio grau é a folga que essa diferença
/// pede — apertá-la mediria a aproximação, não o leque.
const DEG_EPS: f32 = 0.5;

/// **O default é a FILA que sempre shipou, AO BIT** — e declarar o modo explicitamente dá o
/// mesmo, que é o que prova que o param novo não desviou o caminho antigo.
#[test]
fn the_default_mode_is_the_row_that_shipped() {
    let implicit = clone_p(|_, _| {});
    let explicit = clone_p(|g, n| g.set_param(n, MODE, MODE_LINEAR as f32));
    assert_eq!(implicit, vec![[0.0, 0.0], [2.0, 0.0], [4.0, 0.0]]);
    for (i, (a, b)) in implicit.iter().zip(&explicit).enumerate() {
        assert_eq!(
            (a[0].to_bits(), a[1].to_bits()),
            (b[0].to_bits(), b[1].to_bits()),
            "copia {i}: {a:?} contra {b:?}"
        );
    }
}

/// ⭐ **O DEFEITO E A CURA.** A peça está POUSADA no pivô — o gesto canónico (*"põe oito
/// destas à volta de um círculo"*). Em `Linear` ela vira uma fila; em `Radial` vira um anel,
/// e é o `distance`-como-RAIO que o torna possível (uma lei só de rotação daria seis cópias
/// coincidentes, porque o raio dela é zero).
#[test]
fn a_piece_sitting_on_the_pivot_becomes_a_ring() {
    let k = 6;
    let p = clone_p(|g, n| {
        g.set_param(n, MODE, MODE_RADIAL as f32);
        g.set_param(n, "count", k as f32);
        g.set_param(n, "distance", 2.0);
    });
    assert_eq!(p.len(), k, "uma copia por fatia");
    for (i, q) in p.iter().enumerate() {
        assert!(
            (radius(*q, [0.0, 0.0]) - 2.0).abs() < 0.02,
            "copia {i} fora do raio: {q:?}"
        );
    }
    // E os rumos são as `k` fatias de uma volta, em ordem.
    for (i, q) in p.iter().enumerate() {
        let want = 360.0 / k as f32 * i as f32;
        assert!(
            (bearing(*q) - want).abs() < DEG_EPS,
            "copia {i} devia estar em {want} graus e esta' em {:.2}",
            bearing(*q)
        );
    }
}

/// ⭐⭐ **O SETOR — a célula, literalmente.** Com `arc = 90` as quatro cópias repartem a
/// cunha em vez de darem a volta.
///
/// ⚠️ **A régua é o RUMO da última cópia, e ela distingue as duas leis possíveis:** com o
/// passo `arc/k` (a nossa) a última fica a `67,5°`; com `arc/(k−1)` ficaria a `90°` — e essa
/// segunda lei é a que poria a última cópia **em cima da primeira** num giro completo.
#[test]
fn a_partial_arc_makes_a_fan_and_the_step_divides_by_the_count() {
    let k = 4;
    let p = clone_p(|g, n| {
        g.set_param(n, MODE, MODE_RADIAL as f32);
        g.set_param(n, "count", k as f32);
        g.set_param(n, "distance", 2.0);
        g.set_param(n, ARC, 90.0);
    });
    let last = bearing(p[k - 1]);
    assert!(
        (last - 67.5).abs() < DEG_EPS,
        "a ultima copia de um setor de 90 com 4 fatias fica a 67,5 graus, e ficou a {last:.2}"
    );
    // E as quatro estão em fatias iguais de 22,5°.
    for (i, q) in p.iter().enumerate() {
        let want = 22.5 * i as f32;
        assert!(
            (bearing(*q) - want).abs() < DEG_EPS,
            "copia {i}: {want} contra {:.2}",
            bearing(*q)
        );
    }
}

/// **Um setor negativo vira o leque para o outro lado** — sai de graça, e não há caso
/// degenerado a guardar.
#[test]
fn a_negative_sector_fans_the_other_way() {
    let p = clone_p(|g, n| {
        g.set_param(n, MODE, MODE_RADIAL as f32);
        g.set_param(n, "count", 4.0);
        g.set_param(n, "distance", 2.0);
        g.set_param(n, ARC, -90.0);
    });
    let last = bearing(p[3]);
    assert!(
        (last - (360.0 - 67.5)).abs() < DEG_EPS,
        "o leque invertido acaba a -67,5 graus, e acabou a {last:.2}"
    );
}

/// **O pivô é onde o leque gira, e o raio do anel é o da PEÇA + o empurrão.**
///
/// ⚠️ **A primeira versão deste gate exigia raio `distance` e reprovava código correto.** Com
/// a peça fora do pivô o anel não tem raio `distance`: a colocação é
/// `pivot + R(θ)·(v + (distance, 0))` — *empurra em +X, depois gira* —, então o raio é
/// `|v + (distance, 0)|`, o mesmo para todas as cópias porque uma rotação preserva
/// comprimento. O gate certo é **a igualdade dos raios**, que é a afirmação de que aquilo é um
/// anel; exigir um número que só vale quando `v = 0` era medir a fixtura.
#[test]
fn the_pivot_is_where_the_fan_turns_and_every_copy_shares_one_radius() {
    let piv = [3.0_f32, -1.0];
    let (d, k) = (1.5_f32, 5);
    let p = clone_p(|g, n| {
        g.set_param(n, MODE, MODE_RADIAL as f32);
        g.set_param(n, "count", k as f32);
        g.set_param(n, "distance", d);
        g.set_param(n, "pivot_x", piv[0]);
        g.set_param(n, "pivot_y", piv[1]);
    });
    // A fonte está na ORIGEM ⇒ `v = −pivot`, e o raio é `|v + (d, 0)|`.
    let want = (-piv[0] + d).hypot(-piv[1]);
    for (i, q) in p.iter().enumerate() {
        assert!(
            (radius(*q, piv) - want).abs() < 0.02,
            "copia {i} fora do anel de {want:.4}: {q:?} (r={:.4})",
            radius(*q, piv)
        );
    }
    // CONTROLE: o anel é em torno do PIVÔ e não da origem — as cópias não partilham raio ali.
    let spread = p
        .iter()
        .map(|q| radius(*q, [0.0, 0.0]))
        .fold((f32::MAX, f32::MIN), |(lo, hi), r| (lo.min(r), hi.max(r)));
    assert!(
        spread.1 - spread.0 > 0.5,
        "CONTROLE: em torno da origem os raios tinham de DIFERIR ({:.3}..{:.3})",
        spread.0,
        spread.1
    );
}

/// **`angle` aponta o padrão, nos DOIS modos** — em `Linear` a direção da fila, em `Radial`
/// onde a primeira cópia começa. É a mesma pergunta, e é o que o mantém fora do gate de
/// visibilidade.
#[test]
fn the_angle_points_the_pattern_in_the_fan_too() {
    let p = clone_p(|g, n| {
        g.set_param(n, MODE, MODE_RADIAL as f32);
        g.set_param(n, "count", 4.0);
        g.set_param(n, "distance", 2.0);
        g.set_param(n, "angle", 90.0);
    });
    assert!(
        (bearing(p[0]) - 90.0).abs() < DEG_EPS,
        "a primeira copia tinha de comecar a 90 graus, e comecou a {:.2}",
        bearing(p[0])
    );
}

/// **`center` LADEIA o leque, como ladeia a fila** — os dois controles continuam ortogonais.
#[test]
fn center_straddles_the_fan_just_as_it_straddles_the_row() {
    let p = clone_p(|g, n| {
        g.set_param(n, MODE, MODE_RADIAL as f32);
        g.set_param(n, "count", 4.0);
        g.set_param(n, "distance", 2.0);
        g.set_param(n, ARC, 120.0);
        g.set_param(n, "center", 1.0);
    });
    // Postos −1,5 · −0,5 · 0,5 · 1,5 vezes 30° ⇒ −45 · −15 · 15 · 45, simétricos em torno de 0.
    let first = bearing(p[0]) - 360.0;
    let last = bearing(p[3]);
    assert!(
        (first + last).abs() < DEG_EPS,
        "o leque tinha de ficar simetrico em torno do original: {first:.2} e {last:.2}"
    );
}

/// ⚠️ **O leque NÃO escreve `rot`, e o precedente é o irmão.** O `motion.kaleidoscope` — o
/// replicador rotacional que já existia — toca `P`, `Index` e `Count` e nada mais (medido).
/// A roseta em que cada peça olha para fora é `motion.look_at(pivot)` a jusante, um nó que
/// existe exactamente para isso.
#[test]
fn the_fan_never_writes_the_rotation_column() {
    let mut g = Graph::new();
    let src = g.add_node("motion.clone.test.src");
    let clone = g.add_node("motion.clone");
    g.connect(Edge {
        from: (src, 0),
        to: (clone, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(clone, MODE, MODE_RADIAL as f32);
    g.set_param(clone, "count", 5.0);
    g.set_param(clone, "distance", 2.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, clone, 0.0).unwrap();
    let s = out[0].as_stream();
    assert!(
        s.get("rot").is_none(),
        "o leque cunhou uma coluna `rot` que a fonte nao tinha"
    );
    // CONTROLE: ele fez alguma coisa (senão o gate acima passaria com o nó morto).
    assert_eq!(s.count(), 5, "cinco copias");
}

/// **Os três controles do leque só aparecem no modo que os lê** — e o `distance`, que vive
/// nos dois, fica de fora do gate.
#[test]
fn the_fan_controls_are_hidden_in_the_row_mode() {
    let gated: Vec<&str> = PARAM_GATES.iter().map(|g| g.param).collect();
    assert_eq!(gated, vec![ARC, "pivot_x", "pivot_y"], "exatamente os tres");
    for g in PARAM_GATES {
        assert_eq!(g.when, MODE, "todos decididos pelo modo");
        assert_eq!(g.values, &[MODE_RADIAL], "e visiveis so' no leque");
    }
    assert!(
        !gated.contains(&"distance"),
        "o `distance` e' o passo num modo e o RAIO no outro -- escondê-lo mataria o knob \
         mais usado do no' no modo em que ele mais se usa"
    );
    // E todo param do manifesto tem hint: um knob sem painel é inalcançável.
    for p in [MODE, ARC, "pivot_x", "pivot_y"] {
        assert!(
            MANIFEST.params.iter().any(|s| s.name == p),
            "`{p}` fora do manifesto"
        );
        assert!(
            PARAM_HINTS.iter().any(|h| h.param == p),
            "`{p}` sem hint de painel"
        );
    }
}
