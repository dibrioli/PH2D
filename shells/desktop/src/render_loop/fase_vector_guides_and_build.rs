//! **Fase do quadro: CORTES, GUIAS E CONSTRUÇÃO** — as linhas de corte vectoriais, as guias de encaixe e a sessão de
//! construção de forma, desenhadas sobre a câmera do quadro (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_guides_and_build(
        &mut self,
        tool_preview_bits: [Option<u64>; 3],
        window_size: ph2d_host::WindowSize,
        vec_xf: ph2d_vec_scene::VecXforms,
        cam_affine: ph2d_vector::Affine,
        overlay: ph2d_app_vec::overlay::VecOverlayPlan,
        viewport: EditorRect,
    ) -> Option<ph2d_vector::Affine> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            doc_guides,
            sim,
            theme,
            vector_scene,
            vec_scene,
            text_system,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
        // **A LINHA DE CORTE** (W4) — hachurada, com a tesoura na ponta. Ela é a única
        // geometria da cena que o render de ARTE não desenha (perde fill e stroke ao ser
        // adotada como lâmina), e é aqui que ela reaparece.
        //
        // ⚠️ **FORA do `overlay.edit`, e a razão é um defeito reportado** (Enio, 2026-07-31:
        // *"quando movido com Select a aparência rachurada deve permanecer"*): aquele guard é
        // FALSO no Select, então a lâmina — que o render de arte não desenha — **desaparecia
        // por completo** justamente no modo em que se a move. A hachura não é feedback de um
        // MODO: ela é a aparência do objeto, e um objeto não muda de aparência porque o
        // artista pegou outra ferramenta.
        for id in crate::vec_cut_line::cut_lines(sim, &self.vec.entities) {
            if let Some(p) = vec_scene.paths().iter().find(|p| p.id == id) {
                ph2d_vec_render::draw_cut_line(
                    &ph2d_vec_render::build_bezpath(p),
                    ph2d_vec_render::path_to_screen(&vec_xf, id, cam_affine),
                    vector_scene,
                );
            }
        }
        // Smart guides do snap: FORA do guard de modo — explicam o encaixe vivo em
        // qualquer modo, inclusive o gizmo-move do Select (P1, ADR-0112).
        if overlay.snap_guides {
            // As guias do DOCUMENTO primeiro: elas são o fundo permanente contra o qual a
            // guia de snap (viva, tracejada, de um frame) se lê. Desenhá-las por cima
            // esconderia a marca do encaixe atrás da linha que ele acabou de encontrar.
            //
            // ⚠️ O recorte é a JANELA DA CENA, não o retângulo do canvas: o `cam_affine`
            // projeta nas dims da cena, e o chrome dos painéis é pintado DEPOIS, por cima.
            // Pedir o canvas aqui obrigaria a shell a espelhar a aritmética de layout que
            // o `paint_hero_screen` já faz — a segunda porta exata que a régua evita.
            let (sw, sh) =
                ph2d_app_motion::field_gizmo::scene_window_wh(hero.view.center_split, window_size);
            ph2d_vec_render::draw_document_guides(
                &doc_guides.iter().copied().collect::<Vec<_>>(),
                [0.0, 0.0, f64::from(sw), f64::from(sh)],
                cam_affine,
                vector_scene,
            );
            ph2d_vec_render::draw_snap_guides(&self.vec.snap_guides, cam_affine, vector_scene);
            // **O NÚMERO da guia** — depois do traço dela, porque a cena tem de estar
            // livre para o renderizador de texto (a mesma ordem do readout de joint).
            // O zoom sai do MESMO afim que acabou de desenhar o segmento: a precisão
            // que a ficha mostra é a que aquele desenho de fato resolve.
            let c = cam_affine.as_coeffs();
            let px_per_world = (c[0] * c[0] + c[1] * c[1]).sqrt();
            super::render_loop::vec_snap_labels::draw(
                &self.vec.snap_guides,
                cam_affine,
                px_per_world,
                ph2d_editor_core::LengthDisplay::of(&hero.project),
                hero.theme,
                paint_ctx.text,
                vector_scene,
            );
        }
        // **A DECORAÇÃO DA FOLHA** (Enio 2026-08-19) — a faixa hachurada e o nome. Fica FORA
        // do bloco das guias de propósito: aquele é gateado pelo toggle de snap, e a folha
        // tem de se ler como folha esteja o encaixe ligado ou não.
        {
            let c = cam_affine.as_coeffs();
            let px_per_world = (c[0] * c[0] + c[1] * c[1]).sqrt();
            super::render_loop::sheet_overlay::draw(
                sim,
                cam_affine,
                px_per_world,
                hero.theme,
                paint_ctx.text,
                vector_scene,
            );
            // **AS LINHAS DA GRELHA** sobre a folha aberta de uma sprite (Enio, 2026-08-23).
            // ⚠️ Vizinha da decoração da folha-OBJETO de propósito: as duas dizem *«isto está
            // cortado assim»*, uma para a folha empacotada e outra para a grelha de um sprite,
            // e ler as duas seguidas é o que impede a próxima de nascer num terceiro sítio.
            if let Some(e) = super::render_loop::sim_extract_sheet::previewed(hero) {
                super::render_loop::sheet_grid_overlay::draw(
                    sim,
                    e,
                    // ⚠️ **As linhas seguem o quad que foi de facto DESENHADO.** Sob pintura o
                    // quad desdobra-se e centra-se no pivô; fora dela ele é uma célula e a
                    // folha dispõe-se à volta dela. Desenhar sempre a segunda disposição
                    // deslocava as linhas **meia célula** sobre a arte pintada (report do
                    // Enio, 2026-08-23, com foto).
                    super::render_loop::sim_extract_sheet::is_tool_previewed(&tool_preview_bits, e),
                    hero.project.pixels_per_meter,
                    cam_affine,
                    px_per_world,
                    hero.theme,
                    vector_scene,
                );
            }
            // **A LEGENDA DA CENA DE SMOKE** (Enio 2026-08-23: *"melhore as explicações do
            // smoke"*) — o rótulo pousa em cima do caso que ele explica. No-op quando
            // nenhuma cena publicou, que é todo arranque normal do editor.
            super::render_loop::demo_legend::draw(
                &ph2d_app_motion::motion_demo_legend::captions(),
                cam_affine,
                hero.theme,
                paint_ctx.text,
                vector_scene,
            );
        }
        // **As linhas da SIMETRIA** — *"quando ligada linhas aparecem no canvas"* (Enio).
        //
        // ⚠️ FORA do `overlay.snap_guides` de propósito: aquele toggle é do encaixe, e a
        // simetria não é um encaixe — é o eixo do que está a ser desenhado. Escondê-la com o
        // snap deixaria o artista com as cópias na tela e sem o eixo que as produz.
        {
            // Duas linhas, dois FATOS. A de SESSÃO diz *onde o próximo desenho vai espelhar* e
            // fica onde foi semeada; a de cada forma diz *onde AQUELA forma espelha* e viaja
            // com ela. Coincidem até o artista mover o desenho — e é exatamente aí que ver as
            // duas passa a valer, porque a promessa de que a linha acompanha o objeto só é
            // legível contra a que não acompanha.
            let mut axes = if self.vec.draw_config.symmetry.on {
                crate::symmetry_live::live_axes(vec_scene, sim, &self.vec.entities, &vec_xf)
            } else {
                Vec::new()
            };
            if let Some(origin) = self.vec.symmetry_origin {
                axes.push(crate::symmetry_live::session_axis(
                    self.vec.draw_config.symmetry,
                    origin,
                ));
            }
            if !axes.is_empty() {
                let (sw, sh) = ph2d_app_motion::field_gizmo::scene_window_wh(
                    hero.view.center_split,
                    window_size,
                );
                ph2d_vec_render::draw_symmetry_axes(
                    &axes,
                    [0.0, 0.0, f64::from(sw), f64::from(sh)],
                    cam_affine,
                    vector_scene,
                );
            }
        }
        // **O realce do Shape Builder** — as faces sob o cursor e as já pintadas. Fora do
        // `overlay.edit` porque o Build não é um modo de edição de nó: o que ele
        // manipula é a REGIÃO, não a âncora.
        if let Some(b) = self.vec.build.as_mut() {
            let marked: Vec<ph2d_vec_scene::VecPath> = b
                .marked
                .clone()
                .into_iter()
                .filter_map(|f| b.arr.face_path(f).cloned())
                .collect();
            let hover = b.hover.and_then(|f| b.arr.face_path(f).cloned());
            // As faces E as silhuetas já estão em MUNDO (a sessão as assou), então só a
            // câmera. As silhuetas são redesenhadas por cima do véu: uma forma coberta
            // por outra não aparece na tela, e sem elas o realce paira sobre nada.
            ph2d_vec_render::draw_build_faces(
                b.arr.sources(),
                hover.as_ref(),
                &marked,
                b.subtract,
                cam_affine,
                hero.theme,
                vector_scene,
            );
        }
        Some(cam_affine)
    }
}
