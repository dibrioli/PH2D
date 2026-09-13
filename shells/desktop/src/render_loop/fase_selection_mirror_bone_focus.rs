//! **Fase do quadro: O OSSO EM FOCO NO PAINEL DO ESQUELETO** — o pick dos smart bones, o osso em foco e a sua revelação, e o limite, o smart, as acções e
//! o IK do osso publicados no painel do esqueleto (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_selection_mirror_bone_focus(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim, hero_screen, ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // ⭐⭐⭐ **O PICK DO ALVO RESOLVE-SE AQUI**, antes de se perguntar qual osso está em
        // foco — e a ordem é o desenho: quem resolve é *«a selecção passou a ser outra
        // coisa»*, e o clique que a mudou pode ter vindo do CANVAS **ou** da HIERARQUIA. As
        // duas escrevem a mesma selecção, então as duas superfícies saem de graça; um
        // segundo caminho de acerto seria a segunda resposta à mesma pergunta.
        //
        // ⚠️ **E a selecção VOLTA ao osso**, ao contrário dos irmãos `PathPick`: aqui o
        // artista escolheu um objecto *para o osso*, e o contexto dele é o painel do osso.
        // Sem isto a secção Skeleton desaparecia debaixo dele no instante do acerto.
        //
        // ⛔ Clique no vazio **não** desarma (a lista de objectos anima-se por engano com
        // facilidade); quem desiste é o `Escape`.
        // ⛔⛔ **UM PICK NÃO SOBREVIVE AO SUJEITO DELE** (auditoria de 2026-09-08). Ele
        // consome o `Down` primário em **toda** ferramenta, então um pick esquecido é o
        // canvas morto ao botão esquerdo, **sem nada na tela que o diga** — o botão
        // *Picking…* deixa de ser pintado no instante em que o osso deixa de estar em foco.
        //
        // ⚠️ **Pergunta-se o FACTO, não os eventos:** abrir outro projecto, o `Ctrl+Z` e
        // apagar o osso deixam todos os mesmos bits mortos, e uma lista de sítios a limpar
        // esqueceria o quarto. O irmão `vec_path_pick` tem cinco limpezas escritas à mão, e
        // o comentário de uma delas já escrevia a lei: *«não faz sentido: limpa, para não
        // ficar armado e invisível»*.
        if let Some(bits) = self.skeleton.smart_pick
            && ph2d_ecs::Entity::try_from_bits(bits)
                .is_none_or(|e| sim.world().get::<ph2d_skeleton_ecs::SmartBone>(e).is_none())
        {
            self.skeleton.smart_pick = None;
        }
        // ⚠️ **EXACTAMENTE UM seleccionado**, e não *«o primeiro que não é o osso»*: o
        // estado normal do *Bind* é **forma + osso** escolhidos (é a razão de existir do
        // `bone_gesture::selected_bone`), e ali a leitura antiga resolvia o pick **no mesmo
        // quadro em que ele era armado**, sem o artista clicar em nada.
        let alvo_do_pick = self.skeleton.smart_pick.and_then(|bits_osso| {
            let sel: Vec<u64> = hero.gizmo.iter_selected().collect();
            (sel.len() == 1 && sel[0] != bits_osso).then(|| sel[0])
        });
        if let Some(bits_osso) = self.skeleton.smart_pick
            && let Some(alvo) = alvo_do_pick
            && crate::skeleton_smart::set_target(
                sim,
                ph2d_ecs::Entity::from_bits(bits_osso),
                ph2d_ecs::Entity::from_bits(alvo),
            )
        {
            self.skeleton.smart_pick = None;
            hero.gizmo.replace_selection(Some(bits_osso));
        }
        let osso_em_foco = crate::bone_gesture::selected_bone(sim, hero.gizmo.iter_selected());
        // ⭐⭐⭐ **UM OSSO NOVO EM FOCO REVELA A SECÇÃO** (report do dono, 2026-09-08:
        // *«selecionar o bone nem sempre abre a secção de skeleton no painel»*).
        //
        // ⚠️ **A ARESTA é o que se publica, nunca o estado:** com um osso escolhido o
        // painel rolaria a cada quadro e o artista não conseguiria ler mais nada. É a mesma
        // lei que a timeline já segue — *«seleccionar um objecto NOVO leva a timeline à aba
        // Keys»* (Enio, 2026-07-22).
        //
        // ⚠️ **Quem decide se ROLA é o painel**, que é o único sítio onde a faixa visível e
        // o `y` do cabeçalho existem: daqui sai o *pedido*, e um cabeçalho já à vista fica
        // onde está.
        // ⭐⭐⭐ **UM OSSO NOVO TRAZ A ABA DO PAINEL PARA A FRENTE.**
        //
        // ⛔⛔ **É o sucessor do «revelar-ao-focar»** (report do dono, 2026-09-08), e o
        // painel próprio mudou-lhe o EFEITO sem mudar a lei: a rolagem existia porque a
        // secção caía `1394 px` abaixo de 785 px de outro assunto; aqui o cabeçalho é a
        // primeira linha e não há dobra onde se esconder. O que sobra é o encaixe
        // partilhado — se o Inspector estiver por cima, revelar é **trazer a aba**.
        //
        // ⚠️ A ARESTA continua a ser a lei (`skeleton_reveal::on_focus`): pedi-lo em todo
        // quadro prenderia a aba e o artista não conseguiria olhar para outra.
        if crate::skeleton_reveal::on_focus(&mut self.skeleton.osso_revelado, osso_em_foco) {
            // ⭐⭐⭐ **ORDEM DO DONO (2026-09-09):** *«se já existe um osso no mundo, ao
            // seleccionar o osso o painel de Bones é aberto e o botão Transform é
            // seleccionado»*. As três metades saem da MESMA aresta, e é isso que as mantém
            // de acordo: abrir sem armar deixaria a fileira apagada sobre um osso escolhido.
            <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                hero,
                <ph2d_panel_skeleton::SkeletonPanel as ph2d_editor_core::panel::Panel>::ID,
                true,
            );
            hero.store
                .bump_panel_z(ph2d_editor_core::ids::SKELETON_PANEL);
            // ⚠️ **A ferramenta arma-se no QUADRO SEGUINTE** (`bone_arm_pending`): aqui o
            // `gfx` já está emprestado a `sim`/`hero`, e um segundo empréstimo dele não
            // compila. O espelho da shell escreve-se **já**, para este quadro rotear certo
            // e a fileira acender no mesmo instante em que o osso é escolhido.
            self.skeleton.bone_arm_pending = Some(ph2d_tool_vector::BoneAction::Transform);
            self.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Bone;
            self.vec.draw_config.bone_action = ph2d_tool_vector::BoneAction::Transform;
        }
        // ⭐ **PORQUE a secção não tem sujeito** (report do dono, 2026-09-08: *«seleccionar o
        // bone nem sempre abre a secção de skeleton»*). ⚠️ A pergunta tem três respostas que
        // se leem iguais na tela — *nada seleccionado* · *seleccionado e não é osso* · *é
        // osso e a secção está fechada/fora da dobra* —, e esta linha separa-as: ela diz o
        // que ESTÁ seleccionado e o que cada um É.
        if std::env::var_os("PH2D_BONE_LOG").is_some() && osso_em_foco.is_none() {
            let quem: Vec<String> = hero
                .gizmo
                .iter_selected()
                .map(|b| {
                    let e = ph2d_ecs::Entity::from_bits(b);
                    let nome = sim
                        .world()
                        .get::<ph2d_ecs::Name>(e)
                        .map_or_else(|| "<sem nome>".to_string(), |n| n.as_str().to_string());
                    let osso = sim.world().get::<ph2d_skeleton_ecs::Bone>(e).is_some();
                    format!("{nome}(osso={osso})")
                })
                .collect();
            if !quem.is_empty() {
                eprintln!(
                    "[bone] a seccao SKELETON esta' sem sujeito, e ha' {} seleccionado(s): \
                             {quem:?} -- nenhum deles e' um osso",
                    quem.len()
                );
            }
        }
        ph2d_panel_skeleton::set_current_bone(osso_em_foco.and_then(|b| {
            sim.world()
                .get::<ph2d_skeleton_ecs::Bone>(ph2d_ecs::Entity::from_bits(b))
                .map(|v| (v.length, v.strength))
        }));
        // ⭐⭐⭐ **A ÂNCORA do osso em foco** — é isto que decide entre *Add IK* e *Remove IK*
        // no painel, e se os três números dela têm sujeito. ⚠️ Pela MESMA porta que publica
        // os números do osso (`selected_bone`): duas perguntas *"qual osso está aceso?"*
        // divergiriam no primeiro clique.
        // ⭐ O limite da junta em foco, em GRAUS — a mesma porta e o mesmo guarda de foco.
        ph2d_panel_skeleton::set_current_bone_limit(osso_em_foco.and_then(|b| {
            sim.world()
                .get::<ph2d_skeleton_ecs::BoneLimit>(ph2d_ecs::Entity::from_bits(b))
                .map(|l| (l.min.to_degrees(), l.max.to_degrees()))
        }));
        // ⭐ O osso inteligente em foco — a faixa (em GRAUS), a acção e o alvo, pela MESMA
        // porta: publicá-los por portas separadas deixaria um quadro em que a faixa é de um
        // osso e o nome é do anterior.
        let smart = osso_em_foco.and_then(|b| {
            sim.world()
                .get::<ph2d_skeleton_ecs::SmartBone>(ph2d_ecs::Entity::from_bits(b))
                .cloned()
        });
        ph2d_panel_skeleton::set_current_bone_smart(smart.as_ref().map(|s| {
            ph2d_panel_skeleton::SmartBoneView {
                from: s.from.to_degrees(),
                to: s.to.to_degrees(),
                clip: s.clip.clone(),
                target: s.target.clone(),
                picking: self.skeleton.smart_pick == osso_em_foco,
            }
        }));
        // ⭐⭐⭐ **A lista de ACÇÕES, filtrada pelo ALVO** — é ela que responde *«qual
        // animação?»* na tela. ⚠️ Publicada **só** quando há um osso inteligente em foco:
        // sem sujeito ela seria um selector sem nada para escolher.
        ph2d_panel_skeleton::set_current_bone_actions(smart.as_ref().map_or_else(Vec::new, |s| {
            crate::skeleton_smart::actions_for(sim.world(), &self.timeline.doc, s)
        }));
        ph2d_panel_skeleton::set_current_bone_ik(osso_em_foco.and_then(|b| {
            sim.world()
                .get::<ph2d_skeleton_ecs::IkGoal>(ph2d_ecs::Entity::from_bits(b))
                .map(|g| (g.mix, g.softness, f64::from(g.chain), g.bend))
        }));
    }
}
