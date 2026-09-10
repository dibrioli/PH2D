//! ⭐⭐⭐ **A APARÊNCIA É UMA PELE DE WIDGET — nunca um modelo de áreas.**
//!
//! O `CLAUDE.md` §5 lista, entre os smokes deste módulo, *«`PH2D_UI_NEW=0` (o clássico tem de
//! ficar **byte a byte**)»*. ⛔⛔ **Medido em 2026-09-10, isso é falso — e já era falso no dia em
//! que foi escrito.** O sistema de abas de encaixe nasceu em **2026-08-30** (`c7c5c653a`) sem
//! consultar a aparência em sítio nenhum, logo corre nas DUAS; a cláusula entrou no roteador em
//! **2026-09-07** (`5fe3c51cc`), **oito dias depois**. *Uma promessa escrita depois do facto que a
//! desmente não é uma regressão de quem veio a seguir.*
//!
//! # O que o interruptor de facto devolve (censo derivado, 2026-09-10)
//!
//! | consumidor | o que muda |
//! |---|---|
//! | `widget/checkbox/mark.rs` · `toggle.rs` · `slider_with_chip/mod.rs` · `showcase/slider.rs` · `property_box/mod.rs` · `property_box/paint.rs` | os pintores de widget — a PELE |
//! | `screens/hero/menu_rows.rs` | a **família de temas** que a barra do topo oferece (derivados no moderno, os de sempre no clássico) |
//! | `ph2d-panel-widget-lab/src/paint.rs` | a bancada, que se **força** ao redesenho porque é onde ele se estuda |
//! | `ph2d_tokens::Theme::default_for` | o tema de ARRANQUE (`Classic → Forge`, `Redesign → Dark`) |
//!
//! ⇒ o interruptor responde *«a caixa já era assim?»*. Ele **nunca** respondeu *«o ecrã já era
//! assim?»*, e é por isso que o gate irmão
//! [`the_two_looks_are_one_switch_apart`](the_two_looks_are_one_switch_apart.rs) mede TINTA de três
//! widgets: ele sempre soube qual era a pergunta.
//!
//! # ⛔ Por que a cura NÃO é gatear as abas pela aparência
//!
//! Seria manter **dois modelos de áreas** vivos para sempre — cada gate de layout, de coluna, de
//! faixa e de transbordo passaria a ter duas respostas certas —, para comprar uma bissecção que a
//! ferramenta nunca ofereceu. E a ordem do dono de 2026-09-09 (*«mesmo se houver apenas 1 painel, a
//! aba aparece sozinha»*) não traz aparência nenhuma dentro dela.
//!
//! ⇒ **a cláusula do §5 é que se corrige**, e este gate existe para que ela não volte por acidente:
//! no dia em que alguém puser a estrutura do ecrã atrás do interruptor, ele fica vermelho e a
//! decisão passa a ser deliberada.

use std::fs;
use std::path::{Path, PathBuf};

/// ⭐ A EXCEPÇÃO, com o mecanismo — e não uma lista aberta.
///
/// ⚠️ Ela vive em `screens/` e **não** decide geometria: escolhe que quatro nomes de tema a barra
/// do topo lista. Misturar as duas famílias poria um tema tingido ao lado de um plano sem o artista
/// saber que escolhe entre dois sistemas.
const NOT_AN_AREA_MODEL: &[(&str, &str)] = &[(
    "screens/hero/menu_rows.rs",
    "escolhe a FAMILIA de temas do menu da barra do topo, nao geometria nenhuma",
)];

/// O ficheiro que **declara** as duas portas — ele não as consome.
const DOOR: &str = "published.rs";

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// O ficheiro **sem comentários de linha** — a prosa não decide desenho nenhum.
///
/// ⚠️ É a 6.ª vez que esta linha paga a lição: um censo textual que lê um doc-comment como código
/// acusa quem nunca teve o defeito (o `published.rs` explica a aparência inteira em prosa).
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⚠️ **CONSUMIR e PUBLICAR são coisas diferentes**, e o discriminador é a chamada.
///
/// O `screens/hero/paint.rs` lê o ambiente e **publica** (`set_ui_look(ui_look_from_env())`) — ele
/// nunca pergunta *«qual é a aparência?»* para decidir um desenho. Um censo que casasse só o texto
/// `UiLook` acusaria o publicador, que é exactamente quem tem de existir.
fn consumes_the_look(code: &str) -> bool {
    code.contains("ui_is_redesign()") || code.contains("ui_look()")
}

fn rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs")
            && !p
                .file_name()
                .is_some_and(|n| n.to_str().is_some_and(|s| s.ends_with("_tests.rs")))
        {
            out.push(p);
        }
    }
}

/// `(caminho relativo a `src/`, consome?)` para toda a crate.
fn census() -> Vec<(String, bool)> {
    let root = src_root();
    let mut files = Vec::new();
    rs_files(&root, &mut files);
    files.sort();
    files
        .iter()
        .filter_map(|p| {
            let rel = p.strip_prefix(&root).ok()?.to_str()?.replace('\\', "/");
            let code = code_only(&fs::read_to_string(p).ok()?);
            Some((rel, consumes_the_look(&code)))
        })
        .collect()
}

/// ⭐⭐⭐ **Nada que decida a FORMA do ecrã pergunta qual é a aparência.**
#[test]
fn nothing_in_the_area_model_asks_which_look_is_on() {
    let c = census();

    // ⛔ **Controlo de vacuidade, com a metade justa:** uma varredura partida devolve zero
    //    acusados e lê-se como aprovada. A pele existe e tem de ser encontrada.
    let skin: Vec<&String> = c
        .iter()
        .filter(|(rel, yes)| *yes && rel.starts_with("widget/"))
        .map(|(rel, _)| rel)
        .collect();
    assert!(
        skin.len() >= 5,
        "o censo achou só {} pintor(es) de widget a ler a aparência — a varredura está partida, \
         e um censo partido lê-se como aprovado: {skin:?}",
        skin.len()
    );

    let intrusos: Vec<&String> = c
        .iter()
        .filter(|(rel, yes)| {
            *yes && rel.starts_with("screens/") && !NOT_AN_AREA_MODEL.iter().any(|(e, _)| e == rel)
        })
        .map(|(rel, _)| rel)
        .collect();
    assert!(
        intrusos.is_empty(),
        "a estrutura do ecrã passou a depender da aparência em {intrusos:?}.\n\
         Isso é DOIS modelos de áreas vivos ao mesmo tempo: cada gate de coluna, de faixa de abas \
         e de transbordo passa a ter duas respostas certas. Se for deliberado, é decisão do dono e \
         entra em `NOT_AN_AREA_MODEL` com o mecanismo ao lado; se não for, é o interruptor a \
         escorregar de PELE para MODELO."
    );

    // ⚠️ A porta declara-as e não as consome — se isso mudar, o discriminador acima deixou de
    //    separar publicar de consumir.
    assert!(
        c.iter().any(|(rel, yes)| rel == DOOR && *yes),
        "o `{DOOR}` deixou de conter as duas portas — o censo está a medir outra coisa"
    );
}

/// ⭐ **A METADE JUSTA: a excepção ainda descreve alguma coisa?**
///
/// ⛔⛔ Uma catraca sem censo de obsolescência não desce — ela vira licença (§5.0). Uma entrada
/// cujo ficheiro morreu, ou que já não lê a aparência, tem de SAIR.
#[test]
fn every_named_exception_still_describes_a_real_reader() {
    let c = census();
    for (rel, motivo) in NOT_AN_AREA_MODEL {
        assert!(
            !motivo.trim().is_empty(),
            "a excepção {rel} não diz o mecanismo — uma lista sem mecanismo é uma licença"
        );
        let Some((_, consome)) = c.iter().find(|(p, _)| p == rel) else {
            panic!("a excepção {rel} aponta um ficheiro que já não existe — apague a linha");
        };
        assert!(
            consome,
            "a excepção {rel} já não lê a aparência — apague a linha, senão ela é uma licença \
             aberta para o próximo ficheiro com esse nome"
        );
    }
}
