//! ⭐⭐ **Os gates do REFINO LOCAL do bake** (A13) — na barra `0` da `=6`, pela porta do produto.

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
/// Sem o refino o ajuste corta a tampa por uma corda: a imagem EXACTA do contorno fica até
/// `0,2663`–`0,2898` fora do desenho e `6`–`8` células onde ela pinta ficam sem cor (o CONTROLO).
/// Com o refino pelos pesos (`passo = 0,025`, pesos interpolados; o `fecha` que recusa o bico), o
/// afastamento medido (`0,0090` · `0,0067` · `0,0062` · `0,0082`) mais a folga que o próprio ajuste aceita (`2 ×` a tolerância, a do `fecha`), e nenhuma
/// célula a faltar — só a cancelação de camadas de sinais opostos (grau `0`) fica, e essa é a
/// verdade.
#[test]
fn a_tampa_da_junta_dobrada_nao_perde_cor() {
    let refino = ph2d_vec_skin::curva::REFINO_DO_PRODUTO;
    for ((g1, g2), medido) in [
        ((170.0, 110.0), 0.0090),
        ((170.0, 140.0), 0.0067),
        ((170.0, 150.0), 0.0062),
        ((170.0, 170.0), 0.0082),
    ] {
        let (sim, scene, st, ids) = cena(g1, g2);
        let b = Barra::de(&sim, &st, ids[0]);
        let d = ph2d_skeleton_live::skin_live::recook_desenhando(&sim, &mut scene.clone());
        let forma = &d.get(&ids[0]).expect("a barra 0 tem desenho").forma;
        assert_eq!(
            bits(forma),
            bits(&b.assa(refino)),
            "{g1}/{g2}: a régua deixou de medir o que o produto desenha"
        );
        let img = b.imagem(256);
        let (gap, falta) = regua(&img, forma, b.t);
        let (gap0, falta0) = regua(&img, &b.assa(None), b.t);
        println!("  {g1}/{g2}: afastamento {gap:.4} ({falta}) · sem o refino {gap0:.4} ({falta0})");
        assert!(
            gap0 > 0.25 && falta0 > 0,
            "{g1}/{g2}: sem o refino o afastamento é {gap0} ({falta0} células) — a fixtura deixou \
             de conter a tampa cortada"
        );
        let tecto = medido + 2.0 * b.tolerancia();
        assert!(
            gap <= tecto && falta == 0,
            "{g1}/{g2}: a imagem exacta fica {gap} fora do desenho (tecto {tecto}) e {falta} \
             células onde ela pinta ficam sem cor — a tampa da junta voltou a ser cortada"
        );
    }
}
