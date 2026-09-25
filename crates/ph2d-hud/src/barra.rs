//! ⭐⭐⭐ **A BARRA DE VIDA com RASTO ATRASADO** (plano 28, W4) — a lei pura.
//!
//! A barra desenha três faixas: o FUNDO (a vida inteira), a VIDA de agora e, entre as duas, o
//! **rasto** — o pedaço que acabou de se perder, que fica visível um instante e depois escorre até
//! à vida. É o «dano provisório» do *Street Fighter* ([pesquisa 27](../../../docs/Components/27_pesquisa_vida_e_dano.md)
//! §2.1, item 9), e é o que faz um golpe LER-SE: o olho vê quanto saiu antes de o número mudar.
//!
//! # As quatro regras, e a razão de cada uma
//!
//! 1. **Um golpe NOVO recomeça a espera** — enquanto os golpes se sucedem, o rasto segura o valor de
//!    antes do PRIMEIRO; é o que mostra um combo inteiro de uma vez. Um golpe é *a vida descer em
//!    relação à leitura anterior* ([`Rasto::ultimo`]).
//! 2. **Depois da espera, escorre a velocidade CONSTANTE** (fracção da barra por segundo) — nunca
//!    salta para a vida, senão o rasto seria só um atraso.
//! 3. **Uma CURA puxa o rasto para cima na hora** — um rasto abaixo da vida não significa nada, e
//!    um rasto que subisse devagar leria-se como mais dano.
//! 4. **O rasto nunca fica abaixo da vida** — o fim de escorrer é a vida, não zero.
//!
//! ⚠️ **Tudo em FRACÇÃO da vida máxima** (`0..=1`): a lei não sabe que `max` o objecto tem, e é
//! por isso que mudar o `max` a meio não a parte.
//!
//! ⛔ **Sem oráculo:** nenhum motor da pesquisa traz a barra com rasto de fábrica (§5 do plano 28 —
//! *«em todos os outros é um asset ou código»*). As regras são nossas e cada uma tem gate.

/// **O estado do rasto de UMA barra.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rasto {
    /// Onde o rasto está, em fracção da vida máxima.
    pub valor: f32,
    /// A leitura da vida no quadro anterior — é ela que diz se houve um golpe NOVO.
    pub ultimo: f32,
    /// Quanto falta para o rasto começar a escorrer, em segundos.
    pub espera_s: f32,
}

impl Rasto {
    /// **Nasce colado à vida** — sem nada perdido para mostrar. ⚠️ É também o que rebobinar faz:
    /// *rebobinar é renascer*, e um rasto do passado mostraria um golpe que ainda não aconteceu.
    #[must_use]
    pub fn nasce(agora: f32) -> Self {
        let a = fraccao(agora);
        Self {
            valor: a,
            ultimo: a,
            espera_s: 0.0,
        }
    }
}

/// Uma fracção saneada: `NaN` vale `0` (uma vida que não se sabe ler desenha-se VAZIA, nunca
/// cheia — cheia esconderia o defeito), e tudo fica em `0..=1`.
#[must_use]
pub fn fraccao(x: f32) -> f32 {
    if x.is_nan() { 0.0 } else { x.clamp(0.0, 1.0) }
}

/// **Anda o rasto um quadro.**
///
/// `agora` é a vida em fracção; `dt_s` o tempo de JOGO do quadro (zero com o relógio parado — e aí
/// o rasto espera connosco); `atraso_s` quanto ele segura depois de um golpe; `velocidade` quantas
/// barras inteiras escorre por segundo.
///
/// ⚠️ **Um `dt` negativo, não finito ou uma velocidade não finita não andam nada** — um relógio que
/// recua é um REBOBINAR, e quem rebobina chama [`Rasto::nasce`].
#[must_use]
pub fn avanca(r: Rasto, agora: f32, dt_s: f32, atraso_s: f32, velocidade: f32) -> Rasto {
    let agora = fraccao(agora);
    let dt = if dt_s.is_finite() && dt_s > 0.0 {
        dt_s
    } else {
        0.0
    };
    let mut espera = r.espera_s;
    // (1) Um golpe NOVO recomeça a espera.
    if agora < r.ultimo {
        espera = if atraso_s.is_finite() {
            atraso_s.max(0.0)
        } else {
            0.0
        };
    }
    // (3) Uma cura (ou a vida a chegar ao rasto) cola-o à vida — e quem o faz é o `max(agora)` do
    //     fim, SEM ramo próprio. ⚠️ **Um ramo de retorno cedo esteve aqui e a prova de mutação
    //     MATOU-O:** desligá-lo não mudava um bit observável (a espera que ele zerava é sempre
    //     re-armada pelo golpe seguinte), e *uma linha que a mutação não consegue matar não é lei, é
    //     comentário com sintaxe de código*.
    // (2) A espera come o tempo primeiro; o que sobra escorre.
    let sobra = if espera >= dt {
        espera -= dt;
        0.0
    } else {
        let s = dt - espera;
        espera = 0.0;
        s
    };
    let desce = if velocidade.is_finite() {
        velocidade.max(0.0) * sobra
    } else {
        0.0
    };
    Rasto {
        // (4) Nunca abaixo da vida.
        valor: (r.valor - desce).max(agora),
        ultimo: agora,
        espera_s: espera,
    }
}

/// **As três faixas de uma barra**, como `(centro_x, largura)` relativos ao CENTRO da barra —
/// `[fundo, rasto, vida]`, pela ordem em que se desenham.
///
/// ⚠️ **As faixas crescem da ESQUERDA** (a borda esquerda é fixa), que é como toda barra de vida se
/// lê: a vida que se perde some do lado direito.
#[must_use]
pub fn faixas(largura: f32, rasto: f32, agora: f32) -> [(f32, f32); 3] {
    let w = if largura.is_finite() {
        largura.max(0.0)
    } else {
        0.0
    };
    let esquerda = -w * 0.5;
    let faixa = |f: f32| {
        let lw = w * fraccao(f);
        (esquerda + lw * 0.5, lw)
    };
    [(0.0, w), faixa(rasto.max(agora)), faixa(agora)]
}

#[cfg(test)]
#[path = "barra_tests.rs"]
mod tests;
