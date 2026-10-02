//! ⛔⛔ **O `CLAUDE.md` cabe no orçamento — em BYTES, porque a regra em LINHAS já falhou duas vezes.**
//!
//! O `CLAUDE.md` é injetado por inteiro em toda janela, todo subagente e toda worktree, e é relido a
//! **cada passo**: medido nos transcripts de set/2026 (148 mil chamadas), 82 % do custo do Claude é
//! RELER o contexto, a média era 606 mil tokens por passo, e este arquivo era cerca de metade disso
//! (710 KB ⇒ ~330 mil tokens). Acima de 200 mil tokens nenhum subagente de modelo menor conseguia
//! começar: o pedido estourava a janela antes da primeira palavra.
//!
//! ⚠️ **É a SEGUNDA vez.** Em 2026-08-18 ele foi cortado de 917 KB para 41 KB; a regra que devia
//! impedir a volta dizia *«fechar uma linha edita UMA linha do §5»* — e uma «linha» de Markdown não
//! tem tamanho: em seis semanas cada uma virou um parágrafo de 5 a 40 KB (a da Escultura, 338 KB).
//! *Uma regra sem régua é uma nota que envelhece* — este gate é a régua.
//!
//! O que ele exige (história: `docs/archive/estado-2026-10-02/README.md`):
//! - o arquivo inteiro ≤ [`TECTO_DO_ARQUIVO`];
//! - o §5 inteiro ≤ [`TECTO_DO_ROTEADOR`];
//! - cada entrada de módulo do §5.1 é **UMA linha física** de no máximo [`TECTO_DA_ENTRADA`] bytes —
//!   uma continuação recuada seria a mesma narrativa por outra porta.
//!
//! ⛔ A cura de um vermelho é MOVER a narrativa para o handoff da linha (e o link do §5.1 aponta para
//! ele), **nunca** subir um destes números.

use std::path::PathBuf;

/// ⚠️ O recurso é a janela de CONTEXTO, e o número mais apertado é o dos subagentes de modelo menor
/// (200 mil tokens). 40 KB ≈ 18 mil tokens (medido: ~0,46 token por byte neste arquivo, pela
/// diferença do contexto inicial entre 13/09 e 02/10) — menos de 10 % dessa janela, que é o que
/// sobra para o trabalho depois do prompt de sistema, das ferramentas e da memória.
const TECTO_DO_ARQUIVO: usize = 40_000;
/// O §5 em 2026-10-02 mede ~9 KB com 16 módulos; o tecto deixa crescer uns 50 %, não um módulo novo
/// com história dentro.
const TECTO_DO_ROTEADOR: usize = 14_000;
/// O que cabe numa entrada: o que o módulo É, o smoke principal e os links (último handoff, docs,
/// BUGS, história). A maior em 2026-10-02 tem 578 bytes.
const TECTO_DA_ENTRADA: usize = 700;

fn claude_md() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../CLAUDE.md");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("ler {}: {e}", p.display()))
}

/// O texto entre `inicio` (inclusive) e `fim` (exclusive). `None` se um dos dois faltar.
fn trecho<'a>(doc: &'a str, inicio: &str, fim: &str) -> Option<&'a str> {
    let a = doc.find(inicio)?;
    let b = a + doc[a..].find(fim)?;
    Some(&doc[a..b])
}

/// As violações do §5.1: entradas longas demais e linhas que não são entradas (continuações).
fn violacoes_do_5_1(doc: &str) -> (usize, Vec<String>) {
    let Some(sec) = trecho(doc, "### §5.1", "\n## §6") else {
        return (
            0,
            vec!["o §5.1 ou o §6 sumiram — a régua não acha o que medir".into()],
        );
    };
    let mut entradas = 0;
    let mut erros = Vec::new();
    for linha in sec.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        if !linha.starts_with("- ") {
            erros.push(format!(
                "linha que não é entrada (continuação?): «{}»",
                linha.chars().take(70).collect::<String>()
            ));
            continue;
        }
        entradas += 1;
        if linha.len() > TECTO_DA_ENTRADA {
            erros.push(format!(
                "{} bytes (tecto {TECTO_DA_ENTRADA}): «{}»",
                linha.len(),
                linha.chars().take(70).collect::<String>()
            ));
        }
    }
    (entradas, erros)
}

#[test]
fn o_claude_md_cabe_no_orcamento() {
    let doc = claude_md();
    assert!(
        doc.len() <= TECTO_DO_ARQUIVO,
        "o CLAUDE.md tem {} bytes contra o tecto de {TECTO_DO_ARQUIVO}.\n\
         Ele é relido inteiro a CADA passo de todo agente. Cura: mover a narrativa para o handoff da \
         linha (ou para docs/archive/ por scripts/doc-split.py) — nunca subir o tecto.",
        doc.len()
    );
    let roteador = trecho(&doc, "## §5", "\n## §6").expect("o §5 ou o §6 sumiram");
    assert!(
        roteador.len() <= TECTO_DO_ROTEADOR,
        "o §5 tem {} bytes contra o tecto de {TECTO_DO_ROTEADOR}: ele é um ROTEADOR, a narrativa vai \
         para o handoff de cada linha.",
        roteador.len()
    );
}

#[test]
fn cada_modulo_do_5_1_e_uma_linha_curta() {
    let (entradas, erros) = violacoes_do_5_1(&claude_md());
    // Piso de população: uma régua que não achasse nenhuma entrada passaria por vácuo.
    assert!(
        entradas >= 12,
        "a régua achou só {entradas} entradas no §5.1 — o formato mudou e ela passou a medir o nada"
    );
    assert!(
        erros.is_empty(),
        "o §5.1 tem entradas fora do formato (UMA linha, ≤ {TECTO_DA_ENTRADA} bytes):\n  {}\n\
         Cura: a narrativa vai para o handoff; a entrada aponta para ele.",
        erros.join("\n  ")
    );
}

/// Controlo positivo: a régua ACUSA uma entrada longa e uma continuação recuada.
#[test]
fn a_regua_do_5_1_acusa_o_que_existe_para_acusar() {
    let longa = format!("- **X** — {}", "a".repeat(TECTO_DA_ENTRADA));
    let doc = format!("### §5.1 — M\n\n- **Curta** — ok\n{longa}\n  continuação\n\n## §6 — C\n");
    let (entradas, erros) = violacoes_do_5_1(&doc);
    assert_eq!(entradas, 2);
    assert_eq!(erros.len(), 2, "{erros:?}");
}
