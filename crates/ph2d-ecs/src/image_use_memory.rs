//! `ImageUseMemory` — **cada uso da imagem guarda a SUA configuração** (plano GEF 15, dono 27/10: *«Cada modo guarda
//! sua própria configuração»*). Ao sair de Still · Repeated · Animated o objecto guarda aqui o que esse uso tinha
//! (o tamanho, o corte, o ladrilho, as animações); ao voltar, devolve-o tal e qual. Registado: um projecto
//! reaberto e o Desfazer devolvem-no também.
//!
//! O uso VIVO nunca está aqui — está nos componentes do objecto. A lei que escreve e lê vive em
//! `ph2d_app_sprite_anim::image_use`.
//!
//! O 4.º uso, **Region** (plano GEF 16), marca-se pelo [`ImageRegion`]; a área é o [`SpriteRegion`] de sempre
//! (px da imagem), que o render, o retrato, a pele e as ferramentas de píxeis já leem.

use bevy_ecs::component::Component;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    SpriteAnimCursor, SpriteAnimFrames, SpriteAnimations, SpriteAnimator, SpriteGrid, SpriteRegion,
    TextureRepeat, TiledDraw, UvTransform,
};

/// ⭐ **O uso Region** (plano GEF 16): a sprite mostra só uma área da imagem, escolhida no editor de Region. A
/// área é o [`SpriteRegion`] (ausente = a imagem inteira); aqui fica a grelha de snap que o artista escolheu,
/// em colunas × linhas.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageRegion {
    pub grid: [u32; 2],
}

impl Default for ImageRegion {
    fn default() -> Self {
        Self { grid: [1, 1] }
    }
}

/// O que o uso Region tinha quando o artista o deixou: o tamanho, a área e a grelha.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegionLook {
    pub size: [f32; 2],
    pub area: Option<SpriteRegion>,
    pub cut: ImageRegion,
}

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
///
/// ⚠️ **`region` é o ÚLTIMO campo, e lê-se ausente:** o blob de 3 lugares (plano 15, `PROJECT_SCHEMA` 196)
/// acaba antes dele, e o postcard é posicional — ver [`ImageUseMemory::deserialize`].
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize)]
pub struct ImageUseMemory {
    pub still: Option<ImageUseState>,
    pub repeated: Option<ImageUseState>,
    pub animated: Option<ImageUseState>,
    pub region: Option<RegionLook>,
}

impl ImageUseMemory {
    /// Nada guardado.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.still.is_none()
            && self.repeated.is_none()
            && self.animated.is_none()
            && self.region.is_none()
    }

    /// Esquece o aspecto de todos os usos guardados (a imagem mudou); as animações ficam. A área da Region
    /// descrevia a imagem antiga: vai-se toda.
    pub fn forget_looks(&mut self) {
        for s in [&mut self.still, &mut self.repeated, &mut self.animated]
            .into_iter()
            .flatten()
        {
            s.look = None;
        }
        self.region = None;
    }
}

const CAMPOS: &[&str] = &["still", "repeated", "animated", "region"];

/// A forma com nomes (formatos que os escrevem): `region` ausente = nada guardado.
#[derive(Deserialize)]
struct ComNomes {
    still: Option<ImageUseState>,
    repeated: Option<ImageUseState>,
    animated: Option<ImageUseState>,
    #[serde(default)]
    region: Option<RegionLook>,
}

struct Leitor;

impl<'de> Visitor<'de> for Leitor {
    type Value = ImageUseMemory;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("ImageUseMemory")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<ImageUseMemory, A::Error> {
        let mut lugar = |i: usize| -> Result<Option<ImageUseState>, A::Error> {
            seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(i, &self))
        };
        let (still, repeated, animated) = (lugar(0)?, lugar(1)?, lugar(2)?);
        // O blob de 3 lugares acaba aqui: o postcard responde ERRO ao fim dos bytes, não `None`.
        let region = seq.next_element().ok().flatten().flatten();
        Ok(ImageUseMemory {
            still,
            repeated,
            animated,
            region,
        })
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<ImageUseMemory, A::Error> {
        let m = ComNomes::deserialize(de::value::MapAccessDeserializer::new(map))?;
        Ok(ImageUseMemory {
            still: m.still,
            repeated: m.repeated,
            animated: m.animated,
            region: m.region,
        })
    }
}

impl<'de> Deserialize<'de> for ImageUseMemory {
    /// Lê a forma de 4 lugares e a de 3 (sem `region`), a que o `PROJECT_SCHEMA` 196 gravou.
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_struct("ImageUseMemory", CAMPOS, Leitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A forma que o `PROJECT_SCHEMA` 196 gravou: os três primeiros lugares, sem `region`.
    #[derive(Serialize)]
    struct Forma196 {
        still: Option<ImageUseState>,
        repeated: Option<ImageUseState>,
        animated: Option<ImageUseState>,
    }

    fn estado() -> ImageUseState {
        ImageUseState {
            look: Some(ImageUseLook {
                size: [2.0, 1.0],
                grid: None,
                uv: None,
                repeat: None,
                tiled: Some(TiledDraw { tile: [2.0, 1.0] }),
            }),
            ..ImageUseState::default()
        }
    }

    #[test]
    fn a_memoria_de_tres_lugares_le_se_sem_region() {
        let velha = Forma196 {
            still: None,
            repeated: Some(estado()),
            animated: None,
        };
        let bytes = postcard::to_allocvec(&velha).unwrap();
        let lida: ImageUseMemory = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(lida.repeated, Some(estado()));
        assert_eq!(lida.region, None);
    }

    #[test]
    fn a_region_guardada_volta_pelo_postcard_e_pelo_json() {
        let m = ImageUseMemory {
            still: Some(estado()),
            region: Some(RegionLook {
                size: [1.0, 1.0],
                area: Some(SpriteRegion::for_atlas([100.0, 0.0, 100.0, 100.0])),
                cut: ImageRegion { grid: [8, 4] },
            }),
            ..ImageUseMemory::default()
        };
        let p: ImageUseMemory = postcard::from_bytes(&postcard::to_allocvec(&m).unwrap()).unwrap();
        assert_eq!(p, m);
        let j: ImageUseMemory = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
        assert_eq!(j, m);
    }
}
