//! **Fase do quadro: AS PONTES DO MODELADOR 3D, DOS TOKENS E DA ESCULTURA** — a peça de modelagem como entidade, o painel que um projecto abre, a fila da área, a
//! selecção do gizmo 3D, o painel de tokens e o painel da escultura (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;
use ph2d_i18n::tr;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_world_panel_bridges(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
            // ⚠️ **O mapa dos objectos ASSADOS é do shell**, e é por isso que a lei de cada um
            // entra e sai do painel por aqui: a escultura não sabe que um sprite foi assado.
            baked_forms,
            sim,
            toasts,
            tools,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // ADR-0161 W4: a peça de modelagem 3D é uma ENTIDADE, e esta é a ponte
        // que a mantém assim — nasce no mundo, aparece na Hierarquia, e as
        // edições do painel escrevem no COMPONENTE. Inerte sem o smoke armado.
        // ⭐ **O pill é a porta de armar.** A visibilidade do painel É o interruptor do
        // módulo: enquanto a única entrada era `PH2D_FIELD_SMOKE`, ele não existia
        // para quem abre o app.
        // ⭐ **QUEM TOMA O CANVAS LIBERTA QUEM O TINHA** (W40). Enio, 2026-08-22: *"o modo
        // Modelagem nunca é desativado e não consigo usar nenhum outro modo do app… Não consigo
        // esculpir nada pois o modo de modelagem permanece interferindo."*
        //
        // ⚠️ Fecha-se o **painel**, e não se desarma em silêncio: o pill *é* o interruptor do
        // módulo (a linha abaixo), então um desarme invisível deixaria o botão aceso a mentir.
        // A lei (borda, não estado contínuo) e o porquê estão em `ph2d_app_field3d::mode`.
        {
            let clay_on = {
                #[cfg(feature = "sculpt3d")]
                {
                    sculpt3d
                        .as_ref()
                        .is_some_and(ph2d_app_sculpt3d::Sculpt3dScene::clay_on_screen)
                }
                #[cfg(not(feature = "sculpt3d"))]
                {
                    false
                }
            };
            let owner = ph2d_app_field3d::mode::Owner {
                tool: tools.active().map(ph2d_editor_core::Tool::id),
                clay: clay_on,
            };
            if ph2d_app_field3d::mode::note_owner(owner.clone())
                && hero.is_panel_visible(ph2d_panel_model3d::PANEL_ID)
            {
                hero.panel_visibility
                    .insert(ph2d_panel_model3d::PANEL_ID, false);
                toasts.push(ph2d_editor_core::Toast::info(tr(
                    "shell.fase_world_panel_bridges.modelling_stepped",
                )));
            }
            // ⭐⭐ **E a metade SIMÉTRICA**: abrir o MODEL tira o barro da tela — e, desde
            // 2026-08-31, também **larga a ferramenta em mãos**.
            //
            // ⛔⛔ **A lei dizia-se «duas metades simétricas» e só uma soltava uma FERRAMENTA.**
            // Report do Enio: *«se abro Nodes e depois Model, o grafo de Nodes persiste»*. Ele
            // chegou lá pela aba nova, mas o defeito é do módulo e é anterior a ela: abrir o
            // MODEL pelo menu *Window* com o Motion em mãos deixava a `motion_bridge` a
            // reabrir `motion_graph`/`motion_params` **a cada quadro**, porque nada largava a
            // ferramenta. *A metade que faltava não era um caso — era o outro lado da lei.*
            //
            // ⚠️ **O edge é lido UMA vez e fora do `cfg`**: o `model_just_opened` CONSOME a
            // transição (ele troca o valor guardado), então uma segunda chamada no mesmo
            // quadro leria `false` — e, enquanto ele vivia dentro do `#[cfg(sculpt3d)]`, uma
            // build sem aquela feature nunca o avançava.
            let model_opened = ph2d_app_field3d::mode::model_just_opened(
                hero.is_panel_visible(ph2d_panel_model3d::PANEL_ID),
            );
            //
            // ⚠️ A decisão E o re-baseline vivem os dois no `model_takes_the_canvas` — são um
            // acto só, e separá-los deixava uma mutação sobreviver com o produto em ciclo.
            if model_opened
                && let Some(neutral) = tools.default_tool_id()
                && ph2d_app_field3d::mode::model_takes_the_canvas(&owner, &neutral)
            {
                tools.set_active(&neutral);
                self.title_dirty = true;
                toasts.push(ph2d_editor_core::Toast::info(tr(
                    "shell.fase_world_panel_bridges.modelling_took_the",
                )));
            }
            // ⚠️ A saída do BARRO é a **porta do próprio módulo de escultura**
            // (`toggle_clay`), nunca uma escrita aqui: ela conhece a ordem do ciclo (sair do
            // barro vai para a LUZ, não para o desligado), e essa ordem é uma decisão de
            // produto com um dono.
            #[cfg(feature = "sculpt3d")]
            if model_opened
                && let Some(scene) = sculpt3d.as_mut()
                && scene.clay_on_screen()
            {
                let label = scene.toggle_clay();
                eprintln!("[field3d] o MODEL abriu; a escultura cedeu -> {label}");
            }
        }
        // ⭐⭐ **UM PROJETO QUE TRAZ UMA PEÇA ABRE O PAINEL** (W45) — a resposta à pergunta que o
        // load deixou. ⚠️ É aqui e não no load porque **o mundo vive no `gfx`**, e o load corre
        // sem janela: perguntar lá daria *"não há peça"* sempre. Mesma forma (e mesma razão
        // escrita) do `sculpt3d_install_pending` do módulo irmão.
        if ph2d_app_field3d::smoke::take_open_if_part_request()
            && ph2d_app_field3d::scene::world_has_a_part(sim.world_mut())
        {
            ph2d_app_field3d::smoke::ask_open_panel();
        }
        ph2d_app_field3d::smoke::set_armed_by_panel(
            hero.is_panel_visible(ph2d_panel_model3d::PANEL_ID),
        );
        // ⭐⭐⭐ **A FILA É O CABEÇALHO DA ÁREA** — as vistas e a câmera saíram do painel e
        // pintam-se na fila de ferramentas, que já é uma região da área e já subtrai a altura
        // que subtrai (`ph2d_panel_model3d::area_bar`).
        //
        // ⚠️ **Escrito em TODO quadro, desarmado incluído** — é a mesma lei do transbordo do
        // `⋯`: quem fecha o módulo deixa de contribuir e a fila volta ao que era **no mesmo
        // quadro**. Sem o ramo vazio, nove chips ficavam na fila a despachar para um painel
        // que já não está lá.
        let model_armed = hero.is_panel_visible(ph2d_panel_model3d::PANEL_ID);
        ph2d_panel_model3d::publish_area_bar(&mut hero.store, model_armed);
        // ⭐ A seleção do app é a do gizmo 3D: clicar numa linha da Hierarquia é o que faz as
        // setas aparecerem no objeto. Uma seleção própria deste módulo seria uma segunda ideia
        // de "o que está selecionado" no mesmo app.
        // ⭐ Um clique na peça (ou a peça a nascer) pede uma seleção. É a MESMA porta que a
        // Hierarquia usa — uma seleção própria deste módulo seria uma segunda ideia de "o que
        // está selecionado" dentro do mesmo app.
        if let Some(req) = ph2d_app_field3d::scene::ecs_bridge(
            sim,
            hero.gizmo.selection,
            &hero.gizmo.extra_selection,
            vec_scene,
        ) {
            // ⭐ **A lei mora numa porta só** (`field3d_scene::apply`) — o gate chama a MESMA.
            ph2d_app_field3d::scene::apply(&mut hero.gizmo, req);
        }
        // O painel de TOKENS (plano UI/UX W6), na MESMA fase e pela mesma razão: um painel de
        // MUNDO, cuja visibilidade é do artista. ⚠️ Ele tem de correr DEPOIS do dispatch de
        // eventos (o intent de Reset é enfileirado ali) e ANTES do paint (senão o frame
        // pintaria a cor de antes do clique e o picker piscaria de volta).
        if tokens_bridge::dispatch(hero, toasts) {
            self.title_dirty = true;
        }
        // O painel da cena 3D (ADR-0150 W12), na MESMA fase e pela mesma
        // razão dos dois acima: depois do dispatch de eventos (os intents
        // são enfileirados ali) e ANTES do paint (senão o frame pintaria o
        // estado de antes do clique e o chip piscaria de volta).
        // ⚠️ O retorno é o pedido de BAKE, e ele arma o MESMO campo que o
        // `Shift+B` — uma porta, dois pedintes. O gesto é consumido no
        // dispatch do frame SEGUINTE (o `bake::drain` roda mais cedo neste),
        // exatamente como o do teclado.
        // ⭐ **A ponte do painel da escultura vive na fase-filha** logo abaixo — ver o doc dela
        // para porque o corte é por RESPONSABILIDADE e porque o NOME dela começa por `fase_`.
        #[cfg(feature = "sculpt3d")]
        ponte_do_sculpt3d(hero, sculpt3d, baked_forms, &mut self.sculpt3d_req);
        // ⭐⭐⭐ **E O QUE A ESCULTURA TEM A DIZER CHEGA AO ECRÃ** — a caixa de saída da cena
        // (`Sculpt3dScene::take_avisos`), drenada aqui porque é aqui que a fila de avisos e a cena
        // estão os dois em mão. ⚠️ **As recusas da retopologia viviam só no `eprintln!`**, com a
        // cura escrita dentro da frase (*ACHATE a pilha antes*) e o ecrã calado — as DUAS entradas
        // (a tecla `R`/`K` e o botão do painel) falam agora pela mesma porta.
        #[cfg(feature = "sculpt3d")]
        if let Some(scene) = sculpt3d.as_mut() {
            for aviso in scene.take_avisos() {
                toasts.push(ph2d_editor_core::Toast::warning(aviso));
            }
        }
    }
}

/// ⭐⭐ **A PONTE DO PAINEL DA ESCULTURA** — publicar o retrato e aplicar o que o artista pediu.
///
/// ⚠️ **Fase-FILHA, e o corte é por RESPONSABILIDADE:** a irmã acima é *«os painéis de MUNDO»*, e
/// isto é um assunto só (a escultura). O corte foi obrigado pelo tecto de LOC por FUNÇÃO (`201`
/// contra `200`) quando a fileira da LEI chegou — ⛔ e a cura é partir, nunca subir o número do
/// `FN_OVERAGE_OK`, que só desce.
///
/// ⚠️⚠️ **O NOME NÃO começa por `fase_`, e isso é uma decisão com gate a confirmá-la:** o texto
/// emendado do quadro colhe as funções com esse prefixo **e emenda-as onde o QUADRO as chama**
/// (`self.fase_x()`); esta é chamada de dentro de outra fase, logo com aquele nome ela apareceria
/// como **fase ÓRFÃ** — *«definida e nunca chamada pelo quadro»*. ⭐ E a asserção do oráculo fica
/// honesta: ela não é uma fase do quadro, é o corpo de uma. ⇒ os censos que a medem leem o
/// FICHEIRO (`source("render_loop/fase_world_panel_bridges.rs")`), nunca o texto do quadro.
///
/// ⚠️ Ela corre DEPOIS do dispatch de eventos (os intents são enfileirados ali) e ANTES do paint —
/// senão o quadro pintaria o estado de antes do clique e o chip piscaria de volta.
#[cfg(feature = "sculpt3d")]
fn ponte_do_sculpt3d(
    hero: &mut ph2d_editor_core::screens::hero::HeroScreen,
    sculpt3d: &mut Option<ph2d_app_sculpt3d::Sculpt3dScene>,
    baked_forms: &mut std::collections::BTreeMap<u64, ph2d_form_donation::baked_form::BakedForm>,
    pedidos: &mut ph2d_app_sculpt3d::Sculpt3dRequests,
) {
    // ⭐⭐ **A LEI do objecto assado** — lida ANTES do despacho (o painel pinta o estado de
    // agora) e escrita DEPOIS dele. ⚠️ `None` quando o sprite escolhido ainda não tem canais:
    // sem canais não há lei para escolher, e a fileira nem é pintada.
    let lei_do_alvo = hero
        .gizmo
        .iter_selected()
        .next()
        .and_then(|bits| baked_forms.get(&bits))
        .map(|b| b.lei.index());
    for req in ph2d_app_sculpt3d::panel_bridge::dispatch(hero, sculpt3d.as_mut(), lei_do_alvo) {
        match req {
            ph2d_app_sculpt3d::Sculpt3dFrameRequest::Bake => {
                pedidos.bake_request = true;
            }
            ph2d_app_sculpt3d::Sculpt3dFrameRequest::AlphaFromSprite => {
                pedidos.alpha_request = true;
            }
            // ⭐⭐⭐ **Trocar a lei é escrever no DOCUMENTO do objecto e ESQUECER o carimbo.**
            // ⚠️ O `lit_with = None` não é higiene: ele é a porta única da re-acendida
            // (`relight_stale` pergunta *«estes pixels foram acesos pelo rig de agora?»*), e sem
            // ele a lei nova só apareceria no dia em que o artista mexesse numa lâmpada.
            ph2d_app_sculpt3d::Sculpt3dFrameRequest::LeiDoAlvo(lei) => {
                if let Some(bits) = hero.gizmo.iter_selected().next()
                    && let Some(bake) = baked_forms.get_mut(&bits)
                {
                    bake.lei = lei;
                    bake.lit_with = None;
                }
            }
        }
    }
}
