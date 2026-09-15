//! Gates da fronteira do corte.

use super::*;
use ph2d_mesh::shapes;

/// Uma lâmina: o cubo da casa, deslocado em `x`.
fn lamina(cx: f32, meia: f32) -> Mesh {
    let mut m = shapes::cube(meia * 2.0);
    for p in m.positions_mut() {
        p[0] += cx;
    }
    m
}

/// **O corte tira volume — e a peça continua a ENCERRAR volume.**
///
/// ⚠️ A segunda metade não é decoração: um corte que abre a peça deixa-a sem
/// dentro, e a operação seguinte (outra booleana, um remesh) passa a recusar.
/// *Um corte que fecha é a diferença entre uma ferramenta e uma armadilha.*
#[test]
fn um_corte_tira_volume_e_a_peca_continua_fechada() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    assert!(bola.is_closed(), "a fixtura tem de entrar fechada");
    let antes = bola.bounds().max[0];

    let out = corta(&bola, &lamina(1.2, 0.6), Op::Subtrair).expect("o corte");

    assert!(
        out.bounds().max[0] < antes - 0.05,
        "a lâmina estava em x ∈ [0,6; 1,8] e o `x` máximo não recuou: {} contra {antes}",
        out.bounds().max[0]
    );
    assert_eq!(
        ph2d_mesh::border_edges(&out),
        0,
        "o corte deixou a peça ABERTA — ela deixa de ter dentro"
    );
}

/// ⭐⭐⭐ **A PROPRIEDADE QUE DECIDE A ARQUITECTURA: longe do corte, nem um bit.**
///
/// ⚠️ **Este é o gate que separa um CORTE de um corte-mais-remalhamento.** Uma
/// escultura tem densidade **autorada** — fino onde o artista trabalhou, grosso
/// onde não —, e a rota barata desta casa (`malha → campo → malha`) devolve-a
/// uniforme: medido, `0` vértices preservados em `6` de `6` células, contra
/// `26 533` de `26 533` aqui (`SPEC_trim_gesture.md` §1.1–§1.2, e a sonda
/// `ph2d_sdf::remesh::tests::diag_o_preco_da_volta_por_campo` do nosso lado).
///
/// ⇒ *se alguém trocar este motor por um que re-tessela tudo, este gate é o que
/// reprova* — e nenhuma contagem agregada o faria, porque a contagem de saída
/// de uma rota por voxel até pode ser parecida.
#[test]
fn longe_do_corte_nenhum_vertice_se_move_um_bit() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    let longe: std::collections::BTreeSet<[u32; 3]> = bola
        .positions()
        .iter()
        .filter(|p| p[0] < -0.5)
        .map(|p| [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()])
        .collect();
    assert!(
        longe.len() > 50,
        "a fixtura tem de ter população longe do corte, e tem {}",
        longe.len()
    );

    let out = corta(&bola, &lamina(1.2, 0.6), Op::Subtrair).expect("o corte");
    let saida: std::collections::BTreeSet<[u32; 3]> = out
        .positions()
        .iter()
        .map(|p| [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()])
        .collect();

    let perdidos = longe.difference(&saida).count();
    assert_eq!(
        perdidos,
        0,
        "{perdidos} de {} vértices do lado OPOSTO ao corte mudaram — este motor \
         re-tessela longe de onde a lâmina passou, e isso destrói a densidade \
         que o artista autorou",
        longe.len()
    );
}

/// ⛔⛔⛔ **A RECUSA ACONTECE ANTES DE OPERAR — e a metade que importa é que ela
/// NÃO é `Ok` sobre uma malha vazia.**
///
/// Medido no motor cru, fora do repo: com peça aberta o estado da ENTRADA diz
/// `NotManifold` e o do RESULTADO diz **«sem erro»**, com `0` vértices. ⇒ quem
/// verificar o resultado devolve `Ok(vazio)` e o artista perde a escultura.
///
/// ⚠️ **O CONTROLO está dentro do gate:** a mesma lâmina, na mesma posição,
/// sobre a peça FECHADA, tem de cortar. Sem ele este teste passaria com uma
/// porta que recusa **sempre**.
#[test]
fn uma_peca_aberta_e_recusada_e_nunca_devolvida_vazia() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    let mut aberta = Mesh::from_parts(
        bola.positions().to_vec(),
        bola.faces()[..bola.face_count() - 1].to_vec(),
    )
    .expect("a peça com uma face a menos");
    assert!(
        !aberta.is_closed(),
        "a fixtura tem de entrar ABERTA, senão este gate não vê o fenómeno"
    );
    let _ = &mut aberta;

    match corta(&aberta, &lamina(1.2, 0.6), Op::Subtrair) {
        Err(Recusa::PecaAberta) => {}
        Err(outra) => panic!("recusou pela razão errada: {outra:?}"),
        Ok(m) => panic!(
            "DEVOLVEU {} vértices sobre uma peça aberta — se for `0`, é a \
             armadilha do motor a chegar ao artista",
            m.vert_count()
        ),
    }

    // ⭐ O CONTROLO: fechada, a mesma lâmina corta.
    assert!(
        corta(&bola, &lamina(1.2, 0.6), Op::Subtrair).is_ok(),
        "a porta recusa SEMPRE — o gate acima não estaria a medir a abertura"
    );
}

/// **A lâmina aberta é nomeada À PARTE.**
///
/// ⚠️ *«Alguma coisa está aberta» manda o artista procurar nos dois sítios.* As
/// duas recusas existem porque as curas são diferentes: fechar a peça é um
/// remesh; uma lâmina aberta é um gesto degenerado, e a cura é repetir o gesto.
#[test]
fn a_lamina_aberta_e_nomeada_a_parte() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    let l = lamina(1.2, 0.6);
    let rota = Mesh::from_parts(
        l.positions().to_vec(),
        l.faces()[..l.face_count() - 1].to_vec(),
    )
    .expect("a lâmina com uma face a menos");

    assert_eq!(
        corta(&bola, &rota, Op::Subtrair).err(),
        Some(Recusa::LaminaAberta),
        "a recusa tem de nomear a LÂMINA — a peça está fechada"
    );
}

/// **Um corte que apagaria a peça inteira é RECUSA, não resultado.**
///
/// ⚠️ É a única recusa que é facto sobre a SAÍDA, e ela existe pelo mesmo
/// argumento do cabeçalho um nível acima: devolver `Ok` sobre o nada é o que
/// esta porta existe para impedir.
#[test]
fn um_corte_que_apagaria_tudo_e_recusado() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    assert_eq!(
        corta(&bola, &lamina(0.0, 4.0), Op::Subtrair).err(),
        Some(Recusa::ResultadoVazio),
        "uma lâmina que engole a peça tem de recusar"
    );
}

/// **Cada recusa diz o que FALTA, não que falhou.**
#[test]
fn cada_recusa_diz_a_cura() {
    for r in [
        Recusa::PecaAberta,
        Recusa::LaminaAberta,
        Recusa::ResultadoVazio,
    ] {
        let t = r.porque();
        assert!(t.len() > 40, "{r:?} tem frase curta demais: {t:?}");
        assert!(
            t.contains("--"),
            "{r:?} não diz a CURA (a frase tem duas metades: o facto e o que fazer): {t:?}"
        );
    }
}
