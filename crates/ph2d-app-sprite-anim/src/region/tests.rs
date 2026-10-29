//! Os gates do editor de Region (plano GEF 16, gates 2 e 3): a área encaixa nas linhas de Columns × Rows; `Alt` e
//! 1 × 1 vão ao píxel; nunca sai da imagem; um clique escolhe a célula; OK entrega a área e a grelha; Cancel nada.

use super::doc::{Outcome, RegionDoc};
use ph2d_editor_core::zones::Rect;

/// Uma aba sobre a folha 800 × 400 do plano, grelha 8 × 4, vista 1 : 1 com a imagem em (0, 0) e a barra fora.
fn aba() -> RegionDoc {
    let mut d = RegionDoc::new(1, 1, "Player".into(), [800, 400], None, [8, 4]);
    d.layout.area = Rect::new(0.0, -100.0, 2000.0, 2000.0);
    d.layout.canvas = Rect::new(0.0, 0.0, 2000.0, 1000.0);
    d.zoom = 1.0;
    d.origin = [0.0, 0.0];
    d
}

fn multiplo_de_100(r: [f32; 4]) -> bool {
    r.iter().all(|v| v.rem_euclid(100.0) == 0.0)
}

#[test]
fn a_area_desenhada_encaixa_nas_linhas_da_grelha() {
    let mut d = aba();
    // Desenhar fora da área (a área é a imagem inteira: carregar fora dela, à direita, não acerta).
    d.rect = [0.0, 0.0, 100.0, 100.0];
    d.press(437.0, 163.0);
    d.motion(612.0, 288.0);
    d.release(612.0, 288.0);
    assert_eq!(d.rect, [400.0, 200.0, 200.0, 100.0]);
    assert!(multiplo_de_100(d.rect));
    // Mover: o canto encaixa, o tamanho fica.
    d.press(450.0, 230.0);
    d.motion(530.0, 160.0);
    d.release(530.0, 160.0);
    assert_eq!(d.rect, [500.0, 100.0, 200.0, 100.0]);
    // A alça do canto de baixo à direita.
    let (_, _, h) = d
        .handles()
        .into_iter()
        .find(|(sx, sy, _)| (*sx, *sy) == (1, 1))
        .expect("alça");
    d.press(h.x + h.w * 0.5, h.y + h.h * 0.5);
    d.motion(777.0, 349.0);
    d.release(777.0, 349.0);
    assert_eq!(d.rect, [500.0, 100.0, 300.0, 200.0]);
    assert!(multiplo_de_100(d.rect));
}

#[test]
fn alt_e_uma_coluna_vao_ao_pixel() {
    let mut d = aba();
    d.rect = [0.0, 0.0, 100.0, 100.0];
    d.set_keys(true, None);
    d.press(437.3, 163.6);
    d.motion(612.4, 288.8);
    d.release(612.4, 288.8);
    assert_eq!(d.rect, [437.0, 164.0, 175.0, 125.0]);
    let mut d = RegionDoc::new(1, 1, "x".into(), [800, 400], None, [1, 1]);
    assert_eq!(d.snap(0, 437.4), 437.0, "1 × 1: ao píxel");
    d.alt = false;
    assert_eq!(d.snap(1, 163.6), 164.0);
}

#[test]
fn a_area_nunca_sai_da_imagem() {
    let mut d = aba();
    d.rect = [600.0, 300.0, 200.0, 100.0];
    d.press(700.0, 350.0);
    d.motion(5000.0, 5000.0);
    d.release(5000.0, 5000.0);
    assert_eq!(d.rect, [600.0, 300.0, 200.0, 100.0], "presa no canto");
    d.press(700.0, 350.0);
    d.motion(-5000.0, -5000.0);
    d.release(-5000.0, -5000.0);
    assert_eq!(d.rect, [0.0, 0.0, 200.0, 100.0]);
    // A alça esquerda passa da direita: fica com uma célula, nunca invertida nem nula.
    let (_, _, h) = d
        .handles()
        .into_iter()
        .find(|(sx, sy, _)| (*sx, *sy) == (-1, 0))
        .expect("alça");
    d.press(h.x + h.w * 0.5, h.y + h.h * 0.5);
    d.motion(1900.0, 50.0);
    d.release(1900.0, 50.0);
    let [x, y, w, h] = d.rect;
    assert!(
        w >= 1.0 && h >= 1.0 && x + w <= 800.0 && y + h <= 400.0,
        "{:?}",
        d.rect
    );
}

/// Um clique sem arrastar escolhe a célula por baixo — fora da área, e DENTRO dela (a Region nova é a imagem
/// inteira: tudo é «dentro», e o clique tem de escolher na mesma — a foto do 1.º smoke mostrou-o perdido).
/// ⭐ Dono 27/10, smoke da Region: «é preciso ter a opção sem snap». O botão Snap desligado: a área anda ao píxel
/// e um clique não escolhe célula; o `Alt` inverte-o (encaixa enquanto premido); ligado de novo, encaixa.
#[test]
fn o_botao_snap_desliga_o_encaixe() {
    let mut d = aba();
    d.rect = [0.0, 0.0, 100.0, 100.0];
    d.layout.snap = Rect::new(0.0, -50.0, 40.0, 20.0);
    d.press(10.0, -40.0);
    assert!(!d.snap_on, "o botão desliga");
    d.press(437.3, 163.6);
    d.motion(612.4, 288.8);
    d.release(612.4, 288.8);
    assert_eq!(d.rect, [437.0, 164.0, 175.0, 125.0], "sem snap: ao píxel");
    d.press(350.0, 50.0);
    d.release(350.0, 50.0);
    assert_eq!(
        d.rect,
        [437.0, 164.0, 175.0, 125.0],
        "sem snap um clique não escolhe célula"
    );
    d.set_keys(true, None);
    d.press(130.0, 30.0);
    d.motion(260.0, 140.0);
    d.release(260.0, 140.0);
    assert_eq!(
        d.rect,
        [100.0, 0.0, 200.0, 100.0],
        "Alt com o Snap desligado: encaixa"
    );
    d.set_keys(false, None);
    d.press(10.0, -40.0);
    assert!(d.snap_on, "o botão volta a ligar");
}

#[test]
fn um_clique_escolhe_a_celula() {
    let mut d = aba();
    d.rect = [0.0, 0.0, 100.0, 100.0];
    d.press(350.0, 250.0);
    d.release(350.0, 250.0);
    assert_eq!(d.rect, [300.0, 200.0, 100.0, 100.0]);
    d.rect = [0.0, 0.0, 800.0, 400.0];
    d.press(150.0, 50.0);
    d.release(150.0, 50.0);
    assert_eq!(d.rect, [100.0, 0.0, 100.0, 100.0], "dentro da área inteira");
}

#[test]
fn ok_entrega_a_area_e_a_grelha_cancel_nada() {
    let mut d = aba();
    d.rect = [100.0, 0.0, 100.0, 100.0];
    d.editing = Some((super::doc::Field::Cols, "4".into()));
    d.ok();
    assert_eq!(
        d.outcome,
        Some(Outcome::Ok {
            rect: [100, 0, 100, 100],
            grid: [4, 4]
        }),
        "o campo aberto entra no OK"
    );
    let mut d = aba();
    d.layout.cancel = Rect::new(0.0, -50.0, 40.0, 20.0);
    d.press(10.0, -40.0);
    assert_eq!(d.outcome, Some(Outcome::Cancel));
}

#[test]
fn a_grelha_tem_no_maximo_um_pixel_por_celula() {
    let mut d = aba();
    d.editing = Some((super::doc::Field::Rows, "99999".into()));
    d.close_field(true);
    assert_eq!(d.grid, [8, 400]);
    d.editing = Some((super::doc::Field::Cols, "0".into()));
    d.close_field(true);
    assert_eq!(d.grid, [1, 400]);
}
