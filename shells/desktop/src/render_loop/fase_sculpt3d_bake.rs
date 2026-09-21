//! **Fase do quadro: O OBJETO MISTO DO SCULPT3D** — o bake da forma no sprite selecionado, o sprite
//! selecionado como padrão do pincel (o alpha por imagem) e a luz da cena a re-autorar os objetos que ela
//! assou quando a lâmpada anda (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **O módulo inteiro está atrás da `feature` `sculpt3d`** (o `mod` e a chamada no quadro), porque o
//! único statement dele já estava. ⛔ **A RE-ACENDIDA não mora aqui, de propósito**: ela é a fase
//! irmã `fase_relight_baked_forms`, que NÃO está atrás da feature — é o que torna verificável a
//! promessa da rota A (`docs/3D/02.2`): um objeto assado acende sem o módulo 3D no build.

use super::*;

/// **Fase irmã: o SPRITE VIRA O PADRÃO DO PINCEL** — ver o módulo.
///
/// ⚠️⚠️ **Ela é declarada AQUI e não no índice do quadro, e o motivo é um TECTO medido:** o
/// `render_loop/mod.rs` é um índice que cresce ~2 linhas por fase e estava a `599` de `600`.
/// ⭐ O texto emendado do quadro ([`crate::frame_text::splice`]) é **RECURSIVO** — uma fase
/// chamada de dentro de outra é emendada na mesma —, logo esta continua a ser lida por todo gate
/// de ordem desta shell sem custar uma linha ao índice.
#[cfg(feature = "sculpt3d")]
#[path = "fase_sculpt3d_alpha.rs"]
mod fase_sculpt3d_alpha;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_sculpt3d_bake(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sculpt3d,
            baked_forms,
            baked_light,
            next_baked_form,
            surface,
            renderer,
            sim,
            asset_db,
            toasts,
            hero_screen,
            atlas_asset_map,
            ..
        } = FrameGfx::of(gfx);

        // **O OBJETO MISTO** (`docs/3D/02.2`): assa a forma no sprite selecionado, e re-acende os
        // que já foram assados quando a lâmpada anda. Ele mora AQUI, e não ao lado da doação, por
        // uma razão só: é o único ponto do frame em que a cena 3D, o mundo, o renderizador e o
        // mapa de atlas estão os quatro em escopo.
        //
        // ⚠️ Quase sempre não faz nada — sem cena armada sai no primeiro `if`, e com o rig parado
        // ele custa um carimbo por sprite assado, sem tocar a GPU.
        #[cfg(feature = "sculpt3d")]
        if let Some(scene) = sculpt3d.as_mut() {
            let want = std::mem::replace(&mut self.sculpt3d_req.bake_request, false);
            let selected = hero_screen
                .as_ref()
                .and_then(|h| h.gizmo.iter_selected().next());
            if let Some(line) = ph2d_app_sculpt3d::bake::drain(
                scene,
                baked_forms,
                baked_light,
                next_baked_form,
                surface.gpu(),
                want,
                selected,
                sim,
                renderer,
                // ⚠️ **Lido ANTES do bake** — ele substitui a textura, e depois já não há 16
                // bits que ver. A porta é da shell; o que a família precisa é o veredito.
                selected.is_some_and(|bits| {
                    crate::hero_intents::texture_edit::holds_sixteen_bit(
                        ph2d_ecs::Entity::from_bits(bits),
                        sim,
                        renderer,
                    )
                }),
                // ⚠️ **PREGUIÇOSO**: um sprite já assado reúsa o `base` e nunca chega a
                // perguntar, e esta leitura custa uma volta ao device.
                &mut |sim: &mut ph2d_ecs::SimWorld, renderer: &mut _| {
                    selected.and_then(|bits| {
                        crate::hero_intents::texture_edit::read_sprite_source(
                            ph2d_ecs::Entity::from_bits(bits),
                            sim,
                            renderer,
                            asset_db,
                            atlas_asset_map,
                        )
                        .map(|s| s.image)
                    })
                },
            ) {
                // ⚠️ **O prefixo `[sculpt3d]` é do TERMINAL e não do aviso**, e a foto de 21/09 diz
                // porquê: ele aparecia na tela a comer metade da largura do toast, e a frase do
                // artista ficava em `this sprite is fully tra…`. *Quem bissecta quer a etiqueta;
                // quem está a desenhar quer a frase.*
                eprintln!("[sculpt3d] {}", line.frase());
                // ⭐⭐ **E a CARA do aviso segue o veredito** — ver [`Veredito`]: até 21/09 toda
                // recusa saía com o ✓ verde de sucesso, que é o que o dono fotografou.
                toasts.push(if line.assou() {
                    Toast::success(line.frase())
                } else {
                    Toast::error(line.frase())
                });
            }
            // ⭐⭐⭐⭐ **O VISOR PINTA A MATÉRIA QUE O BAKE VAI ACENDER** — ver
            // [`ph2d_app_sculpt3d::albedo::sincroniza`], que é onde a lei e o `61×` que a obrigou
            // estão medidos. Ela mora nesta fase e não na do desenho pelas DUAS razões que a fase
            // já declara no cabeçalho: é aqui que a cena 3D, o mundo, o renderizador e o mapa de
            // atlas estão os quatro em escopo — e ⚠️ **depois do bake de propósito**, porque assar
            // troca a matéria e a memória dela tem de ver a troca no MESMO quadro.
            ph2d_app_sculpt3d::albedo::sincroniza(
                scene,
                baked_forms,
                surface.gpu(),
                selected,
                sim,
                renderer,
                &mut |sim: &mut ph2d_ecs::SimWorld, renderer: &mut _| {
                    selected.and_then(|bits| {
                        crate::hero_intents::texture_edit::read_sprite_source(
                            ph2d_ecs::Entity::from_bits(bits),
                            sim,
                            renderer,
                            asset_db,
                            atlas_asset_map,
                        )
                        .map(|s| s.image)
                    })
                },
            );
            // Enquanto a cena existe, a luz dela AUTORA os objetos que ela assou — **mas só quando
            // o artista de fato MEXE nela**. Ver `Sculpt3dScene::take_rig_edge`: uma cena que
            // acabou de nascer não é um gesto, e tratá-la como um re-acendia todo objeto assado do
            // documento com o rig default, sem ninguém ter pedido.
            if scene.take_rig_edge() {
                ph2d_app_sculpt3d::bake::follow_live_rig(baked_forms, scene.rig());
            }
        }

        // ⚠️ **A irmã corre a seguir, e a ordem é a que ela tinha quando vivia aqui dentro** — o
        // gesto do padrão seguia o de assar no mesmo bloco. Ela re-deriva o `gfx` como toda fase
        // faz, e é por isso que esta chamada é a ÚLTIMA linha: o empréstimo daqui tem de acabar.
        #[cfg(feature = "sculpt3d")]
        self.fase_sculpt3d_alpha();
    }
}
