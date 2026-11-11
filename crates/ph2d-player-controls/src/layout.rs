//! A geometria do painel — a MESMA para a pintura e para o clique. As caixas de tecla medem o próprio texto (maquete
//! C4: «J» é uma caixa estreita, «Press a key» uma larga), por isso a disposição mede com o `TextSystem` da pintura.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{label_budget, rect_for_label};
use ph2d_editor_core::widget::scrollbar_thumb_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::{FontWeight, TextSystem};

use crate::screen::{ControlsScreen, Hit, Hover, Row};

const KEY_H: f32 = m::FIELD_H - m::SEG_INSET;
const ROW_H: f32 = m::FIELD_H;
/// O indicador de rolagem, fino como nas listas de cartões: metade do recuo, no meio da margem direita.
const BAR_W: f32 = m::SEG_INSET * 0.5; // LITERAL-PX-OK: meio recuo
/// O título do painel, um degrau acima do título de cartão (maquete C4).
pub const TITLE_PX: f32 = m::LIST_TITLE_PX;
/// O pé («Reset to defaults» e a dica): menor e apagado, separado das linhas por um fio.
pub const FOOT_PX: f32 = m::LABEL_PX;
const FOOT_H: f32 = m::FIELD_H - m::SEG_INSET;
/// O recuo do painel, IGUAL nos quatro lados: dos lados até às faixas das linhas, em cima até ao texto do título e
/// em baixo desde o texto da última linha da dica.
pub const PAD: f32 = m::CARD_PAD_X;
/// O título centra-se em `CARD_HEAD_H`: o rect começa meia folga acima do `PAD` visível.
const PAD_TOP: f32 = PAD - (m::CARD_HEAD_H - TITLE_PX) * 0.5;
/// O passo entre as linhas da dica: o entrelinhado NORMAL da tipografia (a faixa do botão afastava-as como itens).
pub const HINT_LINE_H: f32 = FOOT_PX * ph2d_tokens::LineHeight::Normal.ratio();
/// Cada linha da dica centra-se em `HINT_LINE_H`: o fundo desconta a meia folga de baixo.
const PAD_BOTTOM: f32 = PAD - (HINT_LINE_H - FOOT_PX) * 0.5;
/// As três frases que a dica pode dizer: o pé reserva as linhas da MAIOR, para não mudar de altura ao escutar.
pub(crate) const HINTS: [&str; 3] = [
    "player_controls.hint_choose",
    "player_controls.hint_listening",
    "player_controls.none",
];

/// A frase que a dica diz agora: sem acções, à escuta, ou a de escolher.
pub(crate) fn hint_key(rows: &[Row], screen: &ControlsScreen) -> &'static str {
    if rows.is_empty() {
        HINTS[2]
    } else if screen.listening() {
        HINTS[1]
    } else {
        HINTS[0]
    }
}

/// Onde fica cada coisa do painel, em pixels de ecrã.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub panel: Rect,
    pub title: Rect,
    /// Por linha À VISTA: a faixa da linha, a coluna do nome e as caixas de tecla. A primeira é a `first`.
    pub rows: Vec<RowLayout>,
    /// O índice (nas linhas da tela) da primeira linha à vista.
    pub first: usize,
    /// Rola: `(trilha, polegar)` do indicador.
    pub bar: Option<(Rect, Rect)>,
    /// O fio entre as linhas e o pé.
    pub rule: Rect,
    pub reset: Rect,
    /// A dica de agora, partida na largura das linhas ([`foot_lines`]): uma faixa de [`HINT_LINE_H`] por linha.
    pub hint: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RowLayout {
    pub band: Rect,
    pub name: Rect,
    pub slots: Vec<Rect>,
    /// Onde vai o aviso da tecla repetida (largura zero quando nenhuma linha avisa).
    pub note: Rect,
}

/// O texto de uma caixa de tecla: o nome da tecla, «Add» na vazia, «Press a key» na que escuta.
#[must_use]
pub fn slot_label(row: &Row, slot: usize, listening_here: bool) -> String {
    if listening_here {
        return tr("player_controls.listening").to_owned();
    }
    row.keys
        .get(slot)
        .map_or_else(|| tr("player_controls.add").to_owned(), |k| k.label())
}

/// A frase do pé partida pelas palavras na largura `w` do painel (maquete C4: o painel abraça as linhas, o pé
/// não o alarga). Uma palavra maior que a linha fica sozinha nela; o painel nunca é mais estreito que a maior.
#[must_use]
pub fn foot_lines(text: &mut TextSystem, s: &str, w: f32) -> Vec<String> {
    let budget = label_budget(w);
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in s.split(' ').filter(|x| !x.is_empty()) {
        let next = if cur.is_empty() {
            word.to_owned()
        } else {
            format!("{cur} {word}")
        };
        if cur.is_empty() || text.prefix_width(&next, FOOT_PX) <= budget {
            cur = next;
        } else {
            out.push(std::mem::replace(&mut cur, word.to_owned()));
        }
    }
    out.push(cur);
    out
}

/// A largura de uma caixa de tecla: o texto inteiro com o respiro do rótulo, nunca mais estreita que alta.
fn chip_w(text: &mut TextSystem, label: &str) -> f32 {
    rect_for_label(text.prefix_width(label, m::VALUE_PX).ceil()).max(KEY_H)
}

impl Layout {
    /// O painel centrado em `area`, do topo da lista, fechado (ninguém escuta).
    #[must_use]
    pub fn of(area: Rect, rows: &[Row], text: &mut TextSystem) -> Self {
        Self::scrolled(area, rows, &ControlsScreen::default(), text)
    }

    /// O mesmo, rolado: começa em `screen.top()` e acerta-o para a linha escolhida ficar à vista (`selected ==
    /// rows.len()` é «Reset to defaults», sempre à vista em baixo). A caixa que escuta mede «Press a key».
    #[must_use]
    pub fn scrolled(
        area: Rect,
        rows: &[Row],
        screen: &ControlsScreen,
        text: &mut TextSystem,
    ) -> Self {
        Self::place(area, rows, screen, text, false)
    }

    /// **O painel no tamanho NATURAL**, a partir de `(x, y)`: todas as linhas à vista (sem indicador de rolagem) e a
    /// largura que o texto pede, sem o tecto de uma vista. A prévia da aba pinta-o assim e ESCALA-o para a tela.
    #[must_use]
    pub fn natural(x: f32, y: f32, rows: &[Row], text: &mut TextSystem) -> Self {
        let at = Rect::new(x, y, 0.0, 0.0);
        Self::place(at, rows, &ControlsScreen::default(), text, true)
    }

    fn place(
        area: Rect,
        rows: &[Row],
        screen: &ControlsScreen,
        text: &mut TextSystem,
        natural: bool,
    ) -> Self {
        let (selected, top) = (screen.selected(), screen.top());
        let listening =
            |i: usize, s: usize| screen.listening() && i == selected && s == screen.slot();
        // A primeira tecla de cada linha numa COLUNA de largura fixa (a maior, contando a que escuta): as caixas
        // começam todas no mesmo x e a borda não fica dentada.
        let press_w = chip_w(text, tr("player_controls.listening"));
        let first_w = rows
            .iter()
            .map(|r| chip_w(text, &slot_label(r, 0, false)))
            .fold(press_w, f32::max);
        let widths: Vec<Vec<f32>> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| {
                (0..r.slots())
                    .map(|s| match s {
                        0 => first_w,
                        _ => chip_w(text, &slot_label(r, s, listening(i, s))),
                    })
                    .collect()
            })
            .collect();
        let name_w = rows
            .iter()
            .map(|r| text.prefix_width_weighted(&r.label, m::VALUE_PX, FontWeight::SEMI_BOLD))
            .fold(0.0_f32, f32::max)
            .ceil();
        // ⭐ A largura do painel é a do MAIOR estado (onda 4: 228 → 272 px ao escutar): cada linha reserva, em
        // repouso, o que a sua caixa mais estreita depois da 1.ª cresce para «Press a key» — só uma escuta de cada vez.
        let mut keys_w = 0.0_f32;
        for r in rows {
            let resto: Vec<f32> = (1..r.slots())
                .map(|s| chip_w(text, &slot_label(r, s, false)))
                .collect();
            let folga = resto
                .iter()
                .map(|w| (press_w - w).max(0.0))
                .fold(0.0_f32, f32::max);
            let gaps = m::SEG_INSET * resto.len() as f32;
            keys_w = keys_w.max(first_w + resto.iter().sum::<f32>() + folga + gaps);
        }
        let note_w = rows
            .iter()
            .filter_map(Row::note)
            .map(|n| text.prefix_width(&n, m::LABEL_PX).ceil() + m::FIELD_GAP)
            .fold(0.0_f32, f32::max);
        // O pé não alarga o painel: só o «Reset to defaults» (curto, uma linha) e a maior palavra da dica.
        let reset_w = rect_for_label(
            text.prefix_width(tr("player_controls.reset"), FOOT_PX)
                .ceil(),
        );
        let word_w = HINTS
            .iter()
            .flat_map(|k| tr(k).split(' '))
            .map(|w| rect_for_label(text.prefix_width(w, FOOT_PX).ceil()))
            .fold(0.0_f32, f32::max);
        let foot_w = reset_w.max(word_w);
        let title_w = text
            .prefix_width_weighted(tr("player_controls.title"), TITLE_PX, FontWeight::SEMI_BOLD)
            .ceil();
        let lead = m::SEG_INSET + name_w + m::CARD_PAD_X;
        let inner_w = (lead + keys_w + note_w + m::SEG_INSET)
            .max(foot_w)
            .max(title_w);
        let w = inner_w + PAD * 2.0;
        let w = if natural { w } else { w.min(area.w) };
        let iw = w - PAD * 2.0;
        let hint_n = HINTS
            .iter()
            .map(|k| foot_lines(text, tr(k), iw).len())
            .max()
            .unwrap_or(1);
        let fixed = PAD_TOP
            + m::CARD_HEAD_H
            + m::FIELD_ROW_GAP * 2.0
            + FOOT_H
            + HINT_LINE_H * hint_n as f32
            + PAD_BOTTOM;
        // A reserva da frase maior não fica como faixa vazia em baixo: a DICA centra-se na altura reservada, e o resto
        // fica ancorado em cima — as linhas e as caixas não saltam ao escutar (onda 4: desciam ~8 px).
        let shown_n = foot_lines(text, tr(hint_key(rows, screen)), iw)
            .len()
            .min(hint_n);
        let lift = HINT_LINE_H * (hint_n - shown_n) as f32 * 0.5;
        let fit = (((area.h - fixed) / ROW_H).floor().max(1.0)) as usize;
        let visible = if natural {
            rows.len()
        } else {
            rows.len().min(fit)
        };
        let mut first = top;
        if selected < rows.len() {
            if selected < first {
                first = selected;
            } else if selected >= first + visible {
                first = selected + 1 - visible;
            }
        }
        let first = first.min(rows.len() - visible);
        let h = fixed + ROW_H * visible as f32;
        let panel = if natural {
            Rect::new(area.x, area.y, w, h)
        } else {
            let h = h.min(area.h);
            Rect::new(
                area.x + (area.w - w) * 0.5,
                area.y + (area.h - h) * 0.5,
                w,
                h,
            )
        };
        let x0 = panel.x + PAD;
        let title = Rect::new(x0, panel.y + PAD_TOP, iw, m::CARD_HEAD_H);
        let list_y = title.y + title.h;
        let mut y = list_y;
        let out_rows = (first..first + visible)
            .map(|i| {
                let band = Rect::new(x0, y, iw, ROW_H);
                y += ROW_H;
                RowLayout::of(band, name_w, lead, &widths[i], note_w)
            })
            .collect();
        let bar = (visible < rows.len()).then(|| {
            let track = Rect::new(
                panel.x + panel.w - (m::CARD_PAD_X + BAR_W) * 0.5,
                list_y,
                BAR_W,
                ROW_H * visible as f32,
            );
            let thumb = scrollbar_thumb_rect(
                track,
                ROW_H * first as f32,
                ROW_H * rows.len() as f32,
                track.h,
            );
            (track, thumb)
        });
        let rule_y = y + m::FIELD_ROW_GAP;
        let rule = Rect::new(x0, rule_y, iw, ph2d_tokens::StrokeToken::Hairline.px());
        // «Reset to defaults» é um BOTÃO do tamanho do rótulo, ao centro (o alvo do clique é ele, não a faixa).
        let rw = reset_w.min(iw);
        let reset = Rect::new(x0 + (iw - rw) * 0.5, rule_y + m::FIELD_ROW_GAP, rw, FOOT_H);
        let hint = Rect::new(
            x0,
            reset.y + FOOT_H + lift,
            iw,
            HINT_LINE_H * shown_n as f32,
        );
        Self {
            panel,
            title,
            rows: out_rows,
            first,
            bar,
            rule,
            reset,
            hint,
        }
    }

    /// O que acende sob `(x, y)`: a faixa de uma linha à vista ou o «Reset to defaults»; `None` = nenhum alvo.
    #[must_use]
    pub fn hover(&self, x: f32, y: f32) -> Option<Hover> {
        if self.reset.contains(x, y) {
            return Some(Hover::Reset);
        }
        let i = self.rows.iter().position(|r| r.band.contains(x, y))?;
        Some(Hover::Row(self.first + i))
    }

    /// O alvo sob `(x, y)`; `None` = fora do painel.
    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> Option<Hit> {
        if !self.panel.contains(x, y) {
            return None;
        }
        for (i, r) in self.rows.iter().enumerate() {
            if let Some(slot) = r.slots.iter().position(|s| s.contains(x, y)) {
                return Some(Hit::Slot {
                    row: self.first + i,
                    slot,
                });
            }
        }
        if self.reset.contains(x, y) {
            return Some(Hit::Reset);
        }
        Some(Hit::Panel)
    }
}

impl RowLayout {
    fn of(band: Rect, name_w: f32, lead: f32, widths: &[f32], note_w: f32) -> Self {
        let name = Rect::new(band.x + m::SEG_INSET, band.y, name_w, ROW_H);
        let ky = band.y + (ROW_H - KEY_H) * 0.5;
        // As caixas começam na coluna depois dos nomes, encostadas à ESQUERDA: a primeira tem largura fixa.
        let mut x = band.x + lead;
        let slots = widths
            .iter()
            .map(|&w| {
                let r = Rect::new(x, ky, w, KEY_H);
                x += w + m::SEG_INSET;
                r
            })
            .collect();
        let note = Rect::new(
            x - m::SEG_INSET + m::FIELD_GAP,
            band.y,
            (note_w - m::FIELD_GAP).max(0.0),
            ROW_H,
        );
        Self {
            band,
            name,
            slots,
            note,
        }
    }
}
