//! **A REVERSÃO** — reconstruir o nível de BAIXO de uma malha que já É uma
//! subdivisão.
//!
//! Adaptado de `reference/sculptgl/src/editing/Reversion.js`, MIT — ver
//! `LICENSES/sculptgl-MIT.txt`.
//!
//! ⚠️ **Isto NÃO é desfazer.** O nome do arquivo original engana: `Reversion` é
//! *reverse subdivision*, o gesto que dá um nível de multiresolução a uma malha
//! que chegou pronta. Um OBJ denso importado tem UM nível, então o artista pode
//! esculpir a pele e **não pode mover a forma grande** — a
//! [`crate::Multires`] só sabe acrescentar níveis para CIMA. Reverter é o
//! único jeito de haver um nível embaixo.
//!
//! # A pergunta, e por que ela tem resposta
//!
//! Uma subdivisão parte os vértices em dois conjuntos: os **PARES** (as imagens
//! dos vértices grossos) e os **ÍMPARES** (um por aresta grossa, mais um por
//! quad grosso). Reverter é descobrir qual vértice é qual — e a chave é que
//! **todo vértice ímpar é REGULAR por construção**: um ponto de aresta de malha
//! de triângulos nasce com valência 6, um de quads com 4. Logo, todo vértice
//! *extraordinário* é par, e é dele que a varredura parte.
//!
//! Depois disso a propagação é local: os vizinhos de um par são todos ímpares, e
//! atravessando cada ímpar chega-se ou ao próximo PAR (um vizinho grosso) ou ao
//! **ponto de face** — que se distingue por tocar **dois** ímpares do mesmo anel
//! em vez de um.
//!
//! # As posições grossas são CÓPIAS, e isso está certo
//!
//! A posição de um vértice par depois da subdivisão é uma média ponderada dos
//! vizinhos grossos, então copiá-la de volta **não** é o inverso aritmético de
//! Catmull-Clark — é uma decimação. O original faz o mesmo, e o modelo não perde
//! nada: quem restitui a forma exata é o **detalhe** que a
//! [`crate::Multires`] computa em seguida, e ele é exato ao bit (é o invariante
//! daquele módulo). Uma inversa "de verdade" resolveria um sistema linear para
//! chegar a uma base cuja subdivisão é a malha fina — e o detalhe ficaria zero
//! em vez de pequeno, o que muda o custo e não muda a tela.
//!
//! # A RENUMERAÇÃO é a metade que o original resolve de outro jeito
//!
//! A [`crate::Multires`] inteira se apoia em *o vértice `i` de baixo É o vértice
//! `i` de cima* — a numeração que a [`crate::subdivide`] impõe. Os vértices
//! pares de uma malha importada estão **espalhados**, então a malha grossa
//! reconstruída não subdivide para a numeração que a fina já tem.
//!
//! O SculptGL guarda um `vertexMapUp` e faz todo consumidor honrá-lo. Nós
//! devolvemos a **permutação** ([`Reversed::renumber`]) e o chamador
//! RENUMERA a malha fina uma vez — porque um mapa persistente seria uma segunda
//! numeração viva, e a numeração é a coisa de que este módulo inteiro depende.
//!
//! # Recusar é o modo de falha, e ele é ESTRUTURAL
//!
//! Nem toda malha é uma subdivisão. Em vez de confiar na etiquetagem, a
//! reconstrução **verifica**: todo quad tem exatamente um canto par, todo
//! triângulo tem zero ou um, e a renumeração tem de sair uma **bijeção** de
//! `0..V`. Qualquer uma que falhe devolve `None` — e devolver `None` é a
//! diferença entre *"esta malha não é uma subdivisão"* e uma malha grossa
//! inventada que o artista descobriria três gestos depois.

use crate::adjacency::Adjacency;
use crate::edges::Edges;
use crate::face::{Face, TRI};
use crate::mesh::Mesh;

/// O nível de baixo, **mais a renumeração que ele impõe ao de cima**.
///
/// ⚠️ As duas metades viajam juntas de propósito: a malha grossa sozinha é
/// inútil (ela não subdivide para a fina que existe), e a permutação sozinha não
/// descreve nada. Separá-las seria oferecer meia resposta a quem tem de usar as
/// duas no mesmo passo.
#[derive(Clone, Debug)]
pub struct Reversed {
    coarse: Mesh,
    renumber: Vec<u32>,
}

impl Reversed {
    /// A malha do nível de baixo.
    #[must_use]
    pub fn coarse(&self) -> &Mesh {
        &self.coarse
    }

    /// `renumber[j]` = de onde vem, na malha fina **como ela está hoje**, o
    /// vértice `j` da numeração que `subdivide(coarse)` impõe.
    ///
    /// É uma bijeção de `0..V_fina` sobre si mesma — verificada, não assumida.
    #[must_use]
    pub fn renumber(&self) -> &[u32] {
        &self.renumber
    }

    /// Desmonta em `(malha grossa, renumeração)`.
    #[must_use]
    pub fn into_parts(self) -> (Mesh, Vec<u32>) {
        (self.coarse, self.renumber)
    }
}

const UNTAGGED: i8 = 0;
const EVEN: i8 = 1;
const ODD: i8 = -1;

/// **Reverte uma subdivisão.** `None` se esta malha não é uma.
///
/// Ver o cabeçalho do módulo: o resultado é a malha grossa MAIS a permutação que
/// a malha fina tem de sofrer para que `subdivide(grossa)` a numere.
#[must_use]
pub fn reverse_subdivision(fine: &Mesh) -> Option<Reversed> {
    // Toda subdivisão multiplica as faces por quatro. É a recusa mais barata que
    // existe, e o original abre com ela pelo mesmo motivo.
    if fine.face_count() == 0 || !fine.face_count().is_multiple_of(4) {
        return None;
    }
    let tags = tag_parity(fine)?;
    let adj = fine.adjacency();
    let edges = fine.edges();
    let ef = EdgeFaces::build(fine.faces(), &edges);

    // O índice grosso de cada par, em ordem CRESCENTE de índice fino: uma ordem
    // que é função da malha, e não da ordem em que a varredura tropeçou neles.
    let mut coarse_of = vec![u32::MAX; fine.vert_count()];
    let mut evens: Vec<u32> = Vec::new();
    for (v, &t) in tags.iter().enumerate() {
        if t == EVEN {
            coarse_of[v] = u32::try_from(evens.len()).ok()?;
            evens.push(u32::try_from(v).ok()?);
        }
    }
    if evens.is_empty() {
        return None;
    }

    let mut cfaces: Vec<Face> = Vec::new();
    // Por face grossa e por canto, o vértice fino que é o ponto de ARESTA dali.
    let mut corner_mid: Vec<[u32; 4]> = Vec::new();
    // Por face grossa, o ponto de FACE — [`TRI`] num triângulo, que não tem.
    let mut centre: Vec<u32> = Vec::new();

    let mut seen_centre = vec![false; fine.vert_count()];
    for (fi, face) in fine.faces().iter().enumerate() {
        if face.is_tri() {
            // Só o triângulo do MEIO abre uma face grossa; os três das quinas
            // são alcançados a partir dele. Um triângulo com DOIS pares não é
            // subdivisão de nada, e a contagem é o que recusa.
            match even_count(face, &tags) {
                0 => {}
                1 => continue,
                _ => return None,
            }
            let (verts, mids) = coarse_tri(fine, &edges, &ef, &tags, &coarse_of, fi, face)?;
            cfaces.push(Face::tri(verts[0], verts[1], verts[2]));
            corner_mid.push([mids[0], mids[1], mids[2], TRI]);
            centre.push(TRI);
        } else {
            let p = sole_even_corner(face, &tags)?;
            let c = face.0[(p + 2) % 4];
            if seen_centre[c as usize] {
                continue;
            }
            seen_centre[c as usize] = true;
            let (verts, mids) = coarse_quad(fine, adj, &edges, &ef, &tags, &coarse_of, fi, p, c)?;
            cfaces.push(Face::quad(verts[0], verts[1], verts[2], verts[3]));
            corner_mid.push(mids);
            centre.push(c);
        }
    }

    let cpos: Vec<[f32; 3]> = evens
        .iter()
        .map(|&v| fine.positions()[v as usize])
        .collect();
    let mut coarse = Mesh::from_parts(cpos, cfaces).ok()?;
    // Os canais seguem os vértices que os carregavam. Um canal que a malha fina
    // não tem NÃO nasce aqui: materializá-lo custaria 12 B/vértice por um plano
    // que ninguém pediu (a lei do `colors_mut`).
    if let Some(src) = fine.colors() {
        let dst = coarse.colors_mut();
        for (i, &v) in evens.iter().enumerate() {
            dst[i] = src[v as usize];
        }
    }
    if let Some(src) = fine.masks() {
        let dst = coarse.masks_mut();
        for (i, &v) in evens.iter().enumerate() {
            dst[i] = src[v as usize];
        }
    }

    let renumber = build_renumber(&coarse, &corner_mid, &centre, &evens, fine.vert_count())?;
    Some(Reversed { coarse, renumber })
}

/// A numeração que `subdivide(coarse)` impõe, resolvida contra a malha fina.
///
/// ⚠️ **A ordem dos três blocos É o contrato do [`crate::subdivide`]** — os
/// pares, depois um por ARESTA na numeração do grafo, depois um por QUAD na
/// ordem das faces. Trocar qualquer um deles aqui não falha: produz uma malha
/// que sobe e desce embaralhada.
fn build_renumber(
    coarse: &Mesh,
    corner_mid: &[[u32; 4]],
    centre: &[u32],
    evens: &[u32],
    fine_verts: usize,
) -> Option<Vec<u32>> {
    let cedges = coarse.edges();
    let vc = evens.len();
    let ec = cedges.len();
    let quads = coarse.faces().iter().filter(|f| !f.is_tri()).count();
    // A contagem tem de fechar ANTES de qualquer escrita: `V + E + Q` é
    // exatamente o que a subdivisão produz, e um total diferente já diz que esta
    // malha não é uma.
    if vc + ec + quads != fine_verts {
        return None;
    }
    let mut map = vec![u32::MAX; fine_verts];
    map[..vc].copy_from_slice(evens);
    for ((f, cf), mids) in coarse.faces().iter().enumerate().zip(corner_mid) {
        for (k, &want) in mids.iter().take(cf.vert_count()).enumerate() {
            let e = cedges.face_edge(f, k)? as usize;
            let slot = vc + e;
            // Duas faces grossas que compartilham uma aresta TÊM de concordar
            // sobre qual vértice fino é o ponto dela. Discordar é a assinatura
            // de uma etiquetagem que fechou por acaso.
            if map[slot] != u32::MAX && map[slot] != want {
                return None;
            }
            map[slot] = want;
        }
    }
    let mut next = vc + ec;
    for (f, cf) in coarse.faces().iter().enumerate() {
        if !cf.is_tri() {
            map[next] = centre[f];
            next += 1;
        }
    }
    // ⚠️ **A bijeção é uma REDE sobre entrada não-confiável, e ela é uma defesa
    // em camada que nenhuma fixture de hoje distingue** — a mutação que a
    // neutraliza sobrevive aos vinte gates deste módulo, e isso foi MEDIDO, não
    // suposto. O motivo é estrutural: um BURACO é impossível (todo slot de
    // aresta é escrito por alguma face, e toda aresta vem de uma face), e um
    // ÍNDICE REPETIDO exige uma topologia adversária que as formas malformadas
    // do `shapes_open` não produzem — as três que passam pelas checagens
    // anteriores revertem corretamente. Ela fica porque um OBJ de terceiro é
    // entrada arbitrária e o custo de um mapa não-bijetivo não é um erro: é uma
    // malha permutada com vértices duplicados, em silêncio. É o mesmo
    // raciocínio do `Mesh::from_parts` validar índices.
    let mut seen = vec![false; fine_verts];
    for &o in &map {
        let o = o as usize;
        if o >= fine_verts || seen[o] {
            return None;
        }
        seen[o] = true;
    }
    Some(map)
}

/// Quantos cantos desta face são pares.
fn even_count(face: &Face, tags: &[i8]) -> usize {
    face.verts()
        .iter()
        .filter(|&&v| tags[v as usize] == EVEN)
        .count()
}

/// A posição do ÚNICO canto par — `None` se não houver exatamente um.
fn sole_even_corner(face: &Face, tags: &[i8]) -> Option<usize> {
    let mut found = None;
    for (k, &v) in face.verts().iter().enumerate() {
        if tags[v as usize] == EVEN {
            if found.is_some() {
                return None;
            }
            found = Some(k);
        }
    }
    found
}

/// A face grossa de um QUAD: os quatro pares em volta do ponto de face `c`, na
/// ordem em que a volta os encontra.
///
/// ⚠️ **A volta é pela aresta `(m, c)`**, e é ela que dá o *winding*: o quad
/// `(v, m, c, m')` de saída partilha essa aresta com `(m, v', m'', c)`, cujo par
/// é o próximo canto grosso. Ordenar por outra coisa — o ângulo, o índice —
/// produziria um quad de mesmos cantos e face virada.
#[allow(clippy::too_many_arguments)]
fn coarse_quad(
    fine: &Mesh,
    adj: &Adjacency,
    edges: &Edges,
    ef: &EdgeFaces,
    tags: &[i8],
    coarse_of: &[u32],
    start: usize,
    start_corner: usize,
    c: u32,
) -> Option<([u32; 4], [u32; 4])> {
    let mut verts = [0u32; 4];
    let mut mids = [0u32; 4];
    let mut cur_f = start;
    let mut cur_p = start_corner;
    for k in 0..4 {
        let f = fine.faces()[cur_f];
        if f.is_tri() || f.0[(cur_p + 2) % 4] != c {
            return None;
        }
        let ev = f.0[cur_p];
        verts[k] = *coarse_of.get(ev as usize).filter(|&&i| i != u32::MAX)?;
        let m = f.0[(cur_p + 1) % 4];
        mids[k] = m;
        let e = edges.id_of(adj, m, c)?;
        let next = ef.other(e, u32::try_from(cur_f).ok()?)? as usize;
        cur_p = sole_even_corner(&fine.faces()[next], tags)?;
        cur_f = next;
    }
    // Quatro passos e de volta ao começo: um quad grosso tem quatro filhos, e
    // uma volta que fecha em outro número descreve outra topologia.
    if cur_f != start {
        return None;
    }
    Some((verts, mids))
}

/// A face grossa de um TRIÂNGULO, a partir do filho do MEIO.
///
/// ⚠️ **O ponto de aresta do canto `k` é o vértice `k + 1` do triângulo do
/// meio**, não o `k`. O filho do meio é `(m01, m12, m20)` e a aresta grossa que
/// sai do canto `k` termina no canto `k + 1`, cujo ponto médio é `m` seguinte —
/// escrever `k` aqui gira a atribuição de aresta em um passo e a subdivisão
/// seguinte põe cada ponto médio no lugar do vizinho.
fn coarse_tri(
    fine: &Mesh,
    edges: &Edges,
    ef: &EdgeFaces,
    tags: &[i8],
    coarse_of: &[u32],
    centre_face: usize,
    face: &Face,
) -> Option<([u32; 3], [u32; 3])> {
    let mut verts = [0u32; 3];
    let mut mids = [0u32; 3];
    for k in 0..3 {
        let e = edges.face_edge(centre_face, k)?;
        let nf = ef.other(e, u32::try_from(centre_face).ok()?)? as usize;
        let nface = fine.faces()[nf];
        if !nface.is_tri() {
            return None;
        }
        let p = sole_even_corner(&nface, tags)?;
        let ev = nface.0[p];
        verts[k] = *coarse_of.get(ev as usize).filter(|&&i| i != u32::MAX)?;
        mids[k] = face.verts()[(k + 1) % 3];
    }
    Some((verts, mids))
}

/// Etiqueta cada vértice como par ou ímpar. `None` quando a propagação se
/// contradiz — dois pares vizinhos, que nenhuma subdivisão produz.
fn tag_parity(fine: &Mesh) -> Option<Vec<i8>> {
    let adj = fine.adjacency();
    let n = fine.vert_count();
    let mut tags = vec![UNTAGGED; n];
    // O carimbo de "está no anel ímpar DESTA rodada". Um `u32` por vértice em
    // vez de limpar um `bool` por rodada — o padrão do `TAG_FLAG` do original,
    // e o que o mantém `O(anel)` em vez de `O(malha)` por vértice visitado.
    let mut in_ring = vec![u32::MAX; n];
    let mut stack: Vec<u32> = Vec::new();
    let mut round: u32 = 0;
    while let Some(seed) = pick_seed(fine, adj, &tags) {
        tags[seed] = EVEN;
        stack.push(u32::try_from(seed).ok()?);
        while let Some(cur) = stack.pop() {
            round = round.checked_add(1)?;
            let ring = adj.vert_verts.neighbours(cur as usize);
            for &o in ring {
                // Dois pares vizinhos: numa subdivisão os pares formam um
                // conjunto independente, então isto é a prova de que a malha não
                // é uma — ou de que o seed era ímpar.
                if tags[o as usize] == EVEN {
                    return None;
                }
                tags[o as usize] = ODD;
                in_ring[o as usize] = round;
            }
            for &o in ring {
                for &w in adj.vert_verts.neighbours(o as usize) {
                    if w == cur || in_ring[w as usize] == round || tags[w as usize] != UNTAGGED {
                        continue;
                    }
                    // ⚠️ **É AQUI que par e ponto-de-face se separam.** O ponto
                    // de face de um quad toca DOIS pontos de aresta do mesmo
                    // anel (os dois lados da face); um vizinho grosso toca só o
                    // ponto da aresta que os liga. O mesmo vale no triângulo: o
                    // ponto médio oposto toca dois, o vizinho toca um.
                    let shared = adj
                        .vert_verts
                        .neighbours(w as usize)
                        .iter()
                        .filter(|&&x| in_ring[x as usize] == round)
                        .count();
                    if shared >= 2 {
                        tags[w as usize] = ODD;
                    } else {
                        tags[w as usize] = EVEN;
                        stack.push(w);
                    }
                }
            }
        }
    }
    Some(tags)
}

/// De onde a varredura parte, por componente conexa.
///
/// ⚠️ **A primeira escolha é um vértice EXTRAORDINÁRIO, e ela não é heurística:**
/// todo vértice ímpar de uma subdivisão é regular por construção, então um
/// extraordinário é par com certeza. Sem extraordinário nenhum — uma grade
/// perfeita — qualquer vértice interior serve: as duas partições possíveis são
/// as duas reversões válidas, e escolher uma delas não é errar.
fn pick_seed(fine: &Mesh, adj: &Adjacency, tags: &[i8]) -> Option<usize> {
    let mut first_interior = None;
    let mut first_any = None;
    for (v, &tag) in tags.iter().enumerate() {
        if tag != UNTAGGED {
            continue;
        }
        if !is_regular(fine, adj, v) {
            return Some(v);
        }
        if first_any.is_none() {
            first_any = Some(v);
        }
        if first_interior.is_none() && !adj.is_border(v) {
            first_interior = Some(v);
        }
    }
    first_interior.or(first_any)
}

/// A valência que um vértice ÍMPAR tem, por construção da subdivisão.
///
/// Adaptado de `detectExtraordinaryVertices`: 6 no interior de triângulos, 4 no
/// de quads, 5 na costura entre os dois; na borda, um a menos por não haver o
/// outro lado.
fn is_regular(fine: &Mesh, adj: &Adjacency, v: usize) -> bool {
    let faces = adj.vert_faces.neighbours(v);
    if faces.is_empty() {
        return false;
    }
    let quads = faces
        .iter()
        .filter(|&&f| !fine.faces()[f as usize].is_tri())
        .count();
    let val = adj.valence(v);
    let border = adj.is_border(v);
    if quads == 0 {
        if border { val == 4 } else { val == 6 }
    } else if quads == faces.len() {
        if border { val == 3 } else { val == 4 }
    } else {
        !border && val == 5
    }
}

/// As faces que usam cada aresta, em CSR.
///
/// ⚠️ Não vive na [`Edges`] pelo mesmo motivo que ela não vive na [`Mesh`]: hoje
/// tem **um** consumidor, que roda quando o artista aperta um botão. O dia em
/// que houver um por-frame ele sobe, com a medição ao lado.
struct EdgeFaces {
    start: Vec<u32>,
    faces: Vec<u32>,
}

impl EdgeFaces {
    fn build(faces: &[Face], edges: &Edges) -> Self {
        let mut start = vec![0u32; edges.len() + 1];
        for e in 0..edges.len() {
            start[e + 1] = start[e] + edges.valence(u32::try_from(e).unwrap_or(u32::MAX));
        }
        let mut fill = start.clone();
        let mut out = vec![u32::MAX; start[edges.len()] as usize];
        for (f, face) in faces.iter().enumerate() {
            for k in 0..face.vert_count() {
                let Some(e) = edges.face_edge(f, k) else {
                    continue;
                };
                let slot = &mut fill[e as usize];
                out[*slot as usize] = u32::try_from(f).unwrap_or(u32::MAX);
                *slot += 1;
            }
        }
        Self { start, faces: out }
    }

    /// A OUTRA face desta aresta. `None` numa borda (uma face só) ou num
    /// não-manifold (três ou mais) — os dois são recusas honestas: nenhuma
    /// subdivisão põe o interior de uma face numa aresta que não tem dois lados.
    fn other(&self, e: u32, f: u32) -> Option<u32> {
        let s = self.start[e as usize] as usize;
        let t = self.start[e as usize + 1] as usize;
        if t - s != 2 {
            return None;
        }
        let (a, b) = (self.faces[s], self.faces[s + 1]);
        if a == f {
            Some(b)
        } else if b == f {
            Some(a)
        } else {
            None
        }
    }
}

#[cfg(test)]
#[path = "reversion_tests.rs"]
mod tests;
