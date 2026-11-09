//! **Os pontos de um «Outline» editado à mão como canais** (FISICA 51/52, `i` = anel, `j` = ponto, o Q4
//! em `f32`, exacto até 2^24): o leitor e o escritor puros que `ph2d-app-physics::canais_corpo` chama.
//! Desenho: `f2_contorno_e_formas_plano.md`, «Auto Key dos pontos do Outline».

use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::FISICA;
use ph2d_ecs::canal_fisica as f;
use ph2d_physics_ecs::{ColliderOutline, OutlineRing as Ring};
use ph2d_timeline::{PropKind, TimelineDoc};

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

/// ⭐ **Aplica a `e` os pontos que a curva mandou** (`lote`, a guarda de [`escritos`]); devolve, por canal,
/// `(kk, escrito, antes, ficou)` — o eco e a condução de quem chama.
pub fn aplicar(
    world: &mut ph2d_ecs::World,
    e: u64,
    lote: &[(u64, FieldKey, f32)],
    anterior: impl Fn(FieldKey) -> Option<f32>,
) -> Vec<(FieldKey, f32, f32, f32)> {
    let Some(o) = ph2d_ecs::Entity::try_from_bits(e).and_then(|x| world.get::<ColliderOutline>(x))
    else {
        return Vec::new();
    };
    let antes = o.rings.to_vec();
    let kv: Vec<(FieldKey, f32)> = lote.iter().map(|&(_, kk, v)| (kk, v)).collect();
    let novos = escritos(&antes, &kv, anterior);
    let agora = novos.clone().unwrap_or_else(|| antes.clone());
    if let Some(r) = novos {
        ph2d_shape_handles::pontos::escreve(world, e, r, true);
    }
    kv.iter()
        .filter_map(|&(kk, v)| Some((kk, v, valor(&antes, kk)?, valor(&agora, kk)?)))
        .collect()
}

/// O valor de `kk` de `e` na chave ANTERIOR a `t` (as trilhas de uma forma têm as chaves nos mesmos tempos).
#[must_use]
pub fn na_chave_anterior(doc: &TimelineDoc, e: u64, kk: FieldKey, t: f64) -> Option<f32> {
    let tr = doc
        .active_clip()
        .track(doc.binding_for(e, PropKind::Field(kk))?.target)?;
    let s = tr
        .keys()
        .iter()
        .map(|c| c.t.to_seconds())
        .filter(|&s| s <= t)
        .reduce(f64::max)?;
    match ph2d_anim::AttributeEvaluator::sample(tr, s) {
        ph2d_anim::AnimValue::Float(x) => Some(x),
        _ => None,
    }
}

fn ponto(kk: &FieldKey) -> bool {
    kk.familia == FISICA && f::e_do_ponto(kk.campo)
}

/// `true` ⇒ `e` publicava pontos e já não publica nenhum: voltou ao traçado (o Fit).
#[must_use]
pub fn e_fit(antes: &std::collections::BTreeMap<FieldKey, f32>, agora: &[(FieldKey, f32)]) -> bool {
    antes.keys().any(ponto) && !agora.iter().any(|(kk, _)| ponto(kk))
}

/// ⭐ **O Fit apaga a trilha dos pontos**: `e` publicava pontos (`antes`) e já não (`agora`: voltou ao
/// traçado); a topologia do Auto Key chama-a no passo de undo dela. `true` se apagou.
pub fn apagar_do_fit(
    doc: &mut TimelineDoc,
    e: u64,
    antes: &std::collections::BTreeMap<FieldKey, f32>,
    agora: &[(FieldKey, f32)],
) -> bool {
    if !e_fit(antes, agora) {
        return false;
    }
    let props = antes.keys().filter(|kk| ponto(kk));
    props.fold(false, |n, &kk| doc.unbind(e, PropKind::Field(kk)) | n)
}
