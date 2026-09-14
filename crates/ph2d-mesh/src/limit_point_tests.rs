//! **O GATE QUE CONFERE A FÓRMULA CONTRA O PROGRAMA QUE ELA DESCREVE.**

use super::{LimitPoint, limit_point};
use crate::shapes;
use crate::subdivide::subdivide;

/// A distância máxima entre o limite calculado e a posição depois de `k`
/// subdivisões, sobre os vértices que têm limite publicado.
fn erro_contra_subdivisao(mesh: &crate::Mesh, k: usize) -> (f32, usize, usize) {
    let limites: Vec<LimitPoint> = (0..mesh.vert_count())
        .map(|v| limit_point(mesh, v))
        .collect();
    let mut fino = mesh.clone();
    for _ in 0..k {
        fino = subdivide(&fino);
    }
    // ⭐ **Os vértices originais ficam nos índices `0..V`** — é uma propriedade
    // do nosso porte do `subdivide`, e é ela que torna esta comparação exacta em
    // vez de uma busca pelo mais próximo.
    let p = fino.positions();
    let mut pior = 0.0f32;
    let mut medidos = 0;
    let mut recusados = 0;
    for (v, l) in limites.iter().enumerate() {
        match l {
            LimitPoint::None => recusados += 1,
            LimitPoint::At(q) => {
                medidos += 1;
                let d = (q[0] - p[v][0])
                    .abs()
                    .max((q[1] - p[v][1]).abs())
                    .max((q[2] - p[v][2]).abs());
                pior = pior.max(d);
            }
        }
    }
    (pior, medidos, recusados)
}

/// ⭐⭐⭐ **O LIMITE É ONDE A SUBDIVISÃO DE FACTO POUSA.**
///
/// ⚠️⚠️ **É este gate que torna a espec §2.3 uma MEDIÇÃO e não uma citação.** Ela
/// dá as máscaras e manda **derivá-las** em vez de copiar uma tabela de pesos
/// por valência; aqui elas são conferidas contra o nosso próprio
/// [`crate::subdivide`] iterado — *a fórmula não é acreditada, é medida contra o
/// programa que ela descreve*.
///
/// ⛔⛔ **E foi ele que REFUTOU a espec:** com a §2.3 escrita à letra (*«ΣE a
/// soma dos pontos médios das arestas do anel e ΣF a soma dos centroides das
/// faces incidentes»*) o canto de `cube(1.0)` calculava **`0,375`** e a
/// subdivisão pousava em **`0,250`**, **estável** em `k = 4` e `k = 7`. *Um erro
/// que não encolhe com `k` não é convergência lenta: é outra superfície.*
///
/// **Medido, depois da cura** — a distância ao limite **encolhe** a cada
/// subdivisão, que é a assinatura da convergência:
///
/// | peça | esquema | `k = 4` | `k = 7` |
/// |---|---|---|---|
/// | `cube(1,0)` | CC, valência 3 | `1,29e-4` | **`5,66e-7`** |
/// | `octahedron(1,0)` | Loop, valência 4 | `1,95e-3` | **`3,05e-5`** |
/// | `uv_sphere(12,16)` | CC + Loop nos pólos | `1,41e-4` | **`2,21e-6`** |
///
/// ⭐ **A esfera UV é a peça que discrimina:** ela tem valências `4`, `5` e `6`
/// no mesmo corpo, logo um peso errado numa delas aparece como um vértice que
/// não converge.
#[test]
fn o_limite_e_onde_a_subdivisao_de_facto_pousa() {
    for (nome, mesh) in [
        ("cube (CC, valencia 3)", shapes::cube(1.0)),
        ("octahedron (Loop, valencia 4)", shapes::octahedron(1.0)),
        (
            "uv_sphere 12x16 (CC + Loop)",
            shapes::uv_sphere(12, 16, 1.0),
        ),
        // ⛔⛔ **AS DUAS DE BAIXO NASCERAM DE MUTAÇÕES SOBREVIVENTES**, e as duas
        // tinham a MESMA causa: *o corpus não continha a configuração*.
        // - trocar o β de **Warren** (`n = 3`) pela fórmula geral do Loop
        //   passava, porque as três peças de cima **não têm um vértice de
        //   triângulos com valência 3** (a esfera tem `4`, `5` e `6`; o octaedro
        //   tem `4`);
        // - fazer o **BORDO** ouvir o anel de dentro passava, porque as três são
        //   **fechadas** e a máscara da corda nunca corria.
        (
            "tetraedro (Loop, valencia 3 -- o beta de Warren)",
            tetraedro(),
        ),
        ("retalho ABERTO (a corda do bordo)", retalho_aberto()),
    ] {
        let (e4, medidos, recusados) = erro_contra_subdivisao(&mesh, 4);
        let (e7, _, _) = erro_contra_subdivisao(&mesh, 7);
        assert!(
            medidos > 0,
            "`{nome}`: nenhum vértice tem limite publicado ({recusados} recusados) \
             — a fixtura não contém o fenómeno"
        );
        // ⚠️⚠️ **A barra é a CONVERGÊNCIA e não um epsilon escolhido**, e é ela
        // que apanha a classe de erro que uma máscara errada produz: um peso
        // errado **estabiliza** numa distância ≠ 0 em vez de encolher. Medido, a
        // razão é de `~50×` a `~230×` entre `k = 4` e `k = 7`; a barra pede
        // apenas `4×`, com folga de uma ordem de grandeza.
        assert!(
            e7 * 4.0 < e4,
            "`{nome}`: a distância ao limite NÃO encolheu ({e4:e} a k=4, {e7:e} a \
             k=7) — uma máscara errada estabiliza, e é exactamente assim que a \
             leitura da espec §2.3 foi refutada"
        );
        assert!(
            e7 < 1e-4,
            "`{nome}`: a subdivisão pousou a {e7:e} do limite calculado — um peso \
             da máscara não é o do nosso `even`"
        );
    }
}

/// ⭐⭐ **O LIMITE NÃO É A PREVISÃO, e a diferença é a razão de este módulo
/// existir** (espec §2.1).
///
/// ⛔ Se `subdivide^k(base)` fosse o limite, um apagador que repusesse o vértice
/// na previsão daria o mesmo resultado — e daria, num gate que só medisse *«o
/// vértice mexeu-se»*. **Medido no canto de `cube(1,0)`** (coordenada original
/// `0,5`):
///
/// | superfície | coordenada do canto |
/// |---|---|
/// | a malha de partida | `0,5000` |
/// | **um passo** de subdivisão (a previsão) | `0,2778` |
/// | ⭐ o **LIMITE** | **`0,2500`** |
///
/// ⇒ a previsão fica **`11 %` acima** do limite, e um apagador que a usasse
/// deixaria esse resíduo a cada passagem. *É por isso que a §4.4 diz, com todas
/// as letras, que a versão barata é OUTRO pincel.*
#[test]
fn o_limite_nao_e_a_previsao_de_um_passo() {
    let cubo = shapes::cube(1.0);
    let partida = extremo_dos_originais(&cubo, cubo.vert_count());
    let limite = (0..cubo.vert_count())
        .map(|v| match limit_point(&cubo, v) {
            LimitPoint::At(q) => q[0].abs().max(q[1].abs()).max(q[2].abs()),
            LimitPoint::None => panic!("o cubo é todo quads — não podia recusar"),
        })
        .fold(0.0f32, f32::max);
    let previsao = extremo_dos_originais(&subdivide(&cubo), cubo.vert_count());

    assert!(
        (partida - 0.5).abs() < 1e-6,
        "a fixtura não é o cubo que este gate descreve: {partida}"
    );
    assert!(
        (limite - 0.25).abs() < 1e-4,
        "o limite do canto do cubo é {limite:.4} e a medição deu 0,2500"
    );
    // ⭐ **O controlo que dá sentido ao gate:** a previsão fica ENTRE os dois, e
    // estritamente acima do limite.
    assert!(
        previsao > limite + 0.02 && previsao < partida,
        "a previsão de um passo ({previsao:.4}) não está entre o limite \
         ({limite:.4}) e a partida ({partida:.4}) — sem essa folga o apagador \
         podia usar a previsão, e a espec §2.1 diz que isso ENCOLHE a peça"
    );
}

/// O maior valor absoluto de coordenada entre os `n` primeiros vértices.
fn extremo_dos_originais(mesh: &crate::Mesh, n: usize) -> f32 {
    mesh.positions().iter().take(n).fold(0.0f32, |a, p| {
        a.max(p[0].abs().max(p[1].abs()).max(p[2].abs()))
    })
}

/// ⛔ **O ANEL MISTO RECUSA, e a recusa é a resposta certa** (ver o cabeçalho).
///
/// ⚠️⚠️ **A fixtura tem de CONTER o fenómeno, e a primeira NÃO continha:** um
/// quad colado a um triângulo põe os cinco vértices no BORDO, e ali a máscara é
/// a da corda — nenhum deles chega sequer à pergunta do misto. É preciso um
/// vértice **INTERIOR** cercado por faces das duas espécies, e é isso que este
/// leque fechado de `2` quads + `2` triângulos constrói.
///
/// ⛔ E ele afirma os DOIS lados: o misto recusa **e** o resto da mesma malha
/// responde — senão um `None` cravado ficaria verde.
#[test]
fn um_anel_misto_recusa_e_o_resto_da_mesma_malha_responde() {
    // Um leque FECHADO à volta do vértice `0`: quad · tri · tri · quad.
    let mesh = crate::Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],  // 0 — o interior, de anel MISTO
            [1.0, 0.0, 0.0],  // 1
            [1.4, 1.4, 0.0],  // 2 — a diagonal do 1.º quad
            [0.0, 1.0, 0.0],  // 3
            [-1.0, 0.0, 0.0], // 4
            [0.0, -1.0, 0.0], // 5
            [1.4, -1.4, 0.0], // 6 — a diagonal do 2.º quad
        ],
        vec![
            crate::Face::quad(0, 1, 2, 3),
            crate::Face::tri(0, 3, 4),
            crate::Face::tri(0, 4, 5),
            crate::Face::quad(0, 5, 6, 1),
        ],
    )
    .expect("a fixtura é uma malha legal");

    let adj = mesh.adjacency();
    assert!(
        !adj.is_border(0),
        "o vértice do leque ficou no BORDO — a fixtura não contém o fenómeno, \
         que é um anel misto INTERIOR"
    );
    assert!(
        limit_point(&mesh, 0).is_none(),
        "o anel misto devolveu um limite — o nosso `even` interpola dois \
         esquemas ali, e essa mistura não tem limite publicado"
    );
    let respondem = (1..mesh.vert_count())
        .filter(|&v| !limit_point(&mesh, v).is_none())
        .count();
    assert!(
        respondem > 0,
        "nenhum outro vértice respondeu — a máscara de bordo deixou de funcionar, \
         e um `None` cravado passaria a metade de cima deste gate"
    );
}

/// **Um TETRAEDRO** — quatro triângulos, e todo vértice com valência `3`.
///
/// ⚠️ É a única configuração em que o nosso `even` usa o **β de Warren**
/// (`0,1875`) em vez da fórmula geral do Loop, e nenhuma das peças do
/// `shapes.rs` a contém. *A mutação que o trocou pela fórmula geral sobreviveu
/// até esta fixtura existir.*
fn tetraedro() -> crate::Mesh {
    crate::Mesh::from_parts(
        vec![
            [1.0, 1.0, 1.0],
            [1.0, -1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
        ],
        vec![
            crate::Face::tri(0, 1, 2),
            crate::Face::tri(0, 2, 3),
            crate::Face::tri(0, 3, 1),
            crate::Face::tri(1, 3, 2),
        ],
    )
    .expect("o tetraedro é uma malha legal")
}

/// **Um retalho 3×3 de quads com BORDO e relevo.**
///
/// ⚠️ **O relevo é obrigatório:** um retalho PLANO é a sua própria superfície
/// limite, logo o erro seria `0` por construção e o gate mediria o nada — a
/// mesma armadilha do anti-vácuo que este repo varre a cada wave. A sela tem
/// curvatura de sinais opostos nos dois eixos, então nenhuma simetria colapsa o
/// erro.
fn retalho_aberto() -> crate::Mesh {
    let n = 4usize;
    let idx = |i: usize, j: usize| (j * n + i) as u32;
    let mut p = Vec::new();
    for j in 0..n {
        for i in 0..n {
            let (x, y) = (i as f32 - 1.5, j as f32 - 1.5);
            p.push([x, y, 0.175 * (x * x - y * y)]);
        }
    }
    let mut f = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            f.push(crate::Face::quad(
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ));
        }
    }
    crate::Mesh::from_parts(p, f).expect("o retalho é uma malha legal")
}
