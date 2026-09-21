//! ⭐⭐⭐⭐ **O ENQUADRAMENTO com que uma forma foi rasterizada** — irmão (`#[path]`) do
//! [`super`].
//!
//! ⛔⛔ **O defeito que ele cura** (report do dono, 2026-09-21, com foto): a porta de assar
//! escrevia o alvo INTEIRO, logo a peça ocupava, dentro dos texels do sprite, a mesma fracção que
//! ocupava **da altura do VIEWPORT** — e o sprite é um rectângulo *dentro* dele. O erro de escala
//! é `altura da vista ÷ altura do sprite no ecrã` e o de posição é a distância entre os dois
//! centros. A lei da cura (o frustum fora-de-eixo) vive na `ph2d-mesh-render`; aqui vive a
//! MEMÓRIA dela, porque a rota B re-rasteriza **por quadro** e um catavento sem esta memória
//! voltaria a encher o sprite no primeiro quadro em que o relógio andasse.
//!
//! # ⛔⛔ Por que ele é um TIPO PRÓPRIO e não o `Framing` da `ph2d-mesh-render`
//!
//! Esta crate é a fronteira que o runtime atravessa **sem o módulo 3D** — o cabeçalho do
//! [`super`] proíbe-a por escrito de arrastar o `wgpu`, os matcaps e o `imageio`. Um `use` daquele
//! tipo traria a crate inteira com ele. ⇒ dois tipos de dados puros e **uma** travessia, com gate
//! de ida-e-volta (`ph2d_app_sculpt3d::recorte`). *O precedente é o `ph2d_pose::pesos`, e a lei é
//! a mesma: a duplicação é honesta enquanto a porta é uma e tem gate.*
//!
//! ⚠️ **O corte deste ficheiro foi o tecto de LOC do irmão** (700), e a cura é CORTE — nunca uma
//! entrada no `FILE_OVERAGE_OK`.

/// ⭐⭐⭐ **COM QUE PEDAÇO DA VISTA estes canais foram rasterizados** — o enquadramento
/// CONGELADO no gesto de assar.
///
/// ⛔⛔ **Ela é um punhado de `f32` e não o [`ph2d_mesh_render::Framing`], e a razão é a promessa
/// do cabeçalho desta crate:** *«o runtime lê os canais SEM o módulo 3D»*. Importar o tipo do
/// renderizador de malhas arrastaria `wgpu`, matcaps e o `imageio` para dentro de quem só precisa
/// de multiplicar `base × luz` — e um objecto reaberto num binário sem escultura deixaria de
/// acender. ⭐ A duplicação é **deliberada e gateada**: quem converte é
/// [`ph2d_app_sculpt3d`], numa porta só, com prova de ida-e-volta.
///
/// ⚠️ `origin`/`size` são FRACÇÕES da vista (`y` do topo), e `aspect` é o da vista INTEIRA —
/// ver [`ph2d_mesh_render::ViewRegion`], que é o tipo do outro lado da conversão.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Recorte {
    /// A razão largura/altura da vista inteira.
    pub aspect: f32,
    /// O canto superior-esquerdo, em fracção da vista.
    pub origin: [f32; 2],
    /// A extensão, em fracção da vista.
    pub size: [f32; 2],
}
