//! ⭐ **A barra de menus da aba de scripts** (13 §4 itens 11 e 13b, D3): os títulos à esquerda, o
//! caminho até o cursor à direita e a engrenagem de «Script options» no fim. Esta crate pinta e
//! responde ao rato; quem a usa ([`ph2d-app-code`]) dá os títulos ([`TabBar::paint`]) e as linhas do
//! menu aberto ([`menu::MenuItem`]), e corre o comando que [`TabBar::press`] devolve.
//!
//! O menu aberto pode ter linhas com **submenu em cascata** (`View ▸ Text rendering ▸ …`): a
//! mecânica (pairar abre, a escolha fecha tudo, Esc fecha um nível) está em [`menu`].

pub mod menu;

use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{folga_do_texto, paint_icon, paint_text_in_weight, resolve};
use ph2d_editor_core::widget::paint_popover_row_highlight;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::mono_label;

pub use crate::cards::{bar_height as height, split_bar as split};

/// O que a barra pintou no último quadro (o rato e os testes lêem-no).
#[derive(Clone, Debug)]
pub struct BarPainted<M> {
    pub header: Rect,
    pub titles: Vec<(M, Rect)>,
    /// As linhas do menu aberto: o comando (ou a linha de um submenu) e a caixa.
    pub rows: Vec<(NodeId, Rect)>,
    pub popup: Option<Rect>,
    /// As linhas de [`rows`](Self::rows) que abrem um submenu.
    pub sub_parents: Vec<NodeId>,
    /// O submenu aberto, se algum.
    pub sub: Option<menu::SubPainted>,
    /// A parte «função» do caminho: a caixa, a linha e a coluna (0-based) da definição.
    pub crumb_fn: Option<(Rect, usize, usize)>,
    /// O caminho como saiu: a caixa do primeiro ao último glifo e as partes pintadas (já com `…`).
    pub crumb: Option<(Rect, Vec<String>)>,
    pub gear: Option<Rect>,
    /// As dicas do rato: a engrenagem, a função do caminho e o alfinete do cartão das opções.
    pub tips: Vec<(Rect, String)>,
    /// A largura em que os menus e a engrenagem cabem sem se tocar (o mínimo da coluna do documento).
    pub min_w: f32,
}

impl<M> Default for BarPainted<M> {
    fn default() -> Self {
        Self {
            header: Rect::default(),
            titles: Vec::new(),
            rows: Vec::new(),
            popup: None,
            sub_parents: Vec::new(),
            sub: None,
            crumb_fn: None,
            crumb: None,
            gear: None,
            tips: Vec::new(),
            min_w: 0.0,
        }
    }
}

/// O estado da barra: o menu aberto, o submenu aberto e a linha sob o rato.
#[derive(Clone, Debug)]
pub struct TabBar<M> {
    pub open: Option<M>,
    pub hover: Option<NodeId>,
    /// A linha do menu aberto cujo submenu está aberto.
    pub sub: Option<NodeId>,
    pub painted: BarPainted<M>,
    /// A dica sob o rato (índice em `painted.tips`) e desde quando.
    pub tip: Option<(usize, u128)>,
    /// A âncora da mira até ao submenu ([`menu`]): o último ponto do rato FORA da mira (fica parada
    /// enquanto o rato vai a caminho, para o tremor da mão não a desfazer).
    pub last: Option<(f32, f32)>,
    /// Quando chegou o evento anterior do rato (uma pausa longa a meio da mira acaba-a).
    pub moved_at: u128,
}

impl<M> Default for TabBar<M> {
    fn default() -> Self {
        Self {
            open: None,
            hover: None,
            sub: None,
            painted: BarPainted::default(),
            tip: None,
            last: None,
            moved_at: 0,
        }
    }
}

fn font() -> f32 {
    TypeToken::Sm.px()
}

/// A largura de `s` na letra da interface da barra (atalhos, caminho, notas).
pub fn width_of(ts: &mut TextSystem, s: &str) -> f32 {
    use crate::peso::{CORPO, largura_em};
    largura_em(ts, s, font(), CORPO)
}

/// Pinta `s` na letra da interface da barra, centrado na altura de `r`.
pub fn label(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    r: Rect,
    ink: ColorToken,
    theme: &Theme,
) {
    let y = r.y + folga_do_texto(r.h, font());
    let w = r.w.max(0.0) + Spacing::Xs.px();
    let (c, corpo) = (resolve(ink, *theme), crate::peso::CORPO);
    paint_text_in_weight(ts, scene, s, [r.x, y], font(), w, c, corpo);
}

/// O caminho até o cursor: o script, a função que o contém (nome, linha e coluna da definição) e a
/// linha do cursor (1-based).
pub struct Crumb {
    pub script: String,
    pub func: CrumbFunc,
    pub line: usize,
}

/// A função que contém o cursor: nome, linha e coluna da definição.
pub type CrumbFunc = Option<(String, usize, usize)>;

impl<M: Copy + PartialEq> TabBar<M> {
    /// Pinta a barra em `r` com os `titles` (o fundo é o do cartão, pintado por quem chama).
    pub fn paint(
        &mut self,
        r: Rect,
        titles: &[(M, &str)],
        crumb: &Crumb,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let pad = crate::cards::well_pad();
        let sm = Spacing::Sm.px();
        self.painted = BarPainted {
            header: r,
            ..BarPainted::default()
        };
        let bar_bg = resolve(crate::cards::card_tone(), *theme);
        let mut x = r.x + pad;
        for &(m, title) in titles {
            let w = mono_label::width(ts, title) + sm * 2.0;
            let tr = Rect::new(x, r.y, w, r.h);
            let open = self.open == Some(m);
            if open {
                paint_popover_row_highlight(scene, tr, *theme);
            }
            let (ink, bg) = if open {
                (ColorToken::AccentFg, resolve(ColorToken::Accent, *theme))
            } else {
                (ColorToken::Text1, bar_bg)
            };
            let lr = Rect::new(x + sm, r.y, f32::INFINITY, r.h);
            mono_label::paint_on(ts, scene, *theme, title, resolve(ink, *theme), bg, lr);
            self.painted.titles.push((m, tr));
            x += w;
        }
        // A engrenagem no fim; o caminho encostado à esquerda dela.
        let side = r.h;
        self.painted.min_w = x - r.x + sm + side + pad;
        let gear = Rect::new(r.x + r.w - pad - side, r.y, side, side);
        let inset = Spacing::Xs.px();
        let glyph = Rect::new(
            gear.x + inset,
            gear.y + inset,
            side - inset * 2.0,
            side - inset * 2.0,
        );
        paint_icon(
            scene,
            IconId::Settings,
            glyph,
            resolve(ColorToken::Text2, *theme),
            StrokeToken::Default.px(),
        );
        self.painted.gear = Some(gear);
        let gear_tip = ph2d_i18n::tr("code.menu.options_tip").to_owned();
        self.painted.tips.push((gear, gear_tip));
        self.paint_crumb(r, x, gear, crumb, scene, ts, theme);
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_crumb(
        &mut self,
        r: Rect,
        x: f32,
        gear: Rect,
        crumb: &Crumb,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let sm = Spacing::Sm.px();
        let sep = "\u{203a}";
        let line = ph2d_i18n::tr_with("code.crumb.line", &[("line", &crumb.line.to_string())]);
        let parts: Vec<(String, bool)> = [
            Some((crumb.script.clone(), false)),
            crumb.func.as_ref().map(|f| (f.0.clone(), true)),
            Some((line, false)),
        ]
        .into_iter()
        .flatten()
        .collect();
        let sep_pad = Spacing::Xs.px();
        let sep_w = width_of(ts, sep) + sep_pad * 2.0;
        let (left, right) = (x + sm, gear.x - sm);
        let parts = fit_crumb(ts, parts, right - left, sep_w);
        if parts.is_empty() {
            return;
        }
        let total: f32 = parts.iter().map(|p| width_of(ts, &p.0)).sum::<f32>()
            + sep_w * (parts.len() - 1) as f32;
        let mut cx = (right - total).max(left);
        let start = cx;
        for (i, (s, is_fn)) in parts.iter().enumerate() {
            if i > 0 {
                let sr = Rect::new(cx + sep_pad, r.y, sep_w - sep_pad * 2.0, r.h);
                label(ts, scene, sep, sr, ColorToken::Text3, theme);
                cx += sep_w;
            }
            let w = width_of(ts, s);
            let pr = Rect::new(cx, r.y, w, r.h);
            let ink = if *is_fn {
                ColorToken::Accent
            } else {
                ColorToken::Text2
            };
            label(ts, scene, s, pr, ink, theme);
            if *is_fn && let Some(f) = &crumb.func {
                self.painted.crumb_fn = Some((pr, f.1, f.2));
                let tip = ph2d_i18n::tr_with("code.crumb.go_tip", &[("name", &f.0)]);
                self.painted.tips.push((pr, tip));
            }
            cx += w;
        }
        let shown = parts.into_iter().map(|p| p.0).collect();
        self.painted.crumb = Some((Rect::new(start, r.y, cx - start, r.h), shown));
    }

    /// A dica sob o rato, depois de `pause_ns` parado, por baixo da caixa dela.
    pub fn paint_tip(
        &self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
        now: u128,
        pause_ns: u128,
    ) {
        let Some((i, since)) = self.tip else { return };
        let Some((r, msg)) = self.painted.tips.get(i) else {
            return;
        };
        if now.saturating_sub(since) < pause_ns {
            return;
        }
        let w = width_of(ts, msg) + Spacing::Sm.px() * 2.0;
        let x = r.x.min(r.x + r.w - w);
        let rect = Rect::new(x, r.y + r.h, w, height());
        let tip = ph2d_editor_core::widget::Tooltip::new(menu::POPOVER_ID, msg.clone());
        ph2d_editor_core::widget::paint_tooltip(&tip, rect, scene, ts, *theme);
    }

    /// O título sob `(x, y)`.
    #[must_use]
    pub fn title_at(&self, x: f32, y: f32) -> Option<M> {
        self.painted
            .titles
            .iter()
            .find(|t| t.1.contains(x, y))
            .map(|t| t.0)
    }

    /// A dica sob `(x, y)` passa a contar desde `now` (a mesma continua a contar).
    pub fn track_tip(&mut self, x: f32, y: f32, now: u128) {
        let at = self.painted.tips.iter().position(|t| t.0.contains(x, y));
        self.tip = at.map(|i| match self.tip {
            Some((j, since)) if j == i => (i, since),
            _ => (i, now),
        });
    }
}

/// `text` como cabe em `w` na letra da barra: inteiro, ou o maior começo seguido de `…` (vazio se
/// nem o `…` cabe).
fn elided(ts: &mut TextSystem, text: &str, w: f32) -> String {
    if width_of(ts, text) <= w {
        return text.to_owned();
    }
    let mut ends: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    while let Some(end) = ends.pop() {
        let s = format!("{}{ELLIPSIS}", &text[..end]);
        if width_of(ts, &s) <= w {
            return s;
        }
    }
    String::new()
}

const ELLIPSIS: &str = "\u{2026}";

/// As partes do caminho que cabem em `room` (dono: o rótulo que não cabe é UMA linha com `…`, nunca
/// por cima de nada): encurta o script, depois a função, e guarda «line N»; se nem assim, o caminho
/// inteiro numa só parte cortada em `…`.
fn fit_crumb(
    ts: &mut TextSystem,
    mut parts: Vec<(String, bool)>,
    room: f32,
    sep_w: f32,
) -> Vec<(String, bool)> {
    let width = |ts: &mut TextSystem, ps: &[(String, bool)]| {
        ps.iter().map(|p| width_of(ts, &p.0)).sum::<f32>() + sep_w * (ps.len() - 1) as f32
    };
    let mut over = width(ts, &parts) - room;
    if over <= 0.0 {
        return parts;
    }
    let ell = width_of(ts, ELLIPSIS);
    let whole = parts
        .iter()
        .map(|p| p.0.as_str())
        .collect::<Vec<_>>()
        .join(" \u{203a} ");
    // Todas menos a última («line N»), da primeira para a última.
    let n = parts.len() - 1;
    for p in parts.iter_mut().take(n) {
        let w = width_of(ts, &p.0);
        p.0 = elided(ts, &p.0, (w - over).max(ell));
        over += width(ts, std::slice::from_ref(p)) - w;
        if over <= 0.0 {
            break;
        }
    }
    if width(ts, &parts) <= room {
        return parts;
    }
    let s = elided(ts, &whole, room.max(0.0));
    if s.is_empty() {
        Vec::new()
    } else {
        vec![(s, false)]
    }
}
