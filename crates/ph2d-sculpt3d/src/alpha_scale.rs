//! **QUE TAMANHO DE FEATURE ESTE MODELO COMPORTA** — o seed da pista de escala.
//!
//! ⚠️ **Módulo irmão do [`super`], e a pergunta é OUTRA:** lá se responde *que
//! padrão é este*; aqui, *quão fino ele pode ser nesta malha*. A segunda é sobre
//! a MALHA e não sobre o padrão — ela lê a adjacência, mede arestas e não conhece
//! uma fórmula sequer —, e foi ela que custou o primeiro smoke desta wave
//! (*"os poros são gigantescos"*): uma escala absoluta não significa nada sem o
//! tamanho do modelo.

/// O tamanho de feature com que um alpha nasce **quando ninguém perguntou ao
/// modelo** — ver [`recommended_scale`], que é quem o produto usa.
///
/// ⚠️ **O trabalho real desta constante é ser SENTINELA**, não ser um bom
/// tamanho: é comparando contra ela que o painel sabe *"o artista ainda não
/// escolheu"* e pode semear a recomendação — o mesmo papel do `Smooth de fábrica`
/// no `arm_inflate_defaults` do Painter.
///
/// ⚠️ **Ela já foi o default de verdade, e o smoke a reprovou** (*"os poros são
/// gigantescos"*, Enio, 2026-08-05). O número era `0,25`, escolhido como *a
/// escala mais fina que a malha da cena RESOLVE* — e resolver não é parecer:
/// medido, `0,25` põe **oito** features atravessando uma esfera unitária, o que
/// lê como cratera. **Uma escala absoluta não significa nada sem o tamanho do
/// modelo**, e era esse o defeito.
pub const DEFAULT_ALPHA_SCALE: f32 = 0.06;

/// O piso da pista de escala.
///
/// Pela lei das dez arestas ele serve uma malha de aresta `0,0002` — três
/// subdivisões abaixo do que a cena mais densa deste módulo abre. Um piso mais
/// alto tiraria do artista a única faixa em que uma malha de milhões de vértices
/// vale a pena.
pub const MIN_ALPHA_SCALE: f32 = 0.002;

/// O teto da pista: quatro features atravessando um modelo de tamanho 2.
///
/// ⚠️ Acima disto o padrão deixa de ser padrão e vira um multiplicador quase
/// constante — o regime que o smoke reprovou. A pista **não** precisa alcançar
/// "uma célula do tamanho da peça": esse valor não desenha textura nenhuma.
pub const MAX_ALPHA_SCALE: f32 = 0.5;

/// **Quantas features o padrão atravessa o modelo** na recomendação.
///
/// ⚠️ **Medido, não escolhido** (`measure_how_many_features_cross_the_model`):
///
/// | escala numa esfera unitária | features | malha para resolver |
/// |---|---|---|
/// | 0,25 | **8** — a cratera que o smoke reprovou | 17 k verts |
/// | 0,12 | 17 | 86 k |
/// | **0,06** | **33 — textura** | 426 k |
/// | 0,03 | 67 — pele | 959 k |
const FEATURES_ACROSS: f32 = 33.0;

/// **A lei das dez arestas**: quantas arestas uma feature precisa medir para ser
/// amostrada como padrão em vez de como chuvisco. Ver [`DEFAULT_ALPHA_SCALE`].
const EDGES_PER_FEATURE: f32 = 10.0;

/// Quantos vértices a estimativa de aresta amostra.
///
/// ⚠️ **Amostra, e não a mediana verdadeira**, porque o retrato do painel a chama
/// a CADA QUADRO: sobre 425 k vértices a mediana exata ordena ~1,2 M
/// comprimentos, e isso é um imposto por frame para um número que só é lido num
/// clique.
///
/// **MEDIDO** (`measure_the_recommendation_per_frame`), e o número foi escolhido
/// pela varredura em vez de pelo conforto: com 2048 amostras o custo é
/// **0,11 ms/quadro**; com **384** ele cai para **0,017** — e a sonda de viés
/// (`measure_the_edge_estimator_bias`) mostra a estimativa em **1,00×** da
/// mediana verdadeira nas duas. Seis vezes mais barato pela mesma resposta.
///
/// ⚠️ **E ele é PLANO na malha** (0,017 a 153 k e a 425 k) — que é o requisito de
/// projeto, não um bônus: um seed cujo custo crescesse com o modelo seria um
/// imposto que aparece justamente na peça pesada.
const EDGE_SAMPLES: usize = 384;

/// **A escala que ESTE modelo comporta** — o seed do `Alpha Scale`.
///
/// Duas restrições, e o resultado é a mais grossa das duas porque as duas são
/// verdadeiras ao mesmo tempo:
///
/// * **o LOOK** — `maior lado ÷ 33`, que é o que faz um padrão parecer textura
///   em vez de cratera, em qualquer tamanho de modelo;
/// * **a REPRESENTABILIDADE** — `10 × aresta`, a lei das dez arestas: mais fino
///   que isso a malha não amostra o padrão, ela o pica.
///
/// ⚠️ **É por isso que uma escala absoluta precisava de um seed.** O número certo
/// é em unidades de objeto (um poro tem o tamanho de um poro, e trocar o pincel
/// não pode mudá-lo), mas *qual* número depende de duas coisas que só o modelo
/// sabe. Um literal no código acerta uma esfera unitária de uma densidade e erra
/// todo o resto — foi exatamente o que o smoke pegou.
///
/// ⚠️ **Numa malha grossa a segunda restrição VENCE**, e isso é honesto: o padrão
/// sai grosso porque a malha não comporta outro. A cura é subdividir, e é o que
/// o smoke manda fazer — o mesmo fato que faz um escultor de ZBrush subdividir
/// antes de pegar um alpha.
///
/// ⚠️ **E há um regime em que NEM o teto basta**, achado pelo gate: numa esfera
/// 24×36 a aresta mede `0,131`, então a lei das dez arestas pediria `1,31` —
/// mais que o modelo inteiro. Ali não existe escala que sirva: a malha **não
/// carrega padrão nenhum**. A recomendação pousa no teto, que é o estado
/// reconhecível; devolver um valor no meio fingiria que resolveu.
#[must_use]
pub fn recommended_scale(mesh: &ph2d_mesh::Mesh) -> f32 {
    let b = mesh.bounds();
    // ⚠️ **O MAIOR LADO, e não a diagonal da caixa** — e a diferença é um fator
    // `√3` que eu shipei errado uma vez. A tabela que fixou `FEATURES_ACROSS`
    // conta features atravessando uma esfera unitária, ou seja sobre o
    // **diâmetro** (2); a diagonal da caixa dela mede `3,464`, então o mesmo
    // número devolvia uma feature 1,73× maior que a medida. Duas réguas para uma
    // grandeza é a doença de sempre, e aqui ela sai como *"os poros continuam
    // grandes"* depois de um conserto que parecia certo.
    let span = (b.max[0] - b.min[0])
        .max(b.max[1] - b.min[1])
        .max(b.max[2] - b.min[2]);
    let look = span / FEATURES_ACROSS;
    let floor = EDGES_PER_FEATURE * sampled_edge(mesh);
    look.max(floor).clamp(MIN_ALPHA_SCALE, MAX_ALPHA_SCALE)
}

/// A aresta mediana de uma AMOSTRA de vértices — ver [`EDGE_SAMPLES`].
///
/// ⚠️ **Duas armadilhas de amostragem, as duas MEDIDAS e as duas curadas aqui.**
///
/// **(1) O passo constante ressoa com a malha.** A primeira versão andava de
/// `len / 2048` em `len / 2048`, e numa esfera UV — cujos vértices são guardados
/// por linhas de latitude — esse passo entra em ressonância com o comprimento da
/// linha: a sonda `measure_the_edge_estimator_bias` mediu o **MESMO** valor
/// (`0,0105`) para duas malhas de 153 k e 734 k vértices, um viés de **2,34×** na
/// maior. O passo agora é o da razão áurea sobre o índice, que não tem relação
/// nenhuma com a ordem em que a malha guarda os vértices.
///
/// **(2) O PRIMEIRO vizinho não é uma aresta típica.** Ele é o que o CSR pôs
/// primeiro, e numa esfera UV isso é sistematicamente o meridiano ou o raio do
/// polo. A amostra toma o **anel inteiro** de cada vértice sorteado.
fn sampled_edge(mesh: &ph2d_mesh::Mesh) -> f32 {
    let pos = mesh.positions();
    let ring = mesh.adjacency();
    if pos.is_empty() {
        return 0.0;
    }
    let n = pos.len();
    let mut lens: Vec<f32> = Vec::with_capacity(EDGE_SAMPLES * 6);
    let mut cursor: u64 = 0;
    for _ in 0..EDGE_SAMPLES.min(n) {
        cursor = cursor.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let v = ((cursor >> 33) as usize) % n;
        let a = pos[v];
        for &nb in ring.vert_verts.neighbours(v) {
            let b = pos[nb as usize];
            let e = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            lens.push(e[2].mul_add(e[2], e[0].mul_add(e[0], e[1] * e[1])).sqrt());
        }
    }
    if lens.is_empty() {
        return 0.0;
    }
    lens.sort_by(f32::total_cmp);
    lens[lens.len() / 2]
}

#[cfg(test)]
#[path = "alpha_scale_tests.rs"]
mod tests;
