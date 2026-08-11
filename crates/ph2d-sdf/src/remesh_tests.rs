//! Gates do botão inteiro — a porta que o shell chama.

use super::*;
use ph2d_mesh::{shapes, shapes_open};

#[test]
fn remeshing_a_sphere_keeps_the_sphere_and_reports_what_it_did() {
    let m = shapes::uv_sphere(24, 32, 1.0);
    let (out, report) = remesh(&m, 40).expect("remesh");

    assert_eq!(report.verts.0, m.vert_count());
    assert_eq!(report.verts.1, out.vert_count());
    assert_eq!(report.holes_filled, 0, "a esfera já é fechada");
    assert!(report.cells > 0);

    let mut worst = 0.0f32;
    for p in out.positions() {
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        worst = worst.max((r - 1.0).abs());
    }
    assert!(worst < 0.06, "o raio erra por {worst}");
}

/// ⚠️ Sem tapar o buraco **não haveria dentro**, e o remesh devolveria uma malha
/// vazia. Este é o gate que prende o passo 1 ao resultado.
#[test]
fn an_open_mesh_still_comes_back_as_a_body() {
    let m = shapes_open::open_tube3();
    let (out, report) = remesh(&m, 24).expect("remesh");
    assert!(report.holes_filled > 0, "não achou buraco no tubo aberto");
    assert!(out.vert_count() > 0, "voltou vazio");

    let edges = out.edges();
    let borders = (0..edges.len() as u32)
        .filter(|e| edges.valence(*e) == 1)
        .count();
    assert_eq!(borders, 0, "a saída tem {borders} arestas de beira");
}

/// ⚠️ **A entrada NÃO é modificada.** Tapar buracos é exigência do algoritmo, e
/// se ela vazasse para a malha do artista o remesh estaria editando geometria
/// que ninguém pediu — e o Ctrl+Z de fora não saberia disso.
#[test]
fn the_input_mesh_is_left_alone() {
    let m = shapes_open::open_tube3();
    let before = (m.vert_count(), m.face_count());
    let _ = remesh(&m, 16).expect("remesh");
    assert_eq!((m.vert_count(), m.face_count()), before);
}

/// Duas peças separadas entram, duas peças separadas saem — a voxelização é a
/// união, não um casco.
#[test]
fn two_separate_bodies_stay_separate() {
    let mut positions = shapes::cube(1.0).positions().to_vec();
    let faces_a = shapes::cube(1.0).faces().to_vec();
    let n = positions.len() as u32;
    let mut faces = faces_a.clone();
    for p in shapes::cube(1.0).positions() {
        positions.push([p[0] + 2.5, p[1], p[2]]);
    }
    for f in &faces_a {
        let v = f.0;
        faces.push(ph2d_mesh::Face::quad(
            v[0] + n,
            v[1] + n,
            v[2] + n,
            v[3] + n,
        ));
    }
    let both = Mesh::from_parts(positions, faces).expect("dois cubos");

    let (out, _) = remesh(&both, 40).expect("remesh");
    // Os dois cubos ficam a 2.5 de distância com lado 1: se a grade os tivesse
    // fundido, não haveria vértice nenhum na faixa entre eles.
    let left = out.positions().iter().filter(|p| p[0] < 0.8).count();
    let right = out.positions().iter().filter(|p| p[0] > 1.7).count();
    let between = out
        .positions()
        .iter()
        .filter(|p| (0.8..=1.7).contains(&p[0]))
        .count();
    assert!(left > 0 && right > 0, "esquerda {left}, direita {right}");
    assert_eq!(between, 0, "{between} vértices no vão entre os dois cubos");
}

/// **Um remesh nunca reporta SUCESSO com uma malha vazia.**
///
/// ⚠️ **As resoluções são MEDIDAS, não escolhidas.** Entre 100 e 200 há ONZE em
/// que o flood fill vaza para dentro e o campo sai sem interior — `112, 151,
/// 160, 161, 168, 180, 181, 193, 194, 196, 197` — e o default que shipa é
/// **150**, a UMA unidade da primeira. O que decide não é a resolução e sim o
/// alinhamento da grade contra os triângulos, então outra malha (outra caixa,
/// outro `step`) vaza noutros números: o 150 não é seguro, é sortudo.
///
/// ⚠️ **E o irmão [`an_open_mesh_still_comes_back_as_a_body`] já afirmava esta
/// mesma propriedade** — na resolução 24 de um tubo aberto, onde ela passa. A
/// fixture dele não continha o fenômeno; este é o mesmo `assert` com uma que
/// contém.
///
/// A afirmação é sobre o RESULTADO, não sobre o vazamento: curar o flood fill
/// faz `remesh` devolver uma malha de verdade nestas resoluções e o gate segue
/// verde — ele não pode ser silenciado pelo conserto, só pela regressão.
#[test]
fn a_remesh_never_reports_success_with_an_empty_mesh() {
    let m = shapes::uv_sphere(96, 144, 1.0);
    for res in [151u32, 320] {
        if let Ok((out, report)) = remesh(&m, res) {
            assert!(
                out.vert_count() > 0,
                "resolução {res}: `Ok` com ZERO vértices — o chamador instala isto \
                 e a escultura do artista SOME da tela com log de sucesso ({report:?})"
            );
        }
    }
}

#[test]
fn the_default_resolution_is_the_references() {
    assert_eq!(DEFAULT_RESOLUTION, 150);
    // E o atalho é o mesmo botão: uma segunda porta com um default próprio é
    // como os dois passam a reconstruir malhas diferentes.
    let m = shapes::cube(1.0);
    let (a, _) = remesh_default(&m).expect("default");
    let (b, _) = remesh(&m, DEFAULT_RESOLUTION).expect("explícito");
    assert_eq!(a.vert_count(), b.vert_count());
}

/// **A recusa NÃO dispara numa peça sadia** — o controle, e ele é a metade que
/// impede um teto de proteger recusando tudo.
///
/// A régua da [`super::RemeshError::Leaked`] é o volume que a malha encerra, e a
/// medição diz que o campo o encontra a menos de 1,1% em 361 resoluções de três
/// formas (`tests/probe_leak.rs`). Este gate pina a consequência: nas
/// resoluções do produto, uma esfera atravessa.
#[test]
fn a_healthy_piece_is_never_refused_for_leaking() {
    let m = shapes::uv_sphere(64, 96, 1.0);
    for res in [64u32, 100, DEFAULT_RESOLUTION, 200] {
        if let Err(super::RemeshError::Leaked {
            found_per_mille, ..
        }) = remesh(&m, res)
        {
            panic!(
                "resolução {res}: a recusa de vazamento disparou numa esfera SADIA \
                 ({found_per_mille}‰) — um teto que recusa o caso bom não protege nada"
            );
        }
    }
}

/// **E o volume é a régua CERTA porque as duas grandezas concordam.**
///
/// ⚠️ Este é o oráculo do limiar, e ele não pergunta ao `remesh`: mede o campo e
/// a malha separadamente. Sem ele, o gate de cima passaria com o limiar em zero
/// — *"nunca recusa"* é trivialmente verdade quando a recusa está desligada.
#[test]
fn the_field_finds_the_volume_the_mesh_encloses() {
    use crate::VoxelField;
    let mut m = shapes::uv_sphere(64, 96, 1.0);
    let _ = ph2d_mesh::fill_holes(&mut m);
    let want = ph2d_mesh::signed_volume(&m).abs();
    for res in [64u32, DEFAULT_RESOLUTION] {
        let mut f = VoxelField::for_bounds(m.bounds(), res);
        f.voxelize(&m);
        let inside = f.flood_fill();
        let s = f.step();
        let got = inside as f32 * s * s * s;
        let frac = got / want;
        assert!(
            (0.95..1.05).contains(&frac),
            "resolução {res}: o campo achou {frac:.4} do volume — a régua da recusa \
             só vale enquanto as duas grandezas concordam"
        );
    }
}

/// **O CAMPO QUE VAZA É RE-AMOSTRADO, e a peça sai inteira** — a cura, nos dois
/// casos que reproduzem.
///
/// O tubo aberto vaza em 280 e 377 (2 de 361 resoluções varridas). Antes desta
/// wave o remesh devolvia ali uma malha VAZIA com `Ok`, e o chamador a
/// instalava; a wave anterior o fez RECUSAR; esta o faz FUNCIONAR.
///
/// ⚠️ **A afirmação inclui a testemunha** (`report.nudged`), e sem ela o gate
/// ficaria verde no dia em que alguém curasse o vazamento por outra via e o
/// deslocamento virasse código morto — verde sobre uma porta que ninguém usa.
#[test]
fn a_leaking_field_is_resampled_and_the_piece_comes_back_whole() {
    let m = ph2d_mesh::shapes_open::open_tube3();
    for res in [280u32, 377] {
        let (out, report) = remesh(&m, res)
            .unwrap_or_else(|e| panic!("resolução {res}: a segunda fase devia curar -- {e}"));
        assert!(
            out.vert_count() > 1000,
            "resolução {res}: saíram {} vértices -- a peça voltou como caco",
            out.vert_count()
        );
        assert!(
            report.nudged,
            "resolução {res}: passou SEM deslocar a grade -- ou o vazamento sumiu \
             por outra via (e esta fixture parou de conter o fenômeno), ou a \
             testemunha está mentindo"
        );
    }
}

/// **E a peça SADIA nunca é deslocada** — o controle, e ele é o que impede a
/// cura de virar política.
///
/// ⚠️ Sem este gate, `for_bounds_phased` chamada SEMPRE passaria no gate de cima
/// e mudaria, em silêncio, cada malha que o remesh já devolvia — uma mudança de
/// aparência para todo mundo, vendida como conserto para dois casos.
#[test]
fn a_healthy_piece_is_never_nudged() {
    let m = shapes::uv_sphere(64, 96, 1.0);
    for res in [64u32, DEFAULT_RESOLUTION, 200] {
        let (_, report) = remesh(&m, res).expect("a esfera não recusa");
        assert!(
            !report.nudged,
            "resolução {res}: a grade foi deslocada numa peça sadia"
        );
    }
}

/// **A MÁSCARA sobrevive à reconstrução** — e é o caso de uso inteiro: mascarar
/// e depois arrumar a topologia é a ordem em que um escultor trabalha.
///
/// ⚠️ **O oráculo é a FORMA do campo, não a média dele.** Uma transferência que
/// preenchesse tudo com a média passaria em *"existe plano"* e em *"o valor está
/// na faixa"*; o que ela não reproduz é a rampa — então o gate mede os dois
/// extremos e o sinal do gradiente.
#[test]
fn an_authored_mask_survives_the_rebuild() {
    let mut mesh = shapes::uv_sphere(20, 30, 1.0);
    {
        let xs: Vec<f32> = mesh.positions().iter().map(|p| p[0]).collect();
        let m = mesh.masks_mut();
        for (i, x) in xs.iter().enumerate() {
            // 1 no lado +x, 0 no lado −x, com a fronteira no meio.
            m[i] = if *x > 0.0 { 1.0 } else { 0.0 };
        }
    }

    let (out, _) = remesh(&mesh, 64).expect("a esfera reconstrói");
    let got = out.masks().expect("a máscara atravessou o remesh");
    assert_eq!(got.len(), out.vert_count());

    // O lado +x continua mascarado e o −x continua livre.
    let mut plus = (0.0f32, 0usize);
    let mut minus = (0.0f32, 0usize);
    for (i, p) in out.positions().iter().enumerate() {
        // ⚠️ Longe da fronteira, onde a resposta é inequívoca: perto dela o
        // ponto mais próximo interpola entre os dois lados, e isso é CERTO.
        if p[0] > 0.5 {
            plus.0 += got[i];
            plus.1 += 1;
        } else if p[0] < -0.5 {
            minus.0 += got[i];
            minus.1 += 1;
        }
    }
    assert!(
        plus.1 > 10 && minus.1 > 10,
        "a fixture não tem os dois lados"
    );
    let (hi, lo) = (plus.0 / plus.1 as f32, minus.0 / minus.1 as f32);
    assert!(
        hi > 0.9 && lo < 0.1,
        "a máscara chegou chapada: +x médio {hi}, −x médio {lo}"
    );
}

/// **E uma malha VIRGEM continua saindo sem plano** — o controle.
///
/// ⚠️ Sem ele a travessia poderia materializar um plano de zeros em toda
/// reconstrução, cobrando 4 B por vértice de quem nunca mascarou e ficando
/// verde no gate acima.
#[test]
fn a_rebuild_of_a_virgin_mesh_carries_no_planes() {
    let mesh = shapes::uv_sphere(16, 24, 1.0);
    assert!(mesh.masks().is_none() && mesh.colors().is_none());
    let (out, _) = remesh(&mesh, 64).expect("a esfera reconstrói");
    assert!(
        out.masks().is_none() && out.colors().is_none(),
        "a reconstrução materializou planos que ninguém autorou"
    );
}
