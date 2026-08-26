//! A forma vetorial vista pelo **gizmo de sprite** (ADR-0111).
//!
//! Não há gizmo vetorial próprio. Havia — 492 linhas que mutavam a geometria — e
//! ele foi removido. Um path com `Transform` é um objeto como qualquer outro, e a
//! matemática de mover/girar/escalar já existe, testada, em `ph2d-editor-core`.
//!
//! A tradução é exata, não uma imitação. O gizmo enquadra um sprite como
//!
//! ```text
//! centro = translation + R·(anchor ⊙ scale)     meia-extensão = half_intrínseco ⊙ scale
//! ```
//!
//! e uma forma vetorial tem a MESMA forma se lermos `anchor` como o **centro da
//! bbox local da curva** e `half_intrínseco` como a **meia-extensão dessa bbox**.
//! Por isso `opposite_anchor_translation` (que fixa o canto oposto ao escalar) vale
//! sem uma linha nova: ela só depende desses dois números.
//!
//! Aqui moram também o **picking** de canvas — clicar numa forma fora da ferramenta
//! vetorial a seleciona como se fosse um sprite — e o marquee.
//!
//! # O pick segue o DESENHO; a CAIXA segue a fonte
//!
//! Com um offset vivo (`ph2d_ecs::VecOffset`) o documento guarda a curva autorada e a tela
//! mostra a derivada. O **hit-test** pergunta ao mesmo `live` que o renderer consome — clicar
//! no que se vê é a definição de apontar, e sem isso a forma crescida era pintada num lugar e
//! clicável noutro.
//!
//! A **bbox do gizmo** (`gizmo_view_for_entity`, acima) fica na FONTE, e é decisão, não
//! esquecimento. Dois motivos: (a) o `d` do offset é uma distância de MUNDO, então escalar a
//! forma não escala a banda — uma caixa que a incluísse pediria `s = (alvo − 2d)/meia_fonte`,
//! e o gizmo derivaria do dedo durante o arrasto, que é a mesma armadilha das 5 tentativas
//! revertidas do Blend Object (ADR-0128: *um gizmo sobre geometria que se move DOBRA*);
//! (b) é o default do Illustrator — o bounding box ignora traço e efeitos de aparência, e
//! "Use Preview Bounds" é preferência **desligada**.

use ph2d_ecs::{Entity, SimWorld, VecPathRef};
use ph2d_editor::GizmoView;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPathId, VecScene, VecViewState};

use crate::vec_entities::VecEntityMap;
use crate::vec_transform::{world_transform, xform_of_transform};

/// Raio de captura do traço, em pixels de tela (× zoom → world). Formas abertas
/// (linha, arco, pen não-fechado) são pegas por proximidade do traço.
const STROKE_HIT_PX: f64 = 8.0;

/// Quanto a folga de um caminho **aberto** (uma linha, um conector) é maior que a da borda de
/// uma forma fechada. Ver o comentário no `contains_path`: uma linha não tem interior, logo a
/// folga *é* a área clicável — enquanto na borda de uma forma ela é só o fio.
const OPEN_PATH_HIT_K: f64 = 1.75;

/// `STROKE_HIT_PX` convertido a world-units no zoom atual.
#[must_use]
pub(crate) fn stroke_hit_r(camera: &Camera2d, window_size: WindowSize) -> f64 {
    let w0 = camera.screen_to_world((0.0, 0.0), window_size);
    let w1 = camera.screen_to_world((1.0, 0.0), window_size);
    let px = ((f64::from(w1[0] - w0[0])).powi(2) + (f64::from(w1[1] - w0[1])).powi(2)).sqrt();
    stroke_hit_r_from(px)
}

/// O mesmo raio, para quem já tem a escala px→mundo na mão. **Uma porta para a constante** — dois
/// sítios a multiplicá-la dariam alcances de captura diferentes para a mesma linha na tela.
#[must_use]
pub(crate) fn stroke_hit_r_from(px_to_world: f64) -> f64 {
    STROKE_HIT_PX * px_to_world
}

/// O `anchor` e o meio-tamanho **intrínsecos** (pré-escala) da forma de `entity`,
/// na linguagem que o gizmo de sprite fala. `None` se a entidade não é um path, ou
/// se o path está vazio.
#[must_use]
pub(crate) fn anchor_half(
    sim: &SimWorld,
    scene: &VecScene,
    entity: Entity,
) -> Option<([f32; 2], [f32; 2])> {
    let vp = sim.world().get::<VecPathRef>(entity)?;
    let (lo, hi) = scene.path_curve_bbox(vp.0)?;
    let anchor = [
        ((lo[0] + hi[0]) * 0.5) as f32,
        ((lo[1] + hi[1]) * 0.5) as f32,
    ];
    let half = [
        ((hi[0] - lo[0]) * 0.5) as f32,
        ((hi[1] - lo[1]) * 0.5) as f32,
    ];
    Some((anchor, half))
}

/// A `GizmoView` de uma forma vetorial — o mesmo `bbox_world` + `pivot` + `rotation`
/// que um sprite publica, para que `paint_sprite_gizmo` desenhe e registre as alças.
///
/// A pose vem do `SimWorld` (`Transform` ∘ cadeia de pais), e não do `PresentWorld`:
/// um path não é extraído para lá — ele não tem `RenderInstance`.
#[must_use]
#[allow(clippy::too_many_arguments)] // as mesmas entradas que a caixa do gizmo pede
pub(crate) fn view(
    sim: &SimWorld,
    scene: &VecScene,
    view_state: &VecViewState,
    entity: Entity,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    pivot_tool_active: bool,
) -> Option<GizmoView> {
    // **Um conector não tem gizmo.** Ele vive na identidade e o re-cook desfaz qualquer pose
    // (`connector_live`, detalhe 3), então a caixa de transformação seria um controle que não
    // controla nada — e, pior, o interior dela registra o hit "Translate" e **engoliria o
    // clique nas alças de ponta** (`connector_handles`), que são os controles que o conector
    // de fato tem. Publicar um gizmo aqui é publicar um ladrão de cliques.
    if sim.world().get::<ph2d_ecs::VecConnector>(entity).is_some() {
        return None;
    }
    // **O SPINE de um Blend Object também não tem gizmo** (ADR-0128, Enio 2026-07-15) — pela mesma
    // razão do conector: a linha é editável só no modo Node, e no Select o que se move são as
    // FORMAS-fonte (cada uma com o seu gizmo). Uma caixa fina sobre a linha, além de inútil, roubaria
    // o clique. A linha também não é PICKÁVEL no Select (o dispatch a filtra dos hits).
    if sim.world().get::<ph2d_ecs::VecBlend>(entity).is_some() {
        return None;
    }
    let (anchor, half_intrinsic) = anchor_half(sim, scene, entity)?;
    let id = sim.world().get::<VecPathRef>(entity)?.0;
    let wt = fold_layout_pose(world_transform(sim, entity), view_state.layout_pose(id));
    Some(gizmo_view_from(
        anchor,
        half_intrinsic,
        wt,
        camera,
        window_size,
        last_pointer,
        pivot_tool_active,
    ))
}

/// **Dobra a pose do AUTO LAYOUT dentro da pose de mundo** — para a caixa do gizmo aparecer onde
/// a forma está, e não onde ela foi autorada.
///
/// A pose que o layout produz é `translate ∘ scale` com os eixos do mundo (o motor não roda nada),
/// então dobrar é: a translação passa pela pose, e a escala multiplica.
///
/// ⚠️ **LIMITE, e ele é geométrico e não de implementação:** com o filho ROTACIONADO *e* a pose com
/// escala NÃO-UNIFORME, o resultado deixa de ser um retângulo orientado — nenhuma `GizmoView` o
/// representa, porque a caixa dela é `(centro, meia-extensão, rotação)`. Nesse caso a caixa fica
/// aproximada, e continua a ser **muito** melhor do que a alternativa: hoje ela aparece no lugar de
/// onde a forma saiu. É a mesma limitação honesta que o collider da física já carrega para o skew.
fn fold_layout_pose(
    mut wt: ph2d_ecs::Transform,
    pose: ph2d_vec_scene::Xform,
) -> ph2d_ecs::Transform {
    if pose.is_identity() {
        return wt;
    }
    let p = pose.apply([f64::from(wt.translation.x), f64::from(wt.translation.y)]);
    wt.translation.x = p[0] as f32;
    wt.translation.y = p[1] as f32;
    wt.scale.x *= pose.0[0] as f32;
    wt.scale.y *= pose.0[3] as f32;
    wt
}

/// Monta a `GizmoView` a partir do `anchor`/`half` **intrínsecos** (pré-escala, na linguagem do gizmo
/// de sprite) e da pose de MUNDO — a MESMA álgebra que o sprite usa. **Porta única:** a forma
/// vetorial ([`view`]) e o container do envelope ([`container_view`]) chamam esta função, então as
/// duas caixas concordam por construção (quad center = pivot + R·(anchor ⊙ scale)).
#[must_use]
pub(crate) fn gizmo_view_from(
    anchor: [f32; 2],
    half_intrinsic: [f32; 2],
    wt: ph2d_ecs::Transform,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    pivot_tool_active: bool,
) -> GizmoView {
    let (sx, sy) = (wt.scale.x, wt.scale.y);
    let half = [
        (half_intrinsic[0] * sx).abs(),
        (half_intrinsic[1] * sy).abs(),
    ];
    // Invariante idêntica à do sprite: quad center = pivot + R·(anchor ⊙ scale).
    let (ax, ay) = (anchor[0] * sx, anchor[1] * sy);
    // T1.3.5 cross-OS bit-identical.
    let (sin_r, cos_r) = libm::sincosf(wt.rotation);
    let cx = wt.translation.x + ax * cos_r - ay * sin_r;
    let cy = wt.translation.y + ax * sin_r + ay * cos_r;
    GizmoView {
        bbox_min_world: [cx - half[0], cy - half[1]],
        bbox_max_world: [cx + half[0], cy + half[1]],
        pivot_world: [wt.translation.x, wt.translation.y],
        pivot_tool_active,
        rotation: wt.rotation,
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: window_size.width as f32,
        window_h: window_size.height as f32,
        canvas: ph2d_editor::zones::Rect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        ),
        cursor_screen: Some(last_pointer),
    }
}

/// A `GizmoView` de um **container de Envelope** (ADR-0129 Fatia 3) — a caixa é a UNIÃO das bboxes
/// LOCAIS (espaço do container) dos filhos, e a pose é a do container. Assim o gizmo de sprite move/
/// gira/escala o envelope INTEIRO como uma unidade (Fatia 2): a seleção é só-o-container (nenhum
/// filho no gizmo), então o drag escreve só o `Transform` do container e os filhos o seguem por
/// parentesco — sem cisalhar. `None` se a entidade não é um envelope ou os filhos sumiram.
#[must_use]
pub(crate) fn container_view(
    sim: &SimWorld,
    scene: &VecScene,
    entity: Entity,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    pivot_tool_active: bool,
) -> Option<GizmoView> {
    let env = sim.world().get::<ph2d_ecs::VecEnvelope>(entity)?;
    // A geometria de cada filho na cena É o espaço LOCAL do container (filho na identidade, geometria
    // deformada re-escrita pelo recook em local). A união das bboxes de curva é a extensão do envelope.
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for child in &env.children {
        if let Some((clo, chi)) = scene.path_curve_bbox(child.path) {
            lo = [lo[0].min(clo[0]), lo[1].min(clo[1])];
            hi = [hi[0].max(chi[0]), hi[1].max(chi[1])];
        }
    }
    if !(lo[0].is_finite() && hi[0] >= lo[0] && hi[1] >= lo[1]) {
        return None;
    }
    let anchor = [
        ((lo[0] + hi[0]) * 0.5) as f32,
        ((lo[1] + hi[1]) * 0.5) as f32,
    ];
    let half_intrinsic = [
        ((hi[0] - lo[0]) * 0.5) as f32,
        ((hi[1] - lo[1]) * 0.5) as f32,
    ];
    let wt = world_transform(sim, entity);
    Some(gizmo_view_from(
        anchor,
        half_intrinsic,
        wt,
        camera,
        window_size,
        last_pointer,
        pivot_tool_active,
    ))
}

#[cfg(test)]
use ph2d_ecs::Transform;
#[cfg(test)]
use ph2d_vec_scene::rectangle;

/// A fixture partilhada pelos gates das DUAS metades (o que o gizmo mostra, o que o ponteiro
/// acha) — hasteada ao módulo para o irmão do pick a alcançar por `use super::*`.
#[cfg(test)]
fn scene_with_square() -> (SimWorld, VecScene, VecEntityMap, Entity) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(rectangle([-1.0, -1.0], [1.0, 1.0]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    (sim, scene, map, e)
}

/// **O hit-test de canvas e o marquee** — módulo irmão, pelo teto de 600 LOC da shell. O corte é
/// por assunto: aqui *o que o gizmo MOSTRA*, ali *o que o ponteiro ACHA*.
#[path = "vec_gizmo_pick.rs"]
mod pick;
pub(crate) use pick::{contains_world, pick_all_at_world, pick_in_world_rect};

#[cfg(test)]
#[path = "vec_gizmo_view_tests.rs"]
mod tests;
