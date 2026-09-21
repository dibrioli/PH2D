//! ⭐⭐⭐ **A COR DO PINCEL É UMA CAIXA** — a amostra, o selector que ela abre, e
//! a ponte entre o `f32` do motor e o `u8` que o artista escolhe.
//!
//! Ordem do dono (2026-09-20, logo depois do smoke da pintura: *«Smoke OK.
//! Agora troque os sliders de cor pelo seletor de Cor (caixa de cor)»*).
//!
//! # ⭐ Porque esta wave não constrói selector nenhum
//!
//! O selector de cor desta casa é **UM** (o OKLCH com roda, canais RGB/HSV,
//! hexadecimal, paletas e conta-gotas), ele já flutua sobre o canvas, e um
//! painel entra nele por **duas linhas**: registar o id como amostra
//! ([`ph2d_editor_core::interaction::WidgetStore::register_picker_swatch`]) e
//! manter a `widget_color` dela em dia. O resto é o `Down` genérico do
//! `pointer_down`, que declara por escrito ter sido **generalizado** de um caso
//! particular do Painter para *«any panel that paints a `ColorSwatch`»*.
//!
//! ⛔ *Um segundo selector neste painel seria a segunda resposta à mesma
//! pergunta, e a que envelhece.* O molde é o do painel do modelador (19/09), que
//! fechou a MESMA ordem noutro painel — e é por isso que esta wave é
//! **composição**, com zero lei nova.
//!
//! # ⚠️ Irmão dos [`super::brush_fileiras`], e o corte é por ESPÉCIE
//!
//! Lá moram as fileiras de CHIPS próprias de cada pincel; aqui mora o único
//! controlo do painel que fala com uma janela flutuante — ele tem um ciclo de
//! vida próprio (abre, é lido, fecha quando o sujeito some) que nenhum chip tem.
//! ⛔ Corte, nunca uma entrada no `FILE_OVERAGE_OK`.

use ph2d_editor_core::paint::{paint_text_elided, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    ColorSwatch, NUMBER_INPUT_MIN_W_PX, SwatchSize, SwatchState, paint_color_swatch,
    property_label_col_w, slider_with_chip_chip_rect, slider_with_chip_label_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, TypeToken};

use crate::state::Sculpt3dSnapshot;
use crate::state_intent::Sculpt3dIntent;

/// ⭐ **`f32` do motor → `u8` do artista.**
///
/// ⚠️ **Sem gama, e isso é a convenção MEDIDA desta casa**, não uma escolha: as
/// duas cópias que o Painter já tem (`encode_rgb` e `encode_rgb3`) fazem
/// exactamente `clamp(0,1) × 255 + 0,5`, e o canal por-vértice que esta cor
/// alimenta entra no shader como **albedo** (`CLAY * vcolor`), no mesmo espaço
/// em que a amostra é pintada. *Uma conversão com gama aqui faria a caixa
/// mostrar uma cor e o barro sair noutra.*
///
/// ⚠️ **É a TERCEIRA cópia desta aritmética no repo e ela é declarada:** as
/// outras duas vivem em `ph2d-panel-painter-layers`, uma crate que esta não
/// conhece nem deve conhecer (o precedente é o `vetor.rs` da `ph2d-boundary`).
/// O que a torna honesta é o gate de ida-e-volta ao lado dela.
#[must_use]
pub(crate) fn para_u8(c: [f32; 3]) -> [u8; 3] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8, // LITERAL-PX-OK: normalizacao sRGB 8 bits
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8, // LITERAL-PX-OK: normalizacao sRGB 8 bits
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8, // LITERAL-PX-OK: normalizacao sRGB 8 bits
    ]
}

/// ⭐ **`u8` do artista → `f32` do motor**, a volta da [`para_u8`].
///
/// ⚠️⚠️ **A volta NÃO é a identidade, e a consequência está MEDIDA e
/// DECLARADA:** a cor de fábrica é `[1,0 · 0,766 · 0,336]` (o `_color` do
/// `Paint.js:13`) e o ciclo devolve `[1,0 · 0,764706 · 0,337255]` — a partir da
/// primeira cor que o artista escolher, o canal passa a ser **quantizado a 8
/// bits**, com erro máximo de `1/510 ≈ 0,00196`.
///
/// ⭐ **E isso é meio byte no ecrã, que tem 8 bits**: invisível por construção.
/// A troca AUMENTA o que o artista alcança — arrastar um chip de passo `0,05`
/// dava **21** valores por canal, a amostra dá **256** — e passa de três gestos
/// a um.
///
/// ⚠️ **O valor de FÁBRICA sobrevive ao bit**, e não por sorte: com o selector
/// fechado a amostra **semeia** e nunca escreve, e com ele aberto a escrita só
/// acontece quando o `u8` de facto mudou (ver [`paint_cor_do_pincel`]).
#[must_use]
pub(crate) fn para_f32(c: [u8; 3]) -> [f32; 3] {
    [
        f32::from(c[0]) / 255.0, // LITERAL-PX-OK: normalizacao sRGB 8 bits
        f32::from(c[1]) / 255.0, // LITERAL-PX-OK: normalizacao sRGB 8 bits
        f32::from(c[2]) / 255.0, // LITERAL-PX-OK: normalizacao sRGB 8 bits
    ]
}

/// ⭐⭐⭐ **A LINHA DA COR** — o rótulo à esquerda, a caixa na goteira do valor.
///
/// # ⚠️⚠️ QUEM MANDA NA COR MUDA CONFORME O SELECTOR ESTÁ ABERTO
///
/// | o selector está… | quem é a verdade | o que esta função faz |
/// |---|---|---|
/// | **fechado** | o pincel | semeia a amostra com a cor dele, todo quadro |
/// | **aberto nesta amostra** | o selector | lê o que ele escreveu e publica um `SetUi` |
///
/// ⛔ **Semear nos dois casos apagaria a escolha debaixo do dedo:** o `hero`
/// espelha o valor vivo do selector para `widget_color(id)` **antes** de os
/// painéis pintarem, e um `set_widget_color` aqui por cima devolveria a cor
/// velha a cada quadro — a roda mover-se-ia e o barro não.
///
/// ⚠️ **E o «mudou?» pergunta-se em `u8`.** Sem essa comparação esta função
/// publicaria um `SetUi` **por quadro** enquanto o selector estivesse aberto:
/// um passo de undo por quadro sobre uma cor que ninguém mexeu.
///
/// ⚠️ **As três metades do registo saem juntas, e cada uma sozinha é um defeito
/// diferente:** sem o `register_picker_swatch` o `Down` não a reconhece e o
/// clique cai no foco; sem o `hit_index` ela não é clicável de todo; sem o
/// anel de foco ela não diz **qual** amostra o selector está a editar.
pub(super) fn paint_cor_do_pincel(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.brush.verb.deposita_a_cor_do_pincel() {
        return y;
    }
    let id = crate::ids::SCULPT3D_COLOR_SWATCH;
    let theme = ctx.host.theme();
    let (nome, goteira) = colunas(x, w, y);

    let font = TypeToken::Sm.px();
    // ⚠️⚠️ **A chave é um LITERAL nas duas chamadas de propósito, e não uma
    // `const`.** O censo que confere o roteiro da cena `=51` colhe os rótulos
    // que este painel pinta varrendo `tr("…")` nos pintores — foi ele que
    // apanhou, no dia em que nasceu, um passo a mandar o dono clicar numa
    // fileira `Light` que o painel chama `Material`. Um `tr(CHAVE)` é
    // **invisível** a essa varredura, e *um censo que não vê uma população
    // lê-se, num relatório, como um censo que a aprovou*.
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        tr("panel.sculpt3d.color"),
        nome.x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        nome.w,
        resolve(ColorToken::Text2, theme),
    );

    let rgb = para_u8(snap.ui.brush.color);
    let aberto = {
        let store = ctx.host.store_mut();
        // ⚠️⚠️ **O LITERAL repete-se aqui de propósito, e não é descuido.** Este
        // registo é o que torna a amostra viva, e ele é de uma espécie que
        // nenhum dos três censos do [`crate::populate`] conhecia: ela **não**
        // entra no `WidgetStore` como widget (o braço do `pointer_down` que a
        // serve devolve antes de o foco ser calculado). ⇒ o censo tem de a poder
        // LER, e a forma que ele lê é `register_picker_swatch(crate::ids::NOME)`
        // — exactamente como lê `&crate::ids::NOME[..]` no registo das fileiras.
        // *Um `id` aqui deixaria o censo cego, e um censo cego sobre uma espécie
        // nova é a licença que aquela secção existe para não ter.*
        store.register_picker_swatch(crate::ids::SCULPT3D_COLOR_SWATCH);
        let aberto = store.picker_target() == Some(id);
        if aberto {
            if let Some(escolhida) = store.widget_color(id) {
                let nova = [escolhida[0], escolhida[1], escolhida[2]];
                if nova != rgb {
                    // ⚠️ **Um `clone` e não uma cópia:** o `Sculpt3dUi` carrega o
                    // estado autorado inteiro (o `Brush` tem um `Arc<AlphaImage>`
                    // dentro), logo ele é `Clone` e não `Copy`. O custo paga-se
                    // **uma vez por cor NOVA** — a guarda do `u8` acima é o que
                    // impede isto de correr por quadro.
                    let mut ui = snap.ui.clone();
                    ui.brush.color = para_f32(nova);
                    crate::state::push_intent(Sculpt3dIntent::SetUi(ui));
                }
            }
        } else {
            // ⚠️ **Opaca**: a cor do pincel não tem alfa, e um `255` inventado
            // aqui é o único honesto — com outro valor a amostra pintaria o
            // xadrez da transparência sobre uma tinta que é sólida.
            store.set_widget_color(id, [rgb[0], rgb[1], rgb[2], 255]);
        }
        aberto
    };

    let swatch = ColorSwatch::new(
        id,
        tr("panel.sculpt3d.color"),
        [rgb[0], rgb[1], rgb[2], 255],
    )
    .size(SwatchSize::Md)
    // ⭐ **Aberto ⇒ focado**, que é o anel que a família moderna traça: com o
    // selector a flutuar sobre o canvas, é este anel que diz **qual** amostra
    // ele está a editar.
    .state(if aberto {
        SwatchState::Focused
    } else {
        SwatchState::Normal
    });
    paint_color_swatch(&swatch, goteira, ctx.scene, theme);
    ctx.host.hit_index_mut().register(id, goteira);

    // ⚠️ **A altura é DERIVADA do rect e não `ROW_H_PX`**: num painel estreito o
    // [`slider_with_chip_chip_rect`] empilha o controlo numa segunda linha, e um
    // número fixo aqui faria a amostra sobrepor-se ao que vem depois exactamente
    // na largura em que o dono trabalha.
    let usado = (goteira.y + goteira.h - y).max(ROW_H_PX);
    y + usado + ph2d_tokens::control_gap_px()
}

/// ⭐⭐⭐ **A repartição da linha — os MESMOS dois números que o
/// [`crate::paint::paint_row`] passa ao pintor das pistas irmãs.**
///
/// ⛔ É isso, e não uma constante partilhada, que faz a amostra aterrar na
/// coluna exacta em que as pistas do raio e da força aterram, nas **duas**
/// aparências (a clássica empilha; a de omissão usa a caixa única). *Uma
/// segunda receita de repartição aqui seria a terceira coluna que o painel do
/// modelador pagou com um report de «widgets embolados».*
fn colunas(x: f32, w: f32, y: f32) -> (Rect, Rect) {
    let linha = Rect::new(x, y, w, ROW_H_PX);
    let label_w = property_label_col_w(x, w);
    (
        slider_with_chip_label_rect(linha, label_w, NUMBER_INPUT_MIN_W_PX),
        slider_with_chip_chip_rect(linha, label_w, NUMBER_INPUT_MIN_W_PX),
    )
}

/// ⭐⭐⭐ **O SELECTOR SEGUE O SUJEITO** — se a amostra deixa de ser pintada, o
/// selector aberto sobre ela **fecha**.
///
/// ⛔⛔ Sem isto, trocar do pincel de pintura para outro (ou fechar o painel)
/// deixaria a janela do selector a flutuar sobre o canvas a editar uma amostra
/// que ninguém desenha — e a cor que ela escrevesse iria para um pincel que o
/// artista já não tem na mão. *É o mesmo controlo órfão que o painel do
/// modelador fechou em 19/09, e a lei chega aqui com ele.*
///
/// ⚠️ **A pergunta é feita à MESMA porta que decide pintá-la**
/// ([`ph2d_sculpt3d::Verb::deposita_a_cor_do_pincel`]) e não a uma lista de
/// verbos ao lado: duas respostas divergiriam no dia do terceiro verbo que
/// deposita cor.
pub(crate) fn fecha_um_selector_orfao(ctx: &mut PaintCtx, pintada: bool) {
    if pintada {
        return;
    }
    if ctx.host.store().picker_target() == Some(crate::ids::SCULPT3D_COLOR_SWATCH) {
        ctx.host.store_mut().set_picker_target(None);
    }
}

#[cfg(test)]
#[path = "brush_cor_tests.rs"]
mod brush_cor_tests;

/// ⭐⭐⭐⭐ **A RESOLUÇÃO DA TINTA** — os chips `Mesh` · `2×` · `4×` · `8×`,
/// logo abaixo da caixa de cor.
///
/// # ⚠️ Porque ela mora AQUI e não na secção `Topology`
///
/// São **duas perguntas com a mesma palavra**, e a fronteira é a que os dois
/// sliders `Detail` do `Density` pagaram em 2026-09-14: *quão fina a MALHA fica
/// debaixo de um traço* é a da topologia, *quão fina a TINTA é nesta peça* é
/// esta. Pô-la lá poria o artista a procurar a resolução da cor numa secção que
/// fala de geometria.
///
/// ⭐ **E o sítio é COLADO à cor**, que é a mesma lei do bloco de cima: um
/// controlo separado do irmão por controlos de outro assunto lê-se como sendo
/// de outro assunto.
///
/// ⚠️ **A POPULAÇÃO é mais larga que a da caixa de cor, de propósito:** a caixa
/// só serve quem DEPOSITA a cor do pincel, e o plano serve quem **escreve no
/// canal** — os dois do anel (borrar e esfregar) puxam a cor da vizinhança e
/// escrevem-na com a mesma resolução. *Herdar a lente da caixa deixaria dois
/// pincéis de cor sem a pergunta.*
pub(super) fn paint_detalhe_da_tinta(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.brush.verb.paints_color() {
        return y;
    }
    let sel = crate::state::DetalheDaTinta::ALL
        .iter()
        .position(|&d| d == snap.ui.tinta_detalhe)
        .unwrap_or(0);
    let labels: Vec<&str> = crate::state::DetalheDaTinta::ALL
        .iter()
        .map(|d| d.label())
        .collect();
    super::widgets::labelled_seg(
        ctx,
        tr("panel.sculpt3d.tinta_detalhe"),
        &crate::ids::SCULPT3D_TINTA_DETALHE,
        &labels,
        sel,
        x,
        w,
        y,
    )
}
