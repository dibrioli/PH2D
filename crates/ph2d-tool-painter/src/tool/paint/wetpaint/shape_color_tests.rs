//! ⭐ **A Shape Color no Wet Paint** (prompt 41, lista 26 A7, plano 27 W6): as duas camadas (Texture
//! Color · Falloff Color) dão a cor de cada CÉLULA do depósito — e, desligadas, o traço de hoje ao byte.
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::texture::TextureKind;

pub(super) const LADO: u32 = 128;

pub(super) fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

pub(super) fn verde_vermelho() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// A tela branca do Wet Paint com o pincel azul de raio 14 e o relógio da água fixo.
pub(super) fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t
}

/// As camadas da Shape Color com a rampa verde → vermelho; `None` = configuradas e DESLIGADAS.
pub(super) fn arma_rampa(t: &mut PainterTool, camadas: Option<(bool, bool)>) {
    t.set_shape_color_ramp(verde_vermelho());
    t.set_shape_falloff_ramp(verde_vermelho());
    t.set_shape_ramp_enabled(camadas.is_some());
    let (textura, queda) = camadas.unwrap_or((true, true));
    t.paint.shape_camadas.textura = textura;
    t.paint.shape_camadas.queda = queda;
    t.set_shape_falloff_ramp(verde_vermelho()); // re-assa as LUTs
}

/// Um traço recto de `de` a `ate` (4 px por evento, um quadro por evento) e `assenta` quadros depois.
pub(super) fn traco(t: &mut PainterTool, y: f32, de: f32, ate: f32, assenta: u32) {
    t.on_canvas_pointer(cp([de, y], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    let passos = ((ate - de) / 4.0).abs().round() as i32;
    let dir = (ate - de).signum();
    for k in 1..=passos {
        #[allow(clippy::cast_precision_loss)]
        t.on_canvas_pointer(cp([de + dir * 4.0 * k as f32, y], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([ate, y], PointerPhase::Up));
    for _ in 0..assenta {
        t.on_tick(1000.0 / 60.0);
    }
}

/// FNV-1a de 64 bits da tela — a impressão ao byte.
pub(super) fn impressao(px: &[u8]) -> u64 {
    px.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// As cenas da identidade: a ponta lisa, o padrão procedural (o canal `padrao`), a cápsula (Continuous),
/// as cerdas, a rampa configurada e DESLIGADA, o modo TOM (B&W, que já agia antes) e a brocha suja (a
/// recolha da cor da tela por um 2.º traço vermelho que cruza o azul ainda molhado).
pub(super) fn cena(n: usize) -> PainterTool {
    let mut t = tela();
    match n {
        0 => {}
        1 => t.set_brush_shape_kind(TextureKind::Voronoi.to_u8()),
        2 => {
            t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
            t.set_brush_spacing(1.5);
            t.set_brush_continuous_spacing(true);
        }
        3 => t.set_brush_shape_kind(TextureKind::Bristles.to_u8()),
        4 => {
            t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
            arma_rampa(&mut t, None);
        }
        5 => {
            t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
            arma_rampa(&mut t, Some((true, true)));
            t.paint.shape_color_ramp_bw = true;
        }
        6 => arma_rampa(&mut t, None),
        _ => unreachable!(),
    }
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    traco(&mut t, 64.0, 16.0, 112.0, if n == 6 { 4 } else { 60 });
    if n == 6 {
        t.set_brush_color_srgb8([220, 40, 30]);
        traco(&mut t, 70.0, 112.0, 16.0, 60);
    }
    t
}

/// As cenas com a secção DESLIGADA e as impressões delas, tiradas no `f6d501defb` (antes da Shape
/// Color no Wet Paint). A cena 5 (o modo TOM, secção ligada) saiu: o tom não agia no Wet Paint (a LUT
/// nunca era assada nesta rota) e agora age — [`o_tom_da_shape_age_no_wet_paint`]. Repintadas pela regra do prompt 51
/// (Spacing 0,1 + Continuous no pincel de omissão): a 1 e a 2 (Spacing 1,5 com Continuous) coincidem — com a cápsula o
/// Spacing já não muda o traço, que é a lei (doc 51).
const HOJE: [(usize, u64); 6] = [
    (0, 0x1d86_41c5_28a4_cbcf),
    (1, 0x29b7_ef6b_8902_4e43),
    (2, 0x29b7_ef6b_8902_4e43),
    (3, 0xfa55_efd8_688f_c815),
    (4, 0x29b7_ef6b_8902_4e43),
    (6, 0x1d29_1d25_2021_b078),
];

/// ⭐ **A SECÇÃO DESLIGADA É O TRAÇO DE HOJE AO BYTE** (o portão de identidade do prompt 41, nascido
/// antes do motor mudar): cada rota do depósito — lisa, padrão, cápsula, cerdas, a rampa configurada e
/// desligada e a brocha suja — pinta a mesma tela que pintava.
#[test]
fn a_secao_desligada_e_o_traco_de_hoje_ao_byte() {
    let agora: Vec<(usize, u64)> = HOJE
        .iter()
        .map(|&(n, _)| (n, impressao(&cena(n).canvas_rgba)))
        .collect();
    assert_eq!(agora, HOJE, "as impressões de hoje: {agora:#x?}");
}

/// **O modo TOM (B&W) da Shape age no Wet Paint** — era um controlo mudo: a rota da água nunca assava a
/// LUT do tom, e a cena com o tom pintava ao byte a do Voronoi cru (a impressão `0x0138…` das duas).
///
/// **Mutação que sangra:** tirar o `ensure_shape_ramp_lut` do `rampa_da_aguada` (a rota do dab chama-o).
#[test]
fn o_tom_da_shape_age_no_wet_paint() {
    assert_ne!(
        impressao(&cena(5).canvas_rgba),
        impressao(&cena(1).canvas_rgba),
        "o tom não mudou a tinta"
    );
}

/// `(n, média, desvio)` de G−R nos píxeis pintados (`255 − min(rgb) > 40`).
pub(super) fn g_menos_r(px: &[u8]) -> (usize, f64, f64) {
    let v: Vec<f64> = px
        .chunks(4)
        .filter(|p| 255 - p[0].min(p[1]).min(p[2]) > 40)
        .map(|p| f64::from(p[1]) - f64::from(p[0]))
        .collect();
    let n = v.len().max(1) as f64;
    let m = v.iter().sum::<f64>() / n;
    let s = (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / n).sqrt();
    (v.len(), m, s)
}

/// A correlação de G−R entre duas telas, nos píxeis pintados nas DUAS.
pub(super) fn correlacao(a: &[u8], b: &[u8]) -> f64 {
    let pint = |p: &[u8]| 255 - p[0].min(p[1]).min(p[2]) > 40;
    let pares: Vec<(f64, f64)> = a
        .chunks(4)
        .zip(b.chunks(4))
        .filter(|(p, q)| pint(p) && pint(q))
        .map(|(p, q)| {
            (
                f64::from(p[1]) - f64::from(p[0]),
                f64::from(q[1]) - f64::from(q[0]),
            )
        })
        .collect();
    let n = pares.len().max(1) as f64;
    let (mx, my) = pares
        .iter()
        .fold((0.0, 0.0), |(x, y), (a, b)| (x + a / n, y + b / n));
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (x, y) in &pares {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx) * (x - mx);
        syy += (y - my) * (y - my);
    }
    sxy / (sxx * syy).sqrt().max(1e-9)
}

/// A cena da régua: Voronoi, rampa verde → vermelho em `camadas`, no meio `media`.
pub(super) fn cena_da_regua(
    media: PaintMedia,
    camadas: Option<(bool, bool)>,
    raio: f32,
) -> PainterTool {
    let mut t = tela();
    t.set_paint_media(media);
    t.set_brush_size_px(raio);
    t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
    if camadas.is_some() {
        arma_rampa(&mut t, camadas);
    }
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t
}

/// `Σ(255 − min(rgb))` — a tinta da tela.
pub(super) fn tinta(px: &[u8]) -> u64 {
    px.chunks(4)
        .map(|p| u64::from(255 - p[0].min(p[1]).min(p[2])))
        .sum()
}

/// Uma rampa de UMA cor (`c`, sRGB `0..1` ⇒ a LUT devolve-a ao byte em linear).
pub(super) fn rampa_de(c: [f32; 4]) -> ColorRamp {
    ColorRamp::new(
        vec![RampStop::new(0.0, c), RampStop::new(1.0, c)],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// O pincel Wet Round de fábrica (a brocha apanha a cor da tela).
pub(super) fn wet_round(t: &mut PainterTool) {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let e = bib
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o Wet Round de fábrica");
    assert!(t.wet_preset_aplicar(&e.preset).is_empty());
    t.wet_preset_aplicar_volumes(&e.preset.parametros.volumes);
    t.paint.wetpaint.tilt_on = false;
    t.set_brush_size_px(14.0);
}

/// Um traço CURTO (4 × 2 px) em `(64, 64)` — o carimbo que a prévia mostra (um clique só não pousa: a
/// janela do rasto aterra no 2.º dab e o soltar larga o resto).
pub(super) fn carimbo(t: &mut PainterTool) {
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=4 {
        #[allow(clippy::cast_precision_loss)]
        t.on_canvas_pointer(cp([64.0 + 2.0 * k as f32, 64.0], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([72.0, 64.0], PointerPhase::Up));
}

/// O traço da cápsula: Voronoi, Texture Color (ou sem rampa), `spacing` e o Continuous.
pub(super) fn capsula(spacing: f32, continuo: bool, rampa: bool) -> PainterTool {
    let mut t = cena_da_regua(PaintMedia::WetPaint, rampa.then_some((true, false)), 14.0);
    t.set_brush_spacing(spacing);
    t.set_brush_continuous_spacing(continuo);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    traco(&mut t, 64.0, 16.0, 112.0, 0);
    t
}

/// ⭐ **A TEXTURE COLOR PINTA CADA CÉLULA, COMO NO DIGITAL** — o traço (Voronoi, verde → vermelho)
/// varia de cor ×3,6 o traço sem rampa (desvio de G−R `20,8` contra `5,7`) e segue o Digital (ρ `0,66`);
/// o carimbo que a prévia mostra também (ρ `0,86`). Medido 16/10, a régua em `sonda_shape_color.rs`.
///
/// **Mutação que sangra:** a rota do dab não passar a cor ao motor (`cor_arg = None`).
#[test]
fn a_texture_color_pinta_cada_celula_como_no_digital() {
    let mut sem = cena_da_regua(PaintMedia::WetPaint, None, 14.0);
    traco(&mut sem, 64.0, 16.0, 112.0, 0);
    let mut dig = cena_da_regua(PaintMedia::Digital, Some((true, false)), 14.0);
    traco(&mut dig, 64.0, 16.0, 112.0, 0);
    let mut wet = cena_da_regua(PaintMedia::WetPaint, Some((true, false)), 14.0);
    traco(&mut wet, 64.0, 16.0, 112.0, 0);
    let (s0, s1) = (g_menos_r(&sem.canvas_rgba).2, g_menos_r(&wet.canvas_rgba).2);
    assert!(
        s1 > 2.5 * s0,
        "o desvio de G−R: {s1:.1} com a rampa, {s0:.1} sem"
    );
    let r = correlacao(&dig.canvas_rgba, &wet.canvas_rgba);
    assert!(r > 0.45, "o traço: ρ {r:.2} contra o Digital");
    let mut dig = cena_da_regua(PaintMedia::Digital, Some((true, false)), 24.0);
    carimbo(&mut dig);
    let mut wet = cena_da_regua(PaintMedia::WetPaint, Some((true, false)), 24.0);
    carimbo(&mut wet);
    let r = correlacao(&dig.canvas_rgba, &wet.canvas_rgba);
    assert!(r > 0.6, "o carimbo: ρ {r:.2} contra o Digital");
}

/// **A FALLOFF COLOR LÊ A COBERTURA, COMO NO DIGITAL E NA PRÉVIA** — o índice é a silhueta × o padrão
/// (o Voronoi ainda recorta): G−R `+71,5` contra `+72,0` do Digital, ρ `0,51`. Com a silhueta só (sem o
/// padrão) era `+16,0` e ρ `−0,58` (medido 16/10).
///
/// **Mutação que sangra:** indexar por `fall` em vez de `fall * sv` (`trail/deposit.rs`).
#[test]
fn a_falloff_color_le_a_cobertura_como_no_digital() {
    let mut dig = cena_da_regua(PaintMedia::Digital, Some((false, true)), 14.0);
    traco(&mut dig, 64.0, 16.0, 112.0, 0);
    let mut wet = cena_da_regua(PaintMedia::WetPaint, Some((false, true)), 14.0);
    traco(&mut wet, 64.0, 16.0, 112.0, 0);
    let (d, w) = (g_menos_r(&dig.canvas_rgba).1, g_menos_r(&wet.canvas_rgba).1);
    assert!(
        (d - w).abs() < 8.0,
        "G−R médio: Wet {w:+.1}, Digital {d:+.1}"
    );
    let r = correlacao(&dig.canvas_rgba, &wet.canvas_rgba);
    assert!(r > 0.3, "ρ {r:.2} contra o Digital");
}

/// ⭐ **A COR SOBREVIVE À ÁGUA NOS PINCÉIS DE FÁBRICA** — a recusa do doc 46 §1 («a mistura K–M
/// homogeneíza o carimbo no primeiro passo») refutada: 10 s depois do traço o desvio de G−R da Texture
/// Color ainda é ×3,4 (Express Oil) a ×16,5 (Oil Bristle) o do traço sem rampa (medido 16/10). Só o
/// estado cru de teste, mais molhado que qualquer pincel de fábrica, mistura a rampa toda (×1,0).
#[test]
fn a_cor_sobrevive_a_agua_nos_pinceis_de_fabrica() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    // A contagem é do `the_factory_presets_are_counted` (tests/it); aqui basta que TODOS os ficheiros carreguem.
    assert!(
        bib.erros.is_empty() && !bib.entradas.is_empty(),
        "pincéis de fábrica ilegíveis: {:?}",
        bib.erros
    );
    let mut fracos = Vec::new();
    for e in &bib.entradas {
        let desvio = |c: Option<(bool, bool)>| {
            let mut t = cena_da_regua(PaintMedia::WetPaint, c, 14.0);
            assert!(t.wet_preset_aplicar(&e.preset).is_empty());
            t.wet_preset_aplicar_volumes(&e.preset.parametros.volumes);
            // A cor de cada FIO (W2) é outra fonte de G−R: o controlo sem rampa deixava de ser liso.
            super::verde_no_soltar_tests::sem_os_fios(&mut t);
            t.paint.wetpaint.tilt_on = false;
            t.set_brush_size_px(14.0);
            t.set_brush_color_srgb8([30, 60, 220]);
            t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
            if c.is_some() {
                arma_rampa(&mut t, c);
            }
            let seed = t.paint.brush;
            t.paint.brush_by_mode.fill(seed);
            traco(&mut t, 64.0, 16.0, 112.0, 600);
            g_menos_r(&t.canvas_rgba).2
        };
        let r = desvio(Some((true, false))) / desvio(None).max(0.1);
        if r < 2.5 {
            fracos.push(format!("{}: ×{r:.1}", e.preset.parametros.name));
        }
    }
    assert!(fracos.is_empty(), "a água apagou a rampa: {fracos:?}");
}

/// **O PADRÃO QUE VIRA COR DEIXA DE RECORTAR A ÁGUA** (a lei do digital, P38; a da aguada, P39): a mesma
/// cor única pela Texture Color no Voronoi pousa a tinta da ponta lisa (`1,000`); pela Falloff Color o
/// Voronoi ainda recorta (`0,821`, o controlo). Medido 16/10.
///
/// **Mutação que sangra:** `padrao_vira_cor = false` na rota do dab.
#[test]
fn o_padrao_que_vira_cor_nao_recorta_a_agua() {
    let ink = |kind: TextureKind, textura: bool| {
        let mut t = tela();
        t.set_brush_shape_kind(kind.to_u8());
        let r = rampa_de([0.05, 0.1, 0.7, 1.0]);
        t.set_shape_color_ramp(r.clone());
        t.set_shape_falloff_ramp(r);
        t.set_shape_ramp_enabled(true);
        t.paint.shape_camadas.textura = textura;
        t.paint.shape_camadas.queda = !textura;
        t.set_shape_falloff_ramp(t.paint.shape_camadas.queda_rampa.clone());
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        traco(&mut t, 64.0, 16.0, 112.0, 60);
        tinta(&t.canvas_rgba) as f64
    };
    let lisa = ink(TextureKind::None, false);
    let (textura, queda) = (
        ink(TextureKind::Voronoi, true) / lisa,
        ink(TextureKind::Voronoi, false) / lisa,
    );
    assert!(
        queda < 0.9,
        "controlo: o Voronoi pela Falloff recorta ({queda:.3})"
    );
    assert!(
        textura > 0.98,
        "a Texture Color: {textura:.3} da ponta lisa"
    );
}

/// **A CÁPSULA (Continuous) LEVA A COR DE CADA SUB-DAB** — o traço a Spacing 1,5 com o Continuous casa
/// com o traço denso (0,05): ρ de G−R `0,97` (medido 16/10).
#[test]
fn a_capsula_leva_a_cor_de_cada_sub_dab() {
    let denso = capsula(0.05, false, true);
    let cont = capsula(1.5, true, true);
    let r = correlacao(&denso.canvas_rgba, &cont.canvas_rgba);
    assert!(r > 0.85, "ρ {r:.2} contra o traço denso");
}

/// **A BROCHA LEVA A COR QUE APANHOU, COM A RAMPA** (a lei do A8 da aguada, na água): o Wet Round
/// atravessa uma faixa azul MOLHADA; depois do cruzamento o azul (B − max(R, G)) sobe `+1,7` com a rampa
/// (`+2,5` com a cor única; `0,0` quando a rampa recarregava a ponta a cada janela). Medido 16/10.
///
/// **Mutação que sangra:** o transfer pousar a tinta limpa sem a sujidade da ponta.
#[test]
fn a_brocha_leva_a_cor_que_apanhou_com_a_rampa() {
    let azul_depois = |sobre_azul: bool| {
        let mut t = tela();
        wet_round(&mut t);
        if sobre_azul {
            t.set_brush_color_srgb8([20, 30, 230]);
            let seed = t.paint.brush;
            t.paint.brush_by_mode.fill(seed);
            t.on_canvas_pointer(cp([40.0, 30.0], PointerPhase::Down));
            for k in 1..=17 {
                #[allow(clippy::cast_precision_loss)]
                t.on_canvas_pointer(cp([40.0, 30.0 + 4.0 * k as f32], PointerPhase::Move));
                t.on_tick(1000.0 / 60.0);
            }
            t.on_canvas_pointer(cp([40.0, 98.0], PointerPhase::Up));
        }
        t.set_brush_color_srgb8([230, 200, 20]);
        t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
        arma_rampa(&mut t, Some((true, false)));
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        traco(&mut t, 64.0, 16.0, 112.0, 0);
        let mut s = 0.0f64;
        for y in 56..72usize {
            for x in 60..80usize {
                let p = &t.canvas_rgba[(y * LADO as usize + x) * 4..][..4];
                s += f64::from(p[2]) - f64::from(p[0]).max(f64::from(p[1]));
            }
        }
        s / 320.0
    };
    let d = azul_depois(true) - azul_depois(false);
    assert!(d > 0.85, "o azul apanhado depois do cruzamento: {d:+.2}");
}

/// **Escolher um Grain no Wet Paint não apaga a Shape Color** — o B&W automático (a rampa do Grain fica dona
/// da cor) só existe onde essa rampa é oferecida (Digital, Impasto); na água ele desligava a Shape Color sem
/// dar cor nenhuma em troca. No Digital continua.
///
/// **Mutação que sangra:** o `on_grain_assigned` ligar o B&W em todo meio.
#[test]
fn escolher_um_grain_no_wet_paint_nao_apaga_a_shape_color() {
    for (media, bw) in [(PaintMedia::WetPaint, false), (PaintMedia::Digital, true)] {
        let mut t = cena_da_regua(media, Some((true, false)), 14.0);
        t.set_brush_texture_kind(1);
        assert_eq!(
            t.paint.shape_color_ramp_bw, bw,
            "{media:?}: o B&W depois do Grain"
        );
    }
}
