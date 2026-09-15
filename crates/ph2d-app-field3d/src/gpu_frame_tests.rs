//! Os gates da costura — ver [`super`].

use super::takes_the_frame;

/// ⭐⭐⭐ **AS DUAS CONDIÇÕES, e cada uma sozinha basta para o quadro ficar na CPU.**
///
/// # ⚠️⚠️ Eram TRÊS até 2026-09-15, e a que saiu era um DEFEITO
///
/// A terceira dizia *«só o quadro ASSENTE»*, e nasceu como cerca contra a regressão do §32 (o
/// gesto lento). ⛔ Ela era a causa directa do report do dono — *«e apagar o AO ao rotacionar a
/// tela»*: o sombreado de contacto **só existe no caminho do dispositivo**, logo enquanto ela
/// existiu ele desaparecia a cada gesto e voltava ao largar.
///
/// ⭐ O que a substitui não é uma cerca, é a lei da W73 a viajar com o pedido: o `antialias` chega
/// ao [`ph2d_field_gpu::trace::MarchSetup`] e o dispositivo **salta o segundo despacho** quando ele
/// é falso — *grosso a mexer, nítido ao assentar*, a mesma lei que a CPU já seguia.
#[test]
fn o_dispositivo_so_toma_uma_peca_que_ele_sabe_desenhar() {
    let limpa = crate::smoke::scene(1);
    let com_escultura = crate::smoke::scene(6);

    // (1) sem adaptador não há dispositivo.
    assert!(!takes_the_frame(None, &limpa));

    let Some(t) = super::shared() else {
        // Sem GPU nesta máquina a outra metade não é exercitável — e dizê-lo é melhor do que
        // passar por não ter medido nada.
        println!("sem adaptador — a condição (2) fica por exercitar");
        return;
    };

    // (2) ⛔ uma peça com ESCULTURA fica na CPU, senão ela desaparece em silêncio.
    assert!(
        !takes_the_frame(Some(t), &com_escultura),
        "o dispositivo tomou uma peça com escultura — ela compila para espaço VAZIO na fita"
    );

    // ⭐ O controlo: com as duas satisfeitas, ele toma.
    assert!(
        takes_the_frame(Some(t), &limpa),
        "com adaptador e peça sem escultura, o dispositivo TEM de tomar — senão esta wave não \
         está ligada a nada"
    );
}
