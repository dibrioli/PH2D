//! §2.2 e §2.4 — quem é vizinho de quem.
//!
//! Um vértice é vizinho de outro quando partilham uma **aresta de alguma face
//! visível**. A lista constrói-se percorrendo as faces incidentes e tomando, em
//! cada face, os **dois** vértices adjacentes a ele no anel da face, sem
//! repetir — ⭐ e é essa formulação (e não «os outros vértices da face») que
//! suporta topologia **não-manifold**, onde três faces partilham uma aresta.

use crate::vetor::{distancia, V3};

/// Adjacência em CSR: `vizinhos[inicio[v] .. inicio[v+1]]`.
///
/// ⚠️ **As ligações artificiais entre peças (§2.4) vivem SEPARADAS**, num vector
/// à parte — e não são apendadas a esta lista. A razão é a §4: a travessia e o
/// crescimento **usam-nas**, a suavização dos pesos **não**. Fundi-las numa
/// lista só torna essa distinção inexprimível, e o defeito seria mudo.
pub struct Vizinhanca {
    inicio: Vec<u32>,
    vizinhos: Vec<u32>,
    /// O par artificial de cada vértice, se houver (§2.4). No máximo **um**.
    pub par: Vec<Option<u32>>,
    /// A ilha (componente ligado) de cada vértice, pela topologia real.
    pub ilha: Vec<u32>,
}

impl Vizinhanca {
    /// Constrói a adjacência a partir dos anéis das faces.
    ///
    /// `escondido` marca os vértices ocultos: uma face com algum vértice oculto
    /// **não é visível** e não contribui com arestas (§9).
    /// ⚠️ **Sem fixtura:** o corpus do oráculo nunca esconde nada (§12.4 não a
    /// lista porque a §14 não tem eixo de ocultação). A regra está escrita da
    /// forma que o resto da lei pressupõe — *declarada, não medida*.
    pub fn construir(n_vertices: usize, faces: &[Vec<u32>], escondido: &[bool]) -> Vizinhanca {
        let oculto = |v: u32| escondido.get(v as usize).copied().unwrap_or(false);
        let mut listas: Vec<Vec<u32>> = vec![Vec::new(); n_vertices];
        for anel in faces {
            if anel.len() < 3 || anel.iter().any(|&v| oculto(v)) {
                continue;
            }
            let n = anel.len();
            for (k, &v) in anel.iter().enumerate() {
                if v as usize >= n_vertices {
                    continue;
                }
                let antes = anel[(k + n - 1) % n];
                let depois = anel[(k + 1) % n];
                listas[v as usize].push(antes);
                listas[v as usize].push(depois);
            }
        }
        // ⚠️ Ordenar é DETERMINISMO, não arrumação: a ordem de visita decide
        // qual vértice fica registado como «o mais afastado» no desempate do
        // §2.3, e a ordem da soma decide o último bit da média do §4.
        let mut inicio = Vec::with_capacity(n_vertices + 1);
        let mut vizinhos = Vec::new();
        inicio.push(0);
        for lista in listas.iter_mut() {
            lista.sort_unstable();
            lista.dedup();
            vizinhos.extend_from_slice(lista);
            inicio.push(vizinhos.len() as u32);
        }
        let ilha = ilhas(n_vertices, &inicio, &vizinhos);
        Vizinhanca {
            inicio,
            vizinhos,
            par: vec![None; n_vertices],
            ilha,
        }
    }

    pub fn n_vertices(&self) -> usize {
        self.inicio.len().saturating_sub(1)
    }

    /// Os vizinhos **topológicos** de um vértice — sem a ligação artificial.
    pub fn de(&self, v: u32) -> &[u32] {
        let a = self.inicio[v as usize] as usize;
        let b = self.inicio[v as usize + 1] as usize;
        &self.vizinhos[a..b]
    }

    /// §2.4 — com «só conectado» **desligado**, o grafo recebe ligações
    /// artificiais entre peças e a travessia atravessa-as.
    ///
    /// ⛔⛔ **O emparelhamento é sequencial e GANANCIOSO por ordem de índice** —
    /// não é o óptimo, e quem fica com quem depende da ordem. Reproduzimo-lo
    /// para ter paridade; ⚠️ **é um defeito conhecido e público do alvo**: com
    /// muitas peças a deformação sai com picos e inconsistente (§2.4, §11.5).
    /// *É um dos sítios nomeados onde superá-lo é ganhar — e a única forma
    /// honesta de o fazer é trocar a lei por trás de um controlo, nunca em
    /// silêncio, porque as fixturas medem esta.*
    ///
    /// ⛔ **Custo `O(V²)`** — declarado, não escondido (§13). No alvo ele é pago
    /// outra vez a cada movimento de câmara, e é essa a queixa pública; aqui é
    /// pago **uma vez por traço**.
    pub fn ligar_pecas(&mut self, posicoes: &[V3], distancia_max: f32) {
        let n = self.n_vertices();
        self.par = vec![None; n];
        for v in 0..n {
            if self.par[v].is_some() {
                continue;
            }
            let mut melhor: Option<(f32, usize)> = None;
            for u in 0..n {
                if u == v || self.par[u].is_some() || self.ilha[u] == self.ilha[v] {
                    continue;
                }
                let d = distancia(posicoes[v], posicoes[u]);
                if d >= distancia_max {
                    continue;
                }
                if melhor.is_none_or(|(md, _)| d < md) {
                    melhor = Some((d, u));
                }
            }
            if let Some((_, u)) = melhor {
                self.par[v] = Some(u as u32);
                self.par[u] = Some(v as u32);
            }
        }
    }

    /// Os vizinhos **para a travessia e o crescimento**: os topológicos mais o
    /// par artificial (§2.2, §3.1).
    pub fn para_travessia(&self, v: u32, saida: &mut Vec<u32>) {
        saida.clear();
        saida.extend_from_slice(self.de(v));
        if let Some(p) = self.par[v as usize] {
            saida.push(p);
        }
    }
}

/// Componentes ligados pela topologia real (sem ligações artificiais).
fn ilhas(n: usize, inicio: &[u32], vizinhos: &[u32]) -> Vec<u32> {
    let mut ilha = vec![u32::MAX; n];
    let mut fila = Vec::new();
    let mut proxima = 0u32;
    for raiz in 0..n {
        if ilha[raiz] != u32::MAX {
            continue;
        }
        ilha[raiz] = proxima;
        fila.push(raiz as u32);
        while let Some(v) = fila.pop() {
            let a = inicio[v as usize] as usize;
            let b = inicio[v as usize + 1] as usize;
            for &u in &vizinhos[a..b] {
                if ilha[u as usize] == u32::MAX {
                    ilha[u as usize] = proxima;
                    fila.push(u);
                }
            }
        }
        proxima += 1;
    }
    ilha
}
