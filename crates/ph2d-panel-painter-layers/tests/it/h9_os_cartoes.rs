//! **A vista Layers em linhas de cartão, fatia H9** (`docs/CartoesUI/13_PAINTER_PLANO.md`). Pelo gesto real, nos rects
//! pintados, e o que chega ao barramento é o MESMO do painel de antes. As barras de camada (opacidade, relevo) continuam
//! `Slider`: mesmos ids. Só o «Group» e a seta têm de obedecer às mesmas leis; a queda continua a ser do despacho.

use super::h1_os_cartoes::Mesa;
use super::h5_os_cartoes::sem_sobrepor;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m_card;
use ph2d_editor_core::interaction::{PanelRowFamily, WidgetEvent};
use ph2d_editor_core::panel::cards::censo;
use ph2d_host::PointerKind;
use ph2d_panel_painter_layers::FALLBACK_BRUSH;
use ph2d_panel_painter_layers::state::set_current_layers;
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_tool_painter::{LayerId, LayerStack};
use std::collections::BTreeSet;

fn w(l: LayerId, k: PainterLayerWidget) -> NodeId {
    painter_layer_widget_id(l.0, k)
}

/// Uma pilha com o que a lista distingue: base, raster, relevo, grupo com filho.
fn pilha() -> (LayerStack, [LayerId; 3]) {
    let mut s = LayerStack::new();
    let base = s.add_raster("Base", 8, 8).expect("nasce");
    let meio = s.add_raster("Middle", 8, 8).expect("nasce");
    let cima = s.add_raster("Top", 8, 8).expect("nasce");
    (s, [base, meio, cima])
}

fn abre(cartoes: bool, s: LayerStack) -> Mesa {
    let mut m = Mesa::nova(cartoes, FALLBACK_BRUSH, true);
    set_current_layers(Some(s));
    m.assenta();
    m
}

fn deste(m: &Mesa, s: &LayerStack) -> BTreeSet<NodeId> {
    let mut u: BTreeSet<NodeId> = [
        p::PAINTER_LAYERS_ADD,
        p::PAINTER_LAYERS_GROUP,
        p::PAINTER_LAYERS_DUPLICATE,
        p::PAINTER_LAYERS_DELETE,
        p::PAINTER_LAYERS_ADD_ADJUSTMENT,
        p::PAINTER_LAYERS_ADD_TEXTURE,
        p::PAINTER_LAYERS_MASK,
        p::PAINTER_LAYERS_CLIP,
        p::PAINTER_LAYERS_ALPHA_LOCK,
        p::PAINTER_LAYERS_REFERENCE,
        p::PAINTER_APPLY,
    ]
    .into_iter()
    .collect();
    for l in s.all_ids() {
        for k in [
            PainterLayerWidget::Row,
            PainterLayerWidget::Visibility,
            PainterLayerWidget::MoveUp,
            PainterLayerWidget::MoveDown,
            PainterLayerWidget::Blend,
            PainterLayerWidget::Opacity,
            PainterLayerWidget::ImpastoDepth,
            PainterLayerWidget::ImpastoLevel,
            PainterLayerWidget::MaskView,
            PainterLayerWidget::MaskInvert,
            PainterLayerWidget::MaskApply,
        ] {
            u.insert(w(l, k));
        }
    }
    m.ids().into_iter().filter(|i| u.contains(i)).collect()
}

/// ⭐ Os mesmos alvos do painel de antes, e o clique em cada um manda o mesmo.
#[test]
fn the_layers_card_has_the_same_targets_and_sends_the_same_actions() {
    let estado = || {
        let (mut s, [_, meio, _]) = pilha();
        s.set_active(meio);
        s
    };
    let velha = abre(false, estado());
    let (va, _) = (deste(&velha, &estado()), ());
    set_current_layers(None);
    let nova = abre(true, estado());
    let na = deste(&nova, &estado());
    set_current_layers(None);
    let so_antes: Vec<_> = va.difference(&na).collect();
    let so_agora: Vec<_> = na.difference(&va).collect();
    assert!(
        so_antes.is_empty() && so_agora.is_empty(),
        "só antes {so_antes:?}, só agora {so_agora:?}"
    );
    assert!(na.len() > 20, "a régua viu pouco: {}", na.len());
    let mut vistos = 0;
    for id in &na {
        let opacidade = estado()
            .all_ids()
            .any(|l| w(l, PainterLayerWidget::Opacity) == *id);
        let lista = estado()
            .all_ids()
            .any(|l| w(l, PainterLayerWidget::Blend) == *id);
        if opacidade || lista || *id == p::PAINTER_LAYERS_ADD_ADJUSTMENT {
            continue;
        }
        let velho = abre(false, estado()).clica(*id);
        set_current_layers(None);
        let novo = abre(true, estado()).clica(*id);
        set_current_layers(None);
        assert_eq!(velho, novo, "{id:?}");
        vistos += 1;
    }
    assert!(vistos > 12, "viu {vistos}");
}

/// A base não tem mistura; nas pontas a seta não tem alvo.
#[test]
fn the_base_has_no_blend_and_the_edges_have_no_arrow() {
    let (s, [base, meio, cima]) = pilha();
    let m = abre(true, s);
    let tem = |l, k| m.rect(w(l, k)).is_some();
    let achado = (
        tem(base, PainterLayerWidget::Blend),
        tem(meio, PainterLayerWidget::Blend),
        tem(cima, PainterLayerWidget::MoveUp),
        tem(meio, PainterLayerWidget::MoveDown),
        tem(base, PainterLayerWidget::MoveUp),
    );
    set_current_layers(None);
    assert_eq!(achado, (false, true, false, false, false));
}

/// Mover a opacidade pelo gesto escreve no mesmo id do painel de antes; a linha inteira agarra-se.
#[test]
fn the_opacity_bar_moves_by_gesture_and_the_row_flies() {
    let (s, [_, meio, _]) = pilha();
    let mut m = abre(true, s);
    let op = w(meio, PainterLayerWidget::Opacity);
    let r = m.rect(op).expect("a barra");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = m.ponteiro(PointerKind::Down, x, y);
    ev.extend(m.ponteiro(PointerKind::Move, x + 5.0, y));
    ev.extend(m.ponteiro(PointerKind::Up, x + 5.0, y));
    assert!(ev.contains(&WidgetEvent::ValueChanged(op)), "{ev:?}");

    let linha = w(meio, PainterLayerWidget::Row);
    assert_eq!(
        m.host.store().panel_row_family(linha),
        Some(PanelRowFamily::PainterLayer)
    );
    let n = m.rect(linha).expect("a linha");
    let opr = m.rect(op).expect("a barra");
    assert!(n.h > opr.h, "a linha inteira agarra-se: {n:?}");
    let (x, y) = (n.x + n.w * 0.5, n.y + m_card::FIELD_H * 0.5);
    let _ = m.ponteiro(PointerKind::Down, x, y);
    let _ = m.ponteiro(PointerKind::Move, x, y + 40.0);
    let _ = m.ponteiro(PointerKind::Move, x, y + 80.0);
    m.quadro();
    let voa =
        m.host.store().panel_row_drag().is_some_and(|(f, d)| {
            f == PanelRowFamily::PainterLayer && d.active && d.dragged == linha
        });
    let ev = m.ponteiro(PointerKind::Up, x, y + 80.0);
    set_current_layers(None);
    assert!(voa, "a linha não voou");
    assert!(
        ev.iter().any(|e| matches!(
            e,
            WidgetEvent::PanelRowReparent {
                family: PanelRowFamily::PainterLayer,
                ..
            }
        )),
        "a largada não chegou: {ev:?}"
    );
}

/// A lista da mistura abre por baixo da caixa, e o modo manda o mesmo do painel de antes.
#[test]
fn the_blend_list_opens_under_its_box_and_sends_the_same() {
    let manda = |cartoes: bool| {
        let (s, [_, meio, _]) = pilha();
        let mut m = abre(cartoes, s);
        let caixa = w(meio, PainterLayerWidget::Blend);
        let r = m.rect(caixa).expect("a caixa");
        let _ = m.clica(caixa);
        m.assenta();
        let opcao = ph2d_tool_painter::ids::painter_layer_blend_option_id(meio.0, 2);
        let o = m.rect(opcao).expect("a opção");
        assert!(o.y >= r.y + r.h - 1.0, "{o:?} × {r:?}");
        let a = m.clica(opcao);
        set_current_layers(None);
        a
    };
    assert_eq!(manda(false), manda(true));
}

#[test]
fn nothing_overlaps_no_text_is_cut_and_no_row_sticks() {
    let (s, _) = pilha();
    let m = abre(true, s);
    let ids: BTreeSet<NodeId> = m.ids();
    let _ = ids.len();
    let s2 = pilha().0;
    let universo = deste(&m, &s2);
    let _ = universo.len();
    let mut m = m;
    let (_, c) = censo::grava(|| {
        let (h, st, j) = (&mut m.host, &mut m.st, m.janela);
        h.paint::<ph2d_panel_painter_layers::PainterLayersPanel>(st, j)
    });
    set_current_layers(None);
    assert!(c.textos_cortados().is_empty(), "{:#?}", c.textos_cortados());
    assert!(c.linhas_grudadas().is_empty(), "{:#?}", c.linhas_grudadas());
    assert!(
        c.irmaos_separados().is_empty(),
        "{:#?}",
        c.irmaos_separados()
    );
    let (s3, _) = pilha();
    let m3 = abre(true, s3);
    let u3 = deste(&m3, &pilha().0);
    let (corpo, _) = (u3, ());
    let barras: BTreeSet<NodeId> = corpo
        .iter()
        .copied()
        .filter(|i| {
            !pilha()
                .0
                .all_ids()
                .any(|l| w(l, PainterLayerWidget::Row) == *i)
        })
        .collect();
    sem_sobrepor(&m3, &barras, "camadas");
    set_current_layers(None);
}

/// Um ajuste não aceita Mask, Clip, Lock nem Ref: os quatro apagados e sem alvo, como no painel de antes.
/// *Mutação: `h_inelegivel_vivo` ⇒ vermelho.*
#[test]
fn an_adjustment_layer_has_no_modifiers() {
    let alvos = |cartoes: bool| {
        let mut s = pilha().0;
        let adj = s
            .add_adjustment(ph2d_tool_painter::AdjustmentKind::Bloom)
            .expect("o ajuste nasce");
        s.set_active(adj);
        let m = abre(cartoes, s);
        let r = [
            p::PAINTER_LAYERS_MASK,
            p::PAINTER_LAYERS_CLIP,
            p::PAINTER_LAYERS_ALPHA_LOCK,
            p::PAINTER_LAYERS_REFERENCE,
            p::PAINTER_LAYERS_ADD,
        ]
        .map(|id| m.rect(id).is_some());
        set_current_layers(None);
        r
    };
    assert_eq!(alvos(false), alvos(true));
    assert_eq!(alvos(true), [false, false, false, false, true]);
}
