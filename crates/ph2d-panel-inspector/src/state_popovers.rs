//! **OS SLOTS DOS POPOVERS DIFERIDOS** — os quatro seletores do Inspector e o rect que o clique
//! de fora consulta.
//!
//! ⚠️ **Irmão de [`super::state`] por CAP de FICHEIRO** (600) — o seletor do VERBO (report do dono,
//! 2026-09-09) levou-o a 615, e a regra da casa é cortar para o irmão, nunca subir a tolerância.
//!
//! ⚠️ **Eles andam juntos por uma LEI, não por vizinhança:** um popover aberto tem de sair da ordem
//! em que a sua seção foi pintada, senão a seção seguinte desenha-lhe por cima. Cada seção deposita
//! aqui o rect do seu chip durante o passe normal; o [`crate::paint_frame_shared`] resgata-o e
//! desenha por último; e o fim do [`crate::paint`] publica o rect do painel no store, que é o que
//! faz um clique FORA fechar o seletor.
//!
//! ⚠️ **Cada slot guarda o MÍNIMO que não se pode rederivar.** Opções e rótulos saem do snapshot no
//! passe diferido — uma cópia deles aqui seria a segunda fonte da mesma verdade, e as duas
//! divergiriam exactamente no quadro em que a escolha muda.

thread_local! {
    /// W3 §7: when the Sorting Layer dropdown is open, the section stashes
    /// `(selected_index, chip_rect)` here so `paint_inspector` paints its
    /// popover LAST (above every other widget). Panel-local — distinct
    /// from the showcase's shared `PENDING_DROPDOWN_CHIP`.
    pub(crate) static PENDING_ORDERING_DD:
        std::cell::Cell<Option<(usize, ph2d_editor_core::zones::Rect)>> =
        const { std::cell::Cell::new(None) };

    /// §12: quando o seletor «Rides Parent Anchor» está aberto, a seção guarda aqui o rect do
    /// chip para o `paint_inspector` pintar o popover POR ÚLTIMO (acima de todo o resto).
    ///
    /// ⚠️ **Só o rect, e nenhum valor.** As opções e a escolha rederivam-se do snapshot
    /// (`current_inspector_anchor`) no passe diferido — guardar aqui uma cópia delas seria uma
    /// segunda fonte para a mesma verdade, e as duas divergiriam no quadro em que a seleção muda.
    pub(crate) static PENDING_MOUNT_DD: std::cell::Cell<Option<ph2d_editor_core::zones::Rect>> =
        const { std::cell::Cell::new(None) };

    /// SIGNAL ACTIONS: quando o seletor do VERBO está aberto, a seção guarda aqui
    /// `(tag escolhida, rect do chip)` para o popover se pintar POR ÚLTIMO.
    ///
    /// ⚠️ **A tag vem no slot e os RÓTULOS não** — e a assimetria é a lei das duas irmãs, não um
    /// descuido: os rótulos rederivam-se do snapshot (`current_inspector_action`), que é a fonte
    /// deles; a tag é a da linha ABERTA no editor, e essa vive no `InspectorState`, que o passe
    /// diferido não alcança. *Guardar o que não se pode rederivar; rederivar o resto.*
    pub(crate) static PENDING_ACTION_DD:
        std::cell::Cell<Option<(u8, ph2d_editor_core::zones::Rect)>> =
        const { std::cell::Cell::new(None) };

    /// ⭐ **O popover que ESTE painel pintou neste quadro** — `(dono, rect do painel)`.
    ///
    /// ⚠️ Ele existe para uma coisa só: o `dispatch::pointer_down` fecha um dropdown aberto quando
    /// o clique cai **fora** do popover e fora do chip (*«se o usuário clicar fora do dropdown ele
    /// deve se fechar»*, Enio 2026-06-24) — e essa lei lê `store.dropdown_popover()`, que **nenhum
    /// dos seletores do Inspector publicava**. Os outros nove painéis do app publicam-no no sítio
    /// onde pintam; aqui o passe diferido não tem `&mut WidgetStore`, então ele deposita e o fim
    /// do `paint` publica.
    pub(crate) static PAINTED_POPOVER:
        std::cell::Cell<Option<(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)>> =
        const { std::cell::Cell::new(None) };
}

pub(crate) fn set_pending_ordering_dd(chip: Option<(usize, ph2d_editor_core::zones::Rect)>) {
    PENDING_ORDERING_DD.with(|c| c.set(chip));
}

pub(crate) fn take_pending_ordering_dd() -> Option<(usize, ph2d_editor_core::zones::Rect)> {
    PENDING_ORDERING_DD.with(|c| c.take())
}

pub(crate) fn set_pending_mount_dd(chip: Option<ph2d_editor_core::zones::Rect>) {
    PENDING_MOUNT_DD.with(|c| c.set(chip));
}

pub(crate) fn take_pending_mount_dd() -> Option<ph2d_editor_core::zones::Rect> {
    PENDING_MOUNT_DD.with(|c| c.take())
}

pub(crate) fn set_pending_action_dd(chip: Option<(u8, ph2d_editor_core::zones::Rect)>) {
    PENDING_ACTION_DD.with(|c| c.set(chip));
}

pub(crate) fn take_pending_action_dd() -> Option<(u8, ph2d_editor_core::zones::Rect)> {
    PENDING_ACTION_DD.with(|c| c.take())
}

/// Regista que um popover foi pintado neste quadro — ver [`PAINTED_POPOVER`].
pub(crate) fn set_painted_popover(id: ph2d_a11y::NodeId, panel: ph2d_editor_core::zones::Rect) {
    PAINTED_POPOVER.with(|c| c.set(Some((id, panel))));
}

pub(crate) fn take_painted_popover() -> Option<(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)> {
    PAINTED_POPOVER.with(|c| c.take())
}
