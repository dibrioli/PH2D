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
/// playhead), as chaves do loop, a chave da folha e a alça do marcador.
fn repousos(theme: Theme) -> [(&'static str, Color); 6] {
    [
        ("canto esticar", resolve(ColorToken::Success, theme)),
        ("canto cortar", resolve(ColorToken::Danger, theme)),
        (
            "fade / duração",
            resolve(ColorToken::TimelinePlayhead, theme),
        ),
        ("loop", resolve(ColorToken::TimelineLoopBrace, theme)),
        ("chave", tinta(TimelineTinta::Chave, theme)),
        ("marcador", resolve(ColorToken::TimelineMarker, theme)),
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

type Alvos = Vec<(NodeId, ph2d_editor_core::interaction::TimelineHitKind)>;

/// Pinta a faixa nos cartões com o store preparado por `prepara`: (gravação, hits registados).
fn pinta_faixa(
    theme: Theme,
    prepara: impl FnOnce(&mut WidgetStore),
) -> (Vec<DrawOp>, Vec<(NodeId, Rect)>, Alvos) {
    let (ops, hits, alvos, ()) = pinta_faixa_lendo(theme, true, prepara, |_| ());
    (ops, hits, alvos)
}

/// [`pinta_faixa`] com ou sem os `cartoes`, e `le` sobre o store DEPOIS da pintura.
fn pinta_faixa_lendo<R>(
    theme: Theme,
    cartoes: bool,
    prepara: impl FnOnce(&mut WidgetStore),
    le: impl FnOnce(&WidgetStore) -> R,
) -> (Vec<DrawOp>, Vec<(NodeId, Rect)>, Alvos, R) {
    pinta_faixa_de(theme, cartoes, &snap_duas_pecas(), prepara, le)
}

/// [`pinta_faixa_lendo`] sobre `snap` (a faixa muda, por exemplo).
fn pinta_faixa_de<R>(
    theme: Theme,
    cartoes: bool,
    snap: &TimelineViewSnapshot,
    prepara: impl FnOnce(&mut WidgetStore),
    le: impl FnOnce(&WidgetStore) -> R,
) -> (Vec<DrawOp>, Vec<(NodeId, Rect)>, Alvos, R) {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(cartoes));
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
    crate::stack_lane_paint::paint(&mut ctx, theme, &g, view, &state, snap);
    let ops = ctx.scene.take_record();
    let hits: Vec<(NodeId, Rect)> = ctx.host.hit_index_mut().iter_registrations().collect();
    let store = ctx.host.store();
    let alvos = hits
        .iter()
        .filter_map(|(id, _)| store.timeline_surface_at_id(*id).map(|(_, k)| (*id, k)))
        .collect();
    let lido = le(store);
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    (ops, hits, alvos, lido)
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
    let (ops0, hits0, _) = pinta_faixa(theme, |_| {});
    let verde = resolve(ColorToken::Success, theme);
    let quente = cor(EstadoAlca::Hover, verde, theme).components;
    let playhead = resolve(ColorToken::TimelinePlayhead, theme);
    let fade_quente = cor(EstadoAlca::Hover, playhead, theme).components;
    assert!(!cores(&ops0).contains(&azul), "em repouso nada é azul");
    for (edge, quente) in [(5u8, quente), (EASE_IN_CODE, fade_quente)] {
        let id = id_de(&snap, edge);
        let (h, hits_h, _) = pinta_faixa(theme, |s| s.set_hot(Some(id)));
        assert!(cores(&h).contains(&quente), "edge {edge}: o hover acende");
        let (p, hits_p, _) = pinta_faixa(theme, |s| s.set_active(Some(id)));
        assert!(cores(&p).contains(&azul), "edge {edge}: o aperto é azul");
        assert!(
            !cores(&p).contains(&halo),
            "edge {edge}: sem halo no aperto"
        );
        let (d, hits_d, _) = pinta_faixa(theme, |s| arrasta(s, id));
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

/// Pinta `pinta` nos cartões com o store preparado: (gravação, hits).
fn nos_cartoes(
    prepara: impl FnOnce(&mut WidgetStore),
    pinta: impl FnOnce(&mut PaintCtx),
) -> (Vec<DrawOp>, Vec<(NodeId, Rect)>) {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    prepara(host.store_mut());
    let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 800.0, 300.0));
    let mut ops = Vec::new();
    let hits = host.paint_fn(&layout, |ctx| {
        ctx.scene.record();
        pinta(ctx);
        ops = ctx.scene.take_record();
    });
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    (ops, hits)
}

/// Os três estados numa alça pintada por `pinta`: hover sai do repouso, aperto azul sem halo,
/// arrasto com halo; `repouso` é a cor que só o repouso pinta. Os hits nunca mudam.
fn tres_estados(
    nome: &str,
    repouso: [f32; 4],
    hover: impl Fn(&mut WidgetStore),
    aperta: impl Fn(&mut WidgetStore),
    arrasta: impl Fn(&mut WidgetStore),
    pinta: impl Fn(&mut PaintCtx),
) {
    let theme = Theme::Forge;
    let azul = acento(theme).components;
    let halo = acento(theme)
        .multiply_alpha(ph2d_tokens::TIMELINE_ALCA_HALO_ALPHA)
        .components;
    let (r, hits0) = nos_cartoes(|_| {}, &pinta);
    assert!(
        cores(&r).contains(&repouso) && !cores(&r).contains(&azul),
        "{nome}: repouso"
    );
    let (h, hits_h) = nos_cartoes(hover, &pinta);
    assert!(!cores(&h).contains(&repouso), "{nome}: o hover acende");
    let (p, hits_p) = nos_cartoes(aperta, &pinta);
    assert!(
        cores(&p).contains(&azul) && !cores(&p).contains(&halo),
        "{nome}: aperto azul"
    );
    let (d, hits_d) = nos_cartoes(arrasta, &pinta);
    assert!(cores(&d).contains(&halo), "{nome}: o arrasto tem halo");
    for hits in [hits_h, hits_p, hits_d] {
        assert_eq!(hits, hits0, "{nome}: o estado não mexe nos hits");
    }
}

fn regista_regua(s: &mut WidgetStore, state: ph2d_editor_core::widget::SliderState) {
    s.register(
        crate::ids::TIMELINE_RULER,
        ph2d_editor_core::interaction::InteractiveState::Slider {
            state,
            value: 0.0,
            orientation: ph2d_editor_core::widget::SliderOrientation::Horizontal,
        },
    );
}

/// O playhead, as bandeiras dos marcadores, o losango do Summary e o da linha de grupo: a mesma
/// porta, os mesmos três estados.
#[test]
fn o_playhead_os_marcadores_e_as_chaves_mestras_mudam_de_estado() {
    use ph2d_editor_core::widget::SliderState as S;
    let theme = Theme::Forge;
    let ruler = crate::ids::TIMELINE_RULER;
    let line = Rect::new(200.0, 0.0, 2.0, 200.0);
    tres_estados(
        "playhead",
        tinta(TimelineTinta::Cursor, theme).components,
        |s| s.set_hot(Some(ruler)),
        |s| {
            regista_regua(s, S::Normal);
            s.set_active(Some(ruler));
        },
        |s| regista_regua(s, S::Dragging),
        |ctx| playhead(ctx, theme, line, 1.0, 10.0),
    );
    let mut snap = TimelineViewSnapshot::default();
    snap.markers.push((1.0, "M".into(), None));
    let marca = crate::ids::timeline_marker_hit_id(0);
    let regiao = Rect::new(100.0, 0.0, 600.0, 22.0);
    let x = |t: f64| 100.0 + t as f32 * 100.0;
    tres_estados(
        "marcador",
        resolve(ColorToken::TimelineMarker, theme).components,
        |s| s.set_hot(Some(marca)),
        |s| s.set_active(Some(marca)),
        |s| arrasta(s, marca),
        |ctx| crate::ruler::paint_markers(ctx, theme, regiao, &x, &snap),
    );
    let mestre = crate::ids::timeline_summary_hit_id(7);
    let miolo = resolve(ColorToken::TimelineSummaryKey, theme);
    let anel = (2.0, resolve(ColorToken::BorderStrong, theme));
    tres_estados(
        "summary",
        miolo.components,
        |s| s.set_hot(Some(mestre)),
        |s| s.set_active(Some(mestre)),
        |s| arrasta(s, mestre),
        |ctx| losango_mestre(ctx, theme, mestre, (50.0, 50.0, 5.0), anel, miolo, 1.0),
    );
    let grupo = crate::ids::timeline_group_key_id(3, 7);
    let tok = ColorToken::TimelineKey;
    tres_estados(
        "grupo",
        tinta(TimelineTinta::FitaPropriedade, theme).components,
        |s| s.set_hot(Some(grupo)),
        |s| s.set_active(Some(grupo)),
        |s| arrasta(s, grupo),
        |ctx| chave_da_folha(ctx, theme, grupo, (50.0, 50.0), tok, 1.0),
    );
}

/// ⭐ O cursor pelo MESMO alvo do clique, na faixa REAL: as pontas e os fades pedem `↔`, o corpo a
/// mão aberta (fechada no aperto), e o fundo sem alvo fica na seta.
#[test]
fn o_cursor_da_faixa_sai_do_alvo_registado() {
    use ph2d_editor_core::doc_surface::PointerLook as L;
    use ph2d_editor_core::interaction::TimelineHitKind as K;
    let (_, _, alvos) = pinta_faixa(Theme::Forge, |_| {});
    let olha = |edge: u8, apertado: bool| {
        alvos
            .iter()
            .find(|(_, k)| matches!(k, K::Strip { edge: e, .. } if *e == edge))
            .unwrap_or_else(|| panic!("edge {edge} registado"))
            .1
            .pointer_look(
                apertado,
                ph2d_editor_core::panel::cards::ferramenta::cards_on(),
            )
    };
    for edge in [0u8, 1, 5, 6, EASE_IN_CODE] {
        assert_eq!(olha(edge, false), Some(L::ResizeEw), "edge {edge}");
    }
    assert_eq!(olha(2, false), Some(L::Grab));
    assert_eq!(olha(2, true), Some(L::Grabbing));
    assert_eq!(
        K::Lane.pointer_look(false, false),
        None,
        "o fundo da folha: a seta"
    );
    assert!(
        alvos.iter().all(|(_, k)| !matches!(k, K::Lane)),
        "a faixa não regista fundo"
    );
}

#[path = "alca_cursor_tests.rs"]
mod cursor;

#[path = "faixa_muda_tests.rs"]
mod faixa_muda;

/// Dono 13/11 («a seleção de keys é quase imperceptível»): a chave escolhida é azul e o anel dela é
/// o azul à opacidade da maquete. Em TODO tema, sobre as fitas onde ela se pinta (propriedade,
/// cabeçalho, Summary), o azul tem contraste ≥ 3:1 (WCAG UI; medido 3,78 no pior) e o anel composto
/// fica a ΔOK ≥ 0,15 do fundo (medido 0,224 no pior).
#[test]
fn a_chave_escolhida_tem_o_azul_e_o_anel_da_maquete_em_todo_tema() {
    use ph2d_tokens::TimelineTinta as T;
    let opaca = |c: Color| {
        let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let k = c.components;
        ph2d_tokens::Color {
            r: u(k[0]),
            g: u(k[1]),
            b: u(k[2]),
            a: 255,
        }
    };
    let sobre = |c: Color, f: Color| {
        let (a, b, t) = (c.components, f.components, c.components[3]);
        Color::new([
            b[0] + (a[0] - b[0]) * t,
            b[1] + (a[1] - b[1]) * t,
            b[2] + (a[2] - b[2]) * t,
            1.0,
        ])
    };
    let mut pior = (f64::MAX, f64::MAX);
    for theme in Theme::ALL {
        let azul = acento(theme);
        assert_eq!(azul, tinta(T::ChaveEscolhida, theme), "{theme:?}");
        assert_eq!(cor_do_anel(theme, true), azul, "aceso = o azul cheio");
        for fita in [T::FitaPropriedade, T::FitaCabecalho, T::SummaryFita] {
            let f = tinta(fita, theme);
            let corpo = opaca(azul).contrast_ratio(&opaca(f));
            let anel = dist(sobre(cor_do_anel(theme, false), f), f);
            eprintln!("{theme:?} {fita:?}: corpo {corpo:.2}:1 · anel ΔOK {anel:.3}");
            assert!(
                corpo >= 3.0,
                "{theme:?} {fita:?}: a escolhida some ({corpo:.2}:1)"
            );
            pior = (pior.0.min(corpo), pior.1.min(anel));
        }
    }
    eprintln!("pior: corpo {:.2}:1 · anel ΔOK {:.3}", pior.0, pior.1);
    assert!(pior.1 >= 0.15, "o anel some no fundo: ΔOK {:.3}", pior.1);
}
