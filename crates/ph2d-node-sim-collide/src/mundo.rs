//! ⭐⭐ **O `sim.collide` e o MUNDO DE CONTACTO** (doc 121 §9.20, ponto 5) — o aperto de mão em
//! colunas ([`ph2d_contact::obstaculo`]): este nó DECLARA o seu obstáculo; o `sim.step` põe-no no
//! mundo como colisor fixo e devolve o recibo; com o recibo, as peças que o mundo resolve deixam de
//! ser projectadas aqui (só se lhes detecta o toque, o `hit`) — projectá-las por cima do solver
//! desfaria o contacto que ele guarda, e com a taça fora do mundo a pilha fica `8×` mais agitada
//! (medido no oráculo).

use super::{SHAPE_BOWL, SHAPE_BOX, SHAPE_DISC, element_restitution};
use ph2d_contact::obstaculo::{self, FormaFixa};
use ph2d_nodegraph::attr::Stream;

/// Os params da geometria do nó, como o `eval` os leu.
pub(super) struct Obstaculo {
    pub shape: i32,
    pub height: f32,
    pub c: [f32; 2],
    pub radius: f32,
    pub plane_n: [f32; 2],
    pub half: [f32; 2],
}

/// Quem o mundo de contacto resolve contra ESTE obstáculo: as peças com colisor válido, e só
/// depois do recibo — sem ele (um `sim.collide` fora da zona, o 1.º passo) ninguém.
pub(super) fn geridas(s: &Stream, chave: u32) -> Vec<bool> {
    match ph2d_contact::colisores(s) {
        Some(c) if obstaculo::tem_recibo(s, chave) => c.iter().map(Option::is_some).collect(),
        _ => vec![false; s.count()],
    }
}

/// Declara o obstáculo ao mundo — só quando alguém neste stream declara colisor (uma cena sem
/// colisores sai como sempre saiu, sem colunas novas). O salto vai POR PEÇA (o `Randomness`), pela
/// mesma porta que a projecção usa.
pub(super) fn declara(
    out: &mut Stream,
    s: &Stream,
    chave: u32,
    o: Obstaculo,
    (atrito, salto): (f32, f32),
    (randomness, seed): (f32, u32),
    ids: Option<&[f32]>,
) {
    let declara_colisor = s.get(ph2d_nodegraph::attr::COLLIDER_COLUMN).is_some()
        || s.get(ph2d_nodegraph::attr::COLLIDER_BOX_COLUMN).is_some();
    if !declara_colisor {
        return;
    }
    let forma = match o.shape {
        SHAPE_DISC => FormaFixa::Disco {
            centro: o.c,
            raio: o.radius,
        },
        SHAPE_BOWL => FormaFixa::Taca {
            centro: o.c,
            raio: o.radius,
        },
        SHAPE_BOX => FormaFixa::Caixa {
            centro: o.c,
            meia: o.half,
            eixo: [o.plane_n[1], -o.plane_n[0]],
        },
        _ => FormaFixa::Plano {
            normal: o.plane_n,
            altura: o.height,
        },
    };
    #[expect(clippy::cast_sign_loss, reason = "uma identidade e' um inteiro >= 0")]
    #[expect(clippy::cast_possible_truncation, reason = "idem")]
    let saltos: Vec<f32> = (0..s.count())
        .map(|i| {
            let key = ids.map_or(i as u32, |v| v[i].max(0.0).round() as u32);
            element_restitution(salto, randomness, seed, key)
        })
        .collect();
    obstaculo::declara(out, chave, forma, atrito, &saltos);
}

/// A porta dos gates de um só passo — a mesma lei, com a chave `0` (nenhum recibo a ler).
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(super) fn collide(
    s: &Stream,
    shape: i32,
    height: f32,
    c: [f32; 2],
    radius: f32,
    restitution: f32,
    friction: f32,
    part: (i32, f32, f32),
    plane_n: [f32; 2],
    rnd: (f32, u32),
    half: [f32; 2],
) -> Stream {
    super::collide_com_chave(
        s,
        shape,
        height,
        c,
        radius,
        restitution,
        friction,
        part,
        plane_n,
        rnd,
        half,
        0,
    )
}
