//! **Um gesto lê os fatos que o DESENHO derivou** — arch-gate sobre a costura que nenhum teste
//! de unidade alcança.
//!
//! # Por que este arquivo existe
//!
//! Os intervalos de RECORTE (`VecClipSpan`) e as poses do auto layout são resultado do passe de
//! LAYOUT, que roda no desenho. O hit-test monta o `VecViewState` dele **do zero** a cada evento
//! de ponteiro, e a porta que ele usava (`vec_entities::view_state`) só sabe o que a ÁRVORE diz —
//! escondido e travado. As duas listas chegavam VAZIAS, e o gesto decidia como se nenhuma moldura
//! existisse e nenhuma forma tivesse sido colocada.
//!
//! ⚠️ **Isto já aconteceu, e passou despercebido por um commit inteiro.** A demoção do PAI no
//! pick (para o clique pegar o filho) lia `view_state.clips`; os gates dela montavam a lista À MÃO
//! e ficaram verdes, enquanto no produto a lista era vazia e a cura era **inerte**. Um gate de
//! unidade é cego à fiação da shell — este é o par dele. (A demoção morreu em 2026-08-04, quando a
//! projeção passou a pôr o pai por baixo; o recorte continua a atravessar esta mesma fronteira.)

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// **Todo arquivo que APONTA usa a porta que funde as duas metades.**
///
/// ⚠️ **A isenção é por DERIVAÇÃO, não por lista:** *aponta* quem chama uma das portas de pick
/// (elas recebem um ponto de MUNDO e perguntam que forma está lá). Quem só quer saber *se pode
/// mexer nisto* — escondido, travado — está a fazer uma pergunta da ÁRVORE, e a porta bare é a
/// resposta certa. Uma lista de nomes exemptos apodrece no dia em que um arquivo novo aponta;
/// esta regra cobre-o sozinha.
const PICK_DOORS: [&str; 3] = [
    "pick_all_at_world(",
    "pick_in_world_rect(",
    "contains_world(",
];

#[test]
fn every_gesture_site_merges_the_derived_facts() {
    let mut checked = 0;
    for entry in walk("src") {
        let src = read(&entry);
        if !PICK_DOORS.iter().any(|d| src.contains(d)) {
            continue; // este arquivo nao aponta
        }
        if entry.contains("_tests.rs") {
            continue; // um gate monta o estado que ele quer julgar
        }
        checked += 1;
        assert_eq!(
            src.matches("ph2d_vec_entities::entities::view_state(")
                .count(),
            0,
            "{entry} APONTA (chama uma porta de pick) e monta o VecViewState do ZERO — os \
             intervalos das molduras e as poses do layout chegam VAZIOS, e o clique decide como \
             se nenhuma moldura existisse. Use `view_state_for_pick`."
        );
    }
    assert!(
        checked >= 2,
        "a varredura nao achou os sitios que apontam ({checked}) — o gate esta' a medir nada"
    );
}

/// Todo `.rs` sob `dir`, recursivamente.
fn walk(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_string()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let name = p.to_string_lossy().to_string();
            if p.is_dir() {
                stack.push(name);
            } else if name.ends_with(".rs") {
                out.push(name);
            }
        }
    }
    out
}

/// **E o desenho PUBLICA o que derivou.** Sem esta metade a porta de fusão funde com nada, e o
/// gate acima passaria sobre um produto igualmente quebrado.
///
/// ⚠️ Desde a OBRA 2 da `line/render-loop` (2026-09-13) o passe de layout e a publicação moram na fase
/// `fase_vector_layout_recook`: o gate lê o QUADRO emendado (`frame_text::render_frame`), e a ORDEM layout →
/// publicação é a de execução.
#[test]
fn the_draw_pass_publishes_the_facts_it_derived() {
    let src = crate::frame_text::render_frame();
    // ⚠️ **Sem espaço em branco dos dois lados**, e a razão é que este gate já expirou uma vez por
    // isso: o campo mudou de nome (duas vezes — `clips` → `parent_spans` → `clips`, ao sabor de o
    // intervalo ser sobre antecipação ou sobre recorte), o `rustfmt` quebrou a linha no ponto, e a
    // agulha deixou de casar com código correto. O que se afirma é *"esta linha existe"*, não
    // *"ela cabe numa linha"*.
    let flat: String = src.chars().filter(|c| !c.is_whitespace()).collect();
    for (what, needle) in [
        ("os intervalos de RECORTE", "self.vec.view_derived.clips"),
        ("as poses do auto layout", "self.vec.view_derived.poses"),
    ] {
        assert!(
            flat.contains(needle),
            "o passe de desenho nao publica {what} — quem aponta nunca os vera'"
        );
    }
    // ⚠️ **A ORDEM é load-bearing**: publicar ANTES do passe de layout publicaria a tabela do
    // frame anterior, e a forma recém-colocada ficaria um frame inteira sem pose.
    //
    // ⚠️ E a metade da ORDEM lê o `flat` pela MESMA razão que a de cima — ela ficou para trás e
    // expirou na W4c.4: a chamada ganhou um argumento, o `rustfmt` a re-quebrou, e a agulha
    // (`".layout_live\n                .recook("`, com a indentação escrita à mão) deixou de casar
    // com código correto. *Uma âncora que inclui espaço em branco afirma a FORMATAÇÃO, não o
    // produto* — e curar só uma das duas metades é como esta voltou a morder.
    let recook = flat
        .find("self.layout_live.recook(")
        .expect("o passe de layout");
    let publish = flat
        .find("self.vec.view_derived.poses")
        .expect("a publicacao das poses");
    assert!(
        publish > recook,
        "as poses sao publicadas ANTES do passe que as produz — a tabela seria a do frame anterior"
    );
}

// ⚠️⚠️ **O `the_hit_test_composes_the_layout_pose` SAIU daqui na integração de 2026-09-17**, com o
// `vec_gizmo_view.rs` que ele julga (hoje em `crates/ph2d-app-vec/tests/it/`). ⛔ Ele lia o ficheiro
// **por caminho em tempo de execução**, logo sobreviveu ao `git mv` como um teste VERMELHO em vez de
// um erro de compilação — o `cargo check --all-targets` ficou verde e ele foi o único vermelho de
// `16 116` na árvore combinada. Do outro lado ele usa `include_str!`, que falha a COMPILAR.
//
// ⚠️ E a varredura acima passou a ver **cinco** sítios onde via sete: os outros dois vieram com o
// cacho, e a crate tem agora a MESMA lei com o piso dela. *Um censo que passa a varrer MENOS
// lê-se como «já não há mais nada»*, e o piso de `2` daqui nunca o teria acusado.
