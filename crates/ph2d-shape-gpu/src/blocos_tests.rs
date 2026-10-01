use super::*;

/// **Completar um trecho acrescenta segmentos de comprimento ZERO no último ponto** — eles somam
/// `0` (`dy = 0`) e mantêm a corrente ligada; um trecho vazio fica vazio.
#[test]
fn completar_poe_zeros_no_ultimo_ponto_e_deixa_o_vazio_vazio() {
    let mut vazio: Vec<[f32; 4]> = Vec::new();
    completa(&mut vazio);
    assert!(vazio.is_empty(), "um trecho vazio nao ganha segmentos");
    let mut v = vec![
        [0.0, 0.0, 1.0, 0.0],
        [1.0, 0.0, 1.0, 1.0],
        [1.0, 1.0, 0.5, 2.0],
    ];
    completa(&mut v);
    assert_eq!(v.len(), SEGS_POR_BLOCO);
    for s in &v[3..] {
        assert_eq!(
            *s,
            [0.5, 2.0, 0.5, 2.0],
            "o enchimento e' o ultimo ponto, parado"
        );
    }
    let mut cheio = vec![[0.0; 4]; SEGS_POR_BLOCO];
    completa(&mut cheio);
    assert_eq!(cheio.len(), SEGS_POR_BLOCO, "um bloco cheio nao cresce");
}

/// **A caixa de cada bloco cobre os segmentos dele, e «encadeado» quer dizer AO BIT** — a corrente
/// que parte (o anel: dois sub-caminhos) tem de ler `0`, senão o shader soma-a pelas pontas e
/// erra (a mutação mede alfa `245` no anel do gate de paridade).
#[test]
fn a_caixa_cobre_o_bloco_e_a_corrente_partida_le_zero() {
    let mut segs: Vec<[f32; 4]> = (0..8)
        .map(|k| {
            #[expect(clippy::cast_precision_loss, reason = "oito pontos")]
            let x = k as f32;
            [x, x * 0.5, x + 1.0, (x + 1.0) * 0.5]
        })
        .collect();
    // um bloco partido: a corrente recomeça noutro ponto a meio
    segs.extend((0..8).map(|k| {
        #[expect(clippy::cast_precision_loss, reason = "oito pontos")]
        let x = k as f32;
        if k == 4 {
            [10.0, -3.0, 11.0, -2.0]
        } else {
            [x, -x, x + 1.0, -(x + 1.0)]
        }
    }));
    let bl = blocos_de(&segs);
    assert_eq!(bl.len(), 2);
    assert_eq!(
        bl[0].encadeado, 1,
        "oito segmentos seguidos sao uma corrente"
    );
    assert_eq!(bl[1].encadeado, 0, "a corrente partida nao e' uma corrente");
    for (b, chunk) in bl.iter().zip(segs.chunks(SEGS_POR_BLOCO)) {
        for s in chunk {
            for (x, y) in [(s[0], s[1]), (s[2], s[3])] {
                assert!(
                    b.caixa[0] <= x && x <= b.caixa[2] && b.caixa[1] <= y && y <= b.caixa[3],
                    "a caixa {:?} nao cobre ({x}, {y})",
                    b.caixa
                );
            }
        }
    }
    assert_eq!(bl[1].caixa, [0.0, -8.0, 11.0, 0.0], "a caixa e' a justa");
}
