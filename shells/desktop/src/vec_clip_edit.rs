//! **O RECORTE da seleção** — a projeção que o painel lê, e o chip que a edita.
//!
//! Módulo irmão do [`crate::vec_frame_edit`], e o corte é por PERGUNTA, não por tamanho: aquele
//! responde *"qual é a moldura desta seleção?"* (a seção Frame, o auto layout, as âncoras), este
//! responde *"quem recorta nesta seleção?"*. Foram a mesma pergunta enquanto só uma moldura podia
//! recortar; deixaram de ser em 2026-08-21, quando o Enio pediu *"a feature Clip Content para
//! qualquer forma vetorial fechada"*.
//!
//! # A regra de elegibilidade é a MESMA, com um candidato a mais
//!
//! *Qual forma selecionada contém tudo o que está selecionado?* — o inverso da expansão de
//! seleção, herdado do `vec_frame_edit` inteiro (inclusive o `is_within`, que é dele). O que muda
//! é só o filtro do candidato: era *"tem `VecFrame`"*, é *"o caminho é FECHADO"*.
//!
//! O que ela recusa segue valendo, e pelas mesmas razões: **um filho sozinho** não oferece o chip
//! (o artista selecionou a forma, não o contêiner); **dois irmãos** não têm resposta única;
//! **contêiner + forma solta** editaria o que o artista não está a olhar.
//!
//! # Por que FECHADO
//!
//! ⚠️ Um caminho aberto não tem interior. O Vello recortaria à mesma — ele fecha a região
//! implicitamente para preencher — mas por uma silhueta que o artista **não desenhou**, e o
//! resultado seria arte a sumir atrás de uma fronteira invisível. É o mesmo predicado
//! (`VecPath::closed`) com que a booleana viva e o Blend recusam caminhos abertos, e por isso o
//! filtro vive AQUI, na fronteira de autoria, e não no componente: o `VecClipContent` descreve
//! uma intenção, e quem decide onde ela pode ser expressa é quem oferece o controlo.

use ph2d_ecs::{Entity, SimWorld, VecClipContent};
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;
use crate::vec_frame_edit::{entity_of, is_within};

/// O caminho `id` é FECHADO? Um id sem caminho na cena descreve um mundo que já não existe, e ali
/// a resposta honesta é "não" — nunca um recorte oferecido sobre geometria que ninguém tem.
fn is_closed(scene: &VecScene, id: VecPathId) -> bool {
    scene.paths().iter().any(|p| p.id == id && p.closed)
}

/// **A forma que recorta nesta seleção** — a fechada que CONTÉM tudo o que está selecionado.
/// `None` = a seleção não oferece o controlo.
#[must_use]
pub(crate) fn clip_subject_of_selection(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<Entity> {
    if selected.is_empty() {
        return None;
    }
    let members: Vec<Entity> = selected
        .iter()
        .filter_map(|id| entity_of(sim, map, *id))
        .collect();
    // ⚠️ A herança literal do `frame_of_selection`: um caminho selecionado sem entidade viva faria
    // *"contém tudo"* deixar de significar o que diz — a forma pareceria conter uma seleção cuja
    // metade ausente ninguém conferiu.
    if members.len() != selected.len() {
        return None;
    }
    selected.iter().zip(members.iter()).find_map(|(&id, &e)| {
        (is_closed(scene, id) && members.iter().all(|&m| is_within(sim, e, m))).then_some(e)
    })
}

/// O recorte da seleção deste frame: `None` = a seleção não oferece o controlo; `Some(false)` =
/// oferece e está desligado.
#[must_use]
pub(crate) fn selected_clip(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<bool> {
    let e = clip_subject_of_selection(sim, scene, map, selected)?;
    Some(sim.world().get::<VecClipContent>(e).is_some())
}

/// Liga/desliga o recorte na forma da seleção. Devolve `true` se algo mudou — o
/// `post_frame_undo` captura por diff, então um no-op não custa passo de undo.
pub(crate) fn set_selected_clip(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
    on: bool,
) -> bool {
    let Some(e) = clip_subject_of_selection(sim, scene, map, selected) else {
        return false;
    };
    if sim.world().get::<VecClipContent>(e).is_some() == on {
        return false;
    }
    let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
        return false;
    };
    // ⚠️ **Desligar REMOVE o componente**, não o põe em falso: a ausência é o estado "não
    // recorta" (é o que faz um documento sem recortes ser byte-idêntico ao de antes da feature),
    // e um componente presente-e-inerte daria ao undo um passo que a tela não distingue.
    if on {
        em.insert(VecClipContent);
    } else {
        em.remove::<VecClipContent>();
    }
    true
}

#[cfg(test)]
#[path = "vec_clip_edit_tests.rs"]
mod tests;
