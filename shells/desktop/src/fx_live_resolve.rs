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
///
/// ⚠️ O deslocamento é arredondado a pixel INTEIRO — o passe amostra o halo por `textureLoad`, e
/// posição sub-pixel numa sombra não é algo que se veja (a textura já é alinhada ao pixel da tela).
#[must_use]
pub(crate) fn resolve_ops(filter: &VecFilter, camera: Affine) -> Vec<FxOpGpu> {
    let [a, b, c, d, _, _] = camera.as_coeffs();
    let cam_scale = ((a * a + b * b).sqrt() + (c * c + d * d).sqrt()) as f32 * 0.5;
    filter
        .ops
        .iter()
        .filter(|o| o.is_active())
        .map(|o| {
            let (ox, oy) = (f64::from(o.offset[0]), f64::from(o.offset[1]));
            // A rampa do Gradient Map, ordenada e empacotada pela porta única do componente — o
            // shader assume ordenado, e ordenar aqui à mão seria a segunda resposta que diverge.
            let (stops, stop_pos, stop_count) = o.ramp_for_device();
            FxOpGpu {
                kind: o.kind,
                sigma_px: (o.radius * cam_scale).max(0.0),
                offset_px: [
                    (a * ox + c * oy).round() as i32,
                    (b * ox + d * oy).round() as i32,
                ],
                tint: o.color,
                // A SEGUNDA ponta da rampa, pelo MESMO caminho da primeira: também não atravessa a
                // câmara (uma cor não é um comprimento), pela razão que os três do ajuste já dão
                // mais abaixo.
                tint_b: o.color_b,
                opacity: o.opacity,
                mode: o.mode,
                // ⚠️ **`blend_code`, não `blend`** — é a metade de HONRAR da porta única
                // `FxOp::takes_blend` (a de OFERECER é do painel). Um arquivo cujo degrau carrega
                // uma lei de um tipo que deixou de a tomar desenharia uma mistura que a UI não
                // mostra; aqui ela vira Normal, e o dispositivo nunca vê um número órfão.
                blend: o.blend_code(),
                stops,
                stop_pos,
                stop_count,
                // O tamanho das ondulações atravessa a mesma conversão do raio — é ela que torna
                // o padrão zoom-invariante (o `noise_p` do shader divide por este número, e o
                // numerador também escala com o zoom, então ele cancela).
                noise_scale_px: (o.scale * cam_scale).max(0.0),
                // ⚠️ **`detail_clamped`, não `detail`** — a metade de HONRAR da porta única, como
                // o `blend_code`: um arquivo com detalhe 0 (ou 200) desenharia um laço vazio (ou
                // caro) que a UI não oferece.
                detail: o.detail_clamped(),
                seed: o.seed,
                // O crescimento atravessa a MESMA conversão do raio (mundo → pixels de tela pelo
                // zoom): engordar 0,06 tem de engordar a mesma fração da forma em qualquer escala.
                // ⚠️ **Sem `max(0.0)`** — aqui o sinal É a operação.
                grow_px: o.grow * cam_scale,
                // ⚠️ **Os três do ajuste NÃO atravessam a câmara, e é a diferença que importa:**
                // eles não são comprimentos. Uma matiz é um ÂNGULO e a saturação/brilho são
                // frações — dar zoom não pode mudar a cor de nada. Multiplicá-los pelo
                // `cam_scale` seria o mesmo erro que dividir o raio por ele.
                hue: o.hue,
                sat: o.sat,
                bright: o.bright,
            }
        })
        .collect()
}
