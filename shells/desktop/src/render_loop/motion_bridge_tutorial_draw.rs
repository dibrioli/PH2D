//! ⭐⭐ **O VOCABULÁRIO DE DESENHO DOS TUTORIAIS — a porta única** (doc 103 §3).
//!
//! ⚠️ **Ele saiu do gerador do ciclo 2 quando o ciclo 3 passou a precisar dele**, e o corte é por
//! RESPONSABILIDADE e não por tamanho: o que está aqui é *como uma figura de tutorial se
//! enquadra e de que cor ela é*, e o que fica em cada gerador é *o que aquele ciclo desenha*.
//! O ciclo 2 desenha nuvens de pontos (um animador move um ponto); o ciclo 3 desenha malhas e
//! marcas orientadas (três dos treze nós dele escrevem `rot` ou `size`, a que um ponto é cego).
//! São figuras diferentes de assuntos diferentes — mas **duas paletas seriam dois tutoriais com
//! duas caras**, e *isso* é a mesma pergunta respondida duas vezes.

/// A moldura que contém tudo, com margem — partilhada pelo grupo: `(cx, cy, largura, altura)`.
///
/// ⚠️ **`pub(super)` desde o ciclo 3, e a razão é a lei desta casa:** o gerador dos
/// deformadores desenha as figuras dele com ESTA função e com [`svg`]. Uma segunda cópia daria
/// dois tutoriais com dois enquadramentos e duas paletas, a divergirem no dia em que um dos
/// dois mudasse — *uma lei escrita em dois sítios ainda não é uma lei, só uma PORTA é*.
///
/// ⚠️⚠️ **Ela era QUADRADA, e isso desperdiçava 70 % de cada figura.** Dez das treze são uma
/// **fila** — largas e baixas —, e num quadrado elas ficam numa faixa fina ao meio: impressas a
/// cinco por linha, as cinco formas de onda da capa saíam com `30 px` de altura. *A moldura de
/// uma figura é a forma do que ela mostra, não a do sítio onde ela vai.*
pub(in crate::render_loop) fn moldura(conjuntos: &[&[[f32; 2]]]) -> (f32, f32, f32, f32) {
    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for c in conjuntos {
        for p in *c {
            x0 = x0.min(p[0]);
            x1 = x1.max(p[0]);
            y0 = y0.min(p[1]);
            y1 = y1.max(p[1]);
        }
    }
    let folga = ((x1 - x0).max(y1 - y0) * 0.06).max(6.0);
    (
        (x0 + x1) * 0.5,
        (y0 + y1) * 0.5,
        x1 - x0 + folga * 2.0,
        y1 - y0 + folga * 2.0,
    )
}

/// ⭐ **A PALETA das figuras dos tutoriais — uma porta só.**
///
/// ⚠️ **O que se partilha entre ciclos é a PALETA e o ENQUADRAMENTO, não o desenho.** O ciclo 2
/// desenha nuvens de pontos porque um animador move um ponto; o ciclo 3 desenha **marcas
/// orientadas**, porque três dos treze nós dele escrevem `rot` ou `size` e um ponto é cego a
/// isso. São figuras diferentes de assuntos diferentes — mas duas paletas seriam dois tutoriais
/// com duas caras, e *isso* é a mesma pergunta respondida duas vezes.
pub(in crate::render_loop) mod cor {
    /// O fundo da figura.
    pub(in crate::render_loop) const FUNDO: &str = "#141317";
    /// O que ENTROU no nó — a régua, e às vezes também a mensagem.
    pub(in crate::render_loop) const FANTASMA: &str = "#4b4560";
    /// O traço que liga a entrada à saída.
    pub(in crate::render_loop) const TRACO: &str = "#6a5b8c";
    /// O que SAIU.
    pub(in crate::render_loop) const FORTE: &str = "#c9a6ff";
}

pub(in crate::render_loop) fn svg(
    fortes: &[[f32; 2]],
    fantasma: &[[f32; 2]],
    (cx, cy, w, h): (f32, f32, f32, f32),
    campo: bool,
) -> String {
    // O raio segue a média geométrica do quadro — numa fila muito larga a altura é que decide
    // se dois pontos se tocam.
    let r = (w * h).sqrt() / if fortes.len() > 800 { 260.0 } else { 74.0 };
    const LARGURA_PX: f32 = 340.0;
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.2} {:.2} {w:.2} {h:.2}\" \
         width=\"{LARGURA_PX:.0}\" height=\"{:.0}\" role=\"img\">\n\
         <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"{:.2}\" fill=\"{}\"/>\n",
        cx - w / 2.0,
        -cy - h / 2.0,
        LARGURA_PX * h / w,
        cx - w / 2.0,
        -cy - h / 2.0,
        w.min(h) / 26.0,
        cor::FUNDO,
    ); // O y do mundo cresce para CIMA e o do SVG para baixo: a figura mostra o que o artista vê.
    for p in fantasma {
        s.push_str(&format!(
            "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{r:.2}\" fill=\"{}\"/>\n",
            p[0],
            -p[1],
            cor::FANTASMA
        ));
    }
    if campo {
        for (a, b) in fantasma.iter().zip(fortes) {
            s.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" \
                 stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linecap=\"round\"/>\n",
                a[0],
                -a[1],
                b[0],
                -b[1],
                cor::TRACO,
                r * 0.7
            ));
        }
    }
    for p in fortes {
        s.push_str(&format!(
            "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{r:.2}\" fill=\"{}\"/>\n",
            p[0],
            -p[1],
            cor::FORTE
        ));
    }
    s.push_str("</svg>\n");
    s
}
