//! **O grão e o papel do pincel** — que grão o motor pinta (o do pincel, o papel como Grain, o dente por cima)
//! e o Grain aplicado que um pincel gravado leva. Irmão de [`super`] pelo tecto de 700 LOC (B52.R12).

use super::super::super::PainterTool;

impl PainterTool {
    /// ⭐ **O grão com que o motor PINTA** quando o papel faz de Grain — a porta ÚNICA: o [`Self::authored_spec`] e a
    /// pergunta «a linha age?» (`inercia.rs`) passam por ela. Perguntar ao pincel cru esmaecia o Accumulate, a
    /// Roughness e o Source Grain que agiam (censo do esmaecido, 28/10).
    pub(crate) fn poe_o_papel_como_grain(&self, brush: &mut ph2d_painter_brush::BrushSpec) {
        if self.grao_e_o_papel_do(brush) && !brush.paper.is_active() {
            // «Same as Paper» com Paper: None (dono 08/11): não há papel, logo não há grão — nunca o papel de reserva.
            brush.texture = ph2d_painter_brush::texture::TextureSettings::default();
            brush.grain_depth = 0.0;
        } else if self.grao_e_o_papel_do(brush) {
            // «Same as Paper» (dono 28/10): o papel do documento, preso à tela, como o papel VISÍVEL o mostra —
            // a Roughness do cartão é o contraste do dente e o Relief a profundidade (`relief_na_tinta`: 0,6 no
            // Relief 0, a crescer até 1), × a Depth do próprio Grain (smoke 28/10: «corresponde parcialmente»). A lei
            // do grão («Grain behaviour», que o painel mostra com a caixa ligada) continua a do pincel.
            let lei = brush.texture.lei;
            brush.texture = super::super::substrate_relief::papel_do_documento(brush.paper);
            brush.texture.lei = ph2d_painter_brush::texture::GrainLaw {
                contraste: super::super::substrate_relief::ganho_do_dente(
                    self.paint.substrate_rough,
                ),
                ..lei
            };
            brush.grain_depth = (brush.grain_depth
                * super::super::wetpaint::grao_no_filme::relief_na_tinta(
                    self.paint.substrate_depth,
                ))
            .clamp(0.0, 1.0);
        } else if self.papel_como_grain_do(brush) {
            brush.texture = brush.paper;
            brush.grain_depth = self.mordida_do_papel(brush);
        } else if self.papel_pode_morder()
            && brush.texture.is_active()
            && !ph2d_painter_brush::dente::sonda_desligou_o_dente()
        {
            // Com um Grain do pincel (ou o Grain Colors), o papel MORDE por cima dele (dono 08/11: «a pintura
            // parecia chapada mesmo com paper com relevo»): o mesmo fator que ele dá como Grain, preso à tela.
            brush.dente = ph2d_painter_brush::dente::Dente::do_papel(
                brush.paper,
                self.mordida_do_papel(brush),
            );
        }
    }

    /// ⭐ **Quanto o papel morde a tinta** (dono 08/11): `Relief × Tooth / PAPER_TOOTH_MAX` — o Relief do papel é a
    /// força e o Tooth o multiplicador dela («usar o Tooth»). Relief 1 = a mordida de antes, quando o Relief só ligava.
    #[must_use]
    pub(crate) fn mordida_do_papel(&self, brush: &ph2d_painter_brush::BrushSpec) -> f32 {
        (self.paint.substrate_depth.clamp(0.0, 1.0) * brush.paper_depth
            / ph2d_painter_brush::PAPER_TOOTH_MAX)
            .clamp(0.0, 1.0)
    }

    /// O pincel de AGORA com o grão com que o motor pinta ([`Self::poe_o_papel_como_grain`]) e se esse grão tem
    /// imagem (a do papel quando ele faz de Grain, como a rota do Wet escolhe) — para quem pergunta «o grão amostra?».
    #[must_use]
    pub(crate) fn grao_que_pinta(&self) -> (ph2d_painter_brush::BrushSpec, bool) {
        let mut b = self.paint.brush;
        self.poe_o_papel_como_grain(&mut b);
        let imagem = if self.papel_como_grain() {
            self.paint.paper_image.is_some()
        } else {
            self.paint.texture_image.is_some()
        };
        (b, imagem)
    }

    /// **No Digital a tinta ENTRA no dente do papel** (dono, 2026-10-05: *«digital realmente cobre o
    /// papel, mas não deveria»* · *«impasto reconhece corretamente o papel»*; escolha: *«a tinta entra
    /// no dente»*). Com o Relief ligado e o Tooth > 0, e nenhum Grain escolhido, o PAPEL faz de Grain do
    /// traço — ancorado à tela, a mesma altura que o relevo ilumina — com a profundidade
    /// `Tooth / PAPER_TOOTH_MAX` pela lei do Grain (`1 + (h − 1)·prof.`: menos tinta nos vales, igual
    /// nos picos). Tooth 0 = o traço de antes, ao byte. Com um Grain escolhido, ele é a textura do traço.
    #[must_use]
    pub(crate) fn papel_como_grain(&self) -> bool {
        self.papel_como_grain_do(&self.paint.brush)
    }

    /// [`Self::papel_como_grain`] com o «Same as Paper» do pincel do LOTE ([`Self::grao_e_o_papel_do`]).
    #[must_use]
    pub(crate) fn papel_como_grain_do(&self, brush: &ph2d_painter_brush::BrushSpec) -> bool {
        self.grao_e_o_papel_do(brush)
            || (self.papel_pode_ser_grain() && self.paint.brush.paper_depth > 0.0)
    }

    /// ⭐ **Grain «Same as Paper»** fora da Aquarela (dono 28/10: *«habilite o mesmo para todos os modos de
    /// pintura»*): o grão do pincel É o papel do documento (`substrate_relief::papel_do_documento`), preso à
    /// tela pela lei do slot do papel. A Aquarela tem a sua (`granulation_use_paper`, a granulação).
    #[must_use]
    pub(crate) fn grao_e_o_papel(&self) -> bool {
        self.grao_e_o_papel_do(&self.paint.brush)
    }

    /// [`Self::grao_e_o_papel`] de um pincel dado — o do LOTE (o adiado, o `pincel_do_fecho`), que leva o «Same as
    /// Paper» do instante dos seus dabs, e não o do pincel vivo (que o artista pode ter alternado a meio do traço).
    #[must_use]
    pub(crate) fn grao_e_o_papel_do(&self, brush: &ph2d_painter_brush::BrushSpec) -> bool {
        brush.grain_same_as_paper
            && self.paint_media() != super::super::media::PaintMedia::Watercolor
    }

    /// O pincel foi aplicado com um Grain que amostra ([`ph2d_painter_brush::GraoAplicado`])? Então um grão armado toma
    /// o lugar das cerdas; senão, desenha por cima delas (`wetpaint/lote.rs::regua_do_grao`).
    #[must_use]
    pub(crate) fn grao_aplicado_amostra(&self) -> bool {
        let g = self.paint.brush.grao_aplicado;
        g.media.is_some()
            || ph2d_painter_brush::BrushSpec {
                texture: g.grao,
                ..self.paint.brush
            }
            .grain_samples(self.paint.texture_image.is_some())
    }

    /// O pincel a gravar: a média do Grain aplicado congela se ele era de imagem e já não é o Grain (o ficheiro não leva
    /// aquela imagem, Bug #61).
    #[must_use]
    pub(crate) fn pincel_a_gravar(&self) -> ph2d_painter_brush::BrushSpec {
        self.pincel_a_gravar_de(self.paint.brush)
    }

    /// [`Self::pincel_a_gravar`] de um pincel qualquer (o vivo ou o guardado num slot).
    #[must_use]
    pub(crate) fn pincel_a_gravar_de(
        &self,
        mut b: ph2d_painter_brush::BrushSpec,
    ) -> ph2d_painter_brush::BrushSpec {
        if b.texture.kind != ph2d_painter_brush::TextureKind::Image {
            b.grao_aplicado = self.grao_aplicado_congelado_de(&b);
        }
        b
    }

    /// A imagem do Grain vai sair: o slot é único, e cada pincel GUARDADO que a tem por referência fica com a média dela
    /// (Bug #65 A4). `com_o_vivo = false` quando o vivo já é outro (o preset que traz a imagem nova).
    pub(crate) fn congela_os_graos_da_imagem_que_sai(&mut self, com_o_vivo: bool) {
        for i in 0..self.paint.brush_by_mode.len() {
            self.paint.brush_by_mode[i].grao_aplicado =
                self.grao_aplicado_congelado_de(&self.paint.brush_by_mode[i]);
        }
        if com_o_vivo {
            self.paint.brush.grao_aplicado = self.grao_aplicado_congelado();
        }
    }

    /// [`ph2d_painter_brush::GraoAplicado`] com a média do Grain de imagem medida agora, enquanto a imagem dele está no slot.
    #[must_use]
    pub(crate) fn grao_aplicado_congelado(&self) -> ph2d_painter_brush::GraoAplicado {
        self.grao_aplicado_congelado_de(&self.paint.brush)
    }

    fn grao_aplicado_congelado_de(
        &self,
        b: &ph2d_painter_brush::BrushSpec,
    ) -> ph2d_painter_brush::GraoAplicado {
        let mut g = b.grao_aplicado;
        if g.grao.kind == ph2d_painter_brush::TextureKind::Image
            && g.media.is_none()
            && let Some(imagem) = self.paint.texture_image.as_ref()
        {
            let (w, h) = self.source_size;
            let de_g = ph2d_painter_brush::BrushSpec {
                texture: g.grao,
                ..*b
            };
            g.media = Some(super::super::wetpaint::media_do_grao(
                &de_g,
                Some(&imagem.as_mask()),
                [w as f32, h as f32],
            ) as f32);
        }
        g
    }

    /// O Digital está em condições de pôr o papel como Grain (o Relief ligado com um papel, sem Grain
    /// escolhido) — só falta o Tooth. É a lei do Tooth ESMAECIDO no Digital (`inercia`).
    #[must_use]
    pub(crate) fn papel_pode_ser_grain(&self) -> bool {
        self.papel_pode_morder() && !self.paint.brush.texture.is_active()
    }

    /// O Digital em condições de o papel morder a tinta (o Relief ligado com um papel e o Tooth > 0): como
    /// Grain sem Grain escolhido ([`Self::papel_pode_ser_grain`]), como dente por cima dele com um.
    #[must_use]
    pub(crate) fn papel_pode_morder(&self) -> bool {
        matches!(self.paint_media(), super::super::media::PaintMedia::Digital)
            && matches!(self.paint.paint_mode, super::super::PaintMode::Paint)
            && self.paint.substrate_depth > 0.0
            && self.paint.brush.paper.is_active()
    }

    /// A versão da imagem do slot que faz de Grain no traço (a do papel quando ele faz de Grain,
    /// [`Self::papel_como_grain`]), para as chaves dos caches (a do papel com o bit alto, para
    /// nunca coincidir com uma do Grain).
    pub(crate) fn versao_da_imagem_do_grain(&self) -> u64 {
        if self.papel_como_grain() {
            self.paint.paper_image_version | (1 << 63)
        } else {
            self.paint.texture_image_version
        }
    }
}
