//! **Os pontos de um «Outline» editado à mão como canais** (FISICA 51/52, `i` = anel, `j` = ponto, o Q4
//! em `f32`, exacto até 2^24): o leitor e o escritor puros que `ph2d-app-physics::canais_corpo` chama.
//! Desenho: `f2_contorno_e_formas_plano.md`, «Auto Key dos pontos do Outline».

use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::FISICA;
use ph2d_ecs::canal_fisica as f;
use ph2d_physics_ecs::{ColliderOutline, OutlineRing as Ring};

fn chave(campo: u16, i: usize, j: usize) -> Option<FieldKey> {
    Some(FieldKey::new(
        FISICA,
        campo,
        u16::try_from(i).ok()?,
        u16::try_from(j).ok()?,
    ))
}

/// ⭐ **Os canais dos pontos** — só de um Outline à mão (`custom`): o traçado da imagem é derivado do
/// pedido (Detail/Alpha/Specks são as suas chaves).
#[must_use]
pub fn publicados(o: &ColliderOutline) -> Vec<(FieldKey, f32)> {
    if !o.custom {
        return Vec::new();
    }
    let mut v = Vec::new();
    for (i, r) in o.rings.iter().enumerate() {
        for (j, p) in r.iter().enumerate() {
            for (campo, c) in [(f::CONTORNO_PONTO_X, p[0]), (f::CONTORNO_PONTO_Y, p[1])] {
                v.extend(chave(campo, i, j).map(|kk| (kk, c as f32)));
            }
        }
    }
    v
}

/// O valor do canal `kk` em `rings` (`None` = fora dos anéis ou não é ponto).
#[must_use]
pub fn valor(rings: &[Ring], kk: FieldKey) -> Option<f32> {
    let p = rings.get(usize::from(kk.i))?.get(usize::from(kk.j))?;
    match kk.campo {
        f::CONTORNO_PONTO_X => Some(p[0] as f32),
        f::CONTORNO_PONTO_Y => Some(p[1] as f32),
        _ => None,
    }
}

/// ⭐ **Os anéis que a curva pede** sobre `rings` (a topologia é a de hoje; a propagação leva-a a todas
/// as chaves). `None` = nada mudou, ou o anel interpolado reprova [`ph2d_shape_handles::pontos::validos`]
/// (auto-intersecção a meio): o quadro SEGURA os anéis que tem — nunca entrega ao solver um inválido.
#[must_use]
pub fn escritos(rings: &[Ring], escritas: &[(FieldKey, f32)]) -> Option<Vec<Ring>> {
    let mut novos = rings.to_vec();
    for &(kk, v) in escritas {
        let eixo = match kk.campo {
            f::CONTORNO_PONTO_X => 0,
            f::CONTORNO_PONTO_Y => 1,
            _ => continue,
        };
        if let Some(p) = novos
            .get_mut(usize::from(kk.i))
            .and_then(|r| r.get_mut(usize::from(kk.j)))
            && v.is_finite()
        {
            p[eixo] = v.round() as i32;
        }
    }
    (novos != rings && ph2d_shape_handles::pontos::validos(rings, &novos)).then_some(novos)
}
