use super::*;

fn cores() -> Cores {
    Cores::do_tema(Theme::Dark)
}

/// Cada figura desenhada tem o bico opaco no ponto quente e cresce com a escala (HiDPI).
#[test]
fn cada_figura_tem_o_bico_no_ponto_quente() {
    for f in Figura::TODAS {
        for escala in [1.0, 2.0] {
            let im = desenha(f, escala, &cores());
            assert_eq!(f64::from(im.lado), (24.0 * escala).ceil(), "{f:?}");
            let (x, y) = im.quente;
            assert!(im.alfa(x, y) > 128, "{f:?} ×{escala}: o bico não é opaco");
            assert_eq!(
                im.alfa(im.lado - 1, 0),
                0,
                "{f:?}: o canto de cima à direita é vazio"
            );
        }
    }
}

/// As figuras são DIFERENTES umas das outras (o selo muda o desenho), menos a mão, que é do sistema.
#[test]
fn as_figuras_desenhadas_sao_distintas() {
    let v: Vec<_> = Figura::TODAS
        .into_iter()
        .filter(|f| *f != Figura::Agarrando)
        .map(|f| desenha(f, 1.0, &cores()).rgba)
        .collect();
    for i in 0..v.len() {
        for j in i + 1..v.len() {
            assert_ne!(v[i], v[j], "{i} e {j} são iguais");
        }
    }
    assert_eq!(Figura::Agarrando.do_sistema(), CursorIcon::Grabbing);
}

/// O selo do «−» é da cor de perigo e o do «+» do realce: o centro do selo diz o que o clique faz.
#[test]
fn o_selo_tem_a_cor_do_que_o_clique_faz() {
    let c = cores();
    let px = |f| {
        let im = desenha(f, 1.0, &c);
        let i = (18 * usize::from(im.lado) + 17) * 4;
        [im.rgba[i], im.rgba[i + 1], im.rgba[i + 2]]
    };
    let rgb = |k: [u8; 4]| [k[0], k[1], k[2]];
    assert_eq!(px(Figura::PenaMais), rgb(c.realce));
    assert_eq!(px(Figura::PenaMenos), rgb(c.perigo));
}

/// `PH2D_CURSOR_DUMP=<pasta>`: grava cada figura em RGBA cru (`<nome>_<escala>x.rgba`) para olhar.
#[test]
fn grava_as_figuras_para_olhar() {
    let Some(dir) = std::env::var_os("PH2D_CURSOR_DUMP") else {
        return;
    };
    for f in Figura::TODAS {
        for e in [1, 2] {
            let im = desenha(f, f64::from(e), &cores());
            let p = std::path::Path::new(&dir).join(format!("{f:?}_{e}x.rgba"));
            std::fs::write(p, &im.rgba).expect("grava");
        }
    }
}
