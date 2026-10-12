//! ⭐⭐⭐ **Todo componente que o «Add Component» oferece tem um CARTÃO no Inspector** —
//! componente = ícone + cartão (`docs/ObjectsAndComponents/01_objeto_e_componente.md`).
//!
//! O defeito que o apanhou (smoke do dono, 2026-10-11): «Add Component ▸ Unlit» punha a
//! `NaoRecebeLuz` e o efeito valia, mas o Inspector não pintava cartão nenhum — o artista não podia
//! ver o componente, editar a opacidade do «Light Occluder», nem retirar nenhum dos dois. Os dois
//! censos da paleta (`reach_tests`) perguntam «o componente chega à paleta?»; nenhum perguntava
//! «e depois de anexado, aparece?».
//!
//! ⇒ a população é DERIVADA da paleta do produto (o `offered_to` do censo de alcance, com *Show
//! all*), e o lado dos cartões é a tabela que o Inspector EXPORTA (`probe_section_components`, a do
//! «mostra o cartão desta família») — nenhuma das duas é uma lista escrita aqui.
//!
//! ⚠️ A [`DIVIDA`] é a catraca: cada entrada diz ONDE o componente se vê sem cartão próprio e
//! porquê; ela só desce. Uma entrada que ganhou cartão próprio reprova
//! (`a_divida_nao_guarda_quem_ja_tem_cartao`) — a catraca sem censo de obsolescência vira licença.

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids as i;
use std::collections::BTreeSet;

/// Onde um componente SEM cartão próprio se vê.
#[derive(Clone, Copy, Debug)]
enum Onde {
    /// Dentro do cartão de OUTRO componente (o principal daquela secção).
    NoCartao(NodeId),
    /// Noutro painel (o Inspector não o mostra).
    NoPainel(&'static str),
    /// Em lado nenhum — o defeito deste censo. Nenhum hoje.
    #[allow(dead_code)]
    EmLadoNenhum,
}

/// **Os componentes que a paleta oferece e que NÃO têm cartão próprio** — `(nome, onde, porquê)`.
/// Medido em 2026-10-11 (22 entradas, nenhuma `EmLadoNenhum`); os endereços são do snapshot que lê
/// o componente. ⚠️ Viver dentro de outro cartão não dá o botão de RETIRAR na maioria deles.
const DIVIDA: &[(&str, Onde, &str)] = &[
    (
        "ph2d::ecs::AnchorMount",
        Onde::NoCartao(i::INSP_LIVE_ANCHOR_SECTION),
        "o «Rides Parent Anchor» das âncoras (shell inspector_anchor.rs)",
    ),
    (
        "ph2d::ecs::AudioListener2D",
        Onde::NoCartao(i::INSP_LIVE_AUDIO_SECTION),
        "o ouvinte é uma linha do cartão Audio (shell inspector_audio.rs)",
    ),
    (
        "ph2d::ecs::CameraFollow",
        Onde::NoCartao(i::INSP_LIVE_CAMERA_SECTION),
        "o seguir é parte do cartão Camera (camera_inspector.rs)",
    ),
    (
        "ph2d::ecs::CameraLimits",
        Onde::NoCartao(i::INSP_LIVE_CAMERA_SECTION),
        "os limites são parte do cartão Camera (camera_inspector.rs)",
    ),
    (
        "ph2d::ecs::ClipChildren",
        Onde::NoCartao(i::INSP_LIVE_VISIBILITY_SECTION),
        "uma linha da Visibility (inspector_visibility.rs)",
    ),
    (
        "ph2d::ecs::Counter",
        Onde::NoCartao(i::INSP_LIVE_HUD_SECTION),
        "o contador é uma linha do cartão HUD (hud_inspector.rs)",
    ),
    (
        "ph2d::ecs::DestroyOutside",
        Onde::NoCartao(i::INSP_LIVE_LIFECYCLE_SECTION),
        "uma linha do Lifecycle (shell inspector_factory.rs)",
    ),
    (
        "ph2d::ecs::Mask2D",
        Onde::NoCartao(i::INSP_LIVE_VISIBILITY_SECTION),
        "uma linha da Visibility (inspector_visibility.rs)",
    ),
    (
        "ph2d::ecs::MaskInteraction",
        Onde::NoCartao(i::INSP_LIVE_VISIBILITY_SECTION),
        "uma linha da Visibility (inspector_visibility.rs)",
    ),
    (
        "ph2d::ecs::OnScreenEnabler",
        Onde::NoCartao(i::INSP_LIVE_VISIBILITY_SECTION),
        "uma linha da Visibility (inspector_visibility.rs)",
    ),
    (
        "ph2d::ecs::ScrollLimits",
        Onde::NoCartao(i::INSP_LIVE_PARALLAX_SECTION),
        "só com ScrollFactor — sem ele não se vê (parallax_inspector.rs)",
    ),
    (
        "ph2d::ecs::ScrollMotion",
        Onde::NoCartao(i::INSP_LIVE_PARALLAX_SECTION),
        "só com ScrollFactor — sem ele não se vê (parallax_inspector.rs)",
    ),
    (
        "ph2d::ecs::ScrollRepeat",
        Onde::NoCartao(i::INSP_LIVE_PARALLAX_SECTION),
        "só com ScrollFactor — sem ele não se vê (parallax_inspector.rs)",
    ),
    (
        "ph2d::ecs::SpriteAnimations",
        Onde::NoCartao(i::INSP_LIVE_ANIM_SECTION),
        "a lista de animações do cartão Animation (shell inspector_anim.rs)",
    ),
    (
        "ph2d::ecs::TextureRepeat",
        Onde::NoCartao(i::INSP_LIVE_SAMPLING_SECTION),
        "uma linha do Sampling (ph2d-inspector-ordering)",
    ),
    (
        "ph2d::ecs::UiButton",
        Onde::NoCartao(i::INSP_LIVE_HUD_SECTION),
        "uma linha do cartão HUD (hud_inspector.rs)",
    ),
    (
        "ph2d::ecs::UiLabel",
        Onde::NoCartao(i::INSP_LIVE_HUD_SECTION),
        "uma linha do cartão HUD (hud_inspector.rs)",
    ),
    (
        "ph2d::ecs::UvTransform",
        Onde::NoCartao(i::INSP_LIVE_SAMPLING_SECTION),
        "a escala/deslocamento UV do Sampling (ph2d-inspector-ordering)",
    ),
    (
        "ph2d::ecs::VisibilityLayer",
        Onde::NoCartao(i::INSP_LIVE_VISIBILITY_SECTION),
        "a grelha de camadas da Visibility (inspector_visibility.rs)",
    ),
    (
        "ph2d::skeleton::Bone",
        Onde::NoPainel("skeleton"),
        "o osso edita-se no painel do esqueleto",
    ),
    (
        "ph2d::skeleton::BoneLimit",
        Onde::NoPainel("skeleton"),
        "o limite de ângulo é do painel do esqueleto",
    ),
    (
        "ph2d::skeleton::SmartBone",
        Onde::NoPainel("skeleton"),
        "o osso esperto é do painel do esqueleto",
    ),
];

fn oferecidos() -> BTreeSet<&'static str> {
    ObjectKind::ALL
        .iter()
        .flat_map(|&k| super::reach_tests::offered_to(k))
        .collect()
}

fn com_cartao() -> BTreeSet<&'static str> {
    ph2d_panel_inspector::probe_section_components()
        .iter()
        .map(|(_, n)| *n)
        .collect()
}

/// ⭐⭐⭐ **Todo componente oferecido tem cartão, ou está na dívida com o porquê.**
#[test]
fn todo_componente_que_a_paleta_oferece_tem_cartao() {
    let cartoes = com_cartao();
    let divida: BTreeSet<&str> = DIVIDA
        .iter()
        .filter(|(_, onde, _)| !matches!(onde, Onde::EmLadoNenhum))
        .map(|(n, _, _)| *n)
        .collect();
    let oferecidos = oferecidos();
    assert!(
        oferecidos.len() >= 10,
        "a paleta ofereceu {} componentes — o padrão deixou de casar e o censo mede nada",
        oferecidos.len()
    );
    let mudos: Vec<&str> = oferecidos
        .iter()
        .copied()
        .filter(|n| !cartoes.contains(n) && !divida.contains(n))
        .collect();
    assert!(
        mudos.is_empty(),
        "o «Add Component» oferece estes e o Inspector não lhes pinta cartão (o artista anexa e \
         não vê, não edita, não retira): {mudos:#?}"
    );
}

/// ⛔ **A dívida não guarda quem já tem cartão, nem quem a paleta deixou de oferecer** — e não tem
/// entradas repetidas nem porquê vazio.
#[test]
fn a_divida_nao_guarda_quem_ja_tem_cartao() {
    let cartoes = com_cartao();
    let oferecidos = oferecidos();
    let mut vistos = BTreeSet::new();
    let seccoes: BTreeSet<NodeId> = ph2d_panel_inspector::probe_section_components()
        .iter()
        .map(|(id, _)| *id)
        .collect();
    for (n, onde, porque) in DIVIDA {
        assert!(vistos.insert(*n), "`{n}` repetido na dívida");
        assert!(!porque.trim().is_empty(), "`{n}` na dívida sem o porquê");
        assert!(
            !matches!(onde, Onde::EmLadoNenhum),
            "`{n}` não se vê em lado nenhum — o artista anexa e não vê: dê-lhe um cartão"
        );
        match onde {
            Onde::NoCartao(s) => assert!(
                seccoes.contains(s),
                "`{n}` aponta para um cartão que não existe"
            ),
            Onde::NoPainel(p) => assert!(
                !p.is_empty() && *p != "inspector",
                "`{n}`: «noutro painel» nomeia esse painel, e não é o Inspector"
            ),
            Onde::EmLadoNenhum => {}
        }
        assert!(
            !cartoes.contains(n),
            "`{n}` já tem cartão — saia da dívida (ela só desce)"
        );
        assert!(
            oferecidos.contains(n),
            "`{n}` já não é oferecido pela paleta — saia da dívida"
        );
    }
}
