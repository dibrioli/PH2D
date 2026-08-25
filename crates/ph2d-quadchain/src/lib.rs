#![forbid(unsafe_code)]
//! `ph2d-quadchain` — **a ORDEM da cadeia de quads**, numa porta que qualquer módulo pode chamar.
//!
//! # Por que esta crate existe
//!
//! A cadeia que transforma uma malha de triângulos numa malha de **quads alinhados à superfície**
//! tem sete passos, e a ordem deles é load-bearing (a fase zero sozinha vale `2×` no enviesamento
//! final). Até 2026-08-24 essa ordem vivia **dentro do shell do módulo de escultura**
//! (`sculpt3d_history_retopo_extract.rs`, `pub(in crate::sculpt3d)`) — alcançável por um módulo só.
//!
//! ⚠️ **E o segundo consumidor chegou**: o modelador implícito extrai a peça por *Dual Contouring*
//! sobre grade, e o placar dele
//! (`ph2d_field_eval::tests::the_scorecard_of_the_extracted_mesh`) mediu **onde ele perde**:
//!
//! | eixo | o extractor de campo | esta cadeia | oráculo `quadwild-bimdf` |
//! |---|---|---|---|
//! | arestas não-manifold · bordo | **0 · 0** | — | — |
//! | `\|f\|` no vértice | **~0,005 célula** | — | — |
//! | 100 % quads | **sim** | sim | sim |
//! | **enviesamento mediano** | ⛔ **25–27°** | ⭐ **5,1–5,5°** | 4,8–7,1° |
//!
//! ⛔ **E o buraco é ESTRUTURAL, não de afinação** — medido: o *mesmo* cubo alinhado com a grade sai
//! a `1,00` de aspecto e `0°` de enviesamento; rodado 45° sai a **`1,41 = √2`** com cauda a `90°`.
//! *A forma de uma face dual segue a GRADE, não a superfície.* Nenhum parâmetro cura isso; o que cura
//! é outra **conectividade** — que é exactamente o que esta cadeia produz.
//!
//! # ⚠️ Ela é a ORDEM, e não o algoritmo
//!
//! Cada passo já existe e é medido na sua própria crate. O que aqui se guarda é **a sequência e as
//! duas leis que a acompanham**:
//!
//! 1. ⛔ **A FASE ZERO é obrigatória.** Sem remalhar isotropicamente à frente, a mesma cadeia dá
//!    `10–12°` em vez de `5–5,5°` — *o dobro, sem uma linha de algoritmo mudar*.
//! 2. ⭐ **O alvo sai da malha que o artista trouxe**, nunca da remalhada — derivá-lo da remalhada
//!    foi medido e mata o controle de detalhe.
//!
//! # ⚠️ Duas cópias desta ordem seria o defeito
//!
//! O shell da escultura tem hoje a dele, e esta crate nasce **sem lhe tocar**: a `line/quadextract`
//! está viva sobre aquele arquivo, e uma migração ali seria colisão de mesmo-símbolo com uma linha
//! em curso (`DIRETRIZ` §1.5.5). ⇒ ela é escrita para que aquela metade adopte esta porta **numa
//! linha**, quando aquela linha quiser. *Duas cópias de uma lei é uma lei que gate nenhum defende —
//! e esta nasce sabendo disso.*

use ph2d_mesh::Mesh;

/// Por que a cadeia recusou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainError {
    /// A extracção das isolinhas recusou o mapa.
    Extract(ph2d_quadextract::ExtractError),
    /// A cadeia correu e não sobrou face nenhuma — o alvo é grosso demais para a peça.
    TooCoarse,
}

/// O que a cadeia tem a dizer sobre o que produziu.
#[derive(Clone, Debug)]
pub struct ChainReport {
    pub quads: usize,
    pub non_quads: usize,
    pub verts: usize,
    /// Arestas de bordo — numa peça fechada tem de ser `0`.
    pub boundary_edges: usize,
    /// Faces do MAPA que se dobraram. ⚠️ **Não são as da saída**: a extracção tolera a dobra por
    /// construção, e o que ela não pode é inventar grade onde o mapa se enrola sobre si próprio.
    pub folded: usize,
    /// O arredondamento deixou toda transição **inteira**?
    pub aligned: bool,
    /// A forma de cada face — a régua que a `line/sculpt3d` calibrou contra o oráculo.
    pub shape: ph2d_quadfill::QuadShape,
}

/// ⭐⭐⭐ **A CADEIA, do triângulo ao quad alinhado.**
///
/// `target_edge` é o comprimento de aresta que se quer na saída, **em unidades da malha de
/// entrada**.
///
/// # Errors
/// Ver [`ChainError`].
pub fn quads_from_mesh(
    reference: &Mesh,
    target_edge: f32,
) -> Result<(Mesh, ChainReport), ChainError> {
    // ── F1. ⛔ **A fase zero, e ela não se salta**: com a triangulação crua a mesma cadeia dá o
    // dobro do enviesamento (medido).
    let mut work = reference.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();

    // ── F2 (campo cruzado) + F3 (traçado dos patches) + G1/G2 (corte e penteado).
    let dual = ph2d_crossfield::Dual::build(&work);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&work, &dual, &field);
    let (cut, _) = ph2d_gridmap::cut_along_patches(&work, &layout);
    let (combed, _) = ph2d_gridmap::comb_patches(&work, &layout, &cut);

    // ⭐ As singularidades saem do CAMPO — o índice por-vértice é um facto dele, e pedir à
    // `ph2d-gridmap` que o re-derive seria reconstruir o que já existe.
    let singular: Vec<u32> = ph2d_crossfield::vertex_index(&work, &dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();

    // ── G3 + G5. O mapa, e o arredondamento uma-a-uma que o torna inteiro.
    let opts = ph2d_gridmap::RoundOptions::default();
    let (map, round) = if ph2d_gridmap::welded_enabled() {
        ph2d_gridmap::round_welded(&work, &cut, &combed, target_edge, opts, &singular)
    } else {
        ph2d_gridmap::round_to_integers(&work, &cut, &combed, target_edge, opts, &singular)
    };

    // ── A extracção das isolinhas inteiras.
    let (tris, uv) = ph2d_gridmap::corner_map(&cut, &map);
    let cm = ph2d_quadextract::CornerMap {
        pos: work.positions(),
        tris: &tris,
        uv: &uv,
    };
    let (out, e) = ph2d_quadextract::extract(&cm, None).map_err(ChainError::Extract)?;
    if out.faces().is_empty() {
        return Err(ChainError::TooCoarse);
    }
    let shape = ph2d_quadfill::quad_shape(&out);
    let report = ChainReport {
        quads: e.quads,
        non_quads: out.face_count() - e.quads,
        verts: out.vert_count(),
        boundary_edges: boundary_edges(&out),
        folded: e.folded_faces,
        aligned: round.shift_frac_max == 0.0,
        shape,
    };
    Ok((out, report))
}

/// O que a cadeia decidiu sobre a peça — ver [`quads_or_keep`].
#[derive(Clone, Debug)]
pub enum Verdict {
    /// A cadeia correu e a saída é melhor. O relatório dela vai junto.
    Adopted(Box<ChainReport>),
    /// ⛔ A cadeia correu e **abriu a peça** — bordo ou aresta não-manifold onde não havia.
    Rejected {
        boundary: usize,
        non_manifold: usize,
    },
    /// A cadeia correu e **não melhorou a forma** — não há motivo para trocar a malha.
    NoGain { before: f32, after: f32 },
    /// A cadeia recusou.
    Refused(ChainError),
    /// ⛔ **A cadeia ESTOUROU.** Ver [`quads_or_keep`] — é um defeito a jusante, e esta porta
    /// existe para que ele não derrube quem pediu uma melhoria opcional.
    Panicked,
}

/// ⭐⭐⭐ **A CADEIA COM VETO** — corre, e só troca a malha se a troca for uma melhoria.
///
/// # Por que ela não é «corre sempre»
///
/// ⛔ **Medido** (`ph2d_field_eval::tests::the_quad_chain_turns_our_mesh_into_oracle_class`), sobre
/// a malha que o modelador implícito extrai:
///
/// | peça | extraída | pela cadeia | veredito |
/// |---|---|---|---|
/// | esfera | `1,48` / `26,6°` / 120 péssimas | ⭐ **`1,08` / `6,4°` / 4** | **a classe do oráculo** (`1,08` / `4,8–7,1°`) |
/// | toro | `1,49` / `24,8°` / 16 | `1,20` / `9,0°` / 9 | melhor |
/// | ⛔ **cubo rodado 45°** | `1,00` / **`0,0°`** / 0 | `1,35` / `17,9°` / 112 | **PIOR — e abre 6 arestas de bordo** |
///
/// ⭐ **A causa é geométrica e nomeável:** numa peça *dura* (faces planas, quinas vivas) a grade
/// dual **já é** a resposta certa — o quad dela pousa na face e sai a `0°`. O campo cruzado não tem
/// nada a que se alinhar numa face plana, e o que ele inventa é pior do que o que já havia.
/// *A cadeia é para a peça orgânica; a grade é para a peça dura.*
///
/// # As duas metades do veto, e nenhuma delas é um peso arbitrário
///
/// 1. ⛔ **Uma peça fechada continua fechada.** Bordo ou aresta não-manifold onde não havia é um
///    veto **duro**, não uma penalização: nenhum ganho de forma paga um buraco.
/// 2. Depois disso, troca-se **se a forma melhorar** (o enviesamento mediano desce).
///
/// ⚠️ *Uma regra de escolha com pesos seria uma opinião com números por cima; estas duas são
/// propriedades.*
///
/// # ⛔ E há uma PRÉ-condição, medida
///
/// A peça que entra tem de ser **fechada e manifold**. Não é zelo: uma calote faz o `ph2d-gridmap`
/// entrar em `panic!`, e um `panic` de uma crate a jusante derruba quem a chamou — um `Result` não
/// a salva. *Uma porta que não pode recusar tem de saber não entrar.*
///
/// # Errors
/// Nunca — a recusa da cadeia vira [`Verdict::Refused`] e a malha de entrada volta intacta.
#[must_use]
pub fn quads_or_keep(reference: &Mesh, target_edge: f32) -> (Mesh, Verdict) {
    let before = ph2d_quadfill::quad_shape(reference);
    let (bound_in, non_in) = edge_census(reference);
    // ⛔ **A CADEIA É PARA PEÇA FECHADA, e a pré-condição não é zelo: sem ela ela ESTOURA.**
    // Medido: uma calote (uma esfera sem as últimas fileiras) faz o `ph2d-gridmap` entrar em
    // `panic!` no `solve.rs`. ⚠️ Um `Result` não a salva — um `panic` de uma crate a jusante derruba
    // quem a chamou. *Uma porta que não pode recusar tem de saber não entrar.*
    if bound_in > 0 || non_in > 0 {
        return (
            reference.clone(),
            Verdict::Rejected {
                boundary: bound_in,
                non_manifold: non_in,
            },
        );
    }
    // ⛔⛔ **A CADEIA ESTOURA em malhas perfeitamente válidas, e o estouro é a jusante.**
    //
    // Medido: um **cubo subdividido** — fechado, manifold, 100 % quads — faz o `ph2d-gridmap`
    // entrar em `index out of bounds: the len is 129 but the index is 157`
    // (`solve.rs:336`, ao emparelhar os lados de uma costura). ⚠️ Não é uma pré-condição que se
    // possa conferir à porta: a malha satisfaz tudo o que se sabe exigir.
    //
    // ⭐ **E por isso esta porta apanha o estouro em vez de o propagar.** Ela oferece uma
    // MELHORIA OPCIONAL: um `panic` a jusante não pode derrubar quem exportou uma peça. O veto já
    // diz *"fica com a entrada a menos que a saída seja melhor"* — um estouro é só mais uma forma
    // de não ser melhor.
    //
    // ⛔ **Isto NÃO é a cura.** O defeito é do `ph2d-gridmap` e a linha dele está viva sobre aquele
    // arquivo (`line/quadextract`); tocá-lo daqui seria colisão de mesmo-símbolo. Ele está nomeado
    // no handoff, com a fixtura que o reproduz.
    let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        quads_from_mesh(reference, target_edge)
    }));
    let Ok(ran) = ran else {
        return (reference.clone(), Verdict::Panicked);
    };
    match ran {
        Err(e) => (reference.clone(), Verdict::Refused(e)),
        Ok((out, r)) => {
            let (bound_out, non_out) = edge_census(&out);
            if bound_out > bound_in || non_out > non_in {
                return (
                    reference.clone(),
                    Verdict::Rejected {
                        boundary: bound_out,
                        non_manifold: non_out,
                    },
                );
            }
            if r.shape.skew_p50 >= before.skew_p50 {
                return (
                    reference.clone(),
                    Verdict::NoGain {
                        before: before.skew_p50,
                        after: r.shape.skew_p50,
                    },
                );
            }
            (out, Verdict::Adopted(Box::new(r)))
        }
    }
}

/// Quantas arestas da malha são tocadas por **uma** face só.
#[must_use]
pub fn boundary_edges(mesh: &Mesh) -> usize {
    edge_census(mesh).0
}

/// Quantas arestas são tocadas por um número de faces **diferente de 2** — o censo de manifold.
#[must_use]
pub fn non_manifold_edges(mesh: &Mesh) -> usize {
    edge_census(mesh).1
}

fn edge_census(mesh: &Mesh) -> (usize, usize) {
    use std::collections::BTreeMap;
    let mut count: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.0;
        let n = if v[3] == v[2] { 3 } else { 4 };
        for k in 0..n {
            let (a, b) = (v[k], v[(k + 1) % n]);
            *count.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    (
        count.values().filter(|c| **c == 1).count(),
        count.values().filter(|c| **c > 2).count(),
    )
}
