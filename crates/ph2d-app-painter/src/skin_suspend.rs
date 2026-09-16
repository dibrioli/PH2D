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
//! ⭐⭐⭐ **E no mesmo dia o dono completou-a** (respostas às perguntas da linha): *«Color Equalization
//! e qualquer outra do tipo que trata apenas cores, não endireita a imagem, pode ser aplicada
//! dobrada. As que mudam tamanho ou padding devem endireitar e se aplicadas quebrar o binding com os
//! ossos.»* ⇒ a segunda tabela, [`FERRAMENTAS_QUE_MUDAM_A_MOLDURA`], e a porta do Apply
//! ([`solta_se_mudou_a_moldura`]). ⚠️ E a suspensão passou a ser um CONJUNTO ([`Alcance`]): essas
//! ferramentas editam a selecção inteira.
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

/// ⭐⭐ **ATÉ ONDE A SUSPENSÃO CHEGA** — a imagem que a ferramenta edita, ou todas as que ela edita.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alcance {
    /// Só a imagem PRINCIPAL da selecção: o Painter trava a selecção numa só (`painter_lock`), e a
    /// Remoção de fundo pré-visualiza e aplica na principal.
    Principal,
    /// A SELECÇÃO inteira: o Padding, o Upscale e o Equalize Sizes aplicam a todas as selecionadas
    /// (os três drenos percorrem o `iter_selected`) — achatar só a principal deixaria as outras
    /// dobradas debaixo da mesma operação.
    Seleccao,
}

/// ⭐⭐⭐ **AS FERRAMENTAS QUE ACHATAM** — a tabela da regra (F6-s). Uma imagem presa a ossos é
/// desenhada ACHATADA enquanto uma destas está na mão: as que pintam, apagam ou deformam pixels, e as
/// que mudam a MOLDURA da imagem ([`FERRAMENTAS_QUE_MUDAM_A_MOLDURA`] — as que têm painel, porque as
/// de um clique não têm «enquanto»).
///
/// ⚠️ **As que ficam de fora, e porquê:** o Color Equalization e toda ferramenta que só trata CORES
/// (decisão do dono: *«pode ser aplicada dobrada»*); o vector, o flip, o motion e o move não editam
/// pixels.
pub const FERRAMENTAS_QUE_ACHATAM: &[(&str, Alcance)] = &[
    ("painter", Alcance::Principal),
    ("bgremoval", Alcance::Principal),
    ("padding", Alcance::Seleccao),
    ("upscale", Alcance::Seleccao),
    ("equalize_sizes", Alcance::Seleccao),
];

/// ⭐⭐ **OS MODOS que, dentro de uma ferramenta da tabela, trabalham SOBRE A DOBRA** — a exceção
/// nomeada pelo dono: o Liquify *«deverá ser capaz de fazer ajustes na imagem dobrada»*.
///
/// ⚠️ **A chave é o `active_paint_mode_id` do Painter, e não o `PaintMode`:** o `Deform` cobre DUAS
/// ferramentas do trilho — o *Liquify* e o *Transform* —, e só a primeira é a exceção (a segunda
/// deforma pixels por gizmo, e achata).
pub const MODOS_SOBRE_A_DOBRA: &[&str] = &["liquify"];

/// ⭐⭐⭐ **AS FERRAMENTAS QUE MUDAM A MOLDURA DA IMAGEM** — o TAMANHO ou a MARGEM. Aplicadas a uma
/// imagem presa, **quebram a ligação com os ossos** (decisão do dono, 2026-09-16).
///
/// ⚠️ **O porquê é geométrico, e não um gosto:** a malha do bind foi traçada sobre a moldura de
/// ANTES (a tinta, em px da textura daquele tamanho). Depois de cortar, acrescentar margem,
/// reamostrar ou assar a escala, a mesma malha lê os texels do sítio errado.
///
/// | ferramenta | o que muda |
/// |---|---|
/// | `padding` | a margem (e o tamanho) |
/// | `upscale` | a resolução |
/// | `equalize_sizes` | o tamanho (tela ou escala) |
/// | `trim_transparency` | corta a margem transparente |
/// | `make_square` | acrescenta margem |
/// | `rasterize` | assa a escala e a rotação nos pixels |
/// | `real_size` | a escala volta a `1:1` |
pub const FERRAMENTAS_QUE_MUDAM_A_MOLDURA: &[&str] = &[
    "padding",
    "upscale",
    "equalize_sizes",
    "trim_transparency",
    "make_square",
    "rasterize",
    "real_size",
];

/// O aviso ao achatar, para quem só mexe em pixels.
pub const AVISO_PIXELS: &str =
    "Editing pixels flattens this image — the bone deformation returns when you leave the tool.";
/// O aviso ao achatar, para quem muda a moldura — ⚠️ ele diz o que o Apply vai fazer, ANTES de o
/// artista carregar no botão.
pub const AVISO_MOLDURA: &str = "This tool changes the image size or margins: the image is shown \
     without the bone deformation, and Apply unbinds it from the bones.";

/// A decisão: que sprites ficam achatadas, e se a ferramenta na mão muda a moldura.
fn decide(tools: &mut ToolRegistry, seleccao: impl IntoIterator<Item = u64>) -> (Vec<u64>, bool) {
    let Some(ferramenta) = tools.active_mut() else {
        return (Vec::new(), false);
    };
    let id = ferramenta.id();
    let Some(&(_, alcance)) = FERRAMENTAS_QUE_ACHATAM.iter().find(|(f, _)| id.0 == *f) else {
        return (Vec::new(), false);
    };
    let sobre_a_dobra = ferramenta
        .as_any_mut()
        .downcast_mut::<ph2d_tool_painter::PainterTool>()
        .is_some_and(|p| MODOS_SOBRE_A_DOBRA.contains(&p.active_paint_mode_id()));
    if sobre_a_dobra {
        return (Vec::new(), false);
    }
    let mut seleccao = seleccao.into_iter();
    let achatadas = match alcance {
        Alcance::Principal => seleccao.next().into_iter().collect(),
        Alcance::Seleccao => seleccao.collect(),
    };
    let moldura = FERRAMENTAS_QUE_MUDAM_A_MOLDURA.contains(&id.0.as_str());
    (achatadas, moldura)
}

/// ⭐⭐⭐ **As sprites cuja pele fica suspensa neste quadro** — vazio quando ninguém está a editar
/// pixels ou a moldura (ou quando o modo na mão trabalha sobre a dobra).
///
/// `seleccao` é a selecção do gizmo, com a PRINCIPAL primeiro (`GizmoStateGroup::iter_selected`).
///
/// ⚠️ **`&mut` por causa do contrato das ferramentas** (`Tool`, congelado): o único caminho até ao
/// Painter concreto é o `as_any_mut`, e perguntar-lhe o modo precisa dele.
///
/// ⚠️ **Ela não pergunta se a sprite TEM pele**, de propósito: quem sabe isso é o
/// `ph2d_skeleton_live::skin_image::is_skinned_image`, do lado de lá, e perguntá-lo aqui seria uma
/// segunda resposta à mesma pergunta — sobre uma sprite sem pele esta devolve um id que não tem
/// malha nenhuma para suspender, e o efeito é exactamente nenhum.
#[must_use]
pub fn sprites_achatadas(
    tools: &mut ToolRegistry,
    seleccao: impl IntoIterator<Item = u64>,
) -> Vec<u64> {
    decide(tools, seleccao).0
}

/// **O conjunto achatado MUDOU desde o último quadro?** — para quem quiser dizê-lo ao artista.
///
/// ⚠️⚠️ **Sem isto a arte SALTA e o artista lê um defeito.** Pegar na ferramenta faz um canvas
/// dobrado endireitar-se de um quadro para o outro; sem uma palavra, o report seguinte é *«a arte
/// saltou»* — e a cura seria explicar o que o app já sabia.
///
/// ⛔ **É um latch por ARESTA, e ele não é um campo da `App`:** o idioma é o que o
/// `color_equalization_bridge` já usa (um `static` + `swap`), e a razão de não ser estado da `App` é
/// a catraca `the_app_only_sheds_fields` — mas também é a mais honesta, porque isto não é estado do
/// documento nem da sessão: é a memória de UM quadro.
///
/// ⚠️ **Guarda uma IMPRESSÃO do conjunto** (FNV-1a sobre os bits, pela ordem da selecção), com `0`
/// para «nenhuma»: o conjunto vazio imprime `0`, e um conjunto cheio nunca (`max(1)`).
#[must_use]
pub fn acabou_de_achatar(achatadas: &[u64]) -> bool {
    use std::sync::atomic::{AtomicU64, Ordering};
    static ULTIMA: AtomicU64 = AtomicU64::new(0);
    let agora = if achatadas.is_empty() {
        0
    } else {
        achatadas
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
                (h ^ b).wrapping_mul(0x0100_0000_01b3)
            })
            .max(1)
    };
    let antes = ULTIMA.swap(agora, Ordering::Relaxed);
    agora != 0 && agora != antes
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
    seleccao: impl IntoIterator<Item = u64>,
    e_pele: impl Fn(ph2d_ecs::Entity) -> bool,
    toasts: &mut ph2d_editor_core::toast::ToastQueue,
) -> Vec<u64> {
    let (achatadas, moldura) = decide(tools, seleccao);
    if acabou_de_achatar(&achatadas)
        && achatadas
            .iter()
            .any(|b| ph2d_ecs::Entity::try_from_bits(*b).is_some_and(&e_pele))
    {
        let aviso = if moldura { AVISO_MOLDURA } else { AVISO_PIXELS };
        toasts.push(ph2d_editor_core::toast::Toast::info(aviso.to_string()));
    }
    achatadas
}

/// ⭐⭐ **O QUE UMA FERRAMENTA MUDOU NUM APPLY** — as entradas da transacção de desfazer, com o id
/// de quem as fez.
///
/// ⚠️ **O id nasce JUNTO da transacção** (`Edicao::new("padding")` no sítio onde antes se escrevia
/// `Vec::new()`), e é isso que impede a gravação de o esquecer: quem grava recebe uma `Edicao`, e
/// uma `Edicao` sem ferramenta não existe. Ela é um `Vec` por `Deref`, então os drenos que enchem a
/// transacção não mudam.
pub struct Edicao<T> {
    /// O id da ferramenta que aplicou (o de `Tool::id`).
    pub ferramenta: &'static str,
    /// O que ela mudou — uma entrada por sprite.
    pub entradas: Vec<T>,
}

impl<T> Edicao<T> {
    /// Uma transacção vazia da ferramenta `ferramenta`.
    #[must_use]
    pub fn new(ferramenta: &'static str) -> Self {
        Self {
            ferramenta,
            entradas: Vec::new(),
        }
    }
}

impl<T> std::ops::Deref for Edicao<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.entradas
    }
}

impl<T> std::ops::DerefMut for Edicao<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.entradas
    }
}

/// ⭐⭐⭐ **O APPLY DE UMA FERRAMENTA QUE MUDA A MOLDURA SOLTA AS IMAGENS DOS OSSOS** (decisão do
/// dono, 2026-09-16) — devolve quantas soltou.
///
/// `ferramenta` é o id da ferramenta que aplicou; `editadas` são as sprites que o Apply DE FACTO
/// mudou (as que entraram na transacção de desfazer — um Trim sobre uma imagem sem margem não muda
/// nada, e não solta nada); `solta` tira a pele a uma e diz se havia pele (quem sabe fazê-lo é a
/// `ph2d-skeleton-live`, e esta crate não depende dela).
///
/// ⚠️ **Uma porta para TODOS os Apply** — o chamador passa sempre o id, e é ESTA tabela que decide.
/// *Uma decisão escrita no sítio de chamada seria esquecida pela próxima ferramenta.*
///
/// ⭐ **O regresso é o Ctrl+Z de sempre:** soltar no mesmo quadro do Apply põe a imagem nova e a
/// ligação perdida no MESMO passo de desfazer, e o aviso di-lo.
pub fn solta_se_mudou_a_moldura(
    ferramenta: &str,
    editadas: impl IntoIterator<Item = u64>,
    mut solta: impl FnMut(u64) -> bool,
    toasts: &mut ph2d_editor_core::toast::ToastQueue,
) -> usize {
    if !FERRAMENTAS_QUE_MUDAM_A_MOLDURA.contains(&ferramenta) {
        return 0;
    }
    let soltas = editadas.into_iter().filter(|b| solta(*b)).count();
    if soltas > 0 {
        toasts.push(ph2d_editor_core::toast::Toast::info(format!(
            "The image size or margins changed: {soltas} image(s) unbound from the bones. \
             Ctrl+Z brings the binding back."
        )));
    }
    soltas
}

#[cfg(test)]
#[path = "skin_suspend_tests.rs"]
mod tests;
