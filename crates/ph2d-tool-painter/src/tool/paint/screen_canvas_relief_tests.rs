//! Os gates do RELEVO na tela da vista 3D (`docs/3D/29`, D2–D4). A cor sai
//! SEM a luz 2D e a espessura sai crua, em píxeis, para a escultura a pousar
//! na peça. ⚠️ Mora em `paint/` (declarado no `impasto_live.rs`) porque arma o
//! pincel pelo `PaintState`, que só é visível daqui.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

const W: u32 = 64;
const H: u32 = 48;

fn cp(p: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um pincel de impasto opaco — o `impasto_canvas` dos gates do corpo, sem a
/// fonte (quem a dá é a tela da vista ou o `set_source`).
fn impasto(t: &mut PainterTool) {
    let b = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        impasto: true,
        impasto_depth: 0.5,
        impasto_smoothing: 0.0,
        impasto_body: 1.0,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
}

fn traco(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp([10.0 + 3.0 * k as f32, 24.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([40.0, 24.0], PointerPhase::Up));
}

/// ⭐⭐⭐ **GATE — na tela da vista a cor sai SEM a luz 2D; numa sprite, com.**
///
/// ⚠️ O CONTROLO é a mesma pincelada numa sprite: lá o passe de luz corre e a
/// drenagem DIFERE dos píxeis que o pincel pousou. Sem ele, um relevo que nem
/// chegasse a existir passaria a metade da tela da vista.
#[test]
fn na_tela_da_vista_a_cor_sai_sem_a_luz_2d() {
    // (1) CONTROLO: numa sprite o impasto acende a drenagem.
    let mut s = PainterTool::default();
    s.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    impasto(&mut s);
    traco(&mut s);
    assert!(s.impasto_visible(), "CONTROLO: numa sprite o relevo acende");
    let (lit, _, _) = s.drain_preview_arc().expect("o traço sujou a sprite");
    assert_ne!(
        lit.as_slice(),
        s.canvas_rgba.as_slice(),
        "CONTROLO: numa sprite a drenagem leva a luz 2D"
    );

    // (2) Na tela da vista, a drenagem É o que o pincel pousou.
    let mut t = PainterTool::default();
    impasto(&mut t);
    assert!(t.bind_screen_canvas(W, H));
    traco(&mut t);
    assert!(!t.impasto_visible(), "a luz 2D ligou na tela da vista");
    let f = t.take_screen_canvas().expect("o traço sujou a tela");
    assert_eq!(
        f.rgba.as_slice(),
        t.canvas_rgba.as_slice(),
        "a tela da vista levou a luz 2D para a peça"
    );
}

/// ⭐⭐⭐ **GATE — a espessura sai em PÍXEIS, na janela pedida, pela lei do
/// passe de luz** (comprometido + traço × `impasto_depth` × `16`).
#[test]
fn a_espessura_da_tela_sai_em_pixeis_na_janela() {
    let mut t = PainterTool::default();
    impasto(&mut t);
    assert!(
        t.screen_canvas_heights_in((0, 0, W, H)).is_none(),
        "sem tela da vista não há espessura a pousar"
    );
    assert!(t.bind_screen_canvas(W, H));
    assert!(
        t.screen_canvas_heights_in((0, 0, W, H)).is_none(),
        "CONTROLO: antes do traço nada tem relevo"
    );
    traco(&mut t);
    let id = t.layers.active().expect("a tela tem uma camada");
    let cheia = t.layer_height_view(id).expect("o traço deixou relevo");
    let prof = t.layers.get(id).expect("a camada existe").impasto_depth;

    let janela = (20, 20, 10, 6);
    let h = t.screen_canvas_heights_in(janela).expect("há relevo");
    assert_eq!(h.len(), 60, "a janela é 10×6");
    let mut algum = false;
    for j in 0..6u32 {
        for i in 0..10u32 {
            let k = ((20 + j) * W + 20 + i) as usize;
            let esperado = cheia[k] * prof * 16.0;
            assert_eq!(
                h[(j * 10 + i) as usize].to_bits(),
                esperado.to_bits(),
                "({i},{j})"
            );
            algum |= esperado > 0.0;
        }
    }
    assert!(algum, "a fixtura não contém relevo na janela");

    // Uma janela a sair da tela é recortada, nunca recusada.
    let borda = t
        .screen_canvas_heights_in((60, 44, 10, 10))
        .expect("há relevo");
    assert_eq!(borda.len(), 4 * 4);
}

/// ⭐⭐ **GATE — a espessura vai no MESMO quadro que a cor, à volta do
/// rectângulo mudado** — a janela cobre o rectângulo com a margem, e o que ela
/// leva é a porta da espessura sobre essa janela, ao bit.
#[test]
fn a_drenagem_leva_a_espessura_a_volta_do_rectangulo() {
    let mut t = PainterTool::default();
    impasto(&mut t);
    assert!(t.bind_screen_canvas(W, H));
    traco(&mut t);
    let f = t.take_screen_canvas().expect("o traço sujou a tela");
    let r = f.rect.expect("uma tela de uma camada devolve o rectângulo");
    let rel = f
        .relief
        .as_ref()
        .expect("um traço de impasto leva espessura");
    let (x, y, w, h) = rel.window;
    assert!(
        x + 2 <= r.0.max(2) && y + 2 <= r.1.max(2),
        "a janela {:?} não cobre a margem à esquerda/topo de {r:?}",
        rel.window
    );
    assert!(
        x + w >= (r.0 + r.2 + 2).min(W) && y + h >= (r.1 + r.3 + 2).min(H),
        "a janela {:?} não cobre a margem à direita/baixo de {r:?}",
        rel.window
    );
    assert_eq!(rel.px.len(), (w * h) as usize);
    assert_eq!(
        Some(&rel.px),
        t.screen_canvas_heights_in(rel.window).as_ref(),
        "a drenagem e a porta da espessura divergem"
    );

    // CONTROLO: sem impasto, a drenagem não leva espessura nenhuma.
    let mut c = PainterTool::default();
    impasto(&mut c);
    c.paint.brush.impasto = false;
    c.paint
        .brush_by_mode
        .iter_mut()
        .for_each(|b| b.impasto = false);
    assert!(c.bind_screen_canvas(W, H));
    traco(&mut c);
    let g = c.take_screen_canvas().expect("o traço sujou a tela");
    assert!(g.relief.is_none(), "um traço sem impasto levou espessura");
}

/// ⭐⭐ **GATE — A MEIO do traço a espessura já sai** — o artista vê o relevo
/// crescer enquanto pinta, antes de largar. ⚠️ Até largar, a espessura vive no
/// envelope do traço aberto e NÃO no comprometido; uma janela que lesse só o
/// comprometido entregava zero a pincelada inteira (foi uma mutação que
/// SOBREVIVEU ao gate de cima, que só lê depois do pen-up).
#[test]
fn a_meio_do_traco_a_espessura_ja_sai() {
    let mut t = PainterTool::default();
    impasto(&mut t);
    assert!(t.bind_screen_canvas(W, H));
    t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
    for k in 1..=6 {
        t.on_canvas_pointer(cp([10.0 + 3.0 * k as f32, 24.0], PointerPhase::Move));
    }
    let id = t.layers.active().expect("a tela tem uma camada");
    assert!(
        t.heights
            .get(&id)
            .is_none_or(|c| c.iter().all(|&h| h == 0.0)),
        "a fixtura não contém o fenómeno: o traço ainda aberto já comprometeu relevo"
    );
    let cheia = t.layer_height_view(id).expect("o traço aberto tem relevo");
    let prof = t.layers.get(id).expect("a camada existe").impasto_depth;
    let h = t
        .screen_canvas_heights_in((0, 0, W, H))
        .expect("há relevo a meio");
    let mut algum = false;
    for (k, (&got, &raw)) in h.iter().zip(&cheia).enumerate() {
        assert_eq!(got.to_bits(), (raw * prof * 16.0).to_bits(), "píxel {k}");
        algum |= got > 0.0;
    }
    assert!(algum, "a meio do traço a janela entregou zero");
}

/// ⭐ **GATE — «este traço molda o relevo?»** — o depósito só com o `Impasto`
/// ligado; a faca sempre; a borracha nunca. É a pergunta da voz da escultura
/// (`docs/3D/29`, D1), e uma resposta larga demais avisaria de um preço que o
/// traço não paga.
#[test]
fn o_traco_molda_o_relevo_so_quando_deixa_corpo() {
    let mut t = PainterTool::default();
    assert!(
        !t.stroke_shapes_relief(),
        "um pincel sem Impasto não molda relevo"
    );
    impasto(&mut t);
    assert!(t.stroke_shapes_relief(), "o depósito com Impasto molda");
    t.paint.eraser = true;
    assert!(!t.stroke_shapes_relief(), "a borracha não molda");
    t.paint.eraser = false;
    let mut f = PainterTool::default();
    f.set_impasto_tool(super::super::impasto_tool::IMPASTO_TOOL_KNIFE);
    assert!(f.stroke_shapes_relief(), "a faca molda o corpo que lá está");
}

/// ⭐⭐⭐ **GATE — o CORPO sai com a espessura, e é ZERO onde ela transborda a
/// tinta** (report do dono de 01/10: *«o traço tem um relevo indesejado na
/// borda»*). O assentamento alisa a espessura e ela ESPALHA-SE para fora da
/// cor; a peça recebia-a sem saber que ali não há tinta.
///
/// ⚠️ A contagem do halo é o CONTROLO de que a fixtura contém o fenómeno (há
/// píxeis com altura e sem cor); sem ela o gate passaria por vácuo num traço
/// cujo alisamento não transbordasse. E a régua é o relevo que a LUZ lê
/// (`altura × corpo`, o que o shader da peça faz), não «corpo exactamente
/// zero»: a cobertura e o alfa de 8 bits não arredondam no mesmo sítio.
#[test]
fn o_corpo_e_zero_onde_a_espessura_transborda_a_tinta() {
    use super::super::media::PaintMedia;
    const W2: u32 = 160;
    const H2: u32 = 120;
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Impasto);
    t.paint.brush.radius_px = 10.0;
    t.paint
        .brush_by_mode
        .iter_mut()
        .for_each(|b| b.radius_px = 10.0);
    assert!(t.bind_screen_canvas(W2, H2));
    t.on_canvas_pointer(cp([20.0, 30.0], PointerPhase::Down));
    for k in 1..=40 {
        let s = k as f32 / 40.0;
        let p = [20.0 + 110.0 * s, 30.0 + 60.0 * s + 20.0 * (s * 6.0).sin()];
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([130.0, 90.0], PointerPhase::Up));
    let rel = t
        .screen_canvas_relief_in((0, 0, W2, H2))
        .expect("o traço deixou relevo");
    assert_eq!(rel.cover.len(), rel.px.len());
    let rgba = &t.canvas_rgba;
    let (mut halo, mut com_cor, mut halo_sem_corpo) = (0usize, 0usize, 0usize);
    let (mut halo_h, mut halo_aceso) = (0.0f32, 0.0f32);
    for (i, (&h, &c)) in rel.px.iter().zip(&rel.cover).enumerate() {
        let a = rgba[i * 4 + 3];
        if h > 1e-4 && a == 0 {
            halo += 1;
            halo_sem_corpo += usize::from(c == 0.0);
            halo_h = halo_h.max(h);
            halo_aceso = halo_aceso.max(h * c);
        }
        if a > 0 && h > 1e-4 {
            com_cor += 1;
            assert!(c > 0.0, "píxel {i}: tinta com espessura saiu sem corpo");
        }
    }
    assert!(
        halo > 0,
        "a fixtura não contém o halo (nenhuma altura sem cor)"
    );
    // Medido (01/10): `1 894` píxeis de halo, `4` com corpo — no máximo
    // `3/255`, a borda onde a cobertura não chega a um byte de alfa — e o
    // relevo que a luz LÊ ali (`h × corpo`) é `1,2 %` do cru. As barras saem
    // dessa tabela com folga para a ordem da composição.
    assert!(
        halo_sem_corpo * 100 >= halo * 99,
        "{halo_sem_corpo} de {halo} píxeis de halo saíram sem corpo"
    );
    assert!(
        halo_aceso <= 0.05 * halo_h,
        "o halo acende {halo_aceso} px de {halo_h} px crus"
    );
    assert!(com_cor > 0, "a fixtura não tem tinta com espessura");
}

/// ⭐⭐⭐ **GATE — a SEMENTE do relevo volta pela janela** (`docs/3D/29` §6): o
/// que se semeia em píxeis sai em píxeis, a menos da ida-e-volta pela
/// profundidade da camada, e o corpo sai `byte/255` AO BIT. ⚠️ É por esse ULP
/// que a escultura guarda a semente LIDA e não a enviada.
#[test]
fn a_semente_do_relevo_volta_pela_janela() {
    let n = (W * H) as usize;
    let px: Vec<f32> = (0..n).map(|k| (k % 13) as f32 * 0.75).collect();
    let cover: Vec<u8> = (0..n).map(|k| (k * 7 % 256) as u8).collect();
    let mut t = PainterTool::default();
    impasto(&mut t);
    assert!(
        !t.seed_screen_canvas_relief(&px, &cover),
        "fora da tela da vista não se semeia"
    );
    assert!(t.bind_screen_canvas(W, H));
    assert!(
        !t.seed_screen_canvas_relief(&px[1..], &cover),
        "um tamanho errado é recusado"
    );
    assert!(t.seed_screen_canvas_relief(&px, &cover));
    let lida = t
        .screen_canvas_relief_in((0, 0, W, H))
        .expect("a semente tem relevo");
    for k in 0..n {
        let tol = 1e-5 * px[k].abs().max(1.0);
        assert!(
            (lida.px[k] - px[k]).abs() <= tol,
            "píxel {k}: {} contra {}",
            lida.px[k],
            px[k]
        );
        assert_eq!(
            lida.cover[k].to_bits(),
            (f32::from(cover[k]) / 255.0).to_bits(),
            "píxel {k}: o corpo"
        );
    }
}

/// Pinta `tool` a direito sobre a tela `W×H`, semeada ou não com um CUME, e
/// devolve a maior mudança de espessura (px) que a tela mostrou.
fn ferramenta_sobre_cume(tool: u8, semeado: bool) -> f32 {
    use super::super::media::PaintMedia;
    let n = (W * H) as usize;
    let cume: Vec<f32> = (0..n)
        .map(|k| {
            let (x, y) = ((k as u32 % W) as f32, (k as u32 / W) as f32);
            let d = ((y - 24.0) / 6.0).powi(2);
            if (8.0..56.0).contains(&x) {
                12.0 * (-d).exp()
            } else {
                0.0
            }
        })
        .collect();
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Impasto);
    assert!(t.bind_screen_canvas(W, H));
    assert!(t.seed_screen_canvas(vec![200u8; n * 4]));
    if semeado {
        let cover: Vec<u8> = cume
            .iter()
            .map(|&h| if h > 0.5 { 255 } else { 0 })
            .collect();
        assert!(t.seed_screen_canvas_relief(&cume, &cover));
    }
    let _ = t.take_screen_canvas();
    t.set_impasto_tool(tool);
    t.paint.brush.radius_px = 10.0;
    t.paint
        .brush_by_mode
        .iter_mut()
        .for_each(|b| b.radius_px = 10.0);
    let antes = t.screen_canvas_heights_in((0, 0, W, H));
    t.on_canvas_pointer(cp([14.0, 24.0], PointerPhase::Down));
    for k in 1..=16 {
        let p = [14.0 + 2.0 * k as f32, 22.0 + (k % 3) as f32];
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([48.0, 24.0], PointerPhase::Up));
    let depois = t.screen_canvas_heights_in((0, 0, W, H));
    match (&antes, &depois) {
        (Some(a), Some(d)) => a
            .iter()
            .zip(d)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f32::max),
        (None, Some(d)) => d.iter().copied().fold(0.0, f32::max),
        _ => 0.0,
    }
}

/// ⭐⭐⭐ **GATE — o alisar e a faca TRABALHAM sobre o relevo que a peça já
/// tem** (report do dono de 01/10: *«smooth, knife e outras tools não funcionam
/// no relevo»*). Com a tela semeada com a espessura da peça, os dois mudam-na;
/// o CONTROLO é a mesma pincelada sem semente, onde o alisar não tem o que
/// alisar — é exactamente o que o dono via.
#[test]
fn sobre_o_relevo_semeado_o_alisar_e_a_faca_trabalham() {
    use super::super::impasto_tool::{IMPASTO_TOOL_KNIFE, IMPASTO_TOOL_SCULPT_BASE};
    let liso = ferramenta_sobre_cume(IMPASTO_TOOL_SCULPT_BASE, false);
    assert!(
        liso < 1e-3,
        "CONTROLO: sem semente o alisar mexeu {liso} px"
    );
    for (nome, tool) in [
        ("smooth", IMPASTO_TOOL_SCULPT_BASE),
        ("knife", IMPASTO_TOOL_KNIFE),
    ] {
        let m = ferramenta_sobre_cume(tool, true);
        assert!(m > 0.5, "{nome}: sobre o relevo semeado mudou só {m} px");
    }
}
