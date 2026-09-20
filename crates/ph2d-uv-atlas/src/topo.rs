//! ⭐ **A TOPOLOGIA DO ATLAS** — cantos, triângulos e vizinhos, numa porta só.
//!
//! ⚠️ **A W1 escreveu o leque de triangulação DUAS vezes na sonda** (o pintor e o
//! contador de texels) e as duas concordavam por acaso. *Uma lei escrita em dois sítios
//! ainda não é uma lei — só uma PORTA é*, e a partir daqui ela tem quatro consumidores: a
//! régua da sobreposição, o corte, o desenho e a contagem.
//!
//! ⛔ **O leque parte do canto `0` e isso é a definição desta casa**, não uma escolha de
//! quem desenha: um quad partido pela outra diagonal daria outros dois triângulos, com
//! outra área e outra sobreposição. Quem quiser mudar a diagonal muda-a **aqui**, e as
//! quatro respostas mudam juntas.

use crate::bases_dos_cantos;
use ph2d_mesh::Mesh;

/// Por canto, o **vértice da malha** a que ele pertence.
///
/// ⚠️ É a metade que falta ao [`crate::Atlas`]: ele diz o `(u, v)` de cada canto e não
/// diz de que vértice ele veio — e sem isso não há como perguntar *«estes dois
/// triângulos são vizinhos na PEÇA?»*, que é a pergunta que separa uma dobra do mapa de
/// duas partes distantes a caírem no mesmo sítio.
#[must_use]
pub fn vertices_dos_cantos(mesh: &Mesh) -> Vec<u32> {
    let mut v = Vec::with_capacity(bases_dos_cantos(mesh).1);
    for f in mesh.faces() {
        v.extend_from_slice(f.verts());
    }
    v
}

/// Os triângulos do atlas, cada um com os **três índices de canto**.
///
/// Uma face de `n` lados dá `n − 2` triângulos em leque a partir do canto `0`.
#[must_use]
pub fn triangulos(mesh: &Mesh) -> Vec<[u32; 3]> {
    let (base, _) = bases_dos_cantos(mesh);
    let mut out = Vec::new();
    for (f, face) in mesh.faces().iter().enumerate() {
        let n = face.verts().len();
        if n < 3 {
            continue;
        }
        let b = base[f];
        for k in 1..(n - 1) {
            out.push([
                b,
                b + u32::try_from(k).unwrap_or(0),
                b + u32::try_from(k + 1).unwrap_or(0),
            ]);
        }
    }
    out
}

/// Por triângulo, a **face da malha** de onde ele saiu.
#[must_use]
pub fn faces_dos_triangulos(mesh: &Mesh) -> Vec<u32> {
    let mut out = Vec::new();
    for (f, face) in mesh.faces().iter().enumerate() {
        let n = face.verts().len();
        if n < 3 {
            continue;
        }
        for _ in 1..(n - 1) {
            out.push(u32::try_from(f).unwrap_or(0));
        }
    }
    out
}

/// Duas metades de uma mesma pergunta sobre um par de triângulos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elo {
    /// Os dois triângulos partilham uma **aresta da malha**.
    pub na_peca: bool,
    /// ⭐ …e o `(u, v)` concorda nas DUAS pontas dessa aresta — ou seja, no atlas eles
    /// estão **colados**, não apenas encostados.
    ///
    /// ⚠️ A diferença é a costura: dois triângulos de lados opostos de um corte
    /// partilham a aresta da peça e vivem em sítios diferentes do atlas.
    pub no_atlas: bool,
}

/// Por aresta da malha, os `(triângulo, canto A, canto B)` que a formam.
type PorAresta = std::collections::BTreeMap<(u32, u32), Vec<(u32, u32, u32)>>;

/// ⚠️ **A tolerância do [`Elo::no_atlas`], como FRACÇÃO do comprimento da aresta.**
///
/// ⛔ Ela é uma CONSTANTE desta porta e não um argumento, porque tinha três chamadores —
/// o corte, a régua da sobreposição e os gates — e *três respostas à mesma pergunta
/// divergem no dia em que alguém afina uma*. O valor está duas ordens de grandeza acima
/// da dispersão medida nas costuras das peças do dono (`cola_max` `2,4e-3` a `9,7e-3`
/// células, sobre arestas de ~1 célula).
pub const TOLERANCIA_DO_ELO: f32 = 1.0e-2;

/// ⭐⭐ **Quem faz fronteira com quem**, por aresta da malha.
///
/// Devolve, para cada par de triângulos que partilha uma aresta da peça, o par ordenado
/// `(a, b)` com `a < b` e o [`Elo`] entre eles.
///
/// ⚠️ **A adjacência sai da MALHA e nunca de soldar UVs iguais.** Soldar por posição no
/// plano juntaria duas partes distantes que o mapa dobrou uma em cima da outra — e essas
/// são exactamente as que o corte tem de SEPARAR. *A pergunta «são vizinhos?» tem de ser
/// respondida por uma grandeza que a dobra não move.*
///
/// ⛔⛔ **A [`TOLERANCIA_DO_ELO`] é RELATIVA ao comprimento da própria aresta**, e não um
/// número em `[0,1]²`. As duas cópias de uma costura colada afastam-se do que a
/// [`crate::Relatorio::cola_max`] mede — em células de grade —, e o atlas divide tudo
/// pelo lado do quadrado, que muda de peça para peça. *Um epsilon absoluto num espaço
/// normalizado mede o TAMANHO DA PEÇA e não a costura.*
#[must_use]
pub fn elos(mesh: &Mesh, uv: &[[f32; 2]]) -> Vec<(u32, u32, Elo)> {
    let vert = vertices_dos_cantos(mesh);
    let tris = triangulos(mesh);
    // Chave de aresta → os cantos de cada triângulo que a formam, na ordem da CHAVE.
    // ⚠️ `BTreeMap` e não `HashMap`: HR-5. Aqui não é decoração — a ordem em que os
    // pares saem daqui alimenta o corte, e uma tabela de dispersão daria outra partição
    // a cada arranque.
    let mut por_aresta: PorAresta = PorAresta::new();
    for (t, tri) in tris.iter().enumerate() {
        for k in 0..3 {
            let (ca, cb) = (tri[k], tri[(k + 1) % 3]);
            let (va, vb) = (vert[ca as usize], vert[cb as usize]);
            let chave = (va.min(vb), va.max(vb));
            let par = if va <= vb { (ca, cb) } else { (cb, ca) };
            por_aresta.entry(chave).or_default().push((
                u32::try_from(t).unwrap_or(0),
                par.0,
                par.1,
            ));
        }
    }
    let mut out: Vec<(u32, u32, Elo)> = Vec::new();
    for lista in por_aresta.values() {
        for i in 0..lista.len() {
            for j in (i + 1)..lista.len() {
                let (ta, ca0, ca1) = lista[i];
                let (tb, cb0, cb1) = lista[j];
                if ta == tb {
                    continue;
                }
                let (Some(&ua0), Some(&ua1), Some(&ub0), Some(&ub1)) = (
                    uv.get(ca0 as usize),
                    uv.get(ca1 as usize),
                    uv.get(cb0 as usize),
                    uv.get(cb1 as usize),
                ) else {
                    continue;
                };
                let comp = (ua1[0] - ua0[0]).hypot(ua1[1] - ua0[1]);
                let tol = (comp * TOLERANCIA_DO_ELO).max(f32::EPSILON);
                let perto = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) <= tol;
                let elo = Elo {
                    na_peca: true,
                    no_atlas: perto(ua0, ub0) && perto(ua1, ub1),
                };
                out.push((ta.min(tb), ta.max(tb), elo));
            }
        }
    }
    out.sort_unstable_by_key(|&(a, b, _)| (a, b));
    // ⚠️ Dois triângulos podem partilhar mais do que uma aresta numa malha estranha; o
    // elo do par é a UNIÃO dos elos das arestas — *ficar com o primeiro faria a resposta
    // depender da ordem de uma tabela de dispersão*.
    let mut fundido: Vec<(u32, u32, Elo)> = Vec::with_capacity(out.len());
    for (a, b, e) in out {
        match fundido.last_mut() {
            Some((pa, pb, pe)) if *pa == a && *pb == b => {
                pe.na_peca |= e.na_peca;
                pe.no_atlas |= e.no_atlas;
            }
            _ => fundido.push((a, b, e)),
        }
    }
    fundido
}
