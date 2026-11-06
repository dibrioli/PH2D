//! ⭐⭐⭐ **NENHUM BRIDGE ESCONDE O INSPECTOR** — a metade de FONTE do report de 2026-09-08.
//!
//! > *«algumas ferramentas ou painéis não criam abas»* · *«inspector … apaga ao ser estreitado sem
//! > que o painel seja recolhido»* — Enio.
//!
//! Cinco bridges tinham um interruptor de flanco `panel_visibility.insert("inspector", !active)`:
//! o modelo de *takeover* anterior às abas. A metade de COMPORTAMENTO vive em
//! `ph2d-panel-registry-init/tests/it/a_slot_with_two_panels_shows_tabs.rs`
//! (`a_tool_panel_joins_the_inspector_as_a_tab_instead_of_replacing_it`); esta afirma que **ninguém
//! voltou a escrevê-lo**, que é a metade que um gate de comportamento não vê: um bridge NOVO com a
//! linha velha passaria naquele e escondia o Inspector na mesma.
//!
//! ⚠️ **Lê só CÓDIGO.** A explicação de por que a linha saiu está nos comentários dos próprios
//! bridges e contém a string proibida — um censo textual que não separa prosa de código mente nos
//! dois sentidos, e esta linha já pagou isso três vezes.

use std::fs;
use std::path::Path;

/// O ficheiro sem comentários de linha **e sem espaço nenhum** — ver o cabeçalho.
///
/// ⛔⛔ **A 1.ª redacção deste gate era CEGA à forma que o `rustfmt` escreve**, e deixou passar a
/// SEXTA superfície no mesmo dia em que nasceu: o `painter_bridge` tinha
///
/// ```text
/// hero.panel_visibility
///     .insert("inspector", !painter_is_active);
/// ```
///
/// — a mesma chamada, partida em duas linhas porque o receptor é longo. Um `contains` sobre o
/// texto cru procura uma sequência que a formatação decide, não uma que o código tem.
///
/// ⚠️ *É a QUARTA vez que esta linha paga a mesma lição — um censo textual tem de conhecer TODAS as
/// formas do que lê* (w25 o `cargo fmt`, w27 a prosa, w33 o doc-comment, e agora a quebra de
/// linha). ⇒ o espaço sai todo antes da comparação.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn no_bridge_hides_the_inspector_to_take_its_slot() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop");
    let mut scanned = 0usize;
    let mut guilty: Vec<String> = Vec::new();
    for e in fs::read_dir(&dir)
        .expect("a pasta dos bridges existe")
        .flatten()
    {
        let p = e.path();
        if p.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        scanned += 1;
        if code_only(&src).contains("panel_visibility.insert(\"inspector\"") {
            guilty.push(p.file_name().unwrap_or_default().to_string_lossy().into());
        }
    }
    // ⚠️ **O controlo do INSTRUMENTO:** uma pasta renomeada devolveria zero ficheiros e este gate
    //    leria como aprovado sobre nada.
    assert!(
        scanned >= 10,
        "só {scanned} bridges varridos em {dir:?} — a varredura perdeu o alvo"
    );
    guilty.sort();
    assert!(
        guilty.is_empty(),
        "estes bridges escondem o Inspector para lhe tomar o encaixe: {guilty:?}\ncura: não \
         esconda ninguém — os dois são OCUPANTES do mesmo encaixe e a fileira de abas nasce \
         sozinha. Quem fica à frente é decidido pelo `slot_tabs::reconcile_z`."
    );
}

/// Os nomes pelos quais este código escreve o id do Inspector: o literal e toda constante com esse valor.
fn formas_do_inspector(code: &str) -> Vec<String> {
    let mut formas = vec!["\"inspector\"".to_owned()];
    // Sem espaços, o `const` chega colado ao nome.
    let pedacos: Vec<&str> = code.split(":&str=\"inspector\"").collect();
    for antes in &pedacos[..pedacos.len() - 1] {
        formas.extend(antes.rsplit("const").next().map(str::to_owned));
    }
    formas
}

/// `true` se o código fecha o Inspector, ou lhe grava um valor que não seja «abre». O smoke que o abre é legítimo.
fn esconde_o_inspector(code: &str) -> bool {
    const ESCREVE: &str = "panel_visibility.insert(";
    let formas = formas_do_inspector(code);
    let mut resto = code;
    while let Some(i) = resto.find(ESCREVE) {
        resto = &resto[i + ESCREVE.len()..];
        let Some((id, depois)) = resto.split_once(',') else {
            break;
        };
        let valor = depois.split(')').next().unwrap_or_default();
        if formas.iter().any(|f| f == id) && valor != "true" {
            return true;
        }
    }
    false
}

fn varre_rs(dir: &Path, achados: &mut Vec<std::path::PathBuf>) {
    for e in fs::read_dir(dir).expect("a pasta existe").flatten() {
        let p = e.path();
        if p.is_dir() {
            varre_rs(&p, achados);
        } else if p.extension().is_some_and(|x| x == "rs") {
            achados.push(p);
        }
    }
}

/// O censo acima só via o `render_loop`, e as pontes moram nas crates de família desde a Fase C.
#[test]
fn no_family_crate_hides_the_inspector_to_take_its_slot() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates");
    let mut familias = 0usize;
    let mut arquivos = Vec::new();
    for e in fs::read_dir(&crates).expect("crates/").flatten() {
        let nome = e.file_name().to_string_lossy().into_owned();
        if nome.starts_with("ph2d-app-") {
            familias += 1;
            varre_rs(&e.path().join("src"), &mut arquivos);
        }
    }
    assert!(
        familias >= 10 && arquivos.len() >= 100 && crates.join("ph2d-app-sfxlab").is_dir(),
        "só {familias} famílias e {} ficheiros: a varredura perdeu o alvo",
        arquivos.len()
    );
    let mut culpados: Vec<String> = arquivos
        .iter()
        .filter(|p| fs::read_to_string(p).is_ok_and(|s| esconde_o_inspector(&code_only(&s))))
        .map(|p| p.strip_prefix(&crates).unwrap_or(p).display().to_string())
        .collect();
    culpados.sort();
    assert!(
        culpados.is_empty(),
        "estas famílias escondem o Inspector para lhe tomar a vaga: {culpados:?}\ncura: o painel entra \
         como ABA ao lado dele (`slot_tabs::reconcile_z`)."
    );
}

#[test]
fn o_censo_le_as_formas_que_o_rustfmt_e_as_constantes_escrevem() {
    let fecha = "const INSPECTOR: &str = \"inspector\";\nfn f(){hero.panel_visibility\n .insert(INSPECTOR, false);}";
    assert!(esconde_o_inspector(&code_only(fecha)));
    let negado = "hero.panel_visibility.insert(\"inspector\", !ativo);";
    assert!(esconde_o_inspector(&code_only(negado)));
    let abre = "hero.panel_visibility.insert(\"inspector\", true);";
    assert!(!esconde_o_inspector(&code_only(abre)));
    let outro = "hero.panel_visibility.insert(\"motion\", false);";
    assert!(!esconde_o_inspector(&code_only(outro)));
}
