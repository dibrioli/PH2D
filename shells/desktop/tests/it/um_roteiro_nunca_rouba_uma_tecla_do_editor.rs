//! ⭐⭐⭐ **UM ROTEIRO QUE NOMEIA UMA TECLA AFIRMA QUE ELA FAZ AQUILO — e o editor pode já a ter.**
//!
//! Report do dono, 2026-09-19: *«conflito com atalho Home que é o view selected»*. O passo (5) do
//! roteiro da `=1` do golpe mandava carregar no `Home` para **rebobinar**, e o `Home` é o *frame
//! selection* do editor ([`handlers_teclas_editor`], braço `KeyCode::Home | KeyCode::KeyF`) — o dono
//! carregou, a **câmera** saltou, e o relógio ficou onde estava.
//!
//! ⚠️⚠️ **Nenhuma régua desta linha podia ver isto.** O gate irmão
//! (`o_roteiro_nomeia_rotulos_que_existem`) resolve **rótulos pintados** do i18n e afirma que o
//! roteiro os contém — ele mede o que ESTÁ na tela, nunca o que uma TECLA faz. *Um passo que nomeia
//! uma linha de painel já era uma afirmação gateada desde o #15; um passo que nomeia uma TECLA era
//! a mesma afirmação sem régua nenhuma.*
//!
//! ⛔ **A régua é DERIVADA do despacho, nunca uma lista escrita à mão** — uma tecla nova reclamada
//! pelo editor entra nesta população no mesmo commit em que nasce, e um roteiro que a nomeie reprova
//! sem ninguém se lembrar de estender nada.
//!
//! ⚠️ **A GUARDA é metade da lei.** As setas também aparecem no despacho
//! (`KeyCode::ArrowUp | KeyCode::ArrowDown if self.flip_state.active`) e **não** são roubadas: com o
//! Flip desligado elas chegam ao jogador, que é o que faz o passo (4) do mesmo roteiro funcionar.
//! ⇒ a extracção salta os braços com guarda, e o **controlo negativo** dentro do gate é precisamente
//! `ArrowUp` — *sem ele, uma extracção que ignorasse o `if` acusaria o passo CERTO e a cura seria
//! apagar a instrução que funciona*.
//!
//! ⚠️ **Só as teclas de NOME entram na varredura** (`Home`, `Space`, `Comma`, …). As de LETRA
//! (`KeyF`, `KeyZ`, …) ficam de fora com motivo medido: um roteiro escreve `Q`, `F` e `Z` no meio da
//! prosa dezenas de vezes, e uma régua que os caçasse mediria a língua portuguesa, não o despacho.
//! *O que o dono encontrou era de nome, e é essa a população que esta régua cobre.*
//!
//! ⚠️ **A varredura é sobre o TEXTO IMPRESSO, nunca sobre o ficheiro** — as duas cenas curadas em
//! 19/09 levam agora, no comentário, a frase *«⛔ não é o `Home`»*, e um censo que não separasse
//! prosa de literal acusaria a própria cura (a lei do `§5.0`: um censo textual que não separa os
//! dois mente nos DOIS sentidos).

use std::collections::BTreeSet;

/// O despacho de teclas do editor. `include_str!` de propósito: se ele mudar de sítio, isto **não
/// compila**, que é a espécie barulhenta.
const DESPACHO: &str = include_str!("../../src/input_dispatch/handlers_teclas_editor.rs");

/// Onde vivem os roteiros que esta shell monta.
fn pasta_das_cenas() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/ph2d-app-components/src")
}

/// Uma tecla é «de NOME» quando não é `KeyA`..`KeyZ` — ver o cabeçalho.
fn e_de_nome(k: &str) -> bool {
    !(k.len() == 4 && k.starts_with("Key"))
}

/// **As teclas que o editor reclama SEM GUARDA**, lidas do despacho.
///
/// ⚠️ O discriminador é o ` if ` na linha do braço: é assim que o `match` de Rust escreve uma
/// guarda, e é o que separa *«o editor come esta tecla»* de *«o editor come-a só neste modo»*.
fn teclas_incondicionais() -> BTreeSet<String> {
    let mut fora = BTreeSet::new();
    for linha in DESPACHO.lines() {
        let t = linha.trim();
        if t.starts_with("//") || !t.contains("KeyCode::") || !t.contains("=>") {
            continue;
        }
        if t.contains(" if ") {
            continue;
        }
        let mut resto = t;
        while let Some(i) = resto.find("KeyCode::") {
            resto = &resto[i + "KeyCode::".len()..];
            let fim = resto
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(resto.len());
            let nome = &resto[..fim];
            if !nome.is_empty() && e_de_nome(nome) {
                fora.insert(nome.to_string());
            }
            resto = &resto[fim..];
        }
    }
    fora
}

/// O texto que o artista de facto LÊ: as linhas do ficheiro que não são comentário.
///
/// ⛔ Não tenta ser um parser de Rust — a cerca é a inversa: uma linha de comentário **nunca** chega
/// ao terminal do dono, logo saltá-la só pode tornar a régua mais fraca, nunca dar falso positivo.
fn texto_impresso(fonte: &str) -> String {
    fonte
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Que teclas do editor este texto nomeia **como tecla** (entre crases).
fn roubos(texto: &str, teclas: &BTreeSet<String>) -> Vec<String> {
    teclas
        .iter()
        .filter(|k| texto.contains(&format!("`{k}`")))
        .cloned()
        .collect()
}

/// ⭐⭐⭐ **Nenhum roteiro manda carregar numa tecla que o editor já come.**
///
/// **Mutações que devem sangrar:** repor `` `Home` rebobina `` no roteiro do golpe · deixar a
/// extracção aceitar braços com guarda (o controlo do `ArrowUp`) · varrer o ficheiro inteiro em vez
/// do texto impresso (a cura de 19/09 cita o `Home` no comentário de propósito).
#[test]
fn um_roteiro_nunca_rouba_uma_tecla_do_editor() {
    let teclas = teclas_incondicionais();

    // ── O CONTROLO da extracção: as DUAS metades, e cada uma sozinha mente ──────────────
    // Sem a positiva, um filtro que devolvesse o conjunto VAZIO deixaria tudo verde por vácuo.
    assert!(
        teclas.contains("Home"),
        "a extraccao perdeu o `Home`, que e' o caso MEDIDO (report do dono, 19/09) — a regua deixou \
         de medir o despacho. Leu: {teclas:?}"
    );
    // Sem a negativa, uma extracção que ignorasse a guarda acusaria o passo (4) do golpe, que é
    // CERTO: com o Flip desligado as setas chegam ao jogador.
    assert!(
        !teclas.contains("ArrowUp") && !teclas.contains("ArrowDown"),
        "as setas entraram como incondicionais — elas TEM guarda (`if self.flip_state.active`), e \
         acusa-las apagaria a instrucao que funciona. Leu: {teclas:?}"
    );

    // ── A varredura ────────────────────────────────────────────────────────────────────
    let pasta = pasta_das_cenas();
    let mut cenas = 0usize;
    let mut bytes = 0usize;
    let mut acusados: Vec<String> = Vec::new();
    for e in std::fs::read_dir(&pasta).expect("a pasta das cenas existe") {
        let p = e.expect("entrada").path();
        let Some(nome) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !nome.ends_with("_smoke.rs") {
            continue;
        }
        let fonte = std::fs::read_to_string(&p).expect("le a cena");
        let texto = texto_impresso(&fonte);
        cenas += 1;
        bytes += texto.len();
        for k in roubos(&texto, &teclas) {
            acusados.push(format!("{nome}: `{k}`"));
        }
    }

    // ⛔ PISO DE POPULAÇÃO nas duas grandezas — uma varredura partida devolve zero acusados e
    // lê-se exactamente como aprovada (a lei do §5.0 sobre censos por prefixo de nome).
    assert!(
        cenas >= 15,
        "a varredura leu {cenas} cenas em {} — o censo partiu-se e um zero de «nao ha roubos» \
         le-se igual a um de «nao medi nada»",
        pasta.display()
    );
    assert!(
        bytes > 50_000,
        "o texto impresso das {cenas} cenas soma {bytes} bytes — o filtro de comentario comeu o \
         ficheiro e a regua esta' a medir o vazio"
    );

    assert!(
        acusados.is_empty(),
        "estes roteiros mandam carregar numa tecla que o EDITOR ja' come sem guarda {teclas:?} — o \
         dono carrega e acontece OUTRA coisa (o `Home` move a camera, nao o relogio). Nomeie o \
         controlo que existe na tela (os chips `Play` / `Pause` / `Reset` da barra de cima):\n  {}",
        acusados.join("\n  ")
    );

    // ── O CONTROLO POSITIVO da varredura: a frase que o dono encontrou é apanhada ───────
    // Sem ele, um `roubos` que devolvesse sempre vazio deixaria a asserção acima verde para sempre.
    let defeito = "(5) carregue em STOP na regua de baixo: o Q deixa de disparar. `Home` rebobina";
    assert_eq!(
        roubos(defeito, &teclas),
        vec!["Home".to_string()],
        "a varredura nao apanha a PROPRIA frase do report de 19/09 — ela nao mede nada"
    );
}
