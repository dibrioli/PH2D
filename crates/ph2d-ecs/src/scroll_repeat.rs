//! **A REPETIÇÃO INFINITA de um fundo de paralaxe** (plano 24, W2) — o deslocamento é corrigido
//! por um número **INTEIRO** de ladrilhos, e é só isso.
//!
//! # ⭐⭐⭐ A lei, MEDIDA no alvo (Godot 4.7.2, MIT, corrido sem interface)
//!
//! Com um ladrilho de `256` e a câmera a varrer `0 → 768` (`k = 0,5`), a origem da camada lê:
//!
//! ```text
//! cam.x 0    origem −436,0
//! cam.x 128  origem −372,0   salto  64,0    ← 0,5 × 128
//! cam.x 256  origem −308,0   salto  64,0
//! cam.x 384  origem   12,0   salto 320,0    ← 64 + 256 = UM ladrilho inteiro
//! cam.x 512  origem   76,0   salto  64,0
//! cam.x 768  origem  204,0   salto  64,0
//! ```
//!
//! ⭐ **A correcção é sempre um múltiplo EXACTO do ladrilho**, e é isso que faz a costura não poder
//! abrir: a imagem a seguir ao salto é a mesma. ⛔ Somar um RESTO faria o erro de `f32` acumular, e
//! ao décimo milésimo ladrilho a costura estaria aberta.
//!
//! # ⛔ Divergência DECLARADA: a JANELA é nossa, a INTEGRALIDADE é a dele
//!
//! A sonda gravou *quando* o alvo corrige (entre `−308` e `−244`) e **não** de onde esse limiar
//! sai — ele depende da escrituração interna do `screen_offset` e do tamanho do viewport, que a
//! sonda não registou. *Um limiar que eu não medi não se porta: copiá-lo seria escrever um número
//! com cara de medição.*
//!
//! ⇒ a nossa janela é **centrada** (`[−t/2, +t/2]`), que é a que não precisa de saber nada sobre o
//! viewport e é a mais estável numericamente (o valor corrigido nunca cresce). O que é PORTADO — e
//! o que importa — é a correcção ser um inteiro de ladrilhos.
//!
//! # ⚠️ E ela envolve o DESLOCAMENTO, nunca a POSIÇÃO
//!
//! Ver o doc da [`crate::ScrollFactor::deslocamento`]: envolver a soma envolveria também a pose que
//! o artista autorou, e o fundo saltaria para a origem assim que ele o arrastasse para além de meio
//! ladrilho.
//!
//! # ⏳ O tamanho do ladrilho é AUTORADO, e a derivação está NOMEADA
//!
//! O plano prometia derivá-lo do conteúdo (*«uma sprite sabe a largura dela»*). ⛔ **Medido: não é
//! alcançável onde a lei corre.** Nesta casa uma sprite é um `SpritePixels(AssetId)` mais um
//! `Transform` — a largura em METROS sai do tamanho do asset e do `pixels_per_meter`, e quem os
//! junta é o EXTRACT, que corre **depois** desta fase (o `RenderInstance::size` só existe do outro
//! lado). *A promessa do plano era sobre um número que ninguém mediu*; a derivação fica como dívida
//! com o mecanismo escrito: ou a fase lê o extract do quadro ANTERIOR (um quadro de atraso sobre
//! uma grandeza constante), ou o extract passa a publicar a caixa antes de desenhar.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

/// **Quanto mede um ladrilho do fundo, em metros.** `0` num eixo = sem repetição nesse eixo.
///
/// ⚠️ **O zero é a ausência e não um erro** — é a mesma convenção do alvo (`repeat_size = 0`), e
/// ela é o que permite repetir só em X, que é o caso de quase todo fundo.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScrollRepeat {
    pub tile: [f32; 2],
}

impl ScrollRepeat {
    /// ⭐ **Corrige um deslocamento por um número INTEIRO de ladrilhos.**
    ///
    /// ⚠️ Um ladrilho não-finito ou `<= 0` deixa o eixo **intocado** — a ausência de repetição é a
    /// omissão, e uma guarda que devolvesse zero apagaria a paralaxe em vez de a não repetir.
    #[must_use]
    pub fn envolve(&self, d: [f32; 2]) -> [f32; 2] {
        [envolve_eixo(d[0], self.tile[0]), envolve_eixo(d[1], self.tile[1])]
    }
}

/// A lei por eixo. ⚠️ `round` e não `floor`: a janela é **centrada**, logo o valor corrigido nunca
/// cresce com a câmera — ver a divergência declarada no cabeçalho.
#[must_use]
pub fn envolve_eixo(d: f32, tile: f32) -> f32 {
    if !(tile > 0.0) || !tile.is_finite() || !d.is_finite() {
        return d;
    }
    d - tile * (d / tile).round()
}
