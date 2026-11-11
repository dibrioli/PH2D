//! B40.A1 — Color Fill, Cut e Colar respeitam o trinco de alfa e a proteção da máscara, como o carimbo.
//! Lê a tela antes/depois por texel: α idêntico no travado, RGBA idêntico no protegido.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::{BrushSpec, Falloff};
use std::sync::Arc;

const N: u32 = 32;

/// Metade esquerda vermelha opaca, metade direita transparente.
fn meia_tela() -> PainterTool {
    let mut t = PainterTool::default();
    let mut rgba = vec![0u8; (N * N * 4) as usize];
    for y in 0..N {
        for x in 0..N / 2 {
            let i = ((y * N + x) * 4) as usize;
            rgba[i..i + 4].copy_from_slice(&[200, 30, 30, 255]);
        }
    }
    t.set_source(rgba, N, N);
    t.paint.brush = BrushSpec {
        radius_px: 4.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 1.0],
        space_attenuation: false,
        ..Default::default()
    };
    t
}

fn trava(t: &mut PainterTool) {
    let id = t.layers.active().expect("uma camada");
    t.set_layer_alpha_locked(id, true);
}

/// Protege a metade esquerda (máscara preta = congelada).
fn protege_a_esquerda(t: &mut PainterTool) {
    t.ensure_mask_scratch();
    let mut m = vec![255u8; (N * N * 4) as usize];
    for y in 0..N {
        for x in 0..N / 2 {
            let i = ((y * N + x) * 4) as usize;
            m[i..i + 3].copy_from_slice(&[0, 0, 0]);
        }
    }
    t.paint.mask_scratch_rgba = Arc::new(m);
    t.bump_mask_scratch_gen();
    assert!(
        t.mask_protection_active(),
        "fixtura: a proteção tem de estar viva"
    );
}

fn alfas(t: &PainterTool) -> Vec<u8> {
    t.canvas_rgba.iter().skip(3).step_by(4).copied().collect()
}

fn esquerda(t: &PainterTool) -> Vec<u8> {
    let mut v = Vec::new();
    for y in 0..N {
        let i = (y * N * 4) as usize;
        v.extend_from_slice(&t.canvas_rgba[i..i + (N / 2 * 4) as usize]);
    }
    v
}

fn px(t: &PainterTool, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * N + x) * 4) as usize;
    [0, 1, 2, 3].map(|k| t.canvas_rgba[i + k])
}

/// Copia um quadrado da metade esquerda e arma-o flutuando sobre `(cx, cy)`.
fn peca_sobre(t: &mut PainterTool, cx: f32, cy: f32) {
    t.set_rect_selection(4, 4, 8, 8);
    t.selection_copy();
    t.selection_paste();
    if let Some(p) = t.paint.paste_patch.as_mut() {
        p.center = [cx, cy];
    }
    t.redraw_paste_patch();
}

#[test]
fn color_fill_com_alfa_travado_nao_muda_o_alfa() {
    let mut t = meia_tela();
    trava(&mut t);
    let antes = alfas(&t);
    t.set_rect_selection(0, 0, N, N);
    t.selection_color_fill();
    assert_eq!(
        alfas(&t),
        antes,
        "o trinco de alfa tem de guardar a silhueta"
    );
    assert_eq!(
        px(&t, 4, 4),
        [0, 0, 255, 255],
        "o Color Fill pinta a cor onde há tinta"
    );
}

#[test]
fn cut_com_alfa_travado_nao_muda_o_alfa() {
    let mut t = meia_tela();
    trava(&mut t);
    let antes = alfas(&t);
    t.set_rect_selection(0, 0, N, N);
    t.selection_cut();
    assert!(t.paint.selection_clipboard.is_some(), "o Cut ainda copia");
    assert_eq!(
        alfas(&t),
        antes,
        "o trinco de alfa tem de guardar a silhueta"
    );
}

#[test]
fn colar_com_alfa_travado_nao_muda_o_alfa() {
    let mut t = meia_tela();
    trava(&mut t);
    let antes = alfas(&t);
    peca_sobre(&mut t, 16.0, 16.0); // meia peça sobre o vazio
    assert_eq!(
        alfas(&t),
        antes,
        "a peça flutuante não pode mudar o alfa travado"
    );
    assert!(t.paste_commit());
    assert_eq!(alfas(&t), antes, "o Enter não pode mudar o alfa travado");
}

#[test]
fn color_fill_nao_pinta_o_protegido() {
    let mut t = meia_tela();
    protege_a_esquerda(&mut t);
    let antes = esquerda(&t);
    t.set_rect_selection(0, 0, N, N);
    t.selection_color_fill();
    assert_eq!(esquerda(&t), antes, "a metade protegida não pode mudar");
    assert_eq!(
        px(&t, 24, 4),
        [0, 0, 255, 255],
        "a metade livre recebe o fill"
    );
}

#[test]
fn cut_nao_apaga_o_protegido() {
    let mut t = meia_tela();
    protege_a_esquerda(&mut t);
    let antes = esquerda(&t);
    t.set_rect_selection(0, 0, N, N);
    t.selection_cut();
    assert_eq!(
        esquerda(&t),
        antes,
        "a metade protegida não pode ser apagada"
    );
}

#[test]
fn colar_nao_pinta_o_protegido() {
    let mut t = meia_tela();
    // A peça vem da esquerda e é largada na direita protegida: o espelho do caso — proteger a direita.
    t.ensure_mask_scratch();
    let mut m = vec![255u8; (N * N * 4) as usize];
    for y in 0..N {
        for x in N / 2..N {
            let i = ((y * N + x) * 4) as usize;
            m[i..i + 3].copy_from_slice(&[0, 0, 0]);
        }
    }
    t.paint.mask_scratch_rgba = Arc::new(m);
    t.bump_mask_scratch_gen();
    assert!(t.mask_protection_active());
    let antes = t.canvas_rgba.as_ref().clone();
    peca_sobre(&mut t, 24.0, 8.0);
    assert!(t.paste_commit());
    assert_eq!(*t.canvas_rgba, antes, "a peça não pode pousar no protegido");
}

/// O RED guardado: pelo caminho antigo (a guarda neutra, verbatim o código de antes do B40.A1) cada um
/// dos seis gates acima reprova; pela porta nova passam. Sem isto um gate que passasse por acaso no
/// caminho antigo não provaria nada.
#[test]
fn o_caminho_antigo_reprova_os_seis_gates() {
    let casos: [(&str, fn()); 6] = [
        (
            "color_fill_alfa",
            color_fill_com_alfa_travado_nao_muda_o_alfa,
        ),
        ("cut_alfa", cut_com_alfa_travado_nao_muda_o_alfa),
        ("colar_alfa", colar_com_alfa_travado_nao_muda_o_alfa),
        ("color_fill_protegido", color_fill_nao_pinta_o_protegido),
        ("cut_protegido", cut_nao_apaga_o_protegido),
        ("colar_protegido", colar_nao_pinta_o_protegido),
    ];
    for (nome, caso) in casos {
        let r = std::panic::catch_unwind(|| {
            super::escrita_direta::CAMINHO_ANTIGO.set(true);
            caso();
        });
        super::escrita_direta::CAMINHO_ANTIGO.set(false);
        assert!(
            r.is_err(),
            "{nome}: o caminho antigo devia reprovar (o gate não discrimina)"
        );
    }
}
