//! ⛔ **Um leitor de JSON EXACTO — porque o `serde_json` de omissão NÃO é.**
//!
//! Medido nesta bancada: o alvo gravou `0.19999999999999998` e o `serde_json::Value::as_f64` leu
//! **`0.2`** — o parser de omissão dele erra o último bit (a exactidão é a *feature*
//! `float_roundtrip`). A 1.ª corrida da paridade leu **dez fixturas a divergir por um ULP**, todas
//! nos relógios, e a lei estava certa: *a régua é que arredondava.*
//!
//! ⚠️ A feature não foi ligada de propósito: numa build da workspace inteira as features das
//! dependências de teste UNIFICAM, e ela mudaria o `serde_json` de toda crate compilada ao lado —
//! um golden noutra parte do repo podia mudar de valor por causa de uma bancada daqui. Este leitor
//! lê cada número com o `str::parse::<f64>` da biblioteca padrão, que é correctamente arredondado.

use std::ops::Index;

/// Um valor JSON — só o que as fixturas do oráculo usam.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Nulo,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

static NULO: Json = Json::Nulo;

impl Index<&str> for Json {
    type Output = Json;
    fn index(&self, k: &str) -> &Json {
        match self {
            Json::Obj(v) => v.iter().find(|(c, _)| c == k).map_or(&NULO, |(_, x)| x),
            _ => &NULO,
        }
    }
}

impl Json {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(x) => Some(*x),
            _ => None,
        }
    }
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Json::Num(x) if *x >= 0.0 && x.fract() == 0.0 => Some(*x as u64),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_array(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Arr(v) => Some(v),
            _ => None,
        }
    }
}

/// Lê um documento inteiro.
///
/// # Panics
/// Se o texto não for JSON — as fixturas são geradas por `JSON.stringify`, e um erro aqui é uma
/// fixtura partida, que tem de falar alto.
#[must_use]
pub fn ler(texto: &str) -> Json {
    let b = texto.as_bytes();
    let mut i = 0;
    let v = valor(b, &mut i);
    espacos(b, &mut i);
    assert_eq!(i, b.len(), "lixo depois do JSON");
    v
}

fn espacos(b: &[u8], i: &mut usize) {
    while *i < b.len() && b[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

fn valor(b: &[u8], i: &mut usize) -> Json {
    espacos(b, i);
    match b[*i] {
        b'{' => {
            *i += 1;
            let mut v = Vec::new();
            espacos(b, i);
            if b[*i] == b'}' {
                *i += 1;
                return Json::Obj(v);
            }
            loop {
                espacos(b, i);
                let Json::Str(k) = valor(b, i) else {
                    panic!("chave que não é texto");
                };
                espacos(b, i);
                assert_eq!(b[*i], b':');
                *i += 1;
                v.push((k, valor(b, i)));
                espacos(b, i);
                match b[*i] {
                    b',' => *i += 1,
                    b'}' => {
                        *i += 1;
                        return Json::Obj(v);
                    }
                    c => panic!("esperava , ou }} e veio {}", c as char),
                }
            }
        }
        b'[' => {
            *i += 1;
            let mut v = Vec::new();
            espacos(b, i);
            if b[*i] == b']' {
                *i += 1;
                return Json::Arr(v);
            }
            loop {
                v.push(valor(b, i));
                espacos(b, i);
                match b[*i] {
                    b',' => *i += 1,
                    b']' => {
                        *i += 1;
                        return Json::Arr(v);
                    }
                    c => panic!("esperava , ou ] e veio {}", c as char),
                }
            }
        }
        b'"' => {
            *i += 1;
            let mut s = String::new();
            loop {
                let c = b[*i];
                *i += 1;
                match c {
                    b'"' => return Json::Str(s),
                    b'\\' => {
                        let e = b[*i];
                        *i += 1;
                        match e {
                            b'n' => s.push('\n'),
                            b't' => s.push('\t'),
                            b'r' => s.push('\r'),
                            b'b' => s.push('\u{8}'),
                            b'f' => s.push('\u{c}'),
                            b'u' => {
                                let h = std::str::from_utf8(&b[*i..*i + 4]).expect("hex");
                                *i += 4;
                                let cp = u32::from_str_radix(h, 16).expect("hex");
                                s.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                            }
                            outro => s.push(outro as char),
                        }
                    }
                    _ => {
                        // Um byte UTF-8 de continuação volta a juntar-se pelo `from_utf8` abaixo.
                        let ini = *i - 1;
                        let mut fim = *i;
                        while fim < b.len() && (b[fim] & 0xC0) == 0x80 {
                            fim += 1;
                        }
                        s.push_str(std::str::from_utf8(&b[ini..fim]).expect("utf-8"));
                        *i = fim;
                    }
                }
            }
        }
        b't' => {
            *i += 4;
            Json::Bool(true)
        }
        b'f' => {
            *i += 5;
            Json::Bool(false)
        }
        b'n' => {
            *i += 4;
            Json::Nulo
        }
        _ => {
            let ini = *i;
            while *i < b.len() && matches!(b[*i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                *i += 1;
            }
            let t = std::str::from_utf8(&b[ini..*i]).expect("número");
            Json::Num(
                t.parse::<f64>()
                    .unwrap_or_else(|e| panic!("número {t:?}: {e}")),
            )
        }
    }
}

/// ⭐ **O CONTROLO do leitor** — o número que o `serde_json` de omissão lê mal.
#[test]
fn o_leitor_le_o_ultimo_bit() {
    assert_eq!(
        ler("[0.19999999999999998]").as_array().expect("arr")[0]
            .as_f64()
            .expect("num")
            .to_bits(),
        0.199_999_999_999_999_98_f64.to_bits()
    );
    assert_eq!(
        ler(r#"{"a":"NaN","b":[true,null]}"#)["a"].as_str(),
        Some("NaN")
    );
}
