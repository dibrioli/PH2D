//! **Edit ▸ FX ▸ Convolution Matrix** (F9): por canal, `clamp(trunc0(Σ w·v / div) + bias)` — a
//! divisão trunca para ZERO (inteiro C) e o bias soma-se DEPOIS; borda replicada (*Tiled* = dá a
//! volta). Lei e matrizes: `oracle_trial_convolution.json` (rodadas 1 e 2), lista em [`KERNELS`].
//!
//! O alfa fica como estava: as entradas do Trial são opacas e nenhuma mediu o kernel sobre o alfa.

use super::{Edge, Filter, Source};

pub use super::kernels::KERNELS;

/// Uma matriz: `weights[j·w + i]` é o peso que um impulso em `p` põe na célula
/// `p + (i − cx, j − cy)` (a RESPOSTA ao impulso, como o Trial a mostra).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kernel {
    pub name: &'static str,
    pub w: u32,
    pub h: u32,
    pub cx: u32,
    pub cy: u32,
    pub div: i32,
    pub bias: i32,
    pub weights: &'static [i32],
}

/// O kernel de nome `name` da lista.
#[must_use]
pub fn kernel(name: &str) -> Option<&'static Kernel> {
    KERNELS.iter().find(|k| k.name == name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Convolution {
    pub kernel: &'static Kernel,
    pub edge: Edge,
}

impl Filter for Convolution {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        let k = self.kernel;
        let mut sum = [0i32; 3];
        for j in 0..k.h {
            for i in 0..k.w {
                let wt = k.weights[(j * k.w + i) as usize];
                if wt == 0 {
                    continue;
                }
                let (dx, dy) = (i as i32 - k.cx as i32, j as i32 - k.cy as i32);
                let c = src.at_edge(x - dx, y - dy, self.edge);
                for (s, v) in sum.iter_mut().zip(c) {
                    *s += wt * i32::from(v);
                }
            }
        }
        let old = src.at_edge(x, y, self.edge);
        let ch = |s: i32| ((s / k.div.max(1)) + k.bias).clamp(0, 255) as u8;
        [ch(sum[0]), ch(sum[1]), ch(sum[2]), old[3]]
    }
}
