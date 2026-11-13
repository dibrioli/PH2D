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
            Self::Whole { .. } | Self::Nova { .. } => PlaneReach::Whole,
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
    #[cfg(test)]
    pub(crate) fn reach(&self) -> PlaneReach {
        match self {
            Self::Both(p) => p.reach(),
            Self::OnlyBefore(_) | Self::OnlyAfter(_) => PlaneReach::Whole,
            Self::Aparece { win, .. } => win.map_or(PlaneReach::Untouched, PlaneReach::Window),
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
#[derive(Clone, Debug)]
pub(crate) struct Confine {
    canvas_w: u32,
    acc: Option<Region>,
    /// Cada janela somada, sem união (B17.5): a união delas é o `acc`.
    caixas: Vec<Region>,
    confined: bool,
}

impl Confine {
    pub(crate) const fn new(canvas_w: u32) -> Self {
        Self {
            canvas_w,
            acc: None,
            caixas: Vec::new(),
            confined: true,
        }
    }

    /// Um plano guardado: o passo por tiles soma CADA tile restaurado (a união é a mesma da `reach`).
    pub(crate) fn add_plano<T>(&mut self, p: &StoredPlane<T>) {
        match p {
            StoredPlane::Tiles { rects, .. } if !rects.is_empty() => {
                rects.iter().for_each(|&j| self.add(PlaneReach::Window(j)));
            }
            _ => self.add(p.reach()),
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
                    self.caixas.push(r);
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
        for (&k, e) in &m.entries {
            match e {
                StoredEntry::Both(p) => self.add_plano(p),
                StoredEntry::OnlyBefore(p) | StoredEntry::OnlyAfter(p) => {
                    let w = self.canvas_w as usize;
                    let r = m
                        .janela_que_aparece(k, w)
                        .unwrap_or_else(|| janela_dos_selados(p, w));
                    self.add(r);
                }
                StoredEntry::Aparece { win, .. } => {
                    self.add(win.map_or(PlaneReach::Untouched, PlaneReach::Window));
                }
            }
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
        for (&k, e) in &m.entries {
            match e {
                StoredEntry::Both(p) => self.add_plano(p),
                StoredEntry::OnlyBefore(p) | StoredEntry::OnlyAfter(p) => {
                    let w = self.canvas_w as usize;
                    let r = m
                        .janela_que_aparece(k, w)
                        .unwrap_or_else(|| janela_fora_do_neutro(p, w, linha_neutra, neutro));
                    self.add(r);
                }
                StoredEntry::Aparece { win, .. } => {
                    self.add(win.map_or(PlaneReach::Untouched, PlaneReach::Window));
                }
            }
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
    pub(crate) fn finish(&self) -> Option<Region> {
        if self.confined { self.acc } else { None }
    }

    /// A região e as caixas que a cobrem (B17.5): vazia quando uma só basta. A união das caixas é a
    /// região — o sujo publicado nunca encolhe, só deixa de ter buracos.
    pub(crate) fn finish_caixas(self) -> Option<(Region, Vec<Region>)> {
        let r = self.finish()?;
        let caixas = funde_as_caixas(self.caixas);
        Some((r, if caixas.len() > 1 { caixas } else { Vec::new() }))
    }
}

/// Teto da lista: quem compõe cada caixa paga um despacho por caixa.
pub(crate) const MAX_CAIXAS: usize = 16;
/// Acima disto a fusão gulosa (O(n³)) custaria mais que a união: fica a caixa só.
const MAX_ENTRADA: usize = 128;

fn area(r: Region) -> u64 {
    u64::from(r.w) * u64::from(r.h)
}

/// Funde as caixas até [`MAX_CAIXAS`], sempre o par que menos área acrescenta (as que se tocam ou
/// sobrepõem fundem-se de graça). Determinista: ordem fixa, empate pelo primeiro par.
pub(crate) fn funde_as_caixas(mut v: Vec<Region>) -> Vec<Region> {
    use crate::tool::paint::region::union_region;
    v.retain(|r| r.w > 0 && r.h > 0);
    v.sort_unstable_by_key(|r| (r.y, r.x, r.h, r.w));
    v.dedup();
    // Os tiles vizinhos na mesma linha juntam-se num passo linear.
    let mut linha: Vec<Region> = Vec::with_capacity(v.len());
    for r in v {
        match linha.last_mut() {
            Some(p) if p.y == r.y && p.h == r.h && p.x + p.w >= r.x => *p = union_region(*p, r),
            _ => linha.push(r),
        }
    }
    let mut v = linha;
    if v.len() > MAX_ENTRADA {
        return Vec::new();
    }
    while v.len() > 1 {
        let mut melhor = (u64::MAX, 0, 0);
        for i in 0..v.len() {
            for j in i + 1..v.len() {
                let d = area(union_region(v[i], v[j])).saturating_sub(area(v[i]) + area(v[j]));
                if d < melhor.0 {
                    melhor = (d, i, j);
                }
            }
        }
        let (d, i, j) = melhor;
        if d > 0 && v.len() <= MAX_CAIXAS {
            break;
        }
        v[i] = union_region(v[i], v[j]);
        v.remove(j);
    }
    v
}

/// A janela dos texels não nulos de um plano de vidro de `stride` texels por linha; `Untouched` se nenhum.
fn janela_dos_selados(p: &[crate::compositor::vidro::Vidro], stride: usize) -> PlaneReach {
    janela_fora_do_neutro(p, stride, linha_sem_vidro, vidro_neutro)
}

/// A linha de vidro sem texel selado: OR dos bytes (vectoriza; o `any` com saída cedo não).
pub(crate) fn linha_sem_vidro(l: &[crate::compositor::vidro::Vidro]) -> bool {
    l.as_flattened().iter().fold(0u8, |a, &b| a | b) == 0
}

pub(crate) fn vidro_neutro(v: &crate::compositor::vidro::Vidro) -> bool {
    *v == [0; 7]
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
    let janela = if JANELA_DA_TINTA_PARALELA.with(std::cell::Cell::get) {
        janela_do_alfa_paralela(px, w)
    } else {
        janela_fora_do_neutro(
            px,
            w,
            |l| l.iter().fold(0u8, |a, p| a | p[3]) == 0,
            |p| p[3] == 0,
        )
    };
    match janela {
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

thread_local! {
    /// **A chave do A×B** (n.º 7 item 3, o «+» do Wet): `true` = a janela do alfa por faixas de linhas em paralelo;
    /// `false` = a busca em série de sempre (o oráculo). As duas dão a mesma janela.
    pub(crate) static JANELA_DA_TINTA_PARALELA: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Linhas por faixa da busca paralela (4K: 2160 linhas → 34 faixas de ~1 MB).
const LINHAS_POR_FAIXA: usize = 64;

/// A janela dos texels de alfa ≠ 0, por faixas de linhas em paralelo: cada faixa devolve (1.ª linha, última linha,
/// menor coluna, maior coluna + 1) dos texels com alfa, e a redução junta-as. Mesmo resultado que
/// [`janela_fora_do_neutro`] com o neutro «alfa 0».
fn janela_do_alfa_paralela(px: &[[u8; 4]], stride: usize) -> PlaneReach {
    use rayon::prelude::*;
    if stride == 0 || px.is_empty() || !px.len().is_multiple_of(stride) {
        return PlaneReach::Whole;
    }
    #[cfg(test)]
    TEXELS_VARRIDOS.with(|c| c.set(c.get() + px.len()));
    type Caixa = Option<(usize, usize, usize, usize)>;
    let junta = |a: Caixa, b: Caixa| match (a, b) {
        (Some(a), Some(b)) => Some((a.0.min(b.0), a.1.max(b.1), a.2.min(b.2), a.3.max(b.3))),
        (x, None) | (None, x) => x,
    };
    let caixa = px
        .par_chunks(stride * LINHAS_POR_FAIXA)
        .enumerate()
        .map(|(f, faixa)| {
            let mut c: Caixa = None;
            for (k, l) in faixa.chunks_exact(stride).enumerate() {
                if l.iter().fold(0u8, |a, p| a | p[3]) == 0 {
                    continue;
                }
                let r = f * LINHAS_POR_FAIXA + k;
                let ini = l.iter().position(|p| p[3] != 0).unwrap_or(0);
                let fim = l.iter().rposition(|p| p[3] != 0).unwrap_or(ini) + 1;
                c = junta(c, Some((r, r, ini, fim)));
            }
            c
        })
        .reduce(|| None, junta);
    match caixa {
        None => PlaneReach::Untouched,
        Some((row, last, col, end)) => PlaneReach::Window(PlaneWindow {
            row,
            rows: last - row + 1,
            col,
            cols: end - col,
            stride,
            plane_len: px.len(),
        }),
    }
}

#[cfg(test)]
thread_local! {
    /// Texels que a busca do neutro varreu neste thread (o gate do B08c.M2 conta-os no Ctrl+Z).
    pub(crate) static TEXELS_VARRIDOS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// A janela dos texels fora do neutro de um plano de `stride` texels por linha; `Untouched` se nenhum.
pub(super) fn janela_fora_do_neutro<T>(
    p: &[T],
    stride: usize,
    linha_neutra: impl Fn(&[T]) -> bool,
    neutro: impl Fn(&T) -> bool,
) -> PlaneReach {
    if stride == 0 || p.is_empty() || !p.len().is_multiple_of(stride) {
        return PlaneReach::Whole;
    }
    #[cfg(test)]
    TEXELS_VARRIDOS.with(|c| c.set(c.get() + p.len()));
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
    /// Só o `confine_report` (teste) lê as entradas: o `Confine` vai pela `janela_que_aparece` (B08c.M2 C1).
    #[cfg(test)]
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
            Self::Nova { .. } => "NOVA",
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
            Self::Aparece { .. } => "APARECE",
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

    /// **A janela da tinta em paralelo é a da série** (n.º 7 item 3): planos com tinta na 1.ª e na última linha, a
    /// cavalo de duas faixas, num texel só, nas colunas extremas, e vazios. Controlo: as duas chaves passam pela
    /// porta pública ([`super::janela_da_tinta`]) e a série é o oráculo.
    #[test]
    fn a_janela_da_tinta_paralela_e_a_da_serie() {
        use super::{JANELA_DA_TINTA_PARALELA, PlaneReach, janela_da_tinta};
        let w = 37usize;
        let h = super::LINHAS_POR_FAIXA * 3 + 5;
        let casos: &[&[(usize, usize)]] = &[
            &[],
            &[(0, 0)],
            &[(h - 1, w - 1)],
            &[(63, 5), (64, 30)],
            &[(10, 36), (150, 0), (70, 12)],
            &[(128, 7)],
            &[(0, 3), (h - 1, 3)],
        ];
        let mut vistas = 0;
        for alfas in casos {
            let mut rgba = vec![0u8; w * h * 4];
            for &(r, c) in *alfas {
                rgba[(r * w + c) * 4 + 3] = 9;
            }
            // RGB sem alfa não conta como tinta.
            rgba[(5 * w + 6) * 4] = 200;
            let com = |p| {
                JANELA_DA_TINTA_PARALELA.with(|c| c.set(p));
                let j = janela_da_tinta(&rgba, w);
                JANELA_DA_TINTA_PARALELA.with(|c| c.set(true));
                j
            };
            let (par, ser) = (com(true), com(false));
            assert_eq!(par, ser, "tinta em {alfas:?}");
            if alfas.is_empty() {
                assert_eq!(ser, PlaneReach::Untouched);
            } else {
                vistas += 1;
                assert!(matches!(ser, PlaneReach::Window(_)), "{alfas:?}: {ser:?}");
            }
        }
        assert_eq!(vistas, casos.len() - 1);
    }

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

    /// **B17.5: um Ctrl+Z de dois tiles em cantos opostos da tela 4K publica só os dois tiles.** A união
    /// continua a ser a tela (3840×2160 = 8 294 400 px), mas as caixas ao lado dela somam 128² + 128×112 =
    /// 30 720 px. Red (o sujo era um retângulo só): 8 294 400 publicados.
    #[test]
    fn um_ctrl_z_de_dois_tiles_opostos_publica_so_os_dois_tiles() {
        use super::super::tiles::TileMarks;
        use super::{Confine, Region, StoredPlane, area};
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
        c.add_plano(&p);
        let (r, caixas) = c.finish_caixas().expect("um passo por tiles é confinado");
        assert_eq!(
            u64::from(r.w) * u64::from(r.h),
            8_294_400,
            "a união é a tela"
        );
        let sujo: u64 = caixas.iter().map(|&q| area(q)).sum();
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
            (sujo, mudou, rects.len(), caixas.len()),
            (
                u64::from(t) * u64::from(t) + u64::from(t) * 112,
                u64::from(t) * u64::from(t) + u64::from(t) * 112,
                2,
                2
            ),
            "o sujo publicado são os dois tiles, não a tela 4K"
        );
    }

    /// A fusão das caixas: a união nunca muda, os vizinhos juntam-se, o teto é [`MAX_CAIXAS`] e a resposta
    /// não depende da ordem de entrada.
    #[test]
    fn as_caixas_fundidas_cobrem_a_mesma_uniao_e_nao_dependem_da_ordem() {
        use super::{MAX_CAIXAS, Region, area, funde_as_caixas};
        use crate::tool::paint::region::union_region;
        let t = 128u32;
        let tile = |c: u32, r: u32| Region {
            x: c * t,
            y: r * t,
            w: t,
            h: t,
        };
        // Uma diagonal de 30 tiles e uma linha de 4 vizinhos.
        let mut v: Vec<Region> = (0..30).map(|i| tile(i, i / 2)).collect();
        v.extend((0..4).map(|c| tile(c, 16)));
        let uniao = v.iter().copied().reduce(union_region);
        let a = funde_as_caixas(v.clone());
        v.reverse();
        let b = funde_as_caixas(v.clone());
        assert_eq!(a, b, "a ordem de entrada não muda a resposta");
        assert!(a.len() <= MAX_CAIXAS && a.len() > 1, "{} caixas", a.len());
        assert_eq!(
            a.iter().copied().reduce(union_region),
            uniao,
            "a união é a mesma"
        );
        for q in &v {
            assert!(
                a.iter().any(|c| c.x <= q.x
                    && c.y <= q.y
                    && c.x + c.w >= q.x + q.w
                    && c.y + c.h >= q.y + q.h),
                "o tile {q:?} fica coberto"
            );
        }
        let area: u64 = a.iter().map(|&q| area(q)).sum();
        let u = uniao.expect("há caixas");
        assert!(
            area * 4 < u64::from(u.w) * u64::from(u.h),
            "a diagonal não vira a caixa dela: {area}"
        );
    }

    /// Acima do teto a fusão gulosa CORRE: 40 tiles em escada com saltos (nenhum par se toca, o passo
    /// linear não junta nada) caem para [`MAX_CAIXAS`] e cada tile de entrada continua coberto (o undo
    /// não perde dados fora da janela publicada).
    #[test]
    fn acima_do_teto_a_fusao_cobre_cada_tile_de_entrada() {
        use super::{MAX_CAIXAS, Region, funde_as_caixas};
        let t = 128u32;
        let v: Vec<Region> = (0..40)
            .map(|i| Region {
                x: 2 * i * t,
                y: i * t,
                w: t,
                h: t,
            })
            .collect();
        let a = funde_as_caixas(v.clone());
        assert_eq!(a.len(), MAX_CAIXAS, "a fusão para no teto");
        let mut r = v.clone();
        r.reverse();
        assert_eq!(
            funde_as_caixas(r),
            a,
            "a ordem de entrada não muda a resposta"
        );
        for q in &v {
            assert!(
                a.iter().any(|c| c.x <= q.x
                    && c.y <= q.y
                    && c.x + c.w >= q.x + q.w
                    && c.y + c.h >= q.y + q.h),
                "o tile {q:?} fica coberto"
            );
        }
    }
}
