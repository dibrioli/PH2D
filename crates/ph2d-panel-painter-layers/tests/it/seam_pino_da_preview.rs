//! **O pino da preview viva, pelo ponteiro e no ECRÃ** (dono 22/10: *«um ícone de pino para fixar o preview
//! (não rolar com o scroll)»*). Fixada, a preview fica no mesmo sítio do ecrã quando o corpo rola e a área
//! rolável começa por baixo dela; desafixada, ela rola com os controlos. O pino é o rect que se pinta e o
//! que se clica, alterna sem chegar a mais ninguém, e é um botão de alternar para quem não vê.
//!
//! **Mutação que sangra:** `paint_pinned_preview` a devolver sempre `header_bottom` (o sticky desligado) ⇒
//! `fixada_nao_rola_e_desafixada_rola` e `a_area_rolavel_comeca_por_baixo_da_preview_fixada` reprovam.

use ph2d_a11y::{NodeId, Role, Toggled};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
};
use ph2d_panel_painter_layers::{
    PainterLayersPanel, a11y_do_pino, preview_fixada, rect_da_preview, rect_do_pino,
    set_current_wet_presets, set_current_wet_preview,
};
use ph2d_tool_painter::ids;
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;
use std::sync::Arc;

const PINO: NodeId = ids::PAINTER_BRUSH_PREVIEW_PIN;
/// Quanto o corpo rola em cada prova.
const ROLA: f32 = 80.0;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

struct Banca {
    host: MockPanelHost,
    st: PainterLayersPanelState,
    tool: PainterTool,
}

impl Banca {
    fn nova() -> Self {
        let mut tool = PainterTool::default();
        tool.set_paint_media(PaintMedia::WetPaint);
        set_current_wet_presets(None);
        set_current_dock_shows_layers(false);
        set_current_wet_preview(Some((Arc::new(vec![200; 256 * 128 * 4]), 256, 128)));
        let host = MockPanelHost::with_panel::<PainterLayersPanel>();
        Banca {
            host,
            st: PainterLayersPanelState,
            tool,
        }
    }

    fn pinta(&mut self) -> Vec<(NodeId, Rect)> {
        set_current_brush(Some(self.tool.brush_settings()));
        self.host
            .paint::<PainterLayersPanel>(&mut self.st, viewport())
    }

    fn rola(&mut self, y: f32) -> Vec<(NodeId, Rect)> {
        self.host
            .store_mut()
            .set_panel_scroll(ph2d_editor_core::ids::PAINTER_LAYERS_PANEL, y);
        self.pinta()
    }

    /// Um clique no centro de `r`, pela porta do ponteiro; devolve os eventos e o que chegou ao barramento.
    fn clica(&mut self, r: Rect) -> (Vec<WidgetEvent>, Vec<EditorAction>) {
        let evs = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        for ev in &evs {
            let _ = self
                .host
                .apply_panel_event::<PainterLayersPanel>(&mut self.st, *ev);
        }
        // O foco que o clique dá é do despacho genérico; o que conta é o que o gesto FAZ.
        let evs = evs
            .into_iter()
            .filter(|e| !matches!(e, WidgetEvent::Focus(_) | WidgetEvent::Blur(_)))
            .collect();
        (evs, self.host.drained_actions())
    }
}

impl Drop for Banca {
    fn drop(&mut self) {
        set_current_wet_preview(None);
    }
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Rect {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} não está pintado"))
}

fn fixa(b: &mut Banca) {
    let rects = b.rola(0.0);
    let (evs, _) = b.clica(rect_of(&rects, PINO));
    assert_eq!(evs, vec![WidgetEvent::Toggled(PINO)]);
    assert!(preview_fixada(b.host.store()), "o clique no pino não fixou");
}

/// (i) Fixada, rolar o corpo `ROLA` px deixa a preview no MESMO sítio do ecrã e move os controlos de baixo;
/// desafixada, a preview anda com eles.
#[test]
fn fixada_nao_rola_e_desafixada_rola() {
    let mut b = Banca::nova();
    let abaixo = ids::PAINTER_BRUSH_SYNC;
    for fixada in [false, true] {
        if fixada {
            fixa(&mut b);
        }
        let r0 = b.rola(0.0);
        let (p0, c0) = (
            rect_da_preview().expect("a preview pinta"),
            rect_of(&r0, abaixo),
        );
        let r1 = b.rola(ROLA);
        let (p1, c1) = (
            rect_da_preview().expect("a preview pinta"),
            rect_of(&r1, abaixo),
        );
        assert!(
            (c0.y - ROLA - c1.y).abs() < 0.5,
            "fixada={fixada}: o controlo de baixo não rolou ({} → {})",
            c0.y,
            c1.y
        );
        let esperado = if fixada { p0.y } else { p0.y - ROLA };
        assert!(
            (p1.y - esperado).abs() < 0.5,
            "fixada={fixada}: a preview foi de {} para {} (esperado {esperado})",
            p0.y,
            p1.y
        );
        // O pino anda com a preview: fixada, o rect pintado é o da porta; desafixada, ele saiu com ela por
        // cima do recorte da área e não se alcança.
        if fixada {
            assert_eq!(rect_of(&r1, PINO), rect_do_pino(p1));
        } else {
            assert!(!r1.iter().any(|(w, r)| *w == PINO && r.w > 0.0 && r.h > 0.0));
        }
    }
}

/// (iv) Fixada, a área rolável começa por baixo da preview: a primeira linha (o Paint Mode) cai por baixo
/// dela e recebe o clique; e, rolado, nenhum controlo do corpo é alcançável atrás da preview.
#[test]
fn a_area_rolavel_comeca_por_baixo_da_preview_fixada() {
    let mut b = Banca::nova();
    fixa(&mut b);
    let rects = b.rola(0.0);
    let p = rect_da_preview().expect("a preview pinta");
    let media = rect_of(&rects, ids::PAINTER_BRUSH_MEDIA);
    assert!(
        media.y >= p.y + p.h,
        "a primeira linha ({}) começa antes do fundo da preview ({})",
        media.y,
        p.y + p.h
    );
    assert_eq!(
        b.host
            .hit_at(media.x + media.w * 0.5, media.y + media.h * 0.5),
        Some(ids::PAINTER_BRUSH_MEDIA)
    );
    let _ = b.rola(ROLA);
    let p = rect_da_preview().expect("a preview pinta");
    let pino = rect_do_pino(p);
    let mut y = p.y + 1.0;
    while y < p.y + p.h {
        for x in [p.x + 1.0, p.x + p.w * 0.5] {
            let alvo = b.host.hit_at(x, y);
            assert!(
                alvo.is_none() || (alvo == Some(PINO) && y >= pino.y && y < pino.y + pino.h),
                "({x}, {y}) atrás da preview fixada chega a {alvo:?}"
            );
        }
        y += 4.0;
    }
}

/// (ii) O clique no rect do pino alterna o estado e não chega a mais ninguém — nem ao tool, nem à preview,
/// nem à tela —, fixada e desafixada; o Enter com o foco nele alterna também.
#[test]
fn o_clique_no_pino_alterna_e_nao_chega_a_outro_alvo() {
    let mut b = Banca::nova();
    for esperado in [true, false] {
        let rects = b.rola(0.0);
        let r = rect_of(&rects, PINO);
        assert_eq!(r, rect_do_pino(rect_da_preview().expect("a preview pinta")));
        assert_eq!(b.host.hit_at(r.x + r.w * 0.5, r.y + r.h * 0.5), Some(PINO));
        let (evs, acoes) = b.clica(r);
        assert_eq!(evs, vec![WidgetEvent::Toggled(PINO)]);
        assert!(acoes.is_empty(), "o pino chegou ao barramento: {acoes:?}");
        assert_eq!(preview_fixada(b.host.store()), esperado);
    }
    let _ = b.pinta();
    let evs = b.host.type_keys_then_enter(PINO, "");
    assert!(evs.contains(&WidgetEvent::Toggled(PINO)), "{evs:?}");
    assert!(preview_fixada(b.host.store()), "o Enter não fixou");
}

/// (iii) Para quem não vê, o pino é um BOTÃO de alternar com o estado e o nome do que o clique faz.
#[test]
fn o_pino_e_um_botao_de_alternar() {
    let mut b = Banca::nova();
    for fixada in [false, true] {
        if fixada {
            fixa(&mut b);
        }
        let rects = b.rola(0.0);
        let r = rect_of(&rects, PINO);
        let no = a11y_do_pino(b.host.store(), r);
        assert_eq!(no.role(), Role::Button);
        assert_eq!(
            no.toggled(),
            Some(if fixada {
                Toggled::True
            } else {
                Toggled::False
            })
        );
        let dica = if fixada {
            "panel.painter_preview.unpin"
        } else {
            "panel.painter_preview.pin"
        };
        assert_eq!(no.label(), Some(ph2d_i18n::tr(dica)));
        assert_eq!(b.host.store().tooltip_for(PINO), Some(ph2d_i18n::tr(dica)));
    }
}
