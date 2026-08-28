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
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Split {
    /// A vista única — o que o módulo sempre teve.
    #[default]
    One,
    /// ⭐ Quatro vistas: cima, lado, frente, e a do artista — com **onde** as duas costuras estão.
    ///
    /// ⚠️ **As fracções vivem AQUI e não num campo ao lado**: a divisão *é* as duas costuras, e um
    /// `t` guardado noutro sítio seria um estado que pode discordar do modo. Elas nascem no meio
    /// ([`Split::quad`]) e são presas ao alcance legal em [`Split::with_t`].
    Quad {
        /// A fracção horizontal da área, medida da esquerda.
        tx: f32,
        /// A fracção vertical, medida do topo.
        ty: f32,
    },
}

impl Split {
    /// Quantos viewports esta divisão tem. ⚠️ **É a fonte da contagem** — quem cria os viewports lê
    /// daqui, e não de um `4` escrito ao lado.
    pub(crate) fn count(self) -> usize {
        match self {
            Self::One => 1,
            Self::Quad { .. } => 4,
        }
    }

    /// ⭐ **A divisão em quatro com as costuras no meio** — como ela nasce.
    pub(crate) fn quad() -> Self {
        Self::Quad { tx: 0.5, ty: 0.5 }
    }

    /// ⭐⭐ **As costuras noutro sítio, presas ao alcance legal.**
    ///
    /// ⚠️ **A trava é a da casa, lida e não re-decidida:** [`ph2d_editor::screens::layout::CenterSplit`]
    /// (o divisor cena/grafo) fixa `T_MIN = 0,25` e `T_MAX = 0,75` com a razão *«a cena e o grafo
    /// guardam sempre um quarto»* — e ela é `NaN`-aware, que é o que impede um arrasto degenerado de
    /// envenenar o layout. *A lei é a mesma; escrevê-la outra vez seria ter duas.*
    pub(crate) fn with_t(self, tx: f32, ty: f32) -> Self {
        use ph2d_editor::screens::layout::CenterSplit;
        match self {
            Self::One => Self::One,
            Self::Quad { .. } => Self::Quad {
                tx: CenterSplit::clamp_t(tx),
                ty: CenterSplit::clamp_t(ty),
            },
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
            (Self::Quad { .. }, 0) => Some(Standard::Top),
            (Self::Quad { .. }, 1) => Some(Standard::Right),
            (Self::Quad { .. }, 2) => Some(Standard::Front),
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
        Split::Quad { tx, ty } => {
            let x1 = area.w.mul_add(tx, area.x).round();
            let y1 = area.h.mul_add(ty, area.y).round();
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

/// A largura da faixa em que a costura se agarra, de cada lado dela.
///
/// ⚠️ **Ela é maior do que a linha desenhada**, e isso não é descuido: a pega de um divisor é uma
/// afirmação sobre o que o **dedo** alcança, não sobre o que o olho vê — é a lei que todo divisor de
/// janela segue, e apontar para uma linha de um pixel seria um gesto que só acerta por sorte.
const GRAB_HALF_PX: f32 = 5.0; // LITERAL-PX-OK: overlay metric (divider grab band)

/// ⭐⭐⭐ **QUE COSTURAS ESTE PONTO AGARRA** — `(a vertical, a horizontal)`.
///
/// `None` quando ele não agarra nenhuma. ⚠️ **As duas juntas são o caso do CENTRO**, e ele é
/// deliberado: agarrar o cruzamento move as duas costuras de uma vez, que é o que o Blender faz e o
/// que a mão espera quando aponta para o meio.
pub(crate) fn seam_grab(area: EditorRect, split: Split, p: [f32; 2]) -> Option<(bool, bool)> {
    let Split::Quad { .. } = split else {
        return None;
    };
    // ⚠️ **As costuras são lidas dos RETÂNGULOS**, nunca recalculadas a partir do `t`: elas são
    // arredondadas na porta, e uma segunda conta aqui erraria por meio pixel — que é exactamente a
    // largura de um gesto que falha de vez em quando.
    let r = rects(area, split);
    let q = r.as_slice();
    let (x1, y1) = (q[0].x + q[0].w, q[0].y + q[0].h);
    let dentro = p[0] >= q[0].x - GRAB_HALF_PX
        && p[0] <= q[3].x + q[3].w + GRAB_HALF_PX
        && p[1] >= q[0].y - GRAB_HALF_PX
        && p[1] <= q[3].y + q[3].h + GRAB_HALF_PX;
    if !dentro {
        return None;
    }
    let v = (p[0] - x1).abs() <= GRAB_HALF_PX;
    let h = (p[1] - y1).abs() <= GRAB_HALF_PX;
    (v || h).then_some((v, h))
}

/// ⭐ **A fracção que o ponteiro nomeia**, dado o retângulo do canvas — a metade inversa da
/// [`rects`].
///
/// ⚠️ Sem trava aqui: quem prende ao alcance legal é o [`Split::with_t`], e prender **duas** vezes
/// esconderia de qual das duas o número saiu.
pub(crate) fn t_at(area: EditorRect, p: [f32; 2]) -> (f32, f32) {
    (
        if area.w > 0.0 {
            (p[0] - area.x) / area.w
        } else {
            0.5
        },
        if area.h > 0.0 {
            (p[1] - area.y) / area.h
        } else {
            0.5
        },
    )
}

/// ⭐⭐⭐ **O CURSOR que a costura debaixo do ponteiro pede** (W93, report do Enio) — `None` quando
/// não há costura ali.
///
/// ⚠️ **Ele sai do MESMO [`seam_grab`] que o gesto usa**, e essa é a lei que o divisor do grafo do
/// Motion já escreve ao lado do dele: *o cursor e o gesto leem a mesma fonte, senão discordam sobre
/// onde a faixa está* — e o defeito seria a seta a aparecer um pixel ao lado de onde o arrasto
/// pega, que se lê como *«às vezes não agarra»*.
///
/// ⭐ **No cruzamento é o `Move`**: ali as duas costuras vão juntas, e uma seta de um eixo só
/// prometeria metade do gesto.
pub(crate) fn seam_cursor(
    area: EditorRect,
    split: Split,
    p: [f32; 2],
) -> Option<winit::window::CursorIcon> {
    use winit::window::CursorIcon;
    match seam_grab(area, split, p)? {
        (true, true) => Some(CursorIcon::Move),
        // ⚠️ A seta é PERPENDICULAR à linha: uma costura vertical move-se na horizontal.
        (true, false) => Some(CursorIcon::EwResize),
        (false, true) => Some(CursorIcon::NsResize),
        (false, false) => None,
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
