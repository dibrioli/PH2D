//! **Assentar não muda o que se vê** (dono 10/10: «pintando sobre tinta seca o traço muda de aparência no
//! mouse up — isso não pode acontecer»). O pigmento que desce da água para o papel (assentar, Dry canvas) ou
//! sobe do papel para a água (re-molhar) muda de camada NO MESMO PONTO: a coluna de tinta é a mesma e a cor
//! que se vê também. As duas transferências verticais resolvem a cor nova pela INVERSA da lei do ecrã
//! ([`cor`], a mesma porta do render); a mistura de pigmentos fica onde acontece de facto — na água, quando
//! chega tinta (o pouso do dab) ou quando ela corre (a advecção). Medido antes (prompt 15b): fábrica Δ29 ·
//! «Pigment mixing» Δ81 · «Glaze layering» Δ28 · as duas Δ126. O `Lei::Porto` guarda as junções de antes.
use crate::colorops::transfer::{srgb255_of_linear, srgb255_to_linear};
use crate::colorops::{km_glaze_channel_linear, velatura};

#[path = "assenta/ponto.rs"]
mod ponto;
use ponto::{Parada, d_de, prepara};
#[path = "assenta/newton.rs"]
mod newton;
pub use newton::{NEWTON_VOLTAS, VOLTAS_DE_FABRICA, VOLTAS_EXATAS, liga_o_palpite, palpite_ligado};
use newton::{newton_voltas, resolve};
#[path = "assenta/memo.rs"]
mod memo;
pub use memo::{
    MEMO_ASSENTA, MEMO_BITS, MEMO_CONTA, MEMO_POTENCIA, assenta_memo, assenta_memo_f32,
    memo_esquece,
};

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
        /// **O óleo debaixo** ([`Ecra::com_oleo`], dono 30/10): tira esta fracção do tapar extra — `0` = a lei da obra
        /// (entre aguarelas o tapar e o seu branco são os do Rebelle), `1` = sobre óleo, a cobertura é a do pigmento.
        oleo: f64,
        /// **A tinta OPACA da célula** ([`Ecra::com_opaca`], dono 31/10): a fracção com que o ponto fica preso ao
        /// SEGMENTO entre as duas tintas (o claro cobre o escuro sem o branco do tapar extra). `0` = a lei do Rebelle.
        opaca: f64,
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
                oleo,
                opaca,
                ..
            } => Ecra::Cobre {
                filme,
                esconde,
                molhada: 1.0,
                vela,
                oleo,
                opaca,
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
                oleo,
                opaca,
                ..
            } if v > 0.0 => Ecra::Cobre {
                filme,
                esconde,
                molhada,
                vela: f64::from(v),
                oleo,
                opaca,
            },
            e => e,
        }
    }

    /// A lei da CÉLULA com o «Glaze layering» que a tinta dela pousou ([`crate::grid::aparencia`], W3b): o tapar vai
    /// da lei da sessão a [`crate::colorops::TINTA_ESCONDE_EM_VELATURA`] pela fracção `v`; `0` devolve a da sessão ao
    /// bit, `1` dá a de velatura ao bit.
    #[inline]
    #[must_use]
    pub fn com_vidrado(self, v: f32) -> Self {
        match self {
            Ecra::Cobre {
                filme,
                esconde,
                molhada,
                vela,
                oleo,
                opaca,
            } if v > 0.0 => Ecra::Cobre {
                filme,
                esconde: if v >= 1.0 {
                    crate::colorops::TINTA_ESCONDE_EM_VELATURA
                } else {
                    esconde + (crate::colorops::TINTA_ESCONDE_EM_VELATURA - esconde) * f64::from(v)
                },
                molhada,
                vela,
                oleo,
                opaca,
            },
            e => e,
        }
    }

    /// A lei da CÉLULA com o óleo que ela tem debaixo ([`crate::grid::oleo_celula`], dono 30/10): sobre o óleo o tapar
    /// extra sai: a aguada cobre pela quantidade de pigmento, sem o branco. `0` devolve a da sessão ao bit.
    /// Se a última mão que pousou na célula tinha o Real Oil ([`OleoCelula::pincel_de_oleo`]) o extra fica (a
    /// cobertura do óleo sobre o óleo) e a célula mistura como a OPACA (dono 31/10, `smoke_31_10_oleo/dono_5.png`:
    /// o extra virava papel à vista e bege guardado). Lê a CÉLULA, nunca o pincel na mão (E10e1).
    ///
    /// [`OleoCelula::pincel_de_oleo`]: crate::grid::oleo_celula::OleoCelula::pincel_de_oleo
    #[inline]
    #[must_use]
    pub fn com_oleo(self, c: crate::grid::oleo_celula::OleoCelula) -> Self {
        match self {
            Ecra::Cobre { .. } if c.oleo > 0.0 && c.pincel_de_oleo => {
                self.com_opaca((f64::from(c.oleo) / OLEO_PLENO).min(1.0) as f32)
            }
            Ecra::Cobre {
                filme,
                esconde,
                molhada,
                vela,
                opaca,
                ..
            } if c.oleo > 0.0 => Ecra::Cobre {
                filme,
                esconde,
                molhada,
                vela,
                oleo: (f64::from(c.oleo) / OLEO_PLENO).min(1.0),
                opaca,
            },
            e => e,
        }
    }

    /// A lei da CÉLULA com a tinta OPACA que ela pousou ([`crate::grid::aparencia::opaca`],
    /// [`crate::tuning::Knob::Opaque`], dono 31/10: «amarelo sobre azul seco fica com bordas brancas»): o ponto fica no
    /// SEGMENTO entre as duas tintas. `0` devolve a da sessão ao bit.
    #[inline]
    #[must_use]
    pub fn com_opaca(self, o: f32) -> Self {
        match self {
            Ecra::Cobre {
                filme,
                esconde,
                molhada,
                vela,
                oleo,
                opaca,
            } if o > 0.0 => Ecra::Cobre {
                filme,
                esconde,
                molhada,
                vela,
                oleo,
                opaca: opaca.max(f64::from(o).min(1.0)),
            },
            e => e,
        }
    }
}

/// A fracção de óleo a partir da qual o tapar extra sai todo: a orla fina do Dabber guarda 4–12 % de
/// óleo (medido no J8, `oleo_celula_tests`) e é óleo à vista; abaixo, a rampa liga ao branco sem degrau.
const OLEO_PLENO: f64 = 0.04;

/// A cor da camada e a sua cobertura (o que o render compõe): `sc` assentado (`a_s`) sob `uc` em
/// suspensão (`a_f`). ⛔ Uma porta só — o render e as transferências verticais passam por aqui.
#[inline]
pub fn cor_e_alfa(e: Ecra, sc: [f64; 3], uc: [f64; 3], a_s: f64, a_f: f64) -> ([f64; 3], f64) {
    cor_e_alfa_tapando(e, sc, uc, a_s, a_f, || tapa(e, a_f))
}

/// O tapar da [`Ecra::Cobre`] a cobertura `a_f`, `h = 1 − (1 − a_f)^esconde` — só lê `a_f`, e o Newton do [`resolve`]
/// avalia o ecrã muitas vezes à MESMA cobertura: a potência calcula-se uma vez por célula ([`prepara`]).
#[inline]
fn tapa(e: Ecra, a_f: f64) -> f64 {
    match e {
        Ecra::Cobre { esconde, .. } => 1.0 - memo::potencia(1.0 - a_f, esconde),
        _ => 0.0,
    }
}

/// O [`tapa`] de `a_f2` reaproveitando o de `a_f` se o alvo o calculou e a cobertura repete (o mesmo `powf`, ao bit).
#[inline]
fn tapa_de_novo(e: Ecra, h_af: Option<f64>, a_f: f64, a_f2: f64) -> f64 {
    match h_af {
        Some(h) if a_f2.to_bits() == a_f.to_bits() => h,
        _ => tapa(e, a_f2),
    }
}

/// [`cor_e_alfa`] com o [`tapa`] de `a_f` à mão.
#[inline]
fn cor_e_alfa_tapando(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    tapa: impl FnOnce() -> f64,
) -> ([f64; 3], f64) {
    let vela = match e {
        Ecra::Cobre { vela, .. } => vela,
        _ => 0.0,
    };
    prepara(e, sc, uc, a_s, a_f, tapa, || d_de(vela, sc, uc)).cor_e_alfa()
}

/// ⭐ **A caixa das tintas na COR que se compõe** (smoke do dono 30/10: as manchas brancas do molhado sobre tinta fina,
/// num papel de COR). A [`Ecra::Cobre`] decide o ponto SOBRE O BRANCO e devolve-o como `(cor, la)`; a caixa de
/// [`cor_e_alfa`] prende o ponto sobre o branco, mas a cor que sai do `molha` ainda passava da mais clara das duas
/// tintas (azul fino sob laranja: `[239,194,168]`, o laranja é `[240,130,60]`) — o branco que a conta embutiu na COR,
/// que só o papel branco esconde. Re-exprime o MESMO ponto sobre o branco com a cobertura que põe cada canal na caixa
/// (`≤` a mais clara das duas): o resto do branco passa a ser transparência, e o papel de cor aparece por ela.
/// E o branco do TAPAR extra (dono 30/10, `o_tapar_extra_nao_pinta_branco_no_papel_de_cor`): o que o claro esconde do
/// escuro é a tinta de baixo afastada — o papel aparece por ela, como no Rebelle (doc 47), nunca branco pintado.
/// Sobre o branco, o mesmo píxel; uma célula sem as duas camadas não muda. `e` é a lei da CÉLULA (o Glaze, o óleo).
#[inline]
#[must_use]
pub fn na_caixa_das_tintas(
    e: Ecra,
    (c, la): ([f64; 3], f64),
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
) -> ([f64; 3], f64) {
    let Ecra::Cobre {
        molhada,
        vela,
        oleo,
        opaca,
        ..
    } = e
    else {
        return (c, la);
    };
    if a_s <= 0.0 || a_f <= 0.0 || la <= 0.0 {
        return (c, la);
    }
    let mut a = la;
    for k in 0..3 {
        let teto = sc[k].max(uc[k]);
        if c[k] > teto && teto < 255.0 {
            a = a.min(la * (255.0 - c[k]) / (255.0 - teto));
        }
    }
    // O BRANCO do tapar extra (`(a_e − a_t)·255` no ponto sobre o branco) é a tinta de baixo AFASTADA, não pigmento:
    // passa a transparência e o papel aparece (dono 30/10, a faixa creme; doc 47). Só na lei seca (a do render).
    // A tinta OPACA não afasta: cobre (dono 31/10, o papel por óleo grosso).
    if molhada == 1.0 {
        let d = d_de(vela, sc, uc);
        let extra =
            (tapa(e, a_f) - a_f).max(0.0) * d * (1.0 - oleo.max(0.0)) * (1.0 - opaca.min(1.0));
        a = a.min(la - extra);
    }
    // A cor re-expressa não desce de 0: o branco que sai é no máximo o que a cor tem.
    let piso = c
        .iter()
        .fold(0.0f64, |m, &v| m.max(la * (255.0 - v) / 255.0));
    let a = a.max(piso);
    if a >= la || a <= 0.0 {
        return (c, la);
    }
    (
        c.map(|v| (255.0 - la * (255.0 - v) / a).clamp(0.0, 255.0)),
        a,
    )
}

/// O que a célula mostra sobre o papel branco — o que assentar e re-molhar preservam.
#[inline]
pub fn sobre_branco(e: Ecra, sc: [f64; 3], uc: [f64; 3], a_s: f64, a_f: f64) -> [f64; 3] {
    let (c, la) = cor_e_alfa(e, sc, uc, a_s, a_f);
    c.map(|v| 255.0 * (1.0 - la) + v * la)
}

/// [`sobre_branco`] com o [`tapa`] de `a_f` já calculado (a porta que os testes medem; o [`resolve`] usa [`prepara`]).
#[cfg(test)]
fn sobre_branco_tapando(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    h: f64,
) -> [f64; 3] {
    let (c, la) = cor_e_alfa_tapando(e, sc, uc, a_s, a_f, || h);
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
    assenta_com_voltas(newton_voltas(), e, sc, uc, a_s, a_f, a_s2, a_f2)
}

/// [`assenta`] com as voltas do Newton à mão ([`VOLTAS_EXATAS`] = a conta exacta): os portões comparam sem mexer na
/// chave global.
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn assenta_com_voltas(
    voltas: u8,
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
) -> [f64; 3] {
    let e = e.seco();
    if let Ecra::Cobre { vela, opaca, .. } = e {
        // O filme `uc` fica parado: a sua metade de `d` faz-se uma vez, e `d(sc, uc)` é o MESMO no alvo e no palpite.
        let par = Parada::de(vela, uc);
        let mut d_scuc = None;
        let mut h_af = None;
        // Com 0 voltas o Newton não lê o alvo nem o `h2`: só o palpite (rodada 4).
        let alvo = if voltas == 0 {
            [0.0; 3]
        } else {
            prepara(
                e,
                sc,
                uc,
                a_s,
                a_f,
                || *h_af.get_or_insert_with(|| tapa(e, a_f)),
                || *d_scuc.get_or_insert_with(|| par.d(vela, sc)),
            )
            .sobre_branco()
        };
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
            let r = prepara(
                e,
                sc,
                uc,
                a_s,
                a_dm,
                || tapa(e, a_dm),
                || *d_scuc.get_or_insert_with(|| par.d(vela, sc)),
            )
            .sobre_branco();
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
        if voltas == 0 {
            return ponto::na_caixa_opaca(opaca, a_s, sc, uc, x0);
        }
        let cego = if vela > 0.0 { CEGO_COM_GLAZE } else { CEGO };
        let h2 = tapa_de_novo(e, h_af, a_f, a_f2);
        let ecra = |x: [f64; 3]| prepara(e, x, uc, a_s2, a_f2, || h2, || par.d(vela, x));
        let x = resolve(
            voltas,
            x0,
            alvo,
            cego,
            |x| ecra(x).sobre_branco(),
            |x, c| ecra(x).sobre_branco_canal(c),
        );
        return ponto::na_caixa_opaca(opaca, a_s, sc, uc, x);
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
            let ecra = |x| cor(e, x, uc, a_s2, a_f2);
            resolve(voltas, x0, alvo, CEGO, ecra, |x, c| ecra(x)[c])
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
    // O assentado `sc` fica parado: a sua metade de `d` faz-se uma vez.
    let vela = match e {
        Ecra::Cobre { vela, .. } => vela,
        _ => 0.0,
    };
    let x0 = if a_f <= 0.0 {
        sc
    } else {
        let w = ((a_f2 - a_f) / a_f2).clamp(0.0, 1.0);
        [0, 1, 2].map(|c| uc[c] + (sc[c] - uc[c]) * w)
    };
    let par = Parada::de(vela, sc);
    let mut h_af = None;
    let alvo = prepara(
        e,
        sc,
        uc,
        a_s,
        a_f,
        || *h_af.get_or_insert_with(|| tapa(e, a_f)),
        || par.d(vela, uc),
    )
    .sobre_branco();
    let h2 = tapa_de_novo(e, h_af, a_f, a_f2);
    let ecra = |x: [f64; 3]| prepara(e, sc, x, a_s2, a_f2, || h2, || par.d(vela, x));
    resolve(
        VOLTAS_EXATAS,
        x0,
        alvo,
        CEGO,
        |x| ecra(x).sobre_branco(),
        |x, c| ecra(x).sobre_branco_canal(c),
    )
}

/// **A caixa das tintas** do re-molhar: o filme que a água leva fica, por canal, entre a cor dele (`fica`) e a
/// do assentado que sobe (`vem`); com o filme vazio, é a do assentado. Sem ela a inversa escrevia, sob filme grosso,
/// uma cor mais clara que as duas, que a água carregava para onde não havia nada a tapar.
#[inline(always)]
fn na_caixa(x: [f64; 3], fica: [f64; 3], vem: [f64; 3], tem_fica: bool) -> [f64; 3] {
    // Sem tinta a ficar (filme vazio), a única tinta é a que vem: a cor guardada do vazio não é tinta.
    if !tem_fica {
        return vem;
    }
    [0, 1, 2].map(|c| x[c].clamp(fica[c].min(vem[c]), fica[c].max(vem[c])))
}

/// **RE-MOLHAR:** a cor nova do filme quando ele recebe pigmento do assentado (`sc` não muda de cor; as
/// coberturas passam a `a_s2`, `a_f2`) — a que mantém [`cor`].
///
/// SALTA (31/10, ao bit): na [`Ecra::Cobre`] com coberturas IGUAIS (o `alpha_of_mass` saturado — 98,9 % das células
/// do wash) ⇒ `w = 0`, `x0 = uc`, e o avaliador em `x0` é a conta do alvo com os mesmos argumentos ⇒ a 1.ª verificação
/// do [`resolve`] sai e devolve `x0`. Este atalho fica `#[inline]` à frente do resto (`levanta_resto`, fora de linha):
/// com os arrays à mão no chamador, 70,9 → 45,2 ms por passe de secagem a 4K (rodada 5, `MEDIDA_31_10_RODADA5.md`).
#[inline(always)]
pub fn levanta(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
) -> [f64; 3] {
    if matches!(e, Ecra::Cobre { .. })
        && a_f > 0.0
        && a_f.to_bits() == a_f2.to_bits()
        && a_s.to_bits() == a_s2.to_bits()
    {
        let w = ((a_f2 - a_f) / a_f2).clamp(0.0, 1.0);
        // A caixa das tintas ([`na_caixa`] com filme) por canal, sem arrays intermédios.
        let mut r = [0.0; 3];
        for c in 0..3 {
            let (u, s) = (uc[c], sc[c]);
            r[c] = (u + (s - u) * w).clamp(u.min(s), u.max(s));
        }
        return r;
    }
    levanta_resto(e, sc, uc, a_s, a_f, a_s2, a_f2)
}

#[inline(never)]
fn levanta_resto(
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
            let ecra = |x| cor(e, sc, x, a_s2, a_f2);
            resolve(VOLTAS_EXATAS, x0, alvo, CEGO, ecra, |x, c| ecra(x)[c])
        }
    }
}

#[cfg(test)]
#[path = "assenta/aproxima_tests.rs"]
mod aproxima_tests;
#[cfg(test)]
#[path = "assenta/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "assenta_bit_tests.rs"]
mod assenta_bit_tests;
