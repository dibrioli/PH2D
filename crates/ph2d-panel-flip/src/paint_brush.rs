//! O card do **PINCEL** do painel Flip — *o que a PONTA faz*, separado de
//! [`crate::paint_sections`], que responde *o que cada MODO oferece*.
//!
//! O corte saiu do teto de LOC do painel (600), e a linha dele é honesta: este arquivo
//! cresce quando a ponta ganha atributo (Airbrush, Self Overlap, a dinâmica de pressão —
//! todos de 2026-07-25), enquanto o irmão cresce quando o painel ganha SEÇÃO. Um método,
//! uma pergunta: `brush` continua sendo um `BodyCtx` como os outros, e o `paint` não sabe
//! que houve split.
//!
//! ⚠️ O gate que forçou isto (`architecture_panel_loc_cap`) mora na `ph2d-editor-core`,
//! então um `cargo test -p ph2d-panel-flip` **não o alcança** — foi por isso que a linha
//! fechou vermelha e só o gate da árvore combinada pegou. É a mesma família do miss do
//! `file_loc_caps` (line/physics) e dos arch-gates de shell (line/Vector).

use crate::ids;
use crate::paint_sections::BodyCtx;
use ph2d_tool_flip::{EraseMode, FlipMode, FlipStyleSnapshot, px_to_slider};

impl BodyCtx<'_> {
    /// **Cada modo mostra SÓ os seus atributos** (Enio 2026-07-12).
    ///
    /// O painel é o da FERRAMENTA, não o da crate: exibir o Hardness do pincel enquanto o
    /// usuário está no balde é oferecer um controle que não faz nada — e um controle que
    /// não faz nada é pior que a ausência dele (o usuário mexe e conclui que o app está
    /// quebrado).
    ///
    /// - **Draw**: o pincel inteiro (Size / Hardness / Opacity / Smoothing).
    /// - **Erase**: só o que a borracha REALMENTE usa — o raio (`width_px`) e a força
    ///   (`opacity`). Ela não tem dureza, nem alisamento, nem cor.
    /// - **Reshape** (W5): as MESMAS duas — o raio do pincel de escultura e a força
    ///   dele. É de propósito que sejam as mesmas: um 2º par de sliders para raio e
    ///   força seria estado duplicado, e trocar de modo exigiria re-ajustar tudo.
    /// - **Edit** (W6): Size / Hardness / Opacity — mas aqui eles **não descrevem um
    ///   pincel**: editam os traços SELECIONADOS (o shell reaplica a partir da cópia
    ///   pristina). **Sem Smoothing**: o alisamento é uma op de GEOMETRIA sobre as
    ///   amostras cruas da caneta, que um traço já desenhado não guarda — um slider que
    ///   não pode agir é exatamente o controle morto que esta doutrina proíbe.
    /// - **Select / Fill**: nada daqui.
    pub(crate) fn brush(&mut self, snap: &FlipStyleSnapshot, mut y: f32) -> f32 {
        let sculpt = snap.mode == FlipMode::Reshape;
        let eraser = snap.mode == FlipMode::Erase;
        let editing = snap.mode == FlipMode::Edit;
        // Os dois modos "raio + força": a borracha e a escultura.
        let radius_force = eraser || sculpt;
        if !radius_force && !editing && snap.mode != FlipMode::Draw {
            return y;
        }
        let label = if eraser {
            "Eraser"
        } else if sculpt {
            "Sculpt"
        } else if editing {
            "Stroke" // não é um pincel: são os atributos do que está selecionado
        } else {
            "Brush"
        };
        y = self.section_label(label, y);
        // Size (px) — o raio da borracha / do pincel de escultura.
        //
        // §4.C: na BORRACHA a linha ganha um toggle de LINK. Ligado (o default) ela usa
        // o Size do PINCEL — literalmente os ids de sempre, então nada muda pra quem
        // nunca tocar no toggle; desligado, ela passa a ler/escrever os ids PRÓPRIOS.
        // O Sculpt segue compartilhando de propósito (a decisão documentada no
        // `params.rs`): você escolheu linkar pintura↔borracha, não pincel↔escultura.
        let (size_id, size_num, size_val) = if eraser && !snap.link_size {
            (
                ids::FLIP_ERASE_SIZE,
                ids::FLIP_ERASE_SIZE_NUM,
                snap.erase_px,
            )
        } else {
            (ids::FLIP_SIZE, ids::FLIP_SIZE_NUM, snap.width_px)
        };
        let track = self
            .store
            .slider(size_id)
            .map(|(_, v)| v)
            .unwrap_or_else(|| px_to_slider(size_val));
        let px = self.store.number_value(size_num).unwrap_or(size_val);
        let px_display = format!("{}", px.round() as i64);
        y = if eraser {
            self.slider_row_linked(
                "Size",
                size_id,
                size_num,
                track,
                px,
                &px_display,
                ids::FLIP_LINK_SIZE,
                snap.link_size,
                y,
            )
        } else {
            self.slider_row("Size", size_id, size_num, track, px, &px_display, y)
        };
        // **A Strength é SOFT-only** (Enio 2026-07-17: *"borracha hard não obedece a
        // strength"* — não obedecia mesmo). Hard CORTA o ponto e Stroke apaga o traço
        // inteiro: as duas são binárias, não têm o que dosar, e o `erase_at` sempre
        // documentou o parâmetro como *"(Soft only)"*. O slider ficava pintado e inerte —
        // o controle morto que a doutrina modal deste painel proíbe *"um controle que não
        // faz nada é pior que a ausência dele: o usuário mexe, nada muda, e conclui que o
        // app está quebrado"*. Agora ele (e o link dele) somem fora do Soft.
        let strength_applies = !eraser || snap.erase == EraseMode::Soft;
        // Hardness (0..1) — só o pincel de DESENHO tem borda.
        if radius_force && !strength_applies {
            return y; // Hard/Stroke: o raio é tudo o que a borracha tem
        }
        if radius_force {
            // Mesma regra de link do Size, no outro eixo (§4.C).
            let (str_id, str_num, str_val) = if eraser && !snap.link_strength {
                (
                    ids::FLIP_ERASE_STRENGTH,
                    ids::FLIP_ERASE_STRENGTH_NUM,
                    snap.erase_strength,
                )
            } else {
                (ids::FLIP_OPACITY, ids::FLIP_OPACITY_NUM, snap.opacity)
            };
            let track = self.store.slider(str_id).map(|(_, v)| v).unwrap_or(str_val);
            let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent chip
            let pct_display = format!("{}", pct.round() as i64);
            // "Strength" é o que a opacidade SIGNIFICA para a borracha e o sculpt.
            return if eraser {
                self.slider_row_linked(
                    "Strength",
                    str_id,
                    str_num,
                    track,
                    pct,
                    &pct_display,
                    ids::FLIP_LINK_STRENGTH,
                    snap.link_strength,
                    y,
                )
            } else {
                self.slider_row("Strength", str_id, str_num, track, pct, &pct_display, y)
            };
        }
        let track = self
            .store
            .slider(ids::FLIP_HARDNESS)
            .map(|(_, v)| v)
            .unwrap_or(snap.hardness);
        y = self.slider_row(
            "Hardness",
            ids::FLIP_HARDNESS,
            ids::FLIP_HARDNESS_NUM,
            track,
            f64::from(track),
            &format!("{track:.2}"),
            y,
        );
        // Opacity (0..100 %).
        let track = self
            .store
            .slider(ids::FLIP_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or(snap.opacity);
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent chip
        y = self.slider_row(
            "Opacity",
            ids::FLIP_OPACITY,
            ids::FLIP_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );
        // Smoothing (0..1) — the "settle". SÓ no Draw: ele reamostra a polilinha a partir
        // das amostras CRUAS da caneta, e um traço já desenhado não as guarda.
        if editing {
            return y;
        }
        let track = self
            .store
            .slider(ids::FLIP_SMOOTHING)
            .map(|(_, v)| v)
            .unwrap_or(snap.smoothing);
        y = self.slider_row(
            "Smoothing",
            ids::FLIP_SMOOTHING,
            ids::FLIP_SMOOTHING_NUM,
            track,
            f64::from(track),
            &format!("{track:.2}"),
            y,
        );
        // **Dinâmica de pressão** — a pressão da caneta vira largura (`params::pressure_width_factor`):
        // Min Width (o piso em pressão zero) + Response (curva macia⇔dura). Só no Draw (é a autoria
        // do traço). No mouse a pressão é 1 ⇒ largura cheia; no tablet, a caneta afina/engrossa.
        let track = self
            .store
            .slider(ids::FLIP_PRESSURE_MIN)
            .map(|(_, v)| v)
            .unwrap_or(snap.pressure_min_width);
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent chip
        y = self.slider_row(
            "Min Width",
            ids::FLIP_PRESSURE_MIN,
            ids::FLIP_PRESSURE_MIN_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );
        let track = self
            .store
            .slider(ids::FLIP_PRESSURE_RESPONSE)
            .map(|(_, v)| v)
            .unwrap_or(snap.pressure_response);
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent chip
        y = self.slider_row(
            "Response",
            ids::FLIP_PRESSURE_RESPONSE,
            ids::FLIP_PRESSURE_RESPONSE_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );
        // **O *tip* pontilhado** (03 §8) — a linha Tip + o Spacing, no módulo-irmão
        // `paint_tip.rs` (o teto de LOC deste arquivo). Só no Draw (o método é no-op fora).
        self.tip_section(snap, y)
    }
}
