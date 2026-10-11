//! ⭐⭐⭐ **O COLISOR QUE UM OBJECTO DECLARA** — o botão `Collide` do cartão (doc 115 §33, decisão do
//! dono de 2026-10-11: *«automático é melhor»*). Ligado, a peça declara a caixa dela e o passe do
//! fim do cozimento separa-a sozinho; não há nada para ligar no `Output`.
//!
//! # A lei: a caixa do objecto é o QUADRADO UNITÁRIO (doc 115 §12.1)
//!
//! A porta que lê a declaração (`ph2d_contact::declarado`) converte de GEOMETRIA para MUNDO
//! multiplicando as meias pelo `size` da própria linha (`meia_mundo = meia · |size|`), e a corrente
//! do objecto já traz esse `size`. ⇒ declarar `size / 2` daria `size² / 2`; a declaração certa é
//! [`MEIA_DO_OBJECTO`], constante. Um `motion.scale` a jusante encolhe a peça **e** o colisor.
//!
//! # As ausências são leis
//!
//! - Sem `collider_offset`: o quadro é centrado no `P`.
//! - Sem `collider` (raio): um objecto é um quadro, e na porta a caixa ganha.
//! - Sem material nem `inv_inertia`: o cartão não os autora, e a ausência é a declaração.
//!
//! ⚠️ É a caixa do QUADRO, não da tinta (uma sprite com margem transparente declara a margem), e a
//! folha de um grupo viaja com `rot = 0` (o limite v1 do médio).

use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, Column, Stream};

/// **As meias extensões do colisor de um objecto, em unidades de GEOMETRIA** — `0,5` e não `1`
/// (a convenção da `source.shape`, que vive em raio 1): a aparência de um objecto ocupa `size`.
pub const MEIA_DO_OBJECTO: [f32; 2] = [0.5, 0.5];

/// **A corrente do objecto com a caixa dele declarada.** Uma corrente vazia sai como entrou: uma
/// coluna de zero linhas não é uma declaração.
#[must_use]
pub fn com_colisor(s: Stream) -> Stream {
    let n = s.count();
    if n == 0 {
        return s;
    }
    s.with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![MEIA_DO_OBJECTO; n]))
}
