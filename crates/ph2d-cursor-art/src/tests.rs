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

/// As cores de repouso das alças da Timeline (as que `alca_estado` publica), com o nome.
fn cores_das_alcas(theme: Theme) -> Vec<(&'static str, [u8; 4])> {
    use ph2d_tokens::TimelineTinta as T;
    let c = |k: ph2d_tokens::Color| [k.r, k.g, k.b, 255];
    vec![
        ("esticar", c(ColorToken::Success.resolve(theme))),
        ("cortar", c(ColorToken::Danger.resolve(theme))),
        ("fade / fim", c(ColorToken::TimelinePlayhead.resolve(theme))),
        ("laco", c(T::LacoDaRegua.resolve(theme))),
        ("chaveta", c(ColorToken::TimelineLoopBrace.resolve(theme))),
        ("marcador", c(ColorToken::TimelineMarker.resolve(theme))),
        ("chave", c(T::Chave.resolve(theme))),
    ]
}

const ALCA: [Figura; 3] = [Figura::Estica, Figura::Mao, Figura::MaoFechada];

/// ⭐ A figura tingida: o ponto quente (miolo do corpo) é a cor da alça, ao bit, e o anel à volta do
/// corpo é o contorno escuro (`Bg0`) — lê-se sobre qualquer fundo. Em toda escala.
#[test]
fn o_corpo_tem_a_cor_da_alca_e_o_halo_e_escuro() {
    let base = cores();
    for (nome, cor) in cores_das_alcas(Theme::Dark) {
        for f in ALCA {
            for escala in [1.0, 2.0] {
                let im = desenha(f, escala, &base.com_corpo(cor));
                let px = |x: u16, y: u16| {
                    let i = (usize::from(y) * usize::from(im.lado) + usize::from(x)) * 4;
                    [im.rgba[i], im.rgba[i + 1], im.rgba[i + 2], im.rgba[i + 3]]
                };
                let (qx, qy) = im.quente;
                assert_eq!(px(qx, qy), cor, "{nome} {f:?} ×{escala}: o corpo");
                // Da esquerda para o centro há um píxel opaco PURO do contorno (o halo de 1,5 px
                // lógicos tem miolo puro a partir de ×2) antes do corpo; a ×1 o halo é só a borda.
                let k = base.contorno;
                let halo = (0..qx).find(|&x| px(x, qy) == k);
                if escala >= 2.0 {
                    let x = halo.unwrap_or_else(|| panic!("{nome} {f:?}: sem halo Bg0"));
                    assert!(
                        (x..qx).any(|x| px(x, qy) == cor),
                        "{nome} {f:?}: corpo depois"
                    );
                }
                let primeiro = (0..qx).find(|&x| px(x, qy)[3] > 0).expect("visível");
                assert_ne!(
                    px(primeiro, qy)[..3],
                    cor[..3],
                    "{nome} {f:?}: a borda é halo"
                );
            }
        }
    }
}

/// A alça com cor pede a figura tingida; sem cor (alvo neutro, `PH2D_CARDS=0`) ou fora de `↔` e
/// mãos, o ícone do sistema fica.
#[test]
fn so_a_alca_com_cor_troca_o_icone_do_sistema() {
    let cor = Some([1, 2, 3, 255]);
    assert_eq!(
        Figura::da_alca(CursorIcon::EwResize, cor),
        Err((Figura::Estica, cor))
    );
    assert_eq!(
        Figura::da_alca(CursorIcon::Grab, cor),
        Err((Figura::Mao, cor))
    );
    assert_eq!(
        Figura::da_alca(CursorIcon::Grabbing, cor),
        Err((Figura::MaoFechada, cor))
    );
    for i in [CursorIcon::EwResize, CursorIcon::Grab, CursorIcon::Grabbing] {
        assert_eq!(Figura::da_alca(i, None), Ok(i), "{i:?} sem cor");
    }
    assert_eq!(
        Figura::da_alca(CursorIcon::NsResize, cor),
        Ok(CursorIcon::NsResize)
    );
    for f in ALCA {
        assert_eq!(Figura::da_alca(f.do_sistema(), cor), Err((f, cor)));
    }
}

/// O jogo dos tingidos: o mesmo pedido devolve o MESMO cursor (sem recriar), o pedido só se cria no
/// `cumpre`, e o tecto despeja o mais velho.
#[test]
fn o_cache_devolve_o_mesmo_e_tem_tecto() {
    let mut t = Tingidos::<u32>::default();
    let k = (Figura::Mao, [10, 20, 30, 255]);
    assert_eq!(t.pega(k), None, "ainda por criar");
    let mut criados = 0;
    t.cumpre(|_| {
        criados += 1;
        Some(criados)
    });
    assert_eq!(t.pega(k), Some(1));
    assert_eq!(t.pega(k), Some(1), "o mesmo, sem recriar");
    t.cumpre(|_| unreachable!("nada pedido"));
    for i in 0..=TECTO_DE_TINGIDOS {
        let _ = t.pega((Figura::Estica, [i as u8, 0, 0, 255]));
        t.cumpre(|_| Some(100 + i as u32));
    }
    assert_eq!(t.len(), TECTO_DE_TINGIDOS);
    assert_eq!(t.pega(k), None, "o mais velho saiu");
}

/// `PH2D_CURSOR_PNG=<ficheiro>`: a folha dos cursores tingidos (uma linha por cor de alça, as três
/// figuras ×2) sobre um fundo claro e um escuro, para olhar.
#[test]
fn grava_a_folha_dos_cursores_tingidos() {
    let Some(saida) = std::env::var_os("PH2D_CURSOR_PNG") else {
        return;
    };
    let base = cores();
    let linhas = cores_das_alcas(Theme::Dark);
    let lado = 48_u32;
    let (w, h) = (lado * 6, lado * linhas.len() as u32);
    let fundo = |x: u32| {
        if x < lado * 3 {
            [228, 230, 236, 255]
        } else {
            [22, 24, 32, 255]
        }
    };
    let mut img = image::RgbaImage::from_fn(w, h, |x, _| image::Rgba(fundo(x)));
    for (j, (_, cor)) in linhas.iter().enumerate() {
        for (i, f) in ALCA.iter().chain(ALCA.iter()).enumerate() {
            let im = desenha(*f, 2.0, &base.com_corpo(*cor));
            for y in 0..u32::from(im.lado) {
                for x in 0..u32::from(im.lado) {
                    let k = ((y * u32::from(im.lado) + x) * 4) as usize;
                    let a = f32::from(im.rgba[k + 3]) / 255.0;
                    let (px, py) = (i as u32 * lado + x, j as u32 * lado + y);
                    let d = img.get_pixel_mut(px, py);
                    for c in 0..3 {
                        let v = f32::from(im.rgba[k + c]) * a + f32::from(d[c]) * (1.0 - a);
                        d[c] = v.round() as u8;
                    }
                }
            }
        }
    }
    img.save(saida).expect("grava o png");
}
