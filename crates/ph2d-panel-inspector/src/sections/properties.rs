//! ⭐⭐⭐ **O CARTÃO DE PROPRIEDADES** — *«o que este objecto DIZ que é»*, logo abaixo do cartão de
//! instância.
//!
//! Ver [`ph2d_editor_core::screens::hero::InspectorPropertiesInfo`] para o buraco que ele fecha
//! (report do Enio, 2026-08-31: *«quando mudo o conteúdo entre `{}` o inspector não muda»*).
//!
//! # ⚠️ Uma fileira, duas espécies
//!
//! | quantos valores | como se pinta | porquê |
//! |---|---|---|
//! | `> 1` | chips, o vigente em `Accent` | há para onde ir, e o clique **troca** de versão |
//! | `1` | o valor em texto | *um valor que não leva a lado nenhum não é oferecido* |
//!
//! ⛔ **Pintar o valor único como botão aceso seria um controlo morto** — a espécie que a caça de
//! 2026-08-30 registou 34 vezes: o clique existe, o artista carrega, e nada acontece.

use super::*;
use ph2d_editor_core::screens::hero::InspectorPropertiesInfo;

/// Que fatia da largura o NOME da propriedade leva.
///
/// ⚠️ **Uma fracção com tecto, e não um número fixo**: num painel estreito um rótulo fixo comeria a
/// fileira toda e os chips ficariam ilegíveis; num painel largo um rótulo proporcional afastaria a
/// pergunta das respostas. ⇒ ele cresce até caber `Size`/`State` e pára.
const AXIS_LABEL_FRACTION: f32 = 0.28; // LITERAL-PX-OK: proporção do rótulo, domínio do cartão

/// O tecto do rótulo, em px.
const AXIS_LABEL_MAX_PX: f32 = 72.0; // LITERAL-PX-OK: tecto do rótulo, domínio do cartão

/// A margem de dentro do cartão — irmã da do cartão de instância.
const CARD_PAD: f32 = 8.0; // LITERAL-PX-OK: inset do cartão, irmão do BODY_PAD do corpo

/// ⭐ **O rótulo do modo PLANO** — o nome que a fileira leva quando a família não se declara em
/// eixos.
///
/// ⚠️ **Ele mora AQUI, e não na lei** (HR-15): a lei devolve o nome vazio e o painel nomeia-o,
/// porque é o painel que o portão da HR-15 varre. Pô-lo em `screens/hero/variant_axes.rs` era uma
/// string de UI numa camada fora do alcance do gate — *uma regra fora do caminho de quem executa
/// não existe*. ⏳ Ele migra com os irmãos deste ficheiro quando o Fluent chegar.
const FLAT_AXIS_LABEL: &str = "Variant";

/// O título do cartão.
///
/// ⚠️ **Ele existe por causa do caso que motivou o cartão**: num objecto solto não há cartão de
/// instância acima, e duas linhas soltas a dizer `Size  Small` não dizem de onde vêm. *O artista
/// escreveu as chaves no NOME — o cartão tem de o ligar de volta ao que ele escreveu.*
const CARD_TITLE: &str = "Properties";

/// Pinta o cartão. Devolve o `y` de baixo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_properties_card(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorPropertiesInfo,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let font = TypeToken::Base.px();
    let small = TypeToken::Sm.px();
    let line = font + Spacing::Xs.px();
    // ⚠️ **A altura é MEDIDA antes de pintar** — o fundo vai primeiro, senão cobre o texto.
    // ⛔ E as fileiras contam-se pelo que a TABELA DE IDS endereça, não pelo que o modelo traz: o
    // pintor salta o que passa do teto, e uma altura que contasse o vector inteiro deixaria um vão
    // vazio no fim do cartão.
    let painted = info.rows.len().min(ids::MAX_INSTANCE_AXES);
    let rows = 1 + painted + usize::from(info.beyond > 0);
    let card_h = CARD_PAD * 2.0 + line * rows as f32;
    fill_rounded_rect(
        scene,
        Rect::new(x, y, w, card_h),
        Radius::Md.px(),
        resolve(ColorToken::Bg2, theme),
    );

    let tx = x + CARD_PAD;
    let tw = (w - CARD_PAD * 2.0).max(0.0);
    let mut ty = y + CARD_PAD;
    paint_text(
        text_system,
        scene,
        CARD_TITLE,
        tx,
        ty,
        small,
        tw,
        resolve(ColorToken::Text2, theme),
    );
    ty += line;

    for (a, ax) in info.rows.iter().enumerate() {
        let Some(row_ids) = ids::INSP_INSTANCE_AXIS_OPTION.get(a) else {
            break;
        };
        let label_w = (tw * AXIS_LABEL_FRACTION).min(AXIS_LABEL_MAX_PX);
        let axis_label = if ax.name.is_empty() {
            FLAT_AXIS_LABEL
        } else {
            ax.name.as_str()
        };
        paint_text(
            text_system,
            scene,
            axis_label,
            tx,
            ty + (line - small) * 0.5,
            small,
            label_w,
            resolve(ColorToken::Text2, theme),
        );
        let chips_x = tx + label_w;
        let chips_w = (tw - label_w).max(0.0);
        // ⛔ **UM valor é TEXTO, não um botão** — ver o cabeçalho deste ficheiro.
        if ax.options.len() < 2 {
            if let Some(only) = ax.options.first() {
                paint_text(
                    text_system,
                    scene,
                    &only.label,
                    chips_x,
                    ty + (line - font) * 0.5,
                    font,
                    chips_w,
                    resolve(ColorToken::Text1, theme),
                );
            }
            ty += line;
            continue;
        }
        let n = ax.options.len();
        let gap = Spacing::Xs.px();
        let cw = ((chips_w - gap * (n.saturating_sub(1)) as f32) / n as f32).max(0.0);
        for (i, v) in ax.options.iter().enumerate() {
            let Some(&id) = row_ids.get(i) else {
                break;
            };
            let host = Rect::new(chips_x + (cw + gap) * i as f32, ty, cw, line);
            hit_index.register(id, host);
            let button = Button::new(id, v.label.clone())
                // ⚠️ A vigente é `Accent` — é o **estado**, e não uma decoração: sem ela a fileira
                // mostra as opções e esconde a resposta.
                .kind(if v.current {
                    ButtonKind::Accent
                } else {
                    ButtonKind::Default
                })
                .visual(store.button_visual(id));
            paint_button(&button, host, scene, text_system, theme);
        }
        ty += line;
    }

    // ⚠️ **O que a tabela de ids não endereça é ESCRITO** — nunca truncado em silêncio. É a mesma
    // lei do `beyond` da fileira de variants do vetor: *um catálogo que some é um catálogo em que o
    // artista deixa de confiar.*
    if info.beyond > 0 {
        paint_text(
            text_system,
            scene,
            &format!("{} more not shown", info.beyond),
            tx,
            ty,
            small,
            tw,
            resolve(ColorToken::Text2, theme),
        );
    }
    y + card_h + SECTION_BOTTOM_PAD_PX
}
