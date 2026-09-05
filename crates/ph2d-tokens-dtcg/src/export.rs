//! **A tabela SAI** — um `.tokens.json` DTCG por modo.

use serde_json::{Map, Value, json};

use ph2d_tokens::color::{Color, ColorToken};
use ph2d_tokens::num::NumToken;
use ph2d_tokens::num_overrides::{NumValue, num_override};
use ph2d_tokens::overrides::{TokenValue, color_override};
use ph2d_tokens::theme::Theme;

/// A chave de `$extensions` desta aplicação, em notação de domínio invertido como a spec pede.
///
/// ⚠️ **Pública, e é ela que fecha o round-trip**: o import procura exactamente esta chave para
/// recuperar uma fórmula. Duas cópias da string — uma no escritor, outra no leitor — seriam duas
/// respostas a *"onde a math viaja?"*, e a segunda a divergir mandaria toda fórmula exportada
/// voltar como um número.
pub const VENDOR_KEY: &str = "dev.ph2d.tokens";

/// Quantas casas decimais um componente de cor leva.
///
/// ⚠️ **6, e o número é MEDIDO, não escolhido**: dois níveis de `u8` consecutivos distam
/// `1/255 ≈ 0,00392`, então um erro de `5e-7` fica **quatro mil vezes** abaixo de meia distância —
/// os 256 níveis voltam exactos, e há gate a percorrê-los todos
/// (`every_byte_level_survives_the_round_trip`). O que isto compra é um arquivo que uma pessoa
/// consegue ler e comparar: sem ele cada canal sai com dezassete dígitos, três vezes por token,
/// oitenta vezes por arquivo.
const COMPONENT_DECIMALS: f64 = 1e6;

/// **O arquivo inteiro, para um modo.**
///
/// Traz os ~80 tokens (não só os autorados) — o *porquê* está no cabeçalho da crate: um export
/// esparso de um projeto de fábrica é um arquivo vazio, e os `{...}` não teriam a que se referir.
#[must_use]
pub fn export(theme: Theme) -> String {
    let mut root = Map::new();
    // ⚠️ O modo viaja para uma PESSOA ler, não para o import obedecer: quem escolhe o modo de
    // destino é o painel que o artista está a olhar. Ver o cabeçalho da crate.
    root.insert(
        "$description".into(),
        Value::String(format!("PH2D design tokens — mode: {}", mode_name(theme))),
    );

    for &t in ColorToken::ALL {
        put(&mut root, t.key(), colour_token(t, theme));
    }
    for &t in NumToken::ALL {
        put(&mut root, t.key(), dimension_token(t, theme));
    }

    // ⚠️ `to_string_pretty` e um `\n` final: o arquivo é para ser lido, comparado num diff e
    // versionado ao lado do projeto. E a ordem das chaves é a do `serde_json::Map` (uma
    // `BTreeMap`), então dois exports da mesma tabela dão os MESMOS bytes — sem isso um diff
    // mostraria a ordem dos cliques do artista em vez do que ele mudou.
    let mut s = serde_json::to_string_pretty(&Value::Object(root))
        .expect("um mapa de strings e numeros finitos sempre serializa");
    s.push('\n');
    s
}

/// O nome do modo como o painel o mostra.
const fn mode_name(theme: Theme) -> &'static str {
    // ⚠️ O id vive no `Theme` (`from_id` é o inverso exacto); um `match` aqui foi o que a família
    //    moderna (2026-09-04) teria de repetir em três crates.
    theme.id()
}

/// Põe `value` no caminho `a.b.c`, criando os grupos que faltarem.
///
/// ⚠️ **Ele sobrepõe o que estiver lá, e o guarda é um GATE, não um `if` aqui:** as duas tabelas
/// de chaves formam uma árvore limpa (nenhuma chave é prefixo de outra) e
/// `no_token_key_is_a_prefix_of_another` mede isso. Um ramo defensivo aqui teria de escolher um
/// vencedor entre um grupo e um token de mesmo nome — uma escolha que nenhum arquivo nosso pede e
/// que seria feita em silêncio.
fn put(root: &mut Map<String, Value>, path: &str, value: Value) {
    let mut parts = path.split('.').peekable();
    let mut cur = root;
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            cur.insert(part.to_string(), value);
            return;
        }
        cur = cur
            .entry(part.to_string())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("o gate no_token_key_is_a_prefix_of_another garante que isto e' um grupo");
    }
}

/// ⚠️ **`$type` em TODO token, e nunca herdado do grupo.** A herança existe na spec e é uma
/// nicety; escrevê-la faria o arquivo depender de o leitor a implementar, e o preço de a não usar
/// é uma linha por token num arquivo que já tem quatro.
fn colour_token(t: ColorToken, theme: Theme) -> Value {
    let mut m = Map::new();
    m.insert("$type".into(), Value::String("color".into()));
    // ⚠️ O ALIAS é lido do SLOT (`color_override`), nunca do valor resolvido: achatá-lo aqui é
    // exactamente o que torna um alias uma cópia — a lei que a `overrides.rs` escreve no
    // cabeçalho, e o motivo de o formato ter a sintaxe `{...}`.
    if let Some(TokenValue::Alias(target)) = color_override(theme, t) {
        m.insert("$value".into(), Value::String(reference(target.key())));
    } else {
        m.insert("$value".into(), colour_value(t.resolve(theme)));
    }
    Value::Object(m)
}

fn dimension_token(t: NumToken, theme: Theme) -> Value {
    let mut m = Map::new();
    m.insert("$type".into(), Value::String("dimension".into()));
    match num_override(theme, t) {
        Some(NumValue::Alias(target)) => {
            m.insert("$value".into(), Value::String(reference(target.key())));
        }
        // ⚠️ **A FÓRMULA sai duas vezes, e as duas são precisas.** O `$value` é o número que ela
        // dá — é o que todo leitor DTCG consome, e o formato não tem expressões. O texto vai para
        // o `$extensions`, que é onde a spec manda pôr o que ela não modela; um round-trip por
        // aqui recupera a fórmula, um por outra ferramenta recupera o número.
        Some(NumValue::Expr(src)) => {
            m.insert("$value".into(), dimension_value(t.px(theme)));
            m.insert(
                "$extensions".into(),
                json!({ VENDOR_KEY: { "formula": src } }),
            );
        }
        _ => {
            m.insert("$value".into(), dimension_value(t.px(theme)));
        }
    }
    Value::Object(m)
}

/// A sintaxe de referência do DTCG — **a mesma que o artista escreve numa fórmula**.
fn reference(key: &str) -> String {
    format!("{{{key}}}")
}

/// O `$value` de uma cor na forma da spec de 2025.10, com o `hex` de ponte.
fn colour_value(c: Color) -> Value {
    json!({
        "colorSpace": "srgb",
        "components": [component(c.r), component(c.g), component(c.b)],
        "alpha": component(c.a),
        "hex": format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
    })
}

/// Um canal `0..=255` na escala `0..=1` da spec, arredondado ao que se lê.
fn component(v: u8) -> f64 {
    (f64::from(v) / 255.0 * COMPONENT_DECIMALS).round() / COMPONENT_DECIMALS
}

/// O `$value` de uma dimensão na forma da spec de 2025.10.
///
/// ⚠️ `unit` é obrigatório **mesmo em zero** — a spec di-lo por extenso, e um `0` sem unidade é a
/// forma de um leitor estrito recusar o arquivo inteiro.
fn dimension_value(px: f32) -> Value {
    json!({ "value": f64::from(px), "unit": "px" })
}

#[cfg(test)]
#[path = "export_tests.rs"]
mod tests;
