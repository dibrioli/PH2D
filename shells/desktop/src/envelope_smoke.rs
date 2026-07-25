//! As cenas de smoke do **ENVELOPE** (ADR-0129) — módulo irmão do [`crate::build_smoke`], teto de
//! LOC (HR-18).
//!
//! `PH2D_BUILD_SMOKE=11` monta o caso `N=1` (uma elipse, gaiola já puxada num trapézio), `=12` o
//! *warp group* (duas elipses sob UMA gaiola) e **`=27` o caso do Enio: um envelope sobre uma
//! ESTRELA (fonte CÔNCAVA)**. Elas usam só os frames 3 e 4, e é por isso que podem sair do `match`
//! do irmão sem mudar a sequência de ninguém.
//!
//! ⚠️ **Elas não são mais a única porta.** Desde as Fatias 4+5 a seção **Envelope** do painel cria,
//! expande e solta um envelope sem env nenhuma — estas cenas ficam por serem o atalho de sempre
//! (arte já montada, gaiola já deformada), não por serem necessárias.

use ph2d_vec_scene::ShapeKind;

use crate::build_smoke::shape;

/// Despacha o frame `f` da cena de envelope do nível `level` (11 ou 12).
pub(crate) fn frame(app: &mut crate::App, f: u32, level: u32) {
    let self_ = app;
    match (f, level) {
        // A cena do ENVELOPE (ADR-0129, Fatia B): UMA elipse, e a gaiola já vem PUXADA num
        // trapézio de perspectiva forte — a forma nasce deformada, para o Enio ver a correção
        // sem arrastar nada. A prova NÃO é o canto obedecer (o ingênuo também acerta o canto);
        // é a lateral curvar liso ENTRE os cantos.
        (3, 11) => {
            let Some(gfx) = self_.gfx.as_mut() else {
                return;
            };
            let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
            gfx.vec_scene.push_path(shape(
                ShapeKind::Ellipse,
                [-2.6, -1.6],
                [2.6, 1.6],
                &[],
                [90, 140, 200],
            ));
        }
        (4, 11) => {
            // A entidade já nasceu (sync do frame anterior) e a forma já foi assentada. `create`
            // (Fatia 3) é SÍNCRONO: assa a forma em MUNDO, cria um CONTAINER na identidade,
            // reparenta a forma como filho na identidade e pendura o `VecEnvelope`. A pose fica no
            // `Transform` do container — no Select o gizmo a move (Fatia 2).
            let container = {
                let Some(gfx) = self_.gfx.as_mut() else {
                    return;
                };
                let Some(id) = gfx.vec_scene.paths().first().map(|p| p.id) else {
                    return;
                };
                crate::envelope_live::create(
                    &mut gfx.sim,
                    &mut gfx.vec_scene,
                    &self_.vec_entities,
                    &[id],
                )
            };
            let Some(container) = container else { return };
            // Estreita o topo para 35% da base: trapézio convexo forte (perspectiva). BL/BR
            // ficam; TR/TL vêm para o centro-topo. Escrito direto no `VecEnvelope` do container.
            if let Some(gfx) = self_.gfx.as_mut() {
                let e = ph2d_ecs::Entity::from_bits(container);
                if let Some(mut env) = gfx.sim.world_mut().get_mut::<ph2d_ecs::VecEnvelope>(e) {
                    let [bl, br, tr, tl] = env.corners;
                    let cx = (tl[0] + tr[0]) * 0.5;
                    let k = 0.35;
                    env.corners = [
                        bl,
                        br,
                        [cx + (tr[0] - cx) * k, tr[1]],
                        [cx + (tl[0] - cx) * k, tl[1]],
                    ];
                    debug_assert!(
                        ph2d_vec_envelope::QuadWarp::is_convex(&env.corners),
                        "a gaiola do smoke tem de ser convexa (mantém o horizonte fora)"
                    );
                }
            }
            // Fatia 1/3: seleciona o FILHO no pen — a regra seleciona-só-o-container
            // (`vec_selection`) põe o CONTAINER no gizmo, e a gaiola (que lê a seleção do gizmo)
            // aparece no NODE. Sem isto a cena nasceria no Select (default).
            if let Some(id) = self_
                .gfx
                .as_ref()
                .and_then(|g| g.vec_scene.paths().first().map(|p| p.id))
            {
                self_.vec_pen.select_many(&[id]);
            }
            self_.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Node);
            eprintln!(
                "[envelope-smoke] elipse deformada por gaiola de perspectiva (modo NODE). \
                 OLHE O MEIO DOS SEGMENTOS: as laterais curvam LISO — se so os 4 cantos \
                 obedecessem e o meio ficasse reto/quebrado, seria o bug ingenuo. \
                 NODE: arraste os CANTOS da gaiola (Fatia 1) -- re-deforma ao vivo, convexo \
                 obrigatorio. SELECT (pill do painel): o GIZMO move/gira/escala o envelope \
                 INTEIRO (Fatia 3 = container) -- a forma deformada anda junta, sem dobrar."
            );
        }
        // A cena do WARP GROUP (ADR-0129 Fatia 3): DUAS elipses sob UMA gaiola. É o que separa
        // um container de dois envelopes soltos — uma gaiola só deforma as duas, e o gizmo do
        // Select move as duas juntas, sem cisalhar.
        (3, 12) => {
            let Some(gfx) = self_.gfx.as_mut() else {
                return;
            };
            let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
            gfx.vec_scene.push_path(shape(
                ShapeKind::Ellipse,
                [-4.0, -1.5],
                [-1.0, 1.5],
                &[],
                [200, 120, 90],
            ));
            gfx.vec_scene.push_path(shape(
                ShapeKind::Ellipse,
                [1.0, -1.5],
                [4.0, 1.5],
                &[],
                [90, 180, 140],
            ));
        }
        (4, 12) => {
            // Cria UM envelope sobre AS DUAS formas → container com 2 filhos (warp group).
            let container = {
                let Some(gfx) = self_.gfx.as_mut() else {
                    return;
                };
                let ids: Vec<_> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
                crate::envelope_live::create(
                    &mut gfx.sim,
                    &mut gfx.vec_scene,
                    &self_.vec_entities,
                    &ids,
                )
            };
            let Some(container) = container else { return };
            // Puxa o topo (40% da base) — uma gaiola só, as duas elipses a seguem.
            if let Some(gfx) = self_.gfx.as_mut() {
                let e = ph2d_ecs::Entity::from_bits(container);
                if let Some(mut env) = gfx.sim.world_mut().get_mut::<ph2d_ecs::VecEnvelope>(e) {
                    let [bl, br, tr, tl] = env.corners;
                    let cx = (tl[0] + tr[0]) * 0.5;
                    let k = 0.4;
                    env.corners = [
                        bl,
                        br,
                        [cx + (tr[0] - cx) * k, tr[1]],
                        [cx + (tl[0] - cx) * k, tl[1]],
                    ];
                    debug_assert!(
                        ph2d_vec_envelope::QuadWarp::is_convex(&env.corners),
                        "a gaiola do smoke tem de ser convexa"
                    );
                }
            }
            if let Some(id) = self_
                .gfx
                .as_ref()
                .and_then(|g| g.vec_scene.paths().first().map(|p| p.id))
            {
                self_.vec_pen.select_many(&[id]);
            }
            self_.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Node);
            eprintln!(
                "[envelope-smoke 12] DUAS elipses sob UMA gaiola (warp group, modo NODE). \
                 As duas curvam pela MESMA perspectiva. NODE: arraste um CANTO da gaiola -- \
                 as duas re-deformam juntas. SELECT (pill do painel): o GIZMO abraça as DUAS \
                 e move/gira/escala o grupo inteiro -- sem cisalhar (a caixa e' a uniao)."
            );
        }
        // A cena do ENVELOPE sobre uma ESTRELA (o caso reportado pelo Enio, 2026-07-24): a fonte é
        // CÔNCAVA (uma estrela de 5 pontas), não uma elipse convexa. A gaiola envolve a BBOX (um
        // retângulo, igual à elipse), então a correção prova que a concavidade da FONTE não muda a
        // gaiola — a estrela deformada segue liso, e as 4 alças de canto ficam LIMPAS (sem alça
        // solta nem linha à deriva, que foi o sintoma da foto — vindo de um build velho da pasta
        // primária, não desta linha).
        (3, 27) => {
            let Some(gfx) = self_.gfx.as_mut() else {
                return;
            };
            let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
            // A estrela EXATA do smoke do Contour (=25): 5 pontas, razão interna 0.45.
            gfx.vec_scene.push_path(shape(
                ShapeKind::Star,
                [-2.4, -1.8],
                [2.4, 1.8],
                &[5.0, 0.45, 0.0],
                [235, 175, 60],
            ));
        }
        // ⚠️ ORDEM DO PRODUTO, de propósito (Enio 2026-07-24): o artista SELECIONA a forma (frame 4)
        // e SÓ ENTÃO clica Envelope (frame 5). Nos DOIS frames separados — não num só — é que o bug
        // aparecia: enveloparr re-parenteia sem tocar o pen, então o `sync_selection` não rerodava a
        // promoção filho→container e a gaiola nunca acendia. A versão antiga do smoke criava e SÓ
        // ENTÃO selecionava (tudo num frame), o que MASCARAVA o bug (o pen mudava e a promoção
        // rerodava por acidente). Gate: `enveloping_a_selected_shape_promotes_the_gizmo_to_the_container`.
        (4, 27) => {
            // O artista seleciona a estrela e entra no Node — o gizmo pousa NA FORMA (ainda sem
            // envelope). O `sync_selection` deste frame promove o pen ao gizmo.
            if let Some(id) = self_
                .gfx
                .as_ref()
                .and_then(|g| g.vec_scene.paths().first().map(|p| p.id))
            {
                self_.vec_pen.select_many(&[id]);
            }
            self_.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Node);
        }
        (5, 27) => {
            let container = {
                let Some(gfx) = self_.gfx.as_mut() else {
                    return;
                };
                let Some(id) = gfx.vec_scene.paths().first().map(|p| p.id) else {
                    return;
                };
                crate::envelope_live::create(
                    &mut gfx.sim,
                    &mut gfx.vec_scene,
                    &self_.vec_entities,
                    &[id],
                )
            };
            let Some(container) = container else { return };
            // O MESMO que o render_loop faz ao clicar Envelope: invalida a memória do sync para a
            // promoção filho→container rerodar (o pen não mudou — enveloparr só re-parenteou).
            self_.vec_sel.invalidate();
            // Puxa o topo a 40% da base — o MESMO trapézio de perspectiva do `=11`, para a estrela
            // nascer deformada e o olho julgar sem arrastar. BL/BR ficam; TR/TL vêm ao centro-topo.
            if let Some(gfx) = self_.gfx.as_mut() {
                let e = ph2d_ecs::Entity::from_bits(container);
                if let Some(mut env) = gfx.sim.world_mut().get_mut::<ph2d_ecs::VecEnvelope>(e) {
                    let [bl, br, tr, tl] = env.corners;
                    let cx = (tl[0] + tr[0]) * 0.5;
                    let k = 0.4;
                    env.corners = [
                        bl,
                        br,
                        [cx + (tr[0] - cx) * k, tr[1]],
                        [cx + (tl[0] - cx) * k, tl[1]],
                    ];
                    debug_assert!(
                        ph2d_vec_envelope::QuadWarp::is_convex(&env.corners),
                        "a gaiola do smoke tem de ser convexa"
                    );
                }
            }
            eprintln!(
                "[envelope-smoke 27] ESTRELA (fonte concava) envelopada na ORDEM DO PRODUTO \
                 (selecionou no frame 4, clicou Envelope no frame 5). A GAIOLA TEM DE APARECER na \
                 hora: 4 cantos com circulos + moldura trapezoidal (perspectiva), e a estrela \
                 deformada LISA. NAO deve haver alcas de no soltas sobre a estrela. Este era o bug: \
                 selecionar-e-depois-envelopar deixava o gizmo no FILHO e a gaiola nao acendia. \
                 Arraste um canto: re-deforma ao vivo, convexo obrigatorio."
            );
        }
        _ => {}
    }
}
