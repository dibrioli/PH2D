//! O painter da **Colorize section** (C2) — irmão do [`crate::paint_sections`], separado
//! só pelo teto de LOC (a costura que o arquivo já tinha). Estende o mesmo [`BodyCtx`].

use crate::ids;
use crate::paint_sections::BodyCtx;
use ph2d_i18n::tr;
use ph2d_tool_flip::{FlipMode, FlipStyleSnapshot, TRAP_MAX_PX, px_to_slider};

impl BodyCtx<'_> {
    /// **Colorize section** (C2) — a paleta do rabisco + as ações do gesto.
    ///
    /// O gesto (rabiscar) vive no canvas (modo Colorize); aqui ficam a COR do próximo
    /// rabisco e os botões que agem sobre os rabiscos ACUMULADOS: **Apply** roda o corte
    /// LazyBrush e materializa as regiões, **Clear** descarta os rabiscos. Os dois são ops do
    /// SHELL (mexem no buffer transiente + no documento), não da tool.
    pub(crate) fn colorize_section(&mut self, snap: &FlipStyleSnapshot, mut y: f32) -> f32 {
        if snap.mode != FlipMode::Colorize {
            return y;
        }
        y = self.section_label(tr("panel.flip.colorize.colorize"), y);

        // A cor do PRÓXIMO rabisco — paleta própria (o picker OKLCH é compartilhado).
        // ⭐ **A PORTA das cores** — ver o irmão `paint_sections.rs`.
        y = ph2d_editor_core::property_row::paint_color_row(
            self.scene,
            self.text_system,
            self.theme,
            self.hit_index,
            self.store,
            self.inner_x,
            self.inner_w,
            y,
            tr("panel.flip.colorize.color"),
            ids::FLIP_COLORIZE_SWATCH,
            snap.colorize_color,
            false,
            ph2d_editor_core::property_row::Seccao::apenas_campos(1),
        );

        // **Size — o MESMO do pincel** (ids `FLIP_SIZE`/`_NUM`), pela regra do Erase/Sculpt:
        // um 2º slider para a mesma grandeza seria estado duplicado, e o artista teria de
        // re-ajustar a cada troca de modo. Ele governa a espessura do rabisco — e como o
        // rabisco SEMEIA pela cápsula, é o Size que decide se um toque curto pega a região.
        let track = self
            .store
            .slider(ph2d_tool_flip::ids::FLIP_SIZE)
            .map(|(_, v)| v)
            .unwrap_or_else(|| px_to_slider(snap.width_px));
        let px = self
            .store
            .number_value(ids::FLIP_SIZE_NUM)
            .unwrap_or(snap.width_px);
        let px_display = format!("{}", px.round() as i64);
        y = self.slider_row(
            tr("panel.flip.colorize.size"),
            ph2d_tool_flip::ids::FLIP_SIZE,
            ids::FLIP_SIZE_NUM,
            track,
            px,
            &px_display,
            y,
        );

        // ── O vazamento pelo VÃO ABERTO de um divisor (6º smoke) ──────────────────────
        //
        // Dois ajustes, dois mecanismos. **Trap** SELA o vão (bola que não passa por um
        // vão < 2r ⇒ dois cômodos, cada um com a sua cor até a linha) — o MESMO knob do
        // balde, que o Colorize já lia mas não expunha aqui. **Bleed** regula, quando o vão
        // fica ABERTO, quão fundo a cor entra (a lente) — contínuo e imune ao zoom. Selar vs.
        // regular: os dois coexistem de propósito.
        let track = self
            .store
            .slider(ph2d_tool_flip::ids::FLIP_TRAP)
            .map(|(_, v)| v)
            .unwrap_or((snap.trap / TRAP_MAX_PX) as f32);
        let trap = f64::from(track) * TRAP_MAX_PX;
        y = self.slider_row(
            tr("panel.flip.colorize.trap"),
            ph2d_tool_flip::ids::FLIP_TRAP,
            ids::FLIP_TRAP_NUM,
            track,
            trap,
            &format!("{}", trap.round() as i64),
            y,
        );
        // Bleed: o track (0..1) É a fração `colorize_bleed`; o chip mostra a %.
        let track = self
            .store
            .slider(ph2d_tool_flip::ids::FLIP_COLORIZE_BLEED)
            .map(|(_, v)| v)
            .unwrap_or(snap.colorize_bleed as f32);
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fracao 0..1 -> leitura em %
        y = self.slider_row(
            tr("panel.flip.colorize.bleed"),
            ph2d_tool_flip::ids::FLIP_COLORIZE_BLEED,
            ids::FLIP_COLORIZE_BLEED_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );

        // Apply (roda o corte) · Clear (descarta os rabiscos).
        self.segmented(
            tr("panel.flip.colorize.scribbles"),
            [
                (
                    crate::ids::FLIP_COLORIZE_APPLY,
                    tr("panel.flip.colorize.apply"),
                    false,
                ),
                (
                    crate::ids::FLIP_COLORIZE_CLEAR,
                    tr("panel.flip.colorize.clear"),
                    false,
                ),
            ],
            y,
        )
    }
}
