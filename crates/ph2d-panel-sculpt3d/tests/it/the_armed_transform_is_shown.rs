//! **O ARM tem de ser VISÍVEL** — arch-gate sobre o fonte do pintor.
//!
//! ⚠️ **Por que um arch-gate e não um seam:** o índice aceso de um segmentado é
//! passado ao widget no `paint` e DESENHADO; ele não pousa no `WidgetStore`, e
//! não há porta que responda *"qual segmento está aceso?"*. Um gate de unidade
//! aqui não poderia falhar pelo motivo que alega — e este repo tem a cicatriz de
//! gates assim, verdes por construção.
//!
//! A propriedade que se pode afirmar, e que é exatamente a mutação a temer: o
//! grupo do transform recebe um índice **derivado de `snap.transform`**, e não o
//! `usize::MAX` que as quatro operações de máscara (gestos, nenhuma acesa)
//! recebem de propósito. Sem isto o único estado em que o botão esquerdo deixa
//! de esculpir seria **invisível**.
//!
//! ⚠️ **Ele varre a FAMÍLIA `src/paint/*.rs`, e não um arquivo.** A primeira
//! versão lia o `body.rs`; o pintor mudou-se para o irmão `mask_tools.rs` no
//! mesmo dia, e o gate reprovou — **o controle positivo funcionando**, em vez de
//! varrer vazio e passar sobre um produto correto. *Afirme a PROPRIEDADE, nunca
//! o endereço.*

/// O fonte de todo pintor do painel, concatenado.
///
/// ⚠️ **ORDENADO, e não na ordem do `read_dir`** — ela é *unspecified* e varia
/// com o sistema de arquivos, então a janela de 600 bytes que o gate abre em
/// volta da chamada cairia sobre vizinhos DIFERENTES em máquinas diferentes.
/// Um gate cuja fixture depende da ordem em que o SO devolve entradas é um gate
/// que passa aqui e reprova no CI sem ninguém ter tocado no produto.
fn painters() -> String {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/paint");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .expect("a pasta do pintor existe")
        .map(|e| e.expect("entrada legivel").path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    files.sort();
    let mut all = String::new();
    for p in files {
        all.push_str(&std::fs::read_to_string(&p).expect("fonte legivel"));
    }
    // Controle positivo: sem isto uma pasta renomeada deixaria a varredura vazia
    // e o gate passaria sobre nada.
    assert!(
        all.len() > 10_000,
        "a varredura do pintor voltou quase vazia ({} bytes) -- reaponte este gate",
        all.len()
    );
    all
}

/// **Recua até uma fronteira de CARACTERE.**
///
/// ⚠️ **Sem isto o gate PANICA sobre produto correto.** O `at` vem do `find` e é
/// fronteira; o `at − 600` **não é** — e o fonte deste painel é prosa portuguesa
/// com acento e `⚠️`, onde um índice cru cai dentro de um `ç` com a mesma
/// facilidade com que cai entre dois. Medido: bastou um doc-comment de outro
/// arquivo da MESMA pasta deslocar a concatenação para o gate morrer com *"end
/// byte index 25297 is not a char boundary"* — uma falha que **não é a que ele
/// alega**, sobre um pintor que nunca foi tocado.
///
/// ⚠️ E ele PANICA em vez de reprovar, que é o pior dos dois: um vermelho manda
/// ler a asserção, um pânico manda desconfiar do produto.
fn floor_boundary(s: &str, mut i: usize) -> usize {
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// A irmã, para o fim da janela: sobe até a próxima fronteira.
fn ceil_boundary(s: &str, mut i: usize) -> usize {
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i.min(s.len())
}

#[test]
fn the_transform_group_is_lit_from_the_snapshot() {
    let src = painters();
    let at = src
        .find("&ids::SCULPT3D_TRANSFORM")
        .expect("nenhum pintor passa o grupo do transform -- o arm ficou sem chips");

    // A janela ANTES da chamada: é onde o `selected` do grupo é computado.
    let before = &src[floor_boundary(&src, at.saturating_sub(600))..at];
    assert!(
        before.contains("snap.transform") || before.contains("snap\n        .transform"),
        "o indice aceso do transform nao vem do retrato -- o arm ficou invisivel"
    );

    // E a mutação a temer, nomeada: o grupo NÃO pode receber o `usize::MAX` das
    // operações de máscara, que é como se diz *"nenhum aceso"*.
    let after = &src[at..ceil_boundary(&src, (at + 400).min(src.len()))];
    let head = after.split(");").next().unwrap_or(after);
    assert!(
        !head.contains("usize::MAX"),
        "o grupo do transform recebeu `usize::MAX` -- nenhum chip acende, e o arm some da tela"
    );
}
