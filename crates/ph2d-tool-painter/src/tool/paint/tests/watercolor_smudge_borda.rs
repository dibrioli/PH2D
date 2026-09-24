//! **O Smudge da aquarela na borda da tela não abre transparência** (report do dono com foto,
//! 2026-09-24: *«o smear empurra pixel das bordas e provoca transparência»*).
//!
//! O mecanismo: o arrasto da BASE ([`ph2d_painter_brush::smear_dab`]) levanta cada pixel do destino
//! em `destino − passo`, e quando essa origem caía FORA da tela lia **transparente**. O comentário
//! ao lado dizia que era *«a orla do dab, falloff ~0 — efeito nulo»*: é verdade a meio da tela e
//! **falso quando o CENTRO do dab está na borda**, onde o peso é cheio. Arrastar da borda para dentro
//! pintava transparência na tela. Medido pela porta do produto (esta fixtura, antes da cura):
//! `r = 24` borda→dentro `4 845` texels com alfa `< 255` (mínimo `178`), fora→dentro `6 068`, ao
//! longo da borda `2 785` — e os outros dois arrastos da casa (o Smear digital e a camada Smear da
//! pilha) liam **zero** no mesmo traço, porque os dois amostram PRESOS à borda (`bilinear_clamped`).
//!
//! ⚠️ O CONTROLO positivo vem primeiro: um zero de transparência sobre um traço que não mexeu nada
//! não ilibaria ninguém.

use super::*;
use crate::tool::paint::media::PaintMedia;

const SIZE: u32 = 128;

/// Tela OPACA em xadrez (o arrasto vê-se) com o pincel de aquarela armado e o Smudge no máximo.
fn tela() -> PainterTool {
    let mut src = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = ((y * SIZE + x) * 4) as usize;
            let c = if (x / 8 + y / 8) % 2 == 0 {
                [220, 40, 40, 255]
            } else {
                [250, 230, 230, 255]
            };
            src[i..i + 4].copy_from_slice(&c);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, SIZE, SIZE);
    t.set_brush_size_px(48.0);
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_wet_smudge(1.0);
    t
}

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    for i in 1..=30 {
        let f = i as f32 / 30.0;
        let p = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
        frame(t);
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
    frame(t);
    frame(t);
}

#[test]
fn o_smudge_da_aquarela_nao_abre_transparencia_na_borda() {
    let e = SIZE as f32;
    let casos: [(&str, [f32; 2], [f32; 2]); 3] = [
        ("da borda para dentro", [e - 1.0, 64.0], [40.0, 64.0]),
        ("de fora para dentro", [e + 30.0, 64.0], [40.0, 64.0]),
        ("ao longo da borda", [e - 6.0, 10.0], [e - 6.0, e - 10.0]),
    ];
    for (nome, a, b) in casos {
        let mut t = tela();
        let antes = t.canvas_rgba.clone();
        traco(&mut t, a, b);
        let mudou = t
            .canvas_rgba
            .as_chunks::<4>()
            .0
            .iter()
            .zip(antes.as_chunks::<4>().0)
            .filter(|(p, q)| p != q)
            .count();
        assert!(
            mudou > 1_000,
            "{nome}: CONTROLO — o traço tem de esfregar tinta ({mudou} texels mudaram)"
        );
        let transp: Vec<u8> = t
            .canvas_rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[3])
            .filter(|&a| a < 255)
            .collect();
        assert!(
            transp.is_empty(),
            "{nome}: o Smudge abriu transparência numa tela opaca — {} texels, alfa mínimo {:?}",
            transp.len(),
            transp.iter().min()
        );
    }
}
