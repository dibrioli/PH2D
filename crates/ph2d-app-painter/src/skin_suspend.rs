//! ⭐⭐⭐ **EDITAR PIXELS ACHATA A ARTE: a pele fica SUSPENSA enquanto uma ferramenta de pixels edita
//! a textura.**
//!
//! Ordem do dono, 2026-09-15 (o Painter): *«Inative a possibilidade de pintar sobre malha deformada
//! por ossos. SE o usuário entrar no modo Painter em imagem deformada por ossos a imagem deixar a
//! deformação para ser pintada. Ao sair do modo painter, ela retorna a deformação.»*
//!
//! ⭐⭐⭐ **E a de 2026-09-16 fez dela uma REGRA** (fila do esqueleto, F6-s): *«nenhuma ferramenta de
//! edição de imagem deve trabalhar com arte dobrada … Isso serve para todas as tools de pintura e
//! remoção e deformação de pixels com exceção do Liquify que deverá ser capaz de fazer ajustes na
//! imagem dobrada. Exceção também para filtros … e shaders …»* ⇒ a decisão é uma TABELA
//! ([`FERRAMENTAS_QUE_ACHATAM`], [`MODOS_SOBRE_A_DOBRA`]), e uma ferramenta nova desta espécie entra
//! nela — ⛔ nunca numa cerca própria.
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
//! | `ph2d_sprite_screen::uv_sob_o_ponteiro` (as três entradas da Remoção de fundo) | o afim do quad |
//! | o passe de sprites | desenha o quad |
//!
//! ⇒ **suspender a malha suspende todas as metades de uma vez**, e nenhuma delas precisa de saber
//! que ferramenta está na mão. *Uma cerca em cada consumidor seria a mesma lei escrita em vários
//! sítios, a divergir no primeiro que alguém refactorasse.* ⭐ E é pela mesma razão que a exceção do
//! Liquify não precisa de código nos consumidores: com ele na mão a malha fica, e cada um deles volta
//! a seguir a dobra sozinho (o anel do Liquify incluído —
//! `painter_bridge_brush_ring::anel_do_liquify`).
//!
//! # ⚠️ O regresso é por CONSTRUÇÃO, não por memória
//!
//! A malha é re-posta **a cada quadro** pelo `attach_skin_meshes`. Sair da ferramenta (ou trocar
//! para o Liquify) é deixar de a suspender, e a deformação volta sozinha no quadro seguinte. ⛔ **Não
//! há estado a repor** — e é isso que impede o modo de ficar «preso achatado» depois de um crash, de
//! um undo ou de trocar de selecção a meio.
//!
//! ⚠️ **A tinta sobrevive intacta:** a malha vive nos bytes opacos da `ph2d_skeleton_ecs::SkinBind`
//! (traçada no bind) e editar escreve na TEXTURA. As duas não se tocam, logo o que for pintado ou
//! removido achatado aparece deformado quando a pele volta — que é o que o artista espera de editar
//! uma textura.

use ph2d_editor_core::ToolRegistry;

/// ⭐⭐⭐ **AS FERRAMENTAS QUE EDITAM PIXELS** — a tabela da regra (F6-s). Uma imagem presa a ossos é
/// desenhada ACHATADA enquanto uma destas está na mão.
///
/// ⚠️ **As que ficam de fora, e porquê** (a tabela inteira está na fila, com as leituras que são
/// pergunta ao dono): o Color Equalization é um FILTRO; o Upscale e o Equalize Sizes reamostram a
/// imagem inteira; o Padding mexe na moldura da tela; o vector, o flip, o motion e o move não editam
/// pixels.
pub const FERRAMENTAS_QUE_ACHATAM: &[&str] = &["painter", "bgremoval"];

/// ⭐⭐ **OS MODOS que, dentro de uma ferramenta da tabela, trabalham SOBRE A DOBRA** — a exceção
/// nomeada pelo dono: o Liquify *«deverá ser capaz de fazer ajustes na imagem dobrada»*.
///
/// ⚠️ **A chave é o `active_paint_mode_id` do Painter, e não o `PaintMode`:** o `Deform` cobre DUAS
/// ferramentas do trilho — o *Liquify* e o *Transform* —, e só a primeira é a exceção (a segunda
/// deforma pixels por gizmo, e achata).
pub const MODOS_SOBRE_A_DOBRA: &[&str] = &["liquify"];

/// ⭐⭐⭐ **A sprite cuja pele fica suspensa neste quadro** — `None` quando ninguém está a editar
/// pixels (ou quando o modo na mão trabalha sobre a dobra).
///
/// `seleccionada` são os bits da entidade que o gizmo tem na mão: é ela que a ferramenta edita.
///
/// ⚠️ **`&mut` por causa do contrato das ferramentas** (`Tool`, congelado): o único caminho até ao
/// Painter concreto é o `as_any_mut`, e perguntar-lhe o modo precisa dele.
///
/// ⚠️ **Ela não pergunta se a sprite TEM pele**, de propósito: quem sabe isso é o
/// `ph2d_skeleton_live::skin_image::is_skinned_image`, do lado de lá, e perguntá-lo aqui seria uma
/// segunda resposta à mesma pergunta — sobre uma sprite sem pele esta devolve um id que não tem
/// malha nenhuma para suspender, e o efeito é exactamente nenhum.
#[must_use]
pub fn sprite_achatada(tools: &mut ToolRegistry, seleccionada: Option<u64>) -> Option<u64> {
    let ferramenta = tools.active_mut()?;
    let id = ferramenta.id();
    if !FERRAMENTAS_QUE_ACHATAM
        .iter()
        .any(|f| id == ph2d_editor_core::ToolId::new(*f))
    {
        return None;
    }
    let sobre_a_dobra = ferramenta
        .as_any_mut()
        .downcast_mut::<ph2d_tool_painter::PainterTool>()
        .is_some_and(|p| MODOS_SOBRE_A_DOBRA.contains(&p.active_paint_mode_id()));
    if sobre_a_dobra {
        return None;
    }
    seleccionada
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
    tools: &mut ToolRegistry,
    seleccionada: Option<u64>,
    e_pele: impl Fn(ph2d_ecs::Entity) -> bool,
    toasts: &mut ph2d_editor_core::toast::ToastQueue,
) -> Option<u64> {
    let achatada = sprite_achatada(tools, seleccionada);
    if let Some(bits) = acabou_de_achatar(achatada)
        && ph2d_ecs::Entity::try_from_bits(bits).is_some_and(&e_pele)
    {
        toasts.push(ph2d_editor_core::toast::Toast::info(
            "Editing pixels flattens this image — the bone deformation returns when you leave the \
             tool."
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
        let mut tools = ToolRegistry::default();
        assert_eq!(
            super::sprite_achatada(&mut tools, Some(42)),
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

    /// Um registo com o Painter NA MÃO, no modo do trilho pedido.
    fn painter_no_modo(modo: &str) -> ToolRegistry {
        let mut tools = ToolRegistry::default();
        let mut p = ph2d_tool_painter::PainterTool::default();
        p.set_paint_tool_mode(modo);
        assert_eq!(p.active_paint_mode_id(), modo, "o modo {modo} nao pegou");
        tools.register(Box::new(p));
        assert!(tools.set_active(&ph2d_editor_core::ToolId::new("painter")));
        tools
    }

    /// ⭐⭐⭐ **A REGRA (F6-s): todo modo do Painter que mexe em pixels achata — e o LIQUIFY não.**
    ///
    /// ⚠️ **O `Transform` é o gémeo que prova a chave:** ele partilha o `PaintMode::Deform` com o
    /// Liquify, e uma exceção escrita sobre o `PaintMode` deixava-o dobrado — deformando pixels por
    /// gizmo sobre a arte dobrada, que é o que a regra proíbe.
    ///
    /// (Mutações: a exceção desaparecer ⇒ RED no Liquify; a exceção ser o `is_deform_mode` ⇒ RED no
    /// Transform.)
    #[test]
    fn every_pixel_mode_flattens_and_liquify_works_on_the_bend() {
        for modo in [
            "brush",
            "eraser",
            "smear",
            "blur",
            "clone",
            "inpaint",
            "transform",
        ] {
            assert_eq!(
                super::sprite_achatada(&mut painter_no_modo(modo), Some(7)),
                Some(7),
                "o modo {modo} do Painter edita pixels e tem de achatar"
            );
        }
        assert_eq!(
            super::sprite_achatada(&mut painter_no_modo("liquify"), Some(7)),
            None,
            "o Liquify trabalha SOBRE a dobra (a excecao do dono)"
        );
    }

    /// ⭐ **A TABELA é a da regra** — a Remoção de fundo entra; uma ferramenta que não edita pixels,
    /// não. ⚠️ Lido da tabela (a Remoção de fundo não é dependência desta crate): o que este gate
    /// prende é o CONTEÚDO dela, e o do Painter prende o CAMINHO.
    #[test]
    fn the_table_is_the_rule() {
        assert!(super::FERRAMENTAS_QUE_ACHATAM.contains(&"bgremoval"));
        assert!(super::FERRAMENTAS_QUE_ACHATAM.contains(&"painter"));
        for fora in [
            "color_equalization",
            "upscale",
            "equalize_sizes",
            "padding",
            "vector",
        ] {
            assert!(
                !super::FERRAMENTAS_QUE_ACHATAM.contains(&fora),
                "{fora} nao edita pixels por pincel — a regra deixa-o sobre a dobra"
            );
        }
    }
}
