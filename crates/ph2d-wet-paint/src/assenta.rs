//! **Assentar não muda o que se vê** (dono 10/10: «pintando sobre tinta seca o traço muda de aparência no
//! mouse up — isso não pode acontecer»). O pigmento que desce da água para o papel (assentar, Dry canvas) ou
//! sobe do papel para a água (re-molhar) muda de camada NO MESMO PONTO: a coluna de tinta é a mesma e a cor
//! que se vê também. As duas transferências verticais resolvem a cor nova pela INVERSA da lei do ecrã
//! ([`cor`], a mesma porta do render); a mistura de pigmentos fica onde acontece de facto — na água, quando
//! chega tinta (o pouso do dab) ou quando ela corre (a advecção).
//!
//! Medido antes (prompt 15b, `sonda_duas_etapas`, amarelo sobre azul seco, o miolo com o rato em baixo e
//! depois de seco): fábrica Δ29 · «Pigment mixing» Δ81 · «Glaze layering» Δ28 · as duas Δ126 níveis.
//! O port (`Lei::Porto`) guarda as junções de antes — a impressão digital não muda.
use crate::colorops::transfer::{srgb255_of_linear, srgb255_to_linear};
use crate::colorops::{km_glaze_channel_linear, velatura};

/// A lei do ecrã para o filme molhado sobre o assentado, que as transferências verticais preservam.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Lei {
    /// O port: as junções de antes (mistura + velatura), sem preservar o ecrã.
    #[default]
    Porto,
    /// O filme por cima do assentado (`lerp` pela cobertura) com a velatura do filme — o render de fábrica.
    PorCima,
    /// «Glaze layering (K-M)»: o filme como camada de reflectância sobre o assentado.
    Camada,
}

/// A lei com o que ela lê dos params: a velatura do filme no [`Lei::PorCima`].
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Ecra {
    #[default]
    Porto,
    PorCima(f64),
    Camada,
}

/// A cor que o ecrã mostra de uma célula: `sc` assentado (cobertura `a_s`) sob `uc` em suspensão (`a_f`).
/// É a conta do render ([`crate::render::render_pigment_row_visual`]) — ⛔ uma porta só.
#[inline]
pub fn cor(e: Ecra, sc: [f64; 3], uc: [f64; 3], a_s: f64, a_f: f64) -> [f64; 3] {
    if a_s <= 0.0 {
        return uc;
    }
    if a_f <= 0.0 {
        return sc;
    }
    match e {
        Ecra::Camada => [0, 1, 2].map(|c| {
            srgb255_of_linear(km_glaze_channel_linear(
                srgb255_to_linear(sc[c]),
                srgb255_to_linear(uc[c]),
                a_f,
            ))
        }),
        Ecra::PorCima(_) | Ecra::Porto => {
            let filme = if let Ecra::PorCima(f) = e { f } else { 0.0 };
            let mut c = [0, 1, 2].map(|k| sc[k] * (1.0 - a_f) + uc[k] * a_f);
            let la = a_s + a_f * (1.0 - a_s);
            velatura(&mut c, a_f / la, filme, a_s, sc, uc);
            c
        }
    }
}

/// Newton por canal sobre a cor que muda (`x`), a partir do palpite físico, até o ecrã voltar a `alvo`
/// (a velatura do filme não compõe em passos: o palpite deixa um resto). A derivada é numérica — com o
/// filme grosso a velatura pesa mais que a cobertura (medido: assumir `1 − a` dava passos 4× grandes e o
/// miolo ia a preto); o passo tem tecto.
fn resolve(mut x: [f64; 3], alvo: [f64; 3], ecra: impl Fn([f64; 3]) -> [f64; 3]) -> [f64; 3] {
    const H: f64 = 0.5;
    for _ in 0..4 {
        let y = ecra(x);
        if (0..3).all(|c| (y[c] - alvo[c]).abs() < 0.01) {
            break;
        }
        for c in 0..3 {
            let mut xh = x;
            xh[c] = if x[c] + H <= 255.0 {
                x[c] + H
            } else {
                x[c] - H
            };
            let d = (ecra(xh)[c] - y[c]) / (xh[c] - x[c]);
            // Onde o ecrã quase não vê esta cor (o filme grosso esconde o assentado) o palpite físico
            // fica: corrigi-lo ali escrevia uma cor ESCONDIDA que aparecia quando o filme afinava
            // (medido: o miolo esverdeava +24 ao secar).
            if d.abs() < 0.1 {
                continue;
            }
            let passo = ((y[c] - alvo[c]) / d).clamp(-48.0, 48.0);
            x[c] = (x[c] - passo).clamp(0.0, 255.0);
        }
    }
    x
}

/// **ASSENTAR:** a cor nova do assentado quando a cobertura do assentado passa de `a_s` a `a_s2` e a do
/// filme de `a_f` a `a_f2` (o filme `uc` não muda de cor) — a que mantém [`cor`].
pub fn assenta(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
) -> [f64; 3] {
    let alvo = cor(e, sc, uc, a_s, a_f);
    if a_f2 <= 0.0 {
        return alvo;
    }
    match e {
        Ecra::Camada => [0, 1, 2].map(|c| {
            let r = srgb255_to_linear(alvo[c]);
            let rt = a_f2 * srgb255_to_linear(uc[c]);
            let k = (1.0 - a_f2) * (1.0 - a_f2);
            let den = k + rt * (r - rt);
            let s = if den > 1e-9 { (r - rt) / den } else { r };
            srgb255_of_linear(s.clamp(0.0, 1.0))
        }),
        _ => {
            // O palpite: a parte que assentou é uma camada de `uc` POR BAIXO do filme que ficou.
            let x0 = if a_s <= 0.0 {
                alvo
            } else {
                // `a_f2 = 1` (filme opaco: `alpha_of_mass` satura a 1 ao bit) esconde o assentado — nada a mover.
                let w = if a_f2 < 1.0 {
                    ((a_f - a_f2) / (1.0 - a_f2)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                [0, 1, 2].map(|c| sc[c] + (uc[c] - sc[c]) * w)
            };
            resolve(x0, alvo, |x| cor(e, x, uc, a_s2, a_f2))
        }
    }
}

/// **RE-MOLHAR:** a cor nova do filme quando ele recebe pigmento do assentado (`sc` não muda de cor; as
/// coberturas passam a `a_s2`, `a_f2`) — a que mantém [`cor`].
pub fn levanta(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
) -> [f64; 3] {
    let alvo = cor(e, sc, uc, a_s, a_f);
    if a_s2 <= 0.0 || a_f2 <= 0.0 {
        return alvo;
    }
    match e {
        Ecra::Camada => [0, 1, 2].map(|c| {
            let r = srgb255_to_linear(alvo[c]);
            let s = srgb255_to_linear(sc[c]);
            let k = (1.0 - a_f2) * (1.0 - a_f2);
            // r = x + k·s/(1 − x·s), x = a·u  ⇒  s·x² − (1 + r·s)·x + (r − k·s) = 0, a raiz pequena.
            let x = if s > 1e-9 {
                let b = 1.0 + r * s;
                let disc = (b * b - 4.0 * s * (r - k * s)).max(0.0);
                (b - disc.sqrt()) / (2.0 * s)
            } else {
                r
            };
            srgb255_of_linear((x / a_f2).clamp(0.0, 1.0))
        }),
        _ => {
            let x0 = if a_f <= 0.0 {
                sc
            } else {
                let w = ((a_f2 - a_f) / a_f2).clamp(0.0, 1.0);
                [0, 1, 2].map(|c| uc[c] + (sc[c] - uc[c]) * w)
            };
            resolve(x0, alvo, |x| cor(e, sc, x, a_s2, a_f2))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AZ: [f64; 3] = [30.0, 100.0, 220.0];
    const AM: [f64; 3] = [245.0, 205.0, 25.0];

    fn perto(a: [f64; 3], b: [f64; 3], tol: f64) -> bool {
        (0..3).all(|c| (a[c] - b[c]).abs() <= tol)
    }

    /// Assentar e re-molhar não mudam o ecrã, nas duas leis, de filme grosso a quase seco (≤ 1,5 níveis: onde
    /// o ecrã quase não vê a cor que muda — sensibilidade < 0,1 —, o palpite físico fica sem a correcção
    /// fina; medido 1,4 no filme a 10 %).
    #[test]
    fn as_transferencias_verticais_nao_mudam_o_ecra() {
        for e in [Ecra::PorCima(0.20), Ecra::PorCima(0.35), Ecra::Camada] {
            for (a_s, a_f, d) in [
                (0.9, 0.95, 0.05),
                (0.999, 0.99, 0.02),
                (0.6, 0.5, 0.2),
                (0.99, 0.3, 0.29),
                (0.2, 0.1, 0.05),
                (0.01, 0.9, 0.1),
                (0.95, 1.0, 0.0),
            ] {
                let antes = cor(e, AZ, AM, a_s, a_f);
                let a_s2 = 1.0 - (1.0 - a_s) * (1.0 - d);
                let a_f2 = 1.0 - (1.0 - a_f) / (1.0 - d);
                let s2 = assenta(e, AZ, AM, a_s, a_f, a_s2, a_f2);
                let depois = cor(e, s2, AM, a_s2, a_f2);
                assert!(
                    perto(antes, depois, 1.5),
                    "{e:?} assenta {a_s},{a_f}: {antes:?} → {depois:?}"
                );
                let u2 = levanta(e, AZ, AM, a_s2, a_f2, a_s, a_f);
                let ida = cor(e, AZ, AM, a_s2, a_f2);
                let volta = cor(e, AZ, u2, a_s, a_f);
                assert!(
                    perto(ida, volta, 1.5),
                    "{e:?} levanta {a_s},{a_f}: {ida:?} → {volta:?}"
                );
            }
        }
    }

    /// A MESMA tinta por cima de si mesma fica a mesma cor (o traço de uma cor não muda ao secar).
    #[test]
    fn a_mesma_tinta_nao_muda() {
        let e = Ecra::PorCima(0.20);
        let s2 = assenta(e, AM, AM, 0.9, 0.8, 0.95, 0.6);
        assert!(perto(s2, AM, 0.01), "{s2:?}");
        assert_eq!(assenta(e, [0.0; 3], AM, 0.0, 0.8, 0.3, 0.7), AM);
    }

    /// Tudo assentado = a cor que se via.
    #[test]
    fn secar_de_vez_fica_com_a_cor_que_se_via() {
        for e in [Ecra::PorCima(0.20), Ecra::Camada] {
            let antes = cor(e, AZ, AM, 0.9, 0.7);
            assert_eq!(assenta(e, AZ, AM, 0.9, 0.7, 0.97, 0.0), antes);
        }
    }
}
