//! **A tradução de um degrau AUTORADO para um degrau de DISPOSITIVO** — irmão de
//! [`super::fx_live`] pelo teto de LOC da shell, e o corte é por responsabilidade: aquele arquivo
//! decide *QUANDO e ONDE* as imagens são cozidas (o memo, os lotes, as texturas de saída), este
//! decide *o que um degrau VIRA* ao cruzar a fronteira para o device.
//!
//! É a fronteira onde as UNIDADES mudam: comprimentos de MUNDO viram pixels de tela pela câmara, a
//! matiz em voltas vira o que o kernel lê, a rampa é ordenada e empacotada pela porta única do
//! componente. ⚠️ **Cores NÃO cruzam a câmara** — uma cor não é um comprimento.

use ph2d_ecs::VecFilter;
use ph2d_render::FxOpGpu;
use ph2d_vector::Affine;

/// **A pilha AUTORADA resolvida em pixels de tela.** Os degraus desligados caem aqui (a pilha os
/// SALTA, como a de geometria salta um `FxEntry` desarmado), então uma pilha toda desligada devolve
/// vazio e a forma sai nua.
#[must_use]
pub fn resolve_ops(filter: &VecFilter, camera: Affine) -> Vec<FxOpGpu> {
    let [a, b, c, d, _, _] = camera.as_coeffs();
    // A conta é a porta ÚNICA do `ph2d-render` — o Motion (`fx.filter`) chama a mesma.
    filter
        .ops
        .iter()
        .filter(|o| o.is_active())
        .map(|o| FxOpGpu::resolve(o, [a, b, c, d]))
        .collect()
}
