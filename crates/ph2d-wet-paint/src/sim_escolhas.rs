//! As escolhas B13.10/B13.4 que um [`crate::sim::Sim`] novo recebe: a omissão do produto (a lei antiga; as variáveis
//! `PH2D_FREIO_NA_GRADE_FINA=1` / `PH2D_SECAGEM_POR_TILE=1` ligam) ou, dentro de [`com_escolhas`], as do fio.

std::thread_local! {
    static DO_FIO: std::cell::Cell<Option<(bool, bool)>> = const { std::cell::Cell::new(None) };
}

/// **As escolhas que um `Sim` novo deste fio recebe** durante `f` (`(freio_na_grade_fina, secagem_por_tile)`), em
/// vez da omissão. Para quem cria a sessão por dentro (as pranchas, que pintam pelo `PainterTool`): o `Sim` nasce
/// no fio do teste e leva as escolhas nos campos, sem estado global.
#[doc(hidden)]
pub fn com_escolhas<R>(
    freio_na_grade_fina: bool,
    secagem_por_tile: bool,
    f: impl FnOnce() -> R,
) -> R {
    let antes = DO_FIO.replace(Some((freio_na_grade_fina, secagem_por_tile)));
    let r = f();
    DO_FIO.set(antes);
    r
}

/// `(freio_na_grade_fina, secagem_por_tile)` de um `Sim` que nasce agora.
pub(crate) fn de_um_sim_novo() -> (bool, bool) {
    DO_FIO.get().unwrap_or_else(|| {
        (
            crate::flow::freio_na_grade_fina_por_omissao(),
            crate::drying::por_tile::secagem_por_tile_por_omissao(),
        )
    })
}
