//! ⭐⭐⭐ **O CENSO DAS ELISÕES — e o CONTROLO POSITIVO sem o qual ele mede silêncio.**
//!
//! ⛔⛔ **Ele nasce de um defeito que a foto do dono mostrava e nenhum instrumento via** (18/09): a
//! coluna dos nomes do Audio Mixer era um literal e `Depth`/`Return` saíam cortados **na língua em
//! que o app ship**. O painel tinha censo de texto verde, gate de costura e gate de ids — e nenhum
//! deles pergunta ***o que foi pintado coube?***
//!
//! ⚠️⚠️ **E a 1.ª redacção do censo LIA ZERO SOBRE O DEFEITO VIVO.** Eu liguei-o ao
//! `paint_text_elided`, e o corte acontece em **quatro** entradas
//! ([`ph2d_editor_core::text_elide`]: `fit` · `fit_weighted` · o pintor cortado · o pintor
//! centrado). *Um censo ligado a um dos caminhos lê zero e parece aprovação* — a forma exacta que
//! esta casa já pagou com o censo por prefixo de nome e com a vassoura do `.gz`. ⇒ o registo mora
//! na LEI (`elide`), que é por onde todos passam, e este ficheiro é a prova de que ele a vê.

use ph2d_editor_core::text_elide::{elisao, fit};
use ph2d_text::TextSystem;

/// ⭐ **ARMADO, ele vê. DESARMADO, não custa nada.** As duas metades: sem a segunda, um censo
/// sempre ligado alocaria uma `String` por corte **por quadro** — o vazamento que o `leak_key` do
/// `ph2d-i18n` já custou a esta casa.
#[test]
fn o_censo_ve_o_corte_quando_armado_e_e_mudo_quando_nao() {
    let mut ts = TextSystem::without_system_fonts();
    let texto = "Translate Y  #4591";
    let inteiro = ts.prefix_width(texto, 12.0);

    // DESARMADO: corta e não regista.
    let _ = fit(&mut ts, texto, 12.0, inteiro * 0.5);
    assert!(
        elisao::cortados().is_empty(),
        "o censo registou com o produto DESARMADO — o caminho do produto está a pagar"
    );

    // ARMADO: vê o corte, com o que se queria e o que saiu.
    let (_, cortados) = elisao::medindo(|| fit(&mut ts, texto, 12.0, inteiro * 0.5));
    assert_eq!(cortados.len(), 1, "um corte, um registo: {cortados:?}");
    assert_eq!(cortados[0].texto, texto);
    assert!(
        cortados[0].pintado.len() < texto.len() && cortados[0].pintado.ends_with('…'),
        "o registo tem de trazer o que foi PINTADO: {:?}",
        cortados[0].pintado
    );

    // ⛔ CONTROLO NEGATIVO: o que CABE não é um corte. Sem esta metade, um censo que registasse
    //    tudo devolveria a lista cheia e ninguém a leria.
    let (_, nenhum) = elisao::medindo(|| fit(&mut ts, texto, 12.0, inteiro + 1.0));
    assert!(
        nenhum.is_empty(),
        "o texto coube e o censo registou-o na mesma: {nenhum:?}"
    );
}

/// ⭐⭐⭐ **O CORTE PARA NADA é o pior dos dois, e entra pela mesma porta.**
///
/// ⛔⛔ Medido no Audio Mixer no dia em que este censo nasceu: o botão de silenciar pinta `"M"`
/// num espaço de `9,0 px` e a letra precisa de `10,1` ⇒ **nem a reticência cabe, e o botão sai
/// VAZIO**. Era isso que a foto do dono mostrava como `…` entre dois botões com letra.
///
/// ⚠️ *Um controlo sem legenda e um controlo morto dão o MESMO report* — e é por isso que o corte
/// para nada tem de aparecer no censo em vez de sair pelo `return` silencioso que a lei já tinha.
#[test]
fn um_corte_para_nada_tambem_e_registado() {
    let mut ts = TextSystem::without_system_fonts();
    let (_, cortados) = elisao::medindo(|| fit(&mut ts, "M", 12.0, 0.5));
    assert_eq!(
        cortados.len(),
        1,
        "o corte para nada tem de ser visível ao censo: {cortados:?}"
    );
    assert_eq!(
        cortados[0].pintado, "",
        "e ele diz que o que saiu foi NADA: {:?}",
        cortados[0].pintado
    );
}

/// ⭐⭐⭐ **O RESPIRO TEM IDA E VOLTA, e as duas são uma lei só.**
///
/// ⛔⛔ **Ela é pública desde 2026-09-18 por causa de um defeito MEU:** o Audio Mixer passou a
/// MEDIR a coluna dos nomes e continuou a cortar, porque a coluna media o **rect** e o pintor
/// centrado gasta o rect **menos o respiro das duas bordas** — `Return` pede `35,9 px`, a coluna
/// dava `35,9` e o orçamento era `19,9`. ⚠️ *E o gate que a conferia media a MESMA grandeza
/// errada, logo passava sobre nomes que continuavam cortados no ecrã.*
///
/// ⇒ quem dimensiona pergunta pela [`rect_for_label`] e quem confere pela [`label_budget`]; esta
/// prova é a que impede as duas de divergirem.
#[test]
fn quem_dimensiona_e_quem_confere_falam_a_mesma_lei() {
    use ph2d_editor_core::paint::{label_budget, rect_for_label};
    for texto_w in [1.0_f32, 6.1, 19.9, 35.9, 55.1, 240.0] {
        let rect = rect_for_label(texto_w);
        assert!(
            label_budget(rect) >= texto_w,
            "um texto de {texto_w} px pediu uma caixa de {rect} e o orçamento dela é {} — a ida e \
             a volta do respiro divergiram",
            label_budget(rect)
        );
    }
    // ⛔ CONTROLO: o respiro EXISTE. Sem esta metade, um `rect_for_label` que devolvesse o próprio
    //    texto passaria a ida-e-volta acima e o defeito voltava inteiro.
    assert!(
        rect_for_label(20.0) > 20.0,
        "a caixa tem de ser MAIOR que o texto — é isso que o respiro é"
    );
}

/// ⭐⭐⭐ **O RESPIRO NUNCA COME MAIS DE METADE DA CAIXA — e a fronteira é DERIVADA.**
///
/// ⛔⛔ Ele era uma subtracção CONSTANTE, logo numa caixa pequena tomava-a quase toda: o botão de
/// silenciar de um strip do mixer mede `25,0 px`, o respiro levava `16,0` e sobravam `9,0` para
/// uma letra `M` que precisa de `10,1` ⇒ **o botão saía VAZIO**. Medido pelo censo de elisões:
/// `4` botões a pintar nada num só painel.
///
/// ⚠️ **A fronteira não é escolhida:** as duas leis cruzam-se onde `w − respiro = w/2`, isto é em
/// `2 × respiro` = **32 px**. ⇒ acima disso **nada muda** (toda caixa normal deste app continua
/// byte a byte como estava) e abaixo o respiro passa a ser proporcional. *É uma melhoria estrita:
/// o que cabia continua a caber.*
#[test]
fn o_respiro_nunca_come_mais_de_metade_da_caixa() {
    use ph2d_editor_core::paint::label_budget;
    let respiro = ph2d_tokens::Spacing::Md.px() * 2.0;
    let fronteira = respiro * 2.0;

    // 1. ACIMA da fronteira: a lei de sempre, ao bit.
    for w in [fronteira + 0.5, 50.0, 100.0, 304.0] {
        assert!(
            (label_budget(w) - (w - respiro)).abs() < 1e-3,
            "a caixa de {w} px mudou de orçamento acima da fronteira — isto tinha de ser inerte"
        );
    }
    // 2. ABAIXO: o respiro deixa de engolir a palavra.
    for w in [10.0, 20.0, 25.0, fronteira - 0.5] {
        assert!(
            label_budget(w) >= w * 0.5,
            "a caixa de {w} px ficou com {} px de orçamento — o respiro comeu mais de metade",
            label_budget(w)
        );
    }
    // 3. ⛔ O CASO MEDIDO: a letra `M` do botão de silenciar cabe na caixa que ele tem.
    let mut ts = ph2d_text::TextSystem::new();
    let m = ts.prefix_width("M", ph2d_tokens::TypeToken::Xs.px());
    assert!(
        label_budget(25.0) >= m,
        "a letra `M` mede {m:.1} px e o botão de 25,0 px dá {:.1} — ele volta a pintar NADA",
        label_budget(25.0)
    );
}
