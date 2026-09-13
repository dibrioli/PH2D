//! ⭐⭐⭐ **A CACHE CONTRA O CASCO NÃO MUDA A IMAGEM DE UMA PEÇA COM POSES** (W148) — sob órbita, pan
//! e zoom — **e serve fitas enquanto não a muda**.
//!
//! O `the_cache_never_changes_the_image` mede uma extrusão na pose identidade e um arrasto de órbita.
//! ⚠️ **A cache do casco pergunta a contenção POR FOLHA, no plano de cada perfil**: numa peça sem poses
//! o plano do perfil É o `(x, y)` do mundo, e uma pergunta feita no espaço errado passaria lá sem
//! ninguém ver. ⇒ aqui cada folha tem rotação oblíqua, escala e translação, a raiz também roda, há um
//! torno (que corta só pela caixa), e a câmera faz os três gestos do módulo.
//!
//! # ⛔ Porque é um binário de teste e não um irmão do `src/tests.rs`
//!
//! A mesma razão do `tape_cache_budget`: o `TAPE_HITS` é do **processo**, e um contador global só é
//! legível onde ninguém mais escreve nele. *Uma cache que nunca acerta passa num gate de imagem com
//! nota máxima* — então a metade que conta os acertos não é opcional.

use ph2d_field::{
    Blend, FieldDoc, FillRule, Node, NodeId, NodeKind, Op, Primitive, Profile, Xform,
};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_render::{
    Gbuffer, Orbit, TAPE_HITS, TapeCache, trace_by_rows_for_test, trace_cached_for_test,
};
use std::sync::atomic::Ordering;

fn quat(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let n = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    let s = (angle * 0.5).sin() / n;
    [axis[0] * s, axis[1] * s, axis[2] * s, (angle * 0.5).cos()]
}

fn ring(n: usize, r_in: f32, r_out: f32) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            let r = if i % 2 == 0 { r_out } else { r_in };
            [r * a.cos(), r * a.sin()]
        })
        .collect()
}

fn posed_piece() -> FieldDoc {
    let prof =
        |pts: Vec<[f32; 2]>| Profile::new(vec![pts], FillRule::NonZero, 1e-4).expect("perfil");
    let x = |axis: [f32; 3], angle: f32, scale: f32, translation: [f32; 3]| Xform {
        translation,
        rotation: quat(axis, angle),
        scale,
    };
    let star = Node::new(
        x([1.0, 1.0, 0.0], 0.7, 0.8, [0.1, -0.05, 0.2]),
        NodeKind::Leaf(Primitive::Extrude {
            profile: prof(ring(48, 0.22, 0.5)),
            half_height: 0.2,
            round: 0.03,
            chamfer: 0.0,
        }),
    );
    let poly = Node::new(
        x([0.0, 0.3, 1.0], 1.9, 1.3, [-0.2, 0.1, -0.1]),
        NodeKind::Leaf(Primitive::Polygon {
            profile: prof(ring(24, 0.25, 0.3)),
            half_height: 0.15,
            round: 0.02,
            chamfer: 0.0,
        }),
    );
    let lathe = Node::new(
        x([0.0, 1.0, 0.0], 0.0, 1.0, [0.0, 0.35, 0.0]),
        NodeKind::Leaf(Primitive::Revolve {
            profile: prof(vec![[0.1, -0.15], [0.3, -0.15], [0.3, 0.15], [0.1, 0.15]]),
        }),
    );
    let root = Node::new(
        x([0.0, 1.0, 0.0], 0.4, 0.9, [0.0; 3]),
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: vec![NodeId(0), NodeId(1), NodeId(2)],
        },
    );
    FieldDoc::new(vec![star, poly, lathe, root], NodeId(3)).expect("a peça posta")
}

/// Os três gestos do módulo, com as leis dele (órbita `0,01 rad/px` · pan de `input_law` · zoom
/// `1,1` por passo), seguidos numa sessão só — a cache vê as três transições.
fn session(h: u32) -> Vec<Orbit> {
    let mut cam = Orbit::default();
    let mut out = Vec::new();
    for i in 0..30 {
        out.push(cam);
        match i {
            0..10 => cam.turn_local([0.0, -4.0, 0.0], 4.0 * 0.01),
            10..20 => {
                let k = cam.half_extent / (h as f32 * 0.5);
                let (right, up, _) = cam.basis();
                for c in 0..3 {
                    cam.target[c] += -right[c] * 5.0 * k + up[c] * 2.0 * k;
                }
            }
            _ => cam.half_extent /= 1.1f32.powf(if i < 25 { 0.5 } else { -0.5 }),
        }
    }
    out
}

/// `(pixels com acerto diferente, pior ângulo entre normais)`.
fn disagreement(a: &Gbuffer, b: &Gbuffer) -> (usize, f64) {
    let mut pix = 0usize;
    let mut ang = 0.0f64;
    for k in 0..a.hit.len() {
        if a.hit[k] != b.hit[k] {
            pix += 1;
            continue;
        }
        if !a.hit[k] {
            continue;
        }
        let (x, y) = (a.normal[k], b.normal[k]);
        let d = f64::from(x[0] * y[0] + x[1] * y[1] + x[2] * y[2]);
        ang = ang.max(d.clamp(-1.0, 1.0).acos().to_degrees());
    }
    (pix, ang)
}

#[test]
fn the_hull_cache_never_changes_the_image_of_a_posed_piece() {
    let reg = Registry::new();
    let doc = posed_piece();
    let (w, h) = (200u32, 120u32);
    let cache = TapeCache::new();
    // ⛔ **O gate mede a política que SHIPA**, e diz se não for ela — um `PH2D_FIELD_TAPE_BOX=1`
    // esquecido no ambiente mediria a W82 e passaria.
    assert!(
        cache.uses_hulls(),
        "a cache por omissão não pergunta pelos cascos — este gate mediria a caixa da W82"
    );
    TAPE_HITS.store(0, Ordering::Relaxed);
    let (mut drawn, mut cache_pix, mut cache_ang) = (0usize, 0usize, 0.0f64);
    let (mut ctrl_pix, mut ctrl_ang) = (0usize, 0.0f64);
    for cam in session(h) {
        let com = trace_cached_for_test(&doc, &reg, &cam, w, h, true, Some(&cache));
        let base = trace_cached_for_test(&doc, &reg, &cam, w, h, true, None);
        // ⭐⭐⭐ O CONTROLO: a marcha por LINHA não especializa nada, e a por ladrilho já discorda dela
        // no último bit. A pergunta não é *«a cache muda alguma coisa?»* mas *«muda MAIS do que a
        // especialização já mudava?»* — ver o `the_cache_never_changes_the_image`.
        let linha = trace_by_rows_for_test(&doc, &reg, &cam, w, h);
        drawn += base.hit.iter().filter(|b| **b).count();
        let (p, a) = disagreement(&base, &com);
        cache_pix += p;
        cache_ang = cache_ang.max(a);
        let (p, a) = disagreement(&base, &linha);
        ctrl_pix += p;
        ctrl_ang = ctrl_ang.max(a);
    }
    let hits = TAPE_HITS.load(Ordering::Relaxed);
    println!(
        "desenhados {drawn} · acertos da cache {hits} · cache: {cache_pix} px, {cache_ang:.4}° · \
         controlo: {ctrl_pix} px, {ctrl_ang:.4}°"
    );
    assert!(
        drawn > 20_000,
        "a sessão quase não desenhou a peça ({drawn} pixels)"
    );
    assert!(
        ctrl_ang > 0.0,
        "o controlo não mediu desacordo nenhum — a barra abaixo não estaria a prender nada"
    );
    // ⭐⭐ **A metade que prova que a cache TRABALHOU** — uma cache que nunca acerta passaria as duas
    // asserções de imagem com nota máxima.
    assert!(
        hits > 1_000,
        "a cache só serviu {hits} fitas em 30 quadros — ela deixou de acertar, e a imagem igual não \
         prova nada sobre as fitas que ela serviria"
    );
    assert_eq!(
        cache_pix, 0,
        "{cache_pix} pixels mudaram de acerto por causa da cache numa peça com poses (o controlo \
         mudou {ctrl_pix}) — uma fita foi servida onde o casco dela não contém a região"
    );
    assert!(
        cache_ang <= ctrl_ang,
        "a cache mexeu a normal {cache_ang:.4}° e a especialização sozinha mexe {ctrl_ang:.4}° — uma \
         fita foi servida onde não vale"
    );
}
