//! O arrasto da W4.0 (escada 202, misto 200 — o caso acoplado): 60 passos num círculo de 40 px, cada passo a partir
//! da solução anterior; a cada passo toda regra cumpre a 1e-4 px e o ponto agarrado está no cursor.

use ph2d_vec_constraints::{Arrasto, Solucao, resolve};

use crate::cena::{Cena, TOL_PX, cena, pior, xy};

fn arrasta(c: &Cena) -> Vec<Vec<[f64; 2]>> {
    let i = usize::try_from(c.json["arrasto"]["ponto"].as_u64().expect("ponto")).expect("ponto");
    let folha = c.folha();
    let mut anterior: Option<Solucao> = None;
    let mut poses = Vec::new();
    for (k, alvo) in c.json["arrasto"]["caminho"]
        .as_array()
        .expect("caminho")
        .iter()
        .map(xy)
        .enumerate()
    {
        let r = resolve(
            &folha,
            &c.regras,
            Some(Arrasto {
                ponto: c.ponto[i],
                alvo,
            }),
            anterior.as_ref(),
        );
        assert!(
            r.falta.is_empty() && r.sem_alvo.is_empty(),
            "passo {k}: {:?} por cumprir",
            r.falta
        );
        let x = c.posicoes(&r.solucao);
        let v = pior(c, &x, Some((i, alvo)));
        assert!(v <= TOL_PX, "passo {k}: viola {v:e} px");
        poses.push(x);
        anterior = Some(r.solucao);
    }
    assert_eq!(poses.len(), 60, "o arrasto da W4.0 tem 60 passos");
    poses
}

#[test]
fn the_drag_keeps_every_rule_at_every_step() {
    for nome in ["escada_202", "misto_200"] {
        arrasta(&cena(nome));
    }
}

#[test]
fn the_drag_is_deterministic() {
    for nome in ["escada_202", "misto_200"] {
        let c = cena(nome);
        let bits = |p: Vec<Vec<[f64; 2]>>| -> Vec<u64> {
            p.iter().flatten().flatten().map(|v| v.to_bits()).collect()
        };
        assert_eq!(
            bits(arrasta(&c)),
            bits(arrasta(&c)),
            "{nome}: o mesmo arrasto 2× dá outro bit"
        );
    }
}

mod dois_pontos {
    use ph2d_vec_constraints::{Arrasto, Folha, Solucao, resolve};
    use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad};
    use ph2d_vec_scene::{VecPath, VecVertex, VertId, Xform};

    const A: PontoCad = PontoCad::Vert(1, VertId(1));
    const B: PontoCad = PontoCad::Vert(1, VertId(2));

    fn path(b: [f64; 2]) -> VecPath {
        let mut p = VecPath {
            id: 1,
            verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner(b)],
            ..VecPath::default()
        };
        p.normaliza_ids();
        p
    }

    #[test]
    fn the_previous_step_picks_the_branch_not_the_drawing() {
        let desenho = path([10.0, 0.0]);
        let regras = [
            RegraCad::nova(Lei::Fixo(A, [0.0, 0.0])),
            RegraCad::nova(Lei::Distancia(A, B, 10.0)),
        ];
        let antes = path([-9.0, 1.0]);
        let palpite = Solucao::da_folha(&Folha::new([(&antes, Xform::IDENTITY)]));
        let r = resolve(
            &Folha::new([(&desenho, Xform::IDENTITY)]),
            &regras,
            None,
            Some(&palpite),
        );
        let b = r.solucao.ponto(B).expect("B");
        assert!(
            r.falta.is_empty() && b[0] < -9.0,
            "B saltou para o ramo do desenho: {b:?}"
        );
    }

    #[test]
    fn dragging_a_fixed_point_breaks_no_rule() {
        let desenho = path([10.0, 0.0]);
        let regras = [
            RegraCad::nova(Lei::Fixo(A, [0.0, 0.0])),
            RegraCad::nova(Lei::Horizontal(SegCad(A, B))),
        ];
        let arrasto = Arrasto {
            ponto: A,
            alvo: [50.0, 50.0],
        };
        let r = resolve(
            &Folha::new([(&desenho, Xform::IDENTITY)]),
            &regras,
            Some(arrasto),
            None,
        );
        assert!(
            r.falta.is_empty(),
            "o cursor não manda mais que a regra: {:?}",
            r.falta
        );
        let a = r.solucao.ponto(A).expect("A");
        assert!(a[0].hypot(a[1]) <= 1e-6, "A preso fica onde está: {a:?}");
    }
}
