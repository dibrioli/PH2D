//! **A lei da gota no produto** (W1e, F1+F5 — a bancada do doc 06 §2, [`super::sonda_gota::bancada`]).
//!
//! O Rebelle a meia inclinação: nada escorre e a orla de BAIXO escurece (1,6 na água 100). O port, com a
//! gravidade de fábrica, deixa a orla de baixo MAIS CLARA que a de cima (0,76) — a água rasteja para o
//! seco em vez de assentar — e com mais gravidade abre o pente. ⚠️ O controlo está DENTRO: o port na
//! mesma bancada tem de reprovar.
use super::sonda_gota::bancada;
use ph2d_wet_paint::solver::ALTURA_DA_GOTA;

const W: usize = 1000;
const H: usize = 640;

fn escuro(px: &[u8], x: usize, y: usize) -> f64 {
    let i = (y * W + x) * 4;
    f64::from(255 - px[i..i + 3].iter().copied().min().unwrap_or(255))
}

/// Do traço `i`: (orla de baixo ÷ de cima, a tinta mais baixa abaixo da orla, px).
fn le(px: &[u8], i: usize) -> (f64, usize) {
    let x0 = 60 + 240 * i;
    let miolo = x0 + 25..x0 + 125;
    let cheia = |y: usize| miolo.clone().filter(|&x| escuro(px, x, y) > 25.0).count() * 2 >= 100;
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
        .filter(|&y| (x0 - 25..x0 + 175).any(|x| escuro(px, x, y) > 25.0))
        .map(|y| y - yb)
        .max()
        .unwrap_or(0);
    (orla, desce)
}

#[test]
fn a_meia_inclinacao_nao_escorre_e_a_orla_de_baixo_escurece() {
    const G: f64 = 0.005;
    let aguas = [1.0, 0.7, 0.4, 0.2];
    let produto = bancada(G, ALTURA_DA_GOTA, &[], Some(4), aguas);
    let port = bancada(G, 0.0, &[], Some(4), aguas);
    let barra = |px: &[u8]| -> Result<(), String> {
        let (orla, _) = le(px, 0);
        let mut f = Vec::new();
        if orla <= 1.0 {
            f.push(format!("água 1: orla de baixo ÷ de cima {orla:.2} ≤ 1 (Rebelle 1,6)"));
        }
        for (i, agua) in aguas.iter().enumerate() {
            let (_, d) = le(px, i);
            if d > 20 {
                f.push(format!("água {agua}: tinta {d} px abaixo da orla"));
            }
        }
        if f.is_empty() { Ok(()) } else { Err(f.join(" · ")) }
    };
    for (nome, px) in [("port", &port), ("produto", &produto)] {
        let l: Vec<(f64, usize)> = (0..4).map(|i| le(px, i)).collect();
        eprintln!("[gota] {nome}: (orla, desce) por traço {l:?}");
    }
    assert!(
        barra(&port).is_err(),
        "CONTROLO: o port passou a barra da meia inclinação — a régua não discrimina"
    );
    if let Err(e) = barra(&produto) {
        panic!("a meia inclinação do produto: {e}");
    }
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
