//! Portões da faixa «Markers» (maquete B6): o nome vira para a esquerda só quando não cabe à direita.

use super::*;
use ph2d_vector::Shape;

/// ⭐ **O nome vira à esquerda perto do fim** (`.c13ml.rl`: «land» no fim do clipe), nunca antes: com
/// lugar à direita fica à direita (a `DX` do centro do chip); sem lugar, acaba a `DX` antes do chip.
/// *Mutação: virar sempre ⇒ reprova.*
#[test]
fn o_nome_vira_a_esquerda_so_quando_nao_cabe_a_direita() {
    let xs = [100.0, 400.0, 790.0];
    let borda = (90.0, 800.0);
    assert_eq!(
        lugar_do_nome(&xs, 0, borda, 40.0),
        (100.0 + DX, 400.0 - DX - 100.0 - DX)
    );
    let (x, w) = lugar_do_nome(&xs, 2, borda, 40.0);
    assert_eq!((x, w), (790.0 - DX - 40.0, 40.0), "land vira à esquerda");
    let (x, _) = lugar_do_nome(&xs, 1, borda, 40.0);
    assert!(x > 400.0, "top cabe à direita e fica lá");
}

const PX_S: f64 = 97.3;

fn snap() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        keys_mode: true,
        loop_range: Some((0.0, 5.0)),
        markers: vec![
            (0.0, "takeoff".to_string(), Some("jump".to_string())),
            (1.1, "top".to_string(), None),
            (2.03, "land".to_string(), Some("land".to_string())),
        ],
        ..TimelineViewSnapshot::default()
    }
}

/// Pinta a faixa nos cartões com `prepara` no store; devolve as ops, a vista, o `y` e o store.
fn pinta_faixa(
    prepara: impl Fn(&mut ph2d_editor_core::interaction::WidgetStore),
) -> (Vec<ph2d_vector::DrawOp>, TimeView, f32, Option<[u8; 4]>) {
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_editor_core::screens::hero::HeroLayout;
    use ph2d_editor_core::screens::slot::{Slot, SlotSet};
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let snap = snap();
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    prepara(host.store_mut());
    let area = Rect::new(0.0, 0.0, 1280.0, 800.0);
    let layout = HeroLayout::for_viewport(area);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = ph2d_text::TextSystem::without_system_fonts();
    let g = geom::resolve(Rect::new(0.0, 0.0, 1200.0, 400.0), 40.0, 200.0, 120.0);
    let view = TimeView {
        time_x: g.time_area.x,
        right: g.time_area.x + g.time_area.w,
        view_start: -0.137,
        px_per_s: PX_S,
    };
    let y = g.rows.y + ROW_H_PX;
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
    ctx.scene.record();
    paint(&mut ctx, Theme::Forge, &g, view, Tab::Keys, &snap, y);
    let ops = ctx.scene.take_record();
    let id = ids::timeline_marker_hit_id(1);
    let tinta = host.store().timeline_cursor_tint(id);
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    (ops, view, y, tinta)
}

/// As caixas das ops preenchidas na cor `c`.
fn caixas(ops: &[ph2d_vector::DrawOp], c: ph2d_vector::Color) -> Vec<ph2d_vector::Rect> {
    ops.iter()
        .filter_map(|o| match o {
            ph2d_vector::DrawOp::Fill {
                brush: ph2d_vector::Brush::Solid(k),
                path,
                ..
            } if *k == c => Some(path.bounding_box()),
            _ => None,
        })
        .collect()
}

fn perto(a: ph2d_vector::Rect, b: ph2d_vector::Rect) -> bool {
    (a.x0 - b.x0).abs() < 0.5
        && (a.x1 - b.x1).abs() < 0.5
        && (a.y0 - b.y0).abs() < 0.5
        && (a.y1 - b.y1).abs() < 0.5
}

fn bbox(pts: &[(f32, f32)]) -> ph2d_vector::Rect {
    let xs = pts.iter().map(|p| f64::from(p.0));
    let ys = pts.iter().map(|p| f64::from(p.1));
    ph2d_vector::Rect::new(
        xs.clone().fold(f64::MAX, f64::min),
        ys.clone().fold(f64::MAX, f64::min),
        xs.fold(f64::MIN, f64::max),
        ys.fold(f64::MIN, f64::max),
    )
}

/// ⭐⭐ **O marcador é uma LINHA no tempo exato com a alça do fade no topo** (dono 10/11: o chip «não
/// mostra a posição exata»). Com zoom e início fracionários, a linha de cada marcador centra-se em
/// `view.x(t)` (± 0,5 px) e ocupa a faixa inteira; as setas são as da porta do fade
/// (`forma_da_alca_de_fade`) no topo da linha; em repouso, hover e aperto a cor é a de
/// `alca_estado::cor` a partir da tinta dos marcadores, e o cursor tinge-se dela.
/// *Mutação (10/11, sangrou): a linha 1 px à direita do tempo ⇒ «x exato» reprova.*
#[test]
fn a_linha_do_marcador_cai_no_tempo_exato_com_a_alca_do_fade() {
    use crate::alca_estado::{EstadoAlca, cor};
    let amarelo = resolve(ColorToken::TimelineMarker, Theme::Forge);
    let (ops, view, y, tinta) = pinta_faixa(|_| {});
    assert_eq!(tinta, None, "em repouso nada tinge o cursor");
    let fills = caixas(&ops, amarelo);
    for (t, _, _) in snap().markers {
        let x = view.x(t);
        let (barra, setas) = forma(x, y);
        let linha = fills
            .iter()
            .find(|b| (b.height() - f64::from(ROW_H_PX)).abs() < 0.5 && b.y0 >= f64::from(y) - 0.5)
            .copied();
        let linha = fills
            .iter()
            .copied()
            .filter(|b| (b.height() - f64::from(ROW_H_PX)).abs() < 0.5)
            .find(|b| ((b.x0 + b.x1) * 0.5 - f64::from(x)).abs() <= 0.5)
            .unwrap_or_else(|| panic!("a linha de t={t} em x={x}: {linha:?} {fills:?}"));
        assert!(
            (linha.y0 - f64::from(y)).abs() < 0.5,
            "a linha começa no topo da faixa"
        );
        let r = ph2d_vector::Rect::new(
            f64::from(barra.x),
            f64::from(barra.y),
            f64::from(barra.x + barra.w),
            f64::from(barra.y + barra.h),
        );
        assert!(perto(linha, r), "a linha é a barra da porta");
        for s in setas {
            let b = bbox(&s);
            assert!(fills.iter().any(|f| perto(*f, b)), "a seta da porta: {b:?}");
            assert!(
                b.y1 < f64::from(y + ROW_H_PX * 0.5),
                "a seta no TOPO da linha"
            );
        }
    }
    let id = ids::timeline_marker_hit_id(1);
    for (estado, prepara) in [(EstadoAlca::Hover, true), (EstadoAlca::Apertada, false)] {
        let (ops, view, y, tinta) = pinta_faixa(|s| {
            if prepara {
                s.set_hot(Some(id));
            } else {
                s.set_active(Some(id));
            }
        });
        let c = cor(estado, amarelo, Theme::Forge);
        assert_ne!(c, amarelo, "{estado:?} muda a cor");
        let (barra, _) = forma(view.x(1.1), y);
        let d = estado.cresce();
        let alvo = ph2d_vector::Rect::new(
            f64::from(barra.x - d),
            f64::from(barra.y - d),
            f64::from(barra.x + barra.w + d),
            f64::from(barra.y + barra.h + d),
        );
        assert!(
            caixas(&ops, c).iter().any(|f| perto(*f, alvo)),
            "{estado:?}: a linha de «top» na cor do estado"
        );
        let k = amarelo.components;
        let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        assert_eq!(
            tinta,
            Some([u(k[0]), u(k[1]), u(k[2]), 255]),
            "{estado:?}: o cursor"
        );
    }
}

/// ⭐ **A alça do marcador não se confunde com a do fade e lê-se em todo tema.** Pisos MEDIDOS em
/// `Theme::ALL` (OKLab; ver o `eprintln!`): marcador × fade (`TimelinePlayhead`) e marcador × fundo da
/// lista (o mais perto de `Bg1`, `Bg2` — a faixa alternada — e `BgElev`).
#[test]
fn a_alca_do_marcador_nao_se_confunde_com_a_do_fade_em_todo_tema() {
    let lab = |c: ph2d_vector::Color| {
        let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let k = c.components;
        let (l, ch, h) = ph2d_tokens::srgb_to_oklch(u(k[0]), u(k[1]), u(k[2]));
        let h = h.to_radians();
        (l, ch * h.cos(), ch * h.sin())
    };
    let dist = |a, b| {
        let (x, y) = (lab(a), lab(b));
        ((x.0 - y.0).powi(2) + (x.1 - y.1).powi(2) + (x.2 - y.2).powi(2)).sqrt()
    };
    let mut pior = [(f64::MAX, Theme::Forge); 2];
    for theme in Theme::ALL {
        let m = resolve(ColorToken::TimelineMarker, theme);
        let d = [
            dist(m, resolve(ColorToken::TimelinePlayhead, theme)),
            [ColorToken::Bg1, ColorToken::Bg2, ColorToken::BgElev]
                .map(|f| dist(m, resolve(f, theme)))
                .into_iter()
                .fold(f64::MAX, f64::min),
        ];
        for (p, d) in pior.iter_mut().zip(d) {
            if d < p.0 {
                *p = (d, theme);
            }
        }
    }
    eprintln!("marcador × fade {:?} · × fundo {:?}", pior[0], pior[1]);
    for (p, piso) in pior.iter().zip([PISO_FADE, PISO_FUNDO]) {
        assert!(p.0 >= piso, "{:.3} < {piso} em {:?}", p.0, p.1);
    }
}

const PISO_FADE: f64 = 0.07; // medido 0,080 (Sunstone), 10/11 — a forma é a mesma, a tinta e a faixa dizem «marcador»
const PISO_FUNDO: f64 = 0.16; // medido 0,186 (Candy), 10/11
