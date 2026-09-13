//! **ONDE UM GESTO PEGA O BARRO** — a âncora do pen-down, e a opção que a põe
//! num VÉRTICE em vez de num ponto qualquer da superfície.
//!
//! ⚠️ **É uma PORTA, e não um `if` no app, por uma razão medida:** o gate que
//! compara o agarrar com o oráculo conduz o kernel directamente, sem raio nem
//! câmara. Se a escolha da âncora vivesse no app, o gate mediria uma âncora que
//! o produto não usa — *uma bancada que não chama a lei do produto não prova
//! nada sobre o produto*.

use ph2d_mesh::Mesh;

/// **A ÂNCORA DO GESTO**, em espaço de objecto.
///
/// Com `no_vertice` desligado é o ponto que o raio acertou; ligado, é a
/// **posição do vértice** mais próximo desse ponto.
///
/// ⭐ **O que a opção compra, medido no oráculo** (uma grelha grossa de `8×8`):
/// o pico do gesto passa de `0,446338` para `0,500001` — ou seja, passa a ser
/// **exactamente** o deslocamento pedido, porque a âncora cai em cima de um
/// vértice e esse vértice recebe peso `1`. Um vértice a mais entra na pegada
/// (`8 → 9`).
///
/// ⚠️ **Numa malha densa a diferença tende a zero** — ela existe para malhas
/// grossas e para o artista poder apontar a um vértice concreto.
///
/// ⚠️ **APROXIMAÇÃO NOMEADA:** a referência toma *o vértice activo*, que é o
/// que o raycast dela elege na face acertada; nós tomamos o mais próximo do
/// ponto de acerto. Nos casos em que a face é convexa e o ponto está dentro
/// dela os dois coincidem, e a fixture da grelha grossa — o pior caso, onde uma
/// face tem meio raio de pincel — confirma-o. ⛔ Não é *«o mesmo por
/// definição»*: é *«o mesmo onde foi medido»*.
#[must_use]
pub fn ancora_do_gesto(mesh: &Mesh, ponto: [f32; 3], no_vertice: bool) -> [f32; 3] {
    if !no_vertice {
        return ponto;
    }
    vertice_mais_proximo(mesh, ponto).map_or(ponto, |v| mesh.positions()[v as usize])
}

/// O vértice mais próximo de um ponto, ou `None` numa malha vazia.
///
/// ⚠️ **Varredura linear, e ela é honesta aqui:** isto corre **uma vez por
/// traço** (no pen-down), não por dab nem por quadro. A alternativa — descer à
/// octree — pagaria uma consulta de raio crescente para responder a uma
/// pergunta que já tem a resposta em `O(V)` com uma passagem sequencial sobre
/// memória contígua.
#[must_use]
pub fn vertice_mais_proximo(mesh: &Mesh, ponto: [f32; 3]) -> Option<u32> {
    let mut melhor: Option<(u32, f32)> = None;
    for (i, p) in mesh.positions().iter().enumerate() {
        let (dx, dy, dz) = (p[0] - ponto[0], p[1] - ponto[1], p[2] - ponto[2]);
        let d2 = dx * dx + dy * dy + dz * dz;
        // ⚠️ **`<` estrito, e o empate fica com o PRIMEIRO** — numa grelha
        // regular há empates exactos (o centro de uma célula está à mesma
        // distância dos quatro cantos), e um `<=` faria a escolha depender da
        // ordem de iteração de uma forma que ninguém declarou.
        if melhor.is_none_or(|(_, b)| d2 < b) {
            melhor = Some((i as u32, d2));
        }
    }
    melhor.map(|(v, _)| v)
}
