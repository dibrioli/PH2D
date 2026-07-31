//! Gates do CORTE ([`super`]) — o primitivo das quatro ferramentas da W4.
//!
//! ⚠️ **As fixtures carregam HANDLES de verdade**, não polilinhas de âncoras soltas: metade destes
//! gates é sobre o que acontece com a tangente do vértice cortado, e uma fixture de handles zerados
//! não contém o fenômeno (todo bug de handle seria indistinguível do certo).

use super::*;
use crate::{FillRule, VecPath, VecVertex, VertexKind};

/// Um vértice com handles DISTINTOS e não-degenerados — o `in` e o `out` diferem entre si e do
/// anchor, então trocá-los, zerá-los ou perdê-los é observável.
fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x - 0.3, y - 0.7],
        out_handle: [x + 0.5, y + 0.2],
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    }
}

fn closed_square() -> VecPath {
    VecPath {
        verts: vec![v(0.0, 0.0), v(4.0, 0.0), v(4.0, 4.0), v(0.0, 4.0)],
        closed: true,
        ..VecPath::default()
    }
}

fn open_chain(n: usize) -> VecPath {
    VecPath {
        verts: (0..n).map(|i| v(i as f64, 0.0)).collect(),
        closed: false,
        ..VecPath::default()
    }
}

#[test]
fn a_closed_contour_opens_at_the_cut_and_the_seam_vertex_appears_twice() {
    let mut scene = VecScene::default();
    let id = scene.push_path(closed_square());
    let before = scene.paths()[0].verts[2];

    let cut = scene.cut_path_at_vertex(id, 2).expect("corte no vértice 2");

    assert!(cut.opened, "um contorno fechado ABRE, não parte");
    assert_eq!(cut.new_path, None, "abrir não cria objeto");
    assert_eq!(scene.paths().len(), 1, "abrir não cria objeto");
    let p = &scene.paths()[0];
    assert!(!p.closed);
    assert_eq!(p.verts.len(), 5, "4 vértices + a costura duplicada");
    assert_eq!(p.verts[0].anchor, before.anchor);
    assert_eq!(p.verts[4].anchor, before.anchor);
}

/// A propriedade que torna o corte reversível: a costura carrega os DOIS handles nas duas cópias,
/// então re-fechar devolve a curva ao bit. Zerar o handle "morto" pareceria higiene e destruiria a
/// tangente autorada.
#[test]
fn the_seam_carries_both_handles_so_re_closing_restores_the_curve() {
    let mut scene = VecScene::default();
    let original = closed_square();
    let id = scene.push_path(original.clone());
    scene.cut_path_at_vertex(id, 2).unwrap();

    let p = &scene.paths()[0];
    assert_eq!(
        p.verts[0], p.verts[4],
        "as duas cópias da costura são IGUAIS"
    );

    // Re-fechar = tirar a duplicata e marcar fechado. Tem de dar a rotação exata do original.
    let mut re = p.clone();
    re.verts.pop();
    re.closed = true;
    let mut expect = original.verts.clone();
    expect.rotate_left(2);
    assert_eq!(re.verts, expect, "re-fechar devolve a curva original");
}

#[test]
fn an_open_contour_splits_in_two_and_the_cut_vertex_is_in_both() {
    let mut scene = VecScene::default();
    let id = scene.push_path(open_chain(5));

    let cut = scene.cut_path_at_vertex(id, 2).expect("corte interior");

    assert!(!cut.opened);
    let new_id = cut.new_path.expect("a 2ª metade vira objeto");
    assert_eq!(scene.paths().len(), 2);
    let a = scene.paths().iter().find(|p| p.id == id).unwrap();
    let b = scene.paths().iter().find(|p| p.id == new_id).unwrap();
    assert_eq!(a.verts.len(), 3, "[0,1,2]");
    assert_eq!(b.verts.len(), 3, "[2,3,4]");
    assert_eq!(
        a.verts[2].anchor, b.verts[0].anchor,
        "as duas metades encostam onde a tesoura passou"
    );
    assert!(!a.closed && !b.closed);
}

/// Duas tesouradas num contorno FECHADO dão DUAS peças abertas — é o que a cena de smoke afirma,
/// e é a composição que faz a faca funcionar sem geometria própria.
#[test]
fn two_cuts_on_a_closed_contour_leave_two_open_pieces() {
    let mut scene = VecScene::default();
    let id = scene.push_path(closed_square());

    scene.cut_path_at_vertex(id, 0).unwrap();
    // Depois de abrir em 0 o contorno é [0,1,2,3,0]; o vértice 2 continua interior.
    let cut = scene.cut_path_at_vertex(id, 2).unwrap();

    assert_eq!(scene.paths().len(), 2);
    assert!(cut.new_path.is_some());
    for p in scene.paths() {
        assert!(!p.closed, "as duas peças ficam ABERTAS");
        assert!(p.verts.len() >= 2);
    }
}

#[test]
fn cutting_the_end_of_an_open_contour_is_refused() {
    let mut scene = VecScene::default();
    let id = scene.push_path(open_chain(4));
    assert_eq!(scene.cut_path_at_vertex(id, 0), None, "a ponta inicial");
    assert_eq!(scene.cut_path_at_vertex(id, 3), None, "a ponta final");
    assert_eq!(scene.cut_path_at_vertex(id, 9), None, "fora do alcance");
    assert_eq!(scene.paths()[0].verts.len(), 4, "nada foi tocado");
}

#[test]
fn a_degenerate_contour_is_refused() {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0)],
        closed: true,
        ..VecPath::default()
    });
    assert_eq!(scene.cut_path_at_vertex(id, 0), None);
}

#[test]
fn the_new_half_inherits_the_style_and_the_effect_stack() {
    let mut scene = VecScene::default();
    let mut src = open_chain(5);
    src.stroke = Some(crate::StrokeSpec::new(
        crate::Rgba8::new(10, 20, 30, 255),
        3.5,
    ));
    src.fill = Some(crate::Paint::Solid(crate::Rgba8::new(40, 50, 60, 255)));
    src.effects.push(crate::effect::FxEntry::new(
        crate::effect::PathEffect::Trim(crate::fx_trim::TrimSpec {
            start: 0.1,
            end: 0.9,
            offset: 0.0,
        }),
    ));
    let id = scene.push_path(src);

    let new_id = scene.cut_path_at_vertex(id, 2).unwrap().new_path.unwrap();

    let a = scene.paths().iter().find(|p| p.id == id).unwrap();
    let b = scene.paths().iter().find(|p| p.id == new_id).unwrap();
    assert_eq!(a.stroke, b.stroke, "o traço é o mesmo");
    assert_eq!(a.fill, b.fill, "o preenchimento é o mesmo");
    assert_eq!(a.effects.len(), b.effects.len(), "a pilha é APARÊNCIA");
    assert_eq!(a.effects, b.effects);
}

/// A metade nova nasce logo ACIMA da fonte: as duas ficam vizinhas na pilha, e nenhuma salta para
/// a frente da cena (o defeito que o blend pagou em 2026-07-14).
#[test]
fn the_new_half_sits_directly_above_the_source_in_z() {
    let mut scene = VecScene::default();
    let below = scene.push_path(open_chain(3));
    let id = scene.push_path(open_chain(5));
    let above = scene.push_path(open_chain(3));

    let new_id = scene.cut_path_at_vertex(id, 2).unwrap().new_path.unwrap();

    let z: Vec<_> = scene.paths().iter().map(|p| p.id).collect();
    assert_eq!(z, vec![below, id, new_id, above]);
}

/// Num COMPOUND o corte fica em casa: separar um contorno em dois objetos mudaria o que a
/// `FillRule` significa — o buraco deixaria de ser buraco no clique que era para ser um corte.
#[test]
fn a_compound_contour_is_cut_in_place_and_stays_one_object() {
    let mut scene = VecScene::default();
    let mut donut = closed_square();
    donut.fill_rule = FillRule::EvenOdd;
    donut.subpaths.push(Contour::new_closed(vec![
        v(1.0, 1.0),
        v(3.0, 1.0),
        v(3.0, 3.0),
        v(1.0, 3.0),
    ]));
    let id = scene.push_path(donut);

    // Índice PLANO 5 = o 2º vértice do BURACO (4 do primário + 1).
    let cut = scene.cut_path_at_vertex(id, 5).expect("corta o buraco");

    assert_eq!(scene.paths().len(), 1, "continua UM objeto");
    assert_eq!(cut.new_path, None);
    assert!(cut.opened, "o buraco era fechado: ele ABRE");
    let p = &scene.paths()[0];
    assert_eq!(p.contour_count(), 2, "nenhum contorno nasceu");
    assert_eq!(p.verts.len(), 4, "o primário não foi tocado");
    let (hole, closed) = p.contour(1).unwrap();
    assert!(!closed);
    assert_eq!(hole.len(), 5, "o buraco ganhou a costura");
    assert_eq!(hole[0].anchor, [3.0, 1.0], "cortado no vértice PEDIDO");
}

/// O mesmo compound, mas com o contorno ABERTO: a 2ª metade vira contorno IRMÃO, e o objeto segue
/// um só. É o caso que separa `locate_vert` correto de um índice plano lido como local.
#[test]
fn an_open_compound_contour_splits_into_a_sibling_contour() {
    let mut scene = VecScene::default();
    let mut p = closed_square();
    p.subpaths.push(Contour {
        verts: (0..5).map(|i| v(10.0 + i as f64, 10.0)).collect(),
        closed: false,
    });
    let id = scene.push_path(p);

    // Plano 6 = o 3º vértice do subpath aberto (4 do primário + 2) — o do meio.
    let cut = scene.cut_path_at_vertex(id, 6).expect("corta o subpath");

    assert_eq!(scene.paths().len(), 1, "compound não vira dois objetos");
    assert_eq!(cut.new_contour, Some(2));
    let p = &scene.paths()[0];
    assert_eq!(p.contour_count(), 3);
    assert_eq!(p.contour(1).unwrap().0.len(), 3, "[10,11,12]");
    assert_eq!(p.contour(2).unwrap().0.len(), 3, "[12,13,14]");
    assert_eq!(p.contour(1).unwrap().0[2].anchor, [12.0, 10.0]);
    assert_eq!(p.contour(2).unwrap().0[0].anchor, [12.0, 10.0]);
}

#[test]
fn cutting_an_unknown_path_is_refused() {
    let mut scene = VecScene::default();
    scene.push_path(open_chain(4));
    assert_eq!(scene.cut_path_at_vertex(999, 1), None);
}

// ── A LÂMINA ────────────────────────────────────────────────────────────────

/// ⚠️ **Um quadrado de arestas RETAS** — e não o `closed_square()` do topo, cujos vértices carregam
/// handles (as "arestas" dele são curvas, e uma lâmina horizontal atravessa-as em sítios que não
/// são `x = 0` nem `x = 4`). Os gates da lâmina são sobre ONDE ela cruza, então a fixture tem de
/// ter cruzamentos que se possam escrever à mão.
fn sharp_square() -> VecPath {
    let sharp = |x: f64, y: f64| VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    };
    VecPath {
        verts: vec![
            sharp(0.0, 0.0),
            sharp(4.0, 0.0),
            sharp(4.0, 4.0),
            sharp(0.0, 4.0),
        ],
        closed: true,
        ..VecPath::default()
    }
}

/// Uma lâmina que atravessa um quadrado de lado a lado cruza-o em **DOIS** pontos, e nos lugares
/// certos. É o caso que faz a faca funcionar: origem fechada + 2 cruzamentos = duas peças.
#[test]
fn a_blade_across_a_square_crosses_it_exactly_twice() {
    let p = sharp_square(); // (0,0)-(4,4), arestas RETAS
    let x = crate::blade_crossings(&p, [-1.0, 2.0], [5.0, 2.0]);

    assert_eq!(x.len(), 2, "duas arestas atravessadas: {x:?}");
    for &(s, t) in &x {
        let pt = crate::point_on_segment(&p, s, t).unwrap();
        assert!((pt[1] - 2.0).abs() < 1e-9, "o cruzamento está na lâmina");
        assert!(pt[0] == 4.0 || pt[0] == 0.0, "nas arestas laterais");
    }
}

/// ⚠️ **Só dentro do SEGMENTO da lâmina.** Uma faca é um traço com começo e fim; cortar para lá da
/// ponta seria a ferramenta a fazer o que ninguém desenhou.
#[test]
fn a_blade_that_stops_short_only_cuts_what_it_reached() {
    let p = sharp_square();
    // Começa fora, à esquerda, e PARA no meio do quadrado.
    let x = crate::blade_crossings(&p, [-1.0, 2.0], [2.0, 2.0]);

    assert_eq!(x.len(), 1, "só a aresta esquerda: {x:?}");
    let pt = crate::point_on_segment(&p, x[0].0, x[0].1).unwrap();
    // ⚠️ Épsilon, não igualdade: o refino é por bisseção, e pedir o bit exato seria um gate que
    // falha sobre produto correto por causa do último passo dela.
    assert!(
        (pt[0] - 0.0).abs() < 1e-9 && (pt[1] - 2.0).abs() < 1e-9,
        "{pt:?}"
    );
}

#[test]
fn a_blade_that_misses_finds_nothing_and_a_degenerate_one_too() {
    let p = sharp_square();
    assert!(crate::blade_crossings(&p, [-5.0, 9.0], [5.0, 9.0]).is_empty());
    assert!(
        crate::blade_crossings(&p, [2.0, 2.0], [2.0, 2.0]).is_empty(),
        "lâmina de comprimento zero"
    );
}

/// **Uma CURVA pode ser atravessada mais de uma vez pelo mesmo segmento**, e a amostragem tem de
/// os achar: dois zeros dentro da mesma amostra cancelariam em sinal e passariam despercebidos.
#[test]
fn a_blade_finds_both_crossings_of_a_single_curved_segment() {
    // Uma cúbica em arco: sobe até y≈0,75 e volta. Uma lâmina em y=0,5 atravessa-a DUAS vezes.
    let mut p = VecPath {
        verts: vec![
            VecVertex {
                anchor: [0.0, 0.0],
                in_handle: [0.0, 0.0],
                out_handle: [0.0, 1.0],
                kind: VertexKind::Corner,
                corner_radius: 0.0,
            },
            VecVertex {
                anchor: [4.0, 0.0],
                in_handle: [4.0, 1.0],
                out_handle: [4.0, 0.0],
                kind: VertexKind::Corner,
                corner_radius: 0.0,
            },
        ],
        closed: false,
        ..VecPath::default()
    };
    p.closed = false;

    let x = crate::blade_crossings(&p, [-1.0, 0.5], [5.0, 0.5]);

    assert_eq!(x.len(), 2, "o arco atravessa a lâmina duas vezes: {x:?}");
    for &(_, t) in &x {
        let pt = crate::point_on_segment(&p, 0, t).unwrap();
        assert!((pt[1] - 0.5).abs() < 1e-7, "y={} != 0,5", pt[1]);
    }
}

/// **A camada SEMÂNTICA da faca**: um cruzamento na PONTA de um segmento é o mesmo do segmento
/// vizinho, e reportá-lo cortaria o mesmo ponto duas vezes. É também o que impede a costura de um
/// corte — que assenta exactamente sobre a lâmina, e é sempre uma ponta — de ser reencontrada.
#[test]
fn a_crossing_at_a_segment_endpoint_is_never_reported() {
    let p = sharp_square();
    // ⚠️ A lâmina tem de ATRAVESSAR a quina, e achar essa fixture custou duas tentativas: a
    // diagonal pelos cantos opostos ROÇA as arestas (as duas ficam do mesmo lado, sem troca de
    // sinal), e uma a 45° pelo canto (4,0) vinda de fora também. Esta entra pelo canto e segue
    // para DENTRO — aí a aresta de baixo e a da direita ficam em lados opostos, e o mesmo ponto
    // seria reportado como ponta de uma e princípio da outra.
    let x = crate::blade_crossings(&p, [5.0, -1.0], [3.0, 1.0]);

    for &(s, t) in &x {
        assert!(
            t > 1e-6 && t < 1.0 - 1e-6,
            "cruzamento na PONTA do segmento {s} (t={t}) -- o vizinho reporta o mesmo ponto"
        );
    }
    // ⚠️ **O preço, nomeado:** com a guarda, uma lâmina que passa EXACTAMENTE por um vértice não
    // corta ali — a travessia é reportada como ponta e cai fora. É medida zero (mover a lâmina um
    // pixel resolve) e o modo de falha é *não cortar*, nunca *cortar no sítio errado*; a
    // alternativa é cortar o mesmo ponto duas vezes e deixar um segmento de comprimento nulo.
    assert!(
        x.is_empty(),
        "a lâmina pelo vértice devia cair fora, e reportou {x:?}"
    );
}
