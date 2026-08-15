//! **A DOBRA de um cabeçalho de secção** — o chevron que roda e a placa que desvanece.
//!
//! Irmão do [`super`] pela linha que separa os dois assuntos: aquele diz o que um cabeçalho **É**
//! (o id, o rótulo, a contagem, o círculo de cor, o a11y); isto diz **como ele se dobra**. O corte
//! nasceu no tecto de 500 LOC quando a rotação chegou, e escolheu-se sozinho — as duas peças aqui
//! partilham a forma inteira: são funções puras do `t` da dobra, sem estado, lidas só pelo pintor.
//!
//! ⚠️ **Um DIRECTÓRIO e não um `section_fold.rs` ao lado:** o `ph2d-widget-sync` varre os `*.rs`
//! do topo de `src/widget/` e cada um vira um WIDGET (sem showcase, sem a11y, sem opt-out) — a
//! cicatriz que o `command_palette_tests.rs` pagou. O dir e o `mod.rs` dedupam para um `mod` só.

use super::SectionHeader;
use crate::icons::IconId;
use crate::paint::{paint_icon, paint_icon_rotated};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Color as VelloColor, VectorScene};

/// **O CHEVRON DA DOBRA — um glifo que RODA, e não dois que se trocam.**
///
/// # O facto que torna isto sem costura, e ele foi MEDIDO
///
/// Os dois glifos que o app já mostra são **a mesma seta**: rodando `chevron-down` por **−90°**
/// em torno do centro da viewbox `(12, 12)`,
///
/// | `chevron-down` | rodado −90° | `chevron-right` |
/// |---|---|---|
/// | `6,9` | `9,18` | `9,18` |
/// | `12,15` | `15,12` | `15,12` |
/// | `18,9` | `9,6` | `9,6` |
///
/// — **ao ponto**. Não é uma aproximação que passa despercebida: a trajectória do chevron
/// atravessa exactamente os dois desenhos que já shipavam. Se não fossem rotações um do outro, a
/// forma daria um salto ao chegar a cada extremo, e a wave não valeria a pena.
///
/// # Os dois repousos são BYTE-IDÊNTICOS, de propósito
///
/// Nos extremos o glifo é pintado **sem rotação nenhuma** (o caminho literal de hoje), e só entre
/// eles é que roda. Duas razões, e nenhuma é estética: `cos(90°)` em `f64` vale `6,1e-17` e não
/// zero, então uma rotação "exacta" de 90° introduziria um cisalhamento de um ulp; e um glifo
/// parado alinhado aos eixos é o que o `paint_icon_path` consegue encaixar no grid de pixels
/// (ele arredonda a origem — o mesmo argumento da `motion::on_pixel_grid`). *Nítido parado,
/// suave em movimento.*
///
/// ⚠️ **E o SINAL não é convenção, é o produto: o gate apanhou-o antes do smoke.** A kurbo roda
/// pela matriz `[[cos,−sin],[sin,cos]]` sobre um espaço **y-para-baixo**, então um `+π/2` — o que
/// a aritmética de papel diz — dá `chevron-LEFT`, e a seta giraria ao contrário durante o gesto
/// inteiro. O sentido certo é `−π/2`, e é o `the_two_shipped_chevrons_are_the_same_arrow_rotated`
/// que o afirma contra o glifo REAL em vez de contra a minha conta.
///
/// ⚠️ **E a rotação acontece DENTRO da viewbox, antes do encaixe.** O `paint_icon_path` escala a
/// VIEWBOX (24×24) e não a bbox do caminho — rodar depois faria a caixa passar de larga-e-baixa a
/// alta-e-estreita e o glifo **respiraria de tamanho** a meio do gesto.
pub(super) fn paint_chevron(scene: &mut VectorScene, rect: Rect, t: f32, color: VelloColor) {
    let t = t.clamp(0.0, 1.0);
    let stroke = StrokeToken::Default.px();
    if t >= 1.0 {
        paint_icon(scene, IconId::ChevronDown, rect, color, stroke);
        return;
    }
    if t <= 0.0 {
        paint_icon(scene, IconId::ChevronRight, rect, color, stroke);
        return;
    }
    // Aberta (`t = 1`) é 0°; fechada (`t = 0`) é −90° — a rotação que leva `Down` a `Right`.
    let angle = f64::from(1.0 - t) * -std::f64::consts::FRAC_PI_2;
    paint_icon_rotated(scene, IconId::ChevronDown, rect, color, stroke, angle);
}

/// **A placa escura de um cabeçalho dobrado — a DECISÃO, separada do desenho.**
///
/// Extraída para ser gateável: o defeito que ela teve não é visível num pixel isolado, é a
/// ASSIMETRIA entre os dois sentidos do gesto, e isso mede-se comparando duas decisões.
pub(super) fn plate_color(header: &SectionHeader, theme: Theme) -> Option<ph2d_tokens::Color> {
    let t = header.fold_t();
    if header.collapsible.is_none() || t >= 1.0 {
        return None;
    }
    crate::motion::blend_token_color(Some(ColorToken::Bg3.resolve(theme)), None, t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;
    use ph2d_vector::{Affine, Point};

    /// ⭐ **O FACTO que torna a rotação sem costura: os dois glifos que o app já mostra são a
    /// MESMA seta.**
    ///
    /// Rodar `chevron-down` por +90° em torno do centro da viewbox tem de dar `chevron-right` —
    /// não «parecido», o mesmo caminho. É isto que garante que a trajectória do chevron ATRAVESSA
    /// exactamente os dois desenhos que já shipavam; se falhasse, a forma daria um salto ao
    /// chegar a cada extremo e a wave seria uma regressão disfarçada de animação.
    ///
    /// *Mutação: rodar +90° (o sentido que a conta de papel dá, e que produz `chevron-left`)
    /// ⇒ sangra — e foi assim que ele apanhou o meu sinal errado ANTES do smoke.*
    #[test]
    fn the_two_shipped_chevrons_are_the_same_arrow_rotated() {
        use crate::icons::cmd_to_path;
        use ph2d_vector::BezPath;

        let build = |icon: IconId| {
            let mut p = BezPath::new();
            for cmd in icon.cmds() {
                p.extend(cmd_to_path(*cmd));
            }
            p
        };
        let mut down = build(IconId::ChevronDown);
        down.apply_affine(Affine::rotate_about(
            -std::f64::consts::FRAC_PI_2,
            Point::new(12.0, 12.0),
        ));
        let right = build(IconId::ChevronRight);

        let pts = |p: &BezPath| -> Vec<(f64, f64)> {
            p.elements()
                .iter()
                .flat_map(|e| e.end_point().map(|q| (q.x, q.y)))
                .collect()
        };
        let (a, b) = (pts(&down), pts(&right));
        assert!(
            !a.is_empty(),
            "o caminho saiu VAZIO — o gate não pode falhar por comparar nada com nada"
        );
        assert_eq!(
            a.len(),
            b.len(),
            "contagem de pontos difere: {a:?} vs {b:?}"
        );
        for (i, (p, q)) in a.iter().zip(b.iter()).enumerate() {
            assert!(
                (p.0 - q.0).abs() < 1e-9 && (p.1 - q.1).abs() < 1e-9,
                "ponto {i}: rodado {p:?} != chevron-right {q:?}"
            );
        }
    }

    /// **A placa segue o `t`, não o flag semântico** — senão ela desvanece a FECHAR e some de
    /// repente a ABRIR, e o cabeçalho fica com duas metades a discordar sobre quando a secção
    /// fechou.
    ///
    /// O oráculo é a ASSIMETRIA: a meio do gesto, os dois sentidos têm de decidir o MESMO.
    /// *Mutação: gatear em `matches!(collapsible, Some(false))` ⇒ o sentido de abrir devolve
    /// `None` e sangra.*
    #[test]
    fn the_header_plate_follows_the_fold_not_the_semantic_flag() {
        let theme = Theme::Forge;
        let mid_closing = SectionHeader::new(NodeId(1), "x")
            .collapsible(false)
            .open_t(0.4);
        let mid_opening = SectionHeader::new(NodeId(1), "x")
            .collapsible(true)
            .open_t(0.4);
        assert_eq!(
            plate_color(&mid_closing, theme),
            plate_color(&mid_opening, theme),
            "a placa decide diferente conforme o SENTIDO do gesto"
        );
        // CONTROLO: a meio ela EXISTE — sem isto o gate compararia `None` com `None`.
        assert!(plate_color(&mid_closing, theme).is_some());
        // E os dois repousos são o mundo de hoje: cheia fechada, ausente aberta.
        let shut = SectionHeader::new(NodeId(1), "x").collapsible(false);
        let open = SectionHeader::new(NodeId(1), "x").collapsible(true);
        assert_eq!(
            plate_color(&shut, theme),
            Some(ColorToken::Bg3.resolve(theme))
        );
        assert!(plate_color(&open, theme).is_none());
        // E um cabeçalho NÃO-dobrável nunca teve placa.
        assert!(plate_color(&SectionHeader::new(NodeId(1), "x"), theme).is_none());
    }
}
