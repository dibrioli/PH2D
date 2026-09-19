#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **`ph2d-tween` — «esta propriedade vai de A a B»** (suplente #22,
//! [`docs/Components/21_plano_tween.md`]).
//!
//! # ⛔⛔ O que esta crate NÃO tem, e porquê
//!
//! Ela **não tem relógio** e **não tem curvas**.
//!
//! O relógio é o [`Timer`](../ph2d_ecs/timer) — duração · repetir · `autostart` · o sinal a cada
//! disparo · um `TimerRuntime` cujo `progress()` é **derivado**, que um sinal já arranca
//! (`StartTimer`) e que o `rewind_runtime` já faz **renascer**. É a mesma medição que o **#19**
//! pagou antes de mim (*«um `SequenceRuntime` seria um segundo relógio, e um verbo `PlaySequence`
//! uma segunda maneira de dizer “começa”»*), e ela vale aqui letra por letra.
//!
//! As curvas são o [`ph2d_anim::Easing`] — `11` famílias × `3` modos, com `is_deterministic` a
//! separar os polinomiais dos transcendentais. A linha do levantamento que abre este item escreve
//! a lei da wave inteira: **«presets de 1 clique: açúcar sobre o motor, nunca 2º motor»**.
//!
//! ⇒ o que sobra, e é só isto: **que propriedade, de onde para onde, com que curva — e o que
//! acontece quando o relógio pára.**
//!
//! # ⭐⭐⭐ Porque isto é um COMPONENTE e não uma faixa de timeline
//!
//! A sonda do §5.0 ([`mede_o_que_a_composicao_ja_da_ao_tween`](../ph2d_timeline)) mediu o
//! concorrente a sério — um `SequencePlayer` sobre um container autorado com uma curva de
//! `Opacity` — e ele **faz um fade, exactamente** (`1,0000 · 0,7500 · 0,5000 · 0,2500 · 0,0000`).
//!
//! O que ele **não** faz é alcançar um objecto que **nasceu durante a corrida**: uma ligação de
//! timeline é AUTORADA e nomeia **uma** entidade (medido: o autorado desvanece a `0,5000` e a
//! cópia fica a `1,0000`). E esta linha acabou de shipar a **fábrica** (#11), os **projécteis**
//! (#14) e as **partículas** (#18), que produzem exactamente esses objectos. *Vinte inimigos
//! iguais precisariam de vinte ligações; um componente viaja na cópia.*
//!
//! # A lei, inteira
//!
//! ```
//! use ph2d_tween::{AoAcabar, Canal, Relogio, Tween, valor};
//!
//! let t = Tween::linear(Canal::Opacity, 1.0, 0.0);
//! // A meio de um fade linear, metade.
//! assert_eq!(valor(&t, Relogio::a_correr(0.5)), Some([0.5, 0.0, 0.0, 0.0]));
//! // Antes de alguém o arrancar, ele NÃO ESCREVE NADA.
//! assert_eq!(valor(&t, Relogio::PARADO), None);
//! // E no fim, `Hold` fica onde chegou.
//! assert_eq!(valor(&t, Relogio::ACABOU), Some([0.0, 0.0, 0.0, 0.0]));
//! let r = Tween {
//!     ao_acabar: AoAcabar::Rewind,
//!     ..t
//! };
//! assert_eq!(valor(&r, Relogio::ACABOU), None);
//! ```

use ph2d_anim::{Easing, EasingFamily, EasingMode};
use serde::{Deserialize, Serialize};

mod canal;
pub use canal::Canal;
mod preset;
pub use preset::Preset;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

/// **O que acontece quando o relógio ACABA** — e as duas respostas são precisas, o que é
/// exactamente porque isto é um campo e não um palpite.
///
/// * um **fade-out** quer [`AoAcabar::Hold`]: o objecto desapareceu, e tem de ficar desaparecido;
/// * um **flash** quer [`AoAcabar::Rewind`]: ele piscou, e tem de voltar ao que era.
///
/// ⚠️ **Sem o [`ph2d_ecs::TimerState::finished`] isto seria inexprimível**, e a W0 desta wave
/// existiu para o trazer: um *one-shot* terminado e um que nunca começou eram o **mesmo estado,
/// bit a bit** (o `advance` pára **e zera**), logo o motor não tinha como separar *«ainda não
/// comecei, não escrevas nada»* de *«acabei, fica onde está»*.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AoAcabar {
    /// **Fica onde chegou.** O default, porque é o que um *fade* quer — e um *fade* que reaparece
    /// no quadro em que acaba é o defeito que esta wave existiu para não ter.
    #[default]
    Hold,
    /// **Devolve o valor AUTORADO** — o motor deixa de escrever e o ledger repõe.
    Rewind,
}

impl AoAcabar {
    /// Os dois, em ordem — **a fonte da iteração**. ⚠️ **APPEND-ONLY**: a posição é a tag e ela
    /// viaja no ficheiro (o postcard é posicional).
    pub const ALL: [AoAcabar; 2] = [AoAcabar::Hold, AoAcabar::Rewind];

    /// O rótulo que o artista lê. Inglês, como as irmãs deste catálogo de motores.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            AoAcabar::Hold => "Hold",
            AoAcabar::Rewind => "Rewind",
        }
    }

    /// A posição em [`Self::ALL`] — a tag que o segmentado do painel usa.
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// O valor desta posição, ou o primeiro. ⚠️ **A POSIÇÃO NO ARRAY É A TAG**, e reordenar
    /// [`Self::ALL`] faria um clique escrever outro valor — e compila.
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }
}

/// **O que o tween faz DENTRO de um período** — e é aqui que mora o *ping-pong*.
///
/// ⛔⛔ **Report do dono, 2026-09-19:** *«onde estão as opções úteis como ping-pong?»*. A sonda do
/// §5.0 ([`mede_o_que_a_composicao_ja_da_ao_pingpong`](../ph2d_ecs)) mediu as três saídas que a
/// casa já tinha, e as três dizem NÃO:
///
/// | o que se tentou | o que dá |
/// |---|---|
/// | o `repeat` do relógio | uma **SERRA** — salto de `0,900` no fim do período contra um passo suave de `0,100` |
/// | escolher outra curva | **`0` de `33`** reflectem: todas vão de `0` a `1` e ficam lá |
/// | dois tweens em contrafase no mesmo canal | não se compõem — o segundo escreve por cima, e não há desfasamento a autorar |
///
/// ⭐⭐⭐ **E ele NÃO é um segundo relógio** — é uma **dobra do progresso**, calculada antes da
/// curva: `u → 1 − |2u − 1|`. O relógio continua monótono e continua a publicar o sinal dele uma
/// vez por volta; o que muda é só o que o tween LÊ. *É a mesma propriedade que faz esta crate não
/// ter estado vivo: uma função pura do relógio rebobina de graça.*
///
/// ⚠️ **A dobra vem ANTES da curva**, e é o que faz um *ease* ir e voltar pelo mesmo caminho (o
/// *Yoyo* do idioma corrente). Depois da curva, a ida e a volta teriam formas diferentes.
///
/// ⭐ **Ele também vale sem `repeat`:** um período com [`Ciclo::PingPong`] vai a `para` e volta a
/// `de` — que é uma pulsação, e é útil.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Ciclo {
    /// **Do princípio.** O default — e o que o produto fazia antes de este campo existir, logo o
    /// caminho de omissão é **byte-idêntico**.
    #[default]
    Reinicia,
    /// **Vai e volta.** O progresso é dobrado: `0 → 1 → 0` dentro do mesmo período.
    PingPong,
}

impl Ciclo {
    /// Os dois, em ordem — **a fonte da iteração**. ⚠️ **APPEND-ONLY**: a posição é a tag e ela
    /// viaja no ficheiro (o postcard é posicional).
    pub const ALL: [Ciclo; 2] = [Ciclo::Reinicia, Ciclo::PingPong];

    /// O rótulo que o artista lê. Inglês, como as irmãs deste catálogo de motores.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Ciclo::Reinicia => "Restart",
            Ciclo::PingPong => "Ping-Pong",
        }
    }

    /// A posição em [`Self::ALL`] — a tag que o segmentado do painel usa.
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// O valor desta posição, ou o primeiro.
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }

    /// ⭐⭐⭐ **A DOBRA** — o progresso que o tween de facto lê.
    ///
    /// ⚠️ **`Reinicia` devolve `u` AO BIT** (e não `u * 1.0`, nem um `clamp` a mais): é isso que
    /// torna o caminho de omissão indistinguível do produto que já shipava.
    #[must_use]
    pub fn dobra(self, u: f64) -> f64 {
        match self {
            Ciclo::Reinicia => u,
            // `u = 0 → 0` · `u = ½ → 1` · `u = 1 → 0`, e sem uma transcendental à vista (HR-5).
            Ciclo::PingPong => 1.0 - (2.0 * u - 1.0).abs(),
        }
    }
}

/// **A configuração de um tween** — CONFIG, nunca estado vivo. Ela viaja no ficheiro.
///
/// ⚠️ **`de` e `para` são sempre `[f32; 4]`, e o canal diz quantas componentes contam**
/// ([`Canal::aridade`]). ⛔ A alternativa — um `enum` de valor, ou um componente por espécie —
/// seria uma segunda lei sobre a mesma pergunta: *o painel pinta 1 campo ou 4?* é **derivado** do
/// canal, exactamente como o `SignalVerb::uses_arg` deriva se a linha pinta o argumento.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tween {
    /// Que propriedade ele escreve.
    pub canal: Canal,
    /// O valor em `u = 0`.
    pub de: [f32; 4],
    /// O valor em `u = 1`.
    pub para: [f32; 4],
    /// A forma da curva — o motor do [`ph2d_anim`], nunca uma cópia dele.
    pub easing: Easing,
    /// O que fazer quando o relógio acaba.
    pub ao_acabar: AoAcabar,
    /// O que ele faz DENTRO de um período — ver [`Ciclo`].
    ///
    /// ⚠️ **O campo é o ÚLTIMO da struct de propósito:** o postcard é posicional, logo acrescentar
    /// no fim é a única forma aditiva — e mesmo assim o `PROJECT_SCHEMA` sobe, porque um ficheiro
    /// sem estes bytes tem de ser **recusado em voz alta** em vez de lido a menos.
    pub ciclo: Ciclo,
}

impl Default for Tween {
    fn default() -> Self {
        // ⚠️ **Um fade-out linear**, e não um tween INERTE (`de == para`): um componente acabado
        // de anexar que não faz nada lê-se como partido, que é a lei que o `Timer::default`
        // ("um segundo, e não zero") já escreve para o irmão que o faz correr.
        Self::linear(Canal::Opacity, 1.0, 0.0)
    }
}

impl Tween {
    /// Um tween ESCALAR linear — o construtor que as fixturas e os presets usam.
    #[must_use]
    pub fn linear(canal: Canal, de: f32, para: f32) -> Self {
        Self {
            canal,
            de: [de, 0.0, 0.0, 0.0],
            para: [para, 0.0, 0.0, 0.0],
            easing: Easing::LINEAR,
            ao_acabar: AoAcabar::Hold,
            ciclo: Ciclo::Reinicia,
        }
    }

    /// Um tween de COR — as quatro componentes contam.
    #[must_use]
    pub fn cor(canal: Canal, de: [f32; 4], para: [f32; 4]) -> Self {
        Self {
            canal,
            de,
            para,
            easing: Easing::new(EasingFamily::Quad, EasingMode::Out),
            ao_acabar: AoAcabar::Rewind,
            ciclo: Ciclo::Reinicia,
        }
    }
}

/// **O relógio, como a lei o vê** — três factos, e nenhum tipo do ECS.
///
/// ⚠️ É isto que mantém a crate testável sem um mundo, e a fronteira onde ela está: quem traduz um
/// [`ph2d_ecs::TimerState`] nisto é a ponte.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Relogio {
    /// Está a andar agora.
    pub a_correr: bool,
    /// Já chegou ao fim (o FACTO da W0, não o acontecimento de um tique).
    pub acabou: bool,
    /// Quanto do período já passou, `0,0..=1,0`.
    pub progresso: f32,
}

impl Relogio {
    /// **Parado e no zero** — o que um timer por arrancar tem, e o estado em que a lei **não
    /// escreve nada**.
    pub const PARADO: Self = Self {
        a_correr: false,
        acabou: false,
        progresso: 0.0,
    };
    /// **Chegou ao fim.**
    pub const ACABOU: Self = Self {
        a_correr: false,
        acabou: true,
        progresso: 0.0,
    };

    /// A andar, com este progresso.
    #[must_use]
    pub fn a_correr(progresso: f32) -> Self {
        Self {
            a_correr: true,
            acabou: false,
            progresso,
        }
    }

    /// Parado a meio, com o progresso guardado — o *Pause* do [`ph2d_ecs::timer::stop`].
    #[must_use]
    pub fn em_pausa(progresso: f32) -> Self {
        Self {
            a_correr: false,
            acabou: false,
            progresso,
        }
    }
}

/// ⭐⭐⭐ **A LEI: o que este tween escreve NESTE quadro, ou `None` se não escreve nada.**
///
/// # As quatro respostas, e a que cada uma custa
///
/// | o relógio | o que sai | porquê |
/// |---|---|---|
/// | **a andar** | `lerp(de, para, easing(p))` | o caso normal |
/// | **acabou**, `Hold` | o valor em `u = 1` | *«ele desapareceu e tem de ficar desaparecido»* |
/// | **acabou**, `Rewind` | **nada** | o ledger repõe o autorado — *«ele piscou e voltou»* |
/// | **em pausa a meio** | o valor em `p` | pausar não é acabar: a imagem CONGELA |
/// | **por arrancar** | **nada** | o objecto fica como o artista o autorou |
///
/// ⚠️⚠️ **`None` é «não escreve», e é isso que faz o objecto voltar à cena sem uma linha a
/// repô-lo** — a lei do `em_corrida` do #19, palavra por palavra.
///
/// ⛔ **A FRONTEIRA DECLARADA:** uma pausa em `progresso == 0` é indistinguível de *«nunca
/// arrancou»* (o [`ph2d_ecs::timer::stop`] não mexe no `elapsed_us`, e os dois leem `0`), e as duas
/// devolvem `None`. Ela é inofensiva por construção — no instante `0` nada se mexeu ainda — e é
/// preferível ao contrário: escrever `de` sobre um objecto que o artista nunca mandou animar seria
/// a ferramenta a mudar a cena sozinha.
///
/// ⚠️ **O valor de `u = 1` é `easing(1)` e não `1`**, e a diferença não existe por acaso: as `33`
/// curvas do Penner satisfazem `f(1) = 1` por construção (as duas que ultrapassam — `Back` e
/// `Elastic` — voltam exactamente ao fim). Escrever `para` directamente seria uma **segunda
/// resposta** à mesma pergunta, e ela divergiria no dia em que alguém acrescentasse uma família
/// que não fecha.
#[must_use]
pub fn valor(t: &Tween, relogio: Relogio) -> Option<[f32; 4]> {
    let u = if relogio.a_correr {
        f64::from(relogio.progresso.clamp(0.0, 1.0))
    } else if relogio.acabou {
        match t.ao_acabar {
            AoAcabar::Hold => 1.0,
            AoAcabar::Rewind => return None,
        }
    } else if relogio.progresso > 0.0 {
        f64::from(relogio.progresso.clamp(0.0, 1.0))
    } else {
        return None;
    };
    // ⭐⭐⭐ **A DOBRA vem ANTES da curva** — ver [`Ciclo`]. Com `Ciclo::Reinicia` ela devolve `u` ao
    // bit, logo este caminho é indistinguível do que shipava antes de o campo existir.
    Some(mistura(t, t.easing.eval(t.ciclo.dobra(u))))
}

/// A mistura crua, dado o `k` que a curva devolveu.
///
/// ⚠️ **Componente a componente, no espaço do SINK** — a decisão do §6.1 do plano, e ela foi
/// **MEDIDA** (sonda `a_cor_em_dois_espacos`), não argumentada.
///
/// ⭐⭐⭐ **A alternativa era séria e caiu por um mecanismo que eu não previa:** o OKLab com arco
/// curto de matiz — o que o `AnimValue::Color` já usa, e o que o levantamento chama de
/// diferenciador — põe a rampa branco→vermelho **FORA DO GAMUTE**: `r = 1,1099` a meio, com `4`
/// das `9` amostras acima de `1`. E um `tint` **é um multiplicador do texel**, logo um valor acima
/// de `1` não é *«outra cor»*, é o sprite a **CLAREAR** — e limitar a entrada deformaria a curva,
/// que era o ponto inteiro de usar OKLab. *O desvio de `0,2159` entre os dois espaços é real e não
/// é o que decide.*
///
/// ⚠️ **A segunda razão aponta ao mesmo lado:** o canal `Opacity` da timeline já interpola assim
/// (um lerp cru em `tint[3]`), e duas leis de mistura no mesmo componente dariam duas respostas a
/// *«que cor é esta a meio?»*.
///
/// ⚠️ **As componentes acima da aridade viajam a `0,0`** e a ponte não lhes toca: escrever quatro
/// números sobre um canal escalar apagaria três campos que não são dele.
fn mistura(t: &Tween, k: f64) -> [f32; 4] {
    let mut out = [0.0_f32; 4];
    #[allow(clippy::cast_possible_truncation)]
    for (i, o) in out.iter_mut().enumerate().take(t.canal.aridade()) {
        *o = (f64::from(t.de[i]) + (f64::from(t.para[i]) - f64::from(t.de[i])) * k) as f32;
    }
    out
}
