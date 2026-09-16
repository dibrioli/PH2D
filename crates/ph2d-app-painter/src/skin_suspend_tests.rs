//! Os gates da porta das ferramentas sobre a arte presa aos ossos — irmão de [`super`] por
//! assunto (a porta tem duas tabelas e três decisões, e os gates delas são maiores que ela).
//!
//! ⚠️ **Só UM teste toca no latch** (`o_latch_fala_uma_vez_…`): ele é um `static` do processo, e os
//! testes correm em paralelo — os outros perguntam à decisão (`decide`), que não o toca.

use super::{Alcance, decide};
use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::tool::Tool;

/// Uma ferramenta que só tem NOME — as ferramentas de imagem não são dependências desta crate, e o
/// que a porta pergunta a uma delas é só o id.
struct SoNome(&'static str);

impl Tool for SoNome {
    fn id(&self) -> ph2d_editor_core::ToolId {
        ph2d_editor_core::ToolId::new(self.0)
    }
    fn label(&self) -> &str {
        self.0
    }
    fn icon_slug(&self) -> &str {
        self.0
    }
    fn build_panel(&self) -> ph2d_editor_core::FloatingPanel {
        ph2d_editor_core::FloatingPanel::new(self.id(), self.0)
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Um registo com a ferramenta `id` NA MÃO.
fn na_mao(id: &'static str) -> ToolRegistry {
    let mut tools = ToolRegistry::default();
    tools.register(Box::new(SoNome(id)));
    assert!(tools.set_active(&ph2d_editor_core::ToolId::new(id)));
    tools
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

/// A selecção dos gates: a principal `7`, mais `8` e `9`.
const SELECCAO: [u64; 3] = [7, 8, 9];

/// ⭐⭐ **SEM FERRAMENTA NA MÃO, NADA É SUSPENSO** — o controlo desta porta.
///
/// ⛔ Sem esta metade, uma porta que devolvesse sempre a selecção achataria toda arte presa a ossos
/// o tempo todo, e a 2.ª mídia deixava de existir. *A cura de um modo tem de estar presa ao modo.*
#[test]
fn sem_ferramenta_na_mao_nada_e_suspenso() {
    let mut tools = ToolRegistry::default();
    assert!(
        super::sprites_achatadas(&mut tools, SELECCAO).is_empty(),
        "sem ferramenta activa nenhuma pele pode ser suspensa"
    );
}

/// ⭐⭐⭐ **O LATCH DISPARA UMA VEZ POR ENTRADA, e volta a armar ao sair.**
///
/// ⚠️ **As três metades**: ele fala na entrada, **cala-se** enquanto se edita (senão o aviso
/// aparece a 60 Hz) e **volta a armar** quando se sai — sem a terceira, entrar uma segunda vez no
/// mesmo canvas seria mudo, que é exactamente quando o artista já esqueceu a primeira.
#[test]
fn o_latch_fala_uma_vez_por_entrada_e_volta_a_armar() {
    use super::acabou_de_achatar as mudou;
    assert!(mudou(&[7]), "a entrada fala");
    assert!(
        !mudou(&[7]),
        "o quadro seguinte com a MESMA sprite tem de ser mudo"
    );
    assert!(!mudou(&[]), "sair e' mudo");
    assert!(
        mudou(&[7]),
        "entrar OUTRA VEZ no mesmo canvas volta a falar"
    );
    // ⚠️ E trocar de sujeito SEM sair também fala: é outra arte a achatar.
    assert!(mudou(&[9]));
    // ⭐ E o CONJUNTO conta: acrescentar uma imagem à selecção achatada é outra arte a achatar.
    assert!(mudou(&[9, 8]));
    assert!(!mudou(&[9, 8]));
    let _ = mudou(&[]);
}

/// ⭐⭐⭐ **A REGRA (F6-s): todo modo do Painter que mexe em pixels achata — e o LIQUIFY não.**
///
/// ⚠️ **O `Transform` é o gémeo que prova a chave:** ele partilha o `PaintMode::Deform` com o
/// Liquify, e uma exceção escrita sobre o `PaintMode` deixava-o dobrado — deformando pixels por
/// gizmo sobre a arte dobrada, que é o que a regra proíbe.
///
/// ⚠️ E o Painter achata **só a principal** — ele trava a selecção numa, e o `8` e o `9` aqui são
/// o controlo de que o alcance não vaza.
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
            decide(&mut painter_no_modo(modo), SELECCAO),
            (vec![7], false),
            "o modo {modo} do Painter edita pixels da PRINCIPAL e tem de a achatar"
        );
    }
    assert!(
        super::sprites_achatadas(&mut painter_no_modo("liquify"), SELECCAO).is_empty(),
        "o Liquify trabalha SOBRE a dobra (a excecao do dono)"
    );
}

/// ⭐⭐⭐ **AS FERRAMENTAS DE MOLDURA achatam a SELECÇÃO inteira, e avisam do Apply** (decisão do
/// dono, 2026-09-16) — a Remoção de fundo achata só a principal, e uma ferramenta de COR não achata
/// nada.
///
/// (Mutações: o alcance ignorado — só a principal — ⇒ RED; o aviso de moldura para toda ferramenta
/// ⇒ RED na Remoção de fundo; o Color Equalization na tabela ⇒ RED.)
#[test]
fn the_frame_tools_flatten_the_whole_selection_and_the_colour_tools_nothing() {
    for moldura in ["padding", "upscale", "equalize_sizes"] {
        assert_eq!(
            decide(&mut na_mao(moldura), SELECCAO),
            (SELECCAO.to_vec(), true),
            "{moldura} edita a seleccao inteira e muda a moldura"
        );
    }
    assert_eq!(
        decide(&mut na_mao("bgremoval"), SELECCAO),
        (vec![7], false),
        "a Remocao de fundo edita a principal e so' mexe em pixels"
    );
    for fora in ["color_equalization", "vector", "move"] {
        assert_eq!(
            decide(&mut na_mao(fora), SELECCAO),
            (Vec::new(), false),
            "{fora} nao pinta, nao apaga e nao muda a moldura — a imagem fica dobrada"
        );
    }
}

/// ⭐⭐ **AS DUAS TABELAS CONCORDAM** — toda ferramenta de moldura com painel achata (*«devem
/// endireitar»*), com o alcance da selecção; e nenhuma de cor está em nenhuma.
///
/// ⚠️ As de UM clique (`trim_transparency` · `make_square` · `rasterize` · `real_size`) não têm
/// «enquanto», logo não estão na tabela do achatamento — e o gate afirma-o, para ninguém as juntar a
/// ela a pensar que falta.
#[test]
fn the_two_tables_agree() {
    let achata = |id: &str| {
        super::FERRAMENTAS_QUE_ACHATAM
            .iter()
            .find(|(f, _)| *f == id)
            .map(|(_, a)| *a)
    };
    for moldura in super::FERRAMENTAS_QUE_MUDAM_A_MOLDURA {
        let de_um_clique =
            ["trim_transparency", "make_square", "rasterize", "real_size"].contains(moldura);
        assert_eq!(
            achata(moldura),
            (!de_um_clique).then_some(Alcance::Seleccao),
            "{moldura}: uma ferramenta de moldura com painel achata a seleccao; uma de um clique \
             nao tem enquanto"
        );
    }
    let cor = "color_equalization";
    assert!(achata(cor).is_none());
    assert!(!super::FERRAMENTAS_QUE_MUDAM_A_MOLDURA.contains(&cor));
    // As duas de pixels, com o alcance delas.
    assert_eq!(achata("painter"), Some(Alcance::Principal));
    assert_eq!(achata("bgremoval"), Some(Alcance::Principal));
}

/// ⭐⭐⭐ **O APPLY de uma ferramenta de moldura SOLTA as imagens presas que mudou — e diz quantas.**
/// Um Apply de cor ou de pixels não solta nada, nem pergunta.
///
/// (Mutações: a guarda da tabela apagada ⇒ RED no Color Equalization; soltar sem contar as que
/// tinham pele ⇒ RED na contagem; o aviso calado ⇒ RED.)
#[test]
fn a_frame_apply_releases_the_bound_images_it_changed_and_a_colour_apply_does_not() {
    let mut toasts = ph2d_editor_core::toast::ToastQueue::new();
    let mut pedidas = Vec::new();
    // `8` não tinha pele: a porta pergunta, e ela não conta.
    let n = super::solta_se_mudou_a_moldura(
        "padding",
        SELECCAO,
        |b| {
            pedidas.push(b);
            b != 8
        },
        &mut toasts,
    );
    assert_eq!(pedidas, SELECCAO.to_vec(), "cada editada e' perguntada");
    assert_eq!(n, 2, "soltou as duas que tinham pele");
    assert_eq!(toasts.len(), 1, "o artista e' avisado");
    let aviso = &toasts.iter().next().expect("aviso").message;
    assert!(
        aviso.contains('2') && aviso.contains("Ctrl+Z"),
        "o aviso diz quantas e como voltar atras: {aviso}"
    );

    for sem_moldura in ["color_equalization", "painter", "bgremoval"] {
        let mut toasts = ph2d_editor_core::toast::ToastQueue::new();
        let n = super::solta_se_mudou_a_moldura(
            sem_moldura,
            SELECCAO,
            |_| panic!("{sem_moldura} nao muda a moldura e nao pode soltar nada"),
            &mut toasts,
        );
        assert_eq!(n, 0);
        assert!(toasts.is_empty(), "{sem_moldura}: aviso sem soltura");
    }

    // Um Apply de moldura que não soltou nada (nenhuma tinha pele) é mudo.
    let mut toasts = ph2d_editor_core::toast::ToastQueue::new();
    assert_eq!(
        super::solta_se_mudou_a_moldura("trim_transparency", [7], |_| false, &mut toasts),
        0
    );
    assert!(toasts.is_empty());
}
