//! Os gates do [`crate::tela_na_malha`] — a tela do Painter pousada na peça.
//!
//! ⚠️ **A vista das fixturas é ORTOGRÁFICA de propósito** (a matriz identidade:
//! `x, y ∈ [-1, 1]` caem direitos no ecrã), porque assim a posição de cada
//! amostra na tela calcula-se à mão e o gate compara contra um número e não
//! contra outra implementação da projecção. A OCLUSÃO continua a ser medida
//! por raios do olho, que é o que o produto faz.

use ph2d_mesh::{Face, Mesh};
use ph2d_mesh_colors::Tinta;

use crate::SculptStroke;
use crate::tela_na_malha::{Tela, TelaNaMalha, Vista};
use crate::tinta_fina::TintaDoTraco;

const ANTES: [f32; 3] = [0.2, 0.4, 0.6];
const LADO: u32 = 100;
const OLHO: [f32; 3] = [0.0, 0.0, 10.0];

const IDENTIDADE: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, 0.0, //
    0.0, 0.0, 1.0, 0.0, //
    0.0, 0.0, 0.0, 1.0,
];

fn vista() -> Vista {
    Vista::nova(IDENTIDADE, (LADO, LADO), OLHO)
}

/// Uma grelha `n × n` de quads no plano `z`, de `-1` a `1`, virada para `+z`
/// (ou para `-z` com `costas`).
fn grelha(n: u32, z: f32, costas: bool) -> (Vec<[f32; 3]>, Vec<Face>) {
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let x = -1.0 + 2.0 * i as f32 / n as f32;
            let y = -1.0 + 2.0 * j as f32 / n as f32;
            pos.push([x, y, z]);
        }
    }
    let w = n + 1;
    let mut faces = Vec::new();
    for j in 0..n {
        for i in 0..n {
            let (a, b, c, d) = (
                j * w + i,
                j * w + i + 1,
                (j + 1) * w + i + 1,
                (j + 1) * w + i,
            );
            faces.push(if costas {
                Face::quad(a, d, c, b)
            } else {
                Face::quad(a, b, c, d)
            });
        }
    }
    (pos, faces)
}

fn malha(n: u32) -> Mesh {
    let (p, f) = grelha(n, 0.0, false);
    let mut m = Mesh::from_parts(p, f).expect("grelha válida");
    m.colors_mut().fill(ANTES);
    m
}

/// Uma tela `LADO²` cheia de uma cor com cobertura `a`, só onde `dentro(x,y)`.
fn tela(cor: [u8; 3], a: u8, dentro: impl Fn(u32, u32) -> bool) -> Vec<u8> {
    let mut rgba = vec![0u8; (LADO * LADO * 4) as usize];
    for y in 0..LADO {
        for x in 0..LADO {
            if dentro(x, y) {
                let o = ((y * LADO + x) * 4) as usize;
                rgba[o..o + 4].copy_from_slice(&[cor[0], cor[1], cor[2], a]);
            }
        }
    }
    rgba
}

fn tudo() -> [u32; 4] {
    [0, 0, LADO, LADO]
}

fn pousa(m: &mut Mesh, rgba: &[u8], r: [u32; 4]) -> SculptStroke {
    let mut s = SculptStroke::default();
    s.begin(m);
    let mut sessao = TelaNaMalha::nova(m, vista(), m.vert_count());
    let t = Tela {
        rgba,
        largura: LADO,
        altura: LADO,
    };
    s.pousa_a_tela(m, &mut sessao, &t, r);
    s
}

/// ⭐ **Tela opaca ⇒ a cor dela, EXACTA**, e o desfazer guarda a de antes.
#[test]
fn uma_tela_opaca_pinta_toda_a_peca_a_vista_com_a_cor_dela() {
    let mut m = malha(4);
    let rgba = tela([255, 0, 0], 255, |_, _| true);
    let s = pousa(&mut m, &rgba, tudo());
    let cores = m.colors().expect("pintada");
    let dentro = m.positions().iter().zip(cores).collect::<Vec<_>>();
    assert_eq!(
        dentro.len(),
        25,
        "a borda da vista também é superfície à vista"
    );
    for (_, c) in dentro {
        assert_eq!(*c, [1.0, 0.0, 0.0]);
    }
    assert!(!s.touched().is_empty(), "o traço capturou quem pintou");
    assert!(
        s.base_colors().iter().all(|c| *c == ANTES),
        "o desfazer tem a cor de antes"
    );
}

/// **A meia cobertura dá a meia mistura** — a lei `base·(1−a) + c·a`.
#[test]
fn meia_cobertura_da_a_mistura_da_camada_por_cima() {
    let mut m = malha(2);
    let rgba = tela([255, 0, 0], 128, |_, _| true);
    pousa(&mut m, &rgba, tudo());
    let a = 128.0 / 255.0;
    let esperado = [
        ANTES[0] * (1.0 - a) + a,
        ANTES[1] * (1.0 - a),
        ANTES[2] * (1.0 - a),
    ];
    let c = m.colors().expect("pintada")[4]; // o centro da grelha 3×3
    for k in 0..3 {
        assert!(
            (c[k] - esperado[k]).abs() < 1e-6,
            "{c:?} contra {esperado:?}"
        );
    }
}

/// **Pousar duas vezes a mesma tela dá o que dá pousar uma** — cada amostra
/// parte da cor de ANTES do traço. Sem isto o quadro seguinte engrossaria a
/// tinta a cada frame.
#[test]
fn pousar_a_mesma_tela_duas_vezes_nao_engrossa_a_tinta() {
    let mut m = malha(4);
    let rgba = tela([0, 255, 0], 100, |_, _| true);
    let mut s = SculptStroke::default();
    s.begin(&m);
    let mut sessao = TelaNaMalha::nova(&m, vista(), m.vert_count());
    let t = Tela {
        rgba: &rgba,
        largura: LADO,
        altura: LADO,
    };
    s.pousa_a_tela(&mut m, &mut sessao, &t, tudo());
    let uma = m.colors().expect("pintada").to_vec();
    let raios = sessao.raios();
    s.pousa_a_tela(&mut m, &mut sessao, &t, tudo());
    assert_eq!(m.colors().expect("pintada"), &uma[..]);
    assert_eq!(sessao.raios(), raios, "a visibilidade decide-se UMA vez");
}

/// **A máscara protege** — a mesma lei do pincel e do `Fill`.
#[test]
fn um_vertice_mascarado_fica_com_a_cor_de_antes() {
    let mut m = malha(2);
    m.masks_mut()[4] = 1.0;
    let rgba = tela([255, 0, 0], 255, |_, _| true);
    pousa(&mut m, &rgba, tudo());
    let c = m.colors().expect("pintada");
    assert_eq!(c[4], ANTES);
    assert_eq!(c[0], [1.0, 0.0, 0.0], "o CONTROLO: o vizinho livre pintou");
}

/// **Só pinta o que muda** — fora do rectângulo nada se toca, mesmo com a tela
/// cheia lá.
#[test]
fn fora_do_rectangulo_mudado_nada_e_tocado() {
    let mut m = malha(4);
    let rgba = tela([255, 0, 0], 255, |_, _| true);
    pousa(&mut m, &rgba, [0, 0, 40, LADO]);
    for (p, c) in m.positions().iter().zip(m.colors().expect("pintada")) {
        if p[0] > 0.0 {
            assert_eq!(*c, ANTES, "vértice {p:?} fora do rectângulo mudou");
        }
    }
    assert!(
        m.colors().expect("pintada").contains(&[1.0, 0.0, 0.0]),
        "o CONTROLO"
    );
}

/// ⭐⭐ **O que está ESCONDIDO não é pintado** — uma placa à frente da metade
/// esquerda. A placa pinta-se; o que está atrás dela não.
#[test]
fn o_que_esta_escondido_atras_de_outra_superficie_nao_e_pintado() {
    let (mut p, mut f) = grelha(4, 0.0, false);
    let base = p.len() as u32;
    // A placa: de x = -1 a x = -0.2, à altura z = 0.5.
    for (x, y) in [(-1.0, -1.0), (-0.2, -1.0), (-0.2, 1.0), (-1.0, 1.0)] {
        p.push([x, y, 0.5]);
    }
    f.push(Face::quad(base, base + 1, base + 2, base + 3));
    let mut m = Mesh::from_parts(p, f).expect("válida");
    m.colors_mut().fill(ANTES);
    let rgba = tela([255, 0, 0], 255, |_, _| true);
    pousa(&mut m, &rgba, tudo());
    let cores = m.colors().expect("pintada");
    let mut escondidos = 0;
    for (v, (q, c)) in m.positions().iter().zip(cores).enumerate() {
        if (v as u32) < base && q[0] < -0.3 && q[1] > -0.99 && q[1] < 0.99 {
            assert_eq!(*c, ANTES, "o vértice {q:?} está atrás da placa");
            escondidos += 1;
        }
    }
    assert!(escondidos > 0, "a fixtura tem vértices escondidos");
    assert_eq!(cores[base as usize + 1], [1.0, 0.0, 0.0], "a placa pintou");
    assert_eq!(
        cores[(2 * 5 + 4) as usize],
        [1.0, 0.0, 0.0],
        "o CONTROLO: à vista pintou"
    );
}

/// **As costas não se pintam** — uma face virada para longe do olho.
#[test]
fn uma_face_de_costas_para_o_olho_nao_e_pintada() {
    let (p, f) = grelha(2, 0.0, true);
    let mut m = Mesh::from_parts(p, f).expect("válida");
    m.colors_mut().fill(ANTES);
    let rgba = tela([255, 0, 0], 255, |_, _| true);
    pousa(&mut m, &rgba, tudo());
    assert!(m.colors().expect("pintada").iter().all(|c| *c == ANTES));
}

/// ⭐⭐⭐ **Com tinta fina a resolução é a da TELA, não a da malha** — uma borda
/// a meio de uma face deixa amostras DENTRO da mesma face de um lado e do
/// outro. É isto que o pincel da escultura não fazia e o Painter faz.
#[test]
fn com_tinta_fina_uma_borda_passa_a_meio_de_uma_face() {
    let m0 = malha(1); // UMA face, os quatro cantos nas bordas da tela
    let faces = || m0.faces().iter().map(Face::verts);
    let tinta = Tinta::semeada(m0.colors().expect("pintada"), faces(), 4);
    let mut m = m0.clone();
    let mut s = SculptStroke::default();
    s.begin(&m);
    s.tinta_fina = Some(TintaDoTraco::nova(tinta, 0));
    let n = s
        .tinta_fina
        .as_ref()
        .map_or(0, |t| t.tinta().amostras().len());
    let mut sessao = TelaNaMalha::nova(&m, vista(), n);
    // Tinta só na metade esquerda da tela.
    let rgba = tela([255, 0, 0], 255, |x, _| x < LADO / 2);
    let t = Tela {
        rgba: &rgba,
        largura: LADO,
        altura: LADO,
    };
    s.pousa_a_tela(&mut m, &mut sessao, &t, tudo());
    let fina = s.tinta_fina.as_ref().expect("emprestada");
    let amostras = fina.tinta().amostras();
    let vermelhas = amostras.iter().filter(|c| **c == [1.0, 0.0, 0.0]).count();
    let intactas = amostras.iter().filter(|c| **c == ANTES).count();
    assert!(
        vermelhas > 0 && intactas > 0,
        "{vermelhas} vermelhas, {intactas} intactas"
    );
    assert!(
        vermelhas > 2,
        "há amostras vermelhas ALÉM dos cantos: a borda passou dentro da face"
    );
    // ⚠️ A semente INTERPOLA, e interpolar cores iguais não devolve a cor nos
    // bits (§18 da tinta fina) — a base compara-se com folga.
    assert!(!fina.tocadas().is_empty(), "o desfazer tem janela");
    assert!(
        fina.base()
            .iter()
            .all(|c| (0..3).all(|k| (c[k] - ANTES[k]).abs() < 1e-6)),
        "a janela guarda a cor de ANTES do traço"
    );
}

/// Uma pousada sobre UMA face com tinta fina, e as amostras antes e depois.
fn pousa_fino(m: &mut Mesh, rgba: &[u8], vezes: u32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let faces = || m.faces().iter().map(Face::verts).collect::<Vec<_>>();
    let tinta = Tinta::semeada(m.colors().expect("pintada"), faces().into_iter(), 4);
    let antes = tinta.amostras().to_vec();
    let mut s = SculptStroke::default();
    s.begin(m);
    s.tinta_fina = Some(TintaDoTraco::nova(tinta, 0));
    let mut sessao = TelaNaMalha::nova(m, vista(), antes.len());
    let t = Tela {
        rgba,
        largura: LADO,
        altura: LADO,
    };
    let mut raios = 0;
    for k in 0..vezes {
        s.pousa_a_tela(m, &mut sessao, &t, tudo());
        if k == 0 {
            raios = sessao.raios();
        }
    }
    assert_eq!(sessao.raios(), raios, "a visibilidade decide-se UMA vez");
    let depois = s
        .tinta_fina
        .as_ref()
        .expect("emprestada")
        .tinta()
        .amostras()
        .to_vec();
    (antes, depois)
}

/// ⭐⭐ **Na tinta fina, pousar duas vezes dá o que dá pousar uma** — a irmã do
/// gate dos vértices, no OUTRO caminho da lei. ⚠️ Escrita por uma mutação
/// SOBREVIVENTE (`L7` do `muta_o_painter_na_peca.sh`): o gate dos vértices não
/// alcança a amostra, e com a mistura sobre a cor CORRENTE a re-projecção por
/// quadro escurecia a tinta fina sem um teste a ver.
#[test]
fn com_tinta_fina_pousar_duas_vezes_nao_engrossa_a_tinta() {
    let rgba = tela([0, 255, 0], 100, |_, _| true);
    let (_, uma) = pousa_fino(&mut malha(1), &rgba, 1);
    let (antes, duas) = pousa_fino(&mut malha(1), &rgba, 2);
    assert_ne!(uma, antes, "o CONTROLO: a primeira pousada pintou");
    assert_eq!(duas, uma, "a segunda pousada engrossou a tinta fina");
}

/// ⭐⭐ **Na tinta fina, a máscara protege a amostra** — a irmã do gate dos
/// vértices. ⚠️ Escrita por uma mutação SOBREVIVENTE (`L4`): com a máscara
/// ignorada no caminho das amostras, o gate dos vértices ficava verde. Com os
/// quatro cantos mascarados toda amostra fica AO BIT; o CONTROLO é a mesma
/// pousada sem máscara, que pinta.
#[test]
fn com_tinta_fina_a_mascara_protege_as_amostras() {
    let rgba = tela([255, 0, 0], 255, |_, _| true);
    let (antes, livre) = pousa_fino(&mut malha(1), &rgba, 1);
    assert_ne!(livre, antes, "o CONTROLO: sem máscara a tela pinta");
    let mut m = malha(1);
    m.masks_mut().fill(1.0);
    let (antes, mascarada) = pousa_fino(&mut m, &rgba, 1);
    assert_eq!(mascarada, antes, "a máscara não protegeu as amostras");
}
