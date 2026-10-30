//! **O CENSO do papel de COR** (dono 30/10: «procure defeitos similares e cure todos», depois do BUGS #53): cada preset
//! de fábrica pinta (ou move) um 2.º traço laranja que cruza um Wet Flat azul sobre um papel bege, a imagem MOSTRADA a meio do 2.º
//! traço e assentada. A família: uma cor composta que leva BRANCO embutido — invisível no papel branco, onde todos os
//! outros portões pintam.

use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};

const W: usize = 160;
const H: usize = 120;
const PAPEL: [u8; 3] = [150, 130, 100];
const TINTAS: [[u8; 3]; 2] = [[40, 90, 220], [240, 130, 60]];

fn presets() -> Vec<(String, std::path::PathBuf)> {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let mut v = Vec::new();
    let mut fila = vec![raiz.clone()];
    while let Some(d) = fila.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                fila.push(p);
            } else if p.to_string_lossy().ends_with(".ph2d-brush.png") {
                let nome = p
                    .strip_prefix(&raiz)
                    .unwrap()
                    .to_string_lossy()
                    .replace(".ph2d-brush.png", "");
                v.push((nome, p));
            }
        }
    }
    v.sort();
    v
}

fn ponteiro(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn mostrada(t: &mut PainterTool) -> Vec<u8> {
    t.invalidate_composite();
    let (px, _, _) = t.take_preview_arc().expect("a imagem mostrada");
    px.to_vec()
}

/// `(a meio, assentada, pintados)`: os píxeis com luminosidade acima da do papel e das duas tintas (+30).
pub(super) fn censo_de(
    caminho: &std::path::Path,
    oleo: bool,
    relevo: bool,
) -> (usize, usize, usize) {
    cena(caminho, oleo, relevo).1
}

/// `(a meio, assentada, pintados)` de [`censo_de`].
type Censo = (usize, usize, usize);

fn cena(
    caminho: &std::path::Path,
    oleo: bool,
    relevo: bool,
) -> (Option<(PainterTool, Vec<u8>)>, Censo) {
    let p = ph2d_brush_presets::ler(&std::fs::read(caminho).unwrap()).unwrap();
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.set_paint_tool_mode("wetpaint");
    // O pincel na mão, com as escolhas da sonda (sem o Real Oil, sem o Emboss).
    let arma = |t: &mut PainterTool, p: &ph2d_brush_presets::Preset| {
        if t.preset_aplicar(p).is_err() {
            return false;
        }
        if !oleo {
            t.set_wet_oleo_real_ligado(false);
        }
        t.set_brush_size_px(24.0);
        if !relevo {
            t.paint
                .wetpaint
                .knobs
                .set(ph2d_wet_paint::tuning::Knob::Emboss, 0.0);
        }
        true
    };
    // O 1.º traço é sempre o Wet Flat azul: os pincéis que MOVEM tinta (Blur, Smear, Blend, Clone) têm onde agir.
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/brushes/WetPainter/Watercolor/Fluid/Wet Flat.ph2d-brush.png");
    let base = ph2d_brush_presets::ler(&std::fs::read(base).unwrap()).unwrap();
    assert!(arma(&mut t, &base), "o Wet Flat");
    let tick = |t: &mut PainterTool| t.on_tick(1000.0 / 60.0);
    t.set_brush_color_srgb8(TINTAS[0]);
    t.on_canvas_pointer(ponteiro(20.0, 50.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponteiro(20.0 + 4.0 * k as f32, 50.0, PointerPhase::Move));
        tick(&mut t);
    }
    t.on_canvas_pointer(ponteiro(140.0, 50.0, PointerPhase::Up));
    for _ in 0..40 {
        tick(&mut t);
    }
    if !arma(&mut t, &p) {
        return (None, (usize::MAX, 0, 0));
    }
    t.set_brush_color_srgb8(TINTAS[1]);
    t.on_canvas_pointer(ponteiro(30.0, 85.0, PointerPhase::Down));
    for k in 1..=24 {
        t.on_canvas_pointer(ponteiro(
            30.0 + 4.0 * k as f32,
            85.0 - 1.5 * k as f32,
            PointerPhase::Move,
        ));
        tick(&mut t);
    }
    let meio = mostrada(&mut t);
    t.on_canvas_pointer(ponteiro(126.0, 49.0, PointerPhase::Up));
    for _ in 0..300 {
        tick(&mut t);
    }
    let fim = mostrada(&mut t);
    let lum =
        |p: &[u8]| 0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2]);
    let teto = lum(&PAPEL).max(lum(&TINTAS[0])).max(lum(&TINTAS[1]));
    let claros = |img: &[u8]| img.chunks(4).filter(|px| lum(px) > teto + 30.0).count();
    let pintados = fim
        .chunks(4)
        .filter(|px| (0..3).any(|k| px[k].abs_diff(PAPEL[k]) > 8))
        .count();
    let n = (claros(&meio), claros(&fim), pintados);
    (Some((t, fim)), n)
}

/// SONDA — a célula mais clara de um preset (`PH2D_SONDA_PRESET`, o caminho relativo a `assets/brushes`) assentado.
#[test]
#[ignore = "diagnóstico"]
fn diag_a_celula_mais_clara_do_censo() {
    let rel = std::env::var("PH2D_SONDA_PRESET").expect("PH2D_SONDA_PRESET");
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let (Some((mut t, fim)), n) = cena(&raiz.join(format!("{rel}.ph2d-brush.png")), false, false)
    else {
        panic!("o preset");
    };
    let lum =
        |p: &[u8]| 0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2]);
    let i = (0..W * H)
        .max_by(|&a, &b| lum(&fim[a * 4..a * 4 + 4]).total_cmp(&lum(&fim[b * 4..b * 4 + 4])))
        .unwrap();
    let (x, y) = (i % W, i / W);
    let mut l = format!(
        "{rel} {n:?}: ({x},{y}) mostrada {:?} camada {:?}\n",
        &fim[i * 4..i * 4 + 4],
        &t.canvas_rgba[i * 4..i * 4 + 4]
    );
    t.wet_bring_home();
    if let Some(s) = t.paint.wetpaint.session.as_ref() {
        let r = usize::from(s.ratio);
        for (k, ly) in s.engine.layers.iter().enumerate() {
            let g = &ly.grid;
            let c = (x / r + 1) + (y / r + 1) * g.s;
            l += &format!(
                "  camada {k}: sett {:.0} {:?} susp {:.0} {:?} vela {:?}\n",
                g.sett[c],
                g.sett_rgb[c],
                g.susp[c],
                g.susp_rgb[c],
                g.vela.get(c)
            );
        }
    }
    eprintln!("{l}");
    if let Ok(d) = std::env::var("PH2D_SONDA_DIR") {
        std::fs::write(format!("{d}/celula.txt"), l).unwrap();
    }
}

/// SONDA — o censo de toda a fábrica (`PH2D_CENSO_OLEO=1` com o Real Oil de cada preset, `PH2D_CENSO_SEM_RELEVO=1`
/// sem o Emboss).
#[test]
#[ignore = "censo: corre-se à mão"]
fn censo_do_papel_de_cor_na_fabrica() {
    let oleo = std::env::var("PH2D_CENSO_OLEO").is_ok();
    let relevo = std::env::var("PH2D_CENSO_SEM_RELEVO").is_err();
    let lista = presets();
    let fatias: Vec<_> = lista.chunks(lista.len().div_ceil(8)).collect();
    let mut linhas: Vec<String> = std::thread::scope(|s| {
        let h: Vec<_> = fatias
            .iter()
            .map(|f| {
                s.spawn(move || {
                    f.iter()
                        .map(|(n, p)| {
                            let (m, a, q) = censo_de(p, oleo, relevo);
                            format!("{m:>6} {a:>6} {q:>6}  {n}")
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        h.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    linhas.sort();
    linhas.reverse();
    let txt = format!(
        "  meio assent pintados  preset (píxeis com luminosidade > a do papel e das tintas + 30)\n{}\n",
        linhas.join("\n")
    );
    eprintln!("{txt}");
    if let Ok(d) = std::env::var("PH2D_SONDA_DIR") {
        std::fs::write(format!("{d}/censo.txt"), txt).unwrap();
    }
}

/// ⭐ **Nenhum pincel de fábrica clareia o papel de COR** (dono 30/10, a família do BUGS #53): dois traços de tintas
/// diferentes que se cruzam sobre o papel bege, sem o Real Oil e sem o Emboss (a luz deles clareia por lei), e nenhum
/// píxel da imagem mostrada a meio do 2.º traço ou assentada passa a luminosidade do papel e das duas tintas (+30).
/// Medido sem a cura (`na_caixa_das_tintas`): 11 presets do Wet acusavam, a meio (Wet Dry Brush 1 891, Wet Round 798,
/// Wet Drip 554, Dabber 4 185, Wet Flat 162, Wet Wash 115, Gouache Flat 114, Knife Oily 95, Knife Stain 87, Knife
/// Grainy 81, Dabber 76); com ela, 0 nos 66.
/// Os ABERTOS (BUGS #54, decisão do dono): o ASSENTAR guarda o ponto sobre o branco e, quando a cobertura sobe a ~1, o
/// branco que se via vira tinta opaca (Marker Bullet sobre o Wet Flat molhado: sett `1703` `[239,236,201]`, assentado
/// 593 px). A cura simulada (a caixa das tintas no `assenta`) zera o censo mas muda 25 miniaturas até 10 níveis sobre o
/// branco (6 óleos aprovados 5–10) e duas leis do Glaze — fica para o dono decidir pela foto.
const ABERTOS: [&str; 1] = ["WetPainter/Marker/Marker/Marker Bullet"];

#[test]
fn nenhum_pincel_de_fabrica_clareia_o_papel_de_cor() {
    let lista = presets();
    assert!(lista.len() >= 60, "a fábrica tem {} presets", lista.len());
    let fatias: Vec<_> = lista.chunks(lista.len().div_ceil(8)).collect();
    let culpados: Vec<String> = std::thread::scope(|s| {
        let h: Vec<_> = fatias
            .iter()
            .map(|f| {
                s.spawn(move || {
                    f.iter()
                        .filter_map(|(n, p)| {
                            let (m, a, q) = censo_de(p, false, false);
                            assert!(
                                m == usize::MAX || q > 1000,
                                "{n}: o traço quase não pintou ({q} px)"
                            );
                            (m != usize::MAX && (m > 0 || a > 0))
                                .then(|| format!("{n}: a meio {m}, assentado {a}"))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        h.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    // A catraca: só encolhe. Curar um ABERTO reprova até ele sair da lista.
    let nomes: Vec<&str> = culpados
        .iter()
        .map(|l| l.split(':').next().unwrap())
        .collect();
    assert_eq!(
        nomes,
        ABERTOS,
        "clareiam o papel de cor (fora da lista dos abertos, ou um aberto curado):\n{}",
        culpados.join("\n")
    );
}
