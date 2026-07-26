//! **O grupo carrega o rig** (W-JG) — arrastar um corpo em repouso move o
//! conjunto articulado a que ele pertence.
//!
//! A W-AnchorFollow tornou a âncora de um joint **body-local**: ela segue o
//! corpo por construção. Isso fechou o *slide* do pivô e abriu, no mesmo
//! movimento, o item seguinte do padrão-ouro — se o corpo carrega a âncora,
//! mover **um** corpo de um par jointado separa as duas âncoras e deixa a pose
//! de repouso **violada**: o joint nasce esticado e o Play o resolve com um
//! puxão que o artista não autorou. Um rig é uma coisa só; arrastar um elo
//! dele arrasta o rig.
//!
//! # A lei é a do [`ph2d_physics_ecs::jointed_group`] — a MESMA do bake
//!
//! A W-BakeJoint já respondeu *"que corpos são um rig?"* e a resposta é o
//! **componente conexo DINÂMICO** do grafo de joints: corpo dinâmico conduz;
//! Static e Kinematic são **fronteiras** (alcançadas, nunca cruzadas). Uma
//! segunda resposta aqui é como o rig que o bake assa passaria a diferir do
//! rig que o arrasto move.
//!
//! Três consequências que essa lei acerta, e que valem a citação porque cada
//! uma seria um bug se fosse de outra forma:
//!
//! - Uma **corrente** de elos dinâmicos anda inteira, pegada por qualquer elo.
//! - Dois pêndulos no **MESMO gancho estático** ficam independentes — o gancho
//!   é uma parede entre eles, não um fio.
//! - O **gancho não anda** quando se arrasta o pêndulo. O joint até a parede
//!   estica, e isso é deliberado: afastar um rig da parede é um ato de autoria,
//!   e o overlay **desenha** o segmento esticado (a metade visível desta wave é
//!   o rig andando junto; o que não é carregado se vê pelo mesmo desenho).
//!
//! # Quando o rig é carregado (as três condições)
//!
//! Decididas pelo chamador e passadas como `carry_rig`:
//!
//! 1. **É um Translate.** *Mover* um corpo é o que a wave trata. Girar/escalar
//!    um rig precisaria decidir um pivô (o joint? a bbox do grupo?), e a
//!    semântica de multi-seleção local hoje gira cada membro em torno do
//!    PRÓPRIO centro — o que num rig não é rotação de rig nenhuma. Ficou
//!    **nomeado, não contrabandeado**.
//! 2. **O relógio está parado** (`!playhead.is_playing()`) — exatamente o gate
//!    de `sync_joint_pivots`, ou seja: o rig é carregado precisamente quando as
//!    âncoras seguem os corpos. Tocando, a pose é do SOLVER, o `settle`
//!    teleporta e a restrição é reimposta no tick seguinte de qualquer forma.
//! 3. **Alt não está apertado.** É o modificador de *escape* que este app já
//!    usa no mesmo gesto (`vec_snap`: Alt fura o ímã), e é a mesma palavra —
//!    *só isto, sem ajuda*. Ele é **inerte** no Translate do
//!    `compute_gizmo_transform`, então não há segundo significado a disputar.
//!
//! # A regra de PARENTESCO, e por que ela é conservadora
//!
//! O translate de grupo aplica o delta de MUNDO ao `Transform` **local** de
//! cada extra. Se um membro é descendente de outro, o de baixo anda duas vezes
//! (o pai já o carrega por herança) — o rig *explodiria*. Então: **o rig só
//! acrescenta um corpo quando nenhum OUTRO candidato lhe é parente** (nem
//! ancestral, nem descendente). O teste é simétrico e independente de ordem, e
//! erra para o lado honesto: quem não é carregado deixa o joint esticado — que
//! se vê —, em vez de andar em dobro, que lê como coisa quebrada.
//!
//! ⚠️ A regra vale só para o que **o rig acrescenta**. A multi-seleção
//! explícita segue como sempre (o artista escolheu aqueles objetos), e o
//! duplo-movimento pai+filho que ela permite é anterior a esta wave — nomeado
//! no handoff, não corrigido de contrabando.

use ph2d_ecs::{ChildOf, Entity, SimWorld, Transform};

use crate::app_state::GroupDragSnapshot;

/// Preenche `out` (o `App::group_drag_starts`) para um gizmo drag que acabou de
/// abrir: os OUTROS selecionados e, se `carry_rig`, o rig articulado do
/// conjunto.
///
/// `selected` é a seleção inteira (o `primary_bits` pode estar dentro dela e é
/// pulado — o snapshot dele vive no `GizmoDragState`).
///
/// **Uma porta, dois sítios de Down** (a alça do gizmo e o pick de canvas):
/// duas cópias desta regra é como arrastar pela alça passaria a carregar o rig
/// e arrastar pelo corpo, não.
pub(crate) fn seed_group_drag_starts(
    out: &mut Vec<GroupDragSnapshot>,
    sim: &mut SimWorld,
    primary_bits: u64,
    selected: &[u64],
    carry_rig: bool,
) {
    out.clear();
    // A regra de sempre: todo OUTRO selecionado que tenha `Transform`.
    for &sel in selected {
        if sel == primary_bits {
            continue;
        }
        if let Some(s) = snapshot_of(sim, sel) {
            out.push(s);
        }
    }
    if !carry_rig {
        return;
    }
    let mut seed: Vec<Entity> = Vec::with_capacity(out.len() + 1);
    seed.push(Entity::from_bits(primary_bits));
    seed.extend(out.iter().map(|s| Entity::from_bits(s.entity_bits)));
    let group = ph2d_physics_ecs::jointed_group(sim.world_mut(), &seed);
    // O conjunto que VAI se mover, decidido ANTES de empurrar qualquer coisa —
    // sem isso a regra de parentesco dependeria da ordem em que os candidatos
    // aparecem, e a ordem de `jointed_group` é por bits (id de alocação), que
    // não tem relação com a árvore.
    let candidates: Vec<u64> = seed
        .iter()
        .chain(group.iter())
        .map(|e| e.to_bits())
        .collect();
    for e in group {
        let bits = e.to_bits();
        if bits == primary_bits || out.iter().any(|s| s.entity_bits == bits) {
            continue;
        }
        if shares_a_chain(sim, e, &candidates) {
            continue;
        }
        if let Some(s) = snapshot_of(sim, bits) {
            out.push(s);
        }
    }
}

/// O snapshot de início que o `advance_gizmo_drag` consome, ou `None` se a
/// entidade não tem `Transform` (nada a mover).
fn snapshot_of(sim: &SimWorld, bits: u64) -> Option<GroupDragSnapshot> {
    let e = Entity::from_bits(bits);
    let t = sim.world().get::<Transform>(e)?;
    let start_transform = ph2d_editor::TransformSnapshot {
        translation: [t.translation.x, t.translation.y],
        rotation: t.rotation,
        scale: [t.scale.x, t.scale.y],
    };
    let pw = ph2d_ecs::parent_world_transform(sim.world(), e);
    Some(GroupDragSnapshot {
        entity_bits: bits,
        start_transform,
        parent_world: ph2d_editor::TransformSnapshot {
            translation: [pw.translation.x, pw.translation.y],
            rotation: pw.rotation,
            scale: [pw.scale.x, pw.scale.y],
        },
    })
}

/// `e` é parente (ancestral OU descendente) de algum outro membro de
/// `candidates`?
///
/// Simétrico de propósito: as duas direções são o MESMO defeito visto dos dois
/// lados (o de baixo anda duas vezes), então perguntar só para cima deixaria
/// passar o caso em que o corpo arrastado é o descendente.
fn shares_a_chain(sim: &SimWorld, e: Entity, candidates: &[u64]) -> bool {
    // Para cima, a partir de `e`.
    if let Some(a) = first_ancestor_in(sim, e, candidates) {
        debug_assert_ne!(a, e);
        return true;
    }
    // E para baixo: `e` é ancestral de algum outro candidato? Perguntado
    // subindo a partir de cada um deles — a árvore só tem ponteiro para cima,
    // e o número de candidatos é o tamanho de um rig.
    candidates.iter().any(|&other| {
        other != e.to_bits()
            && first_ancestor_in(sim, Entity::from_bits(other), &[e.to_bits()]).is_some()
    })
}

/// O primeiro ancestral ESTRITO de `e` que está em `set`, se houver.
fn first_ancestor_in(sim: &SimWorld, e: Entity, set: &[u64]) -> Option<Entity> {
    let mut cur = sim.world().get::<ChildOf>(e).map(ChildOf::parent);
    while let Some(p) = cur {
        if set.contains(&p.to_bits()) {
            return Some(p);
        }
        cur = sim.world().get::<ChildOf>(p).map(ChildOf::parent);
    }
    None
}

#[cfg(test)]
#[path = "joint_rig_drag_tests.rs"]
mod tests;
