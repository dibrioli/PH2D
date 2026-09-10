//! ⭐⭐⭐ **A MÁSCARA DE ALCANCE** — o carimbo deixa de saltar para o que a
//! superfície não liga.
//!
//! # O defeito, medido
//!
//! Um dab junta os vértices dentro de uma **esfera** e pesa cada um pela
//! distância **pelo ar**. Numa peça com duas partes vizinhas — dois dedos, dois
//! vincos, uma dobra — a esfera alcança o outro lado. Medido pela
//! [`sonda_do_falloff_pela_superficie`](../tests/sonda_do_falloff_pela_superficie.rs)
//! em duas esferas com folga `0,05`: **`43,6 %` a `47,9 %` do peso do carimbo
//! cai no dedo ERRADO**. O controlo convexo lê `0,00 %`.
//!
//! # ⭐⭐ O que este módulo NÃO é, e é isso que o torna seguro
//!
//! ⛔ **Ele não troca a distância do falloff.** O peso continua a ser
//! `falloff(|p − c| / R)`, **ao bit**, e por isso toda a paridade com os
//! oráculos (a do SculptGL, a do tecido) fica intacta **por construção** — não
//! por promessa.
//!
//! A razão é que a pergunta que decide não é *«que distância?»* mas *«a
//! superfície liga isto?»*, e alcance não é distância. ⚠️⚠️ **E a diferença é
//! load-bearing:** este passeio anda por ARESTAS, e num quad ir por dois lados
//! onde a superfície atravessa a diagonal custa **`√2`** — medido, é exactamente
//! o `1,41×` que o controlo lê. *Um viés de `√2` num PESO seria visível em todo
//! pincel; num teste de alcance ele não entra.*
//!
//! # ⛔ O caso BRANDO fica FORA, com o motivo
//!
//! Uma parede fina (o pincel atravessa a espessura) tem razão
//! `superfície/ar = π/2 ≈ 1,57`, que é **menor** que o viés `√2` deste passeio
//! ⇒ ele não a mede. Curá-la pede geodésica a sério (método do calor, ou MMP), e
//! é outra wave com espec própria. *A classe que este módulo cura é a
//! INALCANÇÁVEL, e é a que o controlo lê a zero.*
//!
//! # ⛔⛔ O TECIDO não passa por aqui, e é estrutural
//!
//! O [`crate::Verb::Cloth`] **desvia antes do `dab_core`**
//! ([`crate::stroke_symmetry`]: ele é dono da própria expansão de simetria) ⇒ os
//! `86` traços do corpus do tecido não podem ser tocados por esta máscara. Não é
//! uma cerca que alguém tem de lembrar — é o roteamento.

use ph2d_mesh::Mesh;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// **Quantos raios de pincel a superfície tem de andar antes de se chamar
/// «não alcança»** — `2,00 × R`.
///
/// ⭐ **DERIVADO de um planalto medido** (2026-09-10), e o vale existe:
///
/// | factor | esfera convexa (APROVADO) | tubo fino (APROVADO) | dois dedos, folga `0,05` |
/// |---|---|---|---|
/// | `1,00×` | `1,94 %`–`5,09 %` ⛔ | `8,35 %`–`8,98 %` ⛔ | `49,5 %` |
/// | `1,25×` | `0,11 %`–`0,18 %` ⛔ | `0,99 %`–`1,01 %` ⛔ | `47,3 %` |
/// | `1,50×` | `0,00 %` | `0,00 %`–`0,01 %` ⛔ | `47,1 %` |
/// | **`2,00×`** | **`0,00 %`** | **`0,00 %`** | **`47,1 %`** |
/// | `3,00×` · `5,00×` | `0,00 %` | `0,00 %` | `47,1 %` |
///
/// ⇒ `2,00` é a **borda esquerda do planalto**: o menor factor que corta
/// **exactamente zero** peso do lado aprovado, com o lado do defeito já saturado
/// desde `1,50×` e sem se mover até `5,00×`. ⚠️ **O tecto tem de ser generoso
/// por causa do mesmo `√2`:** com ele em `1,00 × R` o passeio cortaria vértices
/// da borda que a superfície ALCANÇA, só por eles estarem na diagonal da grelha
/// — e é isso que aquela primeira coluna mede.
pub const ALCANCE_TECTO: f32 = 2.0;

// ⛔⛔ **UMA CONSTANTE QUE FOI CONSTRUÍDA, MEDIDA E REMOVIDA — e o registo fica.**
//
// Havia aqui um `SEMENTE_RECUO = 0,25`: o ponto de semeadura era deslocado na
// direcção do olho antes de se procurar o vértice mais próximo, para defender
// da armadilha *«o mais próximo do centro do dab está na folha ERRADA quando as
// duas se tocam a menos de uma aresta»*.
//
// ⭐ **A mutação `0,25 → 0,0` SOBREVIVEU a tudo** — os quatro gates deste módulo
// e a suíte inteira da crate (`435` verdes, `exit 0`). Das três leituras de uma
// mutação sobrevivente, a que se aplica é a terceira: *nenhuma fixtura pode
// produzir o fenómeno*, e a razão é geométrica e não uma falha de imaginação:
//
// > O centro do dab está **SOBRE** a superfície que o raio atingiu. Para duas
// > folhas paralelas — placa fina, dois dedos, uma dobra — todo vértice da
// > folha de LÁ é um vértice da folha de CÁ mais um deslocamento na espessura
// > ⇒ `√(lateral² + espessura²) > √(lateral²)`. **A folha de cá ganha sempre**,
// > para qualquer espessura positiva.
//
// ⇒ a semente é o vértice mais próximo do centro, sem recuo nenhum. *Um knob
// que nenhuma fixtura pode acordar é peso morto, e §0.0 proíbe escrever um
// número que a medição não defende.*

/// Uma entrada da fila — `f32` não é `Ord`, e o `BinaryHeap` é max-heap, então a
/// ordem é invertida aqui.
#[derive(Clone, Debug, PartialEq)]
struct Perto(f32, u32);
impl Eq for Perto {}
impl Ord for Perto {
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}
impl PartialOrd for Perto {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// Os buffers do passeio, reusados entre dabs.
///
/// ⚠️⚠️ **A marca de ÉPOCA é o que faz o custo ser `O(visitados)` e não
/// `O(malha)`** — o mesmo padrão que o [`ph2d_mesh::QueryScratch`] já usa, e pela
/// mesma razão: um `vec![∞; n]` por dab escreve a malha inteira a cada evento de
/// ponteiro, e numa peça de um milhão de vértices isso é ~`1 ms` **por dab** só
/// para limpar. ⛔ A sonda que mediu o preço faz o `resize` ingénuo de propósito
/// (ali a malha é pequena e o código é para ler); **não a copie**.
#[derive(Clone, Debug, Default)]
pub(crate) struct Alcance {
    dist: Vec<f32>,
    marca: Vec<u32>,
    epoca: u32,
    fila: BinaryHeap<Perto>,
}

fn dist2(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

impl Alcance {
    /// ⚠️ **Só para o gate da volta da época** — ela é a única maneira de correr
    /// o wrap sem esperar quatro bilhões de dabs, e a cerca que ela testa é
    /// exactamente a que o [`ph2d_mesh::QueryScratch`] documenta.
    #[cfg(test)]
    pub(crate) fn forcar_epoca_para_teste(&mut self, e: u32) {
        self.epoca = e;
    }

    /// **CORTA da `pegada` quem a superfície não alcança** dentro de
    /// `tecto = ALCANCE_TECTO × raio`, e devolve quantos saíram.
    ///
    /// ⚠️ **A `pegada` é a lista VIVA do dab**, e a ordem dos que ficam é
    /// preservada (`retain`): o `dab_core` percorre-a e a ordem de visita é lei
    /// noutros sítios desta crate.
    pub(crate) fn corta(
        &mut self,
        mesh: &Mesh,
        centro: [f32; 3],
        raio: f32,
        pegada: &mut Vec<u32>,
    ) -> usize {
        if pegada.len() < 2 || raio <= 0.0 {
            return 0;
        }
        let pos = mesh.positions();
        if self.marca.len() != pos.len() {
            self.marca = vec![0; pos.len()];
            self.dist = vec![0.0; pos.len()];
            self.epoca = 0;
        }
        self.epoca = self.epoca.wrapping_add(1);
        // A marca `0` é o «nunca visto» do vector recém-criado, então a época
        // nunca pode valer `0` — sem isto a primeira consulta depois de um wrap
        // devolveria a malha inteira como inalcançável, uma vez a cada quatro
        // bilhões e impossível de reproduzir. É a mesma cerca do `QueryScratch`.
        if self.epoca == 0 {
            self.epoca = 1;
            self.marca.fill(0);
        }

        // A semente: o vértice da pegada mais próximo do centro. ⚠️ **Da PEGADA e
        // não da malha** — o centro está sobre a superfície, logo o mais próximo
        // está na pegada por construção, e varrer a malha custaria `O(n)` por
        // dab. Ver a nota apagada acima sobre o recuo que não era preciso.
        let mut semente = pegada[0];
        let mut melhor = f32::INFINITY;
        for &v in pegada.iter() {
            let d = dist2(pos[v as usize], centro);
            if d < melhor {
                melhor = d;
                semente = v;
            }
        }

        let tecto = ALCANCE_TECTO * raio;
        self.fila.clear();
        self.marca[semente as usize] = self.epoca;
        self.dist[semente as usize] = 0.0;
        self.fila.push(Perto(0.0, semente));
        let viz = &mesh.adjacency().vert_verts;
        while let Some(Perto(du, u)) = self.fila.pop() {
            // Um nó melhorado depois de entrar na fila aparece duas vezes; a
            // segunda visita morre aqui, e é isso que dispensa o `decrease-key`.
            if du > self.dist[u as usize] {
                continue;
            }
            for &v in viz.neighbours(u as usize) {
                let nd = du + dist2(pos[u as usize], pos[v as usize]).sqrt();
                if nd > tecto {
                    continue;
                }
                let vi = v as usize;
                if self.marca[vi] != self.epoca || nd < self.dist[vi] {
                    self.marca[vi] = self.epoca;
                    self.dist[vi] = nd;
                    self.fila.push(Perto(nd, v));
                }
            }
        }

        let antes = pegada.len();
        let (marca, epoca) = (&self.marca, self.epoca);
        pegada.retain(|&v| marca[v as usize] == epoca);
        antes - pegada.len()
    }
}

#[cfg(test)]
#[path = "dab_alcance_tests.rs"]
mod tests;
