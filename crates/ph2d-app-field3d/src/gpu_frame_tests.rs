//! Os gates da costura — ver [`super`].

use super::takes_the_frame;

/// ⭐⭐⭐ **AS TRÊS CONDIÇÕES, e cada uma sozinha basta para o quadro ficar na CPU.**
#[test]
fn o_dispositivo_so_toma_o_quadro_assente_de_uma_peca_que_ele_sabe_desenhar() {
    let limpa = crate::smoke::scene(1);
    let com_escultura = crate::smoke::scene(6);

    // (1) sem adaptador não há dispositivo.
    assert!(!takes_the_frame(None, true, &limpa));

    let Some(t) = super::shared() else {
        // Sem GPU nesta máquina as outras duas metades não são exercitáveis — e dizê-lo é melhor
        // do que passar por não ter medido nada.
        println!("sem adaptador — as condições (2) e (3) ficam por exercitar");
        return;
    };

    // (2) o quadro de MOVIMENTO fica na CPU — é a cerca que impede a regressão do §32.
    assert!(
        !takes_the_frame(Some(t), false, &limpa),
        "o dispositivo tomou o quadro de MOVIMENTO — ele tem de ficar byte-idêntico"
    );

    // (3) ⛔ e uma peça com ESCULTURA fica na CPU, senão ela desaparece em silêncio.
    assert!(
        !takes_the_frame(Some(t), true, &com_escultura),
        "o dispositivo tomou uma peça com escultura — ela compila para espaço VAZIO na fita"
    );

    // ⭐ O controlo: com as três satisfeitas, ele toma.
    assert!(
        takes_the_frame(Some(t), true, &limpa),
        "com adaptador, quadro assente e peça sem escultura, o dispositivo TEM de tomar — senão \
         esta wave não está ligada a nada"
    );
}
