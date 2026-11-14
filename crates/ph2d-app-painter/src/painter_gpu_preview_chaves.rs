//! As chaves A×B da prévia na placa (as de execução, `PH2D_*`, e as dos gates): filho de [`super`] (LOC cap).

#[cfg(test)]
thread_local! {
    /// O caminho antigo (semente inteira em todo quadro), o oráculo de `a_regiao_suja_e_a_tela_inteira_ao_bit`.
    pub(crate) static SEMPRE_INTEIRA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// B18.4: o caminho de antes da cura (a camada nova sobe inteira no 1.º toque), o controlo do gate.
    pub(crate) static NOVA_SEM_LIMPAR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// N7 onda 5: a lista nova de um gesto de estrutura confinado não confina a placa (a mutação/o controlo).
    pub(crate) static ESTRUTURA_RECOMPOE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Quantas drenagens confinaram com a lista nova de um gesto de estrutura (o controlo do gate).
    pub(crate) static CONFINADAS_POR_ESTRUTURA: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// n.º 9 item 3: a dobra ignora a parte do relevo e dobra a região inteira (o controlo/a mutação).
    pub(crate) static DOBRA_A_REGIAO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Quantas dobras foram só a parte do relevo de uma região maior (o controlo do gate).
    pub(crate) static DOBRAS_PELA_FIGURA: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// n.º 9 item 3, a chave A×B em execução: `PH2D_DOBRA_A_REGIAO=1` = a troca com a tinta viva dobra o relevo na
/// região inteira (o caminho de antes). Lida uma vez.
pub(super) fn dobra_a_regiao_como_antes() -> bool {
    #[cfg(test)]
    if DOBRA_A_REGIAO.get() {
        return true;
    }
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("PH2D_DOBRA_A_REGIAO").is_some_and(|v| v == "1"))
}

/// N7 onda 5, a chave A×B em execução: `PH2D_ESTRUTURA_RECOMPOE=1` = a placa compõe a tela inteira depois de
/// apagar, duplicar ou agrupar (o caminho de antes). Lida uma vez.
pub(super) fn estrutura_recompoe_como_antes() -> bool {
    #[cfg(test)]
    if ESTRUTURA_RECOMPOE.get() {
        return true;
    }
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("PH2D_ESTRUTURA_RECOMPOE").is_some_and(|v| v == "1"))
}
