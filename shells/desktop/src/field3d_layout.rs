//! ⭐⭐⭐ **A DIVISÃO DO CANVAS 3D** (W90) — a porta única dos retângulos dos viewports.
//!
//! # O que o plano pede, e porque isto é uma porta e não uma conta
//!
//! `docs/3DModeling/03_plano_implicito.md` fecha o canvas de primeira classe como *"modo de viewport
//! próprio, com **cabeçalho e divisão**"*. A divisão é a metade que não existe: o módulo desenha
//! sempre na área inteira, e um modelador vive a ver a peça de frente, de lado e de cima **ao mesmo
//! tempo**.
//!
//! ⚠️⚠️ **A lição que esta casa já pagou está no `CenterSplit::scene_viewport`** (o divisor
//! cena/grafo): *«um valor que é pixels não pode sair fraccionário da porta que o define»*. Lá, `h·t`
//! quase nunca é inteiro e a fracção fez a mesma função dar **duas respostas** — o passe de sprites
//! recebia `422,4` e o `set_scissor_rect` ao lado `422` —, e a diferença de `0,095 %` era invisível
//! parada e **um movimento** num pan (report do Enio, 25/08). ⇒ aqui os quatro retângulos saem de
//! **arestas inteiras** e ladrilham a área **exactamente**: sem folga, sem sobreposição, e a soma das
//! larguras é a largura.
//!
//! ⚠️ **O divisor NÃO é uma folga na geometria.** Insetar cada retângulo faria os quatro traçados
//! perder pixels e o fundo aparecer por baixo; a linha é pintada **por cima**, no chrome.

use ph2d_editor::zones::Rect as EditorRect;

/// ⭐ **Como o canvas está dividido.**
///
/// ⚠️ **`Quad` e não `N`**: a divisão de um modelador não é um número livre — é *frente, lado, cima
/// e a vista do artista*, que é o que o Blender, o Maya e o MoI oferecem e o que as **seis vistas
/// nomeadas** deste módulo (W47) já sabem produzir. *Um divisor arrastável com um número livre é
/// outra feature, e ela pede o cabeçalho primeiro.*
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Split {
    /// A vista única — o que o módulo sempre teve.
    #[default]
    One,
    /// Quatro vistas: cima, lado, frente, e a do artista.
    Quad,
}

impl Split {
    /// Quantos viewports esta divisão tem. ⚠️ **É a fonte da contagem** — quem cria os viewports lê
    /// daqui, e não de um `4` escrito ao lado.
    pub(crate) fn count(self) -> usize {
        match self {
            Self::One => 1,
            Self::Quad => 4,
        }
    }

    /// ⭐ **A vista nomeada de cada quadrante**, ou `None` para a do artista.
    ///
    /// ⚠️ **A disposição é a do Blender**, e não uma escolha minha: `Top` em cima à esquerda,
    /// `Right` em cima à direita, `Front` em baixo à esquerda, e a **perspectiva do artista** em
    /// baixo à direita — que é onde a mão dele já está quando abre a divisão.
    pub(crate) fn named(self, i: usize) -> Option<crate::field3d_views::Standard> {
        use crate::field3d_views::Standard;
        match (self, i) {
            (Self::Quad, 0) => Some(Standard::Top),
            (Self::Quad, 1) => Some(Standard::Right),
            (Self::Quad, 2) => Some(Standard::Front),
            _ => None,
        }
    }
}

/// Os retângulos de uma divisão — sem alocar, porque isto corre em todo quadro.
pub(crate) struct Rects {
    itens: [EditorRect; 4],
    n: usize,
}

impl Rects {
    /// Os retângulos vivos, na ordem dos viewports.
    pub(crate) fn as_slice(&self) -> &[EditorRect] {
        &self.itens[..self.n]
    }
}

/// ⭐⭐⭐ **A PORTA** — os retângulos que os viewports ocupam, em pixels **inteiros**.
///
/// Ver a nota do módulo: quem desenha, quem traça e quem responde *«este clique é meu?»* leem todos
/// daqui, e é isso que os mantém de acordo.
pub(crate) fn rects(area: EditorRect, split: Split) -> Rects {
    // As arestas inteiras. ⚠️ Arredondar as ARESTAS (e não a largura) é o que faz os pedaços
    // ladrilharem exactamente: cada aresta interior é o mesmo número para os dois vizinhos.
    let x0 = area.x.round();
    let y0 = area.y.round();
    let x2 = (area.x + area.w).round();
    let y2 = (area.y + area.h).round();
    let rect = |a: f32, b: f32, c: f32, d: f32| EditorRect::new(a, b, c - a, d - b);
    match split {
        Split::One => Rects {
            itens: [rect(x0, y0, x2, y2); 4],
            n: 1,
        },
        Split::Quad => {
            let x1 = (area.x + area.w * 0.5).round();
            let y1 = (area.y + area.h * 0.5).round();
            Rects {
                itens: [
                    rect(x0, y0, x1, y1),
                    rect(x1, y0, x2, y1),
                    rect(x0, y1, x1, y2),
                    rect(x1, y1, x2, y2),
                ],
                n: 4,
            }
        }
    }
}

/// ⭐ **Qual viewport contém este ponto** — a pergunta que o ponteiro faz.
///
/// ⚠️ **Toma uma SEQUÊNCIA e não os [`Rects`] de propósito:** quem pergunta em tempo de desenho tem
/// o layout acabado de calcular, e quem pergunta em tempo de PONTEIRO tem as áreas guardadas nos
/// viewports (o ponteiro corre fora do quadro). *Duas fontes, uma lei — escrever o teste de
/// pertença duas vezes é como um pixel da costura acaba por ter dois donos.*
///
/// ⚠️ **O teste é semi-aberto** (`>=` no início, `<` no fim): um
/// ponto exactamente sobre a aresta interior pertence a **um** viewport, nunca a dois. Com dois
/// donos, o mesmo pixel daria uma órbita em duas câmeras.
pub(crate) fn hit(rects: impl IntoIterator<Item = EditorRect>, p: [f32; 2]) -> Option<usize> {
    rects
        .into_iter()
        .position(|r| p[0] >= r.x && p[0] < r.x + r.w && p[1] >= r.y && p[1] < r.y + r.h)
}

#[cfg(test)]
#[path = "field3d_layout_tests.rs"]
mod tests;
