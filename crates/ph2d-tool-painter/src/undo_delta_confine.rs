//! **ATÉ ONDE UM PASSO GUARDADO ALCANÇA** — filho de [`super`] (`#[path]`, então ele enxerga os campos
//! privados de [`PlaneWindow`]), pelo mesmo motivo que o `journal_route`: a janela é um tipo com
//! invariantes, e abri-la para a crate inteira só para responder a esta pergunta seria pagar com o
//! encapsulamento dela.
//!
//! # A pergunta
//!
//! *Este passo mudou a figura só dentro de um retângulo?* Quem responde **sim** habilita o undo a
//! publicar esse retângulo em vez do canvas inteiro — e a diferença medida é **386,74 → ~0,6 ms** num
//! quadro de 4096² com impasto (doc 28 §5.62/§5.63).
//!
//! # A regra que torna o `None` seguro
//!
//! ⚠️ **Toda dúvida responde `Whole`.** Um `Untouched` errado serve pixels velhos em volta do
//! retângulo — corrupção visual silenciosa, que nenhum gate de conteúdo pega porque o conteúdo DENTRO
//! da janela está certo. Um `Whole` errado só custa um repaint. As duas respostas não têm o mesmo
//! preço, então a assimetria está no tipo: [`PlaneReach::Whole`] é o default de todo caso que este
//! módulo não sabe descrever.

use super::{ImageEntry, PlaneWindow, StoredEntry, StoredImages, StoredMap, StoredPlane};
use crate::compositor::Region;

/// Quanto de um plano um passo guardado toca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaneReach {
    /// Os dois lados são o mesmo conteúdo — o passo **não toca** este plano.
    Untouched,
    /// Os dois lados diferem só nesta janela.
    Window(PlaneWindow),
    /// O passo pode ter mudado qualquer coisa neste plano.
    Whole,
}

impl PlaneWindow {
    /// Constrói uma janela ARBITRÁRIA — só para os gates de conversão abaixo, que precisam de janelas
    /// que o produto produz raramente (primeiro ou último elemento no meio de um pixel). Mora aqui, e
    /// não no pai, porque é aqui que ela é usada — e o pai está no teto de LOC.
    #[cfg(test)]
    pub(crate) const fn for_test(
        row: usize,
        rows: usize,
        col: usize,
        cols: usize,
        stride: usize,
    ) -> Self {
        Self {
            row,
            rows,
            col,
            cols,
            stride,
            plane_len: 0,
        }
    }

    /// O menor retângulo de PIXELS do canvas que **CONTÉM** esta janela de elementos.
    ///
    /// ⚠️ **Ela arredonda para FORA, e não é a inversa exata de [`Self::from_region`] — de propósito.**
    /// A janela sai de um `diff_window`, que acha o primeiro e o último ELEMENTO que diferem; num plano
    /// RGBA são quatro elementos por pixel, e um passo que muda só o canal verde de um texel produz
    /// `col` que **não é múltiplo de 4**. Exigir alinhamento ali devolvia `None` para todo traço de
    /// pigmento — medido: `canvas=win` em todos os planos e a região saindo `None` assim mesmo.
    ///
    /// ⚠️ **A direção do arredondamento é a única segura, e a assimetria é o argumento:** um retângulo
    /// GRANDE demais custa alguns texels de repaint; um PEQUENO demais deixa na tela a figura anterior,
    /// e nenhum gate de conteúdo o pega — dentro do retângulo está tudo certo. O gate afirma **contenção**
    /// (`from_region` ∘ `to_region` ⊇ o original), nunca igualdade.
    ///
    /// `None` só quando o plano não é medível em pixels do canvas (o stride não o mede), e aí o
    /// chamador cai no repaint inteiro, que é sempre correto.
    pub(crate) fn to_region(self, canvas_w: u32) -> Option<Region> {
        let cw = canvas_w as usize;
        if cw == 0 || self.stride == 0 || !self.stride.is_multiple_of(cw) {
            return None;
        }
        let k = self.stride / cw; // elementos por pixel: 4 no RGBA, 1 nos escalares
        if k == 0 {
            return None;
        }
        let x0 = self.col / k; // floor: o pixel onde o primeiro elemento vive
        let x1 = (self.col + self.cols).div_ceil(k); // ceil: o pixel logo depois do último
        Some(Region {
            x: u32::try_from(x0).ok()?,
            y: u32::try_from(self.row).ok()?,
            w: u32::try_from(x1.saturating_sub(x0)).ok()?,
            h: u32::try_from(self.rows).ok()?,
        })
    }
}

impl<T> StoredPlane<T> {
    /// O alcance deste plano. `Whole` guarda os dois buffers inteiros, então ele não descreve janela
    /// nenhuma — é o caso em que o passo mudou a forma, ou em que a janela não compensava.
    ///
    /// ⚠️ **Com uma exceção MEDIDA: dois lados VAZIOS não mudam figura nenhuma.** O `split` manda para
    /// `Whole` tudo o que ele não sabe medir — e `fits()` recusa comprimento zero —, então as SEIS
    /// superfícies da sessão de Sculpt, que num traço de pigmento comum são buffers vazios de ambos os
    /// lados, chegavam aqui como `Whole` e **azedavam o confinamento de todo passo do produto**
    /// (medido: `spre=WHOLE samt=WHOLE ssum=WHOLE sc=WHOLE sm=WHOLE srgba=WHOLE` em cada entrada).
    ///
    /// Isso era `Whole` conflando *"mudou em toda parte"* com *"não sei medir"*, e um plano vazio não é
    /// nenhum dos dois: instalar qualquer um dos lados devolve o mesmo plano vazio. A pergunta é
    /// `is_empty()` nos DOIS — exata e sem varredura.
    pub(crate) fn reach(&self) -> PlaneReach {
        match self {
            Self::Unchanged => PlaneReach::Untouched,
            Self::Patch { win, .. } => PlaneReach::Window(*win),
            Self::Whole { before, after } if before.is_empty() && after.is_empty() => {
                PlaneReach::Untouched
            }
            Self::Whole { .. } => PlaneReach::Whole,
        }
    }
}

impl<T> StoredEntry<T> {
    /// ⚠️ `OnlyBefore`/`OnlyAfter` são **`Whole`**: o plano inteiro aparece ou desaparece, e restaurá-lo
    /// muda o relevo em todo lugar onde ele tinha cobertura — não há retângulo que descreva isso. É o
    /// caso do **primeiro** traço de relevo numa camada, que por isso cai no repaint inteiro.
    pub(crate) fn reach(&self) -> PlaneReach {
        match self {
            Self::Both(p) => p.reach(),
            Self::OnlyBefore(_) | Self::OnlyAfter(_) => PlaneReach::Whole,
        }
    }
}

impl ImageEntry {
    const fn reach(&self) -> PlaneReach {
        match self {
            Self::Unchanged => PlaneReach::Untouched,
            Self::Patch { win, .. } => PlaneReach::Window(*win),
            Self::Both(..) | Self::OnlyBefore(_) | Self::OnlyAfter(_) => PlaneReach::Whole,
        }
    }
}

/// **O acumulador da região confinada.**
///
/// Ele junta o alcance de vários planos e **azeda para sempre** no instante em que um deles não é
/// confinável. Existe como tipo (em vez de um `Option<Region>` passado à mão) porque o modo de falha de
/// esquecer um plano é uma reivindicação de confinamento boa demais — e um acumulador que só sabe
/// azedar não pode ser usado errado nessa direção.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Confine {
    canvas_w: u32,
    acc: Option<Region>,
    confined: bool,
}

impl Confine {
    pub(crate) const fn new(canvas_w: u32) -> Self {
        Self {
            canvas_w,
            acc: None,
            confined: true,
        }
    }

    pub(crate) fn add(&mut self, reach: PlaneReach) {
        if !self.confined {
            return;
        }
        match reach {
            PlaneReach::Untouched => {}
            PlaneReach::Whole => self.confined = false,
            PlaneReach::Window(w) => match w.to_region(self.canvas_w) {
                // Uma janela que não vira retângulo de pixels é uma janela que não sabemos publicar.
                None => self.confined = false,
                Some(r) => {
                    self.acc = Some(
                        self.acc
                            .map_or(r, |a| crate::tool::paint::region::union_region(a, r)),
                    );
                }
            },
        }
    }

    pub(crate) fn add_map<T>(&mut self, m: &StoredMap<T>) {
        for e in m.entries() {
            self.add(e.reach());
        }
    }

    pub(crate) fn add_images(&mut self, m: &StoredImages) {
        for e in m.entries() {
            self.add(e.reach());
        }
    }

    /// A região, ou `None` para *não confinado*.
    ///
    /// ⚠️ **Um passo confinado a NENHUM plano devolve `None`, não um retângulo vazio.** Um passo que
    /// não toca pixel nenhum (só metadados) não tem o que publicar, e um retângulo de área zero seria
    /// um pedido de repaint que não repinta nada — o chamador cai no caminho de sempre.
    pub(crate) const fn finish(self) -> Option<Region> {
        if self.confined { self.acc } else { None }
    }
}

impl<T> StoredMap<T> {
    pub(crate) fn entries(&self) -> impl Iterator<Item = &StoredEntry<T>> {
        self.entries.values()
    }
}

impl StoredImages {
    fn entries(&self) -> impl Iterator<Item = &ImageEntry> {
        self.entries.values()
    }

    /// O alcance de cada imagem — só para o relatório de diagnóstico.
    #[cfg(test)]
    pub(crate) fn reaches(&self) -> Vec<PlaneReach> {
        self.entries().map(ImageEntry::reach).collect()
    }
}

/// **A VARIANTE literal de um plano guardado** — irmã do [`StoredPlane::reach`] e feita para uma
/// pergunta que ele **não** responde: o `reach` colapsa `Whole`, `OnlyBefore` e `OnlyAfter` num
/// `PlaneReach::Whole` só (de propósito — para ELE as três significam *repinte tudo*), e a §5.66 §4
/// precisa saber **por qual porta** um plano entrou no `Whole`.
#[cfg(test)]
impl<T> StoredPlane<T> {
    pub(crate) const fn variant(&self) -> &'static str {
        match self {
            Self::Unchanged => "-",
            Self::Patch { .. } => "patch",
            Self::Whole { .. } => "WHOLE",
        }
    }
}

#[cfg(test)]
impl<T> StoredEntry<T> {
    pub(crate) const fn variant(&self) -> &'static str {
        match self {
            Self::Both(p) => p.variant(),
            Self::OnlyBefore(_) => "ONLY-BEFORE",
            Self::OnlyAfter(_) => "ONLY-AFTER",
        }
    }
}

#[cfg(test)]
impl<T> StoredMap<T> {
    /// Uma etiqueta por camada. Vazio = o mapa não guardou chave nenhuma.
    pub(crate) fn variant_tags(&self) -> String {
        if self.entries.is_empty() {
            return "-".to_string();
        }
        self.entries
            .values()
            .map(StoredEntry::variant)
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::PlaneWindow;

    /// **A região CONTÉM a janela de que ela veio** — a propriedade, não a igualdade.
    ///
    /// ⚠️ Este gate existe porque a mutação *"arredonde para DENTRO"* **SOBREVIVEU** ao oráculo de
    /// aparência: a janela que um traço do produto produz calhava de ser alinhada a pixel, e ali
    /// `floor` e `ceil` dão o mesmo número. A falha só existe numa janela cujo primeiro ou último
    /// ELEMENTO cai no meio de um pixel — e essa é a janela que um passo que muda um único canal
    /// produz. *Uma fixture que não contém o fenômeno não pode julgá-lo* — então a fixture aqui é
    /// construída para o conter, em vez de esperar que o produto o produza.
    #[test]
    fn the_region_contains_every_element_of_the_window_it_came_from() {
        let cw = 10u32;
        for (col, cols) in [(5usize, 6usize), (1, 2), (4, 8), (7, 1), (0, 40), (39, 1)] {
            let w = PlaneWindow::for_test(2, 3, col, cols, (cw as usize) * 4);
            let r = w.to_region(cw).expect("um plano RGBA e' medivel em pixels");
            let k = 4usize;
            assert!(
                (r.x as usize) * k <= col,
                "col={col} cols={cols}: a regiao comeca DEPOIS do primeiro elemento (x={})",
                r.x
            );
            assert!(
                ((r.x + r.w) as usize) * k >= col + cols,
                "col={col} cols={cols}: a regiao acaba ANTES do ultimo elemento (x={} w={})",
                r.x,
                r.w
            );
            assert_eq!((r.y, r.h), (2, 3), "as linhas passam sem conversao");
        }
    }

    /// Um plano ESCALAR (um elemento por pixel) tem de sair exato — é o controle do gate acima: sem
    /// ele, arredondar para fora poderia estar inflando toda região e ninguém veria.
    #[test]
    fn a_scalar_plane_converts_exactly() {
        let w = PlaneWindow::for_test(4, 5, 6, 7, 10);
        let r = w.to_region(10).expect("stride 10 mede um canvas de 10 px");
        assert_eq!((r.x, r.y, r.w, r.h), (6, 4, 7, 5));
    }
}
