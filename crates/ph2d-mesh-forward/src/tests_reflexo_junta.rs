//! ⭐⭐⭐ **A JUNTA** — o 3.º report do dono (04/10): o reflexo de duas vizinhas SEPARADAS colado por uma
//! ponte, «como se o reflexo fizesse operação booleana» (a caixa azul longe, meio atrás da verde perto vista
//! do centro do cromo). As réguas de média e de grosseiros não a viam (o contorno de todo reflexo domina):
//! esta mede só a FAIXA ENTRE os dois reflexos — os px cujo raio, pela geometria, não acerta nenhuma
//! vizinha e que têm, na mesma linha, uma vizinha DIFERENTE de cada lado. Ali o Cycles mostra o fundo.

use crate::tests_chao_tapa::metal;
use crate::tests_contacto::linear;
use crate::tests_reflexo_perto::{
    JUNTA, LADO, Px, Vista, desenha, desenha_ate, desenhista, oraculo, vizinha_refletida,
};

/// Até quantos px, na mesma linha, se procura a vizinha de cada lado.
const ALCANCE: i32 = 96;

/// A faixa (índices em `px`): o raio não acerta nada, a mais de `2 px` de quem acerta e da silhueta, com
/// uma vizinha diferente de cada lado na mesma linha.
pub(crate) fn faixa(v: &Vista, px: &[Px]) -> Vec<usize> {
    let l = LADO as i32;
    let mut mapa: Vec<Option<Option<usize>>> = vec![None; (l * l) as usize];
    for p in px {
        mapa[(p.j * LADO + p.i) as usize] = Some(vizinha_refletida(v, p));
    }
    let at = |x: i32, y: i32| {
        ((0..l).contains(&x) && (0..l).contains(&y))
            .then(|| mapa[(y * l + x) as usize])
            .flatten()
    };
    px.iter()
        .enumerate()
        .filter(|(_, p)| {
            let (i, j) = (p.i as i32, p.j as i32);
            let limpo = (-2..=2).all(|dy| (-2..=2).all(|dx| at(i + dx, j + dy) == Some(None)));
            // A 1.ª vizinha de um lado (pára na silhueta do cromo).
            let lado = |s: i32| {
                (1..=ALCANCE)
                    .find_map(|k| match at(i + s * k, j) {
                        Some(None) => None,
                        o => Some(o.flatten()),
                    })
                    .flatten()
            };
            limpo && matches!((lado(-1), lado(1)), (Some(a), Some(b)) if a != b)
        })
        .map(|(k, _)| k)
        .collect()
}

/// `(px, |Δ| médio, px com |Δ| > 0,2)` da razão `viz/solo` contra a do Cycles nos px `quais`.
fn mede(
    px: &[Px],
    quais: &[usize],
    (viz, solo): (&[u8], &[u8]),
    (cv, cs): (usize, usize),
) -> (usize, f32, usize) {
    let (mut n, mut s, mut g) = (0usize, 0.0f32, 0usize);
    for &k in quais {
        let p = &px[k];
        let q = (p.j * LADO + p.i) as usize;
        let (lv, ls) = (linear(viz[q * 4 + 1]), linear(solo[q * 4 + 1]));
        if ls < 0.05 || p.col[cs] < 0.05 {
            continue;
        }
        let e = (lv / ls - p.col[cv] / p.col[cs]).abs();
        n += 1;
        s += e;
        g += usize::from(e > 0.2);
    }
    (n, s / n.max(1) as f32, g)
}

/// Com `PH2D_REFLEXO_FOTOS=<pasta>`: `nossa | Cycles | a faixa` (a razão `viz/solo`; a faixa a cinzento).
fn foto(px: &[Px], quais: &[usize], (viz, solo): (&[u8], &[u8]), (cv, cs): (usize, usize), nome: &str) {
    let Ok(pasta) = std::env::var("PH2D_REFLEXO_FOTOS") else {
        return;
    };
    let l = LADO as usize;
    let mut img = vec![0u8; 3 * l * l];
    let b = |x: f32| (x.clamp(0.0, 1.0) * 255.0) as u8;
    for p in px {
        let q = p.j as usize * l + p.i as usize;
        let ls = linear(solo[q * 4 + 1]).max(1.0e-3);
        img[p.j as usize * 3 * l + p.i as usize] = b(linear(viz[q * 4 + 1]) / ls);
        img[p.j as usize * 3 * l + l + p.i as usize] = b(p.col[cv] / p.col[cs].max(1.0e-3));
    }
    for &k in quais {
        img[px[k].j as usize * 3 * l + 2 * l + px[k].i as usize] = 128;
    }
    let mut f = format!("P5 {} {l} 255\n", 3 * l).into_bytes();
    f.extend_from_slice(&img);
    let _ = std::fs::write(format!("{pasta}/{nome}.pgm"), f);
}

/// ⭐⭐⭐ **Entre dois reflexos de vizinhas separadas, o cromo mostra o fundo, como no Cycles** — nítido e
/// a `0,05` (o cromo da cena 42). CONTROLO: sem a azul, a faixa é fundo também no nosso, e a régua passa.
#[test]
#[ignore = "precisa de aparelho"]
fn entre_dois_reflexos_o_cromo_mostra_o_fundo() {
    let v = &JUNTA;
    let Some(mut fw) = desenhista(v) else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let px = oraculo(v);
    let f = faixa(v, &px);
    let mut falhas = Vec::new();
    for (rug, cols) in [(0.0f32, (0usize, 2usize)), (v.rug2, (1, 3))] {
        let viz = desenha(v, &mut fw, metal(rug), true);
        let solo = desenha(v, &mut fw, metal(rug), false);
        let sem_azul = desenha_ate(v, &mut fw, metal(rug), 2, true);
        let m = mede(&px, &f, (&viz, &solo), cols);
        let c = mede(&px, &f, (&sem_azul, &solo), cols);
        foto(&px, &f, (&viz, &solo), cols, &format!("junta_{rug}"));
        foto(&px, &f, (&sem_azul, &solo), cols, &format!("junta_sem_azul_{rug}"));
        eprintln!(
            "junta, cromo {rug}: faixa {} px · |Δ| médio {:.4} · |Δ| > 0,2: {} — CONTROLO sem a azul: {:.4} · {}",
            m.0, m.1, m.2, c.1, c.2
        );
        assert!(m.0 > 300, "a faixa encolheu: {} px", m.0);
        assert!(
            c.1 < BARRA.0 && c.2 <= BARRA.1,
            "CONTROLO — sem a azul a faixa é fundo: a régua tinha de passar"
        );
        if !(m.1 < BARRA.0 && m.2 <= BARRA.1) {
            falhas.push(rug);
        }
    }
    assert!(
        falhas.is_empty(),
        "entre os dois reflexos há uma ponte que o Cycles não tem: {falhas:?}"
    );
}

/// `(|Δ| médio na faixa, px com |Δ| > 0,2)`.
const BARRA: (f32, usize) = (0.03, 0);
