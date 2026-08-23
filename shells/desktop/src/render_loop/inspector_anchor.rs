//! **§12 Sockets / Named Anchors** ([ADR-0072]) — o snapshot que a seção lê e o commit que ela
//! escreve. Irmão do [`super::inspector_ordering`], pela mesma razão dele.
//!
//! # A conversão de unidades mora AQUI, num sítio só
//!
//! ⚠️ O componente guarda a pose num [`ph2d_ecs::Transform`], e **um `Transform` é em metros e
//! radianos** — é a lei da casa, e não há porta de escala. O artista, porém, mede a boca de uma
//! arma **na imagem**, em pixels, e escreve a rotação em graus. Por isso o snapshot converte na
//! saída e o commit converte na entrada, **nas duas pontas do mesmo ficheiro**: duplicar a
//! conversão faria o número que ele lê e o que o motor aplica divergirem no dia em que uma
//! metade fosse corrigida.
//!
//! ⚠️ `bounds`/`center` NÃO se convertem: eles já são pixels da fonte, como o
//! `Sprite::region_rect`, e pela mesma razão — são uma medida tirada da imagem.
//!
//! [ADR-0072]: ../../../../docs/architecture/decisions/0072-named-anchor-unification.md

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, NamedAnchor, NamedAnchorList, SimWorld, World};
use ph2d_editor::{AnchorFieldEdit, InspectorAnchorInfo, InspectorAnchorRow, Toast};

use super::inspector_ordering::queue_set;

const NAMED_ANCHOR_LIST: &str = "ph2d::ecs::NamedAnchorList";
const ANCHOR_MOUNT: &str = "ph2d::ecs::AnchorMount";
const ANCHOR_VISIBILITY: &str = "ph2d::ecs::AnchorVisibility";
const TRANSFORM: &str = "ph2d::ecs::Transform";

/// Constrói o snapshot da §12, ou `None` quando a entidade não é digna de Inspector.
pub(super) fn build_anchor_info(
    world: &World,
    entity_bits: u64,
    selected: &[u64],
    selected_count: usize,
    pixels_per_meter: f32,
) -> Option<InspectorAnchorInfo> {
    let entity = Entity::from_bits(entity_bits);
    world.get::<ph2d_ecs::Transform>(entity)?;
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let list = world.get::<NamedAnchorList>(entity);
    // **Quem monta nas âncoras DESTA entidade** — uma passagem pelos filhos, e depois a contagem
    // por nome. ⚠️ Uma passagem por âncora seria `anchors × filhos` e este snapshot é por-quadro.
    let mounted: Vec<&str> = world
        .get::<ph2d_ecs::Children>(entity)
        .map(|ch| {
            ch.iter()
                .filter_map(|c| world.get::<ph2d_ecs::AnchorMount>(*c))
                .filter(|m| m.is_bound())
                .map(|m| m.anchor.as_str())
                .collect()
        })
        .unwrap_or_default();
    let rows: Vec<InspectorAnchorRow> = list
        .map(|l| {
            l.iter()
                .map(|a| InspectorAnchorRow {
                    riders: mounted.iter().filter(|n| **n == a.name).count(),
                    name: a.name.clone(),
                    pos: [
                        a.transform.translation.x * ppm,
                        a.transform.translation.y * ppm,
                    ],
                    rot_deg: a.transform.rotation.to_degrees(),
                    bounds: a.bounds,
                    center: a.center,
                })
                .collect()
        })
        .unwrap_or_default();
    // Divergência: com seleção múltipla, basta uma lista diferente para o «Mixed».
    let mut mixed = false;
    if selected.len() > 1 {
        let mine = list.cloned().unwrap_or_default();
        for &bits in selected {
            let other = world
                .get::<NamedAnchorList>(Entity::from_bits(bits))
                .cloned()
                .unwrap_or_default();
            mixed |= other != mine;
        }
    }
    // **A outra metade da seção, e o dono dela é o PAI** (ADR-0072 §2.6).
    //
    // ⚠️ As âncoras oferecidas saem de `ph2d_ecs::anchor_names` — a MESMA função que a API de
    // runtime e o MCP usam. Reimplementar aqui um `iter().map(name)` daria uma segunda resposta
    // a «que âncoras tem esta entidade», e é essa a família de defeito que esta linha paga.
    let parent_anchors = world
        .get::<ph2d_ecs::ChildOf>(entity)
        .map(|c| ph2d_ecs::anchor_names(world, c.parent()))
        .unwrap_or_default();
    // ⚠️ Um vínculo com nome VAZIO é «não monta», e não um vínculo — senão o painel acenderia o
    // aviso de pendurado sobre o estado que o artista acabou de escolher.
    let mount = world
        .get::<ph2d_ecs::AnchorMount>(entity)
        .filter(|m| m.is_bound())
        .map(|m| m.anchor.clone());
    // ⚠️ **O deslocamento é o `Transform` LOCAL, e não uma diferença de poses de mundo.** Quando
    // esta entidade monta, o quadro do pai JÁ inclui a âncora (`ph2d_ecs::mount_state`), então a
    // pose local dela é, por construção, o quanto ela está fora dela. Calcular por subtração de
    // mundo daria a mesma resposta por um caminho que pode discordar — e é essa a família de
    // defeito que esta linha paga.
    let mount_offset = world
        .get::<ph2d_ecs::Transform>(entity)
        .map(|t| [t.translation.x * ppm, t.translation.y * ppm])
        .unwrap_or([0.0, 0.0]);
    let vis = world.get::<ph2d_ecs::AnchorVisibility>(entity).copied();
    Some(InspectorAnchorInfo {
        entity_bits,
        rows,
        present: list.is_some(),
        selected_count,
        mixed,
        parent_anchors,
        mount,
        mount_offset,
        vis_in_editor: vis.is_some_and(|v| v.in_editor),
        vis_at_runtime: vis.is_some_and(|v| v.at_runtime),
    })
}

/// **Pousar quem monta EM CIMA da âncora** — a operação que dois gestos partilham (Enio,
/// 2026-08-23): escolher uma âncora no seletor, e carregar em «Reset to Anchor».
///
/// ⚠️ **Zera a POSIÇÃO e mais nada.** A rotação e a escala do filho continuam a ser dele: uma
/// espada tem um ângulo próprio dentro da mão, e repô-lo seria decidir no lugar do artista.
///
/// ⚠️ **Zero EXACTO, e é isso que torna `is_off_anchor` uma comparação exacta.** O único caminho
/// que escreve zero é este; qualquer arrasto deixa resíduo. Um epsilon do outro lado esconderia
/// um deslocamento real de meio pixel.
fn queue_snap_to_anchor(
    sim: &SimWorld,
    entity_bits: u64,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let Some(mut t) = sim.world().get::<ph2d_ecs::Transform>(entity).copied() else {
        return;
    };
    t.translation = ph2d_core::Vec2::ZERO;
    queue_set(queue, registry, entity_bits, TRANSFORM, &t);
}

/// Aplica uma [`AnchorFieldEdit`].
///
/// ⚠️ **Ler-modificar-escrever a lista inteira**: mexer numa âncora não pode repor as outras.
/// E toda edição **anexa** o componente se ele faltar — mexer num controlo da seção é autorar.
///
/// Devolve um aviso quando a edição foi **recusada** (nome inválido, repetido, ou o cap de 64).
/// ⚠️ *Recusar em silêncio seria pior que aceitar*: o artista escreveria um nome, veria a lista
/// não mudar, e não saberia porquê.
pub(super) fn apply_anchor_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: &AnchorFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
    pixels_per_meter: f32,
) -> Option<Toast> {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let entity = Entity::from_bits(entity_bits);
    let mut list = sim
        .world()
        .get::<NamedAnchorList>(entity)
        .cloned()
        .unwrap_or_default();

    match edit {
        AnchorFieldEdit::Add => {
            let name = list.next_free_name();
            if let Err(e) = list.insert(NamedAnchor::socket(name)) {
                return Some(Toast::error(format!("Anchor not added: {}", describe(e))));
            }
        }
        AnchorFieldEdit::Remove(i) => {
            let name = list.iter().nth(usize::from(*i)).map(|a| a.name.clone())?;
            list.remove(&name);
            // ⚠️ A lista pode ficar VAZIA e o componente fica — anexado-e-vazio é um estado
            // que o artista criou, e apagá-lo por ele seria decidir no lugar dele. O «Remove»
            // que retira o componente inteiro não existe de propósito: ele não tem gesto.
        }
        AnchorFieldEdit::Rename(i, new_name) => {
            let idx = usize::from(*i);
            list.iter().nth(idx)?;
            if let Err(e) = ph2d_ecs::validate_anchor_name(new_name) {
                return Some(Toast::error(format!(
                    "Anchor name rejected: {}",
                    describe(e)
                )));
            }
            if list
                .iter()
                .enumerate()
                .any(|(j, a)| j != idx && a.name == *new_name)
            {
                return Some(Toast::error(format!(
                    "Anchor name '{new_name}' is already used on this sprite"
                )));
            }
            if let Some(a) = list.0.get_mut(idx) {
                a.name = new_name.clone();
            }
        }
        AnchorFieldEdit::Mount(name) => {
            // ⚠️ **Esta edição escreve OUTRO componente**, na entidade que monta — e sai por
            // aqui sem tocar na lista de âncoras. Gravar a lista também faria toda escolha de
            // montagem contar como uma edição das âncoras deste objeto: o undo teria dois passos
            // onde houve um gesto, e um sprite sem âncoras nenhumas ganharia um componente vazio
            // só por ter sido preso a uma mão.
            //
            // ⚠️ Escolher «—» escreve um vínculo VAZIO em vez de retirar o componente. É a mesma
            // decisão que o `Remove` de uma âncora toma (anexado-e-vazio é um estado que o
            // artista criou), e um sprite com `AnchorMount { anchor: "" }` desenha byte-idêntico
            // a um que nunca o teve.
            let mount = ph2d_ecs::AnchorMount {
                anchor: name.clone().unwrap_or_default(),
            };
            queue_set(queue, registry, entity_bits, ANCHOR_MOUNT, &mount);
            // **E POUSA-O na âncora** (Enio, 2026-08-23: *«o objeto deve ser colocado na mesma
            // posição da âncora no momento da atribuição»*).
            //
            // ⚠️ **Só ao ESCOLHER uma âncora, nunca ao escolher «—».** Desmontar com um snap
            // atiraria o objeto para a origem do pai sem ninguém o pedir; desmontar é largar, e
            // largar deixa as coisas onde estão.
            if name.is_some() {
                queue_snap_to_anchor(sim, entity_bits, queue, registry);
            }
            return None;
        }
        AnchorFieldEdit::SnapToAnchor => {
            queue_snap_to_anchor(sim, entity_bits, queue, registry);
            return None;
        }
        AnchorFieldEdit::VisibilityInEditor(_) | AnchorFieldEdit::VisibilityAtRuntime(_) => {
            // ⚠️ Ler-modificar-escrever, como a lista: ligar uma caixa não pode repor a outra.
            let mut vis = sim
                .world()
                .get::<ph2d_ecs::AnchorVisibility>(entity)
                .copied()
                .unwrap_or_default();
            match edit {
                AnchorFieldEdit::VisibilityInEditor(on) => vis.in_editor = *on,
                AnchorFieldEdit::VisibilityAtRuntime(on) => vis.at_runtime = *on,
                _ => unreachable!("o braco acima ja' filtrou as duas variantes"),
            }
            queue_set(queue, registry, entity_bits, ANCHOR_VISIBILITY, &vis);
            return None;
        }
        other => {
            let idx = match other {
                AnchorFieldEdit::Pos(i, ..)
                | AnchorFieldEdit::Rot(i, _)
                | AnchorFieldEdit::BoundsOn(i, _)
                | AnchorFieldEdit::Bounds(i, ..)
                | AnchorFieldEdit::CenterOn(i, _)
                | AnchorFieldEdit::Center(i, ..) => usize::from(*i),
                // Tratados acima; o braço existe para o `match` não precisar de curinga, que
                // engoliria em silêncio a próxima variante.
                AnchorFieldEdit::Add
                | AnchorFieldEdit::Remove(_)
                | AnchorFieldEdit::Rename(..)
                | AnchorFieldEdit::Mount(_)
                | AnchorFieldEdit::SnapToAnchor
                | AnchorFieldEdit::VisibilityInEditor(_)
                | AnchorFieldEdit::VisibilityAtRuntime(_) => {
                    return None;
                }
            };
            apply_field(list.0.get_mut(idx)?, other, ppm);
        }
    }
    queue_set(queue, registry, entity_bits, NAMED_ANCHOR_LIST, &list);
    // Chegou aqui = a edição foi aceite. O `None` é «sem aviso», não «sem efeito».
    None
}

/// Escreve um campo de UMA âncora. Separado para manter o `match` de cima legível.
fn apply_field(a: &mut NamedAnchor, edit: &AnchorFieldEdit, ppm: f32) {
    match edit {
        AnchorFieldEdit::Pos(_, axis, v) => {
            // px -> m, na única porta que o faz.
            let m = v / ppm;
            if *axis == 0 {
                a.transform.translation.x = m;
            } else {
                a.transform.translation.y = m;
            }
        }
        AnchorFieldEdit::Rot(_, deg) => a.transform.rotation = deg.to_radians(),
        AnchorFieldEdit::BoundsOn(_, on) => {
            // Ligar semeia um retângulo VISÍVEL: um de área zero seria indistinguível de não
            // ter ligado nada.
            a.set_bounds(on.then_some(a.bounds.unwrap_or([0.0, 0.0, 16.0, 16.0])));
        }
        AnchorFieldEdit::Bounds(_, f, v) => {
            let mut b = a.bounds.unwrap_or([0.0; 4]);
            if let Some(slot) = b.get_mut(usize::from(*f)) {
                *slot = *v;
            }
            a.set_bounds(Some(b));
        }
        AnchorFieldEdit::CenterOn(_, on) => {
            a.set_center(on.then_some(a.center.unwrap_or([0.0, 0.0, 8.0, 8.0])));
        }
        AnchorFieldEdit::Center(_, f, v) => {
            let mut c = a.center.unwrap_or([0.0; 4]);
            if let Some(slot) = c.get_mut(usize::from(*f)) {
                *slot = *v;
            }
            a.set_center(Some(c));
        }
        // ⚠️ Nenhuma destas escreve um campo de UMA âncora, e por isso não têm o que fazer aqui.
        // O braço é explícito para o `match` não precisar de curinga — um curinga engoliria a
        // próxima variante **em silêncio**, que é como um controlo novo nasce mudo.
        AnchorFieldEdit::Add
        | AnchorFieldEdit::Remove(_)
        | AnchorFieldEdit::Rename(..)
        | AnchorFieldEdit::Mount(_)
        | AnchorFieldEdit::SnapToAnchor
        | AnchorFieldEdit::VisibilityInEditor(_)
        | AnchorFieldEdit::VisibilityAtRuntime(_) => {}
    }
}

/// A recusa, em palavras que o artista entende — nunca o nome da variante.
/// ⚠️ **Cada braço nomeia o SEU teto — a função não recebe nenhum.**
///
/// Ela recebia um `cap: usize`, e cada chamador passava o número que achava certo: o `Add` passava
/// `ANCHORS_MAX`, o `Rename` passava `ANCHOR_NAME_MAX_BYTES`. As duas mensagens liam igual
/// («over the limit of 64») e só estavam certas **por coincidência** de os dois tetos serem 64 —
/// mexer num deles tornava uma delas falsa, em silêncio (auditoria 2026-08-22). *Quem sabe de que
/// teto uma falha é, é a falha.*
fn describe(e: ph2d_ecs::AnchorNameError) -> String {
    match e {
        ph2d_ecs::AnchorNameError::Empty => "the name is empty".to_string(),
        ph2d_ecs::AnchorNameError::TooLong => {
            format!("the name is over {} bytes", ph2d_ecs::ANCHOR_NAME_MAX_BYTES)
        }
        ph2d_ecs::AnchorNameError::ControlChar => "the name has a control character".to_string(),
        ph2d_ecs::AnchorNameError::Duplicate => "that name is already used".to_string(),
        ph2d_ecs::AnchorNameError::ListFull => {
            format!("this sprite already has {} anchors", ph2d_ecs::ANCHORS_MAX)
        }
    }
}

// ⛔ Houve aqui um `detach_anchor_list`, atrás de um `#[allow(dead_code)]` e **sem chamador
// nenhum** — retirado na auditoria de 2026-08-22. Ele contradizia, a seis linhas de distância, o
// comentário do braço `Remove`: *«O "Remove" que retira o componente inteiro não existe de
// propósito: ele não tem gesto»*. Uma das duas coisas mentia, e o `allow` era o que a deixava
// mentir em silêncio. *Código morto e comentário velho mentem — e um `allow(dead_code)` é o
// carimbo que autoriza a mentira a ficar.*

#[cfg(test)]
#[path = "inspector_anchor_tests.rs"]
mod tests;
