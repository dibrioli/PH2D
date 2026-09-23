//! **O TRAÇO** — as duas secções que descrevem como uma linha desta cena é desenhada: a
//! largura/alinhamento/perfil (`stroke_style`) e as pontas/tracejado (`stroke_ends_and_dashes`).
//!
//! Irmão (`#[path]`) do [`super::paint_sections`], e o corte é o que o cap de LOC do painel
//! pediu — mas ele é honesto por conta própria: o traço tem vocabulário PRÓPRIO (perfil de
//! largura, junta, ponta, padrão de traço) e cresce por motivos que o preenchimento e a
//! booleana não partilham.

use super::*;
use ph2d_i18n::tr;

impl BodyCtx<'_> {
    /// Width + Stroke swatch + Stroke opacity + Cap / Join + Dash / Gap.
    pub(crate) fn stroke_style(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ph2d_tool_vector::ids::VECTOR_SECTION_STROKE,
            tr("panel.vector.section.stroke"),
            y,
        );
        if collapsed {
            return y;
        }
        // ⭐⭐ **ESTA FORMA TEM TRAÇO?** (plano 34) — a linha que faltava, e o report que a pediu foi
        // o do Enio de 2026-08-27 (*"o contorno funciona com as shapes que eu desejo, mas não
        // funcionam com os teus desenhos"*).
        //
        // ⚠️ **Um checkbox e não um `segmented`**, pela lei que este ficheiro-irmão já escreveu:
        // *um `segmented` é uma escolha entre MODOS nomeados; um checkbox é uma PROPRIEDADE que o
        // objeto tem ou não tem*. Ter traço é o segundo caso.
        //
        // ⚠️ `None` (nada selecionado, ou selecção múltipla) **não pinta a linha** — uma caixa que
        // descreve um objecto que não está lá é pior que caixa nenhuma.
        let tem_traco = state::stroke_present().unwrap_or(false);
        if let Some(on) = state::stroke_present() {
            y = self.checkbox_row(
                crate::ids::VECTOR_STROKE_PRESENT,
                tr("panel.vector.stroke.present"),
                on,
                y,
            );
        }
        // ⭐⭐ **COM QUE TINTA O TRAÇO DESENHA** (plano 35, wave D) — `Solid | Pattern`.
        //
        // ⚠️ **Um `segmented` e não um checkbox**, pela MESMA lei que escolheu o checkbox acima,
        // aplicada à metade certa: *ter traço* é uma propriedade que a forma tem ou não tem;
        // *com que tinta ele desenha* é uma escolha entre MODOS nomeados.
        //
        // ⚠️ `None` (sem traço, sem selecção, ou selecção múltipla) **não pinta a fileira** — sem
        // traço não há tinta de traço a escolher, e uma fileira que descreve o que não está lá é
        // pior que fileira nenhuma.
        //
        // ⛔ A lista **NÃO** é a do preenchimento: o renderer de traço não desenha gradiente, e um
        // chip que produzisse um `StrokePaint::Linear` gravaria estado que nada pinta (plano 35
        // §2.1).
        if let Some(k) = state::stroke_paint_kind() {
            y = self.segmented(
                tr("panel.vector.stroke.type"),
                &[
                    (
                        ph2d_tool_vector::ids::VECTOR_STROKE_KIND_SOLID,
                        tr("panel.vector.stroke.solid"),
                        k == state::StrokePaintKind::Solid,
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_STROKE_KIND_PATTERN,
                        tr("panel.vector.stroke.pattern"),
                        k == state::StrokePaintKind::Pattern,
                    ),
                    // ⭐⭐⭐ **A 3.ª tinta** (plano 36, W4) — a arte que PERCORRE a linha.
                    //
                    // ⚠️ **Clicar ARMA o gesto de duas mãos**, e não abre um diálogo: a arte de um
                    // pincel é uma FORMA do documento (o motor copia geometria), e o tipo
                    // (`BrushStroke::art: VecPathId`) torna a alternativa inexprimível.
                    (
                        ph2d_tool_vector::ids::VECTOR_STROKE_KIND_BRUSH,
                        tr("panel.vector.stroke.brush"),
                        k == state::StrokePaintKind::Brush,
                    ),
                ],
                y,
            );
        }
        // ⛔⛔ **E as rows abaixo FICAM VISÍVEIS mesmo sem traço** — o oposto da lei que o Enio pediu
        // para a secção *Pattern* (*"os parâmetros que um modo não usa não devem aparecer"*), e a
        // diferença é real: esta secção é a ficha da **TOOL** (`snap`), espelhada na selecção. Cada
        // row aqui autora **o traço da próxima forma que se desenhar**, então nenhuma está morta —
        // o que faltava era dizer que elas não alcançam ESTA forma, e é a caixa que o diz.
        // Width slider + px chip.
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_WIDTH)
            .map(|(_, v)| v)
            .unwrap_or_else(|| px_to_slider(snap.stroke_width_px));
        let px = self
            .store
            .number_value(ph2d_tool_vector::ids::VECTOR_WIDTH_NUM)
            .unwrap_or(snap.stroke_width_px);
        // **O TOKEN da ESPESSURA** (W4c.4). ⚠️ A rachura vai sobre o CHIP da row de Width, e não
        // sobre a row inteira: é o número que mente — ele mostra a largura AUTORADA (em px de tela)
        // enquanto o traço desenha o comprimento que o token dá. O retângulo vem da porta do
        // widget, para a marca não cair ao lado do campo num painel estreito, onde a row empilha.
        let width_row = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        y = self.slider_row(
            tr("panel.vector.stroke.width"),
            ph2d_tool_vector::ids::VECTOR_WIDTH,
            ph2d_tool_vector::ids::VECTOR_WIDTH_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );
        // ⚠️ Oferecida sob a MESMA condição da cor do traço, e pela metade que falta: um token de
        // largura numa forma sem traço teria de inventar a COR.
        if let Some(b) = crate::state::token_bindings().filter(|_| tem_traco) {
            let width_chip = ph2d_editor_core::widget::slider_with_chip_chip_rect(
                width_row,
                label_col_w(self.inner_x, self.inner_w),
                self.chip_w,
            );
            if b.width.is_some() {
                self.token_slash(width_chip);
            }
            y = self.token_row(ids::VECTOR_TOKEN_WIDTH, b.width.as_deref(), y);
        }

        // ⭐ Pela PORTA — ver [`BodyCtx::colour_swatch_row_rect`] para a lei da caixa.
        let (proximo, stroke_swatch_rect) = self.colour_swatch_row_rect(
            ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH,
            snap.stroke,
            tr("panel.vector.section.stroke"),
            y,
        );
        // A rachura, pela mesma razão do preenchimento.
        let bindings = crate::state::token_bindings();
        if bindings.as_ref().is_some_and(|b| b.stroke.is_some()) {
            self.token_slash(stroke_swatch_rect);
        }
        y = proximo;

        // **O TOKEN do traço** (plano UI/UX W4), logo abaixo da cor que ele cobre. Só é oferecido
        // quando a seleção TEM traço: o token colore o traço que existe e nunca inventa largura.
        if let Some(b) = bindings.filter(|_| tem_traco) {
            y = self.token_row(ids::VECTOR_TOKEN_STROKE, b.stroke.as_deref(), y);
        }

        // Stroke Opacity slider (single source of the stroke alpha).
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or_else(|| opacity_to_slider(snap.stroke[3]));
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent for the opacity chip
        y = self.slider_row(
            tr("panel.vector.stroke.opacity"),
            ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY,
            ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );

        // **COMO a linha se comporta e TERMINA** — irmã pelo teto de 200 LOC por fn.
        self.stroke_ends_and_dashes(snap, y)
    }

    /// **Cap · Join · Alinhamento · Tracejado · Pontas** — a metade da seção Stroke que decide
    /// *como a linha se comporta e onde ela acaba*, contra a de cima, que decide *com que
    /// espessura e cor ela é desenhada*.
    ///
    /// ⚠️ O corte é por RESPONSABILIDADE e não por tamanho, mas o gatilho foi o teto de 200 LOC
    /// por fn: a row de token da ESPESSURA (W4c.4) levou a `stroke_style` a 207. ⚠️ E ele sai para
    /// uma fn IRMÃ no mesmo arquivo de propósito — o teto de ARQUIVO desta crate ainda tem folga, e
    /// mover para um módulo novo separaria duas metades que se leem juntas.
    fn stroke_ends_and_dashes(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let mut y = y;
        // Cap / Join segmented rows.
        y = self.segmented3(
            tr("panel.vector.stroke.cap"),
            [
                (
                    ph2d_tool_vector::ids::VECTOR_CAP_BUTT,
                    tr("panel.vector.stroke.butt"),
                    snap.cap == StrokeCap::Butt,
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_CAP_ROUND,
                    tr("panel.vector.stroke.round"),
                    snap.cap == StrokeCap::Round,
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_CAP_SQUARE,
                    tr("panel.vector.stroke.square"),
                    snap.cap == StrokeCap::Square,
                ),
            ],
            y,
        );
        y = self.segmented3(
            tr("panel.vector.stroke.join"),
            [
                (
                    ph2d_tool_vector::ids::VECTOR_JOIN_MITER,
                    tr("panel.vector.stroke.miter"),
                    snap.join == StrokeJoin::Miter,
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_JOIN_ROUND,
                    tr("panel.vector.stroke.round"),
                    snap.join == StrokeJoin::Round,
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_JOIN_BEVEL,
                    tr("panel.vector.stroke.bevel"),
                    snap.join == StrokeJoin::Bevel,
                ),
            ],
            y,
        );

        // **ALINHAMENTO** — logo abaixo de Cap/Join porque é a mesma família (*como a caneta
        // se comporta*), e ACIMA do tracejado, que é sobre o PADRÃO e não sobre o lado.
        y = self.segmented3(
            tr("panel.vector.stroke.align"),
            [
                (
                    ph2d_tool_vector::ids::VECTOR_ALIGN_CENTRE,
                    tr("panel.vector.stroke.centre"),
                    snap.align == StrokeAlign::Centre,
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_ALIGN_INNER,
                    tr("panel.vector.stroke.inner"),
                    snap.align == StrokeAlign::Inner,
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_ALIGN_OUTER,
                    tr("panel.vector.stroke.outer"),
                    snap.align == StrokeAlign::Outer,
                ),
            ],
            y,
        );

        // Dash length (multiple of width; 0 = solid).
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_DASH)
            .map(|(_, v)| v)
            .unwrap_or_else(|| dash_to_slider(snap.dash));
        let px = self
            .store
            .number_value(ph2d_tool_vector::ids::VECTOR_DASH_NUM)
            .unwrap_or(snap.dash);
        y = self.slider_row(
            tr("panel.vector.stroke.dash"),
            ph2d_tool_vector::ids::VECTOR_DASH,
            ph2d_tool_vector::ids::VECTOR_DASH_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        // Gap length between dashes.
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_GAP)
            .map(|(_, v)| v)
            .unwrap_or_else(|| gap_to_slider(snap.gap));
        let px = self
            .store
            .number_value(ph2d_tool_vector::ids::VECTOR_GAP_NUM)
            .unwrap_or(snap.gap);
        y = self.slider_row(
            tr("panel.vector.stroke.gap"),
            ph2d_tool_vector::ids::VECTOR_GAP,
            ph2d_tool_vector::ids::VECTOR_GAP_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        // As PONTAS do traço (Start / End). Ficam aqui, e não no catálogo de formas,
        // porque uma ponta é propriedade do stroke: herda a cor e a largura dele e gira
        // com a tangente da curva — a quarta decisão sobre como a caneta termina, ao lado
        // de cap, join e dash. Desenho em `paint_markers` (teto de LOC por arquivo).
        self.marker_rows(snap, y)
    }
}
