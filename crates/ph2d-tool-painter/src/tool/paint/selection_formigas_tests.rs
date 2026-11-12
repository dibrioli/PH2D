//! B20.11: o overlay guardado é o de sempre ao byte, e só se refaz quando a chave muda.

use super::construcoes_do_overlay_da_selecao as construcoes;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::RasterEditTool;

const S: u32 = 96;

fn selecao() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    t.set_rect_selection(10, 12, 60, 50);
    assert!(t.selection_active());
    t
}

fn bytes(t: &PainterTool, passo: u32) -> Vec<u8> {
    let img = t.selection_overlay_imagem(passo).expect("overlay");
    assert_eq!((img.width(), img.height()), (S, S));
    img.rgba().to_vec()
}

#[test]
fn o_overlay_guardado_e_o_de_sempre_e_parado_nao_se_refaz() {
    let mut t = selecao();
    let n0 = construcoes();
    for passo in [0, 3, 7] {
        let cru = t.selection_overlay_rgba(passo).expect("cru").0;
        assert_eq!(bytes(&t, passo), cru, "passo {passo}: o byte do caminho cru");
    }
    assert_eq!(construcoes() - n0, 3, "um passo novo, uma construção");
    for _ in 0..5 {
        let _ = bytes(&t, 7);
    }
    assert_eq!(construcoes() - n0, 3, "o mesmo passo parado não se refaz");

    // Cada coisa de que o desenho depende refaz, e dá o byte do caminho cru.
    t.set_selection_overlay_opacity(0.25);
    assert_eq!(bytes(&t, 7), t.selection_overlay_rgba(7).expect("cru").0);
    assert_eq!(construcoes() - n0, 4, "a opacidade refaz");
    t.set_rect_selection(20, 20, 70, 80);
    assert_eq!(bytes(&t, 7), t.selection_overlay_rgba(7).expect("cru").0);
    assert_eq!(construcoes() - n0, 5, "a máscara nova refaz");
    t.invert_selection();
    assert_eq!(bytes(&t, 7), t.selection_overlay_rgba(7).expect("cru").0);
    assert_eq!(construcoes() - n0, 6, "a inversão refaz");
    t.set_selection_offset(0.8);
    assert_eq!(bytes(&t, 7), t.selection_overlay_rgba(7).expect("cru").0);
    t.set_selection_offset(0.2);
    assert_eq!(bytes(&t, 7), t.selection_overlay_rgba(7).expect("cru").0);

    t.clear_selection();
    assert!(t.selection_overlay_imagem(7).is_none(), "sem seleção, nada");
}

/// Os anéis do Offset (Apply & Keep) mudam os contornos ciano: cada passo dá o byte do caminho cru.
#[test]
fn os_aneis_do_offset_refazem_o_overlay() {
    let mut t = selecao();
    let cru = |t: &PainterTool| t.selection_overlay_rgba(4).expect("cru").0;
    for norm in [0.8, 0.65, 0.9] {
        t.set_selection_offset(norm);
        assert_eq!(bytes(&t, 4), cru(&t), "arrasto {norm}");
        t.selection_offset_apply_keep();
        assert_eq!(bytes(&t, 4), cru(&t), "anel em {norm}");
    }
    t.set_selection_offset(0.5);
    assert_eq!(bytes(&t, 4), cru(&t), "de volta ao centro");
}

#[test]
fn duas_telas_nao_trocam_o_overlay_uma_da_outra() {
    let a = selecao();
    let mut b = selecao();
    b.set_rect_selection(1, 1, 30, 90);
    assert_ne!(bytes(&a, 2), bytes(&b, 2));
    assert_eq!(bytes(&a, 2), a.selection_overlay_rgba(2).expect("cru").0);
}

/// Sonda (B20.11): o custo por quadro do overlay a 4K, refeito × guardado. `-- --ignored --nocapture`.
#[test]
#[ignore = "sonda de relógio"]
fn sonda_formigas_4k() {
    let (w, h) = (3840u32, 2160u32);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_rect_selection(400, 300, 3000, 1800);
    let min = |f: &mut dyn FnMut()| {
        (0..12)
            .map(|_| {
                let t0 = std::time::Instant::now();
                f();
                t0.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min)
    };
    let mut passo = 0;
    for _ in 0..3 {
        let refeito = min(&mut || {
            passo += 1;
            let _ = std::hint::black_box(t.selection_overlay_imagem(passo));
        });
        let guardado = min(&mut || {
            let _ = std::hint::black_box(t.selection_overlay_imagem(passo));
        });
        eprintln!("[formigas 4K] refeito {refeito:.2} ms · guardado {guardado:.4} ms");
    }
}
