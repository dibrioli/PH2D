//! ⛔⛔⛔ **UMA PALAVRA PARA A COISA REUTILIZÁVEL** — report do Enio, 2026-09-06:
//! *«Por que usou Component em vez de prefab? São coisas diferentes?»*
//!
//! # A resposta é NÃO, e a pergunta era um defeito medido
//!
//! O app dizia a mesma coisa de **três** maneiras, na tela:
//!
//! | onde | o que dizia |
//! |---|---|
//! | menu da Hierarquia | *Make Prefab* · Apply to **Master** · Revert to **Master** · Detach from **Master** |
//! | painel vetorial | **Component** · Create **Component** · Update **Main** · Swap **Main** · **Main** missing |
//! | toasts | *«…to master»* · *«…the component still has it»* · *«…is not a component»* |
//!
//! ⚠️⚠️ **E uma das três palavras JÁ SIGNIFICA OUTRA COISA neste app:** o Inspector tem um botão
//! **Add Component**, e ali *componente* é um `Sprite`/`RigidBody`/`Transform`. ⇒ dois botões
//! adjacentes, *Add Component* e *Create Component*, sobre conceitos sem relação. *Uma palavra que
//! já tem dono não pode ser emprestada — e a pergunta do dono é a prova de que ela não foi.*
//!
//! # A palavra que fica é **prefab**, e a escolha é medida
//!
//! Ela já era a das duas portas que **criam** e **abrem** a coisa (*Make Prefab*, *Edit Prefab*), é
//! a da indústria, é a que o **dono** usou ao perguntar — e é a única das quatro que **não colide**
//! com nada. `Master`, `Main` e `Component` eram invenções nossas ou empréstimos ocupados.
//!
//! # ⚠️ Porque o censo é por FICHEIRO, e não pelo repo
//!
//! *Componente* é uma palavra legítima em quase todo o lado (a paleta do Inspector, o
//! `ph2d-component-desc`, os 108 tipos). O que este censo defende é a **família da coisa
//! reutilizável**: os ficheiros abaixo são os que falam com o artista **sobre ela**.

use std::path::Path;

/// Os ficheiros que escrevem, na tela, sobre a coisa reutilizável.
const SURFACES: &[&str] = &[
    "shells/desktop/src/instance_verbs.rs",
    "shells/desktop/src/instance_revert.rs",
    "shells/desktop/src/instance_unmake.rs",
    "shells/desktop/src/asset_card_verbs.rs",
    "shells/desktop/src/vec_component_general.rs",
    "shells/desktop/src/render_loop/hierarchy_delete.rs",
    "shells/desktop/src/hero_intents/hierarchy.rs",
];

/// As tabelas de rótulos — o que o painel e os menus pintam.
const LABEL_TABLES: &[&str] = &[
    "crates/ph2d-i18n/src/vector.rs",
    "crates/ph2d-editor-core/src/screens/hero/menu_rows.rs",
];

/// As palavras que a coisa reutilizável **não** pode ter na tela.
const BANNED: &[&str] = &["Master", "master", "Component", "component", "Main missing"];

/// Todo `.rs` sob `shells/desktop/src` — a população do censo dos fallbacks, **derivada**.
fn rust_files_of_the_shell() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn repo_root() -> &'static Path {
    // `CARGO_MANIFEST_DIR` é `shells/desktop`.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a raiz do repo")
}

/// Os literais de string de um ficheiro, **fora** de comentários.
fn screen_strings(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in body.lines() {
        let code = match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        };
        let mut rest = code;
        while let Some(a) = rest.find('"') {
            let after = &rest[a + 1..];
            let Some(b) = after.find('"') else { break };
            out.push(after[..b].to_string());
            rest = &after[b + 1..];
        }
    }
    out
}

/// ⭐⭐⭐ **Nenhuma frase de tela desta família diz `master`, `main` ou `component`.**
///
/// ⚠️ **A régua é a FRASE, não o identificador** — os nomes de tipo (`VecComponentMain`), de
/// função (`create_main`) e as chaves de i18n (`panel.vector.component.place`) continuam como
/// estão: eles não aparecem na tela, e renomeá-los era um segundo trabalho com outro risco.
///
/// **Mutação que deve sangrar:** pôr de volta qualquer um dos rótulos antigos.
#[test]
fn no_screen_sentence_about_the_reusable_thing_uses_the_old_words() {
    let root = repo_root();
    let mut offenders: Vec<String> = Vec::new();
    for rel in SURFACES {
        let Ok(body) = std::fs::read_to_string(root.join(rel)) else {
            panic!("o ficheiro {rel} mudou de sitio — reancore este censo");
        };
        for s in screen_strings(&body) {
            // Só frases: um literal sem espaço é um nome de chave, de ficheiro ou de env var.
            if !s.contains(' ') {
                continue;
            }
            if let Some(w) = BANNED.iter().find(|w| s.contains(**w)) {
                offenders.push(format!("{rel}: {w:?} em {s:?}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "o app voltou a ter mais de uma palavra para a coisa reutilizavel — e uma delas ja' \
         significa outra coisa (o `Add Component` do Inspector):\n  {}",
        offenders.join("\n  ")
    );
}

/// ⛔⛔ **E o NOME DE RECURSO de uma receita sem nome também é tela** (achado de 2026-09-06).
///
/// O censo acima só olha **frases** — literais com espaço — porque uma palavra sozinha costuma ser
/// chave de i18n, nome de ficheiro ou de variável de ambiente. ⚠️ **Isso deixou passar duas**:
/// `master_named(..).unwrap_or_else(|| "component".to_string())`, que vai para dentro de aspas
/// curvas num toast (*«Editing “component”»*). *Uma exclusão desenhada para reduzir falsos
/// positivos é um buraco com a forma exacta do que ela exclui.*
///
/// ⚠️ **A população é DERIVADA, e não uma lista escrita à mão** — todo `.rs` da shell. A 1.ª
/// redacção deste censo reusou a lista `SURFACES` acima e leu **duas** ocorrências; a árvore tinha
/// **seis**, e as outras quatro (o cartão do Inspector, a escada do *Aplicar*, a fileira de
/// versões) são as que o artista lê com mais frequência. *Um censo que herda a lista do vizinho
/// herda também o recorte dele.*
///
/// **Mutação que deve sangrar:** pôr `"component"` de volta em qualquer um dos seis fallbacks.
#[test]
fn the_fallback_name_of_an_unnamed_recipe_is_not_an_old_word() {
    let mut offenders: Vec<String> = Vec::new();
    let files = rust_files_of_the_shell();
    assert!(
        files.len() > 100,
        "a varredura da shell devolveu {} ficheiros — ela partiu-se e um censo vazio le-se como \
         aprovado",
        files.len()
    );
    for path in files {
        let rel = path
            .strip_prefix(repo_root())
            .unwrap_or(&path)
            .display()
            .to_string();
        let Ok(body) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut rest = body.as_str();
        while let Some(i) = rest.find("unwrap_or_else(|| \"") {
            let after = &rest[i + "unwrap_or_else(|| \"".len()..];
            let Some(j) = after.find('"') else { break };
            let word = &after[..j];
            if BANNED.iter().any(|w| word.contains(w)) {
                offenders.push(format!("{rel}: fallback {word:?}"));
            }
            rest = &after[j + 1..];
        }
    }
    assert!(
        offenders.is_empty(),
        "um nome de recurso mostrado ao artista voltou a usar a palavra antiga:\n  {}",
        offenders.join("\n  ")
    );
}

/// ⭐⭐ **E as TABELAS de rótulos dizem `Prefab`** — o que o artista lê nos botões.
///
/// ⛔ A metade justa: elas têm de conter a palavra nova. Um censo que só proíbe a antiga fica verde
/// sobre uma tabela vazia.
#[test]
fn the_label_tables_say_prefab() {
    let root = repo_root();
    for rel in LABEL_TABLES {
        let body = std::fs::read_to_string(root.join(rel))
            .unwrap_or_else(|_| panic!("o ficheiro {rel} mudou de sitio"));
        let labels = screen_strings(&body);
        assert!(
            labels.iter().any(|s| s.contains("Prefab")),
            "{rel} nao tem um unico rotulo com a palavra `Prefab` — a tabela perdeu a palavra da casa"
        );
        for s in labels {
            if !s.contains(' ') {
                continue;
            }
            assert!(
                !s.contains("Master") && !s.contains("Main missing"),
                "{rel} voltou a dizer `Master`/`Main` num rotulo: {s:?}"
            );
        }
    }
}
