//! **A costura do gizmo da §12 com o ponteiro** — a ponta que faltava para a marca deixar de ser
//! decoração.
//!
//! A matemática vive em [`crate::render_loop::anchor_gizmo`], pura e testada. Aqui mora só o que
//! precisa do mundo: onde está o cursor, quem está selecionado, e por que porta a edição sai.
//!
//! # ⚠️ A edição sai pela MESMA porta do painel
//!
//! O arrasto empurra `EditorAction::InspectorAnchorEdit` no barramento — a ação que a §12 já
//! publica quando o artista digita um número. Não há um segundo caminho de escrita: o commit da
//! shell, o saneamento, o `Toast` de recusa e o undo são os mesmos, e por isso **não podem
//! divergir**. *Duas portas para o mesmo estado é como duas portas divergem* — a lei que esta
//! linha pagou três vezes esta semana.
//!
//! # ⚠️ Um arrasto é UM passo de undo, e ninguém aqui trata disso
//!
//! O `post_frame_undo` suprime o registo enquanto um botão está premido, então o gesto inteiro
//! fecha num passo. É a mesma lei do arrasto da âncora de joint, e é a razão de este ficheiro não
//! ter máquina de transação nenhuma.

use crate::render_loop::anchor_gizmo;

impl crate::App {
    /// Tenta agarrar uma alça do gizmo de âncora. `true` = agarrou, e o `Down` não deve seguir
    /// para o pick de sprite.
    ///
    /// ⚠️ **A seção tem de estar EXPANDIDA**, pela mesma razão que o desenho: alças invisíveis que
    /// agarram são pior que alças que não existem. É o mesmo booleano que o overlay lê.
    pub(crate) fn try_open_anchor_gizmo_drag(&mut self, sx: f32, sy: f32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return false;
        };
        if hero
            .store
            .is_collapsed(ph2d_editor::ids::INSP_LIVE_ANCHOR_SECTION)
        {
            return false;
        }
        // ⚠️ **Um clique DENTRO de um painel nunca agarra uma alça.** As alças vivem em
        // coordenadas de mundo e nada as impede de cair debaixo do Inspector; sem esta porta,
        // clicar num campo da §12 arrancaria um arrasto por trás dele. É a mesma guarda que o
        // arrasto da âncora de joint faz.
        if hero.store.panel_at(sx, sy).is_some() {
            return false;
        }
        let Some(bits) = hero.gizmo.selection else {
            return false;
        };
        let ppm = hero.project.pixels_per_meter;
        let window = gfx.surface.size();
        // O raio de agarre é de TELA; converte-se ao mundo pelo mesmo caminho que o ímã do joint.
        let tol = anchor_gizmo::GRAB_PX * gfx.camera.height_world / window.height as f32;
        let [wx, wy] = gfx.camera.screen_to_world((sx, sy), window);
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let sim = gfx.sim.world();
        let Some(list) = sim.get::<ph2d_ecs::NamedAnchorList>(entity) else {
            return false;
        };
        let Some(sprite_world) = ph2d_ecs::world_transform(sim, entity) else {
            return false;
        };
        let opened = anchor_gizmo::open_drag(
            sprite_world,
            list,
            ph2d_panel_inspector::open_anchor_row(),
            bits,
            ph2d_core::Vec2::new(wx, wy),
            ppm,
            tol,
        );
        self.anchor_gizmo_drag = opened;
        opened.is_some()
    }

    /// Segue o cursor com a alça agarrada, publicando as edições do quadro.
    pub(crate) fn advance_anchor_gizmo_drag(&mut self) {
        let Some(drag) = self.anchor_gizmo_drag else {
            return;
        };
        let pointer = self.last_pointer;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let window = gfx.surface.size();
        let [wx, wy] = gfx.camera.screen_to_world(pointer, window);
        let entity = ph2d_ecs::Entity::from_bits(drag.entity);
        let sim = gfx.sim.world();
        // ⚠️ **A âncora relê-se do MUNDO a cada quadro**, e não se guarda uma cópia na abertura:
        // a edição do quadro anterior já entrou, e arrastar contra a pose velha faria o gesto
        // pisar-se a si próprio — o canto redimensionaria sempre a partir do rect original.
        let (Some(list), Some(sprite_world)) = (
            sim.get::<ph2d_ecs::NamedAnchorList>(entity),
            ph2d_ecs::world_transform(sim, entity),
        ) else {
            return;
        };
        let Some(a) = list.iter().nth(usize::from(drag.row)) else {
            // A âncora foi apagada a meio do arrasto (undo, outra janela): larga em vez de
            // escrever no índice de outra.
            self.anchor_gizmo_drag = None;
            return;
        };
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return;
        };
        let ppm = hero.project.pixels_per_meter;
        let (edits, n) = anchor_gizmo::drag(
            drag.kind,
            drag.row,
            a,
            sprite_world,
            ph2d_core::Vec2::new(wx, wy),
            ppm,
        );
        if n == 0 {
            return;
        }
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return;
        };
        for e in edits.into_iter().flatten() {
            hero.bus
                .push(ph2d_editor::action_bus::EditorAction::InspectorAnchorEdit {
                    entity_bits: drag.entity,
                    edit: e,
                });
        }
        self.any_input_this_frame = true;
    }

    /// Larga a alça. Chamado no `Up`, e também quando o gesto é cancelado.
    pub(crate) fn end_anchor_gizmo_drag(&mut self) {
        self.anchor_gizmo_drag = None;
    }
}
