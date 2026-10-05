//! ⭐⭐ **A luz do impasto, com RELEVO, é a mesma de antes da poda do 3D** (ADR-0179) — ao byte.
//!
//! A `line/poda-3d` tirou a forma doada da luz do impasto: o caminho 2D sempre passou a forma
//! NEUTRA (`[0,0,1,0]`, oclusão `1.0`), e a poda fixou esse neutro e simplificou a conta. A álgebra
//! diz que nada muda; os gates «a tinta PLANA fica intocada ao byte» não o alcançam, porque com
//! relevo zero a normal é `[0,0,1]` qualquer que seja a forma.
//!
//! ⭐ **O oráculo é o PRÓPRIO PH2D de antes da poda, corrido:** este ficheiro foi corrido, sem uma
//! linha mudada, numa worktree em `b1a6f9b07` (o último `main` com 3D) e nesta árvore, e os quatro
//! hashes abaixo são a saída das DUAS corridas — iguais. Mudar a luz do impasto daqui em diante é
//! mudar estes números de propósito, com a medição ao lado.
//!
//! A fixtura é a do `impasto_light_gpu` (traços cruzados = inclinações em todas as direcções, fundo
//! que varia nos três canais = o albedo é ENTRADA do sombreamento), com quatro materiais: o neutro,
//! o metálico, a cera e o brilho alto — os termos que separam o caminho colorido do rápido.

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{PainterTool, Region};

const W: u32 = 64;
const H: u32 = 64;

/// `(shine, roughness, metallic, wax)` → o FNV-1a de 64 bits dos bytes iluminados.
/// Gravado em `b1a6f9b07` (antes da poda) e nesta árvore, 2026-10-05: iguais.
const ESPERADO: [((f32, f32, f32, f32), u64); 4] = [
    ((0.5, 0.5, 0.0, 0.0), 0xf28a_10f8_733d_3acd),
    ((0.8, 0.3, 1.0, 0.0), 0x3028_785c_bfb8_db3b),
    ((0.6, 0.4, 0.0, 0.7), 0x7bc3_cf43_895c_fdf2),
    ((1.0, 0.1, 0.3, 0.2), 0xb21e_832e_cbf4_8329),
];

fn esculpido(shine: f32, rough: f32, metallic: f32, wax: f32) -> PainterTool {
    let cp = |pos: [f32; 2], phase: PointerPhase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut px = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            px.extend_from_slice(&[(x * 4) as u8, (y * 4) as u8, 255 - (x * 2) as u8, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(px, W, H);
    t.toggle_brush_impasto();
    t.set_brush_impasto_depth(0.8);
    t.set_brush_size_px(9.0);
    t.set_brush_color_channel(0, 0.85);
    t.set_brush_color_channel(1, 0.30);
    t.set_brush_color_channel(2, 0.10);
    t.set_impasto_shine(shine);
    t.set_impasto_roughness(rough);
    t.set_impasto_metallic(metallic);
    t.set_impasto_wax(wax);
    t.set_impasto_wax_color([1.0, 0.7, 0.45]);
    for (a, b) in [([12.0f32, 20.0f32], [52.0f32, 30.0f32]), ([30.0, 8.0], [34.0, 56.0])] {
        t.on_canvas_pointer(cp(a, PointerPhase::Down));
        t.on_canvas_pointer(cp([(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5], PointerPhase::Move));
        t.on_canvas_pointer(cp(b, PointerPhase::Move));
        t.on_canvas_pointer(cp(b, PointerPhase::Up));
    }
    t
}

fn iluminado(t: &PainterTool) -> Vec<u8> {
    let mut buf = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            buf.extend_from_slice(&[
                (20 + x * 3) as u8,
                (200 - y * 2) as u8,
                (60 + (x ^ y) * 2) as u8,
                255,
            ]);
        }
    }
    t.apply_impasto_light(&mut buf, Region { x: 0, y: 0, w: W, h: H });
    buf
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[test]
fn a_luz_do_impasto_com_relevo_e_a_de_antes_da_poda() {
    let mut medidos = Vec::new();
    for ((shine, rough, metallic, wax), esperado) in ESPERADO {
        let t = esculpido(shine, rough, metallic, wax);
        let base = {
            let mut b = Vec::new();
            for y in 0..H {
                for x in 0..W {
                    b.extend_from_slice(&[
                        (20 + x * 3) as u8,
                        (200 - y * 2) as u8,
                        (60 + (x ^ y) * 2) as u8,
                        255,
                    ]);
                }
            }
            b
        };
        let lit = iluminado(&t);
        assert_ne!(lit, base, "a fixtura não acendeu nada — o gate mediria o vazio");
        let h = fnv1a(&lit);
        println!("ORACULO ({shine}, {rough}, {metallic}, {wax}) => {h:#018x}");
        medidos.push((esperado, h));
    }
    for (esperado, h) in medidos {
        assert_eq!(h, esperado, "a luz do impasto com relevo mudou (veja o cabeçalho)");
    }
}
