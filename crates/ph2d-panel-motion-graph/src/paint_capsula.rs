//! ⭐⭐⭐ **A CÁPSULA** — o desenho de um nó quando o zoom o encolhe abaixo do limiar do texto.
//!
//! Ordem do dono (2026-09-19): *«se depois disso o zoom continuar a reduzir os nós, o desenho
//! tradicional dos nós se modifica para uma simples cápsula da cor característica do grupo a que o
//! nó pertence, com o nome do nó ocupando toda a cápsula, os parâmetros de ajustes são escondidos
//! e os slots de conexão ficam maiores»*.
//!
//! ⛔⛔ **A 1.ª redacção foi REPROVADA pelo dono, e as quatro queixas tinham DUAS causas:**
//! *«as cápsulas ficaram pequenas e finas, com tamanhos irregulares e fonts irregulares, com slots
//! de conexão pequenos. Quero cápsulas grossas grandes e robustas, fonts de tamanho único bem
//! alinhadas no centro da cápsula e sem 3 pontos (…). Quero slots de conexão grandes e bem
//! visíveis.»*
//!
//! | queixa | causa |
//! |---|---|
//! | tamanhos irregulares · fonts irregulares | a altura seguia a contagem de PINOS, e o nome era dimensionado a partir dela — *uma grandeza a alimentar duas leituras dá duas irregularidades* |
//! | pequenas e finas | `HEADER_H` (26 unidades) é a altura de um cabeçalho, não de uma pastilha |
//! | 3 pontos | o nome era cortado ao caber, em vez de a FONTE ser dimensionada para o nome mais comprido do catálogo caber |
//! | slots pequenos | o raio partia do `SOCKET_R` do cartão, e não do ALVO |
//!
//! ⚠️ **Irmão do [`super::paint_card`] por RESPONSABILIDADE:** aquele desenha o cartão que o
//! artista LÊ de perto; este o que ele RECONHECE de longe.

use super::*;

/// **O nome mais comprido do catálogo**, MEDIDO (sonda de 2026-09-19 sobre os 136 tipos
/// registados): *«Fibonacci Spiral»*, `16` caracteres — mediana `7`, p90 `12`.
///
/// ⛔⛔ **É dele que sai o tamanho da fonte, e é por isso que não há reticências:** em vez de
/// cortar o nome ao que cabe, a fonte é escolhida para o PIOR nome caber. ⚠️ Há um censo do outro
/// lado (`ph2d-app-motion`) a afirmar que nenhum nó registado passa deste número — *um nome novo
/// mais comprido reprova lá, com o endereço, em vez de aparecer cortado na tela do artista*.
pub(crate) const NOME_MAIS_LONGO: f32 = 16.0; // LITERAL-PX-OK: contagem de caracteres MEDIDA no catálogo

/// **O avanço médio por caractere**, em fracção do corpo da fonte — o mesmo idioma que o
/// `geom::crumb_w` usa para o mesmo fim.
///
/// ⭐ **MEDIDO, e com folga declarada:** *«Simulation Zone»* (15 caracteres, o mais largo dos
/// nomes compridos do catálogo) mede `138,0` unidades a corpo `17,9` ⇒ `0,514` por caractere. O
/// `0,58` é esse número com **13 % de folga**, porque uma estimativa que erre para o lado curto
/// devolve as reticências que o dono recusou. ⚠️ E há um gate com o medidor REAL a confirmá-lo.
const AVANCO_POR_CHAR: f32 = 0.58; // LITERAL-PX-OK: fracção do corpo da fonte

/// **O respiro lateral do nome**, em unidades de grafo.
const MARGEM_X: f32 = 12.0; // LITERAL-PX-OK: respiro lateral, unidades de grafo

/// ⭐⭐⭐ **O TAMANHO ÚNICO do nome numa cápsula** — *«fonts de tamanho único»*, e ele é
/// **DERIVADO**: o maior corpo em que [`NOME_MAIS_LONGO`] caracteres ainda cabem na largura da
/// pastilha.
///
/// `(190 − 2 × 12) / (16 × 0,58) = 17,9` unidades, contra as `13` do título do cartão — **1,38×**,
/// e a mesma para toda cápsula porque a altura delas é uma só.
pub(crate) const CAPSULA_FONTE: f32 =
    (geom::CARD_W - 2.0 * MARGEM_X) / (NOME_MAIS_LONGO * AVANCO_POR_CHAR);

/// ⛔ E a ordem *«bem maiores»* é uma afirmação conferível pelo COMPILADOR — `1,3×` o título do
/// cartão, no mínimo. ⚠️ Ela vive aqui e não num teste porque o clippy recusa um `assert!` sobre
/// duas constantes (ele é dobrado antes de correr), que é a mesma cicatriz que o espaçamento do
/// pincel afiado desta casa já pagou.
const _: () = assert!(CAPSULA_FONTE > 1.3 * TITLE_SIZE); // LITERAL-PX-OK: a razão que «bem maiores» nomeia

/// ⭐⭐⭐ **ESTE NOME CABE NUMA CÁPSULA SEM RETICÊNCIAS?** — a porta que o censo do catálogo (em
/// `ph2d-app-motion`, onde os 136 tipos vivem) pergunta por cada nó registado.
///
/// ⛔⛔ **Ela existe porque a propriedade é sobre a LARGURA e o censo natural é sobre o NÚMERO de
/// caracteres.** Dezasseis `M` medem `261` unidades e dezasseis letras de um nome real medem
/// `136` — *um censo de contagem aprovaria um nome que o pintor cortaria*. ⇒ quem responde é o
/// medidor REAL, o mesmo que o corte com reticências consulta.
#[must_use]
pub fn nome_cabe_na_capsula(text_system: &mut ph2d_text::TextSystem, nome: &str) -> bool {
    let largura =
        text_system.prefix_width_weighted(nome, CAPSULA_FONTE, ph2d_text::FontWeight::SEMI_BOLD);
    largura <= geom::CARD_W - 2.0 * MARGEM_X
}

/// **ONDE O NOME COMEÇA** — o `x` que o centra na pastilha, dada a largura MEDIDA do texto.
///
/// ⚠️ **Uma função e não uma linha dentro do pintor**, pela razão que esta linha já pagou duas
/// vezes: uma decisão enfiada no meio do desenho só se deixa gatear por um censo TEXTUAL, e um
/// censo de texto sobrevive a um `if false &&`. *«Bem alinhadas no centro» é uma afirmação sobre
/// um número, e um número mede-se.*
pub(crate) fn x_do_nome(largura_do_texto: f32, body: Rect) -> f32 {
    body.x + (body.w - largura_do_texto) * 0.5
}

/// Desenha a cápsula e devolve o rect do corpo (o que o hit-test regista).
pub(super) fn draw_capsula(
    ctx: &mut PaintCtx,
    state: &MotionGraphPanelState,
    n: &GraphNodeView,
    view: &View,
    theme: Theme,
    body: Rect,
) -> Rect {
    let h = body.h;
    // ⭐ Cantos totalmente arredondados: é isso que faz dela uma cápsula e não um cartão baixo.
    let r = h * 0.5;
    fill_rounded_rect(ctx.scene, body, r, resolve(cat_token(n.category), theme));

    // ⭐⭐ **O NOME, no centro dos DOIS eixos.** ⚠️ Centrado com a medida REAL do texto
    // (`prefix_width_weighted`), e não com a estimativa que dimensionou a fonte: *«bem alinhadas
    // no centro» é uma propriedade que uma estimativa de avanço médio não consegue entregar.*
    let corpo = CAPSULA_FONTE * view.zoom;
    let largura = ctx.text_system.prefix_width_weighted(
        &n.display_name,
        corpo,
        ph2d_text::FontWeight::SEMI_BOLD,
    );
    let disponivel = (body.w - 2.0 * MARGEM_X * view.zoom).max(0.0);
    paint_text_title_elided(
        ctx.text_system,
        ctx.scene,
        &n.display_name,
        x_do_nome(largura.min(disponivel), body),
        // ⚠️ O `y` que o pintor recebe é o TOPO da linha, e a linha mede um corpo: centrar é
        // descontar meio corpo do meio da pastilha.
        body.y + (h - corpo) * 0.5,
        corpo,
        disponivel,
        resolve(ColorToken::Text1, theme),
    );

    // Os pinos, no tamanho do ALVO — ver [`geom::raio_do_pino_na_capsula`].
    for (i, p) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, false, i);
        let raio = geom::raio_do_pino_na_capsula(view, n.inputs.len());
        paint_socket_glyph(ctx, cx, cy, raio, p, theme);
    }
    for (i, p) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, true, i);
        let raio = geom::raio_do_pino_na_capsula(view, n.outputs.len());
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
