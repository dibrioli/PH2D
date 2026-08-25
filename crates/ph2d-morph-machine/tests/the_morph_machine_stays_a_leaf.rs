//! **A LEI DA MÁQUINA CONTINUA LEGÍVEL DO RUNTIME DO JOGO** — irmão do
//! `the_input_map_depends_on_nothing_but_serde`, e pelo mesmo motivo.
//!
//! O pedido do Enio é que o Morph seja *"funcional no runtime do game"*, e o `shells/game` (R1)
//! está adiado: uma lei que morasse na shell do EDITOR seria, por construção, inalcançável do
//! runtime — e reescrevê-la lá seria a segunda porta.
//!
//! ⚠️ **A allowlist é de TRÊS**, e cada um está aqui com o motivo escrito no `Cargo.toml`. O
//! acrescento é deliberado: editar esta lista é o gesto que declara *"o runtime do jogo passa a
//! pagar isto também"*.

use std::path::Path;

/// As secções cujo conteúdo VIAJA para quem depende desta crate.
///
/// ⚠️ `[dev-dependencies]` fica de fora de propósito: ela não alcança consumidor nenhum, e é onde
/// mora o `serde_json` do gate de ida-e-volta.
const REACHES_CONSUMERS: &[&str] = &["[dependencies]", "[build-dependencies]"];

/// **A allowlist.** `serde` (o grafo viaja no `.ph2dproj`) · `ph2d-spring` (folha pura) ·
/// `ph2d-anim` (a `Easing`, o vocabulário de curva DA CASA — ver o `Cargo.toml`).
const ALLOWED: &[&str] = &["serde", "ph2d-spring", "ph2d-anim"];

#[test]
fn the_morph_machine_never_needs_a_world_nor_a_window() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read ph2d-morph-machine/Cargo.toml");

    // ⚠️ O controle POSITIVO vem PRIMEIRO: um gate que procura uma secção ausente encontra zero
    // deps e passa provando nada — o modo de falha de todo gate baseado em parser.
    assert!(
        text.contains("[dependencies]"),
        "a secao `[dependencies]` sumiu do manifesto -- este gate deixou de medir a crate"
    );
    // O segundo controle: a allowlist tem de ser ALCANCADA pelo parser.
    let declared = deps_in(&text, "[dependencies]");
    for must in ALLOWED {
        assert!(
            declared.iter().any(|d| d == must),
            "`{must}` desapareceu do manifesto ({declared:?}) -- ou a crate mudou de desenho, ou \
             este parser nao esta' a ler as dependencias de todo"
        );
    }

    for section in REACHES_CONSUMERS {
        let extra: Vec<String> = deps_in(&text, section)
            .into_iter()
            .filter(|d| !ALLOWED.contains(&d.as_str()))
            .collect();
        assert!(
            extra.is_empty(),
            "`ph2d-morph-machine` ganhou dependencias fora da allowlist em `{section}`: \
             {extra:?}.\n\n\
             Esta crate e' a lei que o EDITOR e o RUNTIME DO JOGO partilham. Toda dep aqui e' paga \
             pelos dois -- e o ponto inteiro de ela ser folha e' que o runtime nao precise de um \
             World, de uma janela, nem do documento vectorial para saber em que forma a maquina \
             esta'.\n\
             Se a dep e' mesmo necessaria, ACRESCENTE-A a' ALLOWED com o motivo escrito no \
             Cargo.toml. A friccao e' deliberada."
        );
    }
}

/// Os nomes de dependência declarados em `section`, até à secção seguinte.
fn deps_in(text: &str, section: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            inside = t == section;
            continue;
        }
        if !inside || t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = t.split_once('=') {
            let name = name.trim().trim_matches('"');
            if !name.is_empty() {
                out.push(name.to_string());
            }
        }
    }
    out
}
