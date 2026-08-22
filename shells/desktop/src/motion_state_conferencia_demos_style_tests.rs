//! Gates da cena `=76` (doc 89, folha 14).
//!
//! ⚠️ **Cozem pelo `pump.cook` de um `MotionState` inteiro, e isso é load-bearing.** A
//! geometria de uma forma vem do shell por CANAL EXTERNO; um `Cook::new()` local não tem
//! external nenhum e devolve **zero instâncias** — que é a assinatura exata da feature
//! quebrada. Um harness assim acusaria o produto de um defeito dele próprio (precedente: o
//! harness do texto, e a sonda de movimento da `=75`).

use super::{DASH, LAP_SECS, TRIM_SPAN, build_style_demo_document};
use crate::motion_state::MotionState;
use ph2d_node_motion_shape::param as sp;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::graph::NodeId;

/// Monta a cena num estado real, publica as membranas em `sec` e devolve os sinks.
fn scene(sec: f64) -> (MotionState, Vec<NodeId>) {
    let mut state = MotionState::new();
    let sinks = build_style_demo_document(&mut state.doc, &state.registry).expect("a cena monta");
    crate::render_loop::motion_externals::publish_all(&mut state, sec);
    (state, sinks)
}

/// `(contagem, handle de geometria)` de uma banda, cozinhada em `sec`.
fn band(state: &mut MotionState, sink: NodeId, sec: f64) -> (usize, u32) {
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, sink, sec)
        .expect("a banda coze");
    let s = out[0].as_stream();
    let handle = match s.get("geometry_id") {
        Some(Column::Scalar(ids)) if !ids.is_empty() => ids[0] as u32,
        _ => 0,
    };
    (s.count(), handle)
}

/// **AS SEIS BANDAS DESENHAM** — nenhuma sai vazia nem sem geometria.
///
/// ⚠️ Uma banda vazia é o modo de falha desta família inteira: a chave do shell não encontrou
/// a do nó, o `eval` clonou o external vazio, e a tela fica com o rótulo e sem a forma.
#[test]
fn every_band_draws_a_shape() {
    let (mut state, sinks) = scene(0.0);
    assert_eq!(sinks.len(), 6, "tres linhas x duas colunas");
    for (k, sink) in sinks.iter().enumerate() {
        let (n, handle) = band(&mut state, *sink, 0.0);
        assert_eq!(n, 1, "banda {k}: uma forma");
        assert!(handle >= 1, "banda {k}: sem geometria viva");
    }
}

/// **AS DUAS METADES DE CADA LINHA SÃO FORMAS DIFERENTES** — senão a cena não diz nada.
///
/// ⚠️ O oráculo é o HANDLE de geometria, e não os params autorados: params diferentes que
/// cozessem o mesmo `VecPath` (um `dash` que não chegasse ao `StrokeSpec`, um `trim` neutro
/// por engano) dariam o MESMO handle — que é exactamente o defeito que a cena existe para
/// mostrar, e um gate sobre a tabela de params não o veria.
#[test]
fn the_two_halves_of_every_row_are_different_geometry() {
    let (mut state, sinks) = scene(0.0);
    for k in 0..3 {
        let left = band(&mut state, sinks[k * 2], 0.0).1;
        let right = band(&mut state, sinks[k * 2 + 1], 0.0).1;
        assert_ne!(left, right, "linha {k}: as duas metades sao iguais");
    }
}

/// **O TRECHO ANDA COM O RELÓGIO** — a linha do meio, à direita.
///
/// ⚠️ **É o gate da rota que desaparecia em silêncio.** O `trim_offset` daquela banda é
/// conduzido por um FIO, e até 2026-08-21 um param conduzido fazia o shell publicar a chave
/// do valor estático enquanto o nó lia a do conduzido — contagem 0, forma ausente. Aqui a
/// cena percorre essa rota de propósito, e o gate exige que o resultado MUDE com o relógio:
/// um shell que ignorasse o fio devolveria o mesmo handle nos dois instantes.
#[test]
fn the_revealed_arc_travels_with_the_clock() {
    // ⚠️ **UM estado só, e o oráculo é a GEOMETRIA — não o handle.** Dois estados frescos
    // dão o mesmo handle para a n-ésima geometria internada, e o gate ficaria verde sem
    // medir nada. E mesmo num estado só, o handle diria apenas *"internou outra coisa"*;
    // o que importa é o trecho estar noutro sítio do anel.
    let (mut state, sinks) = scene(0.0);
    let start_of = |state: &mut MotionState, sec: f64| -> [f64; 2] {
        crate::render_loop::motion_externals::publish_all(state, sec);
        let handle = band(state, sinks[3], sec).1;
        let path = state
            .shape_store
            .get(handle)
            .expect("a geometria do trecho");
        let cooked = path.cooked();
        assert!(!cooked.closed, "o trecho e' um contorno ABERTO");
        cooked.verts[0].anchor
    };
    let a = start_of(&mut state, 0.0);
    let b = start_of(&mut state, f64::from(LAP_SECS) * 0.5);
    let moved = (a[0] - b[0]).hypot(a[1] - b[1]);
    assert!(
        moved > 0.5,
        "meia volta depois a ponta do trecho tem de estar do outro lado do anel, e andou \
         {moved:.3} — o fio nao chegou ao publish"
    );
}

/// **NENHUMA BANDA APARA OU PICOTA SEM TRAÇO** — a lei que o `ParamGateAbove` escreve no
/// painel, aqui exigida da CENA.
///
/// ⚠️ Sem traço, aparar um contorno fá-lo DESAPARECER (um trecho aberto não tem interior), e
/// picotar não faz nada. Uma cena que autorasse isso ensinaria o gesto errado com o app a
/// dar-lhe razão.
#[test]
fn no_band_authors_a_trim_or_a_dash_without_a_stroke() {
    let (state, _) = scene(0.0);
    for n in state.doc.graph.nodes() {
        if n.type_name != "source.shape" {
            continue;
        }
        let ov = state.doc.graph.node_param_overrides(n.id);
        let get = |name: &str| ov.and_then(|m| m.get(name).copied()).unwrap_or(f32::NAN);
        let width = get(sp::STROKE_WIDTH);
        let trims = get(sp::TRIM_END).is_finite() || get(sp::TRIM_START).is_finite();
        let dashes = get(sp::DASH).is_finite();
        assert!(
            !(trims || dashes) || width > 0.0,
            "uma forma da cena apara/picota sem traco"
        );
    }
}

/// **OS NÚMEROS DA MENSAGEM SÃO OS DA CENA** — a mensagem do smoke cita `authored()`, e ela
/// tem de continuar a citar o que a cena de facto autorou.
#[test]
fn the_message_quotes_the_scenes_own_numbers() {
    let (span, lap) = super::authored();
    assert!((span - TRIM_SPAN * 100.0).abs() < 1e-3);
    assert!((lap - LAP_SECS).abs() < 1e-6);
    const { assert!(DASH > 0.0, "o picotado tem de ser pedido") }
}
