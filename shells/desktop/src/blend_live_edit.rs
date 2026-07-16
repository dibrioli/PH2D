//! As operações de **edição e interação** do Blend Object (ADR-0122 C1/C2b) — módulo irmão do
//! [`super`] (teto de 600 LOC do shell). O núcleo do objeto vivo (component, `recook`, os helpers de
//! geometria) fica em `blend_live.rs`; aqui moram as ações que o painel e o canvas disparam:
//!
//! - escolher as fontes (por z ou por clique) e ajustar os passos ao vivo;
//! - **Reset Spine** (volta ao automático);
//! - a **prévia do Pick Shapes** (contorno + polilinha da ordem);
//! - as reconciliações âncora↔forma: arrastar o SPINE move todas as fontes (Select), arrastar uma
//!   ÂNCORA move a forma dela (Node) — o que faz "editar a curva ≡ mover a forma".
//!
//! `use super::*` traz os privados do pai (`center_of`/`world`/`spine_verts`/`spine_stroke`/
//! `shift_vertex_to`/`MAX_BLEND_SOURCES`/`BlendSpines`/`BlendDrag` + os `use` dele), como os módulos
//! de teste já fazem.

use super::*;

/// O traço de REALCE do modo Pick Shapes — um azul de acento, distinto do cinza sutil do spine.
/// Como o [`super::spine_stroke`], é um `Rgba8` em MUNDO por ora; o refinamento correto é um guia por
/// overlay em screen-space com cor por token (ADR-0122 C2b), quando a preview virar UI de verdade.
fn pick_stroke() -> ph2d_vec_scene::StrokeSpec {
    ph2d_vec_scene::StrokeSpec::new(ph2d_vec_scene::Rgba8::new(80, 150, 240, 230), 0.04)
}

/// A **prévia do modo Pick Shapes** (ADR-0122 C2b): o contorno realçado de cada forma escolhida +
/// a polilinha que as costura na ORDEM de clique (a prévia do spine-a-ser). Tudo em MUNDO, para o
/// [`ph2d_vec_render::draw_blend_overlay`] desenhar por cima. `picks` na ordem de clique; formas
/// que sumiram são puladas. Vazio ⇒ nada é desenhado.
///
/// É o que torna a escolha VISÍVEL: o artista clica as formas e vê a cadeia se formar (a linha
/// cresce do 1º clique), então sabe a ordem que o Blend vai usar sem um número na tela.
pub(crate) fn pick_preview(
    scene: &VecScene,
    xforms: &VecXforms,
    picks: &[VecPathId],
) -> Vec<VecPath> {
    let mut out = Vec::new();
    // O contorno de cada forma escolhida (sem fill — só o realce da silhueta).
    for &id in picks {
        if let Some(mut p) = world(scene, xforms, id) {
            p.fill = None;
            p.stroke = Some(pick_stroke());
            out.push(p);
        }
    }
    // A polilinha pela ordem de clique — a prévia do spine (a linha que unirá as formas).
    let centers: Vec<[f64; 2]> = picks
        .iter()
        .filter_map(|&id| center_of(scene, xforms, id))
        .collect();
    if centers.len() >= 2 {
        out.push(VecPath {
            verts: spine_verts(&centers),
            closed: false,
            stroke: Some(pick_stroke()),
            ..VecPath::default()
        });
    }
    out
}

/// As formas FECHADAS selecionadas, na ordem de **z** (a de `paths()`), capadas em
/// [`super::MAX_BLEND_SOURCES`]. É o que o botão "Blend" liga — a ordem da cadeia é a de z, como o
/// "Make" do Illustrator (formas abertas não têm interior para interpolar, então são descartadas).
pub(crate) fn selected_closed_in_z(
    scene: &VecScene,
    pen: &ph2d_vec_edit::PenTool,
) -> Vec<VecPathId> {
    let mut zs: Vec<(usize, VecPathId)> = pen
        .selected_paths()
        .iter()
        .filter_map(|id| {
            scene
                .paths()
                .iter()
                .position(|p| p.id == *id && p.closed)
                .map(|z| (z, *id))
        })
        .collect();
    zs.sort_unstable_by_key(|(z, _)| *z);
    zs.dedup_by_key(|(z, _)| *z);
    zs.into_iter()
        .take(MAX_BLEND_SOURCES)
        .map(|(_, id)| id)
        .collect()
}

/// Ajusta os passos do(s) Blend Object(s) SELECIONADO(s) — o slider Steps ao vivo. Devolve `true`
/// se algum blend foi retunado (nenhum selecionado ⇒ `false`, e o valor é só o de criação futura).
/// Idempotente: não marca a entidade suja se o valor já é o mesmo.
pub(crate) fn set_selected_steps(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    pen: &ph2d_vec_edit::PenTool,
    steps: u32,
) -> bool {
    let mut changed = false;
    for id in pen.selected_paths() {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        if sim
            .world()
            .get::<VecBlend>(e)
            .is_some_and(|b| b.steps != steps)
            && let Some(mut b) = sim.world_mut().get_mut::<VecBlend>(e)
        {
            b.steps = steps;
            changed = true;
        }
    }
    changed
}

/// **Reset Spine:** volta o(s) blend(s) selecionado(s) ao spine AUTOMÁTICO (a reta pelos centros),
/// desfazendo a edição do modo Node. Devolve `true` se algum blend foi resetado.
///
/// Limpa `spine_authored` **E** a memória do auto (`spines`): sem apagar a memória, a detecção do
/// [`super::recook`] compararia o spine BENT ainda na cena com o último auto memorizado (diferentes)
/// e o RE-autoraria no mesmo frame — o reset não pegaria. Com a memória vazia, a detecção não
/// dispara (`is_some_and` é falso) e o ramo automático reescreve a reta e a memoriza de novo.
pub(crate) fn reset_spine(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    pen: &ph2d_vec_edit::PenTool,
    spines: &mut BlendSpines,
) -> bool {
    let mut changed = false;
    for id in pen.selected_paths() {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        if sim
            .world()
            .get::<VecBlend>(e)
            .is_some_and(|b| b.spine_authored)
            && let Some(mut b) = sim.world_mut().get_mut::<VecBlend>(e)
        {
            b.spine_authored = false;
            spines.remove(id); // esquece o auto memorizado (senão o recook re-autora na hora)
            changed = true;
        }
    }
    changed
}

/// **Select: arrastar o SPINE (o objeto blend) move TODAS as fontes juntas** (ADR-0122, ajuste do
/// Enio: "as formas seguem a linha como filhas"). Devolve `true` se moveu algo.
///
/// O gizmo escreve a translação **TOTAL do gesto** (desde o início do arrasto) no `Transform` do
/// blend a cada `CursorMoved`; a `recook` o devolve à identidade a cada frame (a geometria dele é
/// MUNDO). Aqui, quando o `Transform` não é identidade, aplicamos só o INCREMENTO desde a última
/// leitura (`drags` guarda o total já consumido, por blend) a CADA fonte, e o zeramos. As fontes
/// movidas fazem o spine (e os passos) segui-las no `recook` seguinte.
///
/// **`gizmo_dragging` é o que evita o drift:** o `Transform` fica na identidade entre um `CursorMoved`
/// e o render seguinte (o `advance` do gizmo só roda no Move); se limpássemos o total memorizado toda
/// vez que ele é identidade, o Move seguinte re-aplicaria o TOTAL inteiro (o drift brutal). Em vez
/// disso: identidade **durante** um arrasto = pular (guarda o total); só ao **acabar** o arrasto
/// (`!gizmo_dragging`) esquecemos o total, para o próximo gesto começar do zero.
///
/// Translação só — girar/escalar o grupo é follow-up (o gizmo os escreveria no `Transform`, que
/// zeramos; o efeito hoje é nulo, como antes deste ajuste).
pub(crate) fn drag_blend_moves_sources(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    drags: &mut BlendDrag,
    gizmo_dragging: bool,
) -> bool {
    let blends: Vec<(VecPathId, Entity, VecBlend)> = map
        .iter()
        .filter_map(|(&id, &bits)| {
            let e = Entity::from_bits(bits);
            let b = sim.world().get::<VecBlend>(e)?.clone();
            Some((id, e, b))
        })
        .collect();
    let mut moved = false;
    for (spine_id, entity, blend) in &blends {
        let t = sim
            .world()
            .get::<Transform>(*entity)
            .copied()
            .unwrap_or(Transform::IDENTITY);
        if t == Transform::IDENTITY {
            continue; // identidade: o gizmo não escreveu neste render — guarda o total, não limpa
        }
        let cur = [f64::from(t.translation.x), f64::from(t.translation.y)];
        let last = drags.get(spine_id).copied().unwrap_or([0.0, 0.0]);
        let delta = [cur[0] - last[0], cur[1] - last[1]];
        if delta[0] != 0.0 || delta[1] != 0.0 {
            for src_id in &blend.sources {
                if let Some(&bits) = map.get(src_id)
                    && scene.paths().iter().any(|p| p.id == *src_id)
                {
                    crate::vec_transform::translate_shape_world(
                        sim,
                        Entity::from_bits(bits),
                        delta,
                    );
                    moved = true;
                }
            }
        }
        drags.insert(*spine_id, cur);
        // Consumida a pose, o blend volta à identidade (a geometria dele é MUNDO; a recook faria o
        // mesmo, mas aqui é ANTES dela ler os centros das fontes).
        if let Some(mut bt) = sim.world_mut().get_mut::<Transform>(*entity) {
            *bt = Transform::IDENTITY;
        }
    }
    // Fim do arrasto (ou nenhum): esquece os totais, para o próximo gesto começar do zero. Durante o
    // arrasto o total PERSISTE, mesmo nos frames em que o `Transform` está na identidade.
    if gizmo_dragging {
        drags.retain(|id, _| blends.iter().any(|(b, _, _)| b == id));
    } else {
        drags.clear();
    }
    moved
}

/// O mapa (índice de vértice do spine → forma-fonte que ele representa), para o
/// [`drag_spine_anchors_move_sources`] e a [`super::pin_spine_anchors`] concordarem.
///
/// Quando o spine tem **um vértice por fonte** (o caso normal — o modo Node MOVE âncoras, não cria),
/// TODA âncora é de uma fonte, e o `zip` 1-a-1 as liga (inclusive as do MEIO da cadeia). Se o spine
/// tiver MAIS vértices que fontes (pontos de dobra extras — hoje só via smoke), só a 1ª e a última
/// âncora são fontes garantidas.
pub(crate) fn anchor_source_pairs(n_verts: usize, live: &[VecPathId]) -> Vec<(usize, VecPathId)> {
    if n_verts == live.len() {
        live.iter().copied().enumerate().collect()
    } else if n_verts >= 2 && live.len() >= 2 {
        vec![(0, live[0]), (n_verts - 1, live[live.len() - 1])]
    } else {
        Vec::new()
    }
}

/// **Modo Node: arrastar uma ÂNCORA do spine MOVE a forma-fonte dela** (ADR-0122 C2b) — o inverso da
/// pinagem, e o que faz "editar a curva no Node ser igual a mover a forma no Select".
///
/// Cada âncora do spine corresponde a uma forma da cadeia ([`anchor_source_pairs`] — inclusive as do
/// MEIO). Se o artista a arrastou (modo Node), ela difere do centro da fonte; movemos a FONTE por
/// essa delta (`vec_transform::translate_shape_world`). O `recook` logo em seguida re-encosta a
/// âncora no centro — agora coincidentes, **sem salto** —, os passos re-cozem e a curva se adapta.
///
/// Roda ANTES do [`super::recook`] e SÓ no modo Node (no Select a fonte se move pelo gizmo e a âncora
/// a segue). Arrastar uma ALÇA (bézier) em vez da âncora curva o spine sem mover a forma (a âncora
/// não muda → delta zero → nada aqui; a detecção do `recook` autora a curva).
///
/// **Não autora o spine** ao mover uma âncora (mover a forma ≠ curvar a curva): quando o spine é
/// automático, atualiza a âncora na memória do auto (`spines`) para o novo centro, e a detecção do
/// `recook` não a confunde com uma edição de curva (só mexer numa alça ou num ponto de dobra autora).
pub(crate) fn drag_spine_anchors_move_sources(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    spines: &mut BlendSpines,
) {
    let blends: Vec<(VecPathId, VecBlend)> = map
        .iter()
        .filter_map(|(&id, &bits)| {
            let b = sim
                .world()
                .get::<VecBlend>(Entity::from_bits(bits))?
                .clone();
            Some((id, b))
        })
        .collect();
    for (spine_id, blend) in blends {
        let live: Vec<VecPathId> = blend
            .sources
            .iter()
            .copied()
            .filter(|id| scene.paths().iter().any(|p| p.id == *id))
            .collect();
        let Some(sp) = scene.paths().iter().find(|p| p.id == spine_id) else {
            continue;
        };
        for (vi, src_id) in anchor_source_pairs(sp.verts.len(), &live) {
            let e = sp.verts[vi].anchor;
            let Some(c) = center_of(scene, xforms, src_id) else {
                continue;
            };
            let (dx, dy) = (e[0] - c[0], e[1] - c[1]);
            if dx * dx + dy * dy <= 1e-18 {
                continue; // a âncora está sobre o centro: nada foi arrastado
            }
            let Some(&bits) = map.get(&src_id) else {
                continue;
            };
            if crate::vec_transform::translate_shape_world(sim, Entity::from_bits(bits), [dx, dy]) {
                // A âncora agora É o centro da fonte: atualiza a memória do auto para que a detecção
                // do `recook` não trate o movimento da forma como uma edição de curva. Move o vértice
                // INTEIRO (âncora + alças) como o arrasto fez — só a âncora deixaria as alças
                // divergindo do spine atual, e a detecção autoraria à toa.
                if let Some(mem) = spines.get_mut(&spine_id)
                    && let Some(mv) = mem.get_mut(vi)
                {
                    shift_vertex_to(mv, e);
                }
            }
        }
    }
}
