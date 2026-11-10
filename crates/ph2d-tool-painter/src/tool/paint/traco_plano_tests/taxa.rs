//! A cauda com a TAXA (Continuous × Accumulate, Flow baixo) no plano do traço — filho de `traco_plano_tests` por LOC.

use super::*;

/// O recto de 50 a 556 px (raio 20, Spacing 0,4, dureza 0,3) sobre branco: a tinta no eixo, `255 − R`, por píxel.
pub(super) fn eixo_do_recto(b: BrushSpec) -> Vec<u8> {
    let w = 600u32;
    let mut t = tool(w, 110, [255; 4], b);
    let pts: Vec<[f32; 2]> = (0..=110).map(|k| [50.0 + 4.6 * k as f32, 55.0]).collect();
    traco(&mut t, &pts, true);
    (0..w)
        .map(|x| 255 - t.canvas_rgba[((55 * w + x) * 4) as usize])
        .collect()
}

fn recto_de_taxa(cont: bool, forca: f32, blend: BrushBlend, cauda: f32) -> BrushSpec {
    recto_translucido(cont, false, forca, blend, cauda)
}

pub(super) fn recto_translucido(
    cont: bool,
    acc: bool,
    forca: f32,
    blend: BrushBlend,
    cauda: f32,
) -> BrushSpec {
    let mut b = pincel(20.0);
    b.hardness = 0.3;
    b.spacing = 0.4;
    b.continuous_spacing = cont;
    b.accumulate = acc;
    b.flow = 0.3;
    b.strength = forca;
    b.blend = blend;
    b.taper.end = cauda;
    b
}

/// ⭐ **O plano compõe a cápsula INTEIRA** (dono 25/10: o Continuous com a cauda «em degraus»): a região de um lote
/// parava no disco da frente de cada dab (`mask::caixa_do_dab`) e a faixa de trás de cada passo pintava-se no plano sem
/// chegar à tela (o traço `−15 %`, em degraus, sem o início); e o plano a `1/S` não escalava a cobertura do dab de
/// trás (`Dab::ante`). Um traço no plano (Multiply sobre branco) é, texel a texel no eixo, o mesmo traço em Mix (± 1),
/// à Strength 1 e a 0,6.
#[test]
fn o_plano_compoe_a_capsula_inteira() {
    for forca in [1.0, 0.6] {
        let plano = eixo_do_recto(recto_de_taxa(true, forca, BrushBlend::Multiply, 0.0));
        let mix = eixo_do_recto(recto_de_taxa(true, forca, BrushBlend::Mix, 0.0));
        let pior = plano
            .iter()
            .zip(&mix)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        assert!(
            pior <= 1,
            "Strength {forca}: o plano difere do Mix em {pior}/255\n{plano:?}\n{mix:?}"
        );
    }
}

/// A envoltória de um eixo: o máximo em cada janela de um passo (Spacing 0,4 × 40 px = 16 px) à frente — sem o
/// Continuous os dabs espaçados ondulam o corpo inteiro, e é a envoltória que afila.
pub(super) fn envoltoria(v: &[u8]) -> Vec<u8> {
    (0..v.len())
        .map(|i| *v[i..(i + 16).min(v.len())].iter().max().unwrap())
        .collect()
}

/// ⭐ **A cauda com a taxa afila sem discos nem degraus** (dono 25/10, as duas fotos): com o Flow 0,3, nas quatro
/// combinações de Continuous × Accumulate, o corpo é o do traço sem cauda (± 1), da entrada da janela ao fim nada passa
/// do pico do corpo por mais de 2/255, e a cauda, do seu pico ao fim, só desce (± 1) — sem o Continuous e com o
/// Accumulate os dabs espaçados ondulam o corpo inteiro (53…72), e é a ENVOLTÓRIA que desce (± 3).
/// Medido 25/10, pico de 410…600 / pico do corpo — antes → depois:
/// `(T,F)` 61/62 → 61/62 · `(F,F)` 74/67 → 69/67 (o ressalto da entrada) · `(T,T)` 231/231 · `(F,T)` 146/72 → 72/72
/// (os dabs do adensar a passagem inteira cada um: `⌈x⌉` onde o passo pedia `x`). Envoltória `(F,T)`: sobe 3.
#[test]
fn a_cauda_com_a_taxa_afila_sem_discos() {
    for (cont, acc) in [(true, false), (false, false), (true, true), (false, true)] {
        let com = eixo_do_recto(recto_translucido(cont, acc, 0.6, BrushBlend::Mix, 3.0));
        let sem = eixo_do_recto(recto_translucido(cont, acc, 0.6, BrushBlend::Mix, 0.0));
        // A janela da cauda: 3 diâmetros (120 px) antes do último dab (556), e a faixa limpa um raio antes dela.
        let corpo = 60..410;
        let pior = corpo
            .clone()
            .map(|x| com[x].abs_diff(sem[x]))
            .max()
            .unwrap();
        assert!(
            pior <= 1,
            "Continuous {cont} Accumulate {acc}: o corpo com a cauda difere em {pior}/255"
        );
        let pico_do_corpo = corpo.map(|x| sem[x]).max().unwrap();
        let pico = *com[410..600].iter().max().unwrap();
        assert!(
            pico <= pico_do_corpo + 2,
            "Continuous {cont} Accumulate {acc}: a cauda {pico} sobre o corpo {pico_do_corpo}\n{:?}",
            &com[410..600]
        );
        let (eixo, folga) = if !cont && acc {
            (envoltoria(&com[410..600]), 3)
        } else {
            (com[436..600].to_vec(), 1)
        };
        let i = eixo.iter().enumerate().max_by_key(|(_, v)| **v).unwrap().0;
        assert!(
            eixo[i..].windows(2).all(|w| w[1] <= w[0] + folga),
            "Continuous {cont} Accumulate {acc}: a cauda volta a subir\n{eixo:?}"
        );
    }
}
