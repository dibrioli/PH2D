//! Os gates da costura — ver [`super`].

use super::takes_the_frame;

/// ⭐⭐⭐ **AS DUAS CONDIÇÕES, e cada uma sozinha basta para o quadro ficar na CPU.**
///
/// # ⚠️⚠️ Eram TRÊS, e as duas que saíram eram DEFEITOS
///
/// **A do quadro ASSENTE** saiu em 2026-09-15 e era a causa directa do report do dono — *«e apagar
/// o AO ao rotacionar a tela»*: o sombreado de contacto só existe no caminho do dispositivo, logo
/// enquanto ela existiu ele desaparecia a cada gesto. O que a substitui não é uma cerca, é a lei da
/// W73 a viajar com o pedido (o `antialias` chega ao
/// [`ph2d_field_gpu::trace::MarchSetup`] e o dispositivo **salta o segundo despacho**).
///
/// **A da ESCULTURA** saiu no mesmo dia, e era uma cerca legítima que deixou de ter sujeito: a
/// escultura **atravessa** desde a wave da grade ([`ph2d_field_gpu::sculpt`]). O que sobra da
/// pergunta é mais estreita e verdadeira — *a folha amostrada sabe entregar a grade?*
///
/// ⚠️ *Uma cerca que perde o sujeito continua a recusar trabalho que já se sabe fazer.*
#[test]
fn o_dispositivo_so_toma_uma_peca_que_ele_sabe_desenhar() {
    let limpa = crate::smoke::scene(1);
    let com_escultura = crate::smoke::scene(6);
    let reg = crate::smoke::sampled_registry();
    let vazio = ph2d_field_eval::hybrid::Registry::new();

    // (1) sem adaptador não há dispositivo.
    assert!(!takes_the_frame(None, &limpa, &reg));

    let Some(t) = super::shared() else {
        println!("sem adaptador — as outras condições ficam por exercitar");
        return;
    };

    // ⭐ (2) **com a grade na mão, a peça com ESCULTURA é tomada** — é a wave da grade a shipar.
    assert!(
        takes_the_frame(Some(t), &com_escultura, &reg),
        "a peça com escultura ficou na CPU — a grade dela atravessa desde 2026-09-15"
    );
    // ⚠️ E o CONTROLO: sem a escultura no registo ela lê como espaço VAZIO nos dois motores, que é
    // o que o `ABSENT` significa — logo também é tomada, e por outra razão.
    assert!(
        takes_the_frame(Some(t), &com_escultura, &vazio),
        "um nome que o registo não conhece é espaço vazio nos DOIS motores"
    );

    // ⭐ O controlo de sempre: a peça sem escultura nenhuma.
    assert!(
        takes_the_frame(Some(t), &limpa, &reg),
        "com adaptador e peça limpa, o dispositivo TEM de tomar — senão esta wave não está ligada \
         a nada"
    );
}
