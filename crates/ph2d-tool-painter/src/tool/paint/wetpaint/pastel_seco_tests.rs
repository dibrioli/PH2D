//! **O Soft Pastel contra o Rebelle «Pastel Dry / Dry Thin»** (prompt 35, smoke do dono 14/10): as duas leis
//! que a bancada no TEMPO mediu (`ferramentas/pastel_regua.py`; Rebelle: 0 píxeis mudam de 0,5 a 6 s).
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: usize = 400;
const H: usize = 640;

fn preset(nome: &str) -> ph2d_brush_presets::Preset {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .expect("o preset de fábrica")
        .preset
}

fn tela(nome: &str) -> PainterTool {
    let p = preset(nome);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    t.wet_preset_aplicar_volumes(&p.parametros.volumes);
    t.paint.wetpaint.tilt_on = false;
    t
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um recto de 220 px à altura `y`, 4 px por evento e um quadro por evento — até ao pen-up, sem soltar o tempo.
fn recto(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=55 {
        t.on_canvas_pointer(cp([60.0 + 4.0 * k as f32, y], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([280.0, y], PointerPhase::Up));
}

fn mudados_em(t: &mut PainterTool, quadros: u32) -> usize {
    let up = t.canvas_rgba.clone();
    for _ in 0..quadros {
        t.on_tick(1000.0 / 60.0);
    }
    up.as_chunks::<4>()
        .0
        .iter()
        .zip(t.canvas_rgba.as_chunks::<4>().0)
        .filter(|(a, b)| (0..3).any(|c| a[c].abs_diff(b[c]) > 2))
        .count()
}

/// **Um pastel SECO não se mexe depois do traço** — o «ruído no mouse up» do dono: o pigmento de uma ponta
/// sem água pousava SUSPENSO e o primeiro passo da água depois do pen-up arrastava-o (1 854 píxeis no recto
/// de 220 px, medido 14/10). O Wet Round é o CONTROLO: a régua vê a água a correr.
///
/// Mutação que tem de sangrar: `seco = false` em `ph2d-wet-paint/src/trail/transfer.rs`.
#[test]
fn o_pastel_seco_nao_se_mexe_depois_do_traco() {
    let mut t = tela("Soft Pastel");
    recto(&mut t, 70.0);
    assert_eq!(
        mudados_em(&mut t, 360),
        0,
        "o Soft Pastel mudou depois do pen-up — o pigmento seco pousou suspenso"
    );
    let mut c = tela("Wet Round");
    recto(&mut c, 70.0);
    assert!(
        mudados_em(&mut c, 360) > 100,
        "controlo: a tinta molhada tem de correr depois do pen-up, senão a régua não vê nada"
    );
}

/// **A largura do Soft Pastel não depende de ONDE se pinta.** O grão de fábrica era Noise 0,4 preso ao papel:
/// um ladrilho de ~640 px, logo uma MANCHA — o mesmo traço saía com 15 px a `y = 40` e 34 a `y = 450`
/// (medido 14/10). O grão do pastel é de 2–4 px (o Dry Thin do Rebelle: 30 · 29 px no recto e no zig-zague).
///
/// Mutação que tem de sangrar: o grão da receita de volta a `("Noise", 0.4)` (e os presets regenerados).
#[test]
fn a_largura_do_pastel_nao_depende_da_altura_na_tela() {
    let larguras: Vec<usize> = [40.0f32, 250.0, 450.0, 600.0]
        .into_iter()
        .map(|y| {
            let mut t = tela("Soft Pastel");
            recto(&mut t, y);
            let a: Vec<f32> = (y as usize - 40..y as usize + 40)
                .map(|py| {
                    (110..240)
                        .map(|px| {
                            let r = t.canvas_rgba[(py * W + px) * 4];
                            f32::from(255 - r) / 255.0
                        })
                        .sum::<f32>()
                        / 130.0
                })
                .collect();
            let m = a.iter().copied().fold(0.0f32, f32::max);
            a.iter().filter(|v| **v >= 0.5 * m).count()
        })
        .collect();
    let (min, max) = (
        *larguras.iter().min().expect("larguras"),
        *larguras.iter().max().expect("larguras"),
    );
    assert!(
        max - min <= 3,
        "a largura do Soft Pastel varia com a altura: {larguras:?} — o grão é uma mancha, não um grão"
    );
}
