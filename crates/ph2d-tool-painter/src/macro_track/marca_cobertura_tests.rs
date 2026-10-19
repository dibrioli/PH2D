//! **O artista escreveu aqui mesmo sem mudar o byte** (smoke do dono, 19/10: faixas brancas no Play onde a
//! pincelada nova cruza o miolo de uma gravada). Preto sobre o preto que a gravação deixou no fim não muda o
//! píxel; a régua «o que mudou» deixava esse texel sem marca e o Play mostrava ali a gravação daquele
//! instante. Um texel é do artista onde o pincel o cobre com força ≥ ½ (os dabs compostos como o carimbo os
//! compõe): aí o resultado é o do pincel, seja o que for que estava por baixo; na borda fraca mostrar a
//! gravação erra menos que mostrar o fim.

use super::marca::MarcaDoArtista;
use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};

const LADO: u32 = 128;
const L: usize = LADO as usize;
/// A faixa preta que a gravação deixou no fim (linhas).
const FAIXA: std::ops::Range<usize> = 60..68;

fn i(x: usize, y: usize) -> usize {
    y * L + x
}

/// A tela branca com a faixa preta opaca, o Painter em `meio` a pincel preto de 16 px, a marca posta.
fn com_a_faixa(meio: PaintMedia) -> PainterTool {
    let mut src = vec![255u8; L * L * 4];
    for y in FAIXA {
        for x in 0..L {
            src[i(x, y) * 4..i(x, y) * 4 + 3].fill(0);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, LADO, LADO);
    t.set_paint_media(meio);
    t.poe_a_marca(Some(MarcaDoArtista::nova(LADO, LADO, 1)));
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    t
}

/// Uma pincelada vertical em `x`, de cima a baixo, um quadro por evento.
fn vertical(t: &mut PainterTool, x: f32) {
    let n = 12;
    for k in 0..=n {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let y = 20.0 + 90.0 * k as f32 / n as f32;
        let phase = match k {
            0 => PointerPhase::Down,
            k if k == n => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        t.on_canvas_pointer(CanvasPointer {
            pos: [x, y],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        drop(t.take_preview_arc());
        t.on_tick(16.667);
    }
}

fn imagem(t: &PainterTool) -> Vec<u8> {
    t.imagem_aplicada().expect("documento").0
}

/// A régua: a pincelada Digital sobre o branco (linha 40) — a força dela em cada coluna lê-se no byte
/// (preto sobre branco, composto no espaço do píxel: força ≥ ½ ⇔ byte ≤ 127). `true` = miolo (≤ 120),
/// `false` = borda fraca (135..255), `None` = perto do limiar ou fora do pincel.
fn regua() -> Vec<Option<bool>> {
    let mut t = com_a_faixa(PaintMedia::Digital);
    vertical(&mut t, 64.0);
    let depois = imagem(&t);
    (0..L)
        .map(|x| match depois[i(x, 40) * 4] {
            v if v <= 120 => Some(true),
            135..=254 => Some(false),
            _ => None,
        })
        .collect()
}

/// ⭐ Digital e Watercolor (os meios que só marcavam o que mudou; medido 19/10: 136 de 136 texels do
/// miolo sobre a faixa sem marca nos dois — Impasto muda o relevo e o Wet Paint marca o disco inteiro).
#[test]
fn pintar_por_cima_do_que_ja_e_o_resultado_marca_o_miolo_do_pincel() {
    let r = regua();
    let miolo = r.iter().filter(|c| **c == Some(true)).count();
    let borda = r.iter().filter(|c| **c == Some(false)).count();
    assert!(
        miolo > 4 && borda > 4,
        "controlo: a régua tem miolo ({miolo}) e borda ({borda})"
    );
    for meio in [PaintMedia::Digital, PaintMedia::Watercolor] {
        let mut t = com_a_faixa(meio);
        let antes = imagem(&t);
        vertical(&mut t, 64.0);
        let depois = imagem(&t);
        let px = |v: &[u8], k: usize| v[k * 4..k * 4 + 4].to_vec();
        assert_ne!(
            px(&antes, i(64, 40)),
            px(&depois, i(64, 40)),
            "controlo {meio:?}: pintou o branco"
        );
        let m = t.marca_do_artista().expect("a marca").clone();
        for (x, c) in r.iter().enumerate() {
            for y in FAIXA {
                let k = i(x, y);
                assert_eq!(
                    px(&antes, k),
                    px(&depois, k),
                    "controlo {meio:?}: preto sobre o preto ({x}, {y})"
                );
                match c {
                    Some(true) => assert!(
                        m.marcado(k),
                        "{meio:?}: o miolo sobre a faixa ({x}, {y}) não ficou do artista — o Play mostra ali a gravação"
                    ),
                    Some(false) => assert!(
                        !m.marcado(k),
                        "{meio:?}: a borda fraca ({x}, {y}) marcou — o Play mostraria ali o fim antes do tempo"
                    ),
                    None => {}
                }
            }
        }
    }
}
