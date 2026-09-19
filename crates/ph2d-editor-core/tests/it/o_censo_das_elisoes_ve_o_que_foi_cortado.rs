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
    let (_, medidos) = elisao::medindo(|| fit(&mut ts, texto, 12.0, inteiro * 0.5));
    assert_eq!(medidos.len(), 1, "uma medição, um registo: {medidos:?}");
    assert_eq!(medidos[0].texto, texto);
    assert!(!medidos[0].coube(), "ele foi cortado: {medidos:?}");
    assert!(
        medidos[0].pintado.len() < texto.len() && medidos[0].pintado.ends_with('…'),
        "o registo tem de trazer o que foi PINTADO: {:?}",
        medidos[0].pintado
    );

    // ⭐⭐⭐ E O QUE COUBE TAMBÉM É VISTO — a metade que nasceu no mesmo dia, e sem a qual a
    //    pergunta *«e quando alguém traduzir?»* não tem sujeito: o rótulo que cabe hoje nunca
    //    passa pela lei do corte, logo não deixava rasto nenhum.
    let (_, coube) = elisao::medindo(|| fit(&mut ts, texto, 12.0, inteiro + 1.0));
    assert_eq!(
        coube.len(),
        1,
        "a pergunta «cabe?» é uma medição: {coube:?}"
    );
    assert!(coube[0].coube() && !coube[0].nada(), "{coube:?}");
    assert_eq!(coube[0].pintado, texto, "o que coube sai VERBATIM");
    // ⛔ CONTROLO: e ele NÃO é um corte — a vista estreita continua a separar os dois, que é o
    //    que impede esta wave de transformar o censo numa lista onde tudo se lê igual.
    let cortes: Vec<_> = coube.iter().filter(|m| !m.coube()).collect();
    assert!(
        cortes.is_empty(),
        "o texto coube e apareceu na lista de CORTADOS: {cortes:?}"
    );
}

/// ⭐⭐⭐ **OS DOIS PINTORES PERGUNTAM PELA MESMA PORTA — e este gate nasceu de uma mutação que
/// SOBREVIVEU.**
///
/// ⛔⛔ A lei do corte mora num sítio só (`elide`) e a lei do «cabe?» noutro ([`coube`]), e há
/// **dois** pintores que a fazem: o [`fit_weighted`] e o `paint_text_elided`. Pôr o segundo a
/// comparar à mão — que é como ele estava até 2026-09-18 — deixa a varredura do app **VERDE**: o
/// piso de população dela é GLOBAL, logo perder os registos de **uma** porta não o move.
///
/// ⚠️ *Um piso global mede a soma e é cego a uma parcela.* ⇒ a pergunta tem de ser feita à porta,
/// e é isso que este gate faz: cada pintor, um rótulo que CABE, um registo.
#[test]
fn os_dois_pintores_perguntam_pela_mesma_porta() {
    use ph2d_editor_core::paint::paint_text_elided;
    let mut ts = TextSystem::without_system_fonts();
    let texto = "Depth";
    let largo = ts.prefix_width(texto, 12.0) + 10.0;

    let (_, pelo_fit) = elisao::medindo(|| fit(&mut ts, texto, 12.0, largo));
    assert_eq!(pelo_fit.len(), 1, "o `fit` não perguntou pela porta");
    assert!(pelo_fit[0].coube());

    let (_, pelo_pintor) = elisao::medindo(|| {
        let mut cena = ph2d_vector::VectorScene::new();
        paint_text_elided(
            &mut ts,
            &mut cena,
            texto,
            0.0,
            0.0,
            12.0,
            largo,
            ph2d_editor_core::paint::resolve(
                ph2d_tokens::ColorToken::Text1,
                ph2d_tokens::Theme::default(),
            ),
        );
    });
    assert_eq!(
        pelo_pintor.len(),
        1,
        "o pintor cortado comparou à mão e o censo não ouviu — a varredura do app fica VERDE \
         sobre metade dos rótulos: {pelo_pintor:?}"
    );
    assert!(pelo_pintor[0].coube() && pelo_pintor[0].texto == texto);
}

/// ⭐⭐⭐ **A FONTE E O PESO VIAJAM NO REGISTO — sem eles, a pergunta da próxima língua é um
/// palpite.**
///
/// ⚠️ *Medir numa espessura e pintar noutra* é o defeito que o [`ph2d_editor_core::text_elide`] já
/// pagou duas vezes (os números dos cartões do Motion a saírem `0....`). Um gate que re-meça a
/// palavra deformada tem de a medir **na fonte e no peso em que a pergunta original foi feita**,
/// senão ele afirma sobre um rótulo que ninguém pinta.
#[test]
fn o_registo_diz_em_que_fonte_e_peso_a_pergunta_foi_feita() {
    use ph2d_editor_core::text_elide::fit_weighted;
    use ph2d_text::FontWeight;
    let mut ts = TextSystem::without_system_fonts();
    let (_, medidos) =
        elisao::medindo(|| fit_weighted(&mut ts, "Depth", 11.0, 1000.0, FontWeight::SEMI_BOLD));
    assert_eq!(medidos.len(), 1);
    assert!((medidos[0].fonte - 11.0).abs() < 1e-6, "{medidos:?}");
    assert_eq!(medidos[0].peso, FontWeight::SEMI_BOLD, "{medidos:?}");
    assert!((medidos[0].largura - 1000.0).abs() < 1e-6, "{medidos:?}");
}

/// ⭐⭐⭐ **O CENSO É DA THREAD QUE O ARMOU — e a 1.ª redacção tinha a bandeira GLOBAL.**
///
/// ⛔⛔ **Medido em 2026-09-18, um dia depois de o censo nascer:** a bandeira era um `AtomicBool`
/// estático e o armazém um `thread_local`. Sob `cargo test` — que corre os testes em **threads do
/// mesmo processo** — o `desarma` de um gate apanhava o vizinho entre o `arma` dele e a pintura, e
/// o vizinho lia **zero cortes** sobre um corte que aconteceu. *Zero é a cara da aprovação.*
///
/// ⚠️⚠️ **O `nextest` não a podia mostrar**, porque ele dá um PROCESSO a cada teste e ali não há
/// vizinho nenhum — e é com ele que os portões deste repo correm. ⇒ *um instrumento mede-se na
/// ferramenta mais fraca que o corre*, senão ele é correcto só no sítio onde ninguém olha.
///
/// A régua é a única que exprime o defeito: **duas threads**, uma a medir e a outra a armar e
/// desarmar em ciclo. Com a bandeira global, a primeira perde registos.
#[test]
fn o_censo_de_uma_thread_nao_e_desarmado_pela_vizinha() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let pare = Arc::new(AtomicBool::new(false));
    let vizinha = {
        let pare = Arc::clone(&pare);
        std::thread::spawn(move || {
            // A vizinha faz exactamente o que outro gate faz: arma, mede e desarma, em ciclo.
            let mut ts = TextSystem::without_system_fonts();
            while !pare.load(Ordering::Relaxed) {
                let _ = elisao::medindo(|| fit(&mut ts, "Return", 12.0, 4.0));
            }
        })
    };

    let mut ts = TextSystem::without_system_fonts();
    let mut perdidos = 0;
    for _ in 0..2_000 {
        let (_, medidos) = elisao::medindo(|| fit(&mut ts, "Translate Y  #4591", 12.0, 30.0));
        if medidos.len() != 1 {
            perdidos += 1;
        }
    }
    pare.store(true, Ordering::Relaxed);
    vizinha.join().expect("a vizinha morreu");

    assert_eq!(
        perdidos, 0,
        "{perdidos} de 2000 medições desapareceram porque a thread vizinha desarmou o censo — a \
         bandeira voltou a ser global"
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
