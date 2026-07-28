//! A LEI DA DUREZA — o Flip e o Painter respondem a mesma pergunta.
//!
//! Ordem do Enio (2026-07-28, com foto lado a lado): *"o cruzamento de cima é o FLIP, o de baixo
//! é do Painter. O correto é o aspecto do cruzamento de baixo e o flip deveria ser idêntico"*.
//!
//! ⚠️ Isto SOBREPÕE a fidelidade ao Grease Pencil de propósito. O `hardness_mask` do
//! `flip.wgsl` era o `gpencil_stroke_round_cap_mask` ao pé da letra (`smoothstep(0,1,
//! pow(1−dn, mix(0,10,1−h)))`) — fiel, e **incompatível com o resto do app**: a mesma palavra
//! "Hardness" governava duas leis diferentes em dois módulos, e a divergência é silenciosa
//! (nenhum número aparece na tela, só a foto). A lei do app é a do Painter.
//!
//! ⚠️ **O que esta wave iguala é a LEI, não o DEPÓSITO — e a diferença é decisão medida.**
//! O Painter carimba dabs e os compõe por `over` (`1 − Π(1−a_k)`, pitch `0.2·raio`), então o que
//! ele deixa na tela é mais CHEIO que o falloff de um dab: pior delta **0,47 em hardness 0** ·
//! **0,41 em 0,5** · **0,20 em 0,9** (só no aro). O Flip compõe por UNIÃO (`min` de distância), e
//! a secção dele **é** o perfil do dab. ⛔ **Casar o depósito foi considerado e REJEITADO por
//! medição:** o acumulado depende do SPACING e **não tem limite livre dele** — conforme o
//! spacing → 0 o produto satura num disco DURO em toda a pegada. Assar uma curva de depósito no
//! Flip seria assar o *spacing default do Painter* (0,10) num módulo que não tem spacing; a LEI é
//! a única resposta estável, e é ela que a palavra "Hardness" nomeia nos dois lugares.
//! O residual está na sonda `measure_the_two_hardness_laws`; **o smoke decide** se importa.
//!
//! Aqui mora a PARIDADE: a lei vive em WGSL (device) e em Rust (`ph2d_painter_brush`), e duas
//! escritas de uma lei só divergem — então o oráculo é a função Rust REAL do Painter, nunca
//! uma reimplementação local. `ph2d-painter-brush` é crate FOLHA (`[dependencies]` vazio) e
//! entra em `[dev-dependencies]`: o `src/` do flip-render não a toca (machete-safe), exatamente
//! como as crates-nó do gate de paridade da `line/gpu-nodes`.

use ph2d_painter_brush::Falloff;

/// O perfil que o `flip.wgsl` pintava ANTES desta wave (o port literal do GP), sem o termo de
/// AA de borda (que é cobertura de pixel, não perfil). Congelado aqui como REFERÊNCIA: é o
/// código que shipava, e um `pub` sem chamador seria uma segunda resposta esperando alguém.
fn gp_profile(dn: f32, hardness: f32) -> f32 {
    let inv = (1.0 - dn).clamp(0.0, 1.0);
    if hardness > 0.999 {
        return 1.0;
    }
    let soft = 1.0 - hardness;
    let e = 10.0 * soft;
    smoothstep01(inv.powf(e))
}

/// A lei do Painter. ⚠️ **DELEGA para a função real** (`painter_oracle`) de propósito: uma cópia
/// local aqui seria a 3ª escrita da mesma lei, e um gate que a comparasse com o oráculo estaria
/// comparando uma função com ela mesma — a forma sempre-verde. A PARIDADE que importa é
/// `shader ≡ cpu_mask ≡ Painter`, e ela mora em `gpu_render.rs`
/// (`the_union_oracle_is_the_painters_law` fecha o 2º elo; os 9 gates de união fecham o 1º).
fn painter_profile(dn: f32, hardness: f32) -> f32 {
    painter_oracle(dn, hardness)
}

/// O que o Painter DEPOSITA num traço reto — não o dab isolado.
///
/// Ele carimba dabs ao longo do traço e os compõe por `over` (medido no produto: sem buffer de
/// cobertura em Strength 1.0, cada dab faz `ao = a + ab·(1−a)`), então a secção transversal é
/// `1 − Π_k (1 − f(t_k))`. ⚠️ O `spacing` é fração do **DIÂMETRO** (`0.10` default ⇒ passo
/// `0.2·raio`), logo o raio CANCELA e o acumulado é função pura de `(dn, hardness)` — é isso que
/// torna a comparação possível. O Flip compõe por UNIÃO (`min` de distância), então a secção
/// dele É o perfil do dab: as duas leis não podem casar nos dois níveis ao mesmo tempo.
fn painter_deposited(dn: f32, hardness: f32) -> f32 {
    const PITCH: f32 = 0.2; // spacing 0.10 × diâmetro, em unidades de raio
    let mut keep = 1.0_f32;
    let mut k = -64_i32;
    while k <= 64 {
        let along = k as f32 * PITCH;
        let t = (along * along + dn * dn).sqrt();
        if t < 1.0 {
            keep *= 1.0 - painter_profile(t, hardness);
        }
        k += 1;
    }
    1.0 - keep
}

/// O airbrush (Beer-Lambert), inalterado pela wave — está aqui porque as notas dele descreviam
/// o default ANTIGO ("o oposto do pico do `pow`"), e trocar o default pode tê-las tornado falsas.
fn airbrush_profile(dn: f32, hardness: f32) -> f32 {
    let k = 1.0 + (8.0 - 1.0) * hardness.clamp(0.0, 1.0);
    1.0 - (-k * (1.0 - dn * dn).max(0.0).sqrt()).exp()
}

fn smoothstep01(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// O oráculo: a função REAL do Painter, com o platô que a `BrushSpec::falloff_weight` aplica.
/// (A `falloff_weight` é método de `BrushSpec`; o platô é o remap `(t−h)/(1−h)`, e a curva é
/// o preset. Reproduzir o remap aqui e chamar o preset de verdade mantém o oráculo ancorado
/// na crate do Painter sem construir um `BrushSpec` inteiro.)
fn painter_oracle(dn: f32, hardness: f32) -> f32 {
    let h = hardness.clamp(0.0, 1.0);
    if h >= 1.0 {
        return if dn < 1.0 { 1.0 } else { 0.0 };
    }
    let remapped = ((dn - h) / (1.0 - h)).clamp(0.0, 1.0);
    Falloff::Smooth.weight(remapped)
}

// ---------------------------------------------------------------------------------------------
// GATE
// ---------------------------------------------------------------------------------------------

/// O que o platô É: dentro de `hardness` a tinta é CHEIA. Sem esta metade, "adotar a lei do
/// Painter" poderia ser satisfeito por qualquer curva que casasse nas bordas.
#[test]
fn the_core_inside_the_hardness_is_solid() {
    for hi in 1..20 {
        let h = hi as f32 / 20.0;
        let probe = [0.0, h * 0.25, h * 0.5, h * 0.9, h * 0.999];
        for dn in probe {
            let w = painter_profile(dn, h);
            assert!(
                w > 0.999,
                "hardness {h}: dn {dn} deu {w}, esperava platô 1.0"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// SONDA — a tabela que mostra o tamanho da divergência
// ---------------------------------------------------------------------------------------------

#[test]
#[ignore = "sonda: imprime a tabela das duas leis"]
fn measure_the_two_hardness_laws() {
    println!("\n=== perfil ACROSS o traço (alfa por distância normalizada ao eixo) ===");
    for h in [0.9_f32, 0.7, 0.5, 0.3] {
        println!("\n-- hardness {h} --");
        println!("   dn      GP(hoje)   Painter(alvo)    delta");
        for di in 0..=10 {
            let dn = di as f32 / 10.0;
            let a = gp_profile(dn, h);
            let b = painter_profile(dn, h);
            println!("  {dn:4.2}     {a:7.4}     {b:7.4}     {:+7.4}", b - a);
        }
    }

    println!("\n=== O QUE O PAINTER DEPOSITA (acumulado) vs o que o FLIP pinta (dab) ===");
    for h in [0.9_f32, 0.5, 0.0] {
        println!("\n-- hardness {h} --");
        println!("   dn      Flip(dab)   Painter(deposto)   delta");
        for di in 0..=10 {
            let dn = di as f32 / 10.0;
            let a = painter_profile(dn, h);
            let b = painter_deposited(dn, h);
            println!("  {dn:4.2}     {a:7.4}     {b:9.4}      {:+7.4}", b - a);
        }
        let mut worst = 0.0_f32;
        let mut at = 0.0_f32;
        for di in 0..=1000 {
            let dn = di as f32 / 1000.0;
            let d = (painter_deposited(dn, h) - painter_profile(dn, h)).abs();
            if d > worst {
                worst = d;
                at = dn;
            }
        }
        println!("  pior delta {worst:.4} em dn {at:.3}");
    }

    println!("\n=== O AIRBRUSH AINDA SE DISTINGUE DO DEFAULT? (maior |delta| sobre dn) ===");
    println!(" hardness   vs GP(antigo)   vs Painter(novo)");
    for hi in 0..=10 {
        let h = hi as f32 / 10.0;
        let worst = |f: &dyn Fn(f32, f32) -> f32| {
            let mut m = 0.0_f32;
            for di in 0..=200 {
                let dn = di as f32 / 200.0;
                m = m.max((airbrush_profile(dn, h) - f(dn, h)).abs());
            }
            m
        };
        println!(
            "   {h:4.2}      {:6.3}          {:6.3}",
            worst(&gp_profile),
            worst(&painter_profile)
        );
    }

    println!("\n=== LARGURA VISÍVEL: o dn onde o alfa cruza 0.5 (meia-tinta) ===");
    println!(" hardness    GP(hoje)   Painter(alvo)");
    for hi in 0..=10 {
        let h = hi as f32 / 10.0;
        let cross = |f: &dyn Fn(f32, f32) -> f32| {
            let mut last = 1.0_f32;
            for di in 0..=1000 {
                let dn = di as f32 / 1000.0;
                if f(dn, h) < 0.5 {
                    last = dn;
                    break;
                }
            }
            last
        };
        let a = cross(&gp_profile);
        let b = cross(&painter_profile);
        println!("   {h:4.2}      {a:6.3}        {b:6.3}");
    }
}
