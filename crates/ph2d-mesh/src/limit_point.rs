//! ⭐⭐⭐ **O PONTO-LIMITE DA SUBDIVISÃO** — onde um vértice *vai parar* se a
//! malha for subdividida para sempre, em forma fechada e sem iterar.
//!
//! ⛔⛔ **É a peça de substrato que a `SPEC_unblocked_brushes.md` §2.3 nomeia
//! como a ÚNICA que falta aos dois pincéis de multirresolução**, e a razão é
//! §2.1: `subdivide^k(base)` **não é** a superfície-limite, e a diferença não
//! tende a zero na densidade que um artista usa — ela é o resíduo do esquema, é
//! maior perto de vértices irregulares, e este repo já a regista por escrito
//! (*o limite de um cubo de lado 1 tem meia-extensão `0,4198`, não `0,5`*).
//!
//! ⇒ um apagador que reponha o vértice na PREVISÃO em vez de no LIMITE **encolhe
//! a peça** a cada uso, e o artista lê isso como *«o apagador comeu a forma»*.
//!
//! # ⭐ A lei é o AUTO-VECTOR, e ele é DERIVADO do nosso próprio esquema
//!
//! Para um esquema estacionário, o limite de um vértice é o **auto-vector à
//! esquerda dominante** da matriz de subdivisão local, normalizado a soma `1`.
//! ⚠️ **A espec proíbe copiar uma tabela de pesos por valência** (§2.3) — e com
//! razão: uma LUT envelhece calada no dia em que o nosso esquema mudar um
//! literal. As três máscaras abaixo são escritas **em função dos mesmos pesos
//! que o [`crate::subdivide`] usa**, e ⭐⭐ o gate
//! `o_limite_e_onde_a_subdivisao_de_facto_pousa` **mede-as contra o nosso
//! próprio `subdivide` iterado**: *a fórmula não é citada, é conferida contra o
//! programa que ela descreve.*
//!
//! ⚠️ **Isso é possível por uma propriedade do nosso porte:** o `subdivide`
//! mantém os vértices originais nos índices `0..V`, logo
//! `subdivide^k(malha).positions()[v]` é literalmente a trajectória de `v`.
//!
//! # As três configurações, e a quarta que RECUSA
//!
//! | anel | esquema | máscara |
//! |---|---|---|
//! | só triângulos | **Loop** | `(w·V + Σanel) / (w + n)`, com `w = 3/(8β(n))` |
//! | só quads | **Catmull-Clark** | `(n²·V + 4·Σanel + Σdiagonais) / (n·(n+5))` — ⛔ e a espec dizia *médios* e *centroides*, refutado por medição |
//! | bordo | B-spline cúbica da CORDA | `(P₋ + 4V + P₊) / 6` |
//! | **misto tri/quad** | ⛔ **nenhum** | [`LimitPoint::None`] |
//!
//! ⛔⛔ **O misto RECUSA, e a recusa é a resposta certa.** O nosso `even`
//! interpola os dois esquemas ali com uma média ponderada; essa mistura **não
//! tem limite publicado**, e inventar-lhe um seria pôr o vértice numa superfície
//! que não é a de esquema nenhum — *exactamente o defeito do alvo que a §2.4
//! manda NÃO herdar*. Quem consome trata o `None` como *«este vértice já está
//! onde deve»* e **diz quantos** foram.
//!
//! # ⚠️ A divergência DELIBERADA da §2.4
//!
//! O alvo avalia sempre o limite **suave**, mesmo com a subdivisão em modo
//! simples/linear — defeito público e aberto dele. ⇒ **a nossa lei é *a
//! referência é a do esquema EM VIGOR***, e é por isso que estas máscaras saem
//! dos pesos do nosso `even` e não de uma tabela externa.

use crate::mesh::Mesh;

/// Onde um vértice pousa no limite — ou a declaração de que não há limite
/// publicado para aquele anel.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LimitPoint {
    /// O ponto, em espaço de objecto.
    At([f32; 3]),
    /// ⛔ **Anel MISTO** (triângulos e quads no mesmo vértice) ou não-manifold:
    /// o nosso `even` interpola dois esquemas ali e a mistura não tem limite
    /// publicado. *Inventar um seria pôr o vértice numa superfície que não é a
    /// de esquema nenhum.*
    None,
}

impl LimitPoint {
    /// O ponto, ou `p` quando não há limite publicado — a leitura que um
    /// consumidor de geometria quer (*«fica onde está»*).
    #[must_use]
    pub fn or(self, p: [f32; 3]) -> [f32; 3] {
        match self {
            Self::At(q) => q,
            Self::None => p,
        }
    }

    /// Ver [`Self::None`].
    #[must_use]
    pub fn is_none(self) -> bool {
        self == Self::None
    }
}

/// **O peso de suavização `β(n)` do nosso Loop** — os MESMOS três ramos do
/// [`crate::subdivide`], e é isso que mantém a máscara do limite atada ao
/// esquema em vigor.
///
/// ⚠️ **O `n = 3` é o peso de Warren**, e não a fórmula geral: é o que o nosso
/// porte usa, logo é o que o limite tem de assumir. *Uma máscara de limite
/// derivada de outro `β` descreve outra superfície.*
fn loop_beta(n: usize) -> f32 {
    match n {
        6 => 0.0625,
        3 => 0.1875,
        _ => 0.375 / n as f32,
    }
}

/// §2.3 — **o ponto-limite de um vértice**, em `O(anel)` e sem iterar.
///
/// ⚠️ **Ele lê a MESMA adjacência que o `even` percorre** — é o mesmo passeio,
/// com outra tabela de pesos.
#[must_use]
pub fn limit_point(mesh: &Mesh, v: usize) -> LimitPoint {
    let p = mesh.positions();
    if v >= p.len() {
        return LimitPoint::None;
    }
    let adj = mesh.adjacency();
    let ring = adj.vert_verts.neighbours(v);
    let n = ring.len();
    if n < 2 {
        return LimitPoint::None;
    }

    if adj.is_border(v) {
        return limite_de_bordo(mesh, v, ring);
    }

    let faces = adj.vert_faces.neighbours(v);
    let quads = faces
        .iter()
        .filter(|&&f| !mesh.faces()[f as usize].is_tri())
        .count();

    if quads == 0 {
        // ⭐ **LOOP.** Para `V' = (1 − nβ)V + β·Σanel`, o auto-vector à esquerda
        // dominante é `(w, 1, …, 1)` com `w = 3/(8β)`; normalizado, dá a
        // máscara abaixo. ⚠️ **Conferido contra o nosso `subdivide` iterado**,
        // não copiado — ver o gate deste módulo.
        let beta = loop_beta(n);
        let w = 3.0 / (8.0 * beta);
        let mut acc = [0.0f32; 3];
        soma(&mut acc, p[v], w);
        for &u in ring {
            soma(&mut acc, p[u as usize], 1.0);
        }
        return LimitPoint::At(escala(acc, 1.0 / (w + n as f32)));
    }

    if quads != faces.len() {
        // ⛔ Anel MISTO — ver o cabeçalho.
        return LimitPoint::None;
    }

    // ⭐ **CATMULL-CLARK:** `(n²·V + 4·Σanel + Σdiagonais) / (n·(n+5))`, onde as
    // diagonais são o canto OPOSTO de cada quad incidente — o mesmo termo que o
    // nosso `even` já percorre para separar CC de Loop. Para `n = 4` isto é o
    // estêncil clássico `(16, 4, 1)/36`.
    //
    // ⛔⛔ **A ESPEC §2.3 ESTÁ REFUTADA NESTE PONTO, e por MEDIÇÃO.** Ela escreve
    // a mesma fórmula com *«ΣE a soma dos pontos médios das arestas do anel e ΣF
    // a soma dos centroides das faces incidentes»*. Escrita assim, o vértice de
    // canto de `cube(1.0)` pousa em **`0,375`** — e o nosso `subdivide` iterado
    // sete vezes pousa em **`0,250`**, que é exactamente o que a forma do anel
    // dá. *Uma espec atestada afirma o que o revisor podia ver, e ninguém tinha
    // corrido a fórmula contra um esquema.*
    //
    // ⚠️ **Elas não são a mesma máscara escrita de outra maneira:** os pontos
    // médios e os centroides são combinações afins dos MESMOS vértices, mas com
    // outros pesos — a diferença aparece inteira já no caso regular.
    let nf = n as f32;
    let mut acc = [0.0f32; 3];
    soma(&mut acc, p[v], nf * nf);
    for &u in ring {
        soma(&mut acc, p[u as usize], 4.0);
    }
    for &f in faces {
        let q = mesh.faces()[f as usize].verts();
        let i = q.iter().position(|&x| x as usize == v).unwrap_or(0);
        soma(&mut acc, p[q[(i + 2) % 4] as usize], 1.0);
    }
    LimitPoint::At(escala(acc, 1.0 / (nf * (nf + 5.0))))
}

/// **O limite de um vértice de BORDO** — o da curva B-spline cúbica da corda,
/// `(P₋ + 4V + P₊)/6`, e ⭐ **independente do interior**.
///
/// ⚠️ **É a mesma lei que o nosso `even` já obedece ali**: um vértice de borda
/// ouve SÓ a borda. Se ele ouvisse o anel de dentro, a boca da peça seria sugada
/// para o miolo — e o limite herdaria essa sucção.
fn limite_de_bordo(mesh: &Mesh, v: usize, ring: &[u32]) -> LimitPoint {
    let p = mesh.positions();
    let adj = mesh.adjacency();
    let edges = mesh.edges();
    let borda: Vec<u32> = ring
        .iter()
        .copied()
        .filter(|&w| {
            edges
                .id_of(adj, v as u32, w)
                .is_some_and(|e| edges.valence(e) == 1)
        })
        .collect();
    // ⛔ Sem DOIS vizinhos de borda não há corda a seguir — o mesmo desistir do
    // `even`, e pela mesma razão.
    if borda.len() != 2 {
        return LimitPoint::None;
    }
    let mut acc = [0.0f32; 3];
    soma(&mut acc, p[v], 4.0);
    soma(&mut acc, p[borda[0] as usize], 1.0);
    soma(&mut acc, p[borda[1] as usize], 1.0);
    LimitPoint::At(escala(acc, 1.0 / 6.0))
}

fn soma(acc: &mut [f32; 3], p: [f32; 3], w: f32) {
    for k in 0..3 {
        acc[k] = p[k].mul_add(w, acc[k]);
    }
}

fn escala(p: [f32; 3], s: f32) -> [f32; 3] {
    [p[0] * s, p[1] * s, p[2] * s]
}

#[cfg(test)]
#[path = "limit_point_tests.rs"]
mod tests;
