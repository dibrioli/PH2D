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
    /// O mesmo pivô e as mesmas formas, no idioma do Sprite do motor ([`liga_celulas`]).
    pub celulas: ph2d_ecs::CelulasDaFolha,
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
    let pivos_px: Vec<(f64, f64)> = quadros.iter().map(|&f| crate::pivo_px(sprite, f)).collect();
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
        celulas: celulas_de(sprite, &pivos_px),
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

/// O pivô e as formas por célula da folha `0` (a folha do import é uma célula por quadro, na
/// ordem do ficheiro).
#[allow(clippy::cast_possible_truncation)]
fn celulas_de(sprite: &Sprite, pivos: &[(f64, f64)]) -> ph2d_ecs::CelulasDaFolha {
    let quadros: Vec<_> = sprite.frames().iter().map(|f| f.id).collect();
    ph2d_ecs::CelulasDaFolha {
        celula_px: [sprite.width(), sprite.height()],
        pivos_px: pivos
            .iter()
            .map(|&(x, y)| Some([x as f32, y as f32]))
            .collect(),
        formas: sprite
            .slices()
            .iter()
            .filter(|s| slice_e_collider(s))
            .map(|s| ph2d_ecs::FormaPorCelula {
                nome: s.name.clone(),
                rects_px: quadros
                    .iter()
                    .map(|&f| {
                        let pos = u32::try_from(sprite.frame_index(f)?).ok()?;
                        let b = s.visible_at(pos)?.bounds;
                        Some([b.x as f32, b.y as f32, b.w as f32, b.h as f32])
                    })
                    .collect(),
            })
            .collect(),
    }
}

/// Liga ao Sprite do motor `e` (já com a grelha do import) o pivô e as formas por célula: insere
/// [`ph2d_ecs::SpriteCells`] e, por forma, uma «Collision Shape» filha que a segue
/// ([`crate::SegueCelula`]) — o mesmo gesto do Add Collider do Pixel (corpo `Dynamic`).
pub fn liga_celulas(
    world: &mut ph2d_ecs::World,
    e: ph2d_ecs::Entity,
    celulas: ph2d_ecs::CelulasDaFolha,
    m_por_px: f64,
) -> Vec<ph2d_ecs::Entity> {
    let formas: Vec<(u32, String)> = celulas
        .formas
        .iter()
        .enumerate()
        .map(|(i, f)| (i as u32, f.nome.clone()))
        .collect();
    let cells = ph2d_ecs::SpriteCells {
        folhas: vec![celulas],
    };
    let mut filhas = Vec::new();
    for (i, nome) in formas {
        if world.get::<ph2d_physics_ecs::RigidBody>(e).is_none() {
            world.entity_mut(e).insert(ph2d_physics_ecs::RigidBody {
                kind: ph2d_physics_ecs::BodyKind::Dynamic,
            });
        }
        let c = crate::collider_da_celula(&cells, i, 0, m_por_px).unwrap_or_default();
        let filha = ph2d_physics_ecs::nasce_com_forma(world, e, c.shape);
        let mut f = world.entity_mut(filha);
        if let Some(mut col) = f.get_mut::<Collider>() {
            col.offset = c.offset;
        }
        f.insert((crate::SegueCelula { forma: i }, ph2d_ecs::Name::new(nome)));
        filhas.push(filha);
    }
    world.entity_mut(e).insert(cells);
    filhas
}

/// P14e D2: as slices do documento `doc` que têm collider NO MUNDO — uma filha com [`crate::SegueSlice`]
/// num objecto que mostra esse documento. A marca é derivada daqui, nunca guardada no documento.
#[must_use]
pub fn slices_com_collider(
    world: &mut ph2d_ecs::World,
    doc: u32,
) -> std::collections::BTreeSet<u32> {
    let mut q = world.query::<(&crate::SegueSlice, &ph2d_ecs::ChildOf)>();
    q.iter(world)
        .filter(|(_, pai)| {
            world
                .get::<ph2d_ecs::PixelSprite>(pai.parent())
                .is_some_and(|p| p.0 == doc)
        })
        .map(|(s, _)| s.slice)
        .collect()
}

/// A CÓPIA a gravar: `ph2d.collider` nas slices de `colliders`, e em nenhuma outra. O documento
/// não muda (nenhum passo de undo); a gravação recebe o conjunto por esta cópia e o
/// `ph2d-pixel-io` continua sem conhecer o mundo.
#[must_use]
pub fn com_colliders(sprite: &Sprite, colliders: &std::collections::BTreeSet<u32>) -> Sprite {
    let mut s = sprite.clone();
    let mut lista = s.slices().to_vec();
    let mut mudou = false;
    for c in &mut lista {
        let quer = colliders.contains(&c.id.0);
        if slice_e_collider(c) != quer {
            marca_collider(c, quer);
            mudou = true;
        }
    }
    if mudou {
        // `Op::Slices` só troca a lista (as mesmas slices e ids): não tem como recusar.
        let _ = ph2d_pixel_core::apply(&mut s, ph2d_pixel_core::Op::Slices(lista));
    }
    s
}
