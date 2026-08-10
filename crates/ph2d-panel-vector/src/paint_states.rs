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

/// As réguas da MOLA — os mesmos números do modelo (`ph2d_ui_state::MIN/MAX_*`), com gate.
///
/// ⚠️ Estes **não são valores de design**: são as pontas da régua de uma grandeza FÍSICA (rigidez
/// em unidades de mola, amortecimento adimensional). Não existe token de escala para *"quão dura é
/// uma mola"*, e inventar um poria uma constante de física dentro do design system.
const MIN_STIFFNESS: f32 = 1.0;
const MAX_STIFFNESS: f32 = 60.0; // LITERAL-PX-OK: régua de FÍSICA, não de design
const MIN_DAMPING: f32 = 0.1; // LITERAL-PX-OK: régua de FÍSICA, não de design
const MAX_DAMPING: f32 = 2.0;

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

        // **O interruptor da PREVIEW** vem PRIMEIRO, porque ele muda o que a seção inteira
        // significa: ligado, o rato dirige os papéis e a autoria fecha.
        let preview_on = s.preview == Some(true);
        if let Some(on) = s.preview {
            y = self.preview_row(on, y);
        }

        // **Mover o widget carregando TODOS os estados** — ele fica ao lado do interruptor de
        // preview porque os dois dizem *como um GESTO se comporta*, e não *o que existe*; a
        // tabela de papéis abaixo é a segunda pergunta.
        if let Some(on) = s.move_all {
            y = self.checkbox_row(
                ids::VECTOR_STATE_MOVE_ALL,
                tr("panel.vector.states.move_all"),
                on,
                y,
            );
        }

        for i in 0..s.role_labels.len() {
            let label = s.role_labels[i].clone();
            y = self.state_role_row(i, &label, s.recorded[i], s.live == Some(i), !preview_on, y);
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

        // ⚠️ **Com a preview LIGADA a autoria fecha inteira** — nem os verbos, nem a duração.
        //
        // Não é rigor: o mundo, em preview, é uma pose DERIVADA que a máquina escreveu, e gravar
        // dali autoraria uma pose que o artista nunca fez (*"Rec Default"* sobre um hover). E há
        // um segundo motivo, que vale para a duração — um número que não move pose nenhuma: o
        // registro de undo está **suprimido** enquanto a preview corre, então toda edição feita
        // aqui dentro perderia o passo dela. Fechar a autoria remove a armadilha em vez de a
        // documentar. É a mesma linha que o Figma traça entre editar e apresentar.
        if preview_on {
            return y;
        }

        // ⭐ **A MOLA é uma OPÇÃO, e ela TROCA as linhas em vez de as somar.**
        //
        // ⚠️ Rigidez e amortecimento respondem a MESMA pergunta que duração e curva (*quanto
        // tempo, e com que forma*). Oferecer as quatro pediria ao artista que mantivesse dois
        // modelos de acordo, e a cena obedeceria a um deles sem dizer qual — é o par
        // `Width: Auto | Fixed` do texto e o `Mass: Auto | Manual` do editor de áudio.
        y = self.checkbox_row(
            ids::VECTOR_STATE_SPRING,
            tr("panel.vector.states.spring"),
            s.spring.is_some(),
            y,
        );
        if let Some((stiffness, damping)) = s.spring {
            return self.spring_rows(stiffness, damping, y);
        }

        // A DURAÇÃO é o número que o artista de facto afina; a régua é a mesma do modelo
        // (`MAX_DURATION_S`), publicada como fração para o trilho.
        let track = (s.duration_s / MAX_DURATION_S).clamp(0.0, 1.0);
        y = self.slider_row(
            tr("panel.vector.states.duration"),
            ids::VECTOR_STATE_DURATION,
            ids::VECTOR_STATE_DURATION_NUM,
            track,
            f64::from(s.duration_s),
            &format!("{:.2}s", s.duration_s),
            y,
        );

        let y = self.easing_rows(s.easing, y);
        // ⭐ **A TABELA SINAL → PAPEL** vem por ÚLTIMO, e a ordem é a de leitura: *que poses eu
        // tenho* (os papéis) → *como eu transito* (mola ou duração+curva) → *o que me faz
        // transitar*. Ela é a única lista que CRESCE, e uma lista variável no meio empurraria as
        // linhas fixas para baixo a cada `Add`.
        self.signal_rows(&s.bindings, &s.role_labels, y)
    }

    /// **As duas linhas da mola** — e são só duas, porque uma mola tem só dois números.
    ///
    /// ⚠️ As réguas são as do MODELO (`ph2d_ui_state::MIN/MAX_*`), com gate a compará-las: um par
    /// de tetos escrito aqui divergiria no dia em que o modelo mudasse, e o slider entregaria um
    /// valor que a porta depois clampa — o artista arrastaria até ao fim e o número pararia antes.
    fn spring_rows(&mut self, stiffness: f32, damping: f32, y: f32) -> f32 {
        let track = |v: f32, lo: f32, hi: f32| ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
        let y = self.slider_row(
            tr("panel.vector.states.stiffness"),
            ids::VECTOR_STATE_STIFFNESS,
            ids::VECTOR_STATE_STIFFNESS_NUM,
            track(stiffness, MIN_STIFFNESS, MAX_STIFFNESS),
            f64::from(stiffness),
            &format!("{stiffness:.1}"),
            y,
        );
        self.slider_row(
            tr("panel.vector.states.damping"),
            ids::VECTOR_STATE_DAMPING,
            ids::VECTOR_STATE_DAMPING_NUM,
            track(damping, MIN_DAMPING, MAX_DAMPING),
            f64::from(damping),
            &format!("{damping:.2}"),
            y,
        )
    }

    /// **O SELETOR DE CURVA** — a forma da transição, e a direção quando ela tem uma.
    ///
    /// ⚠️ **As duas fileiras saem de `ALL` + `label()`**, nunca de uma tabela deste painel: uma
    /// família nova entra no vocabulário e ganha o chip de graça, exactamente como os tipos de
    /// simetria. É também o que faz os dois consumidores do catálogo (este e o menu da timeline)
    /// dizerem a mesma palavra, com gate na shell a exigi-lo.
    ///
    /// ⚠️ **E a fileira do MODO só é pintada onde ele muda a curva.** `Linear` ignora-o — o `eval`
    /// devolve `u` antes de o olhar —, então oferecê-lo ali seriam três chips que desenham a mesma
    /// coisa. A pergunta é feita ao enum (`uses_mode`), medida por gate, e nunca a uma lista de
    /// exceções escrita aqui.
    fn easing_rows(&mut self, e: ph2d_anim::Easing, y: f32) -> f32 {
        use ph2d_anim::{EasingFamily, EasingMode};

        let fams: Vec<(ph2d_a11y::NodeId, &str, bool)> = EasingFamily::ALL
            .iter()
            .enumerate()
            .map(|(i, f)| (ids::vector_easing_family_id(i), f.label(), *f == e.family))
            .collect();
        let y = self.segmented(tr("panel.vector.states.curve"), &fams, y);

        if !e.family.uses_mode() {
            return y;
        }
        let modes: Vec<(ph2d_a11y::NodeId, &str, bool)> = EasingMode::ALL
            .iter()
            .enumerate()
            .map(|(i, m)| (ids::vector_easing_mode_id(i), m.label(), *m == e.mode))
            .collect();
        self.segmented(tr("panel.vector.states.curve.mode"), &modes, y)
    }

    /// **O interruptor da preview**, e — quando ligada — a linha que diz como sair.
    ///
    /// ⚠️ O botão troca de ESTADO, nunca de rótulo: um botão cujo texto alterna entre *"Preview"*
    /// e *"Exit"* obriga o artista a ler para saber onde está, enquanto um aceso se lê de relance.
    /// É a mesma escolha dos toggles do rail.
    fn preview_row(&mut self, on: bool, y: f32) -> f32 {
        let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        let st = self
            .store
            .button_state(ids::VECTOR_STATE_PREVIEW)
            .unwrap_or(ButtonState::Normal);
        // ⚠️ O *ligado* é o **KIND**, não o `ButtonState`: o `ButtonState` descreve o rato (hover,
        // press) e o kind descreve o que o botão É. Escrever *ligado* no `ButtonState` faria o
        // aceso desaparecer no instante em que o cursor passasse por cima dele.
        let btn = Button::new(ids::VECTOR_STATE_PREVIEW, tr("panel.vector.states.preview"))
            .kind(if on {
                ButtonKind::Accent
            } else {
                ButtonKind::Default
            })
            .state(st);
        paint_button(&btn, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::VECTOR_STATE_PREVIEW, rect);
        let y = y + self.row_h + Spacing::Xs.px();
        if on {
            // ⚠️ A porta de saída é ANUNCIADA. Um modo que toma o rato e não diz como se sai é um
            // modo em que o artista tenta clicar fora, nada acontece, e ele conclui que travou.
            return self.label_line(tr("panel.vector.states.preview.on"), y);
        }
        y
    }

    /// Uma linha: o nome do papel, e os verbos que fazem sentido para ele.
    ///
    /// `verbs` é falso durante a preview — ver o comentário no corpo da seção.
    fn state_role_row(
        &mut self,
        i: usize,
        label: &str,
        recorded: bool,
        live: bool,
        verbs: bool,
        y: f32,
    ) -> f32 {
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

        if !verbs {
            return y + self.row_h + Spacing::Xs.px();
        }

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
