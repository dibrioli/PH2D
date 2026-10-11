//! **A meia inclinação e a lei da gota** (W1e, F1+F5; W1f — a bancada do doc 06 §2,
//! [`super::sonda_gota::bancada`]).
//!
//! O Rebelle a meia inclinação: nada escorre e a orla de BAIXO escurece (1,6 na água 100). A lei da gota
//! da W1e dava a orla escura (1,32) mas cortava o escorrimento em rectângulos; a W1f tirou-a do produto
//! (05 §4b). ⚠️ O controlo está DENTRO de cada gate.
use super::sonda_gota::bancada;
use ph2d_wet_paint::solver::ALTURA_DA_GOTA;

const W: usize = 1000;
const H: usize = 640;

fn escuro(px: &[u8], x: usize, y: usize) -> f64 {
    let i = (y * W + x) * 4;
    f64::from(255 - px[i..i + 3].iter().copied().min().unwrap_or(255))
}

/// Do traço `i`: (orla de baixo ÷ de cima, a tinta mais baixa abaixo da orla, px).
pub(super) fn le(px: &[u8], i: usize) -> (f64, usize) {
    let x0 = 60 + 240 * i;
    let miolo = x0 + 25..x0 + 125;
    // O limiar é 30 % da tinta do próprio traço (mínimo 6; até à tinta do Rebelle eram 25 fixos): com o
    // pigmento baixo as linhas da borda ficavam abaixo dos 25 e a orla lia-se por dentro do traço.
    let mut centro: Vec<f64> = miolo.clone().map(|x| escuro(px, x, 150)).collect();
    centro.sort_by(f64::total_cmp);
    let limiar = (0.3 * centro[centro.len() / 2]).max(6.0);
    let cheia = |y: usize| miolo.clone().filter(|&x| escuro(px, x, y) > limiar).count() * 2 >= 100;
    let (mut yb, mut yt) = (150, 150);
    while yb + 1 < H && cheia(yb + 1) {
        yb += 1;
    }
    while yt > 0 && cheia(yt - 1) {
        yt -= 1;
    }
    let media = |ys: std::ops::Range<usize>| {
        let n = (ys.len() * miolo.len()) as f64;
        ys.flat_map(|y| miolo.clone().map(move |x| (x, y)))
            .map(|(x, y)| escuro(px, x, y))
            .sum::<f64>()
            / n
    };
    let orla = media(yb - 2..yb + 1) / media(yt..yt + 3).max(1e-9);
    let desce = (yb + 1..H)
        .filter(|&y| (x0 - 25..x0 + 175).any(|x| escuro(px, x, y) > limiar))
        .map(|y| y - yb)
        .max()
        .unwrap_or(0);
    (orla, desce)
}

/// O PRODUTO a meia inclinação: nada escorre (≤ 20 px abaixo da orla em todas as águas; Rebelle 0).
///
/// ⚠️ **A W1f tirou a lei da gota do produto** (ela cortava o escorrimento em rectângulos —
/// `achatamento_tests`), e com ela foi a cláusula «a orla de baixo escurece» (com a lei 1,32; sem ela
/// 0,77; Rebelle 1,6 — 05 §4b, o aberto da W1f). O controlo: o quádruplo da gravidade no máximo escorre
/// (o dobro escorria 19 px com o pigmento do Rebelle, 10/10: a tinta clara desce menos que o limiar).
#[test]
fn a_meia_inclinacao_nao_escorre() {
    let aguas = [1.0, 0.7, 0.4, 0.2];
    let desce = |px: &[u8]| (0..4).map(|i| le(px, i).1).max().unwrap_or(0);
    let produto = bancada(0.005, 0.0, &[], Some(4), aguas);
    let controlo = bancada(0.02, 0.0, &[], Some(8), aguas);
    let l: Vec<(f64, usize)> = (0..4).map(|i| le(&produto, i)).collect();
    eprintln!("[gota] produto a meia: (orla, desce) por traço {l:?}");
    let lc: Vec<(f64, usize)> = (0..4).map(|i| le(&controlo, i)).collect();
    eprintln!("[gota] controlo: (orla, desce) por traço {lc:?}");
    assert!(
        desce(&controlo) > 20,
        "CONTROLO: o quádruplo da gravidade no máximo não escorreu — a régua não discrimina"
    );
    assert!(
        desce(&produto) <= 20,
        "a meia inclinação do produto escorre {} px abaixo da orla (Rebelle 0)",
        desce(&produto)
    );
}

/// Com a tela DIREITA a lei não existe (`|g| = 0`): o traço é o de antes, ao byte.
#[test]
fn com_a_tela_direita_a_lei_da_gota_nao_muda_um_byte() {
    let aguas = [1.0, 0.7, 0.4, 0.2];
    assert_eq!(
        bancada(0.005, ALTURA_DA_GOTA, &[], None, aguas),
        bancada(0.005, 0.0, &[], None, aguas),
        "a lei da gota mudou o traço com a tela direita"
    );
}
