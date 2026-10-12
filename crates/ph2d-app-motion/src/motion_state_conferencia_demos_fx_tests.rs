//! Os gates da cena `=70` — a família `fx.*`.
//!
//! ⚠️ **O que estes gates NÃO podem medir**: o glow é um passe de RENDER, então
//! nenhum oráculo de CPU vê o halo. O que a cena tem de provar aqui é o que o COOK
//! entrega ao passe — a fonte emissiva existe, o vagalume estoura, e o nó do glow
//! carrega os knobs. O halo em si é do olho do Enio, e é para isso que a cena serve.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn bands() -> Vec<Stream> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_fx_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 5, "duas sombras, o glow, o vagalume e a forma");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|s| {
            cook.cook(&doc.graph, &reg, *s, 0.0).expect("coze")[0]
                .as_stream()
                .clone()
        })
        .collect()
}

fn tints(st: &Stream) -> Vec<[f32; 4]> {
    match st.get("tint") {
        Some(Column::Vec4(v)) => v.clone(),
        _ => panic!("tint"),
    }
}

/// **A SOMBRA MACIA EMITE UM DISCO, e a DURA continua a emitir um fantasma.**
///
/// ⚠️ A contagem é o oráculo honesto aqui: o disco é geometria, e é o que o olho
/// vai ver como penumbra. `6` peças × (16 taps + 1) contra `6` × 2.
#[test]
fn the_soft_band_emits_a_disc_and_the_hard_one_a_single_ghost() {
    let b = bands();
    assert_eq!(b[0].count(), 6 * 2, "dura: um fantasma por peça");
    assert_eq!(b[1].count(), 6 * 17, "macia: dezasseis taps por peça");
}

/// **A DENSIDADE DO MIOLO É A MESMA NOS DOIS LADOS** — é isto que faz o par ser um
/// par, e não *"a da direita ficou mais clara"*.
///
/// ⚠️ O oráculo é a UNIÃO: `1 − Π(1 − aᵢ)` sobre os taps de UMA peça tem de dar o
/// alfa do fantasma duro. Um gate que comparasse os alfas individuais estaria a
/// medir a coisa errada e ficaria vermelho sobre produto correcto.
#[test]
fn the_two_shadows_carry_the_same_ink() {
    let b = bands();
    let hard = tints(&b[0])[0][3];
    let soft = tints(&b[1]);
    // Os fantasmas vêm em BLOCOS por tap; a peça 0 é a primeira de cada bloco.
    let n = 6usize;
    let union = (0..16).fold(1.0f32, |acc, k| acc * (1.0 - soft[k * n][3]));
    assert!(
        ((1.0 - union) - hard).abs() < 1e-4,
        "a união dos taps deu {:.5} e o fantasma duro vale {hard:.5}",
        1.0 - union
    );
}

/// **A FONTE DO GLOW É HDR** — sem `tint > 1` não há bloom nenhum para esticar.
///
/// ⚠️ Este é o gate que impede a cena de mentir: com o `threshold` em `1,0`, uma
/// banda LDR passaria pelo bright-pass sem contribuir, e o `Anamorphic` pareceria
/// quebrado quando o quebrado seria o cenário.
#[test]
fn the_glow_source_is_actually_above_white() {
    // ⚠️ As estrelas são `source.shape`: a geometria é publicada pela shell
    // (`motion_shape_gen::publish`), então coze-se como o app coze, não pelo `Cook` cru.
    let mut m = crate::motion_state::MotionState::new();
    let sinks = build_fx_demo_document(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    let st = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sinks[2], 0.0)
        .expect("coze")[0]
        .as_stream()
        .clone();
    assert_eq!(
        st.count(),
        4,
        "uma estrela em cada uma das quatro peças do vórtice"
    );
    let peak = tints(&st)
        .iter()
        .map(|c| c[0].max(c[1]).max(c[2]))
        .fold(0.0, f32::max);
    assert!(
        peak > 1.0,
        "a fonte tem de ser emissiva, e o pico deu {peak}"
    );
}

/// **O VAGALUME ESTOURA DE VERDADE** — o `Clamp` tem o que curar.
///
/// ⚠️ Um knob sem alvo é um knob morto. O número tem de ser grande o bastante para
/// o efeito ser visível ao olho, não só maior que `1`.
#[test]
fn the_firefly_is_far_above_the_threshold() {
    let b = bands();
    let (_, _, firefly) = authored();
    let t = tints(&b[3]);
    assert_eq!(t.len(), 1, "uma peça só — é o ponto da banda");
    assert!(
        (t[0][0] - firefly).abs() < 1e-3,
        "o vagalume tem de chegar cru ao passe: {:?}",
        t[0]
    );
    assert!(firefly > 20.0, "e tem de lavar a tela sem teto: {firefly}");
}

/// **HÁ EXACTAMENTE UM `fx.glow` NA CENA, e ele leva os knobs autorados.**
///
/// ⚠️ **Dois seriam um bug de cena**, não uma comparação: o `from_graph` lê o
/// PRIMEIRO e o segundo fica inerte (é a célula que fechou em 19/08, e o app avisa).
/// Este gate é o que impede alguém de «melhorar» a cena pondo o par que as irmãs têm.
#[test]
fn the_scene_authors_exactly_one_glow_and_it_carries_the_knobs() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    build_fx_demo_document(&mut doc, &reg).expect("a cena monta");
    let glows = doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "fx.glow")
        .count();
    assert_eq!(glows, 1, "um segundo glow seria silenciosamente inerte");
    let g = ph2d_node_fx_glow::from_graph(&doc.graph).expect("o nó existe");
    let (_, stretch, _) = authored();
    assert!(
        (g.stretch - stretch).abs() < 1e-6,
        "stretch = {}",
        g.stretch
    );
    assert_eq!(
        g.clamp, 0.0,
        "o teto começa DESLIGADO — é o que o smoke liga"
    );
    assert!(g.intensity > 1.0, "e o halo tem de ser visível de saída");
}

/// ⭐ **AS ESTRELAS RODAM, E FICAM NA TELA** (smoke do dono, 11/10: *«as estrelas estão
/// paradas»* — o vórtice nascia no centro do MUNDO, longe delas, e nenhuma força as alcançava).
///
/// Simula 10 s: no 1.º segundo já andaram; no último ainda andam; nunca saem de 2 unidades do
/// centro (o vórtice sozinho atira-as para fora — é o atrator e o arrasto que as seguram).
#[test]
fn the_stars_orbit_and_stay_on_screen() {
    let c = super::CENTRO_DAS_ESTRELAS;
    let poses = poses_das_estrelas();
    let anda = |a: &[[f32; 2]], b: &[[f32; 2]]| {
        a.iter()
            .zip(b)
            .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
            .fold(0.0, f32::max)
    };
    let longe = poses
        .iter()
        .flatten()
        .map(|p| (p[0] - c[0]).hypot(p[1] - c[1]))
        .fold(0.0, f32::max);
    eprintln!(
        "estrelas: 1.o s {:.3} · ultimo s {:.3} · mais longe {longe:.3}",
        anda(&poses[0], &poses[60]),
        anda(&poses[540], &poses[599])
    );
    assert_eq!(poses.len(), 600);
    assert!(anda(&poses[0], &poses[60]) > 0.1, "paradas no 1.o segundo");
    assert!(
        anda(&poses[540], &poses[599]) > 0.1,
        "pararam antes dos 10 s"
    );
    assert!(longe < 2.0, "fugiram do centro: {longe}");
}

/// A coluna `P` das estrelas (a banda 3) em 10 s a 60 Hz, pelo cozimento da CPU.
fn poses_das_estrelas() -> Vec<Vec<[f32; 2]>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    build_fx_demo_document(&mut doc, &reg).expect("a cena monta");
    let integ = doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.integrate")
        .map(|n| n.id)
        .expect("a banda 3 integra");
    let mut cook = Cook::new();
    let mut poses: Vec<Vec<[f32; 2]>> = Vec::new();
    for k in 0..600 {
        let t = f64::from(k) / 60.0;
        let out = cook.cook(&doc.graph, &reg, integ, t).expect("coze");
        if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
            poses.push(p.clone());
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
    }
    poses
}

/// ⭐ **AS ESTRELAS GIRAM EM VOLTA DO CENTRO DELAS** (doc 131 §1). O gate de cima prova que
/// andam e não fogem — e o defeito original (o vórtice no centro do MUNDO) passava nele se
/// alguma força as empurrasse. Este mede o GIRO: por estrela, o ângulo desenrolado em volta de
/// `CENTRO_DAS_ESTRELAS` ao fim de 10 s, e o raio.
#[test]
fn as_estrelas_da_70_giram_em_volta_do_centro_delas() {
    let c = super::CENTRO_DAS_ESTRELAS;
    let poses = poses_das_estrelas();
    let n = poses[0].len();
    let mut giros = Vec::with_capacity(n);
    for i in 0..n {
        let (mut total, mut rmin, mut rmax) = (0.0f32, f32::INFINITY, 0.0f32);
        let ang = |p: [f32; 2]| (p[1] - c[1]).atan2(p[0] - c[0]);
        for w in poses.windows(2) {
            let mut d = ang(w[1][i]) - ang(w[0][i]);
            if d > std::f32::consts::PI {
                d -= std::f32::consts::TAU;
            } else if d < -std::f32::consts::PI {
                d += std::f32::consts::TAU;
            }
            total += d;
            let r = (w[1][i][0] - c[0]).hypot(w[1][i][1] - c[1]);
            rmin = rmin.min(r);
            rmax = rmax.max(r);
        }
        giros.push((total / std::f32::consts::TAU, rmin, rmax));
    }
    for (i, (v, rmin, rmax)) in giros.iter().enumerate() {
        eprintln!("estrela {i}: {v:+.3} voltas · raio {rmin:.3}..{rmax:.3}");
    }
    // Medido (12/10, a cena curada): as 4 estrelas dão −2,47 a −2,76 voltas em 10 s, raio
    // 0,33..1,70. ⇒ «gira» = pelo menos UMA volta, todas no MESMO sentido, sem cair no centro.
    let sentido = giros[0].0.signum();
    for (i, &(v, rmin, _)) in giros.iter().enumerate() {
        assert!(v.abs() >= 1.0, "a estrela {i} não dá uma volta: {v:+.3}");
        assert_eq!(v.signum(), sentido, "a estrela {i} gira ao contrário");
        assert!(rmin > 0.1, "a estrela {i} caiu no centro: raio {rmin:.3}");
    }
}
