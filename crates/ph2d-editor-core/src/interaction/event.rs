//! [`WidgetEvent`] — the per-frame event emitted by the dispatcher.
//!
//! Extracted from [`super::state`] (Track D1). The enum stays `Copy`
//! and carries only `NodeId`s — value-bearing variants (`ValueChanged`,
//! `TextChanged`, `Toggled`) require the caller to re-read the live
//! state from the store. That keeps the bumpalo arena allocation cost
//! to a single pointer bump and lets the per-frame slice survive
//! mutations to the store between emission and drain.

use ph2d_a11y::NodeId;

/// One event emitted by [`super::dispatch`]. No `String`/`Vec`
/// payloads — value-bearing variants carry only the `NodeId`; the
/// caller re-reads from the store. Keeps events `Copy` so arena
/// allocation costs a single pointer bump.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WidgetEvent {
    /// Button / Tag remove / ContextMenu item / Modal cancel|confirm.
    Click(NodeId),
    /// Two `Click(id)` events on the same widget within
    /// `DOUBLE_CLICK_WINDOW_NS` (350 ms). Dispatcher emits this
    /// instead of the second `Click(id)` so apply_event handlers can
    /// branch on intent (e.g. hierarchy row → focus the entity
    /// instead of selecting it again).
    DoubleClick(NodeId),
    /// M14.7 polish: Enter pressed while a "commit-on-enter"
    /// TextInput owns focus (currently just `HIER_RENAME_INPUT`).
    /// Carries the id so apply_event can read the buffer + apply.
    Submit(NodeId),
    /// M14.7 polish: Esc pressed on a "cancel-on-escape" TextInput.
    /// Same id semantics as `Submit` — caller drops the buffer.
    Cancel(NodeId),
    /// M14.7 polish: pointer held down on a hierarchy row for
    /// `LONG_PRESS_THRESHOLD_NS` (600 ms) without the drag threshold
    /// being exceeded. Hero apply_event interprets this as
    /// "enter inline rename mode for this row" — mirrors the
    /// right-click → "Rename…" path so touch / pen users get a
    /// modeless rename gesture without needing a context menu.
    LongPress(NodeId),
    /// Toggle / Checkbox / Switch — caller reads the new state from
    /// `store.get(id)`.
    Toggled(NodeId),
    /// Slider / NumberInput / ColorPicker channel — caller reads
    /// `store.get(id)` for the new numeric value.
    ValueChanged(NodeId),
    /// TextInput / Combobox query — caller reads `store.text(id)`.
    TextChanged(NodeId),
    Focus(NodeId),
    Blur(NodeId),
    // ⛔ **Não reintroduza uma `SelectionChanged(NodeId)` sem o consumidor na mesma wave.**
    // Ela existiu aqui, declarada «Tabs / Dropdown / TreeView — selected index changed», com
    // UMA ocorrência em todo o repositório: a própria declaração. Nunca foi construída nem
    // casada. E a razão não é esquecimento — é que a selecção deste sistema **não viaja por
    // evento**: o `apply_click` (`dispatch/focus.rs`) declara-o no fonte, *«caller observes via
    // store.get(id)»*, e quem marca a opção é o painel, lendo o `WidgetStore`. Uma variante que
    // duplicasse esse facto daria duas fontes de verdade para a mesma pergunta.
    // Removida em 2026-08-30; o gate `every_widget_event_variant_has_a_producer_and_a_reader`
    // impede que a próxima entre sem leitor.
    /// Eyedropper pick request — emitted when the user clicks
    /// anywhere outside the eyedropper button while eyedropper mode
    /// is pending. The host should sample the rendered pixel at
    /// `(px, py)` (physical pixels) and apply it to the picker at
    /// `parent` via `store.set_blender_value`. Pixel coords are
    /// `u32` so the event keeps `Copy + Eq` (no f32 fields).
    EyedropperPick {
        parent: NodeId,
        px: u32,
        py: u32,
    },
    /// M14.6B: hierarchy drag-reparent intent. Emitted on Up when a
    /// hierarchy DnD resolves to a drop position. `new_parent` is
    /// `None` for a root-level drop. `before` is the sibling the
    /// dragged row should be inserted *above* (`None` means "append
    /// at the end of siblings"). Carries only `NodeId`s so the event
    /// stays `Copy`. Fixture mode applies it directly to the panel
    /// store; live (ECS) mode routes it to the host which translates
    /// `NodeId → Entity` via the bridge and applies `ChildOf`.
    HierReparent {
        dragged: NodeId,
        new_parent: Option<NodeId>,
        before: Option<NodeId>,
        /// M14.7 polish: when set, the host inserts `dragged` AFTER
        /// this target sibling (= "next sibling of target in
        /// target's parent's Children"). Mutually exclusive with
        /// `before` in normal use; the host prefers `before` if both
        /// are accidentally Some. Lets the bottom-30% drop band land
        /// the dragged row as the new last child of a parent without
        /// resolving the next-sibling slot in the dispatcher.
        after: Option<NodeId>,
    },
    /// ⭐⭐⭐ **Uma LINHA DE PAINEL foi arrastada para cima de outra** — a família diz de QUAL painel.
    ///
    /// ⛔⛔ **Ela nasceu `PainterLayerReparent` e generalizou-se em 2026-09-14**, quando o painel
    /// *Tags* ia ser o **terceiro** a copiar a mesma máquina (a Hierarquia é o primeiro, e tem
    /// mutação própria no despacho; as camadas do Painter são o segundo). *Uma lei escrita em dois
    /// sítios ainda não é uma lei — só uma PORTA é*, e a terceira cópia era o momento de a fazer.
    ///
    /// O despacho resolve **só a geometria** (a banda `drop` relativa a uma linha-alvo); quem
    /// possui a estrutura — a `LayerStack` do Painter, a `TagTree` do projecto — é que a aplica.
    /// Os `NodeId` mantêm isto `Copy`.
    PanelRowReparent {
        family: PanelRowFamily,
        dragged: NodeId,
        drop: PanelRowDrop,
    },
}

/// ⭐⭐ **De que painel é uma linha arrastável** — a chave que separa as famílias que partilham a
/// máquina de arrasto do despacho.
///
/// ⚠️ **A Hierarquia NÃO está aqui**, e a ausência é a decisão: o arrasto dela **muta a árvore no
/// próprio despacho** (`hierarchy_move` / `hierarchy_set_parent`) e emite um evento com semântica
/// de irmãos (`before`/`after`/`new_parent`). Estas outras só resolvem geometria e entregam.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PanelRowFamily {
    /// As camadas do Painter (W3 T3.8) — o `LayerStack` é da ferramenta.
    PainterLayer,
    /// ⭐ As tags do projecto (TOP-20 #9, W4b) — a `TagTree` é do documento.
    TagTree,
}

/// Onde uma linha arrastada foi largada, relativamente a uma linha-alvo. Espelho do `HierDrop` da
/// hierarquia, mas quem o resolve é o dono da estrutura, não o despacho.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelRowDrop {
    /// Antes da linha-alvo (mesmo pai) — irmã anterior.
    ///
    /// ⚠️ **A `TagTree` ORDENA-SE sozinha** (pela chave dobrada), então para ela o `Before` e o
    /// `After` significam *«irmã de»* — o sítio na lista não é autorado.
    Before(NodeId),
    /// DENTRO da linha-alvo, como filha (o dono recusa um ciclo, ou um alvo que não aceita filhos).
    Inside(NodeId),
    /// Depois da linha-alvo (mesmo pai) — irmã seguinte.
    After(NodeId),
    /// Largada abaixo de TODAS as linhas → raiz.
    End,
}
