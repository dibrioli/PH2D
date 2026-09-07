//! ⭐⭐ **A GEOMETRIA DA FAIXA DE PARAMS DE UM CARTÃO** — as fileiras, as bandas, a faixa
//! desenhada e as zonas de um selector.
//!
//! ⚠️ **Irmã de [`super`] por RESPONSABILIDADE:** lá vive a geometria do GRAFO (a vista, o
//! corpo do cartão, os sockets, o popup, a moldura da pré-visualização); aqui a do que um cartão
//! **CONTROLA**. As duas mudam por razões diferentes — uma quando o grafo muda de forma, outra
//! quando um controlo ganha um gesto.
//!
//! ⚠️ **Uma coordenada só, e é a FAIXA** (`BandRow`): o pintor, o hit-test e o gesto falam todos
//! em índice de faixa. Se cada um contasse à sua maneira, um clique cairia uma fileira ao lado no
//! primeiro nó com secções.

use super::{CARD_W, HEADER_H, ROW_H, View, card_rows};
use crate::snapshot::GraphNodeView;
use ph2d_editor_core::zones::Rect;

/// **O tamanho do rótulo de um param no cartão.** Menor que o título (13) e igual ao readout
/// (11): a hierarquia do cartão é *nome do nó > o que ele faz > os seus botões*.
pub(crate) const PARAM_LABEL_SIZE: f32 = 11.0; // LITERAL-PX-OK: card param label font size

/// A menor altura de letra que ainda se LÊ num ecrã. Não é gosto: abaixo disto o rótulo é
/// uma mancha cinzenta, e desenhá-lo custa o mesmo que desenhá-lo legível.
const MIN_READABLE_PX: f32 = 9.0; // LITERAL-PX-OK: legibility floor

/// ⭐⭐⭐ **O LOD DO TEXTO da row — e é só do TEXTO** (doc 103 §7).
///
/// ⛔⛔ **A primeira versão escondia a ROW INTEIRA abaixo do limiar, e o smoke do Enio
/// (2026-09-05, foto) devolveu o resultado: «tudo em branco».** A cena abre com `zoom ≈ 0,5`,
/// a faixa continuava RESERVADA (a altura não segue o zoom, e não pode) e nada era desenhado
/// nela — *o pior dos dois mundos: o espaço pago e a informação ausente*. O Blender nunca faz
/// isto: o corpo do nó desenha sempre as caixas dos controlos, e o que desaparece ao afastar é
/// o TEXTO. ⇒ a barra e o nível pintam-se **sempre** (dois rectângulos, o barato), e só os dois
/// textos passam por aqui.
///
/// ⭐ E a barra sozinha ainda INFORMA: uma coluna de níveis diz de relance que knobs estão
/// altos e quais estão no fundo, que é o que um nó afastado tem para dizer.
///
/// Medido em 2026-09-05 (load 2,66): um cartão nu custa **11,3 µs** e uma row de param
/// **13,5 µs** — *uma row custa mais que um cartão inteiro*, porque as duas são dominadas
/// pelo TEXTO. A conta do quadro (`N × (11,3 + R × 13,5) µs` contra 16,67 ms):
///
/// | cena | custo | % do quadro |
/// |---|---:|---:|
/// | 120 cartões nus | 1,41 ms | 8 % |
/// | 40 cartões × 5 rows | 3,15 ms | 19 % |
/// | 40 cartões × 13 rows | 7,49 ms | **45 %** |
/// | 120 cartões × 5 rows | 9,46 ms | **57 %** |
///
/// ⭐⭐ **E o limiar paga-se sozinho:** ele é a LEGIBILIDADE (`11 px × zoom ≥ 9 px` ⇒
/// `zoom ≥ 0,82`), e a esse zoom o recorte do viewport já só deixa **~18 cartões** na tela ⇒
/// `18 × (11,3 + 8 × 13,5) = 2,1 ms` = **13 %**. *Aproximar mostra os params e esconde os
/// cartões; afastar faz o contrário — a mesma alavanca paga as duas coisas.*
///
/// ⚠️ **O que o LOD NÃO faz é mudar a ALTURA do cartão** ([`card_h`] é espaço de GRAFO e não
/// vê o zoom): a faixa fica reservada sempre. Um cartão que encolhesse ao afastar faria os
/// hit-rects saltarem debaixo do dedo a meio de um pinch.
pub(crate) fn param_text_is_drawn(view: &View) -> bool {
    PARAM_LABEL_SIZE * view.zoom >= MIN_READABLE_PX
}

/// **UMA ROW SÓ SE AGARRA QUANDO SE PODE MIRAR** — e é o MESMO limiar do texto, de propósito.
///
/// ⚠️ Parecem duas perguntas e são uma: *«esta row está utilizável por um humano agora?»*. Uma
/// fileira de 4 px não se lê **nem** se acerta (a régua do tablet pede 44 pt), e registá-la
/// roubaria ao corpo do cartão o gesto de ARRASTAR O NÓ sem dar nada em troca.
///
/// ⛔ Se um dia as duas divergirem — por exemplo um modo de toque com alvos maiores — elas
/// separam-se aqui, com a medição ao lado; até lá, uma fonte.
pub(crate) fn param_row_is_grabbable(view: &View) -> bool {
    param_text_is_drawn(view)
}

/// **O QUE ESTÁ NA FILEIRA `i` da faixa** — um cabeçalho de secção ou um param.
///
/// ⚠️ **Uma coordenada só.** O pintor, o hit-test e o gesto falam todos em índice de FAIXA; se
/// cada um contasse à sua maneira, um clique cairia uma fileira ao lado no primeiro nó com
/// secções. É a mesma razão de [`card_h`] viver aqui e não no pintor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BandRow {
    /// O cabeçalho da secção `sections[i]`.
    Header(usize),
    /// A row `params[i]`.
    Param(usize),
}

/// Quantas fileiras a faixa tem: as rows visíveis mais um cabeçalho por secção.
///
/// ⚠️ **`params` já chega FILTRADA** — uma secção fechada não deixa rows na lista —, então esta
/// conta não precisa de saber o estado da dobra: ela está no snapshot.
pub(crate) fn band_len(n: &GraphNodeView) -> usize {
    n.params.len() + n.sections.len()
}

/// O conteúdo da fileira `i`. `O(secções)`, e as secções de um nó contam-se pelos dedos.
pub(crate) fn band_at(n: &GraphNodeView, i: usize) -> Option<BandRow> {
    let mut fileira = 0usize;
    let mut param = 0usize;
    for (si, sec) in n.sections.iter().enumerate() {
        // O cabeçalho vem imediatamente antes da primeira row da secção.
        while param < sec.at as usize {
            if fileira == i {
                return Some(BandRow::Param(param));
            }
            fileira += 1;
            param += 1;
        }
        if fileira == i {
            return Some(BandRow::Header(si));
        }
        fileira += 1;
    }
    while param < n.params.len() {
        if fileira == i {
            return Some(BandRow::Param(param));
        }
        fileira += 1;
        param += 1;
    }
    None
}

/// Quantas fileiras a faixa de params RESERVA (cabeçalhos incluídos).
/// ⚠️ Independente do zoom, de propósito: ver [`param_text_is_drawn`].
pub(crate) fn card_param_rows(n: &GraphNodeView) -> f32 {
    band_len(n) as f32
}

/// O topo da FAIXA DE PARAMS, em espaço de grafo, relativo ao canto do cartão — logo abaixo
/// da última fileira de sockets.
pub(crate) fn param_band_top(n: &GraphNodeView) -> f32 {
    HEADER_H + card_rows(n) * ROW_H
}

/// O topo do READOUT, em espaço de grafo — abaixo dos params. ⚠️ **Uma porta**: o pintor
/// leria a mesma soma à mão e ficaria uma linha atrás no dia em que a faixa mudasse.
pub(crate) fn readout_top(n: &GraphNodeView) -> f32 {
    param_band_top(n) + card_param_rows(n) * ROW_H
}

/// Recuo da FAIXA em relação à borda do cartão — o mesmo dos dois lados, para a row ler como
/// uma peça POUSADA no cartão e não como uma banda que o atravessa.
///
/// ⚠️ **Vive aqui, e não no pintor, pela razão de [`card_h`]:** desde que o clique passou a
/// depender de ONDE dentro da row ele caiu (as setas de um selector), o pintor e o gesto têm de
/// medir a MESMA faixa. Duas cópias e a seta desenhada deixa de ser a seta clicada.
pub(crate) const TRACK_INSET_X: f32 = 6.0; // LITERAL-PX-OK: card param track x-inset
/// Folga vertical dentro da fileira: a faixa não encosta na de cima nem na de baixo.
pub(crate) const TRACK_INSET_Y: f32 = 2.0; // LITERAL-PX-OK: card param track y-inset

/// **A FAIXA desenhada dentro da fileira `i`** — o rectângulo que o pintor preenche e contra o
/// qual o gesto mede as zonas. Uma porta, dois leitores.
pub(crate) fn param_track_rect(row: Rect, z: f32) -> Rect {
    Rect::new(
        row.x + TRACK_INSET_X * z,
        row.y + TRACK_INSET_Y * z,
        (row.w - 2.0 * TRACK_INSET_X * z).max(0.0),
        (row.h - 2.0 * TRACK_INSET_Y * z).max(0.0),
    )
}

/// Largura do alvo de UMA seta de selector, em px lógicos.
///
/// ⚠️ **É o alvo, não o desenho:** o triângulo tem `ARROW_R` de meio-lado (`3,5`), e o resto é
/// a folga que faz o dedo acertar. `13 px` a `zoom 1` deixam `152` dos `178` da faixa para o
/// nome e o valor — e a `zoom 2`, que é a régua do tablet, cada seta mede `26 px`.
pub(crate) const ARROW_W: f32 = 13.0; // LITERAL-PX-OK: card selector arrow hit width
/// Meio-lado do triângulo de uma seta — o mesmo do chevron de secção, para as duas marcas do
/// cartão lerem como a mesma família.
pub(crate) const ARROW_R: f32 = 3.5; // LITERAL-PX-OK: card selector arrow half-size

/// ⭐⭐⭐ **ONDE DENTRO DE UMA ROW O CLIQUE CAIU** (report do Enio, 2026-09-07: *«para esse tipo
/// de campo deveríamos ter duas setas laterais e se clicar no centro abre-se um dropdown»* —
/// o selector do Blender).
///
/// ⚠️ **Só um SELECTOR lê isto** (um enum, uma fonte publicada, um canal). Numa row de número a
/// faixa inteira é uma coisa só — o nível arrasta-se em qualquer ponto, que é o *number field*
/// do Blender —, e cortar-lhe as pontas roubaria alcance ao arrasto sem dar nada em troca.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum RowZone {
    /// A seta da esquerda: a opção ANTERIOR.
    Prev,
    /// O nome: abre a LISTA.
    Centre,
    /// A seta da direita: a opção SEGUINTE.
    Next,
}

/// A zona de `x` dentro da faixa. ⚠️ **Uma faixa estreita não tem centro**: abaixo de duas setas
/// mais um resto, tudo é `Centre` — a lista é sempre legível (o menu é chrome, não escala com o
/// zoom), e duas setas coladas uma à outra seriam dois alvos que ninguém acerta.
pub(crate) fn row_zone(track: Rect, z: f32, x: f32) -> RowZone {
    let Some(seta) = arrow_slot(track, z) else {
        return RowZone::Centre;
    };
    if x < track.x + seta {
        RowZone::Prev
    } else if x > track.x + track.w - seta {
        RowZone::Next
    } else {
        RowZone::Centre
    }
}

/// **A largura de cada seta, ou `None` quando não há onde as pôr** — a porta que o pintor e o
/// gesto leem, para a seta desenhada ser a seta clicada.
///
/// ⚠️ **Uma faixa estreita não tem centro**: com menos de três larguras de seta, as duas
/// colariam uma à outra e o nome ficaria sem alvo. Aí não se desenha nenhuma e a faixa inteira
/// abre a lista — que é legível a qualquer zoom, porque o popup é chrome e não escala.
pub(crate) fn arrow_slot(track: Rect, z: f32) -> Option<f32> {
    let seta = ARROW_W * z;
    (track.w >= 3.0 * seta).then_some(seta) // LITERAL-PX-OK: CONTAGEM (as duas setas mais um meio), nao medida
}

/// O rect de ECRÃ da row de param `i` — a faixa inteira do cartão, que é também o alvo de
/// arrasto (o número arrasta-se em qualquer ponto da row, como no Blender).
///
/// ⚠️ **O mesmo rect serve o pintor e o hit-test**, pela razão que [`card_h`] já documenta:
/// uma row desenhada onde não se clica é um controlo morto sob o dedo.
pub(crate) fn param_row_rect(n: &GraphNodeView, view: &View, i: usize) -> Rect {
    // `i` é o índice de FAIXA (cabeçalhos incluídos) — ver [`BandRow`].
    let (sx, sy) = view.pt(n.x, n.y + param_band_top(n) + i as f32 * ROW_H);
    Rect::new(sx, sy, CARD_W * view.zoom, ROW_H * view.zoom)
}
