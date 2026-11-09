//! «Merge by distance»: a fixtura TEM o fenómeno — um quadrado de 64 Q4 com um cacho de 3 pontos a ≤ 2 Q4 num canto,
//! um par a 1 Q4 do outro lado da volta (último ↔ primeiro) e um entalhe estreito que NÃO pode ser comido.

use super::*;

/// 0..=2: o cacho no canto de baixo; 9 e 0 estão a 1 Q4 (a volta); 4..=6 o entalhe (4 Q4 de largura, 24 de fundo).
fn anel() -> Ring {
    vec![
        [0, 0],
        [2, 0],
        [2, 1],
        [64, 0],
        [64, 30],
        [40, 30],
        [40, 34],
        [64, 34],
        [64, 64],
        [0, 1],
    ]
}

fn todos(d: f32) -> Fusao {
    fundidos(&[anel()], &[], d).expect("há o que fundir")
}

#[test]
fn o_cacho_e_a_volta_viram_um_ponto_cada_um_na_media() {
    let f = todos(3.0);
    // 9, 0, 1 e 2 estão a ≤ 3 Q4 de (0,1), o primeiro do grupo pela volta: quatro pontos viram um.
    assert_eq!(f.fundidos, 3, "{:?}", f.rings);
    assert_eq!(f.retidos, 0);
    assert_eq!(f.rings[0].len(), 7);
    assert!(
        f.rings[0].contains(&[1, 1]),
        "a média de (0,0) (2,0) (2,1) (0,1): {:?}",
        f.rings[0]
    );
    // O entalhe (4 Q4 entre as paredes) fica: os pontos dele estão a 4 Q4, acima de 3.
    for p in [[40, 30], [40, 34], [64, 30], [64, 34]] {
        assert!(f.rings[0].contains(&p), "o entalhe ficou: {p:?}");
    }
}

#[test]
fn nada_perto_e_none_e_zero_nunca_funde() {
    assert_eq!(fundidos(&[anel()], &[], 0.5), None);
    assert_eq!(fundidos(&[anel()], &[], 0.0), None);
}

#[test]
fn com_selecao_so_os_selecionados_se_fundem() {
    let sel = [Ponto { anel: 0, i: 1 }, Ponto { anel: 0, i: 2 }];
    let f = fundidos(&[anel()], &sel, 3.0).expect("1 e 2 fundem");
    assert_eq!(f.fundidos, 1);
    assert!(
        f.rings[0].contains(&[0, 0]) && f.rings[0].contains(&[0, 1]),
        "0 e 9 ficam"
    );
    assert!(
        f.rings[0].contains(&[2, 1]),
        "média de (2,0) e (2,1) arredondada: {:?}",
        f.rings[0]
    );
    assert_eq!(
        f.selecionados.len(),
        1,
        "o ponto fundido continua selecionado"
    );
    let p = f.selecionados[0];
    assert_eq!(f.rings[0][p.i], [2, 1]);
}

#[test]
fn o_grupo_nao_cresce_em_cadeia() {
    // Doze pontos a 2 Q4 um do outro num lado: em cadeia viravam um; ancorado no primeiro, em pares de 3 Q4.
    let mut r: Ring = (0..12).map(|i| [i * 2, 0]).collect();
    r.extend([[64, 0], [64, 64], [0, 64]]);
    let f = fundidos(&[r], &[], 3.0).expect("funde");
    assert!(
        f.rings[0].len() >= 6 + 3,
        "um arco denso não pode sumir: {:?}",
        f.rings[0]
    );
}

#[test]
fn o_anel_que_ficaria_invalido_fica_o_mais_perto_possivel() {
    // Um triângulo com um par perto: fundir deixaria 2 pontos — recusado e contado.
    let r: Ring = vec![[0, 0], [1, 0], [64, 0], [0, 64]];
    let f = fundidos(&[r.clone()], &[], 2.0).expect("o par é candidato");
    assert_eq!(f.rings[0].len(), 3, "{:?}", f.rings);
    let t: Ring = vec![[0, 0], [1, 0], [0, 1]];
    let f = fundidos(&[t.clone()], &[], 2.0).expect("candidato");
    assert_eq!((f.rings[0].clone(), f.fundidos, f.retidos), (t, 0, 1));
}

#[test]
fn varios_aneis_cada_um_com_o_seu_grupo() {
    let buraco: Ring = vec![[44, 4], [44, 20], [45, 20], [56, 20], [56, 4]];
    let f = fundidos(&[anel(), buraco], &[], 3.0).expect("funde");
    assert_eq!(f.rings[1].len(), 4);
    assert_eq!(f.fundidos, 4);
}
