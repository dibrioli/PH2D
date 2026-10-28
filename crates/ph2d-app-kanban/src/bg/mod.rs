//! ⭐ **D11 — o fundo do Kanban**: o que a vista pinta ([`Look`]: a lei do `ph2d-board-render`, a mesma
//! do Quadro, com a imagem já descodificada) e a gaveta «Change background» do menu do quadro (a do
//! Trello: fotografias, cores e gradientes, padrões com as duas cores, e a cor livre). Cada escolha é
//! UM passo de desfazer e viaja como as outras mudanças do cabeçalho (`KanbanOp::Header`); arrastar o
//! seletor da cor livre mostra-a já e grava UMA vez ao largar.

mod paint;
mod place;

use crate::docs::KanbanDocs;
use crate::front::cover::{self, ThumbRef};
use ph2d_board_render::{BgImage, BgTone, KanbanBg, kanban_bg_tone, paint_kanban_bg};
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::widget::{hsv_to_rgba8, rgba_to_hsv};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_kanban_model::{Background, Hue, KanbanOp, KanbanSet, Pattern, PatternKind, Rgb, Tint};
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

pub(crate) use paint::paint;
pub(crate) use place::{Placed, place};

/// ⭐ O fundo que a vista pinta: a chave e a imagem dele (quando já chegou do fio de fundo).
#[derive(Clone, Debug)]
pub(crate) struct Look {
    pub bg: Background,
    pub image: Option<ThumbRef>,
}

impl Look {
    /// O fundo `bg` de `set` (a imagem pede-se ao fio de fundo à largura de um fundo).
    pub(crate) fn of(set: &KanbanSet, bg: Background) -> Self {
        let image = match bg {
            Background::Image(f) => cover::get_at(f, cover::BG_TEXELS_W, || {
                set.file(f).map(|x| x.bytes.0.clone())
            }),
            _ => None,
        };
        Self { bg, image }
    }

    /// Um fundo sem imagem (a cor de espera onde houver uma).
    pub(crate) fn flat(bg: Background) -> Self {
        Self { bg, image: None }
    }

    fn img(&self, area: Rect) -> Option<BgImage<'_>> {
        let t = &self.image.as_ref()?.0;
        let rows = &t.row_max;
        // As linhas da miniatura que caem na faixa do cabeçalho (o recorte `cover` sobre `area`).
        let d = cover_dest(t.size, area);
        #[allow(clippy::cast_precision_loss)]
        let n = rows.len() as f32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (a, b) = (
            (((area.y - d.y) / d.h * n).floor().clamp(0.0, n)) as usize,
            (((area.y + crate::metrics::HEADER_H - d.y) / d.h * n)
                .ceil()
                .clamp(0.0, n)) as usize,
        );
        #[allow(clippy::cast_precision_loss)]
        let mean = rows.iter().sum::<f32>() / n.max(1.0);
        Some(BgImage {
            rgba: &t.rgba,
            size: t.size,
            brightest: rows[a..b.max(a)].iter().copied().fold(0.0, f32::max),
            mean,
        })
    }

    /// Quão claro é o fundo pintado em `area` (o véu do cabeçalho e o chão claro).
    pub(crate) fn tone(&self, theme: Theme, area: Rect) -> BgTone {
        kanban_bg_tone(theme, self.bg, self.img(area).as_ref())
    }

    /// ⭐ Pinta o fundo em `r` (px da vista; `scale` = px de ecrã por px da vista).
    pub(crate) fn paint(&self, scene: &mut VectorScene, theme: Theme, r: Rect, scale: f64) {
        paint_kanban_bg(
            scene,
            theme,
            &KanbanBg {
                bg: self.bg,
                rect: rect_to_vello(r),
                to_scene: Affine::IDENTITY,
                screen_scale: scale,
                image: self.img(r),
            },
        );
    }
}

/// Que cor a cor livre está a mudar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    /// O fundo liso.
    Solid,
    /// A base do padrão.
    Base,
    /// O traço do padrão.
    Ink,
}

/// Um alvo da gaveta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Row {
    Close,
    Upload,
    /// A fotografia actual (já escolhida).
    Photo,
    Color(Hue),
    Gradient(u16),
    Pattern(PatternKind),
    /// Um matiz da base (`Target::Base`) ou do traço (`Target::Ink`) do padrão.
    Tint(Target, Hue),
    /// Abre/fecha o seletor da cor livre de `Target`.
    Custom(Target),
    /// O quadrado saturação × valor e a faixa do matiz do seletor.
    Sv,
    HueStrip,
    /// Dentro da gaveta, sem alvo.
    Inside,
}

/// A gaveta aberta.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Drawer {
    pub scroll: f32,
    /// O seletor da cor livre aberto, e para quê.
    pub custom: Option<Target>,
    /// O matiz, a saturação e o valor do seletor (`0..=1`; o matiz guarda-se à parte para o
    /// quadrado não saltar quando o valor vai a zero).
    pub hsv: [f32; 3],
    /// O fundo que o arrasto do seletor mostra (ainda por gravar).
    pub preview: Option<Background>,
    pub press: Option<Row>,
    pub dragging: bool,
    pub hover: Option<Row>,
    /// «Upload…» pedido: a fila serve-o ([`KanbanDocs::serve_background`]).
    pub pick: bool,
    /// O tema do último quadro pintado (a cor de um matiz, para o seletor partir dela).
    pub theme: Theme,
}

/// O destino da imagem de `size` que enche `r` sem deformar, centrada (o `cover` da lei).
fn cover_dest(size: [u32; 2], r: Rect) -> Rect {
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (size[0].max(1) as f32, size[1].max(1) as f32);
    let s = (r.w / w).max(r.h / h);
    let (dw, dh) = (w * s, h * s);
    Rect::new(r.x + (r.w - dw) / 2.0, r.y + (r.h - dh) / 2.0, dw, dh)
}

/// O padrão actual (ou um novo, nas cores de nascença).
fn pattern_of(bg: Background) -> Option<Pattern> {
    match bg {
        Background::Pattern(p) => Some(p),
        _ => None,
    }
}

fn rgb_of(c: [u8; 4]) -> Rgb {
    Rgb([c[0], c[1], c[2]])
}

/// A cor que `target` tem agora, em sRGB (para o seletor partir dela).
fn current_rgb(theme: Theme, bg: Background, target: Target) -> [u8; 4] {
    let c = match (target, pattern_of(bg)) {
        (Target::Base, Some(p)) => ph2d_board_render::kanban_tint(theme, p.base, false),
        (Target::Ink, Some(p)) => ph2d_board_render::kanban_tint(theme, p.ink, true),
        _ => ph2d_board_render::kanban_board_bg(theme, bg),
    };
    c.to_rgba8().to_u8_array()
}

/// O fundo com a cor livre `rgb` em `target`.
fn with_custom(bg: Background, target: Target, rgb: Rgb) -> Background {
    match (target, pattern_of(bg)) {
        (Target::Base, Some(p)) => Background::Pattern(Pattern {
            base: Tint::Rgb(rgb),
            ..p
        }),
        (Target::Ink, Some(p)) => Background::Pattern(Pattern {
            ink: Tint::Rgb(rgb),
            ..p
        }),
        _ => Background::Custom(rgb),
    }
}

/// ⭐ O fundo que uma escolha da gaveta dá, a partir de `bg` (`None` = a linha não escolhe fundo).
pub(crate) fn chosen(bg: Background, row: Row) -> Option<Background> {
    Some(match row {
        Row::Color(h) => Background::Color(h),
        Row::Gradient(i) => Background::Gradient(i),
        Row::Pattern(kind) => Background::Pattern(match pattern_of(bg) {
            Some(p) => Pattern { kind, ..p },
            None => Pattern::new(kind),
        }),
        Row::Tint(t, h) => {
            let p = pattern_of(bg)?;
            Background::Pattern(match t {
                Target::Base => Pattern {
                    base: Tint::Hue(h),
                    ..p
                },
                _ => Pattern {
                    ink: Tint::Hue(h),
                    ..p
                },
            })
        }
        _ => return None,
    })
}

impl KanbanDocs {
    /// O fundo do Kanban da aba `id` (com a pré-visualização do seletor, se há).
    pub(crate) fn background_of(&self, id: u64) -> Option<Background> {
        let t = self.tabs.iter().find(|t| t.id() == id)?;
        let shown = t.bg.as_ref().and_then(|d| d.preview);
        let set = self.set.borrow();
        shown.or_else(|| set.get(t.view.kanban).map(|k| k.header.background))
    }

    /// ⭐ Grava `bg` como o fundo do Kanban da aba `id` (UM passo de desfazer). `false` = igual.
    pub(crate) fn set_background(&mut self, id: u64, bg: Background) -> bool {
        self.with_doc(id, |_, doc| {
            let Some(mut h) = doc.kanban().map(|k| k.header.clone()) else {
                return false;
            };
            if h.background == bg {
                return false;
            }
            h.background = bg;
            doc.apply(vec![KanbanOp::Header(h)])
        })
        .unwrap_or(false)
    }

    /// ⭐ O «Upload…» da gaveta pela porta do mundo: a imagem escolhida guarda-se por conteúdo e
    /// passa a ser o fundo. Devolve os avisos (um ficheiro que não é uma imagem que o app lê).
    pub fn serve_background(&mut self, world: &mut dyn crate::World) -> Vec<String> {
        let ids: Vec<u64> = self
            .tabs
            .iter()
            .filter(|t| t.bg.as_ref().is_some_and(|d| d.pick))
            .map(crate::tab::Tab::id)
            .collect();
        let mut notices = Vec::new();
        for id in ids {
            if let Some(d) = self.tab_mut(id).and_then(|t| t.bg.as_deref_mut()) {
                d.pick = false;
            }
            let Some(path) = world.pick_background() else {
                continue;
            };
            let bytes = std::fs::read(&path).ok();
            let size = bytes.as_deref().and_then(crate::back::attach::image_size);
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            match (bytes, size) {
                (Some(b), Some(sz)) => {
                    let f = self.set.borrow_mut().put_file(b, Some(sz));
                    self.set_background(id, Background::Image(f));
                }
                _ => notices.push(ph2d_i18n::tr_with("kanban.bg.not_image", &[("t", &name)])),
            }
        }
        notices
    }
}

/// Abre a gaveta na aba `id` (fecha o arquivo e o menu: é o mesmo lado do ecrã).
pub(crate) fn open(docs: &mut KanbanDocs, id: u64) {
    if let Some(t) = docs.tab_mut(id) {
        t.menu = None;
        t.panel = None;
        t.bg = Some(Box::default());
    }
}

/// O que está em `(x, y)`: `None` = fora da gaveta (ou fechada).
fn hit_at(docs: &KanbanDocs, id: u64, x: f32, y: f32) -> Option<Row> {
    let t = docs.tabs.iter().find(|t| t.id() == id)?;
    let d = t.bg.as_ref()?;
    let bg = docs.background_of(id)?;
    let p = place(d, bg, t.layout.board);
    place::hit(&p, x, y)
}

/// O seletor à posição `(x, y)` de `row` (o quadrado ou a faixa): o hsv novo e a pré-visualização.
fn drag_to(docs: &mut KanbanDocs, id: u64, row: Row, x: f32, y: f32) {
    let Some(bg) = docs.set_kanban_bg(id) else {
        return;
    };
    let Some(t) = docs.tab_mut(id) else {
        return;
    };
    let board = t.layout.board;
    let Some(d) = t.bg.as_deref_mut() else {
        return;
    };
    let Some(target) = d.custom else {
        return;
    };
    let p = place(d, bg, board);
    if row == Row::Sv
        && let Some(r) = p.sv
    {
        d.hsv[1] = ((x - r.x) / r.w).clamp(0.0, 1.0);
        d.hsv[2] = 1.0 - ((y - r.y) / r.h).clamp(0.0, 1.0);
    } else if let Some(r) = p.hue {
        d.hsv[0] = ((x - r.x) / r.w).clamp(0.0, 1.0);
    }
    let [h, s, v] = d.hsv;
    d.preview = Some(with_custom(bg, target, rgb_of(hsv_to_rgba8(h, s, v, 1.0))));
}

impl KanbanDocs {
    /// ⚠️ SONDA: os alvos da gaveta «Change background» aberta (`Color(Green)`, `Pattern(Dots)`,
    /// `Sv`…), com o rect de cada um. Vazia = fechada.
    #[must_use]
    pub fn probe_background_rows(&self, id: u64) -> Vec<(String, Rect)> {
        let Some(t) = self.tabs.iter().find(|t| t.id() == id) else {
            return Vec::new();
        };
        let (Some(d), Some(bg)) = (t.bg.as_deref(), self.background_of(id)) else {
            return Vec::new();
        };
        let p = place(d, bg, t.layout.board);
        let mut out: Vec<(String, Rect)> = p
            .rows
            .iter()
            .map(|(r, rect)| (format!("{r:?}"), *rect))
            .collect();
        out.push(("Close".into(), p.close));
        out.extend(p.sv.map(|r| ("Sv".into(), r)));
        out.extend(p.hue.map(|r| ("HueStrip".into(), r)));
        out
    }

    /// O fundo GRAVADO do Kanban da aba `id` (sem a pré-visualização).
    fn set_kanban_bg(&self, id: u64) -> Option<Background> {
        let t = self.tabs.iter().find(|t| t.id() == id)?;
        self.set
            .borrow()
            .get(t.view.kanban)
            .map(|k| k.header.background)
    }
}

/// Uma escolha (o clique que largou onde carregou).
fn act(docs: &mut KanbanDocs, id: u64, row: Row) {
    let Some(bg) = docs.set_kanban_bg(id) else {
        return;
    };
    let theme = docs
        .tab_mut(id)
        .and_then(|t| t.bg.as_ref().map(|d| d.theme))
        .unwrap_or_default();
    match row {
        Row::Close => {
            if let Some(t) = docs.tab_mut(id) {
                t.bg = None;
            }
        }
        Row::Upload => {
            if let Some(d) = docs.tab_mut(id).and_then(|t| t.bg.as_deref_mut()) {
                d.pick = true;
            }
        }
        Row::Custom(target) => {
            let rgba = current_rgb(theme, bg, target);
            let Some(t) = docs.tab_mut(id) else {
                return;
            };
            let board = t.layout.board;
            if let Some(d) = t.bg.as_deref_mut() {
                if d.custom == Some(target) {
                    d.custom = None;
                } else {
                    let (h, s, v, _) = rgba_to_hsv(rgba);
                    d.custom = Some(target);
                    d.hsv = [h, s, v];
                    // O seletor que abre fica à vista (rola a gaveta até à faixa do matiz).
                    let p = place(d, bg, board);
                    if let Some(hue) = p.hue {
                        let below =
                            hue.y + hue.h + crate::metrics::MENU_PAD - (p.body.y + p.body.h);
                        d.scroll = (d.scroll + below.max(0.0)).min(p.max_scroll);
                    }
                }
            }
        }
        _ => {
            if let Some(next) = chosen(bg, row) {
                docs.set_background(id, next);
                // Escolher um padrão abre as cores dele logo abaixo; outro fundo fecha o seletor.
                if let Some(d) = docs.tab_mut(id).and_then(|t| t.bg.as_deref_mut())
                    && !matches!(row, Row::Tint(..) | Row::Pattern(_))
                {
                    d.custom = None;
                }
            }
        }
    }
}

/// ⭐ O rato sobre a gaveta (antes do quadro). `None` = não é com ela.
pub(crate) fn pointer(
    docs: &mut KanbanDocs,
    id: u64,
    kind: PointerKind,
    x: f32,
    y: f32,
) -> Option<bool> {
    let open = docs.tabs.iter().find(|t| t.id() == id)?.bg.is_some();
    if !open {
        return None;
    }
    let hit = hit_at(docs, id, x, y);
    let d = docs.tab_mut(id)?.bg.as_deref_mut()?;
    match kind {
        PointerKind::Move => {
            if d.dragging
                && let Some(row) = d.press
            {
                drag_to(docs, id, row, x, y);
                return Some(true);
            }
            d.hover = hit;
            if hit.is_some() {
                if let Some(t) = docs.tab_mut(id) {
                    t.hover = None;
                }
                return Some(true);
            }
            None
        }
        PointerKind::Down => {
            let Some(row) = hit else {
                // Carregar fora fecha a gaveta e não faz mais nada (o idioma de um popover): o
                // largar que vem é deste carregar.
                if let Some(t) = docs.tab_mut(id) {
                    t.bg = None;
                    t.press = Some(crate::tab::Press {
                        at: (x, y),
                        hit: crate::tab::Hit::Field,
                    });
                }
                return Some(true);
            };
            d.press = Some(row);
            if matches!(row, Row::Sv | Row::HueStrip) {
                d.dragging = true;
                drag_to(docs, id, row, x, y);
            }
            Some(true)
        }
        PointerKind::Up => {
            let press = d.press.take();
            if std::mem::take(&mut d.dragging) {
                if let Some(next) = d.preview.take() {
                    docs.set_background(id, next);
                }
                return Some(true);
            }
            match press {
                Some(p) if hit == Some(p) => act(docs, id, p),
                Some(_) => {}
                None => return None,
            }
            Some(true)
        }
    }
}

/// A roda sobre a gaveta rola-a. `true` = tomada.
pub(crate) fn wheel(docs: &mut KanbanDocs, id: u64, x: f32, y: f32, dy: f32) -> bool {
    if hit_at(docs, id, x, y).is_none() {
        return false;
    }
    let Some(bg) = docs.background_of(id) else {
        return false;
    };
    let Some(t) = docs.tab_mut(id) else {
        return false;
    };
    let board = t.layout.board;
    let Some(d) = t.bg.as_deref_mut() else {
        return false;
    };
    let max = place(d, bg, board).max_scroll;
    d.scroll = (d.scroll + dy).clamp(0.0, max);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choosing_a_pattern_keeps_the_colours_of_the_current_one() {
        let p = Pattern {
            kind: PatternKind::Dots,
            base: Tint::Hue(Hue::Green),
            ink: Tint::Rgb(Rgb([1, 2, 3])),
        };
        let next = chosen(Background::Pattern(p), Row::Pattern(PatternKind::Waves));
        assert_eq!(
            next,
            Some(Background::Pattern(Pattern {
                kind: PatternKind::Waves,
                ..p
            }))
        );
        assert_eq!(
            chosen(Background::Color(Hue::Red), Row::Pattern(PatternKind::Grid)),
            Some(Background::Pattern(Pattern::new(PatternKind::Grid)))
        );
        assert_eq!(
            chosen(
                Background::Color(Hue::Red),
                Row::Tint(Target::Ink, Hue::Lime)
            ),
            None,
            "sem padrão, a cor do traço não escolhe nada"
        );
        let custom = with_custom(Background::Pattern(p), Target::Ink, Rgb([9, 9, 9]));
        assert_eq!(
            custom,
            Background::Pattern(Pattern {
                ink: Tint::Rgb(Rgb([9, 9, 9])),
                ..p
            })
        );
        assert_eq!(
            with_custom(Background::Gradient(2), Target::Solid, Rgb([9, 8, 7])),
            Background::Custom(Rgb([9, 8, 7]))
        );
    }
}
