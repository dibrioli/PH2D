//! ⭐⭐⭐ **O que a HIERARQUIA pede** — as 33 formas de uma família só.
//!
//! # ⚠️ Por que elas saíram do [`super::action_bus`] (2026-09-01)
//!
//! Elas partilham o **sujeito** (uma `row` da lista — 29 das 33 não carregam mais nada), o
//! **dreno** (`shells/desktop/src/render_loop/hierarchy.rs`) e a **origem** (o painel da
//! Hierarquia). Escritas como 33 variantes soltas, elas eram 33 das ~150 do `EditorAction` — e
//! foram elas que puseram aquele ficheiro contra o tecto de 700 LOC.
//!
//! ⛔⛔ **E o gatilho foi MEDIDO:** acrescentar **uma** variante ao `EditorAction` custava **+78
//! linhas**, porque o `rustfmt` reflow-a as 37 variantes-estrutura para multi-linha de uma vez. O
//! ficheiro estava a 676/700 e não aceitava mais nenhuma. *Quando N variantes têm a mesma forma, a
//! forma é que é o dado* — e este ficheiro é o sítio onde ela vive.
//!
//! ⚠️ O molde é o do irmão [`super::action_bus_kinds`], que já guarda os vocabulários que as ações
//! carregam.

use super::action_bus_kinds::SelectModifier;

/// Ver o cabeçalho do módulo.
#[derive(Debug, Clone, PartialEq)]
pub enum HierRequest {
    /// Toggle the `Visibility` component on the entity backing the
    /// hierarchy row whose eye-icon was just clicked. Payload: the
    /// row's `NodeId`. The shell resolves NodeId → Entity via
    /// `HeroLive::bridge.entity_for(row)` and flips `Visibility.hidden`
    /// on `SimWorld`.
    ToggleVisibility { row: ph2d_a11y::NodeId },

    /// 2026-05-26 — user clicked the per-row lock icon. Shell drains
    /// and flips presence of `ph2d_ecs::Locked` on the row's entity
    /// (only the entity is locked; descendants remain editable).
    ToggleLock { row: ph2d_a11y::NodeId },

    /// 2026-05-26 — user clicked the per-row "group lock" (folder)
    /// icon. Shell drains and flips presence of
    /// `ph2d_ecs::GroupedChildren` on the row's entity (descendants
    /// locked; the entity itself remains editable).
    ToggleGroup { row: ph2d_a11y::NodeId },

    /// Drag-and-drop reparent for a hierarchy row. Payload mirrors
    /// the `WidgetEvent::HierReparent` event one-to-one. `new_parent
    /// = None` is a root-level drop; `before`/`after` position the
    /// dragged entity relative to a target sibling. The shell
    /// resolves NodeIds → Entities via the bridge and runs
    /// `hero_intents::drain_reparent` which rebuilds the bevy_ecs
    /// `Children` ordering by re-inserting `ChildOf` in sequence.
    Reparent(crate::screens::hero::HierReparentIntent),

    /// Duplicate the entity backing the hierarchy row. Payload: the
    /// row's `NodeId`. Shell copies Transform / Sprite / Name /
    /// ChildOf onto a freshly-spawned entity, suffixes the name
    /// with `_copy`, and toasts on success. Raised by the row's
    /// right-click → Duplicate menu entry.
    Duplicate { row: ph2d_a11y::NodeId },

    /// Despawn the entity backing the hierarchy row. Payload: the
    /// row's `NodeId`. Cascades through bevy_ecs 0.19's `ChildOf`
    /// relation, taking descendants with it. Also clears
    /// `gizmo_selection` if it pointed at the deleted entity.
    /// Raised by the row's right-click → Delete menu entry.
    Delete { row: ph2d_a11y::NodeId },

    /// ⭐⭐⭐ **Agrupar / desagrupar a selecção** (`line/Vector`, Enio 2026-08-30).
    ///
    /// ⚠️ **Estava em `EditorAction::HierGroup` e desceu para aqui na integração de
    /// 2026-09-04**, sob a regra que o corte do barramento escreveu: *carrega uma `row` da
    /// Hierarquia ⇒ vai para dentro do `HierRequest`*. As duas linhas cruzaram-se — uma
    /// acrescentou o par, a outra partiu o enum — e deixá-lo fora seria a excepção que
    /// desfaz a regra no primeiro sítio onde ela foi testada.
    ///
    /// ⚠️ **O `row` é o SUJEITO EXTRA, não o único** — o menu é por linha e agrupar é sobre um
    /// conjunto, então a shell une a linha clicada à selecção da Hierarquia
    /// ([`crate::screens::hero`] gizmo). Um payload que fosse só a linha faria *Group* nunca ter
    /// dois sujeitos e portanto **nunca funcionar**.
    ///
    /// ⚠️ **O verbo já existia em `Ctrl+G` e era inalcançável** — sem menu, sem botão, sem entrada
    /// de paleta, e cercado à ferramenta Vector. Este par de acções é o alcance dele.
    Group { row: ph2d_a11y::NodeId },

    /// O gémeo — ver [`Self::Group`].
    Ungroup { row: ph2d_a11y::NodeId },

    /// Reset the entity's `Transform` to `Transform::IDENTITY`.
    /// Payload: the row's `NodeId`. Raised by the row's right-click
    /// → Reset Transform menu entry.
    ResetTransform { row: ph2d_a11y::NodeId },

    /// ⭐ **Devolver a instância à receita** (ADR-0164 / F4.4) — apaga TODAS as excepções desta
    /// cópia. Payload: a `NodeId` da linha. Levantada pelo botão direito → *Revert to Master*.
    RevertToMaster { row: ph2d_a11y::NodeId },

    /// ⭐ **A seleção vira RECEITA** e uma instância fica no lugar dela (ADR-0164 / F4.5).
    MakeComponent { row: ph2d_a11y::NodeId },

    /// ⭐ **Instanciar** a receita desta linha (ADR-0164 / F4.5).
    Instantiate { row: ph2d_a11y::NodeId },

    /// ⭐ **Instanciar LIGADO** (Enio, 2026-08-27) — a cópia divide a ARTE da receita.
    InstantiateLinked { row: ph2d_a11y::NodeId },

    /// ⭐ **Destacar** — a instância deixa de seguir a receita (ADR-0164 / F4.5).
    Detach { row: ph2d_a11y::NodeId },

    /// ⭐ **Aplicar ao mestre** — a excepção vira o padrão (ADR-0164 / F4.5).
    ApplyToMaster { row: ph2d_a11y::NodeId },

    /// Spawn a new child entity (identity transform, name "Child")
    /// under the hierarchy row. Payload: the parent row's `NodeId`.
    /// Raised by the row's right-click → Add Child menu entry.
    AddChild { row: ph2d_a11y::NodeId },

    /// ⭐ **Um objeto VAZIO na raiz** — o botão **Add** do cabeçalho da Hierarquia (ADR-0166 / F3).
    ///
    /// ⚠️ **Sem payload, e é isso que o distingue do [`Self::HierAddChild`]:** aquele nasce de uma
    /// LINHA (o pai), este de um botão que não pertence a linha nenhuma. Dar-lhe um `row` seria
    /// inventar um pai para o objeto que o artista pediu **sem** pai.
    ///
    /// ⚠️ O botão existia, era pintado e registado desde a Fase C.2 — e **nada o consumia**. Um
    /// botão morto sob o dedo e um botão ausente dão o mesmo report; este é o primeiro dos dois.
    AddRoot,

    /// Composite the current multi-selection (≥ 2 sprites) into one new Individual-texture sprite at
    /// the union bbox, then despawn the originals. Payload: the right-clicked row's `NodeId` (visual
    /// anchor — the merge inherits its parent / z). Drain toasts when < 2 sprites are selected (no-op).
    MergeSprites { row: ph2d_a11y::NodeId },

    /// **Fundir em CAMADAS** — plano `docs/Sprite_projeto/18` W10 (Enio, 2026-08-21).
    ///
    /// ⚠️ Mesma geometria da [`Self::HierMergeSprites`]; o que muda é que cada fonte fica também
    /// numa camada do documento do Painter. Uma variante própria, e não um `bool` na de cima,
    /// porque o menu tem **duas linhas** e cada uma tem de dizer o que faz pelo nome.
    MergeToLayers { row: ph2d_a11y::NodeId },

    /// **Exportar UMA sprite** para o formato que a extensão escolhida nomear — plano
    /// `docs/Sprite_projeto/18` W9 (Enio, 2026-08-21).
    ///
    /// ⚠️ Levantar não é exportar: o painel não tem o `ExporterRegistry`, nem o renderer, nem os
    /// pixels. O shell tem os três.
    ExportImage { row: ph2d_a11y::NodeId },

    /// "Pack into Sheet" — o mesmo verbo do pill `[SHEET]`, levantado do menu de contexto de uma
    /// linha da hierarquia (Enio 2026-08-19). Payload: a `NodeId` da linha clicada, que a shell
    /// resolve em entidade e usa como **âncora**: se ela pertence à seleção, a folha leva a
    /// seleção inteira; se não, leva só ela — a mesma lei do [`Self::HierMergeSprites`] vizinho,
    /// de propósito (duas semânticas de seleção no mesmo menu seriam adivinhação).
    ///
    /// ⚠️ Dois verbos, escolhidos pelo alvo: sobre sprites CRIA a folha, sobre uma folha
    /// RE-ARRANJA os filhos dela.
    PackSheet { row: ph2d_a11y::NodeId },

    /// "Auto-Arrange Pieces" — re-encaixa os filhos da folha DENTRO da resolução dela
    /// (Enio 2026-08-19). Payload: a `NodeId` da linha clicada, que tem de ser uma folha.
    ///
    /// ⚠️ **Não redimensiona.** A resolução foi escolhida no modal quando a folha nasceu; este
    /// gesto é *arrume*, não *redimensione*. O que não couber acende a moldura.
    ArrangeSheet { row: ph2d_a11y::NodeId },

    /// "Bake Sheet" — compõe os filhos numa imagem e reata cada um a uma região dela: N sprites,
    /// uma textura, um draw call (plano §7.3, W5.2). Muda o documento.
    BakeSheet { row: ph2d_a11y::NodeId },

    /// "Export Sheet" — grava `<nome>.png` + `<nome>.json` ao lado do projeto. ⚠️ **Compõe sem
    /// reatar**: exportar não é editar.
    ExportSheet { row: ph2d_a11y::NodeId },

    /// "Remove from Sheet" — a peça deixa a folha e volta a ser objeto de raiz, **onde está**
    /// (Enio 2026-08-19). Payload: a `NodeId` da linha clicada.
    ///
    /// ⚠️ A shell serve-a pelo MESMO caminho do arrasto-para-a-raiz da hierarquia
    /// (`HierReparentIntent { new_parent: None }`), que já preserva a pose de mundo e reatribui o
    /// `RootOrder`. Uma segunda implementação da mesma saída seria a que se esqueceria do
    /// `RootOrder`.
    RemoveFromSheet { row: ph2d_a11y::NodeId },

    /// "Use as Brush Grain" — shell resolves row → pixels → `set_brush_texture_image` (Enio 2026-06-24).
    UseAsBrushTexture { row: ph2d_a11y::NodeId },

    /// "Use as Brush Shape" — shell resolves row → pixels → `set_brush_shape_image` (Enio 2026-06-25).
    UseAsBrushShape { row: ph2d_a11y::NodeId },

    /// "Use as Watercolor Paper" — shell resolves row → pixels → `use_layers_as_watercolor_paper`
    /// (`docs/Painter/10…` §5).
    UseAsPaper { row: ph2d_a11y::NodeId },

    /// "Use as Granulation" — shell resolves row → pixels → `use_layers_as_granulation`.
    UseAsGranulation { row: ph2d_a11y::NodeId },

    /// Sync `gizmo_selection` to the entity backing the clicked
    /// hierarchy row — cross-panel selection sync from the
    /// hierarchy panel to the canvas gizmo. Payload: the row's
    /// `NodeId`. Live (ECS) mode only; fixture-only rows don't
    /// raise this.
    RowClick { row: ph2d_a11y::NodeId },

    /// Hierarchy-panel multi-select event (Fase 0b). Twin of
    /// [`Self::SelectSprite`] but keyed by the row's `NodeId` —
    /// `ph2d-panel-hierarchy` does not have a `NodeId → Entity`
    /// resolver, so the shell does the lookup via the live bridge
    /// before applying the same `GizmoStateGroup` mutation. Replaces
    /// the legacy [`Self::HierRowClick`] for new emitters; the legacy
    /// variant stays in the enum for now to avoid churning shell
    /// drain logic outside Fase 0e.
    SelectRow {
        row: ph2d_a11y::NodeId,
        modifier: SelectModifier,
    },

    /// Hierarchy-panel range select (Fase 0b). Shift-click on a live
    /// row with a primary already set: shell walks the hierarchy row
    /// order from the current primary's row to `row` and calls
    /// `add_to_selection` on every entity in between (inclusive). No-op
    /// when nothing is selected (no anchor). Canvas clicks have no
    /// natural linear order so they do not emit this variant.
    RangeSelect { row: ph2d_a11y::NodeId },

    /// One-shot seed of the rename TextInput buffer when inline-
    /// rename mode opens. Payload: the row's `NodeId`. Shell reads
    /// the entity's current `Name`, fills `HIER_RENAME_INPUT.text`,
    /// and selects all. Without the one-shot semantic, subsequent
    /// Backspace edits would get clobbered back to the original
    /// name on every frame. Raised by right-click → Rename and by
    /// long-press on the row.
    RenameSeed { row: ph2d_a11y::NodeId },

    /// Finalized rename commit (Enter / blur on the rename
    /// TextInput). Payload: the row's `NodeId` + the trimmed new
    /// name. Shell writes the new `Name` component on the entity
    /// and toasts confirmation, then clears the rename TextInput
    /// buffer. `String` owned-data payload is fine — `EditorAction`
    /// is `Clone` (not `Copy`); see the `editor_action_is_clone_and_partial_eq`
    /// test below.
    RenameCommit {
        row: ph2d_a11y::NodeId,
        new_name: String,
    },

    /// ⭐⭐ **Tirar da biblioteca** — a receita deixa de o ser. A lei das duas metades lê-se
    /// inteira em `shells/desktop/src/instance_unmake.rs`.
    RemoveFromLibrary { row: ph2d_a11y::NodeId },
}
