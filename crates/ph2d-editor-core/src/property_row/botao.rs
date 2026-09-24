//! ⭐⭐⭐ **Onde fica um BOTÃO DE ACÇÃO numa lista de propriedades** — ver [`caixa_do_botao`].

use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::ROW_H_PX;

/// ⭐⭐⭐ **O RECT de um BOTÃO DE ACÇÃO numa lista de propriedades — na coluna do VALOR.**
///
/// ⛔⛔ **Report do dono, 2026-09-24, foto do cartão da junta com uma seta:** *«esses botões que
/// atravessam de lado a lado talvez fiquem melhor na coluna do lado direito»*. O `Swap A / B`, o
/// `Copy Properties` e o `Delete Joint` eram `Rect::new(x, y, w, h)` — a linha inteira —, e numa
/// secção em que TODO o resto arranca o valor num `x` só eles eram as únicas peças a atravessar a
/// coluna do nome. O censo desse dia achou a mesma forma em `27` botões do Inspector.
///
/// ⚠️ **A coluna pede-se como a linha de escolha a pede** (`property_row_columns`, sem nome
/// desejado e sem necessidade declarada): numa lista que já tem uma linha dessas o pedido é o
/// MESMO e não mexe na coluna do painel. ⚠️ **A altura é do chamador, de propósito:** os botões
/// desta casa medem `30` e os campos `22`, e se devem ser iguais é decisão do dono ainda aberta
/// (§9-octodecies.6 do handoff da `line/UIUX`) — esta porta decide ONDE, não QUÃO ALTO.
///
/// ⛔⛔ **Só vai para a coluna se o RÓTULO lá couber; senão atravessa a linha, como antes** — a
/// lei que a [`super::escolha::paint_choice_row`] já tem (ao lado se cabe, PALETA se não). Medido
/// antes de a escrever, com todos a ir para a coluna: à largura de fábrica `5` botões passavam a
/// sair cortados (`Reimport at current px/m`, `Draw Joint on Canvas`, …) e no degrau estreito
/// **`18`**, o `Swap A / B` e o `Delete Joint` da foto incluídos. *Um botão cortado lê-se pior do
/// que um nome cortado: o nome tem o controlo ao lado a explicá-lo, o botão É a explicação.*
///
/// ⚠️ **A pergunta «cabe?» é a do PINTOR** — a largura do rótulo na espessura em que o botão o
/// pinta, contra o [`crate::paint::label_budget`] da caixa —, e mede-se SEM passar pelo
/// `text_elide::coube`, que regista no censo das elisões: uma pergunta de disposição lida como
/// uma pintura poria no censo um rótulo que não foi pintado ali.
#[must_use]
pub fn caixa_do_botao(
    text_system: &mut TextSystem,
    x: f32,
    w: f32,
    y: f32,
    h: f32,
    rotulo: &str,
) -> Rect {
    let row = crate::widget::property_row_columns(x, w, y, ROW_H_PX);
    let largura = text_system.prefix_width_weighted(
        rotulo,
        crate::widget::Button::label_font_px(),
        ph2d_text::FontWeight::MEDIUM,
    );
    if largura <= crate::paint::label_budget(row.control.w) {
        Rect::new(row.control.x, y, row.control.w, h)
    } else {
        Rect::new(x, y, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::caixa_do_botao;
    use ph2d_text::TextSystem;
    use ph2d_tokens::ROW_H_PX;

    const ALTURA: f32 = 30.0; // LITERAL-PX-OK: a altura de um botao desta casa (a porta nao a decide)

    /// ⭐ **Um rótulo curto vai para a coluna do VALOR** — o `x` é o do controlo da linha, e a
    /// largura a dele; a altura é a que o chamador pediu.
    #[test]
    fn um_rotulo_curto_vai_para_a_coluna_do_valor() {
        let mut ts = TextSystem::without_system_fonts();
        let (x, w) = (10.0, 300.0);
        let coluna = crate::widget::property_row_columns(x, w, 0.0, ROW_H_PX).control;
        let r = caixa_do_botao(&mut ts, x, w, 40.0, ALTURA, "Swap");
        assert!(
            coluna.x > x + 1.0,
            "a fixtura não tem coluna de nome: {coluna:?}"
        );
        assert_eq!((r.x, r.w), (coluna.x, coluna.w));
        assert_eq!((r.y, r.h), (40.0, ALTURA));
    }

    /// ⛔ **Um rótulo que não cabe na coluna ATRAVESSA a linha** — cortá-lo seria pior do que o
    /// desalinhamento (ver o doc da porta, com o número).
    #[test]
    fn um_rotulo_que_nao_cabe_atravessa_a_linha() {
        let mut ts = TextSystem::without_system_fonts();
        let (x, w) = (10.0, 300.0);
        let longo = "Reimport at current px/m, with every option this button could ever need";
        let r = caixa_do_botao(&mut ts, x, w, 0.0, ALTURA, longo);
        assert_eq!((r.x, r.w), (x, w));
    }

    /// ⚠️ **A fronteira é a do PINTOR** — varrida a largura do painel em quartos de píxel, o
    /// botão vai para a coluna exactamente quando a largura do rótulo cabe no
    /// [`crate::paint::label_budget`] dela.
    ///
    /// ⛔ **A 1.ª redacção media UM rótulo («MMM…» no maior comprimento que cabia) e a mutação
    /// que tira `1 px` à fronteira SOBREVIVEU**: a folga daquele rótulo até ao orçamento era de
    /// vários píxeis, logo os dois lados davam a mesma resposta. *Uma fronteira mede-se onde a
    /// folga passa por zero* — e uma varredura fina é o que garante que ela passa.
    #[test]
    fn a_fronteira_e_o_orcamento_do_pintor() {
        let mut ts = TextSystem::without_system_fonts();
        let rotulo = "Swap A / B";
        let largura = ts.prefix_width_weighted(
            rotulo,
            crate::widget::Button::label_font_px(),
            ph2d_text::FontWeight::MEDIUM,
        );
        let (mut na_coluna, mut inteiro) = (0, 0);
        for k in 0..1600 {
            let w = 100.0 + k as f32 * 0.25;
            let coluna = crate::widget::property_row_columns(0.0, w, 0.0, ROW_H_PX).control;
            let cabe = largura <= crate::paint::label_budget(coluna.w);
            let r = caixa_do_botao(&mut ts, 0.0, w, 0.0, ALTURA, rotulo);
            assert_eq!(
                r.x == coluna.x && coluna.x > 0.0,
                cabe,
                "w = {w}: {r:?} contra {coluna:?}"
            );
            if cabe {
                na_coluna += 1;
            } else {
                inteiro += 1;
            }
        }
        // O CONTROLO: a varredura atravessa a fronteira (dos dois lados há casos).
        assert!(
            na_coluna > 0 && inteiro > 0,
            "a varredura não atravessou a fronteira"
        );
    }

    /// ⛔ **Perguntar «cabe?» NÃO é pintar** — o censo das elisões não pode ver este rótulo.
    #[test]
    fn a_pergunta_nao_entra_no_censo_das_elisoes() {
        let mut ts = TextSystem::without_system_fonts();
        let ((), medidos) = crate::text_elide::elisao::medindo(|| {
            let _ = caixa_do_botao(&mut ts, 0.0, 300.0, 0.0, ALTURA, "Swap");
            let _ = caixa_do_botao(&mut ts, 0.0, 300.0, 0.0, ALTURA, &"M".repeat(80));
        });
        assert!(medidos.is_empty(), "a porta registou: {medidos:?}");
    }
}
