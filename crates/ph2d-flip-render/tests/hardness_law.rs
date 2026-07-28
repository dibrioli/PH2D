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
//! ⚠️ **CORREÇÃO (2026-07-28, 2ª rodada — esta nota dizia o OPOSTO e a medição a derrubou).**
//!
//! Ela dizia: *"o que esta wave iguala é a LEI, não o DEPÓSITO"*, e rejeitava casar o depósito
//! porque *"o acumulado depende do SPACING e não tem limite livre dele"*. As duas metades do
//! argumento são verdadeiras e **a conclusão era errada**, por duas razões:
//!
//! 1. **O residual que ela mandou o smoke decidir era MAIOR que metade da tinta.** Medido contra
//!    o depósito REAL numa estrela de um traço (sonda `painter_look.rs`): a cunha escura da quina
//!    media **−138 de 255**. O smoke decidiu — duas vezes, com foto.
//! 2. **"Assar o spacing default do Painter" é exatamente o certo aqui.** A regra que esta linha
//!    pagou quatro vezes é *a lei é função do CAMINHO, nunca de quão fino o MOTOR amostrou o
//!    caminho* — e ela segue honrada: a máscara do Flip continua função **pura** da distância ao
//!    caminho, sem nenhuma dependência de como o traço foi amostrado. O `spacing` é propriedade
//!    do **PINCEL QUE ESTAMOS IGUALANDO**, e o pedido do Enio nomeia esse pincel: *"o aspecto do
//!    traço do nosso próprio módulo painter digital"*.
//!
//! **A frase que fica:** o Flip desenha um **TRAÇO**, então o perfil dele é o perfil de **TRAÇO**
//! do Painter (`1 − Π(1−dab_k)`, pitch `0,2·raio`), nunca o de um **DAB** dele. Medido em
//! hardness 0,4 · `dn = 0,70`: um dab pesa **0,500**, o traço pesa **0,916**.
//!
//! Preço NOMEADO: `DEPOSIT_STEP` no `flip.wgsl` é espelho de `spec_default.rs:29`. Se o default do
//! Painter mudar, os dois divergem de novo — e é por isso que a constante cita a origem.
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

/// 🔴 **O que o Painter DEPOSITA num traço reto — e, desde 2026-07-28, o que o Flip PINTA.**
///
/// Ele carimba dabs ao longo do traço e os compõe por `over` (medido no produto: sem buffer de
/// cobertura em Strength 1.0, cada dab faz `ao = a + ab·(1−a)`), então a secção transversal é
/// `1 − Π_k (1 − f(t_k))`. ⚠️ O `spacing` é fração do **DIÂMETRO** (`0.10` default ⇒ passo
/// `0.2·raio`), logo o raio CANCELA e o acumulado é função pura de `(dn, hardness)` — é isso que
/// permite ao Flip, que não tem spacing nenhum, vestir esta curva.
///
/// ⚠️ **E a UNIÃO do Flip reproduz isto EXATAMENTE ao longo do corpo** (medido: traço reto,
/// pior desvio **+1/255** contra o depósito de verdade). No CRUZAMENTO ela também é exata, e por
/// um motivo bonito: o produto sobre TODOS os dabs **fatora por passagem**, então o
/// `1 − (1−P₁)(1−P₂)` que o `flip.wgsl` já compunha deixou de ser heurística e virou a
/// fatoração certa. O que sobra é o **canto CONVEXO**, onde os dabs do Painter recuam em vez de
/// correr paralelos e esta curva o superestima — o Flip pinta uma ponta mais cheia. Fica NOMEADO
/// (medido: +140/255 no vértice de uma estrela de 36°, e **zero** pixel FALTANDO tinta).
///
/// ⚠️ Esta é a 3ª escrita da mesma lei (WGSL · `cpu_mask` · aqui) e a única com laço `±64`: as
/// outras duas param em `±4`, onde o termo já é zero por `√(dn² + along²) ≥ along ≥ 1`. Elas não
/// divergem — o excesso aqui é o CONTROLE de que 4 basta.
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
///
/// Afirmado nos DOIS níveis — no dab e no TRAÇO, que é o que o Flip shipa. O 2º não é implicado
/// pelo 1º por prosa: é por o `over` só acrescentar, e essa é a linha que o gate executa.
#[test]
fn the_core_inside_the_hardness_is_solid() {
    for hi in 1..20 {
        let h = hi as f32 / 20.0;
        let probe = [0.0, h * 0.25, h * 0.5, h * 0.9, h * 0.999];
        for dn in probe {
            for (nome, w) in [
                ("dab", painter_profile(dn, h)),
                ("traco", painter_deposited(dn, h)),
            ] {
                assert!(
                    w > 0.999,
                    "hardness {h}: {nome} em dn {dn} deu {w}, esperava platô 1.0"
                );
            }
        }
    }
}

/// 🔴 **O laço do shader para em `±4` — e este gate LÊ o número no WGSL de verdade.**
///
/// Um dab a `|k|·STEP ≥ 1` está fora do disco para QUALQUER `dn ≥ 0` (`√(dn² + along²) ≥ along`),
/// então os termos além disso são exatamente zero. Mas *"exatamente zero"* é afirmação sobre
/// aritmética, e afirmação sobre aritmética se MEDE.
///
/// ⚠️ **Ele parseia `DEPOSIT_STEP`/`DEPOSIT_HALF` do `flip.wgsl`**, não os repete: um gate que
/// cravasse `4` aqui provaria que 4 basta e ficaria **CEGO ao shader apertar o laço** — e apertar
/// o laço nos DOIS lados da paridade (shader e `cpu_mask`) é a forma sempre-verde que esta linha
/// já pagou duas vezes. A referência `±64` é a âncora que nenhum dos dois pode mover.
#[test]
fn the_shaders_dab_row_is_the_whole_row() {
    const WGSL: &str = include_str!("../src/shaders/flip.wgsl");
    let grab = |nome: &str| -> String {
        let linha = WGSL
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("const {nome}")))
            .unwrap_or_else(|| panic!("const {nome} sumiu do flip.wgsl"));
        linha
            .split('=')
            .nth(1)
            .expect("valor")
            .trim()
            .trim_end_matches(';')
            .to_string()
    };
    let step: f32 = grab("DEPOSIT_STEP").parse().expect("DEPOSIT_STEP");
    let half: i32 = grab("DEPOSIT_HALF").parse().expect("DEPOSIT_HALF");
    // Controle positivo: se o parse pegar lixo, o gate abaixo passaria por acidente.
    assert!(
        step > 0.0 && (1..=64).contains(&half),
        "constantes lidas do WGSL parecem lixo: step {step} half {half}"
    );
    for hi in 0..=20 {
        let h = hi as f32 / 20.0;
        for di in 0..=200 {
            let dn = di as f32 / 200.0;
            let mut keep = 1.0_f32;
            for k in -half..=half {
                let along = k as f32 * step;
                let t = (along * along + dn * dn).sqrt();
                if t < 1.0 {
                    keep *= 1.0 - painter_profile(t, h);
                }
            }
            let curto = 1.0 - keep;
            let longo = painter_deposited(dn, h);
            assert!(
                (curto - longo).abs() < 1e-7,
                "hardness {h} dn {dn}: o laco do shader (step {step}, half {half}) deu {curto}, \
                 a referencia +-64 deu {longo}"
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

    println!("\n=== UM DAB vs O TRACO (o que o Painter deposita, e o que o Flip agora pinta) ===");
    for h in [0.9_f32, 0.5, 0.0] {
        println!("\n-- hardness {h} --");
        println!("   dn      um dab     o TRACO (Flip)     delta");
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
