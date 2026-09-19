//! ⭐⭐⭐ **A POSE DE REPOUSO DE UM OSSO** — o `rest`/`apply_rest` do `Bone2D` do Godot (MIT), a
//! *Rest Position* do Blender, o *Setup Mode* do Spine.
//!
//! # ⛔⛔⛔ O defeito MEDIDO que ela cura
//!
//! Sem ela, a única coisa que o app sabia fazer a uma pose era mandá-la para a **identidade** — e o
//! menu de contexto da Hierarquia oferecia isso a um osso pelo nome de *Reset Transform*, com uma
//! mensagem **verde** a dizer que correra bem. Medido em 2026-09-19 sobre a cena do osso: repor a
//! transformação do osso do meio movia a arte presa **20,000 unidades num desenho de 60** (um
//! terço), e no osso da raiz o mesmo. *Um osso guarda a DIRECÇÃO dele na `rotation` e a POSIÇÃO na
//! `translation`* — «repor a transformação» de um osso não é a identidade, é a pose em que ele
//! nasceu.
//!
//! # ⚠️ Porque é um COMPONENTE e não um campo do [`crate::Bone`]
//!
//! É a forma do [`crate::BoneLimit`] e da [`crate::IkGoal`], e a razão é a mesma: **a ausência é
//! uma resposta**. Um campo `Option<...>` dentro do osso obrigaria todo `Bone::default()` a
//! escolher um valor, e o valor neutro de uma pose é exactamente a identidade — *o mesmo byte que é
//! o defeito*. Com um componente, um osso sem repouso guardado **não tem** o componente, e o verbo
//! recusa em voz alta em vez de o mandar para a origem.
//!
//! ⭐ E de graça: blob-key própria ⇒ o `PROJECT_SCHEMA` **não** se mexe (o precedente da
//! `PhysicsJoint`/W3, escrito no cabeçalho da escada), e o Inspector mostra-o porque o objecto o
//! TEM (ADR-0166).
//!
//! # ⚠️ Porque ele guarda os SEIS números e não um [`ph2d_ecs::Transform`]
//!
//! Porque o `Transform` tem um **invólucro versionado** para gravar (`TransformVersioned` +
//! `migrate_v1_to_v2`), e o doc dele diz por escrito que é esse o caminho que aguenta ler bytes
//! antigos. Um `Transform` aninhado aqui dentro viajaria **posicionalmente**, fora do invólucro:
//! no dia em que ele ganhasse um campo, todo repouso já gravado seria lido errado — em silêncio,
//! porque o postcard não tem nome de campo para reclamar. ⇒ os números são nossos, e um campo novo
//! aqui é um degrau NOSSO.
//!
//! ⚠️ **`f32` e não `f64`**, ao contrário do resto da geometria do documento: o que se guarda é uma
//! **pose**, e a pose desta casa é `f32`. Repor é então uma cópia **ao bit**, e não uma conversão
//! que erra no último dígito cada vez que o artista carrega no botão.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use ph2d_ecs::{SimComponent, Transform};

/// **A pose em que este osso repousa.** Ausente ⇒ este osso ainda não tem repouso guardado, e os
/// verbos que o pedem recusam em voz alta.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoneRest {
    /// A posição local de repouso.
    pub translation: [f32; 2],
    /// A rotação local de repouso, em radianos — **a direcção do osso**, que é o que o `Reset
    /// Transform` da casa apagava.
    pub rotation: f32,
    /// A escala local de repouso.
    pub scale: [f32; 2],
    /// Os dois cisalhamentos locais de repouso, `[skew_x, skew_y]`.
    ///
    /// ⚠️ **Guardados mesmo sendo patológicos num osso**, e de propósito: o que este componente
    /// promete é *«devolve-me a pose que eu guardei»*, e uma promessa que deixa um dos seis números
    /// de fora devolve outra pose. *Repor é uma cópia, não uma reconstrução.*
    pub skew: [f32; 2],
}

impl SimComponent for BoneRest {}

impl BoneRest {
    /// ⭐ **A ÚNICA leitura de uma pose para um repouso.** Escrita com os seis campos **por nome**,
    /// então um campo novo no [`Transform`] é erro de compilação aqui — que é o ponto de ela
    /// existir uma vez só.
    #[must_use]
    pub fn de(t: &Transform) -> Self {
        Self {
            translation: [t.translation.x, t.translation.y],
            rotation: t.rotation,
            scale: [t.scale.x, t.scale.y],
            skew: [t.skew_x, t.skew_y],
        }
    }

    /// ⭐ **A ÚNICA escrita de um repouso numa pose** — a volta exacta do [`Self::de`].
    ///
    /// ⚠️ Ela escreve os **seis** números e nunca `*t = ...`: um `Transform` que ganhasse um campo
    /// passaria a ser zerado aqui por um valor que este componente nunca guardou. *Repor uma pose
    /// não pode apagar o que o repouso não conhece* — e no dia em que esse campo existir, o
    /// [`Self::de`] deixa de compilar, que é onde a decisão pertence.
    pub fn aplica(self, t: &mut Transform) {
        t.translation.x = self.translation[0];
        t.translation.y = self.translation[1];
        t.rotation = self.rotation;
        t.scale.x = self.scale[0];
        t.scale.y = self.scale[1];
        t.skew_x = self.skew[0];
        t.skew_y = self.skew[1];
    }
}

#[cfg(test)]
#[path = "bone_rest_tests.rs"]
mod tests;
