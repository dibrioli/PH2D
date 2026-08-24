//! ⭐⭐⭐ **FASE 3 — as SAÍDAS, e a ORDEM delas.**
//!
//! De cada nó saem tantas saídas quantas as intersecções das isolinhas inteiras com
//! uma vizinhança infinitesimal da malha ali: **quatro** num nó regular interior,
//! **`v`** numa singularidade de valência `v`.
//!
//! ⛔⛔⛔ **A ordem em que as saídas são guardadas é LOAD-BEARING, e é a ordem
//! HORÁRIA SOBRE A SUPERFÍCIE — nunca no domínio.** É ela que faz *«virar à
//! esquerda»*, na extracção de células, ser simplesmente *«a saída seguinte na
//! lista»*.
//!
//! ⚠️ **Duas coisas quebram a correspondência entre a ordem no domínio e a ordem na
//! superfície, e as duas acontecem:**
//! 1. uma transição não-identidade introduz um **salto** na direcção entre
//!    triângulos vizinhos — é por isso que cada saída carrega a transição para a
//!    carta de referência do nó, e não só a direcção crua;
//! 2. um triângulo **dobrado** (área negativa no domínio) **inverte** a ordem das
//!    suas saídas quando volta à superfície — é por isso que o sentido da varredura
//!    é o sinal da área, e não uma constante.
//!
//! # ⚠️ A fronteira relaxa, e sem isso malha com bordo perde saídas
//!
//! Uma isolinha colinear com um raio do leque pertence a **um** canto só: aquele em
//! que o raio é o de **entrada**. Num leque fechado isso conta cada uma exactamente
//! uma vez. Num leque **aberto** o raio de **saída** do último canto não tem
//! sucessor — e é ali, e só ali, que ele também é emitido.

use crate::exact::{P, Xf, orient, same_sense, side_of_ray};
use crate::fan::{Corner, fan_of, seed_corners};
use crate::ingest::Topo;
use crate::nodes::{Node, Site};

/// Um talo de aresta a sair de um nó, ainda sem par.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Port {
    /// O nó de que ela sai.
    pub node: u32,
    /// O triângulo para onde ela aponta.
    pub face: u32,
    /// A imagem do nó **naquela carta**.
    pub at: P,
    /// A direcção no domínio daquela carta.
    pub dir: u8,
    /// ⭐ A transição que leva a carta desta saída à **carta de referência do nó**
    /// (a da primeira saída da lista).
    ///
    /// ⚠️ **É daqui que sai a transição do LEQUE**, e esquecê-la na acumulação de
    /// uma célula dá coordenadas locais impossíveis: `fan(a → b) = to_ref(a) então
    /// to_ref(b)⁻¹`.
    pub to_ref: Xf,
    /// A saída parceira, quando o traço a encontrou.
    pub link: Option<u32>,
    /// A transição acumulada pelo traço até à parceira.
    pub link_xf: Xf,
}

/// As saídas e a lista horária de cada nó.
pub(crate) struct Ports {
    pub ports: Vec<Port>,
    /// Por nó, as saídas em ordem **horária sobre a superfície**.
    pub of_node: Vec<Vec<u32>>,
    /// `(face, u, v, dir)` → saída. A chave é única: numa carta, um ponto de grade
    /// é vértice, é ponto de aresta ou é interior — nunca dois ao mesmo tempo.
    pub by_key: std::collections::BTreeMap<(u32, i64, i64, u8), u32>,
}

/// ⭐ **A EMISSÃO.**
pub(crate) fn emit(topo: &Topo, nodes: &[Node]) -> Ports {
    let mut ports: Vec<Port> = Vec::new();
    let mut of_node: Vec<Vec<u32>> = vec![Vec::new(); nodes.len()];
    let seeds = seed_corners(topo);

    for (n, node) in nodes.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let n32 = n as u32;
        match node.site {
            Site::Vertex(v) => {
                let Some(seed) = seeds[v as usize] else {
                    continue;
                };
                let fan = fan_of(topo, seed);
                let last = fan.corners.len() - 1;
                let open = fan.holonomy.is_none();
                for (i, c) in fan.corners.iter().enumerate() {
                    let at = topo.uv[c.f()][c.kk()];
                    let (s, e, sigma) = wedge_at_vertex(topo, *c, at);
                    if sigma == 0 {
                        continue;
                    }
                    let include_end = open && i == last;
                    for d in sweep(s, e, sigma, include_end) {
                        push(
                            &mut ports,
                            &mut of_node[n],
                            n32,
                            c.face,
                            at,
                            d,
                            fan.to_here[i].inverse(),
                        );
                    }
                }
            }
            Site::Edge { face, side } => {
                let f = face as usize;
                let k = side as usize;
                let at = node.at;
                let (s, e, sigma) = wedge_at_edge(topo, f, k, at);
                let solo = topo.twin[f][k].is_none();
                if sigma != 0 {
                    for d in sweep(s, e, sigma, solo) {
                        push(&mut ports, &mut of_node[n], n32, face, at, d, Xf::IDENTITY);
                    }
                }
                if let Some((g, j)) = topo.twin[f][k] {
                    let across = topo.xf[f][k];
                    let at_g = across.apply(at);
                    let (s2, e2, sigma2) = wedge_at_edge(topo, g as usize, j as usize, at_g);
                    if sigma2 != 0 {
                        for d in sweep(s2, e2, sigma2, false) {
                            push(
                                &mut ports,
                                &mut of_node[n],
                                n32,
                                g,
                                at_g,
                                d,
                                across.inverse(),
                            );
                        }
                    }
                }
            }
            Site::Face { face } => {
                let f = face as usize;
                let [a, b, c] = topo.uv[f];
                let sigma = orient(a, b, c);
                // ⚠️ **A lista INVERTE-SE numa face dobrada** — é o mesmo facto que
                // o sentido da varredura exprime nos outros dois casos.
                let order: [u8; 4] = if sigma >= 0 {
                    [0, 3, 2, 1]
                } else {
                    [0, 1, 2, 3]
                };
                for d in order {
                    push(
                        &mut ports,
                        &mut of_node[n],
                        n32,
                        face,
                        node.at,
                        d,
                        Xf::IDENTITY,
                    );
                }
            }
        }
    }

    let mut by_key = std::collections::BTreeMap::new();
    for (i, p) in ports.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        by_key.insert((p.face, p.at[0], p.at[1], p.dir), i as u32);
    }
    Ports {
        ports,
        of_node,
        by_key,
    }
}

fn push(
    ports: &mut Vec<Port>,
    list: &mut Vec<u32>,
    node: u32,
    face: u32,
    at: P,
    dir: u8,
    to_ref: Xf,
) {
    #[allow(clippy::cast_possible_truncation)]
    let id = ports.len() as u32;
    ports.push(Port {
        node,
        face,
        at,
        dir,
        to_ref,
        link: None,
        link_xf: Xf::IDENTITY,
    });
    list.push(id);
}

/// A cunha de um canto de vértice: o raio de **entrada**, o de **saída**, e o sinal
/// da área.
///
/// ⚠️ **O raio de entrada é o do terceiro vértice e o de saída é o do seguinte**, e
/// não o contrário: o passo horário do leque atravessa o lado `k`, que é a aresta
/// para o vértice **seguinte** — logo é por ela que se sai.
fn wedge_at_vertex(topo: &Topo, c: Corner, at: P) -> (P, P, i8) {
    let q = topo.uv[c.f()][(c.kk() + 1) % 3];
    let r = topo.uv[c.f()][(c.kk() + 2) % 3];
    let sigma = orient(at, q, r);
    (
        [r[0] - at[0], r[1] - at[1]],
        [q[0] - at[0], q[1] - at[1]],
        sigma,
    )
}

/// A cunha de um nó de aresta dentro de uma face: um **meio-plano**.
///
/// ⚠️ O raio de entrada aponta para o vértice `k` e o de saída para o `k+1` —
/// exactamente a mesma lei do canto de vértice, no limite em que o nó desliza ao
/// longo da aresta.
fn wedge_at_edge(topo: &Topo, f: usize, k: usize, at: P) -> (P, P, i8) {
    let a = topo.uv[f][k];
    let b = topo.uv[f][(k + 1) % 3];
    let c = topo.uv[f][(k + 2) % 3];
    let sigma = orient(a, b, c);
    (
        [a[0] - at[0], a[1] - at[1]],
        [b[0] - at[0], b[1] - at[1]],
        sigma,
    )
}

/// ⭐ **A VARREDURA das quatro cardinais**, no sentido que a área manda.
///
/// ⚠️ **Ela começa numa direcção que NÃO é emitida.** É isso que garante que a
/// corrida saia contígua e na ordem certa: a cunha de um triângulo é sempre menor
/// que meia volta, logo há sempre pelo menos uma cardinal de fora, e as de dentro
/// formam um intervalo.
fn sweep(s: P, e: P, sigma: i8, include_end: bool) -> impl Iterator<Item = u8> {
    let emitted = |d: u8| -> bool {
        let ss = side_of_ray(s, d);
        let se = side_of_ray(e, d);
        // ⚠️ **Os dois raios podem ser COLINEARES** (o meio-plano de um nó de
        // aresta), e aí uma cardinal alinhada com `s` no sentido oposto é a que
        // aponta ao longo de `e`. Sair no primeiro `== 0` daria a resposta do raio
        // errado, e a saída da fronteira desaparecia sem erro nenhum.
        if ss == 0 && same_sense(s, d) {
            return true;
        }
        if se == 0 && same_sense(e, d) {
            return include_end;
        }
        se == sigma && ss == -sigma
    };
    // horário no domínio = índice a DESCER (as cardinais estão em ordem
    // anti-horária); numa face dobrada o horário na superfície é o anti-horário no
    // domínio, e o passo inverte-se.
    let step: u8 = if sigma > 0 { 3 } else { 1 };
    let mut out = [0u8; 4];
    let mut n = 0usize;
    match (0..4u8).find(|&d| !emitted(d)) {
        None => {
            let mut d = 0u8;
            while n < 4 {
                out[n] = d;
                n += 1;
                d = (d + step) & 3;
            }
        }
        Some(start) => {
            let mut d = (start + step) & 3;
            for _ in 0..4 {
                if emitted(d) {
                    out[n] = d;
                    n += 1;
                } else if n > 0 {
                    break;
                }
                d = (d + step) & 3;
            }
        }
    }
    out.into_iter().take(n)
}
