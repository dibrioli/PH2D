//! Portões da PEÇA nos cartões (doc 16, onda 3): legível em toda cor e todo tema, a muda distinta da
//! viva, a mistura visível sobre cada peça, e a cor que a PINTURA dá a cada strip.
//!
//! As cores saem de [`cores`] e [`tons_da_mistura`] — as portas que o pintor lê —, compostas sobre a
//! fita da maquete (`FitaPropriedade`) em sRGB, como o Vello compõe; a distância é a OKLab.

use super::*;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_tokens::Theme;

/// A fita sob as peças nos cartões.
fn fita(theme: Theme) -> [f32; 3] {
    rgb(tinta(TimelineTinta::FitaPropriedade, theme))
}

fn rgb(c: Color) -> [f32; 3] {
    let [r, g, b, _] = c.components;
    [r, g, b]
}

/// `c` (com a opacidade dele) composto sobre `fundo`.
fn sobre(c: Color, fundo: [f32; 3]) -> [f32; 3] {
    let [r, g, b, a] = c.components;
    [
        r * a + fundo[0] * (1.0 - a),
        g * a + fundo[1] * (1.0 - a),
        b * a + fundo[2] * (1.0 - a),
    ]
}

fn dist(x: [f32; 3], y: [f32; 3]) -> f64 {
    let lab = |c: [f32; 3]| {
        let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let (l, ch, h) = ph2d_tokens::srgb_to_oklch(u(c[0]), u(c[1]), u(c[2]));
        (l, ch * h.to_radians().cos(), ch * h.to_radians().sin())
    };
    let (p, q) = (lab(x), lab(y));
    ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) + (p.2 - q.2).powi(2)).sqrt()
}

/// As seis cores de peça: as cinco de clipe e a do container.
fn todas() -> Vec<TimelineTinta> {
    let mut v = PECAS.to_vec();
    v.push(TimelineTinta::PecaContainer);
    v
}

fn peca(tinta: TimelineTinta, theme: Theme, muda: bool) -> PecaDesenho<'static> {
    PecaDesenho {
        tinta,
        icon: IconId::Film,
        nome: "Walk",
        extra: None,
        muda,
        escolhida: false,
        theme,
    }
}

/// O fundo da peça em cima, no meio e em baixo, sobre a fita.
fn fundos(p: &PecaDesenho<'_>) -> [[f32; 3]; 3] {
    let c = cores(p);
    let (topo, pe) = (sobre(c.topo, fita(p.theme)), sobre(c.pe, fita(p.theme)));
    let meio = [0, 1, 2].map(|i| (topo[i] + pe[i]) * 0.5);
    [topo, meio, pe]
}

/// ⭐ **O nome lê-se sobre o fundo de toda peça, em todo tema** — do topo ao pé do degradê.
/// Medido 10/11 (12 temas × 6 cores × 3 alturas): o pior é 0,335 (Forge, `PecaB` — o amarelo, no
/// topo); o piso fica logo abaixo.
#[test]
fn o_nome_da_peca_le_se_em_toda_cor_e_todo_tema() {
    let (mut pior, mut onde) = (f64::MAX, String::new());
    for theme in Theme::ALL {
        for t in todas() {
            let p = peca(t, theme, false);
            let texto = rgb(cores(&p).texto);
            for f in fundos(&p) {
                let d = dist(texto, f);
                if d < pior {
                    (pior, onde) = (d, format!("{theme:?} {t:?}"));
                }
            }
        }
    }
    eprintln!("pior nome × fundo: {pior:.3} ({onde})");
    assert!(pior >= PISO_NOME, "o nome a {pior:.3} do fundo em {onde}");
}
const PISO_NOME: f64 = 0.33;

/// ⭐ **A peça de uma faixa muda lê-se diferente da mesma peça viva** (`.c13st.mu`: tudo a 40 %).
/// Medido 10/11, no meio do degradê: o pior é 0,152 (Sunstone, `PecaC`).
#[test]
fn a_peca_muda_le_se_diferente_da_viva() {
    let (mut pior, mut onde) = (f64::MAX, String::new());
    for theme in Theme::ALL {
        for t in todas() {
            let (viva, muda) = (
                fundos(&peca(t, theme, false)),
                fundos(&peca(t, theme, true)),
            );
            let d = dist(viva[1], muda[1]);
            if d < pior {
                (pior, onde) = (d, format!("{theme:?} {t:?}"));
            }
        }
    }
    eprintln!("pior viva × muda: {pior:.3} ({onde})");
    assert!(pior >= PISO_MUDA, "a muda a {pior:.3} da viva em {onde}");
}
const PISO_MUDA: f64 = 0.15;

/// ⭐ **As listras da mistura leem-se sobre toda peça e sobre a fita** (a faixa de fade de fora mora
/// no vão ao lado do corpo), em todo tema: a listra clara contra o vão entre listras. Medido 10/11:
/// o pior é 0,178 (Forge, sobre `PecaB`).
#[test]
fn as_listras_da_mistura_leem_se_sobre_toda_peca() {
    let (mut pior, mut onde) = (f64::MAX, String::new());
    for theme in Theme::ALL {
        let (listra, fundo, _) = tons_da_mistura(theme, false);
        let bases = todas()
            .into_iter()
            .map(|t| (format!("{t:?}"), fundos(&peca(t, theme, false))[1]))
            .chain([("fita".to_owned(), fita(theme))]);
        for (nome, base) in bases {
            let vao = sobre(fundo, base);
            let d = dist(sobre(listra, vao), vao);
            if d < pior {
                (pior, onde) = (d, format!("{theme:?} {nome}"));
            }
        }
    }
    eprintln!("pior listra × vão: {pior:.3} ({onde})");
    assert!(pior >= PISO_LISTRA, "a listra a {pior:.3} do vão em {onde}");
}
const PISO_LISTRA: f64 = 0.17;

fn strip(id: u64, nome: &str, container: Option<usize>) -> StripView {
    StripView {
        id: ph2d_timeline::StripId(id),
        clip_name: nome.into(),
        container,
        t_start: 0.0,
        t_end: 1.0,
        blend_in: 0.0,
        blend_out: 0.0,
        lead_in: 0.0,
        lead_out: 0.0,
        ease_locked_in: false,
        ease_locked_out: false,
        curve_in: None,
        curve_out: None,
        seam: None,
        loop_mode: ph2d_timeline::StripLoop::Once,
        speed: 1.0,
        marks: [0.0; 4],
    }
}

/// Pinta `s` com o pintor REAL e devolve o primeiro preenchimento em degradê (o fundo da peça).
fn fundo_pintado(snap: &TimelineViewSnapshot, s: &StripView) -> Option<String> {
    use ph2d_editor_core::screens::hero::HeroLayout;
    use ph2d_editor_core::screens::slot::{Slot, SlotSet};
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
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
    let view = crate::graph::TimeView {
        time_x: 100.0,
        right: 900.0,
        view_start: 0.0,
        px_per_s: 200.0,
    };
    ctx.scene.record();
    let body = Rect::new(100.0, 60.0, 200.0, 22.0);
    let t = tinta_do_strip(s, snap);
    crate::strip_paint::paint_strip(
        &mut ctx,
        Theme::Forge,
        view,
        body,
        s,
        false,
        0,
        t,
        crate::strip_paint::Fase::Tudo,
    );
    ctx.scene.take_record().into_iter().find_map(|op| match op {
        ph2d_vector::DrawOp::Fill {
            brush: ph2d_vector::Brush::Gradient(g),
            ..
        } => Some(format!("{:?}", g.stops)),
        _ => None,
    })
}

/// ⭐ **A cor que a PINTURA dá a cada strip** (doc 16): a strip de container veste a do container, a
/// de um clipe a do clipe pela ordem dos chips (`snap.clips`, o clipe `i` em `PECAS[i % 5]`), e duas
/// strips do MESMO clipe a mesma. `PH2D_CARDS=0` não pinta degradê nenhum (a strip de sempre).
/// *Mutação: `tinta_do_strip` devolver sempre `PECAS[0]` ⇒ vermelho no container; ignorar a ordem
/// ⇒ vermelho no sexto clipe.*
#[test]
fn a_strip_de_container_e_a_de_clipe_vestem_cores_diferentes_e_o_mesmo_clipe_a_mesma() {
    let snap = TimelineViewSnapshot {
        clips: ["Idle", "Walk", "Jump", "Swing", "Shake", "Run"]
            .map(str::to_owned)
            .to_vec(),
        ..TimelineViewSnapshot::default()
    };
    force_cards_on_this_thread(Some(true));
    let walk_a = fundo_pintado(&snap, &strip(1, "Walk", None));
    let walk_b = fundo_pintado(&snap, &strip(2, "Walk", None));
    let jump = fundo_pintado(&snap, &strip(3, "Jump", None));
    let cont = fundo_pintado(&snap, &strip(4, "Run & jump", Some(0)));
    force_cards_on_this_thread(Some(false));
    let classico = fundo_pintado(&snap, &strip(5, "Walk", None));
    force_cards_on_this_thread(None);
    assert!(walk_a.is_some(), "a peça nos cartões pinta-se em degradê");
    assert_eq!(walk_a, walk_b, "duas strips do mesmo clipe, a mesma cor");
    assert_ne!(walk_a, jump, "dois clipes, duas cores");
    assert_ne!(walk_a, cont, "o container não veste a cor de um clipe");
    assert_eq!(classico, None, "o clássico não muda");
    assert_eq!(tinta_do_strip(&strip(6, "Walk", None), &snap), PECAS[1]);
    assert_eq!(tinta_do_strip(&strip(7, "Run", None), &snap), PECAS[0]);
    assert_eq!(
        tinta_do_strip(&strip(8, "Walk", Some(2)), &snap),
        TimelineTinta::PecaContainer
    );
    let fora = indice_do_clipe(&snap, "Not a clip");
    assert_eq!(
        fora,
        indice_do_clipe(&snap, "Not a clip"),
        "o hash é estável"
    );
}

/// O rótulo de uma strip mora no vão entre as faixas de fade de DENTRO; as de fora não o tocam, e um
/// vão pequeno demais devolve o corpo inteiro.
#[test]
fn o_rotulo_mora_no_vao_entre_os_fades_de_dentro() {
    use crate::stack_ease_grip::BAND_OUT;
    let body = Rect::new(100.0, 0.0, 200.0, 22.0);
    let dentro = [
        (BAND_IN, Rect::new(100.0, 0.0, 40.0, 22.0)),
        (BAND_OUT, Rect::new(260.0, 0.0, 40.0, 22.0)),
        (BAND_IN, Rect::new(60.0, 0.0, 40.0, 22.0)),
    ];
    assert_eq!(vao_livre(body, dentro.into_iter()), (140.0, 260.0));
    let tudo = [(BAND_IN, Rect::new(100.0, 0.0, 190.0, 22.0))];
    assert_eq!(vao_livre(body, tudo.into_iter()), (100.0, 300.0));
}
