//! ⭐⭐⭐ **A CÁPSULA** — o desenho de um nó quando o zoom o encolhe abaixo do limiar do texto.
//!
//! Ordem do dono (2026-09-19): *«se depois disso o zoom continuar a reduzir os nós, o desenho
//! tradicional dos nós se modifica para uma simples cápsula da cor característica do grupo a que o
//! nó pertence, com o nome do nó ocupando toda a cápsula, os parâmetros de ajustes são escondidos
//! e os slots de conexão ficam maiores»* — e, logo a seguir: *«os nomes dos nós ficam bem maiores
//! nas cápsulas»*.
//!
//! ⚠️ **Irmão do [`super::paint_card`] por RESPONSABILIDADE e não por tamanho:** aquele desenha o
//! cartão que o artista LÊ de perto (cabeçalho, sockets por fileira, faixa de params, readout);
//! este desenha o que ele RECONHECE de longe. As duas mudam por razões diferentes — uma quando um
//! controlo ganha um gesto, a outra quando muda o que se lê num grafo afastado.

use super::*;

/// **Quanto da altura da cápsula o nome ocupa.** ⚠️ Não é gosto: a `0,62` a altura de maiúscula
/// (~`0,7 em`) enche `43 %` da pastilha e sobra uma margem de ~`19 %` de cada lado — abaixo disso
/// o nome flutua numa pastilha meia vazia, e acima dele os ascendentes/descendentes tocam a borda
/// arredondada. ⭐ E ela é o que faz o pedido *«bem maiores»* ser **medido**: no limiar da cápsula
/// (`zoom 0,655`) o nome sai a `26 × 0,655 × 0,62 = 10,6 px` contra os `13 × 0,655 = 8,5 px` que
/// o título do cartão daria — **`1,24×`** —, e a vantagem CRESCE ao afastar, porque a cápsula
/// pára de encolher os pinos e o cartão nunca parava de encolher o título.
const NOME_DA_CAPSULA: f32 = 0.62; // LITERAL-PX-OK: fracção da altura da cápsula

/// **A margem lateral do nome**, em fracção da altura — o mesmo respiro do topo e da base, para a
/// pastilha ler-se como uma pastilha e não como um rectângulo com as pontas limadas.
const MARGEM_X: f32 = 0.5; // LITERAL-PX-OK: fracção da altura da cápsula

/// Desenha a cápsula e devolve o rect do corpo (o que o hit-test regista).
///
/// ⭐ **Os sockets PARAM de encolher** — é isso o *«os slots de conexão ficam maiores»*, e o
/// número é DERIVADO e não escolhido: eles são desenhados com o raio que tinham **no limiar da
/// cápsula** (`SOCKET_R × max(zoom, limiar)`), logo a `zoom 0,3` um pino sai `2,2×` maior do que
/// sairia. ⚠️ *A alternativa — um multiplicador — daria um pino que volta a encolher ao afastar
/// mais, e o alvo de clique (`SOCKET_HIT_R`) é fixo em píxeis de ECRÃ desde sempre: o desenho
/// passa a dizer a verdade sobre o alvo.*
pub(super) fn draw_capsula(
    ctx: &mut PaintCtx,
    state: &MotionGraphPanelState,
    n: &GraphNodeView,
    view: &View,
    theme: Theme,
    body: Rect,
) -> Rect {
    let h = body.h;
    let r = h * 0.5;
    // A pastilha, na cor do GRUPO — a mesma `cat_token` que tinge o cabeçalho do cartão
    // completo, para os dois desenhos falarem da mesma família.
    fill_rounded_rect(ctx.scene, body, r, resolve(cat_token(n.category), theme));

    // ⭐⭐ **O NOME, a encher a cápsula.**
    //
    // ⚠️ **Alinhado à ESQUERDA com o mesmo respiro em cima, em baixo e ao lado** — e não centrado:
    // centrar exige medir o texto (`TextSystem::prefix_width_weighted`), e essa medida vive numa
    // crate que este painel não tem; a alternativa, uma estimativa de avanço médio, seria uma
    // SEGUNDA conta da mesma largura ao lado da que o corte com reticências já faz — *e duas
    // contas para a mesma largura põem o nome num sítio e o corte noutro* (a cicatriz que o
    // `crumb_w` deste mesmo ficheiro regista por escrito). Numa pastilha em que o nome quase
    // sempre TRANSBORDA a largura disponível, a diferença visível é nenhuma.
    let tamanho = h * NOME_DA_CAPSULA;
    let margem = h * MARGEM_X;
    let disponivel = (body.w - 2.0 * margem).max(0.0);
    paint_text_title_elided(
        ctx.text_system,
        ctx.scene,
        &n.display_name,
        body.x + margem,
        body.y + (h - tamanho) * 0.5,
        tamanho,
        disponivel,
        resolve(ColorToken::Text1, theme),
    );

    // Os pinos, no tamanho que pararam de encolher — ver o doc acima.
    let raio = crate::geom::raio_do_pino(view, SOCKET_R);
    for (i, p) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, false, i);
        paint_socket_glyph(ctx, cx, cy, raio, p, theme);
    }
    for (i, p) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, true, i);
        paint_socket_glyph(ctx, cx, cy, raio, p, theme);
    }

    // ⚠️ **A selecção e o realce da largada sobrevivem à cápsula**, e têm de sobreviver: os dois
    // gestos que os acendem (arrastar uma carta sobre outra, enfiá-la num fio) são exactamente os
    // que um artista faz com o grafo AFASTADO, para ver a cadeia inteira.
    if state.selected.contains(&n.id) {
        ph2d_editor_core::paint::stroke_frame(
            ctx.scene,
            body,
            r,
            theme,
            ph2d_tokens::visuals::Feel::Selected,
            2.0,
            resolve(ColorToken::Accent, theme),
        );
    }
    if let Some(forca) = crate::realce::realce_do_cartao(state, n.id) {
        ph2d_editor_core::paint::stroke_frame(
            ctx.scene,
            body,
            r,
            theme,
            ph2d_tokens::visuals::Feel::Selected,
            ph2d_tokens::StrokeToken::Heavy.px(),
            resolve(ColorToken::Success, theme).multiply_alpha(forca),
        );
    }
    body
}

#[cfg(test)]
#[path = "paint_capsula_tests.rs"]
mod tests;
