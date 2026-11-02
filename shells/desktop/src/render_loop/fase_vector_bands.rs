//! **Fase do quadro: AS FAIXAS DO DOCUMENTO** (ADR-0154 Fase 2) — o despacho da cena vectorial para o Vello
//! por FAIXA de z, com as imagens de FX, os ladrilhos de padrão, as peles e as artes de pincel, e o vidro
//! jateado da receita aberta (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **As faixas numa fase só, e o motivo é o borrow checker:** `vec_fx`, `vec_patterns` e
//! `brush_arts` são `&BTreeMap` emprestados de `self.fx_live`/`self.texture_pattern_live`/`self.brush_live`
//! e lidos pelos laços das faixas — um empréstimo de `self` não atravessa uma chamada `&mut self`. Por isso a
//! fase-filha do painel autorado (`fase_authored_panel_rows`) corre no INÍCIO, antes desses empréstimos.

use super::*;

/// O painel autorado (a moldura, o retorno do picker e as rows vivas) — fase-filha, num ficheiro irmão (por
/// `#[path]`, para o `render_loop/mod.rs` não crescer acima do tecto dele).
#[path = "fase_authored_panel_rows.rs"]
mod authored_panel_rows;

/// O documento do quadro e os seus três destinos (faixas, inteiro, receita isolada).
#[path = "fase_vector_bands_doc.rs"]
mod doc;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_bands(
        &mut self,
        motion_tool_active: bool,
        vec_view: ph2d_vec_scene::VecViewState,
        vec_xf: ph2d_vec_scene::VecXforms,
        cam_affine: ph2d_vector::Affine,
        vec_live: ph2d_vec_render::LiveGeometry,
        viewport: EditorRect,
    ) -> Option<(
        ph2d_vec_scene::VecViewState,
        ph2d_vec_scene::VecXforms,
        ph2d_vector::Affine,
    )> {
        // O painel autorado segue o documento — ver a fase-filha (antes dos empréstimos das faixas).
        self.fase_authored_panel_rows()?;
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            theme,
            vector_scene,
            vec_scene,
            text_system,
            hero_screen,
            frame_order,
            band_doc_scenes,
            frost_doc_scene,
            frost_front_scene,
            frosting,
            motion,
            luz2d,
            surface,
            flip,
            miniatura,
            ..
        } = FrameGfx::of(gfx);
        let paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
        let vec_fx = self.fx_live.images();
        let vec_patterns = self.texture_pattern_live.tiles();
        // As PELES de widget deste frame (W6.2), depois do `sync` (uma forma recém-marcada já
        // tem entidade) e com a câmera na mão: quem sabe onde a forma está é quem tem a projeção.
        let vec_skins = crate::widget_live::build(
            vec_scene,
            sim,
            &self.vec.entities,
            &vec_xf,
            &vec_live,
            &vec_view,
            cam_affine,
            paint_ctx.text,
            // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
            hero_screen.as_ref()?.theme,
        );
        // ⭐ AS FAIXAS DO DOCUMENTO (ADR-0154 Fase 2): com intercalação o documento vai, por faixa, a cenas próprias;
        // a faixa exprime-se ESCONDENDO o resto (o push/pop do recorte vive fora do filtro). Sem intercalação: o chrome.
        // A arte dos PINCÉIS, memoizada e resolvida UMA vez fora do laço das faixas (14,28 ms sem memo, 04/09).
        let brush_arts = self.brush_live.resolve(
            vec_scene,
            &|id| {
                ph2d_vec_entities::entities::object_selection_for(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    id,
                )
            },
            &vec_xf,
        );
        band_doc_scenes.clear();
        // Há receita aberta? (o interruptor do vidro, escrito UMA vez; pergunta-se ao MUNDO: receita só de imagens.)
        *frosting = ph2d_app_components::master_editing::any_open(sim);
        frost_doc_scene.reset();
        frost_front_scene.reset();
        // ⭐⭐⭐ **AS FORMAS DO MOTION NA PLACA?** (doc 121 W2/W3, B5) — decidido antes de o documento
        // ser codificado (com elas na placa ele vai às FAIXAS); no DISPOSITIVO o buffer é o dele.
        let por_ordem = ph2d_app_motion::motion_shape_placa::por_ordem();
        let ligadas_do_motion = (
            motion_tool_active && por_ordem,
            motion_tool_active && !*frosting && por_ordem,
        );
        let placa = ph2d_app_motion::motion_shape_placa::decide_o_quadro(
            &mut self.motion_shell.placa,
            surface.gpu(),
            motion,
            ligadas_do_motion,
            cam_affine,
        );
        // ⭐ E a placa de cada palco de uma macro em cena (ADR-0186), pela MESMA porta.
        let ligadas = (por_ordem, !*frosting && por_ordem);
        let palcos = self
            .motion_shell
            .palcos
            .decide(surface.gpu(), ligadas, cam_affine);
        // ⭐ A LUZ 2D (W3): acesa, o documento vai às FAIXAS (a colagem acende-o); sem `Luz`, `false`.
        let docs = (&mut *vec_scene, &vec_xf, &mut *flip);
        let luz_acesa = luz2d.codificar(sim.world_mut(), surface, cam_affine, docs);
        // GEF 19 E3: com a placa, cada objecto Motion fica no rank dele; sem ela, por cima de tudo.
        frame_order.motion_na_placa =
            ph2d_app_motion::motion_shape_placa::fica_no_rank(placa, motion);
        let palcos_da_ordem = &self.motion_shell.palcos;
        frame_order.adormece_palcos(|m| palcos_da_ordem.toca_o_de(m));
        let doc_bands =
            ph2d_render::draw_bands::doc_bands_of(frame_order, placa || palcos || luz_acesa);
        // O documento por faixa, com a câmera e as peles de QUEM o vê (o editor e a vista da câmera, A6);
        // a faixa Pixel leva a arte dos Pixel Sprites dela (GEF 19 E2) — fora do palco do jogo, como antes.
        let hero = hero_screen.as_ref()?;
        ph2d_app_pixel::draw::begin_frame();
        let doc = doc::DocDasFaixas {
            scene: vec_scene,
            view: &vec_view,
            xf: &vec_xf,
            live: &vec_live,
            fx: vec_fx,
            patterns: vec_patterns,
            brushes: brush_arts,
            dilated: self.paint_dilate_live.out(),
            order: frame_order,
            world: sim.world(),
            ppm: hero.project.pixels_per_meter,
            editing: ph2d_app_pixel::mode::editing(sim.world(), hero),
            sem_pixel: hero.view.palco_do_jogo,
        };
        let pixel = &mut self.pixel;
        doc.faixas(
            pixel,
            &doc_bands,
            &vec_skins,
            cam_affine,
            band_doc_scenes,
            true,
        );
        // Com faixas o documento já foi codificado nelas — a cena do chrome fica só com o chrome.
        if doc_bands.is_empty() {
            // Com o vidro o documento sai do chrome (aterra no mundo antes do borrão); sem receita, a cena de sempre.
            let target: &mut ph2d_vector::VectorScene = if *frosting {
                frost_doc_scene
            } else {
                vector_scene
            };
            doc.inteiro(&vec_skins, cam_affine, target);
        }
        // A receita aberta, sozinha, na cena ACIMA do vidro — em todos os casos.
        doc.isolada(&vec_skins, cam_affine, frost_front_scene);
        // ⭐ A VISTA DA CÂMERA (A6): o mesmo documento, a luz e as placas do Motion com a câmera dela, nas cenas dela —
        // sempre em faixas (o mundo dela acaba no acumulador). A receita aberta vai por cima, na última faixa.
        let t0 = std::time::Instant::now();
        if let Some(v) = miniatura.pedida() {
            let (cam, janela) = (v.afim(), v.janela());
            // O mesmo nível da luz que o editor escolheu (o «Automático» governa cada vista por si).
            v.luz2d.preferir(hero_screen.as_ref()?.view.luz2d);
            let gpu = surface.gpu();
            let alvo = (janela.width, janela.height);
            v.luz2d
                .codificar_no_alvo(doc.world, gpu, alvo, cam, (doc.scene, &vec_xf));
            ph2d_app_motion::motion_shape_placa::decide_o_quadro(
                &mut v.placa,
                gpu,
                motion,
                ligadas_do_motion,
                cam,
            );
            let palcos = &mut self.motion_shell.palcos;
            palcos.troca_a_vista(&mut v.palcos);
            palcos.decide(gpu, ligadas, cam);
            palcos.troca_a_vista(&mut v.palcos);
            let skins = crate::widget_live::build(
                vec_scene,
                sim,
                &self.vec.entities,
                &vec_xf,
                &vec_live,
                &vec_view,
                cam,
                paint_ctx.text,
                hero_screen.as_ref()?.theme,
            );
            v.band_doc_scenes.clear();
            let bands = ph2d_render::draw_bands::doc_bands_of(frame_order, true);
            doc.faixas(pixel, &bands, &skins, cam, &mut v.band_doc_scenes, false);
            if let Some(topo) = v.band_doc_scenes.last_mut() {
                doc.isolada(&skins, cam, topo);
            }
        }
        miniatura.cpu_codifica = t0.elapsed().as_secs_f64() * 1000.0;
        // ⚠️ O mapa DESENHADO fica guardado (um MOVE): é ele que o PICK lê — produtor novo nasce clicável.
        self.vec.live_drawn = vec_live;
        Some((vec_view, vec_xf, cam_affine))
    }
}
