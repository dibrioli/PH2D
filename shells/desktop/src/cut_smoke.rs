//! **A cena dos CORTES** — `PH2D_BUILD_SMOKE=44` (plano 25 §7, a W4).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não arma modo nenhum**: o gesto que este smoke prova começa no pill, e
//! uma cena que armasse o modo pularia justamente a costura que ela existe para exercer (a cicatriz
//! que o `impasto_smoke` do Painter prega).
//!
//! O que ela monta:
//! - um **losango** FECHADO (a lei da wave: cortado, ele tem de dar duas formas **FECHADAS**, no
//!   lugar onde estava);
//! - **duas fitas com as pontas encostadas** (Average + Join têm de as soldar num vértice só);
//! - **duas fitas com as pontas SEPARADAS** (o Join tem de pôr o segmento entre elas — a outra
//!   metade da lei, e sem ela o smoke não distingue *soldar* de *ligar*);
//! - uma **seta** de sentido óbvio (o Reverse tem de a inverter, e num traço simétrico ninguém veria).

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Largura do traço das referências, em unidades de mundo.
const STROKE_W: f64 = 0.05;

fn vertex(a: [f64; 2]) -> VecVertex {
    VecVertex {
        anchor: a,
        in_handle: a,
        out_handle: a,
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn poly(pts: &[[f64; 2]], closed: bool, rgb: [u8; 3]) -> VecPath {
    VecPath {
        verts: pts.iter().map(|p| vertex(*p)).collect(),
        closed,
        stroke: Some(StrokeSpec::new(
            Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
            STROKE_W,
        )),
        ..VecPath::default()
    }
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => announce(app),
        _ => {}
    }
}

/// **A geometria da cena, numa tabela** — `(pontos, fechado, cor)`.
///
/// ⚠️ Ela é `const` e partilhada com a sonda de baixo de propósito: os vãos que a mensagem anuncia
/// (o "quase a tocar" e o "longe") são MEDIDOS daqui, não escritos de memória — o doc não os
/// repete, justamente para não haver uma segunda cópia a envelhecer. Uma cena que afirma um número que
/// a geometria dela não tem é a forma exacta de um smoke que engana quem o corre.
type Piece = (&'static [[f64; 2]], bool, [u8; 3]);

const PIECES: &[Piece] = &[
    // (1) O ANEL — um losango fechado, à esquerda. A tesoura abre-o e, na 2ª tesourada, parte-o.
    (
        &[[-2.6, 0.0], [-1.8, 0.9], [-1.0, 0.0], [-1.8, -0.9]],
        true,
        [70, 150, 220],
    ),
    // (2) DUAS FITAS COM AS PONTAS ENCOSTADAS (mas NÃO coincidentes). É o par do Average + Join —
    // sem o vão, o Average não teria o que aproximar e o smoke seria sobre uma solda já feita.
    (
        &[[0.0, 1.4], [0.9, 0.9], [1.2, 0.2]],
        false,
        [120, 190, 120],
    ),
    (
        &[[1.32, 0.2], [1.6, 0.9], [2.5, 1.4]],
        false,
        [120, 190, 120],
    ),
    // (3) DUAS FITAS COM AS PONTAS SEPARADAS. O Join tem de ligar com um SEGMENTO — a metade da
    // lei que distingue *soldar* de *ligar*.
    (&[[0.0, -0.6], [0.9, -1.1]], false, [220, 150, 90]),
    (&[[1.9, -1.1], [2.8, -0.6]], false, [220, 150, 90]),
    // (4) A SETA — um traço de sentido ÓBVIO, para o Reverse ser visível. Numa forma simétrica a
    // inversão não muda um pixel, e o smoke não diria nada.
    (
        &[[-2.6, -1.8], [-1.4, -1.8], [-1.7, -1.5]],
        false,
        [200, 110, 200],
    ),
];

/// O vão entre a ponta final da peça `a` e a ponta inicial da peça `b`, em unidades de mundo.
fn gap(a: usize, b: usize) -> f64 {
    let p = PIECES[a].0.last().expect("peça sem pontos");
    let q = PIECES[b].0.first().expect("peça sem pontos");
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt()
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    for (pts, closed, rgb) in PIECES {
        gfx.vec_scene.push_path(poly(pts, *closed, *rgb));
    }
}

fn announce(app: &mut crate::App) {
    let n = app.gfx.as_ref().map_or(0, |g| g.vec_scene.paths().len());
    let gap_near = gap(1, 2);
    let gap_far = gap(3, 4);
    let tol = ph2d_vec_edit::WELD_TOL;
    eprintln!(
        "[smoke] O CORTE (plano 25 §7, W4): {n} formas -- um LOSANGO fechado (azul), duas fitas \
         VERDES com as pontas quase a tocar (vao MEDIDO: {gap_near:.2}), duas fitas LARANJA com as \
         pontas longe (vao {gap_far:.2}) e uma SETA roxa. Nenhum modo esta' armado: o gesto comeca \
         no pill. \
         [A LINHA DE CORTE] (1) na fileira TOOL clique **Cut**; (2) DESENHE uma linha atravessando \
         o losango azul de lado a lado, exactamente como se desenha com a Pen (clique, clique, e \
         **Enter** ou Esc para terminar) -- ela tem de aparecer **HACHURADA em ambar, com uma \
         TESOURA na ponta**; se ela nao aparecer, PARE; (3) repare que logo abaixo da fileira TOOL \
         nasceram DOIS botoes: **Cut** e **Discard Cut Line** -- eles nao existiam antes de a linha \
         existir; (4) pill **Select** e ARRASTE a linha de corte: ela e' um objeto como qualquer \
         outro (mova-a, e no pill **Node** edite os nos dela) -- **a hachura tem de permanecer** \
         enquanto ela e' movida, e ela **nunca** pode ganhar preenchimento nem cor de traco (tente \
         dar-lhe um Fill pela seccao Style: ela tem de continuar hachurada); (5) volte ao pill \
         **Cut**, com a linha ainda selecionada, e clique **Cut**. \
         [A LEI] (6) o losango tem de virar DUAS formas **FECHADAS** (preenchidas!), nao duas \
         fitas abertas -- se sair fita, PARE: e' o defeito que esta wave existe para fechar; \
         (7) as pecas tem de ficar **ONDE o losango estava** -- nenhuma pode saltar para outro \
         lugar da tela; (8) pill Select e arraste uma peca: as duas separam-se limpas; (9) Ctrl+Z \
         devolve o losango inteiro, **num unico** Ctrl+Z. \
         [A LINHA SOBREVIVE] (10) repare que a linha de corte **continua la'** depois do corte -- \
         mova-a para outro sitio e clique **Cut** de novo; (11) clique **Discard Cut Line**: ela \
         some, e os dois botoes somem com ela. \
         [AS TRES CONDICOES] (12) com a linha por cima de VARIAS formas sobrepostas, clique Cut: \
         **TODAS** sao cortadas -- a seleccao nao estreita alvos; (13) pill **Select** e clique no \
         VAZIO (a lamina deixa de estar selecionada); volte ao pill **Cut**: os dois botoes \
         **somem** -- sem a lamina selecionada nao ha' o que cortar; (14) escolha outro pill \
         (Pen, Node, Select): os dois botoes **somem** tambem -- eles sao controles desta \
         ferramenta, e ela nao esta' na mao. \
         [O QUE NAO CORTA, e e' correcto] (15) desenhe uma linha nova que PARE no meio de uma \
         forma e clique Cut: **nada acontece** -- uma regiao menos uma fenda continua inteira; \
         (16) esconda ou TRAVE uma forma na Hierarquia e corte por cima dela: ela sobrevive \
         intacta. \
         [O CORTADOR FECHADO] (17) desenhe uma linha de corte **FECHADA** (termine-a clicando no \
         primeiro ponto) por cima de uma forma e clique Cut: sai o MIOLO e o resto -- um cortador \
         fechado nao precisa de extensao nenhuma. \
         [AS FITAS] (18) as fitas VERDES/LARANJA sao ABERTAS: o corte nao as toca por ora, e isso \
         esta' nomeado no handoff. Use-as para o **Join** (Select + Shift+clique + Join na secao \
         PATH) e o **Average** (pill Node, dois nos, Average) -- as pontas verdes estao a \
         {gap_near:.2}, muito acima da tolerancia de solda ({tol:e}), entao o Join tem de por um \
         SEGMENTO a liga-las; nas laranja (vao {gap_far:.2}) o segmento e' longo e obvio. \
         [REVERSE] (19) clique a SETA roxa e, na secao PATH, **Reverse**; confirme com um MARKER \
         de ponta na secao Stroke -- ele salta de uma extremidade para a outra."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A SONDA da cena** — os números que a mensagem anuncia saem daqui, e este gate afirma que
    /// eles descrevem o material que ela de facto monta.
    ///
    /// ⚠️ O vão "quase a tocar" tem de ser **maior** que a tolerância de solda (senão o passo do
    /// Join anunciaria um segmento de ligação que não apareceria) e **pequeno** ao olho (senão o
    /// artista não leria "quase a tocar"). O vão "longe" tem de ser inconfundível.
    #[test]
    fn the_scene_measures_what_its_message_announces() {
        let near = gap(1, 2);
        let far = gap(3, 4);
        assert!(
            near > ph2d_vec_edit::WELD_TOL * 100.0,
            "as pontas 'quase a tocar' ja' estao dentro da tolerancia de solda ({near}) -- o passo              do Join anunciaria um segmento que nunca apareceria"
        );
        assert!(near < 0.25, "vao 'quase a tocar' grande demais: {near}");
        assert!(far > 0.8, "vao 'longe' pequeno demais: {far}");
        assert_eq!(format!("{near:.2}"), "0.12", "o numero que a mensagem diz");
        assert_eq!(format!("{far:.2}"), "1.00", "o numero que a mensagem diz");
    }

    /// A cena tem de conter as QUATRO coisas que a mensagem manda julgar: um fechado (a tesoura),
    /// dois pares de abertos (o Join nas duas leis) e a seta (o Reverse).
    #[test]
    fn the_scene_contains_every_shape_the_message_asks_about() {
        assert_eq!(PIECES.len(), 6, "6 formas");
        assert_eq!(PIECES.iter().filter(|p| p.1).count(), 1, "UM fechado");
        assert!(
            PIECES.iter().all(|p| p.0.len() >= 2),
            "toda peça tem geometria"
        );
    }
}

/// 📏 **SONDA — a mensagem que a cena imprime**, com os números MEDIDOS da tabela.
///
/// A cena precisa de janela para correr, e a mensagem é o contrato que o Enio lê antes de julgar:
/// esta sonda imprime-a sem janela nenhuma, para que quem escreve a wave a leia ANTES de a
/// anunciar. `cargo test -p ph2d-host-desktop --bins probe_cut_smoke_message -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime a mensagem da cena =44"]
fn probe_cut_smoke_message() {
    let (gap_near, gap_far) = (gap(1, 2), gap(3, 4));
    eprintln!("vao 'quase a tocar' = {gap_near:.4} · vao 'longe' = {gap_far:.4}");
    eprintln!(
        "tolerancia de solda = {:e} ({:.0}x menor que o vao curto)",
        ph2d_vec_edit::WELD_TOL,
        gap_near / ph2d_vec_edit::WELD_TOL
    );
    eprintln!(
        "formas = {} · fechadas = {}",
        PIECES.len(),
        PIECES.iter().filter(|p| p.1).count()
    );
}
