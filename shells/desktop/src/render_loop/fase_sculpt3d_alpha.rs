//! **Fase do quadro: O SPRITE SELECIONADO VIRA O PADRÃO DO PINCEL** — o alpha por IMAGEM da
//! escultura, atrás da `feature` `sculpt3d`.
//!
//! ⛔⛔ **Ela nasceu de um TECTO DE LOC, e o corte é por RESPONSABILIDADE:** a
//! `fase_sculpt3d_bake` chegou a `218` linhas contra o tecto de `200` com **dois assuntos** —
//! *assar a forma num sprite* e *transformar um sprite no padrão do pincel* —, que só partilhavam
//! o sítio porque é ali que a cena 3D, o mundo, o renderizador e o mapa de atlas estão os quatro
//! em escopo; e esse facto vale para as duas.
//!
//! ⚠️ **O nome começa por `fase_` e isso é load-bearing:** o texto emendado do quadro
//! ([`crate::frame_text`]) colhe **só** `self.fase_*(` — com outro nome esta fase desaparecia, em
//! silêncio, do oráculo de toda lei de ordem desta shell. ⭐ E ela é declarada e chamada pela
//! IRMÃ (o `splice` é recursivo): o índice do quadro estava a uma linha do tecto dele.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_sculpt3d_alpha(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sculpt3d,
            renderer,
            sim,
            asset_db,
            toasts,
            tools,
            hero_screen,
            atlas_asset_map,
            ..
        } = FrameGfx::of(gfx);

        #[cfg(feature = "sculpt3d")]
        if let Some(scene) = sculpt3d.as_mut() {
            let selected = hero_screen
                .as_ref()
                .and_then(|h| h.gizmo.iter_selected().next());
            // ⭐⭐ **A LEI vive na família** ([`ph2d_app_sculpt3d::alpha_pedido`]) e o que fica
            // aqui são as duas LEITURAS, por fecho — o molde que o `bake::drain` já usava. O que
            // a shell sabe é *como ler um sprite*; o que fazer com os pixels é da escultura.
            if std::mem::replace(&mut self.sculpt3d_req.alpha_request, false) {
                // ── O que o artista VÊ, e não o que o sprite GUARDA. ──
                // ⚠️ **A porta é a MESMA do «Use as Brush Grain»** — o Painter resolveu esta
                // pergunta uma vez (`composite_to_lum`) e uma segunda resposta divergiria dela. Ela
                // é pedida SEM activar o Painter (`tool_by_id_mut`): perguntar o que a tela mostra
                // não pode trocar a ferramenta da mão do artista.
                // ⚠️ E a luminância entra como CINZA OPACO: a lei do `AlphaImage` é
                // `luminância × alfa`, e sobre um cinza opaco ela devolve a própria luminância.
                let mut ler_camadas = || {
                    let bits = selected?;
                    let painter = tools
                        .tool_by_id_mut(&ph2d_editor_core::ToolId::new("painter"))?
                        .as_any_mut()
                        .downcast_mut::<ph2d_tool_painter::PainterTool>()?;
                    if painter.needs_document_bind(bits) {
                        return None;
                    }
                    let (lum, w, h) = painter.composite_to_lum()?;
                    let rgba: Vec<u8> = lum.iter().flat_map(|&l| [l, l, l, 255]).collect();
                    ph2d_sculpt3d::AlphaImage::from_rgba(w, h, &rgba)
                };
                // ── E o que o sprite guarda, quando não há camadas vivas. ──
                // ⚠️ **STRAIGHT, e a conversão é load-bearing:** num buffer PREMULTIPLICADO a
                // luminância já traz o alfa dentro, e o peso sairia com o alfa ao QUADRADO — toda
                // borda macia ficaria mais fina do que o desenho é.
                let mut ler_sprite = || {
                    // PRECISION-READONLY: o padrão LÊ os pixels para deles derivar uma LUMINÂNCIA
                    // e **nunca os escreve de volta** — a sprite fica como estava. ⚠️ Esta
                    // declaração só apareceu quando o corte de 21/09 separou este gesto do de
                    // assar: enquanto os dois partilhavam ficheiro, a entrada nomeada da irmã
                    // abrigava os dois.
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
                };
                let nome = selected.and_then(|bits| {
                    sim.world()
                        .get::<ph2d_ecs::Name>(ph2d_ecs::Entity::from_bits(bits))
                        .map(|n| std::sync::Arc::from(n.0.as_str()))
                });
                let line = ph2d_app_sculpt3d::alpha_pedido::drain(
                    scene,
                    nome,
                    selected.is_some(),
                    &mut ler_camadas,
                    &mut ler_sprite,
                );
                eprintln!("[sculpt3d] {}", line.frase());
                toasts.push(if line.assou() {
                    Toast::success(line.frase())
                } else {
                    Toast::error(line.frase())
                });
            }
        }
    }
}
