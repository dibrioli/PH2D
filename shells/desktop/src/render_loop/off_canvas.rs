//! ⭐⭐ **O que NÃO está na cena** — a pergunta que o extract faz a cada entidade.
//!
//! ⚠️⚠️ **E ela tem DOIS leitores, de propósito** (F4.6): o extract de sprites e a cadeia de
//! visibilidade do vetor ([`crate::vec_entities`]). Enquanto era só o primeiro, a regra tinha duas
//! respostas — a arte vetorial de um mestre continuava a desenhar **por baixo da cópia** que o
//! *Criar componente* deixa no lugar, e o artista não distinguia uma da outra. Foi isso que fez a
//! propagação parecer morta estando viva (o §14 do handoff): a cena 2 do smoke, com a receita
//! LONGE das cópias, propaga.
//!
//! ⚠️ **Irmão de [`super::sim_extract`] por ASSUNTO** (e porque aquele ficheiro já vive sob uma
//! excepção de LOC): lá mora *como* uma sprite vira `RenderInstance`; aqui mora *se* ela vira.
//! A decisão precisa de nome próprio — ela vive dentro de um closure que pede um renderer, e uma
//! mutação que a desligasse compilaria e passaria a suíte inteira.

use ph2d_ecs::{Entity, VisibilityLayer, World};

/// ⭐⭐⭐ **«Esta entidade DESENHA neste quadro?»** — a porta única do extract, e as **três** razões
/// pelas quais ela pode não desenhar.
///
/// ⚠️ **A terceira nasceu em 2026-08-30** (o `OnScreenEnabler`) e as duas primeiras já existiam
/// soltas dentro do closure do [`super::sim_extract`]. Juntá-las aqui não é arrumação: enquanto a
/// decisão morava no fio, **nenhuma mutação a matava de forma observável** — é exactamente a razão
/// que o doc do [`is_off_canvas`] já dava para o primeiro par, e a chegada de um terceiro motivo é
/// o momento em que ela deixa de ser opinião.
///
/// | razão | pergunta | quem a autora |
/// |---|---|---|
/// | [`is_off_canvas`] | o olho da Hierarquia · a peça de uma receita | o artista · o ADR-0164 |
/// | [`layer_visible`] | a máscara de camadas cruza a da câmara? | as 32 caixas da §8 |
/// | [`super::on_screen_gate::hides`] | ela saiu do rect que ela declara? | os 5 campos da §8 |
///
/// `world_pos` é a pose de mundo que o extract já tem em mãos (o `GlobalTransform` do quadro).
#[must_use]
pub(crate) fn draws_this_frame(
    sim: &World,
    entity: Entity,
    cull_mask: u32,
    world_pos: [f32; 2],
) -> bool {
    !is_off_canvas(sim, entity)
        && layer_visible(sim, entity, cull_mask)
        && !super::on_screen_gate::hides(sim, entity, world_pos)
}

/// ⭐ **A metade das CAMADAS** (W3.T3.12) — *«a máscara desta entidade cruza a da câmara?»*.
///
/// Ausência do componente = visível a toda câmara ([`VisibilityLayer::ALL`]), que é a lei do
/// Godot e o que o Inspector mostra numa grade toda marcada.
///
/// ⛔⛔ **HOJE NENHUMA CÂMARA AUTORA UMA MÁSCARA**: o `Camera2d::cull_mask` só existe como o
/// literal `u32::MAX` em todo o repo (medido 2026-08-30), então **31 dos 32 bits são inertes** e a
/// grade da §8 comporta-se como um interruptor «esconder» que exige desmarcar as 32 caixas uma a
/// uma. ⚠️ **O buraco é do lado do AUTOR, não deste consumidor** — a superfície que falta (uma
/// entrada de menu *View* com o filtro de camadas do viewport) mora em `ph2d-editor-core`
/// (`src/ids/menus.rs` + `screens/hero/menu_rows.rs` + `screens/hero/chrome/view_toggles.rs`), fora
/// desta linha. O gate `two_different_cull_masks_give_two_different_scenes` prende este lado para
/// que a wave do autor encontre a metade dela já provada.
#[must_use]
pub(crate) fn layer_visible(sim: &World, entity: Entity, cull_mask: u32) -> bool {
    sim.get::<VisibilityLayer>(entity)
        .is_none_or(|vl| vl.visible_to(cull_mask))
}

/// ⭐⭐ **«Esta entidade está FORA da tela?»** — a pergunta que decide se ela emite instância.
///
/// Duas razões, e as duas são *«não está na cena»*:
///
/// 1. o **olho** da Hierarquia (`Visibility`), que é autoria do artista;
/// 2. ser peça de uma **RECEITA** (ADR-0164). *Um mestre é autoria guardada, não um objeto na
///    cena* — é a frase do `ph2d_ecs::master`, e até 2026-08-26 ela não valia para os pixels.
///
/// ⛔⛔ **A F4.5 escondia só a RAIZ do mestre com `Visibility`, e a premissa era falsa.** O
/// `sim_extract` di-lo pelo nome no doc do `resolve_clip_grouping`: *«Visibility is per-entity, it
/// does not propagate to descendants»*. Com uma receita que fosse um GRUPO, as peças dela continuavam a
/// desenhar — o artista fazia *Criar componente* e via **dois objetos empilhados**, um que cai e
/// outro que não, que é exatamente o defeito que a nota daquela fatia dizia ter evitado.
///
/// ⚠️ **A marca é o `MasterPiece`, que é DERIVADO** por `assign_master_pieces` (a raiz e toda a
/// descendência, re-carimbado por quadro) — e por isso não pode discordar da árvore. ⛔ Escrever
/// `Visibility` nas peças seria o contrário: a `Visibility` de uma peça é **autoria** e propaga
/// para as instâncias, logo toda cópia nasceria invisível.
///
/// ⚠️ **Função com NOME e não uma linha no fio**: a decisão vive dentro de um closure que pede um
/// renderer, e a mutação que a desligasse compilaria e passaria a suíte inteira.
pub(crate) fn is_off_canvas(sim: &World, entity: Entity) -> bool {
    is_unedited_recipe(sim, entity)
        || sim
            .get::<ph2d_ecs::Visibility>(entity)
            .is_some_and(|v| v.hidden)
}

/// ⭐⭐⭐ **A metade que fala de RECEITAS** — *«esta entidade é peça de uma receita que ninguém
/// está a editar agora?»*.
///
/// ⚠️⚠️ **Ela tem nome próprio porque existe um TERCEIRO leitor**, e a auditoria de 2026-08-27
/// (§1.5) apanhou-o com **metade da lei**: o anel do objeto vazio
/// ([`crate::group_gizmo_view::is_empty_object`]) perguntava só *«é `MasterPiece`?»* e não conhecia
/// o `MasterEditing`. Consequência medida: a raiz de uma receita que seja um GRUPO ou um rig — que
/// é a forma de **toda** receita nascida de *Make Component* sobre um grupo — ficava sem anel, sem
/// caixa e **impegável mesmo enquanto era editada**, que é precisamente o estado que o modo de
/// edição existe para tornar alcançável.
///
/// ⛔ **Não replique a conjunção noutro ficheiro.** Uma lei escrita em dois sítios ainda não é uma
/// lei — só uma PORTA é; foi o que a `line/Vector` pagou no bug #27 e o que esta pagou aqui. Quem
/// precisar de *«está na cena?»* chama [`is_off_canvas`]; quem precisar só da metade da receita
/// (porque o olho fechado é uma pergunta que o seu consumidor já faz noutro sítio) chama esta.
///
/// ⚠️ **Não é o mesmo que [`is_off_canvas`], e a diferença é o olho:** um objeto que o artista
/// escondeu continua a ser um objeto da cena com gizmo — esconder não é deixar de existir. Foi por
/// isso que a cura não foi simplesmente apontar o anel ao `is_off_canvas`.
pub(crate) fn is_unedited_recipe(sim: &World, entity: Entity) -> bool {
    // ⭐⭐ **A receita volta enquanto está a ser EDITADA** (ver `super::master_editing`): esconder
    // sempre tornaria a forma do mestre impossível de mudar, e desenhar sempre põe dois objetos
    // empilhados. A marca é derivada da selecção, e por isso as três famílias leem a MESMA
    // resposta sem ninguém lhes passar a selecção.
    sim.get::<ph2d_ecs::MasterPiece>(entity).is_some()
        && sim.get::<ph2d_ecs::MasterEditing>(entity).is_none()
}

#[cfg(test)]
mod off_canvas_tests {
    use super::{draws_this_frame, is_off_canvas};
    use ph2d_ecs::{
        ChildOf, EnableMode, MasterRoot, Name, OnScreenEnabler, SimWorld, Transform, Visibility,
        VisibilityLayer,
    };

    /// ⭐⭐⭐ **O gate que o `cull_mask` nunca teve: DUAS máscaras, DUAS cenas.**
    ///
    /// Uma afirmação de que o campo foi escrito não mediria nada — o defeito era precisamente um
    /// campo escrito que ninguém lia. Este gate carrega na decisão que o extract de facto toma:
    /// com a MESMA entidade e o MESMO mundo, duas máscaras de câmara diferentes dão duas respostas
    /// diferentes a *«esta sprite emite instância?»*.
    ///
    /// ⚠️ **A metade justa vem primeiro** (a máscara que cruza), senão uma implementação que
    /// recusasse sempre passaria.
    ///
    /// **Mutação:** trocar o `layer_visible` de [`super::draws_this_frame`] por `true` ⇒ RED.
    #[test]
    fn two_different_cull_masks_give_two_different_scenes() {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                Name::new("Backdrop"),
                VisibilityLayer(0b01),
            ))
            .id();
        let w = sim.world();
        assert!(
            draws_this_frame(w, e, 0b01, [0.0, 0.0]),
            "a camara que INCLUI a camada da entidade nao a desenhou"
        );
        assert!(
            !draws_this_frame(w, e, 0b10, [0.0, 0.0]),
            "a camara que EXCLUI a camada da entidade continuou a desenha-la: as 32 caixas da §8 \
             nao chegam ao passe"
        );
    }

    /// ⚠️ **A ausência do componente é «visível a toda câmara»** — a lei do Godot, e o que a grade
    /// toda marcada do Inspector promete. Sem esta metade, uma cura que tratasse a ausência como
    /// `0` apagaria toda a cena.
    #[test]
    fn an_entity_without_a_layer_is_visible_to_every_camera() {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Plain")))
            .id();
        assert!(draws_this_frame(sim.world(), e, 0b1000, [0.0, 0.0]));
    }

    /// ⭐ **As TRÊS razões passam pela mesma porta** — o olho, a camada e o rect. Um consumidor que
    /// só perguntasse a primeira é o que existia antes desta wave.
    ///
    /// **Mutação:** apagar o termo `on_screen_gate::hides` de [`super::draws_this_frame`] ⇒ RED.
    #[test]
    fn the_door_answers_for_the_eye_the_layer_and_the_rect() {
        for (what, decorate) in [
            (
                "o olho",
                (|w: &mut ph2d_ecs::World, e| {
                    w.entity_mut(e).insert(Visibility::hidden());
                }) as fn(&mut ph2d_ecs::World, ph2d_ecs::Entity),
            ),
            ("a camada", |w, e| {
                w.entity_mut(e).insert(VisibilityLayer(0));
            }),
            ("o rect", |w, e| {
                w.entity_mut(e).insert(OnScreenEnabler::new(
                    [0.0, 0.0, 1.0, 1.0],
                    EnableMode::HideVisible,
                ));
            }),
        ] {
            let mut sim = SimWorld::new();
            let e = sim
                .world_mut()
                .spawn((Transform::IDENTITY, Name::new("Thing")))
                .id();
            assert!(
                draws_this_frame(sim.world(), e, u32::MAX, [50.0, 50.0]),
                "{what}: a entidade ja' nao desenhava antes da razao ser posta"
            );
            decorate(sim.world_mut(), e);
            assert!(
                !draws_this_frame(sim.world(), e, u32::MAX, [50.0, 50.0]),
                "{what} nao chegou a' porta do extract"
            );
        }
    }

    /// ⭐⭐⭐ **A RECEITA INTEIRA sai da tela — a raiz e as peças.**
    ///
    /// ⛔ Era este o defeito: a F4.5 escondia só a raiz com `Visibility`, que não desce aos
    /// descendentes; uma receita que fosse um GRUPO continuava a desenhar as peças, e o artista
    /// via dois objetos empilhados.
    ///
    /// (Mutação: tirar o ramo do `MasterPiece` ⇒ RED na peça e na raiz.)
    #[test]
    fn a_recipe_draws_nothing_root_or_piece() {
        let mut sim = SimWorld::new();
        let root = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Recipe"), MasterRoot))
            .id();
        let arm = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Arm"), ChildOf(root)))
            .id();
        // O controlo NEGATIVO vem primeiro: antes do passe derivado, nada está marcado.
        assert!(
            !is_off_canvas(sim.world(), arm),
            "a peca ja' estava fora da tela antes de a receita existir — o gate nao mede nada"
        );
        ph2d_ecs::assign_master_pieces(sim.world_mut());
        for (what, e) in [("a raiz", root), ("a peca", arm)] {
            assert!(
                is_off_canvas(sim.world(), e),
                "{what} da receita continua a desenhar"
            );
        }
    }

    /// ⚠️ **E o olho da Hierarquia continua a valer, per-entidade.**
    ///
    /// (Mutação: tirar o ramo da `Visibility` ⇒ RED.)
    #[test]
    fn the_eye_still_hides_the_entity_it_is_on() {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Thing")))
            .id();
        assert!(!is_off_canvas(sim.world(), e));
        sim.world_mut().entity_mut(e).insert(Visibility::hidden());
        assert!(is_off_canvas(sim.world(), e), "o olho fechado nao escondeu");
    }
}
