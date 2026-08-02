//! **O FOLD DO RELEVO PARALELO É O FOLD SERIAL, AO BIT** — os gates da wave do pen-up longo
//! (doc 28 §5.65).
//!
//! # Por que dois gates e não um
//!
//! As duas rotas **não são duas implementações**: existe UM corpo (o kernel de uma linha) e dois
//! *walkers* que o chamam. Isso torna a identidade verdadeira por construção — e **limita o que um gate
//! de identidade pode provar** ([[feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body]]):
//! um defeito DENTRO do corpo aparece nas duas rotas e sai verde. O que estes gates cobrem é a outra
//! metade, que é onde a paralelização pode errar sozinha: **o mapeamento linha → índice**.
//!
//! E um gate de identidade sozinho fica verde comparando *serial contra serial* se o piso do pool subir
//! ou a fixture encolher, então o irmão de RAZÃO existe para dizer que a rota rápida de fato correu —
//! *um defeito que só um relógio enxerga é um defeito que máquina carregada esconde*.

use super::super::Region;
use super::*;

/// A rota SERIAL do box blur, **CONGELADA** — é o código que shipava antes desta wave, verbatim.
///
/// ⚠️ Ela vive sob `cfg(test)` e não como um `pub(super)` sem chamador: um segundo caminho vivo é uma
/// **segunda resposta** esperando alguém chamá-la (a lição do `warp_axis` e do `serial_side`).
fn serial_box_blur(field: &mut [f32], w: u32, h: u32, r: u32) {
    let (wi, hi) = (w as usize, h as usize);
    if r < 1 || wi == 0 || hi == 0 || field.len() < wi * hi {
        return;
    }
    let rows = |src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize| {
        let inv = 1.0 / (2 * r + 1) as f32;
        let last = w - 1;
        for y in 0..h {
            let row = y * w;
            for x in 0..w {
                let mut sum = 0.0;
                for k in 0..=2 * r {
                    let sx = (x + k).saturating_sub(r).min(last);
                    sum += src[row + sx];
                }
                dst[row + x] = sum * inv;
            }
        }
    };
    let t = |src: &[f32], dst: &mut [f32], w: usize, h: usize| {
        for y in 0..h {
            for x in 0..w {
                dst[x * h + y] = src[y * w + x];
            }
        }
    };
    let mut tmp = vec![0.0f32; wi * hi];
    rows(&field[..wi * hi], &mut tmp, wi, hi, r as usize);
    let mut t2 = vec![0.0f32; wi * hi];
    t(&tmp, &mut t2, wi, hi);
    let mut t3 = vec![0.0f32; wi * hi];
    rows(&t2, &mut t3, hi, wi, r as usize);
    t(&t3, &mut field[..wi * hi], hi, wi);
}

/// Um campo ESTRUTURADO — cristas, vales e zeros. Um campo chato faria qualquer blur concordar, e o
/// gate seria verde por vácuo.
fn ridged(w: usize, h: usize) -> Vec<f32> {
    (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            if (x / 7 + y / 5) % 3 == 0 {
                0.0
            } else {
                ((x * 13 + y * 29) % 97) as f32 / 97.0 - 0.5
            }
        })
        .collect()
}

/// **O blur paralelo é o blur serial, ao BIT.**
///
/// ⚠️ A fixture ATRAVESSA o piso de taps de propósito — abaixo dele o produto roda a rota serial e o
/// gate estaria comparando o serial contra ele mesmo (a armadilha que o `undo_delta_tests` documenta).
/// A mutação que ela existe para pegar é o mapeamento `linha → y` na rota paralela.
#[test]
fn the_parallel_settle_is_the_serial_settle_to_the_byte() {
    for (w, h, r) in [(1024usize, 700usize, 4u32), (777, 513, 3), (2048, 400, 4)] {
        assert!(
            w * h * (2 * r as usize + 1) >= 1 << 21,
            "a fixture {w}x{h} r={r} tem de cruzar o piso de taps, senao mede serial contra serial"
        );
        let src = ridged(w, h);
        let (mut fast, mut slow) = (src.clone(), src.clone());
        settle(
            &mut fast,
            u32::try_from(w).unwrap(),
            u32::try_from(h).unwrap(),
            f32::from(u8::try_from(r).unwrap()) / 4.0,
        );
        serial_box_blur(
            &mut slow,
            u32::try_from(w).unwrap(),
            u32::try_from(h).unwrap(),
            r,
        );
        let bad = fast
            .iter()
            .zip(&slow)
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        assert_eq!(
            bad, 0,
            "{w}x{h} r={r}: {bad} texels divergiram do blur serial"
        );
    }
}

/// **A caminhada por linhas visita EXATAMENTE o que o [`for_each_in`] visita** — mesmos índices
/// globais, mesmo elemento de saída.
///
/// ⚠️ A fixture cruza o piso de células (senão `par_rows_in` delega ao próprio `for_each_in` e o gate
/// compara uma função com ela mesma), e o rect é **deslocado nos dois eixos**: com `rect.x == 0` um erro
/// no offset de coluna é invisível.
#[test]
fn the_row_walk_touches_exactly_what_the_serial_walk_touches() {
    let (w, h) = (900u32, 700u32);
    let rect = Region {
        x: 37,
        y: 21,
        w: 800,
        h: 640,
    };
    assert!(
        (rect.w as usize) * (rect.h as usize) >= 1 << 18,
        "a fixture tem de cruzar o piso de celulas"
    );
    let n = (w as usize) * (h as usize);
    let (mut fast, mut slow) = (vec![0u32; n], vec![0u32; n]);
    // O kernel escreve o PRÓPRIO índice global: se um walker visitar o texel errado, o valor denuncia
    // qual — um oráculo que não pode concordar por acidente.
    par_rows_in(&mut fast, rect, w, |i, d| {
        *d = u32::try_from(i).unwrap() ^ 0xA5A5;
    });
    for_each_in(rect, w, |i| {
        slow[i] = u32::try_from(i).unwrap() ^ 0xA5A5;
    });
    let bad = fast.iter().zip(&slow).filter(|(a, b)| a != b).count();
    assert_eq!(bad, 0, "{bad} texels divergiram da caminhada serial");
    assert!(
        fast.iter().filter(|&&v| v != 0).count() >= 1 << 18,
        "o walk tem de ter escrito a janela inteira"
    );
}

/// **A rota rápida do blur de fato CORRE** — a metade que só um relógio pode ver.
///
/// Sem ela o gate de identidade acima fica verde comparando serial contra serial no dia em que o piso
/// subir. A barra é RAZÃO (nunca wall-clock): `ci-test` compila em `opt-level=1` e um kill de relógio
/// mediria o PERFIL do build, não o código.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn the_parallel_settle_is_faster_than_the_serial_one() {
    use std::time::Instant;
    let (w, h) = (2048usize, 1400usize);
    let src = ridged(w, h);
    let ms = |mut f: Vec<f32>, par: bool| {
        let t0 = Instant::now();
        if par {
            settle(
                &mut f,
                u32::try_from(w).unwrap(),
                u32::try_from(h).unwrap(),
                1.0,
            );
        } else {
            serial_box_blur(
                &mut f,
                u32::try_from(w).unwrap(),
                u32::try_from(h).unwrap(),
                4,
            );
        }
        t0.elapsed().as_secs_f64() * 1000.0
    };
    let _ = ms(src.clone(), true); // aquece o pool: o 1º spin-up não é o regime
    let (par, ser) = (ms(src.clone(), true), ms(src.clone(), false));
    let ratio = ser / par;
    eprintln!("[settle] serial {ser:.2} ms · paralelo {par:.2} ms · {ratio:.2}x");
    assert!(
        ratio >= 1.5,
        "a rota paralela nao esta correndo (serial {ser:.2} / paralelo {par:.2} = {ratio:.2}x) — \
         confira se o BLUR_PAR_MIN_TAPS ainda cabe sob esta fixture"
    );
}
