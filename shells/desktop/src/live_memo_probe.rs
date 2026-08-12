//! SONDA (`--ignored`): **quanto custa mover uma forma que tem geometria VIVA?**
//!
//! # A pergunta
//!
//! Todo produtor de `LiveGeometry` memoiza, e a §11 do plano 25 afirma que **a chave é o
//! MUNDO — que é exatamente o que a animação move**. Se a afirmação valer, então arrastar (ou
//! animar) uma forma com Contour/Offset/Perfil/Simetria re-cozinha o efeito **em todo quadro**,
//! e o memo protege só a cena PARADA.
//!
//! ⚠️ **A sonda mede pela porta do PRODUTO** (`recook`, o que o frame chama), nunca por um laço
//! próprio sobre o kernel: esta casa já pagou três vezes por medir uma peça isolada e chamar o
//! número de produto (doc 28 §5.40 do Painter, e a decomposição do `build_flow_field`).
//!
//! # As duas colunas, e por que a comparação é honesta
//!
//! - **PARADO** — `recook` N vezes com a MESMA pose. É o que o memo promete proteger.
//! - **ANIMADO** — `recook` N vezes com a pose a TRANSLADAR. É o que a timeline faz a 60 Hz.
//!
//! Uma translação **não muda a forma do efeito**: o contorno de uma estrela deslocada é o
//! contorno da estrela, deslocado. Então toda diferença entre as duas colunas é trabalho que a
//! resposta não precisou.
//!
//! Rode: `cargo test -p ph2d-host-desktop --release live_memo -- --ignored --nocapture`

use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecContour, VecOffset, VecPathRef};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VecXforms};

/// Quantos quadros cada coluna mede. Sessenta é um segundo de animação a 60 Hz.
const FRAMES: usize = 60;

/// Uma estrela de 5 pontas — a fixture CARA do `probe_contour_cost` (quinas reentrantes), e a
/// única em que o offset tem trabalho de verdade a fazer.
fn star() -> VecPath {
    let mut pts = Vec::new();
    for i in 0..10 {
        let a = std::f64::consts::PI * 2.0 * f64::from(i) / 10.0 - std::f64::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { 1.0 } else { 0.42 };
        pts.push([a.cos() * r, a.sin() * r]);
    }
    VecPath {
        verts: pts.into_iter().map(VecVertex::corner).collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// A cena mínima: uma estrela posada, com entidade e nome.
fn posed_star() -> (VecScene, SimWorld, VecEntityMap, VecPathId, Entity) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(star());
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Star"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    (scene, sim, map, id, e)
}

/// Empurra a pose de `e` para `x` e devolve os `VecXforms` que o frame publicaria.
fn pose_at(sim: &mut SimWorld, map: &VecEntityMap, e: Entity, x: f32) -> VecXforms {
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e)
        && let Some(mut t) = em.get_mut::<Transform>()
    {
        t.translation = ph2d_core::Vec2::new(x, 0.0);
    }
    crate::vec_transform::build(sim, map)
}

/// Roda `frames` quadros e devolve o custo MEDIANO por quadro, em ms.
///
/// ⚠️ **Mediana, não mínimo:** o primeiro quadro de cada coluna é sempre um miss (o memo nasce
/// vazio), e o mínimo seria exactamente a amostra SEM o fenômeno — a lição de fixture que o
/// Painter pagou no gate de razão do Wet Paint (doc 28 §5.12).
fn median_ms(frames: usize, mut f: impl FnMut(usize)) -> f64 {
    let mut ms = Vec::with_capacity(frames);
    for i in 0..frames {
        let t0 = std::time::Instant::now();
        f(i);
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    ms.sort_by(f64::total_cmp);
    ms[ms.len() / 2]
}

/// A translação do quadro `i` — 0,01 unidade por quadro, o que um arrasto lento faz.
#[allow(clippy::cast_precision_loss)]
fn drift(i: usize) -> f32 {
    i as f32 * 0.01
}

#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn live_memo_probe() {
    println!("[sonda] {FRAMES} quadros por coluna; mediana em ms/quadro");
    println!(
        "[sonda] {:<12} {:>10} {:>10} {:>8} {:>6}",
        "produtor", "PARADO", "ANIMADO", "razao", "saida"
    );

    probe_contour();
    probe_offset();
    probe_profile();
    probe_symmetry();
}

/// ⚠️ **O CONTROLE da sonda.** Um produtor que não produziu nada mede o custo de um `continue`,
/// e a coluna ANIMADO sairia tranquilizadoramente baixa — a mesma armadilha de fixture que o
/// Painter pagou no `build_flow_field` (a máscara `active` vazia fazia todo passe sair pela
/// porta de trás, e a soma casava com o passo *por coincidência*).
fn require(name: &str, produced: usize) -> usize {
    assert!(
        produced > 0,
        "a fixture de {name} nao produziu geometria viva — a sonda estaria a medir um `continue`"
    );
    produced
}

/// ⚠️ **O número sai com o ESCOPO ao lado.** Um custo sem a contagem que ele produziu é
/// inatribuível: `0,04 ms` sobre dezasseis anéis e `0,04 ms` sobre um caminho degenerado são
/// leituras opostas, e só a coluna `saída` as separa.
fn report(name: &str, still: f64, moving: f64, produced: usize) {
    let ratio = if still > 0.0 {
        moving / still
    } else {
        f64::INFINITY
    };
    let flag = if moving > 16.6 {
        "  <= NAO CABE NUM QUADRO DE 60 Hz"
    } else {
        ""
    };
    println!("[sonda] {name:<12} {still:>10.3} {moving:>10.3} {ratio:>7.1}x {produced:>6}{flag}");
}

fn probe_contour() {
    let (scene, mut sim, map, id, e) = posed_star();
    let ent = Entity::from_bits(map[&id]);
    let _ = ent;
    sim.world_mut().entity_mut(e).insert(VecContour {
        steps: 16,
        d: 0.30,
        join: 1, // Round: a quina cara
        to: [255, 255, 255, 255],
        ..VecContour::default()
    });

    let mut live = crate::contour_live::ContourLive::default();
    let xf = pose_at(&mut sim, &map, e, 0.0);
    let still = median_ms(FRAMES, |_| live.recook(&scene, &sim, &map, &xf));

    let mut live = crate::contour_live::ContourLive::default();
    let mut xfs = Vec::with_capacity(FRAMES);
    for i in 0..FRAMES {
        xfs.push(pose_at(&mut sim, &map, e, drift(i)));
    }
    let moving = median_ms(FRAMES, |i| live.recook(&scene, &sim, &map, &xfs[i]));
    let produced = require("contour", live.live().values().map(Vec::len).sum());
    report("contour", still, moving, produced);
}

fn probe_offset() {
    let (scene, mut sim, map, id, e) = posed_star();
    let _ = id;
    sim.world_mut().entity_mut(e).insert(VecOffset {
        d: 0.12,
        join: 1,
        side: 0,
    });

    let mut live = crate::offset_live::OffsetLive::default();
    let xf = pose_at(&mut sim, &map, e, 0.0);
    let still = median_ms(FRAMES, |_| live.recook(&scene, &sim, &map, &xf));

    let mut live = crate::offset_live::OffsetLive::default();
    let mut xfs = Vec::with_capacity(FRAMES);
    for i in 0..FRAMES {
        xfs.push(pose_at(&mut sim, &map, e, drift(i)));
    }
    let moving = median_ms(FRAMES, |i| live.recook(&scene, &sim, &map, &xfs[i]));
    let produced = require("offset", live.live().values().map(Vec::len).sum());
    report("offset", still, moving, produced);
}

fn probe_profile() {
    let (mut scene, mut sim, map, id, e) = posed_star();
    // O perfil só produz fita se a forma TIVER traço — a fixture tem de conter o fenômeno.
    if let Some(p) = scene.paths_mut().iter_mut().find(|p| p.id == id) {
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            0.08,
        ));
    }
    // ⚠️ `VecStrokeProfile::default()` tem `stops` VAZIO — o neutro que a shell REMOVE em vez de
    // guardar. Armá-lo é armar a ausência do efeito; o perfil precisa de paradas de verdade.
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_ecs::VecStrokeProfile {
            stops: ph2d_vec_scene::WidthProfile {
                start: 0.2,
                mid: 1.8,
                end: 0.2,
                position: 0.5,
            }
            .to_stops(),
        });

    let mut live = crate::profile_live::ProfileLive::default();
    let xf = pose_at(&mut sim, &map, e, 0.0);
    let still = median_ms(FRAMES, |_| live.recook(&scene, &sim, &map, &xf));

    let mut live = crate::profile_live::ProfileLive::default();
    let mut xfs = Vec::with_capacity(FRAMES);
    for i in 0..FRAMES {
        xfs.push(pose_at(&mut sim, &map, e, drift(i)));
    }
    let moving = median_ms(FRAMES, |i| live.recook(&scene, &sim, &map, &xfs[i]));
    let produced = require("profile", live.live().values().map(Vec::len).sum());
    report("profile", still, moving, produced);
}

fn probe_symmetry() {
    let (scene, mut sim, map, id, e) = posed_star();
    let _ = id;
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_ecs::VecSymmetry::default());

    let mut live = crate::symmetry_live::SymmetryLive::default();
    let xf = pose_at(&mut sim, &map, e, 0.0);
    let still = median_ms(FRAMES, |_| live.recook(&scene, &sim, &map, &xf, true));

    let mut live = crate::symmetry_live::SymmetryLive::default();
    let mut xfs = Vec::with_capacity(FRAMES);
    for i in 0..FRAMES {
        xfs.push(pose_at(&mut sim, &map, e, drift(i)));
    }
    let moving = median_ms(FRAMES, |i| live.recook(&scene, &sim, &map, &xfs[i], true));
    let produced = require("symmetry", live.live().values().map(Vec::len).sum());
    report("symmetry", still, moving, produced);
}

// ---------------------------------------------------------------------------------------------
// SONDA 2 — a PREMISSA da cura, medida antes de qualquer wave.
// ---------------------------------------------------------------------------------------------

/// SONDA (`--ignored`): **o cozimento COMUTA com uma similaridade?**
///
/// A cura que a §11 do plano 25 nomeia — *memoizar em espaço LOCAL e assar o afim na saída* —
/// vale a pena exactamente na medida em que `offset(assar(P, X), d)` e `assar(offset(P, d/s), X)`
/// desenham a MESMA curva. Se comutarem, o memo passa a acertar sob animação (a geometria local
/// não muda; só o afim muda) e o preço medido pela sonda irmã (0,686 ms no offset · 1,655 no
/// perfil, por forma e por quadro) vai a zero mais o custo de assar.
///
/// ⚠️ **A premissa é MEDIDA, nunca deduzida.** Escrever a wave e descobrir a não-comutação no
/// smoke seria pagar o preço inteiro para aprender um fato que uma sonda de trinta linhas dá.
///
/// ⚠️ **E ela tem CONTROLE:** a última linha é uma pose de escala NÃO-UNIFORME, que **não pode**
/// comutar — um offset é uma distância euclidiana, e sob `diag(2, 0.5)` a distância deixa de ter
/// um único fator. Se o controle também sair `0,000`, a sonda não está a medir comutação nenhuma
/// e o verde é vácuo.
///
/// Rode: `cargo test -p ph2d-host-desktop --release live_memo_commutation -- --ignored --nocapture`
#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn live_memo_commutation_probe() {
    use ph2d_vec_scene::bake_xform;

    const D: f64 = 0.12;
    let join = crate::vec_expand::join_of_code(1);
    let side = crate::vec_expand::side_of_code(0);

    println!("[comuta] offset d = {D}; desvio maximo de vertice entre as duas rotas");
    println!("[comuta] {:<22} {:>12} {:>8}", "pose", "desvio", "escala");

    for (label, x) in poses() {
        let s = x.mean_scale();

        // A rota de HOJE: assa a pose na fonte e coze em MUNDO.
        let mut world = star();
        bake_xform(&mut world, &x);
        let by_world = ph2d_vec_boolean::offset_path(&world, D, join, side);

        // A rota da CURA: coze em LOCAL com a distância dividida pela escala, e assa a saída.
        let mut by_local = ph2d_vec_boolean::offset_path(&star(), D / s, join, side);
        for p in &mut by_local {
            bake_xform(p, &x);
        }

        let dev = max_vertex_deviation(&by_world, &by_local);
        println!("[comuta] {label:<22} {dev:>12.6} {s:>8.3}");
    }
}

/// SONDA (`--ignored`): **e o PERFIL comuta?** — a metade CARA da tabela (1,655 ms/forma/quadro).
///
/// ⚠️ **A pergunta não é a mesma do offset, e a diferença decide o escopo da wave:** o `bake_xform`
/// escala *todo comprimento escalar do path* (o raio do gradiente, o `corner_radius`) e **não**
/// escala o `StrokeSpec.width`. Então uma forma com pose de escala 2× desenha o traço com a
/// **mesma** largura de mundo — e a rota local, que assa DEPOIS, escalaria a fita junto. A
/// pergunta medida é se dividir a largura pela escala antes de cozer devolve a comutação, que é
/// o gêmeo exato do `d_local = d / s` do offset.
#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn live_memo_commutation_profile_probe() {
    use ph2d_vec_scene::{Rgba8, StrokeSpec, bake_xform};

    const W: f64 = 0.08;
    let stops = ph2d_vec_scene::WidthProfile {
        start: 0.2,
        mid: 1.8,
        end: 0.2,
        position: 0.5,
    }
    .to_stops();

    let stroked = || {
        let mut p = star();
        p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), W));
        p
    };

    println!("[comuta] perfil (0,2/1,8/0,2), largura base {W}");
    println!(
        "[comuta] {:<22} {:>12} {:>12} {:>8}",
        "pose", "largura crua", "largura / s", "escala"
    );

    for (label, x) in poses() {
        let s = x.mean_scale();

        let mut world = stroked();
        bake_xform(&mut world, &x);
        let by_world = crate::vec_expand::power_stroke_layers(&world, &stops);

        // (a) a rota local INGENUA: coze com a largura autorada e assa a saida.
        let naive = {
            let mut out = crate::vec_expand::power_stroke_layers(&stroked(), &stops);
            for p in &mut out {
                bake_xform(p, &x);
            }
            max_vertex_deviation(&by_world, &out)
        };

        // (b) a rota local com a largura DIVIDIDA pela escala -- o gemeo do `d / s`.
        let scaled = {
            let mut local = stroked();
            if let Some(sp) = local.stroke.as_mut() {
                sp.width = W / s;
            }
            let mut out = crate::vec_expand::power_stroke_layers(&local, &stops);
            for p in &mut out {
                bake_xform(p, &x);
            }
            max_vertex_deviation(&by_world, &out)
        };

        println!("[comuta] {label:<22} {naive:>12.6} {scaled:>12.6} {s:>8.3}");
    }
}

/// As poses da sonda — quatro similaridades e **um controle** que não é uma.
fn poses() -> Vec<(&'static str, ph2d_vec_scene::Xform)> {
    use ph2d_vec_scene::Xform;
    let (sin, cos) = 0.7_f64.sin_cos();
    let s = 1.6;
    vec![
        ("translacao", Xform([1.0, 0.0, 0.0, 1.0, 3.0, -1.5])),
        ("rotacao", Xform([cos, sin, -sin, cos, 0.0, 0.0])),
        ("escala uniforme 1,6", Xform([s, 0.0, 0.0, s, 0.0, 0.0])),
        (
            "similaridade completa",
            Xform([s * cos, s * sin, -s * sin, s * cos, 3.0, -1.5]),
        ),
        // O CONTROLE: escala NAO-uniforme. Aqui a comutacao tem de FALHAR.
        (
            "nao-uniforme (controle)",
            Xform([2.0, 0.0, 0.0, 0.5, 0.0, 0.0]),
        ),
    ]
}

/// A distância de Hausdorff (discreta, sobre âncoras) entre as duas saídas — **quão longe uma
/// curva está da outra**, e não quão longe o vértice `i` está do vértice `i`.
///
/// ⚠️ **A 1ª versão desta função comparava ÍNDICE a ÍNDICE, e mediu a coisa errada com confiança.**
/// Um contorno fechado é uma sequência **CÍCLICA**, e o `linesweeper` escolhe onde começar a
/// partir das COORDENADAS — então uma pose que gira a forma re-elege o primeiro vértice e o
/// alinhamento por índice reporta o deslocamento da LISTA como desvio geométrico. Medido, ela
/// dizia `1,25` numa forma de raio 1 sob rotação pura, e `0,000` sob translação e sob escala
/// uniforme — exactamente as duas poses que **preservam** a ordem lexicográfica. Eu quase concluí
/// *"a cura não comuta sob rotação"* a partir de um defeito do meu oráculo.
///
/// *Um oráculo tem de modelar a APARÊNCIA — a curva —, nunca a representação que a carrega.*
fn max_vertex_deviation(a: &[VecPath], b: &[VecPath]) -> f64 {
    if a.len() != b.len() {
        return f64::INFINITY;
    }
    a.iter()
        .zip(b)
        .map(|(pa, pb)| one_sided(pa, pb).max(one_sided(pb, pa)))
        .fold(0.0_f64, f64::max)
}

/// Para cada âncora de `from`, a distância à âncora MAIS PRÓXIMA de `to`; devolve a pior.
fn one_sided(from: &VecPath, to: &VecPath) -> f64 {
    if to.verts.is_empty() {
        return f64::INFINITY;
    }
    from.verts
        .iter()
        .map(|v| {
            to.verts
                .iter()
                .map(|w| (v.anchor[0] - w.anchor[0]).hypot(v.anchor[1] - w.anchor[1]))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0_f64, f64::max)
}
