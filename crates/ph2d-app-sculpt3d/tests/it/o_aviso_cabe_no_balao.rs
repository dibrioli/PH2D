//! ⭐⭐⭐⭐ **O AVISO DA SAÍDA CABE NO BALÃO** — medido pela régua do PRODUTO, no PIOR caso.
//!
//! # ⛔⛔⛔ O report que o fez existir
//!
//! Dono, 22/09, sobre o smoke da §20: ***«as mensagens estão cortadas com … não consigo ler
//! tudo»***.
//!
//! O balão de aviso vive numa coluna de largura FIXA, e o que sobra para o texto depois da faixa
//! de severidade, do ícone e dos dois recuos é **`300 px` ≈ 48 caracteres**
//! ([`ph2d_editor_core::toast::text_budget_px`]). A frase que a wave anterior escrevia media
//! **`95`**.
//!
//! ⚠️⚠️ **E a de ANTES dessa wave media `60` — ela nunca foi legível.** Isto é pré-existente, e
//! não uma dívida que a cláusula da tinta fina criou: *o aviso do que um formato não carrega
//! nunca chegou a ser lido por ninguém.*
//!
//! # ⭐⭐ A lei da partição, e ela é medida
//!
//! ***A metade que TEM de ser lida não pode ter parte variável.***
//!
//! Uma frase única cabe com `teste.obj` (`48`) e **estoura com um nome de ficheiro real**
//! (`retrato-da-personagem-v3.obj` ⇒ `62`). E a elisão corta o **FIM** — que é exactamente onde o
//! aviso está. ⇒ a saída escreve **dois** balões: a confirmação leva o nome (pode elidir, e o
//! artista acabou de o escrever) e o aviso fica **sem uma única parte variável**.
//!
//! ⇒ é por isso que este gate mede **só o aviso**: ele é o único dos dois cujo comprimento não
//! depende do que o artista escreveu, logo é o único sobre o qual se pode afirmar alguma coisa.
//!
//! # ⚠️ Porque a barra é o PIOR CASO e não o caso comum
//!
//! O `.obj` guarda cor e peças, logo o aviso dele é curto (`Lost: mask, fine paint`). Quem
//! estoura é o **STL**, que perde tudo — e uma barra medida no caso comum deixaria passar
//! exactamente a linha que o dono não consegue ler.

use ph2d_mesh::MeshFormat;

#[test]
fn o_aviso_de_cada_formato_cabe_no_balao() {
    let mut text = ph2d_text::TextSystem::without_system_fonts();
    let orcamento = ph2d_editor_core::toast::text_budget_px();
    let corpo = ph2d_tokens::TypeToken::Base.px();

    // ⚠️ **O CONTROLO vem primeiro:** se o orçamento viesse a zero (uma porta partida, um token
    //   que mudou de nome), tudo seria elidido e o gate reprovaria por um motivo que não é o
    //   dele. Uma coluna de `360 px` não pode deixar menos de `200` para o texto.
    assert!(
        orcamento > 200.0,
        "o orçamento do balão leu {orcamento:.1} px — a porta está partida, e este gate estava \
         prestes a reprovar por uma razão que não é a dele"
    );

    let mut mau = Vec::new();
    let mut vistos = 0usize;
    for fmt in MeshFormat::ALL {
        // As DUAS colunas: com e sem tinta fina. A com é a longa, e é a da wave.
        for com_tinta in [false, true] {
            vistos += 1;
            let frase = ph2d_mesh::lost_by(fmt, com_tinta);
            let vista = ph2d_editor_core::text_elide::fit(&mut text, &frase, corpo, orcamento);
            if vista != frase {
                mau.push(format!(
                    "{:?} (tinta fina: {com_tinta}) — {} caracteres, lê-se «{vista}»",
                    fmt,
                    frase.chars().count()
                ));
            }
        }
    }

    // ⚠️ Piso de população: `MeshFormat::ALL` a encolher para zero deixaria `mau` vazio e este
    //   gate verde a medir nada — a mesma cegueira que os censos de directório desta casa já
    //   pagaram.
    assert!(
        vistos >= 6,
        "este gate mediu {vistos} combinações e esperava pelo menos 6 (3 formatos × 2 colunas) — \
         ele perdeu o sujeito"
    );
    assert!(
        mau.is_empty(),
        "o aviso da exportação NÃO cabe no balão ({orcamento:.0} px) e o artista lê-o cortado \
         com «…» — report do dono, 2026-09-22:\n  {}",
        mau.join("\n  ")
    );
}
