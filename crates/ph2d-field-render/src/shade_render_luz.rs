//! ⭐⭐ **O VOCABULÁRIO DA LUZ** — o que uma lâmpada é, e o que um pixel recebe.
//!
//! ⚠️ **Irmão do [`super::shade_render`] por RESPONSABILIDADE, forçado pelo tecto de `700` linhas e
//! melhor por isso:** o irmão responde *«que luz esta superfície devolve ao olho»* e isto responde
//! *«que luzes existem, e em que referencial»*. São perguntas de granularidade diferente — uma é por
//! pixel, a outra é da CENA —, e é a segunda que todo chamador de fora da crate importa.
//!
//! ⚠️ **Tudo em espaço de VISTA**, que é onde o G-buffer guarda a normal: as lâmpadas e o céu chegam
//! já nesse referencial, e quem as converte é quem sabe de onde elas vêm.

use ph2d_material::Environment;

/// Uma luz direcional, em espaço de VISTA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lamp {
    /// A direcção **para** a luz, unitária.
    pub to_light: [f32; 3],
    /// A radiância que chega (`cor × intensidade`, a convenção do MaterialX).
    pub radiance: [f32; 3],
}

/// ⭐⭐⭐ **UMA LUZ QUE É UM OBJECTO DA CENA** — um ponto no MUNDO (ordem do dono, 2026-09-14).
///
/// # ⚠️ Porque ela é um tipo À PARTE da [`Lamp`], e não uma variante dela
///
/// As duas respondem a perguntas diferentes **por pixel**: a [`Lamp`] é ancorada no ECRÃ e a
/// direcção dela é a mesma em toda a imagem (é o estúdio — ela não se mexe quando a câmera roda);
/// esta é ancorada no MUNDO, e a direcção e a distância mudam de pixel para pixel. ⇒ um `enum`
/// poria um ramo dentro do laço mais quente do sombreamento para distinguir duas listas que o
/// chamador já tem separadas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointLamp {
    /// Onde ela está, no **MUNDO** — a pose da entidade, propagada.
    pub world: [f32; 3],
    /// A radiância que ela entrega a **UMA unidade** de distância.
    ///
    /// ⚠️ A queda é `1/r²`, logo este é o numerador. A unidade é a do rig da casa (`cor ×
    /// intensidade × π`), medida a uma unidade — ver o doc do `ph2d_field_ecs::FieldLight`.
    pub radiance_at_one: [f32; 3],
}

/// **O PISO DA DISTÂNCIA** de uma [`PointLamp`], em unidades de mundo — a remoção de uma
/// singularidade.
///
/// Um ponto matemático diverge quando a superfície o alcança, e `1/0` entra no sombreamento como
/// `inf`. Abaixo deste piso a peça já está saturada **há muito** — com força `1` e o material de
/// omissão uma difusa satura a `r ≈ 0,9`, isto é a `18×` este raio —, logo o que ele corta é a
/// divisão por zero e mais nada. *É o que uma luz ESFÉRICA de raio `0,05` faria.*
///
/// # ⚠️⚠️ O VALOR dele não é observável, e a lei que o gate prende é a OUTRA metade
///
/// Uma prova de mutação pô-lo a `0` e **sobreviveu**: o byte satura, logo `1/0,0025` e `1/0` pintam
/// os mesmos `255`. *O que era observável era o defeito ao lado dele* — com a luz exactamente sobre
/// o ponto, `d` é o vector **ZERO**, a direcção normalizada sai `[0,0,0]`, o `N·L` dá `0` e o pixel
/// fica **PRETO**. ⇒ abaixo do piso a direcção passa a ser a **NORMAL**, e é essa mutação que sangra
/// (`a_light_falls_off_with_the_square_of_the_distance`).
///
/// *Um piso que protege a aritmética e deixa a geometria degenerada resolve metade de um defeito, e
/// a metade que fica tem o mesmo sintoma.*
pub const POINT_LAMP_MIN_DISTANCE: f32 = 0.05;

/// O quadrado do [`POINT_LAMP_MIN_DISTANCE`] — o piso, na grandeza em que ele é comparado.
pub(crate) const PISO_DA_LAMPADA: f32 = POINT_LAMP_MIN_DISTANCE * POINT_LAMP_MIN_DISTANCE;

/// ⭐⭐ **A radiância que uma [`PointLamp`] ENTREGA a um ponto** — a queda `1/r²`, o piso e a
/// visibilidade, numa porta.
///
/// ⚠️ **É a parte da lei que NÃO depende de referencial**, e é por isso que é ela que se partilha: a
/// DIRECÇÃO para a luz tem de sair no espaço de quem pergunta (o sombreador quer-a em VISTA, o
/// [`crate::bounce`] em MUNDO) e o braço degenerado devolve *«a normal»*, que é uma resposta
/// diferente em cada um. *Partilhar o que é comum e nomear o que não é vale mais que uma porta que
/// converte duas vezes para caber nos dois.*
///
/// ⚠️ A visibilidade entra **aqui, na luz que chega** — nunca no `N·L` e nunca no resultado. Ver o
/// comentário no laço do [`radiance`] para a razão física.
pub(crate) fn chega_da_lampada(lamp: &PointLamp, dist2: f32, visivel: f32) -> [f32; 3] {
    lamp.radiance_at_one
        .map(|c| c * visivel / dist2.max(PISO_DA_LAMPADA))
}

/// A luz de uma cena: as lâmpadas de estúdio, as luzes-objecto e o céu.
pub struct Lighting<'a> {
    /// Ancoradas no ECRÃ — o estúdio.
    pub lamps: &'a [Lamp],
    /// ⭐ Ancoradas no MUNDO — os objectos da cena. `&[]` é o caminho de sempre, ao bit.
    pub points: &'a [PointLamp],
    pub sky: &'a (dyn Environment + Sync),
    /// ⭐⭐⭐ **Quanto de cada [`PointLamp`] CHEGA a cada pixel** — ver [`crate::Shadows`].
    ///
    /// ⚠️ **`None` é o caminho de sempre, ao bit**: sem o passe, toda lâmpada chega inteira a todo
    /// lado, que é exactamente o que o produto fazia até 2026-09-14. ⛔ As luzes de ECRÃ
    /// ([`Lighting::lamps`]) NÃO têm sombra e não é omissão: elas estão ancoradas no ecrã, logo
    /// giram com a câmera — uma sombra que gira com o olhar não pousa nada, ensina o contrário.
    pub shadows: Option<&'a crate::Shadows>,
}
