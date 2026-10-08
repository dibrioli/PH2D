//! ⭐⭐⭐ **A CONTAGEM DE CÓPIAS CHEGA À ROSÁCEA** — o gate do controlo morto `Vector › Symmetry ›
//! Segments` (caça de 2026-08-30).
//!
//! # ⛔ Por que os gates que já existiam ficaram VERDES sobre ele
//!
//! O id aparecia em quatro sítios — o `ids`, o `populate_symmetry`, o `paint_symmetry` (hoje `symmetry_segments` + a fila ②, D10) e o censo
//! `the_painted_control_reaches_a_consumer` — e **os quatro eram declaração, registo ou pintura**.
//! O ponteiro alcançava a barra, o chip mudava de número, e não havia braço de evento nenhum: a
//! contagem que a rosácea desenha ficava pregada no `6` do default para sempre.
//!
//! ⚠️ **Nenhuma sonda deste repo pergunta se o VALOR chega a um consumidor.** O censo de
//! focalizabilidade mede se o widget existe para o dispatcher; os `seam_*` medem se o clique chega
//! à ferramenta. Este mede a **GEOMETRIA**: quantas formas o kernel radial emite depois do gesto.
//!
//! # O oráculo não é a minha própria conversão
//!
//! Duas construções comparadas uma com a outra são cegas a uma mutação partilhada. Aqui as duas
//! metades vêm de sítios **independentes**:
//!
//! - as pontas do curso valem `MIN_SEGMENTS` / `MAX_SEGMENTS`, que são o **vocabulário**
//!   ([`ph2d_symmetry`]), e não um número que este ficheiro escolheu;
//! - no meio do curso o oráculo é **o número que o artista LÊ no chip** — ele viaja pelo espelho
//!   afim do `editor-core`, e a contagem viaja pelo painel → barramento → tool → `SymmetrySpec` →
//!   kernel. *O defeito que o artista vê é exactamente as duas a discordarem.*

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_panel_vector::VectorPanel;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_symmetry::{MAX_SEGMENTS, MIN_SEGMENTS, SymmetryKind, SymmetryStyle};
use ph2d_tool_vector::{VectorStyleSnapshot, VectorTool};
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vec_scene::symmetry::{SymmetrySpec, symmetry_paths};
use ph2d_vec_scene::{VecPath, VecVertex};

/// Publica o que a shell publicaria: simetria LIGADA em **Radial** — sem isto a fileira de
/// Segments nem é pintada (o `Enable` gateia a secção, e o Radial é o único tipo que a mostra).
fn publica_radial() {
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        symmetry: SymmetryStyle {
            on: true,
            kind: SymmetryKind::Radial,
            ..SymmetryStyle::default()
        },
        ..VectorStyleSnapshot::default()
    }));
}

/// Uma pétala qualquer, fora do centro — o conteúdo não importa, só que ela tenha contorno para o
/// kernel repetir.
fn petala() -> VecPath {
    VecPath {
        verts: [[1.0, -0.2], [2.0, 0.0], [1.0, 0.2]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// **Quantas formas a rosácea desenha depois deste gesto** — a travessia INTEIRA: o que a tool
/// guarda vira `SymmetrySpec` (a porta `from_style`, a mesma que a shell usa) e o kernel emite a
/// lista que o canvas pinta.
fn copias_desenhadas(tool: &VectorTool) -> usize {
    let estilo = tool.draw_config().symmetry;
    let spec = SymmetrySpec::from_style(estilo, [0.0, 0.0], [0.0, 1.0]);
    symmetry_paths(&petala(), &spec).len()
}

/// Entrega ao `tool` tudo o que o painel pôs no barramento.
fn dreno(host: &mut MockPanelHost, tool: &mut VectorTool) {
    use ph2d_editor_core::tool::Tool;
    for a in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = a {
            Tool::handle_panel_event(tool, pe);
        }
    }
}

/// A ferramenta como o artista a deixa antes de tocar na barra: simetria **armada** e em
/// **Radial**, pelos mesmos dois cliques que o painel emite.
///
/// ⚠️ Sem isto o `kind` seria o `MirrorX` do default e o `copy_count` valeria **2** em toda a
/// varredura — um gate que passasse a medir o espelho não veria a contagem nem uma vez.
fn tool_em_radial() -> VectorTool {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = VectorTool::default();
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::Click(ph2d_tool_vector::ids::VECTOR_SYM_ON),
    );
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::Click(ph2d_tool_vector::params::symmetry_kind_id(
            SymmetryKind::Radial,
        )),
    );
    t
}

/// ⭐ **As duas PONTAS do curso chegam ao kernel** — e o oráculo é o vocabulário, não uma conta
/// deste ficheiro.
///
/// ⚠️ **As duas pontas, e não uma.** Um gate que só puxasse a barra até ao fim ficaria verde sobre
/// um braço que ignorasse o track e escrevesse `MAX_SEGMENTS` sempre.
#[test]
fn the_segments_slider_delivers_its_count_to_the_rosette() {
    publica_radial();
    for (track, esperado) in [(0.0_f32, MIN_SEGMENTS), (1.0, MAX_SEGMENTS)] {
        let mut host = crate::host_aberto();
        let mut st = VectorPanelState;
        let mut tool = tool_em_radial();
        host.set_slider_value(ph2d_tool_vector::ids::VECTOR_SYM_SEGMENTS, track);
        host.apply_panel_event::<VectorPanel>(
            &mut st,
            WidgetEvent::ValueChanged(ph2d_tool_vector::ids::VECTOR_SYM_SEGMENTS),
        );
        dreno(&mut host, &mut tool);
        assert_eq!(
            copias_desenhadas(&tool),
            esperado as usize,
            "a barra em {track} pediu {esperado} copias e a rosacea desenhou outra coisa - o \
             valor nao atravessa o painel ate' ao kernel (o controlo esta' MORTO, ou o mapa da \
             fronteira divergiu do que o populate da' ao par slider<->chip)"
        );
    }
    ph2d_panel_vector::set_current_vector_style(None);
}

/// ⭐ **A outra mão: DIGITAR no chip também chega à rosácea, e NENHUM dos dois eventos fica órfão.**
///
/// ⚠️ Um commit no chip emite **dois** `ValueChanged` — o do chip e o da barra que ele espelha. O
/// painel encaminha o da barra e **engole** o do chip; sem a segunda metade o evento cai no
/// catch-all e sai do painel como *não consumido*, que é a forma que este app tem de dizer
/// `[hero] unhandled event`. As duas metades são medidas aqui de propósito: a primeira pela
/// geometria, a segunda pelo `EventOutcome`.
#[test]
fn typing_a_count_into_the_chip_reaches_the_rosette() {
    publica_radial();
    let mut host = crate::host_aberto();
    let mut st = VectorPanelState;
    let mut tool = tool_em_radial();
    let alvo = MAX_SEGMENTS - 1;
    let evs = host.type_into_number(
        ph2d_tool_vector::ids::VECTOR_SYM_SEGMENTS_NUM,
        &alvo.to_string(),
    );
    assert!(
        evs.iter().any(
            |e| matches!(e, WidgetEvent::ValueChanged(c) if *c == ph2d_tool_vector::ids::VECTOR_SYM_SEGMENTS_NUM)
        ),
        "digitar no chip nao emitiu o ValueChanged dele - a fixtura deixou de exercitar o caminho"
    );
    for ev in evs {
        let saida = host.apply_panel_event::<VectorPanel>(&mut st, ev);
        // ⚠️ Só os `ValueChanged`: o `Blur` que o commit também emite não tem dono em painel
        // nenhum deste app, e exigi-lo aqui mediria o foco em vez do valor.
        if matches!(ev, WidgetEvent::ValueChanged(_)) {
            assert_eq!(
                saida,
                ph2d_editor_core::panel::EventOutcome::Consumed,
                "o painel deixou {ev:?} passar sem dono - um valor da fileira de Segments que sai \
                 como nao-consumido e' um `[hero] unhandled event` no log do artista"
            );
        }
    }
    dreno(&mut host, &mut tool);
    assert_eq!(
        copias_desenhadas(&tool),
        alvo as usize,
        "digitar {alvo} no chip nao mudou o numero de copias que a rosacea desenha"
    );
    ph2d_panel_vector::set_current_vector_style(None);
}
