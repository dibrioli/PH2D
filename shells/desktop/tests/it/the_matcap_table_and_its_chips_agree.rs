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

/// Os fixos são as opções da fileira que **não são matcaps** — hoje o PLANO, o RIG e a LEI QUE
/// ASSA —, e é por isso que a igualdade não é `len == len`.
///
/// ⛔⛔ **O número esteve escrito à mão DUAS vezes e morreu DUAS vezes:** era `+ 1` até 2026-09-20
/// (entrou o PLANO, por ordem do dono — *«precisamos como no blender modos de shaders além do
/// matcap para pintar»*) e `+ 2` até 2026-09-21 (entrou o **PBR**, pelo report seguinte — *«o que se
/// vê no objecto 3d não é o que se vê na sprite cozida»*). Da segunda vez ele reprovou com
/// `left: 13, right: 12` numa crate que a linha **não editou**, e só a varredura IMPACTADA o viu.
///
/// ⭐⭐⭐ **Por isso ele passou a ser DERIVADO** ([`ph2d_panel_sculpt3d::state::LightMode::FIXOS`]),
/// que é a mesma porta que o pintor da fileira lê. *Um número que morre de cada vez que a lista
/// cresce não é uma constante: é a segunda contagem da mesma coisa* — e o nome do gate deixa de
/// prometer um valor que ele não escolhe.
#[test]
fn ha_um_chip_por_material_mais_os_modos_fixos() {
    let materials = ph2d_mesh_render::MATCAPS.len();
    let fixos = ph2d_panel_sculpt3d::state::LightMode::FIXOS;
    let chips = ph2d_panel_sculpt3d::ids::SCULPT3D_MATCAP.len();
    assert_eq!(
        chips,
        materials + fixos,
        "{chips} chips para {materials} materiais + {fixos} modos fixos — \
         um material ficou inalcançável ou um chip nasceu anônimo"
    );
    // ⭐ **O CONTROLO:** os fixos têm de ser MENOS que a fileira inteira, senão a igualdade acima
    // ficaria verde sobre uma fileira sem material nenhum.
    assert!(
        fixos < chips,
        "controlo: a fileira tem de oferecer materiais ({fixos} fixos de {chips} chips)"
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
