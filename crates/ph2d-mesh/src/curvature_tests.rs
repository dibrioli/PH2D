//! Os gates da curvatura. O que eles protegem, em ordem de importância:
//!
//! 1. **o plano irregular é EXATAMENTE zero** — é a afirmação que autoriza o
//!    laplaciano uniforme neste uso, e sem ela a wave inteira estaria apoiada
//!    numa fama de estimador que não se aplica;
//! 2. **o número não muda com a escala** — a razão de existir o divisor;
//! 3. **o sinal** — côncavo positivo, e é o que faz *cavidade* nomear o lado
//!    certo.

use super::*;
use crate::adjacency::Adjacency;
use crate::face::Face;
use crate::mesh::{Mesh, RegionScratch};
use crate::remap::Remap;
use crate::shapes;

/// Um retalho PLANO com triangulação deliberadamente irregular: os vizinhos do
/// vértice central estão a distâncias e ângulos diferentes.
///
/// ⚠️ A irregularidade é o teste inteiro. Num anel regular *qualquer* estimador
/// acerta o zero; é a assimetria que faz o laplaciano uniforme errar — e é ela
/// que a projeção na normal descarta.
fn irregular_flat_fan() -> (Vec<[f32; 3]>, Vec<Face>) {
    // O centro é o vértice 0; os outros formam um leque em torno dele, no plano
    // z = 0, com raios de 0,3 a 2,7 e passos angulares desiguais.
    let mut positions = vec![[0.0, 0.0, 0.0]];
    let spokes: [(f32, f32); 6] = [
        (0.0, 0.31),
        (0.7, 2.70),
        (1.9, 0.55),
        (2.6, 1.80),
        (4.1, 0.42),
        (5.4, 2.20),
    ];
    for &(a, r) in &spokes {
        positions.push([r * a.cos(), r * a.sin(), 0.0]);
    }
    let mut faces = Vec::new();
    for i in 0..spokes.len() {
        let b = 1 + i as u32;
        let c = 1 + ((i + 1) % spokes.len()) as u32;
        faces.push(Face::tri(0, b, c));
    }
    (positions, faces)
}

/// **UM PLANO NÃO TEM CURVATURA, POR MAIS TORTA QUE SEJA A MALHA — e é EXATO.**
///
/// O laplaciano uniforme sobre este leque é um vetor grande (o centroide do anel
/// está longe do centro, porque os raios variam de 0,31 a 2,70). Ele é grande
/// **no plano**, e a projeção na normal o mata. É por isso que este estimador —
/// impróprio para suavizar — é o certo para ler forma.
///
/// A barra é o épsilon do `f32` sobre a magnitude do próprio laplaciano, não um
/// número escolhido: o que se afirma é *zero*, e o que se mede é o arredondamento
/// de somar seis deslocamentos.
#[test]
fn a_flat_patch_has_no_curvature_however_irregular_its_triangles() {
    let (positions, faces) = irregular_flat_fan();
    let mesh = Mesh::from_parts(positions, faces).expect("o leque e' valido");
    for (v, &k) in mesh.curvatures().iter().enumerate() {
        assert!(
            k.abs() < 1e-6,
            "o vertice {v} de um retalho PLANO leu curvatura {k}"
        );
    }
}

/// **A ESFERA LÊ CONVEXA, E O NÚMERO É METADE DO ÂNGULO DE VIRADA.**
///
/// Forma fechada: numa esfera de raio `R` amostrada com aresta `h`, o centroide
/// do anel afunda `h²/(2R)` ao longo da normal, e dividido pelo raio médio do
/// anel dá `−h/(2R)`. O gate compara com a aresta MEDIDA da própria malha, então
/// ele não depende de eu ter acertado a tesselação.
#[test]
fn a_sphere_reads_convex_and_the_number_is_half_the_turn_across_an_edge() {
    let r = 1.0f32;
    let mesh = shapes::uv_sphere(24, 48, r);
    let adj = Adjacency::build(mesh.vert_count(), mesh.faces());

    // Um vértice do EQUADOR, e ⚠️ **de valência 4**: a `uv_sphere` é uma malha de
    // QUADS, então todo vértice de corpo tem quatro vizinhos e os dois polos têm
    // 48 — um leque degenerado cuja aresta média não é a da grade. Filtrar por
    // valência alta (o palpite que escrevi primeiro) selecionava exatamente os
    // dois polos e mais nada, e o `assert` de fixture pegou.
    let equator = (0..mesh.vert_count())
        .filter(|&v| mesh.positions()[v][1].abs() < 0.05 && adj.valence(v) == 4)
        .collect::<Vec<_>>();
    assert!(
        equator.len() > 40,
        "a fixture nao contem o fenomeno: {} vertices no equador",
        equator.len()
    );

    for &v in &equator {
        let k = mesh.curvatures()[v];
        assert!(k < 0.0, "a esfera e' CONVEXA; o vertice {v} leu {k}");
        let p = mesh.positions()[v];
        let ring = adj.vert_verts.neighbours(v);
        let h: f32 = ring
            .iter()
            .map(|&j| {
                let q = mesh.positions()[j as usize];
                ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)).sqrt()
            })
            .sum::<f32>()
            / ring.len() as f32;
        let want = -h / (2.0 * r);
        assert!(
            (k - want).abs() < 0.15 * want.abs(),
            "vertice {v}: a forma fechada diz {want}, o kernel diz {k}"
        );
    }
}

/// **A MESMA FORMA, DEZ VEZES MAIOR, LÊ O MESMO NÚMERO.**
///
/// É a razão de existir a divisão pelo raio médio do anel. Sem ela a curvatura
/// tem unidade `1/comprimento`, e escalar a peça mudaria o sombreamento — o
/// artista veria a cavidade sumir ao ampliar o modelo e não teria como nomear o
/// que aconteceu.
///
/// ⚠️ A barra não é zero porque escalar por 10 muda os bits de toda coordenada;
/// o que se afirma é que a diferença é ARREDONDAMENTO e não modelo.
#[test]
fn the_number_does_not_change_when_the_model_is_scaled() {
    let small = shapes::uv_sphere(16, 32, 1.0);
    let big = shapes::uv_sphere(16, 32, 10.0);
    assert_eq!(small.vert_count(), big.vert_count());
    let mut worst = 0.0f32;
    for (a, b) in small.curvatures().iter().zip(big.curvatures()) {
        worst = worst.max((a - b).abs());
    }
    assert!(
        worst < 1e-4,
        "a mesma esfera em duas escalas divergiu em {worst} -- o divisor nao esta' cancelando"
    );
}

/// **A MESMA FORMA, MIL UNIDADES LONGE DA ORIGEM, SEGUE UTILIZÁVEL.**
///
/// ⚠️ **Este gate afirma menos do que eu queria que ele afirmasse, e a medição é
/// quem decidiu.** A intenção era pinar a escolha de acumular DESLOCAMENTOS em
/// vez de posições; medido, essa escolha vale **2 a 3×** (2,8e−4 contra 7,0e−4
/// a mil unidades), e não ordens de grandeza — porque o erro que domina já está
/// na coordenada ARMAZENADA, que nenhuma formulação conserta. As duas rotas
/// passam aqui, e a mutação que troca uma pela outra **sobrevive de propósito**:
/// a diferença é real e pequena, e um limite apertado entre 2,8e−4 e 7,0e−4
/// seria um número ajustado à mutação em vez de à propriedade.
///
/// O que ele PROTEGE é o que importa: o número não explode longe da origem —
/// ~1% da curvatura a mil unidades — em vez de virar ruído. A tabela vive no
/// [`crate::curvature`], ao lado da linha que ela justifica.
#[test]
fn the_number_does_not_change_when_the_model_is_far_from_the_origin() {
    let here = shapes::uv_sphere(16, 32, 1.0);
    let far = {
        let mut p = here.positions().to_vec();
        for q in &mut p {
            q[0] += 1000.0;
            q[2] += 1000.0;
        }
        Mesh::from_parts(p, here.faces().to_vec()).expect("so' transladou")
    };
    let mut worst = 0.0f32;
    for (a, b) in here.curvatures().iter().zip(far.curvatures()) {
        worst = worst.max((a - b).abs());
    }
    assert!(
        worst < 1e-3,
        "a mesma esfera transladada divergiu em {worst} -- a soma esta' acumulando posicoes"
    );
}

/// **CÔNCAVO É POSITIVO** — a convenção que faz a palavra *cavidade* nomear o
/// lado certo do sinal, e que o shader consome sem re-decidir.
///
/// A fixture é uma dobra em V: o mesmo leque plano, com o centro empurrado ao
/// longo da normal para um lado e para o outro. O par é o teste — um sinal
/// sozinho passaria por acidente.
#[test]
fn a_valley_reads_positive_and_a_ridge_reads_negative() {
    let (flat, faces) = irregular_flat_fan();
    let dent = |depth: f32| {
        let mut p = flat.clone();
        p[0][2] = depth;
        Mesh::from_parts(p, faces.clone()).expect("a dobra e' valida")
    };
    // A normal aponta para +Z (as faces são anti-horárias no plano), então
    // empurrar o centro para −Z abre um VALE.
    let valley = dent(-0.4);
    let ridge = dent(0.4);
    let kv = valley.curvatures()[0];
    let kr = ridge.curvatures()[0];
    assert!(kv > 0.05, "o VALE tinha de ler positivo; leu {kv}");
    assert!(kr < -0.05, "a CRISTA tinha de ler negativo; leu {kr}");
    assert!(
        (kv + kr).abs() < 0.1 * kv.abs(),
        "a dobra e' simetrica: {kv} e {kr} deviam ser opostos"
    );
}

/// **O PARALELO É BYTE-IDÊNTICO AO SERIAL** — a condição do ADR-0109 para esta
/// rota, afirmada e não prometida.
///
/// A fixture ATRAVESSA o [`PAR_MIN`]: sem isso as duas rotas seriam a mesma e o
/// gate seria verde por vácuo.
#[test]
fn the_parallel_sweep_is_byte_identical_to_the_serial_one() {
    let mesh = shapes::sphere_with_triangles(20_000, 1.0);
    assert!(
        mesh.vert_count() > PAR_MIN,
        "a fixture nao atravessa o piso do pool ({} vertices)",
        mesh.vert_count()
    );
    let adj = Adjacency::build(mesh.vert_count(), mesh.faces());
    let serial: Vec<f32> = (0..mesh.vert_count())
        .map(|v| curvature_at(mesh.positions(), mesh.normals(), &adj.vert_verts, v))
        .collect();
    assert_eq!(
        serial,
        mesh.curvatures(),
        "a rota paralela do `rebuild` discordou do laco serial"
    );
}

/// Uma esfera TRIANGULADA — a `uv_sphere` nasce em quads, e o dyntopo recusa
/// tudo que não é triângulo. ⚠️ Ela é CURVA de propósito: a grade plana que os
/// gates de colapso usam mediria curvatura zero em toda parte, e o oráculo
/// compararia dois vetores de zeros — verde por vácuo sobre a linha que ele
/// existe para proteger.
fn tri_sphere(rings: usize, segs: usize) -> Mesh {
    let mut m = shapes::uv_sphere(rings, segs, 1.0);
    m.triangulate();
    m
}

/// O oráculo das três gates de topologia: **o que um `rebuild` do zero diria**.
///
/// Ele não conhece `refresh_region`, nem o `Remap`, nem qual vértice mudou de
/// casa — ele reconstrói a malha a partir de posições e faces e compara. É a
/// mesma forma do oráculo de bit da W9.3, e é isso que o torna capaz de acusar
/// uma metade esquecida em vez de espelhá-la.
fn assert_matches_a_rebuild(mesh: &Mesh, what: &str) {
    let twin = Mesh::from_parts(mesh.positions().to_vec(), mesh.faces().to_vec())
        .expect("a malha do produto e' valida");
    // ⚠️ **OS DOIS canais, e a lista é o que torna este oráculo honesto.** Ele
    // serve as três gates de topologia; cobrir só a curvatura adimensional o
    // deixaria VERDE sobre uma porta que esqueceu a de mundo, e o sintoma seria
    // o SSS desenhando a forma de antes do traço — exatamente a classe de falha
    // que o comentário do `splice` avisa desde a W6 (*"o dia em que entrar um
    // plano por-vértice novo, quem esquecer dele é esta função"*).
    let planes: [(&str, &[f32], &[f32]); 2] = [
        ("curvatura", mesh.curvatures(), twin.curvatures()),
        ("curvatura de MUNDO", mesh.curv_world(), twin.curv_world()),
    ];
    for (name, mine, theirs) in planes {
        assert_eq!(
            mine.len(),
            theirs.len(),
            "{what}: a contagem de {name} nao bate com a de vertices"
        );
        let mut worst = (0usize, 0.0f32);
        for (v, (a, b)) in mine.iter().zip(theirs).enumerate() {
            // ⚠️ **Erro RELATIVO à escala do canal**, e não absoluto: a de mundo
            // vive em `1/comprimento` e passa de 1 numa peça de raio 1, enquanto
            // a adimensional fica em centésimos. Um limiar absoluto herdado da
            // primeira mediria as duas com a régua da menor.
            let scale = a.abs().max(b.abs()).max(1.0);
            let d = (a - b).abs() / scale;
            if d > worst.1 {
                worst = (v, d);
            }
        }
        assert!(
            worst.1 < 1e-5,
            "{what}: a {name} do vertice {} leu {} e um rebuild diria {} (erro relativo {})",
            worst.0,
            mine[worst.0],
            theirs[worst.0],
            worst.1
        );
    }
}

/// **UM DAB REFRESCA A CURVATURA DE TUDO QUE ELE MUDOU.**
///
/// A pergunta que este gate responde é de ALCANCE, não de aritmética: a lista
/// `refresh_region` foi construída para as normais, e a curvatura tem
/// dependências diferentes (ela lê o anel INTEIRO de cada vértice, não só as
/// faces dele). O gate afirma que a lista basta — porque quem está fora dela não
/// teve posição, normal nem vizinho mexido.
#[test]
fn a_dab_refreshes_every_curvature_that_it_changed() {
    let mut mesh = shapes::uv_sphere(20, 40, 1.0);
    let mut scratch = RegionScratch::default();
    // Empurra uma calota inteira para dentro — um "Draw" negativo.
    let moved: Vec<u32> = (0..mesh.vert_count() as u32)
        .filter(|&v| mesh.positions()[v as usize][1] > 0.55)
        .collect();
    assert!(moved.len() > 30, "a fixture nao contem o fenomeno");
    for &v in &moved {
        let p = &mut mesh.positions_mut()[v as usize];
        p[1] -= 0.25;
    }
    mesh.refresh_region(&moved, &mut scratch);
    assert_matches_a_rebuild(&mesh, "depois de um dab");
}

/// **UM REFINO NÃO DEIXA A CURVATURA VELHA NOS VIZINHOS.**
#[test]
fn a_refine_leaves_no_stale_curvature() {
    let mut mesh = tri_sphere(24, 36);
    let mut births = Vec::new();
    let mut scratch = RegionScratch::default();
    let out = crate::refine_in_sphere(
        &mut mesh,
        [0.0, 1.0, 0.0],
        0.6,
        0.06,
        &mut births,
        &mut scratch,
    );
    assert!(
        matches!(out, crate::Refine::Done { .. }),
        "a fixture nao refinou: {out:?}"
    );
    assert_matches_a_rebuild(&mesh, "depois de um refino");
}

/// **UMA COMPACTAÇÃO CARREGA A CURVATURA JUNTO — e este é o gate da linha que
/// eu podia ter esquecido.**
///
/// O colapso renumera: o vértice do FIM do vetor toma a casa do que morreu. Ele
/// quase nunca está na região que o `refresh_region` recomputa, então sem a
/// linha do [`Mesh::shrink_topology`] ele herdaria a curvatura do morto — uma
/// fresta de sombra pousada num lugar arbitrário da malha, longe do pincel.
///
/// ⚠️ A fixture COLAPSA de verdade (o `assert` de `verts_removed`), senão o
/// oráculo compararia duas malhas idênticas e seria verde por vácuo.
#[test]
fn a_collapse_carries_the_curvature_of_whoever_changed_house() {
    let mut mesh = tri_sphere(24, 36);
    let mut remap = Remap::default();
    let mut scratch = RegionScratch::default();
    let out = crate::collapse_in_sphere(
        &mut mesh,
        [0.0, 0.0, 0.0],
        2.0,
        crate::collapse_target(0.30),
        &mut remap,
        &mut scratch,
    );
    let removed = match out {
        crate::Collapse::Done { verts_removed, .. } => verts_removed,
        other => panic!("a fixture nao contem o fenomeno: {other:?}"),
    };
    assert!(removed > 20, "so' {removed} vertices sairam");
    assert_matches_a_rebuild(&mesh, "depois de um colapso");
}

/// **UM VÉRTICE SOLTO NÃO ENVENENA O BUFFER.**
///
/// Anel vazio não tem direção, e a alternativa — devolver `NaN` de uma divisão
/// por zero — atravessaria o vertex buffer e apagaria o triângulo inteiro no
/// rasterizador, com o sintoma a três sistemas de distância.
#[test]
fn a_loose_vertex_reads_zero_instead_of_nan() {
    let mut p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    p.push([9.0, 9.0, 9.0]); // sem face nenhuma
    let mesh = Mesh::from_parts(p, vec![Face::tri(0, 1, 2)]).expect("valida");
    let k = mesh.curvatures()[3];
    assert!(k.is_finite() && k == 0.0, "o vertice solto leu {k}");
}

/// **NUMA ESFERA DE RAIO `R`, A CURVATURA DE MUNDO É `−1/R`.** O oráculo é o raio
/// que a FIXTURE escolheu — um número que a função não conhece.
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE, e ela expôs um oráculo
/// espelho.** Trocar o denominador `Σd²` pelo quadrado do raio médio (a forma
/// "natural", que carrega um viés de Jensen de 4% num anel anisotrópico) passava
/// em todos os gates de topologia — porque eles comparam o **produto contra um
/// rebuild**, e o rebuild usa a mesma fórmula. Um oráculo que computa o esperado
/// com a função sob teste é sempre verde.
///
/// A tolerância é **1%** e o erro medido é **0,02%**: a folga cobre a
/// discretização da malha, e não caberia o viés de 4% que a mutação reintroduz.
#[test]
fn the_world_curvature_of_a_sphere_is_one_over_its_radius() {
    for r in [0.5f32, 1.0, 2.0, 4.0] {
        let mesh = {
            let mut m = shapes::uv_sphere(48, 72, r);
            m.triangulate();
            m
        };
        // A MEDIANA, e não a média: os polos de uma UV-sphere têm valência
        // diferente do resto e não são representativos da superfície.
        let mut k: Vec<f32> = mesh.curv_world().to_vec();
        k.sort_by(f32::total_cmp);
        let median = k[k.len() / 2];
        let want = -1.0 / r;
        let err = (median - want).abs() / want.abs();
        assert!(
            err < 0.01,
            "raio {r}: a curvatura de mundo leu {median:.4} e a esfera diz {want:.4} \
             (erro relativo {:.2}%) — um vies desta ordem e' a forma de denominador errada",
            err * 100.0
        );
    }
}

/// **E ela CAI PELA METADE quando a peça dobra de tamanho**, que é a propriedade
/// que a irmã adimensional NÃO tem — e a razão inteira de as duas coexistirem.
///
/// ⚠️ O gate afirma as DUAS metades sobre a mesma malha: se alguém colapsasse os
/// dois canais num só, uma das duas cairia. Um gate que só olhasse a de mundo
/// ficaria verde com o Cavity trocado por ela, e a cavidade passaria a clarear
/// quando o artista escalasse a peça — o bug que o cabeçalho deste módulo
/// descreve como *"o sombreamento mudou sozinho"*.
#[test]
fn scaling_the_piece_moves_one_channel_and_not_the_other() {
    let at = |r: f32| {
        let mut m = shapes::uv_sphere(48, 72, r);
        m.triangulate();
        let mut a: Vec<f32> = m.curvatures().to_vec();
        let mut w: Vec<f32> = m.curv_world().to_vec();
        a.sort_by(f32::total_cmp);
        w.sort_by(f32::total_cmp);
        (a[a.len() / 2], w[w.len() / 2])
    };
    let (a1, w1) = at(1.0);
    let (a2, w2) = at(2.0);
    assert!(
        (a1 - a2).abs() < 1e-6,
        "a adimensional TEM de ser invariante de escala: {a1} contra {a2}"
    );
    assert!(
        (w2 / w1 - 0.5).abs() < 0.01,
        "a de mundo TEM de cair pela metade: {w1} contra {w2} (razao {})",
        w2 / w1
    );
}
