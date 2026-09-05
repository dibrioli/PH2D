//! Editor Action Bus — outbound intent queue from the hero screen.
//!
//! Wave 2.5 PR 11.8 foundation. Replaces the per-frame `hero.pending_X`
//! drain pattern with a single FIFO queue of strongly-typed
//! [`EditorAction`] variants. The shell drains the bus once per frame
//! and dispatches each action through a single `apply_editor_action`
//! match arm instead of 20 hand-written drain blocks scattered across
//! [`shells/desktop/src/main.rs`](shells/desktop/src/main.rs) and
//! [`shells/desktop/src/hero_intents.rs`](shells/desktop/src/hero_intents.rs).
//!
//! ## Why a queue instead of `Option<T>` fields
//!
//! Each `hero.pending_X: Option<T>` represented an at-most-one intent
//! per frame. The shell did `if let Some(v) = hero.pending_X.take() { ... }`
//! at ~20 sites in `render_frame()`. That pattern grew main.rs to
//! 2421 LOC and hero_intents.rs to 696 LOC — both currently carrying
//! `// ph2d-loc-cap:` exceptions (Wave 2 PR 11.9).
//!
//! With the bus, each push is a structurally-typed enum variant
//! carrying its payload. Drain is one `match` over a `Vec` instead of
//! 20 conditionals over scattered fields. Migration is incremental —
//! each `pending_X` field that lifts into [`EditorAction`] takes
//! `~10-20 LOC` out of main.rs and shrinks the HR-18 exception window.
//!
//! ## Determinism
//!
//! Actions drain in push order. The hero pushes from within
//! [`HeroScreen::apply_event`] which itself runs once per pointer/key
//! event; the shell drains after the per-frame `apply_event` cascade.
//! Per-event ordering is preserved (HR-5).
//!
//! ⚠️ **O parágrafo de âmbito desta fundação MORREU em 2026-08-31**: ele prometia *«3 variantes
//! representativas — `Trim` / `MakeSquare` / `Bgremoval`»* sobre um enum que já passa das 150, e
//! a migração dos `pending_X` que ele anunciava está feita há muito. *Prosa que descreve o commit
//! em vez do ficheiro envelhece sem que nada fique vermelho.*

/// ⛔⛔⛔ **MEDIDO 2026-09-01: acrescentar UMA variante a este enum custa +78 LINHAS.**
///
/// Não é o conteúdo — é o `rustfmt`. Com as variantes-estrutura em linha o ficheiro é estável a
/// 676; **uma variante nova (com carga ou sem ela) faz o formatador reflow-ar as 37 variantes de
/// estrutura para multi-linha de uma vez**, e o ficheiro salta para ~754, acima do tecto de 700.
/// ⚠️ **Um comentário acrescentado NÃO o dispara** (676 → 677), então o que o move é a contagem de
/// variantes, e não o tamanho.
///
/// ⇒ **quem acrescentar a próxima paga o corte**, e o corte por responsabilidade que falta está
/// nomeado: a família `Hier*` são **33** variantes com o mesmo sujeito (uma `row`) e o mesmo dreno
/// — ela vira `EditorAction::Hierarchy(HierRequest)` num irmão, como a `VariationRequest` fez.
/// ⛔ Não subir o tecto: *a cura de um teto estourado é o corte; subir o número é adiar com juros.*
///
/// One outbound intent from the editor to the shell. Variants are
/// added incrementally as `pending_X` fields migrate into the bus.
/// Each variant carries enough payload that the shell can dispatch
/// without re-reading `HeroScreen` state.
///
/// **Invariant:** every variant is `Copy` or holds owned data — never
/// borrows from `HeroScreen`. The bus must be drainable after the
/// per-frame `apply_event` cascade returns its `&mut self` borrow.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum EditorAction {
    /// Set the active tool by canonical id (the manifest's `id` field).
    /// Generic activation variant (ADR-0040 TG-A): replaces the
    /// per-tool `ActivateBgRemoval` / `ActivatePadding` etc. Shell
    /// drains by routing through `ToolRegistry::set_active`; the
    /// `mode_on` gate for image tools applies at the shell side, not
    /// in the variant. Adding a new modal tool does not require a new
    /// variant.
    ActivateTool {
        tool_id: &'static str,
    },

    /// Apply a one-shot or stateful-bake image edit on `entity_bits`,
    /// dispatched by `tool_id` (the manifest id). Generic image-edit
    /// variant (ADR-0040 TG-A): replaces the per-tool `Trim` /
    /// `MakeSquare` / `RealSize` (stateless one-shots) and `Bgremoval`
    /// (stateful bake reading the active tool's pipeline state). The
    /// shell dispatches by `tool_id` to the matching drain function
    /// (which still reads tool-specific state from the active tool when
    /// needed — Padding/BgRemoval bake). The ordering / `mode_on` /
    /// "stays intact across tool switches" gating applies at the shell
    /// drain site, not in the variant. Adding a one-shot image op does
    /// not require a new variant.
    OneShotImageOp {
        tool_id: &'static str,
        entity_bits: u64,
    },

    /// Generic panel → tool event channel (ADR-0040 TG-B). Typed
    /// panels (`ph2d-panel-bgremoval`, `ph2d-panel-padding`, …) convert
    /// their `WidgetEvent`s into a tool-agnostic [`crate::tool::PanelEvent`]
    /// (carrying the widget's `NodeId` + payload) and push this variant;
    /// the shell drains by calling
    /// `tools.active_mut().map(|t| t.handle_panel_event(ev))`. The
    /// semantic mapping (slider id → `BgRemovalUiEdit::Tolerance(v)`)
    /// lives in the tool's `handle_panel_event`, not in the panel, so
    /// adding a panel-edit semantic does not require a new variant here.
    ToolPanelEvent(crate::tool::PanelEvent),

    /// Generic panel → **timeline** event channel (docs/Timeline). The
    /// bottom-docked `ph2d-panel-timeline` is a document panel (not a tool), so
    /// its transport / ruler / key edits cannot route through
    /// [`Self::ToolPanelEvent`]. It reuses the tool-agnostic
    /// [`crate::tool::PanelEvent`] (a `NodeId` + payload) and the shell drains by
    /// translating the id into a `ph2d_timeline::TimelineIntent` and pushing it
    /// onto its intent queue (the timeline semantics live in the shell, keeping
    /// editor-core free of the timeline crate — mirror of `ToolPanelEvent`).
    TimelinePanelEvent(crate::tool::PanelEvent),

    /// Generic cancel of the active modal tool (ADR-0040 TG-B/TG-C).
    /// Shell drains by calling `tools.set_active(default)` and tearing
    /// down any tool-specific shell-side preview state. Raised by both
    /// panels' Cancel buttons (BgRemoval + Padding).
    CancelActiveTool,

    /// ⭐ **Ligar/desligar o módulo 3D.** Porquê e o que ele arma: `docs/3D/` + ADR-0150.
    ToggleSculpt3d,

    /// Re-decode the entity's sprite source asset at the current
    /// `ProjectSettings::pixels_per_meter` and write the recomputed
    /// world size back to `Sprite.size`. Payload: `entity.to_bits()`.
    /// Raised by the Inspector's "Reimport at current px/m" button
    /// (`INSP_RENDER_SOURCE_REIMPORT`). Texture itself unchanged;
    /// only `Sprite.size` is recomputed.
    Reimport {
        entity_bits: u64,
    },

    /// **Converter a PRECISÃO dos pixels de um sprite** — plano
    /// `docs/Sprite_projeto/18` W5. Levantada pelo par `RGBA8 / RGBA16` da seção Render Source.
    ///
    /// ⚠️ **Levantar não é converter**, e a divisão importa: o painel não tem o `AssetDb` nem o
    /// device, e a conversão para 16 bits **muda a estratégia** da sprite (o atlas é uma textura
    /// com um formato, §3.3). Quem sabe as três coisas é o shell — o painel só diz *o que o artista
    /// pediu*.
    ///
    /// ⚠️ Pedir a precisão que já está **não é uma edição**: o `event.rs` curto-circuita, senão um
    /// clique distraído no botão já aceso viraria um passo de undo sobre uma cena que não mudou.
    InspectorSpritePrecisionChange {
        entity_bits: u64,
        precision: ph2d_color::Precision,
    },

    /// **Quanto esta sprite EMITE** — `docs/Sprite_projeto/18` W8. Levantada pelo slider `Emissive`
    /// da secção Render Source.
    ///
    /// ⚠️ **Não é um campo do `Sprite`, e por isso não viaja pelo `SpriteFieldEdit`.** O que ela
    /// muda é a presença de um **componente opcional** (`ph2d_ecs::SpriteEmissive`), e quem tem o
    /// `ComponentRegistry` para o inserir ou remover é o shell — o mesmo caminho do `TextureFilter`.
    ///
    /// ⚠️ **Intensidade zero REMOVE o componente**, em vez de o deixar a zero. Uma sprite que não
    /// emite não tem por que carregar a linha no ficheiro, e o quadro tem de voltar a ser
    /// byte-idêntico ao de antes de alguém ter mexido no slider.
    InspectorSpriteEmissiveChange {
        entity_bits: u64,
        intensity: f32,
    },

    /// Undo the most recent image-edit (Trim Transparency / Make
    /// Square / Bg Removal). No payload — the shell owns the
    /// snapshot. Single-level by design, and it is now the FALLBACK of
    /// [`Self::UndoStep`] (the shell routes to it when no other undo
    /// owner has a step), not something any button raises directly.
    UndoImageEdit,

    /// **O desfazer do editor** — os botões `TOOL_UNDO` / `TOOL_REDO` da barra
    /// esquerda. `redo = true` é o refazer.
    ///
    /// O shell roteia isto pelo MESMO caminho do Ctrl+Z (`App::undo_or_redo`), e é a
    /// coisa toda: enquanto o botão tinha caminho próprio, ele levantava
    /// `UndoImageEdit` — o undo de IMAGEM, single-level — enquanto o atalho desfazia o
    /// projeto. Mover uma forma e clicar em Undo não fazia nada. E o botão **Redo não
    /// despachava coisa alguma**: era pintado, clicável, e órfão.
    UndoStep {
        redo: bool,
    },

    /// Canvas multi-select event (Fase 0b). Raised by the desktop
    /// shell's canvas pick handler after resolving the click to its
    /// `entity_bits` and reading the OS modifier into a
    /// [`SelectModifier`]. Shell drains by routing through the
    /// matching `GizmoStateGroup` mutation: `Replace` →
    /// `replace_selection`, `Add` → `add_to_selection`, `Toggle` →
    /// `toggle_in_selection`. Hierarchy clicks use the row-keyed
    /// twin [`Self::HierSelectRow`] since the panel cannot resolve
    /// `NodeId → entity_bits` itself.
    SelectSprite {
        entity_bits: u64,
        modifier: SelectModifier,
    },

    /// Drop primary + extras in one go (Fase 0b). Raised by canvas
    /// click on an empty area without modifier, and by pressing Esc
    /// while a selection is active. Shell drains by calling
    /// `GizmoStateGroup::clear_all_selection`.
    ClearSelection,

    /// Reframe the camera. Payload: which mode to fire (Selected
    /// focuses the current `gizmo_selection`; Camera resets to the
    /// project's default view; All frames every sprite in the scene
    /// with a 10% padding). Raised by clicking `TOOL_HOME` on the
    /// LeftRail (which cycles the 3 modes) and by double-clicking
    /// a live hierarchy row (always `Selected`).
    SetViewFocus {
        kind: crate::screens::hero::ViewFocusKind,
    },

    /// Inspector → shell channel for `Transform` edits — the first
    /// end-to-end consumer of the editor command pipeline. Payload
    /// is the full snapshot the inspector built from its NumberInput
    /// buffers (entity_bits + translation/rotation/scale). Shell
    /// drains, builds a `ph2d_ecs::Transform` from the raw fields,
    /// and pushes a `EditorCommand::SetComponent` to its
    /// `EditorCommandQueue`. Raised by NumberInput commits (Enter /
    /// blur) on `INSP_TRANSFORM_*` and by the Reset Transform button.
    InspectorTransformEdit(crate::screens::hero::InspectorTransformInfo),

    /// Inspector → shell channel for `Visibility` commits. Payload:
    /// the POST-toggle snapshot `(entity_bits, visible)`. Shell drains
    /// and pushes a `EditorCommand::SetComponent` for
    /// `ph2d_ecs::Visibility` — same pipeline as
    /// [`Self::InspectorTransformEdit`]. Raised by flipping the
    /// `INSP_VISIBILITY_CHECK` checkbox.
    InspectorVisibilityEdit(crate::screens::hero::InspectorVisibilityInfo),

    /// Inspector → shell channel for a §8 Visibility-SECTION field
    /// (VisibilityLayer bitmask / ClipChildren / MaskInteraction /
    /// OnScreenEnabler). Optional-component edit like
    /// [`Self::InspectorSamplingEdit`] (W3 §3.8). Distinct from
    /// [`Self::InspectorVisibilityEdit`], which is the always-on
    /// "Visible" toggle.
    InspectorVisibilitySectionEdit {
        entity_bits: u64,
        edit: crate::screens::hero::VisibilityFieldEdit,
    },

    /// Inspector → shell channel for `Sprite` source-strategy
    /// switches. Payload: `(entity_bits, requested_strategy)`.
    /// Shell does the actual swap: Atlas → Individual re-decodes
    /// the source asset via `atlas_asset_map` + `acquire_individual`;
    /// Individual → Atlas and HandPacked transitions surface a toast
    /// in v1. Raised by picking a different segment in the Render
    /// Source segmented switcher.
    InspectorSpriteSourceChange {
        entity_bits: u64,
        strategy: crate::screens::hero::RequestedSpriteStrategy,
    },

    /// Inspector → shell channel for a single editable `Sprite` field
    /// (flip, sprite-sheet grid, region, tint channels, opacity, …).
    /// The shell reads the entity's current `Sprite`, applies the one
    /// `edit` (clamping per the schema), and writes the whole struct via
    /// `EditorCommand::SetComponent` — same path as the Transform commit.
    /// Raised by the Render Source / Sprite Sheet / 9-Slice / Color & Tint
    /// sections (W2 Sprite Inspector v2).
    InspectorSpriteEdit {
        entity_bits: u64,
        edit: crate::screens::hero::SpriteFieldEdit,
    },

    /// Inspector → shell channel for a single editable §7 ordering field
    /// (Z Index, Sorting Layer, Y-Sort, Show Behind Parent, …). Unlike
    /// [`Self::InspectorSpriteEdit`], each maps to an *optional* ECS
    /// component: the shell reads-or-defaults, applies, and commits via
    /// `EditorCommand::SetComponent` (insert/update) or `RemoveComponent`
    /// (detach). Raised by the Ordering / Sorting section (W3 Sprite
    /// Inspector v2 §3.7).
    InspectorOrderingEdit {
        entity_bits: u64,
        edit: crate::screens::hero::OrderingFieldEdit,
    },

    /// Inspector → shell channel for a §9 sampling field (Texture Filter
    /// / Repeat). Optional-component edit like
    /// [`Self::InspectorOrderingEdit`] (W3 §3.9).
    InspectorSamplingEdit {
        entity_bits: u64,
        edit: crate::screens::hero::SamplingFieldEdit,
    },
    /// §5 9-Slice (spec Sprite 03 §3.5) — uma edição da autoria de 9-slice, incluindo anexar e
    /// retirar o próprio componente. A seção nasceu em 2026-08-21; até aí a spec descrevia-a e o
    /// repositório não tinha uma linha dela.
    InspectorSliceEdit {
        entity_bits: u64,
        edit: crate::screens::hero::SliceFieldEdit,
    },
    /// §12 Sockets / Named Anchors (ADR-0072) — uma edição da lista de âncoras, incluindo criar
    /// e retirar. ⚠️ `Clone` e não `Copy`: o `Rename` carrega o nome, e um nome é o que
    /// distingue um socket de outro (spec §7.14 anti-padrão 4).
    InspectorAnchorEdit {
        entity_bits: u64,
        edit: crate::screens::hero::AnchorFieldEdit,
    },
    /// §11 Animation (spec Sprite 08) — uma edição da biblioteca de animações **ou** do estado
    /// de reprodução. ⚠️ `Clone` e não `Copy`, como a irmã acima: o `Rename` e o `SetCurrent`
    /// carregam o nome, e é o nome que distingue uma animação de outra.
    InspectorAnimEdit {
        entity_bits: u64,
        edit: crate::screens::hero::AnimFieldEdit,
    },

    /// Inspector → shell channel for a §10 Material & Blend field (Blend
    /// Mode). Optional-component edit like [`Self::InspectorSamplingEdit`]
    /// (§3.10); tag `0` (Mix) detaches the `BlendMode` component.
    InspectorBlendEdit {
        entity_bits: u64,
        edit: crate::screens::hero::BlendFieldEdit,
    },

    /// Inspector → shell channel for a §11 Physics Body field (ADR-0131 D8).
    /// Optional-component edit like [`Self::InspectorBlendEdit`], but the
    /// pair it attaches/detaches is `RigidBody` + `Collider`, and `Add`
    /// deliberately carries no geometry — the shell derives the starting
    /// collider from the sprite's own bounds.
    InspectorPhysicsEdit {
        entity_bits: u64,
        edit: crate::screens::hero::PhysicsFieldEdit,
    },

    /// ⭐ **O `+` do Inspector foi carregado** — abra a paleta de componentes para este objeto
    /// (ADR-0166 / plano F3).
    ///
    /// ⚠️ **Um PEDIDO, não uma edição.** O painel não sabe que componentes existem (o catálogo
    /// vive numa crate-folha e o registo vive na shell), e a paleta precisa dos dois para saber o
    /// que oferecer: o TIPO do objeto, o que ele já tem, e o que o registo sabe construir. Quem
    /// tem essas três respostas é a shell — o painel só diz *quem* perguntou.
    InspectorAddComponentRequested {
        entity_bits: u64,
    },

    /// ⭐ **Limpar as excepções SEM ALVO de uma instância** (ADR-0164 / F5.3).
    ///
    /// `root_bits` é a RAIZ da instância — o `ObjectInstance` mora lá, e uma peça não sabe
    /// quantos órfãos a cópia inteira tem (um órfão não TEM peça).
    ///
    /// ⛔ Existe porque eles **nunca** se apagam sozinhos (a lei do *«unused overrides»* do
    /// Unity): sair por causa de um `Delete` no mestre é perder trabalho do artista em
    /// silêncio. ⇒ o gesto é explícito, e é este.
    InspectorClearUnusedOverrides {
        root_bits: u64,
    },

    /// ⭐⭐⭐ **Largar UMA excepção sem alvo** (ADR-0164 / F5.3-ter) — o `✕` da linha dela.
    ///
    /// ⚠️ **A irmã acima apaga TODAS, e ter só ela era o gesto destrutivo mais barato deste
    /// painel:** desde 2026-09-04 o cartão diz *quais* são, e um artista que quer largar uma de
    /// cinco tinha de largar as cinco. *Uma lista que se lê item a item pede um gesto item a item.*
    ///
    /// ⚠️ **A chave, e nunca o índice da linha nem o rótulo:** `piece` é o `StableId` da peça morta
    /// e `type_id` o do componente. Duas peças podem ter tido o mesmo nome, e o cartão é
    /// reconstruído a cada quadro — um índice diria *«a terceira»* a um cartão que já tem outra
    /// terceira.
    InspectorDropUnusedOverride {
        /// A RAIZ da instância — é lá que o `ObjectInstance` mora.
        root_bits: u64,
        /// O `StableId` da peça que morreu.
        piece: u64,
        /// O `type_id` do componente cuja excepção se larga.
        type_id: u64,
    },

    /// ⭐⭐⭐ **APLICAR num DEGRAU da escada** (ADR-0164 / F5, critério 4).
    ///
    /// `entity_bits` é a PEÇA em que o artista carregou — o escopo do gesto é o que se clicou, como
    /// no *Revert* —, e `master` é o `StableId` da receita escolhida: **a identidade, nunca o
    /// índice do botão**, porque a escada é derivada e reordena-se quando a cena muda.
    ///
    /// ⛔ Ela existe porque *«aplicar ao mestre»* **não tem resposta por omissão** quando há
    /// receitas aninhadas — é a razão pela qual a `PrefabUtility.ApplyPropertyOverride` do Unity
    /// exige o `assetPath` (*«multiple valid targets may exist»*).
    InspectorApplyToLevel {
        entity_bits: u64,
        master: u64,
    },

    /// ⭐⭐⭐ **Trocar a VARIANTE de uma instância** (ADR-0164 / F5, critério 2).
    ///
    /// `master` é o `StableId` do mestre novo — **a identidade, nunca o índice do chip**: a lista
    /// é reconstruída por quadro, e uma posição que reordene entre o pintar e o clicar escolhe a
    /// versão errada sem erro nenhum.
    ///
    /// ⚠️ A shell é quem sabe se os dois mestres são aparentados; o painel só oferece o que o
    /// construtor já filtrou pela MESMA pergunta.
    /// ⭐⭐⭐ **O que a HIERARQUIA pede** — as 33 formas numa família só.
    /// Porquê e cada uma delas: [`HierRequest`].
    Hierarchy(HierRequest),

    /// ⭐⭐ **Mostrar a biblioteca** — *«o que é que eu posso pôr aqui?»*, o clique na ranhura da
    /// textura. Porquê: `crates/ph2d-panel-inspector/src/populate.rs`.
    OpenAssetBrowser,

    InspectorSwapVariant {
        root_bits: u64,
        master: u64,
    },

    /// Inspector → shell channel for a §12 Physics Joint field (W3). The
    /// `entity_bits` are the JOINT object's, not a body's — a joint is an
    /// entity, and this section describes it.
    InspectorJointEdit {
        entity_bits: u64,
        edit: crate::screens::hero::JointFieldEdit,
    },
    /// ⭐⭐ **Renomear o VALOR de uma propriedade** — o campo que o clique no chip aceso abre.
    /// O sujeito é a **RECEITA**, porque é lá que o valor vive; o porquê vive no
    /// `ph2d-panel-inspector/src/event_value.rs`.

    /// Inspector → shell channel for a §13 Pulley Wheel field (W-Pulley W1).
    /// The `entity_bits` are the WHEEL object's — a roldana é uma entidade, e
    /// esta seção descreve ELA, não a corda que a atravessa.
    InspectorWheelEdit {
        entity_bits: u64,
        edit: crate::screens::hero::WheelFieldEdit,
    },

    /// Inspector → shell channel para um campo da §14 Platform Player (W5).
    ///
    /// Sem fan-out, e pela razão da §12: a seção descreve UM personagem, o que
    /// está selecionado. Espalhar um `Add` pela seleção criaria N players num
    /// clique que pediu um.
    InspectorPlayerEdit {
        entity_bits: u64,
        edit: crate::screens::hero::PlayerFieldEdit,
    },

    /// Config → "Image filter" pick. Payload: the chosen
    /// [`ImageFilterMode`]. The hero already wrote
    /// `project.image_filter` (so the menu checkmark is correct on the
    /// next paint); this round-trips the change to the shell, which
    /// owns the GPU sampler state and calls
    /// `SpriteRenderer::set_filter_mode(mode)` to rebuild the atlas +
    /// individual samplers and their bind groups. The shell also stores
    /// the mode so the per-frame BG-Removal Vello preview picks the
    /// matching `peniko::ImageQuality`. Raised by clicking a row in the
    /// `SettingsFilterSubmenu`.
    SetImageFilter {
        mode: crate::project::ImageFilterMode,
    },

    /// Config → "Display" present-mode pick. `vsync = true` → `Fifo`
    /// (smooth, hardware-paced motion); `vsync = false` → `Immediate`
    /// (non-blocking, kills the mouse-move stutter at the cost of
    /// vsync pacing). The shell owns the swap chain and calls
    /// `SurfaceContext::set_present_mode`. Raised by clicking a row in
    /// the `SettingsDisplaySubmenu`.
    SetPresentMode {
        vsync: bool,
    },

    /// Inspector → shell channel for entity-`Name` edits. Payload:
    /// the snapshot `(entity_bits, new_name)`. Shell drains and
    /// pushes a `EditorCommand::SetComponent` for `ph2d_ecs::Name`,
    /// same pipeline as Transform / Visibility. Raised by
    /// `TextChanged` on `INSP_ENTITY_NAME`.
    InspectorNameEdit(crate::screens::hero::InspectorNameInfo),
    /// **O nome do sinal que este objeto emite quando algo chega nele** (W-Signal).
    ///
    /// Reusa o `InspectorNameInfo` porque a carga é a MESMA — uma entidade e uma
    /// string —, e um tipo novo idêntico ao existente seria um segundo formato
    /// para a mesma pergunta.
    InspectorSignalEdit(crate::screens::hero::InspectorNameInfo),
    /// **O nome do sinal que este objeto emite quando algo SAI dele**
    /// (W-SignalLeave) — o gêmeo exato do irmão acima.
    ///
    /// ⚠️ **Uma AÇÃO própria, e não um `leave: bool` na de cima.** As duas rows
    /// escrevem componentes diferentes, então um flag obrigaria o dreno a
    /// perguntar duas coisas para saber uma — e a primeira vez que alguém
    /// esquecesse de o ler, o nome de saída pousaria no componente de chegada e
    /// a porta abriria ao fechar.
    InspectorSignalLeaveEdit(crate::screens::hero::InspectorNameInfo),

    /// Transport control from the TopBar Play/Pause/Reset chips
    /// (`chrome::transport`). The shell owns the ONE clock
    /// (`ph2d_core::Playhead`, W4.T7), so the chrome handler cannot touch it
    /// directly — it raises this, and the shell drains it into the playhead.
    /// Physics, Motion, Timeline and Flip all ride the same clock, so these
    /// three chips drive every time-based subsystem at once.
    Transport(TransportCmd),

    /// ⭐⭐ **Instanciar a partir da biblioteca**, no ponto de mundo `at`. ⚠️ Irmã da
    /// `HierInstantiate` e NÃO a mesma: ali chega uma `row`, aqui um `StableId`. A lei da
    /// queda vive em `shells/desktop/src/asset_drop.rs`.
    AssetInstantiate {
        /// O `StableId` da raiz da receita.
        stable_id: u64,
        /// ⭐⭐ **ONDE**, em coordenadas de MUNDO — `None` = na cascata de sempre.
        ///
        /// ⚠️ **É o MESMO verbo do duplo-clique**, e a única diferença é esta: um duplo-clique não
        /// aponta para lado nenhum (a cascata é a resposta honesta), e uma **queda** aponta. Uma
        /// segunda variante para o arrasto daria dois caminhos para instanciar, e eles divergiriam
        /// no dia em que o verbo ganhasse um passo.
        at: Option<[f32; 2]>,
    },
    /// ⭐⭐ **UM VERBO DE CATÁLOGO** (plano 07, wave A3).
    ///
    /// ⚠️ **O id é um `u128` CRU, e não o `CatalogId`:** este barramento é chrome, e ele não
    /// conhece o modelo de assets — a mesma cerca que fez o [`Self::AssetCardVerb`] carregar um
    /// `DragPayload` em vez de um `AssetRef`. Quem o interpreta é o shell.
    AssetCatalogVerb(CatalogVerb),
    AssetCardVerb {
        /// O endereço do asset no vocabulário de chrome — ver
        /// [`crate::interaction::drag_payload::DragPayload`], que já existia para o dizer sem esta
        /// camada aprender o modelo de assets.
        asset: crate::interaction::drag_payload::DragPayload,
        /// Qual dos três.
        verb: AssetCardAction,
    },
}

/// ⚠️ A **fila** vive no irmão [`super::action_bus_queue`] e é re-exportada aqui: quem escreve
/// `action_bus::ActionBus` continua a escrevê-lo. Ver o cabeçalho de lá para o porquê do corte.
pub use super::action_bus_queue::ActionBus;

/// ⚠️ Os **vocabulários** que as acções carregam vivem no irmão [`super::action_bus_kinds`] e são
/// re-exportados aqui: quem escreve `action_bus::TransportCmd` continua a escrevê-lo. Ver o
/// cabeçalho de lá para o porquê do corte.
pub use super::action_bus_kinds::{AssetCardAction, CatalogVerb, SelectModifier, TransportCmd};

/// ⚠️ A família da **Hierarquia** vive no irmão [`super::action_bus_hier`] e é re-exportada aqui:
/// quem escreve `action_bus::HierRequest` continua a escrevê-lo.
pub use super::action_bus_hier::HierRequest;

#[cfg(test)]
#[path = "action_bus_tests.rs"]
mod tests;
