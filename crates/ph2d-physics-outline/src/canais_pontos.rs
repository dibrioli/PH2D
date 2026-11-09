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

/// `escritas` sobre uma cópia de `rings` (a topologia de hoje; o que cai fora dos anéis é ignorado).
fn sobre(rings: &[Ring], escritas: impl Iterator<Item = (FieldKey, f32)>) -> Vec<Ring> {
    let mut novos = rings.to_vec();
    for (kk, v) in escritas {
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
    novos
}

/// ⭐ **Os anéis que a curva pede** sobre `rings` (a topologia é a de hoje; a propagação leva-a a todas
/// as chaves). Se o anel interpolado reprova [`ph2d_shape_handles::pontos::validos`] (auto-intersecção a
/// meio), o quadro dá a chave ANTERIOR (`anterior(kk)`: o valor de cada canal nela) — determinístico,
/// o mesmo a tocar e a saltar; nunca entrega ao solver um anel inválido. `None` = nada a mudar.
#[must_use]
pub fn escritos(
    rings: &[Ring],
    escritas: &[(FieldKey, f32)],
    anterior: impl Fn(FieldKey) -> Option<f32>,
) -> Option<Vec<Ring>> {
    let valido = |r: &Vec<Ring>| ph2d_shape_handles::pontos::validos(rings, r);
    let mut novos = sobre(rings, escritas.iter().copied());
    if !valido(&novos) {
        let chave = escritas
            .iter()
            .filter_map(|&(kk, _)| Some((kk, anterior(kk)?)));
        novos = sobre(rings, chave);
    }
    (novos != rings && valido(&novos)).then_some(novos)
}
