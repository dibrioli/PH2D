//! **As orlas do traço sem rectângulos e sem serrilha** (smoke do dono 24/10, Painter Digital, vermelho translúcido
//! sobre um fundo azul-esverdeado com grelha): *«defeitos retangulares malditos com Burn edges ou Wet edges > 0»* e
//! *«bordas mais pixeladas com flow alto ou wet edge alto»*. Pela porta do ponteiro (a mesma que a app chama), o
//! Glaze Round de fábrica, um Z que se cruza, ao passo do rato.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: usize = 320;
const H: usize = 240;

/// O fundo do smoke: azul-esverdeado opaco com uma grelha escura de 2 px a cada 40.
fn fundo() -> Vec<u8> {
    let mut px = Vec::with_capacity(W * H * 4);
    for y in 0..H {
        for x in 0..W {
            let grelha = x % 40 < 2 || y % 40 < 2;
            px.extend_from_slice(&if grelha {
                [20, 40, 45, 255]
            } else {
                [40, 150, 140, 255]
            });
        }
    }
    px
}

/// O pincel Digital por omissão, 40 px, vermelho, Strength 0,6 (Accumulate off), com `f` por cima.
fn padrao(f: impl FnOnce(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(fundo(), W as u32, H as u32);
    t.set_paint_media(crate::PaintMedia::Digital);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([220, 30, 30]);
    t.set_brush_strength(0.6);
    f(&mut t);
    t
}

/// O Glaze Round de fábrica, 40 px, vermelho, com `f` por cima.
fn glaze_round(f: impl FnOnce(&mut PainterTool)) -> PainterTool {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let p = &bib
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Glaze Round")
        .expect("Glaze Round de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(fundo(), W as u32, H as u32);
    t.preset_aplicar(p).expect("meio");
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([220, 30, 30]);
    f(&mut t);
    t
}

/// O Z que se cruza (volta em diagonal por cima de si), ao passo de 3 px do rato.
fn caminho_z() -> Vec<[f32; 2]> {
    let nos = [
        [40.0, 40.0],
        [280.0, 40.0],
        [50.0, 200.0],
        [280.0, 200.0],
        [150.0, 20.0],
    ];
    let mut out = vec![nos[0]];
    for par in nos.windows(2) {
        let (a, b): ([f32; 2], [f32; 2]) = (par[0], par[1]);
        let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil() as usize;
        for k in 1..=n {
            let s = k as f32 / n as f32;
            out.push([a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s]);
        }
    }
    out
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O traço como a app o corre: os eventos do rato e um quadro (`paint_tick`) a cada 4 eventos; o traço fica
/// ABERTO (o chamador levanta-o).
fn pinta_aberto(t: &mut PainterTool) {
    let pts = caminho_z();
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for (k, p) in pts[1..].iter().enumerate() {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        if k % 4 == 3 {
            t.paint_tick(1.0 / 60.0);
        }
    }
    t.paint_tick(1.0 / 60.0);
}

/// O resultado VIVO (lote a lote) e a REFERÊNCIA: o mesmo plano final composto numa passagem sobre a tela inteira.
fn vivo_e_referencia(t: &mut PainterTool) -> (Vec<u8>, Vec<u8>) {
    pinta_aberto(t);
    let vivo = t.canvas_rgba.to_vec();
    // Sem plano (o neutro) a referência é o próprio vivo.
    let Some(s) = t.paint.plano_do_traco.clone() else {
        t.on_canvas_pointer(cp(*caminho_z().last().unwrap(), PointerPhase::Up));
        return (vivo.clone(), vivo);
    };
    let tudo = super::Region {
        x: 0,
        y: 0,
        w: W as u32,
        h: H as u32,
    };
    t.compoe_o_plano(&s, tudo);
    let referencia = t.canvas_rgba.to_vec();
    t.on_canvas_pointer(cp(*caminho_z().last().unwrap(), PointerPhase::Up));
    (vivo, referencia)
}

/// `(máx |Δ|, texels com |Δ| > 2)` entre duas telas, em qualquer canal.
fn diferenca(a: &[u8], b: &[u8]) -> (u8, usize) {
    let mut max = 0u8;
    let mut n = 0;
    for (p, q) in a.chunks(4).zip(b.chunks(4)) {
        let d = (0..4).map(|k| p[k].abs_diff(q[k])).max().unwrap();
        max = max.max(d);
        n += usize::from(d > 2);
    }
    (max, n)
}

/// As variantes do smoke: o nome e o ajuste por cima do Glaze Round.
type Ajuste = fn(&mut PainterTool);
/// Um tool montado com o pincel do caso.
type Monta = fn() -> PainterTool;
const VARIANTES: [(&str, Ajuste); 9] = [
    ("padrao_s06", |_| {}),
    ("padrao_s06_molhada", |t| t.set_traco_molhada(0.8)),
    ("padrao_s06_queimada", |t| t.set_traco_queimada(0.8)),
    ("padrao_s06_flow02_molhada", |t| {
        t.set_brush_flow(0.2);
        t.set_traco_molhada(0.8);
    }),
    ("fabrica", |_| {}),
    ("flow_alto", |t| t.set_brush_flow(1.0)),
    ("molhada", |t| t.set_traco_molhada(0.8)),
    ("queimada", |t| t.set_traco_queimada(0.8)),
    ("flow_molhada_queimada", |t| {
        t.set_brush_flow(1.0);
        t.set_traco_molhada(0.8);
        t.set_traco_queimada(0.8);
    }),
];

/// ⛔ Sonda + fotos (`PH2D_FOTO_DIR`): por variante, o vivo, a referência e o mapa da diferença (×8).
#[test]
#[ignore = "sonda: escreve PNG em PH2D_FOTO_DIR"]
fn foto_das_orlas_do_traco() {
    let dir = std::env::var("PH2D_FOTO_DIR").ok();
    let sufixo = std::env::var("PH2D_FOTO_SUFIXO").unwrap_or_else(|_| "antes".into());
    for (nome, f) in VARIANTES {
        let mut t = if nome.starts_with("padrao") {
            padrao(f)
        } else {
            glaze_round(f)
        };
        let (vivo, refe) = vivo_e_referencia(&mut t);
        let (max, n) = diferenca(&vivo, &refe);
        eprintln!("{nome}: vivo × referência máx |Δ| {max} · texels > 2: {n}");
        let Some(dir) = dir.as_deref() else { continue };
        let grava = |sub: &str, px: &[u8], zoom: usize| {
            let mut c = super::tests::look_watercolor_arc::Canvas::new(W * zoom, H * zoom);
            for y in 0..H * zoom {
                for x in 0..W * zoom {
                    let i = ((y / zoom) * W + x / zoom) * 4;
                    c.set(x, y, [px[i], px[i + 1], px[i + 2]]);
                }
            }
            let p = std::path::Path::new(dir).join(format!("orla_{nome}_{sub}_{sufixo}.png"));
            super::tests::look_watercolor_arc::write_png(&p, &c).expect("PNG");
        };
        grava("vivo", &vivo, 2);
        // A borda da diagonal ampliada ×6 (60×60 a partir de (70, 75)): a serrilha vê-se aqui.
        let (x0, y0, lado, zoom) = (70usize, 75usize, 60usize, 6usize);
        let mut c = super::tests::look_watercolor_arc::Canvas::new(lado * zoom, lado * zoom);
        for y in 0..lado * zoom {
            for x in 0..lado * zoom {
                let i = ((y0 + y / zoom) * W + x0 + x / zoom) * 4;
                c.set(x, y, [vivo[i], vivo[i + 1], vivo[i + 2]]);
            }
        }
        let p = std::path::Path::new(dir).join(format!("orla_{nome}_borda_x6_{sufixo}.png"));
        super::tests::look_watercolor_arc::write_png(&p, &c).expect("PNG");
        let mapa: Vec<u8> = vivo
            .chunks(4)
            .zip(refe.chunks(4))
            .flat_map(|(p, q)| {
                let d = (0..3).map(|k| p[k].abs_diff(q[k])).max().unwrap();
                let v = d.saturating_mul(8);
                [v, v, v, 255]
            })
            .collect();
        grava("diferenca_x8", &mapa, 1);
    }
}

/// O traço fechado e o plano dele (antes do pen-up): a tela e o alfa do plano por texel.
fn tela_e_plano(t: &mut PainterTool) -> (Vec<u8>, Vec<f32>) {
    pinta_aberto(t);
    let tela = t.canvas_rgba.to_vec();
    let plano = t
        .paint
        .plano_do_traco
        .as_ref()
        .expect("o traço pinta no plano")
        .plano
        .chunks(4)
        .map(|p| f32::from(p[3]) / 255.0)
        .collect();
    t.on_canvas_pointer(cp(*caminho_z().last().unwrap(), PointerPhase::Up));
    (tela, plano)
}

/// O maior |Δ| de canal entre o texel `i` de duas telas.
fn d(a: &[u8], b: &[u8], i: usize) -> f32 {
    (0..3)
        .map(|k| f32::from(a[i * 4 + k].abs_diff(b[i * 4 + k])))
        .fold(0.0, f32::max)
}

/// ⭐ **B — o lote compõe a orla como a passagem única** (os rectângulos do dono): o traço vivo, lote a lote, é a
/// MESMA tela que o plano final composto de uma vez — com a orla molhada e com a queimada.
#[test]
fn o_lote_compoe_a_orla_como_a_passagem_unica() {
    let casos: [(&str, Monta); 4] = [
        ("Glaze Round · molhada 0,8", || {
            glaze_round(|t| t.set_traco_molhada(0.8))
        }),
        ("Glaze Round · queimada 0,8", || {
            glaze_round(|t| t.set_traco_queimada(0.8))
        }),
        // A ponta DURA: a fronteira do plano chega ao bordo do lote (a orla alcança fora dele).
        ("omissão · dureza 1 · molhada 0,8", || {
            padrao(|t| {
                t.paint.brush.hardness = 1.0;
                t.set_traco_molhada(0.8);
            })
        }),
        ("omissão · Flow 0,2 · molhada 0,8", || {
            padrao(|t| {
                t.set_brush_flow(0.2);
                t.set_traco_molhada(0.8);
            })
        }),
    ];
    for (nome, mk) in casos {
        let (vivo, refe) = vivo_e_referencia(&mut mk());
        let (max, n) = diferenca(&vivo, &refe);
        assert!(
            max <= 1,
            "{nome}: o vivo difere da passagem única em rectângulos — máx |Δ| {max}, {n} texels > 2"
        );
    }
}

/// ⭐ **A — a orla molhada não come o antialias** (*«bordas mais pixeladas com wet edge alto»*): nenhum texel do
/// traço fica mais claro que o miolo (`1 − molhada` do que o mesmo traço sem orla pinta ali).
#[test]
fn a_orla_molhada_nao_come_o_antialias() {
    let base = fundo();
    let casos: [(&str, Ajuste); 2] = [
        ("Glaze Round · Flow 1", |t| t.set_brush_flow(1.0)),
        ("Glaze Round de fábrica", |_| {}),
    ];
    let m = 0.8;
    for (nome, aj) in casos {
        let (ctrl, _) = tela_e_plano(&mut glaze_round(aj));
        let (wet, _) = tela_e_plano(&mut glaze_round(|t| {
            aj(t);
            t.set_traco_molhada(m);
        }));
        let comidos = (0..W * H)
            .filter(|&i| {
                let dc = d(&ctrl, &base, i);
                dc >= 3.0 && d(&wet, &base, i) < (1.0 - m) * dc - 2.0
            })
            .count();
        assert_eq!(
            comidos, 0,
            "{nome}: a orla molhada comeu {comidos} texels da franja"
        );
    }
    // E no pincel por omissão (Strength 0,6, a franja macia larga); o tecto 1 arma o plano sem tapar nada.
    let (ctrl, _) = tela_e_plano(&mut padrao(|t| t.set_traco_tecto(1.0)));
    let (wet, _) = tela_e_plano(&mut padrao(|t| t.set_traco_molhada(m)));
    let comidos = (0..W * H)
        .filter(|&i| {
            let dc = d(&ctrl, &base, i);
            dc >= 3.0 && d(&wet, &base, i) < (1.0 - m) * dc - 2.0
        })
        .count();
    assert_eq!(
        comidos, 0,
        "omissão: a orla molhada comeu {comidos} texels da franja"
    );
}

/// ⭐ **A — a orla queimada não tem degrau na franja**: o escurecimento por unidade de tinta (`Δ ÷ α do plano`)
/// muda devagar entre vizinhos; o degrau de 1 px (a franja sem queimar, a orla toda queimada) é a serrilha.
#[test]
fn a_orla_queimada_nao_tem_degrau_na_franja() {
    let (ctrl, _) = tela_e_plano(&mut glaze_round(|t| t.set_brush_flow(1.0)));
    let (q, plano) = tela_e_plano(&mut glaze_round(|t| {
        t.set_brush_flow(1.0);
        t.set_traco_queimada(1.0);
    }));
    // Só sobre o fundo liso: na grelha escura o Linear Burn satura no preto (o degrau é do fundo, não da orla).
    let liso = |i: usize| (i % W) % 40 >= 2 && (i / W) % 40 >= 2;
    let razao: Vec<Option<f32>> = (0..W * H)
        .map(|i| (plano[i] >= 0.08 && liso(i)).then(|| d(&ctrl, &q, i) / plano[i]))
        .collect();
    let max = razao.iter().flatten().copied().fold(0.0, f32::max);
    assert!(max > 20.0, "a queimada não escureceu nada: {max}");
    let mut degraus = 0;
    for y in 0..H - 1 {
        for x in 0..W - 1 {
            let i = y * W + x;
            for j in [i + 1, i + W] {
                if let (Some(a), Some(b)) = (razao[i], razao[j])
                    && (a - b).abs() > 0.5 * max
                {
                    degraus += 1;
                }
            }
        }
    }
    assert_eq!(
        degraus, 0,
        "degraus de 1 px na orla queimada: {degraus} (máx Δ/α {max})"
    );
}
