//! **Os interruptores dos oráculos da silhueta** (B14c, B14c-cont3) — filho de [`super`] (LOC cap): o corte das
//! células fora da pegada e a caixa da silhueta base; nos testes, um `thread_local` desliga cada um («como antes»).

#[cfg(test)]
thread_local! {
    /// O oráculo de B14c: a ponta lê a silhueta base sem a caixa dela, como antes.
    pub(super) static SEM_MEMO_BASE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// As caixas da silhueta base criadas neste fio (a prova de que o caso as exerce).
    pub(super) static MEMOS_BASE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// O oráculo de B14c-cont3: toda célula da caixa do motor avaliada, como antes.
    pub(super) static SEM_CORTE_FORA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn com_corte_fora() -> bool {
    #[cfg(test)]
    {
        !SEM_CORTE_FORA.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    !crate::wet_diag::SIL_SEM_CORTE.load(std::sync::atomic::Ordering::Relaxed)
}

pub(super) fn com_memo_base() -> bool {
    #[cfg(test)]
    {
        MEMOS_BASE.with(|n| n.set(n.get() + 1));
        !SEM_MEMO_BASE.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    true
}
