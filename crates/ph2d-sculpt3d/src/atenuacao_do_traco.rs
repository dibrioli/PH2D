//! ⭐⭐ **O PASSO E A ATENUAÇÃO DO TRAÇO ARRASTADO** — do [`Verb::Plane`]
//! (`SPEC_pincel_de_plano.md` §14.4) e do [`Verb::DrawSharp`]
//! (`SPEC_pincel_afiado.md` §5).
//!
//! Num traço arrastado os dabs sobrepõem-se, e cada um é enfraquecido para que a soma não dependa
//! do espaçamento. O factor é o inverso do máximo, sobre DEZ fases, da soma da curva de queda nas
//! posições dos dabs vizinhos:
//!
//! ```text
//! a = 1 / max_fase Σ_j curva(|fase − 1 + j·h|)     h = espaçamento_% / 50,  j < ⌊100 / espaçamento_%⌋
//! força por dab = força² · pressão · (1 + a) / 2      (e 0,5 · força² · pressão · a no afastar invertido)
//! ```
//!
//! | curva | espaçamento | `a` | `(1 + a)/2` |
//! |---|---|---|---|
//! | suave | `7 %` (o perfil *aparar*) | `0,140` | **`0,570`** |
//! | suave | `10 %` | `0,200` | `0,600` |
//! | constante | `7 %` | `0,071` | `0,536` |
//!
//! ⚠️ **As fases são DEZ pontos** (`0; 0,1; …; 0,9`), não um máximo no contínuo — é essa
//! discretização que dá a tabela. A curva é a de queda do pincel **sem** a dureza.
//!
//! ⚠️ **Só os DOIS verbos que a declaram, e só arrastados.** A porta por script (a bancada, o
//! oráculo por script) dá os dabs um a um com o factor em `1`, e os outros verbos desta casa não
//! têm esta lei — dá-la a eles mudaria paridades medidas contra as referências deles.
//!
//! ⚠️⚠️ **E os dois NÃO partilham a fórmula do factor, só a do `a`:** o plano usa `(1 + a)/2` e o
//! afiado usa **`a` cru** (`0,24591` a `5 %` com a curva afiada). *Duas leis com a mesma forma são
//! a coisa mais fácil de unificar por engano* — medido, trocá-las erra `4,7e-2` contra `4,9e-4`.
//!
//! | curva | espaçamento | `a` | quem o usa |
//! |---|---|---|---|
//! | afiada | `5 %` (o pincel afiado) | `0,24591` | o afiado, **cru** |
//! | suave | `7 %` (o perfil *aparar*) | `0,140` | o plano, como `(1 + a)/2 = 0,570` |
//!
//! ⚠️ **O passo** é `raio × espaçamento_% / 50` — `0,14 R` a `7 %` —, contra os `0,15 R` que a
//! casa dá a todos os outros verbos. Escrito como o alvo o escreve e não como `0,14`: a `50` px de
//! raio `50 × 7 / 50` é **exactamente** `7`, e `0,14 × 50` não é.

use crate::{Brush, Falloff, PlanoInversao, Verb};

/// O espaçamento do pincel de plano, em % do DIÂMETRO — o perfil *aparar* do alvo (espec §14.2).
pub const ESPACAMENTO_DO_PLANO_PCT: f32 = 7.0;

/// ⛔ **O espaçamento de fábrica do pincel afiado NO ALVO**, em % do DIÂMETRO
/// (`SPEC_pincel_afiado.md` §5.1 e §6), lido correndo o programa.
///
/// ⚠️ **Ele é um FACTO sobre o alvo, e não o que este produto ship** — ver
/// [`ESPACAMENTO_DO_AFIADO_PCT`]. É este o número que as `80` fixturas do corpus
/// fixam no cabeçalho, e é com ele que a bancada arrasta: *uma bancada que
/// arrastasse com o nosso espaçamento deixaria de ter oráculo.*
pub const ESPACAMENTO_DO_AFIADO_DO_ALVO_PCT: f32 = 5.0;

/// ⭐⭐⭐ **O espaçamento do pincel AFIADO que ESTE produto ship**, em % do DIÂMETRO —
/// **`2 %`, por ordem do dono** (16/09: *«o spacing está alto e fica meio pontilhada.
/// Reduza o spacing»*), contra os `5 %` de fábrica do alvo
/// ([`ESPACAMENTO_DO_AFIADO_DO_ALVO_PCT`]).
///
/// ⭐⭐ **A troca é barata, e é MEDIDA — a profundidade do vinco é praticamente
/// independente do espaçamento**, porque quem a limita é a auto-limitação deste
/// pincel (a queda mede-se das posições do **pen-down**, logo o cursor afasta-se
/// delas enquanto cava e a profundidade converge). Varrido sobre a fixtura da
/// silhueta, `D/R` nas seis bandas do regime:
///
/// | espaçamento | `D/R` no meio | `D/R` junto à borda |
/// |---|---|---|
/// | `5 %` (o alvo) | `0,2041` | `0,2061` |
/// | `3 %` | `0,2019` | `0,2061` |
/// | `2 %` (ship) | `0,2007` | `0,2032` |
/// | `1 %` | `0,1992` | `0,2012` |
///
/// ⇒ `2 %` custa **`−1,4 %`** de profundidade e entrega `2,5×` a densidade de
/// dabs. ⚠️ **A espec §5.3 mede a mesma coisa do outro lado** (`8 %` e `10 %`
/// dão `−0,2 %` e `−0,6 %`), e é por isso que ela diz que o espaçamento *«quase
/// não é alavanca»* — verdade sobre a PROFUNDIDADE, e falso sobre a
/// CONTINUIDADE do sulco, que é o que o dono vê.
///
/// ⛔ **E o RECURSO que fixa o piso é o RELÓGIO**, medido (`--release`, mínimo de
/// três, malha de `185 977` vértices, o traço da silhueta a `1` px por evento):
/// `0,472 ms` por evento a `5 %` e **`1,074 ms`** a `2 %`, contra o *kill* de
/// `8 ms` — `13 %` do orçamento. ⚠️ **Ele NÃO cresce só com os dabs:** a
/// granularidade dos candidatos é proporcional ao passo, logo um espaçamento
/// `2,5×` mais fino pede também `2,5×` mais RAIOS contra a superfície congelada.
/// A sonda que o mede é a `diag_o_custo_do_traco_aos_dois_espacamentos`.
///
/// ⛔⛔ **Isto é uma DIVERGÊNCIA DECLARADA do valor de fábrica do alvo**, e o
/// corpus de paridade continua a medir-se com o número DELE: o que diverge é o
/// que o pincel VESTE ao nascer, nunca a lei que ele corre.
pub const ESPACAMENTO_DO_AFIADO_PCT: f32 = 2.0;

/// ⛔⛔ **A ORDEM DO DONO, como erro de COMPILAÇÃO** — reduzir o espaçamento só é
/// uma decisão enquanto o nosso número for menor que o do alvo, e um `assert!` em
/// teste sobre duas constantes é dobrado pelo compilador antes de correr (o
/// clippy di-lo: *«this assertion has a constant value»*). ⇒ a lei mora aqui, ao
/// lado dos dois números, e quem os igualar **não compila**.
const _: () = assert!(
    ESPACAMENTO_DO_AFIADO_PCT < ESPACAMENTO_DO_AFIADO_DO_ALVO_PCT,
    "o dono mandou REDUZIR o espaçamento do afiado (16/09)"
);

/// **O espaçamento declarado por este verbo**, em % do diâmetro — `None` quando o verbo não
/// declara nenhum e cai no passo da casa.
///
/// ⚠️ **Uma porta só, lida pelo passo E pela atenuação**: as duas perguntas partilham o
/// número, e escrevê-lo duas vezes faria um traço com o passo de um pincel e a atenuação de
/// outro — que é precisamente a composição que nenhuma referência declara.
#[must_use]
pub fn espacamento_do_verbo(verb: Verb) -> Option<f32> {
    match verb {
        Verb::Plane => Some(ESPACAMENTO_DO_PLANO_PCT),
        Verb::DrawSharp => Some(ESPACAMENTO_DO_AFIADO_PCT),
        _ => None,
    }
}

/// **O passo entre dois dabs de um traço arrastado**, na régua em que `raio` vem (a app dá pixels).
#[must_use]
pub fn passo_do_traco(pincel: &Brush, raio: f32) -> f32 {
    match espacamento_do_traco(pincel) {
        Some(pct) => passo_de_um_espacamento(pct, raio),
        None => crate::min_spacing(raio),
    }
}

/// ⭐⭐ **O espaçamento que ESTE traço corre** — o do pincel quando ele o declara,
/// senão o do verbo.
///
/// ⚠️ **Uma porta só, lida pelos DOIS passos E pela atenuação.** Escrita duas
/// vezes, um traço andaria com o espaçamento de um pincel e enfraqueceria com o
/// de outro — e foi exactamente isso que aconteceu na primeira tentativa de
/// baixar o nosso: a bancada passou a arrastar a `5 %` e o motor continuou a
/// atenuar a `2 %`, com seis gates de paridade a acusar a LEI por causa de um
/// número.
#[must_use]
pub fn espacamento_do_traco(pincel: &Brush) -> Option<f32> {
    pincel
        .espacamento_pct
        .or_else(|| espacamento_do_verbo(pincel.verb))
}

/// ⭐⭐ **A LEI do passo, com o espaçamento explícito** — `pct` é do DIÂMETRO,
/// logo o passo é `2·raio·pct/100`.
///
/// ⚠️ **Ela existe porque há TRÊS chamadores e um deles não usa o nosso número:**
/// o passo de ecrã, o passo de mundo, e a **bancada de paridade**, que tem de
/// arrastar com o espaçamento do ALVO (o do cabeçalho da fixtura) desde que o
/// dono mandou baixar o nosso. *Escrita três vezes, esta aritmética divergiria
/// no dia em que uma delas ganhasse um factor.*
#[must_use]
pub fn passo_de_um_espacamento(espacamento_pct: f32, raio: f32) -> f32 {
    raio * espacamento_pct / 50.0
}

/// **O factor de atenuação `a`** de uma curva a um espaçamento (em % do diâmetro). `1` fora de
/// `0 < espaçamento < 100`, onde a lei não existe.
#[must_use]
pub fn atenuacao_por_espacamento(curva: Falloff, espacamento_pct: f32) -> f32 {
    if !(espacamento_pct > 0.0 && espacamento_pct < 100.0) {
        return 1.0;
    }
    let h = espacamento_pct / 50.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let vizinhos = (100.0 / espacamento_pct).floor() as u32;
    let mut maior = 0.0f32;
    for k in 0..10u8 {
        let fase = f32::from(k) / 10.0;
        let mut soma = 0.0f32;
        for j in 0..vizinhos {
            #[allow(clippy::cast_precision_loss)]
            let x = (fase - 1.0 + j as f32 * h).abs();
            if x < 1.0 {
                soma += curva.weight(x);
            }
        }
        maior = maior.max(soma);
    }
    if maior > 0.0 { maior.recip() } else { 1.0 }
}

impl Brush {
    /// **O factor por dab do traço** — `1` fora do arrasto e em todo verbo que não o declare.
    ///
    /// ⚠️ Calcula-se UMA vez por dab (o plano guarda-o no plano da pegada; o afiado entra pelo
    /// [`Brush::reach`], que também corre uma vez por dab): são `140` avaliações da curva, e o
    /// laço por-vértice não as pode pagar a cada vértice.
    #[must_use]
    pub fn factor_do_traco(&self) -> f32 {
        let Some(pct) = espacamento_do_traco(self) else {
            return 1.0;
        };
        if !self.traco_arrastado {
            return 1.0;
        }
        let a = atenuacao_por_espacamento(self.falloff, pct);
        // ⭐⭐⭐ **O PINCEL AFIADO usa `a` CRU, e o de plano `(1 + a)/2`** — são
        // duas leis e não um arredondamento: medido, o traço do afiado com `a`
        // reproduz o alvo a `4,9e-4` e com a lei do plano erra `4,7e-2`, **`96×`**
        // pior (espec do afiado §5.3). Dar-lhe a lei do vizinho entregaria um
        // vinco `2,5×` mais forte por dab.
        //
        // ⚠️ *Duas leis com a mesma forma são a coisa mais fácil de unificar por
        // engano* — e é por isso que o corpus de cada um mede a do outro.
        if self.verb == Verb::DrawSharp {
            return a;
        }
        let afastar_invertido = self.plano_inversao == PlanoInversao::Afastar
            && self.invert
            && self.verb.honours_invert();
        if afastar_invertido {
            0.5 * a
        } else {
            (1.0 + a) / 2.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{atenuacao_por_espacamento, passo_do_traco};
    use crate::{Brush, Falloff, Verb};

    /// ⭐ **G-17, a metade da LEI** — a tabela da espec §14.4, ao milésimo que ela publica.
    #[test]
    fn a_atenuacao_do_traco_e_a_da_lei() {
        for (curva, pct, a) in [
            (Falloff::Smooth, 7.0, 0.140),
            (Falloff::Smooth, 10.0, 0.200),
            (Falloff::Constant, 7.0, 0.071),
        ] {
            let lida = atenuacao_por_espacamento(curva, pct);
            assert!(
                (lida - a).abs() < 5e-4,
                "{curva:?} a {pct} %: {lida} contra {a}"
            );
        }
        assert_eq!(atenuacao_por_espacamento(Falloff::Smooth, 100.0), 1.0);
        let pincel = Brush {
            verb: Verb::Plane,
            falloff: Falloff::Smooth,
            traco_arrastado: true,
            ..Brush::default()
        };
        assert!(
            (pincel.factor_do_traco() - 0.570).abs() < 5e-4,
            "{}",
            pincel.factor_do_traco()
        );
        let por_script = Brush {
            traco_arrastado: false,
            ..pincel.clone()
        };
        assert_eq!(
            por_script.factor_do_traco(),
            1.0,
            "a porta por script ganhou atenuacao"
        );
        let outro = Brush {
            verb: Verb::Draw,
            ..pincel
        };
        assert_eq!(
            outro.factor_do_traco(),
            1.0,
            "a lei do plano chegou a outro verbo"
        );
    }

    /// O passo do plano é `7 %` do diâmetro, EXACTO a 50 px; os outros verbos ficam nos `0,15 R`.
    #[test]
    fn o_passo_do_plano_e_sete_por_cento_do_diametro() {
        let plano = Brush {
            verb: Verb::Plane,
            ..Brush::default()
        };
        assert_eq!(passo_do_traco(&plano, 50.0), 7.0);
        let liso = Brush {
            verb: Verb::Draw,
            ..Brush::default()
        };
        assert_eq!(passo_do_traco(&liso, 50.0), crate::min_spacing(50.0));
    }
}
