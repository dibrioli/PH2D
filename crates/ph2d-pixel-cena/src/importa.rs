//! **P14e — o Sprite Inspector abre o que o Pixel grava** (`docs/PixelArt/P14_plano.md` §11 P14e).
//!
//! O import do `.ase` lê o documento pelo MESMO leitor do Pixel (`ph2d_pixel_io::ase::ler`) e
//! deriva tudo pelas MESMAS portas do Pixel Sprite: tags por [`crate::clips_das_tags`], pivô por
//! [`crate::pivo_px`], forma por quadro pela regra do `deriva_formas` (`forma_de_retangulo`),
//! cor e normal por `compose_window`/`compose_normal`. Nenhuma regra é reescrita aqui.

use ph2d_ecs::SpriteAnimations;
use ph2d_physics_ecs::Collider;
use ph2d_pixel_core::user::PropValue;
use ph2d_pixel_core::{Purpose, Slice, SliceId, Sprite, compose_normal, compose_window};

/// A propriedade do mapa do artista (chave `""`) da slice que diz «esta slice é um collider».
pub const COLLIDER_PROP: &str = "ph2d.collider";

/// A slice está marcada como collider (`ph2d.collider = true` no user data)?
#[must_use]
pub fn slice_e_collider(s: &Slice) -> bool {
    s.user
        .props
        .get("")
        .and_then(|m| m.get(COLLIDER_PROP))
        .is_some_and(|v| matches!(v, PropValue::Bool(true)))
}

/// Marca (ou desmarca) a slice como collider, no mapa do artista.
pub fn marca_collider(s: &mut Slice, sim: bool) {
    let m = s.user.props.entry(String::new()).or_default();
    if sim {
        m.insert(COLLIDER_PROP.to_owned(), PropValue::Bool(true));
    } else {
        m.remove(COLLIDER_PROP);
        if m.is_empty() {
            s.user.props.remove("");
        }
    }
}

/// Uma slice collider: a forma (e o offset) em cada quadro; `None` = slice escondida nele (a
/// forma emudece, como `FormaDesligada` no Pixel).
#[derive(Clone, Debug, PartialEq)]
pub struct ColliderImportado {
    pub slice: u32,
    pub nome: String,
    pub por_quadro: Vec<Option<Collider>>,
}

/// O que o import tira de um documento Pixel.
#[derive(Clone, Debug)]
pub struct SpriteImportado {
    pub largura: u32,
    pub altura: u32,
    /// RGBA de alfa reto por quadro (o `Export` do Pixel: sem as camadas Normal Map).
    pub cor: Vec<Vec<u8>>,
    /// A normal por quadro (`compose_normal`), só se alguma camada é Normal Map.
    pub normal: Option<Vec<Vec<u8>>>,
    pub clips: SpriteAnimations,
    /// O pivô (origem) por quadro, em píxeis do sprite.
    pub pivos_px: Vec<(f64, f64)>,
    pub colliders: Vec<ColliderImportado>,
}

/// Deriva o import de um documento Pixel.
#[must_use]
pub fn importa(sprite: &Sprite, nome_sem_tags: &str, m_por_px: f64) -> SpriteImportado {
    let quadros: Vec<_> = sprite.frames().iter().map(|f| f.id).collect();
    let cor = quadros
        .iter()
        .map(|&f| compose_window(sprite, f, None, sprite.bounds(), Purpose::Export))
        .collect();
    let normal = sprite
        .layers()
        .iter()
        .any(|l| l.normal)
        .then(|| quadros.iter().map(|&f| compose_normal(sprite, f)).collect());
    let pivos_px = quadros.iter().map(|&f| crate::pivo_px(sprite, f)).collect();
    let colliders = sprite
        .slices()
        .iter()
        .filter(|s| slice_e_collider(s))
        .map(|s| ColliderImportado {
            slice: s.id.0,
            nome: s.name.clone(),
            por_quadro: quadros
                .iter()
                .map(|&f| crate::forma::collider_visivel(sprite, SliceId(s.id.0), f, m_por_px))
                .collect(),
        })
        .collect();
    SpriteImportado {
        largura: sprite.width(),
        altura: sprite.height(),
        cor,
        normal,
        clips: crate::clips_das_tags(sprite, nome_sem_tags),
        pivos_px,
        colliders,
    }
}

/// Lê um `.ase` pelo leitor do Pixel e deriva o import.
///
/// # Errors
/// O texto do erro do leitor.
pub fn importa_ase(
    bytes: &[u8],
    nome_sem_tags: &str,
    m_por_px: f64,
) -> Result<SpriteImportado, String> {
    let (s, _) = ph2d_pixel_io::ase::ler(bytes).map_err(|e| format!("{e:?}"))?;
    Ok(importa(&s, nome_sem_tags, m_por_px))
}
