//! Os ids das duas tabelas nascem de um macro (`hash_node_id($id)`), que o censo de literais de
//! `ph2d-editor-core` não lê: este teste é quem o cobre (`FORMAS_NAO_LITERAIS`).

use std::collections::BTreeSet;

use ph2d_sfx_lab::tabela::{CAMPOS, OPCOES};

#[test]
fn nenhum_id_se_repete_nas_tabelas_de_campos_e_de_opcoes() {
    assert!(CAMPOS.len() >= 20 && OPCOES.len() >= 8, "tabela vazia");
    let mut vistos = BTreeSet::new();
    let ids = CAMPOS
        .iter()
        .map(|c| (c.id, c.chave))
        .chain(OPCOES.iter().map(|o| (o.id, o.chave)));
    for (id, chave) in ids {
        assert!(vistos.insert(id.0), "id repetido na tabela: {chave}");
    }
}
