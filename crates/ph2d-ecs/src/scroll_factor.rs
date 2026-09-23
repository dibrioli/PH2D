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
            deslocamento_eixo(self.k[0], centro[0]),
            deslocamento_eixo(self.k[1], centro[1]),
        ]
    }

    /// ⭐⭐⭐ **O DOLLY** (plano 24, W5 · §2) — a câmera anda em PROFUNDIDADE, e o primeiro plano
    /// cresce mais que o fundo. **Nenhum motor 2D tem isto**, e é a razão pela qual a Disney
    /// construiu a câmera multiplano em 1937.
    ///
    /// `delta` é o dolly em **fracções da distância focal** (`δ = d/z₀`), e a saída é o tamanho
    /// aparente desta camada **relativo ao plano focal**:
    ///
    /// ```text
    /// escala(k, δ) = (1 − δ) / (1 − k·δ)
    /// ```
    ///
    /// # ⭐⭐⭐ O `z₀` DESAPARECEU, e isso fecha o bloqueador §6.1 do plano
    ///
    /// O plano exigia medir `z₀` antes de a wave abrir (*«a nossa câmera tem `height_world`, um
    /// zoom, e não uma distância focal»*), e a medição diz que **não há número para escolher**:
    /// `escala` depende só de `k` e de `d/z₀`, logo exprimir o dolly como **fracção** elimina a
    /// grandeza. *Um parâmetro adimensional não tem um default para medir.*
    ///
    /// # ⛔⛔ E a degenerescência que o plano publica está REFUTADA pela fórmula dele
    ///
    /// O §2 diz *«`k = 0` é `z = ∞` ⇒ escala ≡ 1 para todo `d` (o que está infinitamente longe
    /// nunca muda de tamanho)»*. **O parêntesis é verdade e a conclusão não**, e as duas grandezas
    /// partilham o nome: o tamanho **ABSOLUTO** de uma camada infinitamente longe de facto não
    /// muda; a `escala` desta lei é **RELATIVA ao plano focal**, e o plano focal CRESCEU. Medido em
    /// aritmética exacta (`k = 1/10` → `0,5263`, `1/10⁴` → `0,50003`, limite **`1 − δ`**):
    ///
    /// | `k` | `δ = ½` | o que é |
    /// |---|---|---|
    /// | `1` | `1,0000` | a camada do plano focal — ela é a referência, e nunca muda |
    /// | `½` | `0,6667` | um fundo: encolhe relativamente ao plano focal |
    /// | `0` | **`0,5000`** | o céu — absoluto intocado, relativo `1 − δ` |
    ///
    /// ⇒ a lei é escrita e a fórmula fechada **já a contém** (`(1−δ)/(1−0·δ) = 1−δ`): não há um
    /// braço `if k == 0`, e é por isso que ela não pode divergir do limite.
    ///
    /// ⚠️ **Com `δ = 0` a saída é `1,0` ao bit** (`(1−0)/(1−0) = 1/1`), e é isso que faz a omissão
    /// desta wave não mexer num pixel do que a W1 já ship.
    ///
    /// ⛔ **A câmera a ATRAVESSAR a camada devolve `None`**, e é recusa e não clamp: `1 − k·δ ≤ 0`
    /// é `d ≥ z`, a câmera passou para lá do fundo, e não há tamanho aparente nenhum. *Um clamp ali
    /// entregaria um número plausível para uma cena impossível.*
    #[must_use]
    pub fn escala_do_dolly(k: f32, delta: f32) -> Option<f32> {
        if !delta.is_finite() || !k.is_finite() {
            return None;
        }
        let den = 1.0 - k * delta;
        if den <= 0.0 {
            return None;
        }
        let e = (1.0 - delta) / den;
        // ⛔ **`δ ≥ 1` é a câmera NO plano focal (ou além dele)** — o plano do mundo passa a ter
        // tamanho aparente zero (ou negativo), e uma escala `≤ 0` escrita no `Transform` não tem
        // volta: a recuperação do autorado é por RAZÃO, e dividir por zero não devolve nada.
        // Recusa, pela mesma razão da travessia (auditoria 26, §3).
        if !e.is_finite() || e <= 0.0 {
            return None;
        }
        Some(e)
    }

    /// ⭐ **A escala por EIXO desta camada**, ou `None` se a câmera atravessa um dos dois.
    ///
    /// ⚠️ **Os dois eixos podem ter `k` diferentes**, logo podem ter escalas diferentes — e isso é
    /// a lei e não um acidente: uma camada de nuvens que corre em X e mal sobe em Y está, em
    /// profundidade, em dois sítios ao mesmo tempo. *O componente é `[f32; 2]` desde a W1, e a W5
    /// não é o sítio para o estreitar.*
    #[must_use]
    pub fn escala(&self, delta: f32) -> Option<[f32; 2]> {
        Some([
            Self::escala_do_dolly(self.k[0], delta)?,
            Self::escala_do_dolly(self.k[1], delta)?,
        ])
    }

    /// ⭐ **A fracção de paralaxe DEPOIS do dolly** — `k(δ) = k · escala(k, δ)`.
    ///
    /// ⚠️ Ela é derivada da escala e não escrita à parte, porque é a **mesma** razão `z₀/z` do
    /// cabeçalho: escrever as duas deixaria o dia em que uma mudasse com a outra a discordar.
    #[must_use]
    pub fn com_dolly(&self, delta: f32) -> Option<Self> {
        let e = self.escala(delta)?;
        Some(Self {
            k: [self.k[0] * e[0], self.k[1] * e[1]],
        })
    }

    /// ⭐⭐⭐ **O deslocamento com a vista CONFINADA** (plano 24, W3) — o congelamento no ecrã.
    ///
    /// ```text
    /// d = centro·(1 − k)  +  k·(centro − confinado)
    /// ```
    ///
    /// ⚠️ **A forma importa e é MEDIDA:** com `confinado == centro` o segundo termo é `k · 0` e a
    /// soma devolve o primeiro **ao bit** ⇒ toda cena sem limites fica byte-idêntica. ⛔ A forma
    /// equivalente `centro − k·confinado` **não** o seria — `c − k·c` e `c·(1 − k)` diferem por um
    /// ULP em `f32`, e isso mudava o que a W1 e a W2 já shipam.
    ///
    /// ⭐ E o declive de FORA sai sozinho: `(1 − k) + k = 1`, sem um segundo ramo a escrevê-lo.
    /// *Um `if` ali seria a segunda resposta a «a camada congelou?», e ela divergiria no joelho.*
    #[must_use]
    pub fn deslocamento_confinado(&self, centro: [f32; 2], confinado: [f32; 2]) -> [f32; 2] {
        [
            deslocamento_confinado_eixo(self.k[0], centro[0], confinado[0]),
            deslocamento_confinado_eixo(self.k[1], centro[1], confinado[1]),
        ]
    }
}

/// A lei da W1 num eixo — `c·(1 − k)`. ⚠️ Uma função por eixo e não um corpo repetido: a
/// [`ScrollFactor::deslocamento`] e a [`saida_eixo`] leem-na as duas, e duas cópias da mesma conta
/// divergem no dia em que uma delas é corrigida.
#[must_use]
pub fn deslocamento_eixo(k: f32, centro: f32) -> f32 {
    centro * (1.0 - k)
}

/// A lei da W3 num eixo — ver o doc da [`ScrollFactor::deslocamento_confinado`] sobre a forma.
#[must_use]
pub fn deslocamento_confinado_eixo(k: f32, centro: f32, confinado: f32) -> f32 {
    deslocamento_eixo(k, centro) + k * (centro - confinado)
}

/// ⭐⭐⭐ **A SAÍDA de UM EIXO de uma camada — a lei inteira numa função** (auditoria 26, §1.1 e
/// §1.4). A ponte chama-a por eixo; os gates chamam-na sem ledger nenhum.
///
/// ```text
/// saída = c + esc · (autorada + deriva − k·confinado)          (e a repetição envolve o parêntesis)
/// ```
///
/// # ⛔⛔ As duas curas que a auditoria de 23/09 impôs
///
/// 1. **A REPETIÇÃO envolve a posição RELATIVA À VISTA, nunca o deslocamento no mundo.** A 1.ª
///    redacção envolvia `d = c·(1−k)` ⇒ `|d| ≤ tile/2` ⇒ a camada ficava presa a meio ladrilho da
///    pose autorada **no MUNDO**, e uma fileira finita saía do ecrã ao fim de ~30 m (medido na
///    cena `=1`: `5/4/2/0` árvores à vista com a câmera em `0/20/30/60`). O oráculo mostra o
///    contrário: a origem RELATIVA AO ECRÃ fica limitada (`−436 … −564` com a câmera de `0` a
///    `768`). Envolver `deriva − k·confinado` é congruente com o antigo **módulo um ladrilho** —
///    a imagem de um padrão repetido é a mesma — e é o relativo à vista que fica limitado.
/// 2. **O DOLLY escala à volta do CENTRO DA VISTA, nunca do pivô.** A pinhole dá, para cada ponto
///    autorado `P`, `X = c + (P/k − c)·k·esc = c + esc·(P − k·c)` — a antiga translava o pivô por
///    `c·(1 − k·esc)` e escalava em torno dele, errando por `(esc − 1)·P`. Com os filhos da camada
///    escalados pelo mesmo `esc`, o ponto `P + q` cai em `c + esc·(P + q − k·c)` exactamente.
///
/// ⚠️ **Sem repetição e sem dolly o braço é o de SEMPRE, ao bit** (`c·(1−k) + k·(c − conf)`), e
/// é isso que mantém a paridade da W1 com o oráculo e as W3/W4 byte-idênticas. ⚠️ `esc` e `tile`
/// chegam já validados pela lei deles (`escala_do_dolly` recusa o impossível; um ladrilho `≤ 0`
/// ou não-finito é «não repete»).
#[must_use]
pub fn saida_eixo(
    k: f32,
    autorada: f32,
    centro: f32,
    confinado: f32,
    deriva: f32,
    esc: f32,
    tile: Option<f32>,
) -> f32 {
    let repete = tile.is_some_and(|t| t.is_finite() && t > 0.0);
    if esc == 1.0 && !repete {
        return autorada + (deslocamento_confinado_eixo(k, centro, confinado) + deriva);
    }
    let r = deriva - k * confinado;
    let r = match tile {
        Some(t) if repete => crate::envolve_eixo(r, t),
        _ => r,
    };
    centro + esc * (autorada + r)
}

#[cfg(test)]
#[path = "scroll_factor_tests.rs"]
mod tests;
