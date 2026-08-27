//! ⭐ **O gizmo de um objeto que não desenha nada** — o VAZIO e o GRUPO (Enio, 2026-08-26).
//!
//! Até aqui `snapshots::build_view` respondia `None` para toda entidade sem geometria própria —
//! *«grupo/outro: sem gizmo próprio»* — e um objeto sem gizmo **não é agarrável de forma nenhuma**:
//! nem para mover, nem para girar, nem para escalar. O botão `Add` da Hierarquia (F3) nasce
//! exatamente assim (`Transform` + `Name` e mais nada), então o objeto que o artista acabou de
//! criar era o único do app que ele não podia pegar.
//!
//! # A caixa é a DELE, sempre — e ter filhos não a muda
//!
//! > *«O objeto vazio deve permanecer com seu gizmo original e não se utilizar do gizmo dos
//! > filhos.»* (Enio, 2026-08-26, 3.ª volta do smoke.)
//!
//! ⛔ **A 1.ª versão fazia a caixa ser a UNIÃO dos filhos visíveis** quando havia algum — a lei do
//! container de um `VecEnvelope` (ADR-0129 Fatia 3) generalizada. Está **construída, medida e
//! REJEITADA por veredito de produto**: um objeto vazio passava a ter um tamanho que não é dele, e
//! a caixa mudava sozinha sempre que um filho se mexia. ⚠️ *Um controlo cuja moldura muda quando o
//! artista não lhe tocou lê-se como o app a decidir por ele.* A árvore dessa versão sobrevive no
//! commit `828bc88f4`; ⛔ uma 2.ª tentativa começa perguntando **o que ficou pior**, não
//! reconstruindo.
//!
//! ⇒ a caixa é sempre o **marcador** do objeto, com meia-extensão derivada do tamanho da alça (ver
//! [`EMPTY_HALF_PX`]). Ela move/gira/escala o `Transform` DELE, e os filhos seguem por parentesco —
//! é isso, e não o tamanho da moldura, que faz o conjunto andar como um objeto só.
//!
//! # ⛔ Quem já tem alças NÃO ganha caixa
//!
//! Uma junta e uma roldana também não têm geometria, e chegam aqui pelo mesmo caminho — mas elas
//! **já publicam alças** (os dots de [`crate::render_loop::point_gizmo`]). Uma caixa por cima
//! registaria o interior dela como *Translate* no hit-index e **engoliria o clique nas alças**, que
//! são os controlos que aquelas entidades de facto têm. *É a mesma razão pela qual o conector e o
//! spine de um Blend não publicam gizmo* (ver o cabeçalho de [`crate::vec_gizmo_view`]).

use ph2d_ecs::{Entity, SimWorld, Transform, Visibility};
use ph2d_editor::GizmoView;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

use crate::vec_transform::world_transform;

/// **Meia-extensão do marcador de um objeto VAZIO, em pixels de arte.**
///
/// ⚠️ **O número é DERIVADO da alça, e o recurso é ela**: a caixa carrega oito
/// (`ph2d_editor::HANDLE_SIZE_PX`, hoje `12`) — quatro quinas e quatro meios de aresta. Com
/// meia-extensão de **duas** alças a caixa tem quatro de largura, que é a menor em que a quina e o
/// meio da aresta não se sobrepõem. Abaixo disso o gizmo existe e **não se consegue usar**, que é o
/// «colapsado» do report; o gate [`tests::the_empty_marker_is_wider_than_the_handles_it_carries`]
/// afirma-o pelos dois lados.
///
/// ⚠️ **Pixels de ARTE, não de tela** — convertidos por `pixels_per_meter`, como toda medida
/// geométrica do canvas. Em px de tela o marcador não escalaria com o zoom e a caixa de um objeto
/// vazio seria a única do app que muda de tamanho de mundo quando ninguém lhe toca.
pub(crate) const EMPTY_HALF_PX: f32 = 2.0 * ph2d_editor::HANDLE_SIZE_PX;

/// ⛔ **Quem já tem gizmo próprio** — ver o cabeçalho.
///
/// Duas famílias, e as razões são diferentes:
///
/// - **junta** e **roldana** são PONTOS com dots agarráveis
///   ([`crate::render_loop::point_gizmo`]); uma caixa por cima engoliria o clique neles.
/// - uma peça de **modelagem 3D** tem o gizmo do módulo MODEL — e a pose dela nem sequer é o
///   `Transform` da casa (é o `FieldPose`), então a caixa sairia na origem do mundo, longe da peça.
///
/// ⚠️ **É uma lista, e ela envelhece:** uma família nova que ganhe alças próprias e não venha aqui
/// nasce com duas caixas sobre o mesmo objeto. O gate
/// [`tests::a_joint_publishes_no_box_so_its_dots_keep_the_click`] guarda-a pelos dois lados.
fn publishes_its_own_handles(sim: &SimWorld, e: Entity) -> bool {
    let w = sim.world();
    w.get::<ph2d_physics_ecs::PhysicsJoint>(e).is_some()
        || w.get::<ph2d_physics_ecs::PulleyWheel>(e).is_some()
        || w.get::<ph2d_field_ecs::FieldObject>(e).is_some()
        || w.get::<ph2d_field_ecs::FieldNode>(e).is_some()
}

/// ⭐ **«Este objeto é um VAZIO?»** — a pergunta que TODOS os consumidores fazem.
///
/// Verdadeira para uma entidade que **não desenha nada por si** e cuja pose é o `Transform` da
/// casa: o objeto do botão `Add`, e todo grupo. ⚠️ *Ter filhos não a torna falsa* — Enio,
/// 2026-08-26: *«o gizmo do objeto vazio deve existir mesmo quando ele ganha filhos»*.
///
/// ⛔ **Uma peça de RECEITA responde `false`**: um mestre não está na cena (não emite instância de
/// desenho nenhuma — [`crate::render_loop::off_canvas`]), e um anel sobre ele seria uma marca no
/// canvas para um objeto que não está lá. *O que não se desenha não tem anel.*
pub(crate) fn is_empty_object(sim: &SimWorld, e: Entity) -> bool {
    let w = sim.world();
    w.get::<Transform>(e).is_some()
        && w.get::<ph2d_render::Sprite>(e).is_none()
        && w.get::<ph2d_ecs::VecPathRef>(e).is_none()
        && w.get::<ph2d_ecs::FlipObjectRef>(e).is_none()
        && w.get::<ph2d_ecs::VecEnvelope>(e).is_none()
        && w.get::<ph2d_ecs::MasterPiece>(e).is_none()
        && !publishes_its_own_handles(sim, e)
}

/// ⭐ **O RAIO do anel no mundo** — uma porta, três consumidores (a tinta, o dedo e a caixa).
///
/// ⚠️ **A escala entra pela média geométrica `√|sx·sy|`**, e não por um eixo escolhido à sorte: é a
/// mesma lei que o traço vetorial usa sob escala não-uniforme (`√|det|`, bug #27 do Vector) — para
/// escala uniforme é a própria escala, e é invariante à rotação. Um anel que fosse elipse teria de
/// ser apanhado por um teste de elipse, e o dedo e a tinta discordariam no dia em que um dos dois
/// esquecesse.
pub(crate) fn marker_world_radius(sim: &SimWorld, e: Entity, pixels_per_meter: f32) -> f32 {
    let wt = world_transform(sim, e);
    marker_half(pixels_per_meter) * (wt.scale.x * wt.scale.y).abs().sqrt()
}

/// A meia-extensão do marcador em unidades de mundo, **antes** da escala do objeto.
fn marker_half(pixels_per_meter: f32) -> f32 {
    EMPTY_HALF_PX / pixels_per_meter.max(f32::MIN_POSITIVE)
}

/// **Todo objeto vazio da cena**, em ordem determinística (por identidade, nunca pelos bits de
/// alocação, que o respawn do undo troca).
///
/// ⚠️ **Não é filtrado pela SELEÇÃO** — Enio, 2026-08-26: *«se desseleciono o objeto vazio, o
/// círculo some. O círculo só pode sumir no runtime»*. O anel é o **corpo** de um objeto que não
/// tem pixels; escondê-lo quando ele não está selecionado é a mesma coisa que não o desenhar.
///
/// ⚠️ Um objeto com o olho FECHADO não entra: `Visibility` é per-entidade neste motor, e um objeto
/// que o artista escondeu não está na tela.
pub(crate) fn empty_objects(sim: &SimWorld) -> Vec<Entity> {
    // ⚠️ Pelos ARQUÉTIPOS, e não por uma `query` — esta é a única travessia do mundo inteiro que
    // corre com `&World` (uma `query` pede `&mut`, e o passe de pintura só tem a partilhada).
    // Precedente: a contagem de componentes em `snapshots`.
    let w = sim.world();
    let mut out: Vec<(u64, Entity)> = Vec::new();
    for archetype in w.archetypes().iter() {
        for ae in archetype.entities() {
            let e = ae.id();
            if w.get::<Visibility>(e).is_some_and(|v| v.hidden) || !is_empty_object(sim, e) {
                continue;
            }
            out.push((w.get::<ph2d_ecs::StableId>(e).map_or(0, |s| s.0), e));
        }
    }
    out.sort_unstable();
    out.into_iter().map(|(_, e)| e).collect()
}

/// ⭐⭐ **O anel PEGA** — os objetos vazios cujo disco contém `world`, em bits de `Entity`.
///
/// ⚠️ Sem isto o anel é decoração: um objeto sem pixels não é alcançável por
/// `pick_sprites_at_world`, logo a única forma de o selecionar era a lista da Hierarquia — e Enio
/// pediu exatamente o contrário (*«não consigo transformar o objeto total a partir do centro do
/// objeto vazio»*). *Uma alça que só se alcança noutro sítio não está no canvas.*
///
/// ⚠️ **Disco, não aro:** o interior conta. Um aro de 1,5 px é um alvo que se persegue.
pub(crate) fn pick_empty_at_world(
    sim: &SimWorld,
    world: [f32; 2],
    pixels_per_meter: f32,
) -> Vec<u64> {
    empty_objects(sim)
        .into_iter()
        .filter(|&e| {
            let c = world_transform(sim, e).translation;
            let r = marker_world_radius(sim, e, pixels_per_meter);
            r > 0.0 && (world[0] - c.x).hypot(world[1] - c.y) <= r
        })
        .map(Entity::to_bits)
        .collect()
}

/// A `GizmoView` de um objeto vazio ou de um grupo — a mesma caixa/pivô/rotação que um sprite
/// publica, para que `paint_sprite_gizmo` desenhe e registe as alças.
///
/// ⚠️ **A caixa é o MARCADOR, sempre** — ver o cabeçalho: a união dos filhos foi construída e
/// rejeitada por veredito de produto.
#[must_use]
pub(crate) fn view(
    sim: &SimWorld,
    entity: Entity,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    pivot_tool_active: bool,
    pixels_per_meter: f32,
) -> Option<GizmoView> {
    if sim.world().get_entity(entity).is_err() || !is_empty_object(sim, entity) {
        return None;
    }
    let half = marker_half(pixels_per_meter);
    Some(crate::vec_gizmo_view::gizmo_view_from(
        [0.0, 0.0],
        [half, half],
        world_transform(sim, entity),
        camera,
        window_size,
        last_pointer,
        pivot_tool_active,
    ))
}

#[cfg(test)]
#[path = "group_gizmo_view_tests.rs"]
mod tests;
