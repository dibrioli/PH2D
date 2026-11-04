//! ⭐ **A barra da paleta, como no Aseprite** (smoke do dono 01/11: os botões da paleta caíam abaixo
//! da dobra a 1040 px): o título «Palette» com a contagem da selecção e, à direita, dois ícones que
//! abrem menus — **≡** (mover, inverter, gradientes, ordenar, usadas, do sprite) e **paleta**
//! (ler, gravar, predefinições, a de omissão). As linhas dos menus são os MESMOS ids dos botões de
//! antes: o despacho não mudou.

use super::*;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::widget::{
    ButtonState, Dropdown, DropdownOption, IconButtonStyle, IconGlyph, paint_icon_button,
};

/// Qual dos dois menus da barra está aberto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MenuPaleta {
    Acoes,
    Predefinicoes,
}

impl MenuPaleta {
    pub(crate) const fn id(self) -> NodeId {
        match self {
            Self::Acoes => ids::PALETTE_MENU,
            Self::Predefinicoes => ids::PRESETS_MENU,
        }
    }

    const fn icone(self) -> IconId {
        match self {
            Self::Acoes => IconId::Menu,
            Self::Predefinicoes => IconId::Palette,
        }
    }
}

/// As linhas de um menu: `(id, rótulo)` na ordem do menu da paleta do Aseprite.
fn linhas(menu: MenuPaleta, info: &PixelPanelInfo) -> Vec<(NodeId, String)> {
    let par = |(id, _, k): &(NodeId, ph2d_editor_core::pixel_panel::PixelRequest, &str)| {
        (*id, tr(k).to_owned())
    };
    let [
        esq,
        dir,
        asc,
        desc,
        usadas,
        do_sprite,
        ler,
        gravar,
        omissao,
        inverte,
        grad,
        grad_hue,
        ..,
    ] = &ids::ACTIONS;
    match menu {
        MenuPaleta::Acoes => {
            let mut v: Vec<_> = [esq, dir, inverte, grad, grad_hue]
                .into_iter()
                .map(par)
                .collect();
            v.extend(ids::SORT_KEYS.iter().map(|&(k, id)| {
                (
                    id,
                    tr_with("panel.pixel.sort_by", &[("key", &tr(k.label_key()))]),
                )
            }));
            v.extend([asc, desc, usadas, do_sprite].into_iter().map(par));
            v
        }
        MenuPaleta::Predefinicoes => {
            let mut v: Vec<_> = [ler, gravar].into_iter().map(par).collect();
            v.extend(
                info.presets
                    .iter()
                    .enumerate()
                    .map(|(i, nome)| (ids::preset_id(i), nome.clone())),
            );
            v.extend(ids::PRESET_ACTIONS.iter().map(par));
            v.push(par(omissao));
            v
        }
    }
}

/// O menu aberto como `Dropdown` (a chave ordenada vem marcada no ≡).
pub(crate) fn dropdown(menu: MenuPaleta, info: &PixelPanelInfo) -> Dropdown<NodeId> {
    let opcoes = linhas(menu, info)
        .into_iter()
        .map(|(id, rotulo)| DropdownOption::new(id, id, rotulo))
        .collect();
    let dd = Dropdown::new(menu.id(), "", opcoes).open(true);
    let chave = ids::SORT_KEYS.iter().find(|(k, _)| *k == info.sort_key);
    match (menu, chave) {
        (MenuPaleta::Acoes, Some((_, id))) => dd.selected(*id),
        _ => dd,
    }
}

/// Os ids de TODAS as linhas que um menu da paleta pode ter (um clique numa fecha os dois).
#[must_use]
pub(crate) fn e_linha(id: NodeId) -> bool {
    ids::ACTIONS.iter().any(|(b, ..)| *b == id)
        || ids::PRESET_ACTIONS.iter().any(|(b, ..)| *b == id)
        || ids::SORT_KEYS.iter().any(|(_, b)| *b == id)
        || with_current(|i| (0..i.presets.len()).any(|k| ids::preset_id(k) == id))
}

impl Corpo<'_> {
    /// A barra: «Palette · n selected» e os dois ícones à direita. Devolve o fundo da barra.
    pub(super) fn barra_da_paleta(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let n = info.selected.len().to_string();
        // Num cartão o título diz «Palette»: aqui só a selecção.
        let chave = if self.cartao.is_some() {
            "card.pixel.selected"
        } else {
            "panel.pixel.palette_selected"
        };
        let titulo = tr_with(chave, &[("n", &n)]);
        let side = self.lado();
        let gap = Spacing::Xs.px();
        let menus = [MenuPaleta::Predefinicoes, MenuPaleta::Acoes];
        #[expect(clippy::cast_precision_loss, reason = "dois ícones")]
        let icones_w = menus.len() as f32 * (side + gap);
        paint_text(
            self.text_system,
            self.scene,
            &titulo,
            self.x,
            y + (side - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            (self.w - icones_w).max(1.0),
            tinta(ColorToken::Text3, self.theme),
        );
        for (i, menu) in menus.into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "dois ícones")]
            let x = self.x + self.w - (i as f32 + 1.0) * side - i as f32 * gap;
            let r = Rect::new(x, y, side, side);
            let id = menu.id();
            let open = matches!(
                self.store.get(id),
                Some(InteractiveState::Dropdown { open: true, .. })
            );
            let visual = if open {
                (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
            } else {
                self.store.button_visual(id)
            };
            if !self.icone_no_cartao(r, id, menu.icone(), open) {
                paint_icon_button(
                    r,
                    IconGlyph::Builtin(menu.icone()),
                    IconButtonStyle::Compact,
                    visual,
                    self.scene,
                    self.theme,
                );
                self.hit_index.register(id, r);
            }
            if open {
                self.menu_aberto = Some((r, menu));
            }
        }
        y + side + ph2d_tokens::control_gap_px()
    }
}
