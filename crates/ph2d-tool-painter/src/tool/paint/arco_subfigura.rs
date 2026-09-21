//! **A fronteira de uma SUB-FIGURA, derivada do ARCO** — a porta única que todo acumulador
//! por-traço da pilha pergunta antes de atravessar de uma figura para a seguinte.
//!
//! Uma sessão de figuras é **UM traço**: o pen-up não a fecha (a figura fica editável até ao
//! Apply), e um lote do [`super::stamp_preview`] é a **CONCATENAÇÃO** das listas de dabs de todas
//! as figuras vivas — a activa mais cada parqueada, e um contorno por região no boolean. Todo
//! estado que se acumula ao longo do traço atravessa essa junta se ninguém a partir, e **dois
//! atravessavam**, com o mesmo report do dono a nomear os dois sintomas (2026-09-21:
//! *«2 círculos com o mesmo pincel e um está diferente do outro»* · *«se dois círculos cada um tem
//! um aspecto»*):
//!
//! 1. A **corrente do esfregão** ([`super::smear_warp`]) — o último dab de um círculo levantava
//!    tinta para o primeiro dab do outro, **atravessando a tela**. Medido: com a 2.ª figura longe
//!    da 1.ª, a 1.ª perdia `14 %` do alfa dela.
//! 2. A **subamostragem por arco** de uma camada maior que o pincel
//!    (o `camada_dabs` do [`super::composite`]) — o acumulador ficava no fim do arco da 1.ª
//!    figura e lia a lista inteira da 2.ª como *«perto demais do último que guardei»*. Medido com
//!    uma camada `Brush` de `size = 3` e dois círculos congruentes: **`0` texels contra `1 835`**.
//!    *A segunda figura não aparecia de todo.*
//!
//! ⚠️ **Elas são o mesmo defeito em dois acumuladores, e é por isso que isto é uma PORTA e não
//! duas linhas.** A primeira foi curada em 2026-09-20 com a regra escrita à mão dentro do
//! esfregão; a segunda apareceu no dia seguinte, no outro acumulador, **com a cura já escrita a
//! três ficheiros de distância**. *Uma lei escrita em dois sítios ainda não é uma lei.*
//!
//! ## A fronteira é DERIVADA, nunca um campo novo
//!
//! Todo `fill_*_preview` recomeça o `Dab::arc_len` em zero (`stroke/ellipse.rs`: *«fresh fill → the
//! Flow along-coordinate starts at the perimeter's origin»*), logo **um arco que anda para TRÁS é
//! uma sub-figura nova**. Não é preciso um campo, um índice de figura nem uma segunda lista: o
//! facto já viaja no dab.
//!
//! ⛔ **Um limiar sobre o comprimento do salto seria um número escolhido** — e um traço à mão
//! livre rápido produz saltos legítimos do mesmo tamanho, logo ele apagaria a corrente onde ela é
//! o produto.

/// `true` quando `arco` pertence a uma sub-figura **NOVA** em relação a `arco_do_ultimo`.
///
/// ⚠️ **A comparação é ESTRITA de propósito.** Dois dabs com *exactamente* o mesmo arco não são uma
/// figura nova — são as cópias que a Simetria, o Spray e o Rough emitem para o **mesmo** ponto do
/// caminho, e parti-las seria partir a corrente dentro de uma figura só. *Esta porta responde
/// «nasceu outra figura?», nunca «este dab está longe do anterior?».*
///
/// ⚠️ **Um arco `NaN` responde `false`** (toda comparação com `NaN` é falsa), e é o valor
/// conservador: sem fronteira, o acumulador segue como seguia. Um `arco_do_ultimo` de
/// `NEG_INFINITY` — o estado inicial de [`super::state::PaintState::composite_arco`] — também
/// responde `false`, e está certo: *o princípio de um traço não é uma fronteira, é o princípio.*
pub(super) fn nasce_uma_subfigura(arco_do_ultimo: f32, arco: f32) -> bool {
    arco < arco_do_ultimo
}

#[cfg(test)]
mod tests {
    use super::nasce_uma_subfigura;

    /// A lei, nos dois sentidos, mais os três valores que a fazem tropeçar.
    #[test]
    fn o_arco_que_anda_para_tras_e_uma_figura_nova() {
        // Ao longo de uma figura o arco CRESCE — nunca há fronteira.
        assert!(!nasce_uma_subfigura(0.0, 0.0));
        assert!(!nasce_uma_subfigura(10.0, 10.5));
        assert!(!nasce_uma_subfigura(10.0, 1_000.0));
        // O `fill_*_preview` da figura seguinte recomeça o arco em zero.
        assert!(nasce_uma_subfigura(251.3, 0.0));
        assert!(nasce_uma_subfigura(251.3, 251.2));

        // ⚠️ CONTROLO da comparação estrita: as cópias da Simetria partilham o arco do original, e
        // uma porta escrita com `<=` partiria a corrente DENTRO de uma figura.
        assert!(
            !nasce_uma_subfigura(37.5, 37.5),
            "dois dabs no mesmo ponto do caminho são a mesma figura"
        );

        // O princípio de um traço, e o valor que nenhuma guarda escrita com `<` vê.
        assert!(!nasce_uma_subfigura(f32::NEG_INFINITY, 0.0));
        assert!(!nasce_uma_subfigura(10.0, f32::NAN));
        assert!(!nasce_uma_subfigura(f32::NAN, 10.0));
    }
}
