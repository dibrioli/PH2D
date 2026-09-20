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
//! ⚠️ **E a fileira deixou de ser só «qual matcap»** — desde 2026-09-20 ela é
//! *«com que LUZ»*, com três famílias: o **plano** (sem luz), o **rig** e os
//! matcaps. Ver [`ph2d_mesh_render::Lighting`].
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

/// Os `+ 2` são o PLANO e o RIG, que não são matcaps: as duas primeiras opções
/// da fileira são *sem luz* e *a luz do DOCUMENTO*, e é por isso que a igualdade
/// não é `len == len`.
///
/// ⛔⛔ **Era `+ 1` até 2026-09-20 e a premissa MORREU por ordem do dono**
/// (*«precisamos como no blender modos de shaders além do matcap para
/// pintar»*): o modo PLANO entrou à frente do rig. A morte fica à vista no
/// diff, que é a lei desta casa para uma premissa que cai — e o nome do gate
/// mudou com ela, porque o antigo prometia a contagem antiga.
#[test]
fn there_are_two_fixed_chips_plus_one_per_material() {
    let materials = ph2d_mesh_render::MATCAPS.len();
    let chips = ph2d_panel_sculpt3d::ids::SCULPT3D_MATCAP.len();
    assert_eq!(
        chips,
        materials + 2,
        "{chips} chips para {materials} materiais + o plano + o rig — \
         um material ficou inalcançável ou um chip nasceu anônimo"
    );
}

/// **Nenhum nome de material é vazio nem repetido — medido na PALAVRA, não na chave.**
///
/// ⚠️ Vazio pinta um chip sem legenda (clicável, e ninguém sabe o quê);
/// repetido pinta dois chips iguais que fazem coisas diferentes — as duas
/// formas de a fileira mentir sem que a contagem acuse.
///
/// ⚠️⚠️ **E desde 2026-09-17 a `ph2d_mesh_render::MATCAPS` entrega CHAVES**, logo
/// a versão anterior deste gate teria passado a medir identificadores: duas
/// chaves distintas que a tabela resolvesse para a MESMA palavra pintariam dois
/// chips iguais com ele **verde**. *Um gate que segue uma indirecção nova sem
/// mudar de grandeza deixa de medir o que o nome dele promete.*
///
/// ⭐ A metade da CHAVE fica, e é outra coisa: ela apanha a entrada que falta na
/// tabela, que o `tr` devolveria crua (`sculpt3d.matcap.red_wax` no chip).
#[test]
fn every_material_has_its_own_name() {
    let keys = ph2d_mesh_render::MATCAPS;
    let words: Vec<&str> = keys.iter().map(|k| ph2d_i18n::tr(k)).collect();
    for (i, (k, w)) in keys.iter().zip(&words).enumerate() {
        assert!(!w.trim().is_empty(), "o material {i} não tem nome");
        assert_ne!(
            w, k,
            "a chave `{k}` do material {i} não está declarada em \
             `ph2d-i18n/src/sculpt_engine.rs` — o chip pintaria o identificador cru"
        );
        assert!(
            !words[..i].contains(w),
            "o material {i} repete a palavra `{w}` (chave `{k}`)"
        );
    }
}
