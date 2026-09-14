//! ⭐⭐⭐ **O VÃO entre duas linhas empilhadas é UMA resposta — e ela tem NOME.**
//!
//! Enio, 2026-09-06, com as fotos do Blender e do Godot ao lado da nossa:
//! *«Blender e Godot com aspecto muito mais compacto e profissional. Espaçamento muito regrado e
//! universal.»*
//!
//! ⚠️ **A palavra que nomeia o mecanismo é «universal», não «compacto».** Medido no fonte do
//! Godot (MIT, `editor/themes/theme_modern.cpp` — a triagem desta linha autoriza ler E portar):
//! **nenhuma** constante de espaço daquele tema é escolhida no sítio onde se pinta. Todas são
//! `base_margin · k` a partir de um `base_spacing = 4`, e a que responde a ESTA pergunta tem
//! nome próprio — `separation_margin` — lido por `BoxContainer`, `HBoxContainer`,
//! `VBoxContainer`, `GridContainer`, `FlowContainer` e `FoldableContainer`. *É o nome que
//! impede a segunda resposta.*
//!
//! ⛔ **E nós tínhamos SETE respostas para a mesma pergunta** (censo de 2026-09-06):
//!
//! | onde | valor | alcance |
//! |---|---|---|
//! | `ROW_H_PX + Spacing::Xs` | 4 px | 21 sítios |
//! | `ROW_H_PX + Spacing::Sm` | **6 px** | 20 sítios — o Inspector e o Painter Layers inteiros |
//! | `ROW_H_PX + Spacing::Xxs` | 2 px | 3 sítios |
//! | um local `gap` / `row_gap` | 4 px | 52 sítios |
//! | `ph2d_panel_grid_snap::layout::row_gap()` | **6 px** | escondida atrás de uma FUNÇÃO |
//! | `showcase::row_gap()` | **6 px** | 18 chamadas — a maquinaria de que o Inspector é feito |
//! | `asset_browser::paint::gap()` | **6 px** | 13 chamadas |
//!
//! ⚠️⚠️ **As três últimas são a lição deste censo: uma cópia atrás de uma FUNÇÃO não aparece na
//! varredura que procura o operador.** A primeira leitura desta wave contou quatro respostas
//! porque procurou `ROW_H + <espaço>`; as outras três só apareceram ao perguntar *«que função
//! desta árvore devolve um degrau da escada e chama-se vão?»*. Um censo que procura a FORMA de
//! uma expressão é cego a quem lhe deu nome.
//!
//! ⚠️ **A escada NÃO era o defeito, e era o suspeito óbvio:** a nossa (`2·4·6·8·12·16·24·32·48`)
//! é `base·k` com `base = 4` em todos os degraus — o mesmo vocabulário do Godot, que usa
//! `base·0.75`, `·1.5`, `·1.75`, `·2`, `·2.5`, `·3`. *O defeito nunca foi que rungs existem; era
//! que a escolha era feita no sítio da pintura.*
//!
//! ⛔ **O que este censo NÃO proíbe, e porquê:** `+ Spacing::Md` / `+ Spacing::Lg` depois de uma
//! altura de linha é o fim de um GRUPO, que é outra pergunta — o Godot também lhe dá outro
//! número (`Separator separation = base_margin · 2` = 8). Ela fica **nomeada e por unificar**
//! ⇒ **FECHOU na wave 19** (`ph2d_tokens::control_gap_px`, 78 sítios). ⚠️ E o `chrome.section-gap`
//! (14) que esta nota dizia não servir foi RENOMEADO para `chrome.inline-icon`, porque os
//! seus **quatro** consumidores usam-no como TAMANHO DE ÍCONE, não como vão de secção.

use std::fs;
use std::path::{Path, PathBuf};

/// Os degraus que respondem *«quanto avança de uma linha para a seguinte»*. `Md`/`Lg` ficam de
/// fora de propósito: são o fim de um grupo (ver o doc do módulo).
const ROW_RUNGS: &[&str] = &["Xxs", "Xs", "Sm"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) == Some("target") {
                continue;
            }
            walk(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// As raízes de UI: `ph2d-editor-core`, as crates de painel e a shell.
fn ui_sources() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    walk(&root.join("crates/ph2d-editor-core/src"), &mut out);
    if let Ok(entries) = fs::read_dir(root.join("crates")) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with("ph2d-panel-") {
                walk(&p.join("src"), &mut out);
            }
        }
    }
    walk(&root.join("shells/desktop/src"), &mut out);
    out.sort();
    out
}

/// O caractere antes de `ROW_H` faz dele outra constante (`VAR_ROW_H`, `HIER_ROW_H`)?
fn is_the_form_row(before: Option<char>) -> bool {
    !matches!(before, Some(c) if c.is_alphanumeric() || c == '_')
}

/// Toda ocorrência de `<altura de linha de formulário> + Spacing::<degrau de linha>`.
fn hand_written_pitches() -> Vec<String> {
    let root = repo_root();
    let mut out = Vec::new();
    for p in ui_sources() {
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .into_owned();
        for (n, line) in src.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for head in ["ROW_H_PX + Spacing::", "ROW_H + Spacing::"] {
                let mut from = 0;
                while let Some(hit) = line[from..].find(head) {
                    let at = from + hit;
                    let before = line[..at].chars().next_back();
                    let tail = &line[at + head.len()..];
                    if is_the_form_row(before) && ROW_RUNGS.iter().any(|r| tail.starts_with(r)) {
                        out.push(format!("{rel}:{}: {}", n + 1, line.trim()));
                    }
                    from = at + head.len();
                }
            }
        }
    }
    out
}

/// ⭐ **Ninguém escreve o passo de uma linha à mão — ele sai da porta.**
#[test]
fn the_row_pitch_is_never_written_at_the_painting_site() {
    let found = hand_written_pitches();
    assert!(
        found.is_empty(),
        "{} sítio(s) escrevem o passo de uma linha em vez de chamar \
         `ph2d_tokens::row_pitch_px()` (ou `control_gap_px()`, se a altura for medida em tempo de \
         pintura). Cada um é a segunda resposta a uma pergunta que já tem uma:\n  {}",
        found.len(),
        found.join("\n  ")
    );
}

/// ⭐⭐ **A metade que apanha a cópia ESCONDIDA — a que a varredura por operador não vê.**
///
/// Três das sete cópias viviam atrás de uma função de crate. Um vão que se chama vão devolve a
/// porta, nunca um degrau da escada. ⚠️ `segmented_gap` e `popover_gap` respondem a OUTRAS
/// perguntas (o vão dentro de um controlo segmentado; a distância de um popover ao seu dono) e
/// não entram — a régua é o nome dizer **linha**.
#[test]
fn a_function_named_row_gap_delegates_instead_of_choosing_a_rung() {
    let root = repo_root();
    let mut offenders = Vec::new();
    for p in ui_sources() {
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .into_owned();
        let lines: Vec<&str> = src.lines().collect();
        for (n, line) in lines.iter().enumerate() {
            let t = line.trim_start();
            let names_a_row_gap = t.contains("fn row_gap(") || t.contains("fn gap(");
            if !names_a_row_gap || !t.contains("-> f32") {
                continue;
            }
            // O corpo é a linha seguinte (as três cópias achadas tinham esta forma), ou o resto
            // desta, quando escrita numa linha só.
            let body = lines.get(n + 1).copied().unwrap_or_default();
            if body.contains("Spacing::") || line.contains("Spacing::") {
                offenders.push(format!("{rel}:{}: {}", n + 1, t));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "{} função(ões) chamadas «vão de linha» escolhem um degrau em vez de delegar em \
         `ph2d_tokens::control_gap_px()`:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}

/// ⭐⭐⭐ **A TERCEIRA metade: o passo vertical de uma pilha pergunta a porta — seja qual for a
/// ALTURA que ele soma.**
///
/// ⛔ **Censo de 2026-09-14: 53 sítios respondiam a esta pergunta, com TRÊS respostas** —
/// `Sm` (6) em 26 · `Xs` (4) em 25 · `Md` (8) em 2. **27 deles no Inspector**, que é o painel que
/// o dono tem aberto o dia inteiro, e a lei diz **3** (`control_gap_px`) desde 2026-09-07:
/// *«entre grupos de botões temos um espaçamento, entre sliders outro espaçamento. Para ambos
/// vamos colocar o padrão de espaçamento de 3 px»*.
///
/// # ⚠️ Porque as outras réguas desta casa não os viam — são três cegueiras diferentes
///
/// | régua | o que ela procura | porque falhou |
/// |---|---|---|
/// | [`the_row_pitch_is_never_written_at_the_painting_site`] | `ROW_H_PX + Spacing::` | exige que a altura seja a da LINHA; estes somam `font`, `cb_h`, `BTN_H`, `CHECK_H`, `BAR_H`, `STRIP_H`… |
/// | `the_tail_of_a_block_is_one_answer` | `y + … + Spacing::` **sem `;`** | estes são INSTRUÇÕES (`y += …;`), não a cauda que sai da função |
/// | `every_stack_of_rows_asks_the_rhythm` | um FICHEIRO que empilha chama alguma porta | é por **ficheiro**: quem chama a porta uma vez sai do censo com dez sítios ainda a escrever o degrau à mão |
///
/// ⇒ *três censos sobre a mesma grandeza, e a interseção deles tinha 53 sítios lá dentro.* É a
/// sexta vez que esta jornada paga a forma **um censo que conhece uma forma da pergunta é cego às
/// outras** — e a cura é sempre a mesma: a régua passa a ser a PERGUNTA (*este cursor vertical
/// avança um vão?*), nunca a sintaxe de um caso.
///
/// # ⭐ A resposta não foi escolhida: ela já estava escrita na secção mais NOVA
///
/// O [`sections/actions.rs`] do Inspector (a *Signal Actions*, 2026-09-10) escreve
/// `cur_y += font + ph2d_tokens::control_gap_px();` em três sítios. ⇒ a pergunta *«o que fica
/// depois de uma linha de texto numa secção?»* já tinha dono nesta casa; os 53 são os sítios
/// escritos **antes** de a porta existir. *Quando o código novo e o velho discordam, o novo é a
/// lei e o velho é a dívida — e o censo é o que os torna comparáveis.*
///
/// ⚠️ **A asserção não é «chama o `control_gap_px`»: é «chama UMA das três».** Qual delas é a
/// resposta do sítio — uma lista pede o `list_row_gap_px`, uma fronteira de cartão pede o
/// `section_gap_px` —, e é isso que impede este gate de empurrar o número errado para uma
/// superfície que responde a outra pergunta.
///
/// ⛔ **Um `y += Spacing::Md.px();` SOZINHO não é acusado**, e a ausência é a decisão: ali não há
/// altura nenhuma a somar, logo ele é um RECUO (o ar dentro de uma caixa), que é outra grandeza e
/// ainda não tem porta. *Alargar esta régua até lá fabricaria dívida sobre uma pergunta que
/// ninguém fez.*
///
/// (Mutação: repor um `+ Spacing::Sm.px()` em qualquer um dos 53 ⇒ RED, nomeando ficheiro e linha.)
#[test]
fn the_vertical_step_of_a_stack_asks_the_door() {
    const RUNGS_DE_VAO: &[&str] = &["Xxs", "Xs", "Sm", "Md", "Lg"];
    const CURSORES: &[&str] = &["y", "cur_y", "yy", "new_y"];
    let root = repo_root();
    let mut offenders = Vec::new();
    for p in ui_sources() {
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        for (n, line) in src.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || !t.ends_with(';') {
                continue;
            }
            let Some((cursor, resto)) = t.split_once("+=") else {
                continue;
            };
            if !CURSORES.contains(&cursor.trim()) {
                continue;
            }
            // ⚠️ **A altura tem de existir**: `y += Spacing::X.px();` sozinho é um RECUO.
            let Some((altura, degrau)) = resto.rsplit_once("+ Spacing::") else {
                continue;
            };
            if altura.trim().is_empty() || !RUNGS_DE_VAO.iter().any(|r| degrau.starts_with(r)) {
                continue;
            }
            // ⛔ **Um degrau MULTIPLICADO não é «qual degrau é o vão» — é uma geometria composta.**
            // O 1.º falso positivo desta régua foi o separador do menu de contexto:
            // `y += 1.0 + Spacing::Xs.px() * 2.0;` é *um fio de 1 px com recuo em cima e em baixo*,
            // ou seja a ALTURA do separador, e não o vão que vem depois de alguma coisa. Acusá-lo
            // mandaria trocar uma composição correcta por um número que responde a outra pergunta.
            if degrau
                .split_once(".px()")
                .is_some_and(|(_, resto)| resto.trim_start().starts_with('*'))
            {
                continue;
            }
            offenders.push(format!("{rel}:{}: {t}", n + 1));
        }
    }
    assert!(
        offenders.is_empty(),
        "{} sitio(s) somam um degrau da escada ao avancar o cursor vertical, em vez de perguntar \
         a porta da grandeza. Cada um e' uma segunda resposta a uma pergunta que ja' tem uma:\n  \
         {}\n\nQual das tres o sitio pede e' a resposta DELE: `ph2d_tokens::control_gap_px()` (3) \
         entre dois controlos de uma seccao · `list_row_gap_px()` (1) entre duas linhas de uma \
         LISTA · `section_gap_px()` (8) de um cartao de seccao para o seguinte.",
        offenders.len(),
        offenders.join("\n  ")
    );
}
