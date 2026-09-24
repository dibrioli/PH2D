//! Os gates do [`crate::preenche`] — o `Fill`.
//!
//! ⚠️ **A fixtura é uma `uv_sphere` de propósito:** os pólos são leques de
//! TRIÂNGULOS e o resto são QUADS, e as duas retículas não partilham a conta
//! dos pesos. Uma fixtura só de quads deixaria metade da lei sem régua.

use ph2d_mesh::{Mesh, shapes};
use ph2d_mesh_colors::Tinta;

use crate::preenche::{Recusa, keep_da_amostra, mistura, preenche_plano, preenche_vertices};

const COR: [f32; 3] = [0.9, 0.1, 0.05];
/// Uma cor ANTES que não é a de fábrica, para o «não mudou» ser uma afirmação.
const ANTES: [f32; 3] = [0.2, 0.4, 0.6];

fn malha() -> Mesh {
    let mut m = shapes::uv_sphere(8, 12, 1.0);
    m.colors_mut().fill(ANTES);
    m
}

fn plano(m: &Mesh, nivel: u8) -> Tinta {
    let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
    Tinta::semeada(m.colors().expect("pintada acima"), faces(), nivel)
}

/// Uma máscara que VARIA pela peça — sem ela os cantos de uma face têm a mesma
/// máscara e a interpolação da retícula é inobservável.
fn mascara_variavel(m: &mut Mesh) {
    let n = m.vert_count();
    for (v, k) in m.masks_mut().iter_mut().enumerate() {
        *k = (v as f32 / n as f32).fract();
    }
}

/// **Livre ⇒ a cor do pincel, EXACTA** — nos vértices e em toda amostra.
#[test]
fn sem_mascara_toda_amostra_sai_exactamente_a_cor_do_pincel() {
    let mut m = malha();
    let mut t = plano(&m, 2);
    assert!(preenche_vertices(&mut m, COR));
    assert!(preenche_plano(&mut t, &m, COR).expect("o plano descreve a malha"));
    assert!(m.colors().expect("pintada").iter().all(|c| *c == COR));
    assert!(t.amostras().iter().all(|c| *c == COR));
}

/// **Toda mascarada ⇒ NADA muda, e as duas portas DIZEM que nada mudou** — é
/// esse `false` que impede o chamador de gravar um passo de desfazer vazio.
#[test]
fn toda_mascarada_nada_muda_e_as_portas_o_dizem() {
    let mut m = malha();
    m.masks_mut().fill(1.0);
    let mut t = plano(&m, 2);
    let antes = t.amostras().to_vec();
    assert!(!preenche_vertices(&mut m, COR));
    assert!(!preenche_plano(&mut t, &m, COR).expect("descreve"));
    assert!(m.colors().expect("pintada").iter().all(|c| *c == ANTES));
    assert_eq!(t.amostras(), &antes[..]);
}

/// **A meio da máscara ⇒ a meio da cor**, pela mistura exacta nas pontas.
#[test]
fn meia_mascara_da_meia_cor() {
    let mut m = malha();
    m.masks_mut().fill(0.5);
    preenche_vertices(&mut m, COR);
    let esperado = mistura(ANTES, COR, 0.5);
    assert!(m.colors().expect("pintada").iter().all(|c| *c == esperado));
    assert!(
        esperado != ANTES && esperado != COR,
        "o controlo: é um ponto do MEIO"
    );
}

/// ⭐⭐ **O prefixo por-vértice do plano é, AO BIT, o que o preenchimento por
/// vértice escreve** — com uma máscara que varia, que é onde as duas contas
/// podiam divergir. Sem isto a vista grossa e a fina mostrariam cores
/// diferentes na mesma peça.
#[test]
fn os_vertices_do_plano_sao_ao_bit_o_preenchimento_por_vertice() {
    let mut m = malha();
    mascara_variavel(&mut m);
    let mut t = plano(&m, 3);
    let mut v = m.clone();
    preenche_vertices(&mut v, COR);
    preenche_plano(&mut t, &m, COR).expect("descreve");
    assert_eq!(t.plano_por_vertice(), v.colors().expect("pintada"));
}

/// ⭐⭐ **O INTERIOR de uma face lê a máscara INTERPOLADA** — a mesma conta que
/// o carimbo do pincel faz. Um quad com máscara `1` num canto só: a amostra do
/// centro (a `lado = 2`) tem os quatro pesos a `¼`, logo `keep = ¾`.
#[test]
fn o_centro_de_um_quad_le_a_mascara_interpolada() {
    let mut m = malha();
    let (fq, quad) = m
        .faces()
        .iter()
        .enumerate()
        .find(|(_, f)| f.verts().len() == 4)
        .map(|(i, f)| (i, f.verts().to_vec()))
        .expect("a uv_sphere tem quads");
    m.masks_mut().fill(0.0);
    m.masks_mut()[quad[0] as usize] = 1.0;
    let mut t = plano(&m, 1);
    assert_eq!(t.lado_da_face(fq), 2, "o controlo: nível 1 é lado 2");
    let mut centro = None;
    t.para_cada_amostra_quad(fq, &quad, |idx, (i, j)| {
        if (i, j) == (1, 1) {
            centro = Some(idx);
        }
    });
    let centro = centro.expect("a retícula de lado 2 tem centro");
    preenche_plano(&mut t, &m, COR).expect("descreve");
    assert_eq!(keep_da_amostra(&[0.25; 4], &[1.0, 0.0, 0.0, 0.0]), 0.75);
    assert_eq!(t.amostras()[centro as usize], mistura(ANTES, COR, 0.75));
    // O canto mascarado ficou; os outros três pintaram-se.
    assert_eq!(t.amostras()[quad[0] as usize], ANTES);
    assert_eq!(t.amostras()[quad[1] as usize], COR);
}

/// ⛔⛔ **Um plano que não descreve a malha é RECUSADO antes de uma escrita** —
/// a lei do §14 da tinta fina (faces a mais estouram; faces a menos escrevem
/// tinta válida no sítio errado, em silêncio).
#[test]
fn um_plano_de_outra_malha_e_recusado_sem_escrita() {
    let m = malha();
    let outra = shapes::uv_sphere(6, 12, 1.0);
    let mut t = {
        let faces = || outra.faces().iter().map(ph2d_mesh::Face::verts);
        Tinta::nova(outra.vert_count(), faces(), 2)
    };
    let antes = t.amostras().to_vec();
    assert_eq!(preenche_plano(&mut t, &m, COR), Err(Recusa::NaoDescreve));
    assert_eq!(t.amostras(), &antes[..]);
}

/// ⭐ **As DUAS pontas da mistura são exactas** — a forma `a + (c − a)·k` não
/// é, e esta é a razão de a lei ser escrita como é.
///
/// ⚠️ **Nem todo par de cores separa as duas formas, e isso foi MEDIDO:** com
/// `c = 0,9` a forma ingénua devolve `c` exacto para TODO `a` em `[0, 1]` (a
/// subtracção é exacta pelo lema de Sterbenz na maior parte do intervalo, e o
/// resto arredonda de volta). Com `c = 0,1` ela erra em `783` de `999` valores,
/// entre eles o `0,4` do `ANTES`. ⇒ é o canal VERDE (`0,1`) que carrega a
/// régua; uma mutação no canal vermelho seria inobservável por construção.
#[test]
fn a_mistura_e_exacta_nas_duas_pontas() {
    for a in [ANTES, [0.1, 0.3, 0.7], [1.0, 0.0, 0.5]] {
        assert_eq!(mistura(a, COR, 1.0), COR);
        assert_eq!(mistura(a, COR, 0.0), a);
    }
}

/// ⭐⭐ **O carimbo do pincel e o `Fill` perguntam à MESMA porta** — o elo que
/// nenhum gate de valor vê, porque as duas contas escritas em separado dariam
/// o mesmo número até ao dia em que alguém mexesse numa.
///
/// ⚠️ A prosa sai antes de se medir, senão o doc-comment que CITA a porta
/// satisfazia a agulha.
#[test]
fn o_carimbo_do_pincel_pergunta_a_mesma_porta() {
    let fonte = include_str!("tinta_fina.rs");
    let codigo: String = fonte
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        codigo.contains("keep: crate::preenche::keep_da_amostra(w, m)"),
        "o carimbo deixou de pedir o `keep` à porta partilhada"
    );
    assert!(
        !codigo.contains("free_weight("),
        "o carimbo voltou a fazer a conta da máscara por conta própria"
    );
}

/// ⭐ **Um vértice que nenhuma face alcança também é pintado** — é a razão de
/// a passagem dos vértices existir antes da das faces. Sem ela, o prefixo
/// por-vértice do plano e a cor por vértice discordariam exactamente ali.
///
/// ⚠️ A fixtura TEM o vértice solto de propósito: numa malha sem ele a
/// passagem é indistinguível das faces (o peso de um canto é `(1, 0, 0)`
/// exacto), e a mutação que a apaga sobreviveria.
#[test]
fn um_vertice_que_nenhuma_face_alcanca_tambem_e_pintado() {
    let base = shapes::uv_sphere(8, 12, 1.0);
    let mut pos = base.positions().to_vec();
    pos.push([5.0, 5.0, 5.0]);
    let solto = pos.len() - 1;
    let mut m = Mesh::from_parts(pos, base.faces().to_vec()).expect("a malha monta");
    m.colors_mut().fill(ANTES);
    let mut t = plano(&m, 2);
    let mut v = m.clone();
    preenche_vertices(&mut v, COR);
    preenche_plano(&mut t, &m, COR).expect("descreve");
    assert_eq!(
        t.amostras()[solto],
        COR,
        "o vertice solto ficou por pintar no plano"
    );
    assert_eq!(t.plano_por_vertice(), v.colors().expect("pintada"));
}

/// ⛔⛔ **E a metade dos CANTOS: as mesmas contagens com uma face de OUTRA
/// forma também é recusada.** Um quad lido como triângulo escreve amostras do
/// quad vizinho — a forma MUDA do defeito, que nenhuma contagem vê.
#[test]
fn um_plano_com_as_mesmas_contagens_e_outra_forma_e_recusado() {
    let m = malha();
    let mut faces = m.faces().to_vec();
    let q = faces
        .iter()
        .position(|f| f.verts().len() == 4)
        .expect("a uv_sphere tem quads");
    let v = faces[q].verts().to_vec();
    faces[q] = ph2d_mesh::Face::tri(v[0], v[1], v[2]);
    let outra = Mesh::from_parts(m.positions().to_vec(), faces).expect("monta");
    assert_eq!(
        (outra.vert_count(), outra.faces().len()),
        (m.vert_count(), m.faces().len()),
        "o controlo: as contagens batem, so' a FORMA de uma face difere"
    );
    let mut t = {
        let faces = || outra.faces().iter().map(ph2d_mesh::Face::verts);
        Tinta::nova(outra.vert_count(), faces(), 2)
    };
    let antes = t.amostras().to_vec();
    assert_eq!(preenche_plano(&mut t, &m, COR), Err(Recusa::NaoDescreve));
    assert_eq!(t.amostras(), &antes[..]);
}

/// ⛔⛔ **A metade CURTA das contagens: um plano com faces A MENOS é recusado**
/// — é a pior das três, porque com menos faces nada sai de alcance por si e a
/// tinta ficaria válida no sítio errado (§14 da tinta fina).
///
/// ⚠️ **A fixtura é o PREFIXO da malha de propósito:** todas as faces do plano
/// têm a forma da face homóloga da malha, logo só a pergunta das CONTAGENS o
/// pode recusar. A fixtura do `um_plano_de_outra_malha…` não isolava esta
/// metade — ali a pergunta da FORMA apanhava primeiro, e a mutação que apaga
/// as contagens sobreviveu.
#[test]
fn um_plano_com_faces_a_menos_e_recusado() {
    let m = malha();
    let menos = &m.faces()[..m.faces().len() - 1];
    let mut t = {
        let faces = || menos.iter().map(ph2d_mesh::Face::verts);
        Tinta::nova(m.vert_count(), faces(), 2)
    };
    let antes = t.amostras().to_vec();
    assert_eq!(preenche_plano(&mut t, &m, COR), Err(Recusa::NaoDescreve));
    assert_eq!(t.amostras(), &antes[..]);
}
