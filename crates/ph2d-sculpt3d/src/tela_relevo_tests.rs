//! Os gates da ESPESSURA da tela pousada na peça (`docs/3D/29`, D2/D3) — a
//! janela de alturas, a conversão de píxel para a peça e a lei da pousada.

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;

use crate::SculptStroke;
use crate::tela_na_malha::{Relevo, Tela, TelaNaMalha, Vista};
use crate::tela_na_malha_tests::{LADO, malha, tudo, vista};
use crate::tinta_fina::TintaDoTraco;

/// ⭐ **A janela lê-se com a convenção da cor** — o centro do píxel `i` em
/// `i + 0,5` devolve o valor dele ao bit, a meio caminho a média, e fora da
/// janela repete-se a borda.
#[test]
fn a_janela_le_se_com_a_convencao_da_cor() {
    let px = [1.0f32, 3.0, 5.0, 7.0]; // janela 2×2 em (10, 20)
    let r = Relevo {
        px: &px,
        janela: [10, 20, 2, 2],
    };
    assert_eq!(r.em(10.5, 20.5), 1.0);
    assert_eq!(r.em(11.5, 20.5), 3.0);
    assert_eq!(r.em(10.5, 21.5), 5.0);
    assert_eq!(r.em(11.0, 20.5), 2.0, "a meio caminho é a média");
    assert_eq!(r.em(0.0, 0.0), 1.0, "fora da janela repete-se a borda");
    assert_eq!(r.em(99.0, 99.0), 7.0);
}

/// ⭐⭐ **Um píxel mede o que a VISTA diz naquele ponto** — numa vista sem
/// perspectiva é constante (`2/LADO`), e numa perspectiva DOBRA com a
/// distância ao olho. Os dois são o CONTROLO um do outro: uma conversão que
/// devolvesse uma constante passaria o primeiro e reprovaria o segundo.
#[test]
fn um_pixel_mede_o_que_a_vista_diz_naquele_ponto() {
    let plana = vista();
    let w = plana.mundo_por_pixel([0.3, -0.2, 0.0]).expect("à frente");
    assert!((w - 2.0 / LADO as f32).abs() < 1e-5, "sem perspectiva: {w}");

    // Olho em `z = 10` a olhar para `-z`: `w_clip = 10 − z`.
    let persp = Vista::nova(
        [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, -1.0, //
            0.0, 0.0, 0.0, 10.0,
        ],
        (LADO, LADO),
        [0.0, 0.0, 10.0],
    );
    let perto = persp.mundo_por_pixel([0.0, 0.0, 0.0]).expect("à frente");
    let longe = persp.mundo_por_pixel([0.0, 0.0, -10.0]).expect("à frente");
    let razao = longe / perto;
    assert!(
        (razao - 2.0).abs() < 1e-3,
        "a distância dobrou e o píxel mediu {razao}×"
    );
    assert!(
        persp.mundo_por_pixel([0.0, 0.0, 10.0]).is_none(),
        "no olho não há medida"
    );
}

/// Pousa a tela `rgba` com a espessura constante `h_px` sobre um plano de
/// nível `nivel`, e devolve o traço (com o plano ainda emprestado).
fn pousa(m: &mut Mesh, nivel: u8, rgba: &[u8], h_px: Option<f32>, vezes: u32) -> SculptStroke {
    let faces: Vec<Vec<u32>> = m.faces().iter().map(|f| f.verts().to_vec()).collect();
    let tinta = Tinta::nova(m.vert_count(), faces.iter().map(Vec::as_slice), nivel);
    pousa_sobre(m, tinta, rgba, h_px, vezes)
}

/// A mesma pousada sobre um plano que JÁ existe — o traço seguinte.
fn pousa_sobre(
    m: &mut Mesh,
    tinta: Tinta,
    rgba: &[u8],
    h_px: Option<f32>,
    vezes: u32,
) -> SculptStroke {
    let mut s = SculptStroke::default();
    s.begin(m);
    s.tinta_fina = Some(TintaDoTraco::nova(tinta, 0));
    let px = vec![h_px.unwrap_or(0.0); (LADO * LADO) as usize];
    let relevo = h_px.map(|_| Relevo {
        px: &px,
        janela: [0, 0, LADO, LADO],
    });
    let t = Tela {
        rgba,
        largura: LADO,
        altura: LADO,
    };
    for _ in 0..vezes {
        let mut sessao = TelaNaMalha::nova(
            m,
            vista(),
            s.tinta_fina.as_ref().expect("e").tinta().amostras().len(),
        );
        s.pousa_a_tela_com_relevo(m, &mut sessao, &t, relevo.as_ref(), tudo());
    }
    s
}

/// ⭐⭐⭐ **GATE — `nova = antes + altura`, a altura convertida no ponto** — e
/// pousar duas vezes a mesma tela dá EXACTAMENTE o que pousar uma.
///
/// ⚠️ A tela é TRANSPARENTE de propósito: a espessura tem de chegar mesmo sem
/// cor nenhuma (o esculpir do impasto não pinta), e é o que o portão do
/// `vazia` teria cortado.
#[test]
fn a_espessura_pousa_se_convertida_e_nao_acumula() {
    let transparente = vec![0u8; (LADO * LADO * 4) as usize];
    let h_px = 5.0;
    let mut m = malha(2);
    let um = pousa(&mut m, 2, &transparente, Some(h_px), 1);
    let t1 = um.tinta_fina.as_ref().expect("emprestada").tinta();
    let alt = t1.alturas().expect("a espessura criou o relevo");
    let esperado = h_px * 2.0 / LADO as f32;
    let tocadas = alt.iter().filter(|&&a| a != 0.0).count();
    assert_eq!(tocadas, alt.len(), "a tela cobre a peça: toda amostra sobe");
    for (i, &a) in alt.iter().enumerate() {
        assert!(
            (a - esperado).abs() < 1e-5,
            "amostra {i}: {a} contra {esperado} (h_px × o píxel no ponto)"
        );
    }

    let mut m2 = malha(2);
    let duas = pousa(&mut m2, 2, &transparente, Some(h_px), 2);
    assert_eq!(
        duas.tinta_fina.as_ref().expect("e").tinta().alturas(),
        Some(alt),
        "pousar a mesma tela duas vezes acumulou espessura"
    );
}

/// ⭐⭐ **CONTROLO — sem espessura na tela, o plano não ganha relevo** (nem um
/// vector de zeros), e com espessura ZERO também não.
#[test]
fn sem_espessura_o_plano_nao_ganha_relevo() {
    let opaca = [255u8, 0, 0, 255].repeat((LADO * LADO) as usize);
    let mut m = malha(2);
    let s = pousa(&mut m, 2, &opaca, None, 1);
    let t = s.tinta_fina.as_ref().expect("e").tinta();
    assert!(!t.tem_relevo(), "uma pincelada de cor criou relevo");
    assert!(
        t.amostras().iter().all(|c| *c == [1.0, 0.0, 0.0]),
        "CONTROLO: a cor pousou"
    );

    let mut m = malha(2);
    let s = pousa(&mut m, 2, &opaca, Some(0.0), 1);
    assert!(
        !s.tinta_fina.as_ref().expect("e").tinta().tem_relevo(),
        "uma espessura ZERO criou o vector de alturas"
    );
}

/// ⭐⭐ **GATE — o traço seguinte SOMA à espessura de antes** (`docs/3D/29`,
/// D3). O gate de cima parte de um plano liso, onde «antes + altura» e «só a
/// altura» são o mesmo número; este parte de relevo que já lá está, e é o
/// único que os separa.
#[test]
fn o_traco_seguinte_soma_a_espessura_de_antes() {
    let transparente = vec![0u8; (LADO * LADO * 4) as usize];
    let mut m = malha(2);
    let primeiro = pousa(&mut m, 2, &transparente, Some(5.0), 1);
    let plano = primeiro.tinta_fina.as_ref().expect("e").tinta().clone();
    let antes = plano.alturas().expect("o 1.º traço criou relevo").to_vec();

    let segundo = pousa_sobre(&mut m, plano, &transparente, Some(3.0), 1);
    let depois = segundo
        .tinta_fina
        .as_ref()
        .expect("e")
        .tinta()
        .alturas()
        .expect("relevo")
        .to_vec();
    let somado = 3.0 * 2.0 / LADO as f32;
    for (i, (a, d)) in antes.iter().zip(&depois).enumerate() {
        assert!(
            (d - (a + somado)).abs() < 1e-5,
            "amostra {i}: {d} contra {a} + {somado} — o 2.º traço substituiu em vez de somar"
        );
    }
}
