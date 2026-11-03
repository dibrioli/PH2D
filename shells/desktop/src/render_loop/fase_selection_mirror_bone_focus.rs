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
        let (win, cam) = (gfx.scene_window(), gfx.camera);
        use ph2d_editor_core::ids::SKELETON_PANEL as BONES;
        let FrameGfx {
            sim,
            hero_screen,
            tools,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // ⭐⭐⭐ **O PICK DO ALVO RESOLVE-SE AQUI**, antes de se perguntar qual osso está em foco:
        // canvas e hierarquia escrevem a mesma selecção, e as duas superfícies saem de graça.
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
        // ⭐⭐⭐ **UM OSSO NOVO TRAZ A ABA DO PAINEL PARA A FRENTE** (report do dono, 2026-09-08).
        // ⚠️ A ARESTA continua a ser a lei (`skeleton_reveal::on_focus`): pedi-lo em todo
        // quadro prenderia a aba e o artista não conseguiria olhar para outra.
        if ph2d_app_skeleton::bone_bridge::on_focus(tools, &mut self.skeleton, osso_em_foco) {
            // ⭐⭐⭐ **Escolher um osso ABRE o painel de Bones e trá-lo à frente** (ordem do dono,
            // 09/09) e, com a ferramenta na mão, o verbo passa a *Transform* (dono 01/11; fora da
            // mão nada se arma, A14) — `bone_bridge::on_focus`.
            <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                hero,
                <ph2d_panel_skeleton::SkeletonPanel as ph2d_editor_core::panel::Panel>::ID,
                true,
            );
            hero.store
                .bump_panel_z(ph2d_editor_core::ids::SKELETON_PANEL);
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
        let doc = ph2d_editor_core::DocPx::of(&hero.project);
        let (pick, linha) = (self.skeleton.smart_pick, &self.timeline.doc);
        publica_o_osso(sim, osso_em_foco, doc, pick, linha);
        ph2d_card_rope::chave::resolve(&mut hero.store, sim.world()); // T36: o alfinete durável
        // O cartão solto preso a um osso: o retrato de cada preso e a corda.
        ph2d_panel_skeleton::publica_presos(&hero.store.loose_pins_of(BONES), |b| {
            let vivo = ph2d_skeleton_live::painel::live_bone(sim, b);
            publica_o_osso(sim, vivo, doc, pick, linha);
        });
        ph2d_card_rope::anda(&mut hero.store, BONES, |b| {
            ph2d_skeleton_live::skin_live::bone_middle(sim, b).map(|w| cam.world_to_screen(w, win))
        });
        #[cfg(feature = "panel-inspector")]
        anda_as_cordas_do_inspector(&mut hero.store, sim, &cam, win);
    }
}

/// O osso `osso` publicado no painel Bones (o em foco, ou um preso): tudo pela MESMA porta — por
/// portas separadas haveria um quadro com a faixa de um osso e o nome de outro.
fn publica_o_osso(
    sim: &ph2d_ecs::SimWorld,
    osso: Option<u64>,
    doc: ph2d_editor_core::DocPx,
    pick: Option<u64>,
    linha: &ph2d_timeline::TimelineDoc,
) {
    use ph2d_panel_skeleton as painel;
    let sujeito = osso.and_then(|b| ph2d_skeleton_live::painel::bone_subject(sim, b));
    painel::set_current_bone_subject(sujeito.as_ref().map(|s| s.0));
    painel::set_current_bone_name(sujeito.map_or_else(String::new, |s| s.1));
    let no_painel = osso.and_then(|b| ph2d_app_skeleton::knobs::spec_on_panel(sim, b, doc));
    painel::set_current_bone(no_painel.map(|(s, _)| s));
    painel::set_current_bone_px_per_unit(no_painel.map_or(1.0, |(_, k)| k));
    // O modo das alças é AUTORIA, e o `spec` acima é o autorado: em `From Chain` os números nem se
    // pintam (mostrar o derivado faria o painel parecer editável e não ser).
    let de = ph2d_ecs::Entity::from_bits;
    painel::set_current_bone_handles(osso.and_then(|b| {
        sim.world()
            .get::<ph2d_skeleton_ecs::Bone>(de(b))
            .map(|o| usize::from(o.handles == ph2d_skeleton::bend::Handles::Auto))
    }));
    painel::set_current_bone_limit(osso.and_then(|b| {
        sim.world()
            .get::<ph2d_skeleton_ecs::BoneLimit>(de(b))
            .map(|l| (l.min.to_degrees(), l.max.to_degrees()))
    }));
    let smart = osso.and_then(|b| {
        sim.world()
            .get::<ph2d_skeleton_ecs::SmartBone>(de(b))
            .cloned()
    });
    painel::set_current_bone_smart(smart.as_ref().map(|s| painel::SmartBoneView {
        from: s.from.to_degrees(),
        to: s.to.to_degrees(),
        clip: s.clip.clone(),
        target: s.target.clone(),
        picking: pick == osso,
    }));
    // A lista de acções filtrada pelo alvo, só com um osso inteligente em foco.
    painel::set_current_bone_actions(smart.as_ref().zip(osso).map_or_else(Vec::new, |(s, b)| {
        crate::skeleton_smart::actions_for(sim.world(), linha, s, de(b))
    }));
    // A lista da ponta sai da porta da família, a MESMA que o dreno lê: a posição é a escolha.
    painel::set_current_bone_tip(osso.and_then(|b| {
        ph2d_app_skeleton::curve_tip::options(sim, de(b)).map(|(o, _)| painel::TipView {
            rotulos: o.rotulos,
            ligado: o.ligado,
            escondidos: o.escondidos,
        })
    }));
    publica_a_ancora(sim, osso);
}

/// A âncora do osso em foco no painel — a conta é da família (`goal::ancora_no_painel`).
fn publica_a_ancora(sim: &ph2d_ecs::SimWorld, osso_em_foco: Option<u64>) {
    let a =
        crate::skeleton_goal::ancora_no_painel(sim, osso_em_foco.map(ph2d_ecs::Entity::from_bits));
    ph2d_panel_skeleton::set_current_bone_ik(a.ik);
    ph2d_panel_skeleton::set_current_bone_aim(a.aim);
    ph2d_panel_skeleton::set_current_bone_ik_auto_side(a.auto_side);
}

/// T36/T44 no Inspector: o Transform de cada objecto preso e a corda até à origem dele.
#[cfg(feature = "panel-inspector")]
fn anda_as_cordas_do_inspector(
    store: &mut ph2d_editor_core::interaction::WidgetStore,
    sim: &ph2d_ecs::SimWorld,
    cam: &ph2d_render::Camera2d,
    win: ph2d_host::WindowSize,
) {
    use ph2d_card_rope::chave::origem;
    use ph2d_editor_core::ids::INSP_PANEL as INSP;
    let w = sim.world();
    ph2d_card_rope::anda(store, INSP, |b| {
        origem(w, b).map(|p| cam.world_to_screen(p, win))
    });
}
