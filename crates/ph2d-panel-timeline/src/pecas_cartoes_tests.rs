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

/// ⭐ **«Além do fim» lê-se sobre toda peça e sobre a fita, e apaga a peça** (dono 10/11): a listra
/// `\` contra o véu por baixo dela ≥ o piso da mistura, e a peça velada contra a mesma peça viva ≥ o
/// piso medido. Medido 10/11 (12 temas × 6 peças + fita): listra 0,262 (Sunstone `PecaB`), véu
/// 0,166 (Sunstone `PecaC`); os pisos ficam logo abaixo.
/// *Mutação: véu a 0 ⇒ a peça velada é a viva e reprova; listra a 0,1 ⇒ reprova sobre a fita.*
#[test]
fn alem_do_fim_le_se_sobre_toda_peca_e_apaga_a_peca() {
    let (mut listra_pior, mut veu_pior) = ((f64::MAX, String::new()), (f64::MAX, String::new()));
    for theme in Theme::ALL {
        let (veu, listra) = crate::ruler::fim::tons_alem(theme);
        let pecas = todas()
            .into_iter()
            .map(|t| (format!("{t:?}"), fundos(&peca(t, theme, false))[1]));
        for (nome, base) in pecas.clone().chain([("fita".to_owned(), fita(theme))]) {
            let velada = sobre(veu, base);
            let d = dist(sobre(listra, velada), velada);
            if d < listra_pior.0 {
                listra_pior = (d, format!("{theme:?} {nome}"));
            }
        }
        for (nome, base) in pecas {
            let d = dist(sobre(veu, base), base);
            if d < veu_pior.0 {
                veu_pior = (d, format!("{theme:?} {nome}"));
            }
        }
    }
    eprintln!(
        "pior listra além × véu: {:.3} ({})",
        listra_pior.0, listra_pior.1
    );
    eprintln!(
        "pior peça velada × viva: {:.3} ({})",
        veu_pior.0, veu_pior.1
    );
    assert!(
        listra_pior.0 >= PISO_LISTRA_ALEM.max(PISO_LISTRA),
        "listra além a {listra_pior:?}"
    );
    assert!(veu_pior.0 >= PISO_VEU, "véu a {veu_pior:?}");
}
const PISO_VEU: f64 = 0.16;
const PISO_LISTRA_ALEM: f64 = 0.25;

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
        volta: 0.0,
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
    ops_pintadas(snap, s, false)
        .into_iter()
        .find_map(|op| match op {
            ph2d_vector::DrawOp::Fill {
                brush: ph2d_vector::Brush::Gradient(g),
                ..
            } => Some(format!("{:?}", g.stops)),
            _ => None,
        })
}

/// As ops que o pintor REAL grava para `s` (escolhida ou não), a 200 px/s a partir de `x = 100`.
fn ops_pintadas(
    snap: &TimelineViewSnapshot,
    s: &StripView,
    escolhida: bool,
) -> Vec<ph2d_vector::DrawOp> {
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
        (false, escolhida),
        0,
        t,
        crate::strip_paint::Fase::Tudo,
    );
    ctx.scene.take_record()
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

/// Os traços (ou preenchimentos) sólidos de cor `c` em `ops`.
fn tracos_de(ops: &[ph2d_vector::DrawOp], c: Color) -> usize {
    use ph2d_vector::{Brush, DrawOp};
    ops.iter()
        .filter(|op| match op {
            DrawOp::Stroke {
                brush: Brush::Solid(k),
                ..
            }
            | DrawOp::Fill {
                brush: Brush::Solid(k),
                ..
            } => *k == c,
            _ => false,
        })
        .count()
}

/// ⭐ **A peça escolhida acende o anel** (doc 17 item 1, maquete B4 `.c13st.s`): o anel claro por
/// dentro e o do foco por fora; a não escolhida não pinta nenhum dos dois.
/// *Mutação: `paint_strip` ignorar `escolhida` ⇒ vermelho.*
#[test]
fn a_peca_escolhida_acende_o_anel_e_a_outra_nao() {
    let snap = TimelineViewSnapshot::default();
    let s = strip(1, "Walk", None);
    force_cards_on_this_thread(Some(true));
    let acesa = ops_pintadas(&snap, &s, true);
    let apagada = ops_pintadas(&snap, &s, false);
    force_cards_on_this_thread(None);
    let p = peca_do_strip(&s, tinta_do_strip(&s, &snap), false, Theme::Forge);
    let p = PecaDesenho {
        escolhida: true,
        ..p.unwrap_or_else(|| unreachable!())
    };
    let [dentro, fora] = cores(&p).anel.expect("a escolhida tem anel");
    assert_eq!(tracos_de(&acesa, fora), 1, "o anel do foco, por fora");
    assert!(tracos_de(&acesa, dentro) >= 1, "o anel claro, por dentro");
    assert_eq!(tracos_de(&apagada, fora), 0, "sem escolha, sem anel");
}

/// Uma strip de `0` a `fim` s, uma volta de `volta` s.
fn em_loop(modo: ph2d_timeline::StripLoop, fim: f64, volta: f64) -> StripView {
    StripView {
        t_end: fim,
        volta,
        loop_mode: modo,
        ..strip(1, "Shake", None)
    }
}

fn vista(px_per_s: f64) -> crate::graph::TimeView {
    crate::graph::TimeView {
        time_x: 100.0,
        right: 2000.0,
        view_start: 0.0,
        px_per_s,
    }
}

/// ⭐ **As voltas do loop** (doc 17 item 4): um risco a cada recomeço — quatro voltas de 0,25 s em
/// 1 s dão 3 riscos em 0,25/0,5/0,75 s; o Ping-Pong risca cada inversão nos mesmos sítios; o dobro da
/// velocidade (meia volta) dá o dobro (mais um); `Once` (volta 0) nada; e abaixo do piso de px nada.
/// *Mutação: `voltas_x` devolver sempre vazio ⇒ vermelho.*
#[test]
fn as_voltas_riscam_cada_recomeco_e_somem_abaixo_do_piso() {
    use ph2d_timeline::StripLoop as L;
    let v = vista(200.0);
    assert_eq!(
        voltas_x(&em_loop(L::Loop, 1.0, 0.25), v),
        vec![150.0, 200.0, 250.0]
    );
    assert_eq!(
        voltas_x(&em_loop(L::PingPong, 1.0, 0.25), v),
        vec![150.0, 200.0, 250.0]
    );
    assert_eq!(
        voltas_x(&em_loop(L::Loop, 1.0, 0.125), v).len(),
        7,
        "2× = 8 voltas, 7 riscos"
    );
    assert!(voltas_x(&em_loop(L::Once, 1.0, 0.0), v).is_empty());
    assert!(
        voltas_x(&em_loop(L::Loop, 1.0, 0.3), v).len() == 3,
        "3 voltas inteiras e um resto"
    );
    let piso = f64::from(ph2d_tokens::chrome::TIMELINE_VOLTA_MIN_PX);
    let junto = vista(piso / 0.25 - 1.0);
    assert!(
        voltas_x(&em_loop(L::Loop, 1.0, 0.25), junto).is_empty(),
        "abaixo do piso"
    );
    assert_eq!(
        voltas_x(&em_loop(L::Loop, 1.0, 0.25), vista(piso / 0.25)).len(),
        3,
        "no piso"
    );
}

/// Os riscos chegam à pintura nos cartões, na cor e opacidade do token.
#[test]
fn os_riscos_das_voltas_chegam_a_pintura() {
    let snap = TimelineViewSnapshot::default();
    let s = em_loop(ph2d_timeline::StripLoop::Loop, 1.0, 0.25);
    force_cards_on_this_thread(Some(true));
    let ops = ops_pintadas(&snap, &s, false);
    force_cards_on_this_thread(None);
    let cor = tinta(TimelineTinta::TextoCabecalho, Theme::Forge)
        .multiply_alpha(ph2d_tokens::chrome::TIMELINE_VOLTA_ALPHA);
    let pontos = tracos_de(&ops, cor);
    // 3 riscos, cada um pontilhado ao longo dos 22 px da peça (1,5 px de ponto, 1,5 px de vão).
    assert!(pontos >= 3 * 7, "os pontos dos 3 riscos: {pontos}");
}
