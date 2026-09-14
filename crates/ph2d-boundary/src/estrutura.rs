//! §6 e §7 — **andar a borda** e **propagar para dentro**, que juntos produzem a
//! fotografia do pen-down.
//!
//! ```text
//! C  andar a borda a partir da âncora   → cadeia + distância de cadeia
//! D  propagar para dentro               → anel, patrono, alcance K, ponto-origem
//! ```
//!
//! ⚠️ **Esta estrutura é fotografada no pen-down e NÃO é recalculada durante o
//! traço** (§3). É isso que torna o resultado função só do arrasto TOTAL e não
//! do caminho — e é o que a prova de fatiamento do corpus mede.

use crate::topologia::Topologia;
use crate::vetor::{V3, distancia};

/// Um vértice fora do alcance da propagação.
pub const SEM_ANEL: u32 = u32::MAX;

/// A fotografia das fases C e D.
#[derive(Clone, Debug)]
pub struct Estrutura {
    /// A âncora — o vértice de borda de onde tudo sai.
    pub ancora: u32,
    /// Os vértices de borda alcançados, na ordem em que o passeio os visitou.
    pub cadeia: Vec<u32>,
    /// Comprimento de arco ACUMULADO desde a âncora, **andando pela borda** —
    /// `SEM_CADEIA` para quem não é da cadeia.
    pub distancia_de_cadeia: Vec<f32>,
    /// A quantos passos topológicos da cadeia cada vértice está — [`SEM_ANEL`]
    /// fora do alcance.
    pub anel: Vec<u32>,
    /// De que vértice da cadeia a onda chegou a cada vértice (§7.2).
    pub patrono: Vec<u32>,
    /// `K` — o maior anel que a propagação atingiu (§7.3).
    pub alcance: u32,
    /// O vértice do anel `K` na coluna da âncora (§7.4).
    pub vertice_origem: u32,
    /// A posição dele — o **ponto-origem**, que é a referência de quase tudo na
    /// fase G e a linha branca da pré-visualização.
    pub ponto_origem: V3,
    /// **O vértice do anel `K` de cada coluna**, indexado pelo patrono —
    /// `u32::MAX` quando aquela coluna não chega ao fundo.
    ///
    /// ⚠️⚠️ **Ele é DERIVADO uma vez, no pen-down, e não procurado por passo.**
    /// A primeira redacção varria a malha inteira a cada consulta, e as leis do
    /// `BEND` e do `EXPAND` consultam-no **por coluna e por evento** — `O(V ×
    /// colunas)` por passo do traço, sobre uma lei cujo custo medido é `1 ms` a
    /// `263 k` vértices. *Uma tabela que se reconstrói a cada leitura é a mesma
    /// varredura com outro nome.*
    fundo: Vec<u32>,
}

/// A marca de «este vértice não é da cadeia».
pub const SEM_CADEIA: f32 = f32::INFINITY;

/// Constrói as fases C e D.
///
/// `raio_de_propagacao` é `raio_inicial × (1 + deslocamento_da_origem)` (§7.3).
pub fn construir(
    topo: &Topologia,
    posicoes: &[V3],
    escondido: &[bool],
    ancora: u32,
    raio_de_propagacao: f32,
) -> Estrutura {
    let n = topo.n_vertices();
    let mut distancia_de_cadeia = vec![SEM_CADEIA; n];
    let cadeia = andar_a_borda(topo, posicoes, escondido, ancora, &mut distancia_de_cadeia);
    let mut e = Estrutura {
        ancora,
        cadeia,
        distancia_de_cadeia,
        anel: vec![SEM_ANEL; n],
        patrono: vec![u32::MAX; n],
        alcance: 0,
        vertice_origem: ancora,
        ponto_origem: posicoes.get(ancora as usize).copied().unwrap_or([0.0; 3]),
        fundo: Vec::new(),
    };
    propagar(topo, posicoes, escondido, raio_de_propagacao, &mut e);
    // §10.1/§10.2 — a semente por coluna, derivada UMA vez.
    //
    // ⚠️ **Em caso de empate fica o de MENOR índice**, e a escolha é
    // determinismo: numa grelha regular cada coluna tem exactamente um vértice
    // no anel `K`, mas numa malha irregular pode ter vários — e a ordem das
    // faces no ficheiro não pode decidir o eixo da dobra.
    e.fundo = vec![u32::MAX; n];
    for v in (0..n).rev() {
        if e.anel[v] == e.alcance {
            let dono = e.patrono[v] as usize;
            if dono < n {
                e.fundo[dono] = u32::try_from(v).unwrap_or(u32::MAX);
            }
        }
    }
    e
}

/// §6 — o passeio pela borda.
///
/// ⭐ **A distância é comprimento de arco REAL, somado aresta a aresta** — não
/// contagem de arestas e não a linha recta até à âncora. Numa borda curva (o aro
/// de um tubo) as duas divergem, e é a primeira que manda.
fn andar_a_borda(
    topo: &Topologia,
    posicoes: &[V3],
    escondido: &[bool],
    ancora: u32,
    dist: &mut [f32],
) -> Vec<u32> {
    let mut cadeia = Vec::new();
    let mut fila = std::collections::VecDeque::new();
    dist[ancora as usize] = 0.0;
    cadeia.push(ancora);
    fila.push_back(ancora);
    while let Some(v) = fila.pop_front() {
        // ⚠️⚠️ **O vértice que faz PARAR entra na cadeia e só não propaga a
        // partir dali** (§6.2) — ele já recebeu distância acima, e é por isso
        // que o teste está aqui e não à entrada. *Pô-lo antes do `push` faria a
        // quina de uma grelha ficar de fora da deformação, e o corpus mede-a
        // dentro.*
        if !topo.passa_nos_testes(v) {
            continue;
        }
        for &w in topo.vizinhos_de_borda(v) {
            let wi = w as usize;
            if escondido.get(wi).copied().unwrap_or(false) || dist[wi].is_finite() {
                continue;
            }
            dist[wi] = dist[v as usize] + distancia(posicoes[wi], posicoes[v as usize]);
            cadeia.push(w);
            fila.push_back(w);
        }
    }
    cadeia
}

/// §7 — a propagação para dentro: busca em largura **multi-origem** a partir da
/// cadeia inteira.
///
/// ⚠️ **Um vértice é atribuído UMA só vez** — quem chegar primeiro fica com ele.
/// É isso que reparte a malha em «colunas», uma por vértice da cadeia.
fn propagar(topo: &Topologia, posicoes: &[V3], escondido: &[bool], raio: f32, e: &mut Estrutura) {
    let mut fronteira: Vec<u32> = Vec::new();
    for &v in &e.cadeia {
        e.anel[v as usize] = 0;
        e.patrono[v as usize] = v;
        fronteira.push(v);
    }
    // ⚠️⚠️ **A distância acumulada NÃO é a profundidade de todos: ela é somada
    // SÓ ao longo da coluna da âncora** (§7.3) — uma parcela por vizinho novo
    // dessa coluna, por passagem. ⇒ o alcance é decidido pelo tamanho das
    // arestas **onde o artista carregou**, e uma malha de densidade irregular
    // alcança mais fundo onde as arestas são curtas.
    let mut acumulada = 0.0f32;
    let mut anel_actual = 0u32;
    let mut seguinte: Vec<u32> = Vec::new();
    while !fronteira.is_empty() {
        // §7.3 — o teste vem ANTES da passagem.
        if acumulada > raio {
            break;
        }
        anel_actual += 1;
        seguinte.clear();
        for &v in &fronteira {
            let dono = e.patrono[v as usize];
            for &u in topo.vizinhos(v) {
                let ui = u as usize;
                if escondido.get(ui).copied().unwrap_or(false) || e.anel[ui] != SEM_ANEL {
                    continue;
                }
                e.anel[ui] = anel_actual;
                e.patrono[ui] = dono;
                seguinte.push(u);
                if dono == e.ancora {
                    // A coluna da âncora: soma a parcela e guarda o vértice
                    // mais recente dela — no fim, ele é o ponto-origem (§7.4).
                    acumulada += distancia(posicoes[ui], posicoes[v as usize]);
                    e.vertice_origem = u;
                    e.ponto_origem = posicoes[ui];
                }
            }
        }
        if seguinte.is_empty() {
            // A propagação parou por **acabarem os vértices**, e não por
            // exceder o raio — ver a divergência declarada abaixo.
            anel_actual -= 1;
            break;
        }
        std::mem::swap(&mut fronteira, &mut seguinte);
    }
    // ⭐⭐ **DIVERGÊNCIA DECLARADA (§13.1): o alcance é ATADO ao anel mais fundo
    // que de facto existe.**
    //
    // No alvo, quando a propagação pára por acabarem os vértices, `K` fica
    // MAIOR do que o anel mais profundo atribuído ⇒ **nenhum vértice está no
    // anel `K`** ⇒ os dados que só são semeados ali (a direcção do `EXPAND`, o
    // par ponto/eixo do `BEND`) ficam por preencher, e o `EXPAND` deixa de
    // deformar por completo (`0` vértices movidos na fixture da peça pequena
    // com deslocamento `2`). É um defeito **ABERTO** do alvo, e o pedido do
    // relator dele é exactamente esta cura.
    //
    // ⇒ atamos `K` ao máximo atribuído, o que torna as duas leis bem definidas e
    // faz o `EXPAND` deformar onde o alvo fica mudo. ⛔ **Isto é uma divergência
    // com gate que a nomeia** — nunca uma correcção silenciosa.
    let maximo = e
        .anel
        .iter()
        .filter(|&&a| a != SEM_ANEL)
        .copied()
        .max()
        .unwrap_or(0);
    e.alcance = anel_actual.min(maximo);
}

impl Estrutura {
    /// O vértice do anel `K` na coluna de `patrono` — a semente das leis do
    /// `BEND` e do `EXPAND` (§10.1, §10.2).
    ///
    /// ⚠️ **Devolve `None` quando a coluna não chega ao anel `K`**, o que é
    /// normal: as colunas não têm todas a mesma profundidade. Quem a lê decide o
    /// que fazer — e a §13.1 diz que o alvo, nesse caso, deixa os dados por
    /// preencher.
    pub fn fundo_da_coluna(&self, patrono: u32) -> Option<u32> {
        match self.fundo.get(patrono as usize).copied() {
            Some(v) if v != u32::MAX => Some(v),
            _ => None,
        }
    }
}
