//! **A tabela ENTRA** — um `.tokens.json` DTCG vira uma lista de valores autorados.

use serde_json::{Map, Value};

use ph2d_tokens::color::Color;
use ph2d_tokens::num_overrides::NumOverride;
use ph2d_tokens::overrides::ColorOverride;
use ph2d_tokens::route::{AuthoredValue, Factory, Routed, factory, route};
use ph2d_tokens::theme::Theme;

use crate::export::VENDOR_KEY;

/// O arquivo não é utilizável — e a mensagem **diz porquê**, porque um import que não acontece sem
/// nada na tela é indistinguível de um botão quebrado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DtcgError {
    /// Não é JSON. Traz a frase do parser (linha e coluna).
    NotJson(String),
    /// É JSON, mas a raiz não é um objeto — um `.tokens.json` é sempre um grupo.
    NotAGroup,
}

impl std::fmt::Display for DtcgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotJson(e) => write!(f, "nao e' JSON valido: {e}"),
            Self::NotAGroup => write!(f, "a raiz de um .tokens.json tem de ser um objeto"),
        }
    }
}

/// O que o arquivo trouxe, já emparelhado com os tokens deste design system.
///
/// ⚠️ As três contagens são três FATOS diferentes, e colapsá-las num número tiraria do artista a
/// única informação que o faz saber o que aconteceu: *"a tua tabela é de outro app"* (`unknown`),
/// *"este arquivo tem `rem` e nós medimos px"* (`dropped`) e *"isto já era o que estava"*
/// (`at_factory`) mandam-no a três lugares diferentes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Imported {
    /// Os overrides de cor a instalar.
    pub colours: Vec<ColorOverride>,
    /// Os overrides numéricos a instalar.
    pub nums: Vec<NumOverride>,
    /// Tokens cuja chave este design system não conhece.
    pub unknown: usize,
    /// Tokens cuja chave existe mas cujo valor não serve (unidade que não medimos, valor da outra
    /// família, um alias pendurado no vazio).
    pub dropped: usize,
    /// Tokens que **já valem a fábrica** — e por isso não são autorados. Ver o cabeçalho da crate.
    pub at_factory: usize,
}

impl Imported {
    /// Quantos tokens este arquivo de facto AUTORA.
    #[must_use]
    pub fn authored(&self) -> usize {
        self.colours.len() + self.nums.len()
    }
}

/// **Lê um `.tokens.json` para o modo dado.**
///
/// ⚠️ O modo é o que o CHAMADOR passa — o vigente do painel —, e não o que o arquivo diz: ver o
/// cabeçalho da crate.
///
/// # Erros
///
/// Devolve [`DtcgError`] quando o arquivo não é JSON ou a raiz não é um grupo. Tudo o mais
/// **degrada por token**: um token que não serve cai e é contado, e o resto entra — recusar o
/// arquivo inteiro por causa de uma linha seria jogar fora a tabela por um `rem`.
pub fn import(src: &str, theme: Theme) -> Result<Imported, DtcgError> {
    let root: Value = serde_json::from_str(src).map_err(|e| DtcgError::NotJson(e.to_string()))?;
    let Value::Object(root) = root else {
        return Err(DtcgError::NotAGroup);
    };
    let mut out = Imported::default();
    walk(&root, "", theme, &mut out);
    Ok(out)
}

/// Percorre a árvore. Uma chave que começa por `$` é metadata do formato, nunca um filho.
fn walk(node: &Map<String, Value>, prefix: &str, theme: Theme, out: &mut Imported) {
    for (name, child) in node {
        if name.starts_with('$') {
            continue;
        }
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        let Some(obj) = child.as_object() else {
            // Um escalar solto onde devia haver um grupo ou um token. Nenhum arquivo válido o tem.
            out.unknown += 1;
            continue;
        };
        // **A presença de `$value` é o que faz de um objeto um TOKEN** — é assim que a spec o
        // define, e é por isso que não há aqui uma lista de nomes de grupo a envelhecer.
        if obj.contains_key("$value") {
            token(&path, obj, theme, out);
        } else {
            walk(obj, &path, theme, out);
        }
    }
}

/// Um token: **a chave decide a família, o valor tem de caber nela**.
///
/// ⚠️ **O `$type` não é consultado, e a ausência é a decisão.** A chave já responde de que família
/// isto é ([`ph2d_tokens::route`]), e perguntar ao `$type` daria uma segunda resposta à MESMA
/// pergunta — com o modo de falha de o arquivo cair por causa de uma *anotação* (um `"number"` em
/// vez de `"dimension"`, que várias ferramentas emitem) em vez de por causa do valor.
fn token(path: &str, obj: &Map<String, Value>, theme: Theme, out: &mut Imported) {
    let Some(fact) = factory(theme, path) else {
        out.unknown += 1;
        return;
    };
    let value = obj.get("$value").unwrap_or(&Value::Null);

    // Um alias tem a mesma forma nas duas famílias, e é a única que atravessa o `$value` como
    // texto — por isso é lida antes de qualquer coisa saber se isto é cor ou comprimento.
    if let Some(target) = value.as_str().and_then(reference_target) {
        push(path, theme, AuthoredValue::Alias(target), out);
        return;
    }

    match fact {
        Factory::Colour(f) => match colour(value) {
            Some(c) if c == f => out.at_factory += 1,
            Some(c) => push(path, theme, AuthoredValue::Colour(c), out),
            None => out.dropped += 1,
        },
        Factory::Px(f) => {
            // ⚠️ A FÓRMULA vence o `$value`, e a ordem é a lei: o `$value` de uma linha com
            // fórmula é o número que ela DEU (o formato não tem math), então tomá-lo seria assar a
            // fórmula num literal — a mesma perda que achatar um alias.
            if let Some(src) = formula(obj) {
                push(path, theme, AuthoredValue::Formula(src), out);
                return;
            }
            match dimension(value) {
                // ⚠️ Comparação EXACTA, e ela é justa: o export escreve `f64::from(px)`, que volta
                // a `f32` sem perder um bit. Uma tolerância aqui deixaria passar por "fábrica" um
                // número que o artista escolheu de propósito ao lado dela.
                Some(px) if px == f => out.at_factory += 1,
                Some(px) => push(path, theme, AuthoredValue::Px(px), out),
                None => out.dropped += 1,
            }
        }
    }
}

/// Encaminha pela porta única e contabiliza o que ela recusar.
fn push(path: &str, theme: Theme, v: AuthoredValue<'_>, out: &mut Imported) {
    match route(theme, path, v) {
        Some(Routed::Colour(o)) => out.colours.push(o),
        Some(Routed::Num(o)) => out.nums.push(o),
        None => out.dropped += 1,
    }
}

/// `"{spacing.md}"` → `Some("spacing.md")`. **A mesma sintaxe que o artista escreve na fórmula.**
fn reference_target(s: &str) -> Option<&str> {
    let inner = s.strip_prefix('{')?.strip_suffix('}')?;
    (!inner.is_empty()).then_some(inner)
}

/// O texto da fórmula, se este arquivo veio daqui.
fn formula(obj: &Map<String, Value>) -> Option<&str> {
    obj.get("$extensions")?
        .get(VENDOR_KEY)?
        .get("formula")?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// Uma cor, nas DUAS formas que o ecossistema emite — ver o cabeçalho da crate.
fn colour(v: &Value) -> Option<Color> {
    match v {
        Value::String(s) => hex(s),
        Value::Object(o) => {
            // ⚠️ Os `components` vencem o `hex` quando os sabemos ler: a spec chama o `hex` de
            // *fallback*, e preferi-lo descartaria a precisão que o arquivo se deu ao trabalho de
            // carregar. Fora do sRGB é o contrário — o `hex` é a única coisa que sabemos ler, e é
            // exactamente para isso que ele existe.
            let alpha = o.get("alpha").and_then(Value::as_f64).map_or(255, byte);
            if o.get("colorSpace").and_then(Value::as_str) == Some("srgb")
                && let Some(c) = o.get("components").and_then(Value::as_array)
                && c.len() == 3
                && let (Some(r), Some(g), Some(b)) = (c[0].as_f64(), c[1].as_f64(), c[2].as_f64())
            {
                return Some(Color {
                    r: byte(r),
                    g: byte(g),
                    b: byte(b),
                    a: alpha,
                });
            }
            let mut c = hex(o.get("hex")?.as_str()?)?;
            if o.contains_key("alpha") {
                c.a = alpha;
            }
            Some(c)
        }
        _ => None,
    }
}

/// `0.0..=1.0` → `0..=255`, com clamp: um arquivo pode trazer um componente fora de gamut.
fn byte(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// `#rgb`, `#rrggbb` e `#rrggbbaa`.
fn hex(s: &str) -> Option<Color> {
    let h = s.strip_prefix('#')?;
    let n = |i: usize, len: usize| -> Option<u8> {
        let d = h.get(i..i + len)?;
        let v = u8::from_str_radix(d, 16).ok()?;
        // `#abc` é `#aabbcc`: cada dígito repete-se, e não é o mesmo que multiplicar por 16.
        Some(if len == 1 { v * 17 } else { v })
    };
    match h.len() {
        3 => Some(Color {
            r: n(0, 1)?,
            g: n(1, 1)?,
            b: n(2, 1)?,
            a: 255,
        }),
        6 => Some(Color {
            r: n(0, 2)?,
            g: n(2, 2)?,
            b: n(4, 2)?,
            a: 255,
        }),
        8 => Some(Color {
            r: n(0, 2)?,
            g: n(2, 2)?,
            b: n(4, 2)?,
            a: n(6, 2)?,
        }),
        _ => None,
    }
}

/// Um comprimento em px, nas formas que o ecossistema emite.
///
/// ⚠️ **`rem` devolve `None` e é CONTADO**, nunca convertido: converter exige um tamanho de
/// fonte-raiz que este app não tem, e escrever `16` seria inventar um número que ninguém autorou.
fn dimension(v: &Value) -> Option<f32> {
    match v {
        Value::Number(n) => n.as_f64().map(|x| x as f32),
        Value::String(s) => {
            let t = s.trim();
            // Sem unidade e com `px` são a mesma coisa; qualquer outra unidade não é nossa.
            let body = t.strip_suffix("px").unwrap_or(t);
            if body.len() != t.len()
                || t.chars()
                    .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
            {
                body.trim().parse::<f32>().ok()
            } else {
                None
            }
        }
        Value::Object(o) => {
            let x = o.get("value")?.as_f64()? as f32;
            match o.get("unit").and_then(Value::as_str) {
                // A spec exige a unidade; um arquivo que a omite numa chave que É nossa só pode
                // estar a falar de px, e recusá-lo seria estrito sobre uma anotação.
                Some("px") | None => Some(x),
                Some(_) => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "import_tests.rs"]
mod tests;
