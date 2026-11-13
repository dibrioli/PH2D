//! **A janela da tinta** (n.º 7 item 3): onde uma camada RGBA8 tem alfa — a porta [`janela_da_tinta`] e a varredura
//! por palavras de 64 bits que a serve.

#[cfg(test)]
use super::TEXELS_VARRIDOS;
use super::{PlaneReach, PlaneWindow, janela_fora_do_neutro};

/// A janela dos píxeis com alfa de um plano RGBA8 de `w` píxeis por linha; `Untouched` se nenhum. É onde virar
/// a marca do Wet de uma camada muda a luz do papel: alfa 0 deixa-o inteiro nas duas leis
/// (`relief_fields::papel_a_vista`), então fora dela a figura é a mesma (B10c). Com 1 texel de moldura: a altura
/// do papel muda nesses texels e a normal dos vizinhos lê-a (diferença central).
pub(crate) fn janela_da_tinta(rgba: &[u8], w: usize) -> PlaneReach {
    let (px, resto) = rgba.as_chunks::<4>();
    if !resto.is_empty() {
        return PlaneReach::Whole;
    }
    let janela = if JANELA_DA_TINTA_POR_PALAVRAS.with(std::cell::Cell::get) {
        janela_do_alfa_por_palavras(px, w)
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

/// **A janela do vidro** de uma camada (n.º 8, B18.6): onde o plano tem um texel selado. Sobre o papel de cor o vidro
/// tira a transparência de cada canal também em texels de alfa 0 (a orla de pigmento, `compositor::vidro::atravessa`),
/// então a figura que a camada faz mora em tinta ∪ vidro; fora dos dois o texel é `[0; 7]` e não tira nada.
pub(crate) fn janela_do_vidro(p: &[crate::compositor::vidro::Vidro], w: usize) -> PlaneReach {
    super::janela_dos_selados(p, w)
}

/// **A janela do corpo** de uma camada (n.º 8 item 3, Impasto): onde o plano `p` (de `w` texels por linha) sai do
/// `neutro` — a altura `±0`, a cobertura 0, o material `[0; 7]`. A luz lê a altura composta pela diferença central, por
/// isso a janela leva 1 texel de moldura (a [`dilata`] do chamador).
pub(crate) fn janela_do_corpo<T>(p: &[T], w: usize, neutro: impl Fn(&T) -> bool) -> PlaneReach {
    super::janela_fora_do_neutro(p, w, |l| l.iter().all(&neutro), &neutro)
}

/// A janela crescida `k` texels para cada lado, cortada pelo plano.
pub(crate) fn dilata(r: PlaneReach, k: usize) -> PlaneReach {
    match r {
        PlaneReach::Window(j) if j.stride > 0 => {
            let linhas = j.plane_len / j.stride;
            let (row, col) = (j.row.saturating_sub(k), j.col.saturating_sub(k));
            PlaneReach::Window(PlaneWindow {
                row,
                rows: (j.row + j.rows + k).min(linhas) - row,
                col,
                cols: (j.col + j.cols + k).min(j.stride) - col,
                ..j
            })
        }
        outro => outro,
    }
}

/// A menor janela que contém as duas (o mesmo plano em texels de `stride` igual); `Whole` se não casam.
pub(crate) fn uniao(a: PlaneReach, b: PlaneReach) -> PlaneReach {
    match (a, b) {
        (PlaneReach::Untouched, x) | (x, PlaneReach::Untouched) => x,
        (PlaneReach::Window(a), PlaneReach::Window(b))
            if a.stride == b.stride && a.plane_len == b.plane_len =>
        {
            let (row, col) = (a.row.min(b.row), a.col.min(b.col));
            PlaneReach::Window(PlaneWindow {
                row,
                rows: (a.row + a.rows).max(b.row + b.rows) - row,
                col,
                cols: (a.col + a.cols).max(b.col + b.cols) - col,
                ..a
            })
        }
        _ => PlaneReach::Whole,
    }
}

thread_local! {
    /// **A chave do A×B** (n.º 7 item 3, o «+» do Wet): `true` = a janela do alfa por palavras de 64 bits
    /// ([`janela_do_alfa_por_palavras`]); `false` = a busca em série por texel de sempre (o oráculo). As duas dão a
    /// mesma janela.
    pub(crate) static JANELA_DA_TINTA_POR_PALAVRAS: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Os bytes de alfa de dois píxeis RGBA8 lidos como um `u64` little-endian.
const ALFA_DO_PAR: u64 = u64::from_le_bytes([0, 0, 0, 0xFF, 0, 0, 0, 0xFF]);

/// A linha sem alfa: OR das palavras de 64 bits com a máscara do alfa (vectoriza; a dobra por texel não).
fn linha_sem_alfa(l: &[[u8; 4]]) -> bool {
    let (pares, resto) = l.as_flattened().as_chunks::<8>();
    pares.iter().fold(0u64, |a, b| a | u64::from_le_bytes(*b)) & ALFA_DO_PAR == 0
        && resto.get(3).is_none_or(|&a| a == 0)
}

/// O 1.º texel com alfa da linha, por palavras.
fn primeiro_alfa(l: &[[u8; 4]]) -> Option<usize> {
    let (pares, _) = l.as_flattened().as_chunks::<8>();
    match pares
        .iter()
        .position(|b| u64::from_le_bytes(*b) & ALFA_DO_PAR != 0)
    {
        Some(i) => Some(if l[2 * i][3] != 0 { 2 * i } else { 2 * i + 1 }),
        None => (l.len() % 2 == 1 && l[l.len() - 1][3] != 0).then(|| l.len() - 1),
    }
}

/// O último texel com alfa da linha, por palavras.
fn ultimo_alfa(l: &[[u8; 4]]) -> Option<usize> {
    if l.len() % 2 == 1 && l[l.len() - 1][3] != 0 {
        return Some(l.len() - 1);
    }
    let (pares, _) = l.as_flattened().as_chunks::<8>();
    let i = pares
        .iter()
        .rposition(|b| u64::from_le_bytes(*b) & ALFA_DO_PAR != 0)?;
    Some(if l[2 * i + 1][3] != 0 {
        2 * i + 1
    } else {
        2 * i
    })
}

/// A janela dos texels de alfa ≠ 0 por palavras de 64 bits: as linhas de fora vão pelo OR mascarado, e nas da faixa
/// cada linha só procura onde pode alargar a caixa (à esquerda da menor coluna, à direita da maior). Mesmo
/// resultado que [`janela_fora_do_neutro`] com o neutro «alfa 0».
fn janela_do_alfa_por_palavras(px: &[[u8; 4]], stride: usize) -> PlaneReach {
    if stride == 0 || px.is_empty() || !px.len().is_multiple_of(stride) {
        return PlaneReach::Whole;
    }
    #[cfg(test)]
    TEXELS_VARRIDOS.with(|c| c.set(c.get() + px.len()));
    let linhas: Vec<_> = px.chunks_exact(stride).collect();
    let Some(row) = linhas.iter().position(|l| !linha_sem_alfa(l)) else {
        return PlaneReach::Untouched;
    };
    let last = linhas
        .iter()
        .rposition(|l| !linha_sem_alfa(l))
        .unwrap_or(row);
    let (mut col, mut end) = (stride, 0usize);
    for l in &linhas[row..=last] {
        if let Some(c) = primeiro_alfa(&l[..col]) {
            col = c;
        }
        if let Some(e) = ultimo_alfa(&l[end..]) {
            end += e + 1;
        }
    }
    PlaneReach::Window(PlaneWindow {
        row,
        rows: last - row + 1,
        col,
        cols: end.saturating_sub(col),
        stride,
        plane_len: px.len(),
    })
}

#[cfg(test)]
mod tests {
    /// **A janela da tinta por palavras é a da série** (n.º 7 item 3): planos com tinta na 1.ª e na última linha,
    /// num texel só, nas colunas extremas (a última de uma largura ímpar fica fora das palavras), nos dois píxeis de
    /// uma palavra, e vazios. Controlo: as duas chaves passam pela porta pública ([`super::janela_da_tinta`]) e a
    /// série é o oráculo.
    #[test]
    fn a_janela_da_tinta_por_palavras_e_a_da_serie() {
        use super::super::PlaneReach;
        use super::{JANELA_DA_TINTA_POR_PALAVRAS, janela_da_tinta};
        for w in [37usize, 38] {
            let h = 197usize;
            let casos: &[&[(usize, usize)]] = &[
                &[],
                &[(0, 0)],
                &[(h - 1, w - 1)],
                &[(63, 5), (64, 30)],
                &[(10, 36), (150, 0), (70, 12)],
                &[(128, 7)],
                &[(0, 3), (h - 1, 3)],
                &[(40, w - 1), (41, w - 2)],
                &[(90, 1), (91, 2), (92, w - 3)],
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
                    JANELA_DA_TINTA_POR_PALAVRAS.with(|c| c.set(p));
                    let j = janela_da_tinta(&rgba, w);
                    JANELA_DA_TINTA_POR_PALAVRAS.with(|c| c.set(true));
                    j
                };
                let (pal, ser) = (com(true), com(false));
                assert_eq!(pal, ser, "w={w} tinta em {alfas:?}");
                if alfas.is_empty() {
                    assert_eq!(ser, PlaneReach::Untouched);
                } else {
                    vistas += 1;
                    assert!(matches!(ser, PlaneReach::Window(_)), "{alfas:?}: {ser:?}");
                }
            }
            assert_eq!(vistas, casos.len() - 1);
        }
    }
}
