//! **TODO VERBO DO INSPECTOR DIZ SE ESPALHA PELA SELEÇÃO — OU DECLARA POR ESCRITO QUE NÃO.**
//!
//! # O defeito que este gate fecha
//!
//! Auditoria de 2026-08-21 ([`docs/Sprite_projeto/20`](../../../docs/Sprite_projeto/20_auditoria_do_inspector_2026-08-21.md) §3):
//! na **mesma seção** do Inspector, o toggle *Region* mudava as cinco sprites selecionadas e
//! *Strategy* / *Format* / *Emissive* mudavam **uma** — sem nenhuma diferença visual. E a caixa
//! *Visible* do topo editava uma enquanto a §8 Visibility logo abaixo editava todas.
//!
//! ⚠️ **Isto não dá erro, não dá aviso e é invisível numa seleção de um.** O artista só descobre
//! quando conta as sprites que mudaram — e a spec diz o contrário por escrito
//! ([`03_inspector_secoes.md`](../../../docs/Sprite_projeto/03_inspector_secoes.md) §3.14:
//! *«edit em qualquer campo aplica imediatamente a TODOS selecionados»*).
//!
//! # Por que a lista de exceções vive AQUI, e por que isso não é a doença
//!
//! ⚠️ Uma condição que enumera os seus leitores apodrece — foi assim que os pontos de cor morreram
//! (§1.2 da mesma auditoria). Mas isto **não é uma condição de produto**: é a barra do gate, e a
//! diferença é que ela **acusa quem falta**. Um verbo novo que não espalhe e não esteja aqui
//! reprova, e quem o escreveu tem de escolher: espalhar, ou dizer porquê. *Uma exceção que custa
//! uma linha de justificação é uma exceção que alguém pensou.*
//!
//! # A fronteira: por que estes três NÃO espalham
//!
//! `Reimport` · `Strategy` · `Format` são **conversões destrutivas** — re-decodificam, re-alojam
//! pixels e podem largar a autoria de folha. A spec pede confirmação modal para campos pesados
//! (§3.14, gate `bulk_edit_confirmation_required_fields`) e essa UI **não existe**; espalhá-los
//! sem ela trocaria um sub-aplicar silencioso por uma conversão em massa silenciosa, que é pior.
//! ⛔ Quando a confirmação existir, estes três saem desta lista.

use std::path::{Path, PathBuf};

fn drain_source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop/mod.rs")
}

/// Os verbos que **não** espalham, cada um com a razão pela qual isso é uma decisão.
///
/// ⚠️ As entradas de `Name`/`Signal` não são exceções de produto: um nome é por-entidade por
/// definição (dar o mesmo nome a cinco sprites destrói a única referência durável que o projeto
/// tem — `stable_name_id`).
const DOES_NOT_FAN_OUT: &[(&str, &str)] = &[
    (
        "InspectorSpritePrecisionChange",
        "conversao destrutiva: re-codifica os pixels e pode largar a folha; espera a confirmacao modal da spec §3.14",
    ),
    (
        "InspectorSpriteSourceChange",
        "conversao destrutiva: re-aloja os pixels (atlas <-> textura propria); mesma espera",
    ),
    (
        "Reimport",
        "conversao destrutiva: re-decodifica o asset ao px/m atual; mesma espera",
    ),
    (
        "InspectorNameEdit",
        "um nome e' por-entidade por definicao — cinco sprites com o mesmo nome destroem o `stable_name_id`",
    ),
    (
        "InspectorSignalEdit",
        "idem: o sinal de entrada e' nomeado, e o nome e' a referencia duravel",
    ),
    (
        "InspectorSignalLeaveEdit",
        "idem: o sinal de saida e' nomeado, e o nome e' a referencia duravel",
    ),
    (
        "InspectorTransformEdit",
        "⏸️ AINDA NAO: `InspectorTransformInfo` nao carrega `selected_count` nem `mixed`, entao espalhar hoje esmagaria posicoes divergentes SEM sinal. Espalhar exige primeiro o par flag+afordancia, como a Visible ganhou",
    ),
];

/// O corpo do dreno, sem comentários — vários deles **citam** `inspector_selection` a explicar a
/// regra, e um comentário não espalha nada.
fn drain_body() -> String {
    let src = std::fs::read_to_string(drain_source()).expect("ler o dreno");
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// O bloco de código de um braço `EditorAction::<nome>` até ao braço seguinte.
fn arm_body<'a>(body: &'a str, action: &str) -> Option<&'a str> {
    let needle = format!("EditorAction::{action}");
    let start = body.find(&needle)?;
    let rest = &body[start + needle.len()..];
    let end = rest.find("EditorAction::").unwrap_or(rest.len());
    Some(&rest[..end])
}

/// **A varredura.** Todo braço da família ou espalha, ou está declarado.
#[test]
fn every_inspector_verb_either_fans_out_or_says_why_not() {
    let body = drain_body();
    // Controlo positivo: sem isto a varredura poderia medir um ficheiro vazio e passar.
    assert!(
        body.contains("for &t in &inspector_selection"),
        "o dreno nao contem fan-out nenhum — a varredura partiu-se e este gate mede o vazio"
    );

    let mut silent: Vec<String> = Vec::new();
    for action in [
        "InspectorSpriteEdit",
        "InspectorOrderingEdit",
        "InspectorSamplingEdit",
        "InspectorBlendEdit",
        "InspectorVisibilitySectionEdit",
        "InspectorVisibilityEdit",
        "InspectorSpriteEmissiveChange",
    ] {
        let Some(arm) = arm_body(&body, action) else {
            silent.push(format!("{action} (braco NAO ENCONTRADO no dreno)"));
            continue;
        };
        if !arm.contains("for &t in &inspector_selection") {
            silent.push(format!("{action} (nao espalha e nao esta' declarado)"));
        }
    }
    assert!(
        silent.is_empty(),
        "estes verbos do Inspector editam so' a entidade PRIMARIA numa selecao multipla, sem \
         nenhuma diferenca visual face aos vizinhos que espalham:\n  {}\n\n\
         Ou espalhe (`for &t in &inspector_selection`), ou acrescente uma entrada a \
         `DOES_NOT_FAN_OUT` neste ficheiro com a RAZAO.\n\
         ⚠️ A spec §3.14 diz «edit em qualquer campo aplica imediatamente a TODOS selecionados» — \
         a excecao e' que precisa de justificacao, nao a regra.",
        silent.join("\n  ")
    );
}

/// **Cada exceção declarada é REAL** — ela não espalha mesmo.
///
/// ⚠️ Sem isto, a lista viraria um cemitério: alguém espalha um verbo, esquece de o tirar daqui, e
/// a próxima pessoa lê que ele «não espalha por ser destrutivo» sobre código que espalha. *Uma
/// allowlist sem controlo positivo é um comentário velho com sintaxe de código.*
#[test]
fn every_declared_exception_is_still_an_exception() {
    let body = drain_body();
    let mut stale: Vec<&str> = Vec::new();
    for (action, _reason) in DOES_NOT_FAN_OUT {
        if let Some(arm) = arm_body(&body, action)
            && arm.contains("for &t in &inspector_selection")
        {
            stale.push(action);
        }
    }
    assert!(
        stale.is_empty(),
        "estes verbos ESTAO declarados como «nao espalha» e espalham: {stale:?}.\n\
         Tire-os de `DOES_NOT_FAN_OUT` — a razao escrita ao lado deles ja' nao descreve o codigo."
    );
}

/// **Toda exceção traz uma razão legível**, e não um `""` para calar o gate.
#[test]
fn no_exception_is_declared_without_a_reason() {
    for (action, reason) in DOES_NOT_FAN_OUT {
        assert!(
            reason.len() > 30,
            "a excecao `{action}` tem uma razao de {} caracteres — uma excecao sem razao e' um \
             silenciamento com sintaxe de documentacao",
            reason.len()
        );
    }
}
