//! ⭐⭐⭐ **A MORDIDA — dissolver doublets, e limpar o arquivo depois.**
//!
//! Irmão de [`crate::cells`] por RESPONSABILIDADE, não por tamanho: aquele módulo
//! responde *«que células fecham?»*, e este responde *«que vértice não devia existir?»*.
//! As duas perguntas cruzam-se num sítio só — o `build` chama [`dissolve_doublets`] e
//! [`compact_verts`] depois de montar as faces.
//!
//! ⛔⛔ **O defeito que este módulo fecha REALIMENTA-SE.** A extracção emitia doublets;
//! o artista carregava a peça outra vez; a fase zero — que só sabe remalhar superfície —
//! rasgava a topologia em cima deles e o `ph2d-gridmap` estourava a jusante. ⇒ *um defeito
//! que a ferramenta emite e não sabe ler é um laço, e fecham-se os DOIS lados*: a saída
//! deixa de o emitir ([`dissolve_doublets`], chamada pelo `build`) **e** a entrada é
//! reparada ([`repair_doublets`], a porta pública que o botão chama).

use ph2d_mesh::{Face, Mesh, MeshError};

/// ⭐⭐⭐ **REPARA UMA MALHA JÁ CONSTRUÍDA** — a mesma lei, para quem chega com a peça pronta.
///
/// ⛔⛔ **A mordida REALIMENTA-SE, e é por isso que esta porta existe.** A saída que o artista
/// exportou em 2026-08-29 tinha `19` doublets, todos em pontas finas; ao voltar a entrar na
/// cadeia, a fase zero — que só sabe remalhar superfície — transforma `χ = 2` em **`χ = 6`
/// com aresta não-manifold**, e a jusante o `ph2d-gridmap` estoura. ⇒ *fechar só o lado da
/// saída deixa toda peça já gravada a partir o botão para sempre.*
///
/// # Errors
/// Nunca — a fusão é exacta e a malha resultante é sempre construível.
pub fn repair_doublets(mesh: &mut Mesh) -> Result<usize, MeshError> {
    let mut faces = mesh.faces().to_vec();
    let n = dissolve_doublets(&mut faces);
    if n > 0 {
        let mut positions = mesh.positions().to_vec();
        compact_verts(&mut positions, &mut faces);
        *mesh = Mesh::from_parts(positions, faces)?;
    }
    Ok(n)
}

/// ⭐⭐⭐ **DEIXA CAIR OS VÉRTICES ÓRFÃOS** — e ela não é arrumação.
///
/// ⛔⛔ **Sem ela a dissolução PARECE preservar a topologia e não preserva a CONTAGEM.** O
/// vértice preso deixa de ser usado por face nenhuma, mas fica no arquivo — e a
/// característica de Euler é `V − E + F` sobre **todos** os vértices, então ela sobe **`1` por
/// mordida**. Medido: dois gates desta crate reprovaram com `χ = 14` contra `2` e `13` contra
/// `1` — *doze órfãos, doze unidades.*
///
/// ⚠️ *«A superfície está certa» e «o ficheiro está certo» são duas afirmações, e a régua
/// mede a segunda.*
pub(crate) fn compact_verts(positions: &mut Vec<[f32; 3]>, faces: &mut [Face]) {
    let mut used = vec![false; positions.len()];
    for f in faces.iter() {
        for &v in f.verts() {
            if let Some(u) = used.get_mut(v as usize) {
                *u = true;
            }
        }
    }
    if used.iter().all(|u| *u) {
        return;
    }
    let mut slot = vec![u32::MAX; positions.len()];
    let mut next = 0u32;
    for (i, u) in used.iter().enumerate() {
        if *u {
            slot[i] = next;
            next += 1;
        }
    }
    let mut kept: Vec<[f32; 3]> = Vec::with_capacity(next as usize);
    for (i, p) in positions.iter().enumerate() {
        if used[i] {
            kept.push(*p);
        }
    }
    *positions = kept;
    for f in faces.iter_mut() {
        let v = f.verts();
        let m: Vec<u32> = v.iter().map(|&x| slot[x as usize]).collect();
        *f = if v.len() == 4 && m.len() == 4 {
            Face::quad(m[0], m[1], m[2], m[3])
        } else {
            Face::tri(m[0], m[1], m[2])
        };
    }
}

/// ⭐⭐⭐ **DISSOLVE OS DOUBLETS** — ver `CellStats::doublets` em [`crate::cells`]. Devolve
/// quantos caíram.
///
/// Um **doublet** é um vértice interior com exactamente **duas** arestas e duas faces; as
/// duas partilham três cantos (`a`, `v`, `b`) e a união delas é um quad. ⭐ Fundi-las é
/// exacto: `V−1`, `E−2`, `F−1`, e `χ` não se mexe.
///
/// ⚠️ **A ORDEM da fusão sai do percurso da fronteira, não de um palpite:** com
/// `Q1 = [a, v, b, p]` e `Q2 = [b, v, a, q]`, apagar as arestas `a–v` e `v–b` deixa
/// `a → q → b → p → a`. *Escrever `[a, p, b, q]` daria o quad com os lados trocados.*
///
/// ⚠️ **E ele corre até assentar:** dissolver um doublet pode criar outro no vizinho.
pub(crate) fn dissolve_doublets(faces: &mut Vec<Face>) -> usize {
    use std::collections::BTreeMap;
    let mut total = 0usize;
    for _ in 0..MAX_DOUBLET_ROUNDS {
        let mut inc: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
        let mut ring: BTreeMap<u32, std::collections::BTreeSet<u32>> = BTreeMap::new();
        for (fi, f) in faces.iter().enumerate() {
            let v = f.verts();
            for k in 0..v.len() {
                inc.entry(v[k]).or_default().push(fi);
                ring.entry(v[k]).or_default().insert(v[(k + 1) % v.len()]);
                ring.entry(v[k])
                    .or_default()
                    .insert(v[(k + v.len() - 1) % v.len()]);
            }
        }
        let mut dead: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        let mut round = 0usize;
        for (&v, fs) in &inc {
            if fs.len() != 2 || ring.get(&v).map_or(0, std::collections::BTreeSet::len) != 2 {
                continue;
            }
            let (f0, f1) = (fs[0], fs[1]);
            if dead.contains(&f0) || dead.contains(&f1) {
                continue;
            }
            let (q0, q1) = (faces[f0].verts().to_vec(), faces[f1].verts().to_vec());
            if q0.len() != 4 || q1.len() != 4 {
                continue;
            }
            let (Some(i), Some(j)) = (
                q0.iter().position(|&x| x == v),
                q1.iter().position(|&x| x == v),
            ) else {
                continue;
            };
            let (a, b, p) = (q0[(i + 3) % 4], q0[(i + 1) % 4], q0[(i + 2) % 4]);
            let (b2, a2, q) = (q1[(j + 3) % 4], q1[(j + 1) % 4], q1[(j + 2) % 4]);
            // ⚠️ **A segunda face tem de ver `a` e `b` ao contrário** — se ela os vê na mesma
            // ordem, as duas não são vizinhas por duas arestas e a fusão inventaria um quad.
            if (a2, b2) != (a, b) {
                continue;
            }
            // ⛔ `p == q` é uma ALMOFADA (duas faces coincidentes) e não um doublet — fundi-la
            // daria um quad com dois cantos iguais. Ver `CellStats::mirrored_cells`.
            if p == q {
                continue;
            }
            faces[f0] = Face::quad(a, q, b, p);
            dead.insert(f1);
            round += 1;
        }
        if round == 0 {
            break;
        }
        let mut keep = 0usize;
        faces.retain(|_| {
            let live = !dead.contains(&keep);
            keep += 1;
            live
        });
        total += round;
    }
    total
}

/// A rede do laço de dissolução — ver [`dissolve_doublets`]. ⚠️ Medido: a peça do artista
/// resolve-se em **uma** ronda; este número é a rede de um caso patológico, não o que manda.
const MAX_DOUBLET_ROUNDS: usize = 8;

#[cfg(test)]
mod doublet_tests {
    use ph2d_mesh::{Face, Mesh};

    /// ⭐⭐⭐ **A MORDIDA DISSOLVE-SE, e a ORDEM da fusão é a do percurso da fronteira.**
    ///
    /// ⛔ A fixtura é o doublet canónico: `v` tem **duas** arestas (`v–a`, `v–b`) e duas
    /// faces, que partilham três cantos. ⚠️ *É a forma exacta dos `19` que a peça do artista
    /// trazia em 2026-08-29, todos em pontas finas* — e é ela que faz a fase zero devolver
    /// `χ = 6` quando aquela peça volta a entrar.
    ///
    /// ⚠️ **O CONTROLE está na asserção da ordem:** apagar `a–v` e `v–b` deixa
    /// `a → q → b → p → a`, e um `[a, p, b, q]` daria os mesmos cantos com os lados trocados
    /// — um quad que se auto-intersecta. *Uma asserção que só contasse faces ficaria verde
    /// sobre ele.*
    #[test]
    fn a_mordida_dissolve_e_a_ordem_e_a_do_percurso() {
        let (a, b, v, p, q) = (0u32, 1, 2, 3, 4);
        let mut mesh = Mesh::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, -1.0, 0.0],
            ],
            vec![Face::quad(a, v, b, p), Face::quad(b, v, a, q)],
        )
        .expect("a fixtura e' construida aqui");
        assert_eq!(mesh.face_count(), 2);

        let n = super::repair_doublets(&mut mesh).expect("a fusao e' exacta");
        assert_eq!(n, 1, "⛔ o doublet tem de ser contado");
        assert_eq!(mesh.face_count(), 1, "⛔ as duas faces fundem-se numa");

        // ⚠️ **A asserção é sobre POSIÇÕES e não sobre índices**: a compactação deixa cair o
        // vértice preso e **renumera** o resto, então um índice esperado seria uma afirmação
        // sobre a arrumação e não sobre a forma. *A pergunta é qual quad saiu, não com que
        // nomes.*
        let at = |i: u32| mesh.positions()[i as usize];
        let got: Vec<[f32; 3]> = mesh.faces()[0].verts().iter().map(|&i| at(i)).collect();
        let want = [
            [0.0f32, 0.0, 0.0], // a
            [1.0, -1.0, 0.0],   // q
            [2.0, 0.0, 0.0],    // b
            [1.0, 1.0, 0.0],    // p
        ];
        let ok = (0..4).any(|r| (0..4).all(|k| got[(k + r) % 4] == want[k]));
        assert!(
            ok,
            "⛔ o quad fundido saiu {got:?} e tinha de ser uma rotacao de {want:?} -- a ordem \
             e' a do percurso da fronteira, e trocar `p` com `q` da' um quad que se \
             auto-intersecta"
        );
        assert_eq!(
            mesh.vert_count(),
            4,
            "⛔ o vertice preso tem de sair do ARQUIVO, e nao so' das faces -- um orfao \
             move a caracteristica de Euler em 1"
        );
        let _ = (v, p, q);
    }

    /// ⛔ **Uma ALMOFADA não é um doublet** — ver `CellStats::mirrored_cells` em
    /// [`crate::cells`].
    ///
    /// ⚠️ Nela os dois quads coincidem, logo `p == q`, e fundi-los daria um quad com **dois
    /// cantos iguais**. *A recusa vive numa linha, e sem este gate ela é invisível.*
    #[test]
    fn uma_almofada_nao_e_dissolvida_como_mordida() {
        let mut mesh = Mesh::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![Face::quad(0, 1, 2, 3), Face::quad(3, 2, 1, 0)],
        )
        .expect("a fixtura e' construida aqui");
        let n = super::repair_doublets(&mut mesh).expect("nao ha' fusao a fazer");
        assert_eq!(
            n, 0,
            "⛔ uma almofada nao se funde -- ela DESCARTA-SE, e noutro sitio"
        );
        assert_eq!(mesh.face_count(), 2);
    }
}
