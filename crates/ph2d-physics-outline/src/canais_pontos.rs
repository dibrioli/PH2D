//! **Os pontos de um «Outline» editado à mão como canais** (FISICA 51/52, `i` = anel, `j` = ponto, o Q4
//! em `f32`, exacto até 2^24): o leitor e o escritor puros que `ph2d-app-physics::canais_corpo` chama.
//! Desenho: `f2_contorno_e_formas_plano.md`, «Auto Key dos pontos do Outline».

use std::collections::BTreeMap;

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
    e_fit(antes, agora) && apagar_pontos(doc, e)
}

/// `true` ⇒ `e` tem alguma trilha de ponto (51/52) na timeline.
#[must_use]
pub fn animados(doc: &TimelineDoc, e: u64) -> bool {
    doc.bindings()
        .iter()
        .any(|b| b.entity == e && matches!(b.prop, PropKind::Field(kk) if ponto(&kk)))
}

/// As formas com pontos animados que o Auto Key não grava (regra B): vazio com o Auto Key ligado.
#[must_use]
pub fn travadas(doc: &TimelineDoc, auto_key: bool) -> std::collections::BTreeSet<u64> {
    if auto_key {
        return Default::default();
    }
    doc.bindings()
        .iter()
        .filter(|b| matches!(b.prop, PropKind::Field(kk) if ponto(&kk)))
        .map(|b| b.entity)
        .collect()
}

/// ⭐ **Apaga todas as trilhas dos pontos de `e`** — o «Delete animation» do modal (regra B) e o Fit
/// ([`apagar_do_fit`]). `true` se apagou alguma.
pub fn apagar_pontos(doc: &mut TimelineDoc, e: u64) -> bool {
    let props: Vec<PropKind> = doc
        .bindings()
        .iter()
        .filter(|b| b.entity == e)
        .filter_map(|b| match b.prop {
            PropKind::Field(kk) if ponto(&kk) => Some(b.prop),
            _ => None,
        })
        .collect();
    props.into_iter().fold(false, |n, p| doc.unbind(e, p) | n)
}

type Pt = [f32; 2];

fn aneis(lidos: impl IntoIterator<Item = (FieldKey, f32)>) -> BTreeMap<u16, Vec<Pt>> {
    ph2d_timeline::topologia::contornos(lidos, FISICA, f::CONTORNO_PONTO_X)
}

/// `true` se os anéis mudaram de nº, ou algum de nº de pontos (acrescentar/apagar um ponto).
#[must_use]
pub fn mudou(antes: &BTreeMap<FieldKey, f32>, agora: &[(FieldKey, f32)]) -> bool {
    let (o, h) = (
        aneis(antes.iter().map(|(&k, &v)| (k, v))),
        aneis(agora.iter().copied()),
    );
    !o.is_empty()
        && !h.is_empty()
        && (o.len() != h.len()
            || h.iter()
                .any(|(i, r)| o.get(i).is_some_and(|x| x.len() != r.len())))
}

/// `true` ⇒ a topologia do Auto Key tem trabalho em `e`: o Fit, ou um ponto acrescentado/apagado.
#[must_use]
pub fn pede_topologia(antes: &BTreeMap<FieldKey, f32>, agora: &[(FieldKey, f32)]) -> bool {
    e_fit(antes, agora) || mudou(antes, agora)
}

/// O parâmetro do ponto `x` projectado no segmento `a→b` (`0..=1`).
fn projectar(a: Pt, b: Pt, x: Pt) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        return 0.0;
    }
    (((x[0] - a[0]) * dx + (x[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
}

/// O anel de UMA chave (`na_chave`, na topologia de ontem) refeito na de hoje: os pontos alinhados
/// levam o valor da chave; cada ponto novo entre dois alinhados cai no mesmo lugar do segmento deles.
fn refazer(ontem: &[Pt], hoje: &[Pt], na_chave: &[Pt]) -> Vec<Pt> {
    let pares = ph2d_timeline::topologia::alinhar(ontem, hoje);
    let de_hoje: BTreeMap<usize, usize> = pares.iter().map(|&(a, b)| (b, a)).collect();
    let n = hoje.len();
    let alinhado = |b: usize| de_hoje.get(&b).copied();
    (0..n)
        .map(|b| {
            if let Some(a) = alinhado(b) {
                return na_chave[a];
            }
            let ante = (1..n)
                .map(|d| (b + n - d) % n)
                .find(|&x| alinhado(x).is_some());
            let depois = (1..n).map(|d| (b + d) % n).find(|&x| alinhado(x).is_some());
            match (ante.and_then(alinhado), depois.and_then(alinhado)) {
                (Some(oa), Some(ob)) if oa != ob => {
                    let u = projectar(ontem[oa], ontem[ob], hoje[b]);
                    let (p, q) = (na_chave[oa], na_chave[ob]);
                    [
                        (p[0] + (q[0] - p[0]) * u).round(),
                        (p[1] + (q[1] - p[1]) * u).round(),
                    ]
                }
                _ => hoje[b],
            }
        })
        .collect()
}

/// ⭐ **Acrescentar/apagar um ponto vale em TODAS as chaves** (lei do dono 19/10; a mesma de
/// `ph2d-app-vec::canais_topologia`, o alinhamento é o partilhado de `ph2d_timeline::topologia`):
/// cada chave dos pontos de `e` é refeita na topologia de hoje; as trilhas dos `(i, j)` que deixaram
/// de existir saem. Também o Fit ([`apagar_do_fit`]). `true` se mexeu na timeline.
pub fn propagar(
    doc: &mut TimelineDoc,
    e: u64,
    antes: &BTreeMap<FieldKey, f32>,
    agora: &[(FieldKey, f32)],
) -> bool {
    if apagar_do_fit(doc, e, antes, agora) {
        return true;
    }
    if !mudou(antes, agora) {
        return false;
    }
    let trilhas: Vec<(FieldKey, ph2d_anim::AnimTarget)> = doc
        .bindings()
        .iter()
        .filter(|b| b.entity == e)
        .filter_map(|b| match b.prop {
            PropKind::Field(kk) if ponto(&kk) => Some((kk, b.target)),
            _ => None,
        })
        .collect();
    let mut tempos: BTreeMap<ph2d_anim::RationalTime, ph2d_anim::Interp> = BTreeMap::new();
    for &(_, tg) in &trilhas {
        for key in doc.active_clip().track(tg).map_or(&[][..], |t| t.keys()) {
            tempos.entry(key.t).or_insert(key.interp);
        }
    }
    if tempos.is_empty() {
        return false;
    }
    let (ontem, hoje) = (
        aneis(antes.iter().map(|(&k, &v)| (k, v))),
        aneis(agora.iter().copied()),
    );
    let amostra = |kk: FieldKey, t: ph2d_anim::RationalTime| {
        let tr = doc
            .active_clip()
            .track(doc.binding_for(e, PropKind::Field(kk))?.target)?;
        match ph2d_anim::AttributeEvaluator::sample(tr, t.to_seconds()) {
            ph2d_anim::AnimValue::Float(x) => Some(x),
            _ => None,
        }
    };
    let mut escritas = Vec::new();
    for (&t, &interp) in &tempos {
        for (&i, h) in &hoje {
            let novo = match ontem.get(&i) {
                Some(o) => {
                    let na: Vec<Pt> = (0..o.len())
                        .map(|j| {
                            let c = |campo| {
                                chave(campo, usize::from(i), j).and_then(|kk| amostra(kk, t))
                            };
                            [
                                c(f::CONTORNO_PONTO_X).unwrap_or(o[j][0]),
                                c(f::CONTORNO_PONTO_Y).unwrap_or(o[j][1]),
                            ]
                        })
                        .collect();
                    refazer(o, h, &na)
                }
                None => h.clone(),
            };
            for (j, p) in novo.iter().enumerate() {
                for (campo, v) in [(f::CONTORNO_PONTO_X, p[0]), (f::CONTORNO_PONTO_Y, p[1])] {
                    escritas.extend(chave(campo, usize::from(i), j).map(|kk| (t, interp, kk, v)));
                }
            }
        }
    }
    for (t, interp, kk, v) in escritas {
        doc.upsert_key(
            e,
            PropKind::Field(kk),
            t,
            ph2d_anim::AnimValue::Float(v),
            interp,
        );
    }
    let sobra = trilhas
        .iter()
        .filter(|(kk, _)| hoje.get(&kk.i).is_none_or(|h| usize::from(kk.j) >= h.len()));
    for &(_, tg) in sobra {
        doc.purge_binding(tg);
    }
    true
}
