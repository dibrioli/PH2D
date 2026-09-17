//! ⭐⭐⭐ **UMA RECUSA DA RETOPOLOGIA CHEGA AO ECRÃ** — e não só ao terminal.
//!
//! ⛔⛔ **O defeito que este gate fecha:** o doc de [`RemeshRefusal::explain`] chama à frase dele
//! *«a FRASE que o artista lê»*, e os **três** sítios que a mostravam eram `eprintln!`. O botão
//! `Quad Retopology` recusava com a cura escrita dentro da frase (*FLATTEN the stack first*, *lower
//! the Detail*) e no ecrã não acontecia **nada** — indistinguível de um botão partido (§5.0: *uma
//! recusa muda*), porque o terminal não é uma superfície do produto: o dono corre o smoke a partir
//! dele, o artista não.
//!
//! ⇒ a cena ganhou uma CAIXA DE SAÍDA (`Sculpt3dScene::fala` / `take_avisos`) e a shell drena-a
//! para a fila de avisos uma vez por quadro. Este gate mede a metade de cá: **nenhuma recusa se
//! explica sem passar pela porta**. A metade de lá (o dreno) é do gate da shell.

/// O `src/` da crate, **sem** os `_tests.rs`: um gate que afirma AUSÊNCIA e lê os próprios testes
/// mede a palavra que eles escrevem de propósito.
fn fonte() -> Vec<(String, String)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut pilha = vec![dir];
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).expect("o `src/` da crate") {
            let p = e.expect("uma entrada").path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let nome = p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if !nome.ends_with("_tests.rs") {
                    out.push((nome, std::fs::read_to_string(&p).expect("ler")));
                }
            }
        }
    }
    assert!(
        out.len() > 50,
        "a varredura leu {} ficheiros — ela ficou cega",
        out.len()
    );
    out
}

#[test]
fn every_refusal_the_sculpture_explains_goes_through_the_outbox() {
    let mut fora_da_porta = Vec::new();
    for (nome, src) in fonte() {
        for (n, linha) in src.lines().enumerate() {
            if linha.contains(".explain()") && !linha.contains("fala(") {
                fora_da_porta.push(format!("{nome}:{} · {}", n + 1, linha.trim()));
            }
        }
    }
    assert!(
        fora_da_porta.is_empty(),
        "estas recusas explicam-se FORA da caixa de saída:\n  {}\n\n\
         A cura é `scene.fala(e.explain())` (ou `self.fala(…)`): ela escreve no terminal E deixa o \
         aviso para a shell drenar. Um `eprintln!` sozinho devolve ao artista um botão que não faz \
         nada e não diz porquê.",
        fora_da_porta.join("\n  ")
    );
}

/// ⚠️ **A porta é UMA** — duas funções a falar pela cena voltariam a deixar uma delas só no log.
#[test]
fn the_outbox_has_exactly_one_door() {
    let n: usize = fonte()
        .iter()
        .map(|(_, s)| s.matches("fn fala(").count())
        .sum();
    assert_eq!(
        n, 1,
        "a cena declara {n} portas de fala — ver `cena.rs::fala`, que é a única"
    );
}

/// ⭐⭐ **A METADE DE LÁ: a shell DRENA a caixa de saída.** Sem ela a cena fala para dentro de um
/// `Vec` que ninguém lê — o mesmo silêncio de antes, com mais código.
///
/// ⚠️⚠️ **Este gate lê a shell a partir de outra crate, e isso tem um modo de falha conhecido**
/// (`feedback_a_gate_that_reads_the_shell_from_another_crate…`): a linha que MOVE o código de sítio
/// não o vê. A cura é o `include_str!` — um ficheiro que mude de nome **não compila** aqui, em vez
/// de emudecer. ⛔ E ele mora nesta crate, e não na shell, porque a catraca
/// `architecture_the_shell_only_shrinks` tem hoje **12 linhas** de folga: um ficheiro de teste novo
/// lá dentro reprovaria a árvore.
#[test]
fn the_shell_drains_what_the_sculpture_has_to_say() {
    const FASE: &str =
        include_str!("../../../../shells/desktop/src/render_loop/fase_world_panel_bridges.rs");
    assert!(
        FASE.contains("take_avisos()"),
        "a fase dos painéis de mundo deixou de drenar `Sculpt3dScene::take_avisos` — as recusas da \
         retopologia voltaram a morrer no terminal"
    );
    assert!(
        FASE.contains("toasts.push"),
        "o que a escultura diz tem de entrar na FILA DE AVISOS — drenar para outro sítio é falar \
         para dentro de um `Vec`"
    );
}
