//! **A LEI DE MISTURA por degrau, no dispositivo** (plano 24 W6).
//!
//! Arquivo próprio porque o assunto é coeso e porque os irmãos estão perto do teto de LOC: *como a
//! cor de um degrau encosta na que já está ali*.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_blend_gpu -- --ignored`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

const W: u32 = 64;
const H: u32 = 64;

/// Um quadrado OPACO de meio-cinza, com uma moldura transparente. O miolo é onde a lei age (é lá
/// que há cobertura para se misturar), e o cinza é escolhido para que Multiply e Screen se afastem
/// dele em direções OPOSTAS — um oráculo que um só dos dois não daria.
const GREY: u8 = 128;

/// A fonte: quadrado opaco `[8, 56)` em ambos os eixos, alfa RETO (o que o Vello escreve).
fn square(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 8..56u32 {
        for x in 8..56u32 {
            let o = ((y * W + x) * 4) as usize;
            bytes[o] = GREY;
            bytes[o + 1] = GREY;
            bytes[o + 2] = GREY;
            bytes[o + 3] = 255;
        }
    }
    make_src(gpu, W, H, &bytes)
}

/// Um Color Overlay de cor `tint`, opacidade cheia, sob a lei `blend`.
fn overlay(tint: [f32; 4], blend: u8) -> FxOpGpu {
    FxOpGpu {
        kind: FxOp::COLOR_OVERLAY,
        sigma_px: 0.0,
        offset_px: [0, 0],
        tint,
        opacity: 1.0,
        mode: 0,
        blend,
    }
}

/// **A LUMA que as leis HSL preservam** — `0,3R + 0,59G + 0,11B` sobre valores **LINEARES**, que é
/// exactamente o `lum()` do `blend_modes.wgsl`, aplicado ao espaço em que a pilha o chama.
///
/// ⚠️ **A primeira versão deste oráculo media outra coisa e o gate nasceu VERMELHO sobre produto
/// CORRETO:** Rec.709 (`0,299/0,587/0,114`) sobre os BYTES sRGB. São dois pesos diferentes em dois
/// espaços diferentes, e preservar um não preserva o outro — medido, o `Color` saía 105,3 onde o
/// fundo estava em 128,0, e a conclusão fácil teria sido "a lei HSL não chegou ao dispositivo".
/// Um oráculo tem de medir a grandeza que a lei promete, não uma prima dela.
fn core_lum_linear(px: &[u8]) -> f64 {
    let to_linear = |b: u8| {
        let c = f64::from(b) / 255.0;
        if c <= 0.040_45 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let mut acc = 0.0;
    let mut n = 0u32;
    for y in 16..48u32 {
        for x in 16..48u32 {
            let o = ((y * W + x) * 4) as usize;
            acc +=
                0.3 * to_linear(px[o]) + 0.59 * to_linear(px[o + 1]) + 0.11 * to_linear(px[o + 2]);
            n += 1;
        }
    }
    acc / f64::from(n)
}

/// A luminância média do MIOLO (longe da orla, onde a cobertura é cheia).
fn core_luma(px: &[u8]) -> f64 {
    let mut acc = 0.0;
    let mut n = 0u32;
    for y in 16..48u32 {
        for x in 16..48u32 {
            let o = ((y * W + x) * 4) as usize;
            assert!(px[o + 3] > 250, "o miolo tem de ser opaco");
            acc += 0.299 * f64::from(px[o])
                + 0.587 * f64::from(px[o + 1])
                + 0.114 * f64::from(px[o + 2]);
            n += 1;
        }
    }
    acc / f64::from(n)
}

fn run(pass: &mut FxStackPass, gpu: &ph2d_gpu::GpuContext, ops: &[FxOpGpu]) -> Vec<u8> {
    let src = square(gpu);
    let dst = make_output_texture(gpu, W, H);
    pass.run(gpu, &src, &dst, W, H, ops, &[]);
    readback(gpu, &dst, W, H)
}

/// **A lei MUDA o desenho, e muda no SENTIDO que ela promete.**
///
/// Um Color Overlay branco sobre cinza: em `Normal` repinta de branco, em `Multiply` **não muda
/// nada** (branco é o neutro do produto), em `Screen` estoura para branco. Um Overlay PRETO faz o
/// espelho. Quatro medições que só fecham juntas se a lei certa correu — trocar `blend_sep` por
/// `cs` (Normal para tudo) colapsa as quatro num número só.
#[test]
#[ignore = "precisa de adaptador GPU"]
fn the_law_moves_the_colour_in_the_direction_it_promises() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    const NORMAL: u8 = 0;
    const MULTIPLY: u8 = 1;
    const SCREEN: u8 = 6;

    let base = core_luma(&run(&mut pass, &gpu, &[]));
    let w_normal = core_luma(&run(&mut pass, &gpu, &[overlay(WHITE, NORMAL)]));
    let w_mul = core_luma(&run(&mut pass, &gpu, &[overlay(WHITE, MULTIPLY)]));
    let w_scr = core_luma(&run(&mut pass, &gpu, &[overlay(WHITE, SCREEN)]));
    let b_mul = core_luma(&run(&mut pass, &gpu, &[overlay(BLACK, MULTIPLY)]));
    let b_scr = core_luma(&run(&mut pass, &gpu, &[overlay(BLACK, SCREEN)]));
    eprintln!(
        "base {base:.1} | branco: normal {w_normal:.1} mul {w_mul:.1} screen {w_scr:.1} \
         | preto: mul {b_mul:.1} screen {b_scr:.1}"
    );

    assert!(
        w_normal > 250.0,
        "Normal repinta de branco (medido {w_normal:.1})"
    );
    // Branco é o NEUTRO do Multiply — a lei corre e o resultado é o fundo, intacto.
    assert!(
        (w_mul - base).abs() < 2.0,
        "branco em Multiply tem de deixar o fundo onde estava ({base:.1} → {w_mul:.1})"
    );
    assert!(
        w_scr > 250.0,
        "branco em Screen estoura para branco ({w_scr:.1})"
    );
    // Preto é o espelho: neutro do Screen, absorvente do Multiply.
    assert!(b_mul < 5.0, "preto em Multiply zera ({b_mul:.1})");
    assert!(
        (b_scr - base).abs() < 2.0,
        "preto em Screen tem de deixar o fundo onde estava ({base:.1} → {b_scr:.1})"
    );
    // E o discriminante que uma lei ignorada não passaria: os cinco números são DISTINTOS onde
    // deveriam ser.
    assert!(
        (w_mul - w_normal).abs() > 100.0,
        "Multiply e Normal têm de divergir — a lei não chegou ao dispositivo"
    );
}

/// **As leis NÃO-SEPARÁVEIS chegam ao dispositivo, e `Color` é a que justifica a wave.**
///
/// `Color` troca a MATIZ preservando a LUMINOSIDADE — é o *tint / duotone* que a fila do plano 24
/// listava como um item à parte, e que o Color Overlay passa a entregar sem um décimo tipo. O
/// oráculo é a própria definição: sobre um fundo cinza, um overlay de qualquer cor em `Color` tem
/// de sair com a luminosidade do CINZA, e em `Normal` com a da cor.
///
/// ⚠️ Gate PRÓPRIO porque o irmão acima usa Multiply/Screen, que são SEPARÁVEIS: neutralizar o
/// ramo `is_hsl` do `fx_blend` deixa-o inteiramente verde — a mutação foi rodada, e foi ela que
/// mostrou que estas quatro leis não tinham cobertura nenhuma.
#[test]
#[ignore = "precisa de adaptador GPU"]
fn the_non_separable_laws_reach_the_device_and_color_keeps_the_luminosity() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    const ORANGE: [f32; 4] = [1.0, 0.4, 0.0, 1.0];
    const COLOR: u8 = 18;
    const LUMINOSITY: u8 = 19;

    let base = core_lum_linear(&run(&mut pass, &gpu, &[]));
    let normal = core_lum_linear(&run(&mut pass, &gpu, &[overlay(ORANGE, 0)]));
    let colour = core_lum_linear(&run(&mut pass, &gpu, &[overlay(ORANGE, COLOR)]));
    let lum = core_lum_linear(&run(&mut pass, &gpu, &[overlay(ORANGE, LUMINOSITY)]));
    eprintln!(
        "lum linear — base {base:.4} | normal {normal:.4} | Color {colour:.4} \
         | Luminosity {lum:.4}"
    );

    // `Color` toma a matiz da FONTE e a luminosidade do FUNDO ⇒ a luma LINEAR não se move.
    assert!(
        (colour - base).abs() < 0.005,
        "Color tem de preservar a luma do fundo ({base:.4} → {colour:.4})"
    );
    // …e `Normal` a move, senão o teste de Color é verde por acidente.
    assert!(
        (normal - base).abs() > 0.02,
        "Normal tem de mover a luma ({base:.4} → {normal:.4}) — sem isso o teste de Color não \
         separa nada"
    );
    // `Luminosity` é o espelho: a LUZ da fonte sobre a matiz do fundo. Num fundo CINZA (sem
    // matiz) o resultado é cinza com a luz da laranja ⇒ a luma vai para a DELA.
    assert!(
        (lum - normal).abs() < 0.02,
        "Luminosity tem de trazer a luz da fonte ({lum:.4} vs normal {normal:.4})"
    );
}

/// **O PESO da lei é o alfa do FUNDO** — a fórmula do W3C, e é ela que faz a mistura desvanecer
/// para Normal onde não há nada com que se misturar.
///
/// `Cs' = (1 − ab)·Cs + ab·B(Cb, Cs)`. Numa rampa de cobertura o resultado tem de CAMINHAR: na
/// ponta translúcida a cor sai a do overlay (Normal puro), na opaca sai a lei cheia.
///
/// ⚠️ **Este gate nasceu porque uma mutação NÃO sangrou:** trocar `mix(colour, b, ab)` por `b`
/// (o peso jogado fora) passava nos três irmãos, porque todos mediam o MIOLO — onde `ab = 1` e
/// `mix(x, y, 1)` É `y`. A fixture não continha o fenômeno: um quadrado de borda dura não tem
/// cobertura parcial nenhuma. Aqui a fonte é uma RAMPA, e é a rampa que torna o peso observável.
#[test]
#[ignore = "precisa de adaptador GPU"]
fn the_weight_of_the_law_is_the_backdrop_alpha() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    // Uma rampa horizontal de cobertura sobre a MESMA cor: `ab` varre `0..1` ao longo de x.
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let o = ((y * W + x) * 4) as usize;
            bytes[o] = GREY;
            bytes[o + 1] = GREY;
            bytes[o + 2] = GREY;
            #[allow(clippy::cast_possible_truncation)]
            let a = ((x * 255) / (W - 1)) as u8;
            bytes[o + 3] = a;
        }
    }
    let src = make_src(&gpu, W, H, &bytes);
    let dst = make_output_texture(&gpu, W, H);
    // Multiply de BRANCO: sobre o fundo é o NEUTRO (deixa o cinza), e em Normal repinta de branco.
    // Os dois extremos são maximamente distantes, então o caminho entre eles é legível.
    pass.run(
        &gpu,
        &src,
        &dst,
        W,
        H,
        &[overlay([1.0, 1.0, 1.0, 1.0], 1)],
        &[],
    );
    let px = readback(&gpu, &dst, W, H);

    // A cor RETA por coluna (o resolve já des-premultiplicou), na linha do meio.
    let straight = |x: u32| -> f64 {
        let o = (((H / 2) * W + x) * 4) as usize;
        f64::from(px[o])
    };
    // Onde a cobertura é quase nula a lei não alcança nada ⇒ Normal ⇒ o BRANCO do overlay.
    let sheer = straight(2);
    // Onde é cheia a lei corre inteira ⇒ Multiply por branco ⇒ o CINZA do fundo.
    //
    // ⚠️ **A ÚLTIMA coluna, e não a penúltima.** A penúltima tem alfa 251/255, então `1 − ab` deixa
    // entrar 1,6% de branco — que sai como 132 em vez de 128. O número está CERTO (é o peso a
    // funcionar), e foi a minha barra que estava errada: ela media 4,0 contra um limite de 4,0.
    // Onde se quer `ab = 1` exacto, amostra-se onde ele é exacto.
    let solid = straight(W - 1);
    let mid = straight(W / 2);
    eprintln!("rampa — translúcido {sheer:.1} · meio {mid:.1} · opaco {solid:.1} (fundo {GREY})");
    assert!(
        sheer > 240.0,
        "na ponta translúcida a lei não tem com que se misturar ⇒ a cor do overlay ({sheer:.1})"
    );
    assert!(
        (solid - f64::from(GREY)).abs() < 4.0,
        "na ponta opaca o Multiply por branco devolve o fundo ({solid:.1} vs {GREY})"
    );
    // E o caminho entre as duas é MONÓTONO — é o `mix` linear no peso, não um degrau.
    assert!(
        mid > solid + 20.0 && mid < sheer - 20.0,
        "o meio da rampa tem de ficar ENTRE as pontas ({solid:.1} < {mid:.1} < {sheer:.1})"
    );
}

/// **A lei de um tipo que NÃO a toma não move um byte.**
///
/// A metade de HONRAR mora no produtor (`FxOp::blend_code`), mas o dispositivo tem de ser inerte
/// mesmo recebendo o número cru — senão a defesa é de UMA camada só, e a camada é a que um teste,
/// um arquivo ou uma ferramenta externa consegue contornar.
///
/// ⚠️ Hoje isto é verdade por CONSTRUÇÃO e não por um `if`: o `fx_blend` só é chamado do
/// `inner_tint` e do Color Overlay, que são exactamente os quatro que tomam. O gate existe para o
/// dia em que alguém o chamar de um quinto sítio sem reler esta frase.
#[test]
#[ignore = "precisa de adaptador GPU"]
fn a_law_on_a_kind_that_does_not_take_one_moves_nothing_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    for kind in 0..FxOp::KINDS as u8 {
        if FxOp::spec(kind).takes_blend {
            continue;
        }
        let authored = FxOp::new(kind);
        let mut op = FxOpGpu {
            kind,
            sigma_px: authored.radius * 40.0,
            offset_px: [0, 0],
            tint: authored.color,
            opacity: authored.opacity,
            mode: authored.mode,
            blend: 0,
        };
        let plain = run(&mut pass, &gpu, &[op]);
        op.blend = 1; // Multiply — a lei mais destrutiva que há sobre um fundo escuro.
        let marked = run(&mut pass, &gpu, &[op]);
        let differing = plain.iter().zip(&marked).filter(|(a, b)| a != b).count();
        assert_eq!(
            differing,
            0,
            "{}: a lei alcançou um tipo que não a toma ({differing} bytes)",
            FxOp::kind_name(kind)
        );
    }
}
