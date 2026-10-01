//! Os gates do RELEVO da tela pousada na peça (`docs/3D/29`, D2/D3 e §6) — a
//! janela de altura e corpo, a conversão de píxel para a peça, a lei da
//! DIFERENÇA contra a semente, e a semente de relevo que a tela recebe.

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;

use crate::SculptStroke;
use crate::tela_na_malha::{Relevo, Tela, TelaNaMalha, Vista};
use crate::tela_na_malha_tests::{LADO, malha, tudo, vista};
use crate::tela_semente_relevo::semente_relevo;
use crate::tinta_fina::TintaDoTraco;

/// ⭐ **A janela lê-se com a convenção da cor** — o centro do píxel `i` em
/// `i + 0,5` devolve o valor dele ao bit, a meio caminho a média, e fora da
/// janela repete-se a borda — nos DOIS canais.
#[test]
fn a_janela_le_se_com_a_convencao_da_cor() {
    let px = [1.0f32, 3.0, 5.0, 7.0]; // janela 2×2 em (10, 20)
    let corpo = [0.1f32, 0.3, 0.5, 0.7];
    let r = Relevo {
        px: &px,
        corpo: &corpo,
        janela: [10, 20, 2, 2],
    };
    assert_eq!(r.em(10.5, 20.5), [1.0, 0.1]);
    assert_eq!(r.em(11.5, 20.5), [3.0, 0.3]);
    assert_eq!(r.em(10.5, 21.5), [5.0, 0.5]);
    assert_eq!(r.em(11.0, 20.5)[0], 2.0, "a meio caminho é a média");
    assert!((r.em(11.0, 20.5)[1] - 0.2).abs() < 1e-7, "o corpo também");
    assert_eq!(
        r.em(0.0, 0.0),
        [1.0, 0.1],
        "fora da janela repete-se a borda"
    );
    assert_eq!(r.em(99.0, 99.0), [7.0, 0.7]);
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

/// O relevo de UMA tela: `[h_px, corpo]` constantes na tela inteira.
#[derive(Clone, Copy)]
struct Uniforme {
    px: f32,
    corpo: f32,
}

/// Pousa a tela `rgba` com o relevo `rel` sobre um plano novo de nível `nivel`,
/// e devolve o traço (com o plano ainda emprestado).
fn pousa(m: &mut Mesh, nivel: u8, rgba: &[u8], rel: Option<Uniforme>, vezes: u32) -> SculptStroke {
    let faces: Vec<Vec<u32>> = m.faces().iter().map(|f| f.verts().to_vec()).collect();
    let tinta = Tinta::nova(m.vert_count(), faces.iter().map(Vec::as_slice), nivel);
    pousa_sobre(m, tinta, rgba, rel, None, vezes)
}

/// A mesma pousada sobre um plano que JÁ existe — o traço seguinte —, com a
/// semente de relevo `semente` quando a tela foi semeada.
fn pousa_sobre(
    m: &mut Mesh,
    tinta: Tinta,
    rgba: &[u8],
    rel: Option<Uniforme>,
    semente: Option<Uniforme>,
    vezes: u32,
) -> SculptStroke {
    let mut s = SculptStroke::default();
    s.begin(m);
    s.tinta_fina = Some(TintaDoTraco::nova(tinta, 0));
    let n = (LADO * LADO) as usize;
    let u = rel.unwrap_or(Uniforme {
        px: 0.0,
        corpo: 0.0,
    });
    let (px, corpo) = (vec![u.px; n], vec![u.corpo; n]);
    let relevo = rel.map(|_| Relevo {
        px: &px,
        corpo: &corpo,
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
        if let Some(sem) = semente {
            assert!(sessao.com_semente_relevo(vec![sem.px; n], vec![sem.corpo; n]));
        }
        s.pousa_a_tela_com_relevo(m, &mut sessao, &t, relevo.as_ref(), tudo());
    }
    s
}

fn relevo_de(s: &SculptStroke) -> Vec<[f32; 2]> {
    s.tinta_fina
        .as_ref()
        .expect("emprestada")
        .tinta()
        .relevo()
        .expect("relevo")
        .to_vec()
}

/// ⭐⭐⭐ **GATE — `nova = antes + (tela − semente)`, a altura convertida no
/// ponto e o CORPO ao lado** — e pousar duas vezes a mesma tela dá EXACTAMENTE
/// o que pousar uma.
///
/// ⚠️ A tela é TRANSPARENTE de propósito: a espessura tem de chegar mesmo sem
/// cor nenhuma (o esculpir do impasto não pinta), e é o que o portão do
/// `vazia` teria cortado.
#[test]
fn a_espessura_pousa_se_convertida_e_nao_acumula() {
    let transparente = vec![0u8; (LADO * LADO * 4) as usize];
    let rel = Uniforme {
        px: 5.0,
        corpo: 0.75,
    };
    let mut m = malha(2);
    let um = pousa(&mut m, 2, &transparente, Some(rel), 1);
    let alt = relevo_de(&um);
    let esperado = rel.px * 2.0 / LADO as f32;
    for (i, &a) in alt.iter().enumerate() {
        assert!(
            (a[0] - esperado).abs() < 1e-5,
            "amostra {i}: {} contra {esperado} (h_px × o píxel no ponto)",
            a[0]
        );
        assert!(
            (a[1] - rel.corpo).abs() < 1e-6,
            "amostra {i}: o corpo pousou {} contra {}",
            a[1],
            rel.corpo
        );
    }

    let mut m2 = malha(2);
    let duas = pousa(&mut m2, 2, &transparente, Some(rel), 2);
    assert_eq!(
        relevo_de(&duas),
        alt,
        "pousar a mesma tela duas vezes acumulou espessura"
    );
}

/// ⭐⭐ **CONTROLO — sem espessura na tela, o plano não ganha relevo** (nem um
/// vector de zeros), e com espessura E corpo ZERO também não.
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
    let s = pousa(
        &mut m,
        2,
        &opaca,
        Some(Uniforme {
            px: 0.0,
            corpo: 0.0,
        }),
        1,
    );
    assert!(
        !s.tinta_fina.as_ref().expect("e").tinta().tem_relevo(),
        "uma espessura ZERO criou o vector de relevo"
    );
}

/// ⭐⭐ **GATE — o traço seguinte SOMA à espessura de antes** (`docs/3D/29`,
/// D3). O gate de cima parte de um plano liso, onde «antes + altura» e «só a
/// altura» são o mesmo número; este parte de relevo que já lá está, e é o
/// único que os separa. ⚠️ Sem semente (a tela começou lisa) a lei é a de
/// antes, e o corpo SATURA em `1`.
#[test]
fn o_traco_seguinte_soma_a_espessura_de_antes() {
    let transparente = vec![0u8; (LADO * LADO * 4) as usize];
    let mut m = malha(2);
    let primeiro = pousa(
        &mut m,
        2,
        &transparente,
        Some(Uniforme {
            px: 5.0,
            corpo: 0.75,
        }),
        1,
    );
    let plano = primeiro.tinta_fina.as_ref().expect("e").tinta().clone();
    let antes = plano.relevo().expect("o 1.º traço criou relevo").to_vec();

    let segundo = pousa_sobre(
        &mut m,
        plano,
        &transparente,
        Some(Uniforme {
            px: 3.0,
            corpo: 0.5,
        }),
        None,
        1,
    );
    let depois = relevo_de(&segundo);
    let somado = 3.0 * 2.0 / LADO as f32;
    for (i, (a, d)) in antes.iter().zip(&depois).enumerate() {
        assert!(
            (d[0] - (a[0] + somado)).abs() < 1e-5,
            "amostra {i}: {} contra {} + {somado} — o 2.º traço substituiu em vez de somar",
            d[0],
            a[0]
        );
        assert_eq!(d[1], 1.0, "amostra {i}: o corpo não saturou em 1");
    }
}

/// ⭐⭐⭐ **GATE — com a tela SEMEADA, o que o pincel não tocou não muda nada**
/// (`docs/3D/29` §6, report do dono de 01/10). A tela começa com o relevo da
/// peça; um quadro em que ela ainda É a semente tem diferença ZERO em toda
/// amostra, e o plano sai AO BIT.
///
/// ⚠️ O CONTROLO é a mesma tela SEM semente: com a lei antiga (`antes + tela`)
/// ela DOBRAVA a espessura da peça — é o que semear sem a lei da diferença
/// faria a cada pincelada.
#[test]
fn com_a_tela_semeada_o_que_nao_mudou_nao_mexe() {
    let transparente = vec![0u8; (LADO * LADO * 4) as usize];
    let base = Uniforme {
        px: 5.0,
        corpo: 0.75,
    };
    let mut m = malha(2);
    let primeiro = pousa(&mut m, 2, &transparente, Some(base), 1);
    let plano = primeiro.tinta_fina.as_ref().expect("e").tinta().clone();
    let antes = plano.relevo().expect("relevo").to_vec();

    let igual = pousa_sobre(
        &mut m,
        plano.clone(),
        &transparente,
        Some(base),
        Some(base),
        1,
    );
    assert_eq!(
        relevo_de(&igual),
        antes,
        "a tela semeada e intocada mexeu no relevo"
    );
    assert!(
        !igual.tinta_fina.as_ref().expect("e").relevo_mudou(),
        "o desfazer registaria um relevo que não mudou"
    );

    // CONTROLO: sem a semente a mesma tela SOMA outra vez.
    let dobrou = pousa_sobre(&mut m, plano, &transparente, Some(base), None, 1);
    let d = relevo_de(&dobrou);
    assert!(
        (d[0][0] - 2.0 * antes[0][0]).abs() < 1e-5,
        "CONTROLO: sem semente a tela devia somar ({} contra {})",
        d[0][0],
        antes[0][0]
    );
}

/// ⭐⭐⭐ **GATE — uma ferramenta que BAIXA a tela baixa a peça** — o report do
/// dono de 01/10 (*«smooth, knife e outras tools não funcionam no relevo»*).
/// Com a tela semeada com `5` px e o alisar a deixá-la em `2`, a peça desce
/// `3` px convertidos, e o corpo desce com a tela.
#[test]
fn uma_ferramenta_que_baixa_a_tela_baixa_a_peca() {
    let transparente = vec![0u8; (LADO * LADO * 4) as usize];
    let base = Uniforme {
        px: 5.0,
        corpo: 0.75,
    };
    let mut m = malha(2);
    let primeiro = pousa(&mut m, 2, &transparente, Some(base), 1);
    let plano = primeiro.tinta_fina.as_ref().expect("e").tinta().clone();
    let antes = plano.relevo().expect("relevo").to_vec();

    let alisada = Uniforme {
        px: 2.0,
        corpo: 0.25,
    };
    let s = pousa_sobre(&mut m, plano, &transparente, Some(alisada), Some(base), 1);
    let depois = relevo_de(&s);
    let desceu = 3.0 * 2.0 / LADO as f32;
    for (i, (a, d)) in antes.iter().zip(&depois).enumerate() {
        assert!(
            (d[0] - (a[0] - desceu)).abs() < 1e-5,
            "amostra {i}: a peça não desceu com a tela ({} contra {})",
            d[0],
            a[0] - desceu
        );
        assert!(
            (d[1] - (a[1] - 0.5)).abs() < 1e-6,
            "amostra {i}: o corpo não desceu com a tela ({})",
            d[1]
        );
    }
}

/// ⭐⭐⭐ **GATE — a semente de relevo é o relevo da peça na unidade da tela**:
/// a altura dividida pelo píxel no ponto e o corpo como está, onde a peça se
/// vê; e `None` num plano sem relevo (a tela começa lisa, a lei de antes).
#[test]
fn a_semente_de_relevo_e_o_relevo_da_peca_em_pixeis() {
    let m = malha(2);
    let faces: Vec<Vec<u32>> = m.faces().iter().map(|f| f.verts().to_vec()).collect();
    let mut t = Tinta::nova(m.vert_count(), faces.iter().map(Vec::as_slice), 2);
    assert!(
        semente_relevo(&m, &t, &vista()).is_none(),
        "CONTROLO: um plano sem relevo não semeia nada"
    );
    let h = 0.04f32;
    for r in t.relevo_mut() {
        *r = [h, 0.6];
    }
    let s = semente_relevo(&m, &t, &vista()).expect("há relevo");
    let esperado = h / (2.0 / LADO as f32);
    assert_eq!(s.px.len(), (LADO * LADO) as usize);
    for (k, (&px, &c)) in s.px.iter().zip(&s.corpo).enumerate() {
        assert!(
            (px - esperado).abs() < 1e-3,
            "píxel {k}: {px} contra {esperado}"
        );
        assert!((c - 0.6).abs() < 1e-6, "píxel {k}: corpo {c}");
    }
}
