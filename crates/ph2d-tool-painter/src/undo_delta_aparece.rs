//! **A janela do plano que APARECE, achada no COMMIT** (B08c.M2) — filho de [`super`] (`#[path]`) para ler
//! os campos de [`PlaneWindow`].
//!
//! O 1.º traço Impasto (ou aguada) numa camada faz `heights`/`covers`/`mats` (ou o vidro) aparecerem
//! (`OnlyAfter`). O lado ausente é o neutro, então o Ctrl+Z alcança só os texels fora do neutro — e achá-los
//! varria os planos INTEIROS a cada pergunta (4K: ~4 ms de largura de banda). Mas o passo só escreveu dentro
//! da janela declarada (a mesma confiança do [`super::StoredPlane::split_tiled`], conferida em debug), logo a
//! busca restrita a ela dá a mesma janela: faz-se uma vez, no commit, e o Ctrl+Z só a lê.

use super::confine::{PlaneReach, janela_fora_do_neutro};
use super::{PlaneWindow, StoredEntry, StoredMap};
use crate::layers::LayerId as RtLayerId;

impl<T> StoredMap<T> {
    /// Guarda a janela fora do neutro de cada plano `OnlyAfter`, buscada só dentro de `hint`.
    pub(crate) fn guarda_janelas_que_aparecem(
        &mut self,
        hint: Option<PlaneWindow>,
        linha_neutra: impl Fn(&[T]) -> bool + Copy,
        neutro: impl Fn(&T) -> bool + Copy,
    ) {
        let Some(h) = hint else { return };
        for (k, e) in &self.entries {
            let StoredEntry::OnlyAfter(p) = e else {
                continue;
            };
            let Some(w) = h.fit_to(p.len()) else {
                continue;
            };
            let r = janela_dentro(p, w, linha_neutra, neutro);
            debug_assert_eq!(
                r,
                janela_fora_do_neutro(p, w.stride, linha_neutra, neutro),
                "o plano que aparece tem texels fora do neutro FORA da janela declarada — o Ctrl+Z \
                 confinado deixaria relevo na tela (B08c.M2)"
            );
            self.aparece.insert(*k, r);
        }
    }

    /// A janela guardada no commit para a camada `k`, se o seu `stride` for o pedido.
    pub(crate) fn janela_que_aparece(&self, k: RtLayerId, stride: usize) -> Option<PlaneReach> {
        self.aparece.get(&k).copied().filter(|r| match r {
            PlaneReach::Window(w) => w.stride == stride,
            _ => true,
        })
    }
}

thread_local! {
    /// **A chave do A×B do relevo que nasce** (F1p, B44.8): `true` = os três planos do 1.º traço Impasto
    /// guardam só a janela ([`StoredEntry::Aparece`]); `false` = o caminho de antes, que segura o plano
    /// inteiro (o oráculo).
    static RELEVO_APARECE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// A chave [`RELEVO_APARECE`] deste fio.
#[must_use]
pub(crate) fn relevo_aparece() -> bool {
    RELEVO_APARECE.with(std::cell::Cell::get)
}

/// Liga/desliga [`RELEVO_APARECE`] neste fio (o gate intercala as duas rotas).
#[cfg(test)]
pub(crate) fn set_relevo_aparece(on: bool) {
    RELEVO_APARECE.with(|c| c.set(on));
}

#[cfg(test)]
#[path = "undo_delta_aparece_tests.rs"]
mod tests;

impl<T: Copy + PartialEq> StoredMap<T> {
    /// **O plano que aparece deixa de ser segurado** (ADR-0205, o 2.º traço; o relevo no F1p): cada
    /// `OnlyAfter` com a janela já achada ([`Self::guarda_janelas_que_aparecem`]) troca o `Arc` vivo por essa
    /// janela. `neutro` é o valor do plano fora dela (o vidro e a altura/cobertura: zeros; o material: o
    /// neutro) — o mesmo que a busca da janela usou, senão o refazer recriaria outro plano.
    pub(crate) fn larga_os_planos_que_aparecem(&mut self, neutro: T) {
        for (k, e) in &mut self.entries {
            let StoredEntry::OnlyAfter(p) = e else {
                continue;
            };
            let win = match self.aparece.get(k) {
                Some(PlaneReach::Untouched) => None,
                Some(PlaneReach::Window(w)) if w.plane_len == p.len() => Some(*w),
                _ => continue,
            };
            let after = win.map_or_else(Box::default, |w| w.extract(p));
            debug_assert!(
                recria(p.len(), win, &after, neutro) == **p,
                "o plano que aparece não é o neutro fora da janela — o refazer recriaria outro (ADR-0205)"
            );
            *e = StoredEntry::Aparece {
                len: p.len(),
                win,
                after,
                neutro,
            };
        }
    }
}

/// O plano de [`StoredEntry::Aparece`]: o neutro com a janela escrita.
pub(super) fn recria<T: Copy>(
    len: usize,
    win: Option<PlaneWindow>,
    after: &[T],
    neutro: T,
) -> Vec<T> {
    let mut v = vec![neutro; len];
    if let Some(w) = win {
        w.blit(after, &mut v);
    }
    v
}

/// [`janela_fora_do_neutro`] restrita às linhas e colunas de `w`; a janela devolvida é do plano inteiro.
fn janela_dentro<T>(
    p: &[T],
    w: PlaneWindow,
    linha_neutra: impl Fn(&[T]) -> bool,
    neutro: impl Fn(&T) -> bool,
) -> PlaneReach {
    let linha = |r: usize| &p[r * w.stride + w.col..r * w.stride + w.col + w.cols];
    let rows = w.row..w.row + w.rows;
    let Some(row) = rows.clone().find(|&r| !linha_neutra(linha(r))) else {
        return PlaneReach::Untouched;
    };
    let last = rows.rev().find(|&r| !linha_neutra(linha(r))).unwrap_or(row);
    let (mut col, mut end) = (w.stride, 0usize);
    for r in row..=last {
        let l = linha(r);
        if let Some(c) = l.iter().position(|v| !neutro(v)) {
            col = col.min(w.col + c);
            end = end.max(w.col + l.iter().rposition(|v| !neutro(v)).unwrap_or(c) + 1);
        }
    }
    PlaneReach::Window(PlaneWindow {
        row,
        rows: last - row + 1,
        col,
        cols: end.saturating_sub(col),
        stride: w.stride,
        plane_len: p.len(),
    })
}

#[cfg(test)]
mod sonda {
    use super::super::confine::Confine;
    use super::super::{PlaneWindow, StoredEntry, StoredMap};
    use std::sync::Arc;
    use std::time::Instant;

    /// Sonda B08c.M2 (`--profile smoke -- --ignored`): a pergunta do Ctrl+Z sobre os três planos 4K que
    /// aparecem, varrendo (A, o de antes) × lendo a janela do commit (B); e o custo da busca no commit.
    #[test]
    #[ignore = "sonda de tempo"]
    fn sonda_ctrl_z_do_plano_que_aparece_4k() {
        let (w, h) = (3840usize, 2160usize);
        let (x0, y0, bw, bh) = (1664usize, 949usize, 448usize, 182usize);
        let mut hs = vec![0f32; w * h];
        let mut cs = vec![0u8; w * h];
        for y in y0 + 10..y0 + bh - 10 {
            for x in x0 + 10..x0 + bw - 10 {
                hs[y * w + x] = 0.5;
                cs[y * w + x] = 200;
            }
        }
        let hint = Some(PlaneWindow::for_test(y0, bh, x0, bw, w));
        let (mut ha, mut ca) = (StoredMap::<f32>::default(), StoredMap::<u8>::default());
        let l7 = crate::layers::LayerId(7);
        ha.entries.insert(l7, StoredEntry::OnlyAfter(Arc::new(hs)));
        ca.entries.insert(l7, StoredEntry::OnlyAfter(Arc::new(cs)));
        let (lh, nh) = (
            |l: &[f32]| l.iter().fold(0u32, |a, v| a | v.to_bits()) == 0,
            |v: &f32| v.to_bits() == 0,
        );
        let (lc, nc) = (
            |l: &[u8]| l.iter().fold(0u8, |a, &b| a | b) == 0,
            |v: &u8| *v == 0,
        );
        let pergunta = |ha: &StoredMap<f32>, ca: &StoredMap<u8>| {
            let t = Instant::now();
            let mut c = Confine::new(w as u32);
            c.add_relevo(ha, lh, nh);
            c.add_relevo(ca, lc, nc);
            (t.elapsed().as_secs_f64() * 1e3, c.finish())
        };
        let (mut a, mut b, mut g) = (f64::MAX, f64::MAX, f64::MAX);
        for _ in 0..5 {
            ha.aparece.clear();
            ca.aparece.clear();
            let (ta, ra) = pergunta(&ha, &ca);
            let t = Instant::now();
            ha.guarda_janelas_que_aparecem(hint, lh, nh);
            ca.guarda_janelas_que_aparecem(hint, lc, nc);
            g = g.min(t.elapsed().as_secs_f64() * 1e3);
            let (tb, rb) = pergunta(&ha, &ca);
            assert_eq!(ra, rb);
            (a, b) = (a.min(ta), b.min(tb));
        }
        eprintln!(
            "B08c.M2 sonda 4K (heights+covers): Ctrl+Z varre {a:.3} ms × lê {b:.4} ms; commit busca {g:.3} ms"
        );
    }
}
