//! Gates da cena `=40` — **o ÁUDIO**.
//!
//! ⚠️ Um A/B tem dois modos de falhar em silêncio: os dois lados iguais (o canal não
//! chegou) **e** os dois lados dirigidos (o "controle" não é controle). Mais um
//! terceiro que só esta cena tem: as barras diferirem e **não se moverem** — o que
//! um campo por-índice qualquer já faria, e que não seria áudio.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// O stream que um sink deixa no instante `t`.
///
/// ⚠️ **Coze pelo `pump.cook` do próprio estado, e isso é load-bearing** — os níveis
/// vêm do shell por CANAL EXTERNO, e o `publish` os escreve NAQUELE cook; um
/// `Cook::new()` local não tem externo nenhum e devolveria bandas zeradas, que é a
/// assinatura exata da feature quebrada.
fn cook(state: &mut MotionState, reg: &NodeRegistry, sink: NodeId, t: f64) -> Stream {
    crate::render_loop::motion_audio_gen::publish(state, t);
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, reg, sink, t)
        .expect("a cena coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e um stream")
    };
    s.clone()
}

fn sizes(s: &Stream) -> Vec<f32> {
    match Stream::get(s, "size") {
        Some(Column::Scalar(v)) => v.clone(),
        Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).collect(),
        _ => Vec::new(),
    }
}

/// **A cena constrói os DOIS lados.** Se um `?` engolisse uma aresta o roteador
/// devolveria `unwrap_or_default()` — uma tela VAZIA, que num smoke lê como *"a
/// feature não foi construída"* em vez de *"a cena está partida"*.
#[test]
fn the_audio_scene_builds_both_rows() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_audio_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), 2, "duas fileiras: controle e dirigida");
}

/// **Só a fileira de baixo respira, e ela MUDA COM O TEMPO.**
///
/// ⚠️ As três metades são precisas: sem a diferença entre barras, um campo constante
/// passaria; sem o CONTROLE, uma cena cujas duas fileiras respirassem passaria; e
/// **sem a comparação entre dois INSTANTES**, um `value.instance_field` qualquer
/// passaria — e essa é a única metade que diz *áudio*.
#[test]
fn only_the_lower_row_breathes_and_it_moves_with_the_playhead() {
    let reg = registry();
    let mut state = MotionState::new();
    let sinks = build_audio_demo_document(&mut state.doc, &reg).expect("a cena constroi");

    let plain = cook(&mut state, &reg, sinks[0], 1.5);
    let a = cook(&mut state, &reg, sinks[1], 1.5);
    let b = cook(&mut state, &reg, sinks[1], 4.5);

    assert_eq!(plain.count(), BANDS, "uma barra por banda");
    assert_eq!(a.count(), BANDS);

    // O CONTROLE, escrito como AUSÊNCIA — e a forma importa.
    //
    // ⚠️ A 1ª versão pedia *"todas as barras com o MESMO tamanho"* e nasceu VERMELHA
    // sobre produto correto: a fileira de cima **não escreve a coluna `size` de
    // todo**, porque quem a escreve é o `motion.drive`, e o renderer usa o tamanho
    // default. *Uma coluna que ninguém escreve não é uma coluna uniforme*, e afirmar
    // uniformidade sobre um vetor vazio seria verdade por VÁCUO na direção oposta.
    let flat = sizes(&plain);
    assert!(
        flat.is_empty() || {
            let (lo, hi) = flat
                .iter()
                .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
            (hi - lo).abs() < 1e-4
        },
        "o controle NAO respira: {flat:?}"
    );

    // Em baixo elas diferem entre si...
    let (sa, sb) = (sizes(&a), sizes(&b));
    let (lo2, hi2) = sa
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    assert!(hi2 - lo2 > 0.2, "as barras diferem entre si: {lo2}..{hi2}");

    // ...e a fileira INTEIRA e' outra num instante diferente. E' esta metade que
    // separa audio de um campo por-indice.
    let moved: f32 = sa
        .iter()
        .zip(&sb)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max);
    assert!(
        moved > 0.2,
        "a fileira muda com o playhead: max delta {moved}"
    );
    eprintln!("[audio] {BANDS} barras, faixa {lo2:.2}..{hi2:.2}, delta no tempo {moved:.2}");
}
