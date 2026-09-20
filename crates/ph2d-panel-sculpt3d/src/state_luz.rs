//! **COM QUE LUZ o barro é mostrado** — os três modos da fileira *Light*.
//!
//! ⚠️ Irmão (`#[path]`) do [`super`], cortado pelo tecto de LOC do painel e pelo
//! ASSUNTO — a mesma forma dos `state_modes` e `state_channel`.

/// ⭐⭐⭐⭐ **COM QUE LUZ o barro é mostrado** — os três modos que a fileira
/// *Light* oferece (report do dono, 2026-09-20: *«precisamos como no blender
/// modos de shaders além do matcap para pintar»*).
///
/// ⛔⛔ **É uma SEGUNDA definição do conceito, e a duplicação é DELIBERADA:**
/// este painel não conhece o renderizador (ele não depende de `ph2d-mesh-render`
/// e não vai passar a depender — é UI, e aquela crate arrasta o `wgpu`), logo o
/// tipo do device não é nomeável aqui. *O que torna as duas honestas é a PONTE
/// ter gate de ida-e-volta* (`ph2d-app-sculpt3d`, `panel.rs`), que é o mesmo
/// desenho da `Deformacao::modo_e_inversao`.
///
/// ⚠️ **O `Flat` é o modo de PINTAR:** sem luz, a cor que o artista escolheu é a
/// cor que ele vê — e é por isso que ele é o primeiro chip da fileira.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LightMode {
    /// Sem luz nenhuma: o albedo cru.
    Flat,
    /// As lâmpadas do documento.
    #[default]
    Rig,
    /// O matcap `i` — a luz do olho.
    Matcap(u8),
}

impl LightMode {
    /// **A OPÇÃO DA FILEIRA que este modo é** — `0` plano, `1` rig, `2 + i` o
    /// matcap `i`.
    ///
    /// ⚠️ **Uma porta e não a aritmética escrita duas vezes:** o pintor precisa
    /// dela para dizer qual chip está aceso e o despacho para a inverter, e as
    /// duas cópias divergiriam no dia do quarto modo — que é exactamente o que
    /// esta wave acabou de ser.
    #[must_use]
    pub const fn option_index(self) -> usize {
        match self {
            Self::Flat => 0,
            Self::Rig => 1,
            Self::Matcap(i) => 2 + i as usize,
        }
    }

    /// A inversa de [`Self::option_index`] — o que o clique na fileira escolheu.
    #[must_use]
    pub fn from_option_index(i: usize) -> Self {
        match i {
            0 => Self::Flat,
            1 => Self::Rig,
            n => Self::Matcap(u8::try_from(n - 2).unwrap_or(u8::MAX)),
        }
    }
}
