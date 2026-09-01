//! ⛔⛔ **O chip de um EIXO de propriedade religa pela porta do `swap`, e por mais nenhuma.**
//!
//! Este é o gémeo geral do `the_variant_chip_goes_through_the_swap_door` do sistema vetorial, e
//! existe pela mesma razão: o `instance_variant::swap` é quem faz o **re-key determinístico** das
//! excepções, quem **sepulta** os órfãos e quem **esquece o eco**. Escrever o vínculo por outra via
//! deixa a cópia a guardar diferenças que apontam peças do mestre ANTIGO — e o sintoma é uma
//! excepção que reaparece no sítio errado, meses depois, sem ninguém saber ligar as duas coisas.
//!
//! ⚠️ **Textual de propósito.** A alternativa é um teste de integração que monte uma família e
//! clique — e ele prova que ESTE caminho funciona, não que **não existe um segundo**. A ausência de
//! uma segunda escrita não se mede correndo o caminho certo.
//!
//! ⛔ Ele descasca comentários antes de varrer: documentar a cura não pode reprovar o portão.
//!
//! ⚠️ **O alvo mudou de ficheiro em 2026-08-31** (`event.rs` → `event_value.rs`): o braço passou a
//! precisar do estado do painel — *qual eixo está aberto para escrita* — e essa família corre antes
//! do `apply_event_impl`, com o `InspectorState` na mão, como o `event_anchor` e o `event_anim`.
//! *Um gate textual segue o código; ele não o prende ao ficheiro em que nasceu.*

use std::path::Path;

fn src(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Tira comentários de linha — a lição da caça de 2026-08-30.
fn strip_comments(s: &str) -> String {
    s.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐ **O chip resolve o alvo pelo modelo que o PINTOU.**
///
/// ⚠️ O `rows` do `InspectorPropertiesInfo` é a lista que o cartão desenhou **neste quadro**; uma
/// segunda travessia (recalcular a família aqui) daria uma ordem que pode divergir, e o chip `Big`
/// escolheria `Medium`. É a lição literal do `addressed_pieces` do vetor.
#[test]
fn the_axis_chip_reads_the_model_that_painted_it() {
    let body = strip_comments(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .expect("raiz do repo")
                .join("crates/ph2d-panel-inspector/src/event_value.rs"),
        )
        .expect("event_value.rs do inspector"),
    );
    assert!(
        body.contains("ids::instance_axis_option(id)"),
        "o braço do chip não usa a PORTA da escada de ids — uma varredura à mão aqui seria a \
         quarta cópia da mesma lei"
    );
    assert!(
        body.contains("info.rows.get(a).and_then(|ax| ax.options.get(v))"),
        "o braço não resolve o chip no modelo que o pintou"
    );
    assert!(
        body.contains("EditorAction::InspectorSwapVariant"),
        "o chip não levanta a acção da troca — ele ficaria pintado e morto"
    );
}

/// ⛔ **E o painel NÃO escreve o vínculo por outra via.**
///
/// ⚠️ A metade que o gate acima não cobre: a acção podia ser levantada **e** uma segunda linha
/// escrever o mestre à mão, o que é o defeito com todos os gates verdes.
#[test]
fn the_inspector_never_writes_the_master_link_by_hand() {
    let body = strip_comments(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .expect("raiz do repo")
                .join("crates/ph2d-panel-inspector/src/event_value.rs"),
        )
        .expect("event_value.rs do inspector"),
    );
    for forbidden in ["InstanceOf {", "InstanceOf::new("] {
        assert!(
            !body.contains(forbidden),
            "o painel escreve o vínculo por `{forbidden}` — o re-key das excepções vive dentro do \
             `instance_variant::swap`, e uma segunda escrita pula-o em silêncio"
        );
    }
    assert!(
        !assigns_field(&body, ".master"),
        "o painel escreve o vínculo por `.master =` — o re-key das excepções vive dentro do \
         `instance_variant::swap`, e uma segunda escrita pula-o em silêncio"
    );
}

/// ⛔⛔ **`.master =` casava DENTRO de `.master == 0`** (achado de 2026-08-31).
///
/// Um gate textual que proíbe uma **atribuição** tem de saber distingui-la de uma **comparação** —
/// senão ele acusa quem escreveu um `if`, com uma mensagem sobre re-key de excepções que não tem
/// nada a ver, e o autor seguinte aprende a ignorá-lo. *Um portão que acusa o inocente deixa de ser
/// lido.*
///
/// ⚠️ Ele exige o `=` **não** seguido de `=` — e aceita qualquer espaço entre o campo e o sinal,
/// porque o `cargo fmt` decide isso e não o autor.
fn assigns_field(body: &str, field: &str) -> bool {
    let b = body.as_bytes();
    let mut from = 0;
    while let Some(i) = body[from..].find(field) {
        let mut j = from + i + field.len();
        while b.get(j) == Some(&b' ') {
            j += 1;
        }
        if b.get(j) == Some(&b'=') && b.get(j + 1) != Some(&b'=') {
            return true;
        }
        from += i + field.len();
    }
    false
}

/// ⚠️ **O CONTROLO do detector** — sem ele, um `assigns_field` que devolvesse sempre `false`
/// deixaria o gate acima verde para sempre, e ele diria exactamente o mesmo que diz hoje.
#[test]
fn the_assignment_detector_tells_an_assignment_from_a_comparison() {
    assert!(assigns_field("link.master = other;", ".master"));
    assert!(assigns_field("link.master   = other;", ".master"));
    assert!(!assigns_field("if choice.master == 0 {", ".master"));
    assert!(!assigns_field("let m = choice.master;", ".master"));
}

/// ⭐⭐ **E o shell dreno passa pela porta do `swap`.**
#[test]
fn the_shell_drains_the_swap_through_its_door() {
    let body = strip_comments(&src("render_loop/mod.rs"));
    assert!(
        body.contains("instance_variant::swap("),
        "o dreno da troca não chama a porta — sem ela não há re-key, nem sepultamento de órfãos"
    );
}
