//! **O PREÇO DO GRUPO DOS DEFORMADORES** — o relógio, a contagem de objectos e a coluna do
//! dispositivo. Irmão do [`motion_deformadores_probe`](super::motion_deformadores_probe) pelo
//! tecto de 600 LOC (HR-18), e o corte é por RESPONSABILIDADE: *aquele* responde **o que o nó
//! declara e o que o cartão pinta** (retratos), este responde **quanto custa e onde corre**.
//!
//! ⚠️ **O que vem junto vem por necessidade, não por arrumação:** o [`acordar`] só faz sentido
//! ao lado do relógio que ele existe para tornar honesto, e o censo
//! [`waking_a_node_takes_it_off_the_identity`] é a régua DELE — separá-los poria a lei num
//! ficheiro e a prova noutro.

use super::motion_deformadores_probe::GRUPO;

/// ⭐⭐⭐ **O DESPERTAR TEM DE ACORDAR** — para cada um dos treze, a saída do nó ACESO difere da
/// saída dele nos defaults.
///
/// ⛔⛔ **É o censo que faltava, e ele nasceu de DOIS defeitos que a tabela escondia** (08/09):
/// - o `motion.rotate` **não tem um único `Slider`** (o controlo dele é um `ParamWidget::Angle`),
///   então o despertar não lhe tocava e a linha dele cronometrava uma rotação de **0°**;
/// - a fracção era a MESMA para todos os hints, então os oito `P0X..P3Y` do
///   `motion.spline_wrap` caíam no mesmo número, os quatro pontos de controlo colapsavam num
///   ponto, a cúbica media comprimento zero e o nó tomava o **atalho inerte** — um `clone`
///   cronometrado como se fosse o embrulho.
///
/// ⚠️ **Nenhuma das duas era visível na tabela:** um `clone` e um deformador barato leem-se
/// iguais numa coluna de razão, e é por isso que a régua tem de ser a **saída**, nunca o relógio.
/// *Um corpus no ponto neutro de um knob não testa esse knob* — a mesma frase que o despertar
/// foi escrito para honrar, e que ele próprio violava em dois nós.
///
/// ⚠️⚠️ **E a 1.ª redacção DESTE gate comparava só a coluna `P`** — que é a terceira vez que
/// esta linha paga a mesma cegueira: o `motion.rotate` e o `motion.scale` escrevem `rot` e
/// `size`, **nunca `P`**, então os dois liam-se «não mudou» com o nó a girar. A comparação é do
/// **stream inteiro**, que é a única que não tem de saber o que cada nó escreve.
///
/// ⚠️ **A grelha é pequena de propósito** (8×8): a pergunta é *«mudou?»*, não *«quanto custa?»*.
#[test]
fn waking_a_node_takes_it_off_the_identity() {
    let mudos = crate::motion::motion_ciclo_probe::quem_o_despertar_nao_acorda(&GRUPO);
    assert!(
        mudos.is_empty(),
        "o despertar nao mexeu nestes: {mudos:?} -- ou o widget deles nao esta' na lista de \
         controlos continuos, ou a fraccao poe o no' de volta na identidade"
    );
}

/// ⭐⭐⭐ **O PREÇO DO GRUPO, e o 🔴 é o achado — nunca a razão** (doc 103 §5.1).
///
/// Um nó que cai na CPU **no meio de uma cadeia** não custa o que ele custa: custa o
/// dispositivo inteiro ([doc 98](../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md)
/// mediu `50,9×`). Por isso a coluna que decide é *«a cadeia inteira é reivindicada?»*, e o
/// relógio está ao lado só para dizer quanto.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture measure_the_deformer_group
/// ```
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn measure_the_deformer_group() {
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(320.0);
    crate::motion::motion_ciclo_probe::tabela(&GRUPO, lado);
}
