//! ⭐⭐⭐ **DE QUEM É CADA FACE** (plano 40) — a lei que faz a tinta sobreviver a uma mudança de
//! topologia.
//!
//! Módulo irmão do [`crate::vec_bucket`], e o corte é por RESPONSABILIDADE: aquele responde
//! *"quando se recoze"*, este responde *"quem herda o quê"*. A segunda é a que o report do Enio de
//! 2026-09-02 nomeia — *"quando atravessamos uma linha com um nó, os preenchimentos se quebram"*.
//!
//! # A lei, e de onde ela vem
//!
//! > **Uma face herda a tinta da região que mais a cobria.**
//!
//! É o *Live Paint* do Illustrator, e as duas metades da regra estão documentadas por eles: partir
//! uma face pinta **as duas** metades, e fundir duas faces com tintas diferentes dá a face à
//! **maior**. ⭐ Aqui as duas caem de uma lei só — não são dois casos com dois códigos.
//!
//! ⛔ **A alternativa que o Enio pôs na mesa — *"limitar a movimentação dos nós de modo que se movam
//! apenas dentro das linhas da área em que estão"* — foi recusada**, e não por gosto: nenhum editor
//! vectorial prende um nó a uma região, o artista deixaria de poder redesenhar a forma que pintou,
//! e a restrição seria impossível de exprimir no instante em que a área de destino ainda não
//! existe. O desenho manda na tinta, nunca o contrário.
//!
//! # ⛔ Porque UMA semente não chega
//!
//! Medido num quadrado de área `400` cortado ao meio por uma linha
//! ([`ph2d-vec-fill/examples`](../../../crates/ph2d-vec-fill/)):
//!
//! | o gesto | a rede passa a ter | com UMA semente |
//! |---|---|---|
//! | a linha entra e PARTE a região | 2 faces de `200` | a tinta fica com uma: **metade some** |
//! | um nó atravessa o topo e FUNDE | `4,17` + `395,83` | **as duas sementes caem na mesma face** |
//!
//! ⇒ *uma semente diz **onde** a tinta estava; ela não diz **quanto** da face era dela.*

use ph2d_vec_fill::{Face, Rede};
use ph2d_vec_scene::point_in_polygon;

/// **A região que um preenchimento pintou da última vez** — achatada, em MUNDO.
///
/// ⚠️ Um preenchimento pode já ser vários contornos (uma região que partiu na edição anterior), e
/// eles são **disjuntos**: estar na região é estar dentro de **algum** deles. ⛔ Não há buraco a
/// considerar — uma face nunca é subtraída de outra aqui.
pub(crate) struct Regiao {
    pub(crate) poligonos: Vec<Vec<[f64; 2]>>,
    /// O ponto do clique que a criou, re-semeado a cada recozimento. Serve o DESEMPATE.
    pub(crate) semente: [f64; 2],
}

impl Regiao {
    fn contem(&self, p: [f64; 2]) -> bool {
        self.poligonos.iter().any(|q| point_in_polygon(q, p))
    }
}

/// A caixa de um conjunto de polilinhas. `None` quando não há ponto nenhum.
fn caixa(polis: &[Vec<[f64; 2]>]) -> Option<([f64; 2], [f64; 2])> {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for p in polis.iter().flatten() {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    lo[0].is_finite().then_some((lo, hi))
}

/// As duas caixas tocam-se?
fn cruzam(a: ([f64; 2], [f64; 2]), b: ([f64; 2], [f64; 2])) -> bool {
    a.0[0] <= b.1[0] && b.0[0] <= a.1[0] && a.0[1] <= b.1[1] && b.0[1] <= a.1[1]
}

/// ⭐⭐⭐ **A VOTAÇÃO**: por face, o índice da região dona — ou `None` se nenhuma tinta a cobria.
///
/// Cada amostra do miolo da face vota na região que a contém; ganha a mais votada. Como as amostras
/// são uniformes sobre a caixa da face, a contagem é proporcional à **área** que cada região cobre
/// dentro dela — que é exactamente a grandeza que *"a face fundida fica com a da maior"* pede.
///
/// ⚠️ **Uma face sem voto nenhum fica por pintar**, e é a resposta certa: uma região que ninguém
/// tinha pintado é nova, e pintá-la sozinha inventaria uma decisão do artista.
///
/// ⚠️ **O empate é resolvido, e tem de ser**: um quadrado cortado ao meio exactamente, com as duas
/// metades pintadas e a parede a desaparecer, dá o mesmo número de votos às duas. Primeiro ganha
/// quem tem a **semente dentro desta face** (o clique original estava mesmo ali); persistindo,
/// ganha o índice mais baixo, que é a ordem do documento. ⛔ Um desempate ao acaso faria a tinta
/// piscar entre duas cores enquanto a mão treme.
pub(crate) fn donos(rede: &Rede, faces: &[Face], regioes: &[Regiao]) -> Vec<Option<usize>> {
    let caixas: Vec<Option<([f64; 2], [f64; 2])>> =
        regioes.iter().map(|r| caixa(&r.poligonos)).collect();
    faces
        .iter()
        .map(|f| {
            let amostras = rede.interior_samples(f);
            if amostras.is_empty() || regioes.is_empty() {
                return None;
            }
            // ⚠️⚠️ **A REJEIÇÃO POR CAIXA vem ANTES do voto, e não é micro-optimização.** Medido numa
            // grelha de 49 faces com 8 preenchimentos: sem ela a votação custa **4,53 ms** — 30% de
            // um quadro, e ela corre a cada quadro de um arrasto de nó. O produto é
            // `faces × amostras × regiões`, e numa grelha cada face toca **uma** região: cortar o
            // factor das regiões é o que a torna pagável.
            let poly = rede.contorno(f);
            let cf = caixa(std::slice::from_ref(&poly))?;
            let candidatos: Vec<usize> = (0..regioes.len())
                .filter(|k| caixas[*k].is_some_and(|c| cruzam(cf, c)))
                .collect();
            let mut votos = vec![0usize; regioes.len()];
            for a in &amostras {
                for &k in &candidatos {
                    if regioes[k].contem(*a) {
                        votos[k] += 1;
                    }
                }
            }
            let max = votos.iter().copied().max().unwrap_or(0);
            if max == 0 {
                return None;
            }
            let empatados: Vec<usize> = (0..regioes.len()).filter(|k| votos[*k] == max).collect();
            if empatados.len() == 1 {
                return empatados.first().copied();
            }
            empatados
                .iter()
                .copied()
                .find(|k| point_in_polygon(&poly, regioes[*k].semente))
                .or_else(|| empatados.first().copied())
        })
        .collect()
}

/// ⭐⭐ **AS FACES DE CADA PREENCHIMENTO, a MAIOR à frente.**
///
/// ⚠️ **A ordem é load-bearing, duas vezes**: a primeira face vira o contorno **primário** do
/// caminho (as outras são `subpaths`) e é dela que sai a **semente** nova. Com a menor à frente, a
/// semente de uma região que partiu iria viver na lasca — e a lasca é justamente o pedaço que a
/// edição seguinte come.
///
/// ⛔ **Um preenchimento pode ganhar VÁRIAS faces, e é esse o ponto**: era isso que faltava quando
/// partir uma região deixava metade por pintar.
pub(crate) fn por_preenchimento(
    faces: &[Face],
    donos: &[Option<usize>],
    quantos: usize,
) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); quantos];
    for (i, dono) in donos.iter().enumerate() {
        if let Some(k) = dono
            && let Some(lista) = out.get_mut(*k)
        {
            lista.push(i);
        }
    }
    for lista in &mut out {
        lista.sort_by(|a, b| faces[*b].area.total_cmp(&faces[*a].area));
    }
    out
}

#[cfg(test)]
#[path = "vec_bucket_claim_tests.rs"]
mod tests;
