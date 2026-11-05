//! Export As e Open: cada formato volta igual pelo nosso leitor, e a escala bate com o `--scale` dele.
//! (`Pillow` e o LibreSprite leem os mesmos ficheiros: `gif_apng_webp_para_os_de_fora`.)

use ph2d_imageio_png::PixelPng;
use ph2d_pixel_core::{ColorMode, Sprite};
use ph2d_pixel_io::abrir::{Grelha, abrir, abrir_sequencia, celulas, importar_folha};
use ph2d_pixel_io::ase;
use ph2d_pixel_io::gif_cores;
use ph2d_pixel_io::render::{Camadas, Direcao, Pedido, Quadros, composto, escalado, ordem};
use ph2d_pixel_io::saida::{AvisoSaida, Formato, exportar};
use ph2d_pixel_io::sheet::{OpcoesFolha, Tipo, montar};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn oraculo() -> Value {
    serde_json::from_str(include_str!("fixtures/oracle_export.json")).unwrap()
}

fn unhex(h: &str) -> Vec<u8> {
    (0..h.len() / 2)
        .map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap())
        .collect()
}

fn doc(o: &Value, n: &str) -> Sprite {
    ase::ler(&unhex(o["entradas_hex"][n].as_str().unwrap()))
        .unwrap()
        .0
}

fn limpo(mut px: Vec<u8>) -> Vec<u8> {
    for c in px.as_chunks_mut::<4>().0 {
        if c[3] == 0 {
            *c = [0; 4];
        }
    }
    px
}

fn q(s: &Sprite, i: usize) -> Vec<u8> {
    composto(s, s.frames()[i].id, Camadas::Visiveis)
}

fn pedido() -> Pedido {
    Pedido {
        escala: 1.0,
        ..Pedido::default()
    }
}

#[test]
fn a_escala_bate_com_o_libresprite() {
    let o = oraculo();
    let s = doc(&o, "folha");
    let f = montar(
        &s,
        &OpcoesFolha {
            tipo: Tipo::Horizontal,
            ..OpcoesFolha::default()
        },
        "t",
        "ase",
    );
    for (nome, fator) in [("scale2", 2.0), ("scale05", 0.5)] {
        let (w, h, px) = escalado(&f.rgba, f.w, f.h, fator);
        let c = &o["escalas"][nome]["sheet"];
        assert_eq!(
            (u64::from(w), u64::from(h)),
            (c["w"].as_u64().unwrap(), c["h"].as_u64().unwrap()),
            "{nome}"
        );
        // A folha DELE é a dos quadros escalados um a um, lado a lado.
        let _ = px;
        let qs = ph2d_pixel_io::render::quadros(
            &s,
            &Pedido {
                escala: fator,
                ..Pedido::default()
            },
        );
        let (qw, qh) = (qs[0].w as usize, qs[0].h as usize);
        let mut folha = vec![0_u8; w as usize * h as usize * 4];
        for (i, q) in qs.iter().enumerate() {
            for y in 0..qh {
                let d = (y * w as usize + i * qw) * 4;
                folha[d..d + qw * 4].copy_from_slice(&q.rgba[y * qw * 4..(y + 1) * qw * 4]);
            }
        }
        let hex: String = Sha256::digest(limpo(folha))
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        // ⛔ Recusa medida: ao reduzir, ele escala cada CEL antes de compor (nenhum deslocamento único
        // explica o resultado). Aqui sai o composto que o artista vê; só o tamanho é comparável.
        if fator >= 1.0 {
            assert_eq!(hex, c["rgba_sha256"].as_str().unwrap(), "{nome}");
        }
    }
}

#[test]
fn a_direcao_nao_repete_as_pontas() {
    let o = oraculo();
    let s = doc(&o, "folha");
    let t = Quadros::Tag(1); // walk, 2..=4
    assert_eq!(ordem(&s, t, Direcao::Forward), [2, 3, 4]);
    assert_eq!(ordem(&s, t, Direcao::Reverse), [4, 3, 2]);
    assert_eq!(ordem(&s, t, Direcao::PingPong), [2, 3, 4, 3]);
    assert_eq!(ordem(&s, t, Direcao::PingPongReverse), [4, 3, 2, 3]);
    assert_eq!(
        ordem(&s, Quadros::Intervalo { de: 3, ate: 1 }, Direcao::Forward),
        [1, 2, 3]
    );
}

#[test]
fn gif_apng_webp_voltam_iguais() {
    let o = oraculo();
    let s = doc(&o, "plano");
    for (f, ext) in [(Formato::Apng, "apng"), (Formato::Webp, "webp")] {
        let r = exportar(&s, &pedido(), f, 0).unwrap();
        let (d, _) = abrir(&r.ficheiros[0].1, ext).unwrap();
        assert_eq!(d.frames().len(), 5, "{ext}");
        let ms: Vec<_> = d.frames().iter().map(|x| x.duration_ms).collect();
        assert_eq!(ms, [100, 150, 70, 333, 20], "{ext}");
        for i in 0..5 {
            assert_eq!(limpo(q(&d, i)), limpo(q(&s, i)), "{ext}, quadro {i}");
        }
    }
    let ix = doc(&o, "indexado");
    let r = exportar(&ix, &pedido(), Formato::Gif, 0).unwrap();
    // A paleta tem uma cor a meio alfa: o aviso é dele, e nenhuma cor se perdeu.
    assert_eq!(r.avisos, [AvisoSaida::GifAlfa]);
}

#[test]
fn o_gif_de_poucas_cores_sai_identico() {
    let o = oraculo();
    let s = doc(&o, "duracoes");
    let r = exportar(&s, &pedido(), Formato::Gif, 0).unwrap();
    assert!(r.avisos.is_empty());
    let (d, _) = abrir(&r.ficheiros[0].1, "gif").unwrap();
    assert_eq!(d.frames().len(), 4);
    for i in 0..4 {
        assert_eq!(q(&d, i), q(&s, i), "quadro {i}");
    }
    // 7 → 10 ms, nunca 0; 15 → 20; 333 → 330; 1004 → 1000.
    let ms: Vec<_> = d.frames().iter().map(|x| x.duration_ms).collect();
    assert_eq!(ms, [10, 20, 330, 1000]);
}

#[test]
fn o_gif_com_cores_demais_reduz_avisa_e_nao_pontilha() {
    let o = oraculo();
    let s = doc(&o, "muitas_cores");
    let r = exportar(&s, &pedido(), Formato::Gif, 0).unwrap();
    assert_eq!(r.avisos, [AvisoSaida::GifReduziu(1024)]);
    let (d, _) = abrir(&r.ficheiros[0].1, "gif").unwrap();
    // Sem pontilhado: a mesma cor de entrada dá sempre a mesma de saída.
    let (a, b) = (q(&s, 0), q(&d, 0));
    let mut mapa = std::collections::BTreeMap::new();
    for (x, y) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
        assert_eq!(*mapa.entry(*x).or_insert(*y), *y);
    }
    let cores: std::collections::BTreeSet<_> = b.as_chunks::<4>().0.iter().collect();
    assert!(cores.len() <= 256, "{}", cores.len());
    assert!(
        cores.len() > 200,
        "o corte desperdiçou a paleta: {}",
        cores.len()
    );
}

#[test]
fn o_alfa_do_gif_corta_como_o_libresprite() {
    let o = oraculo();
    let s = doc(&o, "alfa_parcial");
    let r = exportar(&s, &pedido(), Formato::Gif, 0).unwrap();
    assert_eq!(r.avisos, [AvisoSaida::GifAlfa]);
    let (d, _) = abrir(&r.ficheiros[0].1, "gif").unwrap();
    let dele = unhex(
        o["saidas"]["alfa_parcial"]["gif"]["info"]["first_rgba_hex"]
            .as_str()
            .unwrap(),
    );
    assert_eq!(limpo(q(&d, 0)), limpo(dele));
}

#[test]
fn o_png_guarda_o_modo_e_a_paleta() {
    let o = oraculo();
    let ix = doc(&o, "um_quadro_indexado");
    let r = exportar(&ix, &pedido(), Formato::Png, 0).unwrap();
    assert_eq!(r.ficheiros.len(), 1);
    let (_, _, modo, _) = ph2d_imageio_png::decode_pixel(&r.ficheiros[0].1).unwrap();
    let PixelPng::Indexed(p) = modo else {
        panic!("{modo:?}")
    };
    assert_eq!(p, ix.palette(), "a tabela inteira, com o alfa em tRNS");
    let (d, _) = abrir(&r.ficheiros[0].1, "png").unwrap();
    assert_eq!(d.mode(), ColorMode::Indexed);
    assert_eq!(limpo(q(&d, 0)), limpo(q(&ix, 0)));

    let g = doc(&o, "cinza");
    let r = exportar(&g, &pedido(), Formato::Png, 0).unwrap();
    let (d, _) = abrir(&r.ficheiros[0].1, "png").unwrap();
    assert_eq!(d.mode(), ColorMode::Gray);
    assert_eq!(limpo(q(&d, 0)), limpo(q(&g, 0)));
}

#[test]
fn varios_quadros_em_png_saem_em_sequencia_e_voltam() {
    let o = oraculo();
    let s = doc(&o, "plano");
    let r = exportar(&s, &pedido(), Formato::Png, 0).unwrap();
    let suf: Vec<_> = r.ficheiros.iter().map(|f| f.0.as_str()).collect();
    assert_eq!(
        suf,
        ["1", "2", "3", "4", "5"],
        "como o LibreSprite: nome1.png…"
    );
    let pngs: Vec<_> = r.ficheiros.iter().map(|f| f.1.clone()).collect();
    let d = abrir_sequencia(&pngs).unwrap();
    for i in 0..5 {
        assert_eq!(limpo(q(&d, i)), limpo(q(&s, i)), "quadro {i}");
    }
}

#[test]
fn a_folha_importada_volta_quadro_a_quadro() {
    let o = oraculo();
    let s = doc(&o, "plano");
    let f = montar(
        &s,
        &OpcoesFolha {
            tipo: Tipo::Rows,
            limite: ph2d_pixel_io::sheet::Limite::Colunas(2),
            shape: 3,
            border: 2,
            ..OpcoesFolha::default()
        },
        "t",
        "png",
    );
    let g = Grelha {
        x: 2,
        y: 2,
        w: 24,
        h: 16,
        gap_x: 3,
        gap_y: 3,
        por_colunas: false,
    };
    assert_eq!(celulas(&g, f.w, f.h), (2, 3));
    let d = importar_folha(&f.rgba, f.w, f.h, &g).unwrap();
    assert_eq!(d.frames().len(), 6, "a célula vazia do fim também é quadro");
    for i in 0..5 {
        assert_eq!(limpo(q(&d, i)), limpo(q(&s, i)), "quadro {i}");
    }
}

#[test]
fn abrir_o_que_nao_se_le_diz_o_que_e() {
    assert!(abrir(b"qualquer", "psd").is_err());
    assert!(abrir(b"qualquer", "ase").is_err());
    assert!(abrir(b"qualquer", "png").is_err());
}

#[test]
fn o_corte_mediano_e_deterministico() {
    let px: Vec<u8> = (0..4096_u32)
        .flat_map(|i| [(i % 256) as u8, (i / 16) as u8, (i * 7 % 256) as u8, 255])
        .collect();
    let a = gif_cores::reduzir(&[&px]);
    let b = gif_cores::reduzir(&[&px]);
    assert_eq!(a, b);
    assert!(a.paleta.len() <= 256);
}

/// Grava cada formato numa pasta, para o Pillow e o LibreSprite os lerem (`PH2D_SAIDA_DIR`).
#[test]
#[ignore = "ferramenta: PH2D_SAIDA_DIR=<pasta>"]
fn gif_apng_webp_para_os_de_fora() {
    let dir = std::env::var("PH2D_SAIDA_DIR").unwrap();
    let o = oraculo();
    for n in ["plano", "duracoes", "muitas_cores", "um_quadro_indexado"] {
        let s = doc(&o, n);
        for (f, e) in [
            (Formato::Gif, "gif"),
            (Formato::Apng, "apng"),
            (Formato::Webp, "webp"),
            (Formato::Ase, "ase"),
        ] {
            let r = exportar(&s, &pedido(), f, 0).unwrap();
            std::fs::write(format!("{dir}/{n}.{e}"), &r.ficheiros[0].1).unwrap();
        }
    }
}
