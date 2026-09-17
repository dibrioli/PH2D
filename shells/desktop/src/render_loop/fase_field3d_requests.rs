//! **Fase do quadro: OS PEDIDOS DO MODELADOR 3D** — os pedidos que a UI do `ph2d-app-field3d` deixa para a
//! shell servir: exportar, importar, re-ligar, a paleta de formas, o pick, os avisos e abrir o painel (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;
use ph2d_i18n::tr;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_field3d_requests(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
            toasts,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // ADR-0161 W4: o painel de modelagem abre sozinho na primeira vez que o
        // smoke desenha (auto-play), e só nessa — reabri-lo todo quadro faria o
        // botao de fechar dele nao funcionar.
        // ⭐ **O pedido de exportar**, tirado ao lado do de abrir o painel — os dois
        // atravessam da ponte com a cena para o app pela mesma porta, e por isso são
        // consumidos no mesmo sítio.
        if let Some(level) = ph2d_app_field3d::smoke::take_export_request() {
            ph2d_app_field3d::export::field3d_export(level, toasts);
        }
        // ⭐⭐⭐ **E a RESPOSTA da bancada** (`ph2d_app_field3d::export_job`): desde 2026-08-25 a
        // exportação corre fora da thread que desenha, e o que volta é a mensagem pronta.
        // ⚠️ Ela é drenada **aqui**, ao lado do pedido, pela lei das caixas de correio deste
        // módulo — *uma porta, vários pedintes*.
        if let Some(done) = ph2d_app_field3d::export_job::take_finished() {
            toasts.push(ph2d_editor_core::Toast::info(done));
        }
        // ⭐ **E o de IMPORTAR**, pela mesma porta e pelo mesmo motivo (ADR-0161 W22).
        if ph2d_app_field3d::smoke::take_import_request() {
            ph2d_app_field3d::import::field3d_import(toasts);
        }
        // ⭐⭐⭐ **E o de RELIGAR** (W76), pela mesma porta e pelo mesmo motivo: escolher o
        // arquivo é um diálogo, e um diálogo não corre com o mundo emprestado.
        if let Some(e) = ph2d_app_field3d::smoke::take_relink_request() {
            ph2d_app_field3d::import::field3d_relink(e, toasts);
        }
        // ⭐⭐ **O PERFIL DESENHADO VIRA PEÇA** (W53) — o fluxo do MoI, que o motor tem medido e
        // gateado desde a W3 e que **nenhum botão alcançava**.
        //
        // ⚠️ Servido **aqui** porque quem tem a cena vetorial é o `AppGfx`; a ponte com a cena
        // recebe o mundo. É a mesma divisão dos pedidos acima.
        //
        // ⚠️ **E o shell publica se HÁ contorno**, todo quadro: é isso que faz os dois botões
        // aparecerem só quando há o que extrudar (a lei da W34).
        {
            let closed = crate::blend_live::selected_closed_in_z(vec_scene, &self.vec.pen);
            ph2d_app_field3d::smoke::note_profile(closed.first().copied());
            if let Some(which) = ph2d_app_field3d::smoke::take_profile_request() {
                let msg = ph2d_app_field3d::profile::from_selection(vec_scene, &closed, which);
                toasts.push(ph2d_editor_core::Toast::info(msg));
            }
        }
        // ⭐ **E a escultura da CENA** (W39) — o vínculo que não passa pelo disco.
        //
        // ⚠️ Ela é servida **aqui** e não na ponte com a cena porque quem tem a escultura viva é
        // o `AppGfx`; a ponte recebe o mundo. É a mesma divisão dos dois pedidos acima.
        //
        // ⚠️ **E o shell publica se ela EXISTE**, todo quadro: é isso que faz o botão aparecer
        // só quando há o que trazer (a lei da W34). Sem a feature, fica sempre falso.
        #[cfg(feature = "sculpt3d")]
        {
            let live = sculpt3d
                .as_ref()
                .map(ph2d_app_sculpt3d::Sculpt3dScene::mesh);
            ph2d_app_field3d::smoke::note_live_sculpt(live.is_some());
            if ph2d_app_field3d::smoke::take_scene_sculpt_request() {
                let msg = live.map_or_else(
                    || tr("shell.fase_field3d_requests.there_is_no_sculpture").to_string(),
                    |m| ph2d_app_field3d::import::field3d_scene_sculpt(m.clone()),
                );
                toasts.push(ph2d_editor_core::Toast::info(msg));
            }
        }
        // ⭐⭐⭐ **A PALETA DE FORMAS, as DUAS pontas** (W100) — abrir para quem pediu, e mandar
        // ao mundo o que ela escolheu. Irmã por assunto do `component_attach`, que faz o mesmo
        // com o `+` do Inspector.
        //
        // ⚠️ **Aberta DEPOIS dos dois `note_*` acima**, e a ordem é load-bearing: o modelo dela
        // carrega a disponibilidade de *Extrude*/*Revolve*/*Sculpt from scene*, e construí-lo
        // antes das notas deste quadro mostraria a resposta do quadro anterior — visível
        // exatamente no gesto que importa (escolher o contorno e abrir a paleta a seguir).
        if ph2d_app_field3d::smoke::take_shape_palette_request() {
            let (live_sculpt, profile) = ph2d_app_field3d::smoke::palette_conditions();
            hero.store
                .open_command_palette(ph2d_app_field3d::shape_palette::build(live_sculpt, profile));
        }
        // ⚠️ O pick chega **noutro quadro** (a paleta fica aberta), e o dreno é **CONDICIONAL**:
        // este canal já tinha TRÊS consumidores (a biblioteca do Motion, o `Ctrl+K` e o `+` do
        // Inspector), e um `take` incondicional engoliria o pick de outro — com o sintoma a ser
        // *«às vezes não faz nada»*.
        if let Some(id) = hero
            .store
            .take_command_pick_if(|id| ph2d_app_field3d::shape_palette::slot_of_pick(id).is_some())
            && let Some(slot) = ph2d_app_field3d::shape_palette::slot_of_pick(id)
        {
            ph2d_app_field3d::smoke::ask_shape(slot);
        }
        // ⭐ **E o que o módulo tem a DIZER** (W23 + W25): a escultura que não voltou do
        // arquivo, e a peça que não cozinha. A ponte com a cena descobre as duas ao cozer o
        // documento, e a fila de avisos é daqui. Sem esta linha as duas falham em silêncio — a
        // peça some da tela e nada explica porquê.
        for msg in ph2d_app_field3d::notice::drain() {
            toasts.push(ph2d_editor_core::Toast::info(msg));
        }
        if ph2d_app_field3d::smoke::take_open_panel_request() {
            // O ID vem do PAINEL, nunca de um literal: uma segunda cópia da chave de
            // visibilidade é como se abre um painel que ninguém pinta.
            hero.panel_visibility
                .insert(ph2d_panel_model3d::PANEL_ID, true);
        }
    }
}
