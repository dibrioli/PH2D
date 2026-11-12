//! **A silhueta do carimbo, guardada** (W3b): o depósito e o pente do relevo dos fios leem a MESMA silhueta nas
//! mesmas células, e cada leitura são as sub-amostras do AA — recalculá-la no pente era o custo inteiro do relevo
//! (doc 50_medicao §4: o move a 103 px 1,07 → 2,37 ms). A silhueta é pura: guardá-la dá o traço ao byte.
//! B14.3: a caixa é REAPROVEITADA entre dabs (a do fio que pousa) e marcada por geração — o `nova` não escreve
//! as ~6 MB da caixa do Flat a 240 a cada sub-dab.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Os vetores de uma caixa e a geração da última que os usou.
#[derive(Default)]
struct Caixa {
    geracao: u32,
    marca: Vec<AtomicU32>,
    v: Vec<AtomicU64>,
}

thread_local! {
    /// A caixa do último dab pousado neste fio, à espera do próximo.
    static GUARDADA: RefCell<Caixa> = RefCell::new(Caixa::default());
    /// A da silhueta BASE (sem a ponta), que a bilinear da ponta das cerdas lê quatro vezes por célula.
    static GUARDADA_BASE: RefCell<Caixa> = RefCell::new(Caixa::default());
}

type Guarda = &'static std::thread::LocalKey<RefCell<Caixa>>;

/// A silhueta de UM dab na caixa `[x0, x0 + w) × [y0, y0 + h)` (células), calculada na 1.ª leitura de cada célula.
/// Atómica: o depósito lê-a por linhas em paralelo; duas threads que calculem a mesma célula guardam o MESMO valor.
/// A célula vale se a sua `marca` é a `geracao` deste dab (escrita DEPOIS do valor, `Release`).
pub(super) struct SilMemo {
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    c: Caixa,
    guarda: Guarda,
}

impl SilMemo {
    /// A caixa do pente (`Engine::pentear_relevo`): a meia-extensão `meia` (células) à volta de `(x, y)`.
    pub(super) fn nova(x: f64, y: f64, meia: [f64; 2]) -> Self {
        Self::na(&GUARDADA, x, y, meia)
    }

    /// A caixa da silhueta BASE (B14c): a guardada à parte, para conviver no mesmo dab com a de [`Self::nova`].
    pub(super) fn nova_base(x: f64, y: f64, meia: [f64; 2]) -> Self {
        Self::na(&GUARDADA_BASE, x, y, meia)
    }

    fn na(guarda: Guarda, x: f64, y: f64, meia: [f64; 2]) -> Self {
        let (ax, ay) = (meia[0].ceil() as i32 + 1, meia[1].ceil() as i32 + 1);
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        let (w, h) = (2 * ax + 1, 2 * ay + 1);
        let mut c = guarda
            .try_with(|g| std::mem::take(&mut *g.borrow_mut()))
            .unwrap_or_default();
        let n = (w * h) as usize;
        if c.v.len() < n {
            #[cfg(test)]
            INICIADAS.with(|k| k.set(k.get() + n - c.v.len()));
            c.marca.resize_with(n, || AtomicU32::new(0));
            c.v.resize_with(n, || AtomicU64::new(0));
        }
        c.geracao = c.geracao.wrapping_add(1);
        if c.geracao == 0 {
            c.marca.iter_mut().for_each(|m| *m.get_mut() = 0);
            c.geracao = 1;
        }
        Self {
            x0: cx - ax,
            y0: cy - ay,
            w,
            h,
            c,
            guarda,
        }
    }

    /// A silhueta em `(cx, cy)`: a guardada, ou `sil` (e guarda-a); fora da caixa, `sil` sem guardar.
    pub(super) fn ou(&self, cx: i32, cy: i32, sil: impl Fn(i32, i32) -> f64) -> f64 {
        let (i, j) = (cx - self.x0, cy - self.y0);
        if i < 0 || j < 0 || i >= self.w || j >= self.h {
            crate::wet_diag::conta_sil(4, 1);
            return sil(cx, cy);
        }
        let k = (j * self.w + i) as usize;
        if self.c.marca[k].load(Ordering::Acquire) == self.c.geracao {
            return f64::from_bits(self.c.v[k].load(Ordering::Relaxed));
        }
        let v = sil(cx, cy);
        self.c.v[k].store(v.to_bits(), Ordering::Relaxed);
        self.c.marca[k].store(self.c.geracao, Ordering::Release);
        v
    }
}

/// O FIO da última célula lida em cada linha de um dab: a ponta, as cerdas e a cor dos fios pedem-no na MESMA célula,
/// uma após outra (o motor lê silhueta → padrão → grão → cor por célula). A chave `cx` vai no mesmo `u64` do fio:
/// uma leitura nunca casa o fio de outra célula, em qualquer ordem de threads — o fio é puro, então ao bit.
pub(super) struct FioPorLinha {
    y0: i32,
    v: Vec<Linha>,
}

/// Uma linha por linha de cache: as linhas vizinhas vão em threads diferentes (sem isto, a mesma linha de cache saltava
/// entre núcleos a cada célula).
#[repr(align(64))]
struct Linha(AtomicU64);

impl FioPorLinha {
    const VAZIA: u64 = (i32::MIN as u32 as u64) << 32;

    /// As linhas `y ± meia` (células).
    pub(super) fn nova(y: f64, meia: f64) -> Self {
        let a = meia.ceil() as i32 + 1;
        Self {
            y0: y.round() as i32 - a,
            v: (0..2 * a + 1)
                .map(|_| Linha(AtomicU64::new(Self::VAZIA)))
                .collect(),
        }
    }

    /// O fio de `(cx, cy)`: o guardado na linha, ou `fio()` (e guarda-o); fora das linhas, `fio()`.
    #[inline]
    pub(super) fn ou(&self, cx: i32, cy: i32, fio: impl FnOnce() -> i32) -> i32 {
        let Some(s) = usize::try_from(cy - self.y0)
            .ok()
            .and_then(|j| self.v.get(j))
        else {
            return fio();
        };
        let s = &s.0;
        let v = s.load(Ordering::Relaxed);
        if (v >> 32) as u32 == cx as u32 {
            return v as u32 as i32;
        }
        let f = fio();
        s.store(
            (u64::from(cx as u32) << 32) | u64::from(f as u32),
            Ordering::Relaxed,
        );
        f
    }
}

impl Drop for SilMemo {
    fn drop(&mut self) {
        let c = std::mem::take(&mut self.c);
        let _ = self.guarda.try_with(|g| *g.borrow_mut() = c);
    }
}

#[cfg(test)]
thread_local! {
    /// Células que o `nova` deste fio inicializou (escritas antes da 1.ª leitura).
    pub(super) static INICIADAS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
#[path = "sil_memo_tests.rs"]
mod tests;
