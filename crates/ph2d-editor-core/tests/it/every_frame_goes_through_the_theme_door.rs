//! ⭐⭐⭐ **Toda MOLDURA de controlo passa pela porta do TEMA — e a dívida que ainda não passa tem
//! nome, e só encolhe.**
//!
//! Decisão do Enio (2026-09-04): o modelo é o Godot 4.6 «Modern» — *«minimalista, plana, concisa,
//! coerente e simples»*. A pele plana é uma propriedade do TEMA, e a porta que a impõe é
//! `paint::stroke_frame` / `paint::frame_radius` (⇒ `ph2d_tokens::visuals::{frame, radius}`): no
//! clássico o pintor traça o que sempre traçou; num tema moderno traça o que a tabela diz — nada
//! em repouso.
//!
//! ⛔ **Por que um censo pelo FONTE, e não só o gate de produto ao lado:** um pintor que trace a
//! moldura com `stroke_rounded_rect` directo continua a compilar e a passar a suíte inteira — e
//! desenha um contorno num tema que prometeu não ter nenhum. É a mesma família do
//! `every_form_row_reserves_the_animation_column`: *uma promessa minha de «faço os restantes a
//! seguir» não sobrevive a uma janela de contexto; isto sobrevive.*
//!
//! # A régua
//!
//! Toda CHAMADA a `stroke_rounded_rect(` num `.rs` de produto tem de **perguntar ao tema** — nos
//! argumentos ou na guarda — ou **declarar-se** com `FRAME-RAW-OK: <motivo>` na própria linha.
//!
//! ⭐⭐⭐ **A unidade era o FICHEIRO até 2026-09-07, e as DUAS listas deste ficheiro morreram com
//! a mudança.** Havia uma dívida (`NOT_YET`, vazia desde a wave 3) e uma tabela de isenções por
//! ficheiro — e o motivo de cada isenção vivia ali, longe da linha que ele descrevia. Três sítios
//! do repo já tinham o motivo escrito **no código**, em prosa, sem o gate saber lê-lo. *A decisão
//! vivia ao lado da linha e o portão consultava outra folha.*
//!
//! ⇒ com o marcador na chamada não há duas cópias do motivo, logo não há a metade de obsolescência
//! da lista para manter — sobra a do marcador, que é *«ele ainda fica sobre uma chamada crua?»*.

use std::fs;
use std::path::{Path, PathBuf};

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// ⭐⭐ **Os PAINÉIS e a SHELL também traçam** (wave 4, 2026-09-05): medido, 59 ficheiros em
/// `crates/ph2d-panel-*/src` chamavam `stroke_rounded_rect` directo e **nenhum** conhecia a porta —
/// e é nos painéis que o artista vive. Cada raiz vem com o prefixo da crate, para que a chave de
/// uma isenção nunca colida com a de um ficheiro do `editor-core`.
fn panel_and_shell_roots() -> Vec<(String, PathBuf)> {
    let crates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut out = Vec::new();
    for entry in fs::read_dir(&crates).expect("crates/ legivel").flatten() {
        let p = entry.path();
        let Some(name) = p.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
            continue;
        };
        if p.is_dir() && name.starts_with("ph2d-panel-") && p.join("src").is_dir() {
            out.push((format!("{name}/src"), p.join("src")));
        }
    }
    let shell = crates.join("../shells/desktop/src");
    if shell.is_dir() {
        out.push(("shells/desktop/src".to_string(), shell));
    }
    out.sort();
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("dir legivel") {
        let p = entry.expect("entrada legivel").path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// `(traça molduras?, conhece a porta?)` de um ficheiro de produto.
/// Cada chamada crua a `stroke_rounded_rect` deste ficheiro que **não pergunta ao tema nem se
/// declara**.
///
/// ⛔⛔⛔ **Até 2026-09-07 esta pergunta era por FICHEIRO, e por isso era cega em três formas.**
/// Ela lia *«este ficheiro menciona a porta?»* — logo (a) uma isenção escrita para uma função
/// cobria o ficheiro inteiro (o balão de aviso, 290 linhas abaixo do corpo do `stroke_frame`),
/// (b) um ficheiro que chamasse a porta **uma** vez ficava abençoado em todas as outras, e
/// (c) uma menção a `visuals::` em qualquer linha bastava.
///
/// ⇒ hoje a unidade é a CHAMADA, e ela passa por uma de duas portas:
///
/// - **pergunta ao tema** — nos argumentos **ou na guarda**. ⚠️ A guarda é uma forma a sério, não
///   uma tolerância: o traço de repouso de um `Button` está dentro de um
///   `if …Widgets::of(theme).inactive.bg_stroke.is_visible()`, e a pergunta não aparece em
///   argumento nenhum. *Um censo que só lê os argumentos declara cru o que o `if` já resolveu.*
/// - **declara-se** com `FRAME-RAW-OK: <motivo>` na própria chamada.
///
/// ⭐ **O marcador substituiu uma lista de ficheiros**, e a razão é que o motivo já estava escrito
/// no código em três sítios (`value_slider`, `probe`, `paint_rows`) sem o gate saber ler.
/// *A decisão vivia ao lado da linha e o portão consultava outra folha.* Com o marcador não há
/// duas cópias do motivo, logo não há a metade de obsolescência a manter.
fn raw_calls(p: &Path) -> Vec<String> {
    let src = fs::read_to_string(p).expect("ficheiro legivel");
    let body = match src.find("#[cfg(test)]") {
        Some(at) => &src[..at],
        None => &src[..],
    };
    let doors = door_bodies(body);
    let lines: Vec<&str> = body.lines().collect();
    let mut offs = Vec::with_capacity(lines.len());
    let mut acc = 0usize;
    for l in &lines {
        offs.push(acc);
        acc += l.len() + 1;
    }
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim_start();
        if !l.contains("stroke_rounded_rect(") || t.starts_with("//") || t.starts_with("use ") {
            continue;
        }
        if doors.iter().any(|(a, b)| offs[i] >= *a && offs[i] < *b) {
            continue;
        }
        // ⚠️⚠️ **A janela da guarda ignora COMENTÁRIOS, e isto foi pago por uma mutação que
        // sobreviveu.** O marcador do anel de foco explica, em prosa, que *o traço logo acima
        // pergunta ao tema* — e a explicação nomeia `Widgets::of(theme)`. Com os comentários
        // dentro da janela, esse texto fazia a régua declarar TEMADA a chamada que ele próprio
        // dizia ser crua. *A prosa que descreve o mecanismo dispara o detector do mecanismo.*
        let from = i.saturating_sub(8);
        let window: String = lines[from..=i]
            .iter()
            .map(|l| match l.find("//") {
                Some(at) => &l[..at],
                None => l,
            })
            .collect::<Vec<_>>()
            .join("\n");
        let asks_the_theme = [
            "chrome.",
            "Chrome::",
            "visuals::",
            "Widgets::",
            "stroke_w",
            ".width",
        ]
        .iter()
        .any(|k| window.contains(k));
        let declared = lines[i.saturating_sub(5)..=i]
            .iter()
            .any(|w| w.contains("FRAME-RAW-OK"));
        if !asks_the_theme && !declared {
            out.push(format!("{}:{}", p.display(), i + 1));
        }
    }
    out
}

/// Os intervalos de bytes do corpo de cada `pub fn stroke_frame` — a porta não se mede a si mesma.
fn door_bodies(src: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = src[from..].find("pub fn stroke_frame(") {
        let at = from + rel;
        let Some(open) = src[at..].find('{').map(|o| at + o) else {
            break;
        };
        let mut depth = 0i32;
        let mut end = open;
        for (i, c) in src[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        out.push((at, end));
        from = end;
    }
    out
}

/// Um marcador que já não fica sobre uma chamada crua — a metade de obsolescência do marcador.
fn stale_markers(p: &Path) -> Vec<String> {
    let src = fs::read_to_string(p).expect("ficheiro legivel");
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if !l.contains("FRAME-RAW-OK") {
            continue;
        }
        let to = (i + 6).min(lines.len());
        if !lines[i..to]
            .iter()
            .any(|w| w.contains("stroke_rounded_rect("))
        {
            out.push(format!("{}:{}", p.display(), i + 1));
        }
    }
    out
}

fn is_test_file(rel: &str) -> bool {
    rel.contains("/tests") || rel.ends_with("_tests.rs") || rel.ends_with("tests.rs")
}

/// Todo ficheiro de produto que possa traçar uma moldura.
fn product_sources() -> Vec<PathBuf> {
    let root = src_root();
    let mut files = Vec::new();
    // ⛔⛔ **A RAIZ INTEIRA** (wave 26). Até àquela wave o censo varria `widget/` e `screens/hero/`
    // mais **um ficheiro escrito à mão** (`paint.rs`) — e o vizinho dele, o `progress.rs`, que
    // pinta o outro inquilino da coluna do topo, nunca foi olhado. *Um censo que enumera
    // directórios à mão afirma sobre os que alguém se lembrou de escrever.*
    walk(&root, &mut files);
    for (_, src) in panel_and_shell_roots() {
        walk(&src, &mut files);
    }
    files.retain(|p| {
        let rel = p.to_string_lossy().replace('\\', "/");
        !is_test_file(&rel)
    });
    files.sort();
    files
}

/// ⭐⭐⭐ **CENSO: toda moldura pergunta ao tema — ou DECLARA-SE na própria linha.**
///
/// ⚠️ **A unidade é a CHAMADA** (wave 27). Enquanto era o ficheiro, três formas escapavam: uma
/// isenção escrita para uma função cobria o ficheiro; um ficheiro que chamasse a porta uma vez
/// ficava abençoado em todas as outras; e uma menção a `visuals::` em qualquer linha bastava.
///
/// **Mutação que deve sangrar:** apagar o `FRAME-RAW-OK` de qualquer contorno-mensagem, ou trocar
/// o `stroke_frame` de `widget/card.rs` de volta por `stroke_rounded_rect`.
#[test]
fn every_frame_goes_through_the_theme_door() {
    let stray: Vec<String> = product_sources()
        .iter()
        .flat_map(|p| raw_calls(p))
        .collect();
    assert!(
        stray.is_empty(),
        "estas CHAMADAS tracam uma moldura sem perguntar ao tema e sem se declarar — num tema \
         moderno elas desenham o contorno que a pele plana apagou:\n  {}\n\ncura: \
         `crate::paint::stroke_frame(scene, rect, radius, theme, feel, w, colour)`; se o contorno \
         E' a mensagem (um marquee, um «largue aqui», um anel de foco, o halo de uma amostra de \
         cor), declare-o com `// FRAME-RAW-OK: <motivo>` na chamada.",
        stray.join("\n  ")
    );
}

/// ⛔⛔ **O PISO CONTRA O VÁCUO** — um censo que varre zero ficheiros também não acha nada.
///
/// As duas asserções acima são «a lista está vazia», e uma varredura partida satisfá-las por
/// engano.
///
/// ⚠️⚠️ **Os pisos foram CALIBRADOS contra a varredura partida, não escolhidos** — e a 1.ª
/// redacção deles foi morta por uma mutação: com `> 300` ficheiros e `> 15` marcadores, apagar
/// as raízes dos painéis **sobrevivia**, porque o `editor-core` sozinho já os satisfazia.
/// Medido em 2026-09-07:
///
/// | população | ficheiros | marcadores |
/// |---|---|---|
/// | só o `editor-core` (a varredura partida) | 388 | 17 |
/// | com os painéis e a shell (a certa) | 1982 | 27 |
///
/// ⇒ os pisos ficam **entre** as duas colunas. *Um piso contra o vácuo calibrado sem o mundo
/// partido ao lado mede a população que sobrou, não a que devia estar lá.*
#[test]
fn the_census_actually_reaches_the_tree() {
    let files = product_sources();
    assert!(
        files.len() > 1000,
        "o censo varreu {} ficheiros de produto: a varredura partiu-se, e as outras duas \
         asserticoes ficam VERDES por vacuo",
        files.len()
    );
    let declared: usize = files
        .iter()
        .map(|p| {
            fs::read_to_string(p)
                .map(|s| s.matches("FRAME-RAW-OK").count())
                .unwrap_or(0)
        })
        .sum();
    assert!(
        declared > 20,
        "so' {declared} contornos se declaram como a MENSAGEM: ou alguem os apagou em massa, ou o \
         leitor de marcadores deixou de os ver"
    );
}

/// ⛔ **A metade que impede o marcador de virar licença.**
///
/// Um `FRAME-RAW-OK` que já não fica sobre uma chamada crua descreve o nada — e um marcador órfão
/// é pior que uma lista velha, porque parece uma decisão viva. ⭐ Note que esta é a **única**
/// metade de obsolescência que sobra: quando o motivo vivia numa lista de ficheiros havia duas
/// cópias dele para manter alinhadas; com o marcador há uma.
#[test]
fn no_stale_frame_markers() {
    let stale: Vec<String> = product_sources()
        .iter()
        .flat_map(|p| stale_markers(p))
        .collect();
    assert!(
        stale.is_empty(),
        "estes `FRAME-RAW-OK` ja' nao ficam sobre uma chamada crua — apague-os:\n  {}",
        stale.join("\n  ")
    );
}
