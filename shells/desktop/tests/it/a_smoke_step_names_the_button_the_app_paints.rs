//! ⭐⭐⭐ **UM PASSO DE SMOKE NOMEIA O BOTÃO QUE O APP DE FACTO PINTA.**
//!
//! # Porque este gate nasce (F4.6c, 2026-09-07)
//!
//! O roteiro da cena `PH2D_BUILD_SMOKE=53` mandava carregar em **Create Component** e depois em
//! **Place Instance**. Aqueles dois rótulos deixaram de existir quando a secção passou a falar o
//! modelo geral — hoje o painel escreve **Make Prefab** e **Instantiate** —, e o roteiro
//! continuou a nomeá-los durante todo esse tempo, **com a suíte verde**.
//!
//! ⚠️ É a lei do `CLAUDE.md` §5.0 à letra: *uma cena de smoke que ensina o CONTRÁRIO do que
//! acontece é pior que uma cena ausente* — a ausente não é acreditada. E é o mesmo mecanismo que
//! a `line/components` já pagou em 2026-09-06 (§F5.13), quando três passos mandavam clicar numa
//! linha que a Hierarquia deixara de mostrar: **o gate que existia media a presença de uma FRASE,
//! e a frase continuava lá.**
//!
//! # A régua, e porque ela é NESTA direcção
//!
//! Para cada chave de i18n que a secção *Prefab* **pinta**, o roteiro tem de conter o **valor**
//! dela. ⇒ renomear um botão sem tocar no roteiro fica **vermelho**, que é exactamente o defeito
//! que ninguém viu.
//!
//! ⛔ **A direcção inversa não serve como lei:** exigir que todo `**negrito**` do roteiro seja um
//! rótulo do painel proibiria o dono de enfatizar uma frase (*«a PROVA DA WAVE»*), e o roteiro
//! deixaria de poder falar português. *Uma régua que obriga o texto a ser uma lista de botões
//! deixa de medir o texto que o artista lê.*
//!
//! ⚠️ **Os valores saem do `tr`, nunca escritos aqui** — uma cópia do rótulo neste ficheiro seria
//! a terceira resposta a *«como se chama este botão?»*, e a que envelhece calada.

// ⭐ **A família das instâncias saiu da shell em 2026-09-12** (W2 Fase D): os ficheiros dela
// vivem em `crates/ph2d-app-components/src/`, e este censo mede-a **de fora**. Apontar para fora é
// legítimo e tem precedente (HOWTO §2.6: os ~53 gates de arquitectura do `ph2d-editor-core` varrem
// `shells/desktop/src` da mesma maneira). ⛔ O caminho fica ESCRITO, e não escondido atrás de um
// «tenta aqui, senão ali»: um fallback aceitaria em silêncio o ficheiro errado no dia em que os
// dois existirem.

use std::path::Path;

/// O corpo da cena, para varrer.
fn scene_source(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// ⭐ **Os verbos que a cena `=53` manda exercer, por CHAVE.**
///
/// ⚠️ **Uma lista de chaves e não de rótulos** — é o que faz o gate sobreviver a uma tradução e
/// morder numa renomeação. E ela é curta de propósito: são os verbos que o roteiro **percorre**,
/// não todos os que a secção conhece (o `Swap` e o `Apply` têm cena própria).
const STEPS: &[&str] = &[
    "panel.vector.component.create",
    "panel.vector.component.place",
    "panel.vector.component.edit",
    "panel.vector.component.detach",
    "panel.vector.component.reset",
];

/// **Mutação que deve sangrar:** trocar qualquer `eprintln!` do roteiro por um rótulo antigo
/// (`Create Component`, `Place Instance`) — o gate nomeia a chave e o valor que falta.
#[test]
fn the_component_smoke_names_the_buttons_the_panel_paints() {
    let body = scene_source("../../../crates/ph2d-app-components/src/component_smoke.rs");
    let mut faltam = Vec::new();
    for key in STEPS {
        let label = ph2d_i18n::tr(key);
        assert!(
            !label.is_empty() && label != *key,
            "a chave `{key}` nao tem traducao — o gate estaria a medir o proprio nome dela"
        );
        if !body.contains(label) {
            faltam.push((*key, label));
        }
    }
    assert!(
        faltam.is_empty(),
        "o roteiro da cena =53 nao nomeia botao(oes) que a seccao Prefab pinta: {faltam:?} — um \
         passo que manda carregar num rotulo que nao existe ensina o contrario do que acontece"
    );
}

/// ⛔⛔ **E ele NÃO pode nomear os controlos que morreram com o motor velho.**
///
/// A lista de PEÇAS e a fileira de VARIANTS do painel vetorial deixaram de ser pintadas (F4.6c) —
/// as capacidades vivem noutro gesto (o olho da Hierarquia, o cartão do Inspector), e um passo que
/// mandasse procurá-las na secção *Prefab* mandaria o dono procurar o que não está lá.
///
/// ⚠️ **A régua são as chaves ÓRFÃS**, e ela é derivada: se alguém voltar a pintar aquelas linhas,
/// a cura é apagar a entrada daqui — não é o gate que se afrouxa.
#[test]
fn the_component_smoke_does_not_send_the_owner_after_a_dead_control() {
    let body = scene_source("../../../crates/ph2d-app-components/src/component_smoke.rs");
    let mut fantasmas = Vec::new();
    for key in [
        "panel.vector.component.pieces",
        "panel.vector.component.piece_colour",
        "panel.vector.component.variant",
    ] {
        let label = ph2d_i18n::tr(key);
        if body.contains(&format!("**{label}**")) {
            fantasmas.push((key, label));
        }
    }
    assert!(
        fantasmas.is_empty(),
        "o roteiro manda o dono usar um controlo que o painel ja' nao pinta: {fantasmas:?}"
    );
}
