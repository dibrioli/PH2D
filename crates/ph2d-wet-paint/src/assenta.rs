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
        /// **O Glaze da tinta da célula** (prompt 42, o plano [`crate::grid::Grid::vela`]): a fracção do filme que
        /// FILTRA o assentado — `1` = a multiplicação das duas camadas sobre o branco (o marcador do Rebelle),
        /// nesta fracção — a [`crate::colorops::vela_da_camada`] FIXADA no pouso, contra a camada de baixo de então
        /// (lida da cor de agora, caía 0,94 → 0,77 enquanto o assentado absorvia o filme e o cruzamento clareava
        /// +14 níveis ao secar). `0` = a lei da obra.
        vela: f64,
    },
}

impl Ecra {
    /// A mesma lei vista SECA — assentar e re-molhar preservam o ecrã seco, e (desde 11/10, ordem do dono) o
    /// render desenha com ela também a tinta molhada: o que se vê ao pintar é o que fica.
    #[must_use]
    pub fn seco(self) -> Self {
        match self {
            Ecra::Cobre {
                filme,
                esconde,
                vela,
                ..
            } => Ecra::Cobre {
                filme,
                esconde,
                molhada: 1.0,
                vela,
            },
            e => e,
        }
    }

    /// A lei da CÉLULA: a da sessão com o Glaze da tinta que lá está ([`crate::grid::Grid::vela`]). Só a lei do
    /// Rebelle o lê; `0` devolve a da sessão ao bit.
    #[inline]
    #[must_use]
    pub fn com_vela(self, v: f32) -> Self {
        match self {
            Ecra::Cobre {
                filme,
                esconde,
                molhada,
                ..
            } if v > 0.0 => Ecra::Cobre {
                filme,
                esconde,
                molhada,
                vela: f64::from(v),
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
        vela,
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
    // ⛔ A CAIXA das tintas (Bug #50): o tapar extra pinta-se com a tinta diluída em BRANCO — é o que faz o claro
    // cobrir o escuro como no Rebelle, mas onde o filme é mais ESCURO o branco clareava o ponto além das duas
    // tintas (a orla esbranquiçada). Cada canal pára na mais clara das duas: o Rebelle nunca passa dela (as
    // bordas do óleo sobre o azul seco, `fixtures/rebelle/{oil,express}.png`).
    let mut r = [0, 1, 2].map(|k| t_s[k] * (1.0 - a_e) + c_eff[k] * a_e);
    velatura(&mut r, a_e, filme, 1.0, t_s, c_eff);
    // O GLAZE do pincel: o filme FILTRA o assentado — a fracção `m` do ponto é o produto das duas camadas sobre o
    // branco. Uma vez por camada (o filme e o assentado), nunca por carimbo: é o que não leva ao preto.
    if vela > 0.0 {
        let m = vela;
        for k in 0..3 {
            let f = 255.0 - a_t * (255.0 - uc[k]);
            r[k] += m * (t_s[k] * f / 255.0 - r[k]);
        }
    }
    let r = [0, 1, 2].map(|k| r[k].min(t_s[k].max(uc[k])));
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

/// A sensibilidade do ecrã a uma cor (níveis por nível) abaixo da qual o [`resolve`] não a corrige: onde o filme
/// grosso a esconde, corrigi-la escrevia uma cor ESCONDIDA que aparecia quando o filme afinava (o miolo
/// esverdeava +24 ao secar).
const CEGO: f64 = 0.1;

/// O mesmo sob o Glaze (prompt 42): o PRODUTO das camadas deixa o assentado à vista em todo canal que o filme não
/// zera, com sensibilidade pequena mas real — com [`CEGO`] o assentar ficava no palpite e o ecrã mudava.
/// Grelha `assentar_guarda_o_ecra_com_o_glaze` (estados > 1,5 · o pior; Glaze 0,5 | 1): `0,1` 70 · 7,39 | 33 ·
/// 12,71 — `0,03` 19 · 6,41 | 15 · 12,71 — **`0,01`** 11 · 6,41 | 15 · 12,71 (`0,001` igual; a força bruta
/// local acha 7 · 6,41 | 10 · 10,48). Secar de vez (5 % por passe, 12 pares × 4 massas) desliza em média
/// 4,1 → 2,4 níveis a Glaze 0,55 e 2,8 → 2,6 a 1, o pior igual. ⛔ Newton CONJUNTO 3×3 com retrocesso não ganha
/// nada sobre este (11 · 6,41 | 15 · 12,69).
const CEGO_COM_GLAZE: f64 = 0.01;

/// Newton por canal sobre a cor que muda (`x`), a partir do palpite físico, até o ecrã voltar a `alvo`
/// (a velatura do filme não compõe em passos: o palpite deixa um resto). A derivada é numérica — com o
/// filme grosso a velatura pesa mais que a cobertura (medido: assumir `1 − a` dava passos 4× grandes e o
/// miolo ia a preto); o passo tem tecto.
///
/// ⛔ Os canais NÃO são independentes: o tapar extra pesa a diferença de pigmento das TRÊS cores, e mexer
/// num canal muda o ecrã dos outros. Cada canal lê o ecrã DEPOIS do anterior (Bug #49: com o ecrã do
/// início da volta, a derivada do verde media a mudança do vermelho e o assentado ia a verde-água).
///
/// `cego`: a sensibilidade abaixo da qual o canal fica no palpite ([`CEGO`]; [`CEGO_COM_GLAZE`] sob o Glaze).
fn resolve(
    mut x: [f64; 3],
    alvo: [f64; 3],
    cego: f64,
    ecra: impl Fn([f64; 3]) -> [f64; 3],
) -> [f64; 3] {
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
            if d.abs() < cego {
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
    if let Ecra::Cobre { vela, .. } = e {
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
        let mut x0 = if a_s <= 0.0 || a_s2 <= 0.0 {
            uc
        } else {
            let r = sobre_branco(e, sc, uc, a_s, a_dm);
            r.map(|v| (255.0 - (255.0 - v) / a_s2).clamp(0.0, 255.0))
        };
        // O GLAZE (prompt 42): a parte multiplicada tem inversa FECHADA — o assentado novo é o de antes vezes o
        // filme de antes sobre o de depois, canal a canal. Sem ela o Newton ficava no palpite onde o filme
        // escuro esconde um canal (a regra de cima) e o cruzamento clareava ao secar (medido: até 19 níveis, e
        // a força bruta achava a cor certa a < 0,4).
        if vela > 0.0 && a_s > 0.0 && a_s2 > 0.0 {
            let m = vela;
            for k in 0..3 {
                let t_s = 255.0 - a_s * (255.0 - sc[k]);
                let (f, f2) = (
                    255.0 - a_f * (255.0 - uc[k]),
                    255.0 - a_f2 * (255.0 - uc[k]),
                );
                let t_s2 = if f2 > 0.0 { t_s * f / f2 } else { t_s };
                let produto = (255.0 - (255.0 - t_s2) / a_s2).clamp(0.0, 255.0);
                x0[k] += m * (produto - x0[k]);
            }
        }
        let cego = if vela > 0.0 { CEGO_COM_GLAZE } else { CEGO };
        return resolve(x0, alvo, cego, |x| sobre_branco(e, x, uc, a_s2, a_f2));
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
            resolve(x0, alvo, CEGO, |x| cor(e, x, uc, a_s2, a_f2))
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
    resolve(x0, alvo, CEGO, |x| sobre_branco(e, sc, x, a_s2, a_f2))
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
            resolve(x0, alvo, CEGO, |x| cor(e, sc, x, a_s2, a_f2))
        }
    }
}

#[cfg(test)]
#[path = "assenta/tests.rs"]
mod tests;
