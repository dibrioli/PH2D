//! Gates da multiresolução.
//!
//! ⚠️ **O gate que decide o módulo é a IDA E VOLTA EXATA.** Toda a razão de
//! guardar detalhe em vez de posições é poder descer e subir sem perder nada; se
//! a viagem custa um erro, ela custa esse erro *a cada vez*, e a escultura
//! escorrega de forma que nenhum sintoma nomeia.

use super::*;
use crate::shapes;

/// Empurra um vértice ao longo da própria normal — o "detalhe" das fixtures.
fn bump(mesh: &mut Mesh, v: usize, amount: f32) {
    let n = mesh.normals()[v];
    let p = mesh.positions()[v];
    mesh.positions_mut()[v] = [
        p[0] + n[0] * amount,
        p[1] + n[1] * amount,
        p[2] + n[2] * amount,
    ];
    mesh.rebuild();
}

fn worst(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// ⚠️ **A IDA E VOLTA É EXATA quando nada muda embaixo.**
///
/// `previsão + (topo − previsão) = topo`, desde que o frame seja o mesmo dos dois
/// lados. Um encode e um decode escritos separadamente passariam num vértice de
/// normal alinhada ao eixo e falhariam no resto — por isso o gate mede a malha
/// TODA, e por isso o frame tem uma porta só.
#[test]
fn a_round_trip_that_changes_nothing_below_is_exact() {
    let mut m = Multires::new(shapes::uv_sphere(10, 14, 1.0));
    assert!(m.add_level());
    // Detalhe autorado no topo, em vários lugares.
    for v in [3usize, 17, 42, 88] {
        bump(m.mesh_mut(), v, 0.15);
    }
    let before = m.mesh().positions().to_vec();

    assert!(m.lower());
    assert_eq!(m.level(), 0);
    assert!(m.higher());
    assert_eq!(m.level(), 1);

    let after = m.mesh().positions();
    assert_eq!(after.len(), before.len());
    let err = worst(&before, after);
    assert!(err < 1e-5, "a viagem custou {err} de deslocamento");
}

/// E ela continua exata depois de VÁRIAS viagens — um erro de 1e-6 por volta
/// seria invisível numa e visível em vinte.
#[test]
fn twenty_round_trips_do_not_drift() {
    let mut m = Multires::new(shapes::cube(2.0));
    assert!(m.add_level());
    assert!(m.add_level());
    bump(m.mesh_mut(), 5, 0.2);
    let before = m.mesh().positions().to_vec();
    for _ in 0..20 {
        m.select(0);
        m.select(2);
    }
    let err = worst(&before, m.mesh().positions());
    assert!(err < 1e-5, "vinte viagens acumularam {err}");
}

/// **Esculpir EMBAIXO move o de cima** — a metade que dá sentido a descer.
///
/// ⚠️ **Dois oráculos, e o primeiro é exato de propósito.** Transladar a base
/// inteira tem de transladar o topo pelo MESMO vetor: a tabela de pesos é afim,
/// então ela comuta com uma translação, e o detalhe é uma diferença — que uma
/// translação não muda. Isso é derivação, não um número escolhido.
///
/// ⚠️ O segundo é o empurrão LOCAL, e ali a barra é medida: um empurrão de
/// **0,4** num vértice da base chega ao topo como **0,150**, porque a regra par
/// atenua (é o `α` de Loop/Catmull-Clark). A primeira versão deste gate cravava
/// `> 0,2` — um palpite meu, e o produto estava certo.
#[test]
fn sculpting_the_base_moves_the_level_above_it() {
    // — a translação, com oráculo exato —
    let mut m = Multires::new(shapes::uv_sphere(8, 12, 1.0));
    assert!(m.add_level());
    let before = m.mesh().positions().to_vec();
    assert!(m.lower());
    for p in m.mesh_mut().positions_mut() {
        p[0] += 0.37;
    }
    m.mesh_mut().rebuild();
    assert!(m.higher());
    let worst_err = before
        .iter()
        .zip(m.mesh().positions())
        .map(|(a, b)| {
            ((b[0] - a[0] - 0.37).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max);
    assert!(
        worst_err < 1e-4,
        "transladar a base tem de transladar o topo igual, e desviou {worst_err}"
    );

    // — o empurrão local, atenuado pelo peso par —
    let mut m = Multires::new(shapes::uv_sphere(8, 12, 1.0));
    assert!(m.add_level());
    let before = m.mesh().positions().to_vec();
    assert!(m.lower());
    bump(m.mesh_mut(), 4, 0.4);
    assert!(m.higher());
    let moved = worst(&before, m.mesh().positions());
    assert!(
        (0.10..0.40).contains(&moved),
        "um empurrão de 0,4 na base chega ao topo atenuado (medido 0,150), e mediu {moved}"
    );
}

/// **E o DETALHE sobrevive à mudança da base** — a razão inteira do módulo.
///
/// O oráculo é a distância do vértice detalhado à superfície que a subdivisão
/// PORIA ali: ela é o detalhe, e tem de continuar a mesma depois de a base
/// andar.
#[test]
fn the_detail_survives_a_change_to_the_base() {
    let mut m = Multires::new(shapes::uv_sphere(8, 12, 1.0));
    assert!(m.add_level());
    const V: usize = 30;
    const D: f32 = 0.25;
    bump(m.mesh_mut(), V, D);

    // Quanto o vértice está fora da previsão, antes de mexer embaixo.
    let detail_before = {
        let p = predict(&Multires::new(shapes::uv_sphere(8, 12, 1.0)).mesh().clone());
        let q = m.mesh().positions()[V];
        ((q[0] - p.positions[V][0]).powi(2)
            + (q[1] - p.positions[V][1]).powi(2)
            + (q[2] - p.positions[V][2]).powi(2))
        .sqrt()
    };
    assert!(
        (detail_before - D).abs() < 0.02,
        "a fixture tem de ter detalhe: {detail_before}"
    );

    assert!(m.lower());
    // Translada a base INTEIRA — o teste mais limpo, porque uma translação não
    // gira frame nenhum e isola *o detalhe sobreviveu?* de *o frame girou?*.
    for p in m.mesh_mut().positions_mut() {
        p[0] += 0.5;
    }
    m.mesh_mut().rebuild();
    assert!(m.higher());

    let predicted = predict(&{
        let mut base = shapes::uv_sphere(8, 12, 1.0);
        for p in base.positions_mut() {
            p[0] += 0.5;
        }
        base.rebuild();
        base
    });
    let q = m.mesh().positions()[V];
    let detail_after = ((q[0] - predicted.positions[V][0]).powi(2)
        + (q[1] - predicted.positions[V][1]).powi(2)
        + (q[2] - predicted.positions[V][2]).powi(2))
    .sqrt();
    assert!(
        (detail_after - detail_before).abs() < 1e-4,
        "o detalhe era {detail_before} e virou {detail_after}"
    );
}

/// ⚠️ **A base compartilha os V primeiros vértices com o topo**, e é disso que
/// o `copy_shared_down` depende. Se a subdivisão numerasse os vértices novos
/// antes dos velhos, descer copiaria lixo para a base sem levantar erro.
#[test]
fn the_even_vertices_keep_their_index_through_a_subdivision() {
    let mesh = shapes::uv_sphere(9, 13, 1.0);
    let out = subdivide(&mesh);
    // Não é que as POSIÇÕES sejam iguais (a regra par as move) — é que o
    // vértice `i` de cima é o descendente do vértice `i` de baixo. O que se
    // afirma é a correspondência: cada um está mais perto do seu original do
    // que da média das arestas que o cercam.
    for v in 0..mesh.vert_count() {
        let (a, b) = (mesh.positions()[v], out.positions()[v]);
        let moved = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        assert!(
            moved < 0.2,
            "o vértice {v} andou {moved}: ele não é o descendente do original"
        );
    }
}

/// Subdividir só do topo, e a recusa é `false` — nunca uma pilha que descarta
/// trabalho em silêncio.
#[test]
fn a_level_is_only_added_from_the_top() {
    let mut m = Multires::new(shapes::cube(2.0));
    assert!(m.add_level());
    assert!(m.lower());
    assert!(!m.add_level(), "do meio, não");
    assert_eq!(m.level_count(), 2);
    assert!(m.higher());
    assert!(m.add_level(), "do topo, sim");
    assert_eq!(m.level_count(), 3);
}

/// As bordas da pilha: descer do 0 e subir do topo são no-ops que dizem `false`.
#[test]
fn the_ends_of_the_stack_refuse_instead_of_wrapping() {
    let mut m = Multires::new(shapes::cube(2.0));
    assert!(!m.lower());
    assert!(!m.higher());
    assert_eq!(m.level(), 0);
    assert!(m.add_level());
    assert!(!m.higher());
    assert_eq!(m.level(), 1);
}

/// A MÁSCARA viaja pela pilha — pintar no nível 2, descer e voltar não é uma
/// forma de perder a proteção.
#[test]
fn a_painted_mask_survives_the_round_trip() {
    let mut m = Multires::new(shapes::uv_sphere(8, 12, 1.0));
    assert!(m.add_level());
    let n = m.mesh().vert_count();
    {
        let masks = m.mesh_mut().masks_mut();
        for (i, x) in masks.iter_mut().enumerate().take(n) {
            *x = if i % 3 == 0 { 1.0 } else { 0.0 };
        }
    }
    let before = m.mesh().masks().expect("pintada").to_vec();
    m.select(0);
    m.select(1);
    let after = m.mesh().masks().expect("viaja");
    let err = before
        .iter()
        .zip(after)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(err < 1e-5, "a máscara desviou {err} na viagem");
}

/// `select` caminha os dois sentidos e para onde foi pedido.
#[test]
fn select_walks_to_the_level_it_was_asked_for() {
    let mut m = Multires::new(shapes::cube(2.0));
    for _ in 0..3 {
        assert!(m.add_level());
    }
    assert_eq!(m.level(), 3);
    m.select(0);
    assert_eq!(m.level(), 0);
    m.select(2);
    assert_eq!(m.level(), 2);
    // Fora de alcance para em quem existe, em vez de panicar.
    m.select(99);
    assert_eq!(m.level(), 3);
}

/// ⚠️ **O QUE O ARTISTA VÊ AO DESCER** — o gate que faltava, achado por mutação.
///
/// Apagar o `copy_shared_down` **sobrevive a todos os gates de viagem**: sem ele
/// a base fica como estava, a previsão sai a mesma dos dois lados, e a ida e
/// volta continua EXATA — só que descer passa a mostrar uma malha que ignora
/// tudo o que o artista esculpiu em cima. *O trabalho está guardado no detalhe e
/// invisível no lugar onde ele foi ao procurá-lo.*
#[test]
fn descending_shows_the_work_that_was_done_above() {
    let mut m = Multires::new(shapes::uv_sphere(10, 14, 1.0));
    assert!(m.add_level());
    const V: usize = 7; // um vértice que a base COMPARTILHA com o topo
    let was = m.mesh().positions()[V];
    bump(m.mesh_mut(), V, 0.3);
    let sculpted = m.mesh().positions()[V];

    assert!(m.lower());
    let base = m.mesh().positions()[V];
    let moved =
        ((base[0] - was[0]).powi(2) + (base[1] - was[1]).powi(2) + (base[2] - was[2]).powi(2))
            .sqrt();
    assert!(
        moved > 0.25,
        "a base tem de mostrar o empurrão de 0,3 e mostrou {moved}"
    );
    // E ela mostra EXATAMENTE o que o topo tem: o vértice é o mesmo objeto nos
    // dois níveis, não uma aproximação dele.
    for k in 0..3 {
        assert!(
            (base[k] - sculpted[k]).abs() < 1e-6,
            "eixo {k}: a base diz {} e o topo dizia {}",
            base[k],
            sculpted[k]
        );
    }
}
