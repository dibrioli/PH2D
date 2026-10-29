//! O que está inerte com o pincel de AGORA (`PainterTool::inercias`) — filho de `inercia` por LOC.

use super::*;

impl PainterTool {
    /// O que está inerte com o pincel de AGORA — cada linha pela lei que o motor usa.
    pub(crate) fn inercias(&self) -> Inercias {
        let b = &self.paint.brush;
        let aquarela = self.watercolor_render_active();
        let agua = self.paint.wetpaint.armed;
        // O grão com que o motor PINTA (o papel quando ele faz de Grain, `poe_o_papel_como_grain`): as perguntas do grão
        // fazem-se a ele, não ao pincel cru — que não tem grão e esmaecia o que agia.
        let (efetivo, imagem_do_grao) = self.grao_que_pinta();
        let g = &efetivo;
        // O Follow roda a pegada INTEIRA do dab (`BrushSpec::dab_rotor`): sem textura no Shape, sem ponta achatada
        // e sem um Grain preso ao dab, a ponta redonda não tem ângulo para mostrar.
        let sem_lado = !b.shape_silhouette_active(self.paint.shape_image.is_some())
            && b.shape.kind != ph2d_painter_brush::TextureKind::Bristles
            && !b.pegada_anisotropica()
            && !(g.grain_samples(imagem_do_grao)
                && matches!(
                    g.texture.mapping,
                    ph2d_painter_brush::TextureMapping::ViewPlane
                        | ph2d_painter_brush::TextureMapping::Random
                ));
        // ⛔ A profundidade do grão por dab NÃO dá lado: rodar a pegada redonda só mudava o arredondamento (lia
        // vivo a 20/10, morto a 28/10 — 0 texels); sem lado o `authored_spec` desliga o Follow, ao bit, em todo meio.
        let leis = [
            // A ponta de píxel (L18) é binária: o grão, a Shape e as N imagens tornariam o píxel fraccionário.
            (Dependente::ForaDaPontaDePixel, self.pixel_vivo(b)),
            (Dependente::TaperTip, !b.taper.is_active()),
            (Dependente::TaperOpacity, !b.taper.is_active()),
            // O Accumulate decide DUAS coisas: o tecto do Strength (`stroke_cover_wanted`) e se o
            // Space Attenuation ligado chega ao carimbo (`space_attenuation_reaches` com ele ligado).
            (
                Dependente::Accumulate,
                !self.cobertura_tapada(&BrushSpec {
                    accumulate: false,
                    ..*g
                }) && !(b.space_attenuation
                    && BrushSpec {
                        accumulate: true,
                        ..*b
                    }
                    .space_attenuation_reaches()),
            ),
            (Dependente::SpaceAttenuation, !b.space_attenuation_reaches()),
            (Dependente::JitterUnit, !b.jitter_unit_matters()),
            (Dependente::DashLength, !b.dash_length_matters()),
            // O «Same as Paper» lê a Roughness como o contraste do grão (`ganho_do_dente`), com ou sem relevo à vista.
            (
                Dependente::PaperRoughness,
                self.substrate().is_none() && !self.grao_e_o_papel(),
            ),
            // Com o «Same as Paper» o papel escolhido É o grão do pincel, com ou sem relevo à vista.
            (
                Dependente::Paper,
                !aquarela && !agua && self.paint.substrate_depth <= 0.0 && !self.grao_e_o_papel(),
            ),
            // Na água o depósito (Paint), a borracha e o Blend (a máscara dele, J4 mistura L-2) recebem o Grain:
            // Smear, Wet, Dry e Blow ignoram-no de propósito (`wetpaint/dab_route.rs`).
            (
                Dependente::GrainDaAgua,
                agua && !self.paint.eraser
                    && !matches!(
                        self.paint.wetpaint.tool,
                        super::super::WetTool::Paint | super::super::WetTool::Blend
                    ),
            ),
            (Dependente::Pull, aquarela && !self.wet_mixer_active()),
            (Dependente::DepthSourceGrain, !g.texture.is_active()),
            (Dependente::EraseDaAgua, agua && !self.paint.eraser),
            // No Digital o Tooth é a profundidade do papel feito Grain (`papel_como_grain`).
            (
                Dependente::ToothDoDigital,
                matches!(self.paint_media(), super::super::media::PaintMedia::Digital)
                    && !self.papel_pode_ser_grain(),
            ),
            // A Heavy Edge só age com a tela inclinada (`Sim::sedimenta` multiplica `|g|`).
            (
                Dependente::BordaSemTilt,
                agua && !(self.paint.wetpaint.tilt_on && self.paint.wetpaint.tilt_ring > 0),
            ),
            // Sem Kind o Wet Paint pinta sem desenho de papel (dono 10/10): o Tooth não tem relevo para
            // esculpir e o Texture on Paint não tem papel para imprimir.
            (Dependente::SemPapel, agua && !b.paper.is_active()),
            // «Every Dab / Every Stroke» escolhe QUANDO o Randomize Color sorteia — sem Hue/Sat/Value não
            // há sorteio (`jitter::colour_draw`).
            (Dependente::QuandoDoRandomize, !b.has_colour_jitter_amount()),
            // As cerdas do Shape entram no motor como o `texv` (`wetpaint/dab_route.rs`): um Grain armado
            // toma o lugar delas (W2.4), e as ferramentas da água que não depositam não as lêem.
            (
                Dependente::CerdasDaAgua,
                agua && b.shape.kind == ph2d_painter_brush::TextureKind::Bristles
                    && (g.grain_samples(imagem_do_grao)
                        || (!self.paint.eraser
                            && self.paint.wetpaint.tool != super::super::WetTool::Paint)),
            ),
            // O Follow roda a pegada INTEIRA do dab (`BrushSpec::dab_rotor`): sem textura no Shape, sem ponta
            // achatada e sem um Grain preso ao dab, a ponta redonda não tem ângulo para mostrar.
            (Dependente::FollowSemNada, sem_lado),
            // A Texture Color colore a textura do Shape (prompt 38): sem textura não há o que colorir.
            (
                Dependente::TexturaSemForma,
                !b.shape_silhouette_active(self.paint.shape_image.is_some()),
            ),
            // A cápsula é uma lei da PONTA, em todo meio: não age num método sem Spacing (Dots,
            // Airbrush) nem sem caminho (Drag Dot, Anchored, Grid Stamp), num Spacing já tão fino quanto
            // o sub-passo, nem — na água — num passo que não passa de uma célula do motor
            // (`wetpaint/dab_route.rs`). Só a lêem as rotas do carimbo (Paint, a borracha, a Máscara) e a
            // água: Smear, Blur, Clone, Inpaint, Liquify, Sculpt e Knife têm kernel próprio (medido,
            // `sonda_p40_modos`: 0 texels de diferença com a caixa).
            (Dependente::ContinuoSemPasso, {
                let le = matches!(
                    self.paint.paint_mode,
                    super::super::PaintMode::Paint
                        | super::super::PaintMode::Mask
                        | super::super::PaintMode::WetPaint
                );
                // O raio do traço: na água o do MOTOR (a lei da largura, `dab_spec.rs`). ⛔ Não o
                // `stroke_spec()`: ele pergunta ao painel e voltava aqui (estouro de pilha, medido).
                let raio = match (agua, &self.paint.wetpaint.largura) {
                    (true, Some(lei)) => lei.raio_do_motor(b.radius_px),
                    _ => b.radius_px,
                };
                let s = BrushSpec {
                    continuous_spacing: true,
                    radius_px: raio,
                    ..*b
                };
                !le || !s.lays_capsules()
                    || (agua && s.dab_spacing_px() <= f32::from(self.paint.wetpaint.grid_ratio))
            }),
            // A cor da textura só a pintam as rotas do carimbo (`stamp_route.rs`, `is_color_mode`): a água
            // deposita a cor do pincel pela silhueta (`wet_owns_the_dabs`) e a aguada reconstrói-se da cor
            // do pincel (`watercolor_render_active`) — as duas saem antes dessas rotas (26 C1).
            (Dependente::CorDaTexturaSemLeitor, agua || aquarela),
            // A aguada lê o Grain CRU, como o mapa de granulação próprio (`grao_que_a_aguada_le`): sem a lei do grão; com
            // «Same as Paper» o grão próprio nem está na tela (`BrushSettings::grao_do_papel`); na água só o depósito
            // (Paint) pousa a cor da rampa.
            (
                Dependente::LeiDoGraoNaAguada,
                aquarela && !b.granulation_use_paper,
            ),
            (
                Dependente::CorDoGraoDaAgua,
                agua && (self.paint.eraser
                    || self.paint.wetpaint.tool != super::super::WetTool::Paint),
            ),
            // O cartão Pen (L2), pela lei que o traço lê (`dinamica_do_traco`, `inclinacao_do_pincel`): a
            // curva da opacidade só com a pressão na opacidade; os ângulos e o atraso só com a caneta a mudar
            // o carimbo. (O grão alisado e a rotação NÃO esmaecem: ver `o_censo_so_encolhe`.)
            (
                Dependente::CurvaSemOpacidade,
                !self.dinamica_do_traco().strength_pressure,
            ),
            (Dependente::CanetaDePe, {
                let t = self.inclinacao_do_pincel();
                !t.active() && t.opacity <= 0.0
            }),
            (Dependente::MaciezSemDureza, !agua && b.hardness <= 0.0),
            (Dependente::WashComAccumulate, b.accumulate),
            (
                Dependente::MaterialSemOleo,
                agua && self.paint.wetpaint.oleo_real.is_none(),
            ),
            // A Watercolor com a Shape «Automatic» não lê a pegada deitada (`inclinacao_dos_antigos`).
            (
                Dependente::PegadaDaAutomatica,
                aquarela && b.watercolor_shape_auto,
            ),
            // A contagem sorteada e a partícula só agem com spray (`DabJitter::particula`, `marca::conta`).
            (
                Dependente::MarcaSemSpray,
                b.spray_count <= 1 || !b.stroke_method.allows_jitter(),
            ),
            // A aguada não lê a cobertura do dab (`WASH_DEPOSIT_PEAK`, a mesma razão do cartão Pen).
            (Dependente::OpacidadeNaAguada, aquarela),
            // O fim do traço (L5): a ponta da cauda sem cauda (ou sem o plano que a re-pinta), o patamar sem a
            // carga que acaba; e o modo sobre o vazio sem um modo de cor (L7).
            (
                Dependente::PontaDaCaudaSemCauda,
                !b.taper.end_active() || self.plano_recusado().is_some(),
            ),
            (Dependente::PatamarSemFallOff, !b.taper.fall_active()),
            // Na água a carga que acaba é o pigmento que o PINCEL pousa (`dab_route`): a borracha, Smear, Blend,
            // Wet, Dry e Blow não pousam pigmento.
            (
                Dependente::FallOffDaAgua,
                agua && (self.paint.eraser
                    || self.paint.wetpaint.tool != super::super::WetTool::Paint),
            ),
            (Dependente::VazioSemModoDeCor, !b.blend.e_de_cor()),
            (
                Dependente::PlanoForaDoTraco,
                self.plano_recusado() == Some(super::super::traco_plano::SemPlano::ForaDoTraco),
            ),
            // A cor pela mão muda a cor do DAB: uma Color Ramp dona da cor (no carimbo e na aguada) ou a Texture
            // Color de uma Shape (só nas rotas do carimbo, `stamp_route.rs`) pintam a delas; na água só o depósito
            // (Paint) pousa a cor do dab — Smear, Blend, Wet, Dry e Blow não (a lei do `GrainDaAgua`).
            (
                Dependente::CorDaMaoSemLeitor,
                (agua
                    && !self.paint.eraser
                    && self.paint.wetpaint.tool != super::super::WetTool::Paint)
                    || (!agua
                        && (self.color_ramp_owner(b.texture.is_active())
                            != super::super::ramp_lut::RampLutOwner::None
                            || (!aquarela
                                && self.paint.shape_layers.is_color_mode()
                                && b.shape_silhouette_active(self.paint.shape_image.is_some())))),
            ),
            // Sem pressão no tamanho (nem na opacidade) a curva dele (e o piso, o tecto, a suavização da mão)
            // não têm canal que leia a pressão. Só nos meios antigos: no Wet a pressão chega também ao fluido.
            (Dependente::CurvaDoTamanhoSemPressao, {
                let d = self.dinamica_do_traco();
                !agua && (!d.size_pressure || d.size_min >= 1.0)
            }),
            // A profundidade por dab é `max(min, 1 − jitter·(1 − x))` (`GrainLaw::profundidade_do_dab`): com jitter 0
            // nenhum dab varia (`varia_por_dab`) e o mínimo e o controlo (pressão/sorteio) não têm o que decidir.
            (
                Dependente::ProfundidadeSemJitter,
                !b.texture.lei.varia_por_dab(),
            ),
            // O dual brush (L6): a MESMA porta que arma a 2.ª ponta (`dual_recusado`: o plano no Digital, o pincel
            // que pousa no Wet).
            (
                Dependente::DualForaDoDigital,
                self.dual_recusado() == Some(super::super::traco_plano::SemPlano::ForaDoDigital),
            ),
            (
                Dependente::DualForaDoTraco,
                self.dual_recusado() == Some(super::super::traco_plano::SemPlano::ForaDoTraco),
            ),
            // O piso, o tecto, a suavização e a curva em S moldam a pressão de TODO leitor dela: o tamanho, a
            // opacidade, a dureza, o espalhar, a profundidade do grão e a imagem da ponta (`dab_build`).
            (Dependente::MaoSemPressao, {
                let d = self.dinamica_do_traco();
                !agua
                    && (!d.size_pressure || d.size_min >= 1.0)
                    && !d.strength_pressure
                    && (d.pressure_hardness <= 0.0 || b.hardness <= 0.0)
                    && d.jitter_pressure <= 0.0
                    && !(b.texture.is_active()
                        && b.texture.lei.varia_por_dab()
                        && b.texture.lei.depth_pressure)
                    && !(b.hose.ativa() && b.hose.mode == ph2d_painter_brush::HoseMode::Pressure)
            }),
            // O traço só mede (e filtra) a velocidade com uma lei dela armada (`Dynamics::speed_armed`, a mesma porta
            // que arma a `Mao` em `stroke/speed.rs`).
            (
                Dependente::AtrasoSemVelocidade,
                !self.dinamica_do_traco().speed_armed(),
            ),
        ];
        Inercias {
            dependentes: leis
                .into_iter()
                .filter(|(_, inerte)| *inerte)
                .fold(0, |m, (d, _)| m | d.bit()),
            grain_params: params_inertes(&b.texture),
            shape_params: params_inertes(&b.shape),
        }
    }
}
