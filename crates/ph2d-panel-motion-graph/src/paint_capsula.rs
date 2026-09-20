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
//! ⛔⛔⛔ **E a 2.ª redacção foi reprovada na FORMA** (2026-09-20): *«Fonts maiores. linhas mais
//! grossas. No lugar das capsulas os retângulos como nos headers dos nós, só que grandes»*.
//!
//! | queixa | cura |
//! |---|---|
//! | fonts maiores | o corpo era uma ESTIMATIVA por contagem de caracteres com `13 %` de folga; medido por busca no corpo real, a mesma pastilha aceita `+20 %` ([`CAPSULA_FONTE`]) |
//! | retângulos como nos headers | o raio era `altura × 0,5`, a definição de pastilha; passa a ser o [`CARD_RADIUS`] do cabeçalho ([`raio_do_canto`]) |
//! | linhas mais grossas | o fio continuava a encolher com o zoom depois de tudo o resto parar — `crate::paint::paint_wire::largura_do_fio` |
//!
//! ⚠️ **O nome «cápsula» fica, e é HISTÓRICO:** ele é o nome do REGIME (o desenho de longe), não
//! da forma — que hoje é um rectângulo. Renomeá-lo atravessaria a API pública que o censo do
//! catálogo consome noutra crate, por zero mudança de comportamento; *o que não pode ficar é a
//! forma descrita errada, e por isso a tabela acima está aqui.*
//!
//! ⚠️ **Irmão do [`super::paint_card`] por RESPONSABILIDADE:** aquele desenha o cartão que o
//! artista LÊ de perto; este o que ele RECONHECE de longe.

use super::*;

/// ⭐⭐⭐ **O TAMANHO ÚNICO do nome numa cápsula** — *«fonts de tamanho único»* (2026-09-19) e
/// *«fonts maiores»* (2026-09-20).
///
/// ⭐⭐ **MEDIDO por BUSCA, no corpo REAL:** `mede_o_corpo_maximo_da_capsula` (em
/// `ph2d-app-motion`, onde vivem os **136** tipos registados) procura o maior corpo em que o pior
/// nome do catálogo ainda cabe em `190 − 2 × 12 = 166` unidades, medindo com o mesmo medidor que
/// o pintor consulta. Resposta: **`21,794`**, com *«Simulation Zone»* a assentar em `166,00`.
/// Ship-se `21,5`, `1,3 %` abaixo — a folga é do **empate na borda**, não de uma estimativa.
///
/// ⛔⛔ **A 1.ª redacção CONTAVA CARACTERES, e as reticências que o dono recusou eram pagas por
/// `13 %` de folga num avanço médio:** `n_chars × 0,58` devolvia `17,9`. A medição mostrou a
/// folga a ser **dinheiro em cima da mesa** — `+20 %` de corpo pela mesma pastilha.
///
/// ⛔⛔⛔ **E a 2.ª redacção mediu a corpo `100` e DIVIDIU, supondo a largura linear no corpo —
/// o gate do painel reprovou-a em voz alta.** A largura por unidade de corpo do pior nome:
///
/// | corpo | largura/unidade |
/// |---:|---:|
/// | `100,0` | `7,3608` |
/// | `30,0` | `7,4121` |
/// | `21,8` | **`7,6167`** |
/// | `17,9` | `7,7139` |
///
/// ⇒ *uma medição feita a um corpo não afirma nada sobre outro* (o arredondamento de métricas por
/// tamanho não é linear, e ele erra sempre no sentido que CORTA). A extrapolação pedia `22,33` e
/// o nome mediria `169,8` — cortado.
///
/// Contra as `13` do título do cartão: **1,65×** (eram `1,38×`), e o mesmo corpo para toda
/// cápsula porque a altura delas é uma só.
pub(crate) const CAPSULA_FONTE: f32 = 27.95; // LITERAL-PX-OK: 21,5 x 1,30, ordem do dono

/// ⛔ E a ordem *«bem maiores»* é uma afirmação conferível pelo COMPILADOR — `1,6×` o título do
/// cartão, no mínimo, medido em `1,72×`. ⚠️ Ela vive aqui e não num teste porque o clippy recusa
/// um `assert!` sobre duas constantes (ele é dobrado antes de correr), que é a mesma cicatriz que
/// o espaçamento do pincel afiado desta casa já pagou.
///
/// ⚠️⚠️ **A barra SUBIU de `1,3` para `1,6` com a 2.ª ordem do dono, e isso é uma CATRACA:** sem
/// ela, voltar à estimativa por contagem de caracteres (`1,38×`) passaria neste ponto sem uma
/// linha vermelha — *uma cerca escrita para a 1.ª ordem não defende a segunda*.
const _: () = assert!(CAPSULA_FONTE > 2.0 * TITLE_SIZE); // LITERAL-PX-OK: a razão que «30 % maiores» nomeia

/// ⭐⭐⭐ **ESTE NOME CABE NUMA CÁPSULA SEM RETICÊNCIAS?** — a porta que o censo do catálogo (em
/// `ph2d-app-motion`, onde os 136 tipos vivem) pergunta por cada nó registado.
///
/// ⛔⛔ **Ela existe porque a propriedade é sobre a LARGURA e o censo natural é sobre o NÚMERO de
/// caracteres.** Dezasseis `M` medem `261` unidades e dezasseis letras de um nome real medem
/// `136` — *um censo de contagem aprovaria um nome que o pintor cortaria*. ⇒ quem responde é o
/// medidor REAL, o mesmo que o corte com reticências consulta.
///
/// ⚠️⚠️ **O QUE ELA MEDE MUDOU em 2026-09-20, e o nome ficou:** desde que a largura da pastilha
/// segue o nome, *«cabe na largura do cartão»* deixou de ser a pergunta — a pastilha cresce. O
/// que pode CORTAR um nome é a **estimativa** que a geometria usa quando ninguém mediu
/// ([`geom::estimativa_da_largura`]) ser mais estreita que o texto, e é isso que esta porta
/// agora responde. *Uma porta cuja pergunta muda tem de o dizer, senão o censo do outro lado
/// continua verde a afirmar outra coisa.*
#[must_use]
pub fn nome_cabe_na_capsula(text_system: &mut ph2d_text::TextSystem, nome: &str) -> bool {
    let largura =
        text_system.prefix_width_weighted(nome, CAPSULA_FONTE, ph2d_text::FontWeight::SEMI_BOLD);
    largura <= geom::estimativa_da_largura(nome)
}

/// ⭐⭐⭐ **O RAIO DO CANTO** — *«os retângulos como nos headers dos nós»* (ordem do dono,
/// 2026-09-20), e por isso ele é o **mesmo [`CARD_RADIUS`] que o `paint_card` dá ao cabeçalho**.
///
/// ⛔ **A 1.ª redacção devolvia `altura × 0,5`**, que é a definição de uma pastilha: com ela os
/// topos e os fundos eram semicírculos e o dono leu-os como cápsulas, que é o que ele acabou de
/// recusar. *A diferença entre as duas formas cabe nesta linha.*
///
/// ⚠️ **Uma função e não uma linha dentro do pintor**, pela razão que este ficheiro já pagou duas
/// vezes: uma decisão enfiada no meio do desenho só se deixa gatear por um censo TEXTUAL, e um
/// censo de texto sobrevive a um `if false &&`. *Aqui mede-se o NÚMERO.*
pub(crate) fn raio_do_canto(view: &View) -> f32 {
    CARD_RADIUS * view.zoom
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
    // ⭐⭐ **O canto é o do CABEÇALHO, e é a MESMA constante** — ordem do dono (2026-09-20): *«no
    // lugar das cápsulas os retângulos como nos headers dos nós, só que grandes»*. A 1.ª redacção
    // usava `h × 0,5`, que é o que faz de um rectângulo uma pastilha.
    //
    // ⚠️ **Uma constante partilhada e não um número igual:** o `paint_card` desenha o cabeçalho
    // com este mesmo `CARD_RADIUS`, logo as duas formas **não podem divergir** no dia em que
    // alguém mexer no raio do cartão — *duas respostas à mesma pergunta divergem, uma porta não*.
    let r = raio_do_canto(view);
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
    let disponivel = (body.w - 2.0 * geom::MARGEM_X_DA_CAPSULA * view.zoom).max(0.0);
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
