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

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_bands(
        &mut self,
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
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
        let vec_fx = self.fx_live.images();
        let vec_patterns = self.texture_pattern_live.tiles();
        // As PELES de widget deste frame (plano UI/UX W6.2). Cozidas AQUI, depois do `sync`
        // (senão uma forma recém-marcada ainda não tem entidade) e com a câmera na mão —
        // quem sabe onde a forma está na tela é quem tem a projeção.
        let vec_skins = crate::widget_live::build(
            vec_scene,
            sim,
            &self.vec.entities,
            &vec_xf,
            &vec_live,
            cam_affine,
            paint_ctx.text,
            hero.theme,
        );
        // ⭐⭐⭐ **AS FAIXAS DO DOCUMENTO** (ADR-0154 Fase 2). Quando a cena INTERCALA vetor e
        // sprite, o documento deixa de ir para a cena do chrome e passa a ser codificado uma
        // vez por faixa, em cenas próprias — o presente desenha-as intercaladas com as faixas
        // de sprite, e o chrome fica por cima de tudo, como sempre.
        //
        // ⭐ **A faixa exprime-se ESCONDENDO o resto**, e não filtrando o laço do `dispatch`.
        // A razão está escrita lá dentro: o `push`/`pop` da camada de recorte vive **fora** do
        // filtro de escondido, de propósito, para as molduras se emparelharem mesmo quando não
        // desenham. ⇒ uma forma fora da faixa não desenha **e** a moldura dela continua a
        // recortar quem cai lá dentro. Filtrar o laço desemparelharia a pilha.
        //
        // ⚠️ Sem intercalação isto fica vazio e o documento vai para a cena do chrome, byte a
        // byte como sempre.
        // ⭐⭐⭐ **A arte dos PINCÉIS deste quadro, MEMOIZADA** (`line/Vector`, 2026-08-30) —
        // resolvida aqui pela mesma razão que o ladrilho do padrão o é: a crate de desenho não
        // alcança a cena, e o guarda de ciclo tem de viver onde se pode medir.
        //
        // ⚠️ **Resolvida UMA vez, FORA do laço das faixas** (integração de 2026-09-04): a
        // `line/Vector` memoizou-a porque sem memo `50` pincéis com grupos de `16` custam
        // **14,28 ms — 85,5% de um quadro**; a `line/components` pôs o `dispatch` dentro de um
        // laço por faixa. As duas juntas pagariam a montagem da chave uma vez POR FAIXA, e o
        // mapa é o mesmo para todas — ele é função da cena, não da faixa.
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
        // ⭐⭐⭐ **HÁ RECEITA ABERTA NESTE QUADRO?** — o interruptor do vidro jateado, escrito
        // UMA vez e lido pelo presente (re-derivá-lo lá seria a segunda resposta, e um quadro
        // em que as duas discordassem desenharia a receita duas vezes ou nenhuma).
        //
        // ⚠️⚠️ **A pergunta é ao MUNDO e não à vista do vetor:** o `isolated` dela é enchido a
        // partir das FORMAS marcadas, e uma receita feita só de imagens deixa-o vazio — o vidro
        // nunca subiria justamente para os prefabs de sprite.
        *frosting = ph2d_app_components::master_editing::any_open(sim);
        frost_doc_scene.reset();
        frost_front_scene.reset();
        let doc_bands = crate::draw_bands::doc_bands_of(frame_order);
        for band in &doc_bands {
            let keep = frame_order.vector_ids_in(*band);
            let mut band_view = vec_view.clone();
            for path in vec_scene.paths() {
                if !keep.contains(&path.id) {
                    band_view.hidden.push(path.id);
                }
            }
            let mut target = ph2d_vector::VectorScene::new();
            ph2d_vec_render::dispatch(
                vec_scene,
                &band_view,
                &vec_xf,
                &vec_live,
                vec_fx,
                &vec_skins,
                vec_patterns,
                brush_arts,
                self.paint_dilate_live.out(),
                cam_affine,
                &mut target,
            );
            band_doc_scenes.push(target);
        }
        if !doc_bands.is_empty() {
            // O documento já foi codificado nas faixas — a cena do chrome fica só com o chrome.
        } else {
            // ⭐⭐⭐ **COM O VIDRO, o documento sai da cena do CHROME.** Ele tem de aterrar no
            // acumulador do mundo **antes** do borrão, e os painéis entram depois dele; na
            // mesma cena, os dois seriam borrados ou nítidos juntos. ⛔ Sem receita aberta é a
            // cena de sempre, byte a byte.
            let target: &mut ph2d_vector::VectorScene = if *frosting {
                frost_doc_scene
            } else {
                vector_scene
            };
            ph2d_vec_render::dispatch(
                vec_scene,
                &vec_view,
                &vec_xf,
                &vec_live,
                vec_fx,
                &vec_skins,
                vec_patterns,
                brush_arts,
                self.paint_dilate_live.out(),
                cam_affine,
                target,
            );
        }
        // ⭐⭐⭐ **E A RECEITA, sozinha, na cena que fica ACIMA do vidro.** ⚠️ Ela é codificada
        // aqui **em todos os casos** — com faixas ou sem elas —, porque o `dispatch` e as
        // faixas saltam-na sempre: sem esta chamada a receita aberta simplesmente não desenha.
        // ⛔ Sem receita aberta a porta devolve sem escrever nada.
        ph2d_vec_render::dispatch_isolated(
            vec_scene,
            &vec_view,
            &vec_xf,
            &vec_live,
            vec_fx,
            &vec_skins,
            vec_patterns,
            brush_arts,
            self.paint_dilate_live.out(),
            cam_affine,
            frost_front_scene,
        );
        // ⚠️ **O mapa que foi DESENHADO fica guardado, e é ele que o PICK lê.**
        //
        // O `vec_gizmo_pick` declara no próprio doc que a pergunta *"o que está desenhado
        // aqui?"* é feita ao MESMO mapa que este `dispatch` consome — e a fiação contradizia-o:
        // os seis sítios de pick da `input_dispatch` passavam só o `offset_live`, então tudo o
        // que os outros oito produtores desenham era **visível e não-clicável** (medido: numa
        // simetria armada, 3 de 3 pontos da metade espelhada estão na tela e o clique
        // atravessa).
        //
        // ⚠️ É um **MOVE**, não um clone: `vec_live` morre aqui, e a fusão é remontada do zero
        // no frame seguinte. Guardar custa zero; re-derivar no input custaria a segunda porta.
        //
        // ⚠️ E é isto que faz um produtor NOVO nascer coberto: quem acrescenta uma linha à
        // fusão acima ganha o pick de graça, sem saber que este parágrafo existe.
        self.vec.live_drawn = vec_live;
        Some((vec_view, vec_xf, cam_affine))
    }
}
