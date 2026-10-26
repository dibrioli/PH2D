//! A costura com o vetor: o círculo é GEOMETRIA (§3.2), o resolvido volta ao local pelo afim de cada path, as
//! alças acompanham a âncora e os ids não mudam.

use ph2d_vec_constraints::{EstadoRegra, Folha, estado, resolve};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad};
use ph2d_vec_scene::{VecPath, VecVertex, VertId, Xform, ellipse};

fn path(id: u64, verts: Vec<VecVertex>) -> VecPath {
    let mut p = VecPath {
        id,
        verts,
        ..VecPath::default()
    };
    p.normaliza_ids();
    p
}

fn perto(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).hypot(a[1] - b[1]) <= 1e-6
}

#[test]
fn a_solved_vertex_goes_back_through_its_transform_with_its_handles() {
    let mut p = path(
        1,
        vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::smooth([10.0, 3.0], [8.0, 3.0], [12.0, 3.0]),
        ],
    );
    // local → folha: roda 90° e anda (100, 50).
    let xf = Xform([0.0, 1.0, -1.0, 0.0, 100.0, 50.0]);
    let (a, b) = (PontoCad::Vert(1, VertId(1)), PontoCad::Vert(1, VertId(2)));
    let regras = [
        RegraCad::nova(Lei::Fixo(a, xf.apply([0.0, 0.0]))),
        RegraCad::nova(Lei::Horizontal(SegCad(a, b))),
    ];
    let r = resolve(&Folha::new([(&p, xf)]), &regras, None, None);
    assert!(r.falta.is_empty() && r.sem_alvo.is_empty());
    let antes = p.verts[1];
    assert!(r.solucao.aplica(&mut p, xf));
    let v = p.verts[1];
    assert_eq!(v.id, antes.id);
    assert!(
        (xf.apply(v.anchor)[1] - 50.0).abs() <= 1e-6,
        "horizontal na FOLHA, não no local"
    );
    let d = [v.anchor[0] - antes.anchor[0], v.anchor[1] - antes.anchor[1]];
    assert!(
        perto(
            v.in_handle,
            [antes.in_handle[0] + d[0], antes.in_handle[1] + d[1]]
        ),
        "a alça acompanha a âncora"
    );
    assert!(perto(
        v.out_handle,
        [antes.out_handle[0] + d[0], antes.out_handle[1] + d[1]]
    ));
}

#[test]
fn a_circle_is_written_back_as_a_circle_with_its_ids() {
    let mut recta = path(
        1,
        vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([100.0, 0.0]),
        ],
    );
    let mut c = ellipse([50.0, 30.0], 20.0, 20.0);
    c.id = 2;
    c.normaliza_ids();
    let ids: Vec<VertId> = c.verts.iter().map(|v| v.id).collect();
    let (a, b) = (PontoCad::Vert(1, VertId(1)), PontoCad::Vert(1, VertId(2)));
    let regras = [
        RegraCad::nova(Lei::Fixo(a, [0.0, 0.0])),
        RegraCad::nova(Lei::Fixo(b, [100.0, 0.0])),
        RegraCad::nova(Lei::Fixo(PontoCad::Centro(2), [50.0, 30.0])),
        RegraCad::nova(Lei::Tangente(SegCad(a, b), 2)),
    ];
    let folha = Folha::new([(&recta, Xform::IDENTITY), (&c, Xform::IDENTITY)]);
    let r = resolve(&folha, &regras, None, None);
    assert!(
        r.falta.is_empty() && r.sem_alvo.is_empty(),
        "{:?} {:?}",
        r.falta,
        r.sem_alvo
    );
    r.solucao.aplica(&mut recta, Xform::IDENTITY);
    assert!(perto(recta.verts[0].anchor, [0.0, 0.0]) && perto(recta.verts[1].anchor, [100.0, 0.0]));
    assert!(r.solucao.aplica(&mut c, Xform::IDENTITY));
    assert_eq!(
        c.verts.iter().map(|v| v.id).collect::<Vec<_>>(),
        ids,
        "os ids do círculo ficam"
    );
    let certo = ellipse([50.0, 30.0], 30.0, 30.0);
    for (v, w) in c.verts.iter().zip(&certo.verts) {
        assert!(
            perto(v.anchor, w.anchor)
                && perto(v.in_handle, w.in_handle)
                && perto(v.out_handle, w.out_handle)
        );
    }
}

#[test]
fn a_rule_on_an_ellipse_or_a_missing_vertex_has_no_target() {
    let mut el = ellipse([0.0, 0.0], 20.0, 10.0);
    el.id = 2;
    el.normaliza_ids();
    let recta = path(
        1,
        vec![
            VecVertex::corner([0.0, 30.0]),
            VecVertex::corner([10.0, 30.0]),
        ],
    );
    let (a, b) = (PontoCad::Vert(1, VertId(1)), PontoCad::Vert(1, VertId(2)));
    let regras = [
        RegraCad::nova(Lei::Tangente(SegCad(a, b), 2)),
        RegraCad::nova(Lei::Horizontal(SegCad(a, PontoCad::Vert(1, VertId(9))))),
        RegraCad::nova(Lei::Horizontal(SegCad(a, b))),
    ];
    let folha = Folha::new([(&recta, Xform::IDENTITY), (&el, Xform::IDENTITY)]);
    let r = resolve(&folha, &regras, None, None);
    assert_eq!(r.sem_alvo, vec![0, 1]);
    let e = estado(&folha, &regras);
    assert_eq!(
        e.regras,
        vec![EstadoRegra::SemAlvo, EstadoRegra::SemAlvo, EstadoRegra::Ok]
    );
}
