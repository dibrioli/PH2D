//! ⭐⭐⭐ **O MANUAL DA LINHA DE PROPRIEDADE NÃO PODE NOMEAR UMA PORTA QUE NÃO EXISTE.**
//!
//! ⛔⛔ **Um doc que enuncia a lei que o código não implementa lê-se como AUDITADO** — é a forma
//! mais cara de mentira deste repo, porque quem o lê deixa de conferir. A spec
//! [`docs/UI_New_and_Simple/spec/03_a_linha_de_propriedade.md`] fecha com uma tabela
//! `lei → porta → gate`, e este teste é o que a mantém honesta: **cada nome das duas colunas tem de
//! existir no código**.
//!
//! ⚠️ **Ele não prova que a lei está implementada** — isso é trabalho do gate que a linha NOMEIA.
//! O que ele prova é o elo: *a página aponta para coisas reais, e um `rename` ou uma remoção
//! reprova aqui em vez de apodrecer em silêncio*. (A outra metade — *o gate nomeado ainda corre?* —
//! sai de graça: um teste apagado deixa de ser encontrado como `fn`.)
//!
//! ⚠️ **Piso de população:** uma tabela que encolhe por uma edição distraída passaria a medir nada.

use std::fs;
use std::path::{Path, PathBuf};

const MANUAL: &str = "docs/UI_New_and_Simple/spec/03_a_linha_de_propriedade.md";

/// A tabela do §9 tinha **13** linhas quando nasceu. ⛔ Ela pode CRESCER; encolher é sinal de que
/// alguém apagou uma lei sem apagar o que a defende.
const LINHAS_MINIMAS: usize = 13;

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

/// Todo o Rust das crates — o corpo onde uma porta ou um gate pode viver.
fn todo_o_rust() -> String {
    let mut ficheiros = Vec::new();
    walk(&repo_root().join("crates"), &mut ficheiros);
    let mut junto = String::new();
    for f in &ficheiros {
        if let Ok(s) = fs::read_to_string(f) {
            junto.push_str(&s);
            junto.push('\n');
        }
    }
    assert!(
        ficheiros.len() > 1000,
        "a varredura achou {} ficheiros .rs — ela deixou de alcancar as crates",
        ficheiros.len()
    );
    junto
}

/// ⚠️ **As cinco formas com que um nome NASCE em Rust** — mais o `as`, porque uma porta pode ser
/// re-exportada com outro nome (`MIN_W_PX as NUMBER_INPUT_MIN_W_PX` é exactamente esse caso, e a
/// 1.ª redacção deste gate acusava-a de não existir).
fn declarado(corpo: &str, nome: &str) -> bool {
    [
        format!("fn {nome}"),
        format!("const {nome}"),
        format!("static {nome}"),
        format!("enum {nome}"),
        format!("struct {nome}"),
        format!("type {nome}"),
        format!("as {nome}"),
    ]
    .iter()
    .any(|agulha| corpo.contains(agulha.as_str()))
}

/// O texto da secção **§9** do manual — e só dele.
///
/// ⛔⛔ **A 1.ª redacção deste gate procurava a tabela pela FORMA** (quatro colunas, as duas
/// últimas em crase) e por isso lia qualquer tabela do documento com essa forma. Em 2026-09-15 a
/// §6 ganhou uma tabela de MEDIÇÃO com quatro colunas (`linha | o nome quer | a coluna | a caixa`)
/// e o gate acusou `"90,0` (a metade)"` de não ser uma porta do código — *ele estava certo sobre o
/// texto e errado sobre onde olhar*.
///
/// ⇒ *um censo identifica o seu sujeito pelo ENDEREÇO, nunca pela forma* — a mesma lei que o
/// `CLAUDE.md` §5.0 escreve para os censos que varrem um directório por prefixo. Com o endereço, o
/// gate fica **mais forte**: uma tabela de medição nova noutra secção deixa de o partir, e uma §9
/// que mude de número parte-o **alto** (o piso de população).
fn seccao_nove(md: &str) -> &str {
    let ini = md
        .find("\n## §9 ")
        .expect("o manual perdeu a secção §9 — o gate deixaria de medir a tabela das leis");
    let resto = &md[ini + 1..];
    match resto[1..].find("\n## ") {
        Some(k) => &resto[..k + 2],
        None => resto,
    }
}

/// As células `porta` e `gate` de cada linha da tabela do §9.
fn pares_do_manual(md: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for linha in seccao_nove(md).lines() {
        let l = linha.trim();
        if !l.starts_with('|') {
            continue;
        }
        let celulas: Vec<&str> = l.trim_matches('|').split('|').map(str::trim).collect();
        // `§ | lei | porta | gate` — quatro colunas, e as duas últimas em crase.
        if celulas.len() != 4 {
            continue;
        }
        let (porta, gate) = (celulas[2], celulas[3]);
        if !porta.starts_with('`') || !gate.starts_with('`') {
            continue;
        }
        out.push((
            celulas[1].to_string(),
            porta.trim_matches('`').to_string(),
            gate.trim_matches('`').to_string(),
        ));
    }
    out
}

#[test]
fn the_property_row_manual_names_only_doors_that_exist() {
    let md = fs::read_to_string(repo_root().join(MANUAL))
        .unwrap_or_else(|e| panic!("o manual {MANUAL} nao foi lido: {e}"));
    let pares = pares_do_manual(&md);
    assert!(
        pares.len() >= LINHAS_MINIMAS,
        "a tabela do §9 tem {} linhas e o piso e' {LINHAS_MINIMAS} — ela encolheu, ou o formato \
         das colunas mudou e este gate passou a medir NADA",
        pares.len()
    );
    let corpo = todo_o_rust();
    let mut fantasmas = Vec::new();
    for (lei, porta, gate) in &pares {
        if !declarado(&corpo, porta) {
            fantasmas.push(format!("porta {porta:?} (lei: {lei})"));
        }
        if !declarado(&corpo, gate) {
            fantasmas.push(format!("gate {gate:?} (lei: {lei})"));
        }
    }
    assert!(
        fantasmas.is_empty(),
        "{} nome(s) do manual nao existem no codigo:\n  {}\n\n\
         Ou o nome mudou (actualize o manual) ou a lei deixou de ter quem a defenda (e ai' o \
         manual esta' a prometer o que o app nao faz).",
        fantasmas.len(),
        fantasmas.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO** — sem ele, um parser que deixasse de encontrar linhas passaria por «zero
/// fantasmas» sobre uma tabela inteira por verificar.
///
/// ⚠️ *Uma fixtura sem o fenómeno mede silêncio*: aqui inventa-se um nome que **de certeza** não
/// existe e exige-se que o detector o veja.
#[test]
fn the_ruler_can_see_a_door_that_does_not_exist() {
    let corpo = todo_o_rust();
    assert!(
        !declarado(&corpo, "uma_porta_que_nunca_existiu_no_ph2d"),
        "o detector acha QUALQUER nome — ele nao esta' a medir nada"
    );
    assert!(
        declarado(&corpo, "property_row_columns"),
        "o detector nao acha uma porta que existe — ele esta' a recusar tudo"
    );
}
