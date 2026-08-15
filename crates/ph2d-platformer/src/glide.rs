//! **PLANAR** (`W-Glide`, plano 08 §4.6) — segurar o pulo na queda faz o
//! personagem descer devagar, para atravessar um vão.
//!
//! # ⚠️ A forma desta lei foi MEDIDA, e não é a que o plano supunha
//!
//! O plano escreveu a suspeita — *"planar é um multiplicador de gravidade sob
//! botão"* — e mandou conferi-la. A sonda
//! (`ph2d-physics-ecs/tests/measure_glide.rs`) **refutou-a**, e achou uma
//! terceira forma que não estava escrita em lugar nenhum. As três candidatas,
//! cada uma sendo uma forma que este módulo já usa algures:
//!
//! | forma | onde vive | o que faz |
//! |---|---|---|
//! | **escala** de gravidade | as fases do pulo (`extra = escala − 1`) | acelera menos |
//! | **alvo** de velocidade | o [`crate::wall_slide`] | **põe** a velocidade |
//! | **TETO** de velocidade | ⚠️ **em lugar nenhum** | põe **só se for mais rápida** |
//!
//! **A escala nunca ASSENTA.** Em toda escala até `0,05` — 5% da gravidade do
//! mundo — a descida continua a crescer: **−1,71 / −2,21 / −2,80 m/s** a 1 / 3 /
//! 6 metros de queda. Então *quão depressa se está a descer* é função de **quanto
//! já se caiu**, e um planeio existe para se saber onde se vai aterrar. Ela
//! entrega o alcance (6,78 → 21,88 m de travessia), mas não a previsibilidade.
//!
//! **O alvo assenta e INVERTE quem sobe.** Apertado a subir a 8 m/s ele impõe
//! `Δv = −10,0 m/s`. Isso não é planar; é um botão de *descer agora*.
//!
//! **O teto assenta e só desacelera** — `Δv = 0` no ápice, `0` a subir, `+10`
//! numa queda rápida. É esta.
//!
//! # ⚠️ Por que o teto não estava escrito, e por que a objeção não viaja
//!
//! O doc do [`crate::wall_slide`] guarda o precedente: **a versão-teto DELE foi
//! morta por medição** — *"com o atrito default o personagem não cai, e um teto
//! nunca dispararia"*. Isso é verdade **da PAREDE**, onde há atrito a segurar o
//! corpo. **No ar livre não há**, então a objeção não viaja — e a forma que ela
//! matou lá é a candidata certa aqui.
//!
//! # ⚠️ A LEI mudou de casa, e o `GlideConfig` ficou
//!
//! O freio vive no [`crate::descent`] desde a `W-Fall`, porque o teto de queda
//! autoraria **um segundo** teto sobre a mesma velocidade — e duas portas a
//! clampar o mesmo número dariam ao planeio o poder de *acelerar* uma queda que
//! o teto já limitou. Aqui ficou o que é do planeio: o número que o artista
//! escreve, e a medição acima que escolheu **teto** entre as três formas.
//!
//! ⚠️ **`glide_motor` não existe mais** — quem quer o freio chama
//! [`crate::descent::descent_ceiling`] e depois
//! [`crate::descent::descent_motor`]. Deixá-lo vivo como uma segunda porta para
//! a mesma pergunta é precisamente o que esta wave removeu.

/// **Como o personagem plana.**
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GlideConfig {
    /// **A velocidade máxima de descida enquanto se plana**, em m/s.
    ///
    /// ⚠️ **Um TETO, não uma velocidade** — e a distinção é a wave inteira (ver
    /// o topo deste módulo): um alvo poria a descida NESTE número mesmo a quem
    /// sobe; o teto só age sobre quem já cai mais depressa. Quem chega ao ápice
    /// com o dedo em baixo não é empurrado.
    ///
    /// `0.0` **desliga** o planeio, e é o idioma dos irmãos `coyote_time` ·
    /// `corner_reach` · `wall_slide_speed` · `air_jumps` · `ledge_grab`: um zero
    /// que significa *"esta assistência não existe"*.
    ///
    /// ⚠️ **Não confundir com `fall_gravity`:** aquele governa **toda** queda e
    /// é uma aceleração; este governa a queda **sob o botão** e é uma
    /// velocidade. A medição que os separa está no topo deste módulo.
    pub fall_speed: f32,
}

impl GlideConfig {
    /// O ponto de partida — **DESLIGADO**.
    ///
    /// ⚠️ **Nasce em zero porque toda capacidade autorada deste módulo nasce**
    /// (a parede, o arranque, o agachar, o nado, o pulo do ar, a beirada): um
    /// projeto salvo antes desta wave reabre exatamente como estava, e o hash de
    /// determinismo (`physics_ecs_c9`) fica byte-idêntico — que é a prova
    /// executável de que o degrau de schema não move física.
    pub const STARTING_POINT: Self = Self { fall_speed: 0.0 };

    /// A assistência está autorada?
    #[must_use]
    pub fn armed(&self) -> bool {
        self.fall_speed > 0.0
    }
}

impl Default for GlideConfig {
    fn default() -> Self {
        Self::STARTING_POINT
    }
}
