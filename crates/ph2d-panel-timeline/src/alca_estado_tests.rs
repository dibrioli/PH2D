//! Os estados das alças (dono 10/11): hover ≠ repouso, aperto = acento, arrasto = acento + halo,
//! em todo tema; e o hit rect nunca depende do estado.

use super::*;
use crate::geom;
use crate::graph::TimeView;
use crate::state::TimelinePanelState;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroLayout;
use ph2d_editor_core::screens::slot::{Slot, SlotSet};
use ph2d_timeline::{StackHost, StripSource, TimelineDoc, TimelineState, TimelineViewSnapshot};
use ph2d_tokens::ColorToken;
use ph2d_vector::DrawOp;

/// Pisos MEDIDOS (OKLab, o par mais perto sobre `Theme::ALL` × as tintas de repouso das alças;
/// a medida está no `eprintln!` do portão e na ronda 3c do handoff).
const PISO_HOVER: f64 = 0.05; // medido 0.054 (fade, pior tema)
const PISO_ACENTO: f64 = 0.015; // medido 0.017: num tema o playhead JÁ é azul, o tamanho e o halo dizem o estado

fn lab(c: Color) -> (f64, f64, f64) {
    let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let k = c.components;
    let (l, ch, h) = ph2d_tokens::srgb_to_oklch(u(k[0]), u(k[1]), u(k[2]));
    let h = h.to_radians();
    (l, ch * h.cos(), ch * h.sin())
}

fn dist(a: Color, b: Color) -> f64 {
    let (x, y) = (lab(a), lab(b));
    ((x.0 - y.0).powi(2) + (x.1 - y.1).powi(2) + (x.2 - y.2).powi(2)).sqrt()
}

/// As tintas de repouso das alças: canto de esticar (verde), de cortar (vermelho), o fade e a ⟷ (o
/// playhead), as chaves do loop, e a chave da folha.
fn repousos(theme: Theme) -> [(&'static str, Color); 5] {
    [
        ("canto esticar", resolve(ColorToken::Success, theme)),
        ("canto cortar", resolve(ColorToken::Danger, theme)),
        (
            "fade / duração",
            resolve(ColorToken::TimelinePlayhead, theme),
        ),
        ("loop", resolve(ColorToken::TimelineLoopBrace, theme)),
        ("chave", tinta(TimelineTinta::Chave, theme)),
    ]
}

#[test]
fn os_estados_das_alcas_leem_se_em_todo_tema() {
    let (mut min_h, mut min_a) = ((f64::MAX, ""), (f64::MAX, ""));
    for theme in Theme::ALL {
        for (nome, r) in repousos(theme) {
            // A chave já é a tinta mais clara: o hover dela acende o ANEL (`pinta_chave`).
            let h = if nome == "chave" {
                let anel = tinta(TimelineTinta::FitaPropriedade, theme);
                dist(mistura(anel, r, ph2d_tokens::TIMELINE_ALCA_HOVER_MIX), anel)
            } else {
                dist(cor(EstadoAlca::Hover, r, theme), r)
            };
            let a = dist(cor(EstadoAlca::Apertada, r, theme), r);
            assert_eq!(cor(EstadoAlca::Repouso, r, theme), r, "{theme:?} {nome}");
            assert_eq!(
                cor(EstadoAlca::Arrasto, r, theme),
                acento(theme),
                "{theme:?} {nome}: o arrasto é o acento"
            );
            if h < min_h.0 {
                min_h = (h, nome);
            }
            if a < min_a.0 {
                min_a = (a, nome);
            }
            assert!(
                h >= PISO_HOVER,
                "{theme:?} {nome}: hover a {h:.3} do repouso"
            );
            assert!(
                a >= PISO_ACENTO,
                "{theme:?} {nome}: acento a {a:.3} do repouso"
            );
        }
    }
    eprintln!("piso medido: hover {min_h:?} · acento {min_a:?}");
}

/// O store decide o estado pelo MESMO id do clique: hover só sem nada apertado; aperto sem a folga
/// passada; arrasto depois dela.
#[test]
fn o_estado_sai_do_hot_do_active_e_da_folga() {
    let mut s = WidgetStore::default();
    let (id, outro) = (NodeId(7), NodeId(8));
    assert_eq!(EstadoAlca::de(&s, id), EstadoAlca::Repouso);
    s.set_hot(Some(id));
    assert_eq!(EstadoAlca::de(&s, id), EstadoAlca::Hover);
    s.set_active(Some(outro));
    assert_eq!(
        EstadoAlca::de(&s, id),
        EstadoAlca::Repouso,
        "outro apertado"
    );
    s.set_active(Some(id));
    s.begin_timeline_press(0.0, 0.0, false);
    assert_eq!(EstadoAlca::de(&s, id), EstadoAlca::Apertada);
    s.note_timeline_pointer(40.0, 0.0);
    assert_eq!(EstadoAlca::de(&s, id), EstadoAlca::Arrasto);
}

fn snap_duas_pecas() -> TimelineViewSnapshot {
    let mut doc = TimelineDoc::new();
    let l = doc.add_lane_in(StackHost::Document, "Base".into()).unwrap();
    doc.add_strip_to(StackHost::Document, l, StripSource::Clip(0), 0.5, 2.5)
        .unwrap();
    let mut st = TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    snap
}

/// Pinta a faixa nos cartões com o store preparado por `prepara`: (gravação, hits registados).
fn pinta_faixa(
    theme: Theme,
    prepara: impl FnOnce(&mut WidgetStore),
) -> (Vec<DrawOp>, Vec<(NodeId, Rect)>) {
    let snap = snap_duas_pecas();
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    prepara(host.store_mut());
    let area = Rect::new(0.0, 0.0, 1280.0, 800.0);
    let layout = HeroLayout::for_viewport(area);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = ph2d_text::TextSystem::without_system_fonts();
    let mut ctx = PaintCtx {
        host: &mut host,
        layout: &layout,
        slot: layout
            .slot_rects(SlotSet::of(Slot::Bottom))
            .get(Slot::Bottom),
        viewport: area,
        scene: &mut scene,
        text_system: &mut text,
    };
    let g = geom::Geom {
        region: Rect::new(0.0, 0.0, 900.0, 240.0),
        label_w: 100.0,
        time_area: Rect::new(100.0, 18.0, 800.0, 222.0),
        rows: Rect::new(100.0, 40.0, 800.0, 200.0),
        scrollbar: Rect::new(900.0, 40.0, 0.0, 200.0),
    };
    let view = TimeView {
        time_x: 100.0,
        right: 900.0,
        view_start: 0.0,
        px_per_s: 200.0,
    };
    let state = TimelinePanelState {
        tab: crate::tab::Tab::Arrange,
        ..TimelinePanelState::default()
    };
    ctx.scene.record();
    crate::stack_lane_paint::paint(&mut ctx, theme, &g, view, &state, &snap);
    let ops = ctx.scene.take_record();
    let hits = ctx.host.hit_index_mut().iter_registrations().collect();
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    (ops, hits)
}

fn cores(ops: &[DrawOp]) -> Vec<[f32; 4]> {
    ops.iter()
        .filter_map(|o| match o {
            DrawOp::Fill {
                brush: Brush::Solid(c),
                ..
            }
            | DrawOp::Stroke {
                brush: Brush::Solid(c),
                ..
            } => Some(c.components),
            _ => None,
        })
        .collect()
}

fn id_de(snap: &TimelineViewSnapshot, edge: u8) -> NodeId {
    crate::ids::timeline_strip_hit_id(0, snap.lanes[0].strips[0].id.0, edge)
}

fn arrasta(s: &mut WidgetStore, id: NodeId) {
    s.set_active(Some(id));
    s.begin_timeline_press(0.0, 0.0, false);
    s.note_timeline_pointer(40.0, 0.0);
}

/// Na faixa REAL: o canto e a alça de fade acendem no hover, ficam azuis no aperto, e o arrasto
/// junta o halo; e nenhum estado mexe num único hit rect (o seam).
#[test]
fn as_alcas_da_strip_mudam_de_estado_e_os_hits_nao() {
    let theme = Theme::Forge;
    let snap = snap_duas_pecas();
    let azul = acento(theme).components;
    let halo = acento(theme)
        .multiply_alpha(ph2d_tokens::TIMELINE_ALCA_HALO_ALPHA)
        .components;
    let (ops0, hits0) = pinta_faixa(theme, |_| {});
    let verde = resolve(ColorToken::Success, theme);
    let quente = cor(EstadoAlca::Hover, verde, theme).components;
    let playhead = resolve(ColorToken::TimelinePlayhead, theme);
    let fade_quente = cor(EstadoAlca::Hover, playhead, theme).components;
    assert!(!cores(&ops0).contains(&azul), "em repouso nada é azul");
    for (edge, quente) in [(5u8, quente), (EASE_IN_CODE, fade_quente)] {
        let id = id_de(&snap, edge);
        let (h, hits_h) = pinta_faixa(theme, |s| s.set_hot(Some(id)));
        assert!(cores(&h).contains(&quente), "edge {edge}: o hover acende");
        let (p, hits_p) = pinta_faixa(theme, |s| s.set_active(Some(id)));
        assert!(cores(&p).contains(&azul), "edge {edge}: o aperto é azul");
        assert!(
            !cores(&p).contains(&halo),
            "edge {edge}: sem halo no aperto"
        );
        let (d, hits_d) = pinta_faixa(theme, |s| arrasta(s, id));
        assert!(cores(&d).contains(&halo), "edge {edge}: o arrasto tem halo");
        for hits in [hits_h, hits_p, hits_d] {
            assert_eq!(hits, hits0, "edge {edge}: o estado não mexe nos hits");
        }
    }
}

const EASE_IN_CODE: u8 = crate::stack_ease_grip::EASE_IN;

/// As chaves do loop: a porta `pinta` sobre os rects REAIS dela, nos três estados.
#[test]
fn a_chave_do_loop_e_a_chave_da_folha_mudam_de_estado() {
    let theme = Theme::Forge;
    let azul = acento(theme).components;
    let halo = acento(theme)
        .multiply_alpha(ph2d_tokens::TIMELINE_ALCA_HALO_ALPHA)
        .components;
    let brace = resolve(ColorToken::TimelineLoopBrace, theme);
    let barra = [Rect::new(100.0, 0.0, 2.0, 22.0)];
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 400.0, 200.0));
    for e in [
        EstadoAlca::Repouso,
        EstadoAlca::Hover,
        EstadoAlca::Apertada,
        EstadoAlca::Arrasto,
    ] {
        let mut loop_ops = Vec::new();
        let mut chave_ops = Vec::new();
        host.paint_fn(&layout, |ctx| {
            ctx.scene.record();
            pinta(ctx, theme, e, brace, &barra, &[]);
            loop_ops = ctx.scene.take_record();
            ctx.scene.record();
            pinta_chave(ctx, theme, 50.0, 50.0, false, e);
            chave_ops = ctx.scene.take_record();
        });
        let (l, k) = (cores(&loop_ops), cores(&chave_ops));
        let chave = tinta(TimelineTinta::Chave, theme).components;
        let anel = tinta(TimelineTinta::FitaPropriedade, theme).components;
        match e {
            EstadoAlca::Repouso => {
                assert!(l.contains(&brace.components) && k.contains(&chave) && k.contains(&anel));
            }
            EstadoAlca::Hover => {
                assert!(!l.contains(&brace.components), "o loop acende");
                assert!(!k.contains(&anel), "o anel da chave acende");
            }
            EstadoAlca::Apertada => {
                assert!(l.contains(&azul) && k.contains(&azul));
                assert!(!l.contains(&halo) && !k.contains(&halo));
            }
            EstadoAlca::Arrasto => {
                assert!(l.contains(&azul) && k.contains(&azul));
                assert!(l.contains(&halo) && k.contains(&halo), "o halo do arrasto");
            }
        }
    }
}

#[test]
fn a_leitura_e_o_tempo_em_segundos_sem_menos_zero() {
    assert_eq!(leitura_str(1.25), "1.25");
    assert_eq!(leitura_str(-0.001), "0.00");
    let r = leitura_rect(30.0, 100.0, 50.0, false);
    assert!(r.x > 100.0, "à direita da alça");
    let r = leitura_rect(30.0, 100.0, 50.0, true);
    assert!(r.x + r.w < 100.0, "à esquerda da alça");
}
