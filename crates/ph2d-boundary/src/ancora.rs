//! §5 — **escolher o vértice âncora, e quando o pincel RECUSA o traço inteiro.**
//!
//! # A escolha
//!
//! 1. toma-se o vértice mais próximo do ponto de contacto;
//! 2. **se ele já é de borda, é a âncora**;
//! 3. senão, busca em largura a partir dele, **limitada pelo raio**, contando
//!    passos topológicos — a âncora é o vértice de borda com **menos passos**,
//!    e empates ficam com o primeiro encontrado na ordem de visita;
//! 4. se a busca não achar borda nenhuma, **não há traço**.
//!
//! ⚠️ **O raio que limita esta busca é o INICIAL do traço** (o do pen-down), e
//! não o dinâmico que a pressão possa estar a modular — os autores do alvo
//! corrigiram precisamente isto.
//!
//! # ⚠️⚠️ O sujeito das duas recusas é o vértice SOB O CURSOR, não a âncora
//!
//! E isso é **observável sem conjecturar nada**: na fixture
//! `grade_canto_agarrar_constante` o cursor está na quina de uma grelha e o
//! traço devolve `0` vértices movidos — *mesmo havendo, a uma célula dali,
//! vértices de borda perfeitamente sãos que serviriam de âncora*. ⇒ a recusa não
//! está a olhar para a âncora; está a olhar para o ponto que o artista apontou.

use crate::topologia::Topologia;
use crate::vetor::{V3, distancia2};

/// Porque é que não há traço.
///
/// ⚠️ **Três razões e não uma bandeira** — a §17 exige que a pré-visualização
/// distinga *«não há borda ao alcance»* (não se desenha nada) de uma recusa
/// geométrica, e um `Option` colapsa as três num silêncio só.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Recusa {
    /// Não há vértice de borda dentro do raio a partir do ponto apontado
    /// (§5.1.4) — inclui a malha fechada, que não tem borda nenhuma.
    SemBordaAoAlcance,
    /// O vértice sob o cursor tem `≤ 2` vizinhos: ele está onde **dois troços
    /// de borda se encontram**, e não há regra que escolha entre eles.
    GrauDemasiadoBaixo,
    /// `> 2` vizinhos do vértice sob o cursor são de borda: a borda
    /// **ramifica-se** ali, e uma cadeia única deixa de estar definida.
    BordaRamificada,
}

/// O vértice mais próximo de um ponto — o «vértice activo» que o resto do
/// editor já calcula.
///
/// ⚠️ **Sem limite de distância**, de propósito: o limite é do passo seguinte
/// (a busca), e recusar aqui apagaria a diferença entre *«o cursor está longe»*
/// e *«não há borda perto»*, que a §17 precisa de distinguir.
pub fn mais_proximo(posicoes: &[V3], escondido: &[bool], p: V3) -> Option<u32> {
    let mut melhor: Option<(f32, u32)> = None;
    for (i, &q) in posicoes.iter().enumerate() {
        if escondido.get(i).copied().unwrap_or(false) {
            continue;
        }
        let d = distancia2(q, p);
        if melhor.is_none_or(|(md, _)| d < md) {
            melhor = Some((d, u32::try_from(i).unwrap_or(u32::MAX)));
        }
    }
    melhor.map(|(_, v)| v)
}

/// §5 inteira: a âncora, ou a razão pela qual não há traço.
///
/// `raio` é o **inicial** (§10.7).
pub fn escolher(
    topo: &Topologia,
    posicoes: &[V3],
    escondido: &[bool],
    sob_o_cursor: u32,
    raio: f32,
) -> Result<u32, Recusa> {
    // ⚠️ **A ORDEM é load-bearing e não é arbitrária:** a busca corre PRIMEIRO
    // porque a §5.1.4 (não há borda ao alcance) é a ausência do pincel — a
    // pré-visualização não desenha nada e não há nada a recusar —, enquanto as
    // duas da §5.2 são recusas de um traço que teria âncora. *Trocar a ordem faz
    // a malha fechada ser reportada como «quina», e o report ao artista muda.*
    let ancora =
        procurar(topo, posicoes, escondido, sob_o_cursor, raio).ok_or(Recusa::SemBordaAoAlcance)?;
    let viz = topo.vizinhos(sob_o_cursor);
    if viz.len() <= 2 {
        return Err(Recusa::GrauDemasiadoBaixo);
    }
    if viz.iter().filter(|&&u| topo.e_de_borda(u)).count() > 2 {
        return Err(Recusa::BordaRamificada);
    }
    Ok(ancora)
}

/// §5.1 passos 2 e 3 — a busca em largura limitada pelo raio.
fn procurar(
    topo: &Topologia,
    posicoes: &[V3],
    escondido: &[bool],
    inicial: u32,
    raio: f32,
) -> Option<u32> {
    if topo.e_de_borda(inicial) {
        return Some(inicial);
    }
    let raio2 = raio * raio;
    let origem = *posicoes.get(inicial as usize)?;
    let mut visto = vec![false; topo.n_vertices()];
    let mut fila = std::collections::VecDeque::new();
    visto[inicial as usize] = true;
    fila.push_back(inicial);
    while let Some(v) = fila.pop_front() {
        for &u in topo.vizinhos(v) {
            let ui = u as usize;
            if visto.get(ui).copied().unwrap_or(true) || escondido.get(ui).copied().unwrap_or(false)
            {
                continue;
            }
            // ⚠️ **A cerca é `< raio²` contra a posição INICIAL** — a distância
            // euclidiana ao ponto de partida, não o comprimento do caminho
            // percorrido. As duas divergem numa malha curva.
            if distancia2(posicoes[ui], origem) >= raio2 {
                continue;
            }
            visto[ui] = true;
            // ⭐ A busca é FIFO, logo os vértices saem por **passos
            // topológicos** crescentes ⇒ o primeiro de borda que ela encontra é
            // o de menos passos, e o empate fica com o primeiro da ordem de
            // visita — que é exactamente a regra da §5.1.3, sem precisar de
            // guardar a contagem de passos.
            if topo.e_de_borda(u) {
                return Some(u);
            }
            fila.push_back(u);
        }
    }
    None
}
