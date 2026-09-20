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

/// A menor altura de letra que ainda se LÊ num ecrã — o piso que esta lei teve até 2026-09-19.
/// Não era gosto: abaixo disto o rótulo é uma mancha cinzenta, e desenhá-lo custava o mesmo que
/// desenhá-lo legível.
///
/// ⚠️ Ele fica porque a ordem do dono é RELATIVA (*«20 % a mais que agora»*) — *um pedido relativo
/// que vira um número absoluto no código deixa de poder ser conferido*.
const PISO_ANTIGO_PX: f32 = 9.0; // LITERAL-PX-OK: legibility floor

/// ⭐⭐⭐ **QUANTO MAIS OS NÓS PODEM ENCOLHER antes de o texto sumir** — ordem do dono
/// (2026-09-19): *«permita que até que o zoom reduza os nós em 20 % a mais que agora, as fonts e
/// números dos nós ainda permaneçam visíveis»*.
///
/// ⭐⭐ **O que o paga é uma MEDIÇÃO NOVA, e ela derrubou o argumento que segurava o limiar
/// antigo.** A tabela de 2026-09-05 dizia *«uma row custa MAIS que um cartão inteiro»* (`13,5` µs
/// contra `11,3`), e era disso que vinha o *«o limiar paga-se sozinho»*. Medido outra vez em
/// 2026-09-19 pela mesma sonda ([`crate::measure_card_cost`], `--release`, `load 16,7`):
///
/// | | 2026-09-05 | 2026-09-19 |
/// |---|---:|---:|
/// | um cartão nu | `11,3 µs` | **`12,2 µs`** |
/// | uma row de param | `13,5 µs` | **`2,2 µs`** |
///
/// ⇒ **uma row passou a custar um SEXTO do que custava**, e menos de um quinto de um cartão. O
/// orçamento do novo limiar: a `zoom 0,655` cabem `(0,818/0,655)² = 1,56×` mais cartões no
/// recorte (~`28` contra os ~`18` de antes) ⇒ `28 × (12,2 + 8 × 2,2) = 0,83 ms` = **5,0 % de um
/// quadro**, contra os `3,2 %` do limiar antigo. *A folga de 20 % custa 1,8 pontos percentuais.*
const FOLGA_DO_DONO: f32 = 0.8; // LITERAL-PX-OK: a fracção que a ordem do dono nomeia

/// O piso de hoje — **derivado**, nunca escrito à mão. Ver [`FOLGA_DO_DONO`].
const MIN_READABLE_PX: f32 = PISO_ANTIGO_PX * FOLGA_DO_DONO;

/// ⛔ E a ordem é uma AFIRMAÇÃO conferível: um piso que deixe de ser 20 % abaixo do antigo não
/// compila. ⚠️ `assert!` de teste sobre duas constantes é dobrado pelo compilador antes de correr
/// — o clippy di-lo em voz alta —, então a cerca vive aqui.
const _: () = assert!(MIN_READABLE_PX < PISO_ANTIGO_PX);

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
/// ⭐⭐ **E o limiar paga-se sozinho:** ele é a LEGIBILIDADE (`11 px × zoom ≥ 7,2 px` ⇒
/// `zoom ≥ 0,655` desde 2026-09-19, ver [`FOLGA_DO_DONO`]), e a esse zoom o recorte do viewport
/// deixa **~28 cartões** na tela ⇒ `28 × (12,2 + 8 × 2,2) = 0,83 ms` = **5 %**. *Aproximar mostra
/// os params e esconde os cartões; afastar faz o contrário — a mesma alavanca paga as duas
/// coisas.*
///
/// ⚠️⚠️ **Os números da tabela acima são de 2026-09-05 e a de uma row está DESACTUALIZADA por
/// `6×`** — ver [`FOLGA_DO_DONO`] para a re-medição. *Uma tabela de custo num doc-comment é uma
/// fotografia, e esta envelheceu sem reclamar: ela ainda ensina a FORMA do orçamento (`N ×
/// (cartão + R × row)`), e o número da row já não é o que ela diz.*
///
/// ⚠️ **O que o LOD NÃO faz é mudar a ALTURA do cartão** ([`card_h`] é espaço de GRAFO e não
/// vê o zoom): a faixa fica reservada sempre. Um cartão que encolhesse ao afastar faria os
/// hit-rects saltarem debaixo do dedo a meio de um pinch.
pub(crate) fn param_text_is_drawn(view: &View) -> bool {
    PARAM_LABEL_SIZE * view.zoom >= MIN_READABLE_PX
}

/// ⭐⭐⭐ **O DETALHE COM QUE UM CARTÃO SE DESENHA** — ordem do dono (2026-09-19): *«se depois
/// disso o zoom continuar a reduzir os nós, o desenho tradicional dos nós se modifica para uma
/// simples cápsula da cor característica do grupo a que o nó pertence, com o nome do nó ocupando
/// toda a cápsula, os parâmetros de ajustes são escondidos e os slots de conexão ficam maiores»*.
///
/// ⚠️ **O limiar é o MESMO do texto, e isso é a lei e não uma economia:** a cápsula existe para o
/// regime em que o texto da row já não se lê. *Dois limiares diferentes dariam uma faixa de zoom
/// em que o cartão é «completo» e não tem nada legível dentro — que é exactamente o «tudo em
/// branco» que o smoke de 2026-09-05 devolveu.*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Detalhe {
    /// O cartão de sempre: cabeçalho, sockets por fileira, faixa de params, readout.
    Completo,
    /// Uma cápsula da cor do grupo, com o NOME a enchê-la e os sockets maiores.
    Capsula,
}

/// **O zoom em que a cápsula começa** — o mesmo limiar do texto, publicado porque o desenho da
/// cápsula precisa dele para parar de encolher os pinos (ver `paint_capsula`).
pub(crate) const ZOOM_DA_CAPSULA: f32 = MIN_READABLE_PX / PARAM_LABEL_SIZE;

/// Ver [`Detalhe`].
pub(crate) fn detalhe(view: &View) -> Detalhe {
    if param_text_is_drawn(view) {
        Detalhe::Completo
    } else {
        Detalhe::Capsula
    }
}

/// **A ALTURA de uma cápsula** — o que os sockets dela precisam, com o piso do cabeçalho.
///
/// ⚠️⚠️ **Ela não é uma constante, e a razão é geométrica:** uma cápsula de altura fixa com três
/// entradas teria de as amontoar, e a `SOCKET_HIT_R` é fixa em píxeis de ECRÃ — três alvos de
/// `9 px` a `4 px` de distância um do outro são um só alvo. Mantendo o passo de [`ROW_H`], o
/// espaçamento dos pinos é **o mesmo** do cartão completo, e o que encolhe é só o que não tem
/// pino: o readout, a faixa de params, o selo.
///
/// ⭐ Para a esmagadora maioria dos nós (uma entrada, uma saída) ela é o próprio [`HEADER_H`] —
/// `190 × 26`, que é a pastilha que o dono desenhou por palavras.
pub(crate) fn capsula_h(n: &GraphNodeView) -> f32 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "contagem de portas cabe num f32"
    )]
    let pinos = n.inputs.len().max(n.outputs.len()) as f32;
    HEADER_H.max(pinos * ROW_H)
}

/// **A altura de um cartão NESTE zoom** — a porta única do [`Detalhe`] para a geometria.
///
/// ⛔⛔ **Ela contradiz uma nota que esta casa escreveu, e a contradição é deliberada:** o
/// [`param_text_is_drawn`] declara que *«o que o LOD NÃO faz é mudar a ALTURA do cartão … um
/// cartão que encolhesse ao afastar faria os hit-rects saltarem debaixo do dedo a meio de um
/// pinch»*. Isso continua verdade DENTRO de cada regime — a altura não segue o zoom
/// continuamente —, e o que a ordem do dono acrescenta é um **degrau**: uma mudança de FORMA, num
/// ponto só, como o cartão dobrado de um subgrafo. *Um salto discreto no limiar é o preço de ter
/// dois desenhos; um salto contínuo seria o defeito que aquela nota nomeia.*
pub(crate) fn card_h_at(n: &GraphNodeView, view: &View) -> f32 {
    match detalhe(view) {
        Detalhe::Completo => super::card_h(n),
        Detalhe::Capsula => capsula_h(n),
    }
}

/// ⭐⭐ **O RAIO COM QUE UM PINO É DESENHADO** — *«os slots de conexão ficam maiores»* (ordem do
/// dono, 2026-09-19).
///
/// ⚠️⚠️ **O número é DERIVADO e não escolhido: o pino PARA de encolher no limiar da cápsula.**
/// Ele é desenhado com o raio que tinha em [`ZOOM_DA_CAPSULA`], logo a `zoom 0,3` sai `2,2×`
/// maior do que sairia. *A alternativa — um multiplicador — daria um pino que volta a encolher ao
/// afastar mais, e isso é precisamente o que esta ordem existe para curar.*
///
/// ⭐ **E ele passa a dizer a verdade sobre o ALVO:** a [`super::SOCKET_HIT_R`] é fixa em píxeis de
/// ECRÃ desde sempre (*«o dot desenhado encolhe com o zoom, mas o alvo não»*) — o desenho é que
/// mentia sobre ela.
pub(crate) fn raio_do_pino(view: &View, raio_base: f32) -> f32 {
    raio_base * view.zoom.max(ZOOM_DA_CAPSULA)
}

/// **UMA ROW SÓ SE AGARRA QUANDO SE PODE MIRAR** — e é o MESMO limiar do texto, de propósito.
///
/// ⚠️ Parecem duas perguntas e são uma: *«esta row está utilizável por um humano agora?»*. Uma
/// fileira de 4 px não se lê **nem** se acerta (a régua do tablet pede 44 pt), e registá-la
/// roubaria ao corpo do cartão o gesto de ARRASTAR O NÓ sem dar nada em troca.
///
/// ⛔⛔ **E elas DIVERGIRAM em 2026-09-19, com a medição ao lado** — o doc acima previa o dia
/// (*«se um dia as duas divergirem, elas separam-se aqui»*), e o gatilho foi a ordem do dono de
/// baixar o piso do TEXTO em 20 %. Ela fala de LER; agarrar é outra pergunta, e a resposta dela
/// não mudou:
///
/// | | limiar | altura de uma row |
/// |---|---:|---:|
/// | o texto LÊ-SE | `zoom ≥ 0,655` | `14,4 px` |
/// | a row AGARRA-SE | `zoom ≥ 0,818` | `18,0 px` |
///
/// ⚠️ **O preço de as manter juntas é um gesto que esta linha acabou de construir:** o corpo de um
/// cartão é o que se ARRASTA para trocar dois nós de lugar ou enfiar um num fio, e cada row
/// agarrável rouba-lhe uma faixa. A `14,4 px` uma fileira já não é um alvo — é uma armadilha entre
/// o artista e o arrasto do cartão. *Ler de longe é grátis; agarrar de longe custa o gesto que
/// está por baixo.*
pub(crate) fn param_row_is_grabbable(view: &View) -> bool {
    PARAM_LABEL_SIZE * view.zoom >= PISO_ANTIGO_PX
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

/// O topo do READOUT, em espaço de grafo — logo abaixo dos params. ⚠️ **Uma porta**: o
/// pintor leria a mesma soma à mão e ficaria uma linha atrás no dia em que a faixa mudasse.
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
