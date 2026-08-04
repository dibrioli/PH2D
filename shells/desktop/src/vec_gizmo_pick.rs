//! **O que o PONTEIRO acha** — o hit-test de canvas e o marquee do módulo vetorial.
//!
//! Irmão do [`super`] pelo teto de 600 LOC da shell, cortado por ASSUNTO: lá mora *o que o gizmo
//! MOSTRA* (a caixa, o pivô, a rotação de uma forma); aqui *o que o ponteiro ACHA* (que forma está
//! sob este ponto, e quais o retângulo do marquee apanha).
//!
//! ⚠️ As duas metades leem os MESMOS fatos derivados — a pose que o auto layout deu, os intervalos
//! das molduras —, e é isso que impede a caixa de aparecer num sítio e o clique de pegar noutro.

use super::*;

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
    live: &LiveGeometry,
    view_state: &VecViewState,
    entity: Entity,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> bool {
    let Some(vp) = sim.world().get::<VecPathRef>(entity) else {
        return false;
    };
    contains_path(sim, scene, live, view_state, entity, vp.0, p, stroke_hit_r)
}

/// Amostras por segmento na varredura de proximidade do traço.
const STROKE_SAMPLES: u32 = 24;

/// **A forma que se VÊ é a que se PEGA** — `p` (mundo) contra a geometria DERIVADA de um
/// caminho, que já vem assada em mundo (a pose viaja dentro dela).
///
/// A pergunta *"o que está desenhado aqui?"* é feita ao MESMO mapa que o
/// [`ph2d_vec_render::dispatch`] consome; nada aqui re-deriva um offset. O ponto não volta ao
/// espaço local porque a derivada não tem um: ela é de mundo por construção.
///
/// ⚠️ Uma entrada **VAZIA** é a aniquilação (o offset comeu a forma) e sai `false` — nada
/// desenhado, nada pego. É por isso que o `offset_live::recook` insere a entrada mesmo vazia:
/// ausente significaria *"desenhe/pegue a fonte"*, e a forma voltaria a ser clicável onde a
/// tela está limpa.
fn hits_derived(items: &[ph2d_vec_scene::VecPath], p: [f64; 2], stroke_hit_r: f64) -> bool {
    items.iter().any(|item| {
        if ph2d_vec_scene::contains_point(item, p) {
            return true;
        }
        let Some((_, _, d2)) = ph2d_vec_scene::nearest_point_on_path(item, p, STROKE_SAMPLES)
        else {
            return false;
        };
        // Mesmas duas folgas do caminho da FONTE (a tinta que se vê conta; uma linha não tem
        // interior). O raio já é de mundo e a geometria também — não há escala a cruzar.
        let half_ink = item.stroke.as_ref().map_or(0.0, |s| s.width * 0.5);
        let open = (0..item.contour_count()).all(|c| !matches!(item.contour(c), Some((_, true))));
        let slop = if open {
            stroke_hit_r * OPEN_PATH_HIT_K
        } else {
            stroke_hit_r
        };
        d2.sqrt() <= slop + half_ink
    })
}

/// `p` (mundo) pega o path `id`: no INTERIOR (formas fechadas) OU a ≤ `stroke_hit_r`
/// do TRAÇO (formas abertas e a borda de fechadas).
#[allow(clippy::too_many_arguments)] // as mesmas entradas que DESENHAR um caminho pede
fn contains_path(
    sim: &SimWorld,
    scene: &VecScene,
    live: &LiveGeometry,
    view_state: &VecViewState,
    entity: Entity,
    id: VecPathId,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> bool {
    // A DERIVADA manda quando existe: com um offset vivo o documento guarda a curva autorada e
    // a tela mostra outra coisa, então apalpar a fonte pegaria a forma onde ela NÃO está (e
    // deixaria de pegá-la onde está). O mesmo `live` que o renderer consome, pela mesma chave.
    if let Some(items) = live.get(&id) {
        return hits_derived(items, [f64::from(p[0]), f64::from(p[1])], stroke_hit_r);
    }
    // ⚠️ **A POSE do auto layout entra aqui.** A forma colocada é DESENHADA pela geometria assada
    // (o ramo acima), mas quando o pick não recebe essa geometria o que sobra é a pose autorada —
    // e ela não se mexeu: o clique procurava a forma no lugar de onde ela saiu. A pose vem DEPOIS
    // do transform do path, porque age sobre a geometria já posta no mundo.
    let x = xform_of_transform(world_transform(sim, entity)).then(&view_state.layout_pose(id));
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
///
/// # Uma MOLDURA é o fundo do que ela contém — para desenhar E para apontar
///
/// ⚠️ Na pilha de z crua o contêiner é o ÚLTIMO membro da própria sub-árvore (é o que emparelha
/// o push e o pop da camada de recorte), logo o mais À FRENTE dela. O renderer sabe disso e
/// **antecipa** o desenho da moldura para a abertura do intervalo; o apontar não sabia, e o
/// retângulo dela ganhava todo clique dos próprios filhos — *"quando dentro do Frame não consigo
/// selecionar as formas"* (Enio, 2026-08-02).
///
/// Duas respostas para *"onde nesta pilha está esta moldura?"*: desenhada no fundo, apontada na
/// frente. A cura lê a MESMA fonte que o renderer (`VecViewState::clips`, que é literalmente
/// *quem foi antecipado*) e demove quem ela nomeia para trás dos próprios descendentes.
#[must_use]
pub(crate) fn pick_all_at_world(
    sim: &SimWorld,
    scene: &VecScene,
    live: &LiveGeometry,
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
            && contains_path(sim, scene, live, view_state, e, path.id, p, stroke_hit_r)
        {
            out.push((bits, path.id));
        }
    }
    demote_hoisted_frames(sim, view_state, &mut out);
    out.into_iter().map(|(bits, _)| bits).collect()
}

/// Põe cada moldura ANTECIPADA atrás dos descendentes dela que também foram acertados.
///
/// A chave é *quantos dos outros acertos descendem de mim* — e ela é uma ordem total válida
/// porque a descendência é uma floresta: se `a` é ancestral de `b`, todo descendente de `b` é
/// descendente de `a`, **mais o próprio `b`**, logo a contagem de `a` é estritamente maior. A
/// ordenação é ESTÁVEL, então dois irmãos sem parentesco mantêm a ordem de z que já tinham.
///
/// ⚠️ **Só quem o renderer antecipou é demovido, e a lista é a MESMA que ele consome.** Desde que
/// o filho passou a desenhar sobre o pai (a lei de Godot), *todo* pai com descendente vetorial é
/// antecipado — e por perguntar à lista, e não a um predicado próprio, o apontar acompanhou a
/// mudança do desenho **sem uma linha aqui**. Um segundo predicado (*"tem filhos?"*) responderia
/// hoje o mesmo e divergiria no dia em que a antecipação ganhasse uma condição.
fn demote_hoisted_frames(
    sim: &SimWorld,
    view_state: &VecViewState,
    hits: &mut [(u64, ph2d_vec_scene::VecPathId)],
) {
    let hoisted = |id| view_state.parent_spans.iter().any(|c| c.parent == id);
    let depth_below = |me: u64, id| {
        if !hoisted(id) {
            return 0usize;
        }
        hits.iter()
            .filter(|(other, _)| *other != me && is_descendant_of(sim, *other, me))
            .count()
    };
    let keys: Vec<usize> = hits
        .iter()
        .map(|(bits, id)| depth_below(*bits, *id))
        .collect();
    let mut idx: Vec<usize> = (0..hits.len()).collect();
    idx.sort_by_key(|&i| keys[i]); // estável: empate mantém a ordem de z
    let reordered: Vec<_> = idx.into_iter().map(|i| hits[i]).collect();
    hits.copy_from_slice(&reordered);
}

/// `who` desce de `root` na árvore do editor?
fn is_descendant_of(sim: &SimWorld, who: u64, root: u64) -> bool {
    let w = sim.world();
    let mut cur = Entity::from_bits(who);
    let root = Entity::from_bits(root);
    for _ in 0..MAX_PICK_DEPTH {
        let Some(parent) = w.get::<ph2d_ecs::ChildOf>(cur).map(|c| c.parent()) else {
            return false;
        };
        if parent == root {
            return true;
        }
        cur = parent;
    }
    false
}

/// Teto de profundidade da caminhada de ancestral — defesa contra árvore corrompida, o mesmo
/// número e pelo mesmo motivo do [`crate::vec_selection`].
const MAX_PICK_DEPTH: usize = 64;

/// A forma mais ao topo sob `p`, ou `None`. Conveniência dos testes: o canvas
/// consome a lista inteira ([`pick_all_at_world`]) para poder ciclar entre
/// sobreposições.
#[cfg(test)]
#[allow(dead_code)]
#[must_use]
fn pick_at_world(
    sim: &SimWorld,
    scene: &VecScene,
    live: &LiveGeometry,
    view_state: &VecViewState,
    map: &VecEntityMap,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> Option<u64> {
    pick_all_at_world(sim, scene, live, view_state, map, p, stroke_hit_r)
        .into_iter()
        .next()
}

/// A caixa de MUNDO que o marquee compara — a da geometria DERIVADA quando existe uma, a da
/// fonte (bbox local × pose) quando não. `None` = caminho sem geometria nenhuma.
fn world_bbox(
    sim: &SimWorld,
    scene: &VecScene,
    live: &LiveGeometry,
    e: Entity,
    id: VecPathId,
) -> Option<([f64; 2], [f64; 2])> {
    if let Some(items) = live.get(&id) {
        // Já é de mundo: a união das caixas das peças, sem passar pela pose (assá-la de novo
        // colocaria a caixa duas vezes longe da forma).
        return items
            .iter()
            .filter_map(|it| ph2d_vec_scene::curve_bbox_in_frame(it, 1.0, 0.0))
            .reduce(|(a_lo, a_hi), (b_lo, b_hi)| {
                (
                    [a_lo[0].min(b_lo[0]), a_lo[1].min(b_lo[1])],
                    [a_hi[0].max(b_hi[0]), a_hi[1].max(b_hi[1])],
                )
            });
    }
    let (lo, hi) = scene.path_curve_bbox(id)?;
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
    Some((wlo, whi))
}

/// Toda forma vetorial cuja bbox de mundo intersecta o retângulo — o marquee.
#[must_use]
pub(crate) fn pick_in_world_rect(
    sim: &SimWorld,
    scene: &VecScene,
    live: &LiveGeometry,
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
        let Some((wlo, whi)) = world_bbox(sim, scene, live, e, path.id) else {
            continue;
        };
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

/// Os gates do que o ponteiro ACHA.
#[cfg(test)]
#[path = "vec_gizmo_pick_tests.rs"]
mod tests;

/// Os gates da área de captura de um traço, e da lei de quem ganha o clique.
#[cfg(test)]
#[path = "vec_gizmo_view_hit_tests.rs"]
mod hit_tests;

/// Os gates do pick com um OFFSET vivo — a geometria DERIVADA manda.
#[cfg(test)]
#[path = "vec_offset_pick_tests.rs"]
mod offset_pick_tests;
