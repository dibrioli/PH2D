//! ⭐⭐⭐ **O VÃO entre um ÍCONE e o rótulo dele — decisão do dono, e é UM número.**
//!
//! Censado em 2026-09-07: **cinco** respostas para *«quanto dista um ícone da legenda dele?»* —
//! `12` à mão no balão de aviso · `8` na lista, na árvore, no combobox e na barra do topo · `6` no
//! menu de contexto e no cabeçalho de secção · `4` na Hierarquia, nas camadas do Painter e na
//! pilha do Vector.
//!
//! ⭐ **O `4` era veredito ESCRITO do dono** — *«nome mais próximos dos ícones»*, 2026-05-24, com
//! data e citação no código. Perguntado se valia só para aquele painel ou para o app inteiro, ele
//! respondeu **«para o app todo»** (2026-09-07). ⇒ [`ph2d_tokens::icon_label_gap_px`].
//!
//! ⛔⛔ **A divergência do modelo é deliberada:** o Godot Modern parte a pergunta por classe
//! (`Tree.icon_h_separation` = 6 · `Button` = 4 · `CheckBox` = 8). Aqui é **4** para todos, porque
//! um veredito de produto medido no ecrã ganha de um número portado. *Um gate que não registasse
//! isto faria a próxima pessoa "corrigir" a casa de volta para o modelo.*
//!
//! # O que este vão NÃO é — e cada um tem lei própria
//!
//! - **seta → ícone**: a Hierarquia dá-lhe `Xxs` = 2, por outro veredito do mesmo dia;
//! - **ícone → ícone** numa fileira de botões: é a lei do GRUPO (wave 20), e as peças encostam;
//! - **ícone → chip**: dois widgets, não um widget e a legenda dele.
//!
//! ⇒ os três estão na lista de isenções, com o mecanismo, porque a régua textual não os distingue.

use std::fs;
use std::path::{Path, PathBuf};

/// As superfícies que pintam um ícone seguido do rótulo DELE.
///
/// ⚠️ **Metade de OBSOLESCÊNCIA**: cada uma tem de continuar a chamar a porta. Se uma deixar de
/// pintar o par ícone+rótulo, tire-a daqui — senão o censo mede uma população que já não existe.
const PAINTS_AN_ICON_AND_ITS_LABEL: &[&str] = &[
    "crates/ph2d-editor-core/src/paint.rs",
    "crates/ph2d-editor-core/src/widget/combobox.rs",
    "crates/ph2d-editor-core/src/widget/list_item.rs",
    "crates/ph2d-editor-core/src/widget/section_header/mod.rs",
    "crates/ph2d-editor-core/src/widget/tree_view.rs",
    "crates/ph2d-editor-core/src/screens/hero/context_menu_overlay.rs",
    "crates/ph2d-editor-core/src/screens/hero/topbar/cluster_painter.rs",
    "crates/ph2d-panel-hierarchy/src/row.rs",
    "crates/ph2d-panel-vector/src/paint_stack_rows.rs",
];

/// `(caminho, porquê o vão ali NÃO é «um ícone e a legenda dele»)`.
const NOT_THIS_QUESTION: &[(&str, &str)] = &[
    (
        "crates/ph2d-panel-painter-layers/src/paint.rs",
        "ICONE -> ICONE: a fileira de botoes do cabecalho. O vao entre duas PECAS de um controlo \
         e' a lei do grupo (wave 20), nao a legenda de um icone",
    ),
    (
        "crates/ph2d-editor-core/src/screens/hero/chrome/input_map.rs",
        "COLUNA RESERVADA: o `icon_w` ali e' a largura que a linha guarda para o botao de apagar \
         na ponta direita, e a soma que a regua ve^ e' OUTRO argumento da mesma chamada",
    ),
    (
        "crates/ph2d-panel-vector/src/paint_effects.rs",
        "ICONE -> ICONE: uma fileira de encaixes de icone a` direita da linha; o passo entre dois \
         encaixes e' a lei do grupo, nao a legenda de nenhum deles",
    ),
    (
        "crates/ph2d-panel-vector/src/paint_filters.rs",
        "ICONE -> ICONE: a mesma fileira de encaixes, no painel irma~o",
    ),
    (
        "crates/ph2d-editor-core/src/widget/showcase/actions.rs",
        "ICONE -> CHIP: a bancada po~e um botao ao lado de uma etiqueta; sa~o dois widgets, e nao \
         um widget com a legenda dele",
    ),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) != Some("target") {
                walk(&p, out);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn ui_sources() -> Vec<(String, String)> {
    let root = repo_root();
    let mut files = Vec::new();
    walk(&root.join("crates/ph2d-editor-core/src"), &mut files);
    if let Ok(entries) = fs::read_dir(root.join("crates")) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with("ph2d-panel-") {
                walk(&p.join("src"), &mut files);
            }
        }
    }
    walk(&root.join("shells/desktop/src"), &mut files);
    files.sort();
    let mut out = Vec::new();
    for p in files {
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        // ⚠️ Os testes internos ficam de fora: o gate que mede cópias contém, por construção, a
        // composição que ele afirma — e a 1.ª corrida acusou-se a si próprio.
        if rel.contains("/tests/") {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&p) {
            let body = match raw.find("#[cfg(test)]") {
                Some(at) => raw[..at].to_string(),
                None => raw,
            };
            out.push((rel, body));
        }
    }
    out
}

/// As INSTRUÇÕES de um fonte — comentários fora, continuações juntas.
///
/// ⚠️⚠️ **A 1.ª redacção lia LINHAS, e o `cargo fmt` derrotou-a no mesmo dia:** ele partiu
/// `host.x + field_pad_x() + inline_icon_size(host) + icon_label_gap_px()` em quatro linhas, e
/// nenhuma delas tinha ao mesmo tempo o nome do ícone e o vão. *Um censo que parseia o fonte tem
/// de saber todas as formas do que lê* — 8.ª ocorrência nesta linha, e a primeira em que o
/// adversário é o formatador do próprio repo.
fn statements(src: &str) -> Vec<String> {
    let mut flat = String::with_capacity(src.len());
    for line in src.lines() {
        let code = match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        };
        flat.push_str(code);
        flat.push(' ');
    }
    // ⚠️ **O corte é a profundidade ZERO de parênteses.** Partir só por `;` cola os BRAÇOS de um
    // `match` numa instrução só — foi assim que o planeador da barra do Flip apareceu como falso
    // positivo, com o `ICON_W` de um braço e o `Spacing::Xs` de outro. Uma vírgula de argumento
    // vive DENTRO de parênteses e não corta, que é o que preserva `paint(x + icon_w + gap, …)`.
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    for c in flat.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            _ => {}
        }
        // Chaveta e `;` cortam SEMPRE; a vírgula só a profundidade zero de parênteses — é isso
        // que separa os braços de um `match` sem partir `paint(x + icon_w + vão, …)`.
        let cuts = matches!(c, ';' | '{' | '}') || (depth <= 0 && c == ',');
        if cuts {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out
}

/// Esta instrução avança de um ÍCONE somando um vão?
///
/// ⚠️ A forma que se lê é `<algo com "icon"> + <um vão>` na mesma expressão. A régua não distingue
/// *legenda* de *outro widget* — é por isso que existe a lista [`NOT_THIS_QUESTION`], **com o
/// mecanismo de cada uma**: uma isenção sem motivo escrito é a porta pela qual a 6.ª resposta
/// volta.
fn advances_past_an_icon(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let names_an_icon = ["icon_w", "icon_size", "icon_px", "icon_rect", "glyph_x"]
        .iter()
        .any(|n| lower.contains(n));
    if !names_an_icon {
        return false;
    }
    // …e soma um VÃO. ⚠️ **As três formas contam, e a terceira é a CURADA** — sem ela a metade de
    // obsolescência acusa exactamente os ficheiros que passaram pela porta, que foi o que a 1.ª
    // corrida fez. *Uma régua de «isto ainda existe?» que não reconhece a forma curada declara
    // obsoleto tudo o que se arrumou.*
    line.contains("+ Spacing::") || line.contains("icon_label_gap_px") || adds_a_bare_number(line)
}

/// `+ 12.0` e afins — o literal cru, que é a forma com que a 5.ª resposta nasceu.
fn adds_a_bare_number(line: &str) -> bool {
    let b = line.as_bytes();
    for i in 0..b.len().saturating_sub(2) {
        if b[i] == b'+' && b[i + 1] == b' ' && b[i + 2].is_ascii_digit() {
            return true;
        }
    }
    false
}

/// ⭐⭐⭐ **Toda superfície que põe um rótulo depois de um ícone lê o vão da porta.**
#[test]
fn every_label_after_an_icon_takes_its_gap_from_the_door() {
    // ⚠️ **Censo de MUNDO ABERTO.** A 1.ª redacção só olhava a lista declarada — e uma superfície
    // NOVA, que é precisamente como uma 6.ª resposta nasce, passava sem ser vista. *Um censo que
    // só mede a própria lista não impede nada.*
    let exempt: Vec<&str> = NOT_THIS_QUESTION.iter().map(|(f, _)| *f).collect();
    let mut strays = Vec::new();
    for (rel, src) in ui_sources() {
        if exempt.contains(&rel.as_str()) || src.contains("icon_label_gap_px") {
            continue;
        }
        if statements(&src).iter().any(|st| advances_past_an_icon(st)) {
            strays.push(rel);
        }
    }
    assert!(
        strays.is_empty(),
        "estas superficies pintam um icone e a legenda dele sem passar por \
         `ph2d_tokens::icon_label_gap_px()`:\n  {}",
        strays.join("\n  ")
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** — cada declarada tem de continuar a avançar depois de um ícone.
#[test]
fn every_declared_surface_still_paints_an_icon_before_its_label() {
    let sources = ui_sources();
    for declared in PAINTS_AN_ICON_AND_ITS_LABEL {
        let found = sources
            .iter()
            .find(|(rel, _)| rel == declared)
            .unwrap_or_else(|| panic!("`{declared}` ja' nao existe: a lista esta' velha"));
        assert!(
            statements(&found.1)
                .iter()
                .any(|st| advances_past_an_icon(st)),
            "`{declared}` ja' nao avanca depois de um icone — tire-o da lista"
        );
    }
}

/// ⚠️ E as ISENÇÕES também envelhecem: cada uma tem de continuar a existir e a avançar num ícone.
#[test]
fn every_exemption_still_describes_a_real_surface() {
    let sources = ui_sources();
    for (rel, why) in NOT_THIS_QUESTION {
        let found = sources
            .iter()
            .find(|(r, _)| r == rel)
            .unwrap_or_else(|| panic!("a isencao `{rel}` ({why}) aponta um ficheiro que sumiu"));
        assert!(
            statements(&found.1)
                .iter()
                .any(|st| advances_past_an_icon(st)),
            "a isencao `{rel}` ja' nao descreve nada: o ficheiro nao avanca depois de um icone"
        );
    }
}

/// ⛔ **O valor é o do dono, não o do modelo** — e a divergência é escrita, não deriva.
#[test]
fn the_gap_is_the_owners_four_and_not_the_models_six() {
    assert_eq!(
        ph2d_tokens::icon_label_gap_px(),
        4.0,
        "o vao icone->rotulo saiu dos 4 px que o dono decidiu em 2026-09-07 («para o app todo»), \
         que sao o mesmo numero do veredito dele de 2026-05-24 na Hierarquia"
    );
    assert_ne!(
        ph2d_tokens::icon_label_gap_px(),
        6.0,
        "alguem «corrigiu» o vao de volta para o `Tree.icon_h_separation` do Godot: a divergencia \
         e' DELIBERADA e tem dono e data"
    );
}
