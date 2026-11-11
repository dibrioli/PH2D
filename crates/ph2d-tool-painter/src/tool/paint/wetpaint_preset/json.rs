//! Cada campo do `BrushSpec` para JSON e de volta, pelo NOME do campo, numa tabela ÚNICA: uma linha por campo com
//! a ida, a volta e a condição de gravar ([`Campo`]). Os do preset ([`super::CAMPOS_DO_PINCEL`]) em
//! [`tabela_preset`]; os que só a macro leva em [`tabela_obra`] e [`tabela_meio`].
//!
//! Leis: `f32` escreve-se pelo menor texto que volta ao mesmo `f32` (`0.1`, nunca
//! `0.10000000149011612`); enums pelo NOME da variante (reordenar o enum não estraga um preset);
//! chave ausente ou ilegível = o valor de `base` (o preset diz o que fixa, o resto é o de fábrica).

use ph2d_painter_brush::BrushSpec;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::OnceLock;

mod curvas;
#[cfg(test)]
mod enums_tests;
mod tabela_meio;
mod tabela_obra;
mod tabela_preset;
mod valores;

/// Um campo do pincel: o nome no ficheiro, a ida, a volta e se grava (o campo «só quando ligado» diz aqui a sua
/// condição, ao lado do valor por omissão que a volta usa na ausência).
pub(super) struct Campo {
    nome: &'static str,
    ler: fn(&BrushSpec) -> Value,
    escrever: fn(&mut BrushSpec, Option<&Value>),
    grava: fn(&BrushSpec) -> bool,
}

/// O campo que se grava sempre.
const fn c(
    nome: &'static str,
    ler: fn(&BrushSpec) -> Value,
    escrever: fn(&mut BrushSpec, Option<&Value>),
) -> Campo {
    Campo {
        nome,
        ler,
        escrever,
        grava: |_| true,
    }
}

/// O campo que só se grava quando `grava`: o preset sem ele é o de antes, ao byte.
const fn so(
    nome: &'static str,
    ler: fn(&BrushSpec) -> Value,
    escrever: fn(&mut BrushSpec, Option<&Value>),
    grava: fn(&BrushSpec) -> bool,
) -> Campo {
    Campo {
        nome,
        ler,
        escrever,
        grava,
    }
}

/// Ligado = não `<= 0` (um NaN grava-se, como sempre se gravou).
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn ligado(x: f32) -> bool {
    !(x <= 0.0)
}

/// A linha do campo. Um nome que não é campo do `BrushSpec` é erro de quem chama.
fn campo(nome: &str) -> &'static Campo {
    static INDICE: OnceLock<BTreeMap<&'static str, &'static Campo>> = OnceLock::new();
    let indice = INDICE.get_or_init(|| {
        let mut m = BTreeMap::new();
        for c in tabela_preset::CAMPOS
            .iter()
            .chain(tabela_obra::CAMPOS)
            .chain(tabela_meio::CAMPOS)
        {
            assert!(
                m.insert(c.nome, c).is_none(),
                "campo do pincel repetido: {}",
                c.nome
            );
        }
        m
    });
    indice
        .get(nome)
        .copied()
        .unwrap_or_else(|| unreachable!("campo do pincel sem conversão: {nome}"))
}

/// O sub-conjunto do pincel que o preset leva, por nome.
pub(crate) fn pincel_para_json(b: &BrushSpec, campos: &[&str]) -> BTreeMap<String, Value> {
    campos
        .iter()
        .map(|&n| campo(n))
        .filter(|c| (c.grava)(b))
        .map(|c| (c.nome.to_owned(), (c.ler)(b)))
        .collect()
}

/// `base` com os campos que o JSON fixa. Chaves fora de `campos` (de ondas futuras) são ignoradas
/// aqui e sobrevivem no [`ph2d_brush_presets::Parametros`].
pub(crate) fn pincel_de_json(
    m: &BTreeMap<String, Value>,
    campos: &[&str],
    base: BrushSpec,
) -> BrushSpec {
    let mut b = base;
    for &n in campos {
        (campo(n).escrever)(&mut b, m.get(n));
    }
    if campos.contains(&"grao_aplicado") && !m.contains_key("grao_aplicado") {
        b.grao_aplicado = ph2d_painter_brush::GraoAplicado {
            grao: b.texture,
            media: None,
        };
    }
    // O legado de antes da secção Rendering sair (24/10): o tecto por dab morava em `traco`; o tecto do traço e as
    // orlas dele já não existem e ignoram-se.
    if campos.contains(&"tecto_por_dab")
        && !m.contains_key("tecto_por_dab")
        && let Some(t) = m.get("traco").and_then(|t| t.get("tecto_por_dab"))
    {
        b.tecto_por_dab = t.as_bool().unwrap_or(false);
    }
    // O pincel que sai respeita o seu tecto (o Size do artista não passa dele, J3 decisão 8).
    if let Some(d) = b.size_max_px {
        b.radius_px = b.radius_px.min(0.5 * d);
    }
    b
}
