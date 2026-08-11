//! **OS CANAIS AUTORADOS ATRAVESSAM UMA TROCA DE TOPOLOGIA** — por PROXIMIDADE.
//!
//! Filho (`#[path]`) do [`super`] pelo motivo do `mesh_planes`: ele lê o octree,
//! que é privado da malha de propósito. Um irmão precisaria abri-lo, e aí a
//! próxima consulta nasceria fora das leis que o `verts_in_sphere` mantém.
//!
//! # Por que ele existe, e por que a lei já estava escrita
//!
//! O cabeçalho do [`crate::merge`] diz: *"a máscara e a cor viajam junto …
//! descartá-los seria destruir trabalho autorado em silêncio: uma máscara é o
//! que o artista pintou para PROTEGER, e ela some no gesto que ele achou que só
//! juntava peças"*. A fusão honra isso porque os vértices dela são uma
//! CONCATENAÇÃO — o canal viaja por índice.
//!
//! ⚠️ **O remesh não honrava, e é a mesma lei uma operação adiante.** Ele
//! devolve uma malha construída do zero pelo campo, sem plano nenhum, então
//! reconstruir a casca apagava toda máscara e toda cor — no gesto cuja razão de
//! existir é *arrumar a topologia*, que é justamente o que o artista faz DEPOIS
//! de já ter mascarado.
//!
//! # O que viaja: o AUTORADO. O que não viaja: o MEDIDO
//!
//! ⚠️ **A cor e a máscara são escolhas; o AO e a espessura são MEDIÇÕES da
//! geometria.** Carregar uma medição através de uma troca de topologia entrega
//! ao artista um número que descreve uma malha que não existe mais — e ele não
//! tem sintoma: a peça fica sombreada por uma oclusão de outra forma. O gesto
//! inverso já existe e é direto (assar de novo), e o módulo já diz isso noutro
//! lugar: o bake de AO nem entra na história, *"o que o undo guarda é GEOMETRIA,
//! e o AO é uma medição dela"*.
//!
//! # A régua é o PONTO MAIS PRÓXIMO da superfície, não o vértice mais próximo
//!
//! ⚠️ A malha nova pode ser muito mais grossa que a velha (a resolução do remesh
//! é do artista), e aí *o vértice mais próximo* dá um degrau por face — a borda
//! de uma máscara pintada com falloff sairia serrilhada. O ponto mais próximo
//! traz as BARYCÊNTRICAS junto ([`TriEdges::closest_bary`]), e o valor
//! interpolado é o que a superfície de fato tinha ali.
//!
//! # E ela roda em PARALELO — a mesma forma que a crate já paralelizou
//!
//! ⚠️ **Um `Sample` por vértice de saída é um GATHER**: função pura de entradas
//! IMUTÁVEIS (a malha de origem, os triângulos preparados, as esferas, a régua)
//! escrevendo **só o próprio slot**. É a forma que o ADR-0109 pede, a mesma das
//! normais e da curvatura deste crate (ADR-0150) e do traço de AO da irmã
//! `ph2d-sdf` (ADR-0156) — e a byte-identidade não é argumentada, é **MEDIDA**
//! contra a rota serial congelada, varrendo 1, 2, 4, 8, 16 e 32 threads.
//!
//! ⚠️ **A ESCOLHA do vencedor é por `<` estrito, e é isso que a torna imune ao
//! escalonamento:** empates ficam com o primeiro candidato, a lista de
//! candidatos vem de uma consulta que só depende do ponto (o
//! [`crate::octree::Octree::faces_in_sphere`] **limpa** a saída ao entrar), e
//! nenhuma soma atravessa thread nenhuma. O que a condição 3 do ADR-0145 recusa
//! — soma em ponto flutuante cuja ordem entre threads possa mudar — não existe
//! aqui: as três barycêntricas são somadas dentro do vértice, na mesma ordem.
//!
//! ⚠️ **E aquele `clear()` é LOAD-BEARING para esta rota de um jeito que nunca
//! foi para a serial** — medido, não suposto: removê-lo deixa **os seis gates
//! de valor VERDES** (os candidatos velhos estão mais longe e perdem, então o
//! resultado continua plausível) e derruba **só** o gate de paridade, porque as
//! duas rotas passam a acumular conjuntos diferentes. É o defeito que nenhum
//! oráculo de valor enxerga, e a razão de a varredura por número de threads
//! existir.

use rayon::prelude::*;

use crate::mesh::Mesh;
use crate::tri_geom::TriEdges;

/// Quantas vezes o raio de busca pode dobrar antes de desistir.
///
/// ⚠️ **Ele não é um teto de recurso, é uma parada:** o raio começa numa aresta
/// média e dobrar oito vezes o multiplica por 256, então uma malha cuja diagonal
/// caiba em 256 arestas médias é sempre alcançada. Além disso o que existe é
/// geometria degenerada, para a qual a resposta honesta é o default do canal.
const MAX_GROWS: u32 = 8;

/// **Leva os canais AUTORADOS de `from` para `to`, por proximidade.**
///
/// Para cada vértice de `to`, o ponto mais próximo da superfície de `from` e o
/// valor interpolado ali. Sem plano nenhum em `from` **não escreve um byte** —
/// e é isso que torna a saída de um remesh sobre malha virgem byte-idêntica ao
/// que era antes desta função existir.
///
/// ⚠️ **A malha de destino pode ser mais GROSSA ou mais fina, e não importa:**
/// a pergunta é feita por vértice de saída, contra a superfície de entrada.
pub fn transfer_authored(from: &Mesh, to: &mut Mesh) {
    transfer_by(from, to, Route::Parallel);
}

/// A travessia inteira pela rota **SERIAL** — o oráculo dos gates de identidade.
#[cfg(test)]
fn transfer_authored_serial(from: &Mesh, to: &mut Mesh) {
    transfer_by(from, to, Route::Serial);
}

/// **Qual WALKER percorre os vértices de saída** — nunca qual aritmética.
///
/// ⚠️ É o idioma do ADR-0145 (*um corpo, dois walkers*): não existe uma versão
/// paralela do kernel para divergir da serial. O corpo é o [`Probe::sample`], e
/// a rota escolhe apenas **quem o chama e em que ordem** — que é a única coisa
/// que a byte-identidade permite variar. Fora dos testes o enum tem **uma**
/// variante e o `match` é trivial.
enum Route {
    Parallel,
    #[cfg(test)]
    Serial,
}

fn transfer_by(from: &Mesh, to: &mut Mesh, route: Route) {
    let want_mask = from.masks().is_some();
    let want_color = from.colors().is_some();
    if (!want_mask && !want_color) || from.face_count() == 0 || to.vert_count() == 0 {
        return;
    }

    let mut tris: Vec<[u32; 3]> = Vec::new();
    from.triangle_indices(&mut tris);
    if tris.is_empty() {
        return;
    }
    // ⚠️ **O índice de face → triângulos é construído UMA vez.** Derivá-lo por
    // consulta seria uma varredura das faces por vértice de saída — quadrático
    // num gesto que já move milhões de vértices.
    let mut tri_start: Vec<u32> = Vec::with_capacity(from.face_count() + 1);
    let mut acc = 0u32;
    for f in from.faces() {
        tri_start.push(acc);
        acc += if f.is_tri() { 1 } else { 2 };
    }
    tri_start.push(acc);

    // ⚠️ **E os triângulos são PREPARADOS uma vez, que é para isto que o
    // [`TriEdges`] existe** — o doc dele diz: *"o consumidor prepara UM triângulo
    // e depois pergunta a ele por dezenas de voxels"*. A primeira versão desta
    // função fazia o oposto: o octree devolve as faces das FOLHAS tocadas
    // (conservador, ~75 por consulta na malha medida), e cada uma era preparada
    // de novo a cada vértice de saída — as mesmas cinco grandezas, recomputadas
    // centenas de vezes por triângulo.
    let pos = from.positions();
    let prepared: Vec<TriEdges> = tris
        .iter()
        .map(|&[a, b, c]| TriEdges::new(pos[a as usize], pos[b as usize], pos[c as usize]))
        .collect();

    // ⚠️ **A esfera envolvente de cada triângulo — o rejeito BARATO.** O octree
    // devolve as faces das FOLHAS tocadas, não as faces perto do ponto, então a
    // esmagadora maioria dos candidatos está longe demais para vencer. Testá-los
    // exatamente é pagar as sete regiões do Eberly por uma resposta que uma
    // subtração já dá.
    //
    // ⚠️ **E o rejeito é EXATO, não uma aproximação:** ele só descarta quem
    // *provadamente* não pode vencer — a menor distância possível de `p` ao
    // triângulo é `|p − centro| − raio`, e a comparação é feita sem raiz
    // (`d² > melhor + r·(r + 2·√melhor)`), com a raiz do melhor tirada uma vez
    // por melhoria em vez de uma por candidato.
    let spheres: Vec<([f32; 3], f32)> = tris
        .iter()
        .map(|&[a, b, c]| {
            let (u, v, w) = (pos[a as usize], pos[b as usize], pos[c as usize]);
            let ctr = [
                (u[0] + v[0] + w[0]) / 3.0,
                (u[1] + v[1] + w[1]) / 3.0,
                (u[2] + v[2] + w[2]) / 3.0,
            ];
            let r2 = [u, v, w]
                .iter()
                .map(|q| {
                    let d = [q[0] - ctr[0], q[1] - ctr[1], q[2] - ctr[2]];
                    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
                })
                .fold(0.0f32, f32::max);
            (ctr, r2.sqrt())
        })
        .collect();

    // A ordem de grandeza de uma aresta: a superfície cresce com a ÁREA, então
    // `lado / sqrt(triângulos)` é a régua natural. O que a torna suficiente não
    // é a precisão dela e sim o DOBRO do `nearest_triangle`.
    let seed = (from.bounds().longest_edge() / (tris.len() as f32).sqrt()).max(f32::MIN_POSITIVE);

    let probe = Probe {
        from,
        tris: &tris,
        prepared: &prepared,
        spheres: &spheres,
        tri_start: &tri_start,
        masks: from.masks(),
        colors: from.colors(),
        seed,
    };
    let samples = match route {
        Route::Parallel => probe.sample_all(to.positions()),
        #[cfg(test)]
        Route::Serial => probe.sample_all_serial(to.positions()),
    };

    // ⚠️ **O espalhamento é SERIAL de propósito, e é a divisão que o
    // `face_normals_of` desta crate já documenta:** *"computar para um vetor
    // contíguo e espalhar depois mantém a leitura pura e a escrita sequencial,
    // que é barata"*. Aqui ele custa uma cópia de campo por vértice contra uma
    // consulta de ponto-mais-próximo — não há o que colher paralelizando-o, e
    // manter a seção paralela um **map puro** é o que torna o invariante 2 do
    // ADR-0109 trivialmente verdadeiro em vez de argumentado.
    if want_mask {
        to.put_masks(samples.iter().map(|s| s.mask).collect());
    }
    if want_color {
        to.put_colors(samples.iter().map(|s| s.color).collect());
    }
}

/// O que um vértice de saída RECEBE — já interpolado, pronto para o espalhamento.
///
/// ⚠️ **Ele carrega os dois canais mesmo quando só um foi pedido**, e o preço
/// está medido: 16 B por vértice de saída, ou **19,7 MB** na malha de 1,23 M
/// vértices que um remesh a 512 devolve — contra os 922 MB que o campo daquele
/// mesmo gesto pede no pico. Separá-los por canal pouparia metade disso ao custo
/// de duas travessias ou de um `match` sobre quais planos existem, e a segunda
/// forma é como um canal novo nasce esquecido.
#[derive(Clone, Copy)]
struct Sample {
    mask: f32,
    color: [f32; 3],
}

impl Sample {
    /// O que um vértice recebe quando a busca não acha nada: o default do canal.
    const EMPTY: Self = Self {
        mask: crate::mesh::DEFAULT_MASK,
        color: crate::mesh::DEFAULT_COLOR,
    };
}

/// O que a busca precisa saber, montado uma vez.
struct Probe<'a> {
    from: &'a Mesh,
    tris: &'a [[u32; 3]],
    prepared: &'a [TriEdges],
    spheres: &'a [([f32; 3], f32)],
    tri_start: &'a [u32],
    masks: Option<&'a [f32]>,
    colors: Option<&'a [[f32; 3]]>,
    seed: f32,
}

impl Probe<'_> {
    /// **A rota que SHIPA** — um [`Sample`] por vértice de saída, em paralelo.
    ///
    /// ⚠️ **Sem piso de pool, e a ausência é MEDIDA** (`measure_the_parallel_gain`,
    /// 32 threads, pool aquecido, `load 1,89`):
    ///
    /// | vértices de saída | serial | paralelo | ganho |
    /// |---|---|---|---|
    /// | 26 | 0,013 ms | 0,028 ms | **0,45×** |
    /// | 114 | 0,039 | 0,028 | 1,38× |
    /// | 762 | 0,259 | 0,054 | 4,77× |
    /// | 3 962 | 1,633 | 0,166 | 9,84× |
    /// | 64 442 | 21,264 | 1,490 | 14,28× |
    /// | **1 230 882** (a escala do produto) | **371,4** | **24,7** | **15,04×** |
    ///
    /// O ponto de virada fica em ~60 vértices, e **abaixo dele a perda é de 15
    /// microssegundos** — enquanto o único chamador é o remesh, cuja saída mais
    /// magra (resolução 64, a mais grossa que o painel oferece) já traz **19 mil
    /// vértices**, e a mais gorda **1,23 milhão**. O piso do
    /// [`crate::normals::PAR_MIN`] existe porque um dab de DETALHE toca centenas
    /// de vértices e é um caso REAL do produto; aqui esse caso não existe, e um
    /// limiar sem caso é um número a manter em dia.
    ///
    /// ⚠️ **Três corridas, e as duas primeiras foram descartadas pela MÁQUINA:**
    /// sob `load 9` a tabela dizia 14,09× e sob `load 16` dizia 5,67×, com o
    /// lado **serial** — código intocado — a mover-se de 21,1 para 28,7 ms. *Um
    /// número que se move sobre código intocado é a máquina, não o código*; as
    /// corridas calmas reproduzem-se a poucos porcento (14,13× e 14,28×).
    ///
    /// ⚠️ **O `for_each_init` dá a cada tarefa o SEU rascunho de candidatas.**
    /// Um `Vec` partilhado seria escrita concorrente; um `Vec` novo por vértice
    /// seria uma alocação por consulta, que é o que o rascunho existe para
    /// evitar.
    fn sample_all(&self, points: &[[f32; 3]]) -> Vec<Sample> {
        let mut out = vec![Sample::EMPTY; points.len()];
        out.par_iter_mut()
            .zip(points.par_iter())
            .for_each_init(Vec::new, |faces, (o, &p)| *o = self.sample(p, faces));
        out
    }

    /// A rota **SERIAL, CONGELADA** — o oráculo do gate de identidade.
    ///
    /// ⚠️ É o código que shipava antes do `rayon` entrar aqui, verbatim, e vive
    /// sob `cfg(test)` porque um `pub` sem chamador de produto seria uma
    /// **segunda resposta** esperando quem a chame — a lição que esta casa já
    /// pagou com o `warp_axis`, o `serial_side` e o `bake_ao_serial`.
    #[cfg(test)]
    fn sample_all_serial(&self, points: &[[f32; 3]]) -> Vec<Sample> {
        let mut faces = Vec::new();
        points.iter().map(|&p| self.sample(p, &mut faces)).collect()
    }

    /// O valor da superfície de origem no ponto mais próximo de `p`.
    fn sample(&self, p: [f32; 3], faces: &mut Vec<u32>) -> Sample {
        let mut out = Sample::EMPTY;
        let Some(([a, b, c], s, t)) = self.nearest(p, faces) else {
            return out;
        };
        let w = [1.0 - s - t, s, t];
        if let Some(m) = self.masks {
            out.mask = w[0] * m[a as usize] + w[1] * m[b as usize] + w[2] * m[c as usize];
        }
        if let Some(src) = self.colors {
            let (ca, cb, cc) = (src[a as usize], src[b as usize], src[c as usize]);
            for k in 0..3 {
                out.color[k] = w[0] * ca[k] + w[1] * cb[k] + w[2] * cc[k];
            }
        }
        out
    }

    /// O triângulo de `from` mais próximo de `p`, com as barycêntricas do ponto.
    ///
    /// ⚠️ **O raio CRESCE em vez de ser adivinhado.** Um raio fixo grande varre
    /// faces demais em toda consulta; um pequeno devolve vazio, e a resposta
    /// seria o default do canal — a máscara sumindo em silêncio exatamente onde
    /// a malha é esparsa. Dobrar é o que torna a semente uma otimização e não
    /// uma premissa.
    fn nearest(&self, p: [f32; 3], faces: &mut Vec<u32>) -> Option<([u32; 3], f32, f32)> {
        let mut radius = self.seed;
        for _ in 0..MAX_GROWS {
            self.from.octree.faces_in_sphere(p, radius, faces);
            let mut best: Option<(f32, [u32; 3], f32, f32)> = None;
            // A raiz do melhor até agora — atualizada só quando o melhor MUDA,
            // que é raro depois dos primeiros candidatos.
            let mut best_sq = f32::INFINITY;
            let mut best_root = f32::INFINITY;
            for &fi in faces.iter() {
                // ⚠️ Um quad é DOIS triângulos, e a lista já os traz decompostos
                // — percorrer `Face::verts()` cru trataria o quarto índice (que
                // num triângulo é o sentinela `TRI`) como um vértice.
                let (lo, hi) = (
                    self.tri_start[fi as usize] as usize,
                    self.tri_start[fi as usize + 1] as usize,
                );
                for k in lo..hi {
                    let (ctr, r) = self.spheres[k];
                    let d = [p[0] - ctr[0], p[1] - ctr[1], p[2] - ctr[2]];
                    let dc2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                    if dc2 > best_sq + r * (r + 2.0 * best_root) {
                        continue;
                    }
                    let (sq, s, t) = self.prepared[k].closest_bary(p);
                    if sq < best_sq {
                        best_sq = sq;
                        best_root = sq.sqrt();
                        best = Some((sq, self.tris[k], s, t));
                    }
                }
            }
            if let Some((_, tri, s, t)) = best {
                return Some((tri, s, t));
            }
            radius *= 2.0;
        }
        None
    }
}

#[cfg(test)]
#[path = "mesh_transfer_tests.rs"]
mod tests;
