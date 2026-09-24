//! Os gates do [`crate::tela_semente`] — o retrato da peça — e da lei da
//! DIFERENÇA que ele arma no [`crate::tela_na_malha`].
//!
//! ⚠️ As fixturas são as do `tela_na_malha_tests`: vista ORTOGRÁFICA (a posição
//! de cada píxel calcula-se à mão) e uma grelha de quads virada ao olho.

use ph2d_mesh::{Face, Mesh};
use ph2d_mesh_colors::Tinta;

use crate::SculptStroke;
use crate::tela_na_malha::{Tela, TelaNaMalha};
use crate::tela_na_malha_tests::{ANTES, LADO, grelha, malha, tudo, vista};
use crate::tela_semente::semente;
use crate::tinta_fina::TintaDoTraco;

fn byte(c: f32) -> u8 {
    (c * 255.0 + 0.5) as u8
}

fn pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
    let o = ((y * LADO + x) * 4) as usize;
    [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]]
}

/// ⭐ **Uma peça que enche a vista enche o retrato com a cor dela** — todo píxel
/// opaco, na cor da peça em bytes sRGB (o formato da tela do Painter).
#[test]
fn uma_peca_que_enche_a_vista_enche_o_retrato() {
    let r = semente(&malha(4), None, &vista());
    let esperado = [byte(ANTES[0]), byte(ANTES[1]), byte(ANTES[2]), 255];
    for (x, y) in [(0, 0), (50, 50), (99, 99), (10, 80)] {
        assert_eq!(pixel(&r, x, y), esperado, "píxel ({x}, {y})");
    }
}

/// **Fora da silhueta o retrato é TRANSPARENTE** — o pincel não arrasta cor
/// nenhuma do vazio.
#[test]
fn fora_da_silhueta_o_retrato_e_transparente() {
    // Meia peça: de x = −1 a x = 0.
    let pos: Vec<[f32; 3]> = [[-1.0, -1.0], [0.0, -1.0], [0.0, 1.0], [-1.0, 1.0]]
        .iter()
        .map(|p| [p[0], p[1], 0.0])
        .collect();
    let mut m = Mesh::from_parts(pos, vec![Face::quad(0, 1, 2, 3)]).expect("válida");
    m.colors_mut().fill(ANTES);
    let r = semente(&m, None, &vista());
    assert_eq!(
        pixel(&r, 25, 50)[3],
        255,
        "o CONTROLO: a peça está no retrato"
    );
    assert_eq!(pixel(&r, 75, 50)[3], 0, "o vazio ficou pintado");
}

/// ⭐⭐ **O que está ESCONDIDO não entra no retrato** — o píxel é da superfície
/// mais PERTO do olho, e a ordem das faces na malha não decide nada.
#[test]
fn o_retrato_mostra_a_superficie_mais_perto_e_nao_a_ultima_desenhada() {
    // A placa (à frente) PRIMEIRO na lista, o fundo depois: sem profundidade
    // o fundo, desenhado por último, taparia a placa.
    let placa: Vec<[f32; 3]> = [[-1.0, -1.0], [-0.2, -1.0], [-0.2, 1.0], [-1.0, 1.0]]
        .iter()
        .map(|p| [p[0], p[1], 0.5])
        .collect();
    let (fundo, faces_fundo) = grelha(2, 0.0, false);
    let mut pos = placa;
    let base = pos.len() as u32;
    pos.extend(fundo);
    let mut faces = vec![Face::quad(0, 1, 2, 3)];
    faces.extend(faces_fundo.iter().map(|f| {
        let v = f.verts();
        Face::quad(v[0] + base, v[1] + base, v[2] + base, v[3] + base)
    }));
    let mut m = Mesh::from_parts(pos, faces).expect("válida");
    let cores = m.colors_mut();
    cores[..4].fill([1.0, 0.0, 0.0]);
    cores[4..].fill(ANTES);
    let r = semente(&m, None, &vista());
    assert_eq!(pixel(&r, 20, 50), [255, 0, 0, 255], "a placa à frente");
    assert_eq!(
        pixel(&r, 80, 50),
        [byte(ANTES[0]), byte(ANTES[1]), byte(ANTES[2]), 255],
        "o CONTROLO: o fundo à vista"
    );
}

/// **As costas não entram** — uma face virada para longe do olho.
#[test]
fn uma_face_de_costas_nao_entra_no_retrato() {
    let (p, f) = grelha(2, 0.0, true);
    let m = Mesh::from_parts(p, f).expect("válida");
    let r = semente(&m, None, &vista());
    assert!(r.chunks(4).all(|px| px[3] == 0));
}

/// ⭐⭐⭐ **Com tinta fina o retrato tem a resolução da TINTA, não da malha** —
/// uma face só, com a metade esquerda do plano pintada, dá um retrato com a
/// borda A MEIO da face. É isto que deixa o borrão arrastar o detalhe fino.
#[test]
fn com_tinta_fina_o_retrato_tem_a_resolucao_da_tinta() {
    let m = malha(1);
    let faces = || m.faces().iter().map(Face::verts);
    let mut tinta = Tinta::semeada(m.colors().expect("pintada"), faces(), 4);
    let cantos = m.faces()[0].verts();
    let mut pintar = Vec::new();
    tinta.para_cada_amostra_quad(0, cantos, |idx, (i, _)| {
        if i <= 1 {
            pintar.push(idx);
        }
    });
    for idx in pintar {
        tinta.amostras_mut()[idx as usize] = [1.0, 0.0, 0.0];
    }
    let r = semente(&m, Some(&tinta), &vista());
    assert_eq!(
        pixel(&r, 2, 50),
        [255, 0, 0, 255],
        "a margem esquerda é tinta"
    );
    assert_eq!(
        pixel(&r, 97, 50),
        [byte(ANTES[0]), byte(ANTES[1]), byte(ANTES[2]), 255],
        "o CONTROLO: a direita não"
    );
}

// ── A LEI DA DIFERENÇA ─────────────────────────────────────────────────────

fn pousa_semeado(m: &mut Mesh, canvas: &[u8]) -> (SculptStroke, usize) {
    let retrato = semente(m, None, &vista());
    let mut s = SculptStroke::default();
    s.begin(m);
    let mut sessao = TelaNaMalha::nova(m, vista(), m.vert_count());
    sessao.com_semente(retrato);
    let t = Tela {
        rgba: canvas,
        largura: LADO,
        altura: LADO,
    };
    s.pousa_a_tela(m, &mut sessao, &t, tudo());
    (s, sessao.raios())
}

/// ⭐⭐⭐ **Uma tela semeada que o pincel NÃO tocou não muda nada, AO BIT, e não
/// lança um raio** — a diferença anula-se por construção (os mesmos bytes dos
/// dois lados), e é isso que deixa o retrato ter erros sem eles aparecerem.
#[test]
fn uma_tela_semeada_intacta_nao_muda_nada_e_nao_custa_raios() {
    let mut m = malha(4);
    let antes = m.colors().expect("pintada").to_vec();
    let retrato = semente(&m, None, &vista());
    let (s, raios) = pousa_semeado(&mut m, &retrato);
    assert_eq!(m.colors().expect("pintada"), &antes[..]);
    assert!(s.touched().is_empty(), "o traço não capturou ninguém");
    assert_eq!(raios, 0, "o intacto sai antes do raio de oclusão");
}

/// ⭐⭐ **O que o pincel mudou chega à peça como DIFERENÇA** — a metade
/// esquerda passa a vermelho e a direita fica, ao bit.
#[test]
fn o_que_a_tela_semeada_mudou_chega_a_peca() {
    let mut m = malha(4);
    let mut canvas = semente(&m, None, &vista());
    for y in 0..LADO {
        for x in 0..LADO / 2 - 5 {
            let o = ((y * LADO + x) * 4) as usize;
            canvas[o..o + 4].copy_from_slice(&[255, 0, 0, 255]);
        }
    }
    pousa_semeado(&mut m, &canvas);
    for (p, c) in m.positions().iter().zip(m.colors().expect("pintada")) {
        if p[0] < -0.2 {
            assert!(
                (c[0] - 1.0).abs() < 1e-5 && c[1].abs() < 1e-5 && c[2].abs() < 1e-5,
                "{p:?} → {c:?}"
            );
        } else if p[0] > 0.2 {
            assert_eq!(*c, ANTES, "{p:?} fora do traço mudou");
        }
    }
}

/// **Um modo que ESCURECE a tela escurece a peça** — a diferença leva a
/// relação, não uma cor absoluta (é o caminho do Multiply e da aquarela).
#[test]
fn escurecer_a_tela_escurece_a_peca() {
    let mut m = malha(2);
    let mut canvas = semente(&m, None, &vista());
    for px in canvas.chunks_mut(4) {
        for e in px.iter_mut().take(3) {
            *e /= 2;
        }
    }
    pousa_semeado(&mut m, &canvas);
    let c = m.colors().expect("pintada")[4];
    for e in 0..3 {
        let esperado =
            f32::from(byte(ANTES[e]) / 2) / 255.0 - f32::from(byte(ANTES[e])) / 255.0 + ANTES[e];
        assert!((c[e] - esperado).abs() < 1e-5, "{c:?} canal {e}");
    }
}

/// **A borracha não arranca a cor da peça** — um píxel apagado não tem cor
/// que se compare, e a peça fica. ⚠️ Declarado: apagar tinta de uma peça é
/// outra pergunta (voltar ao barro?), e é do dono.
#[test]
fn um_pixel_apagado_nao_muda_a_peca() {
    let mut m = malha(2);
    let antes = m.colors().expect("pintada").to_vec();
    let mut canvas = semente(&m, None, &vista());
    canvas.chunks_mut(4).for_each(|px| px[3] = 0);
    pousa_semeado(&mut m, &canvas);
    assert_eq!(m.colors().expect("pintada"), &antes[..]);
}

/// **Na tela semeada, pousar duas vezes = pousar uma** — a partida continua a
/// ser a base do traço.
#[test]
fn na_tela_semeada_pousar_duas_vezes_nao_acumula() {
    let mut m = malha(2);
    let mut canvas = semente(&m, None, &vista());
    canvas.chunks_mut(4).for_each(|px| px[0] = 255);
    let retrato = semente(&m, None, &vista());
    let mut s = SculptStroke::default();
    s.begin(&m);
    let mut sessao = TelaNaMalha::nova(&m, vista(), m.vert_count());
    sessao.com_semente(retrato);
    let t = Tela {
        rgba: &canvas,
        largura: LADO,
        altura: LADO,
    };
    s.pousa_a_tela(&mut m, &mut sessao, &t, tudo());
    let uma = m.colors().expect("pintada").to_vec();
    assert_ne!(uma[4], ANTES, "o CONTROLO: a primeira pousada mudou a peça");
    s.pousa_a_tela(&mut m, &mut sessao, &t, tudo());
    assert_eq!(m.colors().expect("pintada"), &uma[..]);
}

/// **A máscara protege também na diferença.**
#[test]
fn na_tela_semeada_a_mascara_protege() {
    let mut m = malha(2);
    m.masks_mut()[4] = 1.0;
    let mut canvas = semente(&m, None, &vista());
    canvas.chunks_mut(4).for_each(|px| px[0] = 255);
    pousa_semeado(&mut m, &canvas);
    let c = m.colors().expect("pintada");
    assert_eq!(c[4], ANTES);
    assert_ne!(c[0], ANTES, "o CONTROLO: o vizinho livre mudou");
}

/// ⭐⭐ **Na tinta fina a diferença chega às AMOSTRAS** — o caminho do plano,
/// com o retrato feito do próprio plano.
#[test]
fn na_tela_semeada_a_diferenca_chega_a_tinta_fina() {
    let m0 = malha(1);
    let faces = || m0.faces().iter().map(Face::verts);
    let tinta = Tinta::semeada(m0.colors().expect("pintada"), faces(), 4);
    let retrato = semente(&m0, Some(&tinta), &vista());
    let mut canvas = retrato.clone();
    for y in 0..LADO {
        for x in 0..LADO / 2 {
            let o = ((y * LADO + x) * 4) as usize;
            canvas[o..o + 3].copy_from_slice(&[255, 0, 0]);
        }
    }
    let antes = tinta.amostras().to_vec();
    let mut m = m0.clone();
    let mut s = SculptStroke::default();
    s.begin(&m);
    s.tinta_fina = Some(TintaDoTraco::nova(tinta, 0));
    let mut sessao = TelaNaMalha::nova(&m, vista(), antes.len());
    sessao.com_semente(retrato);
    let t = Tela {
        rgba: &canvas,
        largura: LADO,
        altura: LADO,
    };
    s.pousa_a_tela(&mut m, &mut sessao, &t, tudo());
    let depois = s
        .tinta_fina
        .as_ref()
        .expect("emprestada")
        .tinta()
        .amostras();
    let mudaram = depois.iter().zip(&antes).filter(|(a, b)| a != b).count();
    let iguais = depois.iter().zip(&antes).filter(|(a, b)| a == b).count();
    assert!(
        mudaram > 2 && iguais > 2,
        "{mudaram} mudaram, {iguais} iguais"
    );
}

/// 🔎 **SONDA (não é gate)** — o preço do retrato no pen-down, sobre uma esfera
/// do tamanho da peça de fábrica e uma vista do tamanho da doca, com e sem o
/// plano de tinta fina. Corre-se em `--release`:
/// `cargo test -p ph2d-sculpt3d --release --lib -- --ignored diag_o_preco_do_retrato --nocapture`
#[test]
#[ignore = "sonda de relógio"]
fn diag_o_preco_do_retrato() {
    for (aneis, segs) in [(128usize, 256usize), (256, 512)] {
        let mut m = ph2d_mesh::shapes::uv_sphere(aneis, segs, 1.0);
        m.colors_mut().fill(ANTES);
        let (w, h) = (1400u32, 900u32);
        let v = perspectiva(w, h);
        let faces = || m.faces().iter().map(Face::verts);
        for nivel in [None, Some(3u8)] {
            let tinta = nivel.map(|l| Tinta::semeada(m.colors().expect("pintada"), faces(), l));
            let t0 = std::time::Instant::now();
            let reps = 5;
            let mut opacos = 0;
            for _ in 0..reps {
                let r = semente(&m, tinta.as_ref(), &v);
                opacos = r.chunks(4).filter(|p| p[3] == 255).count();
            }
            let ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(reps);
            println!(
                "  {} vértices · plano {nivel:?} · {w}×{h} · {opacos} píxeis na peça · {ms:.2} ms",
                m.vert_count()
            );
        }
    }
}

/// Perspectiva de 45°, olho em `z = 3` a olhar para `−z`.
fn perspectiva(w: u32, h: u32) -> crate::tela_na_malha::Vista {
    let f = 1.0 / (22.5f32.to_radians()).tan();
    let asp = w as f32 / h as f32;
    let (n, fa) = (0.1f32, 100.0f32);
    let proj = [
        f / asp,
        0.0,
        0.0,
        0.0, //
        0.0,
        f,
        0.0,
        0.0, //
        0.0,
        0.0,
        fa / (n - fa),
        -1.0, //
        0.0,
        0.0,
        -3.0 * fa / (n - fa) + fa * n / (n - fa),
        3.0,
    ];
    crate::tela_na_malha::Vista::nova(proj, (w, h), [0.0, 0.0, 3.0])
}

/// ⭐⭐ **O retrato interpola com PERSPECTIVA** — um plano muito inclinado, com
/// a cor a variar ao longo dele: o píxel onde cai o MEIO do plano (em espaço
/// do objecto) tem a cor do meio. ⚠️ Nas fixturas ortográficas a correcção é
/// invisível (`w` constante), e é por isso que este gate existe: uma
/// interpolação no ecrã poria a cor do meio no meio do ECRÃ, que num plano
/// inclinado é outro sítio da superfície.
#[test]
fn o_retrato_interpola_com_perspectiva() {
    let (w, h) = (400u32, 400u32);
    let v = perspectiva(w, h);
    // Um plano de x = −1 (perto, z = 1,2) a x = +1 (longe, z = −1,2).
    let pos = vec![
        [-1.0, -0.5, 1.2],
        [1.0, -0.5, -1.2],
        [1.0, 0.5, -1.2],
        [-1.0, 0.5, 1.2],
    ];
    let mut m = Mesh::from_parts(pos, vec![Face::quad(0, 1, 2, 3)]).expect("válida");
    let cores = m.colors_mut();
    cores[0] = [1.0, 0.0, 0.0];
    cores[3] = [1.0, 0.0, 0.0];
    cores[1] = [0.0, 0.0, 1.0];
    cores[2] = [0.0, 0.0, 1.0];
    let r = semente(&m, None, &v);
    let meio = v.ecra([0.0, 0.0, 0.0]).expect("à frente do olho");
    let (x, y) = (meio[0] as u32, meio[1] as u32);
    let o = ((y * w + x) * 4) as usize;
    let px = &r[o..o + 4];
    assert_eq!(px[3], 255, "o meio do plano está no retrato");
    // A cor do meio é meio vermelho, meio azul — a menos do meio píxel.
    assert!(
        (i32::from(px[0]) - 128).abs() <= 6 && (i32::from(px[2]) - 128).abs() <= 6,
        "o meio do plano lê {px:?}, e a cor do meio é ~[128, 0, 128]"
    );
}
