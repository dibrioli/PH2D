//! ⭐ **O QUE O GESTO AGARRA, E O QUE ELE MOVE** — a metade de **gizmo** da ponte com a cena.
//!
//! ⚠️ O corte saiu do teto de LOC do shell (HR-18) e a fronteira **não é arbitrária**: o arquivo pai
//! responde *o que a peça É* (cozer, publicar, drenar intenções); este responde *quem está debaixo do
//! ponteiro e para onde ele vai*. As duas metades já se liam assim — o pai chama, este obedece.
//!
//! ⚠️ **Módulo-filho por `#[path]`, e não uma crate**: as funções aqui tocam o mesmo `SimWorld`, o
//! mesmo estado de smoke e o mesmo `SelectRequest` do pai. Uma fronteira de crate obrigaria a tornar
//! público o que hoje é detalhe de um arquivo.

use ph2d_ecs::SimWorld;
use ph2d_field::FieldDoc;
use ph2d_field_ecs::{FieldNode, FieldObject};

use super::SelectRequest;
use crate::field3d_smoke::with_smoke;

/// ⭐ **O gesto vale para a SELEÇÃO INTEIRA** (W27), em torno do pivô que o gizmo mostra.
///
/// ⚠️ **O defeito que isto fecha:** a seleção deste módulo é a do app (clicar na Hierarquia com
/// `Ctrl` escolhe vários, e a fileira de operações já contava com isso desde a W9) — e o arrasto
/// movia **um**. Duas linhas acesas, uma a andar: o artista lê aquilo como o gizmo estar partido.
///
/// ⚠️ **O pivô é o do GIZMO, não o de cada nó**, e é o que faz um giro girar o conjunto em vez de
/// cada peça sobre si mesma. Com um nó só, o pivô **é** a origem dele e as duas leis coincidem
/// byte-a-byte (ver [`ph2d_field_ecs::rotate_world_about`]) — não há caso especial.
pub(super) fn apply_motion(
    sim: &mut SimWorld,
    primary: u64,
    chosen: &[bevy_ecs::entity::Entity],
    motion: crate::field3d_gizmo::Motion,
) {
    let world = sim.world_mut();
    let primary = bevy_ecs::entity::Entity::from_bits(primary);
    // ⚠️ **Quem está agarrado entra sempre**, mesmo que a seleção do app já não o contenha: o gesto
    // foi começado nele, e o `Grip` congelou-o.
    let mut all: Vec<bevy_ecs::entity::Entity> = vec![primary];
    all.extend(chosen.iter().copied().filter(|e| *e != primary));
    // ⚠️ E um filho de outro escolhido **não anda duas vezes** — ver `top_level`.
    let targets: Vec<bevy_ecs::entity::Entity> = ph2d_field_ecs::top_level(world, &all)
        .into_iter()
        .filter(|e| movable(world, *e))
        .collect();
    let pivot = selection_pivot(world, &targets);
    for e in targets {
        match motion {
            crate::field3d_gizmo::Motion::Translate(d) => {
                ph2d_field_ecs::translate_world(world, e, d);
            }
            crate::field3d_gizmo::Motion::Rotate { axis, angle } => {
                ph2d_field_ecs::rotate_world_about(world, e, axis, angle, pivot);
            }
            crate::field3d_gizmo::Motion::Scale(f) => {
                ph2d_field_ecs::scale_about(world, e, f, pivot);
            }
        }
    }
}

/// A mesma porta, aberta para os gates da W27 — o caminho real (`ecs_bridge`) pergunta pelo estado
/// do smoke, que um teste não encena.
#[cfg(test)]
pub(crate) fn apply_motion_for_test(
    sim: &mut SimWorld,
    primary: u64,
    chosen: &[bevy_ecs::entity::Entity],
    motion: crate::field3d_gizmo::Motion,
) {
    apply_motion(sim, primary, chosen, motion);
}

/// ⭐ **Este nó pode ser mexido por um gesto?** — a pergunta única, e ela junta duas leis da CASA.
///
/// | lei | de onde vem | o que significa aqui |
/// |---|---|---|
/// | **escondido** (W28) | o olho da Hierarquia ([`ph2d_ecs::Visibility`]) | mover o que a peça não mostra é um gesto sem resposta na tela |
/// | **trancado** (W29) | o cadeado ([`ph2d_ecs::is_locked_for_edit`]) | *"Cadeado trava apenas o objeto"* — Enio, 2026-05-26, escrito no doc do componente |
///
/// ⚠️ **O predicado do cadeado é o da casa, não um novo**: ele já é consultado pelo gizmo 2D, pelo
/// Flip, pelas juntas e pelo vetorial — e ele **sobe a cadeia** à procura de um antepassado com
/// `GroupedChildren`, que é o que trancar um grupo significa. Escrever aqui um `get::<Locked>` seria
/// a segunda resposta a *"isto pode mexer?"*, e ela nasceria já sem a metade do grupo.
pub(super) fn movable(world: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity) -> bool {
    !ph2d_field_ecs::is_hidden(world, e) && !ph2d_ecs::is_locked_for_edit(world, e)
}

/// **Onde o gizmo pousa numa seleção**: a média das origens de mundo dos escolhidos.
///
/// ⚠️ A média das ORIGENS, e não o centro das caixas: a caixa de um campo implícito custa uma
/// varredura, e o que o artista agarra é o que ele vê — as setas estão sobre as origens.
pub(crate) fn selection_pivot(
    world: &bevy_ecs::world::World,
    targets: &[bevy_ecs::entity::Entity],
) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    let mut n = 0.0f32;
    for e in targets {
        let t = ph2d_field_ecs::world_xform(world, *e).translation;
        for k in 0..3 {
            sum[k] += t[k];
        }
        n += 1.0;
    }
    if n == 0.0 {
        return [0.0; 3];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Resolve um clique guardado: `Some(Entity)` no que estiver sob ele, `Some(Clear)` no fundo.
pub(super) fn resolve_pick(
    sim: &mut SimWorld,
    doc: Option<&FieldDoc>,
    px: [f32; 2],
) -> Option<SelectRequest> {
    let (cam, area) = with_smoke(|s| (s.cam, s.area))?;
    let area = area?;
    let doc = doc?;
    let screen = ph2d_field_render::Screen::new(
        area.w.round().max(1.0) as u32,
        area.h.round().max(1.0) as u32,
        cam.half_extent,
    );
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e)?;
    Some(
        crate::field3d_pick::node_under(world, root, doc, &cam, screen, px)
            .map_or(SelectRequest::Clear, |e| SelectRequest::Entity(e.to_bits())),
    )
}

/// ⭐ **Onde o gizmo tem de aparecer** — a pose de MUNDO do nó selecionado.
///
/// ⚠️ A seleção é a do **app** (`hero.gizmo.selection`), e não uma deste módulo: clicar numa linha
/// da Hierarquia é o gesto que faz as setas aparecerem. Uma seleção própria seria uma segunda ideia
/// de *"o que está selecionado"* dentro do mesmo aplicativo, e as duas divergiriam no primeiro
/// clique.
///
/// Devolve `None` quando o selecionado não é um nó de modelagem — um sprite selecionado não pode
/// fazer aparecer um gizmo 3D em cima dele.
pub(super) fn anchor_for(
    sim: &mut SimWorld,
    selected: Option<u64>,
    chosen: &[bevy_ecs::entity::Entity],
) -> Option<crate::field3d_gizmo::Anchor> {
    let bits = selected?;
    let frame = with_smoke(|s| s.gizmo_frame).unwrap_or_default();
    let entity = bevy_ecs::entity::Entity::from_bits(bits);
    let world = sim.world_mut();
    world.get::<FieldNode>(entity)?;
    // ⭐ **Um nó ESCONDIDO ou TRANCADO não tem gizmo** (W28/W29): setas que não mexem em nada são um
    // gesto sem resposta na tela. A linha da Hierarquia continua lá, com o olho e o cadeado a
    // dizerem porquê — e é por eles que se desfaz.
    //
    // ⚠️ **Aqui o módulo DIVERGE do gizmo 2D da casa, e é de propósito.** Lá o desenho fica e o
    // *Down* é recusado; aqui as alças são o único sinal de que o gesto existe, e alças que não
    // agarram seriam a mesma coisa que um botão pintado e morto. O que se ganha lá — *«ele está
    // ali»* — este módulo ganha na Hierarquia, que é onde o cadeado se vê.
    if !movable(world, entity) {
        return None;
    }
    let pose = ph2d_field_ecs::world_xform(world, entity);
    // ⭐ **Com vários escolhidos, o gizmo pousa no MEIO deles** (W27) — e é esse ponto que o giro e
    // o tamanho usam como pivô. Com um só, a média é a própria origem: a lei antiga é o caso
    // particular desta, e não um ramo à parte.
    //
    // ⚠️ Os **eixos** continuam a ser os do principal: é o que mantém o seletor Global/Local a
    // significar alguma coisa numa seleção (o «Local» de um conjunto é o do objeto ativo, que é o
    // que todo modelador faz).
    let mut all: Vec<bevy_ecs::entity::Entity> = vec![entity];
    all.extend(chosen.iter().copied().filter(|e| *e != entity));
    let targets = ph2d_field_ecs::top_level(world, &all);
    Some(crate::field3d_gizmo::Anchor {
        entity: bits,
        origin: selection_pivot(world, &targets),
        // ⚠️ Os eixos viajam **já resolvidos**: a lei do gizmo não sabe que existe uma escolha de
        // referencial, e quem a faz é quem tem a pose. Ver `Anchor::axes`.
        axes: frame.axes(pose.rotation),
    })
}

/// ⭐ **Duplicar um nó** — a porta ÚNICA, e os dois lugares que duplicam chamam-na.
///
/// ⚠️ **Uma lei, dois chamadores**: o botão do painel e a linha *Duplicate* da Hierarquia. Cada um
/// com a sua conta seria a segunda resposta a *"onde vai a cópia?"*, e elas divergiriam no primeiro
/// ajuste — com o artista a ver o mesmo gesto fazer duas coisas conforme por onde o pediu. É a mesma
/// lição que o bloco vetorial da Hierarquia já tem escrita ao lado.
///
/// # A cópia sai UM DEGRAU da grelha, para a direita da TELA
///
/// ⚠️ Não é decoração, e a alternativa foi considerada: **duplicar em cima do original** é o que o
/// Blender faz — e ele resolve o resto entrando logo em modo de mover. Aqui não há esse modo, então
/// uma cópia exatamente por baixo seria **um botão que parece não fazer nada**: a única prova seria
/// uma linha nova na Hierarquia.
///
/// O **quanto** é o degrau da grelha (derivado do enquadramento: o menor número redondo que ainda se
/// consegue mirar); o **para onde** é a direita da câmera, que é para onde «o próximo» vai em
/// qualquer arrumação.
///
/// Devolve os bits da cópia, para quem chamar a poder selecionar. `None` quando não há o que
/// duplicar (ver `ph2d_field_ecs::duplicate`: a raiz **é** a peça).
pub(crate) fn duplicate_node(
    world: &mut bevy_ecs::world::World,
    node: bevy_ecs::entity::Entity,
) -> Option<u64> {
    let (cam, screen) = view()?;
    duplicate_with_view(world, node, &cam, screen)
}

/// A mesma lei, **com a vista em mãos** — e é a separação que o resto do módulo já usa.
///
/// ⚠️ Ela existe para o gate: [`duplicate_node`] lê a câmera do estado do módulo, e um teste não
/// consegue (nem deve) encená-lo. Aqui a vista entra por parâmetro e o resto é o caminho de
/// produção inteiro.
pub(crate) fn duplicate_with_view(
    world: &mut bevy_ecs::world::World,
    node: bevy_ecs::entity::Entity,
    cam: &ph2d_field_render::Orbit,
    screen: ph2d_field_render::Screen,
) -> Option<u64> {
    let (right, _, _) = cam.basis();
    let step = crate::field3d_gizmo::snap_step(screen);
    let off = [right[0] * step, right[1] * step, right[2] * step];
    ph2d_field_ecs::duplicate(world, node, off).map(|e| e.to_bits())
}

/// A câmera e o enquadramento deste quadro — `None` quando o módulo não está armado.
fn view() -> Option<(ph2d_field_render::Orbit, ph2d_field_render::Screen)> {
    with_smoke(|s| {
        let a = s
            .area
            .unwrap_or(ph2d_editor::zones::Rect::new(0.0, 0.0, 1.0, 1.0));
        (
            s.cam,
            ph2d_field_render::Screen::new(
                a.w.round().max(1.0) as u32,
                a.h.round().max(1.0) as u32,
                s.cam.half_extent,
            ),
        )
    })
}
