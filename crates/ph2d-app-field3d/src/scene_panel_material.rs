//! ⭐⭐⭐ **O MATERIAL DA SELECÇÃO** — quantas formas um pedido de material pinta, e como se diz.
//!
//! # Por que um módulo irmão
//!
//! O [`super`] responde *«o que o painel mostra e o que ele oferece»*, linha a linha. Isto responde
//! **uma** pergunta, e ela atravessa as duas pontas da costura: *sobre QUANTAS formas uma escrita de
//! material cai?* — o retrato precisa dela para a nota, e o dreno precisa dela para escrever.
//!
//! ⚠️ O corte foi forçado pelo tecto de `700` LOC do ficheiro (⛔ *corte, nunca `FILE_OVERAGE_OK`*) e
//! é melhor por isso: as três funções são **um assunto só**, e juntas leem-se sem o resto do painel.

use super::*;

/// ⭐⭐⭐ **AS FOLHAS QUE UM PEDIDO DE MATERIAL ALCANÇA** — a lei do alcance, num sítio só.
///
/// # ⚠️ Porque o MATERIAL espalha e uma DIMENSÃO não
///
/// Um pedido de dimensão escreve no nó que o artista apontou, e mais nada: largura, raio e posição
/// são **daquela forma**, e espalhá-los por uma selecção seria esmagar o desenho.
///
/// ⭐ **Um material é a única coisa que um artista atribui a MUITOS objectos de uma vez** — é o que
/// todo DCC faz com *«assign material to selection»*, e é o gesto que o dono acabou de pedir ao
/// pedir a caixa de cor: pintar uma peça, não uma face.
///
/// ⇒ o alcance é a **selecção inteira**, com cada GRUPO resolvido nas folhas debaixo dele — porque
/// quem o traçado sabe nomear por pixel é a folha (`docs/Render3d/05` §11), e um material escrito
/// num grupo não teria quem o lesse.
///
/// # ⚠️ A ordem é a da travessia, e a lista não tem repetidos
///
/// Escolher um grupo **e** um filho dele é um gesto normal (clicar no grupo, `Shift`+clicar numa
/// forma), e sem a de-duplicação a folha receberia a escrita duas vezes. Isso é inofensivo hoje — a
/// segunda escreve o mesmo valor — e deixa de o ser no dia em que alguém aqui escrever um
/// incremento em vez de um valor.
pub(crate) fn material_targets(
    world: &bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
) -> Vec<bevy_ecs::entity::Entity> {
    let mut out: Vec<bevy_ecs::entity::Entity> = Vec::new();
    for &e in selection {
        // ⚠️ **A travessia parte DAQUI**, e não da raiz: `walk` sobre um nó devolve a sub-árvore
        // dele, que é exactamente *«as folhas deste grupo»*. Uma folha devolve-se a si própria.
        for (n, _) in ph2d_field_ecs::walk(world, e) {
            let folha = matches!(
                world.get::<ph2d_field_ecs::FieldNode>(n).map(|f| &f.shape),
                Some(ph2d_field::NodeShape::Leaf(_))
            );
            if folha && !out.contains(&n) {
                out.push(n);
            }
        }
    }
    out
}

/// ⭐⭐⭐ **O MATERIAL DA SELECÇÃO** — as linhas, e a nota que diz sobre quantas formas elas escrevem.
///
/// # ⚠️ Duas coisas de uma vez, e elas são a mesma
///
/// 1. **Um GRUPO passa a oferecer material.** O [`ph2d_field_ecs::params_of`] não lhe dá linhas
///    nenhumas — e está certo: um grupo **não tem** material, quem o traçado sabe nomear por pixel é
///    a folha. O que ele tem é uma **sub-árvore de folhas**, e é sobre elas que a escrita cai.
/// 2. **A nota diz sobre quantas.** Ver [`ph2d_panel_model3d::ParamRow::subject`]: *espalhar sem
///    sinal troca um sub-aplicar silencioso por um esmagamento silencioso.*
///
/// ⚠️ **As linhas de um grupo são as do primeiro alvo, filtradas ao material** — e não uma segunda
/// tabela escrita aqui. As pontas de um número de material (`Span::SoftFromZero(1)`) são do
/// documento, e re-escrevê-las neste ficheiro seria a segunda resposta à mesma pergunta.
///
/// ⚠️ **O `entity` das linhas é o do PRIMEIRO alvo**, e ele é o que a amostra mostra — quem decide
/// o alcance é o dreno, a partir da selecção, e não este id. É por isso que a nota é obrigatória.
pub(super) fn material_for_the_selection(
    world: &bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
    view_span: f32,
    linhas: &mut Vec<ph2d_panel_model3d::ParamRow>,
) {
    let alvos = material_targets(world, selection);
    let Some(&primeiro) = alvos.first() else {
        return;
    };
    // ⭐ **Um grupo não trouxe linhas de material** — as dele são as do primeiro alvo.
    if !linhas
        .iter()
        .any(|r| matches!(r.param, ph2d_field::Param::Material(_)))
    {
        let mut dele = param_rows(world, &[primeiro], view_span);
        dele.retain(|r| matches!(r.param, ph2d_field::Param::Material(_)));
        // ⚠️ A primeira das linhas herdadas **abre a secção** — sem isto os números do material
        // apareceriam colados às dimensões do grupo, que é o report de 2026-08-30 outra vez.
        if let Some(p) = dele.first_mut() {
            p.section = Some("panel.model3d.section.material");
        }
        linhas.extend(dele);
    }
    // ⛔ **Uma forma só não precisa de nota** — ela é o sujeito óbvio, e uma nota ali seria ruído
    // permanente sobre o gesto mais comum do painel.
    if alvos.len() < 2 {
        return;
    }
    let difere = alvos.iter().any(|&a| {
        world
            .get::<ph2d_field_ecs::FieldMaterial>(a)
            .copied()
            .unwrap_or_default()
            != world
                .get::<ph2d_field_ecs::FieldMaterial>(primeiro)
                .copied()
                .unwrap_or_default()
    });
    let mut nota = format!(
        "{}: {} {}",
        ph2d_i18n::tr("panel.model3d.material_applies_to"),
        alvos.len(),
        ph2d_i18n::tr("panel.model3d.shapes")
    );
    // ⚠️ **E o aviso só aparece quando o valor mostrado é FALSO sobre as outras** — com as formas
    // de acordo, a amostra descreve todas elas e um «diferem» ali seria mentira ao contrário.
    if difere {
        nota.push_str(" — ");
        nota.push_str(ph2d_i18n::tr("panel.model3d.material_mixed"));
    }
    if let Some(p) = linhas
        .iter_mut()
        .find(|r| matches!(r.param, ph2d_field::Param::Material(_)))
    {
        p.subject = Some(nota);
    }
}

/// ⭐⭐⭐ **O ALCANCE de um pedido de material que CHEGOU** — a porta que o dreno usa.
///
/// # ⚠️ Porque ela não é só o [`material_targets`]
///
/// O pedido traz o `entity` da **linha** que o produziu, e a selecção pode ter mudado entre o
/// quadro que a pintou e o quadro que a drena (um clique no canvas, um desfazer). ⇒ se o alvo da
/// linha **não** estiver no alcance da selecção de agora, a escrita cai **só nele**.
///
/// ⛔ *Sem esta cerca, um pedido de um quadro atrás pintaria uma selecção que não era a dele* — e o
/// artista veria formas que nunca escolheu mudar de cor. É a mesma família do id da amostra
/// (`docs/Render3d/05` §12.3): **um pedido carrega o sujeito que o produziu, e quem o executa
/// confere-o contra o presente.**
#[must_use]
pub(crate) fn material_reach(
    world: &bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
    entity: u64,
) -> Vec<bevy_ecs::entity::Entity> {
    let dele = bevy_ecs::entity::Entity::from_bits(entity);
    let alcance = material_targets(world, selection);
    if alcance.contains(&dele) {
        alcance
    } else {
        vec![dele]
    }
}
