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
