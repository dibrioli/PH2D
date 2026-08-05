//! **A seção STATES** do painel (plano UI/UX W7) — irmã de [`super::paint_widget`] pelo teto de
//! 600 LOC, e o corte é o mesmo: aqui mora *que poses esta forma tem*.
//!
//! # Uma linha por PAPEL, e os papéis são um catálogo fixo
//!
//! Os quatro papéis existem sempre; o que varia é quais têm pose. Uma lista que crescesse com o
//! que foi gravado esconderia os que ainda não foram — e é justamente onde o artista precisa de
//! clicar para gravá-los.
//!
//! # Três verbos, e dois deles só existem depois do primeiro
//!
//! **Rec** grava; **Show** põe a cena naquela pose; **Clear** esquece. Pintar Show e Clear num
//! papel vazio daria dois cliques que não fazem nada, que é como o artista aprende a não confiar
//! nos botões desta janela — a mesma lei do *Wear a Widget* / *Back to Drawing*.

use ph2d_editor_core::widget::{Button, ButtonKind, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

/// O teto do slider — o MESMO número do modelo (`ph2d_ui_state::MAX_DURATION_S`), com gate a
/// compará-los: uma segunda régua faria o trilho encher num valor e o clamp cortar noutro.
const MAX_DURATION_S: f32 = 2.0;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::paint_sections::LABEL_COL_W;
use crate::state;

impl BodyCtx<'_> {
    /// **A seção STATES** — as poses desta forma e quanto tempo a transição leva.
    pub(crate) fn ui_states_section(&mut self, y: f32) -> f32 {
        let Some(s) = state::ui_states_state() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_STATES,
            tr("panel.vector.section.states"),
            y,
        );
        if collapsed {
            return y;
        }

        for i in 0..s.role_labels.len() {
            let label = s.role_labels[i].clone();
            y = self.state_role_row(i, &label, s.recorded[i], s.live == Some(i), y);
        }

        // ⚠️ O readout da pré-visualização é a única coisa que diz ao artista que a cena NÃO está
        // na pose de repouso. Sem ele, uma cena parada num hover parece o Default, e a próxima
        // gravação do Default o sobrescreve com a pose errada.
        if let Some(i) = s.live {
            y = self.label_line(
                &format!("{} {}", tr("panel.vector.states.showing"), s.role_labels[i]),
                y,
            );
        }

        // A DURAÇÃO é o número que o artista de facto afina; a régua é a mesma do modelo
        // (`MAX_DURATION_S`), publicada como fração para o trilho.
        let track = (s.duration_s / MAX_DURATION_S).clamp(0.0, 1.0);
        self.slider_row(
            tr("panel.vector.states.duration"),
            ids::VECTOR_STATE_DURATION,
            ids::VECTOR_STATE_DURATION_NUM,
            track,
            f64::from(s.duration_s),
            &format!("{:.2}s", s.duration_s),
            y,
        )
    }

    /// Uma linha: o nome do papel, e os verbos que fazem sentido para ele.
    fn state_role_row(&mut self, i: usize, label: &str, recorded: bool, live: bool, y: f32) -> f32 {
        use ph2d_editor_core::paint::{paint_text, resolve};
        use ph2d_tokens::{ColorToken, TypeToken};

        let text = if live {
            ColorToken::Accent
        } else if recorded {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        };
        paint_text(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            LABEL_COL_W,
            resolve(text, self.theme),
        );

        let x0 = self.inner_x + LABEL_COL_W;
        let w = (self.inner_x + self.inner_w - x0).max(0.0);
        let gap = Spacing::Xs.px();
        // Um papel vazio tem UM verbo; um gravado tem três. A largura segue a contagem em vez de
        // reservar espaço para botões que não estão lá.
        let n = if recorded { 3 } else { 1 };
        #[allow(clippy::cast_precision_loss)]
        let bw = ((w - gap * (n - 1) as f32) / n as f32).max(0.0);

        let mut bx = x0;
        let button = |ctx: &mut Self, id, lbl: &str, kind, x: f32| {
            let rect = Rect::new(x, y, bw, ctx.row_h);
            let st = ctx.store.button_state(id).unwrap_or(ButtonState::Normal);
            let btn = Button::new(id, lbl).kind(kind).state(st);
            paint_button(&btn, rect, ctx.scene, ctx.text_system, ctx.theme);
            ctx.hit_index.register(id, rect);
        };

        if recorded {
            button(
                self,
                ids::vector_state_apply_id(i),
                tr("panel.vector.states.show"),
                ButtonKind::Default,
                bx,
            );
            bx += bw + gap;
        }
        button(
            self,
            ids::vector_state_record_id(i),
            tr("panel.vector.states.rec"),
            ButtonKind::Accent,
            bx,
        );
        if recorded {
            bx += bw + gap;
            button(
                self,
                ids::vector_state_clear_id(i),
                tr("panel.vector.states.clear"),
                ButtonKind::Default,
                bx,
            );
        }
        y + self.row_h + Spacing::Xs.px()
    }
}
