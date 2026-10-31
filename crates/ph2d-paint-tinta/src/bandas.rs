//! Uma grandeza por banda espectral. `N = 3` são R, G, B LINEARES; `N = 8` é genérico (ver o doc
//! F4_1 §bandas: a calibração a 8 está em aberto).

/// Valores por banda (reflectância, `𝒦`, `𝒮`…).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bandas<const N: usize>(pub [f32; N]);

impl<const N: usize> Bandas<N> {
    /// Todas as bandas com o mesmo valor.
    #[must_use]
    pub const fn uniforme(x: f32) -> Self {
        Self([x; N])
    }

    /// Aplica `f` banda a banda, em ordem.
    #[must_use]
    pub fn mapa(self, mut f: impl FnMut(usize, f32) -> f32) -> Self {
        let mut out = [0.0; N];
        for (i, o) in out.iter_mut().enumerate() {
            *o = f(i, self.0[i]);
        }
        Self(out)
    }
}

impl<const N: usize> Default for Bandas<N> {
    fn default() -> Self {
        Self([0.0; N])
    }
}
