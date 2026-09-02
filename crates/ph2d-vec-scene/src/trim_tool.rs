//! ⭐⭐⭐ **A ferramenta TRIM** (plano 38) — *"deleta segmentos entre pontos ou entre linhas
//! sobrepostas"* (Enio, 2026-08-31).
//!
//! # A lei
//!
//! > **O pedaço sob o cursor é a extensão de caminho entre as duas FRONTEIRAS mais próximas, uma de
//! > cada lado. Clicar apaga-o.**
//!
//! **Fronteira** é qualquer uma destas quatro, numa lista só ([`boundaries`]):
//!
//! 1. um **cruzamento** com outro caminho visível;
//! 2. um **auto-cruzamento** do próprio caminho;
//! 3. um **nó** (âncora) do próprio contorno;
//! 4. uma **ponta aberta**.
//!
//! É a regra do Fusion 360 — *"trims to the nearest **crossing or node**"* — e o *"or node"* é
//! exactamente o *"entre pontos"* do pedido.
//!
//! ⚠️⚠️ **E ela cura de graça a queixa nº 1 do Fusion.** Lá, um círculo sem cruzamentos é apagado
//! INTEIRO (*"I seem to only be able to delete entire circle sketches"*) — porque um círculo do
//! Fusion **não tem nós**. Aqui um `ShapeKind::Ellipse` cozinha âncoras, então clicar entre duas
//! tira **um quarto**. *A regra é a mesma; a diferença está no substrato, e é a nosso favor.*
//!
//! ⛔ **O modo de DOIS PASSOS é uma recusa medida por outra pessoa.** A Autodesk trocou o `TRIM` por
//! omissão em **2021**, de *"escolha as arestas de corte primeiro"* para *"tudo corta, clique no
//! pedaço"*, e deixou o antigo atrás de uma variável (`TRIMEXTENDMODE = 0`). Nascemos no rápido.
//!
//! # O que sobra
//!
//! | o contorno | o pedaço | o resultado |
//! |---|---|---|
//! | aberto | no MEIO | **dois** |
//! | aberto | numa PONTA | **um**, mais curto |
//! | fechado | qualquer | **um**, agora aberto |
//! | qualquer | a peça toda | **nenhum** ⇒ o caminho é apagado |
//!
//! ⚠️ **Os pedaços ficam no MESMO `VecPath`, como contornos dele** — e não em caminhos separados.
//! É o que preserva a cor, a largura, o tracejado, os efeitos e o `Transform` sem uma linha de
//! código, e o que mantém **um** passo de undo. ⛔ A alternativa (dois objectos) é uma decisão de
//! produto: ela obriga a duplicar o estilo e a pose, e a escolher qual dos dois herda o id.

use crate::arc_cut::{EPS, Geom, strands_of};
use crate::{Contour, VecPath, VecVertex};

/// **AS FRONTEIRAS de um contorno**, em fracção de arco `0..=1`, ordenadas e sem repetidas.
///
/// `cruzamentos` são as fracções que o chamador já achou contra as OUTRAS geometrias (e contra este
/// mesmo contorno); os **nós** e as **pontas** saem daqui, porque são factos da geometria e não da
/// cena.
///
/// ⚠️ **Uma lista só, e é isso que faz um cruzamento VENCER um nó quando está mais perto** — que é
/// o caso *"entre linhas sobrepostas"* do pedido. Duas listas obrigariam quem escolhe o pedaço a
/// arbitrar entre elas, e a arbitragem seria uma segunda lei.
#[must_use]
pub fn boundaries(verts: &[VecVertex], closed: bool, cruzamentos: &[f64]) -> Vec<f64> {
    let Some(g) = Geom::of(verts, closed) else {
        return Vec::new();
    };
    let mut out: Vec<f64> = Vec::with_capacity(g.lens.len() + cruzamentos.len() + 1);
    // Os NÓS: a fracção acumulada antes de cada segmento. Num aberto, o `0` é a ponta de partida e
    // o `1` (acrescentado no fim) é a de chegada — as duas pontas são fronteira pela mesma linha.
    let mut andado = 0.0;
    for &l in &g.lens {
        out.push(andado / g.total);
        andado += l;
    }
    if !closed {
        out.push(1.0);
    }
    out.extend(cruzamentos.iter().copied().filter(|f| f.is_finite()));
    out.sort_by(f64::total_cmp);
    out.dedup_by(|a, b| (*a - *b).abs() < EPS);
    out
}

/// **O PEDAÇO sob o cursor** — `(de, até)` em fracção de arco. `None` quando não há fronteira
/// nenhuma (um contorno degenerado).
///
/// Num contorno **FECHADO** o intervalo pode dar a volta pela emenda (`até < de`), e é por isso que
/// ele não é normalizado aqui: quem corta ([`crate::arc_cut::strands_of`]) já sabe ler a volta.
///
/// ⚠️ Num contorno **ABERTO** com uma fronteira só de um lado, o pedaço vai até à ponta — e quando
/// as fronteiras são as duas pontas (uma reta de um segmento, sem cruzamento), o pedaço é a peça
/// toda. *Apagar a peça toda é a resposta certa e é a do Fusion*: uma reta que não cruza nada não
/// tem meio para aparar.
#[must_use]
pub fn piece_at(fronteiras: &[f64], closed: bool, cursor: f64) -> Option<(f64, f64)> {
    if fronteiras.is_empty() {
        return None;
    }
    let c = cursor.clamp(0.0, 1.0);
    // ⚠️⚠️ **O CURSOR EM CIMA DE UMA FRONTEIRA** (achado pela sonda do report de 31/08): com uma
    // folga simétrica, `antes` e `depois` acham a MESMA e o pedaço nasce de largura ZERO — o
    // realce não aparece e o clique não faz nada, que lê como *"a ferramenta não pega aqui"*.
    // A folga é para TRÁS num lado e para a FRENTE no outro: o pedaço passa a ser sempre o que
    // está **à frente** do cursor, e um cursor em cima de um nó escolhe o lado seguinte.
    let antes = fronteiras.iter().rev().find(|&&f| f <= c + EPS).copied();
    let depois = fronteiras.iter().find(|&&f| f > c + EPS).copied();
    if closed {
        // A volta: sem fronteira antes, a anterior é a ÚLTIMA (pela emenda); sem fronteira depois, a
        // seguinte é a PRIMEIRA. Com uma fronteira só, o pedaço é a volta inteira.
        let de = antes.or_else(|| fronteiras.last().copied())?;
        let ate = depois.or_else(|| fronteiras.first().copied())?;
        return Some((de, ate));
    }
    Some((antes.unwrap_or(0.0), depois.unwrap_or(1.0)))
}

/// **CORTA o pedaço `(de, até)` do contorno `k`** do caminho e devolve o caminho que sobra.
///
/// `None` = **não sobrou geometria nenhuma** e o chamador tem de apagar o caminho (é a peça toda,
/// e é a resposta do Fusion).
///
/// ⚠️ **Os contornos que não são o `k` viajam intactos** — um compound perde só a fita em que se
/// carregou. E a ordem é preservada, com o primeiro sobrevivente a virar o contorno primário: o
/// `VecPath` guarda um contorno em `verts`/`closed` e os outros em `subpaths`, então *alguém* tem de
/// ser o primário e a escolha estável é a ordem.
#[must_use]
pub fn sever(path: &VecPath, k: usize, de: f64, ate: f64) -> Option<VecPath> {
    let mut contornos: Vec<Contour> = Vec::with_capacity(path.subpaths.len() + 2);
    for (i, c) in contours_of(path).enumerate() {
        if i != k {
            contornos.push(c);
            continue;
        }
        let Some(g) = Geom::of(&c.verts, c.closed) else {
            continue; // degenerado: some, como sumiria de qualquer corte
        };
        for (verts, closed) in strands_of(&g, &[(de, ate)]) {
            contornos.push(Contour { verts, closed });
        }
    }
    contornos.retain(|c| c.verts.len() >= 2);
    let mut it = contornos.into_iter();
    let primeiro = it.next()?;
    let mut out = path.clone();
    out.verts = primeiro.verts;
    out.closed = primeiro.closed;
    out.subpaths = it.collect();
    Some(out)
}

/// Os contornos de um caminho, na ordem canónica: o primário e depois os `subpaths`.
///
/// ⚠️ **É o índice desta ordem que o [`sever`] recebe**, e é o mesmo que o hit-test devolve — uma
/// numeração só, senão o realce acende numa fita e o corte come outra.
pub fn contours_of(path: &VecPath) -> impl Iterator<Item = Contour> + '_ {
    std::iter::once(Contour {
        verts: path.verts.clone(),
        closed: path.closed,
    })
    .chain(path.subpaths.iter().cloned())
}

/// **A FRACÇÃO DE ARCO do ponto do contorno mais próximo de `p`**, com a distância até ele.
/// `None` num contorno degenerado.
///
/// ⚠️ **Mede sobre a poligonal de detecção** (`Geom::edges`), que é a MESMA que acha os
/// cruzamentos: o realce e as fronteiras têm de falar da mesma curva amostrada, senão o pedaço
/// acende num sítio e é cortado noutro por um erro de amostragem.
#[must_use]
pub fn nearest_fraction(verts: &[VecVertex], closed: bool, p: [f64; 2]) -> Option<(f64, f64)> {
    let g = Geom::of(verts, closed)?;
    let mut melhor: Option<(f64, f64)> = None;
    for e in g.edges() {
        let d = [e.p1[0] - e.p0[0], e.p1[1] - e.p0[1]];
        let ll = d[0] * d[0] + d[1] * d[1];
        let t = if ll <= EPS {
            0.0
        } else {
            (((p[0] - e.p0[0]) * d[0] + (p[1] - e.p0[1]) * d[1]) / ll).clamp(0.0, 1.0)
        };
        let q = [e.p0[0] + d[0] * t, e.p0[1] + d[1] * t];
        let dist = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt();
        if melhor.is_none_or(|(_, m)| dist < m) {
            melhor = Some((e.f0 + (e.f1 - e.f0) * t, dist));
        }
    }
    melhor
}

/// **AS FRACÇÕES em que este contorno é atravessado** — por qualquer um dos `outros` e por **ele
/// próprio**.
///
/// `escala` é o tamanho de referência da cena (a diagonal da caixa, por exemplo): duas travessias
/// mais próximas que uma fracção dela são a MESMA, e sem isso um cruzamento raso a ângulo pequeno
/// entra duas vezes e o pedaço nasce com largura zero.
#[must_use]
pub fn crossings_against(
    verts: &[VecVertex],
    closed: bool,
    outros: &[(Vec<VecVertex>, bool)],
    escala: f64,
) -> Vec<f64> {
    let Some(alvo) = Geom::of(verts, closed) else {
        return Vec::new();
    };
    let mut geoms = vec![alvo];
    geoms.extend(outros.iter().filter_map(|(v, c)| Geom::of(v, *c)));
    let edges: Vec<Vec<_>> = geoms.iter().map(Geom::edges).collect();
    if edges.iter().map(Vec::len).sum::<usize>() > crate::arc_cut::MAX_SAMPLES {
        // ⛔ Caminho patológico: sem fronteiras de cruzamento. Os NÓS continuam a valer, então a
        // ferramenta degrada para *"apara entre pontos"* em vez de parar — e isso é uma resposta.
        return Vec::new();
    }
    let mut out: Vec<f64> = Vec::new();
    for c in crate::arc_cut::crossings(&geoms, &edges, escala) {
        // O contorno `0` é o alvo; uma travessia dele consigo mesmo entra pelos DOIS lados.
        if c.a.0 == 0 {
            out.push(c.a.1);
        }
        if c.b.0 == 0 {
            out.push(c.b.1);
        }
    }
    out.extend(touches(&geoms[0], outros, escala));
    out
}

/// ⭐⭐⭐ **A FOLGA DO TOQUE**, como fracção da escala da arte.
///
/// # ⛔⛔ Por que a flecha sozinha não chega (report do Enio, 2026-09-01/02)
///
/// A régua natural é o **erro de amostragem do alvo** — um ponto que está sobre a curva pode estar a
/// até isso da poligonal que a representa. ⚠️ **Mas ela é ZERO numa RECTA**: a corda É a curva. Uma
/// curva que termina sobre uma aresta recta tinha de a tocar **ao bit** para a fronteira existir, e
/// meio pixel de folga fazia a região abrir. *Antes de 01/09 a flecha era calculada contra o ponto
/// médio da corda e devolvia `0,55` numa recta de 100 — errado, e era esse erro que segurava as
/// junções em «T».*
///
/// ⚠️ **Fracção da diagonal da arte, e MINÚSCULA de propósito** (`1e-3` — `0,4` num desenho de 400).
/// Ela perdoa o tremor da mão e o resíduo de vírgula flutuante, e **não fecha um vão que se veja**.
/// ⛔ Não é a largura do traço: essa foi construída, shipada e **revertida** — ela move geometria e
/// fecha vãos que o artista quer abertos (plano 40 §7).
pub const TOUCH_FRACTION: f64 = 1e-3;

/// A folga com que uma ponta conta como pousada em `alvo` — ver [`TOUCH_FRACTION`].
fn touch_tol(alvo: &Geom, escala: f64) -> f64 {
    alvo.sampling_error().max(escala * TOUCH_FRACTION)
}

/// ⭐⭐⭐ **TODOS OS CRUZAMENTOS de uma lista de contornos, distribuídos por contorno.**
///
/// ⛔⛔ **Existe porque perguntar por contorno é `O(n³)`**, e o tecto de amostras transforma isso
/// num **penhasco mudo**: o [`crossings_against`] soma as arestas do alvo **mais as de todos os
/// outros**, então numa lista de `n` contornos o mesmo total é somado `n` vezes e comparado com o
/// mesmo tecto. Medido em círculos que se cruzam: a `64` deles (4 096 arestas) a resposta é certa e
/// custa **764 ms**; a `65` (4 160) **todos os cruzamentos desaparecem** e cada forma volta a ser um
/// anel inteiro — o preenchimento salta de `2 235` para `7 844` de área, sem um aviso.
///
/// Aqui as arestas são construídas **uma vez**, o motor corre **uma vez**, e o tecto é comparado
/// **uma vez** — com o número que a medição deu.
/// ⭐⭐⭐ **O TECTO DA PASSAGEM ÚNICA**, em arestas de amostragem — MEDIDO, não escolhido.
///
/// | arestas | montar a rede |
/// |---|---|
/// | 4 096 | `11,9 ms` |
/// | 6 144 | `25,7 ms` |
/// | 8 192 | `45,5 ms` |
/// | **12 288** | **`102,0 ms`** |
/// | 16 384 | `180,6 ms` |
/// | 32 768 | `715,4 ms` |
///
/// ⚠️ **É outro orçamento que não o do [`crate::arc_cut::MAX_SAMPLES`]**, e é por isso que são dois
/// números: aquele serve o Trim, que pergunta **por quadro** (16,7 ms); este serve o balde, que
/// monta a rede **quando o desenho muda** — um soluço de 100 ms ali é o que a mão sente como
/// «pensou um instante», e é o mesmo critério de morte que o balde do Flip usa.
///
/// `12 288` arestas são **768 segmentos** no documento inteiro.
pub const MAX_SAMPLES_BATCH: usize = 12_288;

/// ⚠️ **Devolve `None` quando o documento passa do tecto — e isso é uma RECUSA, não um vazio.**
///
/// ⛔⛔ A resposta antiga era devolver **zero cruzamentos**, e ela é a pior possível: sem
/// cruzamentos toda forma volta a ser um anel inteiro, e o preenchimento **salta para a forma toda**
/// em vez de desaparecer. *Uma resposta errada em silêncio é pior que nenhuma resposta* — medido: a
/// `64` círculos a lente mede `2 235`; a `65`, `7 844`.
#[must_use]
pub fn crossings_all(contornos: &[(Vec<VecVertex>, bool)], escala: f64) -> Option<Vec<Vec<f64>>> {
    let mut out: Vec<Vec<f64>> = vec![Vec::new(); contornos.len()];
    // ⚠️ Um contorno degenerado não entra no motor, mas continua a ocupar o índice de saída: quem
    // chama indexa pela posição na lista que passou.
    let mut donos: Vec<usize> = Vec::new();
    let mut geoms: Vec<Geom> = Vec::new();
    for (i, (v, c)) in contornos.iter().enumerate() {
        if let Some(g) = Geom::of(v, *c) {
            donos.push(i);
            geoms.push(g);
        }
    }
    let edges: Vec<Vec<crate::arc_cut::Edge>> = geoms.iter().map(Geom::edges).collect();
    if edges.iter().map(Vec::len).sum::<usize>() > MAX_SAMPLES_BATCH {
        return None;
    }
    for c in crate::arc_cut::crossings(&geoms, &edges, escala) {
        out[donos[c.a.0]].push(c.a.1);
        out[donos[c.b.0]].push(c.b.1);
    }
    // Os TOQUES: a ponta de um contorno ABERTO que pousa sobre outro.
    for (k, g) in geoms.iter().enumerate() {
        let tol = touch_tol(g, escala);
        for (j, (verts, closed)) in contornos.iter().enumerate() {
            if j == donos[k] || *closed || verts.len() < 2 {
                continue;
            }
            for ponta in [verts[0].anchor, verts[verts.len() - 1].anchor] {
                if let Some((frac, dist)) = nearest_fraction(&g.verts, g.closed, ponta)
                    && dist <= tol
                {
                    out[donos[k]].push(frac);
                }
            }
        }
    }
    Some(out)
}

/// ⭐⭐⭐ **OS TOQUES: as pontas de OUTROS contornos que POUSAM sobre este.**
///
/// # O report que isto existe para curar (Enio, 2026-08-31, com foto)
///
/// *"Os pontos do círculo cortado estão sobre o outro círculo, mas este outro não reconhece os
/// pontos … e desse modo não me permite deletar o segmento entre os pontos."*
///
/// ⚠️⚠️ **Uma ponta que TERMINA sobre a outra curva não é um CRUZAMENTO.** Um cruzamento é a
/// travessia de duas cordas que continuam para os dois lados; um arco aparado **acaba** ali, e não
/// há segunda metade para atravessar. Medido no report: depois de aparar um dos círculos, o outro
/// ficava com **UMA** fronteira onde tinha duas.
///
/// ⚠️ **A tolerância é MEDIDA, não escolhida**: é o erro de amostragem do próprio contorno
/// ([`Geom::sampling_error`]). Um ponto que está sobre a curva pode estar a até isso da poligonal
/// que a representa — no caso do report, `0,0323` contra uma flecha de `0,12`.
///
/// ⛔ **Só as PONTAS, e só de contornos ABERTOS.** Uma âncora interior de outra curva que pouse
/// aqui já é apanhada pelo cruzamento (a curva dela continua para os dois lados), e um contorno
/// fechado não tem ponta. *Alargar isto a toda âncora poria fronteira em cada vizinho que passa
/// perto, e o pedaço encolheria até ao ruído.*
fn touches(alvo: &Geom, outros: &[(Vec<VecVertex>, bool)], escala: f64) -> Vec<f64> {
    let tol = touch_tol(alvo, escala);
    let mut out = Vec::new();
    for (verts, closed) in outros {
        if *closed || verts.len() < 2 {
            continue;
        }
        for ponta in [verts[0].anchor, verts[verts.len() - 1].anchor] {
            if let Some((frac, dist)) = nearest_fraction(&alvo.verts, alvo.closed, ponta)
                && dist <= tol
            {
                out.push(frac);
            }
        }
    }
    out
}

/// **O ERRO DA AMOSTRAGEM deste contorno** — a maior distância entre a curva verdadeira e a corda
/// que a poligonal de detecção usa no lugar dela.
///
/// É a régua de *"este ponto está SOBRE a curva"*, e é medida em vez de escolhida: a flecha cresce
/// com o raio e com o ângulo de cada segmento.
#[must_use]
pub fn sampling_error(verts: &[VecVertex], closed: bool) -> f64 {
    Geom::of(verts, closed).map_or(0.0, |g| g.sampling_error())
}

/// **A GEOMETRIA DO PEDAÇO** — o que vai ser apagado, para se poder DESENHAR antes de o ser.
///
/// ⚠️⚠️ **É a mesma porta que o [`sever`] usa, do outro lado.** O corte fica com o COMPLEMENTO de
/// `(de, até)` e o realce fica com `(de, até)`; construir o realce por uma segunda conta seria a
/// forma clássica de acender uma coisa e apagar outra. *O que se vê e o que some têm de sair da
/// mesma resposta.*
#[must_use]
pub fn piece_geometry(
    verts: &[VecVertex],
    closed: bool,
    de: f64,
    ate: f64,
) -> Option<Vec<VecVertex>> {
    let g = Geom::of(verts, closed)?;
    // ⛔⛔ **A VOLTA PELA EMENDA.** O `pieces_between` quer `hi > lo` — num contorno fechado uma
    // faixa que atravessa a costura escreve-se `hi > 1`, e não `hi < lo`. Sem esta linha um pedaço
    // que passa pela emenda devolvia **nada**: o realce do Trim apagava-se enquanto o corte
    // (que vai por `strands_of`, e esse normaliza) continuava a comer o pedaço certo.
    // ⚠️ *É exactamente a divergência «acende uma coisa e apaga outra» que o gate desta wave
    // proíbe* — e ele não a apanhou porque as duas portas só discordam sobre a emenda, e nenhuma
    // fixtura tinha um pedaço que passasse por lá. Achada pelo gate de SOLDAR, três horas depois.
    // ⚠️ **`<=` e não `<`**: com UMA fronteira só, o `piece_at` devolve `(f, f)` — e isso significa
    // *a volta inteira*, não um pedaço vazio (é o que o doc dele promete e o que um anel cortado num
    // ponto é). Com `<`, aquele caso devolvia zero arcos.
    let ate = if closed && ate <= de { ate + 1.0 } else { ate };
    let pedacos = crate::fx_trim::pieces_between(&g.segs, &g.lens, g.total, de, ate, closed);
    let v = crate::fx_trim::rebuild(&g.verts, &g.segs, &pedacos, g.n);
    (v.len() >= 2).then_some(v)
}

#[cfg(test)]
#[path = "trim_tool_tests.rs"]
mod tests;
