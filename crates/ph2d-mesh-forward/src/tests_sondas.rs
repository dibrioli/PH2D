//! ⭐⭐ **As capturas GUARDADAS não guardam o passado** — refazem-se só quando o que as faces veem muda,
//! e um quadro depois de qualquer caminho de edições é, ao byte, o de um desenhista que nasceu agora
//! (gate `quadro_pronto_na_hora`: nada acumulado entre quadros).

use crate::tests::{ID, cena};
use crate::tests_chao_tapa::{PECAS, metal};
use crate::tests_contacto::{LADO, camera_do_blender};
use crate::tests_reflexo::{desenhista, desenhista_em, fosca};
use crate::{Forward, Instancia};

const ALVO: [f32; 3] = [0.0, 0.2, 0.15];

fn objs(desloca: f32) -> Vec<Instancia> {
    PECAS
        .iter()
        .enumerate()
        .map(|(k, (c, _, _))| {
            let mut m = ID;
            m[3][..3].copy_from_slice(c);
            if k == 2 {
                m[3][0] += desloca;
            }
            Instancia {
                malha: k as u64 + 1,
                modelo: m,
            }
        })
        .collect()
}

fn quadro(
    fw: &mut Forward,
    o: &[Instancia],
    mats: &[[f32; ph2d_material::wgsl::PACKED]],
    giro: f32,
    exposicao: f32,
) -> Vec<u8> {
    let de = [
        0.6 * giro.cos() - giro.sin(),
        0.15,
        0.6 * giro.sin() + giro.cos(),
    ];
    let mut c = cena(o, mats, camera_do_blender(de, ALVO, 0.8));
    c.tamanho = (LADO, LADO);
    c.chao = Some(0.0);
    c.caixa_tan = Some(0.47);
    c.exposicao = exposicao;
    fw.quadro(&c).expect("quadro")
}

/// ⭐⭐ **Refazem-se só quando a cena muda** — girar a câmara e mudar a exposição não refazem; mover
/// uma vizinha, trocar um material e subir uma malha refazem.
#[test]
#[ignore = "precisa de aparelho"]
fn as_capturas_so_se_refazem_quando_a_cena_muda() {
    let Some(mut fw) = desenhista() else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    if !fw.tem_brilho() {
        eprintln!("esta placa não desenha Rgba16Float — sem capturas, o gate não corre");
        return;
    }
    let (f, m0) = (fosca(), metal(0.0));
    let mats = [f, m0, f];
    let _ = quadro(&mut fw, &objs(0.0), &mats, 0.0, 0.0);
    let base = fw.sondas_refeitas();
    assert_eq!(base, 1, "o 1.º quadro faz as capturas");
    for k in 1..5 {
        let _ = quadro(&mut fw, &objs(0.0), &mats, 0.3 * k as f32, 0.5 * k as f32);
    }
    assert_eq!(fw.sondas_refeitas(), base, "girar e expor não refazem");
    let _ = quadro(&mut fw, &objs(0.05), &mats, 0.0, 0.0);
    assert_eq!(fw.sondas_refeitas(), base + 1, "mover uma vizinha refaz");
    let _ = quadro(&mut fw, &objs(0.05), &[m0, m0, f], 0.0, 0.0);
    assert_eq!(fw.sondas_refeitas(), base + 2, "trocar um material refaz");
    let (p, n, idx) = crate::tests::esfera(PECAS[2].1);
    let (ao, mat) = (vec![1.0; p.len()], vec![2u32; p.len()]);
    fw.sobe(
        3,
        &crate::Malha {
            posicoes: &p,
            normais: &n,
            ao: &ao,
            material: &mat,
            indices: &idx,
        },
    );
    let _ = quadro(&mut fw, &objs(0.05), &[m0, m0, f], 0.0, 0.0);
    assert_eq!(fw.sondas_refeitas(), base + 3, "subir uma malha refaz");
}

/// ⭐⭐ **Girar PERTO não refaz** — a câmara junto da caixa, com a cena cortada pelo ecrã: o mapa de
/// sombra da VISTA enquadra-se pelo que se vê e muda a cada giro; o das capturas é o da cena inteira.
/// (A prova de mutação de 04/10: enquadrar as capturas pela câmara sobrevivia com a cena toda à vista.)
#[test]
#[ignore = "precisa de aparelho"]
fn girar_perto_nao_refaz_as_capturas() {
    let Some(mut fw) = desenhista() else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    if !fw.tem_brilho() {
        return;
    }
    let mats = [fosca(), metal(0.0), fosca()];
    let o = objs(0.0);
    let perto = |fw: &mut Forward, giro: f32| {
        let de = [
            0.6 * giro.cos() - giro.sin(),
            0.4,
            0.6 * giro.sin() + giro.cos(),
        ];
        let mut c = cena(&o, &mats, camera_do_blender(de, PECAS[0].0, 0.15));
        c.tamanho = (LADO, LADO);
        c.chao = Some(0.0);
        c.caixa_tan = Some(0.47);
        fw.quadro(&c).expect("quadro")
    };
    let a = perto(&mut fw, 0.0);
    let base = fw.sondas_refeitas();
    for k in 1..6 {
        let _ = perto(&mut fw, 0.4 * k as f32);
    }
    assert_eq!(fw.sondas_refeitas(), base, "girar perto refez as capturas");
    assert_eq!(perto(&mut fw, 0.0), a, "a mesma câmara deu outro quadro");
}

/// ⭐⭐⭐ **O quadro depois das edições é o de um desenhista novo, ao byte** — o mesmo desenhista passa
/// por outra pose, outro material e outra câmara; o quadro final é o que um desenhista que nasce agora
/// dá para a mesma cena. Controlo: as capturas existem (refeitas) e mudam a imagem.
#[test]
#[ignore = "precisa de aparelho"]
fn o_quadro_com_capturas_guardadas_e_o_de_um_desenhista_novo() {
    let (Some(mut velho), Some(mut novo)) = (desenhista(), desenhista()) else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let (f, m0, m5) = (fosca(), metal(0.0), metal(0.5));
    let _ = quadro(&mut velho, &objs(0.0), &[f, m5, f], 0.0, 0.0);
    let _ = quadro(&mut velho, &objs(-0.1), &[m0, m0, m0], 0.7, 0.0);
    let _ = quadro(&mut velho, &objs(0.05), &[f, m0, f], 0.7, 0.0);
    let a = quadro(&mut velho, &objs(0.05), &[f, m0, f], 0.0, 0.0);
    let b = quadro(&mut novo, &objs(0.05), &[f, m0, f], 0.0, 0.0);
    assert_eq!(
        a, b,
        "o quadro depende do caminho: as capturas guardaram o passado"
    );
    novo.liga_reflexos(false);
    let sem = quadro(&mut novo, &objs(0.05), &[f, m0, f], 0.0, 0.0);
    if velho.tem_brilho() {
        assert!(
            velho.sondas_refeitas() >= 3,
            "CONTROLO: as capturas refizeram-se"
        );
        assert_ne!(a, sem, "CONTROLO: as capturas mudam a imagem");
    }
}

/// ⭐ **Uma peça sozinha não tem capturas** — e o quadro é, ao byte, o de sem capturas.
#[test]
#[ignore = "precisa de aparelho"]
fn uma_peca_sozinha_nao_tem_capturas() {
    let Some(mut fw) = desenhista() else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let so = &objs(0.0)[1..2];
    let mats = [fosca(), metal(0.0), fosca()];
    let com = quadro(&mut fw, so, &mats, 0.0, 0.0);
    assert_eq!(fw.sondas_refeitas(), 0, "uma peça sozinha fez capturas");
    fw.liga_reflexos(false);
    assert_eq!(quadro(&mut fw, so, &mats, 0.0, 0.0), com);
}

/// ⭐ **As capturas cabem no GLES** — o backend do WebGL2: renderizar numa camada e num nível de uma
/// matriz de texturas, copiar níveis `Rgba16Float`, ler a matriz. O espelho da cena do oráculo, GLES
/// contra o nativo, nos pixels do espelho.
#[test]
#[ignore = "precisa de aparelho GL"]
fn as_capturas_cabem_no_gles() {
    let Some(mut gl) = desenhista_em(wgpu::Backends::GL) else {
        eprintln!("sem adaptador GL nesta máquina — o gate não corre aqui");
        return;
    };
    let Some(mut nativo) = desenhista() else {
        return;
    };
    let mats = [fosca(), metal(0.0), fosca()];
    let a = quadro(&mut gl, &objs(0.0), &mats, 0.0, 0.0);
    let b = quadro(&mut nativo, &objs(0.0), &mats, 0.0, 0.0);
    eprintln!(
        "GLES formato {:?}, capturas refeitas {} · nativo {}",
        gl.formato(),
        gl.sondas_refeitas(),
        nativo.sondas_refeitas()
    );
    if !gl.tem_brilho() {
        eprintln!(
            "o GLES desta máquina não desenha Rgba16Float — sem capturas lá; nada a comparar"
        );
        return;
    }
    assert_eq!(gl.sondas_refeitas(), 1, "o GLES não fez as capturas");
    let (mut n, mut s, mut pior) = (0usize, 0u64, 0u8);
    for (x, y) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
        if x[3] == 255 && y[3] == 255 {
            let d = x[1].abs_diff(y[1]);
            s += u64::from(d);
            pior = pior.max(d);
            n += 1;
        }
    }
    eprintln!(
        "{n} px · |Δ| médio {:.3}/255 · máx {pior}",
        s as f64 / n as f64
    );
    assert!(n > 10_000, "a cena mal aparece");
    assert!((s as f64 / n as f64) < 1.0, "o GLES afastou-se do nativo");
}
