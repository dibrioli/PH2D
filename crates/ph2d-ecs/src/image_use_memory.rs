//! `ImageUseMemory` — **cada uso da imagem guarda a SUA configuração** (plano GEF 15, dono 27/10: *«Cada modo guarda
//! sua própria configuração»*). Ao sair de Still · Repeated · Animated o objecto guarda aqui o que esse uso tinha
//! (o tamanho, o corte, o ladrilho, as animações); ao voltar, devolve-o tal e qual. Registado: um projecto
//! reaberto e o Desfazer devolvem-no também.
//!
//! O uso VIVO nunca está aqui — está nos componentes do objecto. A lei que escreve e lê vive em
//! `ph2d_app_sprite_anim::image_use`.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::{
    SpriteAnimCursor, SpriteAnimFrames, SpriteAnimations, SpriteAnimator, SpriteGrid,
    TextureRepeat, TiledDraw, UvTransform,
};

/// Como um uso mostra a imagem: o tamanho da sprite e o corte, o UV e o ladrilho.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageUseLook {
    pub size: [f32; 2],
    pub grid: Option<SpriteGrid>,
    pub uv: Option<UvTransform>,
    pub repeat: Option<TextureRepeat>,
    pub tiled: Option<TiledDraw>,
}

/// O que um uso tinha quando o artista o deixou.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImageUseState {
    /// `None` depois de uma troca de imagem: o aspecto descrevia a imagem antiga, e o uso deriva-o de novo.
    pub look: Option<ImageUseLook>,
    pub animations: Option<SpriteAnimations>,
    pub animator: Option<SpriteAnimator>,
    pub frames: Option<SpriteAnimFrames>,
    pub cursor: Option<SpriteAnimCursor>,
}

/// Os usos guardados de um objecto, um por uso que ele NÃO está a usar agora.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImageUseMemory {
    pub still: Option<ImageUseState>,
    pub repeated: Option<ImageUseState>,
    pub animated: Option<ImageUseState>,
}

impl ImageUseMemory {
    /// Nada guardado.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.still.is_none() && self.repeated.is_none() && self.animated.is_none()
    }

    /// Esquece o aspecto de todos os usos guardados (a imagem mudou); as animações ficam.
    pub fn forget_looks(&mut self) {
        for s in [&mut self.still, &mut self.repeated, &mut self.animated]
            .into_iter()
            .flatten()
        {
            s.look = None;
        }
    }
}
