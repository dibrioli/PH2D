//! ⭐⭐ **O gate da TAMPA da junta** (A13) — na barra `0` da `=6`, pela porta do produto.

use super::*;

/// Os bits de todos os nós de um caminho, contorno a contorno.
fn bits(p: &ph2d_vec_scene::VecPath) -> Vec<u64> {
    let mut v = Vec::new();
    for c in 0..p.contour_count() {
        if let Some((vs, fechado)) = p.contour(c) {
            v.push(u64::from(fechado));
            for x in vs {
                for q in [x.anchor, x.in_handle, x.out_handle] {
                    v.extend(q.map(f64::to_bits));
                }
            }
        }
    }
    v
}

/// ⭐⭐⭐ **GATE — A13: a tampa de fora de uma junta a `170°` não perde cor.**
///
/// O produto (meio-ângulo, `AMOSTRAS_POR_FORMA` uniformes): a imagem EXACTA fica a menos do medido
/// mais a folga do ajuste (`2 ×` a tolerância) do desenho, e nenhuma célula onde ela pinta fica sem
/// cor. ⛔ O CONTROLO é a média em CÍRCULO com metade das amostras (a lei e o orçamento de antes da
/// A13): a volta concentra-se na junta, o ajuste corta a tampa por uma corda (`0,2663`–`0,2898`) e há
/// `6`–`8` células sem cor. (Na amostragem do produto o círculo ainda fica a `0,092`–`0,132`.)
#[test]
fn a_tampa_da_junta_dobrada_nao_perde_cor() {
    use ph2d_skeleton::MisturaDoAngulo;
    let p = ph2d_skeleton_live::skin_desenho::AMOSTRAS_POR_FORMA;
    for ((g1, g2), medido) in [
        ((170.0, 110.0), MEDIDO[0]),
        ((170.0, 140.0), MEDIDO[1]),
        ((170.0, 150.0), MEDIDO[2]),
        ((170.0, 170.0), MEDIDO[3]),
    ] {
        let (sim, scene, st, ids) = cena(g1, g2);
        let b = Barra::de(&sim, &st, ids[0]);
        let d = ph2d_skeleton_live::skin_live::recook_desenhando(&sim, &mut scene.clone());
        let forma = &d.get(&ids[0]).expect("a barra 0 tem desenho").forma;
        assert_eq!(
            bits(forma),
            bits(&b.assa(p)),
            "{g1}/{g2}: a régua deixou de medir o que o produto desenha"
        );
        let (gap, falta) = regua(&b.imagem(256), forma, b.t);
        let c = b.com_lei(MisturaDoAngulo::Circulo);
        let (gap0, falta0) = regua(&c.imagem(256), &c.assa(p / 2), b.t);
        println!("  {g1}/{g2}: afastamento {gap:.4} ({falta}) · círculo {gap0:.4} ({falta0})");
        assert!(
            gap0 > 0.25 && falta0 > 0,
            "{g1}/{g2}: com a média em círculo o afastamento é {gap0} ({falta0} células) — a \
             fixtura deixou de conter a tampa cortada"
        );
        let tecto = medido + 2.0 * b.tolerancia();
        assert!(
            gap <= tecto && falta == 0,
            "{g1}/{g2}: a imagem exacta fica {gap} fora do desenho (tecto {tecto}) e {falta} \
             células onde ela pinta ficam sem cor — a tampa da junta voltou a ser cortada"
        );
    }
}

/// O afastamento MEDIDO do produto em `170/{110,140,150,170}`.
const MEDIDO: [f64; 4] = [0.0067, 0.0077, 0.0080, 0.0080];
