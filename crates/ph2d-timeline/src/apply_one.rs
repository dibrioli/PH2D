//! ⭐⭐⭐ **APLICAR UM CLIP FORA DO TRANSPORTE** — irmão do [`super::apply`] pelo teto de 700 LOC, e o
//! corte é por RESPONSABILIDADE: ali responde-se *«em que instante o transporte está?»* (e a
//! resposta é o documento inteiro), aqui *«toca ESTE clip nesta posição»*, porque a posição é
//! derivada de outra coisa.
//!
//! O 1.º cliente é o **osso inteligente** do esqueleto (o *Smart Bone* do Moho): o ângulo de um osso
//! de controlo é o tempo de uma acção. ⚠️ Outros virão com a mesma forma — a velocidade de um corpo,
//! um sinal — e é por isso que esta porta não sabe nada sobre ossos.

use bevy_ecs::world::World;

use crate::TimelineDoc;

/// ⭐⭐⭐ **APLICA **UM** CLIP NUM TEMPO** — sem tocar no transporte, no clip activo nem na pilha do
/// Arrange.
///
/// É a porta que um **motor de fora da timeline** precisa: alguém que diga *«toca ESTE clip nesta
/// posição»* porque a posição é derivada de outra coisa (o ângulo de um osso, a velocidade de um
/// corpo, um sinal). O caminho normal ([`apply_from_doc`]) responde a outra pergunta — *«em que
/// instante o transporte está?»* — e a resposta dele é o documento inteiro.
///
/// ⚠️ **`doc` entra por referência PARTILHADA, e isso é a garantia**: esta porta não pode mexer no
/// `active_clip` nem no scratch. ⛔ Trocar o clip activo, aplicar e repor seria o mesmo efeito com
/// uma janela de um quadro em que o documento mente sobre si próprio — e o painel da timeline lê-o
/// no mesmo quadro.
///
/// ⚠️ **Ela escreve só o que o clip ANIMA**, e nada mais: um clip com uma track é uma escrita. Quem
/// chama compõe o resto (a pose de base vem de quem a escreveu antes).
///
/// Devolve quantas propriedades foram escritas — `0` quando o clip está vazio, quando nenhuma
/// binding dele resolve nesta sessão, ou quando o índice não existe.
pub fn apply_one_clip(world: &mut World, doc: &TimelineDoc, clip: usize, t: f64) -> usize {
    apply_one_clip_except(world, doc, clip, t, |_| false)
}

/// [`apply_one_clip`] que salta as entidades de `skip` (a arrastada pelo gizmo, a pose deslocada) —
/// a mesma mão que o apply da cena respeita.
pub fn apply_one_clip_except(
    world: &mut World,
    doc: &TimelineDoc,
    clip: usize,
    t: f64,
    skip: impl Fn(u64) -> bool,
) -> usize {
    let Some(named) = doc.clips().get(clip) else {
        return 0;
    };
    let mut feitas = 0;
    for (target, _) in named.clip.tracks() {
        // ⚠️ A binding é a ponte `track → (entidade, propriedade)`, e ela pode faltar: um objecto
        // apagado deixa a track no clip e a binding marcada `missing`. Saltar em silêncio seria o
        // no-op que o P6 proíbe — quem mostra o crachá é o painel, e aqui basta não escrever.
        let Some(b) = doc.bindings().iter().find(|b| b.target == *target) else {
            continue;
        };
        if b.missing || b.entity == 0 || skip(b.entity) {
            continue;
        }
        let Some(v) = named.clip.sample(*target, t) else {
            continue;
        };
        // ⛔⛔ **`try_from_bits`, NUNCA `from_bits`** — a lei desta crate, escrita em prosa no
        // `apply.rs` e no `persist.rs`, e este ficheiro era o único dos 15 sítios a violá-la
        // (auditoria de 2026-09-08). O guarda de cima cobre o **sentinela documentado** (`0`, a
        // binding destacada) e mais nada: qualquer outro padrão de bits inválido — os de uma sessão
        // anterior, que é precisamente o que um `.ph2dproj` carregado traz — faz o `from_bits`
        // **abortar o processo**, enquanto o `try_` devolve `None`.
        let Some(e) = ph2d_ecs::Entity::try_from_bits(b.entity) else {
            continue;
        };
        if world.get_entity(e).is_err() {
            continue;
        }
        // ⚠️ **Sem caminho de movimento e sem orientação**: os dois são propriedades da FAIXA no
        // transporte (o `MotionPath` de uma strip, o *orient to path*), e este clip não está numa
        // strip. Passá-los seria inventar um contexto que o chamador não tem.
        crate::apply_prop::write_prop(world, e, b, None, v, false);
        feitas += 1;
    }
    feitas
}

/// ⭐⭐ **A ANIMAÇÃO de cada objecto, no relógio dele** (doc ObjectsAndComponents 01 §14, item 1):
/// o clipe que ele pediu ([`ph2d_ecs::ClipPlayback`]) desde o pedido, ou o de ENTRADA desde o
/// início — dobrado pelo loop do clipe ([`TimelineDoc::clip_local_time`]). Os objectos que a Cena
/// arranjou ficam com o que a Cena compôs ([`TimelineDoc::autoplay_entries`]). Devolve quantos
/// objectos tocaram.
pub fn apply_autoplay(
    world: &mut World,
    doc: &TimelineDoc,
    t: f64,
    skip: impl Fn(u64) -> bool + Copy,
    refusals: &mut Vec<crate::ClipRefusal>,
) -> usize {
    let mut tocaram = 0;
    for (dono, entrada) in doc.autoplay_entries() {
        let e = ph2d_ecs::Entity::try_from_bits(dono).filter(|e| world.get_entity(*e).is_ok());
        // ⚠️ Um pedido do FUTURO — o relógio voltou para antes dele (rewind, scrub, recomeçar) — é
        // largado: senão revivia ao passar outra vez pelo instante, sem o sinal ter disparado. E um
        // pedido de um clipe que a animação não tem é largado E DITO, uma vez.
        if let Some(e) = e
            && let Some(p) = world.get::<ph2d_ecs::ClipPlayback>(e)
        {
            let futuro = p.desde > t;
            let desconhecido = doc.clip_named(dono, &p.clip).is_none();
            if futuro || desconhecido {
                world.entity_mut(e).remove::<ph2d_ecs::ClipPlayback>();
            }
            if desconhecido && !futuro {
                refusals.push(crate::ClipRefusal::UnknownClip);
            }
        }
        let pedido = e
            .and_then(|e| world.get::<ph2d_ecs::ClipPlayback>(e))
            .and_then(|p| p.em(t))
            .and_then(|(nome, desde)| Some((doc.clip_named(dono, nome)?, desde)));
        let (clip, local) = pedido.unwrap_or((entrada, t));
        let escritas =
            apply_one_clip_except(world, doc, clip, doc.clip_local_time(clip, local), skip);
        tocaram += usize::from(escritas > 0);
    }
    tocaram
}
