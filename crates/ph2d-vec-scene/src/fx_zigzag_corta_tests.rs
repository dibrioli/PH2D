use super::*;
use crate::fx_zigzag::{ZigZagSpec, onda, zigzag_contour};

fn quinas(p: &[[f64; 2]]) -> Vec<VecVertex> {
    p.iter().map(|&a| VecVertex::corner(a)).collect()
}

/// **G34 — o nó do Zig Zag num vale é cortado (lei C do dono).** A nuvem da foto
/// `img_F40/zigzag_opcoes.png`, nas duas forças medidas (a lei A tinha 1 cruzamento): nenhum fica, e o
/// corte só tira o laço — o contorno perde menos de 5 % do comprimento.
#[test]
fn o_no_do_zigzag_num_vale_e_cortado() {
    let k = crate::ShapeKind::Cloud;
    let forma = crate::cook(k, [0.0, 0.0], [120.0, 80.0], &k.defaults());
    let r = crate::effect::FxCtx::of(&forma).ref_size;
    for (size, smooth) in [(5.0, true), (12.0, false)] {
        let spec = ZigZagSpec {
            amplitude: size,
            ridges: 10.0,
            smooth,
            ..ZigZagSpec::default()
        };
        let cheio = onda(&forma.verts, true, &spec, r, None).unwrap();
        assert!(
            cruzamentos(&cheio, true) > 0,
            "Size {size}: a fixtura já não tem o nó"
        );
        let (v, closed) = zigzag_contour(&forma.verts, true, &spec, r, None);
        assert!(closed);
        assert_eq!(cruzamentos(&v, true), 0, "Size {size}: o laço ficou");
        let (_, antes) = achata(&cheio, cheio.len());
        let (_, depois) = achata(&v, v.len());
        assert!(
            depois < antes && depois >= (1.0 - FRACAO) * antes,
            "Size {size}: {antes} → {depois}"
        );
    }
}

/// O laço pequeno de um traço aberto sai pelo cruzamento `(3, 0)`; os vértices dele saem também.
#[test]
fn o_laco_pequeno_sai_pelo_ponto_do_cruzamento() {
    let v = quinas(&[[0.0, 0.0], [4.0, 0.0], [3.0, 1.0], [3.0, -2.0], [40.0, 0.0]]);
    assert_eq!(cruzamentos(&v, false), 1);
    let c = sem_nos(v, false);
    let ancoras: Vec<[f64; 2]> = c.iter().map(|v| v.anchor).collect();
    assert_eq!(ancoras.len(), 4, "{ancoras:?}");
    assert_eq!(ancoras[0], [0.0, 0.0]);
    assert!(
        (ancoras[1][0] - 3.0).abs() < 1e-9 && ancoras[1][1].abs() < 1e-9,
        "{ancoras:?}"
    );
    assert_eq!(&ancoras[2..], &[[3.0, -2.0], [40.0, 0.0]]);
    assert_eq!(cruzamentos(&c, false), 0);
}

/// O laço maior que um quarto do contorno é a forma a dobrar: fica.
#[test]
fn o_laco_grande_fica() {
    let v = quinas(&[[0.0, 0.0], [4.0, 0.0], [3.0, 1.0], [3.0, -2.0], [6.0, 0.0]]);
    assert_eq!(sem_nos(v.clone(), false), v);
}
