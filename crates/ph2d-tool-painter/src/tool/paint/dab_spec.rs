//! ⭐⭐⭐ **O DAB QUE O MOTOR VAI EMITIR** — irmão do [`super::grid_stamp_settings`] pelo tecto de 700
//! LOC (`architecture_workspace_file_loc_cap`), cortado por RESPONSABILIDADE: ali mora *a grelha do
//! carimbo*, aqui *que forma e que tamanho a próxima marca vai ter*.
//!
//! ⚠️ **As três portas deste ficheiro são a mesma pergunta em três alturas** — a especificação
//! completa (`stroke_spec`), a pegada que o cursor mostra (`cursor_dab`) e o raio ANTES da
//! deformação (`dab_footprint_px`, que é o que a porta de canvas usa para perguntar à malha). Elas
//! têm de viver juntas porque a do meio é feita da de cima, e a de cima do avesso da de baixo.

use super::super::PainterTool;

impl PainterTool {
    /// **O spec com que um traço corre** — para o Grid Stamp, o frame da CÉLULA já resolvido.
    ///
    /// ⚠️ Existe porque `has_shape` só é conhecido AQUI (os pixels do Shape vivem no tool) e é
    /// preciso em DOIS lugares que não podem discordar: o motor, que emite o `radius_px` de cada dab
    /// ao abrir o traço, e o carimbo, que estica a silhueta até as bordas da célula. Resolver uma
    /// vez e entregar o mesmo spec aos dois é o que impede a tinta de encostar num lado e sobrar do
    /// outro; [`ph2d_painter_brush::BrushSpec::as_grid_stamp`] é idempotente, então aplicá-la de novo
    /// no carimbo não é uma segunda resposta.
    ///
    /// ⭐⭐⭐ **E a DEFORMAÇÃO DO CANVAS entra aqui, pela MESMA razão** (report do dono com foto,
    /// 2026-09-14: *«o pincel é redondo mas pinta como se os polígonos não estivessem
    /// deformados»*). Sobre uma arte presa a um esqueleto e dobrada, um disco de textura chega ao
    /// ecrã como uma lasca; o que tem de ser pintado é a elipse que a deformação endireita, e ela
    /// são três números que o motor já consome — raio, achatamento e ângulo
    /// ([`ph2d_painter_brush::canvas_warp::warped_dab`]).
    ///
    /// ⚠️ **Os DOIS leitores desta porta continuam a concordar**, que é a razão de ela existir: o
    /// motor emite cada dab com este raio e o carimbo estica a silhueta com este achatamento.
    /// ⛔ **Sem deformação a conta é o NO-OP ao bit** — toda pincelada deste app é a de sempre.
    #[must_use]
    pub(crate) fn stroke_spec(&self) -> ph2d_painter_brush::BrushSpec {
        let brush = self.paint.brush;
        let mut brush = if brush.stroke_method == ph2d_painter_brush::StrokeMethod::GridStamp {
            brush.as_grid_stamp(self.shape_silhouette_active())
        } else {
            brush
        };
        let w = ph2d_painter_brush::canvas_warp::warped_dab(
            self.paint.canvas_warp,
            brush.dab_flatten,
            brush.dab_angle_deg,
        );
        // ⚠️ **A cerca é a que o MOTOR já aceita do artista** (`BRUSH_SIZE_MAX_PX`), e o recurso é
        // nomeado: o custo de um dab cresce com o raio ao QUADRADO, e uma compressão de `50×` num
        // triângulo quase colapsado pediria um disco que a cache de carimbo nunca viu. ⛔ Não é um
        // palpite de segurança — é o mesmo número que o slider do painel já entrega ao motor.
        brush.radius_px = (brush.radius_px * w.radius_scale).clamp(
            super::brush_ranges::BRUSH_SIZE_MIN_PX,
            super::brush_ranges::BRUSH_SIZE_MAX_PX,
        );
        brush.dab_flatten = w.flatten;
        brush.dab_angle_deg = w.angle_deg;
        brush
    }

    /// ⭐⭐⭐ **A PEGADA DO DAB COMO O MOTOR A VAI EMITIR** — o raio em píxeis de imagem e a
    /// [`ph2d_painter_brush::FootprintDeform`] já com a deformação da arte e a orientação viva.
    ///
    /// ⛔⛔ **Ela existe porque o ANEL DO CURSOR reconstruía a elipse por fora** (item 1 da fila do
    /// esqueleto, 2026-09-14): ele lia o achatamento e o rotor do instantâneo **autorado** e
    /// montava `(cos θ, m·sin θ)` à mão, noutra crate. Quando a pegada passou a carregar a
    /// deformação da arte (a wave anterior), o anel ficou a mostrar a forma de REPOUSO por cima de
    /// uma arte dobrada — *a mesma lei escrita duas vezes diverge no dia em que uma delas aprende
    /// alguma coisa.*
    ///
    /// ⚠️ **A composição é a do motor, chamada e não copiada:** o [`Self::stroke_spec`] (que já
    /// aplica a deformação) mais o `follow_rotor` sobre o rumo VIVO — as mesmas duas funções que o
    /// `BrushSpec::dab_rotor` compõe, menos o salto aleatório por dab, que é ruído e não forma.
    #[must_use]
    pub fn cursor_dab(&self) -> (f32, ph2d_painter_brush::FootprintDeform) {
        let spec = self.stroke_spec();
        let rotor = spec.follow_rotor(self.live_heading());
        (spec.radius_px, spec.dab_footprint(rotor))
    }

    /// ⭐⭐⭐ **O RAIO QUE O DAB VAI OCUPAR na imagem, ANTES da deformação** — o que a porta de canvas
    /// precisa para perguntar à malha *«que deformação fazes sobre um disco deste tamanho?»*
    /// ([`ph2d_render::mesh_uv`]).
    ///
    /// ⚠️ **É o raio do [`Self::stroke_spec`] SEM a composição da deformação**, e tem de ser: o que
    /// a malha responde é a entrada daquela composição, e realimentá-la com a saída dela seria um
    /// laço. ⛔ E é o raio do *Grid Stamp* quando ele manda, porque ali o footprint é a CÉLULA e não
    /// o tamanho do pincel.
    #[must_use]
    pub fn dab_footprint_px(&self) -> f32 {
        let brush = self.paint.brush;
        let brush = if brush.stroke_method == ph2d_painter_brush::StrokeMethod::GridStamp {
            brush.as_grid_stamp(self.shape_silhouette_active())
        } else {
            brush
        };
        brush.clamped_radius()
    }

    /// A silhueta de **Shape** está de fato ativa (kind escolhido *e*, para `Image`, pixels
    /// carregados) — a porta que o `stroke_spec` e o carimbo perguntam.
    #[must_use]
    pub(crate) fn shape_silhouette_active(&self) -> bool {
        self.paint
            .brush
            .shape_silhouette_active(self.paint.shape_image.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::PainterTool;

    /// ⭐⭐⭐ **A PEGADA DO CURSOR RODA COM O TRAÇO** — a metade que um censo de texto não mede.
    ///
    /// ⛔⛔ O gate da shell afirmava isto lendo `bs.dab_rotor` no fonte do anel; quando a lei mudou
    /// de sítio ele só soube dizer *«a string sumiu»*. Aqui mede-se o FACTO.
    ///
    /// ⚠️ **O rumo é posto pelo caminho do PRODUTO** (`on_canvas_hover`, duas vezes), e não por um
    /// setter só de teste: o rumo vivo é derivado de dois passeios consecutivos, e um atalho mediria
    /// outro programa.
    ///
    /// ⚠️ A metade ANTI-VÁCUO é a 2.ª: um pincel que **não** segue o traço tem de ficar parado — sem
    /// ela, uma pegada que rodasse sempre passaria na primeira.
    #[test]
    fn the_cursor_footprint_turns_with_the_stroke() {
        let mut t = PainterTool::default();
        t.toggle_brush_texture_rake(); // o Grain Rake: agora o dab segue o traço
        let parado = t.cursor_dab().1.outline_at(0.0);
        t.on_canvas_hover([0.0, 0.0]);
        t.on_canvas_hover([0.0, 40.0]); // um passeio para BAIXO
        let virado = t.cursor_dab().1.outline_at(0.0);
        let d = (parado[0] - virado[0]).hypot(parado[1] - virado[1]);
        assert!(
            d > 0.1,
            "a pegada do cursor não rodou com o rumo do traço (desvio {d}): o anel fica parado \
             enquanto a ponta que ele desenha vira"
        );

        let mut livre = PainterTool::default(); // sem rake: nada segue o traço
        let a = livre.cursor_dab().1.outline_at(0.0);
        livre.on_canvas_hover([0.0, 0.0]);
        livre.on_canvas_hover([0.0, 40.0]);
        let b = livre.cursor_dab().1.outline_at(0.0);
        assert!(
            (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-6,
            "um pincel que NÃO segue o traço rodou na mesma: a pegada está a ignorar a lei do rake"
        );
    }
}
