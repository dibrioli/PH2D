//! ⭐⭐⭐ **A TAMPA DE FORA DA JUNTA FICA REDONDA** (report do dono de 2026-10-06 na `=6` a `170`:
//! *«a parte externa da junta … sem permanecer arredondada»*; ordem «Arredondar»).
//!
//! A aresta da CÓPIA do *Repeater* passa a `0,019` da junta 1 e a `0,128` da junta 2; a pele roda
//! cada ponto em torno da junta, logo a tampa dessa aresta tem raio `≈ d` quando a volta se
//! concentra — e a média em CÍRCULO concentra-a (`2·tan(g/2)` por unidade de peso em `w = ½`). O
//! meio-ângulo (`4·tan(g/4)`) espalha-a pela zona dos pesos. Régua: o [`raio_da_tampa`] (o arco
//! onde a curva faz os 80 % do meio da viragem, sobre essa viragem).

use crate::sonda_tampa_redonda_tests::{JUNTAS, Lado, P, cena, raio_da_tampa};
use ph2d_skeleton::MisturaDoAngulo;

/// `(raio da tampa da cópia, da barra)` na junta `ji` da barra 0, pela lei `lei`.
fn tampas(l: &Lado, ji: usize, lei: MisturaDoAngulo) -> (f64, f64) {
    let (xj, _, yf, _) = JUNTAS[ji];
    let mut f = l.prep.guardado.path.clone();
    f.subpaths.retain(|c| c.closed);
    let pls = crate::smoke_bone_copias::sondas::polilinhas(&f, 512);
    let mut copia: Vec<P> = pls[1]
        .iter()
        .filter(|p| (p[0] - xj).abs() <= 0.6 && (p[1] + 0.6).abs() < 0.3)
        .copied()
        .collect();
    copia.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let barra: Vec<P> = (0..=600)
        .map(|i| [xj - 0.6 + 0.002 * f64::from(i), yf])
        .collect();
    let posa = |aresta: &[P]| -> Vec<P> {
        aresta
            .iter()
            .map(|p| l.pele.blend_com(*p, &l.pesos(*p), lei))
            .collect()
    };
    (
        raio_da_tampa(&posa(&copia)).0,
        raio_da_tampa(&posa(&barra)).0,
    )
}

/// ⭐⭐⭐ **GATE — a `=6` a `170/170` e `170/140`: a tampa da cópia tem pelo menos o raio medido
/// menos `0,02` em cada junta, e a da barra pelo menos a meia-largura (`0,375`).** Régua sobre a
/// pele EXACTA (cada ponto posto pela lei), não sobre o desenho assado.
///
/// | pose · junta | cópia: círculo → meio-ângulo | piso | barra: círculo → meio-ângulo |
/// |---|---:|---:|---:|
/// | `170/170` · 1 (`d 0,019`) | `0,028` → `0,166` | `0,146` | `0,396` → `0,519` |
/// | `170/170` · 2 (`d 0,128`) | `0,166` → `0,307` | `0,287` | `0,404` → `0,529` |
/// | `170/140` · 1 | `0,028` → `0,165` | `0,145` | `0,397` → `0,520` |
/// | `170/140` · 2 | `0,243` → `0,347` | `0,327` | `0,495` → `0,579` |
///
/// ⛔ O CONTROLO: a média em círculo, na mesma fixtura, fica ABAIXO do piso da cópia em todas —
/// senão o gate não mede o bico que o dono fotografou.
#[test]
fn a_tampa_de_fora_da_junta_fica_redonda() {
    const PISO: [[f64; 2]; 2] = [[0.146, 0.287], [0.145, 0.327]];
    for (k, (g1, g2)) in [(170.0_f32, 170.0_f32), (170.0, 140.0)]
        .into_iter()
        .enumerate()
    {
        let (sim, st, ids) = cena(g1, g2);
        let l = Lado::de(&sim, &st, ids[0]);
        assert_eq!(l.pele.mistura(), MisturaDoAngulo::MeioAngulo);
        for (ji, piso) in PISO[k].iter().enumerate() {
            let (c_meio, b_meio) = tampas(&l, ji, MisturaDoAngulo::MeioAngulo);
            let (c_circ, _) = tampas(&l, ji, MisturaDoAngulo::Circulo);
            let j = ji + 1;
            assert!(
                c_meio >= *piso,
                "{g1}/{g2} junta {j}: a tampa da cópia tem raio {c_meio:.3} (< {piso}) — o bico voltou"
            );
            assert!(
                b_meio >= 0.375,
                "{g1}/{g2} junta {j}: a tampa da barra tem raio {b_meio:.3}, menos que a meia-largura"
            );
            assert!(
                c_circ < *piso,
                "{g1}/{g2} junta {j}: o CONTROLO (círculo) dá {c_circ:.3} — a fixtura perdeu o bico"
            );
        }
    }
}
