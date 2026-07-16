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
//! `shift_vertex_to`/`MAX_BLEND_SOURCES`/`BlendSpines` + os `use` dele), como os módulos de teste já
//! fazem.

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

/// Os blends que a seleção TOCA, como `(spine_id, entidade)`: aquele cujo **spine** está selecionado
/// (o modo Node) **ou** aquele que tem QUALQUER **forma-fonte** selecionada (o modo Select — onde a
/// linha nem é selecionável, e portanto são as FORMAS que dizem de que blend se fala).
///
/// É a ÚNICA porta para "de que blend o painel fala" (Enio 2026-07-15) — o Steps e o Reset Spine a
/// compartilham; duas respostas divergiriam. Devolve o `spine_id` junto porque a memória do auto
/// (`BlendSpines`) é chaveada por ELE, não pelo path que o artista selecionou.
fn blends_touched_by(
    sim: &SimWorld,
    map: &VecEntityMap,
    sel: &[VecPathId],
) -> Vec<(VecPathId, Entity)> {
    map.iter()
        .filter_map(|(&id, &bits)| {
            let e = Entity::from_bits(bits);
            let b = sim.world().get::<VecBlend>(e)?;
            let touched = sel.contains(&id) || b.sources.iter().any(|s| sel.contains(s));
            touched.then_some((id, e))
        })
        .collect()
}

/// Ajusta os passos do(s) Blend Object(s) que a seleção TOCA — o slider Steps ao vivo. Devolve
/// `true` se algum blend foi retunado (nada tocado ⇒ `false`, e o valor é só o de criação futura).
/// Idempotente: não marca a entidade suja se o valor já é o mesmo.
///
/// **Basta QUALQUER objeto do blend estar selecionado** ([`blends_touched_by`]) — a linha (modo
/// Node) ou uma das formas (modo Select). Antes só o spine contava, e como a linha não é selecionável
/// no Select, o slider ficava inerte justo no modo em que se mexe nas formas (Enio 2026-07-15).
pub(crate) fn set_selected_steps(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    pen: &ph2d_vec_edit::PenTool,
    steps: u32,
) -> bool {
    let mut changed = false;
    for (_, e) in blends_touched_by(sim, map, pen.selected_paths()) {
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

/// **Reset Spine:** volta ao spine AUTOMÁTICO (a reta pelos centros) o(s) blend(s) que a seleção
/// TOCA — a linha OU qualquer forma dele ([`blends_touched_by`], a mesma porta do Steps). Desfaz a
/// edição do modo Node. Devolve `true` se algum blend foi resetado.
///
/// Limpa `spine_authored` **E** a memória do auto (`spines`, chaveada pelo SPINE): sem apagar a
/// memória, a detecção do [`super::recook`] compararia o spine BENT ainda na cena com o último auto
/// memorizado (diferentes) e o RE-autoraria no mesmo frame — o reset não pegaria. Com a memória
/// vazia, a detecção não dispara (`is_some_and` é falso) e o ramo automático reescreve a reta.
pub(crate) fn reset_spine(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    pen: &ph2d_vec_edit::PenTool,
    spines: &mut BlendSpines,
) -> bool {
    let mut changed = false;
    for (spine_id, e) in blends_touched_by(sim, map, pen.selected_paths()) {
        if sim
            .world()
            .get::<VecBlend>(e)
            .is_some_and(|b| b.spine_authored)
            && let Some(mut b) = sim.world_mut().get_mut::<VecBlend>(e)
        {
            b.spine_authored = false;
            spines.remove(&spine_id); // esquece o auto memorizado (senão o recook re-autora na hora)
            changed = true;
        }
    }
    changed
}
/// **Expand** (ADR-0122 Fase D) — o *"pode ser expandido em múltiplas formas com um botão"* do Enio
/// (#7): materializa os passos VIRTUAIS do(s) blend(s) que a seleção toca em paths REAIS na cena, e
/// descarta o objeto vivo. Espelha o `drop_shape_params` da Live Shape ("Convert to Curves") — o
/// objeto vivo vira geometria morta, editável ponto a ponto.
///
/// **As fontes PERSISTEM.** Expandir não consome os operandos (≠ booleana): é o Illustrator, e é o
/// que o `recook` já dizia ao redesenhar cada fonte por cima dos passos dela.
///
/// Os passos saem de [`super::cook_links`] — a MESMA porta que o `recook` desenha. É o ponto todo:
/// uma 2ª porta faria as formas **saltarem** no clique, justo na operação que promete entregar o que
/// estava na tela. Cada passo já vem com o estilo interpolado e assado em MUNDO; o path nasce sem
/// pose (a entidade nasce na identidade no `sync`), então a geometria de mundo está certa.
///
/// **O spine morre junto:** ele era o corpo do objeto vivo; sem o componente ficaria um path
/// invisível e órfão na Hierarquia. Removê-lo da cena basta — o `vec_entities::sync` despawna a
/// entidade dele, e o `VecBlend` vai junto.
///
/// Devolve **uma sequência de z (fundo → topo) por blend expandido**, para o chamador enfileirar em
/// `vec_restack`: quem manda no z é a ÁRVORE (ADR-0110), e as entidades dos passos só nascem no
/// `sync` seguinte — por isso o pedido espera em vez de ser escrito aqui.
pub(crate) fn expand(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    pen: &mut ph2d_vec_edit::PenTool,
) -> Vec<Vec<VecPathId>> {
    let mut runs: Vec<Vec<VecPathId>> = Vec::new();
    let mut selection: Vec<VecPathId> = Vec::new();
    for (spine_id, e) in blends_touched_by(sim, map, pen.selected_paths()) {
        let Some(blend) = sim.world().get::<VecBlend>(e).cloned() else {
            continue;
        };
        let live = live_sources(scene, &blend);
        if live.len() < 2 {
            continue; // sem transição para materializar; o Release é quem limpa um blend morto
        }
        let worlds: Vec<VecPath> = live
            .iter()
            .filter_map(|&id| world(scene, xforms, id))
            .collect();
        let centers: Vec<[f64; 2]> = live
            .iter()
            .filter_map(|&id| center_of(scene, xforms, id))
            .collect();
        let offsets = spine_offsets_of(scene, spine_id, &centers, &blend);
        let links = cook_links(&worlds, blend.steps as usize, &offsets);
        scene.remove_path(spine_id);

        // A sequência de z, como o overlay a desenhava: fonte0 → passos do elo0 → fonte1 → … Os
        // passos de um elo entram logo ACIMA da fonte de trás DELE (a posição é buscada a cada elo:
        // a inserção anterior já moveu os índices).
        let mut run: Vec<VecPathId> = Vec::new();
        for (i, steps) in links.into_iter().enumerate() {
            run.push(live[i]);
            let at = scene
                .paths()
                .iter()
                .position(|p| p.id == live[i])
                .map_or(0, |z| z + 1);
            for (k, p) in steps.into_iter().enumerate() {
                run.push(scene.insert_path(at + k, p));
            }
        }
        run.push(live[live.len() - 1]);
        selection.extend(run.iter().copied());
        runs.push(run);
    }
    if !selection.is_empty() {
        // A seleção vira o RESULTADO inteiro (fontes + passos): é o que o Expand entregou, e o spine
        // que podia estar selecionado acabou de sair da cena — deixar a seleção num id morto faria o
        // painel falar de um objeto que não existe.
        pen.select_many(&selection);
    }
    runs
}

/// **Release** (ADR-0122 Fase D) — desfaz o blend: os passos somem e as formas-fonte ficam, como
/// antes de o objeto existir. Nada é materializado (≠ [`expand`]). `true` se algum blend foi solto.
///
/// Como os passos são VIRTUAIS (só o spine está na cena), soltar o blend é **remover o spine**: o
/// `vec_entities::sync` despawna a entidade, o `VecBlend` vai junto, o overlay do frame seguinte
/// nasce vazio, e as fontes — que nunca foram tocadas — seguem lá.
///
/// **É a saída do blend pelo canvas.** A linha não é selecionável no modo Select (ADR-0122), então o
/// Delete não a alcança; sem este botão, desfazer um blend exigiria caçar "Blend N" na Hierarquia ou
/// bater Ctrl+Z até antes da criação — e o Ctrl+Z leva junto tudo o que veio depois.
pub(crate) fn release(
    sim: &SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    pen: &mut ph2d_vec_edit::PenTool,
) -> bool {
    let mut sources: Vec<VecPathId> = Vec::new();
    let mut released = false;
    for (spine_id, e) in blends_touched_by(sim, map, pen.selected_paths()) {
        if let Some(blend) = sim.world().get::<VecBlend>(e) {
            sources.extend(live_sources(scene, blend));
        }
        scene.remove_path(spine_id);
        released = true;
    }
    if released {
        // A seleção passa às FONTES (o spine acabou de sair da cena): o artista fica com o material
        // na mão, pronto para re-blendar — e a seleção não aponta para um id morto.
        pen.select_many(&sources);
    }
    released
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
                    && let Some(auto) = mem.auto.as_mut()
                    && let Some(mv) = auto.get_mut(vi)
                {
                    shift_vertex_to(mv, e);
                }
            }
        }
    }
}
