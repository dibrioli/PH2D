//! **A LINHA DA AQUARELA TEM DE TRAZER A JANELA QUE ELA DIVIDE — E NÃO PODE EMUDECER.**
//!
//! ⚠️ **Este gate nasceu de duas cicatrizes deste repo, e a segunda é a cara:**
//!
//! 1. *Um custo sem divisor não é atribuível* — a lição do `stamps: media` (o
//!    gate irmão): `composite media 23ms` admite *a janela cresceu* e *o custo
//!    por texel subiu*, que pedem curas OPOSTAS. Daí o `ns/texel` ao lado.
//! 2. *Um instrumento MUDO lê-se como resultado* — quando a sim da água saiu da
//!    thread do frame, ninguém mais chamava o `note_step` e o log passou a
//!    imprimir `sim media 0.00ms x0`, que se lê como *"a simulação não custa
//!    nada"* e significava *"ninguém mede a simulação"* (doc 28 §5.40). Um
//!    zero honesto e um zero por desconexão são indistinguíveis na tela.
//!
//! Arch-gate sobre o fonte porque o `[frame]` só existe com janela — nenhum
//! teste de unidade alcança aquele `eprintln`.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// O `ns/texel` é impresso ao lado do tempo, com a janela que o produz.
///
/// Mutação que sangra: tirar `ns_per_texel` do formato — a linha volta a dizer
/// só a média, e um smoke não distingue *o pincel cresceu* de *a máquina está
/// disputada*.
#[test]
fn the_watercolor_line_prints_the_window_it_divides_by() {
    let line = SRC
        .split("aquarela: composite media")
        .nth(1)
        .expect("o `[frame]` imprime a linha da aquarela");
    // ⚠️ A janela é a INSTRUÇÃO, não um número de bytes: um arch-gate ancorado em distância é um
    // proxy que expira no dia em que alguém acrescenta um argumento (a cicatriz que a `line/Vector`
    // pagou em dois gates de shell). Aqui ela termina onde o `eprintln!` termina.
    let head = line
        .split("\n                }")
        .next()
        .expect("o `eprintln!` da aquarela fecha");
    for needle in [
        "ns/texel",
        "wash.window_px_per_composite",
        "wash.ns_per_texel",
    ] {
        assert!(
            head.contains(needle),
            "a linha da aquarela perdeu `{needle}` — um custo sem o divisor dele não é atribuível, \
             e as duas leituras de `composite media` pedem curas opostas"
        );
    }
}

/// **As cinco fases são de fato ALIMENTADAS pelo produto.**
///
/// O log pode imprimir zeros para sempre se ninguém chamar os `note_*`; foi
/// exatamente assim que a linha da água ficou muda por uma wave inteira. Este
/// gate afirma a FIAÇÃO, não o formato.
///
/// ⚠️ **Ele varre a ÁRVORE, e não uma lista de arquivos** — a 1ª versão trazia
/// os três arquivos à mão e ficou VERMELHA no mesmo dia, quando o teto de LOC
/// empurrou o envelope do composite para um filho novo. *Um gate por-arquivo
/// protege o arquivo que alguém lembrou de listar*, e o sítio que se move é
/// justamente o que ninguém relista.
#[test]
fn every_watercolor_phase_is_fed_by_the_product() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/ph2d-tool-painter/src");
    let mut all = String::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = e.expect("entrada legível").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                all.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
            }
        }
    }
    // Controle positivo: a varredura de fato leu o crate (um root errado daria vazio, e o gate
    // reprovaria TUDO por um motivo que não é o dele).
    assert!(
        all.contains("pub fn note_composite"),
        "a varredura não achou o próprio `wash_diag` — o root está errado, não a fiação"
    );
    for phase in [
        "note_composite",
        "note_stamp",
        "note_pour",
        "note_dry",
        "note_pendown",
    ] {
        assert!(
            all.contains(&format!("wash_diag::{phase}")),
            "nenhum sítio do produto chama `wash_diag::{phase}` — a fase vai imprimir 0.00ms x0 para \
             sempre, e um zero por DESCONEXÃO é indistinguível de um zero honesto na tela"
        );
    }
}
