use super::*;

const MAPA: Mapa = Mapa {
    x: 100.0,
    w: 400.0,
    total_s: 8.0,
};

fn arrasto(part: u8) -> Arrasto {
    // 2 s visíveis em 800 px de coluna, de 2 a 4 s; agarrado em 3 s.
    Arrasto {
        part,
        mapa: MAPA,
        inicio: 2.0,
        fim: 4.0,
        largura: 800.0,
        pega: 3.0,
    }
}

#[test]
fn o_mapa_vai_e_volta() {
    assert_eq!(MAPA.x(MAPA.t(250.0)), 250.0);
    assert_eq!(MAPA.t(MAPA.x + MAPA.w), MAPA.total_s);
}

/// *Mutação: sem o mínimo ⇒ um polegar de 0 px, sem pontas nem meio para agarrar.*
#[test]
fn o_polegar_nunca_fica_mais_estreito_que_o_minimo_nem_sai_da_calha() {
    let (a, b) = polegar(MAPA, 0.0, 0.001);
    assert!((b - a - POLEGAR_MIN).abs() < 1e-3, "{a}..{b}");
    assert_eq!(a, MAPA.x, "encostado ao começo, não antes dele");
    let (a, b) = polegar(MAPA, 8.0 - 0.001, 0.001);
    assert!((b - (MAPA.x + MAPA.w)).abs() < 1e-3 && b - a >= POLEGAR_MIN - 1e-3);
    let [e, d] = alvos_das_pontas(Rect::new(a, 0.0, b - a, 10.0));
    assert!(e.x + e.w < d.x, "as pontas não se tocam: sobra o meio");
}

/// A ponta direita mexe o fim e deixa o início; a esquerda o contrário.
/// *Mutação: trocar `PONTA_E` por `PONTA_D` no `match` ⇒ reprova.*
#[test]
fn cada_ponta_deixa_a_outra_parada() {
    let (s, px) = arrasta(arrasto(PONTA_D), 2.0); // o fim recua 1 s: 1 s visível
    assert_eq!(s, 2.0);
    assert!((px - 800.0).abs() < 1e-9, "{px}");
    let (s, px) = arrasta(arrasto(PONTA_E), 4.0); // o início avança 1 s: 3..4
    assert!(
        (s - 3.0).abs() < 1e-9 && (px - 800.0).abs() < 1e-9,
        "{s} {px}"
    );
    assert!((s + 800.0 / px - 4.0).abs() < 1e-9, "o fim mexeu");
}

/// As pontas não se cruzam, o zoom pára nos limites e o tempo não fica negativo.
#[test]
fn as_pontas_param_nos_limites() {
    let (s, px) = arrasta(arrasto(PONTA_E), 1000.0);
    assert!(
        (px - MAX_PX_PER_S).abs() < 1e-6,
        "passar a outra ponta pára no zoom máximo: {px}"
    );
    assert!(s < 4.0);
    let (s, px) = arrasta(arrasto(PONTA_E), -1000.0);
    assert!(s >= 0.0 && px >= MIN_PX_PER_S);
    let (s, px) = arrasta(arrasto(PONTA_D), -1000.0);
    assert!(s == 2.0 && (px - MAX_PX_PER_S).abs() < 1e-6, "{s} {px}");
}

/// O meio rola sem mudar o zoom e nunca antes do zero.
#[test]
fn o_meio_rola_e_para_no_zero() {
    let (s, px) = arrasta(arrasto(MEIO), 3.5);
    assert_eq!((s, px), (2.5, 400.0));
    assert_eq!(arrasta(arrasto(MEIO), -10.0).0, 0.0);
}
