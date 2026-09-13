//! **Fase do quadro: OS SMART BONES E OS NÚMEROS DO OSSO** — acrescentar, escolher, recortar e tirar smart bones, os números do smart, do limite e do
//! IK, e os controlos que nasceram mudos (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct BoneSmartAndKnobsIntents {
    pub(super) pending_ik_knob: Option<(IkKnob, f64)>,
    pub(super) pending_limit_knob: Option<(bool, f64)>,
    pub(super) pending_smart_add: bool,
    pub(super) pending_smart_remove: bool,
    pub(super) pending_smart_knob: Option<(bool, f64)>,
    pub(super) pending_smart_clip: Option<usize>,
    pub(super) pending_smart_pick: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_bone_smart_and_knobs(
        &mut self,
        intents: BoneSmartAndKnobsIntents,
        mudos_antes: usize,
        osso: ph2d_ecs::Entity,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, toasts, .. } = FrameGfx::of(gfx);
        let BoneSmartAndKnobsIntents {
            pending_ik_knob,
            pending_limit_knob,
            pending_smart_add,
            pending_smart_remove,
            pending_smart_knob,
            pending_smart_clip,
            pending_smart_pick,
        } = intents;
        // ⭐⭐⭐ **O OSSO INTELIGENTE** — o componente entra VAZIO, e o artista escolhe.
        //
        // ⚠️⚠️ **ELE NÃO CRIA NADA** (ordem do dono, 2026-09-08: *«porque criar Bone Action
        // no inspector e na timeline? Melhor não criar nada»*). O desenho anterior fabricava
        // um clip com o nome do osso e abria a timeline nele — duas coisas por um clique,
        // nenhuma pedida. ⛔ E adoptar o clip ABERTO, que foi o desenho antes desse, era
        // pior ainda: um documento novo tem **uma** acção chamada `"Main"`, logo todo
        // controlo casava com a animação principal da cena, calado.
        //
        // ⇒ o gesto **anexa** o controlo e mais nada; quem lhe dá sujeito são as duas
        // linhas do painel — o *Pick Object* e o selector *Action*.
        if pending_smart_add {
            sim.world_mut()
                .entity_mut(osso)
                .insert(ph2d_skeleton_ecs::SmartBone::default());
        }
        // ⭐⭐⭐ **ARMAR O PICK DO ALVO** — o OSSO é capturado aqui, e não lido no clique
        // seguinte: aquele clique MUDA a selecção, então lê-lo então leria o alvo no lugar
        // do sujeito. É a lei do `PathPick`, escrita no doc dele.
        if pending_smart_pick {
            self.skeleton.smart_pick = Some(osso.to_bits());
        }
        // ⭐⭐⭐ **TROCAR A ACÇÃO** pelo selector — tudo por UMA porta
        // ([`crate::skeleton_smart::choose_action`]), que é onde a lei vive e onde ela é
        // gateada: a POSIÇÃO resolve-se contra a lista que o PAINEL PINTOU (filtrada pelo
        // alvo), nunca contra `doc.clips()`, e o que se guarda é o NOME.
        //
        // ⛔⛔ Este bloco tinha a lei escrita **aqui** e o gate do outro lado da porta: a
        // mutação que repunha `doc.clips().get(i)` deixava a suíte verde e trazia de volta o
        // report do dono (*«não consegue selecionar o clip desejado»*).
        if let Some(i) = pending_smart_clip
            && let Some((nome, aberta)) =
                crate::skeleton_smart::choose_action(sim, &self.timeline.doc, osso, i)
            && aberta
        {
            // ⛔⛔ **A acção ABERTA é oferecida e o motor recusa-a** — um documento novo tem
            // **uma** acção (`"Main"`) e ela **está aberta**, logo a única opção da lista era
            // a única que não corre, e nada na tela o dizia. ⚠️ A lei fica (um controlo não
            // percorre o que o artista está a gravar — os dois escreveriam o mesmo objecto
            // no mesmo quadro); o que não pode é ser **calada**.
            toasts.push(ph2d_editor_core::Toast::warning(format!(
                "\"{nome}\" is open in the timeline, so you are EDITING it - the bone will \
                         not run it. Switch the timeline to another animation to see it play."
            )));
        }
        if pending_smart_remove {
            // ⭐⭐⭐ **E a POSE VOLTA** — a porta faz as duas metades, como a do *Remove IK*.
            crate::skeleton_smart::remove(sim, &self.timeline.doc, osso, &mut self.preview_drive);
        }
        if let Some((e_to, graus)) = pending_smart_knob
            && let Some(mut sb) = sim
                .world_mut()
                .get_mut::<ph2d_skeleton_ecs::SmartBone>(osso)
        {
            // ⚠️ A MESMA conversão graus→radianos do limite, e pela mesma razão.
            let rad = graus.to_radians();
            if e_to {
                sb.to = rad;
            } else {
                sb.from = rad;
            }
        }
        if let Some((e_max, graus)) = pending_limit_knob
            && let Some(mut l) = sim
                .world_mut()
                .get_mut::<ph2d_skeleton_ecs::BoneLimit>(osso)
        {
            // ⚠️ **A conversão GRAUS→RADIANOS vive aqui**, na porta entre o campo (que fala
            // a unidade do artista) e o componente (que fala a do `Transform::rotation`).
            // ⛔ Sem ela um `90` digitado seria noventa RADIANOS — catorze voltas.
            //
            // ⛔⛔ **E pela MESMA porta do arrasto** (`set_edge`, auditoria de 2026-09-08):
            // escrever cru deixava o campo produzir `min > max`, que a lei lê como faixa de
            // meia-largura **zero** — a junta congela no ponto médio, e o desenho normaliza
            // os dois extremos, logo o canvas continua a pintar um sector normal enquanto o
            // osso não roda um grau.
            crate::bone_limit::set_edge(&mut l, e_max, graus.to_radians());
        }
        if let Some((qual, v)) = pending_ik_knob
            && let Some(mut g) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::IkGoal>(osso)
        {
            match qual {
                // ⛔ `0..1` é a faixa da LEI, não uma escolha: fora dela o `blend_angle`
                // satura nos extremos, e um campo que aceita `7` mentiria sobre o efeito.
                IkKnob::Mix => g.mix = v.clamp(0.0, 1.0),
                // ⛔ Piso em zero e SEM tecto: a suavidade é uma fracção do alcance, e o
                // `softened_distance` já a apara pelo próprio alcance — §0.0, o limite é do
                // recurso e não um palpite.
                IkKnob::Softness => g.softness = v.max(0.0),
                // ⛔ Idem: quem apara a corrente é a ARVORE, no passe.
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "o campo é f64 e a contagem de ossos é u32; o piso em 0 já corre acima"
                )]
                IkKnob::Chain => g.chain = v.max(0.0) as u32,
            }
        }
        // ⭐⭐⭐ **E O APP DIZ, seja qual for a ordem em que o artista chegou aqui.**
        //
        // ⚠️ **Avisa e FAZ na mesma**, ⛔ não recusa: tirar a âncora depois é um gesto que
        // existe (*Remove IK*), e um verbo que recusa deixaria o artista sem caminho. O que
        // não pode é o app ficar **calado** sobre um controlo que ele sabe que vai nascer
        // mudo.
        if crate::skeleton_smart::governed_controls(sim).len() > mudos_antes {
            toasts.push(ph2d_editor_core::Toast::warning(
                "This bone is driven by an IK anchor, so its angle is derived - turning it \
                         will not run the action. Use a free bone, or Remove IK.",
            ));
        }
    }
}
