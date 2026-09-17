//! ⭐⭐⭐ **A CHAVE DE UM RÓTULO É DERIVADA DO ID; O TEXTO É QUE É AUTORADO.**
//!
//! ⚠️ **Medido antes de a wave começar:** `display_name` coincidia com a derivação do
//! `canonical_name` em **46 de 80** — `FlipObjectRef` mostra-se *Flip Object*, `PaintedDoc` mostra-se
//! *Painted Document*. ⇒ *o TEXTO não é derivável e tem de ser autorado numa tabela*; a **chave**,
//! essa, é — e é por isso que ela não é um segundo literal que alguém possa escrever ao lado do
//! primeiro. Este ficheiro é o que torna essa frase uma propriedade.
//!
//! ⛔ **Sem ele a chave seria exactamente o defeito que a tabela existe para curar:** duas strings
//! a descrever a mesma coisa, a segunda escrita à mão, a envelhecer no primeiro `rename`.

/// **`CamelCase` → `snake`,** a lei da derivação. `AudioListener2D` → `audio_listener_2d`.
///
/// ⚠️ **O dígito abre palavra e NÃO a fecha**, e isso é medido: sem a 1.ª metade,
/// `AudioListener2D` lê-se `audio_listener2_d` (o `D` vem depois de um dígito e seria tratado como
/// início de palavra); sem a 2.ª, `Model3D` parte-se em três. *As duas regras são uma só lei sobre
/// a fronteira letra↔dígito, e ela precisa dos dois lados.*
fn snake(s: &str) -> String {
    let b: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 4);
    for (i, &c) in b.iter().enumerate() {
        let prev = if i == 0 { '\0' } else { b[i - 1] };
        // ⚠️ **Os dois lados da MESMA fronteira, num `||` só** — o clippy recusa dois braços com
        //    o mesmo corpo, e ele tem razão: a lei é *«abre palavra»*, e ela tem duas causas.
        let abre_palavra = (c.is_ascii_digit() && prev.is_ascii_alphabetic())
            || (c.is_uppercase()
                && i > 0
                && !prev.is_ascii_digit()
                && (prev.is_lowercase() || b.get(i + 1).is_some_and(|n| n.is_lowercase())));
        if abre_palavra {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out.trim_matches('_').to_string()
}

/// O último segmento de um `canonical_name` — `ph2d::ecs::AudioSource2D` → `AudioSource2D`.
fn tipo(canonical: &str) -> &str {
    canonical.rsplit("::").next().unwrap_or(canonical)
}

#[test]
fn every_component_name_key_is_derived_from_its_type() {
    let todos: Vec<_> = ph2d_component_desc::all().collect();
    // ⚠️ Piso de população: um catálogo vazio passaria trivialmente (a armadilha §2.7 do HOWTO).
    assert!(
        todos.len() >= 80,
        "o catálogo tem {} descritores — encolheu?",
        todos.len()
    );
    let mut erradas = Vec::new();
    for d in &todos {
        let esperada = format!("component.{}.name", snake(tipo(d.canonical_name)));
        if d.display_key != esperada {
            erradas.push(format!(
                "{} declara {:?} e a derivação do id dá {:?}",
                d.canonical_name, d.display_key, esperada
            ));
        }
    }
    assert!(
        erradas.is_empty(),
        "estas chaves não são a derivação do id delas:\n  {}\n\nA lei é \
         `component.<snake(último segmento do canonical_name)>.name` — ela não se escolhe.",
        erradas.join("\n  ")
    );
}

/// ⭐⭐ **E as chaves derivadas são ÚNICAS** — *uma derivação sem este gate cala-se no dia em que
/// duas famílias declararem o mesmo nome de tipo* (`ph2d::vec::Frame` e `ph2d::ui::Frame` dariam a
/// mesma chave, e a segunda calaria a primeira em silêncio).
#[test]
fn two_components_never_derive_the_same_key() {
    use std::collections::BTreeMap;
    let mut por_chave: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for d in ph2d_component_desc::all() {
        por_chave
            .entry(d.display_key)
            .or_default()
            .push(d.canonical_name);
    }
    let colisoes: Vec<String> = por_chave
        .iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(k, v)| format!("{k} ← {v:?}"))
        .collect();
    assert!(
        colisoes.is_empty(),
        "duas famílias derivam a MESMA chave e a segunda cala a primeira:\n  {}",
        colisoes.join("\n  ")
    );
}

/// ⭐ **A chave de um campo acaba no `field_id` dele.**
///
/// ⚠️ **O dono da chave de um campo é o BLOCO `const`, não o componente**, e isso é medido: o
/// `MARKER` é partilhado por **5** descritores e o `TAGS` por **2**. Um `FieldDesc` partilhado só
/// pode carregar UMA chave, logo derivá-la do componente daria duas chaves para o mesmo byte. ⇒ o
/// que este gate PODE afirmar em runtime é a metade que não depende do nome do bloco — e é a metade
/// onde uma cópia-e-cola erra: o **id**.
#[test]
fn every_field_key_ends_in_its_own_field_id() {
    let mut erradas = Vec::new();
    let mut n = 0usize;
    for d in ph2d_component_desc::all() {
        for f in d.fields {
            n += 1;
            let sufixo = format!(".{}", f.field_id);
            if !f.label_key.starts_with("component.field.") || !f.label_key.ends_with(&sufixo) {
                erradas.push(format!(
                    "{}: campo {} declara {:?}",
                    d.canonical_name, f.field_id, f.label_key
                ));
            }
        }
    }
    assert!(n >= 120, "o censo viu {n} campos — a população encolheu?");
    assert!(
        erradas.is_empty(),
        "a chave de um campo é `component.field.<bloco>.<field_id>`:\n  {}",
        erradas.join("\n  ")
    );
}

#[test]
fn the_snake_law_is_the_one_that_was_measured() {
    // ⚠️ Os casos que a 1.ª redacção errou, guardados como controlo: sem eles a lei re-escreve-se
    //    «simplificada» e volta a partir `2D` em dois.
    for (tipo, esperado) in [
        ("AudioListener2D", "audio_listener_2d"),
        ("Model3D", "model_3d"),
        ("ZIndexOverride", "z_index_override"),
        ("YSort", "y_sort"),
        ("Ccd", "ccd"),
        ("OneWayPlatform", "one_way_platform"),
        ("Sculpt3dPieceRef", "sculpt_3d_piece_ref"),
    ] {
        assert_eq!(snake(tipo), esperado, "a lei do snake mudou em {tipo}");
    }
}
