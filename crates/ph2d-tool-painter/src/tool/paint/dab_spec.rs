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
        self.compose_canvas_warp(self.authored_spec())
    }

    /// O spec do artista, **sem** a deformação da arte — o que um traço congela no pen-down.
    #[must_use]
    pub(crate) fn authored_spec(&self) -> ph2d_painter_brush::BrushSpec {
        let brush = self.paint.brush;
        if brush.stroke_method == ph2d_painter_brush::StrokeMethod::GridStamp {
            brush.as_grid_stamp(self.shape_silhouette_active())
        } else {
            brush
        }
    }

    /// ⭐⭐⭐ **A deformação VIVA composta num spec autorado** — a porta que o traço em curso volta a
    /// chamar a cada ponto ([`ph2d_painter_brush::Stroke::set_canvas_dab`]).
    ///
    /// ⛔ Ela recebe o autorado em vez de o ler de `self.paint.brush` **de propósito**: a meio de um
    /// traço o artista pode ter mexido num slider, e o spec de um traço em curso é o do pen-down.
    /// *O que viaja aqui é a dobra da ARTE, nunca a intenção do artista.*
    #[must_use]
    pub(crate) fn compose_canvas_warp(
        &self,
        mut brush: ph2d_painter_brush::BrushSpec,
    ) -> ph2d_painter_brush::BrushSpec {
        let w = ph2d_painter_brush::canvas_warp::warped_dab(
            self.paint.canvas_warp,
            brush.dab_flatten,
            brush.dab_angle_deg,
        );
        // ⛔⛔ **A CERCA ESTAVA NA FAIXA DO SLIDER, E O PRÓPRIO FICHEIRO DIZ QUE ELA NÃO É O TECTO
        // DO MOTOR** (*«the interactive range, not the engine's hard cap»*). Report do dono, 6.ª
        // foto: *«pinta com diâmetro menor onde é mais estreito»* — medido, o pedido contra o
        // entregue com o tecto de `512`:
        //
        // | raio autorado | comprime a | pedido | entregue |
        // |---|---|---|---|
        // | `160` | `4×` | `640` | `512` ⛔ |
        // | `64` | `16×` | `1 024` | `512` ⛔ |
        //
        // Um raio cortado encolhe a elipse nos DOIS eixos, logo a marca chega ao ecrã menor —
        // exactamente onde a arte comprime, que é onde ela é estreita.
        //
        // ⭐ **O tecto é agora o do MOTOR** (`MAX_BRUSH_RADIUS_PX`), e isso só é afordável porque a
        // caixa do dab passou a seguir a PEGADA em vez do círculo
        // ([`ph2d_painter_brush::FootprintDeform::extent`]): sobre arte comprimida o raio e o
        // achatamento crescem juntos, logo a área visitada é `R²/|det W|` — os texels que a arte
        // tem naquele sítio — e não `R²`.
        brush.radius_px = (brush.radius_px * w.radius_scale).clamp(
            super::brush_ranges::BRUSH_SIZE_MIN_PX,
            ph2d_painter_brush::MAX_BRUSH_RADIUS_PX,
        );
        brush.dab_flatten = w.flatten;
        brush.dab_angle_deg = w.angle_deg;
        brush.dab_curve = w.curve;
        brush
    }

    /// ⭐⭐⭐ **A PEGADA QUE O ARTISTA VÊ** — o raio e a [`ph2d_painter_brush::FootprintDeform`] da
    /// marca **como ela aparece no ecrã**, com a orientação viva do traço.
    ///
    /// ⛔⛔⛔ **A 1.ª redacção devolvia a pegada que o MOTOR EMITE, e isso é o contrário** (report do
    /// dono com foto, 2026-09-14: *«o gizmo do pincel se deforma ao passar por cima das faces
    /// dobradas»*). Sobre arte dobrada o motor pinta na textura a elipse que a deformação
    /// **endireita** — no ecrã ela sai redonda. Desenhar essa elipse directamente no ecrã mostra-a
    /// torta: *o anel passou a mentir exactamente onde ele antes acertava.*
    ///
    /// ⭐ **E a lei é uma IDENTIDADE, com gate:** `W · (W⁻¹·E) = E` — o que o motor pinta, levado
    /// pela deformação, **é** a elipse autorada (`the_painted_dab_seen_through_the_warp_is_the_
    /// authored_ellipse`, medido sobre a saída real da porta, que passa por uma decomposição em três
    /// números e podia não voltar). ⇒ o anel desenha a AUTORADA, e por isso ele e a tinta concordam
    /// no ecrã por construção.
    ///
    /// ⚠️ **Ela continua a ser uma PORTA e não o instantâneo** (que também a traz): o anel não pode
    /// ter lei de orientação própria — foi remontá-la noutra crate que deixou esta wave inverter o
    /// sentido sem nada acusar. A composição é a do motor: o `follow_rotor` sobre o rumo VIVO.
    #[must_use]
    pub fn cursor_dab(&self) -> (f32, ph2d_painter_brush::FootprintDeform) {
        let b = self.paint.brush;
        let pegada = ph2d_painter_brush::FootprintDeform::new(b.dab_flatten, b.dab_angle_deg);
        (
            b.radius_px,
            pegada.rotated_by(b.follow_rotor(self.live_heading())),
        )
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
    use crate::tool::paint::brush_ranges;

    /// ⭐⭐⭐ **A PEGADA DO CURSOR É A AUTORADA, NUNCA A QUE O MOTOR PINTA** — o gate do report com
    /// foto (2026-09-14: *«o gizmo do pincel se deforma ao passar por cima das faces dobradas»*).
    ///
    /// ⛔⛔⛔ **Nenhum gate apanhava esta inversão, e eu enviei-a.** O gate estrutural da shell mede
    /// *«o anel lê a porta»* e ficou verde; o gate da identidade (`ph2d-painter-brush`) mede a LEI e
    /// também ficou verde — *a lei estava certa e a porta devolvia o outro lado dela*. ⇒ o que
    /// faltava é esta asserção: sobre uma arte deformada, a pegada do cursor tem de continuar a ser
    /// a do artista, e tem de **DIFERIR** da que o motor emite.
    ///
    /// ⚠️ A 2.ª metade é a que mata a inversão: sem ela, uma pegada que devolvesse a pintada
    /// passaria sempre que a deformação fosse a identidade.
    #[test]
    fn the_cursor_footprint_is_the_authored_one_not_the_painted_one() {
        let mut t = PainterTool::default();
        t.set_brush_dab_flatten(0.4);
        t.set_brush_dab_angle(30.0);
        let repouso = t.cursor_dab();
        // Uma arte comprimida ao meio num eixo — o regime da foto.
        t.set_canvas_warp(ph2d_painter_brush::canvas_warp::CanvasWarp::linear([[0.5, 0.0], [0.0, 1.0]]));
        let dobrada = t.cursor_dab();
        assert!(
            (dobrada.0 - repouso.0).abs() < 1e-6,
            "o RAIO do anel mudou com a deformação ({} contra {}): no ecrã a marca continua do \
             mesmo tamanho, e o anel tem de continuar também",
            dobrada.0,
            repouso.0
        );
        for k in 0..64 {
            let (a, b) = (
                repouso.1.outline_at(k as f32 / 64.0),
                dobrada.1.outline_at(k as f32 / 64.0),
            );
            assert!(
                (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-6,
                "a FORMA do anel mudou com a deformação no ponto {k}: ele passou a desenhar a \
                 elipse que o motor pinta na TEXTURA, que no ecrã sai redonda — o anel fica torto \
                 exactamente onde a marca fica certa"
            );
        }
        // ANTI-VÁCUO: a pegada que o motor EMITE tem mesmo de ser outra, senão não há o que separar.
        let pintada = t.stroke_spec();
        assert!(
            (pintada.dab_flatten - t.paint.brush.dab_flatten).abs() > 1e-3
                || pintada.radius_px != t.paint.brush.radius_px,
            "com esta deformação o motor emite a MESMA pegada que o artista autorou: a fixtura não \
             produz a diferença que este gate existe para separar"
        );
    }

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
    /// ⭐⭐⭐ **A CURVATURA DA ARTE CHEGA AO QUE O MOTOR PINTA** — o elo que ninguém mede.
    ///
    /// ⛔⛔ **O §5.0 deste repo avisa exactamente por isto:** *nenhum instrumento pergunta se o
    /// VALOR chega a um consumidor.* Os gates de registo provam que o clique chega à ferramenta; o
    /// gate de arquitectura prova que a porta de canvas pergunta à malha; a identidade da
    /// `canvas_warp` prova a LEI. Nenhum deles nota se o `set_canvas_warp` guarda a curvatura e a
    /// deita fora no caminho para o `BrushSpec` — e o app ficaria exactamente como antes da wave,
    /// com todos eles verdes.
    ///
    /// ⚠️ **A asserção que conta é a última**: não basta o campo viajar, a PEGADA que o kernel vai
    /// consumir (a que o falloff, a silhueta do Shape e o Grão lêem) tem de a carregar.
    #[test]
    fn the_fold_of_the_art_reaches_the_dab_the_engine_paints() {
        let mut t = PainterTool::default();
        t.set_brush_dab_flatten(0.2);
        t.set_brush_dab_angle(15.0);
        assert!(
            t.stroke_spec().dab_curve.is_flat(),
            "em repouso o dab emitido não pode ter curvatura nenhuma"
        );
        t.set_canvas_warp(ph2d_painter_brush::canvas_warp::CanvasWarp {
            linear: [[0.8, 0.1], [-0.05, 1.1]],
            curve: [
                [0.10, -0.06, 0.04, 0.03, 0.0, -0.02, 0.0],
                [-0.05, 0.11, 0.0, 0.0, 0.06, 0.0, -0.04],
            ],
        });
        let spec = t.stroke_spec();
        assert!(
            !spec.dab_curve.is_flat(),
            "a dobra da arte não chegou ao `BrushSpec` — o campo viaja e alguém o deita fora"
        );
        assert!(
            !spec.footprint_deform().is_flat_footprint(),
            "a dobra chegou ao campo e NÃO à pegada que o kernel consome — o falloff, a silhueta              do Shape e o Grão continuariam a ler uma elipse"
        );
    }

    /// ⭐⭐⭐ **O DIÂMETRO PEDIDO É O DIÂMETRO EMITIDO** — o report da 6.ª foto do dono
    /// (*«pinta com diâmetro menor onde é mais estreito»*, 2026-09-14).
    ///
    /// ⛔⛔ O tecto era a faixa do SLIDER (`BRUSH_SIZE_MAX_PX = 512`), cujo próprio doc diz *«the
    /// interactive range, **not** the engine's hard cap»*. Um raio cortado encolhe a elipse nos DOIS
    /// eixos, logo a marca chega ao ecrã menor — e corta exactamente onde a arte comprime, que é
    /// onde ela é estreita. Medido antes: um pincel de `160` era cortado já a `4×` de compressão.
    ///
    /// ⚠️ **A cerca a `32×` fica, e é do MODELO, não do tecto:** ali o `dab_flatten` satura no
    /// [`ph2d_painter_brush::DAB_FLATTEN_MAX`] (`0,95`) e a elipse deixa de poder ser mais fina —
    /// uma lasca infinitamente fina não é pintável. É por isso que o corpus pára em `16×`.
    #[test]
    fn the_diameter_the_engine_emits_is_the_one_the_fold_asks_for() {
        for raio in [8.0_f32, 64.0, 160.0] {
            for k in [1.0_f32, 0.5, 0.25, 0.125, 0.0625] {
                let mut t = PainterTool::default();
                t.set_brush_size_px(raio);
                let warp = ph2d_painter_brush::canvas_warp::CanvasWarp::linear([
                    [k, 0.0],
                    [0.0, 1.0],
                ]);
                t.set_canvas_warp(warp);
                let pedido =
                    raio * ph2d_painter_brush::canvas_warp::warped_dab(warp, 0.0, 0).radius_scale;
                let emitido = t.stroke_spec().radius_px;
                assert!(
                    (emitido - pedido).abs() <= pedido * 1e-4,
                    "com a arte comprimida {}× o motor emite raio {emitido} onde a deformação pede \
                     {pedido} — a marca chega ao ecrã menor do que o artista pediu",
                    1.0 / k
                );
            }
        }
        // ⛔ O CONTROLO: sem a composição não há nada a cortar, e este gate passaria sobre uma porta
        // que ignorasse a deformação. A `8×` o raio TEM de ter crescido muito acima da faixa do
        // slider — que é o tecto que estava aqui e que cortava.
        let mut t = PainterTool::default();
        t.set_brush_size_px(160.0);
        t.set_canvas_warp(ph2d_painter_brush::canvas_warp::CanvasWarp::linear([
            [0.125, 0.0],
            [0.0, 1.0],
        ]));
        assert!(
            t.stroke_spec().radius_px > brush_ranges::BRUSH_SIZE_MAX_PX * 2.0,
            "o raio emitido não passou da faixa do slider — o corpus deixou de exercitar o tecto \
             que era o defeito"
        );
    }

    /// ⭐⭐⭐ **A DOBRA DA ARTE É RELIDA ENQUANTO A MÃO ANDA** — o report da 7.ª foto do dono
    /// (*«sem melhorias»*, 2026-09-14), que era o defeito por baixo dos outros dois.
    ///
    /// ⛔⛔ Um traço captura o `BrushSpec` no pen-down, e isso é DESENHO. Mas quatro daqueles campos
    /// **não são do artista** — eles são derivados da deformação da arte, que muda de sítio para
    /// sítio. Congelá-los faz o traço inteiro pintar com a dobra do **sítio onde começou**: a marca
    /// está certa no primeiro ponto e errada em todo o resto do caminho. *Toda a maquinaria por-dab
    /// estava correcta e nunca era relida* — e nenhum gate desta linha a exercitava ao longo de um
    /// CAMINHO, só num ponto.
    ///
    /// ⚠️ A metade ANTI-VÁCUO é a segunda asserção: sem ela, um traço que ignorasse a deformação
    /// INTEIRA passaria (os dois raios seriam o autorado).
    #[test]
    fn the_fold_is_re_read_while_the_hand_travels() {
        use ph2d_editor_core::tool::{CanvasPaintTool as _, PointerPhase, RasterEditTool as _};
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 512 * 512 * 4], 512, 512);
        t.set_brush_size_px(20.0);
        let ponto = |p: [f32; 2], phase| ph2d_editor_core::tool::CanvasPointer {
            pos: p,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        // O traço NASCE sobre arte em repouso…
        t.set_canvas_warp(ph2d_painter_brush::canvas_warp::CanvasWarp::rest());
        t.on_canvas_pointer(ponto([100.0, 256.0], PointerPhase::Down));
        let no_inicio = t.paint.stroke.as_ref().map(ph2d_painter_brush::Stroke::radius_px);
        // …e entra numa zona comprimida a meio do caminho.
        t.set_canvas_warp(ph2d_painter_brush::canvas_warp::CanvasWarp::linear([
            [0.25, 0.0],
            [0.0, 1.0],
        ]));
        t.on_canvas_pointer(ponto([300.0, 256.0], PointerPhase::Move));
        let a_meio = t.paint.stroke.as_ref().map(ph2d_painter_brush::Stroke::radius_px);
        let (ini, meio) = (
            no_inicio.expect("o traço abriu"),
            a_meio.expect("o traço continua aberto"),
        );
        assert!(
            (meio - ini * 4.0).abs() < ini * 0.05,
            "o traço entrou numa zona comprimida 4× e o dab ficou com raio {meio} (era {ini}) — a \
             dobra da arte não está a ser relida, e o traço inteiro pinta com a do primeiro ponto"
        );
        // ⛔ O CONTROLO: em repouso o raio tem de ser o autorado — senão isto mediria uma porta que
        // inflasse o dab por outra razão qualquer.
        assert!(
            (ini - 20.0).abs() < 1e-3,
            "sobre arte em repouso o dab nasceu com raio {ini} e o artista pediu 20"
        );
    }

    /// Sonda: o TAMANHO que a porta entrega, varrendo a compressão da arte.    /// Sonda: o TAMANHO que a porta entrega, varrendo a compressão da arte.    /// Sonda: o TAMANHO que a porta entrega, varrendo a compressão da arte.
    #[test]
    #[ignore = "sonda: o diametro entregue contra a compressao"]
    fn probe_o_diametro_contra_a_compressao() {
        for raio in [8.0_f32, 32.0, 64.0, 160.0] {
            println!("\n  raio autorado {raio}");
            println!("    compressao   radius_scale   raio emitido   PEDIDO   flatten");
            for k in [1.0_f32, 0.5, 0.25, 0.125, 0.0625, 0.03125] {
                let mut t = PainterTool::default();
                t.set_brush_size_px(raio);
                t.set_canvas_warp(ph2d_painter_brush::canvas_warp::CanvasWarp::linear([
                    [k, 0.0],
                    [0.0, 1.0],
                ]));
                let d = ph2d_painter_brush::canvas_warp::warped_dab(
                    ph2d_painter_brush::canvas_warp::CanvasWarp::linear([[k, 0.0], [0.0, 1.0]]),
                    0.0,
                    0,
                );
                let spec = t.stroke_spec();
                println!(
                    "    {:8.4}   {:10.3}   {:10.2}   {:8.2}   {:.4}",
                    k,
                    d.radius_scale,
                    spec.radius_px,
                    raio * d.radius_scale,
                    spec.dab_flatten
                );
            }
        }
    }

}
