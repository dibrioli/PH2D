//! **O ECO DE UMA LARGADA** — irmão do [`super`] pelo tecto de LOC do painel e por
//! RESPONSABILIDADE. Ordem do dono (2026-09-19): *«Após a troca o conjunto linha e nó piscam e se
//! acentam.»*

use std::cell::RefCell;

thread_local! {
    static PISCADA: RefCell<Option<Piscada>> = const { RefCell::new(None) };
}

/// ⭐⭐⭐ **O ECO de uma largada que ACONTECEU** — ordem do dono (2026-09-19): *«Após a troca o
/// conjunto linha e nó piscam e se acentam.»*
///
/// ⚠️⚠️ **A INTENSIDADE vem resolvida da shell, e é isso que mantém o painel sem relógio.** O
/// `GraphViewSnapshot::now` é o PLAYHEAD — as riscas a marchar leem-no de propósito, para não
/// mentirem sobre um grafo pausado —, e um eco de gesto preso a ele ficaria **aceso para sempre**
/// no primeiro pause. Quem tem um relógio de PAREDE é a shell (`FixedStep::wall_seconds`), então é
/// ela que faz a conta e publica o `t`. *O painel pinta o número que recebe; ele não conta tempo.*
///
/// ⚠️ **Canal lateral e não campo do retrato**, pela razão que o [`CARD_TEXTS`] acima já escreve.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Piscada {
    /// Os nós envolvidos (os dois de uma troca; o que entrou num fio).
    pub nos: Vec<u32>,
    /// Os fios envolvidos, pela ponta de chegada.
    pub fios: Vec<(u32, u16)>,
    /// Quanto do eco ainda resta, `1` no instante da acção e `0` quando ele acaba.
    pub t: f32,
}

/// Publica (ou apaga) o eco da última largada — shell.
pub fn set_graph_flash(v: Option<Piscada>) {
    PISCADA.with(|c| *c.borrow_mut() = v);
}

/// Lê o eco (painel). `None` fora de uma largada recente.
#[must_use]
pub fn current_graph_flash() -> Option<Piscada> {
    PISCADA.with(|c| c.borrow().clone())
}
