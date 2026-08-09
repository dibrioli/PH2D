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
fn painters() -> String {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/paint");
    let mut all = String::new();
    for e in std::fs::read_dir(dir).expect("a pasta do pintor existe") {
        let p = e.expect("entrada legivel").path();
        if p.extension().is_some_and(|x| x == "rs") {
            all.push_str(&std::fs::read_to_string(&p).expect("fonte legivel"));
        }
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

#[test]
fn the_transform_group_is_lit_from_the_snapshot() {
    let src = painters();
    let at = src
        .find("&ids::SCULPT3D_TRANSFORM")
        .expect("nenhum pintor passa o grupo do transform -- o arm ficou sem chips");

    // A janela ANTES da chamada: é onde o `selected` do grupo é computado.
    let before = &src[at.saturating_sub(600)..at];
    assert!(
        before.contains("snap.transform") || before.contains("snap\n        .transform"),
        "o indice aceso do transform nao vem do retrato -- o arm ficou invisivel"
    );

    // E a mutação a temer, nomeada: o grupo NÃO pode receber o `usize::MAX` das
    // operações de máscara, que é como se diz *"nenhum aceso"*.
    let after = &src[at..(at + 400).min(src.len())];
    let head = after.split(");").next().unwrap_or(after);
    assert!(
        !head.contains("usize::MAX"),
        "o grupo do transform recebeu `usize::MAX` -- nenhum chip acende, e o arm some da tela"
    );
}
