//! ADR-0114 §4.B — **o domínio SEGMENT do Edit Mode** (o 3º domínio do GP,
//! `02_referencia §11`), módulo-irmão de [`crate::stroke`].
//!
//! Um *segmento* (aqui: **pedaço**) é o trecho de um traço entre dois **cortes**, e um
//! corte é o lugar onde outro traço do quadro **cruza** este — por isso o modo é *"corte
//! por interseção VISUAL"*. Clicar num pedaço acende o pedaço inteiro.
//!
//! **O domínio do DADO continua sendo o Point** (`FlipStroke::point_sel`, W8): o Segment
//! não inventa um 3º vetor, ele é uma **política de pick** que traduz um clique num
//! CONJUNTO de pontos. É o que o `§11` cravou (*"segment→Point + pós-processo"*), e o
//! Blender faz igual: o gesto produz uma máscara de pontos e o modo a **expande** para os
//! pedaços tocados (`apply_mask_as_segment_selection`).
//!
//! # A representação: o vetor de DONOS (e por que ele apaga dois casos especiais)
//!
//! A saída de [`piece_of_point`] é `dono[p]` = o id do pedaço a que o ponto `p` pertence.
//! Os dois casos que o `§11` nomeia como armadilhas somem sozinhos nessa forma:
//!
//! - **cíclica sem corte** = todo mundo com dono `0` ⇒ UM pedaço = a curva toda. O
//!   *"fallback"* não é um `if`, é o caso geral (e é o caso comum do balde: o balde produz
//!   traço fechado, e sem o fallback clicar numa forma fechada intacta não acenderia nada).
//! - **o pedaço que enrola** (o último de uma cíclica) aparece como **duas corridas** no
//!   vetor (`[1,1,0,0]` ⇒ o pedaço 1 é `{0,1}`). Os *"DOIS ranges"* do `§11` são o mesmo
//!   pedaço visto num vetor linear — não há o que costurar.
//!
//! # A divergência DELIBERADA da referência (uma verruga do Blender, com gate)
//!
//! O Blender monta os pedaços com `IndexRange`s e, no último de uma cíclica, passa o ponto
//! de partida por `clamp_range(points, p + 1)` — que **satura em `last()`** em vez de
//! **enrolar para `0`**. Quando o corte cai na **costura** (o segmento `n-1 → 0`), isso
//! atribui o último ponto ao pedaço ERRADO: num quadrado cortado nos segmentos 1 e 3, o
//! Blender devolve `{2}` e `{0,1,3}` onde a geometria manda `{2,3}` e `{0,1}` — o ponto 3
//! está ANTES do corte em `mid(3→0)`, logo é do primeiro pedaço.
//!
//! **Aqui esse sítio não existe** — e é essa a razão de a verruga não nos alcançar. A
//! saturação é consequência da REPRESENTAÇÃO: quem descreve um pedaço por um `IndexRange`
//! precisa de um `begin` que caiba no vetor, e `n-1 + 1` não cabe. O vetor de donos não
//! constrói range nenhum (cada ponto simplesmente *aponta* o pedaço dele), então não há o
//! que saturar. O gate `a_cut_on_the_seam_keeps_the_seam_point_in_the_piece_before_it`
//! **não guarda um `if` deste arquivo: guarda a ESCOLHA de representação** — se um dia
//! alguém reescrever isto em ranges, ele fica vermelho antes de o artista ver o ponto
//! saltar de pedaço. (Ele morre com a mutação `last_cut` ignorando a costura, que o gate do
//! wrap não pega — as duas cíclicas cortam em lugares diferentes de propósito.)
//!
//! Fora essa quina, [`piece_of_point`] foi **conferido caso a caso** contra o
//! `foreach_curve_segment` da referência (aberto/fechado, corte em `λ=0`, corte no ponto 0,
//! corte único) — ver `segment_tests.rs`.
//!
//! # Custo: **não há BVH aqui, e é de propósito** (medido, não chutado)
//!
//! O `§11` manda cortar contra um *BVH 2D*, e o [`cuts`] é O(segmentos do traço × segmentos
//! do quadro) — o mesmo O(N²) que a própria referência carrega com um `TODO` em cima
//! (`grease_pencil_segments_geom.cc`: *"This method of finding intersections is O(N^2) and
//! should be replaced with something faster"*). Medido em `--release` num quadro de
//! **2940 segmentos** (60 traços × 49 — um quadro de line-art generoso):
//!
//! | gesto | custo |
//! |---|---|
//! | **um clique** (1 traço × o quadro) | **211 µs** |
//! | marquee, se cortasse TODOS os traços | 12,6 ms |
//!
//! O clique é o caminho quente e ele é imperceptível — nada disto roda por frame, só no
//! gesto. E o marquee não paga o N²: quem o aplica pergunta *"a caixa tocou este traço?"*
//! **antes** de pedir os cortes (`flip_select_segment::apply_marquee_segments`), e uma
//! caixa toca poucos traços.
//!
//! Então o BVH resolveria um problema que não existe: ele custaria construção por gesto,
//! uma estrutura a mais para manter, e um 2º lugar onde a pergunta *"quem cruza quem?"*
//! poderia responder diferente do laço direto. Se um dia um quadro de verdade doer, o
//! número acima é o baseline contra o qual medir — **não construa o BVH sem ele**.

use std::ops::Range;

use ph2d_core::Vec2;

/// Um **segmento-cortador**: `(a, b)`, em espaço de OBJETO (o espaço comum das camadas —
/// ver `flip_select_segment` no shell).
///
/// Por que objeto e não tela: o corte é uma pergunta de **cruzamento**, e cruzamento é
/// invariante afim — o mesmo par de segmentos que se cruza na tela se cruza no objeto, e a
/// **fração λ do corte é a mesma nos dois** (um afim preserva razões ao longo de uma reta).
/// O Blender projeta em tela porque as camadas dele são 3D; as nossas são um afim 2D do
/// mesmo objeto, então a projeção seria trabalho (e erro de arredondamento) por nada.
pub type Cutter = (Vec2, Vec2);

/// **Os CORTES de um traço**: `cuts[k]` = a fração `λ ∈ [0,1]` do cruzamento **mais
/// próximo** no segmento que começa no ponto `k`, ou `None` se nada o cruza. O
/// comprimento é o nº de segmentos do traço (`n-1` aberto, `n` fechado — a convenção de
/// [`crate::FlipStroke::segments`], costura inclusa).
///
/// - `cutters` = TODOS os segmentos visíveis do quadro, em espaço de objeto;
/// - `own` = a fatia de `cutters` que são os segmentos DESTE traço, **na ordem de
///   `segments()`** (é o `tree_data_range` da referência).
///
/// **Ignora os 3 vizinhos** (o anterior, ele mesmo, o próximo — dentro do MESMO traço):
/// sem isso todo segmento cruzaria os vizinhos nas pontas que ele compartilha com eles, e
/// o traço inteiro viraria corte. Segmentos de OUTROS traços nunca são ignorados — e o
/// próprio traço **corta a si mesmo** nos não-vizinhos, que é o que faz um "8" ter pedaços.
///
/// **Só o corte mais próximo por segmento** — é a limitação da referência (o raycast dela
/// guarda um `hit` só), herdada de propósito: dois traços cruzando o MESMO segmento de
/// polilinha produzem UM corte. Numa polilinha densa (o traço da caneta) isso é invisível;
/// numa esparsa (um retângulo de 4 pontos cruzado 2× na mesma aresta) o 2º cruzamento não
/// corta. Está gateado (`only_the_nearest_cut_of_a_segment_is_kept`) para que a próxima
/// linha saiba que é uma decisão, não um esquecimento.
#[must_use]
pub fn cuts(cutters: &[Cutter], own: Range<usize>, closed: bool) -> Vec<Option<f32>> {
    let m = own.len();
    (0..m)
        .map(|k| {
            let (a, b) = cutters[own.start + k];
            // Os 3 vizinhos, na numeração LOCAL do traço (o `%` só existe no fechado: num
            // aberto o segmento -1 e o segmento m não existem, e é isso que as pontas são).
            let prev = if closed {
                Some((k + m - 1) % m)
            } else {
                k.checked_sub(1)
            };
            let next = if closed {
                Some((k + 1) % m)
            } else {
                (k + 1 < m).then_some(k + 1)
            };
            let mut best: Option<f32> = None;
            for (j, &(c, d)) in cutters.iter().enumerate() {
                if own.contains(&j) {
                    let local = j - own.start;
                    if local == k || Some(local) == prev || Some(local) == next {
                        continue;
                    }
                }
                if let Some(lambda) = seg_isect_lambda(a, b, c, d)
                    && best.is_none_or(|x| lambda < x)
                {
                    best = Some(lambda);
                }
            }
            best
        })
        .collect()
}

/// O cruzamento de `a→b` com `c→d`: a fração ao longo de **`a→b`**, ou `None`.
///
/// Paralelos/colineares **não cruzam** (o `kind <= 0` da referência): duas linhas que se
/// sobrepõem não têm um ponto de corte, têm um trecho — e cortar num deles seria escolher
/// arbitrariamente. Encostar de PONTA conta (o `EXACT` da referência): é por isso que os 3
/// vizinhos precisam ser ignorados.
fn seg_isect_lambda(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<f32> {
    let r = b - a;
    let s = d - c;
    let denom = r.x * s.y - r.y * s.x;
    if denom == 0.0 {
        return None; // paralelo ou colinear
    }
    let q = c - a;
    let t = (q.x * s.y - q.y * s.x) / denom;
    let u = (q.x * r.y - q.y * r.x) / denom;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some(t)
}

/// **A que PEDAÇO cada ponto pertence** — `dono[p]`, o mapa completo do domínio Segment.
///
/// `cuts` vem de [`cuts`]; `n_points` e `closed` são do traço. Um ponto pertence ao pedaço
/// aberto pelo **último corte at-or-before ele** (ciclicamente, no fechado). Um corte em
/// `λ = 0` cai EM CIMA do ponto `k` e portanto o **inclui** (é o *"start point with zero
/// fraction is included"* da referência); em `λ > 0` o corte fica entre `k` e `k+1`, e o
/// pedaço novo começa em `k+1`.
///
/// Sem corte algum ⇒ tudo `0` (a curva toda, um pedaço só — o fallback do `§11`).
#[must_use]
pub fn piece_of_point(cuts: &[Option<f32>], n_points: usize, closed: bool) -> Vec<u32> {
    let mut owner = vec![0u32; n_points];
    if n_points == 0 {
        return owner;
    }
    // O id do pedaço que cada corte ABRE. Num traço ABERTO a ponta é um corte implícito
    // (o traço começa), e ela fica com o id 0 — daí os cortes começarem em 1. Num FECHADO
    // não há ponta: quem abre o 1º pedaço é o 1º corte.
    let head = u32::from(!closed);
    let mut id_of_cut = vec![0u32; cuts.len()];
    let mut next_id = head;
    let mut last_cut: Option<usize> = None;
    for (k, c) in cuts.iter().enumerate() {
        if c.is_some() {
            id_of_cut[k] = next_id;
            next_id += 1;
            last_cut = Some(k);
        }
    }
    // O dono do ponto 0. Aberto: a cabeça. Fechado: andando para TRÁS a partir do 0 dá a
    // volta e cai no ÚLTIMO corte — e é exatamente isso que faz o pedaço final enrolar.
    let mut cur = if closed {
        last_cut.map_or(0, |k| id_of_cut[k])
    } else {
        0
    };
    for (p, o) in owner.iter_mut().enumerate() {
        let cut = cuts.get(p).copied().flatten();
        // Corte EM CIMA do ponto: ele já é do pedaço novo.
        if cut == Some(0.0) {
            cur = id_of_cut[p];
        }
        *o = cur;
        // Corte DEPOIS do ponto: quem muda de pedaço é o `p+1` em diante.
        if cut.is_some_and(|l| l > 0.0) {
            cur = id_of_cut[p];
        }
    }
    owner
}

/// **O ponto-SONDA de um clique**: o clique caiu no segmento `i` à fração `t` — que ponto
/// representa o pedaço em que ele caiu? (Depois é só `dono[sonda]`.)
///
/// Um segmento pode ter um corte no meio, e aí o clique está de um lado OU do outro: além
/// do corte (`t > λ`, com `λ > 0`) o pedaço é o que o corte abriu, cujo 1º ponto inteiro é
/// `i+1` (enrolando no fechado). Nos demais casos o clique está no pedaço que já era dono
/// do ponto `i` — inclusive quando `λ = 0`, porque aí o corte está em cima do `i` e o `i`
/// **já é** do pedaço novo (o `dono` cuida disso).
#[must_use]
pub fn probe_point(cuts: &[Option<f32>], n_points: usize, i: usize, t: f32) -> usize {
    debug_assert!(n_points > 0, "sonda num traco vazio");
    match cuts.get(i).copied().flatten() {
        Some(lambda) if lambda > 0.0 && t > lambda => (i + 1) % n_points,
        _ => i,
    }
}

#[cfg(test)]
#[path = "segment_tests.rs"]
mod tests;
