//! **A PARALAXE — um número por objecto** (plano [24](../../../docs/Components/24_plano_paralaxe.md), W1).
//!
//! # A lei, e porque ela é UM número e não dois
//!
//! Numa câmara pinhole com o plano focal (onde o artista desenha, e que mapeia 1:1) à distância
//! `z₀`, e uma camada à distância `z`:
//!
//! ```text
//! projecção          u  = f·x / z
//! a câmara anda Δ    Δu = −f·Δ / z        no plano focal:  Δu₀ = −f·Δ / z₀
//! ──────────────────────────────────────────────────────────────────────────
//! paralaxe           Δu / Δu₀            =  z₀ / z  ≡  k
//! tamanho aparente   (f·s/z) / (f·s/z₀)  =  z₀ / z  ≡  k      ← O MESMO NÚMERO
//! ```
//!
//! ⭐⭐⭐ **A fracção de paralaxe e o factor de escala são a MESMA grandeza.** É por isso que o
//! modelo «fracção» (Godot · Phaser · Construct) e o modelo «multiplano» (Disney · OpenToonz ·
//! After Effects) não são alternativas: o segundo é o primeiro mais uma linha, e essa linha é a W5.
//! ⛔ **Guardar a DISTÂNCIA ao lado da fracção seria guardar o mesmo número duas vezes** — dois
//! campos que têm de concordar é o defeito que esta casa já pagou; a distância é uma LEITURA
//! (`z = z₀/k`).
//!
//! # A referência é a ORIGEM DO MUNDO, e isso é uma decisão
//!
//! A pose escrita é `autorada + centro_da_câmara · (1 − k)`, logo com a câmara na origem **nada se
//! mexe**: o artista põe o fundo onde ele deve estar *quando a câmara está em zero*.
//!
//! ⚠️ **A alternativa era guardar a posição da câmara no momento em que o objecto foi autorado** —
//! e ela custa ESTADO, que teria de sobreviver ao ficheiro, ao `Ctrl+Z` e ao rebobinar. A origem
//! do mundo não custa nada e **é o que o alvo faz** (medido: com a câmara em `0` a camada dele
//! está na base dela, e o declive é exactamente `1 − scroll_scale`).
//!
//! ⛔ **E não se pode escrever `autorada + (centro − autorada)·(1 − k)`**, que é a forma do
//! multiplano do Flip: ali todos os planos estão ancorados na origem do objecto e coincidem
//! enquadrados de frente, e aqui isso **TELEPORTARIA** para o centro da vista um fundo que o
//! artista pôs a um canto.
//!
//! # O que este componente NÃO tem
//!
//! ⛔ **Nenhum campo `enabled`**: `k = 1` é o objecto normal do mundo e não escreve um bit — a
//! ausência de efeito é o valor de fábrica, não um interruptor a mais.
//! ⛔ **Nenhum `offset` próprio**: a posição é a do `Transform`, que já existe.
//! ⛔ **Nenhum par de interruptores à maneira do alvo** (`follow_viewport`/`ignore_camera_scroll`):
//! medido, eles dão **4 combinações para 3 comportamentos**, com um estado duplicado e um sinal
//! invertido — e um `k` negativo exprime a inversão sem uma caixa.

use crate::SimComponent;
use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

/// **Quanto do movimento do mundo este objecto guarda.**
///
/// | `k` | o que é |
/// |---|---|
/// | `1` | objecto normal — está no plano em que o artista desenha (**o de fábrica**) |
/// | `0` | infinitamente longe ⇒ **preso à vista** (é o que um HUD é) |
/// | `0 < k < 1` | fundo: fica para trás |
/// | `k > 1` | primeiro plano: passa à frente |
/// | `k < 0` | anda ao contrário — legítimo, e é o que o alvo esconde atrás de um interruptor |
///
/// ⚠️ **Por EIXO**, e não um escalar: é a lei do alvo (medida — `scroll_scale` é um `Vector2`) e é
/// o caso das nuvens que correm de lado e mal sobem.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScrollFactor {
    /// `[x, y]` — ver a tabela do tipo.
    pub k: [f32; 2],
}

impl SimComponent for ScrollFactor {}

impl Default for ScrollFactor {
    /// ⭐ **`[1, 1]` — o objecto do mundo.** Anexar o componente e não lhe tocar tem de dar uma cena
    /// **byte-idêntica**, e é por isso que o valor de fábrica é o neutro da lei e não um fundo
    /// bonito: *tudo o que é novo shipa desligado*.
    fn default() -> Self {
        Self { k: [1.0, 1.0] }
    }
}

impl ScrollFactor {
    /// O neutro, escrito uma vez para os gates não o repetirem.
    pub const NEUTRO: [f32; 2] = [1.0, 1.0];

    /// **Este objecto não desloca nada?** ⚠️ A comparação é **exacta e é de propósito**: o que ela
    /// decide é *«escrevo no `Transform` ou não toco nele?»*, e um epsilon aqui faria um `k` quase
    /// neutro deixar de ser conduzido — o objecto ficaria parado com o painel a dizer que não está.
    #[must_use]
    pub fn e_neutro(&self) -> bool {
        self.k == Self::NEUTRO
    }

    /// ⭐⭐⭐ **O DESLOCAMENTO sozinho** — `centro · (1 − k)`, sem a pose.
    ///
    /// ⚠️ **Ele existe porque a REPETIÇÃO envolve o deslocamento e não a posição** (a W2 do plano
    /// 24): a lei dela é *«corrige por um número INTEIRO de ladrilhos»*, e um ladrilho é uma
    /// grandeza do deslocamento. Somar a pose primeiro e envolver a soma envolveria também a
    /// posição que o artista autorou — o fundo saltaria para a origem assim que ele o arrastasse
    /// para além de meio ladrilho.
    ///
    /// ⭐ **E a `desloca` DELEGA-lhe**, e não o contrário: escrita duas vezes, o dia em que a lei
    /// mudasse deixava a repetição a corrigir um deslocamento que já não é o que o produto aplica.
    #[must_use]
    pub fn deslocamento(&self, centro: [f32; 2]) -> [f32; 2] {
        [
            centro[0] * (1.0 - self.k[0]),
            centro[1] * (1.0 - self.k[1]),
        ]
    }
}
