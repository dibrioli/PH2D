//! ⭐⭐⭐ **A COLUNA DO RÓTULO de uma linha de propriedade é UMA resposta — e era SEIS.**
//!
//! ⛔⛔ **Report do dono, 2026-09-14, com duas fotos:** *«Label acima do campo numérico! Muito
//! ruim!»* (a secção LEG do Platform Player) e *«número na frente da label»* (a Sprite Sheet) —
//! seguido de *«falta para nós um modelo pronto e bem estabelecido com todas as regras para todos
//! os widgets … o inspector está assustador de horrível»*.
//!
//! Medido no mesmo dia: a largura da coluna do rótulo tinha **seis** respostas no app, cada uma um
//! literal com dispensa de gate —
//!
//! | valor | onde |
//! |---|---|
//! | `96` | `inspector/ordering` (×2) · `inspector/anchor_mount_row` |
//! | `78` | `inspector/sprite_sheet` · `inspector/transform` |
//! | `84` | `panel-color-equalization` |
//! | `76` | `panel-bgremoval` |
//! | `72` | `panel-equalize-sizes` |
//! | `150` | `panel-grid-snap` |
//!
//! — e, além delas, **duas** funções do Inspector nem coluna tinham: empilhavam o rótulo por cima
//! do campo (`sections/rows::num_row`, `sections/visibility::number_row`), que é a foto 1.
//!
//! # ⛔ Uma largura FIXA está errada por construção, e a prova não é de gosto
//!
//! A coluna docada é **arrastável** (`WidgetStore::DOCK_W_MIN`..`720`). Um rótulo de `96 px` numa
//! coluna aberta a `720` deixa o controlo com `600`; na largura mínima ele come a linha. *Os seis
//! literais não são seis gostos — são seis leituras da MESMA coluna à largura de omissão.*
//! ⇒ a porta é a [`ph2d_editor_core::widget::property_row_columns`], que devolve a coluna como
//! **fracção** da linha, com o piso do controlo nomeado (o *stepper* mais um dígito).
//!
//! # ⚠️ A catraca traz o censo de obsolescência ao lado
//!
//! `CLAUDE.md` §5.0: *uma catraca sem censo de obsolescência não desce — ela vira LICENÇA.* A
//! segunda metade pergunta, por entrada, se o ficheiro ainda existe e se ainda escreve o literal.

use std::fs;
use std::path::{Path, PathBuf};

/// ⏳ **Dívida MEDIDA, e só ENCOLHE.** Os painéis que ainda escrevem a coluna à mão — o Inspector
/// saiu desta lista em 2026-09-14, no commit que abriu a porta.
///
/// ⚠️ **Não acrescente entradas.** Uma linha de propriedade nova chama a porta; é uma chamada.
const AINDA_A_MAO: &[&str] = &[
    // ⚠️⚠️ **A varredura corrigiu o meu próprio número: não eram SEIS respostas, são ONZE.** O
    // censo que escrevi à mão procurava `label_col_w`/`LABEL_COL_W` no Inspector e nos painéis que
    // eu tinha aberto; esta régua varre as fontes de UI todas e devolve `64` (×4) · `72` · `78`
    // (×2) · `84` · `176` **além** dos seis que a foto do dono me pôs à frente. *Um censo escrito à
    // mão mede os sítios de que já se suspeita.*
    //
    // ⭐⭐⭐ **E em 2026-09-14 os DOZE painéis saíram daqui, num dia.** A dívida do painel de vetor
    // dizia *«33 sítios … muitos deles sem o `y`/`row_h` em alcance, logo a conversão é uma wave
    // própria e não um `sed`»* — e a largura **nunca dependeu do vertical**
    // ([`ph2d_editor_core::widget::property_label_col_w`] lê `x` e `w` e mais nada). *O preço
    // estava escrito na moeda errada, e os outros onze painéis foram cobrados pelo mesmo engano.*
    //
    // ⚠️ **O da timeline responde a OUTRA pergunta** — a coluna de nome de uma FAIXA, não a de uma
    // linha de propriedade —, e fica aqui por a régua ser textual e não saber distinguir as duas.
    // Quem o converter decide primeiro se a pergunta é a mesma; ⛔ não a force.
    "crates/ph2d-panel-timeline/src/tracks.rs",
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

fn ui_sources(root: &Path) -> Vec<PathBuf> {
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

/// Os sítios que escrevem a coluna do rótulo como NÚMERO — `<algo>label_col<algo> = 123.0`.
fn hand_written_label_columns(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for p in ui_sources(root) {
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        let rel = p
            .strip_prefix(root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        // ⛔ **A própria porta não é um sítio de pintura** — ela é onde a resposta VIVE.
        if rel.ends_with("widget/property_box/mod.rs") {
            continue;
        }
        for (n, line) in src.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") {
                continue;
            }
            let baixo = t.to_ascii_lowercase();
            if !baixo.contains("label_col") {
                continue;
            }
            // ⚠️ A régua é a ATRIBUIÇÃO de um número, não a menção: um sítio que passa a
            // `row.label.w` adiante menciona o nome e não escolhe nada.
            let Some((_, dir)) = t.split_once('=') else {
                continue;
            };
            if dir.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
                out.push(format!("{rel}:{}: {t}", n + 1));
            }
        }
    }
    out
}

/// ⭐ **Ninguém escolhe a largura da coluna do rótulo — ela sai da porta.**
#[test]
fn the_label_column_is_never_chosen_at_the_painting_site() {
    let root = repo_root();
    let found = hand_written_label_columns(&root);
    let fora: Vec<&String> = found
        .iter()
        .filter(|l| !AINDA_A_MAO.iter().any(|d| l.starts_with(d)))
        .collect();
    assert!(
        fora.is_empty(),
        "{} sitio(s) escolhem a largura da coluna do rotulo em vez de chamar \
         `ph2d_editor_core::widget::property_row_columns`:\n  {}\n\nUma largura FIXA nao sobrevive \
         a arrastar a coluna docada (DOCK_W_MIN..720) — e a mesma pergunta ja' teve SEIS respostas \
         neste app.",
        fora.len(),
        fora.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** — uma tolerância que já não descreve nada sai da lista.
#[test]
fn the_tolerated_list_still_describes_something() {
    let root = repo_root();
    let found = hand_written_label_columns(&root);
    let mortas: Vec<&&str> = AINDA_A_MAO
        .iter()
        .filter(|d| !root.join(d).exists() || !found.iter().any(|l| l.starts_with(**d)))
        .collect();
    assert!(
        mortas.is_empty(),
        "entrada(s) STALE na lista de tolerancia — o ficheiro sumiu ou ja' nao escolhe a coluna, \
         entao a linha sai da lista:\n  {mortas:?}"
    );
}

/// ⭐⭐ **A TERCEIRA metade: o PISO DO CONTROLO é honrado — a linha nunca entrega um campo inútil.**
///
/// ⛔⛔ **A primeira redacção deste gate comparava as duas portas** (`property_row_columns().label.w`
/// contra `property_label_col_w()`) **e a prova de mutação disse-o inútil: ela SOBREVIVEU.** Desde
/// 2026-09-14 a porta grande **chama** a pequena, logo as duas mudam juntas — *um gate que compara
/// duas construções é cego a uma mutação partilhada*, e este comparava a função consigo própria.
///
/// ⇒ o que se mede aqui é a **LEI**, com um oráculo do lado de fora: o rótulo cresce com a linha,
/// mas **nunca** à custa do controlo. O recurso é nomeado ([`PropertyRow`]): a coluna do *stepper*
/// de um `NumberInput` mais um dígito. Apagar o tecto (`.min(…)`) da porta faz uma linha de `100 px`
/// entregar um campo de `48` onde o piso é `52` — e este gate reprova.
#[test]
fn a_property_row_never_starves_its_control() {
    use ph2d_editor_core::widget::{property_label_col_w, property_row_columns};
    let gap = ph2d_tokens::Spacing::Md.px();
    let piso = ph2d_tokens::ICON_BTN_SIZE_PX + ph2d_tokens::Spacing::Lg.px();
    let mut apertadas = 0;
    let mut largas = 0;
    for w in (0..=760).step_by(4).map(|n| n as f32) {
        let r = property_row_columns(0.0, w, 0.0, 22.0);
        let util = r.label.w + gap + r.control.w;
        // ⚠️ A linha inteira é repartida: rótulo + vão + controlo. (A coluna de animação já saiu
        // antes disto, e é por isso que o total não é `w`.)
        //
        // ⛔ **Abaixo do piso a porta não reparte nada — ela entrega os MÍNIMOS** (um controlo de
        // `1 px`, um vão inteiro), e a soma passa a linha por construção. *Uma linha que já não
        // cabe não é uma linha de propriedade*, e afirmar a partição ali seria medir o degenerado.
        if w >= gap + piso {
            assert!(
                util <= w + 0.001,
                "w={w}: as colunas somam {util}, mais que a linha"
            );
        }
        if util >= gap + piso {
            // ⭐ **O caso que a mutação mata:** cabe um controlo utilizável ⇒ ele TEM de o ser.
            assert!(
                r.control.w + 0.001 >= piso,
                "w={w}: o controlo ficou com {} px, abaixo do piso de {piso} — o rotulo ({} px) \
                 comeu-o. O tecto da coluna do rotulo e' o RECURSO, nao um gosto.",
                r.control.w,
                r.label.w
            );
            apertadas += 1;
        }
        if w >= 200.0 {
            // ⚠️ E numa linha larga o rótulo é a MENOR das duas colunas — senão a fracção deixou
            // de ser uma fracção.
            assert!(
                r.label.w < r.control.w,
                "w={w}: rotulo {} >= controlo {}",
                r.label.w,
                r.control.w
            );
            largas += 1;
        }
        // A metade sem vertical responde o mesmo — ela é quem a porta grande chama.
        assert!((property_label_col_w(0.0, w) - r.label.w).abs() < f32::EPSILON);
    }
    // ⚠️ Piso de população nas DUAS famílias: uma varredura que colapse passa trivialmente.
    assert!(
        apertadas >= 100 && largas >= 100,
        "varredura magra: {apertadas} / {largas}"
    );
}
