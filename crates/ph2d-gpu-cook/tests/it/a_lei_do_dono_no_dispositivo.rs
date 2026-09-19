//! ⭐⭐⭐ **A LEI DO DONO NA ROTA QUE O PRODUTO DE FACTO CORRE.**
//!
//! Ordem do dono, 2026-09-19: *«o grid continua desenhando quadrados. A ordem foi não desenhar
//! nada»*. A lei vive em duas casas — a CPU ([`ph2d_eval_motion::lower`]) e o **dispositivo** —, e
//! o cozimento deste módulo é **GPU-resident por omissão** ⇒ *é esta a rota que o artista vê*.
//!
//! ⛔⛔ **E até 2026-09-19 ela não existia aqui, com a rota da CPU inteiramente gateada:** o
//! `lower_module` nunca leu o `so_com_forma` e o `lower_signature` nem sequer o comia. A prova de
//! que este ficheiro faltava é directa — **duas mutações sobreviveram** a apagar os dois braços da
//! decisão, porque `so_com_forma` aparecia nesta crate num ficheiro de produto e em **zero**
//! testes. *Uma lei escrita só numa rota é uma lei que o produto não tem.*
//!
//! ⚠️ **Sem `#[ignore]` e sem adapter, de propósito:** a decisão não tem um pixel dentro, e a
//! porta [`ph2d_gpu_cook::instances::veredito_do_dispositivo`] existe exactamente para que este
//! gate possa correr no CI. Um gate que pedisse um `GpuContext` para medir isto nunca lá correria.

use ph2d_gpu_cook::instances::veredito_do_dispositivo as veredito;
use ph2d_render::SinkStyle;

fn com_a_lei() -> SinkStyle {
    SinkStyle {
        so_com_forma: true,
        ..SinkStyle::PLAIN
    }
}

/// ⭐⭐⭐ **As QUATRO células, e as três que dizem «desenha» valem mais que a que corta.**
///
/// Sem a célula da GEOMETRIA VIVA, a lei apagaria **toda forma** que o grafo produz na placa;
/// sem a do LADRILHO, o report do dono reabre; e sem a da lei DESLIGADA, o
/// `PH2D_MOTION_SO_COM_FORMA=0` deixaria de bissectar coisa nenhuma.
#[test]
fn o_veredito_corta_so_onde_a_lei_manda() {
    // (1) Lei ligada, sem ladrilho e sem geometria: é a posição nua ⇒ **não desenha**.
    let (desenha, st) = veredito(com_a_lei(), false, false);
    assert!(
        !desenha,
        "uma corrente sem aparencia nenhuma nao pode produzir instancia: e' o report do dono"
    );
    assert!(st.so_com_forma, "e a lei continua ligada no estilo assado");

    // (2) Lei ligada, COM ladrilho: quem trouxe um átlas desenha com ele.
    let (desenha, _) = veredito(com_a_lei(), false, true);
    assert!(desenha, "quem tem `uv_rect` desenha como sempre desenhou");

    // (3) Lei ligada, COM geometria viva: a lei DESLIGA-SE — e isto é a divergência declarada
    //     contra a CPU, que pergunta pelo VALOR (`geometry_id > 0`) e não pela presença.
    let (desenha, st) = veredito(com_a_lei(), true, false);
    assert!(
        desenha,
        "uma forma viva desenha-se mesmo sem ladrilho: cair para o lado conservador e' a lei"
    );
    assert!(
        !st.so_com_forma,
        "e o estilo ASSADO tem de a trazer desligada, senao a assinatura do pipeline mente"
    );

    // (4) Lei DESLIGADA: nada corta, nem sem aparência nenhuma.
    let (desenha, st) = veredito(SinkStyle::PLAIN, false, false);
    assert!(
        desenha,
        "`PH2D_MOTION_SO_COM_FORMA=0` tem de devolver o quadro de antes"
    );
    assert!(!st.so_com_forma);
}

/// ⛔⛔ **A LEI NÃO ENTRA NA ASSINATURA DO PIPELINE — e é isso que a OBRIGA a cortar antes.**
///
/// ⚠️ **A primeira redacção deste gate afirmava o CONTRÁRIO e reprovou sobre produto correcto.**
/// Medido: o [`lower_signature`] come `blend`, `pivot`, `sampling` e `stream_order`, e **não** o
/// `so_com_forma` — o `is_plain` chega a neutralizá-lo por escrito. *A lei é um CORTE binário
/// antes do despacho, não uma variante de shader*, e as duas leituras não são estilo: são
/// desenhos diferentes do produto.
///
/// ⭐ E a propriedade é load-bearing nos dois sentidos: se ela entrasse na assinatura **sem**
/// cortar, cada sink com a lei ligada assava uma pipeline própria (uma entrada de cache por
/// estilo) **e** os quadrados do report continuavam a desenhar-se. Ter as duas metades escritas
/// aqui é o que impede alguém de «resolver» isto ensinando o hash a comer o campo.
#[test]
fn a_lei_corta_antes_porque_a_assinatura_nao_a_ve() {
    let com_ladrilho = [false, false, false, false, true, false, false, false];

    // (1) A assinatura é CEGA à lei — com as mesmas colunas, ligada e desligada dão o mesmo hash.
    let (desenha, st) = veredito(com_a_lei(), false, true);
    assert!(desenha);
    assert_eq!(
        ph2d_gpu_cook::lower::lower_signature(com_ladrilho, st),
        ph2d_gpu_cook::lower::lower_signature(com_ladrilho, SinkStyle::PLAIN),
        "a lei nao pode multiplicar as pipelines em cache: ela nao e' uma variante de shader"
    );

    // (2) ⇒ logo a ÚNICA coisa que separa os dois desenhos é o corte, e ele tem de existir.
    //     Sem esta metade, (1) sozinha leria-se como «a lei não faz nada», que é o report.
    let (desenha_nu, _) = veredito(com_a_lei(), false, false);
    assert!(
        !desenha_nu,
        "se a assinatura e' cega a' lei, o corte e' o UNICO sitio onde ela pode acontecer"
    );
}
