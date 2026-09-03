//! **O que o menu Ficheiro pediu** — três bandeiras que o shell drena.
//!
//! ⚠️ `Save`, `Save As…` e `Open Project…` (`CTX_MENU_SAVE` / `_SAVE_AS` / `_OPEN_PROJECT`) foram
//! **placeholders até 2026-08-23**: eles fechavam o menu, devolviam `true` — o gesto parecia
//! consumido — e não faziam nada. *Um botão que consome o clique e não age é pior que um botão
//! ausente: o artista conclui que gravou.*
//!
//! ⚠️ **Aqui só se diz o que foi PEDIDO.** Quem decide o que acontece é o shell, porque é ele que
//! tem o disco e o seletor de ficheiros — o painel não sabe (nem pode saber) se este `Save` precisa
//! de perguntar um caminho. É o mesmo mecanismo do `import_requested`, e não um segundo.
//!
//! ⚠️ **Um struct, e não três campos soltos no [`super::HeroScreen`]:** aquele ficheiro está a dois
//! LOC do teto (HR-18, 700), e três campos com o doc que eles merecem levavam-no por cima. O corte
//! para um irmão é a cura da casa — nunca uma isenção.

/// As três bandeiras. Limpas pelo shell (`std::mem::take`) depois de agir.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileMenuRequests {
    /// Gravar no ficheiro da sessão — ou perguntar, se ainda não houver um.
    pub save: bool,
    /// Gravar perguntando sempre.
    pub save_as: bool,
    /// Abrir outro projeto. ⚠️ Este pergunta **sempre**: é o gesto que deita fora o trabalho não
    /// gravado, e a decisão vive do lado do shell (`project_io`).
    pub open: bool,
    /// **Exportar o desenho vectorial como SVG** (plano 40). ⚠️ Pergunta sempre o caminho: um
    /// export não tem «o ficheiro da sessão» — o projecto tem, e não é o mesmo ficheiro.
    pub export_svg: bool,
}
