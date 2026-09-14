//! §4 — **o que conta como contorno**, e a adjacência de que todas as fases
//! seguintes vivem.
//!
//! # ⭐ A regra, numa linha
//!
//! Uma aresta é **de borda** quando tem **menos de duas** faces incidentes — `0`
//! ou `1`. ⚠️ Uma aresta solta (sem face nenhuma) conta, e os dois vértices dela
//! são vértices de borda.
//!
//! # ⚠️⚠️ Porque é preciso guardar o CONJUNTO DAS ARESTAS, e não só a marca
//!
//! *«São vizinhos de borda os vértices de borda ligados por uma aresta»* é a
//! regra certa, e **`ambos são vértices de borda` não a exprime**: numa tira de
//! faces, dois vértices de borda podem estar ligados por uma aresta
//! **interior**. ⇒ a pergunta é *«esta ARESTA é de borda?»* (§4.4). Os autores
//! do alvo pagaram exactamente este defeito.
//!
//! ⇒ esta estrutura guarda **duas** vizinhanças e elas não são a mesma:
//!
//! | vizinhança | quem a lê |
//! |---|---|
//! | [`Topologia::vizinhos`] — todas as arestas | a propagação para dentro (§7), o teste de grau e o de «borda ao redor» (§5.2), o alisar (§10.6) |
//! | [`Topologia::vizinhos_de_borda`] — só por **aresta de borda** | o passeio da cadeia (§6) |
//!
//! # ⚠️ Geometria ESCONDIDA também produz contorno, e isto está DECLARADO
//!
//! Um vértice cujas faces incidentes não estão todas visíveis é tratado como de
//! borda (§4.2). ⛔ **É a única regra da espec sem fixture** — o harness do
//! oráculo nunca esconde geometria —, e por isso ela entra aqui pela porta da
//! frente (`escondido`) e fica **nomeada** em vez de suposta: quem ligar o
//! esconder tem de reconferir com uma corrida nova.

/// A malha, do ponto de vista deste pincel: quem é vizinho de quem, e onde a
/// superfície acaba.
///
/// ⚠️ **Ela é `O(malha)` a construir e não muda durante o traço** — §4.5 exige
/// que o censo corra **uma vez por malha** e seja refeito quando a topologia
/// mudar. Quem a guarda é o chamador.
#[derive(Clone, Debug)]
pub struct Topologia {
    inicio: Vec<u32>,
    vizinhos: Vec<u32>,
    inicio_borda: Vec<u32>,
    vizinhos_borda: Vec<u32>,
    de_borda: Vec<bool>,
}

impl Topologia {
    /// Constrói o censo a partir dos anéis das faces.
    ///
    /// ⭐ `faces` é um iterador de **anéis** (`&[u32]`) pelo motivo da irmã da
    /// pose: a malha da escultura guarda triângulos e quads num `[u32; 4]` com
    /// sentinela e a bancada lê-os de um ficheiro — pedir uma `Vec<Vec<u32>>`
    /// obrigaria um dos dois a copiar a malha inteira.
    ///
    /// `escondido` marca os vértices ocultos: uma face com algum vértice oculto
    /// **não é visível**, logo não conta faces para as arestas dela (§4.2).
    pub fn construir<'f, I>(n_vertices: usize, faces: I, escondido: &[bool]) -> Topologia
    where
        I: IntoIterator<Item = &'f [u32]>,
    {
        let oculto = |v: u32| escondido.get(v as usize).copied().unwrap_or(false);
        // ⚠️ **A contagem é POR ARESTA, e a aresta é um par ordenado do menor
        // para o maior** — senão `(a,b)` e `(b,a)` contam como duas.
        let mut arestas: Vec<(u32, u32)> = Vec::new();
        let mut incidencias: Vec<u32> = Vec::new();
        let mut adjacentes: Vec<Vec<u32>> = vec![Vec::new(); n_vertices];
        // Índice de aresta por par, num mapa por vértice (o menor dos dois).
        let mut por_vertice: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n_vertices];
        let aresta_de = |a: u32,
                         b: u32,
                         arestas: &mut Vec<(u32, u32)>,
                         incidencias: &mut Vec<u32>,
                         por_vertice: &mut Vec<Vec<(u32, u32)>>|
         -> usize {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            if let Some(&(_, i)) = por_vertice[lo as usize].iter().find(|&&(o, _)| o == hi) {
                return i as usize;
            }
            let i = arestas.len();
            arestas.push((lo, hi));
            incidencias.push(0);
            por_vertice[lo as usize].push((hi, i as u32));
            i
        };

        for anel in faces {
            if anel.len() < 3 {
                continue;
            }
            // ⚠️ **Uma face com vértice oculto não é visível e não conta faces
            // para as arestas dela** — mas as arestas continuam a EXISTIR, e é
            // isso que faz o esconder produzir borda (§4.2).
            let visivel = !anel.iter().any(|&v| oculto(v));
            let n = anel.len();
            for (k, &v) in anel.iter().enumerate() {
                if v as usize >= n_vertices {
                    continue;
                }
                let w = anel[(k + 1) % n];
                if w as usize >= n_vertices {
                    continue;
                }
                let i = aresta_de(v, w, &mut arestas, &mut incidencias, &mut por_vertice);
                if visivel {
                    incidencias[i] += 1;
                }
                adjacentes[v as usize].push(w);
                adjacentes[w as usize].push(v);
            }
        }

        let mut de_borda = vec![false; n_vertices];
        let mut adj_borda: Vec<Vec<u32>> = vec![Vec::new(); n_vertices];
        for (i, &(a, b)) in arestas.iter().enumerate() {
            if incidencias[i] >= 2 {
                continue;
            }
            de_borda[a as usize] = true;
            de_borda[b as usize] = true;
            adj_borda[a as usize].push(b);
            adj_borda[b as usize].push(a);
        }

        // ⚠️ **Ordenar é DETERMINISMO, não arrumação:** a ordem de visita decide
        // qual vértice fica com um empate na escolha da âncora (§5.1) e qual
        // chega primeiro a um vértice interior na propagação (§7.1) — e essa
        // atribuição reparte a malha em colunas. *A ordem das faces no ficheiro
        // não pode decidir a deformação.*
        let (inicio, vizinhos) = achatar(&mut adjacentes);
        let (inicio_borda, vizinhos_borda) = achatar(&mut adj_borda);
        Topologia {
            inicio,
            vizinhos,
            inicio_borda,
            vizinhos_borda,
            de_borda,
        }
    }

    pub fn n_vertices(&self) -> usize {
        self.de_borda.len()
    }

    /// Todos os vizinhos topológicos de um vértice.
    pub fn vizinhos(&self, v: u32) -> &[u32] {
        fatia(&self.inicio, &self.vizinhos, v)
    }

    /// Só os vizinhos ligados por uma **aresta de borda** (§4.4).
    pub fn vizinhos_de_borda(&self, v: u32) -> &[u32] {
        fatia(&self.inicio_borda, &self.vizinhos_borda, v)
    }

    /// Este vértice pertence a alguma aresta de borda?
    pub fn e_de_borda(&self, v: u32) -> bool {
        self.de_borda.get(v as usize).copied().unwrap_or(false)
    }

    /// Quantos vértices de borda há — o censo do §4.1, contado.
    pub fn conta_vertices_de_borda(&self) -> usize {
        self.de_borda.iter().filter(|b| **b).count()
    }

    /// Quantas arestas de borda há.
    pub fn conta_arestas_de_borda(&self) -> usize {
        // Cada aresta de borda aparece uma vez em cada uma das duas pontas.
        self.vizinhos_borda.len() / 2
    }

    /// ⭐⭐ **As DUAS recusas da §5.2, num sítio só** — porque elas são o mesmo
    /// par de testes em dois papéis: o traço inteiro é recusado se o vértice
    /// **sob o cursor** falhar (§5.2), e o passeio da cadeia **não atravessa**
    /// um vértice que falhe (§6.2).
    ///
    /// ⚠️ **Escrevê-las em dois sítios seria a segunda resposta à mesma
    /// pergunta**, e o dia em que uma delas ganhasse um terceiro teste as duas
    /// divergiriam — em silêncio, porque as duas fixtures que as medem são
    /// diferentes.
    ///
    /// | teste | recusa quando |
    /// |---|---|
    /// | **grau** | o vértice tem `≤ 2` vizinhos |
    /// | **borda ao redor** | `> 2` dos vizinhos são vértices de borda |
    ///
    /// ⚠️ O segundo conta **vizinhos que SÃO de borda**, e não vizinhos
    /// alcançados por aresta de borda — as duas contagens divergem numa tira de
    /// faces, e é a primeira que a espec mede.
    pub fn passa_nos_testes(&self, v: u32) -> bool {
        let viz = self.vizinhos(v);
        if viz.len() <= 2 {
            return false;
        }
        let ao_redor = viz.iter().filter(|&&u| self.e_de_borda(u)).count();
        ao_redor <= 2
    }
}

fn fatia<'a>(inicio: &[u32], dados: &'a [u32], v: u32) -> &'a [u32] {
    let i = v as usize;
    if i + 1 >= inicio.len() {
        return &[];
    }
    &dados[inicio[i] as usize..inicio[i + 1] as usize]
}

/// Ordena, deduplica e achata em CSR.
fn achatar(listas: &mut [Vec<u32>]) -> (Vec<u32>, Vec<u32>) {
    let mut inicio = Vec::with_capacity(listas.len() + 1);
    let mut dados = Vec::new();
    inicio.push(0);
    for lista in listas.iter_mut() {
        lista.sort_unstable();
        lista.dedup();
        dados.extend_from_slice(lista);
        inicio.push(u32::try_from(dados.len()).unwrap_or(u32::MAX));
    }
    (inicio, dados)
}
