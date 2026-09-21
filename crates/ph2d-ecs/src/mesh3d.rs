//! `Mesh3D` — **o CATAVENTO**: a malha 3D que um sprite mantém VIVA, e a pose dela.
//!
//! A [`crate::BakedForm`] é a **rota A** (`docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md`):
//! a forma é rasterizada **uma vez**, os canais viajam no documento, e a luz relê-os quando o rig
//! muda. Esta é a **rota B**: a malha continua a existir, rasteriza **por quadro** para o G-buffer,
//! e é isso que faz **virar o objecto e a luz acompanhar**.
//!
//! # ⭐⭐⭐ Porque a POSE mora aqui, e não no `Transform`
//!
//! ⛔⛔ **Isto é uma medição, não uma preferência.** O [`crate::Transform`] tem `rotation: f32` e
//! exprime **apenas** a rotação no plano do ecrã — e a §5.0 do catavento mediu que *essa* a rota A
//! já dá **exactamente**: uma rotação `R` em torno do eixo da vista leva as normais a `R·n` e a
//! imagem a `R·imagem`, e as duas são operações 2D sobre o plano já assado (`0,00°` de desacordo de
//! normais, contra `28,58°` de um plano fixo). **Fora do plano não há operação 2D nenhuma**
//! (`31,69°`), porque aparecem faces que não estavam na imagem.
//!
//! ⇒ *a rotação que justifica esta rota é exactamente a que FALTA àquele campo*, logo ela tem de
//! viajar aqui. Tabelas: `docs/Render3d/17_a_rota_b_o_catavento.md` §1.5.
//!
//! # ⛔ A PRESENÇA é a decisão, e não um `bool`
//!
//! O `02.2` diz que a escolha entre as duas rotas é *«uma propriedade do objeto»* — e ela é **ter
//! ou não ter esta componente**. Um campo `live: bool` ao lado seria um segundo sítio a dizer a
//! mesma coisa, e os dois divergiriam no primeiro dia em que alguém escrevesse um sem o outro.
//! ⚠️ Com as duas presentes ganha a **VIVA**: ela é o opt-in explícito, e a assada é o que estava
//! lá antes.
//!
//! # ⛔ O que ela NÃO faz
//!
//! Não põe geometria no ECS — a cena 3D continua dona, exactamente como no
//! [`crate::sculpt_piece_ref`]. E **não** carrega material: a lei que acende é global
//! (`material_da_forma()` não recebe argumentos) e a escolha por objecto que já existe e já é
//! gravada é a `Lei` do assado. *Um `MeshShading { sss, ao, cavity, material }` seria quatro knobs
//! sem consumidor.*

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A malha 3D que este sprite mantém viva, com a orientação dela.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mesh3D {
    /// A peça da cena 3D que doa a forma.
    ///
    /// ⚠️ **Um `u32` cru, e não um tipo do módulo de escultura** — a direcção da seta importa: a
    /// shell conhece os dois, e nenhum dos dois conhece o outro. Idêntico ao
    /// [`crate::Sculpt3dPieceRef`].
    pub piece: u32,
    /// A volta em torno do eixo vertical do ecrã — a pá do catavento a virar.
    pub yaw: f32,
    /// A inclinação para cima e para baixo.
    pub pitch: f32,
}

impl SimComponent for Mesh3D {}
