//! **A contenção é ESTRUTURAL, não disciplinar** — irmão do `the_event_core_is_a_leaf` do
//! `ph2d-runtime`, com **uma** diferença deliberada.
//!
//! O plano manda esta crate nascer folha porque o pedido do Enio é que a máquina de estados do
//! Morph seja *"funcional no runtime do game"*, e o `shells/game` (R1) está adiado: uma lei de
//! entrada que morasse na shell do **editor** seria, por construção, inalcançável do runtime — e
//! reescrevê-la lá seria a segunda porta que este repo já pagou seis vezes.
//!
//! ⚠️ **Aqui a propriedade NÃO é "zero" — é uma allowlist de UM**, e a diferença tem motivo. O
//! `ph2d-runtime` carrega um tipo que o mixer de áudio lê num contexto de tempo real; esta crate
//! carrega **conteúdo autorado que viaja no `.ph2dproj`**, e serialização é a razão de ela existir.
//! Uma allowlist envelhece pior que um "zero" — o preço de a ter é que ela **nomeia** o que
//! permite e **porquê**, e qualquer acrescento tem de editar esta linha, deliberadamente.
//!
//! ⛔ O que este gate impede, concretamente: `winit` (a shell normaliza antes), `bevy_ecs` (o
//! runtime do jogo não pode ser obrigado a ter um World para ler um botão) e qualquer crate de
//! editor.

use std::path::Path;

/// As secções cujo conteúdo VIAJA para quem depende desta crate.
///
/// ⚠️ `[dev-dependencies]` fica de fora de propósito: ela não alcança consumidor nenhum.
const REACHES_CONSUMERS: &[&str] = &["[dependencies]", "[build-dependencies]"];

/// **A allowlist, e ela é de UM.** Ver o cabeçalho para o motivo.
const ALLOWED: &[&str] = &["serde"];

#[test]
fn the_input_map_depends_on_nothing_but_serde() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read ph2d-input/Cargo.toml");

    // ⚠️ O controle POSITIVO, e ele vem PRIMEIRO: um gate que procura uma secção ausente encontra
    // zero deps e passa provando nada — o modo de falha de todo gate baseado em parser.
    assert!(
        text.contains("[dependencies]"),
        "a secao `[dependencies]` sumiu do manifesto. Ou o ficheiro foi renomeado, ou este gate \
         passou a medir a coisa errada -- em nenhum dos dois casos ele esta' a provar que a crate \
         e' uma folha."
    );
    // O segundo controle positivo: a allowlist tem de ser ALCANCADA. Se o `serde` sair do
    // manifesto sem este gate reclamar, o parser deixou de ver a linha que devia ver.
    let declared = deps_in(&text, "[dependencies]");
    assert!(
        declared.iter().any(|d| d == "serde"),
        "o `serde` desapareceu do manifesto ({declared:?}). Ou a crate deixou de ser \
         serializavel -- e entao ela deixou de poder viajar no .ph2dproj --, ou este parser nao \
         esta' a ler as dependencias de todo."
    );

    for section in REACHES_CONSUMERS {
        let extra: Vec<String> = deps_in(&text, section)
            .into_iter()
            .filter(|d| !ALLOWED.contains(&d.as_str()))
            .collect();
        assert!(
            extra.is_empty(),
            "`ph2d-input` ganhou dependencias fora da allowlist em `{section}`: {extra:?}.\n\n\
             Esta crate e' a lei de entrada que o EDITOR e o RUNTIME DO JOGO partilham. Toda dep \
             aqui e' paga pelos dois -- e o ponto inteiro de ela ser folha e' que o runtime nao \
             precise de arrastar winit nem um World para saber que o jogador carregou num botao.\n\
             Se a dep e' mesmo necessaria, ACRESCENTE-A a' ALLOWED deste gate com o motivo \
             escrito. A friccao e' deliberada."
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

// ⛔ **AQUI ESTAVA UM SEGUNDO GATE, e ele foi REMOVIDO no dia em que nasceu (2026-08-24).**
//
// Ele varria o `src/` à procura de `winit::` / `bevy_ecs::` / `ph2d_editor_core::` "escrito pelo
// nome". Falhou na primeira corrida — e **acusou um comentário**: o doc-header do `keyboard.rs`
// explica que a shell normaliza para o mesmo espaço de keycode que o
// `ph2d_editor_core::interaction::dispatch::keymap` consome. Prosa, não código.
//
// ⚠️ E o defeito não era o alcance da agulha, era o gate **não provar nada**: uma crate não
// consegue escrever `use winit::` sem a dependência, e a dependência é exactamente o que o gate
// acima já recusa — inclusive por `path`, porque a allowlist compara **nomes**. Um gate que só
// pode disparar sobre documentação, e que não apanha nada que o irmão deixe passar, é ruído — e
// ruído acaba silenciado por alguém a afrouxar a barra, que é como se perde o gate que importa.
//
// *A contenção real é a lista de dependências; o compilador é quem a aplica.*
