//! ⭐⭐⭐ **A GEOMETRIA de uma pílula — o que ela GASTA, e o caminho inverso.**
//!
//! ⚠️ **Saiu do [`super`] por TETO DE LOC** (`521 > 500`, 2026-09-19) e o corte é de **assunto**,
//! o mesmo do [`crate::paint_label_box`] um nível acima: o ficheiro-mãe responde *que aspecto tem
//! uma pílula*; este responde *quanto dela é do TEXTO*. As cinco funções são uma lei só — ida
//! ([`Tag::width_for`]), volta ([`Tag::label_budget`]), a faixa que o pintor usa
//! ([`Tag::label_rect`]) e o sítio do `×` ([`Tag::close_rect`]) —, e é por isso que elas viajam
//! juntas.
//!
//! ⛔⛔ **Antes desta wave havia TRÊS cópias dos mesmos dois números** (o `close_rect`, o pintor, e
//! um `chip_w` no painel do Inspector), e a terceira cobrou: *uma lei escrita em dois sítios ainda
//! não é uma lei — só uma PORTA é.*

use super::{Tag, TypeToken};
use crate::zones::Rect;
use ph2d_text::TextSystem;

/// A fonte de um rótulo de pílula — **uma**, e é por ela que a [`Tag::natural_width`] mede.
pub(super) const LABEL_FONT_SIZE_TOKEN: TypeToken = TypeToken::Xs;

/// O recuo horizontal de uma pílula: metade da altura dela, com piso.
fn pad_x(h: f32) -> f32 {
    (h * 0.5).max(8.0) // LITERAL-PX-OK: tag pill horizontal pad scales with height (chrome geometry)
}

/// O `×` de uma pílula removível: 70 % da altura, entre `10` e `16`.
fn close_size(h: f32) -> f32 {
    (h * 0.7).clamp(10.0, 16.0) // LITERAL-PX-OK: close icon scales 70% of pill height with min/max
}

/// ⭐⭐ **Tudo o que uma pílula REMOVÍVEL gasta que não é rótulo** — recuo à esquerda, meio recuo
/// antes do `×`, o `×`, e o recuo à direita.
///
/// ⛔ Ela é a razão de [`Tag::label_rect`] e [`Tag::width_for`] não poderem divergir: as duas
/// chamam-na, em vez de cada uma somar os mesmos quatro termos à sua maneira.
fn removable_chrome(h: f32) -> f32 {
    pad_x(h) * 2.5 + close_size(h) // LITERAL-PX-OK: contagem de vaos (1 + 0,5 + 1), nao uma medida
}

impl Tag {
    /// Rect of the close `X` icon inside `host` (or `None` for
    /// non-removable tags). Hosts use this to register a dedicated
    /// hit zone for the close action so a click on the X removes
    /// the tag rather than activating the whole pill.
    pub fn close_rect(&self, host: Rect) -> Option<Rect> {
        if !self.removable {
            return None;
        }
        let pad_x = pad_x(host.h);
        let close_size = close_size(host.h);
        // ⚠️ O `X` mora numa ESQUINA e o host é variável: `pad_x + close_size` pode passar da
        // largura da pílula, e aí a borda esquerda do ícone cai FORA dela — por cima do rótulo,
        // que é o vizinho da esquerda. O piso é `host.x`; num host mais estreito que o próprio
        // ícone nada cabe, e transbordar pela DIREITA (na borda da pílula) é o menor dos males.
        Some(Rect::new(
            (host.x + host.w - pad_x - close_size).max(host.x),
            host.y + (host.h - close_size) * 0.5,
            close_size,
            close_size,
        ))
    }

    /// ⭐⭐⭐ **A FAIXA QUE O RÓTULO OCUPA** dentro da pílula.
    ///
    /// Numa pílula **removível** ela para antes do `×`; numa pílula lisa é a caixa inteira, porque
    /// ali o texto é centrado e o respiro dele é o da caixa de rótulo da casa.
    #[must_use]
    pub fn label_rect(&self, host: Rect) -> Rect {
        if !self.removable {
            return host;
        }
        let pad = pad_x(host.h);
        Rect::new(
            host.x + pad,
            host.y,
            (host.w - removable_chrome(host.h)).max(0.0),
            host.h,
        )
    }

    /// ⭐⭐⭐ **O ORÇAMENTO do rótulo — o que a pílula lhe dá depois do que ela GASTA.**
    ///
    /// ⛔⛔ **Ela nasce de um defeito medido** (2026-09-19, varredura das elisões): o pintor
    /// entregava a faixa acima ao [`crate::paint::paint_text_centered`], que **volta a descontar**
    /// o respiro de uma caixa de rótulo — e numa pílula esse respiro **já foi pago**, porque o
    /// `pad_x` dela é maior do que ele. Medido: `filter` mede `24,26` e recebia `12,13`; na secção
    /// *Tags* do Inspector **todo** chip recebia metade do que pedia (`Ground` `38,95 → 22,95`),
    /// e a varredura do app não via nenhum deles porque um Inspector de fábrica não tem objecto.
    ///
    /// ⚠️ *Contar o respiro de um vizinho que não existe é a mesma família de «um orçamento é a
    /// largura de um ESPAÇO»: o número deixou de descrever a caixa.*
    #[must_use]
    pub fn label_budget(&self, host: Rect) -> f32 {
        let faixa = self.label_rect(host);
        if self.removable {
            faixa.w
        } else {
            crate::paint::label_budget(faixa.w)
        }
    }

    /// ⭐⭐⭐ **O CAMINHO INVERSO: que largura de pílula um texto de `text_w` precisa.**
    ///
    /// ⛔⛔⛔ **O `next_up` não é paranoia — é a única linha que faz a ida-e-volta FECHAR.** Em
    /// `f32` `(t + c) − c` fica **abaixo** de `t` em ~`96 %` do domínio desta pílula (pior défice
    /// medido `3,05e-5 px`), e a elisão compara `<=`: *um défice de um ULP corta a palavra
    /// inteira*. ⇒ a inversa confere-se contra a **LEI** ([`Self::label_budget`]) e nunca contra a
    /// álgebra que a escreveu.
    #[must_use]
    pub fn width_for(text_w: f32, h: f32, removable: bool) -> f32 {
        if !removable {
            // Uma pílula lisa É uma caixa de rótulo — o par dela já existe e é o da casa.
            return crate::paint::rect_for_label(text_w);
        }
        let chrome = removable_chrome(h);
        let w = text_w + chrome;
        if w - chrome < text_w { w.next_up() } else { w }
    }

    /// ⭐⭐ **A largura NATURAL desta pílula** — a inversa alimentada pelo rótulo que ela carrega.
    ///
    /// ⚠️ Ela existe para o consumidor não ter de saber a FONTE: medir a `Sm` e pintar a `Xs` é a
    /// mesma classe de defeito que medir num peso e pintar noutro.
    #[must_use]
    pub fn natural_width(&self, text_system: &mut TextSystem, h: f32) -> f32 {
        let texto = text_system.prefix_width(&self.label, LABEL_FONT_SIZE_TOKEN.px());
        Self::width_for(texto, h, self.removable)
    }
}
