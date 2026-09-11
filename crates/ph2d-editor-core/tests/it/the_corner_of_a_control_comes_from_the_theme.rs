//! ⭐⭐⭐ **A QUINA de um controlo vem da porta do TEMA — e o que fica de fora é CANVAS ou FORMA.**
//!
//! A wave 2/3 levou a porta da **moldura** (`stroke_frame`) a 44 pintores. O **raio** foi atrás
//! dela só onde o mesmo sítio já a chamava — e ficou fora de **65** sítios, incluindo os
//! **primitivos** que todo painel usa.
//!
//! ⛔⛔ **O caso que nomeia a wave:** `Button::radius()` devolvia `Radius::Md` (**4**) sem
//! perguntar nada, enquanto um chip segmentado ao lado pintava **3** (o `corner_radius` do Godot
//! Modern, wave 8). *Dois widgets encostados na mesma fileira, dois raios* — e a causa não é um
//! esquecimento pontual: **um primitivo que escolhe sozinho não é uma excepção, é a resposta que
//! ~100 sítios herdam sem saber.**
//!
//! # A partição, e ela tem TRÊS classes e não duas
//!
//! - **a quina de um CONTROLO** → pela porta. É o corpo desta wave;
//! - **uma FORMA deliberada** (`Radius::Full` numa pílula, num avatar redondo, no polegar de um
//!   interruptor) → ⛔ **fora**, e o mecanismo é que a porta *achata* (`visuals::radius` devolve
//!   `MODERN_CORNER_RADIUS_PX` para qualquer `classic > 0`): passar uma pílula por ela transforma
//!   um estádio num rectângulo. *Um raio que É a forma não é a mesma pergunta que um raio que é a
//!   quina;*
//! - **o CANVAS** → fora: a régua de uma timeline, a célula de uma tira de filme, o marquee de
//!   selecção, a camada de áudio sobre o desenho. Ali o raio é geometria do documento, e um tema
//!   não o governa.
//!
//! # ⚠️ A régua conta PARÊNTESES, e a 1.ª redacção não contava
//!
//! Duas chamadas já **estavam** dentro da porta — em `frame_radius(\n  theme,\n  Radius::…)`, uma
//! chamada de três linhas — e uma varredura por LINHA acusou-as. *Um censo que parseia o fonte tem
//! de saber todas as formas do que lê*, e a forma multi-linha já mordeu esta linha seis vezes.

use std::fs;
use std::path::{Path, PathBuf};

/// `(caminho relativo, porquê o raio NÃO passa pela porta)`.
const OUTSIDE_THE_DOOR: &[(&str, &str)] = &[
    // ── FORMA: a porta achata, e aqui o raio É a forma ────────────────────────────────────────
    (
        "crates/ph2d-editor-core/src/widget/avatar.rs",
        "FORMA: `Radius::Full` num avatar CIRCULAR — a porta achatava o circulo num rectangulo",
    ),
    (
        "crates/ph2d-editor-core/src/widget/progress_bar.rs",
        "FORMA: a barra e' uma PILULA (`Full`); achatada ela deixa de se distinguir de um campo",
    ),
    (
        "crates/ph2d-editor-core/src/widget/icon_button.rs",
        "FORMA: o `Xl` e' o CHIP redondo da TopBar, e ele ja' entra na porta como o fallback \
         classico dela — o `Radius::Xl` que sobra e' o argumento, nao a resposta",
    ),
    // ── MECANISMO ─────────────────────────────────────────────────────────────────────────────
    (
        "crates/ph2d-editor-core/src/widget/toggle_classic.rs",
        "pintor SO' do classico (`PH2D_UI_NEW=0`): ele nunca corre num tema moderno",
    ),
    (
        "crates/ph2d-editor-core/src/widget/skin.rs",
        "a PELE de um DOCUMENTO (vetor autorado): o raio e a moldura sao do artista, nao do tema",
    ),
    (
        "crates/ph2d-editor-core/src/widget/showcase/notes.rs",
        "um POST-IT nao e' cromo — cor de marcador fixa e nenhum tema em maos; achata-lo seria \
         achatar a unica superficie do app que e' de proposito um objeto",
    ),
    (
        "crates/ph2d-panel-inspector/src/paint_frame.rs",
        "o contorno de MARCADOR de uma seccao: mesma familia do post-it, mesma cor fixa, e este \
         pintor tambem nao recebe tema",
    ),
    (
        "crates/ph2d-editor-core/src/widget/showcase/status.rs",
        "⚠️ NAO e' um raio: `Radius::Xl2` e' usado como o TAMANHO de um spinner (20 px). E' a \
         mesma especie do `chrome.section-gap` que a wave 19 renomeou, e a cura e' um token de \
         tamanho — wave propria, nomeada aqui para nao se perder",
    ),
    // ── CANVAS: geometria do documento, que um tema nao governa ────────────────────────────────
    (
        "crates/ph2d-editor-core/src/screens/hero/selection.rs",
        "CANVAS: o marquee de seleccao — o contorno E' o significado",
    ),
    (
        "crates/ph2d-editor-core/src/screens/hero/canvas.rs",
        "CANVAS: a moldura de «largue aqui» durante um arrasto",
    ),
    (
        "crates/ph2d-panel-flip-frames/src/paint_cells.rs",
        "CANVAS: as celulas de uma TIRA DE FILME",
    ),
    (
        "crates/ph2d-panel-timeline/src/ruler.rs",
        "CANVAS: a regua de uma timeline",
    ),
    (
        "crates/ph2d-panel-timeline/src/ruler_veil.rs",
        "CANVAS: o veu da regua",
    ),
    (
        "crates/ph2d-panel-timeline/src/strip_paint.rs",
        "CANVAS: as tiras de clip",
    ),
    (
        "crates/ph2d-panel-timeline/src/stack_lane_paint.rs",
        "CANVAS: as faixas empilhadas",
    ),
    (
        "crates/ph2d-panel-timeline/src/summary_paint.rs",
        "CANVAS: o resumo da timeline",
    ),
    (
        "crates/ph2d-panel-timeline/src/scrollbar.rs",
        "CANVAS: a barra de rolagem DA timeline, desenhada sobre ela",
    ),
    (
        "crates/ph2d-panel-timeline/src/container_list.rs",
        "CANVAS: a lista de contentores da timeline",
    ),
    (
        "crates/ph2d-panel-timeline/src/marker_rename.rs",
        "CANVAS: a caixa de renomear um marcador, sobre a regua",
    ),
    (
        "shells/desktop/src/render_loop/audio_overlay.rs",
        "CANVAS: a camada de audio pinta-se SOBRE o desenho, fora de todo painel",
    ),
    (
        "crates/ph2d-app-field3d/src/gizmo_paint.rs",
        "CANVAS: o gizmo 3D",
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
            // ⭐⭐ **E as crates de FAMÍLIA, desde a W2** (11/09). A metade da UI de um módulo que
            // vivia em `shells/desktop/src` está a mudar-se para `crates/ph2d-app-<fam>`, uma
            // família de cada vez — e uma varredura que não as siga perde a população **em
            // silêncio**: a isenção fica obsoleta, o censo de obsolescência acusa-a, e a leitura
            // fácil é apagar a isenção. *Isso não cura nada: apaga a cobertura junto com a linha.*
            //
            // ⚠️ A `ph2d-viewport3d` está aqui pela mesma razão — ela é a moldura 3D partilhada
            // pelos dois módulos 3D, e é código de UI como qualquer painel.
            if name.starts_with("ph2d-app-") || name == "ph2d-viewport3d" {
                walk(&p.join("src"), &mut out);
            }
        }
    }
    walk(&root.join("shells/desktop/src"), &mut out);
    out.sort();
    out
}

/// Está a posição `at` **dentro** dos argumentos de um `frame_radius(`?
///
/// ⚠️ **Conta parênteses para trás**, e é isso que a separa de uma varredura por linha: a chamada
/// pode ocupar três linhas, e duas do repo ocupam.
fn inside_the_door(src: &str, at: usize) -> bool {
    let mut depth = 0i32;
    let bytes = src.as_bytes();
    let mut i = at;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b')' => depth += 1,
            b'(' => {
                if depth == 0 {
                    return src[..i].trim_end().ends_with("frame_radius");
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    false
}

/// Corta o corpo `#[cfg(test)]` — um teste que afirma o valor de fábrica não é um pintor.
fn without_tests(src: &str) -> String {
    match src.find("#[cfg(test)]") {
        Some(at) => src[..at].to_string(),
        None => src.to_string(),
    }
}

fn strays() -> Vec<String> {
    let root = repo_root();
    let exempt: Vec<&str> = OUTSIDE_THE_DOOR.iter().map(|(f, _)| *f).collect();
    let mut out = Vec::new();
    for p in ui_sources() {
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        if exempt.contains(&rel.as_str()) {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&p) else {
            continue;
        };
        let src = without_tests(&raw);
        for (n, line) in src.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            let Some(col) = line.find("Radius::") else {
                continue;
            };
            if !line[col..].contains(".px()") {
                continue;
            }
            let abs = src.lines().take(n).map(|l| l.len() + 1).sum::<usize>() + col;
            if !inside_the_door(&src, abs) {
                out.push(format!("{rel}:{}: {}", n + 1, line.trim()));
            }
        }
    }
    out
}

/// ⭐ **Ninguém escolhe a quina de um controlo — ela sai da porta do tema.**
///
/// **Mutação que deve sangrar:** repor `Radius::Md.px()` em `Button::radius` — é exactamente o
/// estado em que esta wave encontrou o widget mais usado do app.
#[test]
fn the_corner_of_a_control_is_never_chosen_at_the_painting_site() {
    let found = strays();
    assert!(
        found.is_empty(),
        "{} sitio(s) escolhem a quina de um controlo em vez de chamar \
         `paint::frame_radius(theme, <o raio classico>)`. Num tema moderno cada um desenha uma \
         quina que os vizinhos ja' nao tem:\n  {}",
        found.len(),
        found.join("\n  ")
    );
}

/// ⛔ **A metade que impede a lista de virar licença** — e ela tem DUAS perguntas, porque uma
/// isenção pode morrer de duas maneiras: o ficheiro desaparecer, ou ele deixar de escrever raio
/// nenhum.
#[test]
fn the_exemptions_still_describe_something() {
    let root = repo_root();
    let mut stale = Vec::new();
    for (rel, _) in OUTSIDE_THE_DOOR {
        match fs::read_to_string(Path::new(&root).join(rel)) {
            Err(_) => stale.push(format!("{rel}: o ficheiro nao existe")),
            Ok(src) => {
                if !without_tests(&src).contains("Radius::") {
                    stale.push(format!(
                        "{rel}: ja' nao escreve raio nenhum — a isencao descreve nada"
                    ));
                }
            }
        }
    }
    assert!(
        stale.is_empty(),
        "{} isencao(oes) obsoleta(s):\n  {}",
        stale.len(),
        stale.join("\n  ")
    );
}

/// ⭐⭐ **A porta ACHATA, e é por isso que uma FORMA não pode passar por ela.**
///
/// ⚠️ Sem esta afirmação, a partição em três classes lê-se como zelo: ela é o **mecanismo** que
/// obriga `Radius::Full` a ficar de fora. Uma pílula que atravessasse a porta sairia com a quina
/// de um botão, e um avatar redondo sairia quadrado.
#[test]
fn the_door_flattens_every_positive_radius_including_a_pill() {
    use ph2d_tokens::{Radius, Theme};
    let pill = Radius::Full.px();
    let chip = Radius::Sm.px();
    let flat_pill = ph2d_editor_core::paint::frame_radius(Theme::Dark, pill);
    let flat_chip = ph2d_editor_core::paint::frame_radius(Theme::Dark, chip);
    assert_eq!(
        flat_pill, flat_chip,
        "a porta trata uma PILULA ({pill}) e um chip ({chip}) de maneiras diferentes — entao a \
         razao pela qual as formas ficam de fora desta lei deixou de existir, e a lista de \
         isencoes tem de ser reconferida"
    );
    assert!(
        flat_pill < pill,
        "a porta deixou de achatar: ela devolve {flat_pill} para uma pilula de {pill}"
    );
    // E no clássico ela devolve exactamente o que recebeu — é isso que torna a conversão
    // byte-idêntica para quem corre `PH2D_UI_NEW=0`.
    assert_eq!(
        ph2d_editor_core::paint::frame_radius(Theme::Forge, chip),
        chip
    );
}
