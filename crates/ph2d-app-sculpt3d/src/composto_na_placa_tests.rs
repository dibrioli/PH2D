//! Gates SEM placa da composição na placa (`docs/3D/30` §13, W1b): o que a
//! placa recebe de cada camada.

use super::*;
use ph2d_mesh_colors::Tinta;

fn plano(k: u8) -> Tinta {
    let mesh = crate::scenes::tinta_fina::peca();
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    match mesh.colors() {
        Some(c) => Tinta::semeada(c, faces(), k),
        None => Tinta::nova(mesh.vert_count(), faces(), k),
    }
}

/// A versão e as linhas por subir da camada `id`.
fn marca(p: &PilhaDaPeca, id: LayerId) -> (u64, Option<(u32, u32)>) {
    let n = p.plano(id).expect("plano").na_placa;
    (n.versao, n.linhas)
}

/// ⭐⭐⭐⭐ **GATE — Todo escritor dos píxeis de uma camada muda a VERSÃO, e as
/// linhas por subir cobrem o que ele escreveu** — senão a placa compõe a camada
/// de antes, sem erro nenhum. CONTROLO: mudar só o metadado não muda a versão
/// (é o que deixa o arrasto do painel sem subir píxel nenhum).
#[test]
fn cada_escritor_da_camada_muda_a_versao() {
    let t = plano(3);
    let mut p = PilhaDaPeca::de_tinta(&t);
    let n = p.amostras();
    let (_, h) = dobra(n);
    let cima = p.nova_camada("cima").expect("camada");
    fn p_ids(p: &PilhaDaPeca) -> Vec<LayerId> {
        p.pilha().all_ids().collect()
    }
    let ids = p_ids(&p);
    p.subiu_a_placa(ids);
    let mut antes = marca(&p, cima);
    assert_eq!(antes.1, None, "subiu: nada por subir");

    // O CONTROLO: o metadado não mexe em píxel.
    let mut nova = p.pilha().clone();
    nova.set_opacity(cima, 0.3);
    p.troca_metadado(nova).expect("metadado");
    assert_eq!(marca(&p, cima), antes, "o metadado não é píxel");

    let mut confere = |p: &mut PilhaDaPeca, linhas: (u32, u32), quem: &str| {
        let agora = marca(p, cima);
        assert_ne!(agora.0, antes.0, "{quem}: a versão não mudou");
        assert_eq!(agora.1, Some(linhas), "{quem}: as linhas por subir");
        let ids = p_ids(p);
        p.subiu_a_placa(ids);
        antes = marca(p, cima);
    };

    // O desfazer de um traço: uma amostra na linha 4 da dobra.
    p.troca_janela(cima, &[4 * 1024 + 7], &[[1, 2, 3, 4]])
        .expect("janela");
    confere(&mut p, (4, 4), "troca_janela");

    // O traço: amostras nas linhas 1 e 2.
    p.define_activa(cima);
    let (id, w) = p.trabalho_da_activa(&t).expect("a activa pinta");
    assert_eq!(id, cima);
    p.recebe_do_traco(cima, &w, &[2 * 1024 - 1, 2 * 1024 + 5]);
    p.fim_do_traco();
    confere(&mut p, (1, 2), "recebe_do_traco");

    // O desfazer do balde: a camada inteira.
    let copia = p.copia_do_plano(cima).expect("plano");
    p.troca_plano(cima, copia).expect("plano");
    confere(&mut p, (0, h - 1), "troca_plano");

    // O documento: a camada inteira.
    let px = vec![[9, 9, 9, 255]; n];
    p.plano_mut(cima).expect("plano").escreve(&px, None);
    confere(&mut p, (0, h - 1), "escreve");

    // Uma máscara nova nasce por subir, inteira.
    let m = p.nova_mascara(cima).expect("máscara");
    assert_eq!(
        marca(&p, m).1,
        Some((0, h - 1)),
        "a máscara nova sobe inteira"
    );
}

/// ⭐⭐ **GATE — A cor por vértice sem compor a peça é o pedaço da peça
/// composta, ao bit** (`PilhaDaPeca::por_vertice` contra `pinta_tinta`), na
/// pilha TRANSLÚCIDA (o fundo lê-se).
#[test]
fn a_cor_por_vertice_e_o_prefixo_da_peca_composta() {
    let t = plano(3);
    let mut p = PilhaDaPeca::de_tinta(&t);
    let base = p.base().expect("base");
    p.define_opacidade(base, 0.6);
    let cima = p.nova_camada("cima").expect("camada");
    let n = p.amostras();
    let px: Vec<[u8; 4]> = (0..n)
        .map(|i| [(i * 7) as u8, (i * 13) as u8, 90, (i * 29) as u8])
        .collect();
    p.plano_mut(cima).expect("plano").escreve(&px, None);
    let mesh = crate::scenes::tinta_fina::peca();
    let mut inteira = t.clone();
    p.pinta_tinta(&mut inteira, || p.fundo_semeado(&mesh, 3));
    let v = mesh.vert_count();
    let bits =
        |a: &[[f32; 3]]| -> Vec<[u32; 3]> { a.iter().map(|c| c.map(f32::to_bits)).collect() };
    assert_eq!(bits(&p.por_vertice()), bits(&inteira.amostras()[..v]));
    assert!(
        inteira.amostras()[..v] != t.amostras()[..v],
        "CONTROLO: a camada mudou os vértices"
    );
}
