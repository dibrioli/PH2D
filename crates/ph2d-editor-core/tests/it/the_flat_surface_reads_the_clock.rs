//! **A SUPERFÍCIE PLANA TAMBÉM LÊ O RELÓGIO** — o último ramo do eixo do hover que pintava DURO.
//!
//! # O elo que faltava, e por que nenhum gate o via
//!
//! O eixo do hover tem três elos ([`the_pointer_and_the_clock_agree_on_who_lights_up`]): o ponteiro
//! promove, o relógio publica e integra, o pintor mistura por `t`. As famílias de *widget* estão
//! todas ligadas — `Button::bg_color`, o tint do `IconButton`, a caixa do `Checkbox`, os quatro do
//! texto e do chip.
//!
//! Mas há uma quarta rota, e ela não é um widget: a **superfície plana**
//! (o mapa duro do `widget/button_surface.rs`), que mapeia `ButtonState → ColorToken` e
//! **não tem `t` nenhum na assinatura**. Cinco sítios de pintura resolviam-na directamente, então
//! as setas de reordenar camada, os botões da máscara, as amostras do *ramp* e os chips do *stroke
//! apply* **saltavam** ao lado de todo o resto do app, que amacia.
//!
//! ⚠️ **A suíte inteira ficava verde**, e por uma razão estrutural: os gates de eixo passam o par
//! à mão a um *widget*, e esta rota **não passa por widget nenhum** — é uma função livre resolvida
//! dentro do pintor. Nenhum gate por-widget podia alcançá-la.
//!
//! # A lei que este gate escreve
//!
//! *Um sítio que pinta a superfície plana de um botão REGISTADO pede o par, nunca o estado.*
//!
//! A porta é [`ph2d_editor_core::widget::flat_button_surface_color`], que recebe
//! `(ButtonState, f32)` — o mesmo truque do `Button::visual`: **um par não se pode passar pela
//! metade**, então o sítio seguinte não pode esquecer o `t`.
//!
//! ⚠️ **O que continua legítimo é o token de REPOUSO por si.** Um sítio que quer *"a superfície
//! neutra"* e não tem botão nenhum não deve chamar isto — deve nomear o token
//! (`ColorToken::Bg2`). Pedir `flat_button_surface(ButtonState::Normal)` era dizer o mesmo por um
//! caminho que finge haver um estado.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a raiz do repo está dois níveis acima da crate")
        .to_path_buf()
}

/// Todo `.rs` de produção sob `crates/` e `shells/`.
fn sources() -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    for top in ["crates", "shells"] {
        walk(&root().join(top), &mut out);
    }
    out
}

fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if !matches!(
                p.file_name().and_then(|s| s.to_str()),
                Some("target" | "tests")
            ) {
                walk(&p, out);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs")
            && !p
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.ends_with("_tests.rs"))
            && let Ok(s) = std::fs::read_to_string(&p)
        {
            out.push((p, s));
        }
    }
}

/// ⚠️ **A porta isenta-se pelo MÓDULO, não pelo nome do ficheiro** — ela era
/// `widget/button_surface.rs` e passou a ser `widget/button_surface/mod.rs` quando o tecto de
/// LOC a cortou em *cor* + *forma de grupo* (2026-09-06). *Um censo que aponta a um FICHEIRO mede
/// o sítio, não a lei — e quem corta um ficheiro em dois não devia ter de saber que censo de
/// outra pessoa aponta para ele.* (É a segunda vez que esta linha paga isto; a primeira foi o
/// cartão de asset, na wave 6.)
fn is_the_door_itself(p: &std::path::Path) -> bool {
    p.to_string_lossy().contains("widget/button_surface")
}

/// ⛔ **NINGUÉM RE-INVENTA O MAPA DURO.**
///
/// ⚠️ A porta certa é agora a única alcançável — o mapa duro é **privado**, e foi essa
/// privacidade que fez o **compilador** enumerar os cinco sítios. O que a privacidade NÃO impede é
/// alguém voltar a escrever o `match` à mão noutro ficheiro, que é exactamente o que este app já
/// fez **quatro vezes** antes de o `button_visual` nascer.
///
/// ⚠️ **A agulha são DUAS linhas, e a segunda é a que a torna honesta.** `Pressed => AccentSoft`
/// sozinha acusa a matriz do RAIL (`cluster_painter`, `tool_rail`), que é **deliberadamente
/// diferente**: o repouso dela é `BgElev`, não `Bg2` — um chip de rail assenta sobre o painel
/// elevado, e ali o hover move a BORDA, não a superfície. Um gate que a acusasse estaria a chamar
/// cópia a uma matriz que ninguém copiou. É o que separa *duas respostas para a mesma pergunta* de
/// *duas perguntas*.
#[test]
fn nobody_re_invents_the_hard_surface_map() {
    let offenders: Vec<String> = sources()
        .into_iter()
        .filter(|(p, s)| {
            !is_the_door_itself(p)
                && s.contains("ButtonState::Pressed => ColorToken::AccentSoft")
                && s.contains("_ => ColorToken::Bg2")
        })
        .map(|(p, _)| p.strip_prefix(root()).unwrap_or(&p).display().to_string())
        .collect();
    assert!(
        offenders.is_empty(),
        "estes sítios re-escreveram o mapa duro da superfície plana em privado — a porta é \
         `flat_button_surface_color(store.button_visual(id), theme)`, e uma segunda cópia do mapa \
         é a que fica para trás no dia em que o eixo mudar:\n  {}",
        offenders.join("\n  ")
    );
}

/// ⛔ **E NINGUÉM PEDE A SUPERFÍCIE DE UM `Normal` LITERAL** — a agulha sobrevive à privacidade
/// porque o nome ainda é escrevível DENTRO da crate.
///
/// ⚠️ `flat_button_surface(ButtonState::Normal)` devolve exactamente `ColorToken::Bg2`, e
/// escrevê-lo assim é dizer *"a superfície de repouso de um botão"* num sítio onde **não há botão
/// nenhum** — o `t` não está em falta, o estado é que é fingido. Quem quer o tom neutro nomeia o
/// token. Medido: um sítio fazia-o (o chip de modo do relevo).
#[test]
fn nobody_asks_the_flat_surface_for_a_literal_resting_state() {
    let offenders: Vec<String> = sources()
        .into_iter()
        .filter(|(p, s)| {
            !is_the_door_itself(p) && s.contains("flat_button_surface(ButtonState::Normal)")
        })
        .map(|(p, _)| p.strip_prefix(root()).unwrap_or(&p).display().to_string())
        .collect();
    assert!(
        offenders.is_empty(),
        "estes sítios pedem a superfície de um estado LITERAL — o tom é `ColorToken::Bg2`, e \
         nomeá-lo diz o que eles querem sem fingir um botão:\n  {}",
        offenders.join("\n  ")
    );
}
