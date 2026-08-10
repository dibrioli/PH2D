//! **A cerca de Chesterton do afim de LAYOUT** — centro, escala e rotação moram em dois nós, e
//! isso é uma decisão, não um buraco.
//!
//! `motion.transform` dá o CENTRO (`offset_x`/`offset_y`) e a ESCALA (`scale`); a ROTAÇÃO é
//! `motion.orbit` com `speed = 0`, que gira `P` em torno de um pivô — o que girar um layout
//! significa — e que já paga o `cos/sin` que os outros dois teriam de pagar uma segunda vez.
//! `motion.rotate` gira a base do PRÓPRIO sprite e não tem pivô de propósito: ele só acumula um
//! escalar, que é o que o mantém transcendental-free (HR-5).
//!
//! ⚠️ **Este arquivo existe porque a ausência da frase já custou uma proposta de trabalho.** A
//! folha do doc 89 para a família de DISTRIBUIÇÃO listou *"Coordinates: centro + rotação"* como
//! P1 em **sete** nós, chamando o uso do orbit de *"abuso semântico"*; a folha da família de
//! TRANSFORM recusou a mesma feature no `motion.transform` pelo motivo oposto (*"`motion.orbit
//! (speed = 0)` **é** a rotação de layout"*). Duas folhas, vereditos opostos, e a fatoração
//! escrita em doc-comment nenhum — o que é precisamente o que um comentário não consegue impedir
//! e um gate consegue.
//!
//! ⚠️ **O que estes gates NÃO afirmam:** que a fatoração é grátis. Usar o orbit parado custa o
//! memo do cook (`Effect::Temporal` ⇒ o fingerprint keya no playhead), medido em **2294×** sobre
//! 102.400 elementos pela sonda `ph2d-gpu-cook::measure_static_orbit`. Curar isso exige que o
//! cook possa perguntar *"este nó é temporal NESTE instante?"*, e `NodeManifest`/`OpResolver` são
//! contrato congelado (§6) ⇒ ADR, nunca uma edição.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::effect::Effect;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todos os nós");
    reg
}

/// ⚠️ CONTROLE POSITIVO: um nome que não resolve devolveria uma lista VAZIA, e toda asserção de
/// ausência abaixo ficaria verde por vácuo — o `expect` é o que torna a varredura falha ALTA em
/// vez de silenciosa no dia em que um nó for renomeado.
fn manifest_of(reg: &NodeRegistry, name: &str) -> &'static ph2d_nodegraph::node::NodeManifest {
    reg.manifests().find(|m| m.name == name).unwrap_or_else(|| {
        panic!(
            "`{name}` tem de estar registrado — se foi renomeado, esta \
             cerca inteira estava medindo o vazio"
        )
    })
}

fn params_of(reg: &NodeRegistry, name: &str) -> Vec<&'static str> {
    manifest_of(reg, name)
        .params
        .iter()
        .map(|p| p.name)
        .collect()
}

fn effect_of(reg: &NodeRegistry, name: &str) -> Effect {
    manifest_of(reg, name).effect
}

/// **O centro e a escala do layout são do `motion.transform`; a rotação NÃO.**
///
/// Acrescentar um `rotation`/`angle` aqui não é completar um nó — é uma segunda resposta a *como
/// se gira um layout*, num nó que teria de começar a pagar trig para dar a mesma resposta que o
/// vizinho já dá.
#[test]
fn the_layout_transform_owns_the_centre_and_the_scale_but_not_the_rotation() {
    let reg = registry();
    let p = params_of(&reg, "motion.transform");
    for want in ["offset_x", "offset_y", "scale"] {
        assert!(p.contains(&want), "o centro e a escala são daqui: {p:?}");
    }
    for forbidden in ["rotation", "angle", "spin"] {
        assert!(
            !p.contains(&forbidden),
            "`{forbidden}` em `motion.transform` — a rotação de layout é `motion.orbit(speed=0)`, \
             que já paga o cos/sin; ver o doc-comment dos dois nós antes de mexer aqui. Params: {p:?}"
        );
    }
}

/// **A rotação de layout existe, e é o orbit em torno de um pivô.** A metade positiva: sem ela o
/// gate acima seria só uma proibição, e uma proibição sem o endereço da alternativa é como a
/// próxima varredura conclui que a capacidade falta.
#[test]
fn the_layout_rotation_is_the_orbit_about_a_pivot() {
    let reg = registry();
    let p = params_of(&reg, "motion.orbit");
    for want in ["pivot_x", "pivot_y", "angle", "speed"] {
        assert!(p.contains(&want), "o orbit gira em torno de um pivô: {p:?}");
    }
}

/// **`motion.rotate` gira o SPRITE, não o layout — e por isso não tem pivô.**
///
/// O nó só acumula um escalar em `rot`; a trig é do lowering. Um pivô a traria para dentro do nó
/// e quebraria o HR-5 aqui, para duplicar o que o orbit já faz.
#[test]
fn the_sprite_rotate_has_no_pivot_because_that_is_what_keeps_it_trig_free() {
    let reg = registry();
    let p = params_of(&reg, "motion.rotate");
    assert!(p.contains(&"angle"), "ele gira: {p:?}");
    for forbidden in ["pivot_x", "pivot_y", "center_x", "center_y"] {
        assert!(
            !p.contains(&forbidden),
            "`{forbidden}` em `motion.rotate` — girar em torno de um ponto é `motion.orbit`; este \
             nó acumula um escalar para ficar transcendental-free (HR-5). Params: {p:?}"
        );
    }
}

/// **E o preço da fatoração está NOMEADO, não escondido:** o nó que dá a rotação é `Temporal` e o
/// que dá o centro é `Pure`, então uma rotação estática re-cozinha todo frame.
///
/// ⚠️ Este gate não pede que isso mude — ele impede que a assimetria seja descoberta por acidente
/// uma terceira vez. O número vive na sonda `ph2d-gpu-cook::measure_static_orbit`.
#[test]
fn the_two_halves_of_the_layout_affine_are_memoized_differently_and_this_is_known() {
    let reg = registry();
    assert_eq!(
        effect_of(&reg, "motion.transform"),
        Effect::Pure,
        "o centro/escala é memoizável"
    );
    assert_eq!(
        effect_of(&reg, "motion.orbit"),
        Effect::Temporal,
        "a rotação lê o playhead — e com `speed = 0` paga o memo por nada (2294× a 102.400 \
         elementos). Mudar isto exige que o cook pergunte o efeito por INSTÂNCIA, e \
         `NodeManifest`/`OpResolver` são contrato congelado (§6)"
    );
}

/// **As sete distribuições NÃO carregam centro nem rotação, e é a mesma decisão.**
///
/// Elas mintam um layout; o afim que o posiciona é dos dois nós acima. Dar a cada uma o seu par
/// de coordenadas seria a mesma capacidade em sete lugares — que foi exatamente a proposta que
/// esta cerca existe para responder.
#[test]
fn the_distributions_do_not_carry_their_own_coordinates() {
    let reg = registry();
    for node in [
        "motion.grid",
        "motion.fibonacci",
        "motion.scatter",
        "motion.lattice",
        "motion.voronoi",
        "motion.distribute_poisson",
        "motion.distribute_radial",
    ] {
        let p = params_of(&reg, node);
        for forbidden in ["center_x", "center_y", "rotation"] {
            assert!(
                !p.contains(&forbidden),
                "`{forbidden}` em `{node}` — o afim de layout é \
                 `motion.transform` (centro/escala) + `motion.orbit(speed=0)` (rotação), e \
                 repeti-lo por distribuição é a mesma capacidade em sete lugares. Params: {p:?}"
            );
        }
    }
}
