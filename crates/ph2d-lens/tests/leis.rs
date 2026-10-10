//! As leis que nenhum oráculo instalado mede (o grão é aleatório, o balanço é colorimetria
//! publicada) — gateadas pelas propriedades que as definem.

use ph2d_lens::cubo::{Cubo, ErroCubo};
use ph2d_lens::grade::{self, Grade};
use ph2d_lens::grao::{self, Grao};
use ph2d_lens::vinheta::{self, Vinheta};
use ph2d_lens::{Aberracao, Imagem, Lente, aplica_cpu};

fn neutra() -> Lente {
    Lente {
        aberracao: Aberracao {
            amount: 0.0,
            linear: false,
            angle: 0.0,
        },
        vinheta: Vinheta {
            amount: 0.0,
            midpoint: 1.0,
            roundness: 0.0,
            feather: 0.5,
            center: [0.0; 2],
            cor: [0.0; 3],
        },
        grade: Grade {
            exposure: 0.0,
            temperatura: 0.0,
            matiz: 0.0,
            contraste: 0.0,
            saturacao: 0.0,
            lut_mix: 1.0,
        },
        grao: Grao {
            amount: 0.0,
            size: 1.0,
            response: 0.0,
            colorido: false,
            animado: true,
            semente: 7,
        },
    }
}

fn imagem() -> Imagem {
    let (w, h) = (40, 24);
    let px = (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as f32 / w as f32, (i / w) as f32 / h as f32);
            [x, y, 1.0 - x * y, 1.0]
        })
        .collect();
    Imagem { w, h, px }
}

#[test]
fn a_lente_neutra_devolve_a_imagem_ao_bit() {
    let l = neutra();
    assert!(l.neutra(false));
    let src = imagem();
    assert_eq!(aplica_cpu(&src, &l, None, 3), src);
}

#[test]
fn phi_bate_com_os_valores_tabelados() {
    for (t, v) in [
        (0.0, 0.5),
        (1.0, 0.841_344_7),
        (-1.0, 0.158_655_3),
        (2.0, 0.977_249_9),
        (-3.0, 0.001_349_9),
    ] {
        assert!((vinheta::phi(t) - v).abs() < 2e-7, "phi({t})");
    }
}

#[test]
fn a_vinheta_e_um_dentro_e_zero_longe_e_meio_na_borda_de_uma_elipse_grande() {
    let (semi, s) = ([400.0, 300.0], 4.0);
    assert!((vinheta::mascara([0.0, 0.0], semi, 2.0, s) - 1.0).abs() < 1e-4);
    assert!(vinheta::mascara([460.0, 0.0], semi, 2.0, s) < 1e-4);
    assert!((vinheta::mascara([400.0, 0.0], semi, 2.0, s) - 0.5).abs() < 0.01);
    // O expoente maior (quase retângulo) cobre o canto que a elipse deixa de fora.
    assert!(vinheta::mascara([340.0, 250.0], semi, 8.0, s) > 0.99);
    assert!(vinheta::mascara([340.0, 250.0], semi, 2.0, s) < 0.01);
}

#[test]
fn a_vinheta_negativa_clareia_e_a_positiva_vai_para_a_cor() {
    let mut v = neutra().vinheta;
    v.amount = 1.0;
    v.cor = [0.2, 0.0, 0.0];
    let g = vinheta::geometria(&v, [100.0, 100.0]);
    let canto = vinheta::aplica([0.5; 3], [0.5, 0.5], &v, &g);
    assert!((canto[0] - 0.2).abs() < 0.05 && canto[1] < 0.05);
    v.amount = -1.0;
    let canto = vinheta::aplica([0.5; 3], [0.5, 0.5], &v, &g);
    assert!(canto.iter().all(|c| *c > 0.95));
}

#[test]
fn o_balanco_neutro_e_a_identidade_exacta_e_o_quente_puxa_para_o_vermelho() {
    assert_eq!(
        grade::matriz_balanco(0.0, 0.0),
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
    );
    let m = grade::matriz_balanco(1.0, 0.0);
    let branco = [0, 1, 2].map(|i| m[i][0] + m[i][1] + m[i][2]);
    assert!(branco[0] > branco[1] && branco[1] > branco[2], "{branco:?}");
    let m = grade::matriz_balanco(-1.0, 0.0);
    let branco = [0, 1, 2].map(|i| m[i][0] + m[i][1] + m[i][2]);
    assert!(branco[2] > branco[0], "{branco:?}");
    // O magenta tira verde.
    let m = grade::matriz_balanco(0.0, 1.0);
    let branco = [0, 1, 2].map(|i| m[i][0] + m[i][1] + m[i][2]);
    assert!(branco[1] < branco[0] && branco[1] < branco[2], "{branco:?}");
}

#[test]
fn a_exposicao_dobra_a_luz_e_o_contraste_fixa_o_cinzento_medio() {
    let id = grade::matriz_balanco(0.0, 0.0);
    let mut g = neutra().grade;
    g.exposure = 1.0;
    let c = grade::aplica([0.1, 0.2, 0.3], &g, &id, None);
    assert!(
        (c[0] - 0.2).abs() < 1e-4 && (c[2] - 0.6).abs() < 1e-4,
        "{c:?}"
    );
    g.exposure = 0.0;
    g.contraste = 0.7;
    let c = grade::aplica([grade::PIVO; 3], &g, &id, None);
    assert!(c.iter().all(|v| (v - grade::PIVO).abs() < 1e-4), "{c:?}");
    g.contraste = 0.0;
    g.saturacao = -1.0;
    let c = grade::aplica([0.6, 0.2, 0.1], &g, &id, None);
    assert!(
        (c[0] - c[1]).abs() < 1e-3 && (c[1] - c[2]).abs() < 1e-3,
        "cinzento: {c:?}"
    );
}

#[test]
fn o_grao_e_deterministico_por_tique_e_de_media_zero() {
    let g = Grao {
        amount: 1.0,
        size: 1.0,
        response: 0.0,
        colorido: false,
        animado: true,
        semente: 11,
    };
    let s3 = grao::semente(&g, 3);
    let a = grao::aplica([0.5; 3], [10.5, 4.5], &g, s3);
    assert_eq!(
        a,
        grao::aplica([0.5; 3], [10.5, 4.5], &g, grao::semente(&g, 3)),
        "o scrub devolve o mesmo grão"
    );
    assert_ne!(
        a,
        grao::aplica([0.5; 3], [10.5, 4.5], &g, grao::semente(&g, 4)),
        "o grão animado mexe"
    );
    assert!(a[0] == a[1] && a[1] == a[2], "mono");
    let parado = Grao {
        animado: false,
        ..g
    };
    assert_eq!(grao::semente(&parado, 3), grao::semente(&parado, 900));
    let n: Vec<f32> = (0..64 * 64)
        .map(|i| grao::ruido([(i % 64) as f32 + 0.5, (i / 64) as f32 + 0.5], 1.0, s3, 0))
        .collect();
    let media = n.iter().sum::<f32>() / n.len() as f32;
    let var = n.iter().map(|v| (v - media).powi(2)).sum::<f32>() / n.len() as f32;
    assert!(
        media.abs() < 0.05 && (var - 1.0).abs() < 0.08,
        "media {media} var {var}"
    );
}

#[test]
fn a_resposta_poupa_as_sombras_e_as_luzes() {
    let g = Grao {
        amount: 1.0,
        size: 1.0,
        response: 1.0,
        colorido: false,
        animado: false,
        semente: 1,
    };
    let preto = grao::aplica([0.0; 3], [3.5, 3.5], &g, 1);
    let meio = grao::aplica([0.5; 3], [3.5, 3.5], &g, 1);
    assert_eq!(preto, [0.0; 3]);
    assert!(meio[0] != 0.5);
}

#[test]
fn o_leitor_recusa_o_que_nao_e_uma_lut_3d() {
    assert_eq!(Cubo::ler("LUT_1D_SIZE 4\n"), Err(ErroCubo::So1d));
    assert_eq!(Cubo::ler("0 0 0\n"), Err(ErroCubo::SemTamanho));
    assert_eq!(
        Cubo::ler("LUT_3D_SIZE 1\n"),
        Err(ErroCubo::TamanhoInvalido(1))
    );
    assert_eq!(
        Cubo::ler("LUT_3D_SIZE 2\n0 0 0\n"),
        Err(ErroCubo::Contagem {
            esperado: 8,
            lido: 1
        })
    );
    assert_eq!(
        Cubo::ler("LUT_3D_SIZE 2\nfoo\n"),
        Err(ErroCubo::LinhaInvalida(2))
    );
    let id: String = (0..8)
        .map(|i| format!("{} {} {}\n", i & 1, (i >> 1) & 1, i >> 2))
        .collect();
    let c = Cubo::ler(&format!("TITLE \"x\"\n# nota\nLUT_3D_SIZE 2\n{id}")).unwrap();
    let v = c.tetraedrica([0.25, 0.5, 0.75]);
    assert!((v[0] - 0.25).abs() < 1e-6 && (v[1] - 0.5).abs() < 1e-6 && (v[2] - 0.75).abs() < 1e-6);
}
