//! **A Borracha na tinta morta imita a viva** (decisão 4 do dono): nem corte seco, nem nada de novo a nascer.

use super::agua_sobre_oleo_tests::{H, W, mostrada};
use super::borracha_de_fabrica_tests::{apaga, diferentes};
use super::sonda_borracha_morta::morta;
use super::*;

fn soma_delta(a: &[u8], b: &[u8]) -> u64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| u64::from(x.abs_diff(*y)))
        .sum()
}

fn borracha(t: &mut PainterTool) {
    t.set_paint_tool_mode("eraser");
}

/// O pixel do miolo do rasto, onde a pressão é fraca (`y = 75`) ou cheia (`y = 100`, ainda sobre o óleo).
fn no_rasto(p: &[u8], y: usize) -> &[u8] {
    let x = 120 + (y - 60) * 7 / 20;
    &p[(y * W + x) * 4..(y * W + x) * 4 + 3]
}

/// ⭐ Uma passada amassa: muda mais pixels do que o corte seco mudava, com muito menos força, e a tinta continua lá.
#[test]
fn uma_passada_amassa_e_nao_corta() {
    let mut t = morta();
    let antes = mostrada(&mut t);
    borracha(&mut t);
    let r = apaga(&mut t);
    assert!(
        diferentes(&antes, &r) > 1036,
        "o rasto é mais largo que o corte"
    );
    assert!(
        soma_delta(&antes, &r) < 196_654 / 2,
        "metade da força do corte"
    );
    let azul = |c: &[u8]| c[2] > c[0] + 40;
    assert!(
        azul(no_rasto(&r, 75)),
        "a tinta fica: {:?}",
        no_rasto(&r, 75)
    );
    assert!(t.paint.wetpaint.session.is_none(), "nenhuma sessão nasce");
}

/// Passada a passada a tinta vai saindo, até ao papel.
#[test]
fn varias_passadas_tiram_a_tinta() {
    let mut t = morta();
    borracha(&mut t);
    let alfa = |t: &PainterTool| -> u64 { t.canvas_rgba.chunks(4).map(|p| u64::from(p[3])).sum() };
    let a0 = alfa(&t);
    apaga(&mut t);
    let a1 = alfa(&t);
    for _ in 0..12 {
        apaga(&mut t);
    }
    let a2 = alfa(&t);
    assert!(a1 < a0, "uma passada tira alguma coisa");
    assert!(a2 < a1, "e continua a tirar");
    let o = (100 * W + 134) * 4;
    assert!(
        t.canvas_rgba[o + 3] < 40,
        "ao fim, papel: α {}",
        t.canvas_rgba[o + 3]
    );
}

/// O Erase Strength é um controlo vivo aqui também: mais força, menos tinta.
#[test]
fn o_erase_strength_manda() {
    let alfa = |erase: f64| -> u64 {
        let mut t = morta();
        borracha(&mut t);
        t.paint.wetpaint.knobs.erase = erase;
        apaga(&mut t);
        t.canvas_rgba.chunks(4).map(|p| u64::from(p[3])).sum()
    };
    assert!(alfa(0.9) < alfa(0.1));
}

/// Desfazer devolve a imagem ao bit, de uma vez.
#[test]
fn o_undo_devolve_a_imagem() {
    let mut t = morta();
    let antes = mostrada(&mut t);
    borracha(&mut t);
    apaga(&mut t);
    assert!(t.undo_last(), "o traço desfaz-se");
    assert_eq!(diferentes(&antes, &mostrada(&mut t)), 0);
}

/// A selecção prende: fora dela, nem um pixel muda.
#[test]
fn a_selecao_prende_a_borracha() {
    let mut t = morta();
    t.set_rect_selection(100, 50, 20, 40);
    let antes = t.canvas_rgba.to_vec();
    borracha(&mut t);
    apaga(&mut t);
    let mut mudou_fora = 0;
    let mut mudou_dentro = 0;
    for y in 0..H {
        for x in 0..W {
            let o = (y * W + x) * 4;
            if antes[o..o + 4] != t.canvas_rgba[o..o + 4] {
                if (100..120).contains(&x) && (50..90).contains(&y) {
                    mudou_dentro += 1;
                } else {
                    mudou_fora += 1;
                }
            }
        }
    }
    assert_eq!(mudou_fora, 0, "fora da selecção");
    assert!(mudou_dentro > 0, "CONTROLO: dentro dela apaga");
}

/// Com a tinta viva a Borracha é a dela.
#[test]
fn a_tinta_viva_nao_passa_por_aqui() {
    let t = super::agua_sobre_oleo_tests::com_oleo();
    assert!(!t.borracha_morta_assume(), "há sessão: a Borracha é a viva");
}
