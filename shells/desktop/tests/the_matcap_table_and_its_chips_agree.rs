#![cfg(feature = "sculpt3d")]
//! **A tabela de materiais e a fileira de chips têm de ter o mesmo tamanho.**
//!
//! Os materiais do matcap vivem em três lugares, cada um pelo seu motivo:
//!
//! | onde | o quê | por quê ali |
//! |---|---|---|
//! | `mesh.wgsl` | as cores e os expoentes | nenhum consumidor de CPU os lê |
//! | `ph2d_mesh_render::MATCAPS` | os NOMES | o painel os pinta |
//! | `ids::SCULPT3D_MATCAP` | os ids dos chips | um id é um id |
//!
//! Os dois primeiros já se prendem um ao outro dentro da `ph2d-mesh-render`
//! (`the_shader_has_exactly_one_arm_per_named_material`). O terceiro é o que
//! **nenhuma das duas crates consegue ver**: o painel não importa o renderizador
//! (ele carregaria o `wgpu` inteiro para escrever seis palavras) e o
//! renderizador não conhece `NodeId`. Só o shell depende dos dois — então o gate
//! mora aqui, pelo mesmo motivo que o `every_panel_the_shell_drives_is_in_its_registry`.
//!
//! ⚠️ **A feature.** O renderizador é opcional no shell (`sculpt3d`), e este
//! arquivo só compila com ela — que está na lista `default`, exatamente para o
//! clippy e os gates a alcançarem. Ver o doc do módulo `sculpt3d` do shell.
//!
//! ⚠️ **O modo de falha é silencioso nos DOIS sentidos.** Um material a mais que
//! os chips fica **inalcançável** — ele existe no shader e nenhum gesto o
//! escolhe. Um chip a mais que os materiais é pintado, é clicável, despacha um
//! índice que o `ShadeRaw::pack` prende no último, e o artista vê **a cera
//! vermelha ao pedir outra coisa**. Nenhum dos dois produz erro.

/// O `+ 1` é o RIG, que não é um matcap: a primeira opção da fileira é a luz do
/// DOCUMENTO, e é por isso que a igualdade não é `len == len`.
#[test]
fn there_is_one_chip_for_the_rig_plus_one_per_material() {
    let materials = ph2d_mesh_render::MATCAPS.len();
    let chips = ph2d_panel_sculpt3d::ids::SCULPT3D_MATCAP.len();
    assert_eq!(
        chips,
        materials + 1,
        "{chips} chips para {materials} materiais + o rig — \
         um material ficou inalcançável ou um chip nasceu anônimo"
    );
}

/// **Nenhum nome de material é vazio nem repetido.**
///
/// ⚠️ Vazio pinta um chip sem legenda (clicável, e ninguém sabe o quê);
/// repetido pinta dois chips iguais que fazem coisas diferentes — as duas
/// formas de a fileira mentir sem que a contagem acuse.
#[test]
fn every_material_has_its_own_name() {
    let names = ph2d_mesh_render::MATCAPS;
    for (i, n) in names.iter().enumerate() {
        assert!(!n.trim().is_empty(), "o material {i} não tem nome");
        assert!(
            !names[..i].contains(n),
            "o material {i} repete o nome `{n}`"
        );
    }
}
