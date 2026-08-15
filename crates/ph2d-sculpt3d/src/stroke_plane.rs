//! **O PLANO que quatro verbos ajustam** — o estimador da superfície sob o dab,
//! e as duas metades do culling que a referência aplica a ele.
//!
//! ⚠️ **Filho (`#[path]`) do [`super`] pela MESMA razão do irmão
//! [`super::target`]:** estes métodos leem o `pre` congelado e os slots, e um
//! módulo irmão os obrigaria a virar `pub(crate)`.
//!
//! ⚠️ **O corte deste arquivo é o `e` que o cabeçalho do irmão carregava** —
//! *"o alvo de cada verbo, **e** o plano que quatro deles ajustam"*. São dois
//! assuntos: para onde um verbo APONTA difere entre os dezasseis; que forma a
//! superfície TEM é uma pergunta só, com um estimador só, que Flatten, Fill,
//! Scrape e Clay partilham.

use super::*;

/// O plano ajustado à pegada de um dab.
///
/// ⚠️ **Inclinado, nunca horizontal** — um ajuste horizontal *cava uma cratera
/// na encosta* em vez de achatá-la (lição paga no `plane.rs` do Painter 2D).
/// O estimador é a média ponderada pelo falloff das posições e das normais da
/// pegada, que é o `calc_area_normal_and_center` do Blender; ele difere de um
/// ajuste por mínimos quadrados de verdade numa sela, e a divergência está
/// registrada aqui em vez de escondida.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PlaneFit {
    pub(super) point: [f32; 3],
    pub(super) normal: [f32; 3],
}

impl SculptStroke {
    pub(super) fn fit_plane(&self, mesh: &Mesh, brush: &Brush, dab: &Dab) -> PlaneFit {
        // ⚠️ **O conjunto FRONTAL, e é a metade que o original faz
        // INCONDICIONALMENTE.** O `getFrontVertices` (`SculptBase.js:206-221`)
        // filtra por `n · eyeDir <= 0`, e o `Brush.js:32-34` / `Flatten.js:25-27`
        // o consomem sem perguntar a ninguém — é ele que decide a DIREÇÃO do
        // Draw e o PLANO do Flatten.
        //
        // ⚠️ **A outra metade do culling — filtrar o que se MOVE — NÃO entra.**
        // Ela é um checkbox do usuário, `_culling = false` por default em dez
        // tools (`GuiSculptingTools.js:62`), e portá-la ligada seria divergir
        // com a ferramenta em silêncio (livro-razão §A).
        //
        // Sem isto, um dab perto da silhueta ajusta o plano com vértices que
        // olham para o OUTRO lado, e o Draw empurra numa direção que o artista
        // não vê.
        let mut fit = self.fit_plane_over(mesh, brush, dab, true);
        if fit.is_none() {
            // Pegada inteiramente de costas (um dab que pegou só o outro lado da
            // silhueta): sem frontais não há o que cullar, e recusar aqui seria
            // devolver um plano NaN. A pegada inteira é a melhor resposta que
            // existe, e é a que havia antes desta fatia.
            fit = self.fit_plane_over(mesh, brush, dab, false);
        }
        fit.unwrap_or(PlaneFit {
            point: dab.center,
            normal: [0.0, 1.0, 0.0],
        })
    }

    /// O ajuste sobre a pegada, opcionalmente só nos vértices que olham para o
    /// olho. `None` = ninguém pesou (conjunto vazio, ou todo peso zero).
    fn fit_plane_over(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        front_only: bool,
    ) -> Option<PlaneFit> {
        // ⚠️ **O PLANO É O DA REFERÊNCIA, e o produto CHAMA os kernels
        // portados** (`SculptBase.js:224-261`) em vez de os re-derivar. Até
        // 2026-08-11 esta função tinha uma soma própria, ponderada pelo
        // **FALLOFF** e lida do `pre` congelado, com o racional escrito ao lado:
        // *"o plano descreve a superfície sob o pincel, e força/pressão/máscara
        // dizem o quanto agir sobre ela, não que forma ela tem"*. O racional é
        // defensável e o **preço estava medido**: a NORMAL saía idêntica
        // (`cos 1,000000`) e o CENTRO ficava **0,029 fora, inteiramente ao
        // longo da normal** — `5,8 %` do raio.
        //
        // ⚠️ **E é por isso que o Draw não sentia e o Flatten sentia.** O Draw
        // consome só a direção (media `1,01×`); quem consome o PONTO entra pelo
        // `signed_distance`, que enxerga exatamente a componente ao longo da
        // normal — o Flatten media `0,54×` e o Clay `1,74×`.
        //
        // ⚠️ **A ponderação é a MÁSCARA, não o falloff** (`mAr[ind + 2]`).
        //
        // ⚠️ **E a leitura é a VIVA, desde 2026-08-12 — ela era do `pre`
        // CONGELADO, e essa era a razão de o Accumulate ser INERTE nos quatro
        // verbos de plano.** A nota que defendia o congelamento dizia *"a
        // divergência é a mesma que o `Grip::Stamp` já carrega"*, e isso
        // **deixou de ser verdade** quando a metade 2 pôs o carimbo a compor
        // sobre o vivo: a premissa mudou sob os pés da nota, que sobreviveu ao
        // fato. Medido pela porta do produto, com o plano congelado o
        // interruptor valia **1,04× no Clay** (0,99× no Flatten, 1,00× no
        // Scrape) contra 1,74× no Draw — *o barro subia até o plano do pen-down
        // e PARAVA*, e nenhum valor do checkbox mudava isso.
        //
        // A referência recomputa os dois por dab sobre a malha VIVA
        // (`SculptBase.areaNormal` lê `getNormals()`, `areaCenter` lê
        // `getVertices()`, e `Flatten.stroke` os chama a cada `stroke`), e é
        // isso que faz um Clay CONSTRUIR: o plano sobe com a tinta.
        //
        // ⚠️ **O medo que a nota antiga registrava — *"o Flatten perseguindo a
        // superfície que ele achata"* — não se realiza, e é geometria:** o
        // Flatten PROJETA em direção ao plano, e uma projeção preserva o
        // centroide, então o plano ajustado sobre a pegada não corre. Quem sobe
        // é o Clay, cujo plano é deslocado `raio · 0,1` — e subir é o que ele
        // existe para fazer.
        let front = |v: u32| {
            let n = mesh.normals()[v as usize];
            !front_only || n[0] * dab.eye[0] + n[1] * dab.eye[1] + n[2] * dab.eye[2] <= 0.0
        };
        // O peso da referência: `1` é livre. O nosso `DEFAULT_MASK` é o oposto,
        // e o `free_weight` é a porta única dessa conversão.
        let free = |v: u32| {
            if front(v) {
                f64::from(crate::mask_ops::free_weight(
                    self.base_mask[self.slot[v as usize] as usize],
                ))
            } else {
                0.0
            }
        };
        // ⚠️ **DE QUE SUPERFÍCIE o plano é ajustado — e a resposta é do VERBO,
        // porque é da REFERÊNCIA dele.**
        //
        // O SculptGL recomputa sobre a malha VIVA a cada `stroke`
        // (`SculptBase.areaNormal` lê `getNormals()`, `areaCenter` lê
        // `getVertices()`), e é o que os quatro verbos de plano dele fazem. O
        // Blender ramifica: o
        // `sculpt.cc::calc_area_normal_and_center_node_mesh` abre com
        //
        // ```text
        // if (ss.cache && !ss.cache->accum) { ... orig_positions / orig_normals ... return; }
        // ```
        //
        // ou seja **com o Accumulate desligado — o default — ele lê o pen-down
        // CONGELADO**, e é isso que faz o barro subir até o plano e PARAR.
        //
        // ⚠️ **Ler o vivo na faixa custou TRÊS sintomas medidos**, cada um numa
        // wave diferente: o crescimento sub-linear que nunca fecha (§7.21) · a
        // divergência entre os dois passes de simetria quando eles se tocam
        // (§7.22, 1,77% residual) · e a faixa a **não saturar** com a força em
        // `1,0` — ela ultrapassa o plano, o plano persegue, e o auto-limite
        // desaparece.
        //
        // ⚠️ **É a mesma lei da [`crate::RefMode::kernel_for`], um andar acima:**
        // uma referência só governa as ferramentas que ela TEM.
        // ⚠️ **E o [`Verb::ClayThumb`] entra na MESMA lista, porque a regra é da
        // REFERÊNCIA e não do verbo:** o `clay_thumb.cc` chama o mesmo
        // `calc_brush_plane`, então herda o `!accum ⇒ orig` acima. Para ele só
        // a NORMAL muda de fonte (o plano dele passa pelo centro do DAB, não
        // pelo centro de área) — e é a normal congelada que impede a base da
        // inclinação de perseguir o barro que ela própria moveu.
        //
        // ⚠️ **O [`Verb::Blob`] fica FORA, e é decisão medida-antes-de-mexer:**
        // ele também é do Blender e também ajusta um plano, mas hoje lê o vivo,
        // e trocá-lo mudaria o desenho de um verbo que esta wave não toca. Quem
        // o quiser dentro traz a medição junto.
        let live = !matches!(brush.verb, Verb::ClayStrips | Verb::ClayThumb) || brush.accumulate;
        let nrm_of = |v: u32| {
            if live {
                mesh.normals()[v as usize]
            } else {
                self.base_nrm[self.slot[v as usize] as usize]
            }
        };
        let pos_of = |v: u32| {
            if live {
                mesh.positions()[v as usize]
            } else {
                self.base_pos[self.slot[v as usize] as usize]
            }
        };
        let normal = crate::ref_kernels::area_normal_with(&self.footprint, |v| {
            let n = nrm_of(v);
            ([f64::from(n[0]), f64::from(n[1]), f64::from(n[2])], free(v))
        })?;
        let point = crate::ref_kernels::area_center_with(&self.footprint, |v| {
            let p = pos_of(v);
            ([f64::from(p[0]), f64::from(p[1]), f64::from(p[2])], free(v))
        })?;
        let mut point = [point[0] as f32, point[1] as f32, point[2] as f32];
        let normal = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
        // O offset move o PLANO, não os vértices — é o knob que faz do Flatten
        // um Clay sem um segundo verbo.
        //
        // ⚠️ **Ele fica DEPOIS do kernel de propósito:** o `areaCenter` da
        // referência não o conhece (é nosso), e aplicá-lo por dentro faria a
        // soma que os gates de paridade comparam deixar de ser a dela.
        let off = brush.plane_offset * dab.radius;
        for k in 0..3 {
            point[k] += normal[k] * off;
        }
        Some(PlaneFit { point, normal })
    }
}
