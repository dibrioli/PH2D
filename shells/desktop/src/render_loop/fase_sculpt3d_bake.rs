//! **Fase do quadro: O OBJETO MISTO DO SCULPT3D** — o bake da forma no sprite selecionado, o sprite
//! selecionado como padrão do pincel (o alpha por imagem) e a luz da cena a re-autorar os objetos que ela
//! assou quando a lâmpada anda (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **O módulo inteiro está atrás da `feature` `sculpt3d`** (o `mod` e a chamada no quadro), porque o
//! único statement dele já estava. ⛔ **A RE-ACENDIDA não mora aqui, de propósito**: ela é a fase
//! irmã `fase_relight_baked_forms`, que NÃO está atrás da feature — é o que torna verificável a
//! promessa da rota A (`docs/3D/02.2`): um objeto assado acende sem o módulo 3D no build.

use super::*;

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
            tools,
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
                eprintln!("{line}");
                toasts.push(Toast::success(line));
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
            // **O SPRITE SELECIONADO VIRA O PADRÃO DO PINCEL** — o alpha por
            // IMAGEM. Mora aqui pela MESMA razão do bake logo acima: é o único
            // ponto do frame em que a cena 3D, o mundo 2D, o renderizador e o
            // mapa de atlas estão os quatro em escopo.
            if std::mem::replace(&mut self.sculpt3d_req.alpha_request, false) {
                // ── O que o artista VÊ, e não o que o sprite GUARDA. ──
                // ⚠️ Um sprite cuja aparência vem do sistema de CAMADAS do Painter (procedurais,
                // ajustes, blend) ainda aponta para a imagem de origem: ler a origem devolve outra
                // textura, e o padrão sai diferente do que está na tela (Enio, 2026-08-09: *"veja a
                // textura ao lado e veja a textura no preview"*).
                //
                // ⚠️ **A porta já existe e é a MESMA do "Use as Brush Grain"** — o Painter resolveu
                // esta pergunta uma vez (`composite_to_lum`, cujo doc-comment a nomeia) e uma
                // segunda resposta divergiria dela. Ela é pedida SEM ativar o Painter
                // (`tool_by_id_mut`), porque perguntar o que a tela mostra não pode trocar a
                // ferramenta da mão do artista.
                //
                // ⚠️ E a luminância entra como CINZA OPACO: a lei do `AlphaImage` é
                // `luminância × alfa`, e sobre um cinza opaco ela devolve a própria luminância —
                // então o composite atravessa exato, sem um segundo cálculo de luminância.
                let live = selected.and_then(|bits| {
                    let painter = tools
                        .tool_by_id_mut(&ph2d_editor_core::ToolId::new("painter"))?
                        .as_any_mut()
                        .downcast_mut::<ph2d_tool_painter::PainterTool>()?;
                    if painter.needs_document_bind(bits) {
                        return None;
                    }
                    let (lum, w, h) = painter.composite_to_lum()?;
                    let rgba: Vec<u8> = lum.iter().flat_map(|&l| [l, l, l, 255]).collect();
                    ph2d_sculpt3d::AlphaImage::from_rgba(w, h, &rgba).map(|a| (a, "as CAMADAS"))
                });
                // ── E o que o sprite guarda, quando não há camadas vivas. ──
                // ⚠️ **STRAIGHT, e a conversão é load-bearing:** a lei do `AlphaImage` é
                // `luminância × alfa`, e num buffer PREMULTIPLICADO a luminância já traz o alfa
                // dentro — o peso sairia com o alfa ao QUADRADO, e toda borda macia ficaria mais
                // fina do que o desenho é. Um sprite `Individual` volta premultiplicado do
                // readback, então este não é um caso de canto.
                let baked = || {
                    let r = selected.and_then(|bits| {
                        crate::hero_intents::texture_edit::read_sprite_source(
                            ph2d_ecs::Entity::from_bits(bits),
                            sim,
                            renderer,
                            asset_db,
                            atlas_asset_map,
                        )
                    })?;
                    let img = r.image.into_straight();
                    ph2d_sculpt3d::AlphaImage::from_rgba(img.width, img.height, &img.pixels)
                        .map(|a| (a, "a imagem"))
                };
                let line = match live.or_else(baked) {
                    Some((a, what)) => {
                        // ⚠️ **O readout diz a ESCALA, não os pixels.** A versão anterior
                        // reportava `WxH` — e a medição mostra que a resolução da fonte tem efeito
                        // **ZERO** sobre o tamanho do padrão no modelo (o `AlphaImage::sample`
                        // mapeia em unidades de LADRILHO: a mesma imagem a 64² e a 4096² dá 80
                        // transições ao longo das mesmas 2 unidades de objeto). O número que de
                        // fato governa o que o artista vê é o `Alpha Scale`, e era justamente ele
                        // que mudava sem aparecer em lugar nenhum.
                        // ⚠️ **O nome vem do `Name` do objeto, com o fallback
                        // dizendo o que ele É.** Um sprite pode não ter nome —
                        // e um chip em branco seria o mesmo defeito do "None"
                        // que esta wave conserta, com outra roupa.
                        let from: std::sync::Arc<str> = selected
                            .and_then(|bits| {
                                sim.world()
                                    .get::<ph2d_ecs::Name>(ph2d_ecs::Entity::from_bits(bits))
                                    .map(|n| std::sync::Arc::from(n.0.as_str()))
                            })
                            .unwrap_or_else(|| std::sync::Arc::from("Sprite"));
                        let scale = scene.set_alpha_image(a, from);
                        format!(
                            "[sculpt3d] padrao: {what} do sprite selecionado (escala {scale:.3})"
                        )
                    }
                    None if selected.is_some() => {
                        "[sculpt3d] o sprite nao descreve uma imagem".to_string()
                    }
                    None => "[sculpt3d] selecione um sprite para usar como padrao".to_string(),
                };
                eprintln!("{line}");
                toasts.push(Toast::success(line));
            }
            // Enquanto a cena existe, a luz dela AUTORA os objetos que ela assou — **mas só quando
            // o artista de fato MEXE nela**. Ver `Sculpt3dScene::take_rig_edge`: uma cena que
            // acabou de nascer não é um gesto, e tratá-la como um re-acendia todo objeto assado do
            // documento com o rig default, sem ninguém ter pedido.
            if scene.take_rig_edge() {
                ph2d_app_sculpt3d::bake::follow_live_rig(baked_forms, scene.rig());
            }
        }
    }
}
