//! **O CONFINAMENTO de um fundo de paralaxe** (plano 24, W3) — *a borda do fundo nunca entra em
//! cena*.
//!
//! # ⭐⭐⭐ A lei, MEDIDA no alvo, e ela tem DOIS JOELHOS
//!
//! Região `−600..600` (largura `1 200`), ecrã `720`, `k = 0,5`:
//!
//! ```text
//! cam.x  −1200 … −360   declive +1,0000     ← a camada CONGELA no ecrã
//! cam.x   −240 …  +240   declive +0,5000     ← a paralaxe autorada
//! cam.x   +360 … +1200   declive +1,0000     ← congela outra vez
//! ```
//!
//! ⭐ **O joelho está em `|cam| = 240 = (região − ecrã)/2`** — exactamente o ponto em que a borda da
//! **VISTA** alcança a borda da **REGIÃO**. ⇒ *enquanto a vista couber dentro da região, a camada
//! faz a paralaxe escrita; quando a borda da vista toca a da região, a camada passa a andar `1:1`
//! com a câmera*, isto é, **congela no ecrã** — e é por isso que a borda do fundo nunca aparece.
//!
//! ⛔ **Não é uma reescala e não é o `k` que muda:** é um **clamp da VISTA contra um RECTÂNGULO**.
//!
//! ⚠️⚠️ **DUAS leis foram construídas e REFUTADAS antes desta, as duas pelo mesmo defeito de
//! régua:** medir um declive **MÉDIO** sobre uma curva que tem joelhos. *Um clamp não tem um
//! declive; tem PEDAÇOS* — e a média de dois pedaços não é nenhum deles. O gate amostra os
//! pedaços, nunca a média.
//!
//! # ⭐ A composição fica exacta, e é por isso que ela é escrita assim
//!
//! ```text
//! d = centro·(1 − k)  +  k·(centro − confinado)
//! ```
//!
//! Dentro da região `confinado == centro`, o segundo termo é `k · 0` e a soma devolve o primeiro
//! **ao bit** ⇒ toda cena sem limites é byte-idêntica. ⛔ A forma equivalente `centro − k·confinado`
//! **não** o seria: `c − k·c` e `c·(1 − k)` diferem por um ULP em `f32`, e isso mudaria o que a
//! W1 e a W2 já shipam.
//!
//! Fora, o declive é `(1 − k) + k = 1` — o congelamento, sem um segundo ramo a escrevê-lo.
//!
//! # ⏳ A REGIÃO é autorada, e a derivação está NOMEADA (a mesma do ladrilho da W2)
//!
//! O plano prometia derivá-la da caixa do conteúdo. ⛔ **Medido: não é alcançável onde a lei
//! corre** — a caixa em metros de uma sprite sai do asset e do `pixels_per_meter`, e quem os junta
//! é o EXTRACT, que corre **depois** desta fase.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

/// **A região, em metros do mundo, dentro da qual a vista pode passear.**
///
/// ⚠️ **Um eixo é limitado quando `max > min`** — um intervalo vazio não confina nada, e é essa a
/// omissão (`[0,0]`/`[0,0]`). É a mesma convenção do ladrilho zero da [`crate::ScrollRepeat`], e é
/// ela que permite confinar só em Y, que é o caso de um fundo de plataformas.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScrollLimits {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl ScrollLimits {
    /// ⭐ **O centro da vista, confinado à região.**
    ///
    /// `meia_vista` é a meia-largura/meia-altura da vista da câmera do jogo, em metros — é ela que
    /// põe o joelho em `(região − ecrã)/2`, e é a razão de esta wave precisar de um dado que a W1
    /// deliberadamente **não** deixava atravessar.
    ///
    /// ⚠️ **Uma região mais ESTREITA que a vista fixa-a no CENTRO dela.** Sem essa metade o clamp
    /// seria `clamp(c, min+h, max−h)` com o limite de baixo acima do de cima — e o `f32::clamp`
    /// entra em **pânico** ali. *É o mesmo caso que a câmera do jogo já pagou*, com a mesma cura.
    #[must_use]
    pub fn confina(&self, centro: [f32; 2], meia_vista: [f32; 2]) -> [f32; 2] {
        [
            confina_eixo(centro[0], meia_vista[0], self.min[0], self.max[0]),
            confina_eixo(centro[1], meia_vista[1], self.min[1], self.max[1]),
        ]
    }
}

/// A lei por eixo — ver a `⚠️` do [`ScrollLimits::confina`] sobre a região estreita.
#[must_use]
pub fn confina_eixo(c: f32, meia: f32, min: f32, max: f32) -> f32 {
    if !(max > min) || !min.is_finite() || !max.is_finite() || !c.is_finite() {
        return c;
    }
    let meia = if meia.is_finite() { meia.abs() } else { 0.0 };
    let (lo, hi) = (min + meia, max - meia);
    if lo > hi {
        // A região é mais estreita que a vista ⇒ ela CABE inteira, e o sítio honesto é o centro.
        return (min + max) * 0.5;
    }
    c.clamp(lo, hi)
}
