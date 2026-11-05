//! O `.ase`: volta igual, e o composto bate com o LibreSprite corrido (`oraculo_export.py`).

use ph2d_pixel_core::{ColorMode, FrameId, Sprite, compose_picked};
use ph2d_pixel_io::ase;
use serde_json::Value;

fn oraculo() -> Value {
    serde_json::from_str(include_str!("fixtures/oracle_export.json")).unwrap()
}

fn unhex(h: &str) -> Vec<u8> {
    (0..h.len() / 2)
        .map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap())
        .collect()
}

fn entrada(o: &Value, n: &str) -> Vec<u8> {
    unhex(o["entradas_hex"][n].as_str().unwrap())
}

fn quadro(s: &Sprite, f: FrameId) -> Vec<u8> {
    compose_picked(s, f, &|l| s.is_shown(l.id) && !l.reference)
}

/// O RGB de uma célula de alfa 0 não se vê, e os dois divergem nele sem consequência.
fn sem_cor_no_buraco(mut px: Vec<u8>) -> Vec<u8> {
    for c in px.as_chunks_mut::<4>().0 {
        if c[3] == 0 {
            *c = [0; 4];
        }
    }
    px
}

#[test]
fn o_composto_bate_com_o_libresprite_em_cada_quadro() {
    let o = oraculo();
    for n in o["quadros"].as_object().unwrap().keys() {
        let (s, _) = ase::ler(&entrada(&o, n)).unwrap();
        let q = o["quadros"][n].as_array().unwrap();
        assert_eq!(q.len(), s.frames().len(), "{n}");
        for (i, (f, q)) in s.frames().iter().zip(q).enumerate() {
            let meu = sem_cor_no_buraco(quadro(&s, f.id));
            let dele = sem_cor_no_buraco(unhex(q["rgba_hex"].as_str().unwrap()));
            assert_eq!(
                q["w"].as_u64().unwrap() * q["h"].as_u64().unwrap() * 4,
                dele.len() as u64
            );
            // A folha DELE vem recortada só se o sprite for maior; aqui vem inteira.
            let w = q["w"].as_u64().unwrap() as usize;
            // ⛔ Recusa medida: modo de mistura sobre destino TRANSPARENTE. O LibreSprite dá preto; o
            // Trial (P3, `g*.ase`, 6 144/6 144) dá a fonte, e a lei é a do Trial. Só se ignora ali.
            let fundo = compose_picked(&s, f.id, &|l| l.name == "fundo");
            let misturadas = s.layers().iter().any(|l| l.blend.code() != 0);
            let sobre_vazio: Vec<bool> = fundo
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| misturadas && c[3] == 0)
                .collect();
            let dif: Vec<_> = meu
                .as_chunks::<4>()
                .0
                .iter()
                .zip(dele.as_chunks::<4>().0)
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .filter(|(k, _)| !sobre_vazio[*k])
                .map(|(k, (a, b))| (k % w, k / w, *a, *b))
                .collect();
            assert!(dif.is_empty(), "{n}, quadro {i}: {} células; {:?}", dif.len(), &dif[..dif.len().min(8)]);
        }
    }
}

#[test]
fn o_ase_nosso_volta_igual() {
    let o = oraculo();
    for n in o["entradas_hex"].as_object().unwrap().keys() {
        let (a, _) = ase::ler(&entrada(&o, n)).unwrap();
        let (bytes, avisos) = ase::escrever(&a).unwrap();
        assert!(avisos.is_empty(), "{n}: {avisos:?}");
        let (b, _) = ase::ler(&bytes).unwrap();
        assert_eq!(a, b, "{n}");
        let (de_novo, _) = ase::escrever(&b).unwrap();
        assert_eq!(bytes, de_novo, "{n}: não é determinístico");
    }
}

#[test]
fn o_documento_guarda_o_que_o_ficheiro_diz() {
    let o = oraculo();
    let (s, _) = ase::ler(&entrada(&o, "completo")).unwrap();
    let nomes: Vec<_> = s.layers().iter().map(|l| l.name.as_str()).collect();
    assert_eq!(nomes, ["fundo", "grp", "mult", "tela", "oculta"]);
    let (grp, mult) = (&s.layers()[1], &s.layers()[2]);
    assert!(grp.is_group());
    assert_eq!(mult.parent, Some(grp.id));
    assert_eq!(mult.opacity, 128);
    assert_eq!(mult.blend.code(), 1);
    assert!(!s.layers()[4].visible);
    let ms: Vec<_> = s.frames().iter().map(|f| f.duration_ms).collect();
    assert_eq!(ms, [100, 150, 70, 333, 20]);
    let t: Vec<_> = s.tags().iter().map(|t| (t.name.as_str(), t.from, t.to, t.repeat)).collect();
    assert_eq!(t, [("idle", 0, 1, 0), ("walk", 2, 4, 2), ("back", 1, 3, 3)]);
    // O ligado continua ligado, o dono é o do quadro 1.
    let c = s.entry((s.layers()[0].id, s.frames()[2].id)).unwrap();
    assert_eq!(c.link, Some(s.frames()[1].id));
    let (ix, _) = ase::ler(&entrada(&o, "indexado")).unwrap();
    assert_eq!(ix.mode(), ColorMode::Indexed);
    assert_eq!(ix.palette().len(), 8);
    assert_eq!(ix.palette()[4], [255, 255, 0, 128]);
    let (gr, _) = ase::ler(&entrada(&o, "cinza")).unwrap();
    assert_eq!(gr.mode(), ColorMode::Gray);
}

/// Grava os `.ase` NOSSOS numa pasta, para o `oraculo_export.py --verifica` os dar ao LibreSprite (M2).
#[test]
#[ignore = "ferramenta: PH2D_ASE_SAIDA=<pasta>"]
fn grava_os_nossos_para_o_oraculo() {
    let dir = std::env::var("PH2D_ASE_SAIDA").unwrap();
    let o = oraculo();
    for n in o["entradas_hex"].as_object().unwrap().keys() {
        let (a, _) = ase::ler(&entrada(&o, n)).unwrap();
        let (b, _) = ase::escrever(&a).unwrap();
        std::fs::write(format!("{dir}/{n}.ase"), b).unwrap();
    }
}

/// O LibreSprite leu ESTES bytes e deu os mesmos quadros (`oracle_ase_nosso.json`). Mudou o escritor,
/// fica vermelho: corra a verificação de novo, nunca regrave só o hash.
#[test]
fn o_libresprite_le_o_nosso_ase() {
    use sha2::{Digest, Sha256};
    let f: Value = serde_json::from_str(include_str!("fixtures/oracle_ase_nosso.json")).unwrap();
    let o = oraculo();
    for (n, sha) in f["sha256"].as_object().unwrap() {
        let (a, _) = ase::ler(&entrada(&o, n)).unwrap();
        let (b, _) = ase::escrever(&a).unwrap();
        let hex: String = Sha256::digest(&b).iter().map(|x| format!("{x:02x}")).collect();
        assert_eq!(&hex, sha.as_str().unwrap(), "{n}");
    }
}
