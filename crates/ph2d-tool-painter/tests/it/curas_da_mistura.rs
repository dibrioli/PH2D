//! As curas da mistura Wet blend de 08/11 (`docs/Painter/AUDITORIA_MISTURA_2026-11-08.md`, Bugs #73–#79): cada teste
//! reprova sem a sua cura (provado por mutação) e é a régua da sonda da auditoria reduzida ao veredito.

use crate::familias::{csp_secos, csp_tinta_grossa};
use crate::presets_de_fabrica::preset_do_meio;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{LinhaDaPonta, ModoMisturaOpaca, PainterTool};

const W: usize = 400;
const H: usize = 240;
const Y: f32 = 120.0;
const BRUSH: u8 = 0;
const SMEAR: u8 = 1;
const BLUR: u8 = 2;
const ERASE: u8 = 3;

fn receita(nome: &str) -> ph2d_brush_presets::Preset {
    let r = csp_tinta_grossa::MEIOS
        .iter()
        .chain(csp_secos::MEIOS.iter())
        .find(|r| r.nome == nome)
        .unwrap_or_else(|| panic!("receita {nome}"));
    preset_do_meio(r)
}

fn aplica(t: &mut PainterTool, p: &ph2d_brush_presets::Preset, size: f32) {
    assert_eq!(t.preset_aplicar(p), Ok(Vec::new()));
    t.set_brush_size_px(size);
}

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], n: usize) {
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    for k in 1..=n {
        let u = k as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u],
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
    t.on_tick(8.0);
}

/// Branco opaco com um bloco vermelho no meio (`x 160..260`), pintado sem a mistura.
fn tela_com_bloco() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_brush_size_px(20.0);
    t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
    t.set_brush_color_srgb8([200, 0, 0]);
    for k in 0..9 {
        let y = Y - 40.0 + 10.0 * k as f32;
        traco(&mut t, [160.0, y], [260.0, y], 60);
    }
    t
}

fn com_pincel(nome: &str, mut t: PainterTool) -> PainterTool {
    aplica(&mut t, &receita(nome), 15.0);
    t.set_brush_color_srgb8([0, 40, 220]);
    t
}

fn monta_composite(t: &mut PainterTool, pilha: &[(u8, f32)]) {
    if !t.composite_enabled() {
        t.toggle_composite();
    }
    for (i, &(op, s)) in pilha.iter().enumerate() {
        t.acrescenta_camada(op);
        t.set_composite_layer_strength(i, s);
    }
    assert_eq!(t.composite_len(), pilha.len());
}

/// O traço horizontal de `x 40` a `340` com `n` eventos, sobre o bloco, com a pilha `pilha` (vazia = sem Composite).
fn pinta_bloco(nome: &str, off: bool, pilha: &[(u8, f32)], n: usize) -> Vec<u8> {
    let mut t = com_pincel(nome, tela_com_bloco());
    if off {
        t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
    }
    if !pilha.is_empty() {
        monta_composite(&mut t, pilha);
    }
    traco(&mut t, [40.0, Y], [340.0, Y], n);
    t.run_full().0
}

fn media(p: &[u8], x: usize, y: usize) -> [f32; 4] {
    let mut s = [0.0f32; 4];
    let mut n = 0.0;
    for yy in y - 2..=y + 2 {
        for xx in x - 4..=x + 4 {
            let i = (yy * W + xx) * 4;
            for c in 0..4 {
                s[c] += f32::from(p[i + c]);
            }
            n += 1.0;
        }
    }
    s.map(|v| v / n)
}

fn diff_medio(a: &[u8], b: &[u8]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| f64::from(x.abs_diff(*y)))
        .sum::<f64>()
        / a.len() as f64
}

const MISTURADORES: [&str; 4] = ["Round Mixing", "Gouache", "Oil Paint", "Finger Tip"];

/// ⭐ #73: a camada **Erase** do Composite apaga com o Wet blend ligado (pintava `[95,120,233,255]`).
#[test]
fn a_camada_erase_do_composite_apaga_com_a_mistura() {
    for nome in MISTURADORES {
        let p = pinta_bloco(nome, false, &[(ERASE, 1.0), (BRUSH, 1.0)], 60);
        let fora = media(&p, 100, Y as usize);
        let controlo = media(&pinta_bloco(nome, false, &[], 60), 100, Y as usize);
        assert!(
            controlo[3] > 250.0,
            "{nome}: CONTROLO — o traço pinta {controlo:?}"
        );
        assert!(fora[3] < 1.0, "{nome}: a camada Erase pintou {fora:?}");
    }
}

/// ⭐ #75: na receita do dono (Blur 1 · Smear 0,25 · Brush 0,10) a Strength da camada Brush age com a mistura (Δ 0).
#[test]
fn a_strength_da_brush_na_receita_do_dono_age_com_a_mistura() {
    for nome in ["Round Mixing", "Gouache", "Oil Paint"] {
        let a = pinta_bloco(
            nome,
            false,
            &[(BLUR, 1.0), (SMEAR, 0.25), (BRUSH, 0.10)],
            60,
        );
        let b = pinta_bloco(nome, false, &[(BLUR, 1.0), (SMEAR, 0.25), (BRUSH, 1.0)], 60);
        let d = diff_medio(&a, &b);
        assert!(d > 1.0, "{nome}: Strength 0,1 × 1 da camada Brush Δ {d:.3}");
    }
}

/// ⭐ #74: no Composite cada camada mistura só a tela de antes do traço — o mesmo traço dá o mesmo byte com 60 ou
/// 120 eventos (o reservatório partilhado saltava entre as camadas por lote: Δ máx 20–34).
#[test]
fn o_composite_com_mistura_da_o_mesmo_byte_em_qualquer_cadencia() {
    for nome in MISTURADORES {
        let pilha = [(BRUSH, 1.0), (BRUSH, 1.0)];
        let a = pinta_bloco(nome, false, &pilha, 60);
        let b = pinta_bloco(nome, false, &pilha, 120);
        assert!(
            a == b,
            "{nome}: 60 × 120 eventos Δ {:.3}",
            diff_medio(&a, &b)
        );
    }
}

/// ⭐ #74: a camada de cima não apanha a tinta que a de baixo pôs neste traço (amarela por cima de azul: saía
/// `[163,169,183]`). A de cima só vê a tela de antes — o branco —, logo fica amarela.
#[test]
fn a_camada_de_cima_do_composite_nao_apanha_a_tinta_da_de_baixo() {
    let mut t = com_pincel("Round Mixing", tela_com_bloco());
    monta_composite(&mut t, &[(BRUSH, 1.0), (BRUSH, 1.0)]);
    t.set_composite_layer_color(0, [1.0, 0.85, 0.0]);
    traco(&mut t, [40.0, Y], [340.0, Y], 60);
    let p = t.run_full().0;
    let c = media(&p, 100, Y as usize);
    // Amarelo clareado pelo branco que apanha (`[255,243,176]`), nunca o cinzento-azul de antes.
    assert!(
        c[0] > 230.0 && c[2] < c[0] - 50.0,
        "a camada amarela de cima apanhou o azul de baixo: {c:?}"
    );
}

/// Transparente à esquerda de `x 200`, branco no meio, listras vermelho/azul/amarelo à direita de `x 400` (a tela da
/// sonda `sonda_auditoria_papel_grao`, mais larga).
const WL: usize = 600;
const HL: usize = 300;

fn listras() -> Vec<u8> {
    let mut b = vec![0u8; WL * HL * 4];
    for y in 0..HL {
        for x in 200..WL {
            let c: [u8; 4] = if x < 400 {
                [255, 255, 255, 255]
            } else {
                match ((x - 400) / 20) % 3 {
                    0 => [220, 30, 30, 255],
                    1 => [30, 60, 220, 255],
                    _ => [240, 220, 30, 255],
                }
            };
            b[(y * WL + x) * 4..][..4].copy_from_slice(&c);
        }
    }
    b
}

/// O pincel `nome` em verde, das listras ao vazio (`x 580 → 20`, `y 80`), e um 2.º traço só no vazio (`y 220`).
fn pinta_listras(nome: &str, prepara: impl FnOnce(&mut PainterTool)) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(listras(), WL as u32, HL as u32);
    aplica(&mut t, &receita(nome), 15.0);
    t.set_brush_color_srgb8([0, 160, 60]);
    prepara(&mut t);
    let passos = |a: f32, b: f32| ((a - b).abs() / 4.0) as usize;
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    for (x0, x1, y) in [(580.0f32, 20.0f32, 80.0f32), (20.0, 180.0, 220.0)] {
        let n = passos(x0, x1);
        t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
        for k in 1..=n {
            let x = x0 + (x1 - x0) * k as f32 / n as f32;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
        t.on_tick(8.0);
    }
    t.run_full().0
}

/// A média (RGB pesado pelo alfa, alfa) e quantos texels de alfa `> 40` saem do verde por 40, na faixa
/// `x0..x1 × yc ± 7`.
fn regiao(px: &[u8], x0: usize, x1: usize, yc: usize) -> ([f32; 3], f32, usize) {
    let (mut rgb, mut wa, mut a, mut n, mut fora) = ([0.0f32; 3], 0.0f32, 0.0f32, 0.0f32, 0);
    for y in yc - 7..yc + 7 {
        for x in x0..x1 {
            let p = &px[(y * WL + x) * 4..][..4];
            let al = f32::from(p[3]);
            for k in 0..3 {
                rgb[k] += f32::from(p[k]) * al;
            }
            wa += al;
            a += al / 255.0;
            n += 1.0;
            let g = i32::from(p[1]);
            if p[3] > 40 && (i32::from(p[0]) > g + 40 || i32::from(p[2]) > g + 40) {
                fora += 1;
            }
        }
    }
    if wa > 0.0 {
        rgb = rgb.map(|c| c / wa);
    }
    (rgb, a / n, fora)
}

/// ⭐ #76: o esbatedor (Amount 0) leva para o vazio a cor que apanhou, não a do pincel (pintava `(75,185,115)`).
#[test]
fn o_esbatedor_nao_solta_a_cor_do_pincel_no_rasto_que_vai_ao_vazio() {
    for nome in ["Finger Tip", "Gouache Blender"] {
        let p = pinta_listras(nome, |_| {});
        let (rgb, a, _) = regiao(&p, 150, 190, 80);
        assert!(
            a > 0.3,
            "{nome}: CONTROLO — o rasto chega ao vazio (α {a:.2})"
        );
        assert!(
            rgb[0] > 150.0,
            "{nome}: o rasto no vazio é o verde do pincel {rgb:?}"
        );
    }
}

fn rampa_vermelho_azul() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// ⭐ #77: com a mistura ligada as rampas de cor (Grain Colors, Shape Color) pintam a cor delas (contagem 0).
#[test]
fn as_rampas_de_cor_pintam_com_a_mistura() {
    let papel = r#"{"angle_deg": 0, "flow": false, "kind": "PaperCold", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [128.0, 128.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
    for nome in ["Round Mixing", "Oil Paint", "Crayon"] {
        let grao = pinta_listras(nome, |t| {
            let mut p = receita(nome);
            p.parametros.brush.insert(
                "texture".into(),
                serde_json::from_str(papel).expect("papel"),
            );
            aplica(t, &p, 15.0);
            t.set_brush_color_srgb8([0, 160, 60]);
            t.set_texture_ramp(rampa_vermelho_azul());
            t.set_texture_ramp_enabled(true);
        });
        let (_, _, fora) = regiao(&grao, 20, 180, 220);
        assert!(
            fora > 500,
            "{nome}: Grain Colors não pintou a rampa ({fora})"
        );
    }
    let shape = pinta_listras("Gouache", |t| {
        t.set_shape_color_ramp(rampa_vermelho_azul());
        t.set_shape_ramp_enabled(true);
    });
    let (_, _, fora) = regiao(&shape, 20, 180, 220);
    assert!(
        fora > 500,
        "Gouache: Shape Color não pintou a rampa ({fora})"
    );
}

/// ⭐ #77: o Per-Layer Color (Shape de duas camadas, verde e amarela) pinta as cores das camadas com a mistura (Δ 0).
#[test]
fn o_per_layer_color_pinta_com_a_mistura() {
    const S: u32 = 64;
    let metade = |cima: bool| -> (Vec<u8>, u32, u32) {
        let v = (0..S * S)
            .map(|i| if ((i / S) < S / 2) == cima { 255 } else { 0 })
            .collect();
        (v, S, S)
    };
    for nome in ["Round Mixing", "Oil Paint"] {
        let corre = |por_camada: bool| {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
            aplica(&mut t, &receita(nome), 30.0);
            t.set_brush_color_srgb8([0, 40, 220]);
            t.set_brush_shape_layers(vec![metade(false), metade(true)]);
            if por_camada {
                t.toggle_brush_shape_per_layer_color();
                t.set_brush_shape_layer_color(0, [0.1, 0.8, 0.1]);
                t.set_brush_shape_layer_color(1, [1.0, 0.85, 0.0]);
            }
            traco(&mut t, [40.0, Y], [340.0, Y], 60);
            t.run_full().0
        };
        let d = diff_medio(&corre(true), &corre(false));
        assert!(
            d > 3.0,
            "{nome}: o Per-Layer Color não mudou o traço (Δ {d:.3})"
        );
    }
}

/// ⭐ #77: a ponta de PÍXEL carimba binária com a mistura (a rota saía antes dela: a orla anti-aliased do pincel).
#[test]
fn a_ponta_de_pixel_carimba_binaria_com_a_mistura() {
    let orla = |pixel: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
        aplica(&mut t, &receita("Round Mixing"), 8.0);
        t.set_brush_color_srgb8([0, 40, 220]);
        if pixel {
            t.set_ponta(LinhaDaPonta::Pixel, 1.0);
        }
        traco(&mut t, [40.0, Y], [340.0, Y], 60);
        let p = t.run_full().0;
        // Texels da faixa com alfa a meio (`16..240`) contra os cheios.
        let (mut meio, mut cheios) = (0, 0);
        for y in Y as usize - 12..Y as usize + 12 {
            for x in 60..320 {
                match p[(y * W + x) * 4 + 3] {
                    16..=239 => meio += 1,
                    240..=255 => cheios += 1,
                    _ => {}
                }
            }
        }
        (meio, cheios)
    };
    let (meio_aa, _) = orla(false);
    let (meio, cheios) = orla(true);
    assert!(
        meio_aa > 200,
        "CONTROLO — o pincel liso tem orla ({meio_aa})"
    );
    assert!(cheios > 1000, "a ponta de píxel pintou ({cheios})");
    assert!(
        meio * 4 < meio_aa,
        "a ponta de píxel tem orla: {meio} contra {meio_aa}"
    );
}

/// ⭐ Dono 08/11 (revoga o #78): os pincéis que SORTEIAM a pegada (Pointillism, Pastel) carimbam a 3 % com a mistura
/// e ENCHEM — o look cheio que ele aprovou (Pointillism 0,71 contra 0,38 Off; Pastel 0,36 contra 0,16).
#[test]
fn os_pinceis_que_espalham_enchem_com_a_mistura() {
    for nome in ["Pointillism", "Pastel"] {
        let a = |off: bool| {
            let p = pinta_listras(nome, |t| {
                if off {
                    t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
                }
            });
            regiao(&p, 20, 180, 220).1
        };
        let (wet, off) = (a(false), a(true));
        assert!(off > 0.05, "{nome}: CONTROLO — o traço pinta ({off:.3})");
        assert!(
            wet > off * 1.5,
            "{nome}: com a mistura a cobertura voltou a ser a rala do Off — {wet:.3} contra {off:.3}"
        );
    }
}

/// ⭐ #73: a camada **Blur** do Composite borra com a mistura — ela mede a cobertura apagando, e a mistura pintava o
/// plano dela (cobertura 0: o borrão não corria). Medido contra o mesmo par sem a mistura.
#[test]
fn a_camada_blur_do_composite_borra_com_a_mistura() {
    for nome in ["Round Mixing", "Oil Paint"] {
        let borra = |off: bool| {
            let com = pinta_bloco(nome, off, &[(BLUR, 1.0), (BRUSH, 1.0)], 60);
            let sem = pinta_bloco(nome, off, &[(BRUSH, 1.0), (BRUSH, 0.0)], 60);
            diff_medio(&com, &sem)
        };
        let (wet, off) = (borra(false), borra(true));
        assert!(
            off > 0.2,
            "{nome}: CONTROLO — o Blur borra sem a mistura (Δ {off:.3})"
        );
        assert!(
            wet > off * 0.3,
            "{nome}: o Blur não borra com a mistura — Δ {wet:.3} contra {off:.3}"
        );
    }
}

/// ⭐ #79: «Sample: All visible layers» (de fábrica no Gouache Blender e no Finger Tip) — o esbatedor numa camada
/// NOVA vazia sobre o bloco vermelho arrasta o vermelho; com «This layer» não faz nada (a camada dele está vazia).
#[test]
fn o_esbatedor_numa_camada_nova_arrasta_a_cor_das_de_baixo() {
    for nome in ["Gouache Blender", "Finger Tip"] {
        let corre = |so_esta: bool| {
            let mut t = tela_com_bloco();
            assert!(t.add_raster_layer("traço").is_some(), "camada nova");
            let mut p = receita(nome);
            if so_esta {
                let m = p
                    .parametros
                    .brush
                    .get_mut("mistura_opaca")
                    .expect("mistura");
                let mut o: serde_json::Value = match &*m {
                    serde_json::Value::String(s) => serde_json::from_str(s).expect("json"),
                    x => x.clone(),
                };
                o.as_object_mut().expect("obj").remove("sample");
                *m = o;
            }
            aplica(&mut t, &p, 15.0);
            traco(&mut t, [200.0, Y], [340.0, Y], 60);
            let px = t.run_full().0;
            // Depois do bloco (x 300): o vermelho levado para o branco.
            media(&px, 300, Y as usize)
        };
        let (todas, esta) = (corre(false), corre(true));
        assert!(
            esta[1] > 240.0,
            "{nome}: CONTROLO — com This layer nada se arrasta {esta:?}"
        );
        assert!(
            todas[1] < esta[1] - 20.0,
            "{nome}: All visible layers não arrastou o vermelho de baixo — {todas:?} contra {esta:?}"
        );
    }
}
