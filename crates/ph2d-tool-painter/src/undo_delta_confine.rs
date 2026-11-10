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
            // A caixa dos tiles RESTAURADOS, não a janela declarada (B17.5): fora deles os lados são iguais.
            Self::Tiles { win, rects, .. } => PlaneReach::Window(caixa_dos(rects).unwrap_or(*win)),
            Self::Whole { before, after } if before.is_empty() && after.is_empty() => {
                PlaneReach::Untouched
            }
            Self::Whole { .. } => PlaneReach::Whole,
        }
    }
}

/// A menor janela que contém todas as caixas (mesmo plano: mesmo `stride` e `plane_len`).
fn caixa_dos(rects: &[PlaneWindow]) -> Option<PlaneWindow> {
    let (first, rest) = rects.split_first()?;
    Some(rest.iter().fold(*first, |c, r| {
        let (row, col) = (c.row.min(r.row), c.col.min(r.col));
        let fim_r = (c.row + c.rows).max(r.row + r.rows);
        let fim_c = (c.col + c.cols).max(r.col + r.cols);
        PlaneWindow {
            row,
            rows: fim_r - row,
            col,
            cols: fim_c - col,
            ..c
        }
    }))
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

    /// O VIDRO ausente é o plano de zeros: um texel nunca selado dá o alfa único do píxel, exacto
    /// ([`crate::compositor::vidro::alfas`]). Então o lado que aparece (o 1.º traço de aguada numa camada)
    /// alcança só a janela dos texels selados, não a tela (B10c).
    pub(crate) fn add_vidros(&mut self, m: &StoredMap<crate::compositor::vidro::Vidro>) {
        for e in m.entries() {
            self.add(match e {
                StoredEntry::Both(p) => p.reach(),
                StoredEntry::OnlyBefore(p) | StoredEntry::OnlyAfter(p) => {
                    janela_dos_selados(p, self.canvas_w as usize)
                }
            });
        }
    }

    /// O RELEVO ausente é o plano neutro: camada sem `heights` sai da dobra e altura 0 com cobertura 0 é a
    /// identidade dela ([`crate::layers::fold_relief_step`]); `covers` ausente lê 0 e `mats` ausente lê o
    /// material neutro (`relief_fields::material_at`). Então o lado que aparece (o 1.º traço Impasto numa
    /// camada) alcança só a janela dos texels fora do neutro, não a tela (B10c).
    pub(crate) fn add_relevo<T>(
        &mut self,
        m: &StoredMap<T>,
        linha_neutra: impl Fn(&[T]) -> bool + Copy,
        neutro: impl Fn(&T) -> bool + Copy,
    ) {
        for e in m.entries() {
            self.add(match e {
                StoredEntry::Both(p) => p.reach(),
                StoredEntry::OnlyBefore(p) | StoredEntry::OnlyAfter(p) => {
                    janela_fora_do_neutro(p, self.canvas_w as usize, linha_neutra, neutro)
                }
            });
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

/// A janela dos texels não nulos de um plano de vidro de `stride` texels por linha; `Untouched` se nenhum.
fn janela_dos_selados(p: &[crate::compositor::vidro::Vidro], stride: usize) -> PlaneReach {
    // OR dos bytes da linha: vectoriza (o `any` com saída cedo não).
    janela_fora_do_neutro(
        p,
        stride,
        |l| l.as_flattened().iter().fold(0u8, |a, &b| a | b) == 0,
        |v| *v == [0; 7],
    )
}

/// A janela dos píxeis com alfa de um plano RGBA8 de `w` píxeis por linha; `Untouched` se nenhum. É onde virar
/// a marca do Wet de uma camada muda a luz do papel: alfa 0 deixa-o inteiro nas duas leis
/// (`relief_fields::papel_a_vista`), então fora dela a figura é a mesma (B10c). Com 1 texel de moldura: a altura
/// do papel muda nesses texels e a normal dos vizinhos lê-a (diferença central).
pub(crate) fn janela_da_tinta(rgba: &[u8], w: usize) -> PlaneReach {
    let (px, resto) = rgba.as_chunks::<4>();
    if !resto.is_empty() {
        return PlaneReach::Whole;
    }
    match janela_fora_do_neutro(
        px,
        w,
        |l| l.iter().fold(0u8, |a, p| a | p[3]) == 0,
        |p| p[3] == 0,
    ) {
        PlaneReach::Window(j) => {
            let linhas = px.len() / w;
            let (row, col) = (j.row.saturating_sub(1), j.col.saturating_sub(1));
            PlaneReach::Window(PlaneWindow {
                row,
                rows: (j.row + j.rows + 1).min(linhas) - row,
                col,
                cols: (j.col + j.cols + 1).min(w) - col,
                ..j
            })
        }
        outro => outro,
    }
}

/// A janela dos texels fora do neutro de um plano de `stride` texels por linha; `Untouched` se nenhum.
fn janela_fora_do_neutro<T>(
    p: &[T],
    stride: usize,
    linha_neutra: impl Fn(&[T]) -> bool,
    neutro: impl Fn(&T) -> bool,
) -> PlaneReach {
    if stride == 0 || p.is_empty() || !p.len().is_multiple_of(stride) {
        return PlaneReach::Whole;
    }
    let linhas: Vec<_> = p.chunks_exact(stride).collect();
    let Some(row) = linhas.iter().position(|l| !linha_neutra(l)) else {
        return PlaneReach::Untouched;
    };
    let last = linhas.iter().rposition(|l| !linha_neutra(l)).unwrap_or(row);
    let (mut col, mut end) = (stride, 0usize);
    for l in &linhas[row..=last] {
        if let Some(c) = l.iter().position(|v| !neutro(v)) {
            col = col.min(c);
            end = end.max(l.iter().rposition(|v| !neutro(v)).unwrap_or(c) + 1);
        }
    }
    PlaneReach::Window(PlaneWindow {
        row,
        rows: last - row + 1,
        col,
        cols: end.saturating_sub(col),
        stride,
        plane_len: p.len(),
    })
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
            Self::Tiles { .. } => "tiles",
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

    /// O vidro que APARECE alcança a caixa dos texels selados (também os de píxel intacto: a orla de alfa 0),
    /// nada fora dela; um plano de zeros não alcança nada.
    #[test]
    fn o_vidro_que_aparece_alcanca_so_os_texels_selados() {
        use super::{PlaneReach, janela_dos_selados};
        let (w, h) = (10usize, 6usize);
        let mut p = vec![[0u8; 7]; w * h];
        assert_eq!(janela_dos_selados(&p, w), PlaneReach::Untouched);
        p[w + 2] = [0, 0, 0, 0, 9, 0, 0];
        p[4 * w + 7] = [1, 2, 3, 4, 5, 6, 7];
        let PlaneReach::Window(j) = janela_dos_selados(&p, w) else {
            panic!("dois texels selados dão uma janela");
        };
        let r = j.to_region(w as u32).expect("escalar");
        assert_eq!((r.x, r.y, r.w, r.h), (2, 1, 6, 4));
    }

    /// Um plano ESCALAR (um elemento por pixel) tem de sair exato — é o controle do gate acima: sem
    /// ele, arredondar para fora poderia estar inflando toda região e ninguém veria.
    #[test]
    fn a_scalar_plane_converts_exactly() {
        let w = PlaneWindow::for_test(4, 5, 6, 7, 10);
        let r = w.to_region(10).expect("stride 10 mede um canvas de 10 px");
        assert_eq!((r.x, r.y, r.w, r.h), (6, 4, 7, 5));
    }

    /// **O passo guardado por tiles alcança os tiles RESTAURADOS, não a janela declarada** (B17.5): o
    /// commit do Impasto declara a caixa do traço inteiro e o undo repintava-a toda. Dois tiles em cantos
    /// opostos de uma janela declarada que é a tela: o alcance é a caixa dos dois, e não a tela.
    #[test]
    fn o_passo_por_tiles_alcanca_a_caixa_dos_tiles_que_mudaram() {
        use super::super::tiles::TileMarks;
        use super::{PlaneReach, Region, StoredPlane};
        use crate::undo::journal::TILE;
        use std::sync::Arc;
        let (w, h, t) = (1024u32, 768u32, TILE as u32);
        let before = Arc::new(vec![7u8; (w * h) as usize]);
        let mut a = (*before).clone();
        a[(t + 3) as usize * w as usize + t as usize + 1] = 9; // tile (1, 1)
        a[(2 * t + 3) as usize * w as usize + 3 * t as usize + 1] = 9; // tile (2, 3)
        let tudo = Region { x: 0, y: 0, w, h };
        let mut marks = TileMarks::default();
        marks.mark(tudo);
        let px = marks
            .for_hint(Some(tudo), 1)
            .expect("os marks viram a janela");
        let hint = PlaneWindow::from_region(tudo, w, w as usize);
        let (mut b, mut d) = (Arc::clone(&before), Arc::new(a));
        let p = StoredPlane::split_tiled(&mut b, &mut d, w as usize, hint, Some((&px, w)));
        assert_eq!(p.variant(), "tiles");
        let PlaneReach::Window(j) = p.reach() else {
            panic!("um passo por tiles tem janela");
        };
        let r = j.to_region(w).expect("escalar mede-se em pixels");
        assert_eq!((r.x, r.y, r.w, r.h), (t, t, 3 * t, 2 * t));
    }

    /// **O NÚMERO do B17.5 que fica por curar: um Ctrl+Z de dois tiles em cantos opostos da tela 4K publica
    /// a tela INTEIRA.** O sujo é um retângulo só (`Confine` → `mark_dirty` → dreno/placa), então a caixa
    /// dos dois tiles restaurados é 3840×2160 = 8 294 400 px para 128² + 128×112 = 30 720 px (270×).
    /// Fica VERMELHO quando o sujo passar a várias caixas: aí inverta-o para «suja só os dois».
    #[test]
    fn um_ctrl_z_de_dois_tiles_opostos_publica_a_tela_4k_e_este_e_o_numero() {
        use super::super::tiles::TileMarks;
        use super::{Confine, Region, StoredPlane};
        use crate::undo::journal::TILE;
        use std::sync::Arc;
        let (w, h, t) = (3840u32, 2160u32, TILE as u32);
        let before = Arc::new(vec![7u8; (w * h) as usize]);
        let mut a = (*before).clone();
        a[w as usize + 1] = 9; // tile (0, 0)
        a[(h - 2) as usize * w as usize + (w - 2) as usize] = 9; // o último tile
        let tudo = Region { x: 0, y: 0, w, h };
        let mut marks = TileMarks::default();
        marks.mark(tudo);
        let px = marks
            .for_hint(Some(tudo), 1)
            .expect("os marks viram a janela");
        let hint = PlaneWindow::from_region(tudo, w, w as usize);
        let (mut b, mut d) = (Arc::clone(&before), Arc::new(a));
        let p = StoredPlane::split_tiled(&mut b, &mut d, w as usize, hint, Some((&px, w)));
        assert_eq!(p.variant(), "tiles");
        let mut c = Confine::new(w);
        c.add(p.reach());
        let r = c.finish().expect("um passo por tiles é confinado");
        let sujo = u64::from(r.w) * u64::from(r.h);
        let StoredPlane::Tiles { rects, .. } = &p else {
            panic!("variant tiles");
        };
        let mudou: u64 = rects
            .iter()
            .map(|j| {
                j.to_region(w)
                    .map_or(0, |q| u64::from(q.w) * u64::from(q.h))
            })
            .sum();
        assert_eq!(
            (sujo, mudou, rects.len()),
            (
                8_294_400,
                u64::from(t) * u64::from(t) + u64::from(t) * 112,
                2
            ),
            "o sujo publicado é a caixa dos dois tiles — a tela 4K"
        );
    }
}
