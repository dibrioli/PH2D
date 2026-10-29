//! ⭐ **Cada linha do Tuning PELA PORTA DO RATO** (auditoria J6, 28/10): a varredura deriva da tabela de knobs
//! ([`rows::rows`]) — arrasta o slider (ou clica a opção apagada de uma escolha) por COORDENADA de ecrã, entrega o
//! barramento à ferramenta e confirma que o valor chegou ao knob e que o painel o redesenha. Mandar o evento à mão
//! (o `seam.rs`) não prova o clique: foi assim que o «Same as Paper» do pincel ficou morto com o portão verde.
//!
//! E as duas curas da mesma família: a linha de um knob que só UMA ferramenta da água lê só aparece com ela na mão
//! (`WetKnobs::dona_do_knob` — editada com outra, voltava atrás sozinha), e o material do Real Oil (Depth, Gloss,
//! Paper Press) esmaece com a dica enquanto ele está desligado.

use ph2d_a11y::NodeId;
use ph2d_editor_core::Tool;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_wet_tuning::{WetTuningPanel, rows, set_current_brush, state};
use ph2d_tool_painter::{PaintMedia, PainterTool, WetKnobs, WetTool};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 4400.0)
}

/// O Wet Paint com o Real Oil ligado e `ferramenta` na mão.
fn ferramenta(ferramenta: WetTool, oleo: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_oleo_real_ligado(oleo);
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        ferramenta.fio().to_owned(),
    ));
    assert_eq!(
        t.brush_settings().wet_tool,
        ferramenta,
        "a ferramenta não ficou na mão"
    );
    t
}

struct Mesa {
    host: MockPanelHost,
    st: state::WetTuningPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova(t: &PainterTool) -> Self {
        set_current_brush(Some(t.brush_settings()));
        let mut host = MockPanelHost::with_panel::<WetTuningPanel>();
        host.set_panel_visible(WetTuningPanel::ID, true);
        for s in rows::SECTIONS {
            host.store_mut().set_collapsed(s.header, false);
        }
        let mut st = state::WetTuningPanelState;
        let rects = host.paint::<WetTuningPanel>(&mut st, viewport());
        Mesa { host, st, rects }
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(i, r)| *i == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    /// Os eventos do gesto pelo painel, o barramento à ferramenta e o painel re-pintado (um quadro do app).
    fn entrega(
        &mut self,
        eventos: Vec<ph2d_editor_core::interaction::WidgetEvent>,
        t: &mut PainterTool,
    ) -> usize {
        for ev in eventos {
            let _ = self
                .host
                .apply_panel_event::<WetTuningPanel>(&mut self.st, ev);
        }
        let mut n = 0;
        for a in self.host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = a {
                t.handle_panel_event(pe);
                n += 1;
            }
        }
        set_current_brush(Some(t.brush_settings()));
        self.rects = self.host.paint::<WetTuningPanel>(&mut self.st, viewport());
        n
    }
}

#[test]
fn cada_linha_do_tuning_aceita_o_rato_e_o_valor_chega_a_ferramenta() {
    let mut falhas = Vec::new();
    for row in rows::rows() {
        let def = &ph2d_wet_paint::tuning::KNOB_DEFS[row.knob];
        let dona = WetKnobs::dona_do_knob(def.knob).unwrap_or(WetTool::Paint);
        let mut t = ferramenta(dona, true);
        let mut mesa = Mesa::nova(&t);
        let antes = t.brush_settings().wet_knobs.knobs[row.knob];
        let eventos = if let Some(opts) = &row.options {
            let alvo = usize::from(antes.round() < 0.5);
            let Some(r) = mesa.rect(opts[alvo].1) else {
                falhas.push(format!("{}: a opção {alvo} não foi pintada", row.key));
                continue;
            };
            mesa.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5)
        } else {
            let Some(r) = mesa.rect(row.slider) else {
                falhas.push(format!(
                    "{}: o slider não foi pintado com {dona:?} na mão",
                    row.key
                ));
                continue;
            };
            // Do meio da trilha para a ponta mais longe do valor de agora.
            let (x0, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
            let x1 = if row.track_of(antes) < 0.5 {
                r.x + r.w * 0.95
            } else {
                r.x + r.w * 0.05
            };
            mesa.host.drag_at(x0, y, x1, y)
        };
        if mesa.entrega(eventos, &mut t) == 0 {
            falhas.push(format!(
                "{}: o gesto do rato não chegou à ferramenta",
                row.key
            ));
            continue;
        }
        let depois = t.brush_settings().wet_knobs.knobs[row.knob];
        if (depois - antes).abs() < row.step.max(1e-9) * 0.5 {
            falhas.push(format!(
                "{}: o knob não mudou ({antes} → {depois})",
                row.key
            ));
            continue;
        }
        if row.options.is_none() {
            let visto = mesa.host.store().slider(row.slider).map(|(_, v)| v);
            if visto.is_none_or(|v| (v - row.track_of(depois)).abs() > 1e-3) {
                falhas.push(format!(
                    "{}: o painel não redesenha o valor ({visto:?} vs {})",
                    row.key,
                    row.track_of(depois)
                ));
            }
        }
    }
    set_current_brush(None);
    assert!(
        falhas.is_empty(),
        "{} linha(s):\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}

/// A linha de um knob que só UMA ferramenta da água lê só aparece com ELA na mão — com o Pincel (ou outra) o slider
/// escrevia um valor que a troca de ferramenta recarregava do slot dela (a edição sumia). A população vem da lei.
#[test]
fn a_linha_de_uma_ferramenta_so_aparece_com_ela_na_mao() {
    let deles: Vec<_> = rows::rows()
        .iter()
        .filter_map(|r| {
            WetKnobs::dona_do_knob(ph2d_wet_paint::tuning::KNOB_DEFS[r.knob].knob).map(|d| (r, d))
        })
        .collect();
    for chave in ["dryerLevel", "dryerEdge", "blendReach", "waterPush"] {
        assert!(
            deles.iter().any(|(r, _)| r.key == chave),
            "{chave} devia ser de uma ferramenta só"
        );
    }
    for na_mao in [
        WetTool::Paint,
        WetTool::Smear,
        WetTool::Blend,
        WetTool::Wet,
        WetTool::Dry,
        WetTool::Blow,
    ] {
        let mesa = Mesa::nova(&ferramenta(na_mao, true));
        for (row, dona) in &deles {
            assert_eq!(
                mesa.rect(row.slider).is_some(),
                *dona == na_mao,
                "{} (de {dona:?}) com {na_mao:?} na mão",
                row.key
            );
        }
    }
    set_current_brush(None);
}

/// Depth, Gloss e Paper Press só agem com o Real Oil: desligado, a linha leva a dica (e o véu); ligado, nenhuma.
#[test]
fn o_material_do_oleo_esmaece_sem_o_real_oil() {
    let dica = ph2d_i18n::tr(ph2d_tool_painter::Dependente::MaterialSemOleo.dica());
    for (oleo, esperado) in [(false, true), (true, false)] {
        let mesa = Mesa::nova(&ferramenta(WetTool::Paint, oleo));
        for chave in ["oilDepth", "oilGloss", "paperPress"] {
            let row = rows::rows()
                .iter()
                .find(|r| r.key == chave)
                .expect("a linha existe");
            assert!(mesa.rect(row.slider).is_some(), "{chave} pintada");
            for id in [row.slider, row.chip] {
                assert_eq!(
                    mesa.host.store().tooltip_for(id).is_some_and(|t| t == dica),
                    esperado,
                    "{chave} com o Real Oil {oleo}"
                );
            }
        }
    }
    set_current_brush(None);
}
