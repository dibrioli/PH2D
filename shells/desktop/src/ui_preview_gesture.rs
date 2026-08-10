//! **O RATO dentro do modo de preview** (plano UI/UX W7r) — a metade da shell, ao lado do modelo
//! em [`crate::render_loop::ui_preview`].
//!
//! # O gesto é MODAL, e por isso mora antes de tudo
//!
//! Enquanto a preview corre não existe seleção, arrasto, gizmo nem caneta: o clique é da UI que o
//! artista desenhou. É a mesma classe dos picks armados (o conta-gotas de caminho-guia, o
//! eyedropper de corpo do joint), e a razão é a mesma — *um modo em curso é dono do clique*.
//!
//! ⚠️ **O Move NÃO é consumido, e a assimetria é deliberada.** O `Down`/`Up` primário são da
//! preview porque um deles abriria um arrasto de edição; o movimento do cursor não abre nada, e
//! consumi-lo mataria o pan e o zoom — que o Figma mantém vivos no modo de apresentação dele pela
//! mesma razão: olhar de perto não é editar.

use ph2d_vec_scene::VecPathId;

impl crate::App {
    /// **A CADEIA de hospedeiros sob o cursor**, em coordenadas de tela — do mais interno
    /// para fora, vazia se nenhum.
    ///
    /// ⚠️ **As anotações NÃO são filtradas aqui**, ao contrário do irmão do conector
    /// ([`crate::App::shape_under_cursor`]): um rótulo *é* parte do botão, e pulá-lo faria o hover
    /// morrer exatamente onde o texto está — que é o meio do controle. A pergunta certa é *"o que
    /// foi tocado pertence à sub-árvore de algum hospedeiro?"*, e quem a responde é
    /// [`crate::render_loop::ui_preview::host_under`].
    pub(crate) fn ui_preview_host_at(&self, sx: f32, sy: f32) -> Vec<VecPathId> {
        let Some(gfx) = self.gfx.as_ref() else {
            return Vec::new();
        };
        let Some(world) = self.vec_world_at((sx, sy)) else {
            return Vec::new();
        };
        let window_size = gfx.surface.size();
        let view = crate::vec_entities::view_state_for_pick(
            &gfx.sim,
            &self.vec_entities,
            &self.vec_view_derived,
        );
        let hits = crate::vec_gizmo_view::pick_all_at_world(
            &gfx.sim,
            &gfx.vec_scene,
            self.offset_live.live(),
            &view,
            &self.vec_entities,
            [world[0] as f32, world[1] as f32],
            crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, window_size),
        );
        // `pick_all_at_world` devolve do TOPO para o fundo (bits de entidade); a preview quer
        // caminhos, e o **primeiro** que pertence a um hospedeiro ganha — o de cima é o que o
        // artista vê.
        let picked: Vec<VecPathId> = hits
            .into_iter()
            .filter_map(|bits| {
                self.vec_entities
                    .iter()
                    .find(|&(_, &b)| b == bits)
                    .map(|(&id, _)| id)
            })
            .collect();
        crate::render_loop::ui_preview::host_under(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec_entities,
            &gfx.ui_states,
            &picked,
        )
    }

    /// **Entrega os dois fatos do rato à preview.** `pressed` é o botão primário AGORA.
    ///
    /// ⚠️ O hospedeiro é recomputado a cada evento em vez de guardado: o `Down` pode chegar sobre
    /// outra forma que o último `Move` (um clique rápido depois de um salto de cursor não emite
    /// `Move` nenhum em alguns compositores), e um `hot` memorizado acenderia o botão errado.
    pub(crate) fn ui_preview_point(&mut self, sx: f32, sy: f32, pressed: bool) {
        if !self.ui_preview.is_on() {
            return;
        }
        let hit = self.ui_preview_host_at(sx, sy);
        // ⚠️ *Field splitting*: `ui_preview` mora no `App` e as máquinas dentro do `gfx`, e os
        // dois são emprestados no mesmo chamado. Um helper que devolvesse o `gfx` inteiro
        // emprestaria o bloco e o compilador recusaria — a mesma razão do `Deref` do motor de
        // água.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let clicked = self
            .ui_preview
            .point(&mut gfx.ui_machines, &gfx.ui_states, &hit, pressed);
        // ⭐ **UM CLIQUE NA PREVIEW GRITA O NOME DO QUE FOI CLICADO** — a metade de PRODUTOR da
        // tabela sinal → ação (item 4 do estudo dos contêineres).
        //
        // ⚠️ **Nenhum campo autorado novo, e isso é o desenho:** o nome do sinal é o `Name` da
        // entidade — o mesmo que a Hierarquia mostra e o mesmo que um botão do painel autorado já
        // publica (`AuthoredIntent::Fired`). Um segundo lugar para digitar o nome de uma coisa é
        // um segundo lugar que discorda do primeiro, e é literalmente a razão pela qual o
        // `VecWidget` não guarda `key`.
        //
        // ⚠️ **Quem dispara é um HOSPEDEIRO, e é a cerca que impede o ruído:** só uma forma com
        // poses autoradas está na cadeia, então um desenho comum clicado na preview não grita
        // nada. *Interativo* já era a condição de existir aqui.
        //
        // ⚠️ **E ele publica na MESMA saída da timeline e da física** (R0/ADR-0143): quem escuta
        // casa num NOME e nunca pergunta a origem, então uma colisão e um botão com o mesmo nome
        // movem a mesma cena. É o desacoplamento do ADR-0075, e é o que torna a ligação reusável.
        if let Some(host) = clicked
            && let Some(name) = host_name(gfx, &self.vec_entities, host)
        {
            self.signals
                .publish(ph2d_runtime::Signal::from_control(&name));
        }
    }
}

/// O `Name` da entidade de `host` — o nome com que ele grita.
fn host_name(
    gfx: &crate::app_state::AppGfx,
    map: &crate::vec_entities::VecEntityMap,
    host: VecPathId,
) -> Option<String> {
    let bits = *map.get(&host)?;
    gfx.sim
        .world()
        .get::<ph2d_ecs::Name>(ph2d_ecs::Entity::from_bits(bits))
        .map(|n| n.0.clone())
}
