//! Os gates do PREÇO da tela do Painter pousada na tinta fina — os dois atalhos
//! do [`crate::tela_na_malha`] (o bloco da retícula que a caixa não alcança e
//! a oclusão decidida por píxel) têm de pintar o MESMO que o caminho sem eles.
//!
//! ⚠️ Medido na peça da cena `=52` a `256x` (sonda
//! `diag_o_preco_de_pintar_em_cada_degrau`): sem os dois, o pen-down custava
//! `247 ms` e cada quadro `45,7`; com os dois, `15,0` e `9,1`.

use ph2d_mesh::{Face, Mesh};
use ph2d_mesh_colors::Tinta;

use crate::SculptStroke;
use crate::tela_na_malha::{Tela, TelaNaMalha};
use crate::tela_na_malha_tests::{LADO, grelha, malha, tudo, vista};
use crate::tinta_fina::{TintaDoTraco, bilinear};

const VERMELHO: [f32; 3] = [1.0, 0.0, 0.0];

fn opaca() -> Vec<u8> {
    [255u8, 0, 0, 255].repeat((LADO * LADO) as usize)
}

/// O que uma pousada deixou: as amostras antes e depois, a posição de cada
/// amostra e as duas sondas de custo da sessão.
struct Pousada {
    antes: Vec<[f32; 3]>,
    depois: Vec<[f32; 3]>,
    onde: Vec<[f32; 3]>,
    raios: usize,
    projetadas: usize,
}

/// Pousa a tela opaca em `r` sobre `m` com um plano ao `nivel`.
fn pousa_no_plano(m: &mut Mesh, nivel: u8, r: [u32; 4]) -> Pousada {
    let faces: Vec<Vec<u32>> = m.faces().iter().map(|f| f.verts().to_vec()).collect();
    let tinta = Tinta::semeada(
        m.colors().expect("pintada"),
        faces.iter().map(Vec::as_slice),
        nivel,
    );
    let antes = tinta.amostras().to_vec();
    let mut onde = vec![[f32::NAN; 3]; antes.len()];
    for (fi, cantos) in faces.iter().enumerate() {
        let lado = tinta.lado_da_face(fi) as f32;
        tinta.para_cada_amostra_quad(fi, cantos, |idx, (i, j)| {
            let w = bilinear(i as f32 / lado, j as f32 / lado);
            let mut p = [0.0f32; 3];
            for (&v, &wk) in cantos.iter().zip(&w) {
                for (pe, q) in p.iter_mut().zip(m.positions()[v as usize]) {
                    *pe += q * wk;
                }
            }
            onde[idx as usize] = p;
        });
    }
    let mut s = SculptStroke::default();
    s.begin(m);
    s.tinta_fina = Some(TintaDoTraco::nova(tinta, 0));
    let mut sessao = TelaNaMalha::nova(m, vista(), antes.len());
    let rgba = opaca();
    let t = Tela {
        rgba: &rgba,
        largura: LADO,
        altura: LADO,
    };
    s.pousa_a_tela(m, &mut sessao, &t, r);
    let depois = s
        .tinta_fina
        .as_ref()
        .expect("emprestada")
        .tinta()
        .amostras()
        .to_vec();
    Pousada {
        antes,
        depois,
        onde,
        raios: sessao.raios(),
        projetadas: sessao.projetadas(),
    }
}

/// ⭐⭐ **O bloco que a caixa não alcança não muda NADA do que se pinta.** Um
/// rectângulo que corta faces E blocos a meio (a `64x` cada face tem quatro
/// blocos por lado): o conjunto de amostras pintadas é EXACTAMENTE o das que
/// se projectam dentro da caixa (o rectângulo com um píxel de folga), contado
/// aqui sem atalho nenhum. Uma caixa de bloco errada deixa amostras de fora.
#[test]
fn os_blocos_que_a_caixa_nao_alcanca_nao_mudam_o_que_se_pinta() {
    let r = [37u32, 22, 11, 29];
    let mut m = malha(2);
    let Pousada {
        antes,
        depois,
        onde,
        ..
    } = pousa_no_plano(&mut m, 6, r);
    let caixa = [
        r[0] as f32 - 1.0,
        r[1] as f32 - 1.0,
        (r[0] + r[2]) as f32 + 1.0,
        (r[1] + r[3]) as f32 + 1.0,
    ];
    let v = vista();
    let mut esperadas = 0;
    for (idx, p) in onde.iter().enumerate() {
        let s = v.ecra(*p).expect("à frente da câmera");
        let dentro = s[0] >= caixa[0] && s[0] <= caixa[2] && s[1] >= caixa[1] && s[1] <= caixa[3];
        if dentro {
            esperadas += 1;
            assert_eq!(
                depois[idx], VERMELHO,
                "a amostra {idx} em {s:?} ficou por pintar"
            );
        } else {
            assert_eq!(
                depois[idx], antes[idx],
                "a amostra {idx} em {s:?} fora da caixa mudou"
            );
        }
    }
    assert!(esperadas > 0, "o CONTROLO: a caixa apanha amostras");
    assert!(
        esperadas < onde.len() / 4,
        "o CONTROLO: a caixa corta a peça"
    );
}

/// ⭐⭐ **A oclusão decide-se UMA vez por (face, píxel)**, e não por amostra.
/// A `256x` uma face que cobre a tela põe `~6,5` amostras por píxel e por eixo;
/// o raio que decide uma serve todas as do mesmo píxel. O CONTROLO é o número
/// de amostras pintadas, que continua a ser o de todas.
#[test]
fn a_oclusao_decide_se_uma_vez_por_pixel_e_nao_por_amostra() {
    let mut m = malha(1);
    let Pousada { depois, raios, .. } = pousa_no_plano(&mut m, 8, tudo());
    let pintadas = depois.iter().filter(|c| **c == VERMELHO).count();
    assert_eq!(
        pintadas,
        depois.len(),
        "o CONTROLO: a tela opaca pintou tudo"
    );
    assert!(
        raios * 4 < pintadas,
        "{raios} raios para {pintadas} amostras: a oclusão não é partilhada no píxel"
    );
}

/// ⭐⭐ **Com tinta fina o que está ESCONDIDO não é pintado** — a irmã do gate
/// dos vértices, no caminho das amostras e com a oclusão por píxel. Uma placa
/// à frente da metade esquerda: atrás dela nada muda, ao lado pinta.
#[test]
fn com_tinta_fina_o_que_esta_escondido_nao_e_pintado() {
    let (mut p, mut f) = grelha(2, 0.0, false);
    let base = p.len() as u32;
    for (x, y) in [(-1.0, -1.0), (-0.2, -1.0), (-0.2, 1.0), (-1.0, 1.0)] {
        p.push([x, y, 0.5]);
    }
    f.push(Face::quad(base, base + 1, base + 2, base + 3));
    let mut m = Mesh::from_parts(p, f).expect("válida");
    m.colors_mut().fill(crate::tela_na_malha_tests::ANTES);
    let Pousada {
        antes,
        depois,
        onde,
        ..
    } = pousa_no_plano(&mut m, 6, tudo());
    let (mut escondidas, mut a_vista) = (0, 0);
    for (idx, q) in onde.iter().enumerate() {
        let interior = q[1] > -0.99 && q[1] < 0.99;
        if q[2] == 0.0 && q[0] < -0.3 && interior {
            assert_eq!(
                depois[idx], antes[idx],
                "a amostra {q:?} está atrás da placa"
            );
            escondidas += 1;
        } else if q[2] == 0.0 && q[0] > -0.1 && interior {
            assert_eq!(depois[idx], VERMELHO, "a amostra {q:?} está à vista");
            a_vista += 1;
        }
    }
    assert!(
        escondidas > 0 && a_vista > 0,
        "{escondidas} escondidas, {a_vista} à vista"
    );
}

/// ⭐⭐ **Um traço pequeno numa face fina projecta a PEGADA, não a face.** A
/// `256x` a face que cobre a tela tem `66 049` amostras e uma caixa de seis
/// píxeis apanha poucas centenas: os blocos que ela não alcança nem se
/// projectam. ⚠️ Sem este gate os blocos seriam INOBSERVÁVEIS (pintam o mesmo
/// com e sem eles — é o gate irmão) e a mutação que os desliga sobrevivia.
#[test]
fn um_traco_pequeno_projecta_a_pegada_e_nao_a_face() {
    let mut m = malha(1);
    let Pousada {
        antes,
        depois,
        projetadas,
        ..
    } = pousa_no_plano(&mut m, 8, [45, 45, 4, 4]);
    let pintadas = depois.iter().zip(&antes).filter(|(d, a)| d != a).count();
    assert!(pintadas > 0, "o CONTROLO: a caixa pintou");
    assert!(
        projetadas >= pintadas,
        "o CONTROLO do instrumento: toda amostra pintada foi projectada ({projetadas} < {pintadas})"
    );
    assert!(
        projetadas * 10 < depois.len(),
        "{projetadas} de {} amostras projectadas para pintar {pintadas}",
        depois.len()
    );
}
