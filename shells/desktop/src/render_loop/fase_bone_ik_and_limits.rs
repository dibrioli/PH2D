//! **Fase do quadro: O IK E OS LIMITES DO OSSO** — acrescentar, tirar e dobrar a âncora de IK, e acrescentar e tirar o limite de ângulo (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct BoneIkAndLimitsIntents {
    pub(super) pending_ik_add: bool,
    /// ⭐ *Look At* — a mesma âncora com a corrente em UM (ver `goal::add_look_at`).
    pub(super) pending_look_at: bool,
    /// ⭐ *Mirror Branch* — o lado oposto (ver `ph2d_skeleton_live::espelho`).
    pub(super) pending_bone_mirror: bool,
    pub(super) pending_ik_remove: bool,
    pub(super) pending_ik_bend: Option<ph2d_skeleton::BendSide>,
    pub(super) pending_bone_handles: Option<ph2d_skeleton::bend::Handles>,
    pub(super) pending_bone_tip: Option<usize>,
    pub(super) pending_limit_add: bool,
    pub(super) pending_limit_remove: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_bone_ik_and_limits(
        &mut self,
        intents: BoneIkAndLimitsIntents,
        osso: ph2d_ecs::Entity,
    ) -> Option<ph2d_ecs::Entity> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            component_registry,
            ..
        } = FrameGfx::of(gfx);
        let BoneIkAndLimitsIntents {
            pending_ik_add,
            pending_look_at,
            pending_bone_mirror,
            pending_ik_remove,
            pending_ik_bend,
            pending_bone_handles,
            pending_bone_tip,
            pending_limit_add,
            pending_limit_remove,
        } = intents;
        if pending_ik_add {
            match crate::skeleton_goal::add(sim, osso) {
                Some(_) => eprintln!(
                    "[ph2d-vec] osso: ancora de IK criada na ponta -- arraste o LOSANGO e a                              corrente segue-o, para sempre (a timeline anima-o como qualquer objecto)"
                ),
                None => eprintln!(
                    "[ph2d-vec] osso: este osso ja' tem ancora -- so' pode haver uma por corrente"
                ),
            }
        }
        // ⭐⭐⭐ **APONTAR** — a mesma âncora do irmão acima, com a corrente em UM.
        //
        // ⚠️ **Dois verbos e um motor, e isso está MEDIDO**
        // (`ph2d_app_skeleton::goal::sonda_do_apontar_tests`): a lei do alcance com a corrente
        // resolvida em UM já apontava com erro `0,000000°`. *O que faltava era o nome* — e a lente
        // do painel, que ali esconde os dois knobs que a medição diz serem inertes.
        if pending_look_at {
            match ph2d_skeleton_live::goal::add_look_at(sim, osso) {
                Some(_) => eprintln!(
                    "[ph2d-vec] osso: este osso passa a APONTAR para o losango -- arraste-o e o \
                     osso vira-se para ele; o campo `Aim Offset` roda o olhar em relacao ao eixo"
                ),
                None => eprintln!(
                    "[ph2d-vec] osso: este osso ja' tem ancora -- so' pode haver uma por corrente"
                ),
            }
        }
        // ⭐⭐⭐ **ESPELHAR o ramo** — o lado esquerdo construído a partir do direito.
        //
        // ⚠️ **O registo de componentes é o que faz a cópia carregar o que ESTA shell não conhece**
        // (o limite de ângulo, a curvatura, o repouso, e o que vier): a lei chama a cópia profunda,
        // que só sabe copiar o que o registo descreve. ⛔ Uma cópia campo a campo aqui esqueceria o
        // primeiro componente novo, em silêncio.
        if pending_bone_mirror {
            match ph2d_skeleton_live::espelho::espelha(sim, component_registry, osso) {
                Some(_) => eprintln!(
                    "[ph2d-vec] osso: ramo espelhado -- a copia e' irma do original e os nomes \
                     trocaram de lado; a ancora de IK e o osso inteligente NAO viajam (eles nomeiam \
                     outros objectos da cena)"
                ),
                None => {
                    eprintln!("[ph2d-vec] osso: nao deu para espelhar -- a pose do pai e' singular")
                }
            }
        }
        if pending_ik_remove {
            crate::skeleton_goal::remove(sim, osso, &mut self.preview_drive);
        }
        if let Some(lado) = pending_ik_bend
            && let Some(mut g) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::IkGoal>(osso)
        {
            g.bend = lado;
        }
        // ⭐⭐⭐⭐ **DE ONDE VÊM AS DUAS ALÇAS DE CURVATURA** (F8, 2026-09-16).
        //
        // ⚠️ **Só o MODO muda; o `curve` fica INTOCADO** — voltar a `Manual` devolve exactamente o
        // que o artista tinha escrito. *Um modo que sobrescreve o valor autorado é um modo que não
        // se desliga.*
        if let Some(modo) = pending_bone_handles
            && let Some(mut b) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(osso)
        {
            b.handles = modo;
        }
        // ⭐⭐⭐⭐ **QUEM MANDA NA PONTA DA CURVA** (o *custom handle*, ordem do dono de 2026-09-16).
        //
        // ⚠️ **A escolha resolve-se contra a lista de AGORA** (a porta da família, a mesma que o
        // painel pintou): um índice que já não existe **não escreve nada**, porque uma escolha
        // inventada mudaria a curva por um clique que o artista não deu.
        if let Some(i) = pending_bone_tip
            && let Some(tip) = ph2d_app_skeleton::curve_tip::choice_at(sim, osso, i)
            && let Some(mut b) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(osso)
        {
            b.curve_tip = tip;
        }
        // ⭐⭐⭐ **O LIMITE DE ÂNGULO** — os dois verbos e os dois extremos.
        if pending_limit_add && !crate::bone_limit::add_limit(sim, osso) {
            eprintln!("[ph2d-vec] osso: esta junta ja' tem limite -- so' pode haver um por osso");
        }
        if pending_limit_remove {
            crate::bone_limit::remove_limit(sim, osso);
        }
        Some(osso)
    }
}
