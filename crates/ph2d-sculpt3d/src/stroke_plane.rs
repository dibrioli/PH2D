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
    /// **A superfície local do `l-mode`** — ver [`super::surface`].
    ///
    /// ⚠️ **`None` é o mundo inteiro que já shipava, e é isso que mantém o `S`
    /// e o `B` byte-idênticos:** o [`super::aim::signed_distance`] não ramifica
    /// em MODO nenhum — ele subtrai uma altura que, sem quadric, **é zero**. A
    /// representação apaga o caso especial, e nenhum dos quatro verbos precisa
    /// de saber que existe um `l-mode`.
    pub(super) surface: Option<super::surface::Quadric>,
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
            surface: None,
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
        // ⚠️ **E a [`Verb::MultiplaneScrape`] entra pela MESMA razão que o
        // polegar:** o `multiplane_scrape.cc:572` chama o mesmo
        // `calc_brush_plane`, logo herda o `!accum ⇒ orig`.
        //
        // ⚠️ **DOCUMENTADO em vez de gateado, e com o número:** a mutação que o
        // tira desta lista **não sangra**, e a razão é GEOMETRIA — o V é
        // simétrico em torno da dobradiça, então a normal média das duas facetas
        // que ele deixa é a MESMA que ele encontrou, e a leitura viva não deriva.
        // Medido num traço que insiste no mesmo lugar (a sonda
        // `what_the_frozen_hinge_buys`): profundidade **0,08814 congelado contra
        // 0,09477 vivo** a 20 dabs, e a diferença **não compõe** (0,10745 ×
        // 0,11138 a 60 dabs · 0,11157 × 0,11474 a 120). Um gate sobre 7% seria
        // um gate que alguém silencia; a regra fica por ser a da referência.
        //
        // ⚠️ **E a medição só existiu depois de a FIXTURE ser corrigida:** com o
        // `accumulate` herdado do [`Verb::Draw`] (que é `true`) a condição abaixo
        // devolve `live` sempre, e as duas rotas saíam **byte-idênticas** — o
        // ramo do `pre` era inalcançável no teste inteiro.
        let live = !matches!(
            brush.verb,
            Verb::ClayStrips | Verb::ClayThumb | Verb::MultiplaneScrape
        ) || brush.accumulate;
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
        let fit = PlaneFit {
            point,
            normal,
            surface: None,
        };
        // ⚠️ **A ALTURA é medida do ponto PRÉ-OFFSET, e o `(u, v)` do ponto
        // final — e a assimetria é o que mantém o offset VIVO.**
        //
        // A minha primeira versão ajustava tudo contra o ponto já deslocado, com
        // o racional de que *"o `c0` absorve o `−off`"*. Absorve mesmo, e é
        // exactamente esse o defeito: um quadric ajustado aos dados descreve a
        // forma **onde quer que se ponha a origem**, então `c0` absorvia o
        // offset, o `signed_distance` subtraía-o de volta, e o alvo saía o mesmo
        // — **o knob `plane_offset` ficava INERTE sob o `l-mode`**, que é o
        // controle morto que este módulo recusa. Ele era um verbo inteiro a
        // desaparecer: o [`Verb::Clay`] *é* o Flatten contra um plano levantado.
        //
        // Somar o `off` de volta na altura faz o `c` guardar a superfície na
        // origem PRÉ-offset; o `signed_distance` mede a partir da origem final,
        // e a diferença entre as duas é o offset — que passa a levantar a
        // superfície, exactamente como levantava o plano.
        //
        // ⚠️ **O `(u, v)` NÃO leva correção porque o offset corre ao longo da
        // NORMAL** — ele não move a origem tangencialmente, e mexer nas duas
        // coordenadas seria consertar o que não estava partido.
        //
        // ⚠️ **Uma mutação SOBREVIVEU até este gate existir**, e ela é
        // literalmente a versão anterior: nenhuma fixture da suíte usava
        // `plane_offset != 0` sob o `l-mode`, então o knob morto era invisível.
        let surface = if brush.mode.fits_local_surface(brush.verb) {
            let frame = super::surface::frame_of(&fit);
            let inv_r = if dab.radius > 0.0 {
                1.0 / dab.radius
            } else {
                0.0
            };
            super::surface::fit(
                self.footprint.iter().map(|&v| {
                    let p = pos_of(v);
                    let d = [
                        p[0] - fit.point[0],
                        p[1] - fit.point[1],
                        p[2] - fit.point[2],
                    ];
                    let u = (d[0] * frame.0[0] + d[1] * frame.0[1] + d[2] * frame.0[2]) * inv_r;
                    let v2 = (d[0] * frame.1[0] + d[1] * frame.1[1] + d[2] * frame.1[2]) * inv_r;
                    let h = d[0] * normal[0] + d[1] * normal[1] + d[2] * normal[2] + off;
                    (u, v2, h, free(v) as f32)
                }),
                frame,
                inv_r,
            )
        } else {
            None
        };
        Some(PlaneFit { surface, ..fit })
    }

    /// **OS DOIS PLANOS DA LÂMINA EM V**, resolvidos uma vez por dab.
    ///
    /// ⚠️ **`&mut self` porque o modo dinâmico tem MEMÓRIA** — o ângulo lido é
    /// interpolado contra o do dab anterior ([`crate::MULTIPLANE_ANGLE_SMOOTH`]),
    /// e sem essa memória a abertura do V salta quando a lâmina cruza uma quina.
    /// No modo fixo o campo é escrito na mesma, para que sair do modo dinâmico
    /// não deixe um valor obsoleto a ressuscitar no traço seguinte.
    ///
    /// `None` = este dab não deposita, e são os **dois** `return` da referência:
    /// sem direção não há dobradiça (`is_zero(grab_delta_symm)`, que é também o
    /// *"delay the first daub"*), e no modo dinâmico um lado vazio não tem normal
    /// (`if (!sample) return;`).
    pub(super) fn scrape_planes(
        &mut self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        plane: &PlaneFit,
    ) -> Option<ScrapePlanes> {
        // ⚠️ **A MESMA porta do polegar**, e a dobradiça é o outro eixo: o
        // `along` é o `Y` da moldura da referência (o que corre com o traço) e é
        // em torno DELE que os dois planos giram, enquanto o
        // [`Verb::ClayThumb`] gira em torno do `X`, que o atravessa. Um segundo
        // `cross` com um segundo piso aqui seria a segunda resposta a *"este dab
        // tem direção?"*.
        let along = super::target::stroke_axis(plane.normal, dab.path)?;
        let across = super::target::cross(along, plane.normal);
        let flip = brush.invert;

        let angle_deg = if brush.scrape_dynamic {
            // ⚠️ **A amostragem é do VIVO**, e a divergência com a normal de
            // área (congelada, logo acima) é da REFERÊNCIA: o
            // `sample_node_surface_mesh` colhe de `vert_positions`/`vert_normals`
            // — a superfície como ela está AGORA —, enquanto o `calc_brush_plane`
            // que dá a dobradiça lê o `orig` do pen-down. São duas perguntas: *em
            // que direção esta lâmina está deitada* (o gesto) e *que forma a
            // superfície tem debaixo dela neste instante* (a leitura).
            let side = |want_positive: bool| -> Option<([f32; 3], [f32; 3])> {
                let w = |v: u32| {
                    let p = mesh.positions()[v as usize];
                    let u = (p[0] - dab.center[0]) * across[0]
                        + (p[1] - dab.center[1]) * across[1]
                        + (p[2] - dab.center[2]) * across[2];
                    // ⚠️ **`u <= 0` cai no lado NEGATIVO**, e é o
                    // `local_positions[i][0] <= 0.0f` da referência ao pé da
                    // letra. O empate tem de morar num lado só: espalhado pelos
                    // dois, um vértice exactamente sobre a dobradiça votaria
                    // duas vezes.
                    if (u > 0.0) != want_positive {
                        return 0.0;
                    }
                    let n = mesh.normals()[v as usize];
                    if n[0] * dab.eye[0] + n[1] * dab.eye[1] + n[2] * dab.eye[2] > 0.0 {
                        return 0.0;
                    }
                    f64::from(crate::mask_ops::free_weight(
                        self.base_mask[self.slot[v as usize] as usize],
                    ))
                };
                let n = crate::ref_kernels::area_normal_with(&self.footprint, |v| {
                    let n = mesh.normals()[v as usize];
                    ([f64::from(n[0]), f64::from(n[1]), f64::from(n[2])], w(v))
                })?;
                let c = crate::ref_kernels::area_center_with(&self.footprint, |v| {
                    let p = mesh.positions()[v as usize];
                    ([f64::from(p[0]), f64::from(p[1]), f64::from(p[2])], w(v))
                })?;
                Some((
                    [n[0] as f32, n[1] as f32, n[2] as f32],
                    [c[0] as f32, c[1] as f32, c[2] as f32],
                ))
            };
            let (n_pos, c_pos) = side(true)?;
            let (n_neg, c_neg) = side(false)?;

            // O ângulo entre as duas normais amostradas — a abertura que a
            // SUPERFÍCIE já tem. `acos` de um produto escalar de unitários, com
            // o clamp que o `f32` obriga (uma soma normalizada pode sair a
            // `1 + 1e-7` e devolver `NaN`).
            let cosine =
                (n_pos[0] * n_neg[0] + n_pos[1] * n_neg[1] + n_pos[2] * n_neg[2]).clamp(-1.0, 1.0);
            let mut rad = cosine.acos();
            // ⚠️ **O knob VIRA UM ACRÉSCIMO aqui, e a pressão o escala** —
            // `sampled_angle += DEG2RADF(brush.multiplane_scrape_angle) *
            // ss.cache->pressure` (`:632`).
            rad += brush.scrape_angle_deg.to_radians() * dab.pressure.clamp(0.0, 1.0);

            // ⚠️ **CÔNCAVO INVERTE**, e o teste é geométrico: o ponto médio dos
            // dois centros amostrados cai ATRÁS do cursor numa crista e À FRENTE
            // dele num vale, então o sinal de `n · (cursor − meio)` diz de que
            // lado da dobra a lâmina está (`:635`).
            let mid = [
                f32::midpoint(c_pos[0], c_neg[0]),
                f32::midpoint(c_pos[1], c_neg[1]),
                f32::midpoint(c_pos[2], c_neg[2]),
            ];
            let toward = [
                dab.center[0] - mid[0],
                dab.center[1] - mid[1],
                dab.center[2] - mid[2],
            ];
            if plane.normal[0] * toward[0]
                + plane.normal[1] * toward[1]
                + plane.normal[2] * toward[2]
                < 0.0
            {
                rad = -rad;
            }
            // ⚠️ **O Ctrl ZERA o V no modo dinâmico em vez de o inverter**, e a
            // referência escreve o porquê: *"so you can trim plane surfaces
            // without changing the brush"* (`:640`). Inverter é o que ele faz no
            // modo FIXO — os dois são o mesmo gesto com significados diferentes,
            // e é o modo que decide qual.
            let sampled = if flip { 0.0 } else { rad.to_degrees() };
            self.scrape_angle_deg = crate::MULTIPLANE_ANGLE_SMOOTH.mul_add(
                self.scrape_angle_deg,
                (1.0 - crate::MULTIPLANE_ANGLE_SMOOTH) * sampled,
            );
            self.scrape_angle_deg
        } else {
            let a = if flip {
                -brush.scrape_angle_deg
            } else {
                brush.scrape_angle_deg
            };
            self.scrape_angle_deg = a;
            a
        };

        // ⚠️ **A ORIGEM é o centro do DAB, e a exceção é do modo dinâmico
        // invertido:** ali a referência NÃO reescreve o `area_position` (o
        // `else` do `:643`), então o corte plano do Ctrl acontece contra o plano
        // de ÁREA — o mesmo que os quatro verbos de plano usam, com o
        // `plane_offset` do artista já dentro dele.
        let origin = if brush.scrape_dynamic && flip {
            plane.point
        } else {
            dab.center
        };

        let (sin_half, cos_half) = (angle_deg.to_radians() * 0.5).sin_cos();
        Some(ScrapePlanes {
            origin,
            across,
            normal: plane.normal,
            sin_half,
            cos_half,
            // ⚠️ **O culling só existe com o V ABERTO** (`if (angle >= 0.0f)`,
            // `:405`): ali a lâmina raspa e só o que está acima do próprio
            // meio-plano é matéria a remover. Com o V fechado a projeção é
            // bilateral, e é isso que faz a mesma ferramenta ENCHER uma dobra
            // côncava.
            cull: angle_deg >= 0.0,
        })
    }
}

/// **A LÂMINA EM V**, resolvida uma vez por dab — ver
/// [`SculptStroke::scrape_planes`].
///
/// ⚠️ **Um par de planos guardado como UM objeto**, e não dois [`PlaneFit`]: os
/// dois partilham a origem e a dobradiça, e diferem **apenas no sinal** com que
/// a normal tomba. Guardá-los separados poria o mesmo fato em dois lugares, e
/// nada impediria o dia em que um deles herdasse uma origem diferente do outro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ScrapePlanes {
    /// O ponto por onde os DOIS planos passam.
    pub(super) origin: [f32; 3],
    /// O eixo unitário que ATRAVESSA o traço — é o sinal da coordenada ao longo
    /// dele que escolhe qual meio-plano serve o vértice.
    pub(super) across: [f32; 3],
    /// A normal de área: a bissetriz do V.
    pub(super) normal: [f32; 3],
    /// `sin(θ/2)`, **com sinal** — negativo é o V fechado (a dobra côncava).
    pub(super) sin_half: f32,
    /// `cos(θ/2)`.
    pub(super) cos_half: f32,
    /// Só o que está ACIMA do próprio meio-plano é tocado?
    pub(super) cull: bool,
}

impl ScrapePlanes {
    /// A normal do meio-plano que serve este ponto.
    ///
    /// ⚠️ **UMA expressão para os dois planos, e é a representação a apagar o
    /// caso especial:** a referência monta dois `float4` girando a normal de
    /// `+θ/2` e `−θ/2` e depois escolhe entre eles por índice; num frame
    /// ortonormal a rotação da normal em torno do eixo do traço é exactamente
    /// `sin(θ/2)·across + cos(θ/2)·n`, então o índice vira o SINAL de um dos dois
    /// termos. Cada meio-plano tomba **para o lado que ele serve** — é isso que
    /// abre o V.
    pub(super) fn normal_at(self, p: [f32; 3]) -> [f32; 3] {
        let u = (p[0] - self.origin[0]) * self.across[0]
            + (p[1] - self.origin[1]) * self.across[1]
            + (p[2] - self.origin[2]) * self.across[2];
        let s = if u > 0.0 {
            self.sin_half
        } else {
            -self.sin_half
        };
        [
            s * self.across[0] + self.cos_half * self.normal[0],
            s * self.across[1] + self.cos_half * self.normal[1],
            s * self.across[2] + self.cos_half * self.normal[2],
        ]
    }
}
