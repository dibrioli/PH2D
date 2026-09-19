//! O **plano** da barra: a sequência de controles e onde cada um cai.
//!
//! Existe separado do paint por uma razão dura: a tira precisa saber **quantas
//! linhas** a barra vai ocupar *antes* de pintar a superfície (o painel cresce
//! para caber). Medir e pintar por caminhos diferentes é como as duas versões
//! divergem — então há UMA sequência (`items`) e UM algoritmo de quebra
//! (`plan`), e tanto a medida quanto a pintura consomem o mesmo resultado.
//!
//! **A barra QUEBRA, nunca esconde.** A versão anterior descartava em silêncio
//! todo item que não coubesse na linha ("a barra nunca transborda"), e o efeito
//! era pior que transbordar: num viewport de 1280px NOVE dos dezoito controles
//! sumiam — as ops de chave, o hold, o ciclo e o tween inteiro — sem scroll, sem
//! overflow, sem qualquer sinal de que existiam. E como o teste era por-item
//! (`x + w <= right`), uma caixa estreita ENTRAVA depois que um botão largo fora
//! descartado: a barra saía com buracos, fora de ordem. Um controle invisível é
//! um controle que não existe.

use crate::ids;
use crate::state::FlipStripSnapshot;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};

/// Largura de um botão de ícone (quadrado, altura da linha).
pub(crate) const ICON_W: f32 = 28.0; // LITERAL-PX-OK: strip toolbar icon slot
/// Largura de uma caixa numérica pequena da barra.
pub(crate) const NUM_W: f32 = 46.0; // LITERAL-PX-OK: strip toolbar number chip
/// Largura de um toggle de texto (Ghost / Auto / Additive).
pub(crate) const TOGGLE_W: f32 = 62.0; // LITERAL-PX-OK: strip toolbar toggle
/// ⭐⭐⭐ **A largura de um chip de escolha DERIVA da lista que ele oferece.**
///
/// ⛔⛔ Era o literal `84,0`, e desses **`46` não são texto** (dois recuos, o vão e o chevron) ⇒ o
/// rótulo ficava com `38`. Medido em 2026-09-18: `No Cycle` mede `56,4` e saía **`No…`**; e o pior
/// nem era visível ao censo, que mede a opção ESCOLHIDA — `Ping Pong` (`62,9`) e `Ease In Out`
/// (`70,4`) vivem nas mesmas listas e nunca tinham sido medidos por ninguém.
///
/// ⚠️ **A régua é a LISTA, nunca o item em mãos** — é a mesma lei que a coluna dos nomes do Audio
/// Mixer passou a seguir em 18/09, e a conta do invólucro sai da porta que o PINTOR usa
/// ([`ph2d_editor_core::widget::dropdown_chip_width_for`]), nunca de uma segunda cópia aqui.
pub(crate) fn chip_w(text_system: &mut TextSystem, row_h: f32, nomes: &[TextKey]) -> f32 {
    chip_w_em(ph2d_i18n::idioma(), text_system, row_h, nomes)
}

/// ⭐⭐ **A mesma, com o IDIOMA dado** — a porta pela qual um gate mede a tensão de uma tradução
/// sem mexer no ambiente do processo.
///
/// ⚠️ Sem ela um gate compararia a palavra DEFORMADA com um chip medido em INGLÊS, e acusaria o
/// produto certo: foi assim que a 1.ª redacção do gate da coluna do Audio Mixer nasceu vácua.
pub(crate) fn chip_w_em(
    idioma: ph2d_i18n::Idioma,
    text_system: &mut TextSystem,
    row_h: f32,
    nomes: &[TextKey],
) -> f32 {
    let fonte = TypeToken::Base.px();
    let maior = nomes
        .iter()
        .map(|k| text_system.prefix_width(ph2d_i18n::tr_em(idioma, k.key()), fonte))
        .fold(0.0_f32, f32::max);
    ph2d_editor_core::widget::dropdown_chip_width_for(maior, row_h)
}
/// Largura média de um glifo em fração do tamanho da fonte — só para RESERVAR o
/// espaço de um rótulo curto da barra (não é métrica de design; a medida real do
/// texto sai do shaper, que aqui seria caro por um "FPS").
const GLYPH_W_RATIO: f32 = 0.62; // LITERAL-PX-OK: estimativa de largura de glifo

/// Um controle da barra (ou um respiro entre grupos).
pub(crate) enum Item {
    Icon(NodeId, IconId),
    Toggle(NodeId, &'static str, bool),
    Number(NodeId, f64),
    Label(&'static str),
    Cycle(u8),
    /// O chip de easing do tween (dropdown de 4 presets).
    Ease(u8),
    /// Respiro entre grupos — some no início de uma linha nova.
    Gap,
}

impl Item {
    /// A largura que o item reserva. `None` = não é um controle (respiro).
    fn width(&self, text_system: &mut TextSystem, row_h: f32) -> Option<f32> {
        Some(match self {
            Item::Icon(..) => ICON_W,
            Item::Toggle(..) => TOGGLE_W,
            Item::Number(..) => NUM_W,
            Item::Label(t) => {
                TypeToken::Sm.px() * GLYPH_W_RATIO * t.len() as f32 + Spacing::Xs.px()
            }
            Item::Cycle(_) => chip_w(text_system, row_h, &crate::paint_toolbar::CYCLE_NAMES),
            Item::Ease(_) => chip_w(text_system, row_h, &crate::paint_toolbar::TWEEN_EASE_NAMES),
            Item::Gap => return None,
        })
    }
}

/// **A sequência canônica da barra** — transporte · Ghost · autoria · ops de
/// chave · tween · ciclo. É a única lista; medir e pintar leem esta.
pub(crate) fn items(snap: &FlipStripSnapshot) -> Vec<Item> {
    // A exposição da chave ATIVA (o que a caixa "Hold" edita).
    let hold = snap
        .current_key
        .and_then(|k| snap.cells.iter().find(|c| c.key == k))
        .map_or(1, |c| c.exposure);
    let play_icon = if snap.playing {
        IconId::Pause
    } else {
        IconId::Play
    };
    vec![
        // Transporte: ◀ desenho · play/pause · desenho ▶
        Item::Icon(ids::FLIP_PREV_DRAWING, IconId::SkipBack),
        Item::Icon(ids::FLIP_PLAY, play_icon),
        Item::Icon(ids::FLIP_NEXT_DRAWING, IconId::SkipForward),
        Item::Label(tr("panel.flip_frames.toolbar.fps")),
        Item::Number(ids::FLIP_FPS_NUM, f64::from(snap.fps)),
        Item::Gap,
        // Ghost Frames: liga/desliga + quantos antes/depois
        Item::Toggle(
            ids::FLIP_GHOST,
            tr("panel.flip_frames.toolbar.ghost"),
            snap.ghost,
        ),
        Item::Number(ids::FLIP_GHOST_BEFORE_NUM, f64::from(snap.ghost_before)),
        Item::Number(ids::FLIP_GHOST_AFTER_NUM, f64::from(snap.ghost_after)),
        // **Pin** (light table, T3.9): fixa a chave ATUAL como referência — ela vira
        // fantasma além dos vizinhos, em qualquer modo e fora do alcance. Mora no grupo
        // do Ghost porque é a mesma pergunta ("o que mais eu vejo enquanto desenho?"),
        // e não uma op de chave: ele não cria, não apaga e não move nada.
        Item::Toggle(
            ids::FLIP_KEY_PIN,
            tr("panel.flip_frames.toolbar.pin"),
            snap.current_pinned,
        ),
        Item::Gap,
        // Autoria: o que nasce ao desenhar depois do hold
        Item::Toggle(
            ids::FLIP_AUTOKEY,
            tr("panel.flip_frames.toolbar.auto"),
            snap.autokey,
        ),
        // **Falloff** (W7 — multiframe): com ele ligado, os quadros vizinhos marcados
        // recebem menos influência do pincel que o quadro ativo. Vive ao lado do Auto
        // porque os dois são política de AUTORIA (o que o gesto faz nos outros quadros),
        // e não parâmetro de pincel.
        Item::Toggle(
            ids::FLIP_FALLOFF,
            tr("panel.flip_frames.toolbar.falloff"),
            snap.falloff,
        ),
        Item::Toggle(
            ids::FLIP_ADDITIVE,
            tr("panel.flip_frames.toolbar.additive"),
            snap.additive,
        ),
        Item::Gap,
        // Ops de chave: + · duplicar · **instanciar** · apagar · exposição · mover ±1
        Item::Icon(ids::FLIP_KEY_ADD, IconId::Plus),
        Item::Icon(ids::FLIP_KEY_DUP, IconId::Copy),
        // **Duplicate as instance** (o *linked duplicate* do Blender): a chave nova aponta
        // para o MESMO desenho — editar uma edita as duas. É o elo, não a cópia: daí o
        // ícone de corrente ao lado do de cópia, e o pontinho que a célula ganha.
        Item::Icon(ids::FLIP_KEY_INSTANCE, IconId::Link),
        // **Unlink** — quebra o vínculo desta chave com a arte compartilhada. Só faz
        // sentido ao lado do Link: um desfaz o que o outro faz.
        Item::Icon(ids::FLIP_KEY_UNLINK, IconId::Unlink),
        Item::Icon(ids::FLIP_KEY_DELETE, IconId::Trash),
        Item::Label(tr("panel.flip_frames.toolbar.hold")),
        Item::Number(ids::FLIP_HOLD_NUM, f64::from(hold)),
        Item::Icon(ids::FLIP_KEY_LEFT, IconId::ChevronLeft),
        Item::Icon(ids::FLIP_KEY_RIGHT, IconId::ChevronRight),
        Item::Gap,
        // Tween: quantos inbetweens + gerar
        Item::Label(tr("panel.flip_frames.toolbar.tween")),
        Item::Number(ids::FLIP_TWEEN_NUM, f64::from(snap.tween_count)),
        // O TEMPO dos inbetweens (o motor sempre soube; a barra é que não oferecia) e
        // o que fazer com os traços que existem em só uma das chaves.
        Item::Ease(snap.tween_ease),
        Item::Toggle(
            ids::FLIP_TWEEN_FADE,
            tr("panel.flip_frames.toolbar.fade"),
            snap.tween_fade,
        ),
        // **Pairs** — abre o overlay de correção de correspondência (a lição CACAni: o
        // matcher erra, o artista corrige). Aceso enquanto a sessão está aberta.
        Item::Toggle(
            ids::FLIP_TWEEN_PAIRS,
            tr("panel.flip_frames.toolbar.pairs"),
            snap.tween_pairs,
        ),
        Item::Toggle(
            ids::FLIP_TWEEN_ADD,
            tr("panel.flip_frames.toolbar.add"),
            false,
        ),
        Item::Gap,
        // Ciclo (post behavior da camada ativa)
        Item::Cycle(snap.cycle),
    ]
}

/// Onde cada item cai, dado o rect da PRIMEIRA linha. Quebra para a linha
/// seguinte quando o item não cabe — **nunca descarta**. Devolve também quantas
/// linhas a barra ocupou (≥ 1), que é o que faz a tira crescer.
///
/// Um item mais largo que a linha inteira transborda em vez de sumir: melhor um
/// controle cortado (que o usuário vê e pode alcançar redimensionando) do que um
/// controle ausente (que ele conclui que não existe).
pub(crate) fn plan(
    items: &[Item],
    first_row: Rect,
    row_h: f32,
    text_system: &mut TextSystem,
) -> (Vec<Rect>, u32) {
    let left = first_row.x;
    let right = first_row.x + first_row.w;
    // ⭐⭐ **Pelas portas do RITMO** (2026-09-18). O gate `every_stack_of_rows_asks_the_rhythm`
    //    acusou este ficheiro assim que ele passou a MEDIR texto e a nomear o `ROW_H_PX` — e ele
    //    tinha razão desde sempre: a barra empilha LINHAS, e a altura chegava por argumento, logo
    //    o censo dela não a via. *Um `Spacing::Xs` escrito à mão passa em todo gate desta casa e
    //    continua fora do ritmo.*
    let gap = ph2d_tokens::control_gap_px();
    let row_gap = ph2d_tokens::control_gap_px();

    let mut out = Vec::with_capacity(items.len());
    let mut x = left;
    let mut y = first_row.y;
    let mut rows = 1u32;

    for item in items {
        let Some(w) = item.width(text_system, row_h) else {
            // Respiro: só separa grupos DENTRO de uma linha; no começo de uma
            // linha nova ele não deve empurrar o primeiro controle para dentro.
            if x > left {
                x += Spacing::Md.px();
            }
            out.push(Rect::new(x, y, 0.0, 0.0));
            continue;
        };
        // Quebra: não cabe e a linha já tem algo → próxima linha.
        if x > left && x + w > right {
            x = left;
            y += row_h + row_gap;
            rows += 1;
        }
        out.push(Rect::new(x, y, w, row_h));
        x += w + gap;
    }
    (out, rows)
}

/// Quantas linhas a barra ocupa nesta largura — o que a tira precisa saber
/// ANTES de pintar a superfície (ela cresce para caber a barra inteira).
pub(crate) fn rows(
    inner: Rect,
    row_h: f32,
    snap: &FlipStripSnapshot,
    text_system: &mut TextSystem,
) -> u32 {
    plan(&items(snap), inner, row_h, text_system).1
}

/// A altura total de uma barra de `rows` linhas.
pub(crate) fn bar_height(rows: u32, row_h: f32) -> f32 {
    let n = rows.max(1) as f32;
    n * row_h + (n - 1.0) * Spacing::Xs.px()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> FlipStripSnapshot {
        FlipStripSnapshot {
            has_layer: true,
            fps: 12.0,
            ..Default::default()
        }
    }

    /// **O bug que motivou o módulo:** nenhum controle pode sumir, em nenhuma
    /// largura plausível. Antes, um viewport de 1280 comia nove deles.
    #[test]
    fn no_control_is_ever_dropped_at_any_width() {
        let s = snap();
        let mut ts0 = ph2d_text::TextSystem::without_system_fonts();
        let n_controls = items(&s)
            .iter()
            .filter(|i| i.width(&mut ts0, 28.0).is_some())
            .count();
        for w in [
            320.0f32, 640.0, 800.0, 1024.0, 1280.0, 1440.0, 1920.0, 3440.0,
        ] {
            let mut ts = ph2d_text::TextSystem::without_system_fonts();
            let (rects, rows) = plan(&items(&s), Rect::new(0.0, 0.0, w, 28.0), 28.0, &mut ts);
            let placed = rects.iter().filter(|r| r.w > 0.0 && r.h > 0.0).count();
            assert_eq!(
                placed,
                n_controls,
                "largura {w}: {} controles sumiram (a barra escondeu em vez de quebrar)",
                n_controls - placed
            );
            assert!(rows >= 1, "largura {w}: zero linhas");
        }
    }

    /// A barra larga cabe numa linha só (não se paga altura à toa).
    #[test]
    fn a_wide_bar_stays_on_one_row() {
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let (_, rows) = plan(
            &items(&snap()),
            Rect::new(0.0, 0.0, 2000.0, 28.0),
            28.0,
            &mut ts,
        );
        assert_eq!(rows, 1);
    }

    /// Estreitar exige mais linhas — e a contagem é monótona (mais largura nunca
    /// pede mais linhas).
    ///
    /// Nota de calibração: a largura aqui é a da BARRA, não a da janela — a tira
    /// desconta o rail, o dock e o padding, então uma janela de 1280px dá uma
    /// barra de ~800. É por isso que o corte real aparece cedo.
    #[test]
    fn narrower_needs_more_rows_monotonically() {
        let s = snap();
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let mut row_at = |w: f32| plan(&items(&s), Rect::new(0.0, 0.0, w, 28.0), 28.0, &mut ts).1;
        assert!(row_at(800.0) > 1, "uma barra de 800px tem de quebrar");
        for pair in [(400.0, 640.0), (640.0, 900.0), (900.0, 1400.0)] {
            assert!(
                row_at(pair.0) >= row_at(pair.1),
                "mais estreito ({}) pediu MENOS linhas que {}",
                pair.0,
                pair.1
            );
        }
    }

    /// Nenhum controle é pintado FORA da faixa horizontal (a não ser que ele
    /// sozinho seja mais largo que a linha — aí transborda, mas existe).
    #[test]
    fn controls_stay_inside_the_row_when_they_fit() {
        let s = snap();
        let w = 1280.0;
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let (rects, _) = plan(&items(&s), Rect::new(0.0, 0.0, w, 28.0), 28.0, &mut ts);
        for r in rects.iter().filter(|r| r.w > 0.0) {
            assert!(
                r.x >= 0.0 && r.x + r.w <= w + 0.01,
                "controle fora da faixa: {r:?}"
            );
        }
    }

    /// ⭐⭐⭐ **TODA OPÇÃO das duas listas cabe no chip que o plano reserva — nas DUAS línguas.**
    ///
    /// ⛔⛔ Antes de 2026-09-18 o chip era o literal `84 px` e o rótulo ficava com `38`: `No Cycle`
    /// (`56,4`) saía **`No…`**, e a varredura das elisões só via ESSE, porque ela mede o que está
    /// PINTADO — a opção escolhida. ⚠️ `Ping-Pong` (`65,5`) e `Ease In-Out` (`72,9`) vivem nas
    /// mesmas listas e **nunca tinham sido medidos por ninguém**.
    ///
    /// ⇒ *quem dimensiona um chip de escolha mede a LISTA, nunca o item em mãos* — e o orçamento
    /// vem da porta que o PINTOR usa, senão o gate e a cura partilham a suposição.
    #[test]
    fn toda_opcao_das_listas_cabe_no_chip_que_o_plano_reserva() {
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let fonte = TypeToken::Base.px();
        let row_h = ph2d_tokens::ROW_H_PX;
        let mut apertados = Vec::new();
        for idioma in [ph2d_i18n::Idioma::Ingles, ph2d_i18n::Idioma::Teste] {
            for nomes in [
                &crate::paint_toolbar::CYCLE_NAMES[..],
                &crate::paint_toolbar::TWEEN_EASE_NAMES[..],
            ] {
                // ⚠️ A largura é a DESSA língua — comparar o texto deformado com um chip medido em
                //    inglês acusaria o produto certo (a 1.ª redacção do gate do mixer).
                let w = chip_w_em(idioma, &mut ts, row_h, nomes);
                let orcamento = ph2d_editor_core::widget::dropdown_label_budget(
                    ph2d_editor_core::zones::Rect::new(0.0, 0.0, w, row_h),
                );
                for k in nomes {
                    let texto = ph2d_i18n::tr_em(idioma, k.key());
                    let largura = ts.prefix_width(texto, fonte);
                    if largura > orcamento {
                        apertados.push(format!(
                            "{idioma:?} {texto:?} mede {largura:.1} num orçamento de {orcamento:.1}"
                        ));
                    }
                }
            }
        }
        assert!(
            apertados.is_empty(),
            "estas opções saem cortadas no chip delas:\n  {}",
            apertados.join("\n  ")
        );
    }

    /// ⛔ **E o CONTROLO: o literal de antes cortava, e não por pouco.**
    ///
    /// Sem esta metade, um chip que por acaso ficasse largo passaria o gate acima sem que a
    /// MEDIÇÃO da lista tivesse alguma coisa a ver com isso.
    #[test]
    fn o_chip_literal_de_84px_cortava_metade_das_opcoes() {
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let fonte = TypeToken::Base.px();
        let row_h = ph2d_tokens::ROW_H_PX;
        let orcamento = ph2d_editor_core::widget::dropdown_label_budget(
            ph2d_editor_core::zones::Rect::new(0.0, 0.0, 84.0, row_h),
        );
        let cortadas: Vec<&str> = crate::paint_toolbar::CYCLE_NAMES
            .iter()
            .chain(crate::paint_toolbar::TWEEN_EASE_NAMES.iter())
            .map(|k| k.tr())
            .filter(|t| ts.prefix_width(t, fonte) > orcamento)
            .collect();
        assert_eq!(
            cortadas,
            vec![
                "No Cycle",
                "Ping-Pong",
                "Linear",
                "Ease In",
                "Ease Out",
                "Ease In-Out"
            ],
            "a fixtura desta wave é o chip literal de 84 px a cortar SEIS das oito opções — se ela \
             deixar de as cortar, os rótulos mudaram e a medição do cabeçalho tem de ser refeita"
        );
    }
}
