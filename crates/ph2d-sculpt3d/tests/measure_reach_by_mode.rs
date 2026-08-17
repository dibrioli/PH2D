//! **QUANTO CADA VERBO DESLOCA, E DE QUE REFERÊNCIA VEIO O NÚMERO.**
//!
//! Imprime, não afirma — o molde dos irmãos `measure_*`. Ele existe porque o
//! `reach()` tinha **uma** fração para o catálogo inteiro (`0,1`, que é o
//! `deform = intensidade · raio · 0,1` do `Brush.js:62` do SculptGL) mais um
//! `if verb == ClayStrips` a corrigi-la para `1,0` — e a pergunta que ninguém
//! tinha feito é *quantos outros verbos são do Blender e herdaram o número do
//! SculptGL*.
//!
//! ⚠️ **E a resposta tem duas metades, com a segunda a refutar a primeira
//! versão desta sonda:** SETE verbos só existem no Blender, mas a magnitude do
//! Blender **não é uma fração do raio na maioria das tools** — o Layer usa
//! `brush.height`, o Multiplane Scrape move para um PLANO, o Surface Smooth é
//! um alisamento. Ela imprime hoje o que a FONTE declara por verbo
//! ([`ph2d_sculpt3d::VerbProfile::reach`]), e um `--` é *"a fonte não responde
//! com uma fração"*, nunca *"é 1,0"*.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --test measure_reach_by_mode -- --ignored --nocapture`

use ph2d_sculpt3d::{Brush, RefMode, Verb};

#[test]
#[ignore = "sonda: imprime a tabela, não afirma"]
fn measure_reach_by_mode() {
    const R: f32 = 1.0;
    println!("\nverbo               modos      reach(R=1)   a FONTE declara");
    let mut only_b = 0;
    for v in Verb::ALL {
        let modes: Vec<_> = RefMode::offered_for(v).map(|m| format!("{m:?}")).collect();
        let b = Brush {
            verb: v,
            radius: R,
            ..Brush::default()
        };
        let declared = v
            .profile(RefMode::B)
            .and_then(|p| p.reach)
            .map_or("--  (nao e' fracao de raio)".to_string(), |f| {
                format!("{f:.4}")
            });
        let b_only = modes == ["B"];
        if b_only {
            only_b += 1;
        }
        println!(
            "{:<18}  {:<9}  {:>9.4}   {}{}",
            v.label(),
            modes.join(","),
            b.reach(R).abs(),
            declared,
            if b_only { "   <- so' B" } else { "" }
        );
    }
    println!("\nverbos que SO' existem no Blender: {only_b}");
}
