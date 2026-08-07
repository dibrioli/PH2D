//! Os gates do **canal assado e da validade dele**.
//!
//! ⚠️ **Eles dirigem as portas do PRODUTO** (`refine_in_sphere`,
//! `collapse_in_sphere`), não um `VertexAppend` montado à mão. Um gate que
//! constrói a edição de topologia por dentro testa a aritmética que ele mesmo
//! escreveu; o que precisa ser verdade é que o AO sobreviva ao caminho que o
//! **pincel** de fato percorre.

use super::*;
use crate::collapse::collapse_in_sphere;
use crate::dyntopo::refine_in_sphere;
use crate::remap::Remap;
use crate::shapes;
use crate::{Face, RegionScratch};

fn tri_sphere(rings: usize, segs: usize) -> Mesh {
    let mut m = shapes::uv_sphere(rings, segs, 1.0);
    m.triangulate();
    m
}

/// Um AO que VARIA com o lugar — é ele que torna a compactação observável.
/// Um plano uniforme sobrevive a qualquer remap, inclusive a um errado.
fn seed_by_position(m: &mut Mesh) {
    let ao: Vec<f32> = m
        .positions()
        .iter()
        .map(|p| ((p[0] + 1.5) / 3.0).clamp(0.0, 1.0))
        .collect();
    m.set_ao(ao);
}

fn expected_at(p: [f32; 3]) -> f32 {
    ((p[0] + 1.5) / 3.0).clamp(0.0, 1.0)
}

// ------------------------------------------------- a validade, sem topologia

/// **Sem bake não há o que envelhecer.** Uma malha nunca assada que se
/// declarasse velha faria a UI anunciar um problema que ninguém tem.
#[test]
fn sem_bake_nao_ha_o_que_envelhecer() {
    let mut m = tri_sphere(8, 12);
    assert!(m.ao().is_none());
    assert!(!m.ao_is_stale(), "malha sem bake nao pode estar velha");

    // Mexer na forma de uma malha sem AO continua não produzindo velhice.
    m.positions_mut()[0][1] += 0.3;
    assert!(
        !m.ao_is_stale(),
        "sem AO, mexer na forma nao envelhece nada"
    );
}

/// ⚠️ **O gate central do canal:** mexer nas posições deixa o AO velho. É a
/// única coisa que separa *um número medido* de *um número que já foi medido*,
/// e o modo de falha sem ele é invisível — uma fresta clara demais lê como
/// escolha de iluminação.
#[test]
fn mexer_nas_posicoes_deixa_o_ao_velho() {
    let mut m = tri_sphere(8, 12);
    m.set_ao(vec![1.0; m.vert_count()]);
    assert!(!m.ao_is_stale(), "o controle: acabou de ser assado");

    m.positions_mut()[0][1] += 0.3;
    assert!(
        m.ao_is_stale(),
        "a forma mudou e o AO nao se declarou velho"
    );
}

/// E assar de novo é o **único** jeito de a validade voltar.
#[test]
fn assar_de_novo_devolve_a_validade() {
    let mut m = tri_sphere(8, 12);
    m.set_ao(vec![1.0; m.vert_count()]);
    m.positions_mut()[0][1] += 0.3;
    assert!(m.ao_is_stale());

    m.set_ao(vec![0.5; m.vert_count()]);
    assert!(!m.ao_is_stale(), "um bake novo tem de devolver a validade");
    assert_eq!(m.ao().unwrap()[0], 0.5);
}

/// ⚠️ **O undo devolve a VALIDADE junto com os números.** Separar os dois
/// deixaria quem restaura reinstalar um AO velho anunciando-o como fresco —
/// e aí o canal mente com a UI concordando.
#[test]
fn o_undo_devolve_a_validade_junto() {
    for velho in [false, true] {
        let mut m = tri_sphere(8, 12);
        m.set_ao(vec![0.25; m.vert_count()]);
        if velho {
            m.positions_mut()[0][1] += 0.3;
        }
        assert_eq!(m.ao_is_stale(), velho);

        let (plano, stale) = m.take_ao().expect("havia AO");
        assert_eq!(stale, velho, "o take tem de trazer a validade");
        assert!(m.ao().is_none(), "o take tira o plano");
        assert!(!m.ao_is_stale(), "sem plano nao ha velhice");

        m.put_ao(plano, stale);
        assert_eq!(m.ao_is_stale(), velho, "o put tem de repor a validade");
    }
}

/// Tirar o canal devolve os 4 B/vértice **e** zera a velhice.
#[test]
fn limpar_devolve_o_plano_e_a_validade() {
    let mut m = tri_sphere(8, 12);
    m.set_ao(vec![1.0; m.vert_count()]);
    m.positions_mut()[0][1] += 0.3;
    m.clear_ao();
    assert!(m.ao().is_none() && !m.ao_is_stale());
}

/// Um plano de comprimento errado não falha, ele **lê o vizinho** — a mesma
/// razão de o `put_masks` validar.
#[test]
#[should_panic(expected = "um valor por vertice")]
fn um_ao_de_comprimento_errado_e_recusado() {
    let mut m = tri_sphere(8, 12);
    m.set_ao(vec![1.0; 3]);
}

// ----------------------------------------------------- as portas de TOPOLOGIA

/// **O vértice novo herda a MÉDIA dos pais, e não um placeholder.**
///
/// ⚠️ O oráculo é o *uniforme que continua uniforme*: se o splice empurrasse
/// `0.0` (o tratamento que a curvatura recebe, e corretamente, porque o
/// `refresh_region` a reescreve), o mínimo cairia a zero — um buraco preto no
/// meio de uma superfície assada, que **ninguém reescreve depois**.
#[test]
fn o_vertice_novo_herda_a_media_dos_pais() {
    let mut m = tri_sphere(12, 18);
    let antes = m.vert_count();
    m.set_ao(vec![1.0; antes]);

    let r = refine_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.6,
        0.05,
        &mut Vec::new(),
        &mut RegionScratch::default(),
    );
    assert!(
        m.vert_count() > antes,
        "o controle: algo tem de ser partido ({r:?})"
    );

    let ao = m.ao().expect("o AO tem de sobreviver ao refino");
    assert_eq!(ao.len(), m.vert_count(), "um valor por vertice, sempre");
    let min = ao.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        (min - 1.0).abs() < 1e-6,
        "um uniforme tem de continuar uniforme; o minimo caiu para {min:.4} \
         (placeholder em vez de media dos pais?)"
    );
}

/// E o refino **deixa o canal velho** — a geometria ganhou vértices que ninguém
/// mediu.
#[test]
fn o_refino_deixa_o_ao_velho() {
    let mut m = tri_sphere(12, 18);
    m.set_ao(vec![1.0; m.vert_count()]);
    assert!(!m.ao_is_stale(), "o controle");

    refine_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.6,
        0.05,
        &mut Vec::new(),
        &mut RegionScratch::default(),
    );
    assert!(
        m.ao_is_stale(),
        "a topologia cresceu e o AO nao se declarou velho"
    );
}

/// ⚠️ **O AO viaja a COMPACTAÇÃO** — o gate que a curvatura já tem, e o motivo
/// de ele existir: quando o colapso renumera os vértices, um plano que não for
/// remapeado fica com os valores **de outros vértices**, e a malha continua
/// perfeitamente válida com o sombreamento embaralhado.
///
/// O oráculo é um AO que VARIA com a posição: um plano uniforme sobrevive
/// inclusive a um remap errado, e seria verde por vácuo.
#[test]
fn o_ao_viaja_a_compactacao() {
    let mut m = tri_sphere(16, 24);
    seed_by_position(&mut m);
    let antes = m.vert_count();

    let c = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.8,
        0.35,
        &mut Remap::default(),
        &mut RegionScratch::default(),
    );
    assert!(
        m.vert_count() < antes,
        "o controle: algo tem de colapsar ({c:?})"
    );

    let ao = m.ao().expect("o AO tem de sobreviver ao colapso");
    assert_eq!(ao.len(), m.vert_count(), "um valor por vertice, sempre");

    // Cada sobrevivente ainda carrega o valor do LUGAR onde está. A folga
    // acomoda os vértices que absorveram um vizinho (a média dos dois).
    let mut pior = 0.0f32;
    for (v, p) in m.positions().iter().enumerate() {
        pior = pior.max((ao[v] - expected_at(*p)).abs());
    }
    assert!(
        pior < 0.05,
        "o AO nao viajou com os vertices: pior desvio {pior:.4}"
    );
    assert!(
        m.ao_is_stale(),
        "a topologia encolheu e o AO nao se declarou velho"
    );
}

/// A malha sem AO atravessa as duas portas sem alocar o canal — a metade
/// *ausência* do par presença/ausência.
#[test]
fn sem_ao_as_portas_de_topologia_nao_inventam_o_canal() {
    let mut m = tri_sphere(12, 18);
    refine_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.6,
        0.05,
        &mut Vec::new(),
        &mut RegionScratch::default(),
    );
    collapse_in_sphere(
        &mut m,
        [0.0, 0.0, -1.0],
        0.8,
        0.35,
        &mut Remap::default(),
        &mut RegionScratch::default(),
    );
    assert!(m.ao().is_none(), "ninguem assou: o canal nao pode existir");
    assert!(!m.ao_is_stale());
}

/// O `truncate` (a inversa de fechar buraco) também carrega o canal.
#[test]
fn o_truncate_carrega_o_canal() {
    let mut m = tri_sphere(10, 14);
    m.set_ao(vec![0.75; m.vert_count()]);
    let (v, f) = (m.vert_count(), m.face_count());
    let faces_ok = m
        .faces()
        .iter()
        .all(|face: &Face| face.verts().iter().all(|&x| (x as usize) < v - 1));
    if !faces_ok {
        return; // a fixture nao permite cortar o ultimo vertice; nada a provar
    }
    m.truncate(v - 1, f).expect("cortar o ultimo vertice");
    let ao = m.ao().expect("o canal sobrevive");
    assert_eq!(ao.len(), m.vert_count());
    assert!(m.ao_is_stale(), "cortar geometria envelhece o bake");
}
