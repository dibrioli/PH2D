//! **FASE 4a — TRAÇAR cada saída até à sua parceira.**
//!
//! Do ponto de partida, na direcção da saída, caminha-se **triângulo a triângulo**,
//! acumulando as transições, até o alvo cair dentro do triângulo corrente. O alvo
//! está a **uma célula** de distância: é o ponto de grade vizinho.
//!
//! ⭐ **A regra de escolha da aresta de saída é o que faz os casos especiais
//! desaparecerem.** Quando o segmento passa exactamente por um vértice, as duas
//! arestas candidatas intersectam-no; escolher a que tem **menos vértices sobre o
//! segmento** faz isolinhas que passam por vértices — e triângulos degenerados numa
//! linha — deixarem de precisar de tratamento próprio.
//!
//! ⛔⛔ **A mudança de orientação tem de ser vista e respondida.** Se ao passar de um
//! triângulo para o seguinte o sinal da área muda, origem e alvo trocam e a direcção
//! inverte-se — sem isso o traço atravessa uma dobra e **sai a andar para trás**.
//!
//! ⛔ **Bordo:** uma aresta com uma face só **aborta** o traço, e a saída fica
//! **pendente**. Saídas pendentes são ignoradas na extracção de células.

use crate::exact::{P, Xf, opposite, orient, step};
use crate::ingest::Topo;
use crate::ports::Ports;

/// O tecto de triângulos que um traço pode atravessar.
///
/// ⚠️ **É um tecto de SANIDADE e não de qualidade.** O alvo está a uma célula de
/// distância e um triângulo mede da ordem de uma célula, então um traço são gasta
/// unidades de passos; um que gaste centenas está a andar em círculo sobre um mapa
/// que não fecha, e o que interessa é que ele **pare e seja contado**.
const MAX_STEPS: usize = 256;

/// O que o traçado mediu de si próprio.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct WalkStats {
    /// Saídas emparelhadas.
    pub linked: usize,
    /// ⚠️ Saídas que morreram no **bordo** — esperado numa malha aberta.
    pub boundary: usize,
    /// ⛔ Saídas que chegaram e **não** acharam a parceira no destino.
    pub orphan: usize,
    /// ⛔ Traços que estouraram o tecto de passos.
    pub runaway: usize,
    /// ⛔⛔ **Traços que chegaram a uma parceira JÁ EMPARELHADA com outra.**
    ///
    /// ⚠️ Acontece onde duas cartas se sobrepõem: dois talos diferentes chegam ao
    /// mesmo ponto de grade pela mesma direcção. *A primeira redacção sobrescrevia a
    /// ligação da outra e deixava o par ANTIGO a apontar para uma saída que já não
    /// apontava de volta* — uma meia-ligação assimétrica, que faz a extracção de
    /// células virar à esquerda para dentro de uma célula alheia. Era daí que saíam as
    /// quatro células de TRÊS lados que o teorema proíbe.
    pub contested: usize,
    /// Passos gastos, somados — a régua do custo.
    pub steps: usize,
    /// Quantas vezes o traço atravessou uma mudança de orientação.
    pub flips: usize,
}

/// ⭐ **O TRAÇADO DE TODAS AS SAÍDAS.**
pub(crate) fn trace_all(topo: &Topo, ports: &mut Ports) -> WalkStats {
    let mut st = WalkStats::default();
    for i in 0..ports.ports.len() {
        if ports.ports[i].link.is_some() {
            continue;
        }
        #[allow(clippy::cast_possible_truncation)]
        match trace_one(topo, ports, i as u32, &mut st) {
            Outcome::Linked(j, acc) => {
                // ⛔ **A parceira já tem dona.** Sobrescrever partiria a ligação dela
                // ao meio: uma meia-ligação sem a outra metade.
                if ports.ports[j as usize].link.is_some() {
                    st.contested += 1;
                    continue;
                }
                ports.ports[i].link = Some(j);
                ports.ports[i].link_xf = acc;
                ports.ports[j as usize].link = Some(i.try_into().unwrap_or(u32::MAX));
                ports.ports[j as usize].link_xf = acc.inverse();
                st.linked += 1;
            }
            Outcome::Boundary => st.boundary += 1,
            Outcome::Orphan => st.orphan += 1,
            Outcome::Runaway => st.runaway += 1,
        }
    }
    st
}

enum Outcome {
    Linked(u32, Xf),
    Boundary,
    Orphan,
    Runaway,
}

fn trace_one(topo: &Topo, ports: &Ports, id: u32, st: &mut WalkStats) -> Outcome {
    let p = ports.ports[id as usize];
    let mut face = p.face as usize;
    let mut o = p.at;
    let mut t = [
        p.at[0] + step(p.dir, topo.one)[0],
        p.at[1] + step(p.dir, topo.one)[1],
    ];
    let mut dir = p.dir;
    let mut acc = Xf::IDENTITY;
    let mut entry: Option<usize> = None;
    for _ in 0..MAX_STEPS {
        st.steps += 1;
        if contains(topo, face, t) {
            #[allow(clippy::cast_possible_truncation)]
            let key = (face as u32, t[0], t[1], opposite(dir));
            return match ports.by_key.get(&key) {
                Some(&j) if j != id => Outcome::Linked(j, acc),
                _ => Outcome::Orphan,
            };
        }
        let Some(k) = exit_side(topo, face, entry, o, t) else {
            return Outcome::Orphan;
        };
        let Some((g, j)) = topo.twin[face][k] else {
            return Outcome::Boundary;
        };
        let x = topo.xf[face][k];
        let before = face_sign(topo, face);
        let after = face_sign(topo, g as usize);
        acc = acc.then(x);
        o = x.apply(o);
        t = x.apply(t);
        dir = x.dir(dir);
        face = g as usize;
        entry = Some(j as usize);
        // ⛔⛔ **A dobra vista pelo SINAL DA ÁREA.** Sem esta troca o traço
        // atravessa a dobra e passa a andar para trás — e o sintoma não é um erro,
        // é uma malha com faces a menos e ninguém a dizer porquê.
        if before != 0 && after != 0 && before != after {
            core::mem::swap(&mut o, &mut t);
            dir = opposite(dir);
            st.flips += 1;
        }
    }
    Outcome::Runaway
}

/// O sinal da área da imagem de uma face.
pub(crate) fn face_sign(topo: &Topo, f: usize) -> i8 {
    let [a, b, c] = topo.uv[f];
    orient(a, b, c)
}

/// O ponto está **dentro ou sobre** o triângulo-imagem?
fn contains(topo: &Topo, f: usize, q: P) -> bool {
    let [a, b, c] = topo.uv[f];
    let s = orient(a, b, c);
    if s == 0 {
        return false;
    }
    let e = [orient(a, b, q), orient(b, c, q), orient(c, a, q)];
    e.iter().all(|&x| x == 0 || x == s)
}

/// ⭐ **A ARESTA POR ONDE O SEGMENTO SAI** — e o desempate que apaga os casos
/// especiais.
fn exit_side(topo: &Topo, f: usize, entry: Option<usize>, o: P, t: P) -> Option<usize> {
    let mut best: Option<(usize, u8)> = None;
    for k in 0..3usize {
        if entry == Some(k) {
            continue;
        }
        let a = topo.uv[f][k];
        let b = topo.uv[f][(k + 1) % 3];
        if !crosses(o, t, a, b) {
            continue;
        }
        let n = u8::from(on_segment(o, t, a)) + u8::from(on_segment(o, t, b));
        if best.is_none_or(|(_, c)| n < c) {
            best = Some((k, n));
        }
    }
    best.map(|(k, _)| k)
}

/// Os dois segmentos tocam-se ou cruzam-se? Fechado nos extremos, de propósito: um
/// segmento que passa por um vértice **atravessa** as duas arestas que ali se
/// encontram, e é o desempate que decide qual delas serve.
fn crosses(o: P, t: P, a: P, b: P) -> bool {
    let (d1, d2) = (orient(o, t, a), orient(o, t, b));
    let (d3, d4) = (orient(a, b, o), orient(a, b, t));
    d1 * d2 <= 0 && d3 * d4 <= 0
}

/// O ponto está no segmento `[o, t]`, extremos incluídos?
fn on_segment(o: P, t: P, q: P) -> bool {
    orient(o, t, q) == 0
        && q[0] >= o[0].min(t[0])
        && q[0] <= o[0].max(t[0])
        && q[1] >= o[1].min(t[1])
        && q[1] <= o[1].max(t[1])
}
