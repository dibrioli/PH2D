//! **Fase do quadro: A SUPRESSÃO DO GIZMO E A MOLDURA 3D** — o gizmo da sprite suprimido sob o Deform Transform e
//! o Flip, os mapas de alças do quadro limpos, e a moldura/geração do modelador 3D (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_gizmo_suppression_and_field3d_frame(&mut self, viewport: EditorRect) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
            tools,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // Onda 2C: clear the gizmo hit_map BEFORE paint_hero_screen
        // runs. `paint_hero_screen` now paints BOTH the primary gizmo
        // AND the multi-selection extras + global gizmo (the latter
        // via `paint_sprite_gizmo_keyed`, which populates `hit_map` —
        // those entries drive the dispatcher's group-transform routing
        // in `on_mouse_input`). Painting them inside `paint_hero_screen`
        // (before the floating panels) keeps gizmos BELOW the panels
        // both visually and in hit-test (z-order fix 2026-05-31).
        // ADR-0076: the vertex-edit / authoring vector tools (Direct / Pen /
        // Pencil / Shape) must NOT show the object-transform gizmo. Its painted
        // box + handles overlay the shape, and those tools' `vector_*_world`
        // reject any click over a `hit_index` widget — so the gizmo's hit-rects
        // would block EVERY vertex/handle grab + canvas click (Enio: "Direct
        // não move pontos/handles"). The gizmo belongs to object selection
        // (Select) + the arrow/Move tools. Suppress the painted view here; the
        // selection stays armed (hierarchy highlight) — just no box/handles.
        // The Deform Transform temperament shows its OWN whole-region gizmo (drawn in the painter
        // overlays). The object-transform gizmo would sit ON TOP (paint_hero_screen draws after those
        // overlays) and fight it, so suppress it while Deform Transform is active — the deform box IS the
        // transform gizmo there (Enio 2026-07-04).
        let painter_deform_transform = tools
            .active_mut()
            .and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
            })
            .is_some_and(|p| p.deform_gizmo().is_some());
        // A correção de pares do Flip é o MESMO caso das tools de vetor acima: o overlay de
        // Pairs quer o clique do canvas para re-parear, e a caixa+alças do gizmo do objeto
        // registram hits no `hit_index` que fazem `on_canvas` virar falso — roubando TODO
        // clique de re-par. Enquanto Pairs está aberto, o gizmo do objeto some (a seleção
        // fica armada; só a caixa/alças somem).
        let flip_pairs_active =
            self.flip_state.active && self.flip_state.strip.tween_correct.is_some();
        let suppress_gizmo = painter_deform_transform
            || flip_pairs_active
            || tools
                .active()
                .map(|t| {
                    let id = t.id();
                    id == ph2d_editor_core::ToolId::new("vector_direct")
                            || id == ph2d_editor_core::ToolId::new("vector_pen")
                            || id == ph2d_editor_core::ToolId::new("vector_pencil")
                            || id == ph2d_editor_core::ToolId::new("vector_shape")
                            // Motion Nodes: a tool Motion é dona do canvas (o único gizmo é o
                            // do field, slot próprio `field_view`). Um sprite selecionado por
                            // acaso ao entrar mostraria seu gizmo de sprite projetado na
                            // janela CHEIA (deslocado da cena que renderiza na banda do split)
                            // — some junto com o resto do chrome de sprite.
                            || id == ph2d_editor_core::ToolId::new("motion")
                })
                .unwrap_or(false);
        if suppress_gizmo {
            hero.gizmo.view = None;
            hero.gizmo.extra_views.clear();
            hero.gizmo.global_view = None;
        }
        hero.gizmo.gizmo_hit_map.clear();
        // Its sibling for the anchor dots — same reason (no stale entry
        // from a joint that left the scene), same frame.
        hero.gizmo.point_hit_map.clear();
        // ADR-0161 — o smoke do módulo de modelagem 3D (`PH2D_FIELD_SMOKE=1..3`).
        //
        // ⚠️ **ANTES do `paint_hero_screen`, e é a correção de um smoke do Enio (19/08):**
        // *"o fundo está cinza escuro e ACIMA do canvas"*. Depois da chrome, o traçado é uma
        // sobreposição que tapa o app; antes dela, é **conteúdo de canvas**, com painéis e
        // chrome por cima — que é o que um visualizador 3D tem de ser.
        //
        // O equivalente estrutural é o `sculpt3d`, que entra como passe de GPU no `present.rs`
        // com `LoadOp::Load`. Aqui o traçado é de CPU, então o análogo honesto é a ordem de
        // pintura. Um canvas 3D de primeira classe (modo próprio, não `env`) é item de wave.
        //
        // No-op silencioso sem a variável; todo o estado vive no próprio módulo, de propósito
        // (`field3d_smoke`, §"Estado contido").
        // ⭐⭐ **A PARTE LIVRE DA ÁREA** (W50) — Enio, no smoke da W49: *"fica escondido entre
        // botões […] quando houver painel à direita melhor deslocar o gizmo para esquerda e
        // abaixar um pouco"*.
        //
        // ⚠️ A área que o módulo recebe é o **viewport inteiro**, e a moldura do app é pintada
        // por cima dele. Os retângulos vêm de quem os conhece — o `panel_rect` do store (só
        // publicado enquanto o painel está aberto) e o índice de acerto da faixa do topo (só
        // escrito no quadro em que ela de facto pintou). A **lei** de como eles empurram o
        // gizmo é pura e vive no módulo (`ph2d_viewport3d::navball::safe_corner`).
        {
            let mut obstacles: Vec<ph2d_editor_core::zones::Rect> = Vec::new();
            for id in crate::forwarding::CHROME_BACKDROPS {
                if let Some(r) = hero.hit_index.rect_for(id) {
                    obstacles.push(r);
                }
            }
            // ⚠️ **Todos** os painéis publicados neste quadro, sem lista de ids: uma segunda
            // cópia da lista que o `cursor_over_hero_panel` já carrega seria uma lista a mais
            // para alguém esquecer. Um painel flutuante no meio do canvas não move o gizmo — a
            // lei só conta quem toca a **aresta** da área.
            obstacles.extend(hero.store.panel_rects());
            // ⭐⭐ **A ÁREA é a de DESENHO, não a janela** (2026-08-30). Ela era o viewport
            // inteiro, e por isso as colunas docadas tocavam-lhe a aresta e **empurravam** o
            // gizmo — o remédio do sintoma que a D1 manda retirar quando os painéis passam a
            // ser regiões irmãs. Com a área certa, uma coluna docada deixa de a alcançar e a
            // fuga fica **inerte por construção**, sem uma linha de lei mudar.
            //
            // ⚠️ **A fuga FICA**, e não por preguiça: o que ainda a alcança são as janelas que
            // declaram flutuar (Grid Snap, galeria) — e a lei dela já diz que só conta quem
            // toca a **aresta**. Apagá-la deixaria o gizmo por baixo de uma dessas.
            //
            // ⚠️ `last_canvas` **é** a `HeroLayout::draw_area` publicada pelo quadro anterior
            // (ver `screens/hero/paint.rs`); no primeiro quadro ela é degenerada, e aí vale a
            // janela — que é o comportamento de sempre.
            let area = ph2d_viewport3d::layout::area(
                hero,
                ph2d_editor_core::zones::Rect::new(viewport.x, viewport.y, viewport.w, viewport.h),
            );
            let safe = ph2d_viewport3d::navball::safe_corner(area, &obstacles);
            ph2d_app_field3d::smoke::note_safe(safe);
            // ⭐⭐⭐ **E A ESCULTURA LÊ O MESMO PAR** (2026-09-08, ordem do Enio: *«traga esses
            // features para esse módulo»*). ⚠️ **Calculado UMA vez e publicado nos dois**, e não
            // duas vezes com a mesma receita: a lista de obstáculos deste quadro é a coisa cara
            // e é a que envelhece — dois censos dela divergiriam no quadro em que um painel
            // abre. *A área do canvas 3D é uma pergunta só; ter dois donos é ter duas
            // respostas.*
            #[cfg(feature = "sculpt3d")]
            if let Some(scene) = sculpt3d.as_mut() {
                // ⚠️ **A ÁREA primeiro, os gizmos depois** — eles moram no
                // quadrante ACTIVO, e quem sabe onde ele está é a divisão,
                // que acaba de ser publicada.
                scene.note_canvas(area);
                scene.note_nav(safe, self.last_pointer);
                scene.note_gizmo_hot(self.last_pointer);
            }
        }
        // ⭐⭐ **A VIAGEM ENTRE VISTAS** (W51) — Enio: *"falta um Lerp() rápido para mudança
        // suave das views como no blender"*.
        //
        // ⚠️ **A curva e a duração são as da CASA**, não minhas: `Role::Surface` é o papel cujo
        // doc descreve este caso à letra — *"viaja (o reduced motion mata-a) e **nunca
        // ultrapassa**… uma roda nomeia um DESTINO, e passar dele e voltar lê como a régua a
        // mentir"*. Uma vista nomeada é um destino, e com `Role::Travel` a peça passaria da
        // frente e voltava, com a janela inteira a balançar.
        //
        // ⚠️ **E ela NÃO morre no *reduced motion*** (W52, decisão do Enio): o papel é o
        // `Viewpoint`, e o critério dele é que *o que substitui esta animação é um CORTE que
        // desorienta mais do que ela*. Ver `ph2d_app_field3d::flight::ROLE`.
        if let Some((generation, fresh)) = ph2d_app_field3d::smoke::flight_track() {
            let id = ph2d_editor_core::ids::model3d_view_travel(generation);
            if fresh {
                // Semear em 0: a primeira vez que um id é visto, o `animate` **chega** ao alvo
                // (um widget que acaba de aparecer não tem de onde vir). Sem esta linha a
                // viagem estaria terminada antes de começar.
                hero.motion.animate(id, 0.0, ph2d_app_field3d::flight::ROLE);
            }
            let t = hero.motion.animate(id, 1.0, ph2d_app_field3d::flight::ROLE);
            ph2d_app_field3d::smoke::note_flight_progress(t);
        }
    }
}
