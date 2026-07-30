//! **A WESTON é expressável hoje?** (W-Pulley W5) — o último item aberto do plano,
//! que diz *"não é expressável hoje — e é topologia, não um número que falta"*.
//!
//! ⚠️ **Esta linha viu quatro notas parecidas DISSOLVEREM na medição, então esta foi
//! tratada como hipótese — e ela SOBREVIVEU.** As duas peças de uma Weston existem
//! (o tambor **DIFERENCIAL** do W4, `radius`/`radius_out` com `gear()` pesando os
//! trechos a jusante; a cadernal **MÓVEL** do W3, vantagem 2), e montá-las juntas
//! **não** dá uma Weston.
//!
//! # O que esta sonda mede, e o que ela DEIXOU de medir
//!
//! Sobra aqui **uma** medição, e ela é sólida: o `gear` de fato **pesa a corda** — a
//! rota de um tambor `R = 0,35` sai de 8,185 m com `r = R` para 12,994 m com
//! `r = 0,15`, e a engrenagem na ponta acompanha `R/r` **exatamente** (1,0000 ·
//! 1,1667 · 1,4000 · 2,3333). O mecanismo diferencial é real e está certo.
//!
//! ⚠️ **A tabela de VANTAGEM foi REMOVIDA em vez de shipada, e o porquê é a lição:**
//! eu montei "as duas peças juntas" — tambor composto no cenário, cadernal móvel na
//! carga, ponta B morta na lança — e o rig **não assenta**. A deriva da carga em 2 s
//! contra o esforço saiu **não-monotônica e toda positiva** (2,03 · 1,90 · 4,16 ·
//! −0,03 · 3,90 · 2,98 · 2,19 m com esforço de 0,01 a 8 kg): balística, não
//! quase-estática. Um arranjo que não descansa não tem vantagem mecânica a medir, e
//! publicar uma coluna tirada dele seria um número com casas decimais sobre nada.
//!
//! ⚠️ **E antes disso a busca binária MENTIU:** ela imprimiu *"controle: esforço
//! 0,0100 kg, vantagem 199,99"*, e 0,01 é o **piso** dela — a carga nunca desceu, e a
//! coluna era o número que uma busca colapsada devolve. *Uma busca só significa algo
//! depois de a função medida ter sido vista cruzando o zero.* Foi o diagnóstico da
//! deriva crua que a derrubou.
//!
//! # A conclusão, e por que ela para aqui
//!
//! Numa Weston a corrente toca o tambor composto **DUAS vezes, com a carga NO MEIO**
//! (diâmetro grande → cadernal móvel → diâmetro pequeno). O modelo de `radius_out`
//! põe os dois contatos **ADJACENTES** — a corda entra num diâmetro e sai no outro
//! **no mesmo nó**, sem nada entre eles —, e duas roldanas concêntricas são
//! **geometricamente recusadas** pela rota (`|C₂−C₁| > |s₂r₂ − s₁r₁|`, §W4 do plano).
//! É topologia, exatamente como a nota dizia: **um nó cujos dois contatos são
//! separados na rota** = uma segunda restrição por corda.
//!
//! **Não construída** — decisão de produto, precisa de ordem do Enio. O desenho
//! escalonado está no §W5 do plano.
//!
//! `cargo test -p ph2d-physics --test measure_weston -- --ignored --nocapture`

use ph2d_physics::world::rope_route::{self, RopeWheel};

const BOOM_Y: f32 = 7.0;
const SPAN: f32 = 0.9;
const EFFORT_Y: f32 = 3.5;

/// O diâmetro de ENTRADA do tambor composto.
const R_BIG: f32 = 0.35;

/// A outra metade: o `gear` de fato PESA a corda, ou o segundo raio é decorativo?
///
/// Mede a rota diretamente — sem solver, sem equilíbrio —, porque é ali que a
/// engrenagem entra (`RopeRoute::length` é a soma **pesada**).
#[test]
#[ignore = "measurement, not a gate"]
fn measure_that_the_second_diameter_weighs_the_rope() {
    println!("\n=== O `gear` pesa a corda? (a rota, sem solver) ===\n");
    println!(
        "{:>14} | {:>12} | {:>12}",
        "r de saida", "rota (m)", "gear na ponta"
    );
    for out in [R_BIG, 0.30, 0.25, 0.15] {
        let mut wheels = vec![RopeWheel {
            centre: [0.0, BOOM_Y],
            radius: R_BIG,
            radius_out: Some(out),
            side: 1,
            id: 1,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        }];
        let (wa, wb) = ([-SPAN, EFFORT_Y], [SPAN, EFFORT_Y]);
        rope_route::resolve_sides(wa, wb, &mut wheels, &mut Vec::new());
        let r = rope_route::route(wa, wb, &wheels, &mut Vec::new()).expect("roteia");
        println!("{out:>14.2} | {:>12.4} | {:>12.4}", r.length, R_BIG / out);
    }
    println!(
        "\n  Com `r = R` o gear e 1 e a rota e a soma simples. Quanto menor o `r`,\n  \
         mais cada metro a jusante VALE -- e e dessa razao que a vantagem continua\n  \
         de um diferencial nasce."
    );
}
