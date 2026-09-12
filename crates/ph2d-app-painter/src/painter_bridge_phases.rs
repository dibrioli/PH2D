//! **AS FASES CURTAS do quadro do Painter** — os pedaços do [`crate::painter_bridge::dispatch`]
//! que respondem a UMA pergunta e não partilham estado com o resto dele.
//!
//! # ⚠️ Por que eles saíram (W2 Fase D)
//!
//! Não foi estética: o `painter_bridge.rs` passou a viver em `crates/` e ali o teto por ficheiro é
//! o `architecture_workspace_file_loc_cap` (**700**), que ⛔ **não honra o marcador
//! `// ph2d-loc-cap:`** que o ficheiro carregava desde que era da shell. *Mover um ficheiro pode
//! trocar o REGIME de teto que o governa, e uma isenção textual não viaja com ele.*
//!
//! A cura é a que o `CLAUDE.md` §5.0 manda — **corte por responsabilidade**, nunca uma entrada
//! nova na lista de folgas. Cada função daqui é uma fase nomeada do quadro, com os argumentos
//! escritos em TIPOS: nenhuma delas precisa da `App`, e nenhuma precisa do resto do `dispatch`.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::ToolRegistry;

/// **A visibilidade dos docks que a ferramenta possui.**
///
/// Com o Painter activo, a ranhura partilhada do Inspector é tomada pelo painel de Camadas. O
/// esconder é disparado por ARESTA, para não pisar um toggle manual do rail.
pub(crate) fn dock_visibility(hero: &mut HeroScreen, painter_is_active: bool) {
    // When the painter (layers + effects) tool is active, the shared Inspector
    // slot is taken over by the docked Layers panel. Edge-triggered inspector
    // hide so it doesn't stomp a manual rail toggle.
    hero.panel_visibility
        .insert("painter_layers", painter_is_active);
    // The Wet Tuning side panel can only be open UNDER the painter: with the
    // tool inactive the downcast block below never runs, so the OFF half is
    // written here (a stale `true` would leave the panel floating tool-less).
    if !painter_is_active {
        hero.panel_visibility.insert("wet_tuning", false);
    }
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(painter_is_active, Ordering::Relaxed);
        if was != painter_is_active {
            // ⚠️ **A promoção FICA, o esconder é que saiu.** Ela não é o *takeover*: o
            //    `PAINTER_LAYERS_PANEL` não está na lista de recurso da ordem z do
            //    `ph2d-editor-core`, logo sem ela o painel nem entra na travessia de pintura —
            //    e com as abas ela é também o que o põe à FRENTE da fila ao abrir a ferramenta.
            if painter_is_active {
                hero.store
                    .bump_panel_z(ph2d_editor_core::ids::PAINTER_LAYERS_PANEL);
            }
        }
    }
}

/// **O picker de cor partilhado escreve no PINCEL.**
pub(crate) fn forward_picker_colour(hero: &mut HeroScreen, tools: &mut ToolRegistry) {
    // The shared Blender picker mirrors its live value into `widget_color(PAINTER_COLOR_THUMB)` each frame,
    // but the panel's widget→brush forward is SKIPPED in Selection mode and STOPS the instant the picker
    // closes — so the final pick (the one that closed the picker) never reached the brush and the next Fill
    // used the PREVIOUS colour (Enio 2026-07-03). Forward it here instead: every frame the painter is active
    // (works in Selection mode too) while the picker targets the thumb, AND once on the open→close edge to
    // catch that final pick. Reads the picker's OWN value (not `widget_color`, which the panel overwrites),
    // so it's independent of paint order.
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static PICKER_WAS_OPEN: AtomicBool = AtomicBool::new(false);
        let open = hero.store.picker_target() == Some(ph2d_editor_core::ids::PAINTER_COLOR_THUMB);
        let was_open = PICKER_WAS_OPEN.swap(open, Ordering::Relaxed);
        if (open || was_open)
            && let Some((value, _, _, _)) = hero
                .store
                .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
            && let Some(painter) = tools.active_mut().and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
            })
        {
            let [r, g, b, _] = value.rgba;
            painter.set_brush_color_srgb8([r, g, b]);
        }
    }
}

/// **A DOAÇÃO de forma — publica o TAMANHO, consome a NOTÍCIA.**
///
/// O canal nos dois sentidos entre a escultura e a tinta. ⚠️ Este é o único ponto que liga as
/// duas, e ele **não sabe disso**: o que vê é um plano de `f32` que alguém deixou no canal. É essa
/// ignorância que mantém a promessa de `docs/3D/02.3` — apagar o módulo 3D deixa isto a existir,
/// a publicar um tamanho que ninguém lê e a nunca receber notícia.
pub(crate) fn donate_form(
    tools: &mut ToolRegistry,
    painter_is_active: bool,
    donated_form: &mut ph2d_form_donation::donated_form::DonatedForm,
) {
    //
    // Este é o único ponto do shell que liga uma escultura à tinta, e ele não sabe disso: o que ele
    // vê é um plano de `f32` que alguém deixou no canal. É essa ignorância que mantém a promessa de
    // `docs/3D/02.3` — apagar o módulo 3D deixa este bloco existindo, publicando um tamanho que
    // ninguém lê e nunca recebendo notícia.
    //
    // ⚠️ **A ordem é publicar-DEPOIS-instalar, e ela é load-bearing:** o produtor lê o tamanho no
    // frame SEGUINTE, então publicar aqui é o que faz um documento recém-bindado ser rasterizado.
    // Instalar antes de publicar não muda nada hoje e mentiria sobre a dependência.
    if painter_is_active
        && let Some(tool) = tools.active_mut()
        && let Some(painter) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
    {
        let (w, h) = painter.canvas_size();
        // Canvas vazio é **ausência**, não `(0, 0)`: o produtor tem de ficar quieto, e um par de
        // zeros o faria rasterizar uma extensão que o `form_plane` recusa de qualquer jeito.
        donated_form.canvas = (w != 0 && h != 0).then_some((w, h));
        if let Some(news) = donated_form.news.take() {
            // ⚠️ **As duas metades são instaladas do MESMO `news`, e é isto que o arch-gate
            // `the_bridge_installs_both_halves_of_a_donation` exige.** O tool as aceita por portas
            // separadas (o neutro da oclusão é `1.0`, então uma ausência ali é legítima); o que não
            // pode é este sítio entregar uma e esquecer a outra, porque aí a fresta que o artista
            // vê na escultura não apareceria na tinta e nada daria erro.
            let (normal, occlusion) = match news {
                Some(planes) => (Some(planes.normal), Some(planes.occlusion)),
                None => (None, None),
            };
            painter.set_donated_form(normal);
            painter.set_donated_occlusion(occlusion);
        }
    }
}

/// **O BIND do documento:** quando o Painter não tem documento para a selecção, empurra-lhe os
/// pixels dela.
///
/// ⚠️ Por `bind_document` e **não** pelo `set_source` genérico, para que a pilha de camadas do
/// sprite que SAI fique guardada por id em vez de achatada — trocar de sprite preserva as camadas
/// de cada um (Enio 2026-06-26).
///
/// ⚠️ **Quem LÊ os pixels é o `read_source`, um fecho da shell:** ler pixels de um sprite precisa
/// de `SimWorld + SpriteRenderer + AssetDb`, e a `ph2d-tool-runtime` já declarou por escrito que
/// isso *«is shell foundation»*. Ele devolve alfa DIRECTO, que é o que o `bind_document` quer.
/// **O que o bind precisa emprestado**, num nome só.
///
/// ⛔⛔ **Isto NÃO é um saco para calar o `too_many_arguments`** — silenciar um diagnóstico é
/// armengo mesmo quando a ferramenta parece exagerada, e o `#[allow]` está proibido por escrito
/// neste repo. O agrupamento é o que a própria queixa pede: *«agrupar argumentos numa struct»*.
///
/// ⚠️ **Construída no SÍTIO DA CHAMADA, com os campos nomeados** (o molde do `CanvasCtx` da
/// `line/app-physics`, Fase C): um construtor que recebesse a `App` emprestaria-a INTEIRA e o
/// empréstimo disjunto desaparecia — *ele só existe quando os campos são nomeados no mesmo escopo
/// que os usa*.
pub(crate) struct BindCtx<'a> {
    pub hero: &'a mut HeroScreen,
    pub tools: &'a mut ToolRegistry,
    pub renderer: &'a mut ph2d_render::SpriteRenderer,
    pub last_painter_pushed_entity: &'a mut Option<u64>,
    pub painter_preview_gpu: &'a mut Option<ph2d_preview_slot::PreviewGpu>,
    pub painter_gpu_preview: &'a mut Option<crate::painter_gpu_preview::PainterGpuPreview>,
    pub toasts: &'a mut ph2d_editor_core::toast::ToastQueue,
}

pub(crate) fn bind_document(
    ctx: BindCtx<'_>,
    painter_is_active: bool,
    // ⚠️ **O `renderer` CHEGA pelo argumento e não pela captura**, e a razão é um empréstimo:
    // ele vive no [`BindCtx`] *e* é preciso para ler os pixels. Um fecho que o capturasse pedia
    // acesso único a algo que o contexto já emprestou (`E0500`) — *o empréstimo disjunto só
    // existe quando cada campo é nomeado UMA vez*.
    read_source: impl FnOnce(
        ph2d_ecs::Entity,
        &mut ph2d_render::SpriteRenderer,
    ) -> Option<(Vec<u8>, u32, u32)>,
) {
    let BindCtx {
        hero,
        tools,
        renderer,
        last_painter_pushed_entity,
        painter_preview_gpu,
        painter_gpu_preview,
        toasts,
    } = ctx;
    // Push the selected sprite's pixels into the painter, but via `bind_document` (NOT the generic
    // `set_source`) so the OUTGOING sprite's multi-layer stack is stashed by id instead of flattened —
    // switching sprites preserves each sprite's layers (Enio 2026-06-26). Painter canvas storage is RGBA8
    // straight (matches bgremoval's `into_straight()`); 0×0 sources are rejected at the boundary.
    //
    // ⚠️ **The TOOL decides whether it needs a document** (`needs_document_bind`), not this memo. The
    // memo was a second copy of a fact the tool owns, and it went stale in one specific way: leaving the
    // Painter with nothing unbaked tears the canvas down without clearing it, so the memo still named
    // the sprite while the tool had no pixels — and the re-push that would have fixed it was skipped
    // *because* the memo said it was already done. Coming back, the canvas was 0×0, so every canvas
    // pointer fell through and the artist dragged the sprite instead of painting it (Enio 2026-07-22).
    // `last_painter_pushed_entity` still records what was pushed (the bake bookkeeping below reads it),
    // but it no longer gets a vote on whether to push.
    if painter_is_active
    && let Some(tool) = tools.active_mut()
    && let Some(painter) = tool
        .as_any_mut()
        .downcast_mut::<ph2d_tool_painter::PainterTool>()
    && let Some(bits) = hero.gizmo.selection
    && painter.needs_document_bind(bits)
    // PRECISION-READONLY: isto é o BIND do documento do Painter, não uma escrita. O documento
    // dele é de 8 bits por desenho (`docs/Sprite_projeto/19` §4), e quem escreve os pixels de
    // volta é o `hero_intents::image_edit::painter`, no Apply, por `commit_edited_texture` — o
    // funil que avisa. ⚠️ Selecionar um sprite com o Painter ligado **não** custa precisão
    // nenhuma: sem Apply, a sprite não muda.
    //
    // ⚠️ **Quem LÊ é um fecho da shell** (W2 Fase D): ler os pixels de um sprite precisa de
    // `SimWorld + SpriteRenderer + AssetDb`, e a `ph2d-tool-runtime` já declarou por escrito que
    // isso *«is shell foundation»* e recusa depender daquilo. O idioma da casa é o mesmo dela —
    // *bridges produce this via a shell-specific reader closure*.
    && let Some((pixels, pw, ph)) = read_source(ph2d_ecs::Entity::from_bits(bits), renderer)
    // ⚠️ As dimensões entram na CADEIA e não num `if` aninhado: o `into_straight` agora corre
    // do lado da shell, dentro do fecho, então o que chega aqui já são pixels + dimensões — e um
    // documento de 0×0 nunca deve chegar a `bind_document` (foi um canvas 0×0 que fez todo
    // ponteiro de canvas cair através, Enio 2026-07-22).
    && pw != 0
    && ph != 0
    {
        painter.bind_document(bits, pixels, pw, ph);
        // ⚠️ E COMPILA os shaders do preview GPU agora, no vão humano entre escolher o sprite e
        // levar o mouse à tela — senão os 28 ms de criação de pipeline caem no primeiro traço, que
        // é o gesto em que o artista está esperando (doc 28 §4.8, medido).
        crate::painter_gpu_preview::prewarm(
            painter_gpu_preview,
            renderer,
            painter,
            bits,
            painter_preview_gpu,
            toasts,
        );
        // E instala a ponte do CARIMBO no mesmo vão, pela mesma razão: construir o passe
        // compila um shader, e o custo não pode cair no primeiro traço (doc 33 §S3).
        crate::painter_stamp_device::install(painter, renderer);
        // Impasto smoke: arm the brush the first time a document binds, so the artist drags and sees
        // thick lit paint instead of hunting for the knobs. One-shot; never overwrites their edits.
        crate::impasto_smoke::arm_brush_once(painter);
        crate::wetpaint_smoke::arm_brush_once(painter);
        crate::substrate_smoke::arm_brush_once(painter);
        crate::line_smoke::arm_brush_once(painter);
        *last_painter_pushed_entity = Some(bits);
        // The bind abandons any pending Fill (tool side); close its now-orphaned adjust modal too, so
        // switching sprites never leaves a stale Fill modal floating over the new one.
        hero.store.close_fill_modal();
    }
}

/// **O caminho INACTIVO e o APPLY** — as duas maneiras de a ponte assentar no fim do quadro.
///
/// Com a ferramenta inactiva, limpa só o estado LOCAL da ponte (o do tool já foi limpo pelo
/// `on_deactivate`, via `ToolRegistry::set_active` — espelho da correcção da auditoria C1 da
/// remoção de fundo). Com um Apply pendente, empurra um `OneShotImageOp` por sprite e **liberta a
/// ranhura de pré-visualização explicitamente**, porque o produtor de GPU desligou o
/// `None => release` do lado da CPU.
///
/// Devolve `true` quando o Apply correu — o sinal de que a posse da pré-visualização voltou à CPU.
pub(crate) fn settle_inactive_and_apply(
    hero: &mut HeroScreen,
    renderer: &mut ph2d_render::SpriteRenderer,
    painter_is_active: bool,
    apply_selection: &[u64],
    painter_preview: &mut Option<ph2d_tool_runtime::PreviewCache>,
    last_painter_pushed_entity: &mut Option<u64>,
    painter_preview_gpu: &mut Option<ph2d_preview_slot::PreviewGpu>,
) -> bool {
    if !painter_is_active {
        *painter_preview = None;
        *last_painter_pushed_entity = None;
    }
    if apply_selection.is_empty() {
        return false;
    }
    for bits in apply_selection {
        hero.bus
            .push(ph2d_editor_core::action_bus::EditorAction::OneShotImageOp {
                tool_id: "painter",
                entity_bits: *bits,
            });
    }
    *painter_preview = None;
    crate::painter_bridge_upload::release_preview_texture(renderer, painter_preview_gpu);
    true
}
