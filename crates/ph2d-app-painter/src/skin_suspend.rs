//! ⭐⭐⭐ **PINTAR ACHATA A ARTE: a pele fica SUSPENSA enquanto o Painter edita a textura.**
//!
//! Ordem do dono, 2026-09-15: *«Inative a possibilidade de pintar sobre malha deformada por ossos.
//! SE o usuário entrar no modo Painter em imagem deformada por ossos a imagem deixar a deformação
//! para ser pintada. Ao sair do modo painter, ela retorna a deformação.»*
//!
//! # Por que isto é UMA porta e não uma cerca em cada sítio
//!
//! ⭐⭐ **Toda a app deriva de *«esta sprite tem malha posada?»***, e é isso que faz esta ordem
//! custar um parâmetro em vez de uma wave:
//!
//! | quem pergunta | o que devolve sem malha |
//! |---|---|
//! | [`crate::canvas_map::CanvasMap`] (grelha, guias, gizmos, contornos) | degenera no afim do quad — o chrome volta a ser recto |
//! | `ph2d_render::mesh_uv` (o dedo: conta-gotas, dab, alças) | `MeshUv::Quad` — o ponteiro volta ao quad |
//! | o passe de sprites | desenha o quad |
//!
//! ⇒ **suspender a malha suspende as três metades de uma vez**, e nenhuma delas precisa de saber o
//! que é um Painter. *Uma cerca em cada consumidor seria a mesma lei escrita em três sítios, a
//! divergir no primeiro que alguém refactorasse.*
//!
//! # ⚠️ O regresso é por CONSTRUÇÃO, não por memória
//!
//! A malha é re-posta **a cada quadro** pelo `attach_skin_meshes`. Sair do Painter é deixar de a
//! suspender, e a deformação volta sozinha no quadro seguinte. ⛔ **Não há estado a repor** — e é
//! isso que impede o modo de ficar «preso achatado» depois de um crash, de um undo ou de trocar de
//! selecção a meio.
//!
//! ⚠️ **A tinta sobrevive intacta:** a malha vive nos bytes opacos da [`ph2d_skeleton_ecs::SkinBind`]
//! (traçada no bind) e pintar escreve na TEXTURA. As duas não se tocam, logo o que for pintado
//! achatado aparece deformado quando a pele volta — que é o que o artista espera de pintar uma
//! textura.

use ph2d_editor_core::ToolRegistry;

/// ⭐⭐⭐ **A sprite cuja pele fica suspensa neste quadro** — `None` quando ninguém está a pintar.
///
/// `seleccionada` são os bits da entidade que o gizmo tem na mão: é ela que o Painter edita.
///
/// ⚠️ **Ela não pergunta se a sprite TEM pele**, de propósito: quem sabe isso é o
/// `ph2d_skeleton_live::skin_image::is_skinned_image`, do lado de lá, e perguntá-lo aqui seria uma
/// segunda resposta à mesma pergunta — sobre uma sprite sem pele esta devolve um id que não tem
/// malha nenhuma para suspender, e o efeito é exactamente nenhum.
#[must_use]
pub fn sprite_achatada(tools: &ToolRegistry, seleccionada: Option<u64>) -> Option<u64> {
    let pintando = tools
        .active()
        .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("painter"));
    pintando.then_some(seleccionada).flatten()
}

/// **Quem passou a estar achatada desde o último quadro** — para quem quiser dizê-lo ao artista.
///
/// ⚠️⚠️ **Sem isto a arte SALTA e o artista lê um defeito.** Pegar no pincel faz um canvas dobrado
/// endireitar-se de um quadro para o outro; sem uma palavra, o report seguinte é *«a arte saltou»* —
/// e a cura seria explicar o que o app já sabia.
///
/// ⛔ **É um latch por ARESTA, e ele não é um campo da `App`:** o idioma é o que o
/// `color_equalization_bridge` já usa (um `static` + `swap`), e a razão de não ser estado da `App` é
/// a catraca `the_app_only_sheds_fields` — mas também é a mais honesta, porque isto não é estado do
/// documento nem da sessão: é a memória de UM quadro.
///
/// `0` é «nenhuma» (uma `Entity` nunca vale `0` em bits — o índice `0` carrega geração `1`).
#[must_use]
pub fn acabou_de_achatar(achatada: Option<u64>) -> Option<u64> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static ULTIMA: AtomicU64 = AtomicU64::new(0);
    let agora = achatada.unwrap_or(0);
    let antes = ULTIMA.swap(agora, Ordering::Relaxed);
    (agora != 0 && agora != antes).then_some(agora)
}

/// ⭐⭐⭐ **A PORTA QUE A SHELL CHAMA: decide E avisa, numa linha.**
///
/// ⛔ **Ela existe pela catraca `the_shell_only_shrinks`.** A 1.ª redacção pôs a decisão, o latch e
/// o texto do aviso dentro da fase do quadro, e a shell cresceu `25` linhas — *a cura é MOVER para
/// a crate da família, nunca subir o número*. E o sítio certo era este desde o princípio: o que ali
/// ficava era vocabulário do Painter escrito na shell.
///
/// `e_pele` responde *«esta entidade é uma imagem presa a ossos?»* — quem sabe isso é a
/// `ph2d-skeleton-live`, e esta crate não depende dela. ⚠️ Ele só decide o **aviso**, nunca a
/// suspensão: suspender uma sprite sem pele não tem efeito nenhum, mas avisar sobre ela seria
/// mentir ao artista.
pub fn achata_e_avisa(
    tools: &ToolRegistry,
    seleccionada: Option<u64>,
    e_pele: impl Fn(ph2d_ecs::Entity) -> bool,
    toasts: &mut ph2d_editor_core::toast::ToastQueue,
) -> Option<u64> {
    let achatada = sprite_achatada(tools, seleccionada);
    if let Some(bits) = acabou_de_achatar(achatada)
        && ph2d_ecs::Entity::try_from_bits(bits).is_some_and(&e_pele)
    {
        toasts.push(ph2d_editor_core::toast::Toast::info(
            "Painting flattens this image — the bone deformation returns when you leave the Painter."
                .to_string(),
        ));
    }
    achatada
}

#[cfg(test)]
mod tests {
    use ph2d_editor_core::ToolRegistry;

    /// ⭐⭐ **SEM O PAINTER NA MÃO, NADA É SUSPENSO** — o controlo desta porta.
    ///
    /// ⛔ Sem esta metade, um `sprite_achatada` que devolvesse sempre a selecção achataria toda arte
    /// presa a ossos o tempo todo, e a 2.ª mídia deixava de existir. *A cura de um modo tem de estar
    /// presa ao modo.*
    #[test]
    fn sem_o_painter_na_mao_nada_e_suspenso() {
        let tools = ToolRegistry::default();
        assert_eq!(
            super::sprite_achatada(&tools, Some(42)),
            None,
            "sem ferramenta activa nenhuma pele pode ser suspensa"
        );
    }

    /// ⭐⭐⭐ **O LATCH DISPARA UMA VEZ POR ENTRADA, e volta a armar ao sair.**
    ///
    /// ⚠️ **As três metades**: ele fala na entrada, **cala-se** enquanto se pinta (senão o aviso
    /// aparece a 60 Hz) e **volta a armar** quando se sai — sem a terceira, entrar uma segunda vez
    /// no mesmo canvas seria mudo, que é exactamente quando o artista já esqueceu a primeira.
    #[test]
    fn o_latch_fala_uma_vez_por_entrada_e_volta_a_armar() {
        assert_eq!(super::acabou_de_achatar(Some(7)), Some(7), "a entrada fala");
        assert_eq!(
            super::acabou_de_achatar(Some(7)),
            None,
            "o quadro seguinte com a MESMA sprite tem de ser mudo"
        );
        assert_eq!(super::acabou_de_achatar(None), None, "sair e' mudo");
        assert_eq!(
            super::acabou_de_achatar(Some(7)),
            Some(7),
            "entrar OUTRA VEZ no mesmo canvas volta a falar"
        );
        // ⚠️ E trocar de sujeito SEM sair também fala: é outra arte a achatar.
        assert_eq!(super::acabou_de_achatar(Some(9)), Some(9));
        let _ = super::acabou_de_achatar(None);
    }
}
