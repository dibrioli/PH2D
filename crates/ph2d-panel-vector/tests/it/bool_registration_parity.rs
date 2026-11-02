//! Gate de arquitetura — **paridade de REGISTRO** da família booleana do painel vetorial.
//!
//! Irmão do `architecture_topbar_registration_parity` da `ph2d-editor-core`, e existe pela mesma
//! razão exacta: um chip **PINTADO** e **HIT-INDEXADO** mas sem `InteractiveState` no
//! `populate_ops` tem `is_focusable() == false` — o Down nunca o torna ativo, o Up nunca emite
//! `Click`, e **ele está morto sob o ponteiro** enquanto a suíte inteira fica verde, porque nada
//! mais afirma a paridade.
//!
//! # A conta desta classe neste repo
//!
//! - as **36 células** da matriz de física;
//! - os **10 chips** de ferramenta do Painter;
//! - as **4 pills** vetoriais PENCIL/SHAPE/SELECT/DIRECT (CI-verde-e-mortas por várias sessões);
//! - e, em 2026-08-22, os **4 chips do verbo por forma** — que levaram o Enio a reportar
//!   *"os botões não funcionam"* **duas vezes**.
//!
//! ⚠️ O gesto real sobre a ④ (`the_vector_bars_answer_a_real_pointer`, crate do registo) apanha os
//! que existem HOJE. Este gate apanha **o próximo**, no dia em que alguém o acrescentar — que é a
//! diferença entre consertar e prevenir.
//!
//! # Por que a família `VECTOR_BOOL_*`, e não todo id do painel
//!
//! Nem todo id pintado é um botão: sliders, campos numéricos e cabeçalhos de seção registam-se de
//! outras formas (ou não se registam). Um gate universal precisaria de uma lista de exceções tão
//! grande que se tornaria ruído — e ⛔ *um gate ruidoso é silenciado, não obedecido*. A família
//! booleana é homogénea (são todos botões/chips), então aqui a paridade é exacta e sem exceções.

use std::collections::BTreeSet;
use std::path::Path;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("não leio {}: {e}", path.display()))
}

/// Os nomes da família que COMBINA formas (`VECTOR_BOOL_*`, `VECTOR_FLATTEN`, `VECTOR_COMPOUND_*`)
/// que aparecem em `src`, sem o prefixo `ids::`.
fn bool_ids(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for prefix in ["VECTOR_BOOL_", "VECTOR_FLATTEN", "VECTOR_COMPOUND_"] {
        for (i, _) in src.match_indices(prefix) {
            let tail = &src[i..];
            let end = tail
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(tail.len());
            out.insert(tail[..end].to_string());
        }
    }
    out
}

/// **Todo chip booleano que o painel PINTA é registrado no `populate_ops`.**
///
/// ⚠️ A direção importa: o gate mede *pintado ⇒ registrado*, e não o contrário. Um id registrado e
/// não pintado é inofensivo (um botão que não aparece); um id pintado e não registrado é um
/// controlo **morto sob o dedo**, e do lado de fora ele é indistinguível de um que ignora o
/// clique — foi por isso que o report *"não funcionam"* não bastou para localizar a causa.
#[test]
fn every_painted_boolean_chip_is_registered_in_populate() {
    // ⚠️ **O ficheiro mudou em 2026-09-05 e outra vez em 2026-11-02** — a secção Boolean saiu do
    // `paint_sections.rs` para o `paint_boolean.rs` (teto de LOC), e na F50 saiu do PAINEL para a
    // barra ④ (`bars_actions.rs`; o «⋯» dela saiu no smoke do dono, 02/11). O controlo `!painted.is_empty()` é o que apanha
    // a régua apontada a um ficheiro velho: *a régua quebrou, não o produto*.
    let painted = bool_ids(&read("src/bars_actions.rs"));
    let registered = bool_ids(&read("src/populate_ops.rs"));
    assert!(
        !painted.is_empty(),
        "o scanner não achou id booleano nenhum no pintor — a régua quebrou, não o produto"
    );
    let dead: Vec<&String> = painted.difference(&registered).collect();
    assert!(
        dead.is_empty(),
        "chip(s) booleano(s) PINTADO(S) e não registrado(s) no `populate_ops` — mortos sob o \
         ponteiro, com a suíte verde:\n  {dead:?}\n\
         conserto: uma linha `button(store, ids::<ID>);` no `populate_ops.rs`."
    );
}
