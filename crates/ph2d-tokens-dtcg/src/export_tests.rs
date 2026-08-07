//! Os gates do EXPORT — *o arquivo tem a forma que a spec de 2025.10 pede, e o que o artista
//! autorou sai como o que ele autorou*.

use super::*;
use ph2d_tokens::num_overrides::{NumValue, set_num_override};
use ph2d_tokens::overrides::{TokenValue, set_color_override};
use ph2d_tokens::spacing::Spacing;
use serde_json::Value;

fn parse(theme: Theme) -> Value {
    serde_json::from_str(&export(theme)).expect("o export tem de ser JSON")
}

/// **Instala o parser de math** — sem ele a porta de escrita RECUSA uma fórmula, e o gate que quer
/// ver uma fórmula viajar não conseguiria sequer criá-la. É a MESMA porta que a shell usa no boot.
fn with_math() {
    ph2d_token_math::install();
}

/// Anda por um caminho `a.b.c` no documento.
fn at<'a>(root: &'a Value, path: &str) -> &'a Value {
    let mut cur = root;
    for part in path.split('.') {
        cur = cur
            .get(part)
            .unwrap_or_else(|| panic!("o caminho {path:?} nao existe no arquivo"));
    }
    cur
}

/// **Nenhuma chave é PREFIXO de outra** — o guarda do `put`, que sobrepõe sem perguntar.
///
/// ⚠️ Ele mede o que o `put` **assume**: um `spacing` solto (sem degrau) ao lado de `spacing.md`
/// faria um dos dois desaparecer do arquivo, em silêncio. É o gate, e não um `if` no `put`, porque
/// o ramo defensivo teria de escolher um vencedor entre um grupo e um token de mesmo nome — uma
/// escolha que nenhum arquivo nosso pede.
#[test]
fn no_token_key_is_a_prefix_of_another() {
    let mut keys: Vec<&str> = ColorToken::ALL.iter().map(|t| t.key()).collect();
    keys.extend(NumToken::ALL.iter().map(|t| t.key()));
    assert!(keys.len() > 30, "a varredura achou {} chaves", keys.len());
    for a in &keys {
        for b in &keys {
            if a == b {
                continue;
            }
            assert!(
                !b.starts_with(&format!("{a}.")),
                "a chave {a:?} e' um GRUPO em {b:?} e um TOKEN por conta propria — um dos dois \
                 sumiria do arquivo"
            );
        }
    }
}

/// **A raiz é um grupo, e todo token carrega o próprio `$type`.**
#[test]
fn the_root_is_a_group_and_every_token_carries_its_own_type() {
    let doc = parse(Theme::Forge);
    assert!(doc.is_object());
    assert!(
        doc.get("$description")
            .and_then(Value::as_str)
            .is_some_and(|d| d.contains("forge")),
        "o modo tem de estar dito no arquivo, para uma PESSOA o ler"
    );
    for t in ColorToken::ALL {
        assert_eq!(at(&doc, t.key()).get("$type").unwrap(), "color");
    }
    for t in NumToken::ALL {
        assert_eq!(at(&doc, t.key()).get("$type").unwrap(), "dimension");
    }
}

/// **O PONTO de uma chave numérica é o separador de grupo** — `spacing.md` é `md` dentro de
/// `spacing`, e é isso que faz o caminho DTCG ser a nossa chave sem tradução nenhuma.
#[test]
fn the_dot_in_a_numeric_key_is_the_group_separator() {
    let doc = parse(Theme::Forge);
    let g = doc.get("spacing").expect("o grupo spacing");
    assert!(
        g.is_object() && g.get("$value").is_none(),
        "spacing e' GRUPO"
    );
    assert!(g.get("md").expect("spacing.md").get("$value").is_some());
}

/// **Uma cor sai na forma de 2025.10 — objeto, com o `hex` de ponte.**
#[test]
fn a_colour_is_the_2025_10_object_with_the_hex_bridge() {
    let doc = parse(Theme::Forge);
    let v = at(&doc, "accent").get("$value").expect("$value");
    assert_eq!(v.get("colorSpace").unwrap(), "srgb");
    assert_eq!(v.get("components").unwrap().as_array().unwrap().len(), 3);
    assert!(v.get("alpha").is_some());
    let hex = v.get("hex").unwrap().as_str().unwrap();
    assert!(
        hex.starts_with('#') && hex.len() == 7,
        "o hex de ponte tem de ser #rrggbb, veio {hex:?}"
    );
    // E ele descreve a MESMA cor que os componentes.
    let c = ColorToken::Accent.resolve(Theme::Forge);
    assert_eq!(hex, format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b));
}

/// **Uma dimensão sai na forma de 2025.10 — objeto com `value` e `unit`.**
///
/// ⚠️ A `unit` é exigida pela spec **mesmo em zero**, e um leitor estrito recusa o arquivo inteiro
/// sem ela.
#[test]
fn a_dimension_is_the_2025_10_object_and_the_unit_is_always_there() {
    let doc = parse(Theme::Forge);
    for t in NumToken::ALL {
        let v = at(&doc, t.key()).get("$value").expect("$value");
        assert_eq!(v.get("unit").unwrap(), "px", "{} saiu sem unidade", t.key());
        assert!(
            (v.get("value").unwrap().as_f64().unwrap() as f32 - t.px(Theme::Forge)).abs() < 1e-6
        );
    }
}

/// **Os 256 níveis de um canal voltam EXACTOS** — o número que justifica o `COMPONENT_DECIMALS`.
///
/// ⚠️ É a medição por trás do arredondamento, e não uma afirmação: sem ela, "6 casas chegam" é um
/// palpite, e o modo de falha seria uma cor que volta um nível ao lado depois de um round-trip.
#[test]
fn every_byte_level_survives_the_round_trip() {
    for v in 0u8..=255 {
        let back = (component(v) * 255.0).round() as u8;
        assert_eq!(back, v, "o nivel {v} voltou como {back}");
    }
}

/// **Um ALIAS viaja como referência, nunca como a cor que ele resolve.**
///
/// ⚠️ É a lei que a `overrides.rs` escreve no cabeçalho — achatar é o que torna um alias uma cópia
/// —, e é a razão de o formato ter a sintaxe `{...}`.
#[test]
fn an_alias_travels_as_a_reference_not_as_the_colour_it_resolves_to() {
    set_color_override(
        Theme::Forge,
        ColorToken::BorderStrong,
        Some(TokenValue::Alias(ColorToken::Accent)),
    )
    .expect("border-strong -> accent nao fecha laco");
    let doc = parse(Theme::Forge);
    assert_eq!(
        at(&doc, "border-strong").get("$value").unwrap(),
        "{accent}",
        "o elo tem de sair como referencia"
    );

    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Lg),
        Some(NumValue::Alias(NumToken::Spacing(Spacing::Md))),
    )
    .expect("spacing.lg -> spacing.md nao fecha laco");
    let doc = parse(Theme::Forge);
    assert_eq!(
        at(&doc, "spacing.lg").get("$value").unwrap(),
        "{spacing.md}"
    );
}

/// **Uma FÓRMULA sai duas vezes: o número no `$value`, o texto no `$extensions`.**
#[test]
fn a_formula_travels_in_the_extension_and_the_value_is_the_number_it_gives() {
    with_math();
    let md = NumToken::Spacing(Spacing::Md).factory_px();
    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Lg),
        Some(NumValue::Expr("{spacing.md} * 2".into())),
    )
    .expect("a formula parseia");
    let doc = parse(Theme::Forge);
    let t = at(&doc, "spacing.lg");
    assert_eq!(
        t.get("$value")
            .unwrap()
            .get("value")
            .unwrap()
            .as_f64()
            .unwrap() as f32,
        md * 2.0,
        "o $value de uma formula e' o numero que ela DA -- e' o que todo leitor DTCG consome"
    );
    assert_eq!(
        t.get("$extensions")
            .unwrap()
            .get(VENDOR_KEY)
            .unwrap()
            .get("formula")
            .unwrap(),
        "{spacing.md} * 2"
    );
}

/// **Dois exports da mesma tabela dão os MESMOS bytes.**
///
/// ⚠️ Sem isto um diff mostraria a ordem dos cliques do artista em vez do que ele mudou.
#[test]
fn two_exports_of_the_same_table_are_the_same_bytes() {
    set_color_override(
        Theme::Forge,
        ColorToken::Accent,
        Some(TokenValue::Literal(Color {
            r: 10,
            g: 200,
            b: 30,
            a: 255,
        })),
    )
    .expect("um literal nunca fecha laco");
    assert_eq!(export(Theme::Forge), export(Theme::Forge));
}

/// **O arquivo traz a TABELA INTEIRA, e não só o que o artista tocou.**
///
/// ⚠️ Um export esparso de um projeto de fábrica seria um arquivo vazio — e os `{...}` de um alias
/// não teriam a que se referir.
#[test]
fn the_file_carries_the_whole_table_even_with_nothing_authored() {
    let doc = parse(Theme::Forge);
    let n = doc.as_object().unwrap().len();
    // `$description` + os tokens de cor na raiz + os TRÊS grupos numéricos.
    assert!(
        n >= ColorToken::ALL.len(),
        "o arquivo tem {n} entradas na raiz — a tabela nao chegou inteira"
    );
    for t in ColorToken::ALL {
        assert!(at(&doc, t.key()).get("$value").is_some(), "{}", t.key());
    }
    for t in NumToken::ALL {
        assert!(at(&doc, t.key()).get("$value").is_some(), "{}", t.key());
    }
}

/// **Os quatro modos exportam tabelas de cor distintas** — o arquivo é de UM modo.
#[test]
fn each_mode_exports_its_own_colour_table() {
    let a = parse(Theme::Forge);
    let b = parse(Theme::Workshop);
    // ⚠️ A propriedade é *"as duas tabelas DIFEREM"*, e não *"este token difere"*: nomear um token
    // faz a fixture depender de um valor do `tokens.json` que ninguém prometeu manter — o `bg-0`
    // do Forge e o do Workshop são hoje a MESMA cor, e o gate falhava sobre produto certo.
    assert!(
        ColorToken::ALL
            .iter()
            .any(|t| at(&a, t.key()) != at(&b, t.key())),
        "os dois modos exportaram a mesma tabela de cor"
    );
    // E a ESCALA é a mesma nos quatro (o `tokens.json` guarda-a fora de `themes`).
    for t in NumToken::ALL {
        assert_eq!(at(&a, t.key()), at(&b, t.key()), "{}", t.key());
    }
}
