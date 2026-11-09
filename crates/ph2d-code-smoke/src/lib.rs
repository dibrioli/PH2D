//! ⭐ **Os roteiros do smoke conduzido do editor de scripts** (`PH2D_CODE_SMOKE=<n>
//! PH2D_CODE_SMOKE_PASSO=<k>`): os gestos (`Gesture`), as esperas (`Until`) e o roteiro de cada cena
//! (`STEPS*`). Só dados: quem os conduz pelas portas do artista é o `ph2d-app-code::smoke_drive`;
//! os scripts e as formas das cenas moram no `ph2d-code-scenes`. Saíram das `ph2d-app-*` no fecho da Onda D
//! (catraca `the_shell_only_shrinks_and_every_app_crate_too`).

use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;

/// O passo até onde a cena 3 se conduz sozinha (0 = nada, o estado em que o dono a recebe).
pub const STEP_ENV: &str = "PH2D_CODE_SMOKE_PASSO";

/// `cena` = acabado o roteiro, a aba `Scene` passa à frente (o clique do dono na aba): a foto das
/// formas, que a aba do script tapa.
pub const VIEW_ENV: &str = "PH2D_CODE_SMOKE_FOTO";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gesture {
    /// O cursor `ahead` bytes depois do início da 1.ª `needle` do texto (o `:` da paleta).
    Go {
        needle: &'static str,
        ahead: usize,
    },
    Key(Key, Modifiers),
    /// Um objecto da Hierarchy largado SEM tecla na 1.ª `needle` (item 15: abre a pergunta).
    DropObject {
        needle: &'static str,
        name: &'static str,
    },
    /// Uma letra por quadro, como a mão: a gramática assenta entre as teclas (num quadro só, a
    /// letra depois de um Enter num comentário ainda se lia como comentário e a lista não abria).
    Type(&'static str),
    Wait(Until),
    /// O rato (sem botão) sobre o carácter `ahead` bytes depois da 1.ª `needle`.
    Point {
        needle: &'static str,
        ahead: usize,
    },
    /// O Ctrl carregado (`true`) ou largado, como a janela o diz ao `WidgetStore`.
    Ctrl(bool),
    /// Um clique na linha `k` da lista de problemas pintada.
    ClickProblem(usize),
    /// Um clique no separador `k` da faixa de baixo (0 = Console, 1 = Problems).
    ClickBandTab(usize),
    /// Um clique no título `k` da barra da aba (0 = File … 4 = Run).
    ClickTabMenu(usize),
    /// O rato (sem botão) sobre a linha `k` do menu aberto da barra.
    PointTabMenuRow(usize),
    /// Um clique na engrenagem da barra (o cartão «Script options»).
    ClickGear,
    /// Um clique no botão `k` do cartão «Stopped» (0 = Continue, 1 = Next line).
    ClickDebug(usize),
    /// Um clique na coluna das marcas (o ponto de parada) da linha da 1.ª `needle`.
    ClickMark {
        needle: &'static str,
    },
    /// O botão direito sobre o carácter `ahead` bytes depois da 1.ª `needle` (o menu do item 14).
    RightClick {
        needle: &'static str,
        ahead: usize,
    },
    /// O resto do passo entra no quadro seguinte (cada Tab vê-se).
    Frame,
    /// Um clique com `mods` sobre o carácter `ahead` bytes depois da 1.ª `needle` (o Ctrl+clique).
    Click {
        needle: &'static str,
        ahead: usize,
        mods: Modifiers,
    },
    /// Uma tecla na aba ACTIVA da família (a página de ajuda, ou o script).
    TabKey(Key, Modifiers),
    /// O rato (sem botão) sobre / um clique no alvo da Bancada pintado no último quadro (Onda B).
    PointAim(ph2d_code_viewmodel::Aim),
    ClickAim(ph2d_code_viewmodel::Aim),
    /// Uma letra por quadro na aba activa (o campo da busca da página).
    TabType(&'static str),
    /// Um clique no link pintado da página `page` que leva a `to` no membro `member`.
    Link {
        page: &'static str,
        to: &'static str,
        member: &'static str,
    },
    /// O Ctrl+Z pela JANELA (a shell recebe-o como do teclado; a tela virtual ignora teclas reais).
    WindowUndo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Until {
    /// Um problema cuja mensagem contém o texto.
    Problem(&'static str),
    /// A lista aberta com este rótulo nela.
    ListHas(&'static str),
    /// A lista aberta com este rótulo em PRIMEIRO.
    ListTop(&'static str),
    /// As acções do Alt+Enter abertas.
    Actions,
    /// Um esqueleto com buracos activo.
    Snippet,
    /// A caixinha do F2 aberta e, numa `ph2d.property`, com os objectos lidos (a ajuda conta-os).
    RenameBox,
    /// A aba sem ● e sem caixinha.
    Saved,
    /// Pelo menos `n` problemas na calha.
    Problems(usize),
    /// O assistente respondeu sobre o nome sob o rato: `true` = o Ctrl+clique alcança-o e o
    /// sublinhado foi pintado; `false` = não alcança, e nada se pintou.
    Reach(bool),
    /// A lista de problemas pintada com linhas.
    ProblemRows,
    /// O texto contém isto.
    Has(&'static str),
    /// A página `page` é a aba activa e pintou `member` no topo.
    Page {
        page: &'static str,
        member: &'static str,
    },
    /// O script é a aba activa.
    Script,
    /// O assistente já verificou o script (publicou os diagnósticos dele).
    Checked,
    /// A busca da página activa contou (o «1 of N»).
    Found,
    /// O jogo parado neste script, com o título do cartão a conter o texto («on Star»).
    Stopped(&'static str),
    /// Valores ao vivo pintados no fim de alguma linha, e o jogo a correr.
    Live,
    /// Nada sublinhado e o ponteiro sobre o carácter `ahead` depois de `needle` não é a mão.
    Plain { needle: &'static str, ahead: usize },
    /// A cena aparece no lugar da aba (o espiar da Bancada).
    Peeking,
    /// A célula `(objecto, propriedade)` da grelha diz este valor.
    CellIs(usize, usize, &'static str),
}

pub mod steps;
pub use steps::NONE;
pub use steps::{STEPS, STEPS_4, STEPS_5, STEPS_10, STEPS_11, STEPS_12, STEPS_13};

/// O roteiro da cena `level` (`None` = a cena não tem fotos conduzidas).
#[must_use]
pub fn steps(level: u32) -> Option<&'static [&'static [Gesture]]> {
    match level {
        3 => Some(STEPS),
        4 => Some(STEPS_4),
        5 => Some(STEPS_5),
        10 => Some(STEPS_10),
        11 => Some(STEPS_11),
        12 => Some(STEPS_12),
        13 => Some(STEPS_13),
        _ => None,
    }
}
