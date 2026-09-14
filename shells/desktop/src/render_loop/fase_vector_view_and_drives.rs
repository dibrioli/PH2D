//! **Fase do quadro: A VISTA VECTORIAL, OS ESTILOS CONDUZIDOS E AS RECOZEDURAS DE FORMA** — a vista do
//! documento, os vínculos e os estilos/widgets conduzidos, as transformações, e as recozeduras de conector,
//! morph, envelope, esqueleto e blend (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_view_and_drives(
        &mut self,
        vector_active: bool,
    ) -> Option<(ph2d_vec_scene::VecViewState, ph2d_vec_scene::VecXforms)> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            vec_scene,
            hero_screen,
            hero_live,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let mut vec_view = ph2d_vec_entities::entities::view_state(sim, &self.vec.entities);
        // **As MOLDURAS** (plano UI/UX W0): que intervalo da pilha cada uma recorta. Sai do
        // MESMO snapshot que acabou de ditar a pilha de z — derivá-lo de outra fonte seria uma
        // segunda resposta a *"em que ordem estas formas estão?"* — e da pilha FINAL, porque o
        // Z global pode ter tirado um descendente de dentro do intervalo.
        if let Some(live) = hero_live.as_ref() {
            let order: Vec<ph2d_vec_scene::VecPathId> =
                vec_scene.paths().iter().map(|p| p.id).collect();
            vec_view.clips = crate::vec_frame_spans::clip_spans(sim, &live.z_snapshot, &order);
        }
        // **OS TOKENS** (plano UI/UX W4): a tinta que cada binding produz no modo VIGENTE.
        // Resolvido aqui, no passe de DESENHO, e não dentro do `view_state` — aquela porta é
        // chamada por todo hit-test e gesto, e nenhum deles pergunta de que cor a forma é.
        vec_view.bound = crate::vec_bindings::resolve(
            sim,
            &self.vec.entities,
            crate::vec_bindings::TokenCtx {
                theme: hero.theme,
                pixels_per_meter: hero.project.pixels_per_meter,
            },
        );
        // ⭐⭐⭐ **A APARÊNCIA QUE UM MOTOR CONDUZ** — hoje a opacidade que a linha do tempo
        // escreve num caminho vetorial (`ph2d_ecs::VecDrivenStyle`). Depois dos tokens (ela
        // desvanece a tinta que eles resolveram) e **antes** das rows autoradas: se as duas
        // falarem da mesma forma, quem manda é o controlo que o artista está a segurar, e o
        // motor é o estado de fundo — o precedente é o passe de estados de UI, mais abaixo.
        let driven = crate::vec_driven_style::resolve(sim, &self.vec.entities);
        crate::vec_driven_style::apply(&driven, &mut vec_view);
        // ⭐ E o componente volta ao AUTORADO — depois de ser lido, nunca antes. Sem isto,
        // apagar uma track de opacidade deixava a forma congelada no último valor da curva
        // para sempre (ver o doc da função).
        crate::vec_driven_style::settle_to_authored(sim, &self.vec.entities, vec_scene);
        // **AS ROWS AUTORADAS** (plano UI/UX W8b.3): o valor VIVO de cada controle que dirige
        // uma forma. Depois dos tokens, porque a opacidade desvanece o que de fato vai ser
        // desenhado; e aqui, no passe de desenho, pela MESMA razão que os tokens — nenhum
        // hit-test pergunta em que ponto um slider está.
        let drives = crate::vec_widget_drive::resolve(sim, &self.vec.entities, &hero.store);
        crate::vec_widget_drive::apply(&drives, &mut vec_view);
        // ADR-0111 — cada path tem `Transform`. A geometria dele é LOCAL; este é
        // o afim que a leva ao mundo (a cadeia de pais inclusa).
        let mut vec_xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
        // **Conectores, 2ª metade:** a geometria é uma função pura da RELAÇÃO — re-cozida
        // aqui, todo frame, sobre os afins DESTE frame. É o que faz a linha SEGUIR a
        // forma que o gizmo acabou de mover.
        crate::connector_live::recook(
            sim,
            vec_scene,
            &self.vec.entities,
            &vec_xf,
            &mut self.vec.connect_sides,
        );
        // **Morph Objects, 2ª metade:** a forma é função pura das duas fontes e do `t` —
        // re-cozida aqui, todo frame, sobre os afins DESTE frame. É o que a faz SEGUIR a
        // forma que o gizmo acabou de mover, e o que faz o `t` da timeline virar movimento.
        // (O `t` já foi escrito: o apply da timeline roda antes desta metade do frame.)
        // ⭐ **A MÁQUINA DE MORPH, um quadro** (plano 32 W5) — ela escreve o PAR e o `t`, e o
        // `recook` logo abaixo transforma-os em forma. ⚠️ **Antes do recook, de propósito**: é
        // a mesma ordem pela qual o `t` da timeline vira movimento.
        //
        // ⚠️ **Só no MODO DE PRÉ-VISUALIZAÇÃO** (plano 32 W9). A condição de uma seta é uma
        // tecla; a escutar durante a edição, carregar em `Z` morfa a forma **e** faz o que o
        // `Z` faz no editor — os dois, sem nada na tela a explicar.
        //
        // ⛔ **O playhead deixou de ser a porta**, e a troca é a cura de um report do Enio
        // (2026-08-25): o Play **não tranca o teclado do editor**, então com ele a andar as
        // setas do teclado morfavam a forma *e* moviam as formas. Este modo tranca.
        crate::morph_machine_drive::tick(
            &mut self.morph_machines,
            sim,
            &self.vec.entities,
            &ph2d_input::Input::new(&hero.input_map, &self.input_actions),
            // ⛔⛔ **O sistema de States tem PRECEDÊNCIA** (W11e, 2.º report do Enio): ordenar
            // os dois motores dentro do quadro não bastava, porque a transição só fala no
            // MEIO — no repouso e na chegada quem escrevia era a máquina de teclas, parada
            // onde o ▶ a deixou. Ver `morph_machine_drive::drives`.
            crate::morph_machine_drive::drives(self.morph_preview, self.ui_state_live),
            self.fixed_step.fixed_dt(),
            &mut self.preview_drive,
        );
        // ⭐⭐⭐ **O conjunto de estados ANIMADO POR UMA TRANSIÇÃO DE UI** (plano 32 W11c) —
        // corre depois do `tick` e ANTES do `recook`, que é a única janela em que faz sentido:
        // ele escreve o par e o `t`, e o `recook` é quem os transforma em geometria.
        //
        // ⚠️ **Depois do `tick` de propósito:** se as duas coisas escrevem o mesmo objecto,
        // quem manda é a transição de UI — ela é o gesto que o artista acabou de fazer, e a
        // máquina de teclas é o estado de fundo.
        crate::morph_machine_drive::apply_ui_steps(
            sim,
            &self.vec.entities,
            &self.ui_cooked.morph_steps,
            &mut self.preview_drive,
        );
        crate::morph_live::recook(
            sim,
            vec_scene,
            &self.vec.entities,
            &vec_xf,
            &mut self.vec.morph_plans,
        );
        // **Envelope Objects (ADR-0129):** a forma de cada filho é a fonte autorada deformada
        // pela gaiola comum — re-cozida aqui, todo frame. Sem xforms nem mapa: a fonte é LOCAL do
        // container e é o `Transform` do container (via `vec_transform::build`) que leva os filhos
        // ao mundo; o recook varre os containers por QUERY (eles não têm path).
        crate::envelope_live::recook(sim, vec_scene);
        // ⭐⭐⭐ **O ESQUELETO** (estudo 42 item 5): a forma presa aos ossos é re-cozida da fonte
        // autorada e da pose de AGORA. Ao lado do envelope de propósito — os dois deformam a
        // partir de uma fonte guardada em bytes — e DEPOIS dele, porque a pele fala de formas
        // que já existem na cena e o envelope pode acabar de reescrever uma.
        //
        // ⭐⭐⭐ **AS ÂNCORAS** — a cinemática INVERSA que persiste. Corre **imediatamente antes**
        // do recook da pele, e a ordem é load-bearing: ela escreve a pose dos ossos, e o recook
        // é quem transforma a pose em geometria. Ao contrário, a pele mostraria a pose do quadro
        // anterior — um atraso de um quadro, invisível parado e visível a arrastar.
        //
        // ⚠️ O `preview_drive` entra na assinatura porque o que este passe escreve é
        // **pré-visualização**: o documento é a pose da ÂNCORA, e a rotação dos ossos governados
        // é derivada dela.
        // ⭐⭐⭐ **OS OSSOS INTELIGENTES correm ANTES da âncora**, e a ordem é load-bearing:
        // um controlo escreve a pose de BASE (ele é a correcção autorada) e a IK é a restrição
        // que persegue um alvo — ela tem de ver a pose já corrigida. ⛔ Ao contrário, a IK
        // resolveria sobre uma pose que a acção ainda vai mudar, e o alvo deixaria de ser
        // alcançado no mesmo quadro.
        // ⚠️ Sem `xforms`: a pele resolve a pose de cada osso e da forma pela hierarquia (a
        // propagação de `Transform` que a casa já corre), que é a mesma razão de a cinemática
        // directa não precisar de código.
        crate::skeleton_live::recook(sim, vec_scene);
        // **Select: arrastar o objeto blend move as fontes** — o gizmo mira as FONTES (não o
        // spine), então ele as move NATIVAMENTE como grupo (`vec_selection::sync_selection`
        // redireciona a seleção do gizmo). O spine as segue no `recook`. Nada a fazer aqui: um
        // gizmo sobre o spine dobraria (Transform + bbox que já andou); sobre as fontes não, a
        // geometria delas é fixa e só o `Transform` se move.
        // **Modo Node: arrastar uma ÂNCORA do spine move a forma-fonte dela** (ADR-0128 C2b) —
        // o inverso da pinagem. Roda ANTES do recook: move a fonte para a âncora arrastada e o
        // recook então re-encosta a âncora no centro (agora coincidentes, sem salto). Como a
        // fonte se moveu, o `vec_xf` é refeito para os passos deste frame já saírem do lugar
        // novo. Só no Node — no Select a fonte se move pelo gizmo (acima) e a âncora a segue.
        if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node {
            crate::blend_live::drag_spine_anchors_move_sources(
                sim,
                vec_scene,
                &self.vec.entities,
                &vec_xf,
                &mut self.vec.blend_spines,
            );
            vec_xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
        }
        // **Blend Objects, 2ª metade:** os passos são função pura das fontes — re-cozidos
        // aqui, todo frame, sobre os afins DESTE frame. É o que faz a transição SEGUIR a
        // forma que o gizmo acabou de mover (ADR-0128). O buffer é zerado e repopulado.
        crate::blend_live::recook(
            sim,
            vec_scene,
            &self.vec.entities,
            &vec_xf,
            &mut self.vec.blend_spines,
            &mut self.vec.blend_overlay,
        );
        // **Modo Node: o spine sobe para o topo** (ADR-0128) — acima de TODAS as formas e
        // passos, para ser visto e editado. Retira o traço da cena (some do `dispatch`, logo
        // abaixo) e o acrescenta ao fim do overlay do blend (desenhado por último). Em Select
        // o spine fica no seu z (traço sutil), como o Illustrator — o `recook` restaura o
        // traço-base todo frame, então voltar de Node não o deixa invisível.
        if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node {
            crate::blend_live::elevate_spines(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.blend_overlay,
            );
        }
        Some((vec_view, vec_xf))
    }
}
