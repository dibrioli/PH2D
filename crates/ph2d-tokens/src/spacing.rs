//! Spacing scale tokens. Source: `docs/design/tokens.json` → `spacing.*`.
//!
//! 8 px base scale with sub-base steps for tight UI density. The
//! canonical section gap is 14 px (non-power-of-2 — design choice;
//! see tokens.json).
//!
//! Wave 4 stage A: values now come from `crate::generated::SPACING_*`,
//! `DENSITY_*` and `CHROME_*` const tables (codegen'd by build.rs from
//! `docs/design/tokens.json`). Designer edits the JSON; Rust replicates.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Spacing {
    /// `xxs` — 2 px (sub-pixel divider gaps).
    Xxs,
    /// `xs` — 4 px (icon ↔ label tight inline).
    Xs,
    /// `sm` — 6 px (compact rows).
    Sm,
    /// `md` — 8 px (default inline padding).
    Md,
    /// `lg` — 12 px (default vertical rhythm, panel padding).
    Lg,
    /// `xl` — 16 px (comfortable padding).
    Xl,
    /// `xl2` (`2xl` in JSON) — 24 px (section separation).
    Xl2,
    /// `xl3` (`3xl` in JSON) — 32 px (major panel margin).
    Xl3,
    /// `xl4` (`4xl` in JSON) — 48 px (hero spacing).
    Xl4,
}

impl Spacing {
    /// O valor de **FÁBRICA** — a tabela gerada do `tokens.json`, sem passar pela camada de
    /// override. `const fn`, e é isso que a mantém legal em contexto `const`.
    ///
    /// ⚠️ Quem quer o número que o app **DESENHA** chama [`Spacing::px`]. Os dois nomes existem
    /// para que o sítio de uso diga qual das duas perguntas está a fazer: um nome só tornaria a
    /// diferença invisível no diff.
    pub const fn factory_px(self) -> f32 {
        match self {
            Self::Xxs => crate::generated::SPACING_XXS,
            Self::Xs => crate::generated::SPACING_XS,
            Self::Sm => crate::generated::SPACING_SM,
            Self::Md => crate::generated::SPACING_MD,
            Self::Lg => crate::generated::SPACING_LG,
            Self::Xl => crate::generated::SPACING_XL,
            Self::Xl2 => crate::generated::SPACING_XL2,
            Self::Xl3 => crate::generated::SPACING_XL3,
            Self::Xl4 => crate::generated::SPACING_XL4,
        }
    }

    /// O valor **VIVO** — o que o artista autorou neste modo, ou a fábrica.
    ///
    /// ⚠️ **Não recebe modo, e isso é a wave inteira numa assinatura:** a pergunta *"qual é o modo
    /// vigente?"* é respondida **uma vez por quadro** pelo [`crate::num_runtime::publish`], que a
    /// lê de onde ela é POSSUÍDA. Enfiá-la aqui obrigaria os ~1200 sítios de leitura a carregá-la,
    /// e a resposta seria a mesma nos 1200.
    ///
    /// ⚠️ Com a escala de fábrica intacta isto é **uma leitura de bool** e o resultado é o
    /// [`Spacing::factory_px`], bit a bit.
    #[must_use]
    pub fn px(self) -> f32 {
        crate::num_runtime::live(crate::num::NumToken::Spacing(self)).unwrap_or(self.factory_px())
    }

    /// Token id (matches JSON key).
    pub const fn id(self) -> &'static str {
        match self {
            Self::Xxs => "xxs",
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xl2 => "2xl",
            Self::Xl3 => "3xl",
            Self::Xl4 => "4xl",
        }
    }
}

/// Fixed section gap (non-power-of-2). Per tokens.json `chrome.section-gap`.
pub const SECTION_GAP_PX: f32 = crate::generated::CHROME_SECTION_GAP;

/// Row height by density. Per tokens.json `density.*`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Density {
    /// `compact` row height — max density.
    Compact,
    /// `cozy` row height — balanced.
    Cozy,
    /// `comfortable` row height — comfortable (default for tablet/Pencil).
    #[default]
    Comfortable,
}

impl Density {
    pub const fn row_h_px(self) -> f32 {
        match self {
            Self::Compact => crate::generated::DENSITY_COMPACT,
            Self::Cozy => crate::generated::DENSITY_COZY,
            Self::Comfortable => crate::generated::DENSITY_COMFORTABLE,
        }
    }
}

/// Default icon-button square size. Per tokens.json `chrome.icon-btn-size`.
pub const ICON_BTN_SIZE_PX: f32 = crate::generated::CHROME_ICON_BTN_SIZE;

/// Default body row height. Per tokens.json `chrome.row-h`.
pub const ROW_H_PX: f32 = crate::generated::CHROME_ROW_H;

/// ⭐⭐⭐ **O AVANÇO de uma linha de formulário para a seguinte — e é UMA resposta, não quatro.**
///
/// Enio, 2026-09-06 (com as fotos do Blender e do Godot lado a lado): *«Blender e Godot com
/// aspecto muito mais compacto e profissional. Espaçamento muito regrado e universal.»*
///
/// ⚠️ **A palavra é «universal», e ela nomeia o mecanismo, não o número.** Medido no fonte do
/// Godot (MIT, `editor/themes/theme_modern.cpp`): **nenhuma constante de espaço daquele tema é
/// escolhida** — todas são `base_margin · k` a partir de um `base_spacing = 4`, e a que os
/// contentores lêem tem NOME próprio (`separation_margin`), lido por `BoxContainer`,
/// `HBoxContainer`, `VBoxContainer`, `GridContainer`, `FlowContainer` e `FoldableContainer`.
/// *Um nome é o que impede a segunda resposta.*
///
/// ⛔ **E nós tínhamos QUATRO respostas para a MESMA pergunta**, censadas em 2026-09-06:
/// `Xs` (4 px) em 63 sítios · **`Sm` (6 px) em 20** · `Xxs` (2 px) em 3 · `Md` (8 px) em 1.
/// Os 20 do `Sm` não estavam espalhados: eram o **Inspector** e o **Painter Layers** inteiros —
/// os dois painéis em que o artista vive respiravam 50 % mais que o resto do app, e nenhum
/// teste podia ver isso, porque cada sítio estava certo sozinho.
///
/// **O valor é o do modelo:** `separation_margin` do Godot Modern = `base_spacing` = **4 px**,
/// que é o `Spacing::Xs` desta casa. ⚠️ E ele é o número da PILHA de linhas de formulário, que
/// é a superfície destes painéis — o Godot tem outros dois para outras duas superfícies, e cada
/// um é derivado, nunca escolhido: uma **lista** (`Tree`) tem `pow(base·0.175, 3)` = **0**, as
/// linhas encostam; uma **grelha** tem `widget_margin.y − 2` = **3**. ⇒ quando esta casa precisar
/// de uma dessas, ela nasce **aqui, com nome e derivação**, e não num `+ Spacing::Qualquer` no
/// sítio da pintura.
pub fn row_pitch_px() -> f32 {
    ROW_H_PX + row_gap_px()
}

/// ⭐⭐ **O VÃO sozinho — para quem empilha linhas de ALTURA VARIÁVEL.**
///
/// É este o primitivo do modelo, e o [`row_pitch_px`] é a conveniência: no Godot o que tem nome é
/// a **separação** (`separation_margin`), e é o contentor que a soma à altura de cada filho.
/// ⚠️ Descobri-o pela construção — a 1.ª versão desta porta só sabia responder `altura + vão`, e
/// havia sítios a empilhar uma caixa cuja altura é medida em tempo de pintura (`y + h + vão`).
/// *Uma porta que só serve a metade dos chamadores deixa a outra metade a escrever o número.*
pub fn row_gap_px() -> f32 {
    Spacing::Xs.px()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_strictly_increasing() {
        let scale = [
            Spacing::Xxs,
            Spacing::Xs,
            Spacing::Sm,
            Spacing::Md,
            Spacing::Lg,
            Spacing::Xl,
            Spacing::Xl2,
            Spacing::Xl3,
            Spacing::Xl4,
        ];
        for w in scale.windows(2) {
            assert!(
                w[0].px() < w[1].px(),
                "scale broken at {:?} → {:?}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn ids_match_tokens_json() {
        assert_eq!(Spacing::Xxs.id(), "xxs");
        assert_eq!(Spacing::Xl2.id(), "2xl");
        assert_eq!(Spacing::Xl4.id(), "4xl");
    }

    #[test]
    fn density_row_height_strictly_increasing() {
        assert!(Density::Compact.row_h_px() < Density::Cozy.row_h_px());
        assert!(Density::Cozy.row_h_px() < Density::Comfortable.row_h_px());
    }

    #[test]
    fn comfortable_is_default() {
        assert_eq!(Density::default(), Density::Comfortable);
    }
}
