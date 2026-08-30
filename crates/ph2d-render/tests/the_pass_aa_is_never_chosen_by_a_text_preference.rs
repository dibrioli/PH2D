//! **O anti-aliasing do passe não é uma preferência de texto.**
//!
//! Até 2026-08-30 o shell fazia isto, uma vez por quadro
//! (`shells/desktop/src/render_loop/present.rs`):
//!
//! ```ignore
//! let prefer_msaa = ph2d_editor::paint::text_rendering().params().prefer_msaa;
//! vello_pass.render_to_intermediate(gpu, scene, size, TRANSPARENT, prefer_msaa)?;
//! ```
//!
//! …e `render_to_intermediate` traduzia esse `bool` em
//! `AaConfig::Msaa16` ou `AaConfig::Area`.
//!
//! ⛔ **O erro não é o valor escolhido — é quem escolhe.** O `AaConfig`
//! do Vello vive em `RenderParams`, logo vale para o **PASSE INTEIRO**;
//! não existe um AA por caminho de desenho. Este passe carrega o chrome
//! do editor **e** a arte vectorial do documento no mesmo `Scene`, por
//! isso um preset de tipografia (`TextRendering::CrispHeavyPlus`)
//! escolhia a rasterização das formas do artista.
//!
//! ⭐ **O preço estava escrito no nosso próprio código, ao lado da
//! bandeira que o ligava**, em dois sítios: «pode stipplar strokes
//! vetoriais finos» (`ph2d-tokens`) e «MSAA16 produced visible
//! stippling on thin (1-1.5 px) vector strokes at near-axis angles»
//! (`ph2d-render`). E a justificação concedia o defeito na mesma frase:
//! «CrispHeavyPlus opts INTO Msaa16 … without re-introducing stipple
//! (glyphs aren't 1-1.5 px strokes at near-axis angles — **the problem
//! case is vectors**)» — certa sobre os glifos, cega quanto ao
//! `AaConfig` ser por passe. *Uma premissa que nomeia a vítima e
//! conclui que não há vítima.*
//!
//! Report do dono do produto: «manchas animadas parecendo TV antiga» à
//! volta das formas vectoriais — `docs/Atualizar Stack/04_registro.md`
//! §22.2 (mecanismo A).
//!
//! ⚠️ **A saída legítima é arquitectura, não uma bandeira:** chrome e
//! documento em **dois passes**, cada um com o seu `AaConfig` (segundo
//! alvo, segundo `Renderer`, mais uma composição). Enquanto isso não
//! existir, o passe é `AaConfig::Area` — analítico, melhor em traço
//! fino e mais barato — e ninguém lho pergunta.
//!
//! # A forma do gate
//!
//! Duas lentes, e cada uma carrega a **metade justa** — a asserção que
//! morre quando o próprio gate deixa de ver o produto. Um censo de
//! fonte que não encontra nada passa em todas as proibições e não prova
//! coisa nenhuma; é o modo de falha de todo gate que faz parse.
//!
//! 1. **Ninguém pode injectar uma preferência** — censo da workspace:
//!    `prefer_msaa` não existe, `AaConfig`/`Msaa16` só existem dentro
//!    de `crates/ph2d-render/src/`.
//! 2. **A decisão é uma constante** — em `vello_pass.rs`, todo
//!    `antialiasing_method:` vale `AaConfig::Area`, e nenhuma
//!    assinatura de render aceita um `bool`.

use std::path::{Path, PathBuf};

/// A ÚNICA pasta onde uma decisão de `AaConfig` pode viver. É a crate
/// que possui o `Renderer` do Vello e o `RenderParams` que o carrega.
const AA_OWNER: &str = "crates/ph2d-render/src";

/// O ficheiro que de facto escolhe. Nomeado para a metade justa: se ele
/// mudar de nome sem o gate mudar, o gate diz-o em vez de ficar verde.
const AA_DECIDER: &str = "crates/ph2d-render/src/vello_pass.rs";

/// Este ficheiro. Ele **nomeia** as três coisas que proíbe, nas
/// mensagens de erro e nas variáveis do censo — sem esta excepção o
/// gate acusa-se a si próprio, e o único jeito de o calar seria apagar
/// a explicação de porque a proibição existe.
///
/// ⚠️ A excepção tem a sua própria metade justa lá em baixo: o censo
/// **exige** ter encontrado este ficheiro. Um renomear silencioso
/// deixaria a excepção a apontar para nada — e uma excepção que não
/// exclui ninguém é uma linha morta a fingir que protege.
const SELF_PATH: &str =
    "crates/ph2d-render/tests/the_pass_aa_is_never_chosen_by_a_text_preference.rs";

/// Piso do censo. Não é um alvo — é o detector de walk partido. A
/// workspace tem milhares de `.rs`; se este número não for alcançado, o
/// gate não está a ler o que julga estar.
const MIN_FILES_SCANNED: usize = 500;

/// Raiz da workspace a partir da crate que hospeda o gate.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("workspace root")
}

/// Remove comentários de linha (`//`, e portanto também `///` e `//!`).
///
/// ⚠️ **Load-bearing:** a cura DEIXOU o porquê escrito em comentários
/// que nomeiam `AaConfig`, `Msaa16` e `prefer_msaa` — em `present.rs`,
/// em `typography.rs` e neste próprio ficheiro. Um censo sobre o texto
/// cru acusaria a documentação da cura de ser a doença, e o próximo
/// leitor apagaria a explicação para ficar verde.
///
/// Fronteira declarada: um `//` dentro de um literal de string é
/// tratado como comentário. Nenhuma das proibições abaixo é sobre
/// conteúdo de string, então o falso-negativo não tem sujeito aqui.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Todos os `.rs` de `crates/*/{src,tests}` e `shells/*/{src,tests}`.
/// ⛔ Não desce a `target/` porque nunca entra numa: só estas quatro
/// pastas por membro são visitadas.
fn workspace_sources(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for group in ["crates", "shells"] {
        let Ok(members) = std::fs::read_dir(root.join(group)) else {
            continue;
        };
        for member in members.flatten() {
            for sub in ["src", "tests"] {
                collect_rs(&member.path().join(sub), &mut out);
            }
        }
    }
    out.sort();
    out
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Caminho relativo à raiz, com `/` — para mensagens e para o teste de
/// pertença a [`AA_OWNER`].
fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// **Lente 1 — ninguém pode injectar uma preferência no passe.**
#[test]
fn the_pass_aa_is_never_chosen_by_a_text_preference() {
    let root = workspace_root();
    let files = workspace_sources(&root);

    let mut scanned = 0usize;
    let mut owner_hits = 0usize;
    let mut decider_hits = 0usize;
    let mut saw_self = false;
    let mut aa_outside: Vec<String> = Vec::new();
    let mut msaa_anywhere: Vec<String> = Vec::new();
    let mut prefer_msaa: Vec<String> = Vec::new();

    for path in &files {
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        scanned += 1;
        let code = strip_line_comments(&raw);
        let r = rel(&root, path);

        if r == SELF_PATH {
            saw_self = true;
            continue;
        }

        if code.contains("prefer_msaa") {
            prefer_msaa.push(r.clone());
        }
        if code.contains("Msaa16") {
            msaa_anywhere.push(r.clone());
        }
        if code.contains("AaConfig") {
            if r.starts_with(AA_OWNER) {
                owner_hits += 1;
                if r == AA_DECIDER {
                    decider_hits += code.matches("AaConfig").count();
                }
            } else {
                aa_outside.push(r.clone());
            }
        }
    }

    // ── METADE JUSTA ────────────────────────────────────────────────
    // Sem estas três, um walk partido (raiz errada, pasta renomeada,
    // strip que come o ficheiro) devolve listas vazias e o gate declara
    // vitória sobre nada.
    assert!(
        scanned >= MIN_FILES_SCANNED,
        "o censo leu só {scanned} ficheiros `.rs` (piso {MIN_FILES_SCANNED}) a partir de {} — \
         este gate não está a ler a workspace, e as proibições abaixo estão a passar por vazio, \
         não por limpeza",
        root.display()
    );
    assert!(
        owner_hits > 0,
        "o censo não encontrou UMA menção a `AaConfig` dentro de `{AA_OWNER}` — ou o dono da \
         decisão mudou de sítio (actualize `AA_OWNER`), ou o strip de comentários comeu o código. \
         Enquanto isto não for verdade, este gate não vê o produto a escolher um AA e não prova \
         nada."
    );
    assert!(
        saw_self,
        "o censo varreu {scanned} ficheiros e NÃO encontrou `{SELF_PATH}` — a auto-excepção deste \
         gate está a apontar para um caminho que já não existe. Ou o ficheiro foi renomeado \
         (actualize `SELF_PATH`), ou o walk não alcança `crates/*/tests`."
    );
    assert!(
        decider_hits >= 2,
        "`{AA_DECIDER}` menciona `AaConfig` só {decider_hits}× (esperadas ≥2: o `use` e ao menos \
         um `antialiasing_method:`). O ficheiro que decide o AA mudou de forma ou de nome — \
         reaponte `AA_DECIDER` antes de acreditar no verde."
    );

    // ── AS PROIBIÇÕES ───────────────────────────────────────────────
    assert!(
        prefer_msaa.is_empty(),
        "`prefer_msaa` voltou à árvore, em {:?}.\n\n\
         Ele foi removido em 2026-08-30 e a remoção é o produto, não uma limpeza: um preset de \
         TEXTO escolhia o `AaConfig` do passe INTEIRO, vectores do artista incluídos, e MSAA16 \
         stippla traços finos (1-1,5 px) em ângulos quase-axiais — «manchas animadas parecendo TV \
         antiga» (`docs/Atualizar Stack/04_registro.md` §22.2).\n\
         Se o objectivo é chrome e documento com AA diferentes, isso são DOIS PASSES — \
         arquitectura, não uma bandeira.",
        prefer_msaa
    );
    assert!(
        msaa_anywhere.is_empty(),
        "`AaConfig::Msaa16` reapareceu em {:?}.\n\n\
         O passe carrega chrome e arte vectorial no MESMO `Scene`, então isto não é uma escolha \
         sobre glifos: é sobre as formas do artista. Medido e escrito no `vello_pass.rs`: MSAA16 \
         stippla traços de 1-1,5 px em ângulos quase-axiais, e `Area` é mais suave E mais barato.",
        msaa_anywhere
    );
    assert!(
        aa_outside.is_empty(),
        "`AaConfig` apareceu FORA de `{AA_OWNER}`, em {:?}.\n\n\
         A decisão de anti-aliasing pertence a quem possui o `Renderer` do Vello. Um consumidor \
         que a nomeia é um consumidor a caminho de a injectar — foi exactamente assim que um \
         preset de tipografia acabou a decidir a rasterização das formas do documento.",
        aa_outside
    );
}

/// **Lente 2 — a decisão é uma constante, não um ramo.**
///
/// A lente 1 proíbe o vocabulário; esta prova a **forma**. Um `bool`
/// chamado `smooth`, `crisp` ou `quality` a chegar ao
/// `antialiasing_method` não usaria nenhuma palavra proibida e teria
/// exactamente o mesmo defeito.
#[test]
fn the_pass_aa_is_a_constant_and_no_render_signature_takes_a_flag() {
    let root = workspace_root();
    let path = root.join(AA_DECIDER);
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("ler {}: {e}", path.display()));
    let code = strip_line_comments(&raw);

    // ── METADE JUSTA: o produto tem de estar mesmo a escolher um AA ──
    let picks: Vec<&str> = code
        .match_indices("antialiasing_method:")
        .map(|(i, m)| {
            let rest = &code[i + m.len()..];
            let end = rest.find('\n').unwrap_or(rest.len());
            rest[..end].trim()
        })
        .collect();
    assert!(
        picks.len() >= 2,
        "encontrei {} atribuição(ões) de `antialiasing_method:` em `{AA_DECIDER}` (esperadas ≥2: \
         `render_to_intermediate` e `render`). Ou o passe deixou de escolher um AA, ou \
         este parse deixou de o ver — nos dois casos o verde abaixo não vale nada.",
        picks.len()
    );

    for (i, value) in picks.iter().enumerate() {
        assert_eq!(
            *value, "AaConfig::Area,",
            "a atribuição #{i} de `antialiasing_method` vale `{value}` em vez da constante \
             `AaConfig::Area,`.\n\n\
             Um `if`/`match`/parâmetro aqui é o defeito de 2026-08-30 a voltar: o `AaConfig` do \
             Vello é por PASSE, e este passe carrega o chrome E a arte vectorial do documento. \
             Quem escolhe aqui escolhe pelo artista.\n\
             Chrome e documento com AA diferentes = DOIS PASSES (segundo alvo, segundo \
             `Renderer`, mais uma composição). Ver `docs/Atualizar Stack/04_registro.md` §22.2."
        );
    }

    // ── Nenhuma assinatura de render aceita uma bandeira ─────────────
    // A porta pela qual a preferência entrava era um `bool` no
    // parâmetro. Fechá-la na assinatura é mais forte que a proibir por
    // nome: `prefer_msaa`, `smooth_text` e `crisp: bool` são o mesmo
    // defeito com três rótulos.
    //
    // ⚠️ O parêntese fecha-se por PROFUNDIDADE, não pelo primeiro `)`.
    // A 1.ª versão deste censo usava `rest.find(')')` e parava no
    // `size: (u32, u32)` — cortando a lista ANTES do último parâmetro,
    // que é exactamente onde `prefer_msaa: bool` vivia. *Um parse que
    // desiste no primeiro fecho lê a metade inofensiva da assinatura.*
    let mut signatures = 0usize;
    for (i, _) in code.match_indices("pub fn render") {
        let rest = &code[i..];
        let Some(open) = rest.find('(') else { continue };
        let mut depth = 0i32;
        let mut close = None;
        for (j, c) in rest[open..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(open + j);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(close) = close else { continue };
        let name = rest[..open].trim_start_matches("pub fn ").trim();
        let args = &rest[open + 1..close];
        signatures += 1;
        assert!(
            !args.contains("bool"),
            "`{name}` aceita um `bool` na assinatura:\n({args})\n\n\
             Toda bandeira booleana numa função de render deste passe é uma preferência de \
             chamador a caminho do `antialiasing_method` — que é por PASSE e vale para os \
             vectores do documento. Foi assim que `prefer_msaa` nasceu."
        );
    }
    assert!(
        signatures >= 3,
        "o parse achou {signatures} assinatura(s) `pub fn render*` em `{AA_DECIDER}` (esperadas \
         ≥3: `render_to_intermediate`, `render_and_readback`, `render`). O ficheiro mudou de \
         forma — este gate não está a inspeccionar as funções que julga."
    );
}
