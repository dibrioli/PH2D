//! ⭐⭐⭐ **UMA IMAGEM PRESA AO ESQUELETO É UMA SPRITE DO QUADRO, desenhada como MALHA.**
//!
//! (plano `docs/Skeleton/03_plano_a_pele_no_passe_de_sprites.md`, W2, 2026-09-13)
//!
//! ⛔⛔ **Até esse dia ela era desenhada por DOIS motores com uma guarda entre eles** — o extract não
//! emitia a instância dela e o Vello desenhava-a deformada por CIMA do quadro. Daí os cinco defeitos
//! da fila F6-h: fora da ordem, visível com o olho fechado, sem as propriedades da sprite, com
//! costuras entre peças, e meio quad ao lado com *Centered* desligado. Os gates que aqui viviam
//! provavam essa camada; estes provam a lei nova, e cada um prova também que a antiga não voltou.
//!
//! # ⚠️⚠️ O que um gate que VARRE O FONTE não pode ver
//!
//! Estes leem o texto do produto: apanham uma chamada a **desaparecer**, a mudar de **selector** e
//! a mudar de **ordem**; não apanham uma chamada morta por uma condição. ⛔ **A cura NÃO é apertar
//! a varredura.** A metade que falta é medida do outro lado: os gates de unidade da
//! `ph2d-skeleton-live` provam que a malha posta é a do quad em repouso e que só a instância base a
//! recebe, o gate de GPU `ph2d-render::sprite_mesh_gpu` prova que a malha desenha o que o quad
//! desenha, e o smoke julga a costura inteira. *Um gate de texto responde «isto está escrito», nunca
//! «isto corre».*

const EXTRACT: &str = include_str!("../../src/render_loop/sim_extract.rs");

/// **O fonte sem comentários** — sem isto, uma nota que cita a chamada conta como chamada.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") { "" } else { l }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **A SPRITE PRESA EMITE A SUA INSTÂNCIA** — nenhuma guarda a tira do braço que emite.
///
/// ⚠️ É esse braço que lhe dá o rank, a visibilidade e as propriedades (tinta, opacidade, mistura,
/// recorte). ⛔ E a pergunta *«está presa?»* não volta a este ficheiro: o extract não precisa de
/// saber o que é uma pele, e a última vez que soube tirou a imagem do quadro.
#[test]
fn a_bound_sprite_emits_its_instance_like_any_sprite() {
    let src = code_only(EXTRACT);
    let j = src
        .find("let Some(spr) = sim.get::<Sprite>(sim_entity)")
        .expect("o braço que emite a instância da sprite deixou de existir");
    let i = src[..j]
        .rfind("if ")
        .expect("o braço que emite a sprite perdeu a condição");
    let condicao = &src[i..j];
    assert!(
        !condicao.contains("skinned") && !condicao.contains("SkinBind"),
        "o braço que emite a sprite voltou a perguntar se ela está presa: `{condicao}`"
    );
    assert!(
        !src.contains("SkinBind"),
        "o extract voltou a ler a pele (`SkinBind`) — quem sabe o que é uma pele é a \
         `ph2d-skeleton-live`, depois do extract"
    );
}

/// ⭐⭐⭐ **A MALHA É POSTA DEPOIS DO EXTRACT, e a camada do Vello não voltou.**
///
/// ⚠️ **A ordem é a leitura:** a malha vai para a instância que o extract emitiu — antes dele não há
/// instância nenhuma (o `present` é refeito por quadro), e é por isso que uma sprite escondida não
/// recebe malha. Mede-se no QUADRO emendado (`frame_text::render_frame`), que é a ordem em que corre.
#[test]
fn the_frame_attaches_the_skin_mesh_after_the_extract_and_draws_no_skin_layer() {
    let src = code_only(&crate::frame_text::render_frame());
    let extract = src
        .find("sim_extract::run(")
        .expect("o quadro deixou de correr o extract");
    let malha = src
        .find("skeleton_skin_image::attach_skin_meshes(")
        .expect("o quadro deixou de pôr a malha nas imagens presas");
    assert!(
        extract < malha,
        "a malha é posta ANTES do extract (byte {malha} contra {extract}) — ali não há instância \
         nenhuma para a receber, e a imagem desenha-se sem deformar"
    );
    assert!(
        !src.contains("draw_skinned_images"),
        "a camada do Vello voltou: a imagem presa seria desenhada duas vezes, e a de cima fora da \
         ordem do quadro"
    );
}

/// ⭐⭐⭐ **O MESMO BOTÃO *Bind* alcança as duas mídias.**
///
/// ⚠️ **O sujeito de uma imagem é a SELECÇÃO DO GIZMO**, e não a lista de caminhos do pen — são
/// duas famílias com dois selectores, e ler o do vector daria sempre **zero** imagens. Este gate
/// mede que o dreno lê o selector certo.
#[test]
fn the_bind_verb_reaches_both_media() {
    // ⚠️ O dreno do *Bind* lê-se no QUADRO pela ordem em que corre (`frame_text::render_frame`): desde a OBRA 2 da
    // `line/render-loop` (2026-09-13) ele mora na `fase_skeleton_verbs`, e a selecção do gizmo continua no `mod.rs`.
    let src = code_only(&crate::frame_text::render_frame());
    // ⚠️⚠️ **A ÂNCORA mudou de `if pending_bone_bind {` para o BLOCO ROTULADO, e o gate reprovou
    // sobre produto CERTO** (2026-09-18): a recusa do bind precisa de sair do bloco sem levar com
    // ela o soltar e os knobs, e isso pede um `'bind: { … break 'bind }`. *Um gate ancorado no
    // idioma reprova no dia em que o idioma muda* — a afirmação dele continua intacta, e só a
    // âncora foi curada.
    let i = src
        .find("'bind: {")
        .expect("o dreno do *Bind* deixou de existir");
    let corpo = &src[i..i + 2600];
    assert!(
        corpo.contains("skeleton_live::bind("),
        "o *Bind* deixou de prender FORMAS"
    );
    assert!(
        corpo.contains("skeleton_live::bind_image("),
        "o *Bind* não alcança as IMAGENS — o botão existe, o gesto existe, e a 2.ª mídia fica \
         inalcançável por dentro"
    );
    assert!(
        corpo.contains("selecao_bits"),
        "o *Bind* de imagens não lê a selecção do gizmo — lendo a lista do pen, ele acha sempre \
         zero imagens e a recusa é MUDA"
    );
}

/// ⭐⭐⭐ **O MESMO botão *Release* solta as duas mídias** — o irmão do gate acima, e o report do dono
/// (2026-09-18: *«ainda não temos a opção de desconectar a malha do osso»*).
///
/// ⛔⛔⛔ **O *Bind* alcançava as duas mídias desde a wave da 2.ª mídia e o *Release* alcançava UMA.**
/// A lei da imagem (`skin_image::release_image`) existia, e o **único** chamador de produto dela era
/// automático — uma ferramenta que muda a moldura solta o osso sozinha. *Uma lei sem gesto é uma lei
/// que o artista não tem*, e este repo tem outra nomeada assim (o `dock_columns::close`).
///
/// ⚠️ **As três metades são três defeitos:** não soltar imagens · soltá-las pelo selector errado (a
/// lista do pen dá sempre zero imagens) · e soltá-las também no *Expand*, que é a operação de que
/// uma imagem não tem sujeito.
#[test]
fn the_release_verb_reaches_both_media() {
    let src = code_only(&crate::frame_text::render_frame());
    let i = src
        .find("if let Some(keep) = pending_bone_release {")
        .expect("o dreno do *Release* deixou de existir");
    let corpo = &src[i..i + 1400];
    assert!(
        corpo.contains("skeleton_live::release("),
        "o *Release* deixou de soltar FORMAS"
    );
    assert!(
        corpo.contains("release_image("),
        "o *Release* não alcança as IMAGENS — a lei existe, o botão existe, e a única rota de \
         produto para ela é automática (o solta-se-sozinho de uma ferramenta de moldura)"
    );
    assert!(
        corpo.contains("selecao_bits"),
        "o *Release* de imagens não lê a selecção do gizmo — lendo a lista do pen ele acha sempre \
         zero imagens, que é o mesmo que não existir"
    );
    assert!(
        corpo.contains("Keep::Source"),
        "o *Release* de imagens deixou de ser cercado ao Keep::Source: no Keep::Deformed ele é o \
         *Expand*, que troca o desenho autorado pela geometria de agora — e uma imagem não tem \
         geometria autorada"
    );
}

/// ⛔⛔ **E o PAINEL tem de SABER que a imagem está presa, senão o botão nem é pintado.**
///
/// ⚠️ **O cabeçalho da fase já prometia isto por escrito** (*«forma presa ou imagem com pele»*)
/// enquanto o código lia só `selected_paths()` — *um doc que declara a lei que o código não
/// implementa lê-se como auditado*, e foi o dono que o apanhou.
#[test]
fn the_panel_learns_that_an_image_is_bound() {
    const FASE: &str = include_str!("../../src/render_loop/fase_selection_mirror_skin.rs");
    assert!(
        FASE.contains("is_skinned_image("),
        "a fase que publica o facto ao painel nao pergunta se a IMAGEM esta' presa: com uma \
         imagem escolhida o painel le' `false` e as saidas nem sao pintadas"
    );
    assert!(
        FASE.contains("gizmo.iter_selected()"),
        "a fase le' a imagem pelo selector errado: a lista de caminhos do pen da' sempre zero \
         imagens, e o facto publicado seria falso para sempre"
    );
    assert!(
        FASE.contains("Skinned { vector, imagem }"),
        "o facto publicado deixou de carregar as DUAS midias num tipo: dois publicadores por \
         campo deixam um quadro em que uma metade e' desta seleccao e a outra da anterior"
    );
}
