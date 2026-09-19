//! **Fase do quadro: OS SMART BONES E OS NÚMEROS DO OSSO** — acrescentar, escolher, recortar e tirar smart bones, os números do smart, do limite e do
//! IK, e os controlos que nasceram mudos (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;
use ph2d_i18n::{tr, tr_with};

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
        //
        // ⛔ **A lei saiu daqui para a família** (censo dos verbos, 2026-09-19): das catorze rotas
        // desta secção esta era a única cujo efeito estava escrito **dentro da fase do quadro**, e
        // por isso a única que o censo não conseguia correr sem re-escrever a lei.
        if pending_smart_add {
            let _ = ph2d_app_skeleton::smart::add(sim, osso);
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
            toasts.push(ph2d_editor_core::Toast::warning(tr_with(
                "shell.fase_bone_smart_and_knobs.is_open_in_the",
                &[("nome", &nome)],
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
        // ⭐⭐⭐ **MUDAR O `Chain` RE-CAPTURA O LADO DA DOBRA** — ordem do dono (2026-09-14: *«o lado
        // da dobra é capturado no momento em que carrega Add IK e sempre que IK Chain for
        // mudado»*). A razão é geométrica: o lado descreve **uma corrente**, e subir o número troca
        // a corrente por outra — o bit guardado passaria a falar de uma geometria que já não é a
        // que está debaixo do artista.
        //
        // ⚠️ **Lido ANTES de escrever**, e tem de ser: o `captured_side` precisa de `&sim` e o
        // `g` é um empréstimo mutável do mesmo mundo. ⛔ E é lido com a corrente NOVA, que é a
        // pergunta certa — com a velha ele devolveria o lado que já lá está.
        let lado_novo = match pending_ik_knob {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "o campo é f64 e a contagem de ossos é u32; o piso em 0 já corre abaixo"
            )]
            // ⛔⛔ **O MISTO NÃO É RE-CAPTURADO, e a excepção é a lei dele.** A captura existe
            // porque o lado descreve UMA corrente e subir o número troca a corrente por outra — mas
            // o que ela devolve é um lado FORÇADO (`Ccw`/`Cw`), e escrevê-lo por cima de um `Mixed`
            // apagaria a escolha do artista **em silêncio**, no gesto mais provável de todos (pôr o
            // `IK Chain` no tamanho certo depois de escolher o modo). ⭐ E o misto não precisa da
            // re-captura: ele lê o lado de cada junta da pose autorada, a cada resolução — uma
            // corrente maior traz juntas novas, e cada uma chega com o seu.
            Some((IkKnob::Chain, v))
                if sim
                    .world()
                    .get::<ph2d_skeleton_ecs::IkGoal>(osso)
                    .is_none_or(|g| g.bend != ph2d_skeleton::BendSide::Mixed) =>
            {
                Some(ph2d_skeleton_live::goal::side_for_chain(
                    sim,
                    osso,
                    v.max(0.0) as u32,
                ))
            }
            _ => None,
        };
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
                // ⭐⭐ **GRAUS na tela, RADIANOS no documento** — a mesma lei do limite da junta, e
                // a conversão vive aqui, num sítio só. ⛔ **Sem piso nem tecto:** um desvio é um
                // arco e dar a volta inteira é legítimo (apontar «para trás» é `180`), e a lei do
                // `wrap_pi` a jusante já o normaliza — §0.0, um limite tem de nomear um recurso.
                IkKnob::Offset => g.offset = v.to_radians(),
            }
            if let Some(lado) = lado_novo {
                g.bend = lado;
            }
        }
        // ⭐⭐⭐ **E O APP DIZ, seja qual for a ordem em que o artista chegou aqui.**
        //
        // ⚠️ **Avisa e FAZ na mesma**, ⛔ não recusa: tirar a âncora depois é um gesto que
        // existe (*Remove IK*), e um verbo que recusa deixaria o artista sem caminho. O que
        // não pode é o app ficar **calado** sobre um controlo que ele sabe que vai nascer
        // mudo.
        if crate::skeleton_smart::governed_controls(sim).len() > mudos_antes {
            toasts.push(ph2d_editor_core::Toast::warning(tr(
                "shell.fase_bone_smart_and_knobs.this_bone_is_driven_by",
            )));
        }
    }
}
