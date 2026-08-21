//! Os gates da pintura — o que se **lê** numa linha.

use super::{STEPS_ACROSS_THE_RANGE, decimals_for_step};

/// Os **cursos** que este painel serve, de ponta a ponta. ⚠️ É a tabela que justifica a regra
/// existir: eles vão de `2e-4` a `360`, seis ordens de grandeza, e uma constante única não serve as
/// duas pontas ao mesmo tempo.
///
/// ⚠️ **O curso, e não o passo** — porque o passo é uma fração dele e os valores da linha são da
/// ordem do curso. Sondar um passo de `2e-6` a partir de `45,0` não mediria a regra: mediria o ULP
/// do `f32` em 45, que é `3,8e-6` e engoliria o passo antes de a formatação chegar a ver.
const REAL_COURSES: [(&str, f32); 4] = [
    ("ângulo", 360.0),
    ("posição ou largura num quadro de 2,4", 2.4),
    ("filete de uma peça de 0,12", 0.12),
    // ⚠️ Uma peça pequena é real: nada impede digitar `0,0002` num raio, e aí a parede do filete —
    // e portanto o curso da linha dele — é desse tamanho. É esta linha que uma constante reprova.
    ("filete de uma peça de 2e-4", 2.0e-4),
];

/// ⭐ **Dois passos vizinhos do arrasto LEEM diferente.**
///
/// ⚠️ É a lei inteira da regra, e é o que uma constante não consegue cumprir para todos os cursos ao
/// mesmo tempo: com três casas fixas, dois passos do filete de uma peça pequena leem **o mesmo
/// número** — o artista arrasta, a peça muda e a tela não.
///
/// ⚠️ As sondas são **relativas ao curso**, e é o que torna o gate honesto na ponta fina: um valor
/// grande com um passo minúsculo não é um caso deste painel, é um caso que o `f32` já não representa.
#[test]
fn two_neighbouring_drag_steps_never_read_the_same() {
    for (what, course) in REAL_COURSES {
        let step = course / STEPS_ACROSS_THE_RANGE;
        let d = decimals_for_step(step);
        for start in [0.0f32, course * 0.5, -course * 0.5, course, -course] {
            let a = format!("{:.d$}", f64::from(start), d = d);
            let b = format!("{:.d$}", f64::from(start + step), d = d);
            assert_ne!(
                a, b,
                "{what}: em {start} dois passos leem «{a}» com {d} casas — a tela não acompanha o \
                 arrasto"
            );
        }
    }
}

/// ⭐ **O que é digitado ENTRE dois passos aparece** — é a casa extra, e ela tem um motivo.
///
/// ⚠️ Sem ela, escrever `45,5` num ângulo mostraria `46`: o documento guardaria um número e o painel
/// mostraria outro. *Um painel que arredonda o que lhe foi escrito mente sobre o documento* — e a
/// mentira é indistinguível de o campo não ter aceitado o valor.
#[test]
fn a_value_typed_between_two_steps_is_not_rounded_away() {
    for (what, course) in REAL_COURSES {
        let step = course / STEPS_ACROSS_THE_RANGE;
        let d = decimals_for_step(step);
        let base = course * 0.25;
        let between = base + step * 0.5;
        assert_ne!(
            format!("{:.d$}", f64::from(base), d = d),
            format!("{:.d$}", f64::from(between), d = d),
            "{what}: meio passo desaparece na tela com {d} casas"
        );
    }
}

/// **Um passo degenerado não produz um `panic` nem um formato absurdo.**
///
/// ⚠️ Uma faixa de curso zero é possível (um nó cujo teto colapsou), e `log10(0)` é `−inf`: sem a
/// guarda, o `as usize` de um infinito é o tipo de coisa que só aparece na peça de alguém.
#[test]
fn a_degenerate_step_still_gives_a_usable_number_of_decimals() {
    for bad in [0.0f32, -1.0, f32::NAN, f32::INFINITY] {
        let d = decimals_for_step(bad);
        assert!((1..=6).contains(&d), "passo {bad} deu {d} casas");
    }
    // ⚠️ E o teto morde: abaixo de 1e-6 o `f32` de uma coordenada de ordem 1 já não distingue dois
    // valores (ULP = 1,19e-7), então mais casas escreveriam ruído do tipo — ver `decimals_for_step`.
    assert_eq!(decimals_for_step(1e-12), 6);
}
