//! **A FOLHA COMO OBJETO** — criar uma, e arranjar as peças dentro dela.
//!
//! O desenho está no plano [`docs/Sprite_projeto/17`] §7, e a decisão que o gera é do Enio: a
//! folha não é uma prévia dentro de um painel, é **uma «imagem virtual» no canvas** — área
//! transparente, gizmo de sprite, sombra decorativa, na hierarquia, escondível, redimensionável,
//! movível, duplicável.
//!
//! ## Quase nada disto é código deste módulo
//!
//! A folha é um **retângulo vivo que ganhou um componente**, exatamente como a moldura
//! (`ph2d_ecs::VecFrame`). Por ser um retângulo, o fill, o traço, a pilha de efeitos (o **Drop
//! Shadow** que faz a sombra), o gizmo, o hit-test, o z-order, o undo e o save **já existem**.
//! Este módulo só faz duas coisas: **nascer** e **arranjar**.
//!
//! ## E arrastar uma peça também não é código deste módulo
//!
//! As peças são **filhos** da folha, então mover uma é mover um filho — com o gizmo, o snap e o
//! undo que o app já tem. *A representação apaga o caso especial.* O auto-arranjo apenas
//! **propõe** poses; quem decide é o artista, com o mouse.
//!
//! ## ⚠️ O RE-arranjo ainda não está aqui, e é decisão, não esquecimento
//!
//! O verbo *"arranjar outra vez"* (depois de o artista acrescentar, tirar ou redimensionar uma
//! peça) precisa de um **botão**, e o botão precisa da ferramenta de imagem que o plano §7.5
//! nomeia. Escrever a função agora e ligá-la depois é exatamente o *"armar e fiar depois"* que a
//! DIRETIVA proíbe — e é a doença que esta linha inteira existe para curar: a seção Render Source
//! tinha três controles mortos por esse mecanismo. Ela volta **no mesmo commit** que o botão.

use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, SpriteSheetFrame, Transform, VecShape};
use ph2d_render::Sprite;
use ph2d_sprite_sheet::{Layout, LayoutItem, PackError, PackOptions};
use ph2d_vec_scene::{ShapeKind, VecScene};

use crate::vec_entities::VecEntityMap;

/// O teto de folha que este módulo assume até alguém passar o do dispositivo.
///
/// ⚠️ É o **mínimo garantido pela especificação do WebGPU** em qualquer adaptador
/// (`max_texture_dimension_2d`), e não um número escolhido: uma folha maior que isto pode não
/// subir na máquina de outra pessoa, e o modo de falha seria no projeto dela.
const SHEET_MAX_SIDE: u32 = 8192;

/// Cria uma folha a partir dos sprites selecionados: o retângulo nasce, eles viram filhos dele, e
/// o arranjo automático já os coloca. Devolve os bits da folha.
///
/// `None` quando a seleção não tem sprite nenhum, ou quando o arranjo não é possível (a razão
/// sobe pelo `Err` de [`arrange_children`], que o chamador toasta).
pub(crate) fn create_from_selection(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    selection: &[u64],
    pixels_per_meter: f32,
) -> Result<u64, SheetFrameError> {
    let pieces = collect_pieces(sim, selection);
    if pieces.is_empty() {
        return Err(SheetFrameError::NoSprites);
    }
    // O tamanho da folha sai do ARRANJO, não de um palpite: empacota-se primeiro para saber de
    // quanto espaço se precisa, e o retângulo nasce já com essa medida.
    let frame = SpriteSheetFrame::at_density(pixels_per_meter);
    let plan = plan_for(&pieces, &frame)?;
    let side_m = plan.size as f32 / density(&frame);

    // O centro da seleção — a folha nasce onde o artista estava a olhar.
    let center = selection_center(&pieces);
    let entity = spawn_rect(sim, scene, map, center, side_m)?;
    let name = crate::name_unique::unique_name(sim, "Sprite Sheet");
    if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
        e.insert(frame);
        e.insert(Name::new(name));
    }
    // As peças viram filhos ANTES de serem colocadas: `place` escreve poses LOCAIS, e uma pose
    // local sobre um pai que ainda não é pai seria interpretada como mundo — o clássico salto
    // para o canto no primeiro frame.
    for p in &pieces {
        if let Ok(mut e) = sim.world_mut().get_entity_mut(p.entity) {
            e.insert(ChildOf(entity));
        }
    }
    place(sim, &pieces, &plan, &frame, side_m);
    Ok(entity.to_bits())
}

/// Por que uma folha não pôde nascer ou ser arranjada.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SheetFrameError {
    /// A seleção (ou a folha) não tem sprite nenhum.
    NoSprites,
    /// O retângulo não pôde ser criado no documento vetorial.
    ShapeFailed,
    /// O empacotador recusou — a razão dele, verbatim.
    Pack(PackError),
}

impl std::fmt::Display for SheetFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSprites => write!(f, "select at least one sprite first"),
            Self::ShapeFailed => write!(f, "the sheet rectangle could not be created"),
            Self::Pack(e) => write!(f, "{e}"),
        }
    }
}

impl From<PackError> for SheetFrameError {
    fn from(e: PackError) -> Self {
        Self::Pack(e)
    }
}

/// Uma peça candidata: a entidade, o nome e o tamanho EFETIVO em metros.
struct Piece {
    entity: Entity,
    name: String,
    /// `Sprite.size × Transform.scale` — o que o artista de facto vê.
    ///
    /// ⚠️ Usar `Sprite.size` cru arranjaria pela caixa INTRÍNSECA e deixaria uma peça escalada a
    /// sobrepor a vizinha, ou a nadar num buraco — o arranjo tem de concordar com a tela.
    size_m: [f32; 2],
    world: [f32; 2],
}

fn collect_pieces(sim: &SimWorld, bits: &[u64]) -> Vec<Piece> {
    let world = sim.world();
    let mut out = Vec::new();
    for &b in bits {
        let e = Entity::from_bits(b);
        let (Some(sprite), Some(t)) = (world.get::<Sprite>(e), world.get::<Transform>(e)) else {
            continue;
        };
        out.push(Piece {
            entity: e,
            name: world
                .get::<Name>(e)
                .map(|n| n.0.clone())
                .unwrap_or_else(|| format!("piece_{b:x}")),
            size_m: [
                (sprite.size[0] * t.scale.x).abs(),
                (sprite.size[1] * t.scale.y).abs(),
            ],
            world: [t.translation.x, t.translation.y],
        });
    }
    out
}

/// A densidade utilizável — o mesmo piso do `pixels_for`, para as duas contas concordarem.
fn density(cfg: &SpriteSheetFrame) -> f32 {
    if cfg.pixels_per_meter.is_finite() && cfg.pixels_per_meter > 0.0 {
        cfg.pixels_per_meter
    } else {
        1.0
    }
}

/// Arranja as peças em pixels, à densidade da folha.
fn plan_for(pieces: &[Piece], cfg: &SpriteSheetFrame) -> Result<Layout, SheetFrameError> {
    let items: Vec<LayoutItem> = pieces
        .iter()
        .map(|p| LayoutItem {
            name: p.name.clone(),
            width: cfg.pixels_for(p.size_m[0]),
            height: cfg.pixels_for(p.size_m[1]),
        })
        .collect();
    Ok(ph2d_sprite_sheet::layout(
        &items,
        PackOptions {
            padding: cfg.padding,
            max_size: SHEET_MAX_SIDE,
        },
    )?)
}

fn selection_center(pieces: &[Piece]) -> [f32; 2] {
    let (mut sx, mut sy) = (0.0f32, 0.0f32);
    for p in pieces {
        sx += p.world[0];
        sy += p.world[1];
    }
    let n = pieces.len() as f32;
    [sx / n, sy / n]
}

/// Escreve a pose LOCAL de cada peça a partir do arranjo.
///
/// ⚠️ **A conversão de eixo acontece aqui, e uma vez só.** A folha conta pixels com `(0,0)` no
/// canto superior-esquerdo (a convenção do Aseprite, do `region_rect` e do `Asset::ImageRgba8`);
/// o mundo tem `(0,0)` no centro do retângulo e o **`+y` para CIMA**. Espalhar esta inversão por
/// dois sítios é como ela passa a discordar de si própria.
fn place(sim: &mut SimWorld, pieces: &[Piece], plan: &Layout, cfg: &SpriteSheetFrame, side_m: f32) {
    let ppm = density(cfg);
    let half = side_m * 0.5;
    for (i, (_, rect)) in plan.places.iter().enumerate() {
        let Some(p) = pieces.get(i) else { continue };
        let cx_px = rect[0] as f32 + rect[2] as f32 * 0.5;
        let cy_px = rect[1] as f32 + rect[3] as f32 * 0.5;
        let local = ph2d_core::Vec2::new(cx_px / ppm - half, half - cy_px / ppm);
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(p.entity) {
            t.translation = local;
        }
    }
}

/// Cria o retângulo vivo que É a folha, e devolve a entidade dele.
fn spawn_rect(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    center: [f32; 2],
    side_m: f32,
) -> Result<Entity, SheetFrameError> {
    let shape = VecShape::Param {
        kind: ShapeKind::Rectangle.as_u16(),
        w: side_m as f64,
        h: side_m as f64,
        values: [0.0; ph2d_ecs::MAX_SHAPE_VALUES],
    };
    // A MESMA porta que a ferramenta de forma usa para cozinhar a geometria — uma folha desenhada
    // à mão e uma criada por este botão têm de ser o mesmo objeto.
    let path = crate::vec_shape_live::recook_shape(&shape).ok_or(SheetFrameError::ShapeFailed)?;
    let id = scene.push_path(path);
    // O sync é o que dá entidade a um path novo; sem ele o mapa não teria a chave e a folha
    // nasceria como geometria sem objeto — invisível para a hierarquia, o gizmo e o undo.
    crate::vec_entities::sync(sim, scene, map);
    let bits = map.get(&id).copied().ok_or(SheetFrameError::ShapeFailed)?;
    let entity = Entity::from_bits(bits);
    if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
        if let Some(mut t) = e.get_mut::<Transform>() {
            t.translation = ph2d_core::Vec2::new(center[0], center[1]);
        }
        e.insert(shape);
    }
    Ok(entity)
}
