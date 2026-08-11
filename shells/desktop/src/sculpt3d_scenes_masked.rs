//! **AS PEÇAS MASCARADAS que duas cenas montam** — e os números que os roteiros
//! delas citam.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE:** o irmão [`super::scenes`] responde
//! *"que cena está armada?"* e *"com que peça o smoke abre?"*; aqui mora *como
//! uma máscara é pintada numa malha, e quantos vértices ela cobre*. As duas
//! perguntas têm consumidores diferentes — o roteador do smoke pergunta à de lá,
//! e o ROTEIRO impresso pergunta às contagens daqui.
//!
//! ⚠️ **As contagens saem da MESMA peça que a cena monta**, nunca de uma conta
//! paralela: um roteiro que anuncia um número que a peça não tem é pior que um
//! roteiro sem número — ele faz o artista procurar um defeito que não existe.

/// **Quantos vértices a calota da `=22` mascara, e de quantos** — o número que
/// torna aquele smoke válido.
///
/// ⚠️ Ele sai da MESMA [`masked_dome`] que a cena monta, e não de uma conta
/// paralela: um roteiro que anuncia um número que a peça não tem é pior que um
/// roteiro sem número — ele faz o artista procurar um defeito que não existe.
#[must_use]
pub(crate) fn masked_dome_counts() -> (usize, usize) {
    let m = masked_dome();
    let total = m.vert_count();
    let masked = m
        .masks()
        .map_or(0, |k| k.iter().filter(|&&v| v >= 0.5).count());
    (masked, total)
}

/// A esfera que a cena `=22` abre à direita, **já mascarada** com uma calota.
///
/// ⚠️ A máscara é uma calota `y > 0,35` — grande o bastante para a casca ser
/// legível a qualquer distância, e com fronteira LONGA, que é onde a costura
/// vive e onde um erro de enrolamento se vê.
#[must_use]
pub(super) fn masked_dome() -> ph2d_mesh::Mesh {
    let mut m = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let n = m.vert_count();
    let cap: Vec<bool> = (0..n).map(|i| m.positions()[i][1] > 0.35).collect();
    let mask = m.masks_mut();
    for i in 0..n {
        mask[i] = f32::from(u8::from(cap[i]));
    }
    m
}

/// `=23` — a cena do **TRANSFORM**: a máscara MOVE.
///
/// ⚠️ **Duas esferas, e a assimetria é a mesma da `=22`:** a da DIREITA já vem
/// mascarada (o artista arma e julga a LEI, sem o confundidor de ter acabado de
/// pintar), a da ESQUERDA vem nua (é ela que prova a costura *pincel → botão*).
///
/// ⚠️ **E a máscara dela é MACIA, de propósito.** Uma máscara dura faria o
/// transform mover meia esfera rigidamente — correto, e **cego à lei**: as duas
/// interpolações possíveis concordam em todo vértice de peso 0 ou 1. O defeito
/// que esta wave corrige (o lerp da referência, que colapsa o vértice de meio
/// peso sobre o eixo) só é VISÍVEL na banda de transição — que é também o que um
/// pincel de máscara macio pinta. *A fixture tem de conter o fenômeno*, e uma
/// cena de smoke é uma fixture.
pub(crate) fn transform_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("23")
}

/// **Quantos vértices da esfera da `=23` estão PARCIALMENTE livres, e de
/// quantos** — o número que torna aquele smoke válido.
///
/// ⚠️ O que conta é a BANDA (`0 < peso < 1`): é ela que separa as duas leis, e
/// uma cena que a tivesse vazia deixaria o artista julgando uma propriedade que
/// ele não consegue ver. Sai da MESMA malha que a cena monta.
#[must_use]
pub(crate) fn soft_masked_counts() -> (usize, usize) {
    let m = soft_masked_sphere();
    let total = m.vert_count();
    let band = m
        .masks()
        .map_or(0, |k| k.iter().filter(|&&v| v > 0.02 && v < 0.98).count());
    (band, total)
}

/// A esfera que a cena `=23` abre à direita, com máscara MACIA.
#[must_use]
pub(super) fn soft_masked_sphere() -> ph2d_mesh::Mesh {
    let mut m = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let n = m.vert_count();
    // O peso LIVRE varre 0..1 com a latitude: o polo sul fica pregado, o norte
    // livre, e o meio é a banda.
    let free: Vec<f32> = (0..n)
        .map(|i| (0.5 + m.positions()[i][1]).clamp(0.0, 1.0))
        .collect();
    let mask = m.masks_mut();
    for i in 0..n {
        mask[i] = 1.0 - free[i];
    }
    m
}

/// `=26` — a cena do **ACHATAR E DA MÁSCARA QUE ATRAVESSA**.
///
/// ⚠️ **Uma cena só, e não duas, porque é UMA história do artista:** ele mascara
/// para proteger, subdivide para trabalhar o detalhe, e depois quer arrumar a
/// topologia. Os dois defeitos desta wave estão exatamente nessa sequência — o
/// remesh recusava com a pilha montada e mandava *reverter* (o que a deixa mais
/// alta), e quando ele enfim rodava, apagava a máscara.
pub(crate) fn flatten_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("26")
}

/// **Quantos vértices a faixa da `=26` mascara, e de quantos.**
#[must_use]
pub(crate) fn flatten_scene_counts() -> (usize, usize) {
    let m = half_masked_sphere();
    let total = m.vert_count();
    let masked = m
        .masks()
        .map_or(0, |k| k.iter().filter(|&&v| v >= 0.5).count());
    (masked, total)
}

/// A peça da `=26`: uma esfera **grossa**, com metade mascarada.
///
/// ⚠️ **GROSSA de propósito, e é o que torna o smoke julgável.** A cena pede
/// duas subdivisões, e sobre a esfera de 96×144 que o resto do módulo abre isso
/// daria milhões de vértices — o achatar mediria segundos e o remesh a 512
/// mediria mais. Com 16×24 os três níveis são 384 → 1.536 → 6.144 vértices, e a
/// pergunta que o artista responde é sobre o que ele VÊ, não sobre esperar.
///
/// ⚠️ **E a fronteira da máscara é RETA (`x > 0`), não uma calota:** ela cruza a
/// esfera inteira, então uma travessia que deslocasse o valor por meia face
/// seria visível como a linha entortando — que é o defeito que a interpolação
/// barycêntrica existe para não ter.
#[must_use]
pub(super) fn half_masked_sphere() -> ph2d_mesh::Mesh {
    let mut m = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    let n = m.vert_count();
    let side: Vec<bool> = (0..n).map(|i| m.positions()[i][0] > 0.0).collect();
    let mask = m.masks_mut();
    for i in 0..n {
        mask[i] = f32::from(u8::from(side[i]));
    }
    m
}
