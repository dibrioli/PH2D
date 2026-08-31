//! `ph2d-ui-testkit` — headless harness to drive a `Panel` + `Tool` seam
//! in a unit test, **without** the desktop shell or a GPU.
//!
//! ## Why this exists (blindagem Fase 0.1)
//!
//! The 2026-06-20 forensic diagnosis found the recurring "green-but-dead"
//! bug class: a widget that PAINTS, REGISTERS and COMPILES but is silently
//! inert because one of the ~13 hand-wired sites between the panel
//! (`WidgetStore` ids + `apply_event`) and the tool (`handle_panel_event`
//! → `apply_ui_edit`) was forgotten. Unit tests on the tool stay green
//! (they call `apply_ui_edit` directly) and the `*_contract_surface` gates
//! stay green (they count symbols, not behavior). Nobody finds the dead
//! wire until a human clicks it.
//!
//! [`MockPanelHost`] closes that gap: it is a real `PanelHostInternal`
//! backed by a real [`WidgetStore`] + [`ActionBus`], so a test can run the
//! FULL path the shell runs —
//!
//! ```text
//! P::populate(store)               // boot registration
//!   → set the widget's stored value (what a drag does)
//!   → P::apply_event(state, host, WidgetEvent::ValueChanged(id))   // panel
//!   → host.drained_actions()       // what the shell drains each frame
//!   → tool.handle_panel_event(pe)  // shell forwards ToolPanelEvent
//!   → assert tool.<observable state> changed
//! ```
//!
//! If ANY site in that chain is missing, the assertion goes red. That is
//! the behavioral signal the project was missing.
//!
//! ## Placement note
//!
//! This crate is deliberately **not** named `ph2d-panel-*` / `ph2d-tool-*`
//! / `ph2d-node-*`: those prefixes are swept by the registry codegen and
//! the LOC-cap gate's `collect_panel_dirs`. A test-only crate with one of
//! those prefixes would be mis-registered (the `node-sync glob` gotcha).
//! Consume it from a panel/tool crate's `[dev-dependencies]` — the
//! `architecture_cycle_prevention` gate reads `[dependencies]` only, so a
//! dev-dep edge builds no runtime cycle.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::interaction::dispatch::keymap::KEY_ENTER;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::interaction::{dispatch_key, dispatch_pointer, dispatch_text_input};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHost, PanelHostInternal};
use ph2d_editor_core::project::ProjectSettings;
use ph2d_editor_core::screens::slot::SlotSet;
use ph2d_editor_core::screens::{HeroLayout, HeroSelection};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// One second of the synthetic pointer clock — the gap [`MockPanelHost::click_at`] leaves
/// between clicks so the dispatcher never mistakes two of them for a double-click.
const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// A throwaway [`PanelHostInternal`] for tests. Holds the real widget
/// store + action bus a panel reads/writes; the grid-snap / selection /
/// visibility surfaces are present (the trait requires them) but inert —
/// most seam tests only touch `store()` / `store_mut()` / `bus_mut()`.
pub struct MockPanelHost {
    store: WidgetStore,
    /// O relógio da UI viva. ⚠️ Nasce VAZIO de propósito: com o mapa vazio o
    /// `PanelHostInternal::button_visual` devolve o neutro `1.0`, e um gate de painel continua a
    /// medir exactamente o que media antes da UI viva existir. Quem quer exercitar o `t` tica-o.
    hit_index: HitIndex,
    bus: ActionBus,
    selection: Option<HeroSelection>,
    grid_snap: GridSnapState,
    grid_snap_panel_rect: Option<Rect>,
    project: ProjectSettings,
    theme: Theme,
    visible: BTreeMap<String, bool>,
    /// Monotonic clock for the synthetic pointer events ([`MockPanelHost::click_at`]). A
    /// counter, not a wall clock: the dispatcher's double-click window is a real threshold,
    /// so the gap between clicks has to be deterministic or the gate flakes.
    clock_ns: u128,
}

impl MockPanelHost {
    /// An empty host: no widgets registered yet. Use [`Self::with_panel`]
    /// to also run a panel's `populate`.
    pub fn new() -> Self {
        Self {
            store: WidgetStore::with_capacity(32),
            hit_index: HitIndex::new(),
            bus: ActionBus::new(),
            selection: None,
            grid_snap: GridSnapState::default(),
            grid_snap_panel_rect: None,
            project: ProjectSettings::default(),
            theme: Theme::default(),
            visible: BTreeMap::new(),
            clock_ns: 0,
        }
    }

    /// A host with panel `P`'s widgets pre-registered (runs `P::populate`,
    /// the same boot step the host orchestrator runs once).
    ///
    /// ⚠️ **Esta é METADE do que o app faz.** Veja [`Self::with_panel_and_shared_chrome`] antes de
    /// escrever um gate que pergunte *"este id está registado?"*.
    pub fn with_panel<P: Panel>() -> Self {
        let mut host = Self::new();
        P::populate(&mut host.store);
        host
    }

    /// **O host com as DUAS metades que o app de facto popula** — o chrome partilhado *e* o painel.
    ///
    /// # Por que existe
    ///
    /// O `HeroScreen::new` popula o store a partir de **duas** fontes
    /// ([`hero.rs:278`](../../ph2d-editor-core/src/screens/hero.rs) → `pre_populate_store`):
    /// o chrome partilhado (`pre_populate::populate_shared` — as marcas de seção colapsável, os
    /// pontos de cor, as linhas de menu) **e** o `Panel::populate` de cada painel. O
    /// [`Self::with_panel`] só corre a segunda.
    ///
    /// ⚠️ **Um gate montado só sobre a segunda mede um store que o app nunca tem** — e responde
    /// «não registado» a ids que estão registadíssimos, o que o faz acusar o legítimo e ser
    /// desligado. Foi exatamente o que aconteceu quando a varredura
    /// `every_painted_id_is_reachable` nasceu (2026-08-21): ela acusou 22 ids, e **14 deles eram
    /// falso-positivo desta lacuna** — incluindo cabeçalhos que funcionam há meses.
    ///
    /// Use este construtor sempre que a pergunta do teste envolver **registo**; o
    /// [`Self::with_panel`] chega para perguntas que só tocam nos widgets do próprio painel.
    pub fn with_panel_and_shared_chrome<P: Panel>() -> Self {
        let mut host = Self::new();
        ph2d_editor_core::screens::hero::pre_populate::populate_shared(&mut host.store);
        P::populate(&mut host.store);
        host
    }

    /// Drive one event through `P::apply_event` — the exact entry point the
    /// host dispatcher uses. Returns the panel's [`EventOutcome`].
    pub fn apply_panel_event<P: Panel>(
        &mut self,
        state: &mut P::State,
        ev: WidgetEvent,
    ) -> EventOutcome {
        P::apply_event(state, self, ev)
    }

    /// Set a registered slider's stored value — what a pointer drag writes
    /// into the store *before* the dispatch emits `ValueChanged(id)`. Panics
    /// (never silently no-ops) if `id` is absent or not a slider.
    /// Read-only view of the widget store — so a seam test can assert on the state a
    /// panel's `populate` seeded (collapsed sections, default values, …), not just on
    /// what an event did to it. Additive: nothing in the testkit changes shape.
    pub fn store(&self) -> &WidgetStore {
        &self.store
    }

    /// Rola um painel — o que a roda do mouse escreve no store antes da pintura seguinte.
    ///
    /// ⚠️ Um método NOMEADO em vez de um `store_mut()` genérico: a segunda forma seria uma porta
    /// aberta para um gate escrever qualquer coisa no store e depois "provar" o que ele mesmo
    /// semeou. Esta responde a uma pergunta só — *e se o artista rolar?* —, que é o que separa
    /// *onde um botão está* de *se o artista chega lá*.
    pub fn set_panel_scroll(&mut self, panel: NodeId, y: f32) {
        self.store.set_panel_scroll(panel, y);
    }

    /// **O relógio de movimento correu até ao fim** — toda dobra de secção salta para o seu
    /// alvo semântico (`is_collapsed` ⇒ 0, senão 1).
    ///
    /// ⚠️ Existe porque a F4b fez o CORPO de uma secção interpolar: depois de um clique no
    /// cabeçalho o flag semântico já virou, mas o `t` ainda desce, e o painel de um harness
    /// headless — que não tem o tique do `HeroScreen` — nunca o veria chegar a zero. Sem esta
    /// porta um gate de dobra afirmaria *"a row sumiu"* sobre um produto que a está a esconder
    /// **gradualmente**, e reprovaria a animação em vez de a medir.
    ///
    /// ⚠️ Método NOMEADO, nunca um `store_mut()`: ele responde a UMA pergunta — *e se o artista
    /// esperar a animação acabar?* — em vez de abrir o store para um gate semear o que depois
    /// vai "provar" (o mesmo argumento do [`Self::set_panel_scroll`]).
    pub fn settle_section_folds(&mut self) {
        for id in self.store.collapsible_ids() {
            let target = if self.store.is_collapsed(id) {
                0.0
            } else {
                1.0
            };
            self.store.set_section_open_live(id, target);
        }
    }

    pub fn set_slider_value(&mut self, id: NodeId, value: f32) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Slider { value: v, .. }) => *v = value,
            Some(_) => panic!("set_slider_value: {id:?} is registered but is not a Slider"),
            None => panic!("set_slider_value: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Set a registered number chip's committed value. Panics if `id` is
    /// absent or not a `NumberInput`.
    pub fn set_number_value(&mut self, id: NodeId, value: f64) {
        match self.store.get_mut(id) {
            Some(InteractiveState::NumberInput { value: v, .. }) => *v = value,
            Some(_) => panic!("set_number_value: {id:?} is registered but is not a NumberInput"),
            None => panic!("set_number_value: {id:?} is not registered (did populate run?)"),
        }
    }

    /// **DIGITAR de verdade num chip numérico**, pelos dispatchers REAIS: foco, um
    /// `dispatch_text_input` por caractere, e Enter. Devolve os `WidgetEvent` que o commit
    /// emitiu (tipicamente `ValueChanged(id)`), para o chamador entregá-los ao painel.
    ///
    /// ⚠️ **Por que o testkit precisava disto, e é o achado que o motivou:** [`set_number_value`]
    /// ESCREVE o valor no store e pula o commit inteiro — `apply_chip_value_with_mirror`, que é
    /// onde o espelho chip↔slider decide o que sobrevive. Toda a família de gates do range do
    /// `motion-params` usava o setter, então nenhum deles jamais exercitou a camada onde o valor
    /// digitado morria, e os três ficaram VERDES enquanto o produto capava a caixa no máximo do
    /// SLIDER (Enio, smoke de 2026-08-07: *"Máximo de 20 em grid"*).
    ///
    /// ⚠️ **PINTE o painel antes de chamar isto.** A faixa do chip (`set_number_range`) e o link
    /// com o slider nascem no `paint`; sem ele o chip não tem régua nem espelho, e o teste passa
    /// a medir uma fixture que o produto não tem.
    ///
    /// Panics se `id` não estiver registrado ou não for um `NumberInput`.
    pub fn type_into_number(&mut self, id: NodeId, text: &str) -> Vec<WidgetEvent> {
        match self.store.get_mut(id) {
            Some(InteractiveState::NumberInput { buffer, caret, .. }) => {
                buffer.clear();
                *caret = 0;
            }
            Some(_) => panic!("type_into_number: {id:?} is registered but is not a NumberInput"),
            None => panic!("type_into_number: {id:?} is not registered (did populate run?)"),
        }
        self.store.set_focus(Some(id));
        let arena = Bump::new();
        for ch in text.chars() {
            let _ = dispatch_text_input(&mut self.store, ch, &arena);
        }
        let key = ph2d_host::KeyEvent {
            keycode: KEY_ENTER,
            modifiers: ph2d_host::Modifiers {
                shift: false,
                ctrl: false,
                alt: false,
                meta: false,
            },
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        dispatch_key(&mut self.store, key, &arena).to_vec()
    }

    /// Set a registered text input's buffer — what a real keystroke would have
    /// left there before dispatch emits `TextChanged(id)`. Panics if `id` is
    /// absent or not a `TextInput`.
    ///
    /// **Por que o testkit precisa disto:** um arm de `TextChanged` LÊ o buffer
    /// (`host.store().text(id)`) em vez de receber a string no evento, então um
    /// seam que só despacha o evento testaria sempre a string vazia — e um arm
    /// que mandasse o texto errado passaria. O caret vai para o fim, como depois
    /// de digitar.
    pub fn set_text(&mut self, id: NodeId, value: &str) {
        match self.store.get_mut(id) {
            Some(InteractiveState::TextInput { text, caret, .. }) => {
                text.clear();
                text.push_str(value);
                *caret = text.len();
            }
            Some(_) => panic!("set_text: {id:?} is registered but is not a TextInput"),
            None => panic!("set_text: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Set a registered toggle's stored on-state — what the paint pass mirrors
    /// from the snapshot before dispatch emits `Toggled(id)`. Panics if `id` is
    /// absent or not a `Toggle`.
    pub fn set_toggle_on(&mut self, id: NodeId, on: bool) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Toggle { on: o, .. }) => *o = on,
            Some(_) => panic!("set_toggle_on: {id:?} is registered but is not a Toggle"),
            None => panic!("set_toggle_on: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Mutable access às definições do projeto — unidade de leitura, `pixels_per_meter`, snaps.
    ///
    /// **Porque é que o testkit precisava disto:** havia `project()` (leitura) e não havia o par.
    /// A conversão px↔m do Inspector lê `host.project().display_unit` e `pixels_per_meter`, por
    /// isso um teste de costura só conseguia exercitá-la no default (`Meters`, onde a conversão é
    /// a identidade) — ou seja, **só na metade em que ela não faz nada**. Os dois testes que
    /// provavam o round-trip em pixels estão desligados desde 2026-06, e esta ausência é a razão
    /// mecânica: não havia porta por onde pôr o projeto em `Pixels`.
    pub fn project_mut(&mut self) -> &mut ProjectSettings {
        &mut self.project
    }

    /// Set a registered **checkbox**'s stored value — the sibling of [`Self::set_toggle_on`] for
    /// the other of the two boolean widgets. Panics if `id` is absent or not a `Checkbox`.
    ///
    /// ⚠️ **Toma o VALOR, não um `bool`, de propósito.** `CheckboxValue` tem três estados, e o
    /// terceiro — `Indeterminate` — é a affordance de *«Mixed»* que uma seleção múltipla com
    /// valores divergentes pinta. Uma porta que só aceitasse `bool` tornaria esse estado
    /// inalcançável a todo teste de costura, que é precisamente como ele passou a existir no
    /// painter sem uma única afirmação a defendê-lo.
    ///
    /// **Porque é que o testkit precisava disto:** havia `set_toggle_on` e não havia o par. Os
    /// checkboxes da sprite (Flip H/V, Centered, Tint Fill, Region…) são `Checkbox`, não `Toggle`
    /// — e por isso a família inteira de `InspectorSpriteEdit` era, na prática, **inalcançável**
    /// por um teste de costura. Vinte e uma variantes chegaram a 2026-08 com zero afirmações
    /// vivas, e esta ausência é metade da razão.
    pub fn set_checkbox_value(
        &mut self,
        id: NodeId,
        value: ph2d_editor_core::widget::CheckboxValue,
    ) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Checkbox { value: v, .. }) => *v = value,
            Some(_) => panic!("set_checkbox_value: {id:?} is registered but is not a Checkbox"),
            None => panic!("set_checkbox_value: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Set a registered dropdown's open state — what the generic dispatcher writes when
    /// the user clicks a chip. Panics if `id` is absent or not a `Dropdown`.
    ///
    /// **Why the testkit needs this at all:** the open/close of a dropdown is done by the
    /// SHELL's generic dispatch, not by the panel's `apply_event` — so a seam test driving
    /// `apply_event` alone cannot reach the state *"this popover is open"*, and any rule
    /// about it (e.g. *opening one closes the other*) would be untestable at this seam.
    pub fn set_dropdown_open(&mut self, id: NodeId, open: bool) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Dropdown { open: o, .. }) => *o = open,
            Some(_) => panic!("set_dropdown_open: {id:?} is registered but is not a Dropdown"),
            None => panic!("set_dropdown_open: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Read a registered dropdown's open state (the mirror of
    /// [`Self::set_dropdown_open`], so a gate can assert on it).
    #[must_use]
    pub fn dropdown_is_open(&self, id: NodeId) -> Option<bool> {
        match self.store.get(id) {
            Some(InteractiveState::Dropdown { open, .. }) => Some(*open),
            _ => None,
        }
    }

    /// Drain everything the panel pushed onto the action bus so far. The
    /// shell does the same each frame; tests inspect the result to assert
    /// the panel actually emitted the right [`EditorAction`].
    pub fn drained_actions(&mut self) -> Vec<EditorAction> {
        self.bus.drain().collect()
    }

    /// **Run panel `P`'s REAL paint pass, headless, and return what it made
    /// clickable** — every `(id, rect)` the paint registered in the hit index.
    ///
    /// This closes the last hole in the "green-but-dead" family. The seam test
    /// above proves `populate → apply_event → tool`; the wiring-parity gate
    /// reads *source text*. **Neither one runs `paint`.** So a widget can be
    /// registered, wired, unit-tested and contract-clean while its paint call
    /// sits behind an early `return` (or was never written at all) — and the
    /// user's report is simply *"the button doesn't exist"*, with every gate
    /// green. What a user can click is what the PAINT registered, so that is
    /// what a test must read.
    ///
    /// The panel is forced visible first (a paint gated on `panel_visible`
    /// would otherwise return before drawing anything).
    pub fn paint<P: Panel>(&mut self, state: &mut P::State, viewport: Rect) -> Vec<(NodeId, Rect)> {
        self.paint_with_layout::<P>(state, HeroLayout::for_viewport(viewport), viewport)
    }

    /// [`Self::paint`], but with the layout given explicitly.
    ///
    /// `for_viewport` builds an UNSPLIT centre (`CenterSplit::None`), and a panel that lives
    /// in the split — the Motion graph, the Motion timeline slot — then gets a **zero-sized
    /// rect and returns before drawing anything**. Every paint gate for those panels was
    /// therefore impossible to write, which is why none existed: the add-menu could be
    /// unclickable in the running app with every test in the crate green (Enio, smoke
    /// 2026-07-13). A harness that cannot lay a panel out cannot test what the artist clicks.
    /// **Run panel `P`'s paint pass with the panel HIDDEN** — the housekeeping half.
    ///
    /// ⚠️ Every other paint helper here **forces the panel visible**, for a good reason
    /// (a paint gated on `panel_visible` would return before drawing anything, and these
    /// helpers exist to read what it drew). The consequence, measured by the audit of
    /// 2026-07-29 (§4 D-K): **no gate in this repo could exercise a panel's hidden
    /// branch** — and that branch is not empty. It is where a panel drops its stale rects,
    /// its in-flight gestures, its published flags and (now) its live preview channel. A
    /// panel that forgets one of those keeps driving the app with nothing on screen to
    /// stop it, which is exactly the defect that produced this method.
    ///
    /// Returns nothing, because a hidden panel registers nothing: what a gate asserts
    /// after calling this is the state it *dropped*, not the rects it *made*.
    pub fn paint_hidden<P: Panel>(&mut self, state: &mut P::State, viewport: Rect) {
        self.set_panel_visible(P::ID, false);
        self.hit_index.clear_for_frame();
        let layout = HeroLayout::for_viewport(viewport);
        let mut scene = VectorScene::new();
        let mut text_system = TextSystem::without_system_fonts();
        let mut ctx = PaintCtx {
            host: self,
            layout: &layout,
            // ⚠️ **O arnês entrega o encaixe que o painel DECLARA.** Ele não corre o hero, logo não
            // há mapa de excepções nem faixa de abas — e é isso que se quer aqui: um teste de
            // painel mede o painel na casa dele, não uma arrumação que o artista fez.
            //
            // ⛔⛔ **`SlotSet::of(…)`, NUNCA `ANY_DOCK`.** O `ANY_DOCK` contém as DUAS metades de
            // cada coluna, e a lei do `slot_rects` é *«a metade só existe quando a irmã está
            // ocupada»* ⇒ pedir `ANY_DOCK` **parte a coluna ao meio**. Medido: o corpo do painel do
            // Motion caiu para `346 px` e 26 nós passaram a «transbordar» sem uma linha de produto
            // se mexer. *Um conjunto de ocupação não é uma lista de sítios possíveis: é quem lá
            // está.*
            slot: layout
                .slot_rects(SlotSet::of(P::DEFAULT_SLOT))
                .get(P::DEFAULT_SLOT),
            viewport,
            scene: &mut scene,
            text_system: &mut text_system,
        };
        P::paint(state, &mut ctx);
    }

    /// ⭐⭐⭐ **QUANTA GEOMETRIA O PAINEL DE FACTO EMITIU** — `(n_paths, n_path_segments)` da
    /// cena Vello, depois de pintar `P`.
    ///
    /// ⛔⛔ **Ela existe por causa do achado §4.2 da auditoria do `source.lsystem`:** o gate que
    /// prometia medir *«a queixa chega a PIXEL»* media o `content_h` — a **linha reservada** —,
    /// escrito por um `y +=` que não é a pintura. *Apagar a pintura inteira deixava-o verde.*
    /// A auditoria nomeou o buraco no arnês: *«`MockPanelHost` expõe `store()`, `paint()` e
    /// `painted_rect()`, e nada de texto»*.
    ///
    /// ⚠️⚠️ **O que conta TEXTO é o número de GLIFOS, não os caminhos.** A 1.ª redacção desta
    /// função devolvia `n_path_segments`, e o gate que a estreou leu **`42` contra `42`**: o
    /// Vello encaminha texto por `draw_glyphs`, cuja saída vive em `resources.glyphs`, e
    /// **nenhum glifo entra na contagem de caminhos**. *Uma régua que devolve o mesmo número dos
    /// dois lados não distingue «não pintou» de «não vejo o que ele pintou».*
    ///
    /// ⚠️ E o `TextSystem` deste arnês é `without_system_fonts` — ele **empacota o Inter**, então
    /// há glifos; mas quem usa isto deve medir também que texto MAIOR dá mais glifos, que é a
    /// metade que prova que o balde se enche.
    ///
    /// Devolve `(glifos, segmentos de caminho)`.
    pub fn paint_and_count_geometry<P: Panel>(
        &mut self,
        state: &mut P::State,
        viewport: Rect,
    ) -> (u32, u32) {
        self.set_panel_visible(P::ID, true);
        self.hit_index.clear_for_frame();
        let layout = HeroLayout::for_viewport(viewport);
        let mut scene = VectorScene::new();
        let mut text_system = TextSystem::without_system_fonts();
        {
            let mut ctx = PaintCtx {
                host: self,
                layout: &layout,
                viewport,
                scene: &mut scene,
                text_system: &mut text_system,
            };
            P::paint(state, &mut ctx);
        }
        let enc = scene.inner().encoding();
        (
            u32::try_from(enc.resources.glyphs.len()).unwrap_or(u32::MAX),
            enc.n_path_segments,
        )
    }

    pub fn paint_with_layout<P: Panel>(
        &mut self,
        state: &mut P::State,
        layout: HeroLayout,
        viewport: Rect,
    ) -> Vec<(NodeId, Rect)> {
        self.set_panel_visible(P::ID, true);
        self.hit_index.clear_for_frame();
        let mut scene = VectorScene::new();
        let mut text_system = TextSystem::without_system_fonts();
        {
            let mut ctx = PaintCtx {
                host: self,
                layout: &layout,
                slot: layout
                    .slot_rects(SlotSet::of(P::DEFAULT_SLOT))
                    .get(P::DEFAULT_SLOT),
                viewport,
                scene: &mut scene,
                text_system: &mut text_system,
            };
            P::paint(state, &mut ctx);
        }
        self.hit_index.iter_registrations().collect()
    }

    /// **Drive a real pointer event through the real dispatcher**, against the hit index the
    /// last [`Self::paint`] filled in.
    ///
    /// This is the half that hand-pushed gestures skip — and therefore the half where a
    /// regression hides: the dispatcher decides whether a click on a popup becomes a widget
    /// event, a graph gesture, or nothing at all, purely from what the PAINT registered. A
    /// test that pushes the gesture itself has already assumed the answer.
    ///
    /// Returns the widget events the dispatch emitted (a graph gesture is not one of them —
    /// it lands in the store, for the panel's next `paint` to drain).
    pub fn dispatch_pointer_event(&mut self, event: ph2d_host::PointerEvent) -> Vec<WidgetEvent> {
        let arena = bumpalo::Bump::new();
        // `_with_text` — the variant the SHELL uses (it threads a live `TextSystem` so a click
        // into a text field lands the caret on the right glyph). A harness that dispatched the
        // no-text variant would be testing a path the app does not take.
        let mut ts = TextSystem::without_system_fonts();
        ph2d_editor_core::interaction::dispatch_pointer_with_text(
            &mut self.store,
            &self.hit_index,
            event,
            Some(&mut ts),
            &arena,
        )
        .to_vec()
    }

    /// Sugar over [`Self::paint`] for the common assertion: *is this widget on
    /// screen and clickable?* A widget painted with a degenerate (zero-area)
    /// rect is NOT clickable, so it does not count as painted.
    pub fn painted_rect<P: Panel>(
        &mut self,
        state: &mut P::State,
        viewport: Rect,
        id: NodeId,
    ) -> Option<Rect> {
        self.paint::<P>(state, viewport)
            .into_iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| r)
    }

    /// **Drive a REAL pointer click at `(x, y)`** — Down then Up — through the same
    /// [`dispatch_pointer`] the shell runs, over the hit index the last [`Self::paint`] built.
    /// Returns the `WidgetEvent`s the dispatcher emitted (feed them to `apply_panel_event`).
    ///
    /// ## Why this exists (the last hole in the "green-but-dead" family)
    ///
    /// [`Self::paint`] proves a widget REGISTERS A HIT RECT and [`Self::apply_panel_event`]
    /// proves the panel FORWARDS an event it is handed. Neither proves a POINTER on that rect
    /// ever becomes that event — and it does not, unless the id also carries an
    /// `InteractiveState` in the store: `dispatch_pointer`'s Down only makes a hit `active`
    /// when it is *focusable*, and an id absent from the store is not. So a widget can paint,
    /// hit-register, forward and route — every gate green — and still be **stone dead under
    /// the mouse** because `populate` never registered it. That is not a hypothetical: it is
    /// the Impasto light rig (Enio 2026-07-12, *"nem o checkbox nem se pode selecionar outra
    /// luz"*), and before it the hierarchy companions.
    ///
    /// A widget is not done when it paints. It is done when a test CLICKS it.
    pub fn click_at(&mut self, x: f32, y: f32) -> Vec<WidgetEvent> {
        let mut out = Vec::new();
        for kind in [PointerKind::Down, PointerKind::Up] {
            // Space the clicks a full second apart: inside the double-click window the
            // dispatcher would upgrade the second Click to a DoubleClick and the assertion
            // would fail for a reason that has nothing to do with the seam under test.
            self.clock_ns += NANOS_PER_SECOND;
            let arena = Bump::new();
            let event = PointerEvent {
                x,
                y,
                pressure: 1.0,
                kind,
                source: PointerSource::Mouse,
                button: PointerButton::Primary,
                timestamp_ns: self.clock_ns,
            };
            out.extend_from_slice(dispatch_pointer(
                &mut self.store,
                &self.hit_index,
                event,
                &arena,
            ));
        }
        out
    }

    /// **Drive a REAL pointer DRAG** from `(x0, y0)` to `(x1, y1)` — Down, Move, Up — through the same
    /// [`dispatch_pointer`] the shell runs.
    ///
    /// [`Self::click_at`] proves a widget is alive under the mouse; it cannot prove anything about a
    /// control whose whole meaning is the MOTION — a `CurvePoint` handle emits nothing on a Down/Up in
    /// the same place, so a gate built from clicks would be green over a handle that never moves a value.
    ///
    /// The Move carries the same button state as the Down, which is what makes the dispatcher treat it
    /// as a drag of the active widget rather than as hover.
    pub fn drag_at(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<WidgetEvent> {
        let mut out = Vec::new();
        for (kind, x, y) in [
            (PointerKind::Down, x0, y0),
            (PointerKind::Move, x1, y1),
            (PointerKind::Up, x1, y1),
        ] {
            self.clock_ns += NANOS_PER_SECOND;
            let arena = Bump::new();
            let event = PointerEvent {
                x,
                y,
                pressure: 1.0,
                kind,
                source: PointerSource::Mouse,
                button: PointerButton::Primary,
                timestamp_ns: self.clock_ns,
            };
            out.extend_from_slice(dispatch_pointer(
                &mut self.store,
                &self.hit_index,
                event,
                &arena,
            ));
        }
        out
    }

    /// The id a pointer at `(x, y)` actually LANDS ON — the dispatcher's own resolution
    /// (topmost = last registered wins). When [`Self::click_at`] emits nothing, this says
    /// whether the widget lost the hit to something painted over it, or was never hit at all.
    pub fn hit_at(&self, x: f32, y: f32) -> Option<NodeId> {
        self.hit_index.hit(x, y)
    }
}

impl Default for MockPanelHost {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelHost for MockPanelHost {
    fn theme(&self) -> Theme {
        self.theme
    }

    fn project(&self) -> &ProjectSettings {
        &self.project
    }
}

impl PanelHostInternal for MockPanelHost {
    fn store(&self) -> &WidgetStore {
        &self.store
    }

    fn store_mut(&mut self) -> &mut WidgetStore {
        &mut self.store
    }

    fn hit_index_mut(&mut self) -> &mut HitIndex {
        &mut self.hit_index
    }

    fn store_and_hit_index_mut(&mut self) -> (&WidgetStore, &mut HitIndex) {
        (&self.store, &mut self.hit_index)
    }

    fn bus(&self) -> &ActionBus {
        &self.bus
    }

    fn bus_mut(&mut self) -> &mut ActionBus {
        &mut self.bus
    }

    fn selection(&self) -> Option<&HeroSelection> {
        self.selection.as_ref()
    }

    fn selection_mut(&mut self) -> &mut Option<HeroSelection> {
        &mut self.selection
    }

    fn panel_visible(&self, id: &str) -> bool {
        self.visible.get(id).copied().unwrap_or(false)
    }

    fn set_panel_visible(&mut self, id: &str, value: bool) {
        self.visible.insert(id.to_string(), value);
    }

    fn grid_snap_state(&self) -> &GridSnapState {
        &self.grid_snap
    }

    fn grid_snap_state_mut(&mut self) -> &mut GridSnapState {
        &mut self.grid_snap
    }

    fn store_and_grid_snap_state_mut(&mut self) -> (&WidgetStore, &mut GridSnapState) {
        (&self.store, &mut self.grid_snap)
    }

    fn grid_snap_panel_rect(&self) -> Option<Rect> {
        self.grid_snap_panel_rect
    }

    fn set_grid_snap_panel_rect(&mut self, rect: Option<Rect>) {
        self.grid_snap_panel_rect = rect;
    }
}
