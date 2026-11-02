//! O gate do editor de curva: o hit-test do ponto, o premir que acrescenta, o arrasto que nunca
//! cruza os vizinhos, e as pontas fixas.

use super::*;

/// Uma caixa de 255 px: um px por valor (as contas lêem-se de cabeça).
const CAIXA: Rect = Rect {
    x: 10.0,
    y: 20.0,
    w: 255.0,
    h: 255.0,
};

fn ecra(q: (u8, u8)) -> [f32; 2] {
    curve_to_screen(CAIXA, q)
}

#[test]
fn ida_e_volta_do_valor_ao_ecra_e_o_y_cresce_para_cima() {
    for q in [(0, 0), (64, 200), (255, 255), (128, 0)] {
        assert_eq!(curve_from_screen(CAIXA, ecra(q)), q);
    }
    assert!(ecra((0, 255))[1] < ecra((0, 0))[1], "o y alto fica em cima");
    assert_eq!(curve_from_screen(CAIXA, [-50.0, 900.0]), (0, 0), "preso");
}

#[test]
fn o_hit_test_agarra_o_ponto_mais_perto_dentro_do_raio() {
    let pts = [(64, 64), (192, 128)];
    assert_eq!(curve_point_at(CAIXA, &pts, ecra((65, 63))), Some(0));
    assert_eq!(curve_point_at(CAIXA, &pts, ecra((190, 130))), Some(1));
    let longe = curve_grab_radius() + 2.0;
    let [x, y] = ecra((64, 64));
    assert_eq!(curve_point_at(CAIXA, &pts, [x + longe, y]), None);
}

#[test]
fn premir_numa_zona_vazia_acrescenta_um_ponto_na_ordem_do_x() {
    let mut pts = vec![(64, 64), (192, 192)];
    assert_eq!(curve_press(CAIXA, &mut pts, ecra((128, 30)), 8), Some(1));
    assert_eq!(pts, vec![(64, 64), (128, 30), (192, 192)]);
    // Sobre um ponto agarra-o, não acrescenta.
    assert_eq!(curve_press(CAIXA, &mut pts, ecra((192, 191)), 8), Some(2));
    assert_eq!(pts.len(), 3);
    // Cheio: nada entra.
    assert_eq!(curve_press(CAIXA, &mut pts, ecra((20, 200)), 3), None);
    assert_eq!(pts.len(), 3);
}

#[test]
fn as_pontas_sao_fixas_nao_se_acrescentam_nem_se_tiram() {
    let mut pts = Vec::new();
    assert_eq!(curve_press(CAIXA, &mut pts, ecra((0, 0)), 8), None);
    assert_eq!(curve_press(CAIXA, &mut pts, ecra((255, 255)), 8), None);
    assert!(pts.is_empty());
    assert!(!curve_remove_at(CAIXA, &mut pts, ecra((0, 0))));
}

#[test]
fn o_arrasto_nunca_cruza_os_vizinhos_e_prende_o_y() {
    let mut pts = vec![(64, 64), (128, 128), (192, 192)];
    curve_drag(CAIXA, &mut pts, 1, ecra((250, 10)));
    assert_eq!(pts[1], (191, 10), "encosta ao vizinho da direita");
    curve_drag(CAIXA, &mut pts, 1, ecra((3, 255)));
    assert_eq!(pts[1], (65, 255), "encosta ao da esquerda");
    curve_drag(CAIXA, &mut pts, 0, [-100.0, 9999.0]);
    assert_eq!(pts[0], (1, 0), "nunca chega à ponta (0,0)");
    curve_drag(CAIXA, &mut pts, 2, [9999.0, -100.0]);
    assert_eq!(pts[2], (254, 255), "nem à ponta (255,255)");
    assert!(pts.windows(2).all(|w| w[0].0 < w[1].0));
}

#[test]
fn duplo_clique_sobre_um_ponto_tira_o() {
    let mut pts = vec![(64, 64), (192, 128)];
    assert!(curve_remove_at(CAIXA, &mut pts, ecra((64, 64))));
    assert_eq!(pts, vec![(192, 128)]);
    assert!(!curve_remove_at(CAIXA, &mut pts, ecra((64, 64))), "vazio");
}

#[test]
fn a_capacidade_e_um_ponto_por_diametro_de_agarrar() {
    let n = curve_capacity(CAIXA);
    assert!(n >= 2);
    assert!(f32::from(u16::try_from(n).unwrap()) * curve_grab_radius() * 2.0 <= CAIXA.w);
    assert_eq!(curve_capacity(Rect::new(0.0, 0.0, 0.0, 9.0)), 0);
}
