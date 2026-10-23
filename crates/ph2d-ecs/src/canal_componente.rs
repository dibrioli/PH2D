//! **A família COMPONENTE** — o canal de QUALQUER campo do catálogo (`ph2d-component-desc`), sem um
//! vocabulário à mão por componente (plano F27 Bloco B, `docs/UI_New_and_Simple/F27_bloco_B_plano.md`).
//!
//! A chave: `campo = field_id << 2 | faixa` (a faixa é x/y/z/w de um Vec2/Vec4/Color), `i:j` = os 32
//! bits altos do `stable_type_id` do componente (o id travado dos prefabs). Tudo o que se GRAVA é
//! estável; o `grupo` (o losango único de x/y) é um índice denso do catálogo, só em sessão.
//!
//! O valor do canal é o GUARDADO (radianos, fracção, linear), como nas outras famílias: a `View` é
//! do cartão e do script, não da curva.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use ph2d_component_desc::{ComponentDesc, FieldDesc, FieldKind, View};
use serde_json::Value;

use crate::canal::familia::COMPONENTE;
use crate::canal::{CampoInfo, FieldKey};
use crate::scene::registry::stable_type_id;

/// Quantas faixas cabem nos 2 bits baixos do `campo`.
pub const FAIXAS: u16 = 4;
/// O maior `field_id` que cabe nos 14 bits altos do `campo`.
pub const MAX_FIELD_ID: u16 = u16::MAX >> 2;

/// Os 32 bits do id estável de um componente que a chave leva.
#[must_use]
pub fn id32(canonical: &str) -> u32 {
    u32::try_from(stable_type_id(canonical) >> 32).unwrap_or(u32::MAX)
}

/// A chave da faixa `faixa` do campo `field_id` do componente `canonical`.
#[must_use]
pub fn chave(canonical: &str, field_id: u16, faixa: u16) -> Option<FieldKey> {
    if field_id > MAX_FIELD_ID || faixa >= FAIXAS {
        return None;
    }
    let id = id32(canonical);
    #[expect(clippy::cast_possible_truncation, reason = "as duas metades do id")]
    let (i, j) = ((id >> 16) as u16, id as u16);
    Some(FieldKey::new(COMPONENTE, field_id << 2 | faixa, i, j))
}

/// Quantas faixas o campo tem como canal — `0` = não é canal (e porquê está no gate).
#[must_use]
pub fn faixas(f: &FieldDesc) -> u16 {
    if f.serde_key().is_empty() || f.is_ref.is_some() {
        return 0;
    }
    if matches!(f.view, View::Named(_) | View::Not) {
        return 0;
    }
    match (f.kind, f.view) {
        (FieldKind::Enum, View::Options(_)) | (FieldKind::Enum, View::Toggle(..)) => 1,
        (
            FieldKind::Enum
            | FieldKind::Text
            | FieldKind::Curve
            | FieldKind::Gradient
            | FieldKind::Palette
            | FieldKind::Marker
            | FieldKind::Ref,
            _,
        ) => 0,
        (FieldKind::Vec2, _) => 2,
        (FieldKind::Vec4 | FieldKind::Color, _) => 4,
        (
            FieldKind::Scalar
            | FieldKind::Int
            | FieldKind::Angle
            | FieldKind::Seed
            | FieldKind::Toggle,
            _,
        ) => 1,
    }
}

struct Tabela {
    por_id: BTreeMap<u32, &'static ComponentDesc>,
    grupos: BTreeMap<(u32, u16), u16>,
}

fn tabela() -> &'static Tabela {
    static T: OnceLock<Tabela> = OnceLock::new();
    T.get_or_init(|| {
        let mut por_id = BTreeMap::new();
        let mut grupos = BTreeMap::new();
        for d in ph2d_component_desc::all() {
            let id = id32(d.canonical_name);
            por_id.insert(id, d);
            for f in d.fields.iter().filter(|f| faixas(f) > 0) {
                let n = u16::try_from(grupos.len()).unwrap_or(u16::MAX);
                grupos.entry((id, f.field_id)).or_insert(n);
            }
        }
        Tabela { por_id, grupos }
    })
}

/// O componente, o campo e a faixa que uma chave nomeia.
#[must_use]
pub fn resolver(k: FieldKey) -> Option<(&'static ComponentDesc, &'static FieldDesc, u16)> {
    if k.familia != COMPONENTE {
        return None;
    }
    let id = u32::from(k.i) << 16 | u32::from(k.j);
    let d = *tabela().por_id.get(&id)?;
    let f = d.fields.iter().find(|f| f.field_id == k.campo >> 2)?;
    let faixa = k.campo & (FAIXAS - 1);
    (faixa < faixas(f)).then_some((d, f, faixa))
}

/// Todas as chaves de um componente do catálogo, campo a campo e faixa a faixa.
pub fn chaves_de(d: &'static ComponentDesc) -> impl Iterator<Item = FieldKey> {
    d.fields.iter().flat_map(move |f| {
        (0..faixas(f)).filter_map(move |faixa| chave(d.canonical_name, f.field_id, faixa))
    })
}

#[must_use]
pub fn info(k: FieldKey) -> Option<CampoInfo> {
    let (d, f, _) = resolver(k)?;
    Some(CampoInfo {
        rotulo: f.label_key,
        discreto: matches!(
            f.kind,
            FieldKind::Int | FieldKind::Seed | FieldKind::Toggle | FieldKind::Enum
        ),
        grupo: *tabela().grupos.get(&(id32(d.canonical_name), f.field_id))?,
        por_i: false,
        texto: false,
    })
}

#[must_use]
pub fn membro(k: FieldKey) -> Option<&'static str> {
    const XYZW: [&str; 4] = ["canal.m.x", "canal.m.y", "canal.m.z", "canal.m.w"];
    const RGBA: [&str; 4] = ["canal.m.r", "canal.m.g", "canal.m.b", "canal.m.a"];
    let (_, f, faixa) = resolver(k)?;
    let faixa = usize::from(faixa);
    match f.kind {
        FieldKind::Color => RGBA.get(faixa).copied(),
        FieldKind::Vec2 | FieldKind::Vec4 => XYZW.get(faixa).copied(),
        _ => Some(f.label_key),
    }
}

/// O identificador guardado de uma opção (`options!` escreve-o como código: `Full`, `0`, `null`).
fn opcao(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.to_owned()))
}

fn escolhas(f: &FieldDesc) -> Option<Vec<Value>> {
    match f.view {
        View::Options(o) => Some(o.iter().map(|(_, v)| opcao(v)).collect()),
        View::Toggle(off, on) => Some(vec![opcao(off), opcao(on)]),
        _ => None,
    }
}

fn nomes(f: &FieldDesc) -> [&'static str; 4] {
    match f.view {
        View::Pair(a, b) => [a, b, "", ""],
        _ if f.kind == FieldKind::Color => ["r", "g", "b", "a"],
        _ => ["x", "y", "z", "w"],
    }
}

/// O lugar da faixa dentro do valor GUARDADO do campo.
fn lugar<'v>(f: &FieldDesc, faixa: u16, v: &'v mut Value) -> Option<&'v mut Value> {
    if v.is_array() {
        return v.get_mut(usize::from(faixa));
    }
    if v.is_object() && faixas(f) > 1 {
        return v.get_mut(nomes(f)[usize::from(faixa)]);
    }
    (faixa == 0).then_some(v)
}

/// ⭐ **A faixa `faixa` do valor guardado de `f`, como número** — a leitura única do canal.
#[must_use]
pub fn ler_valor(f: &FieldDesc, faixa: u16, v: &Value) -> Option<f32> {
    if let Some(e) = escolhas(f) {
        let i = e.iter().position(|o| o == v)?;
        #[expect(clippy::cast_precision_loss, reason = "uma lista de opções")]
        return Some(i as f32);
    }
    let mut v = v.clone();
    #[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
    match lugar(f, faixa, &mut v)? {
        Value::Number(n) => n.as_f64().map(|x| x as f32),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// ⭐ **Escreve `x` na faixa** do valor guardado; devolve se mudou. A lei do VALOR (faixas, pisos) é
/// do componente (`property_laws` no funil), não daqui: aqui só a FORMA (inteiro, booleano, opção).
pub fn escrever_valor(f: &FieldDesc, faixa: u16, v: &mut Value, x: f32) -> bool {
    if !x.is_finite() {
        return false;
    }
    let Some(e) = escolhas(f) else {
        return lugar(f, faixa, v).is_some_and(|slot| numero(slot, x));
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "arredondado e preso à lista"
    )]
    let i = (x.round().max(0.0) as usize).min(e.len().saturating_sub(1));
    match e.into_iter().nth(i) {
        Some(novo) if *v != novo => {
            *v = novo;
            true
        }
        _ => false,
    }
}

/// Um número na forma que o lugar já tem (booleano, inteiro, real).
fn numero(slot: &mut Value, x: f32) -> bool {
    let n = match slot {
        Value::Bool(_) => Value::Bool(x >= 0.5),
        // Um inteiro positivo não diz se o tipo é sem sinal: escreve-se com sinal, e um negativo num
        // `u32` é recusado pelo serde no funil (o eco mostra o que ficou).
        Value::Number(n) if n.is_i64() || n.is_u64() => {
            #[expect(clippy::cast_possible_truncation, reason = "arredondado")]
            let i = x.round() as i64;
            Value::from(i)
        }
        Value::Number(_) => Value::from(f64::from(x)),
        _ => return false,
    };
    if *slot == n {
        return false;
    }
    *slot = n;
    true
}

#[cfg(test)]
#[path = "canal_componente_tests.rs"]
mod tests;
