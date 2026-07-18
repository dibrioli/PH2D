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

use ph2d_ecs::{Entity, SimWorld, VecPathRef};
use ph2d_editor::GizmoView;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
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
    STROKE_HIT_PX * px
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
pub(crate) fn view(
    sim: &SimWorld,
    scene: &VecScene,
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

/// Monta a `GizmoView` a partir do `anchor`/`half` **intrínsecos** (pré-escala, na linguagem do gizmo
/// de sprite) e da pose de MUNDO — a MESMA álgebra que o sprite usa. **Porta única:** a forma
/// vetorial ([`view`]) e o container do envelope ([`container_view`]) chamam esta função, então as
/// duas caixas concordam por construção (quad center = pivot + R·(anchor ⊙ scale)).
#[must_use]
fn gizmo_view_from(
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

/// O ponto de mundo `p` cai DENTRO da forma de `entity`?
///
/// É o análogo vetorial de `pick_sprite_at_world` para uma entidade já conhecida —
/// o que decide se um Down no interior da forma inicia um move (e não um pan).
///
/// `stroke_hit_r` (world-units) é o raio de captura do TRAÇO: uma forma ABERTA
/// (linha, arco, pen não-fechado) não tem interior, então sem isso ela nunca seria
/// pega — e o gizmo de Select nunca a agarraria (Enio 2026-07-09).
#[must_use]
pub(crate) fn contains_world(
    sim: &SimWorld,
    scene: &VecScene,
    entity: Entity,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> bool {
    let Some(vp) = sim.world().get::<VecPathRef>(entity) else {
        return false;
    };
    contains_path(sim, scene, entity, vp.0, p, stroke_hit_r)
}

/// Amostras por segmento na varredura de proximidade do traço.
const STROKE_SAMPLES: u32 = 24;

/// `p` (mundo) pega o path `id`: no INTERIOR (formas fechadas) OU a ≤ `stroke_hit_r`
/// do TRAÇO (formas abertas e a borda de fechadas).
fn contains_path(
    sim: &SimWorld,
    scene: &VecScene,
    entity: Entity,
    id: VecPathId,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> bool {
    let x = xform_of_transform(world_transform(sim, entity));
    let Some(inv) = x.inverse() else {
        return false; // forma colapsada
    };
    let local = inv.apply([f64::from(p[0]), f64::from(p[1])]);
    if scene.path_contains_point(id, local) {
        return true;
    }
    // Proximidade do traço: a curva é local, o raio é world → converte pela escala.
    let Some(path) = scene.paths().iter().find(|pp| pp.id == id) else {
        return false;
    };
    // **A PONTA DE SETA é clicável.** Ela não faz parte do `VecPath` — é construída a partir
    // dele + do `StrokeSpec` (`stroke_head`, a MESMA função que o renderer chama). Enquanto o
    // hit-test não a construía, o triângulo, que é a parte GORDA da seta e a que o olho mira,
    // não selecionava nada: a única área clicável de um conector era o fio da linha. Era a
    // queixa do Enio, e a causa não era o raio de captura — era metade do desenho que
    // simplesmente não existia para o mouse.
    if let Some(s) = path.stroke.as_ref() {
        for at_start in [true, false] {
            if let Some((_, head)) = ph2d_vec_scene::stroke_head(path, s, at_start)
                && ph2d_vec_scene::contains_point(&head, local)
            {
                return true;
            }
        }
    }

    // COZIDA: apalpar o traço é apalpar a tinta que está na tela, e uma quina arredondada
    // deixou de passar pelo bico afiado que o documento ainda guarda. (`nearest_point_on_path`
    // em si fica na FONTE — o índice de segmento que ela devolve endereça os verts autorados,
    // e é o que insere vértice; aqui só interessa a distância.)
    if let Some((_, _, d2)) =
        ph2d_vec_scene::nearest_point_on_path(&path.cooked(), local, STROKE_SAMPLES)
    {
        // **A TINTA QUE SE VÊ CONTA.** O raio de captura é a folga MAIS a metade da largura
        // desenhada — não a folga sozinha. Sem isso, uma linha grossa só era pegável nos 8 px
        // centrais dela: o usuário clicava visivelmente EM CIMA do traço e nada acontecia,
        // porque o hit-test media a distância até a CURVA e ignorava a espessura com que ela
        // é pintada.
        let half_ink = path.stroke.as_ref().map_or(0.0, |s| s.width * 0.5);
        // **Uma LINHA precisa de mais folga que a borda de uma forma** — e é por isso que os
        // 8 px eram apertados. Eles foram calibrados para o contorno de uma forma FECHADA, que
        // tem um interior para mirar: ali a folga só serve para pegar o fio da borda, e uma
        // folga grande roubaria cliques do que está atrás. Numa linha, a curva é o ÚNICO alvo
        // que existe — não há interior nenhum —, então a folga é a área clicável inteira. Ela
        // só pode roubar clique de outra linha, o que praticamente não acontece.
        let open = (0..path.contour_count()).all(|c| !matches!(path.contour(c), Some((_, true))));
        let slop = if open {
            stroke_hit_r * OPEN_PATH_HIT_K
        } else {
            stroke_hit_r
        };
        return d2.sqrt() * x.mean_scale() <= slop + half_ink;
    }
    false
}

/// Toda forma vetorial sob `p` (mundo), **do topo para o fundo** — a lista que o
/// clique-cíclico do canvas consome. Escondida ou travada não entra, como um sprite
/// não entraria.
///
/// `paths` está em ordem de z (fundo → topo), então varre-se ao contrário.
#[must_use]
pub(crate) fn pick_all_at_world(
    sim: &SimWorld,
    scene: &VecScene,
    view_state: &VecViewState,
    map: &VecEntityMap,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> Vec<u64> {
    let mut out = Vec::new();
    for path in scene.paths().iter().rev() {
        if !view_state.is_pickable(path.id) {
            continue;
        }
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_ok()
            && contains_path(sim, scene, e, path.id, p, stroke_hit_r)
        {
            out.push(bits);
        }
    }
    out
}

/// A forma mais ao topo sob `p`, ou `None`. Conveniência dos testes: o canvas
/// consome a lista inteira ([`pick_all_at_world`]) para poder ciclar entre
/// sobreposições.
#[cfg(test)]
#[must_use]
fn pick_at_world(
    sim: &SimWorld,
    scene: &VecScene,
    view_state: &VecViewState,
    map: &VecEntityMap,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> Option<u64> {
    pick_all_at_world(sim, scene, view_state, map, p, stroke_hit_r)
        .into_iter()
        .next()
}

/// Toda forma vetorial cuja bbox de mundo intersecta o retângulo — o marquee.
#[must_use]
pub(crate) fn pick_in_world_rect(
    sim: &SimWorld,
    scene: &VecScene,
    view_state: &VecViewState,
    map: &VecEntityMap,
    rect_min: [f32; 2],
    rect_max: [f32; 2],
) -> Vec<u64> {
    let mut out = Vec::new();
    for path in scene.paths() {
        if !view_state.is_pickable(path.id) {
            continue;
        }
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let Some((lo, hi)) = scene.path_curve_bbox(path.id) else {
            continue;
        };
        let x = xform_of_transform(world_transform(sim, e));
        // Os 4 cantos do bbox LOCAL sobem ao mundo; uma forma girada dá um
        // quadrilátero, e o bbox dele é o que se compara com o marquee.
        let corners = [
            x.apply(lo),
            x.apply([hi[0], lo[1]]),
            x.apply(hi),
            x.apply([lo[0], hi[1]]),
        ];
        let (mut wlo, mut whi) = (corners[0], corners[0]);
        for c in &corners[1..] {
            wlo = [wlo[0].min(c[0]), wlo[1].min(c[1])];
            whi = [whi[0].max(c[0]), whi[1].max(c[1])];
        }
        let overlaps = whi[0] >= f64::from(rect_min[0])
            && wlo[0] <= f64::from(rect_max[0])
            && whi[1] >= f64::from(rect_min[1])
            && wlo[1] <= f64::from(rect_max[1]);
        if overlaps {
            out.push(bits);
        }
    }
    out
}

#[cfg(test)]
#[path = "vec_gizmo_view_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "vec_gizmo_view_hit_tests.rs"]
mod hit_tests;
