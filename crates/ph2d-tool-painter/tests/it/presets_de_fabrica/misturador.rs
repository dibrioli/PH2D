//! ⭐ **A área do Blend cabe no círculo** (dono 03/11, *«a área afetada é MAIOR que o círculo»*): a tabela dos Blender
//! mede o que MUDA a olho, não a [`largura_equivalente`](super::regua_da_largura::largura_equivalente) (área ÷ pico): num
//! misturador o pico mora na orla e a equivalente lê ~metade da pegada — o cursor de 52 px pintava 80 (tabela ao lado
//! do [`LIMIAR_VISIVEL`]). O alarme continua a ler a equivalente, contínua no raio.

use super::memoria::{custo_da_cena, em_paralelo_pela_memoria};
use super::*;

/// Os raios do motor da tabela (o raio 224 mede 400–600 px; acima, a mesma razão).
const MOTORES: [f32; 14] = [
    2.0, 3.0, 4.5, 7.0, 10.0, 14.0, 20.0, 28.0, 40.0, 56.0, 80.0, 112.0, 160.0, 224.0,
];
/// O limiar do que se vê: 13/255 ≈ 5 %. Medido 03/11, motor 28 · 56 · 160 (Blender Soft): a 5 % 51 · 101 · 286, a 10 %
/// 29 · 100 · 284, a equivalente 29,5 · 65,5 · 175. A 10 % a banda fraca de fora escapa até ao motor ~50, e é ela que o
/// dono vê passar do cursor.
const LIMIAR_VISIVEL: u8 = 13;

/// Quanto a área mudada pode passar do círculo. Medido 03/11 com esta tabela: o pior é o Blender Textured a 52 (64, o grão
/// esparso); os outros cabem em ± 2 px. Com a tabela da equivalente: 80 a 52 e 160 a 104 (1,54×).
const FOLGA_DO_CIRCULO: f32 = 1.25;

/// A extensão da mudança por coluna, a mediana das colunas do meio. ⛔ Só para a tabela: salta onde a banda fraca passa o
/// limiar (motor 20 → 28: 21 → 51), que é onde o olho a passa a ver; a régua do alarme é a contínua.
fn largura_visivel(px: &[u8], w: u32, h: u32) -> f32 {
    let mut v: Vec<f32> = ((w / 3)..(2 * w / 3))
        .filter_map(|x| {
            let muda = |y: &u32| 255 - px[((y * w + x) * 4) as usize] >= LIMIAR_VISIVEL;
            let lo = (0..h).find(|y| muda(y))?;
            let hi = (0..h).rfind(|y| muda(y))?;
            Some((hi - lo + 1) as f32)
        })
        .collect();
    v.sort_by(f32::total_cmp);
    v.get(v.len() / 2).copied().unwrap_or(0.0)
}

/// A tabela raio do motor → largura visível de um Blender (~1 min de CPU por preset).
pub(super) fn tabela_do_misturador(p: &Preset) -> Vec<[f64; 2]> {
    let custos: Vec<u64> = MOTORES.iter().map(|&m| custo_da_cena(p, m, true)).collect();
    let fios = std::thread::available_parallelism().map_or(8, std::num::NonZero::get) / 2;
    let larguras = em_paralelo_pela_memoria(&custos, fios, |i| {
        let (px, w, h, _) = imagem_da_largura(p, MOTORES[i], true, Relogio::DeUmaVez, 0);
        largura_visivel(&px, w, h)
    });
    MOTORES
        .iter()
        .zip(larguras)
        .map(|(&m, l)| [f64::from(m), f64::from(l)])
        .collect()
}

#[test]
fn a_area_do_blend_cabe_no_circulo() {
    let presets: Vec<Preset> = ficheiros_do_wet()
        .into_iter()
        .map(|(_, b)| ler(&b).unwrap())
        .filter(|p| p.parametros.paint_mode.as_deref() == Some("Blend"))
        .collect();
    assert_eq!(presets.len(), 4);
    let casos: Vec<(usize, f32)> = (0..presets.len())
        .flat_map(|k| [13.0, 26.0, 52.0].map(|r| (k, r)))
        .collect();
    let custos: Vec<u64> = casos
        .iter()
        .map(|&(k, r)| custo_da_cena(&presets[k], r, false))
        .collect();
    let fios = std::thread::available_parallelism().map_or(8, std::num::NonZero::get) / 2;
    let ls = em_paralelo_pela_memoria(&custos, fios, |j| {
        let (k, r) = casos[j];
        let (px, w, h, _) = imagem_da_largura(&presets[k], r, false, Relogio::DeUmaVez, 0);
        largura_visivel(&px, w, h)
    });
    for (&(k, r), l) in casos.iter().zip(ls) {
        let nome = &presets[k].parametros.name;
        assert!(
            l <= FOLGA_DO_CIRCULO * 2.0 * r + 2.0,
            "{nome}: o círculo tem {} px e a área mudada {l}",
            2.0 * r
        );
    }
}
