//! **AS DUAS CONVENÇÕES DE COR DA PILHA** — o eixo que os outros gates não medem.
//!
//! São duas perguntas, e o módulo errava as duas: em que ESPAÇO a pilha compõe (luz linear, não
//! bytes codificados) e o que o ALFA da fonte significa (reto, não premultiplicado).
//!
//! ⚠️ **O buraco de oráculo que esta suíte fecha.** Todos os gates de FX escritos até aqui medem
//! *variação AO LONGO de uma aresta* — ondulação, pente, dente. Um defeito **constante ao longo da
//! aresta** é invisível a eles, e esta linha já pagou esse preço duas vezes: a linha preta do bevel
//! (o perfil valia 1 no fio da borda, igualzinho em todo ponto) e agora a lavagem da cor. O que se
//! afirma aqui é a **COR ATRAVÉS da banda**, e cada gate escolhe um número que as duas convenções
//! respondem DIFERENTE.
//!
//! Os dois fatos de fundo, os dois MEDIDOS: o `render_to_intermediate` do Vello entrega **alfa
//! reto** em sRGB (censo abaixo: 1696 de 1696 texels parciais com a cor cheia), e `rgb/a` sobre
//! bytes codificados **não é** uma des-premultiplicação — é ela composta com uma transferência
//! não-linear, com o erro a crescer quando a cobertura cai.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_linear_gpu -- --ignored`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, op, readback, try_headless_gpu};

/// A cor da estrela do smoke — a mesma das fotos do report.
const AMBER: [f64; 3] = [235.0, 175.0, 60.0];

fn s2l(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn l2s(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Um texel como o **Vello** o escreve: a cor sRGB CHEIA com o alfa ao lado (alfa **RETO**).
///
/// Não é suposição — é o que `the_source_carries_straight_alpha_not_premultiplied` mede no
/// rasterizador de verdade, e é a premissa que o módulo inteiro tinha ao contrário.
fn vello_texel(colour_srgb: [f64; 3], coverage: f64) -> [u8; 4] {
    let mut out = [0u8; 4];
    for c in 0..3 {
        out[c] = colour_srgb[c].round().clamp(0.0, 255.0) as u8;
    }
    out[3] = (coverage * 255.0).round().clamp(0.0, 255.0) as u8;
    out
}

/// **A COR DE UM TEXEL PARCIALMENTE COBERTO É A COR DA FORMA.**
///
/// É o defeito reportado, no oráculo que o expõe: uma rampa de cobertura de âmbar, pilha VAZIA, e
/// a cor RETA que sai tem de ser âmbar em toda a rampa. Sob a convenção antiga a divisão pelos
/// bytes sRGB sobre-corrige — medido, (255,255,82) a meia cobertura e BRANCO a um quarto —, e o
/// artista lê isso como um dente claro no fio da borda.
///
/// ⚠️ **Este gate cobre as DUAS conversões da porta de entrada.** A viagem é
/// `sRGB reto → linear → ×α → (pilha) → ÷α → sRGB`: falhar a premultiplicação faz o `resolve`
/// dividir por α uma cor que nunca foi multiplicada (a cor estoura para BRANCO ao cair a
/// cobertura), e falhar a transferência de um dos lados lava a banda. Sem piso de cobertura, e
/// isso é MEDIÇÃO: a rampa inteira fica dentro de **2 níveis**.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_partly_covered_edge_keeps_the_shapes_colour() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_linear] sem adapter — skip");
        return;
    };
    let (w, h) = (32u32, 4u32);
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let cov = f64::from(x + 1) / f64::from(w);
            let px = vello_texel(AMBER, cov);
            let o = ((y * w + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&px);
        }
    }
    let src = make_src(&gpu, w, h, &bytes);
    let dst = make_output_texture(&gpu, w, h);
    let mut pass = FxStackPass::new(&gpu);
    pass.run(&gpu, &src, &dst, w, h, &[], &[]);
    let out = readback(&gpu, &dst, w, h);

    let mut worst = 0i32;
    let mut worst_cov = 0.0;
    for x in 0..w {
        let cov = f64::from(x + 1) / f64::from(w);
        let o = ((w + x) * 4) as usize;
        let got = [out[o], out[o + 1], out[o + 2]];
        let d = (0..3)
            .map(|c| (i32::from(got[c]) - AMBER[c] as i32).abs())
            .max()
            .unwrap_or(0);
        eprintln!(
            "[fx_stack_linear] cobertura {cov:.3}  ->  ({}, {}, {})  desvio {d}",
            got[0], got[1], got[2]
        );
        if d > worst {
            worst = d;
            worst_cov = cov;
        }
    }
    assert!(
        worst <= 2,
        "a cor reta desvia {worst} níveis do âmbar da forma (pior em cobertura {worst_cov:.3}) — \
         a des-premultiplicação está a acontecer no espaço errado"
    );
}

/// **UM BORRÃO MISTURA LUZ, e a média de preto com branco é 187, não 128.**
///
/// É o número mais conhecido da questão, e o que torna este gate impossível de satisfazer por
/// acidente: numa fronteira preto/branco os pesos gaussianos somam meio a meio, então o resultado
/// é `encode(0,5) ≈ 187`. Borrar os BYTES sRGB dá `encode⁻¹` de lugar nenhum — 128 —, e é essa a
/// franja escura clássica que todo blur em espaço de gama produz.
///
/// ⚠️ O par que **cavalga** a fronteira é o oráculo (não um texel só): a fronteira cai ENTRE dois
/// texels, então cada um vê 45%/55% e só a média deles é o meio exato.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn blurring_black_and_white_gives_the_linear_mean_not_the_srgb_one() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_linear] sem adapter — skip");
        return;
    };
    let (w, h) = (64u32, 8u32);
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let v = if x < w / 2 { 0u8 } else { 255u8 };
            let o = ((y * w + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[v, v, v, 255]);
        }
    }
    let src = make_src(&gpu, w, h, &bytes);
    let dst = make_output_texture(&gpu, w, h);
    let mut pass = FxStackPass::new(&gpu);
    // sigma 4 ⇒ meia-largura 12; a fronteira está a 32 texels de qualquer borda da textura, então
    // o kernel nunca puxa a transparência de fora (que enviesaria a média).
    pass.run(
        &gpu,
        &src,
        &dst,
        w,
        h,
        &[op(FxOp::BLUR, 4.0, [0.0; 4])],
        &[],
    );
    let out = readback(&gpu, &dst, w, h);

    let row = 4u32;
    let at = |x: u32| -> f64 { f64::from(out[(((row * w) + x) * 4) as usize]) };
    let (left, right) = (at(w / 2 - 1), at(w / 2));
    let mid = (left + right) / 2.0;
    let want = l2s(0.5) * 255.0;
    eprintln!(
        "[fx_stack_linear] fronteira: {left:.0} | {right:.0}  media {mid:.1}  (linear {want:.1} · gama 128)"
    );
    assert!(
        (mid - want).abs() <= 2.0,
        "a média na fronteira é {mid:.1} e a luz manda {want:.1} — o borrão está a somar bytes \
         codificados, que é a franja escura de sempre"
    );
    assert!(
        left > 160.0,
        "o lado escuro da fronteira lê {left:.0}: mesmo sem a média, borrar em gama não chega perto"
    );
}

/// **UM COLOR OVERLAY À FORÇA TOTAL PINTA EXATAMENTE AQUELA COR.**
///
/// O `tint` chega do painel em sRGB (é o que a swatch mostra), e o miolo da pilha só fala linear —
/// então ele atravessa DUAS conversões, e uma identidade é a única forma de as prender juntas.
/// Um cinza médio é o fixture certo: branco e preto são pontos fixos das duas convenções e não
/// distinguiriam nada (a armadilha do fixture que não contém o fenómeno).
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_colour_overlay_at_full_strength_paints_exactly_that_colour() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_linear] sem adapter — skip");
        return;
    };
    let (w, h) = (8u32, 8u32);
    let bytes = vec![255u8; (w * h * 4) as usize];
    let src = make_src(&gpu, w, h, &bytes);
    let dst = make_output_texture(&gpu, w, h);
    let mut pass = FxStackPass::new(&gpu);
    let grey = 128.0 / 255.0;
    pass.run(
        &gpu,
        &src,
        &dst,
        w,
        h,
        &[op(
            FxOp::COLOR_OVERLAY,
            0.0,
            [grey as f32, grey as f32, grey as f32, 1.0],
        )],
        &[],
    );
    let out = readback(&gpu, &dst, w, h);
    let o = (((h / 2) * w + w / 2) * 4) as usize;
    let got = [out[o], out[o + 1], out[o + 2], out[o + 3]];
    eprintln!("[fx_stack_linear] overlay 128 -> {got:?}");
    for (c, &v) in got.iter().take(3).enumerate() {
        assert!(
            i32::from(v).abs_diff(128) <= 1,
            "o canal {c} saiu {v} para um tint de 128 — o tint não está a ser linearizado na \
             entrada ou a saída não está a ser codificada (o erro seria ~188)"
        );
    }
}

/// **UM HALO PINTA A COR AUTORADA** — os degraus que compõem por baixo (Glow) e o Contorno.
///
/// O gate do Color Overlay cobre o dispatch PONTUAL; estes dois cobrem os outros dois braços onde
/// o `tint` entra na imagem (o `cs_op_v` e o `cs_op_field`). Onde o halo é opaco e a forma não
/// está, o que sai TEM de ser o que a swatch mostra — a viagem sRGB → linear → sRGB é uma
/// identidade, e é o único jeito de a prender nas duas pontas.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_halo_paints_the_colour_the_swatch_shows() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_linear] sem adapter — skip");
        return;
    };
    let (w, h) = (48u32, 48u32);
    // Um quadrado opaco no meio: o halo nasce à volta dele, sobre transparência.
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    for y in 18..30 {
        for x in 18..30 {
            let o = ((y * w + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let src = make_src(&gpu, w, h, &bytes);
    let grey = 128.0f32 / 255.0;
    let tint = [grey, grey, grey, 1.0];
    let mut pass = FxStackPass::new(&gpu);
    for (name, kind, sigma, probe) in [
        // O glow espalha: um texel logo fora da forma tem halo forte e nenhuma forma por baixo.
        ("glow", FxOp::GLOW, 6.0f32, (24u32, 16u32)),
        // O contorno é uma dilatação exata: dentro da banda a cobertura é 1.
        ("outline", FxOp::OUTLINE, 5.0, (24, 15)),
    ] {
        let dst = make_output_texture(&gpu, w, h);
        pass.run(&gpu, &src, &dst, w, h, &[op(kind, sigma, tint)], &[]);
        let out = readback(&gpu, &dst, w, h);
        let o = ((probe.1 * w + probe.0) * 4) as usize;
        let got = [out[o], out[o + 1], out[o + 2], out[o + 3]];
        eprintln!("[fx_stack_linear] {name} halo -> {got:?}");
        assert!(
            got[3] > 32,
            "{name}: a sonda caiu onde não há halo ({}), a fixture não contém o fenómeno",
            got[3]
        );
        for (c, &v) in got.iter().take(3).enumerate() {
            assert!(
                i32::from(v).abs_diff(128) <= 2,
                "{name}: o canal {c} saiu {v} para um tint de 128 — o tint deste braço não passa \
                 pela porta de linearização (o erro seria ~188)"
            );
        }
    }
}

/// **O `tint` CRU É LIDO EXATAMENTE UMA VEZ, e é dentro da porta que o converte.**
///
/// ⚠️ Os gates acima medem três braços; o shader tem CINCO sítios de tint (halo do `cs_op_v`,
/// inner, bevel, contorno, overlay) e o sexto nasce sem gate. **Enumerar leitores apodrece** — esta
/// linha já pagou isso no `needs_heading` do Painter —, então a afirmação estrutural é feita sobre
/// a FONTE: fora de `tint_lin`, ninguém toca `g.tint.rgb`.
///
/// O `.a` do tint fica de fora de propósito: alfa não é transferido, é linear por definição.
#[test]
fn the_raw_tint_is_read_only_inside_the_door_that_linearises_it() {
    let src = include_str!("../src/fx_stack_shader.rs");
    let door = "fn tint_lin() -> vec3<f32> { return srgb_to_linear3(g.tint.rgb); }";
    assert!(
        src.contains(door),
        "a porta `tint_lin` mudou de forma — o gate ficaria verde sobre um shader que não a tem"
    );
    let rest = src.replace(door, "");
    let strays: Vec<_> = rest.match_indices("g.tint.rgb").collect();
    assert!(
        strays.is_empty(),
        "{} sítio(s) leem `g.tint.rgb` cru fora de `tint_lin` — um tint em sRGB usado como se \
         fosse linear pinta ~1,45x claro, e nenhum gate de forma o vê",
        strays.len()
    );
}

/// **A VIAGEM sRGB → LINEAR f16 → sRGB DEVOLVE O BYTE DE ENTRADA, nos 256.**
///
/// O ingest roda SEMPRE, inclusive numa pilha vazia, então a identidade da pilha vazia passou a
/// depender da precisão do espaço de trabalho. Isto pina a frase do doc em vez de a supor: meia
/// precisão em ponto flutuante tem ~10 bits de mantissa e a transferência é monótona e suave, logo
/// o erro fica muito abaixo de meio nível de 8 bits — mas *muito abaixo* é uma afirmação, e uma
/// afirmação sobre números merece um teste.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_round_trip_through_linear_returns_every_byte_unchanged() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_linear] sem adapter — skip");
        return;
    };
    let (w, h) = (16u32, 16u32);
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    for i in 0..(w * h) as usize {
        let v = u8::try_from(i).unwrap_or(255);
        bytes[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
    }
    let src = make_src(&gpu, w, h, &bytes);
    let dst = make_output_texture(&gpu, w, h);
    let mut pass = FxStackPass::new(&gpu);
    pass.run(&gpu, &src, &dst, w, h, &[], &[]);
    let out = readback(&gpu, &dst, w, h);
    let mut bad = Vec::new();
    for i in 0..(w * h) as usize {
        let want = u8::try_from(i).unwrap_or(255);
        if out[i * 4] != want {
            bad.push((want, out[i * 4]));
        }
    }
    assert!(
        bad.is_empty(),
        "{} níveis não sobreviveram à ida e volta: {:?}",
        bad.len(),
        &bad[..bad.len().min(8)]
    );
}

/// **A FONTE TRAZ ALFA RETO, NÃO PREMULTIPLICADO** — o contrato com o rasterizador, medido nele.
///
/// ⚠️ **É o gate que faltava, e a sua ausência custou a wave inteira.** O doc do `fx_stack` afirmava
/// *"premultiplicada — é o que o Vello escreve"*, e a afirmação era falsa. Ela sobreviveu porque num
/// texel OPACO e num VAZIO as duas convenções produzem exatamente os mesmos bytes: só a cobertura
/// PARCIAL as separa, e toda fixture de cobertura parcial do módulo tinha sido escrita pela mesma
/// mão que escreveu a premissa. **Nenhum gate perguntava ao Vello.**
///
/// O oráculo é o rasterizador de verdade — não a nossa ideia dele —, e por isso este gate também
/// avisa se um upgrade de Vello trocar a convenção debaixo de nós.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_source_carries_straight_alpha_not_premultiplied() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_linear] sem adapter — skip");
        return;
    };
    let (w, h) = (256u32, 256u32);
    let Ok(mut scratch) =
        ph2d_render::VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (w, h))
    else {
        eprintln!("[fx_stack_linear] sem VelloPass — skip");
        return;
    };
    // Um disco: a borda curva dá centenas de texels de cobertura parcial em todos os ângulos.
    let mut scene = vello::Scene::new();
    scene.fill(
        vello::peniko::Fill::NonZero,
        vello::kurbo::Affine::IDENTITY,
        vello::peniko::Color::from_rgba8(235, 175, 60, 255),
        None,
        &vello::kurbo::Circle::new((128.0, 128.0), 96.0),
    );
    scratch
        .render_to_intermediate(
            &gpu,
            &scene,
            (w, h),
            vello::peniko::Color::TRANSPARENT,
            false,
        )
        .expect("render");
    let px = readback(&gpu, scratch.intermediate_texture(), w, h);

    let (mut straight, mut premul, mut other, mut n) = (0u32, 0u32, 0u32, 0u32);
    for i in 0..(w * h) as usize {
        let (r, g, b, a) = (px[i * 4], px[i * 4 + 1], px[i * 4 + 2], px[i * 4 + 3]);
        if a == 0 || a == 255 {
            continue;
        }
        n += 1;
        let af = f64::from(a) / 255.0;
        if [r, g, b] == [235, 175, 60] {
            straight += 1;
        } else if (s2l(f64::from(r) / 255.0) - s2l(AMBER[0] / 255.0) * af).abs() < 0.02 {
            premul += 1;
        } else {
            other += 1;
        }
    }
    eprintln!(
        "[fx_stack_linear] {n} texels parciais: RETO {straight} · PREMUL {premul} · outro {other}"
    );
    assert!(
        n > 400,
        "a fixture não tem banda parcial suficiente: {n} texels"
    );
    assert_eq!(
        straight, n,
        "{premul} texels premultiplicados e {other} de outra forma entre {n} — a convenção da \
         FONTE mudou, e o `cs_ingest` é quem tem de aprender (ele é a única porta que a lê)"
    );
}
