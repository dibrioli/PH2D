//! **Os PRESETS de um clique** (suplente #22) — *Fade In* · *Fade Out* · *Flash*.
//!
//! # ⛔⛔ Eles são AÇÚCAR, e a linha do levantamento que abre este item diz-o por escrito
//!
//! *«presets de 1 clique (lei 8: **açúcar sobre o motor, nunca 2º motor**)»*. ⇒ um preset não tem
//! lei nenhuma: ele **ESCREVE OS CAMPOS** que o artista escreveria à mão, e desaparece. Não há um
//! `Tween::Preset` no componente, não há um braço no `valor`, e o ficheiro gravado é
//! indistinguível de um que alguém tenha afinado a dedo.
//!
//! ⚠️ **E é isso que os torna honestos:** depois de carregar em *Flash*, todos os cinco campos
//! ficam à vista e todos se mexem. *Um preset que guardasse o nome dele seria um modo, e um modo
//! que o artista não pode desmontar é uma caixa preta.*
//!
//! # ⭐⭐ Cada preset escreve TAMBÉM a duração do relógio, e é aí que ele ganha o «um clique»
//!
//! O relógio de um tween é o `Timer` do mesmo índice (a lei do módulo pai). Um preset que só
//! escrevesse os campos deixaria o artista com um *flash* de **um segundo** — o valor de fábrica do
//! timer —, que é seis vezes mais lento do que a coisa que ele acabou de pedir, e ele leria isso
//! como *«o preset não funcionou»*.
//!
//! # ⚠️ As durações são de PRODUTO, e o que as prende é o QUADRO
//!
//! ⛔ Elas não são um tecto de recurso, logo não se medem numa varredura — mas também não são
//! escolhidas no ar: a cerca é a taxa de quadros.
//!
//! | preset | duração | porquê |
//! |---|---|---|
//! | *Flash* | `0,12 s` | **`7,2` quadros a 60 Hz** — abaixo de ~4 quadros um pisca lê-se como um artefacto de desenho, e acima de ~15 deixa de ser um pisca e passa a ser uma mudança de cor |
//! | *Fade In* / *Fade Out* | `0,40 s` | **`24` quadros** — abaixo de ~0,15 s um desvanecer lê-se como um CORTE (a transição não chega a ser vista), e é a mesma fronteira que o `Fade` de qualquer editor de vídeo assume |
//!
//! ⚠️ **E o artista muda-as com um arrasto**, na linha do timer que já está no painel dele.

use crate::{AoAcabar, Canal, Tween};
use ph2d_anim::{Easing, EasingFamily, EasingMode};
use serde::{Deserialize, Serialize};

/// Ver o cabeçalho do módulo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Preset {
    /// O objecto **aparece** — de transparente a opaco.
    #[default]
    FadeIn,
    /// O objecto **desaparece**, e FICA desaparecido.
    FadeOut,
    /// ⭐ A silhueta **pisca** numa cor e **volta à arte** — o *flash de dano*.
    Flash,
}

impl Preset {
    /// Todos, em ordem — **a fonte da iteração**. ⚠️ **APPEND-ONLY**: a posição é a tag, e ela
    /// viaja num clique (o painel manda a posição do chip).
    pub const ALL: [Preset; 3] = [Preset::FadeIn, Preset::FadeOut, Preset::Flash];

    /// O rótulo que o artista lê. Inglês, como as irmãs deste catálogo de motores.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Preset::FadeIn => "tween.preset.fade_in",
            Preset::FadeOut => "tween.preset.fade_out",
            Preset::Flash => "tween.preset.flash",
        }
    }

    /// A posição em [`Self::ALL`].
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// O preset desta posição, ou o primeiro.
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }

    /// **Quanto tempo ele dura, em microssegundos** — ver a tabela do cabeçalho.
    #[must_use]
    pub const fn duracao_us(self) -> u64 {
        match self {
            Preset::Flash => 120_000,
            Preset::FadeIn | Preset::FadeOut => 400_000,
        }
    }

    /// ⭐⭐⭐ **O tween que este preset escreve** — cinco campos, e nada mais.
    ///
    /// ⚠️ **As curvas não são enfeite:** um *fade* com `Quad Out` chega depressa ao quase-nada e
    /// demora-se no fim, que é como o olho lê um desvanecer; um *fade in* com `Quad In` faz o
    /// contrário, e é o que impede a imagem de «saltar» para dentro. O *flash* é **linear** de
    /// propósito — ele é curto demais para uma curva ser vista, e uma ease ali só atrasaria o pico.
    ///
    /// ⚠️ **O `Flash` usa `Silhueta` + `Rewind`, e os dois são obrigatórios:** a arte não volta por
    /// interpolação (um `tint_fill` branco é uma silhueta BRANCA), ela volta por o motor **deixar
    /// de escrever**. ⛔ Com `Hold` o objecto ficaria uma silhueta acesa para sempre — e é essa a
    /// queixa que o painel sabe dizer.
    #[must_use]
    pub fn tween(self) -> Tween {
        match self {
            Preset::FadeIn => Tween {
                easing: Easing::new(EasingFamily::Quad, EasingMode::In),
                ..Tween::linear(Canal::Opacity, 0.0, 1.0)
            },
            Preset::FadeOut => Tween {
                easing: Easing::new(EasingFamily::Quad, EasingMode::Out),
                ..Tween::linear(Canal::Opacity, 1.0, 0.0)
            },
            Preset::Flash => Tween {
                easing: Easing::LINEAR,
                ao_acabar: AoAcabar::Rewind,
                // ⚠️ **Branco no fim, e não «a cor de antes»**: um tween vai de A a B, e o «antes»
                // é o que o LEDGER devolve quando ele deixa de escrever. Pôr a cor autorada aqui
                // seria uma segunda memória dela, e ela envelheceria no dia em que o artista a
                // mudasse.
                ..Tween::cor(Canal::Silhueta, VERMELHO, [1.0, 1.0, 1.0, 1.0])
            },
        }
    }
}

/// A cor do pisca. ⚠️ **Vermelho porque é o que o artista espera de um flash de DANO**, que é o
/// caso que o levantamento nomeia — e ele muda-a com um arrasto, na linha que o painel já mostra.
const VERMELHO: [f32; 4] = [1.0, 0.25, 0.2, 1.0];

#[cfg(test)]
#[path = "preset_tests.rs"]
mod tests;
