//! ⭐ **O gizmo de um objeto que não desenha nada** — o VAZIO e o GRUPO (Enio, 2026-08-26).
//!
//! > *«O objeto vazio criado na hierarquia é invisível e ao agregar filhos e selecionar o objeto
//! > (o pai) não se consegue transformar o conjunto como um objeto só. […] O gizmo da sprite deve
//! > atuar sobre o objeto vazio e quando não for vazio (quando tiver filhos visíveis) deve se
//! > adequar ao tamanho total do objeto.»*
//!
//! Até aqui `snapshots::build_view` respondia `None` para toda entidade sem geometria própria —
//! *«grupo/outro: sem gizmo próprio»* — e um objeto sem gizmo **não é agarrável de forma nenhuma**:
//! nem para mover, nem para girar, nem para escalar. O botão `Add` da Hierarquia (F3) nasce
//! exatamente assim (`Transform` + `Name` e mais nada), então o objeto que o artista acabou de
//! criar era o único do app que ele não podia pegar.
//!
//! # As DUAS respostas, e a lei é a mesma da caixa do envelope
//!
//! 1. **Tem filhos visíveis com geometria** ⇒ a caixa é a **UNIÃO** deles, medida no espaço LOCAL
//!    do pai. O gizmo escreve só o `Transform` do pai e os filhos seguem por parentesco — é o que
//!    faz o conjunto mover-se *como um objeto só*, sem cisalhar (ADR-0129 Fatia 3, que já fazia
//!    isto para o container de um `VecEnvelope`).
//! 2. **Não tem nenhum** ⇒ a caixa é o **marcador do vazio**, com meia-extensão derivada do
//!    tamanho da alça (ver [`EMPTY_HALF_PX`]). Uma caixa de extensão zero desenharia as oito alças
//!    umas por cima das outras no mesmo pixel — *colapsada*, na palavra do report.
//!
//! ⚠️ **A união é medida no espaço do PAI, e não no mundo.** [`vec_gizmo_view::gizmo_view_from`]
//! aplica a pose do pai (rotação e escala incluídas) à caixa que recebe; entregar-lhe uma caixa já
//! em mundo aplicá-la-ia **duas vezes**, e o gizmo derivaria do objeto a cada grau de rotação.
//!
//! # ⛔ Quem já tem alças NÃO ganha caixa
//!
//! Uma junta e uma roldana também não têm geometria, e chegam aqui pelo mesmo caminho — mas elas
//! **já publicam alças** (os dots de [`crate::render_loop::point_gizmo`]). Uma caixa por cima
//! registaria o interior dela como *Translate* no hit-index e **engoliria o clique nas alças**, que
//! são os controlos que aquelas entidades de facto têm. *É a mesma razão pela qual o conector e o
//! spine de um Blend não publicam gizmo* (ver o cabeçalho de [`crate::vec_gizmo_view`]).

use ph2d_ecs::{Children, Entity, SimWorld, Transform, Visibility};
use ph2d_editor::GizmoView;
use ph2d_flip::FlipDoc;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::{VecScene, Xform};

use crate::vec_transform::{world_transform, xform_of_transform};

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

/// **A caixa de um objeto sem geometria própria** — e qual das duas respostas ela é.
///
/// ⚠️ É um `enum` e não um par de números porque o **círculo** do canvas
/// ([`crate::render_loop::empty_object_overlay`]) e a **caixa** do gizmo têm de concordar sobre
/// *«este objeto está vazio»*: duas leituras separadas da mesma pergunta são duas respostas que um
/// dia divergem. *Uma lei, dois consumidores.*
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum GroupBox {
    /// A união das peças visíveis, no espaço local do pai.
    Union { anchor: [f32; 2], half: [f32; 2] },
    /// Não há nada lá dentro — o marcador do vazio, centrado na origem.
    Empty { half: [f32; 2] },
}

impl GroupBox {
    /// O par `(âncora, meia-extensão)` que o gizmo fala.
    pub(crate) fn anchor_half(self) -> ([f32; 2], [f32; 2]) {
        match self {
            Self::Union { anchor, half } => (anchor, half),
            Self::Empty { half } => ([0.0, 0.0], half),
        }
    }
}

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

/// A caixa intrínseca de uma peça FOLHA, no espaço local dela — `None` se ela não desenha nada.
///
/// ⚠️ **As três perguntas saem das MESMAS portas que o gizmo de cada família usa** (a caixa de
/// sprite de [`crate::render_loop::sheet_grid_overlay::gizmo_box`], o `anchor_half` do vetor e o do
/// Flip). Recalcular aqui daria uma união que não coincide com a caixa que a mesma peça mostra
/// quando é ela a selecionada.
fn leaf_anchor_half(
    sim: &SimWorld,
    scene: &VecScene,
    flip: &FlipDoc,
    e: Entity,
    pixels_per_meter: f32,
) -> Option<([f32; 2], [f32; 2])> {
    if let Some(spr) = sim.world().get::<ph2d_render::Sprite>(e) {
        // A folha ABERTA é uma pré-visualização de quem está selecionado; dentro de um grupo a
        // peça é a célula viva, que é o que o renderer desenha.
        return Some(crate::render_loop::sheet_grid_overlay::gizmo_box(
            spr,
            sim.world().get::<ph2d_ecs::SpriteGrid>(e).copied(),
            pixels_per_meter,
            false,
            false,
        ));
    }
    if sim.world().get::<ph2d_ecs::VecPathRef>(e).is_some() {
        return crate::vec_gizmo_view::anchor_half(sim, scene, e);
    }
    if sim.world().get::<ph2d_ecs::FlipObjectRef>(e).is_some() {
        return crate::flip_gizmo_view::anchor_half(sim, flip, e);
    }
    None
}

/// ⭐ **A caixa do grupo** — `None` quando a entidade não é candidata (já tem alças próprias, ou
/// nem sequer existe).
///
/// ⚠️ **Um filho ESCONDIDO não entra**, e a sub-árvore dele também não: o eixo é *«o que se vê»*, e
/// a receita de um componente é escondida de propósito (F4.5) — uma caixa que a envolvesse mediria
/// um objeto que não está na tela.
pub(crate) fn box_of(
    sim: &SimWorld,
    scene: &VecScene,
    flip: &FlipDoc,
    entity: Entity,
    pixels_per_meter: f32,
) -> Option<GroupBox> {
    if sim.world().get_entity(entity).is_err() || publishes_its_own_handles(sim, entity) {
        return None;
    }
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [f32::NEG_INFINITY; 2];
    // Pilha `(entidade, afim local-dela → local-do-pai)`. A raiz entra na identidade: o espaço em
    // que os `Transform` dos filhos vivem É o espaço local dela.
    let mut stack: Vec<(Entity, Xform)> = vec![(entity, Xform::IDENTITY)];
    while let Some((e, to_root)) = stack.pop() {
        let Some(kids) = sim.world().get::<Children>(e) else {
            continue;
        };
        for &child in kids.iter() {
            if sim
                .world()
                .get::<Visibility>(child)
                .is_some_and(|v| v.hidden)
            {
                continue;
            }
            let local = sim
                .world()
                .get::<Transform>(child)
                .copied()
                .unwrap_or(Transform::IDENTITY);
            let x = xform_of_transform(local).then(&to_root);
            if let Some((a, h)) = leaf_anchor_half(sim, scene, flip, child, pixels_per_meter) {
                // Os QUATRO cantos, e não o par (mín, máx): sob rotação a caixa do filho não é
                // eixo-alinhada no espaço do pai, e unir só dois cantos perderia os outros dois.
                for sx in [-1.0f32, 1.0] {
                    for sy in [-1.0f32, 1.0] {
                        let p = x.apply([f64::from(a[0] + sx * h[0]), f64::from(a[1] + sy * h[1])]);
                        lo = [lo[0].min(p[0] as f32), lo[1].min(p[1] as f32)];
                        hi = [hi[0].max(p[0] as f32), hi[1].max(p[1] as f32)];
                    }
                }
            }
            stack.push((child, x));
        }
    }
    let half_empty = EMPTY_HALF_PX / pixels_per_meter.max(f32::MIN_POSITIVE);
    if !(lo[0].is_finite() && hi[0] >= lo[0] && hi[1] >= lo[1]) {
        return Some(GroupBox::Empty {
            half: [half_empty, half_empty],
        });
    }
    Some(GroupBox::Union {
        anchor: [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5],
        half: [(hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5],
    })
}

/// A `GizmoView` de um grupo ou de um vazio — a mesma caixa/pivô/rotação que um sprite publica,
/// para que `paint_sprite_gizmo` desenhe e registe as alças.
#[must_use]
#[allow(clippy::too_many_arguments)] // as mesmas entradas que as outras três publicadoras pedem
pub(crate) fn view(
    sim: &SimWorld,
    scene: &VecScene,
    flip: &FlipDoc,
    entity: Entity,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    pivot_tool_active: bool,
    pixels_per_meter: f32,
) -> Option<GizmoView> {
    let (anchor, half) = box_of(sim, scene, flip, entity, pixels_per_meter)?.anchor_half();
    Some(crate::vec_gizmo_view::gizmo_view_from(
        anchor,
        half,
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
