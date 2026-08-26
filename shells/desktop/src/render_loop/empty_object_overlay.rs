//! ⭐ **O CÍRCULO de um objeto vazio** (Enio, 2026-08-26: *«para o objeto vazio precisamos de um
//! gizmo simples — um círculo simples de um tamanho razoável»*).
//!
//! Um objeto sem geometria não emite `RenderInstance` nenhuma: ele existe na Hierarquia, tem pose,
//! tem filhos — e **não há um pixel dele na tela**. Sem uma marca o artista não sabe onde ele está,
//! e é a mesma lição que o realce do Flip pagou: *o que não se vê não existe*.
//!
//! # Por que só o SELECIONADO
//!
//! ⚠️ Um círculo por cada objeto vazio da cena encheria o canvas de marcas para objetos que ninguém
//! está a editar — e, ao contrário das âncoras de uma junta (que **só** são alcançáveis pelo
//! canvas), um objeto vazio já é alcançável pela lista da Hierarquia. A marca serve o gesto que
//! está a acontecer, e por isso acompanha a seleção.
//!
//! # ⚠️ A pergunta é feita UMA vez
//!
//! *«Este objeto está vazio?»* é respondida por [`crate::group_gizmo_view::box_of`] — a **mesma**
//! função que dimensiona a caixa do gizmo. Uma segunda leitura aqui seria uma segunda opinião, e um
//! dia o círculo apareceria num objeto cuja caixa já é a união dos filhos.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_flip::FlipDoc;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vec_scene::VecScene;
use ph2d_vector::{Affine, Circle, Stroke, VectorScene};

/// Espessura do anel, em px de TELA.
///
/// ⚠️ Ela sai daqui sob `Affine::IDENTITY`: no Vello o transform de um `stroke` **multiplica** a
/// espessura, então entregar o afim mundo→tela transformaria 1,5 px em `1,5 × px_por_metro`. É o
/// defeito que o realce do Flip apanhou num smoke em 2026-07-13.
const RING_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// **Desenha o anel do objeto vazio selecionado.** No-op para tudo o resto.
#[allow(clippy::too_many_arguments)] // as entradas de qualquer overlay de canvas
pub(super) fn draw_empty_object_mark(
    sim: &SimWorld,
    scene: &VecScene,
    flip: &FlipDoc,
    selected: Option<u64>,
    pixels_per_meter: f32,
    theme: Theme,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let Some(entity) = selected.map(Entity::from_bits) else {
        return;
    };
    let Some(crate::group_gizmo_view::GroupBox::Empty { half }) =
        crate::group_gizmo_view::box_of(sim, scene, flip, entity, ppm)
    else {
        return;
    };
    let wt = crate::vec_transform::world_transform(sim, entity);
    let c = wt.translation;
    // ⚠️ **O raio é MEDIDO na tela, e não convertido à mão**: o anel tem de crescer com o zoom
    // exatamente como a caixa do gizmo cresce, e a única coisa que sabe a conversão é a câmara.
    // Um `raio × zoom` escrito aqui seria a segunda régua, e ela divergiria no primeiro pan.
    let (sx, sy) = camera.world_to_screen([c.x, c.y], window);
    let (ex, ey) = camera.world_to_screen([c.x + half[0] * wt.scale.x.abs(), c.y], window);
    let r = f64::from(((ex - sx).powi(2) + (ey - sy).powi(2)).sqrt());
    if !(r.is_finite() && r > 0.0) {
        return;
    }
    vector_scene.inner_mut().stroke(
        &Stroke::new(RING_PX),
        Affine::IDENTITY,
        ph2d_editor::paint::resolve(ColorToken::Selection, theme),
        None,
        &Circle::new((f64::from(sx), f64::from(sy)), r),
    );
}
