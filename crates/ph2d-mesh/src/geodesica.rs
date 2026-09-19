//! ⭐⭐⭐⭐ **A DISTÂNCIA PELA SUPERFÍCIE, a sério** — marcha rápida de
//! Kimmel–Sethian sobre a malha, com tecto.
//!
//! # A pergunta que ela responde
//!
//! *Andando pelo barro, e nunca pelo ar, a que distância está este vértice
//! daquele ponto?* — e **pára** quando passa de um tecto, porque quem pergunta
//! é sempre um pincel e um pincel tem raio.
//!
//! # ⛔⛔ Porque é que ela NÃO é Dijkstra sobre as arestas
//!
//! Um caminho por arestas só pode tomar **as direcções que a malha tem**. Numa
//! grelha quadrada, ir a `45°` custa `2n` arestas onde a superfície mede
//! `n√2` ⇒ a distância sai inflada até **`√2 = 1,414`**, e o erro depende da
//! DIRECÇÃO. Duas coisas caem daí, e as duas já foram pagas neste repo:
//!
//! * as curvas de nível deixam de ser círculos e passam a ser os **losangos do
//!   grafo** — foram as *estrias* que o dono viu na transição do pincel de pose
//!   (2026-09-17), e a cura foi exactamente esta marcha;
//! * quem usa a distância como **alcance** tem de abrir o tecto para caber o
//!   viés, e um tecto aberto deixa passar o que devia cortar — é a razão
//!   escrita no [`crate::geodesica`]-irmão do pincel
//!   (`ph2d_sculpt3d::dab_alcance`), cujo `ALCANCE_TECTO` vale `2,00 × R`
//!   **por causa deste `√2`**, e que por isso não apanha uma parede fina.
//!
//! ⭐⭐ **A cura é a frente atravessar TRIÂNGULOS:** um vértice actualiza-se a
//! partir de uma FACE cujos outros dois cantos já estão fixos, e o valor sai de
//! uma quadrática que interpola a frente **dentro** da face em vez de a fazer
//! dobrar num vértice — ver [`atravessa`].
//!
//! ⚠️ **A aresta fica como TECTO, não como lei:** quando a direcção
//! característica cai fora do triângulo, a resposta certa é mesmo a aresta.
//! *Tomar sempre o mínimo é o que mantém a monotonia de que a marcha depende.*
//!
//! # ⚠️⚠️ Os cantos NÃO saem da adjacência, e isso não é detalhe
//!
//! Numa malha de **quads** — a esfera do módulo é uma, e a saída do botão de
//! retopologia também — dois vizinhos de um vértice **nunca** são vizinhos
//! entre si, logo procurar triângulos na lista de adjacência devolve **zero**
//! actualizações e a marcha degenera silenciosamente em Dijkstra. ⇒ o terceiro
//! ponto vem da **FACE** ([`Adjacency::vert_faces`]), e é lá que ele é lido.
//!
//! ⭐ Num triângulo o par é a aresta **oposta**; num quad é a **diagonal** do
//! canto — e nos dois casos o segmento está DENTRO da face, logo um caminho que
//! o atravesse é um caminho a sério sobre a superfície.
//!
//! # ⛔ A duplicação com a `ph2d-pose` é DELIBERADA, e tem gate
//!
//! A [`ph2d_pose::pesos`] tem a mesma marcha, e ela **não** passa a chamar esta:
//! aquela crate declara **zero dependências** no `Cargo.toml` dela porque *não
//! sabe o que é uma `Mesh`*, e é isso que a mantém do lado de lá da parede
//! clean-room. É o mesmo precedente que a `ph2d-boundary` já escreveu para o
//! `vetor.rs`.
//!
//! ⚠️ **O que torna a duplicação honesta é o gate de CONCORDÂNCIA**
//! (`ph2d-sculpt3d/tests/it/as_duas_marchas_concordam.rs`): as duas
//! [`atravessa`] são chamadas sobre o mesmo corpus de triângulos e têm de
//! devolver o mesmo `f32`. *Uma divergência passa a ser um portão vermelho em
//! vez de uma deriva muda.*

use crate::Mesh;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// `f32` ordenável para a fila. A distância nunca é `NaN` aqui — as arestas têm
/// comprimento finito e a semente entra a zero.
#[derive(Clone, Debug, PartialEq)]
struct Ordenavel(f32);
impl Eq for Ordenavel {}
impl PartialOrd for Ordenavel {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Ordenavel {
    fn cmp(&self, o: &Self) -> Ordering {
        self.0.partial_cmp(&o.0).unwrap_or(Ordering::Equal)
    }
}

fn comprimento(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

/// Os buffers da marcha, reusados entre chamadas.
///
/// ⚠️⚠️ **A marca de ÉPOCA é o que faz o custo ser `O(visitados)` e não
/// `O(malha)`** — a mesma disciplina do [`crate::QueryScratch`] e pela mesma
/// razão: um `vec![∞; n]` por dab escreve a malha inteira a cada evento de
/// ponteiro, e numa peça de um milhão de vértices isso é ~`1 ms` **por dab** só
/// para limpar.
#[derive(Clone, Debug, Default)]
pub struct Geodesica {
    d: Vec<f32>,
    /// `== epoca` ⇒ `d[v]` tem significado.
    visto: Vec<u32>,
    /// `== epoca` ⇒ `d[v]` está FIXO (já saiu da fila com o valor final).
    fixo: Vec<u32>,
    epoca: u32,
    fila: BinaryHeap<(std::cmp::Reverse<Ordenavel>, u32)>,
}

impl Geodesica {
    /// ⚠️ **Só para o gate da volta da época** — ela é a única maneira de correr
    /// o wrap sem esperar quatro bilhões de dabs.
    ///
    /// ⚠️ Ela atravessa a fronteira da crate (o gate do PINCEL mede o wrap pela
    /// porta do produto, e um `#[cfg(test)]` é invisível do outro lado), logo
    /// mora atrás da `test-support` — do tamanho do que atravessa, e nada mais.
    #[cfg(any(test, feature = "test-support"))]
    pub fn forcar_epoca_para_teste(&mut self, e: u32) {
        self.epoca = e;
    }

    /// **Marcha a partir do vértice `semente`, e pára no `tecto`.**
    ///
    /// ⛔⛔ **SEMEAR NO PONTO foi construído, MEDIDO e RECUSADO** (2026-09-19).
    /// A ideia era tirar um termo de erro: quem pergunta é um pincel, e o centro
    /// dele está onde o raio bateu — **dentro de uma face**, nunca em cima de um
    /// vértice —, logo semear o vértice mais próximo a zero desloca o campo por
    /// até meia aresta. As duas medições disseram que ela não paga:
    ///
    /// * **não move o planalto que escolhe o tecto do pincel** — a única célula
    ///   que se mexeu na varredura inteira foi a esfera rugosa a `1,00 × R`
    ///   (`1,68 % → 1,28 %` de peso cortado), e a borda do planalto ficou onde
    ///   estava;
    /// * **e custa PARIDADE**: medida no corpus do `Scene Project`, ela leva o
    ///   corte de `1`–`3` vértices por dab para `2`–`8`, e o placar do oráculo
    ///   desce de `13` para `12` fixturas dentro da barra. O mecanismo é que
    ///   somar `d0` a todo o campo aperta o tecto na mesma medida.
    ///
    /// ⇒ *uma correcção que não move o número que ela existe para mover, e que
    /// custa um degrau de catraca, é uma recusa medida.*
    ///
    /// ⛔ **A semente nunca é actualizada** — ela é a condição de fronteira, e
    /// deixá-la descer poria a lei a re-derivar o dado que lhe foi entregue.
    pub fn marcha(&mut self, mesh: &Mesh, semente: u32, tecto: f32) {
        let pos = mesh.positions();
        let n = pos.len();
        if self.visto.len() != n {
            self.visto = vec![0; n];
            self.fixo = vec![0; n];
            self.d = vec![0.0; n];
            self.epoca = 0;
        }
        self.epoca = self.epoca.wrapping_add(1);
        // A marca `0` é o «nunca visto» dos vectores recém-criados, então a
        // época nunca pode valer `0` — sem isto a primeira consulta depois de um
        // wrap devolveria a malha inteira como alcançada, uma vez a cada quatro
        // bilhões e impossível de reproduzir. É a mesma cerca do `QueryScratch`.
        if self.epoca == 0 {
            self.epoca = 1;
            self.visto.fill(0);
            self.fixo.fill(0);
        }
        self.fila.clear();
        if n == 0 || (semente as usize) >= n || !(tecto.is_finite() && tecto > 0.0) {
            return;
        }

        let (epoca, adj) = (self.epoca, mesh.adjacency());
        let faces = mesh.faces();
        self.visto[semente as usize] = epoca;
        self.d[semente as usize] = 0.0;
        self.fila.push((std::cmp::Reverse(Ordenavel(0.0)), semente));

        // ⭐⭐ **O LEQUE DA SEMENTE entra pela CORDA, e não pela marcha.**
        //
        // Sobre as faces que TOCAM a semente o caminho mais curto é o segmento
        // recto — ele está dentro delas por construção —, e a marcha não o sabe
        // produzir: a primeira coroa só tem a semente fixa, logo nenhuma face
        // tem os DOIS outros cantos resolvidos e a travessia nunca arma. Sem
        // isto a frente nasce quadrada e o erro que ela herda viaja para fora:
        // medido na chapa, a razão na 1.ª coroa lê `1,207` (que é a diagonal do
        // quad resolvida por dois lados) contra `1,000` com o leque.
        //
        // ⚠️ **A corda é um limite INFERIOR da geodésica** (corda ≤ arco), logo
        // numa peça curva ela lê um pouco curto — e a direcção do erro é a que
        // interessa a um pincel: ele corta de menos, nunca de mais.
        for &fi in adj.vert_faces.neighbours(semente as usize) {
            for &u in faces[fi as usize].verts() {
                let ui = u as usize;
                if u == semente || ui >= n {
                    continue;
                }
                let corda = comprimento(pos[semente as usize], pos[ui]);
                if corda <= tecto && (self.visto[ui] != epoca || corda < self.d[ui]) {
                    self.visto[ui] = epoca;
                    self.d[ui] = corda;
                    self.fila.push((std::cmp::Reverse(Ordenavel(corda)), u));
                }
            }
        }

        while let Some((std::cmp::Reverse(Ordenavel(dv)), v)) = self.fila.pop() {
            let vi = v as usize;
            // Um nó melhorado depois de entrar na fila aparece duas vezes; a
            // segunda visita morre aqui, e é isso que dispensa o `decrease-key`.
            if self.visto[vi] != epoca || dv > self.d[vi] {
                continue;
            }
            if dv > tecto {
                break;
            }
            self.fixo[vi] = epoca;
            for &u in adj.vert_verts.neighbours(vi) {
                let ui = u as usize;
                if self.fixo[ui] == epoca {
                    continue;
                }
                // (a) A ARESTA — sempre disponível, e é o tecto.
                let mut melhor = dv + comprimento(pos[vi], pos[ui]);
                // (b) As FACES que põem `v` e `u` lado a lado: o terceiro canto
                //     é o outro braço, e a frente atravessa esse triângulo.
                for &fi in adj.vert_faces.neighbours(ui) {
                    let anel = faces[fi as usize].verts();
                    let k = anel.len();
                    let Some(p) = anel.iter().position(|&x| x == u) else {
                        continue;
                    };
                    let antes = anel[(p + k - 1) % k];
                    let depois = anel[(p + 1) % k];
                    let w = if antes == v {
                        depois
                    } else if depois == v {
                        antes
                    } else {
                        continue;
                    };
                    let wi = w as usize;
                    if w == u || wi >= n || self.fixo[wi] != epoca {
                        continue;
                    }
                    if let Some(t) = atravessa(pos[ui], pos[vi], pos[wi], dv, self.d[wi]) {
                        melhor = melhor.min(t);
                    }
                }
                // ⛔⛔ **O tecto corta na ENTRADA, e não só na saída da fila.**
                // Parar de EXPANDIR ao passar do tecto não chega: quem já foi
                // relaxado fica marcado, e a marca é o que a máscara do pincel
                // lê ⇒ um anel inteiro além do tecto contava como alcançado.
                // Medido: com o corte só na saída, quatro gates de paridade de
                // oráculo desta crate reprovaram — *a cerca de uma marcha mora
                // onde ela ESCREVE, não onde ela pára.*
                if melhor > tecto {
                    continue;
                }
                if self.visto[ui] != epoca || melhor < self.d[ui] {
                    self.visto[ui] = epoca;
                    self.d[ui] = melhor;
                    self.fila.push((std::cmp::Reverse(Ordenavel(melhor)), u));
                }
            }
        }
    }

    /// **A superfície alcança este vértice dentro do tecto da última marcha?**
    #[must_use]
    pub fn alcanca(&self, v: u32) -> bool {
        self.visto.get(v as usize).copied() == Some(self.epoca) && self.epoca != 0
    }

    /// A distância pela superfície, ou `∞` em quem a marcha não alcançou.
    #[must_use]
    pub fn distancia(&self, v: u32) -> f32 {
        if self.alcanca(v) {
            self.d[v as usize]
        } else {
            f32::INFINITY
        }
    }
}

/// A frente a atravessar **um triângulo** — a actualização que torna o círculo
/// redondo (Kimmel–Sethian).
///
/// `c` é o vértice a resolver; `p` e `q` são os outros dois cantos da face, com
/// os tempos `tp` e `tq` já fixos. Devolve `None` quando a direcção
/// característica cai **fora** do triângulo — ali a resposta certa é a aresta, e
/// quem chama já a tem.
///
/// ⚠️ **As duas cercas escrevem-se MULTIPLICADAS e nunca divididas** pelo
/// cosseno: a forma clássica `a·cosθ < h < a/cosθ` inverte de sentido com um
/// ângulo obtuso em `c`, e `h·cosθ < a` é a mesma condição sem esse ramo.
#[must_use]
pub fn atravessa(c: [f32; 3], p: [f32; 3], q: [f32; 3], tp: f32, tq: f32) -> Option<f32> {
    // `a` é o canto de tempo MENOR — a quadrática é escrita a partir dele.
    let (pa, pb, ta, tb) = if tp <= tq {
        (p, q, tp, tq)
    } else {
        (q, p, tq, tp)
    };
    if !(ta.is_finite() && tb.is_finite()) {
        return None;
    }
    let u = tb - ta;
    let ea = [pa[0] - c[0], pa[1] - c[1], pa[2] - c[2]];
    let eb = [pb[0] - c[0], pb[1] - c[1], pb[2] - c[2]];
    let lb = (ea[0] * ea[0] + ea[1] * ea[1] + ea[2] * ea[2]).sqrt();
    let la = (eb[0] * eb[0] + eb[1] * eb[1] + eb[2] * eb[2]).sqrt();
    if !(la > 0.0 && lb > 0.0) {
        return None;
    }
    let cos_t = (ea[0] * eb[0] + ea[1] * eb[1] + ea[2] * eb[2]) / (la * lb);
    let sin2_t = (1.0 - cos_t * cos_t).max(0.0);
    let quad = la * la + lb * lb - 2.0 * la * lb * cos_t;
    if quad <= 0.0 {
        return None;
    }
    let lin = 2.0 * lb * u * (la * cos_t - lb);
    let cons = lb * lb * (u * u - la * la * sin2_t);
    let disc = lin * lin - 4.0 * quad * cons;
    if disc < 0.0 {
        return None;
    }
    let t = (-lin + disc.sqrt()) / (2.0 * quad);
    if t.partial_cmp(&u) != Some(Ordering::Greater) {
        return None;
    }
    let h = lb * (t - u) / t;
    if !(la * cos_t < h && h * cos_t < la) {
        return None;
    }
    Some(ta + t)
}

#[cfg(test)]
#[path = "geodesica_tests.rs"]
mod tests;
