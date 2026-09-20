//! ⭐⭐⭐ **O GATE DO FIO DO CAMPO** — o bind escreve, num irmão.
//!
//! ⚠️ **Este irmão nasceu de um TECTO DE LOC vermelho** (2026-09-20, `skin_live_tests.rs` a `701`
//! contra `700`), e o corte é por RESPONSABILIDADE: os gates do pai medem a LEI (o que o quadro
//! desenha); este mede o **FIO** (o bind guardou o campo?), que é a pergunta que nenhum deles faz.

use super::bind;
use super::tests::palco_com_vertices_na_junta;
use ph2d_ecs::Entity;
use ph2d_skeleton_ecs::SkinBind;

/// ⭐⭐⭐ **O BIND ESCREVE O CAMPO DO DOMÍNIO** — o fio, e não a lei.
///
/// # ⛔⛔ Porque este gate tem de existir à parte dos outros
///
/// Os gates da `ph2d-vec-skin` provam que o campo **resolve** e que a lei da curva **o consulta**;
/// o [`a_tabela_de_pesos_do_caminho_chega_ao_desenho`] constrói a expectativa com o campo que ele
/// próprio leu do bind. ⇒ **nenhum deles reprova se o bind deixar de o guardar**: ali os dois lados
/// ficam sem campo e concordam. *É o terceiro passo que um `grep` não vê — o painel escreve, alguém
/// lê, e o LEITOR decide ou entrega a quem descarta.*
///
/// ⚠️ **As DUAS metades:** o campo existe **e** ele fecha com a tabela (mesma contagem de ossos).
/// Um campo gravado com outro número de colunas seria pior que nenhum — ele entra na lei da curva
/// e entrega pesos plausíveis sobre os ossos errados.
#[test]
fn o_bind_guarda_o_campo_do_dominio() {
    let (mut sim, scene, map, id, _ossos) = palco_com_vertices_na_junta();
    assert_eq!(bind(&mut sim, &scene, &map, &[id], None), 1);
    let e = Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("a pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("a fonte lê-se");

    let campo = g
        .campo
        .as_ref()
        .expect("o bind TEM de guardar o campo do domínio — sem ele a lei da curva volta à recta");
    assert!(
        campo.valida(),
        "o campo guardado não fecha: {} vértices, {} pesos",
        campo.malha.rest.len(),
        campo.pesos.len()
    );
    assert_eq!(
        campo.ossos(),
        g.ossos(),
        "o campo e a tabela têm de cobrir os MESMOS ossos — a coluna `j` é o tendão `j` nas duas"
    );
    assert!(
        campo.malha.rest.len() > 100,
        "a malha do domínio tem só {} vértices — ela é o que dá resolução ao campo",
        campo.malha.rest.len()
    );
}
