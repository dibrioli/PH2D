//! Os gates do plano dos gizmos (W4.6): selos só com a tool, cotas sempre; o que se pinta é o que se acerta; o que a
//! regra liga; a cor da forma.

use super::*;
use crate::cad_tool::tests::mesa;
use ph2d_vec_scene::cad::RegraCad;

fn regras(m: &mut crate::cad_tool::tests::Mesa, leis: &[Lei]) {
    m.scene
        .regras_mut()
        .extend(leis.iter().copied().map(RegraCad::nova));
    m.scene.normaliza_ids();
}

fn plano_de(m: &crate::cad_tool::tests::Mesa) -> Plano {
    m.vec.cad_tool.plano.borrow().clone().expect("o plano")
}

#[test]
fn badges_only_with_the_tool_dimensions_always() {
    let mut m = mesa();
    let (a, b) = (m.lado(1, 2), m.lado(2, 3));
    regras(
        &mut m,
        &[
            Lei::Horizontal(a),
            Lei::Vertical(b),
            Lei::Distancia(a.0, a.1, 2.0),
        ],
    );
    m.desenha(false);
    let p = plano_de(&m);
    assert!(p.selos.is_empty(), "selos sem a tool na mão");
    assert_eq!(p.cotas.len(), 1, "a cota tem de estar sempre");
    assert!(
        p.pontos.is_empty() && p.segmentos.is_empty(),
        "alvos sem a tool"
    );
    m.desenha(true);
    let p = plano_de(&m);
    assert_eq!(p.selos.len(), 2, "um selo por regra que não é medida");
    assert_eq!(p.cotas.len(), 1);
    assert_eq!(p.pontos.len(), 4);
    assert_eq!(p.segmentos.len(), 4);
    let icones: Vec<IconId> = p.selos.iter().map(|s| s.icon).collect();
    assert_eq!(icones, vec![IconId::CadHorizontal, IconId::CadVertical]);
}

/// ⭐ Cada selo e cada número acerta-se no MEIO do rect que se pinta; um ponto, perto dele; um lado, no meio dele.
#[test]
fn the_hit_rect_is_the_painted_rect() {
    let mut m = mesa();
    let (a, b) = (m.lado(1, 2), m.lado(2, 3));
    let v1 = m.v(1);
    regras(
        &mut m,
        &[
            Lei::Igual(a, b),
            Lei::Fixo(v1, [0.0; 2]),
            Lei::Distancia(b.0, b.1, 1.0),
        ],
    );
    m.desenha(true);
    let p = plano_de(&m);
    let centro = |r: &Rect| [f64::from(r.x + r.w / 2.0), f64::from(r.y + r.h / 2.0)];
    assert_eq!(
        p.selos.len(),
        3,
        "o Igual tem um selo em cada lado e o Fixo um"
    );
    assert!(
        p.selos.iter().filter(|s| s.par == Some(1)).count() == 2,
        "o par do Igual numera-se nos dois"
    );
    for s in &p.selos {
        assert_eq!(p.acerta(centro(&s.rect)), Some(Acerto::Selo(s.regra)));
    }
    for c in &p.cotas {
        assert_eq!(p.acerta(centro(&c.rect)), Some(Acerto::Numero(c.regra)));
    }
    let q = p.no_ecra(m.pos(3));
    assert_eq!(
        p.acerta([q[0] + 2.0, q[1] - 2.0]),
        Some(Acerto::Alvo(Alvo::Ponto(m.v(3))))
    );
    let (q1, q2) = (p.no_ecra(m.pos(1)), p.no_ecra(m.pos(2)));
    assert_eq!(
        p.acerta([(q1[0] + q2[0]) / 2.0, (q1[1] + q2[1]) / 2.0 + 3.0]),
        Some(Acerto::Alvo(Alvo::Lado(m.lado(1, 2))))
    );
    // Controlo: longe de tudo não há acerto.
    assert_eq!(p.acerta([-1000.0, -1000.0]), None);
}

/// O que uma regra liga são os pontos de ecrã dela (o hover e a escolhida acendem-nos).
#[test]
fn hover_lights_what_the_rule_binds() {
    let mut m = mesa();
    let a = m.lado(2, 3);
    regras(&mut m, &[Lei::Vertical(a)]);
    m.desenha(true);
    let p = plano_de(&m);
    let id = m.scene.regras()[0].id;
    assert_eq!(
        p.ligados.get(&id),
        Some(&vec![p.no_ecra(m.pos(2)), p.no_ecra(m.pos(3))])
    );
    let s = &p.selos[0];
    let sobre = [f64::from(s.rect.x + 1.0), f64::from(s.rect.y + 1.0)];
    assert_eq!(
        p.acerta(sobre),
        Some(Acerto::Selo(id)),
        "o hover sobre o selo é a regra"
    );
}

/// A cor de cada lado: definido (os dois pontos presos) = `Text1`; livre = `Info`; uma regra em conflito = `Danger`.
#[test]
fn the_shape_colour_says_free_defined_conflict() {
    let mut m = mesa();
    let (b, d) = (m.lado(1, 2), m.lado(2, 3));
    let (v1, v2) = (m.v(1), m.v(2));
    regras(
        &mut m,
        &[Lei::Fixo(v1, [0.0; 2]), Lei::Fixo(v2, [2.0, 0.0])],
    );
    m.desenha(true);
    let cor_de = |p: &Plano, a: [f64; 2], b: [f64; 2]| {
        p.lados
            .iter()
            .find(|(x, y, _)| *x == a && *y == b)
            .map(|l| l.2)
    };
    let p = plano_de(&m);
    let (q1, q2, q3) = (
        p.no_ecra(m.pos(1)),
        p.no_ecra(m.pos(2)),
        p.no_ecra(m.pos(3)),
    );
    assert_eq!(
        cor_de(&p, q1, q2),
        Some(ColorToken::Text1),
        "o lado de baixo está definido"
    );
    assert_eq!(
        cor_de(&p, q2, q3),
        Some(ColorToken::Info),
        "o lado direito está livre"
    );
    // Duas distâncias que não cabem juntas no mesmo lado: conflito.
    regras(
        &mut m,
        &[Lei::Distancia(d.0, d.1, 1.0), Lei::Distancia(d.0, d.1, 5.0)],
    );
    let _ = b;
    m.desenha(true);
    let p = plano_de(&m);
    assert_eq!(
        cor_de(&p, q2, q3),
        Some(ColorToken::Danger),
        "o conflito não pinta o lado"
    );
}

/// ⭐ Report do dono (26/10, cena 7): numa linha aberta de dois pontos (a tampa) o centro da forma cai no próprio lado,
/// «selo para dentro, cota para fora» empatava e o «= 1» tapava o número. Nenhum selo tapa um número nem outro selo; a
/// cota de referência mostra-se entre parênteses.
#[test]
fn no_badge_covers_a_number_or_another_badge_and_a_reference_reads_in_parentheses() {
    let mut m = mesa();
    let tampa = m.scene.push_path(ph2d_vec_scene::VecPath {
        verts: [[0.1, 1.0], [1.0, 2.0]]
            .into_iter()
            .map(ph2d_vec_scene::VecVertex::corner)
            .collect(),
        ..ph2d_vec_scene::VecPath::default()
    });
    m.scene.normaliza_ids();
    let t = |i: usize| PontoCad::Vert(tampa, m.scene.path(tampa).expect("a tampa").verts[i].id);
    let lado_tampa = SegCad(t(0), t(1));
    let topo = m.lado(2, 3);
    regras(
        &mut m,
        &[
            Lei::Igual(topo, lado_tampa),
            Lei::Distancia(lado_tampa.0, lado_tampa.1, 1.3),
            Lei::Distancia(topo.0, topo.1, 1.9),
        ],
    );
    m.scene.regras_mut()[1].referencia = true;
    m.desenha(true);
    let p = plano_de(&m);
    let cruza = |a: &Rect, b: &Rect| {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    };
    assert_eq!(p.selos.len(), 2, "o Igual tem um selo em cada lado");
    assert_eq!(p.cotas.len(), 2);
    for (i, s) in p.selos.iter().enumerate() {
        for c in &p.cotas {
            assert!(
                !cruza(&s.rect, &c.rect),
                "um selo tapa o número «{}»",
                c.texto
            );
        }
        for o in &p.selos[..i] {
            assert!(!cruza(&s.rect, &o.rect), "dois selos um por cima do outro");
        }
    }
    let ref_ = &m.scene.regras()[1];
    for c in &p.cotas {
        let entre = c.texto.starts_with('(') && c.texto.ends_with(')');
        assert_eq!(
            entre,
            c.regra == ref_.id,
            "só a REF vai entre parênteses: «{}»",
            c.texto
        );
    }
}
