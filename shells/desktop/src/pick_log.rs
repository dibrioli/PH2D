//! ⭐ **O diagnóstico da escolha do conta-gotas** — `PH2D_PICK_LOG=1`.
//!
//! ⛔⛔ **Ele existe porque um report de *«não funciona»* sobre este caminho NÃO É DIAGNOSTICÁVEL
//! POR LEITURA** (2026-09-15, segundo report do dono sobre a mesma ferramenta). A escolha atravessa
//! **duas** fontes (a composição AUTORADA do Painter e a leitura do ECRÃ) e o ecrã tem **dois
//! modos** (o mundo vem da saída do tonemap, ou do acumulador num quadro intercalado) — e cada uma
//! das quatro combinações falha de maneira diferente e **silenciosa**.
//!
//! Uma linha por escolha, com tudo o que decide: as guardas, as duas respostas e os DOIS texels
//! crus. *A régua tem de dizer de onde veio o número, senão a próxima corrida volta com «ainda não
//! funciona».*

/// `PH2D_PICK_LOG=1` — lido uma vez.
#[must_use]
pub(crate) fn armado() -> bool {
    static A: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *A.get_or_init(|| std::env::var_os("PH2D_PICK_LOG").is_some_and(|v| v != "0"))
}

/// Uma linha com tudo o que a escolha decidiu. Ver o cabeçalho do módulo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn diz(
    px: u32,
    py: u32,
    mundo_e_o_acumulador: bool,
    selection: Option<u64>,
    on_panel: bool,
    autorada: Option<[u8; 4]>,
    do_ecra: Option<[u8; 4]>,
    texel_mundo: Option<[u8; 4]>,
    texel_chrome: Option<[u8; 4]>,
) {
    let fonte = if mundo_e_o_acumulador {
        "acumulador"
    } else {
        "tonemap"
    };
    eprintln!(
        "[pick] ({px}, {py}) sel={selection:?} painel={on_panel} fonte-do-mundo={fonte}\n\
         [pick]   AUTORADA (Painter) = {autorada:?}\n\
         [pick]   DO ECRA            = {do_ecra:?}\n\
         [pick]   texel mundo (BGRA) = {texel_mundo:?}   texel chrome (RGBA) = {texel_chrome:?}"
    );
}
