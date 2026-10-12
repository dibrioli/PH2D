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
    /// O filme como camada de reflectância sobre o assentado (o assentado como fundo OPACO). ⛔ Fora do
    /// produto: a lei de antes do «Glaze layering», só o CONTROLO dos portões (`mistura_wet_drip_tests`).
    Camada,
    /// **A lei do Rebelle** (dono 10/10, [`Ecra::Cobre`]): a claridade é a quantidade de pigmento, a
    /// camada de cima tapa a de baixo mais do que a sua força, e molhada parece mais forte.
    Cobre,
}

/// A lei com o que ela lê dos params: a velatura do filme no [`Lei::PorCima`].
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Ecra {
    #[default]
    Porto,
    PorCima(f64),
    /// [`Lei::Camada`] — ⛔ só o controlo dos portões.
    Camada,
    /// O filme é uma TINTA da cor guardada com a força da sua cobertura `a_f` (a claridade vem do
    /// pigmento: acumula até à cor escolhida), que TAPA o assentado com `h = 1 − (1 − a_f)^esconde`
    /// (`≥ a_f`: uma tinta clara tapa mais do que a sua força — o azul claro seco do Rebelle tapa ~60 % do
    /// vermelho; o extra pesado pela diferença de pigmento, a mesma tinta só soma), mais a velatura `filme`. Molhada, a força do tom vale `1 − (1 − a_f)^molhada`; o tapar
    /// não muda ao secar — o traço clareia SEM mudar de tom (o Rebelle: (131,124,201) → (194,188,224)).
    Cobre {
        filme: f64,
        esconde: f64,
        molhada: f64,
    },
}

impl Ecra {
    /// A mesma lei vista SECA — assentar e re-molhar preservam o ecrã seco, e (desde 11/10, ordem do dono) o
    /// render desenha com ela também a tinta molhada: o que se vê ao pintar é o que fica.
    #[must_use]
    pub fn seco(self) -> Self {
        match self {
            Ecra::Cobre { filme, esconde, .. } => Ecra::Cobre {
                filme,
                esconde,
                molhada: 1.0,
            },
            e => e,
        }
    }
}

/// A cor da camada e a sua cobertura (o que o render compõe): `sc` assentado (`a_s`) sob `uc` em
/// suspensão (`a_f`). ⛔ Uma porta só — o render e as transferências verticais passam por aqui.
#[inline]
pub fn cor_e_alfa(e: Ecra, sc: [f64; 3], uc: [f64; 3], a_s: f64, a_f: f64) -> ([f64; 3], f64) {
    let Ecra::Cobre {
        filme,
        esconde,
        molhada,
    } = e
    else {
        return (cor(e, sc, uc, a_s, a_f), a_s + a_f * (1.0 - a_s));
    };
    if a_f <= 0.0 {
        return (sc, a_s);
    }
    // MOLHADA: a COBERTURA do ponto inteiro (a tinta de cima e a seca debaixo da água) vale a densidade
    // `1 + (molhada − 1)·a_f`, com a cor da camada fixa — o papel molhado escurece tudo por igual e só até à
    // cor pura (o Rebelle: o cruzamento (131,124,201) → (194,188,224), seno do ângulo dos perfis 0,05; e o
    // amarelo molhado acumulado é a própria tinta, (245,205,25)). ⛔ Escalar a densidade da COR passava da
    // cor pura: o amarelo molhado ficava mais escuro que a tinta (0,90 do azul seco nas camadas).
    let p = 1.0 + (molhada - 1.0) * a_f;
    // `r` é o ponto sobre o branco; devolve a cor da camada e a cobertura que o compõem.
    let molha = |r: [f64; 3], la: f64| {
        let c = r.map(|v| ((v - 255.0 * (1.0 - la)) / la).clamp(0.0, 255.0));
        let la_w = if p == 1.0 {
            la
        } else {
            1.0 - (1.0 - la).powf(p)
        };
        (c, la_w)
    };
    let a_t = a_f;
    if a_s <= 0.0 {
        return molha(uc.map(|v| 255.0 - a_t * (255.0 - v)), a_t);
    }
    // O tapar EXTRA é da diferença de pigmento: a mesma tinta sobre si mesma só soma pigmento (sem ele,
    // re-molhar o azul para dentro de um filme azul clareava-o — medido no teste das transferências).
    let h = 1.0 - (1.0 - a_f).powf(esconde);
    let a_e = a_t + (h - a_t).max(0.0) * crate::colorops::diferenca_de_pigmento(sc, uc);
    let t_s = sc.map(|v| 255.0 - a_s * (255.0 - v));
    let c_eff = uc.map(|v| 255.0 - (a_t / a_e) * (255.0 - v));
    let mut r = [0, 1, 2].map(|k| t_s[k] * (1.0 - a_e) + c_eff[k] * a_e);
    velatura(&mut r, a_e, filme, 1.0, t_s, c_eff);
    let la = a_s + a_e * (1.0 - a_s);
    molha(r, la)
}

/// O que a célula mostra sobre o papel branco — o que assentar e re-molhar preservam.
#[inline]
pub fn sobre_branco(e: Ecra, sc: [f64; 3], uc: [f64; 3], a_s: f64, a_f: f64) -> [f64; 3] {
    let (c, la) = cor_e_alfa(e, sc, uc, a_s, a_f);
    c.map(|v| 255.0 * (1.0 - la) + v * la)
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
        Ecra::Cobre { .. } => cor_e_alfa(e, sc, uc, a_s, a_f).0,
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
///
/// ⛔ Os canais NÃO são independentes: o tapar extra pesa a diferença de pigmento das TRÊS cores, e mexer
/// num canal muda o ecrã dos outros. Cada canal lê o ecrã DEPOIS do anterior (Bug #49: com o ecrã do
/// início da volta, a derivada do verde media a mudança do vermelho e o assentado ia a verde-água).
fn resolve(mut x: [f64; 3], alvo: [f64; 3], ecra: impl Fn([f64; 3]) -> [f64; 3]) -> [f64; 3] {
    const H: f64 = 0.5;
    let mut y = ecra(x);
    for _ in 0..4 {
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
            y = ecra(x);
        }
    }
    x
}

/// **ASSENTAR:** a cor nova do assentado quando a cobertura do assentado passa de `a_s` a `a_s2` e a do
/// filme de `a_f` a `a_f2` (o filme `uc` não muda de cor) — a que mantém [`cor`]. Sem a caixa das tintas de
/// [`levanta`]: a tinta clara que TAPA a escura (a lei do Rebelle) guarda-se mais clara que a escura, e com o
/// tecto o cruzamento seco ficava malva (`05 §4b` «Mistura II», tetraedro 0 → 15 %, o Rebelle 0).
pub fn assenta(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
) -> [f64; 3] {
    let e = e.seco();
    if let Ecra::Cobre { .. } = e {
        let alvo = sobre_branco(e, sc, uc, a_s, a_f);
        if a_s <= 0.0 && a_f2 <= 0.0 {
            return uc;
        }
        // O palpite: o que assentou TAPA o assentado pela mesma lei (uma camada de `uc` com a cobertura da
        // massa que desceu) — com o palpite «o assentado de antes», sob um filme grosso o ecrã quase não o vê
        // e a cor escondida voltava ao secar (a armadilha do `resolve`).
        let a_dm = if a_s < 1.0 {
            (1.0 - (1.0 - a_s2) / (1.0 - a_s)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let x0 = if a_s <= 0.0 || a_s2 <= 0.0 {
            uc
        } else {
            let r = sobre_branco(e, sc, uc, a_s, a_dm);
            r.map(|v| (255.0 - (255.0 - v) / a_s2).clamp(0.0, 255.0))
        };
        return resolve(x0, alvo, |x| sobre_branco(e, x, uc, a_s2, a_f2));
    }
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

/// [`levanta`] na [`Ecra::Cobre`], antes da caixa das tintas.
fn levanta_cobre(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
) -> [f64; 3] {
    if a_f2 <= 0.0 {
        return uc;
    }
    let alvo = sobre_branco(e, sc, uc, a_s, a_f);
    let x0 = if a_f <= 0.0 {
        sc
    } else {
        let w = ((a_f2 - a_f) / a_f2).clamp(0.0, 1.0);
        [0, 1, 2].map(|c| uc[c] + (sc[c] - uc[c]) * w)
    };
    resolve(x0, alvo, |x| sobre_branco(e, sc, x, a_s2, a_f2))
}

/// **A caixa das tintas** do re-molhar: o filme que a água leva fica, por canal, entre a cor dele (`fica`) e a
/// do assentado que sobe (`vem`); com o filme vazio, é a do assentado. Sem ela a inversa escrevia, sob filme grosso,
/// uma cor mais clara que as duas, que a água carregava para onde não havia nada a tapar.
fn na_caixa(x: [f64; 3], fica: [f64; 3], vem: [f64; 3], tem_fica: bool) -> [f64; 3] {
    // Sem tinta a ficar (filme vazio), a única tinta é a que vem: a cor guardada do vazio não é tinta.
    if !tem_fica {
        return vem;
    }
    [0, 1, 2].map(|c| x[c].clamp(fica[c].min(vem[c]), fica[c].max(vem[c])))
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
    let e = e.seco();
    if let Ecra::Cobre { .. } = e {
        return na_caixa(
            levanta_cobre(e, sc, uc, a_s, a_f, a_s2, a_f2),
            uc,
            sc,
            a_f > 0.0,
        );
    }
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
            let cor = |e, sc, uc, a_s, a_f| sobre_branco(e, sc, uc, a_s, a_f);
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

    /// A MESMA tinta por cima de si mesma fica a mesma cor (o traço de uma cor não muda ao secar). Na lei do
    /// Rebelle, duas camadas da mesma tinta são UMA com a massa das duas: o tapar a mais só pesa quando o
    /// pigmento muda (mutação que sangra: tapar a mais sem a diferença de pigmento clareia `0,80 → 0,58`).
    #[test]
    fn a_mesma_tinta_nao_muda() {
        let e = Ecra::PorCima(0.20);
        let s2 = assenta(e, AM, AM, 0.9, 0.8, 0.95, 0.6);
        assert!(perto(s2, AM, 0.01), "{s2:?}");
        assert_eq!(assenta(e, [0.0; 3], AM, 0.0, 0.8, 0.3, 0.7), AM);
        let cobre = Ecra::Cobre {
            filme: 0.2,
            esconde: 3.0,
            molhada: 1.0,
        };
        let (duas, uma) = (
            sobre_branco(cobre, AM, AM, 0.6, 0.5),
            sobre_branco(cobre, AM, AM, 0.8, 0.0),
        );
        assert!(perto(duas, uma, 0.5), "duas camadas {duas:?} ≠ uma {uma:?}");
    }

    /// Tudo assentado = a cor que se via.
    #[test]
    fn secar_de_vez_fica_com_a_cor_que_se_via() {
        for e in [Ecra::PorCima(0.20), Ecra::Camada] {
            let antes = cor(e, AZ, AM, 0.9, 0.7);
            assert_eq!(assenta(e, AZ, AM, 0.9, 0.7, 0.97, 0.0), antes);
        }
    }

    /// Na lei do Rebelle o assentar preserva o ecrã SECO em transferências reais (≤ 5 % da massa por passe)
    /// e a MESMA tinta não muda ao trocar de camada (re-molhar). Re-molhar pigmento DIFERENTE muda o tapar
    /// (a camada de cima tapa mais) — sem invariante aí.
    #[test]
    fn na_lei_do_rebelle_assentar_preserva_o_ecra_seco() {
        let e = Ecra::Cobre {
            filme: 0.2,
            esconde: 3.0,
            molhada: 1.6,
        };
        for (a_s, a_f, d) in [
            (0.9, 0.95, 0.05),
            (0.6, 0.5, 0.05),
            (0.99, 0.3, 0.03),
            (0.2, 0.1, 0.05),
        ] {
            let a_s2 = 1.0 - (1.0 - a_s) * (1.0 - d);
            let a_f2 = 1.0 - (1.0 - a_f) / (1.0 - d);
            let antes = sobre_branco(e.seco(), AZ, AM, a_s, a_f);
            let s2 = assenta(e, AZ, AM, a_s, a_f, a_s2, a_f2);
            let depois = sobre_branco(e.seco(), s2, AM, a_s2, a_f2);
            assert!(
                perto(antes, depois, 1.5),
                "assenta {a_s},{a_f}: {antes:?} → {depois:?}"
            );
        }
        // A massa conserva-se: (1 − 0,95)(1 − 0,2) = (1 − 0,9)(1 − 0,6).
        let u2 = levanta(e, AM, AM, 0.95, 0.2, 0.9, 0.6);
        assert!(perto(u2, AM, 0.5), "a mesma tinta: {u2:?}");
    }

    /// **Bug #49 — azul escuro a assentar sobre azul claro seco não vai a verde.** O assentar corre em MUITOS
    /// passes (2–5 % da massa cada) e o ecrã tem de ficar o mesmo em todos: 4 azuis claros × 4 escuros, `a_s`
    /// 0,2–0,99, `a_f` 0,5–0,95, 60 passes. Antes (cada canal com a derivada do ecrã do início da volta) o
    /// pior subia o verde +15 a `d = 0,05` e o assentado ficava verde-água (83,178,211).
    #[test]
    fn assentar_azul_sobre_azul_nao_muda_o_tom() {
        let e = Ecra::Cobre {
            filme: 0.0,
            esconde: 5.0,
            molhada: 1.0,
        };
        let claros = [
            [120.0, 180.0, 235.0],
            [90.0, 190.0, 240.0],
            [150.0, 200.0, 255.0],
            [170.0, 210.0, 240.0],
        ];
        let escuros = [
            [20.0, 40.0, 140.0],
            [10.0, 30.0, 110.0],
            [0.0, 0.0, 128.0],
            [40.0, 60.0, 160.0],
        ];
        let mut pior = 0.0f64;
        for sc0 in claros {
            for uc in escuros {
                for a_s0 in [0.2, 0.5, 0.8, 0.95, 0.99] {
                    for a_f0 in [0.5, 0.8, 0.95] {
                        for d in [0.02, 0.05] {
                            let (mut sc, mut a_s, mut a_f) = (sc0, a_s0, a_f0);
                            let v0 = sobre_branco(e, sc, uc, a_s, a_f);
                            for _ in 0..60 {
                                let a_s2 = 1.0 - (1.0 - a_s) * (1.0 - d * a_f);
                                let a_f2 = (1.0 - (1.0 - a_f) / (1.0 - d * a_f)).max(0.0);
                                sc = assenta(e, sc, uc, a_s, a_f, a_s2, a_f2);
                                (a_s, a_f) = (a_s2, a_f2);
                                let v = sobre_branco(e, sc, uc, a_s, a_f);
                                pior = pior.max((v[1] - v[2]) - (v0[1] - v0[2]));
                            }
                        }
                    }
                }
            }
        }
        assert!(pior <= 3.0, "o assentar subiu o verde {pior:+.1} níveis");
    }

    /// Quanto `x` sai, por canal, da caixa de `a` e `b` (níveis; `0` dentro).
    fn fora_da_caixa(x: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
        (0..3)
            .map(|c| (x[c] - a[c].max(b[c])).max(a[c].min(b[c]) - x[c]))
            .fold(0.0, f64::max)
    }

    /// **O re-molhar fica na caixa das tintas** (prompt 16, Fase E — o adversário da Fase D′): em 2 240
    /// estados (4 tintas × 4, `a_s` 1e-4–0,95, `a_f` 0,05–0,999, `d` 0,5–5 %), com o tapar do produto e o do
    /// «Glaze layering», o filme que a água leva fica, por canal, entre as duas tintas que se juntam. O estado
    /// pior do adversário (azul escuro sob quase branco, `a_s` 0,2, `a_f` 0,3, `d` 0,05) entra por nome.
    /// CONTROLO: sem a caixa reprova.
    #[test]
    fn as_transferencias_verticais_ficam_na_caixa_das_tintas() {
        const AZUL: [f64; 3] = [20.0, 50.0, 110.0];
        const VERMELHO: [f64; 3] = [220.0, 60.0, 60.0];
        const QUASE_BRANCO: [f64; 3] = [248.0, 246.0, 240.0];
        let tintas = [AZUL, AM, VERMELHO, QUASE_BRANCO];
        let ecra = |esconde| Ecra::Cobre {
            filme: crate::colorops::VELATURA_DO_FILME,
            esconde,
            molhada: crate::colorops::TINTA_MOLHADA_ESCURECE,
        };
        let (mut n, mut pior, mut livre) = (0usize, 0.0f64, 0.0f64);
        for esconde in [
            crate::colorops::TINTA_ESCONDE,
            crate::colorops::TINTA_ESCONDE_EM_VELATURA,
        ] {
            let e = ecra(esconde);
            for sc in tintas {
                for uc in tintas {
                    for a_s in [1e-4, 1e-3, 1e-2, 0.05, 0.2, 0.6, 0.95] {
                        for a_f in [0.05, 0.3, 0.6, 0.9, 0.96, 0.99, 0.999] {
                            for d in [0.005, 0.02, 0.05] {
                                if a_f <= d + 1e-6 {
                                    continue;
                                }
                                let a_s2 = 1.0 - (1.0 - a_s) * (1.0 - d);
                                let a_f2 = 1.0 - (1.0 - a_f) / (1.0 - d);
                                let u2 = levanta(e, sc, uc, a_s2, a_f2, a_s, a_f);
                                let solto = levanta_cobre(e, sc, uc, a_s2, a_f2, a_s, a_f);
                                n += 1;
                                pior = pior.max(fora_da_caixa(u2, sc, uc));
                                livre = livre.max(fora_da_caixa(solto, sc, uc));
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(n, 2 * 2240);
        assert!(pior <= 1e-9, "o filme saiu da caixa das tintas: {pior:.3}");
        assert!(
            livre > 1.0,
            "CONTROLO: sem a caixa o filme nunca saiu dela ({livre:.3}) — a varredura não discrimina"
        );
        let e = ecra(crate::colorops::TINTA_ESCONDE);
        let (a_s, a_f, d) = (0.2, 0.3, 0.05);
        let (a_s2, a_f2) = (1.0 - (1.0 - a_s) * (1.0 - d), 1.0 - (1.0 - a_f) / (1.0 - d));
        let u2 = levanta(e, AZUL, QUASE_BRANCO, a_s2, a_f2, a_s, a_f);
        assert!(fora_da_caixa(u2, AZUL, QUASE_BRANCO) <= 1e-9, "{u2:?}");
    }

    /// Re-molhar tinta SECA (filme vazio) leva a cor do assentado: a cor guardada de um filme sem massa é a de
    /// uma tinta que já saiu. CONTROLO: a cor velha do vazio é outra tinta, longe do assentado.
    #[test]
    fn re_molhar_tinta_seca_leva_a_cor_do_assentado() {
        let velha = [220.0, 60.0, 60.0];
        for esconde in [
            crate::colorops::TINTA_ESCONDE,
            crate::colorops::TINTA_ESCONDE_EM_VELATURA,
        ] {
            let e = Ecra::Cobre {
                filme: crate::colorops::VELATURA_DO_FILME,
                esconde,
                molhada: crate::colorops::TINTA_MOLHADA_ESCURECE,
            };
            let u2 = levanta(e, AZ, velha, 0.6, 0.0, 0.55, 0.1);
            assert!(
                perto(u2, AZ, 1.0),
                "esconde {esconde}: {u2:?} ≠ o assentado {AZ:?}"
            );
            assert!(
                !perto(velha, AZ, 1.0),
                "CONTROLO: a cor velha não discrimina"
            );
        }
    }
}
