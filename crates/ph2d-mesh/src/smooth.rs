//! **A MÉDIA DO ANEL** — a porta única de *"para onde um vértice escorrega
//! quando alguém o suaviza"*.
//!
//! Ela nasceu dentro do verbo Smooth (`ph2d-sculpt3d::stroke_target`) e saiu
//! para cá quando o **extract** passou a precisar da mesma resposta: relaxar a
//! costura de uma peça recém-extraída é o mesmo laplaciano, com as mesmas duas
//! regras de borda. Duas cópias divergiriam exatamente onde já custou uma
//! medição — a boca de uma malha aberta sendo sugada para o miolo —, e o
//! sintoma seria *"o Smooth respeita a beira e o extract não"*.
//!
//! ⚠️ **A posição chega por CLOSURE, e é isso que torna a porta compartilhável:**
//! o traço lê o `pre` **congelado** no pen-down (é o que o torna idempotente sob
//! um pincel parado) e o extract lê a posição **viva** da malha nova. A LEI é a
//! mesma; a fonte da posição não é, e um parâmetro `&Mesh` teria forçado uma das
//! duas a mentir.

use crate::adjacency::Adjacency;

/// A média das posições do anel de `v` — o alvo do Smooth, e o **oposto** do
/// Sharpen.
///
/// `base` é a posição do próprio `v`, devolvida sem alteração sempre que o anel
/// não tem o que dizer. `pos` responde *onde está o vizinho `nb`* — o traço
/// entrega o congelado, o extract entrega o vivo.
///
/// ⚠️ **Duas regras de BORDA, e as duas existem porque uma malha aberta tem
/// beira** (o `vertOnEdge` do `Mesh.js`):
///
/// 1. **Valência ≤ 2 CONGELA.** Com dois vizinhos a média é o ponto médio da
///    corda entre eles, então suavizar a ponta de uma tira a escorrega para
///    dentro da corda — a geometria some, e é um caminho só de ida.
/// 2. **Um vértice de borda medeia só com vizinhos TAMBÉM de borda.** Com o
///    anel inteiro, a média inclui os vizinhos do anel de DENTRO e a boca é
///    sugada para o miolo: medido no `open_tube3`, a altura cai de **2 para
///    1,3597** em seis passes — a peça encolhe pelas duas pontas e nada na
///    ferramenta diz por quê. Restrita à borda, os vizinhos estão no MESMO
///    anel, logo na mesma altura, e a beira alisa ao longo dela mesma.
///
/// ⚠️ **Fora de uma malha aberta as duas são inertes**: numa `uv_sphere` não há
/// vértice de borda nem valência < 3 (medido, zero de ambos). Foi por isso que a
/// classe inteira ficou invisível até existir uma fixture que a contivesse.
///
/// ⚠️ **E é a regra 2 que dá ao extract duas condutas certas com uma lei só:**
/// numa casca FECHADA (espessura ≠ 0) a costura não é borda, então o anel
/// inteiro entra e o lábio arredonda; num trecho de UMA folha a costura **é** a
/// borda, então ela desliza ao longo de si mesma e a fronteira serrilhada que a
/// máscara pintada à mão deixou se acalma **sem** o trecho encolher.
#[must_use]
pub fn ring_average(
    adj: &Adjacency,
    v: u32,
    base: [f32; 3],
    pos: impl Fn(u32) -> [f32; 3],
) -> [f32; 3] {
    let vi = v as usize;
    let ring = adj.vert_verts.neighbours(vi);
    if adj.valence(vi) <= 2 {
        return base;
    }
    let border = adj.is_border(vi);
    let mut acc = [0.0f32; 3];
    let mut n = 0u32;
    for &nb in ring {
        // Um vértice de borda só ouve a própria borda; um interior ouve tudo.
        if border && !adj.is_border(nb as usize) {
            continue;
        }
        let p = pos(nb);
        for k in 0..3 {
            acc[k] += p[k];
        }
        n += 1;
    }
    // ⚠️ `< 2` e não `== 0`: um único vizinho de borda dá uma "média" que é a
    // posição dele, e o vértice saltaria para cima do vizinho.
    //
    // ⚠️ **DEFESA EM CAMADAS, e ela é inalcançável em malha manifold — MEDIDO,
    // não suposto.** A curva de borda de uma superfície manifold é um LOOP
    // FECHADO, então todo vértice nela tem exatamente dois vizinhos de borda: no
    // `open_tube3` são 12 de 12. Só entrada não-manifold (que apenas o
    // `from_obj` pode trazer, e ele não tem chamador de produção) alcança este
    // ramo, então ele fica documentado em vez de gateado — fabricar uma quinta
    // fixture para ele seria construir a classe antes do consumidor.
    if n < 2 {
        return base;
    }
    let inv = 1.0 / n as f32;
    [acc[0] * inv, acc[1] * inv, acc[2] * inv]
}
