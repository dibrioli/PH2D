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
    assert_eq!(sinks.len(), 4, "duas sombras, o glow e o vagalume");
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
    let b = bands();
    let t = tints(&b[2]);
    let peak = t
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
