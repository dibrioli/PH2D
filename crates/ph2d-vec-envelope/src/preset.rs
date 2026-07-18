//! **Presets de gaiola** (ADR-0129 §4, Fatia C): Arc / Flag / Wave / … — cada um GERA a gaiola.
//!
//! # Por que isto veio DEPOIS do Coons, e não antes
//!
//! A fila do ADR lista C antes de D, e o handoff da linha chegou a afirmar que preset era *"só um
//! conjunto de `corners`; é UI, não motor"*. Isso vale para os presets **quad-expressáveis**
//! (Perspective, Free Distort) e **é falso para os que a palavra evoca**: com 4 cantos e lados retos
//! um "Arc" é um trapézio, e trapézio não é arco. O próprio ADR já dizia a versão certa — *"o preset
//! só vale se for GERADOR de gaiola … como gerador, **Quad e 4-curvas** saem quase de graça"*. Com a
//! Fatia D fechada, este arquivo é quase todo tabela.
//!
//! # A representação: cada preset é uma BARRIGA por lado
//!
//! Um preset não move os cantos — ele **enverga os lados**. Então ele é 8 números: para cada lado, o
//! quanto os seus 2 controles saem da corda, medido ao longo da **normal EXTERNA** daquele lado.
//! Positivo = para fora da gaiola.
//!
//! Escolher a normal (e não `y` do mundo) é o que torna a tabela legível: *Bulge* é "todo mundo para
//! fora", *Squeeze* é "os laterais para dentro", e a assimetria de direção dos lados (o lado de cima
//! é percorrido de TR para TL, ao contrário do de baixo) **se cancela** — a dupla inversão (sentido
//! do lado × sinal da normal) faz *Flag* e *Bulge* saírem com os dois lados escritos igual. Se a
//! tabela estivesse em `y` de mundo, cada linha teria de lembrar de qual lado ela fala.
//!
//! # A amplitude é MEDIDA para nunca dobrar
//!
//! É a diferença entre um knob honesto e um que "para de funcionar" no fim do curso: a alça do gesto
//! Mesh **para na fronteira** porque a mão pode pedir o impossível; um slider de preset não pode
//! pedir, porque a faixa dele é **desenhada**. Ver [`AMP`] — o primeiro valor tentado comprava a
//! garantia só para as formas que eu já tinha escrito, que é exatamente o tipo de garantia que
//! silencia até o dia em que alguém acrescenta uma linha.

use crate::{CageEdges, rest_edges};

/// Os cantos do quadrado unitário, ordem `[BL, BR, TR, TL]` — o domínio em que todo preset é
/// escrito. Quem o leva ao retângulo-fonte do envelope é o chamador (um afim de escala e
/// translação, que não muda nada do que está gateado aqui).
pub const UNIT_CAGE: [[f64; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

/// A barriga de cada lado, nos 2 controles, ao longo da normal EXTERNA — na ordem dos lados
/// (`[BL→BR, BR→TR, TR→TL, TL→BL]`) e, dentro de cada lado, na ordem em que ele é percorrido.
pub type EdgeBows = [[f64; 2]; 4];

/// Amplitude máxima de barriga, em fração do lado do quadrado unitário.
///
/// **Não é calibração de gosto: é o teto medido que torna a faixa inteira do slider utilizável.** A
/// garantia que ele compra é a forte — *qualquer* combinação de barrigas até `AMP`, em qualquer
/// direção, ao longo de toda a faixa de `bend`, **não dobra o patch** — e por isso ela vale também
/// para a linha que alguém acrescentar à tabela amanhã, não só para as sete que existem hoje.
///
/// ⚠️ **`0.35` foi o primeiro valor tentado e ele NÃO comprava isso.** Ele é seguro para as formas da
/// tabela atual (que envergam um par de lados por vez) e dobra quando os **quatro** lados envergam
/// juntos — um caso que nenhum preset de hoje produz, e que por isso teria passado despercebido até
/// o preset que o produzisse. `0.30` é o maior valor que sobrevive ao caso de quatro lados; o preço
/// é 14% de curso, invisível na tela, e o ganho é uma garantia que não depende da tabela.
pub const AMP: f64 = 0.30;

/// A gaiola do preset no quadrado unitário: os cantos (sempre os do quadrado — **um preset não move
/// canto**) e os lados envergados por `bows · bend`.
///
/// `bend ∈ [-1, 1]`; fora disso é clampado, porque a garantia de não-dobra é sobre essa faixa e um
/// chamador distraído não deve poder comprá-la de volta.
#[must_use]
pub fn preset_cage(bows: &EdgeBows, bend: f64) -> ([[f64; 2]; 4], CageEdges) {
    let bend = bend.clamp(-1.0, 1.0);
    let corners = UNIT_CAGE;
    let mut edges = rest_edges(&corners);
    for (i, edge) in edges.iter_mut().enumerate() {
        let a = corners[i];
        let b = corners[(i + 1) % 4];
        // A normal EXTERNA do lado: os cantos estão em sentido anti-horário, então girar a direção
        // do lado em −90° aponta para fora. Não normalizamos porque no quadrado unitário todo lado
        // mede exatamente 1 — e é *por isso* que o preset é escrito aqui e mapeado depois.
        let n = [b[1] - a[1], -(b[0] - a[0])];
        for (j, c) in edge.iter_mut().enumerate() {
            let k = bows[i][j] * bend;
            *c = [c[0] + n[0] * k, c[1] + n[1] * k];
        }
    }
    (corners, edges)
}

#[cfg(test)]
#[path = "preset_tests.rs"]
mod tests;
