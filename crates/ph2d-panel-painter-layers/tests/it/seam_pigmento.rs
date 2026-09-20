//! **A fileira `Pigment` é oferecida EXACTAMENTE nos meios que a sentem — e o dedo chega-lhe.**
//!
//! ⭐⭐ Estes gates nascem da ordem do dono de 2026-09-20 (*«ligue o digital»*), que tirou a cerca
//! `watercolor &&` do `effective_pigment_mix`: a mistura subtractiva deixou de ser da aguada e
//! passou a valer para todo meio que componha um dab pela porta `blend_over_pigment`.
//!
//! ⛔⛔ **Alargar o alcance de uma LEI cria imediatamente duas maneiras de ela e o BOTÃO
//! discordarem, e as curas são OPOSTAS:**
//!
//! * o meio que **lê** a lei e não vê a fileira tem um knob **INALCANÇÁVEL** (cura: pintá-la);
//! * o meio que **vê** a fileira e não lê a lei tem um knob **MORTO** (cura: escondê-la).
//!
//! *As duas leem-se igual numa tabela de risco.* O que as separa é a medição, e ela vive do lado da
//! ferramenta (`diag_pigmento_por_meio`, na `ph2d-tool-painter`): Digital · Watercolor · Impasto
//! movem o barro com o knob; o **Wet Paint** devolve o pixel **ao bit** (o depósito é do solver de
//! fluido, que tem o Kubelka–Munk dele). A porta que carrega esse veredito é a
//! [`PaintMedia::offers_pigment_mixing`], e **é contra ela que estes gates medem a TELA**.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

fn pintado(media: PaintMedia) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    let mut t = PainterTool::default();
    t.set_paint_media(media);
    set_current_brush(Some(t.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

/// Quantas vezes o id da fileira é pintado com retângulo VIVO neste meio.
fn vezes_pintada(media: PaintMedia) -> usize {
    let (_, _, rects) = pintado(media);
    rects
        .iter()
        .filter(|(id, r)| {
            *id == ph2d_tool_painter::ids::PAINTER_WATERCOLOR_MIX && r.w > 0.0 && r.h > 0.0
        })
        .count()
}

/// ⭐⭐⭐ **A TELA CONCORDA COM A PORTA, NOS DOIS SENTIDOS.**
///
/// ⚠️ **As duas metades são obrigatórias e nenhuma basta:** só a positiva deixaria passar uma
/// fileira pintada num meio que a ignora (o knob MORTO), e só a negativa deixaria passar a lei
/// ligada sem botão nenhum (o INALCANÇÁVEL) — que é exactamente o estado em que o Digital esteve
/// entre o momento em que a lei foi trocada e esta ordem.
#[test]
fn a_fileira_do_pigmento_e_pintada_nos_meios_que_a_porta_declara() {
    let mut erros: Vec<String> = Vec::new();
    for media in MEIOS {
        let n = vezes_pintada(media);
        let oferece = media.offers_pigment_mixing();
        if oferece && n == 0 {
            erros.push(format!(
                "{media:?} SENTE a mistura e a fileira não é pintada — knob INALCANÇÁVEL"
            ));
        }
        if !oferece && n > 0 {
            erros.push(format!(
                "{media:?} NÃO lê a lei e a fileira é pintada {n}× — knob MORTO"
            ));
        }
    }
    assert!(erros.is_empty(), "{}", erros.join(" · "));
}

/// ⛔ **E ela é pintada UMA vez por quadro, nunca duas.**
///
/// A aguada hospeda-a no cartão *Water*; os outros dois meios que a sentem ganham o cartão
/// *Mixing*. ⚠️ Se a subtracção da aguada em `paint_brush_sections.rs` cair, os dois cartões
/// pintam o MESMO id no mesmo quadro — e isso não é um controlo a mais: o `HitIndex::hit` resolve
/// de trás para a frente, logo um dos dois fica **tapado sob o dedo** enquanto os dois se veem.
#[test]
fn a_fileira_do_pigmento_nunca_e_pintada_duas_vezes_no_mesmo_quadro() {
    for media in MEIOS {
        let n = vezes_pintada(media);
        assert!(
            n <= 1,
            "{media:?} pinta o Pigment {n}× no mesmo quadro — dois hospedeiros a correr juntos"
        );
    }
}

/// ⭐⭐ **O NÚMERO QUE A FILEIRA MOSTRA É `0` COM A MISTURA DESLIGADA** — a lei do controlo fundido.
///
/// O `Pigment` é o par *toggle + amount* que a redesign de 2026-07-07 fundiu num slider só: `0`
/// desliga **guardando** a quantidade, para que subir de novo a devolva sem perda. O pincel de
/// fábrica tem exactamente esse estado (`pigment: false`, `pigment_mix: 0,5`), logo *uma fileira
/// que mostrasse o campo cru abriria o painel a anunciar meia mistura com a mistura desligada*.
///
/// ⚠️ É a metade de LEITURA da lei cuja metade de ESCRITA é o `set_brush_pigment_mixing`, e ela
/// vive numa porta só (`paint_pigment::pigment_amount`) porque agora tem **dois** hospedeiros.
#[test]
fn a_fileira_mostra_zero_quando_a_mistura_esta_desligada() {
    let id = ph2d_tool_painter::ids::PAINTER_WATERCOLOR_MIX;
    for media in MEIOS.iter().filter(|m| m.offers_pigment_mixing()) {
        let (host, _, _) = pintado(*media);
        let mostrado = host.store().number_value(id).unwrap_or(f64::NAN);
        assert!(
            mostrado.abs() < 1e-6,
            "{media:?} abre a mostrar {mostrado} com o `pigment` desligado — a fileira está a \
             pintar o campo CRU (`pigment_mix`, que de fábrica é 0,5) em vez do valor derivado"
        );
    }
}

/// ⭐⭐ **O DEDO CHEGA AO NÚMERO NO DIGITAL** — a costura inteira, pelo caminho real do painel.
///
/// ⚠️ **Ela não é herdada do gate da aguada.** O `seam::watercolor_sliders_forward_setvalue` varre
/// `PAINTER_WATERCOLOR_FIELDS`, e o censo derivado da tela do `seam_watercolor_cards` mede *o que a
/// aquarela ACRESCENTA ao Digital* — com a fileira a existir nos dois, ela sai da diferença e fica
/// fora dos dois. *Um campo que muda de casa sai da população que o media sem que nada acuse.*
#[test]
fn o_pigmento_do_digital_chega_a_ferramenta() {
    let id = ph2d_tool_painter::ids::PAINTER_WATERCOLOR_MIX;
    let (mut host, mut st, _) = pintado(PaintMedia::Digital);
    assert!(
        host.store().number_value(id).is_some(),
        "CONTROLO: a fileira do Digital tem de estar registada como campo numérico"
    );
    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    let chegou = host.drained_actions().iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(ph2d_editor_core::tool::PanelEvent::SetValue(i, _))
                if *i == id
        )
    });
    assert!(
        outcome == EventOutcome::Consumed && chegou,
        "o Pigment do Digital é pintado e o ValueChanged morre dentro do painel \
         (outcome {outcome:?}, chegou {chegou})"
    );
}
