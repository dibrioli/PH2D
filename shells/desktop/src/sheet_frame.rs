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
//! ## O RE-arranjo voltou COM o botão, como prometido
//!
//! ⚠️ O verbo *"arranjar outra vez"* foi escrito, ficou **órfão** (o `dead_code` apanhou-o) e foi
//! **removido** — porque escrever a função e ligá-la depois é o *"armar e fiar depois"* que a
//! DIRETIVA proíbe, e é a doença que esta linha inteira existe para curar (a seção Render Source
//! tinha três controles mortos por esse mecanismo). Ele volta agora, no MESMO commit em que
//! nasce o pill que o chama: clicar `[SHEET]` com uma folha selecionada re-arranja os filhos
//! dela. *Uma função sem chamador não é trabalho adiantado; é código morto com data de validade.*

use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, SpriteSheetFrame, Transform, VecShape};
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::Sprite;
use ph2d_sprite_sheet::{Layout, LayoutItem, PackError, PackOptions};
use ph2d_vec_scene::{ShapeKind, VecScene};

use crate::vec_entities::VecEntityMap;

/// **As folhas que estão entre estes alvos.** Vazio ⇒ o gesto é de CRIAR.
pub(crate) fn sheets_among(sim: &SimWorld, targets: &[u64]) -> Vec<u64> {
    targets
        .iter()
        .copied()
        .filter(|&b| {
            sim.world()
                .get::<SpriteSheetFrame>(Entity::from_bits(b))
                .is_some()
        })
        .collect()
}

/// **RE-ARRANJAR** as folhas dadas, com os toasts. Devolve se houve o que reportar.
///
/// ⚠️ Não sai no primeiro erro: com duas folhas alvo, a segunda tem de ser arranjada na mesma.
/// Reporta-se a PRIMEIRA razão — ela nomeia a peça grande demais, e é isso que diz ao artista o
/// que fazer a seguir.
pub(crate) fn repack_all(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    sheets: &[u64],
    toasts: &mut ToastQueue,
) {
    let mut moved = 0usize;
    let mut failure: Option<String> = None;
    for bits in sheets {
        match arrange_children(sim, scene, *bits) {
            Ok(n) => moved += n,
            Err(e) => {
                failure.get_or_insert_with(|| e.to_string());
            }
        }
    }
    match failure {
        Some(e) => {
            toasts.push(Toast::error(format!("Sheet: {e}")));
        }
        None => {
            toasts.push(Toast::success(format!("Sheet re-packed: {moved} pieces")));
        }
    }
}

/// **A resolução que o encaixe PEDE**, arredondada para cima até uma das oferecidas no modal.
///
/// ⚠️ **Arredondada para CIMA, e é isso que a torna uma sugestão honesta:** o valor cru do
/// empacotador é o mínimo em que aquelas peças cabem, e oferecer o degrau imediatamente abaixo
/// dele seria propor uma folha que já nasce vermelha. Se o encaixe pedir mais do que a maior
/// resolução oferecida, devolve-se a maior — a folha nasce com transbordo e a moldura di-lo, que é
/// melhor do que recusar-se a abrir o modal e não explicar nada.
///
/// `None` quando não há peça nenhuma entre os alvos (o chamador toasta a razão).
pub(crate) fn suggested_size(
    sim: &SimWorld,
    targets: &[u64],
    pixels_per_meter: f32,
) -> Option<u32> {
    let pieces = collect_pieces(sim, targets);
    if pieces.is_empty() {
        return None;
    }
    let cfg = SpriteSheetFrame::at_density(pixels_per_meter);
    // O encaixe natural, sem teto autorado: é ele que sabe de quanto se precisa.
    let needed = plan_for(&pieces, &cfg, SHEET_MAX_SIDE)
        .map(|p| p.size)
        .unwrap_or(SHEET_MAX_SIDE);
    let offered = ph2d_editor::ids::CTX_MENU_SHEET_SIZES;
    Some(
        offered
            .iter()
            .map(|(px, _)| *px)
            .find(|&px| px >= needed)
            .unwrap_or_else(|| offered.iter().map(|(px, _)| *px).max().unwrap_or(needed)),
    )
}

/// **CRIAR** a folha na resolução escolhida, com os toasts. Devolve os bits dela.
pub(crate) fn create_at(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    targets: &[u64],
    pixels_per_meter: f32,
    size_px: u32,
    toasts: &mut ToastQueue,
) -> Option<u64> {
    match create_from_selection(sim, scene, map, targets, pixels_per_meter, size_px) {
        Ok(sheet) => {
            // ⚠️ A contagem é a das peças que de facto entraram, lida da árvore — não
            // `targets.len()`. Selecionar três sprites e uma forma vetorial anunciava "4 pieces
            // packed" e mostrava três: *o número tem de vir de onde a coisa aconteceu.*
            let pieces = child_bits(sim, Entity::from_bits(sheet)).len();
            toasts.push(Toast::success(format!(
                "Sheet: {pieces} pieces packed into {size_px} \u{00d7} {size_px}"
            )));
            Some(sheet)
        }
        // ⚠️ A razão sobe VERBATIM do empacotador. Um "não foi possível" mandaria o artista
        // adivinhar entre cem sprites.
        Err(e) => {
            toasts.push(Toast::error(format!("Sheet: {e}")));
            None
        }
    }
}

/// O teto de folha que este módulo assume até alguém passar o do dispositivo.
///
/// ⚠️ É o **mínimo garantido pela especificação do WebGPU** em qualquer adaptador
/// (`max_texture_dimension_2d`), e não um número escolhido: uma folha maior que isto pode não
/// subir na máquina de outra pessoa, e o modo de falha seria no projeto dela.
const SHEET_MAX_SIDE: u32 = 8192;

/// Cria uma folha a partir dos sprites selecionados, **na resolução que o artista escolheu**: o
/// retângulo nasce com `size_px` de lado, eles viram filhos dele, e o arranjo coloca-os.
///
/// ⚠️ **O tamanho passa a ser AUTORADO** (Enio 2026-08-19: *"Ao criar uma sheet um modal com a
/// resolução deve aparecer antes da criação"*). Ele saía do arranjo — o empacotador media e o
/// retângulo nascia com essa medida —, e isso era certo enquanto ninguém tinha opinião. Agora tem:
/// uma folha é um alvo de exportação, e 512×512 é um requisito do projeto, não um resultado.
///
/// ⚠️ **E por isso a folha pode nascer APERTADA.** Se as peças não couberem em `size_px`, o
/// encaixe é refeito sem teto e as peças assentam a partir do canto superior-esquerdo — as que
/// sobram ficam **visivelmente fora** e a moldura acusa (`sheet_bounds::health`). É melhor do que
/// as duas alternativas: recusar a criação deixaria o artista sem nada e sem saber quanto falta;
/// crescer a folha em silêncio apagaria a escolha que ele acabou de fazer.
pub(crate) fn create_from_selection(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    selection: &[u64],
    pixels_per_meter: f32,
    size_px: u32,
) -> Result<u64, SheetFrameError> {
    let pieces = collect_pieces(sim, selection);
    if pieces.is_empty() {
        return Err(SheetFrameError::NoSprites);
    }
    let frame = SpriteSheetFrame::at_density(pixels_per_meter);
    let plan = plan_within(&pieces, &frame, size_px)?;
    let side_m = size_px as f32 / density(&frame);

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

/// Re-arranja os filhos de uma folha que já existe — o `[SHEET]` clicado sobre a própria folha.
///
/// ⚠️ **Isto escreve `Transform`, e não viola a lei do [ADR-0153]** (*"o passe publica onde as
/// coisas ficam; ele não escreve onde elas estão"*). Aquela proíbe um passe **por-quadro** de
/// tocar poses — senão cada quadro de um resize vira um passo de undo. Aqui é **um clique**: uma
/// edição autorada, um passo de undo. A distinção é *por-quadro vs. por-gesto*.
///
/// ⚠️ **Ele NÃO redimensiona a folha, e isto mudou em 2026-08-19.** Redimensionar era o certo
/// enquanto o tamanho era derivado do arranjo; deixou de ser quando o artista passou a escolhê-lo
/// no modal — re-arranjar apagaria a escolha dele, em silêncio, num gesto que ele pediu para
/// *arrumar*, não para *redimensionar*. Encaixa-se DENTRO da resolução que lá está, e o que não
/// couber acende a moldura.
///
/// [ADR-0153]: ../../../docs/architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md
pub(crate) fn arrange_children(
    sim: &mut SimWorld,
    _scene: &mut VecScene,
    frame_bits: u64,
) -> Result<usize, SheetFrameError> {
    let entity = Entity::from_bits(frame_bits);
    let Some(cfg) = sim.world().get::<SpriteSheetFrame>(entity).copied() else {
        return Err(SheetFrameError::NotASheet);
    };
    let children = child_bits(sim, entity);
    let pieces = collect_pieces(sim, &children);
    if pieces.is_empty() {
        return Err(SheetFrameError::NoSprites);
    }
    // O lado ATUAL da folha, em pixels à densidade dela — é este o teto do encaixe.
    let Some(side_m) = current_side_m(sim, entity) else {
        return Err(SheetFrameError::NotASheet);
    };
    let size_px = cfg.pixels_for(side_m).max(1);
    let plan = plan_within(&pieces, &cfg, size_px)?;
    place(sim, &pieces, &plan, &cfg, side_m);
    Ok(pieces.len())
}

/// O lado atual da folha em metros — do `VecShape`, que **é** o tamanho.
fn current_side_m(sim: &SimWorld, entity: Entity) -> Option<f32> {
    match sim.world().get::<VecShape>(entity)? {
        VecShape::Param { w, .. } => Some((*w as f32).abs()),
        VecShape::Text(_) => None,
    }
}

/// Os filhos diretos de uma entidade, em bits.
fn child_bits(sim: &mut SimWorld, parent: Entity) -> Vec<u64> {
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &ChildOf)>();
    q.iter(world)
        .filter(|(_, c)| c.0 == parent)
        .map(|(e, _)| e.to_bits())
        .collect()
}

// ⚠️ **`resize_frame` VIVEU AQUI e foi removida em 2026-08-19**, quando o tamanho da folha passou
// a ser autorado no modal: ela redimensionava a folha para caber o arranjo, e isso agora apagaria
// a escolha do artista. Ficou órfã no mesmo commit em que perdeu o chamador, e sair é a regra
// desta linha — *uma função sem chamador não é trabalho adiantado; é código morto com data de
// validade*. Se um dia houver um "Fit sheet to contents" explícito, ela volta com o botão, e a
// receita que ela cuidava (o `vec_shape_live::resize_recipe`, que mantém geometria e receita em
// passo) é o que essa segunda tentativa tem de reler: escrever só a geometria faz o
// redimensionamento evaporar na primeira edição de parâmetro.

/// Por que uma folha não pôde nascer ou ser arranjada.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SheetFrameError {
    /// A seleção (ou a folha) não tem sprite nenhum.
    NoSprites,
    /// A entidade clicada não é uma folha.
    NotASheet,
    /// O retângulo não pôde ser criado no documento vetorial.
    ShapeFailed,
    /// O empacotador recusou — a razão dele, verbatim.
    Pack(PackError),
}

impl std::fmt::Display for SheetFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSprites => write!(f, "select at least one sprite first"),
            Self::NotASheet => write!(f, "select a sprite sheet, or sprites to pack"),
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

/// **Encaixa dentro de `size_px`; se não couber, encaixa sem teto.**
///
/// ⚠️ O segundo encaixe não é um remendo: as poses saem do MESMO empacotador e são portanto
/// determinísticas e sem sobreposição — só que a caixa que as contém é maior do que a folha, e é
/// isso que o artista vê e que a moldura nomeia. *Degradar com a mesma lei é diferente de inventar
/// um arranjo de emergência.*
fn plan_within(
    pieces: &[Piece],
    cfg: &SpriteSheetFrame,
    size_px: u32,
) -> Result<Layout, SheetFrameError> {
    match plan_for(pieces, cfg, size_px) {
        Ok(p) => Ok(p),
        Err(_) => plan_for(pieces, cfg, SHEET_MAX_SIDE),
    }
}

/// Arranja as peças em pixels, à densidade da folha, dentro de `max_size`.
fn plan_for(
    pieces: &[Piece],
    cfg: &SpriteSheetFrame,
    max_size: u32,
) -> Result<Layout, SheetFrameError> {
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
            max_size,
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
