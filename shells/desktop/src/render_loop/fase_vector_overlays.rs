//! **Fase do quadro: OS OVERLAYS VECTORIAIS** — as formas vivas do Motion, o overlay do blend, o pick de
//! caminho, o PLANO de overlay do quadro, o contorno de proveniência, a peça do Trim e a face do balde (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_overlays(
        &mut self,
        motion_tool_active: bool,
        vector_active: bool,
        vec_xf: ph2d_vec_scene::VecXforms,
        cam_affine: ph2d_vector::Affine,
    ) -> Option<(
        ph2d_app_vec::overlay::VecOverlayPlan,
        ph2d_vec_scene::VecXforms,
        ph2d_vector::Affine,
    )> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        // ⭐ **A janela da CENA, não a do quadro** — sob um split do centro a cena desenha num
        // sub-rectângulo e a projecção MUDA (ver [`crate::scene_mapping`]). Fora do split é a
        // janela inteira, bit a bit.
        let janela_da_cena = gfx.scene_window();
        let FrameGfx {
            surface,
            renderer,
            camera,
            vector_scene,
            vec_scene,
            hero_screen,
            motion,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // ADR-0154: the live GPU shapes of the Motion scene. Gated on the
        // Motion tool like the Motion sprites (present.rs) — a `source.shape`'s
        // `geometry_id` instances are drawn into the SAME scene the vector
        // document rides in, so they composite behind the chrome and over the
        // sprites (Fase 1: vector over sprite), aligned with the Motion sprites
        // by the same `cam_affine`.
        if motion_tool_active {
            // ⭐⭐⭐ **A ARTE dos quads do passe vectorial** (a terceira média): resolvida
            // aqui porque é aqui que o `renderer` e a GPU estão em mão, e memoizada em
            // [`ph2d_app_motion::motion_leaf_images`] porque cada leitura PARA a GPU.
            let (gpu, atlas, individual) = (surface.gpu(), renderer.atlas(), renderer.individual());
            // ⚠️ **O `synced` é a única porta**, e ele recebe o relógio de mudança do
            // atlas — esquecer a sincronização é erro de compilação (§2.5).
            let mut cache = self.motion_shell.leaf_images.synced(atlas.epoch());
            let mut art = |tex: u32, uv: [f32; 4]| cache.art(gpu, atlas, individual, tex, uv);
            ph2d_app_motion::motion_shape_gen::encode(
                &motion.pump.vector_instances,
                &motion.shape_store,
                &mut art,
                cam_affine,
                vector_scene,
            );
            // ⛔⛔ **LARGAR O ATLAS** (auditoria §2.5): a cópia em CPU dele é `268 MB` e
            // ficava retida pela vida do processo. Os RECORTES ficam — eles são a resposta
            // memoizada e são pequenos.
            self.motion_shell.leaf_images.end_frame();
        }
        // ⭐⭐⭐ **OS GIZMOS DE NÓ, LOGO A SEGUIR À ARTE QUE ELES MANIPULAM** (doc 109 §6 — report do
        // dono, 2026-09-13: *«o collider deve aparecer na frente da shape (z-index maior)»*).
        //
        // ⚠️ **O sítio É a resposta**: no Vello quem pinta depois fica por cima, e estas duas linhas
        // moravam na `fase_selection_highlight`, que corre ANTES desta — o gizmo ficava ATRÁS dos
        // quadrados que ele manipula. Medido no quadro emendado (`arte em 623773, gizmo em 484970`),
        // e é o mesmo defeito que o report de 08/09 do gizmo de warp já tinha dado.
        if let Some(v) = ph2d_app_motion::warp_gizmo::view() {
            let port = ph2d_app_motion::warp_gizmo::param_port(motion, v.node);
            ph2d_app_motion::warp_overlay::draw_warp_gizmo(
                true,
                &v,
                &port,
                camera,
                hero.view.center_split,
                surface.size(),
                vector_scene,
            );
        } else {
            // ⚠️ O outro lado da sonda: sem esta linha, um `PH2D_WARP_DIAG=1` que não imprime nada
            // lê-se como *«a sonda não está a correr»*.
            ph2d_app_motion::warp_overlay::diag("nao ha' retrato publicado (`view()` = None)");
        }
        if let Some(v) = ph2d_app_motion::collider_gizmo::view() {
            ph2d_app_motion::collider_gizmo_overlay::draw(
                &v,
                camera,
                hero.view.center_split,
                surface.size(),
                vector_scene,
            );
        }
        // O **overlay** do Blend Object (ADR-0128): os passos virtuais + as fontes de cima
        // reempilhadas, na ordem de z (a última fonte por cima do último passo). Desenha depois
        // do `dispatch` (que já pôs as fontes no z da cena, embaixo); o overlay reestabelece a
        // pilha do blend por cima. O interleaving fino contra o resto da cena é da Fase C.
        ph2d_vec_render::draw_blend_overlay(&self.vec.blend_overlay, cam_affine, vector_scene);
        // ⛔ **AS SETAS DO MORPH NÃO SE DESENHAM** (Enio, 2026-08-25: *"as setas são virtuais e
        // ninguém jamais vê"*). A W3a pintava-as aqui, em âmbar, entre as formas que ligavam.
        // Elas deixaram de existir como desenho porque deixaram de ser AUTORADAS: o conjunto é
        // o grafo COMPLETO, gerado por um botão — desenhar `n(n-1)` setas entre formas que já
        // estão escondidas é ruído sobre uma resposta que ninguém precisa de ler no canvas.
        // A lista da seção *Morph States* é a superfície, e é lá que a condição se escolhe.
        // **Pick Shapes** (ADR-0128 C2b): realça as formas escolhidas e costura a ORDEM de
        // clique numa polilinha (a prévia do spine). Fora do modo Pick, a lista não vale —
        // limpa, para não vazar escolhas velhas para o próximo blend.
        if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend {
            let preview =
                crate::blend_live::pick_preview(vec_scene, &vec_xf, &self.vec.blend_picks);
            ph2d_vec_render::draw_blend_overlay(&preview, cam_affine, vector_scene);
        } else if !self.vec.blend_picks.is_empty() {
            self.vec.blend_picks.clear();
        }
        // **O Picker de caminho-guia** (Enio 2026-07-23): armado, o caminho sob o cursor é o que
        // o clique vai prender — a silhueta dele acende, o idioma do conta-gotas. `path_at` é o
        // MESMO resolvedor do clique, então o realce nunca mente sobre o que será escolhido. Fora
        // do modo Select (ou sem a tool) o pick não faz sentido: limpa, para não ficar armado e
        // invisível — a saída sem compromisso, como o clique no vazio.
        if let Some(pick) = self.vec.path_pick {
            if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select {
                let w = camera.screen_to_world(self.last_pointer, janela_da_cena);
                let a = camera.screen_to_world((0.0, 0.0), janela_da_cena);
                let b = camera.screen_to_world((1.0, 0.0), janela_da_cena);
                // LITERAL-PX-OK: raio de acerto em px, o MESMO do picking de canvas (`path_at`).
                let hit_r =
                    10.0 * f64::from(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt());
                if let Some(gid) =
                    self.vec
                        .pen
                        .path_at(vec_scene, [f64::from(w[0]), f64::from(w[1])], hit_r)
                    && gid != pick.source()
                    && let Some(outline) = crate::vec_pick::hover_outline(vec_scene, &vec_xf, gid)
                {
                    ph2d_vec_render::draw_blend_overlay(&[outline], cam_affine, vector_scene);
                }
            } else {
                self.vec.path_pick = None;
            }
        }
        // Âncoras/handles/gradiente/marquee só interessam a quem edita nós; no
        // modo Select quem fala é o gizmo (ADR-0112). As guias de snap são caso à
        // parte (valem em TODOS os modos) — `vec_overlay` separa as duas políticas
        // num ponto testável (P1).
        let overlay =
            crate::vec_overlay::vec_overlay_plan(vector_active, self.vec.draw_config.mode);
        // ⭐ **O CONTORNO DE PROVENIÊNCIA** (estudo de UI viva, C2) — a forma que o ponteiro
        // aponta, venha ele do canvas ou de uma linha da Hierarquia.
        //
        // ⚠️ **FORA do `overlay.edit`, e é a metade do desenho:** aquele portão só abre no
        // modo Node, e é justamente no modo **Select** — onde o artista escolhe qual das cinco
        // formas de uma booleana quer — que a pergunta *"qual delas é esta?"* se faz.
        //
        // ⚠️ E **não** quando ela já está selecionada: o gizmo já a nomeia, e um segundo
        // realce por cima do primeiro diz duas vezes a mesma coisa com tintas diferentes.
        //
        // ⚠️ **SEM portão de modo** (Enio, 2026-08-23: *"pode estender isso para todos os
        // objetos mesmo fora do modo vector?"*): o realce segue o CLIQUE, e o clique pega
        // objecto em qualquer modo. Um portão de modo aqui faria a pergunta *"qual destes é
        // este?"* só ter resposta onde ela já era mais fácil.
        if let Some(bits) = self.hovered_object
            && !self.hover_outline.is_empty()
            && !hero.gizmo.iter_selected().any(|s| s == bits)
        {
            ph2d_vec_render::draw_hover_outline(&self.hover_outline, cam_affine, vector_scene);
        }
        // ⭐⭐⭐ **O REALCE DO TRIM** (plano 38): o pedaço que o clique vai apagar, a vermelho,
        // como no Fusion. ⚠️ **A geometria vem da MESMA porta que o corte** — ela é calculada
        // no dreno do ponteiro e guardada, então o que acende neste quadro é literalmente o que
        // o `vec_trim::apply` vai comer. Uma segunda conta aqui seria a divergência mais cara
        // que uma ferramenta destrutiva pode ter.
        if !self.vec.trim_piece.is_empty() {
            ph2d_vec_render::draw_trim_piece(&self.vec.trim_piece, cam_affine, vector_scene);
        }
        // ⭐⭐⭐ **A FACE que o Balde vai preencher** (plano 40), na TINTA que ele vai depositar.
        // ⚠️ A geometria vem da MESMA porta que o preenchimento usa; e a tinta é a corrente,
        // não uma cor neutra — um realce noutra cor prometeria uma coisa e entregaria outra.
        // ⚠️ A tinta é lida do CAMPO (`vec_pen`), e não pelo `bucket_paint()`: um método em
        // `&self` pede o objecto INTEIRO emprestado, e aqui há um empréstimo mútuo vivo. Os
        // campos são disjuntos; a lei do `alpha == 0` é a mesma dos dois lados.
        let tinta_balde = self.vec.pen.style().fill;
        if let Some(face) = self.vec.bucket_face.as_ref()
            && tinta_balde.a != 0
        {
            let t = tinta_balde;
            ph2d_vec_render::draw_bucket_face(
                &face.face,
                [t.r, t.g, t.b, t.a],
                cam_affine,
                vector_scene,
            );
        }
        // ⭐⭐⭐ **ONDE UM CLIQUE DA CANETA PORIA UM PONTO** (report do dono, 2026-09-19: *«não tem
        // indicação visual que você está em cima da linha para criar um ponto»*), ao lado dos dois
        // realces acima e pela mesma razão: ele responde a *«o que este clique faria?»*.
        //
        // ⚠️ **A posição vem da MESMA porta que o clique usa** (`PenTool::previa_de_insercao`, que
        // chama o `insert_hit` e decide o raio com a mesma linha do press). Uma segunda conta aqui
        // acenderia a marca num sítio em que o clique já não insere — o defeito que o realce do Trim
        // e o do Balde nomeiam por escrito, logo acima.
        if let Some(p) = self.vec.previa_insercao {
            ph2d_vec_render::draw_insert_preview(p, cam_affine, vector_scene);
        }
        Some((overlay, vec_xf, cam_affine))
    }
}
