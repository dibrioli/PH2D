//! **Arch-gate: TODA ferramenta que aponta para o canvas pergunta pela MALHA — não só o pincel.**
//!
//! ⛔⛔ **A wave de 2026-09-14 curou o Painter e deixou as vizinhas a mapear pelo quad de repouso**
//! (censo de 2026-09-15). O conta-gotas apanhava a cor do texel errado; e as TRÊS entradas de canvas
//! da Remoção de fundo faziam algo **pior que o afim**: montavam uma CAIXA ALINHADA AOS EIXOS a
//! partir de `translation ± size/2`, cega à **rotação**, à pose do **PAI** (ela lia o `Transform`
//! LOCAL) e à **malha**. *A mesma conta, escrita três vezes, errada nas três.*
//!
//! ⚠️ **Este gate é uma FAMÍLIA, não um sítio** — é essa a forma que a wave anterior não tinha: ela
//! gateou a porta que curou e nenhuma sonda perguntou *«quem MAIS resolve um ponteiro de canvas?»*.
//! Aqui a resposta é contada nos ficheiros, com piso de população e com a lei ANTIGA proibida pelo
//! nome, que é o que impede a recaída silenciosa.

use std::path::Path;

/// Um ficheiro do repo, com os comentários retirados (uma agulha não se satisfaz com prosa).
fn fonte(rel: &str) -> String {
    let f = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join(rel);
    std::fs::read_to_string(&f)
        .unwrap_or_else(|e| panic!("{}: {e}", f.display()))
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O CONTA-GOTAS do Painter lê o texel que a arte DESENHA ali.**
#[test]
fn the_eyedropper_samples_the_texel_the_art_draws_there() {
    let rel = "shells/desktop/src/forwarding_picker.rs";
    let src = fonte(rel);
    // Controlo positivo: é ESTE o sítio que amostra a composição do Painter.
    assert!(
        src.contains("painter.sample_composite_at_uv(su, sv)"),
        "{rel} deixou de amostrar a composição — este gate perdeu o sujeito"
    );
    assert!(
        src.contains("let malha = ph2d_render::mesh_uv("),
        "{rel} amostra pelo afim do QUAD DE REPOUSO: numa arte dobrada o conta-gotas devolve a cor \
         de outro sítio. A lei é a `ph2d_render::mesh_uv`, e `MeshUv::Quad` deixa o caminho de \
         sempre (a grelha da folha, o *Repeat Image*) intocado."
    );
    // ⚠️ **Citar a porta não é consultá-la** — a mutação que esta metade mata é o `let _ = mesh_uv(..)`
    // ao lado do afim de sempre (ela SOBREVIVEU à 1.ª redacção do gate irmão, no pincel).
    assert!(
        src.contains("ph2d_render::MeshUv::Use { u, v, .. } => (u, v)"),
        "{rel} pergunta à malha e DEITA FORA a UV que ela devolve."
    );
    // ⚠️ E um clique FORA da arte desenhada não amostra o quad por baixo: cai para a leitura do ecrã.
    assert!(
        src.contains("if malha == ph2d_render::MeshUv::Refuse {"),
        "{rel} aceita a recusa da porta como se fosse uma UV: um clique fora da arte dobrada \
         passaria a devolver a cor do quad de repouso, que não está no ecrã."
    );
}

/// ⭐⭐⭐ **AS TRÊS ENTRADAS DA REMOÇÃO DE FUNDO passam por UMA porta, e a caixa MORREU.**
///
/// ⚠️ **O piso de população é o coração deste gate:** se alguém apagar uma das três entradas (ou lhe
/// mudar o nome) a contagem cai e o gate reprova — sem ele, um censo que varre menos lê-se como
/// *«não há mais nada»*, que é a armadilha muda do HOWTO §2.7.
#[test]
fn the_background_remover_resolves_its_pointer_through_one_door() {
    const PORTA: &str = "crates/ph2d-sprite-screen/src/uv_sob_o_ponteiro.rs";
    let porta = fonte(PORTA);
    assert!(
        porta.contains("fn uv_sob_o_ponteiro(")
            && porta.contains("ph2d_render::mesh_uv(")
            && porta.contains("sprite_image_to_screen_affine("),
        "{PORTA} deixou de ser a porta única: ela tem de responder pelos DOIS desenhos — a malha \
         posada e o afim do quad (que é quem desdobra a grelha de uma folha)."
    );
    // ⚠️ Os três estados são o que separa «não é nosso» de «é nosso e está fora da arte»: colapsá-los
    // troca, em silêncio, quem fica com o botão do rato.
    for estado in ["Uv(f32, f32)", "ForaDaArte", "SemSujeito"] {
        assert!(
            porta.contains(estado),
            "{PORTA} perdeu o estado `{estado}` — os três chamadores fazem coisas DIFERENTES com \
             cada um, e um `Option` aqui apaga essa diferença."
        );
    }

    let mut chamadas = 0usize;
    for rel in [
        "shells/desktop/src/input_dispatch/eyedropper.rs",
        "shells/desktop/src/input_dispatch/protect_brush.rs",
    ] {
        let src = fonte(rel);
        chamadas += src
            .matches("ph2d_sprite_screen::uv_sob_o_ponteiro(")
            .count();
        // ⛔⛔ **A LEI ANTIGA, proibida pelo NOME.** A caixa era
        // `camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], …)`, e ela lia a pose LOCAL — uma
        // sprite filha ou rodada já amostrava no sítio errado, antes de haver malha nenhuma.
        assert!(
            !src.contains("world_to_screen("),
            "{rel} voltou a montar a caixa alinhada aos eixos do ponteiro. Ela é cega à rotação, à \
             pose do pai e à malha — a resposta é a porta `uv_sob_o_ponteiro`."
        );
        assert!(
            !src.contains("get::<Transform>("),
            "{rel} voltou a ler a pose LOCAL da sprite: numa sprite FILHA falta a cadeia do pai, e \
             o ponteiro cai fora da pegada dela (o defeito que o Painter pagou em 2026-08-19)."
        );
    }
    assert_eq!(
        chamadas, 3,
        "as entradas de canvas da Remoção de fundo são TRÊS (o conta-gotas, o dab de protecção e o \
         *Add area*) e {chamadas} chamam a porta: uma delas voltou a ter lei própria."
    );
}

/// ⭐⭐⭐ **O CHROME DO CANVAS é pintado onde a arte DESENHA — as duas metades do mesmo controlo.**
///
/// ⛔⛔ **A wave anterior curou o DEDO e deixou o OLHO** (medido 2026-09-15): o
/// `deliver_canvas_pointer` resolve o clique pela malha posada desde 14/09 e os editores de CURVA e
/// de LINHA continuavam a pintar as alças pelo afim do quad de repouso. ⇒ um controlo **desenhado
/// por um mapa e agarrado por outro** — a espécie de controlo morto que o `CLAUDE.md` §5.0 declara
/// que nenhuma sonda deste repo apanha. *As duas direcções viajam juntas ou nenhuma viaja.*
///
/// ⚠️ Este gate afirma que os desenhadores **consultam** a porta; que a porta **responde certo** é
/// medido noutro sítio (`ph2d-app-painter/tests/the_canvas_map_lands_on_the_posed_art.rs`) — *citar
/// uma porta não é consultá-la, e consultá-la não é tê-la certa.*
#[test]
fn the_curve_and_line_chrome_is_painted_where_the_art_draws_it() {
    // A porta.
    const MAPA: &str = "crates/ph2d-app-painter/src/canvas_map.rs";
    let mapa = fonte(MAPA);
    assert!(
        mapa.contains("fn point(&self, p: [f32; 2]) -> Point")
            && mapa.contains("ph2d_render::drawn_mesh_of(present, sim_entity_bits)"),
        "{MAPA} deixou de perguntar à malha posada: sem isso o `CanvasMap` é o afim com outro nome."
    );

    // Os dois editores que autoram pontos em px de IMAGEM e os deixam agarráveis no canvas.
    for rel in [
        "crates/ph2d-app-painter/src/painter_bridge_curve_overlay.rs",
        "crates/ph2d-app-painter/src/painter_bridge_line_overlay.rs",
    ] {
        let src = fonte(rel);
        assert!(
            src.contains("CanvasMap::new("),
            "{rel} pinta as alças pelo afim do QUAD DE REPOUSO. Numa arte dobrada elas ficam longe \
             da tinta — e longe do sítio onde o ponteiro as agarra, que já pergunta à malha."
        );
        assert!(
            src.contains("mapa.point(p)"),
            "{rel} constrói o mapa e continua a mapear pelo afim: *citar a porta não é consultá-la*."
        );
        // ⛔ **A LEI ANTIGA, proibida pelo NOME** — é ela que volta sozinha na primeira alça nova.
        assert!(
            !src.contains("affine * Point::new"),
            "{rel} voltou a mapear um ponto autorado pelo afim do quad."
        );
    }

    // ⭐⭐⭐ **A GRELHA é a peça em que a SUBDIVISÃO decide** (item 4 do dono, 2026-09-15). As alças
    // da curva já chegavam achatadas em muitos pontos; uma linha da rede atravessa o canvas INTEIRO,
    // logo ela é UM segmento — e sobre uma dobra saía recta por cima de arte curva.
    //
    // ⚠️ **As DUAS metades**: consultar a porta **e** pedir-lhe o SEGMENTO. Mapear os dois extremos
    // pelo `point` e ligar com um `line_to` passaria a primeira e desenharia exactamente a recta que
    // esta wave veio tirar.
    const GRELHA: &str = "crates/ph2d-app-painter/src/painter_bridge_grid.rs";
    let grelha = fonte(GRELHA);
    assert!(
        grelha.contains("CanvasMap::new("),
        "{GRELHA} desenha a rede pelo afim do QUAD DE REPOUSO."
    );
    assert!(
        grelha.contains("mapa.segment(de, ate, |p| path.line_to(p))"),
        "{GRELHA} mapeia os extremos e liga-os a direito: sobre uma dobra a linha da rede fica por \
         cima de arte curva. A porta é o `CanvasMap::segment`."
    );
    assert!(
        !grelha.contains("affine * Point::new"),
        "{GRELHA} voltou a mapear um ponto autorado pelo afim do quad."
    );
    // ⚠️ E a porta tem de SUBDIVIDIR de facto — sem isto ela é o `point` com outro nome.
    assert!(
        mapa.contains("fn pedacos(&self, a: [f32; 2], b: [f32; 2]) -> u32")
            && mapa.contains("if self.malha.is_none() {"),
        "{MAPA}: o `segment` deixou de derivar quantos pedaços o desvio pede, ou deixou de devolver \
         UM ponto sobre um quad plano (onde o mapa é afim e partir não move um pixel)."
    );

    // ⭐⭐ **OS CONTORNOS: os selos de operação e o gizmo de selecção.** Os dois desenham a FIGURA
    // de uma forma já pousada, e o contorno do selo é o MESMO que o clique alcança
    // (`stroke_outline`) — pelo quad de repouso, *o que se vê* e *o que se clica* ficam em sítios
    // diferentes sobre arte dobrada.
    //
    // ⚠️ **A CAIXA tem de fechar pela porta também** (`polyline(.., true)`): quem a desenha chama
    // `close_path`, que liga o último canto ao primeiro **a direito** — sem isso três arestas seguem
    // a arte e a quarta corta por cima dela, e as três primeiras convencem o olho.
    for (rel, fechadas) in [
        (
            "crates/ph2d-app-painter/src/painter_bridge_op_badges.rs",
            "mapa.polyline(&b.outline, b.closed)",
        ),
        (
            "crates/ph2d-app-painter/src/painter_bridge_selection_gizmos.rs",
            "mapa.polyline(&g.box_corners, true)",
        ),
    ] {
        let src = fonte(rel);
        assert!(
            src.contains("CanvasMap::new("),
            "{rel} desenha a figura pelo afim do QUAD DE REPOUSO."
        );
        assert!(
            src.contains(fechadas),
            "{rel} deixou de pedir a POLILINHA à porta: mapear canto a canto e ligar a direito \
             desenha exactamente as rectas que esta wave veio tirar."
        );
        assert!(
            !src.contains("affine * Point::new"),
            "{rel} voltou a mapear um ponto autorado pelo afim do quad."
        );
    }

    // ⚠️ E o GIZMO de transformação é o mesmo controlo: se ele ficar no afim, a caixa afasta-se das
    // alças que ela enquadra (e das alças que o dedo agarra).
    const GIZMO: &str = "crates/ph2d-app-painter/src/painter_bridge_gizmo.rs";
    let gizmo = fonte(GIZMO);
    assert_eq!(
        gizmo
            .matches("mapa: &crate::canvas_map::CanvasMap<'_>")
            .count(),
        2,
        "{GIZMO} tem DOIS desenhadores do gizmo de transformação (a caixa e a alça do centro) e os \
         dois têm de receber o MAPA: metade no afim põe a caixa longe do que ela enquadra."
    );

    // E o DEDO do menu de alça: o botão secundário sobre o mesmo ponto de controlo.
    const MENU: &str = "shells/desktop/src/input_dispatch/painter_curve_input.rs";
    let menu = fonte(MENU);
    assert!(
        menu.contains("let malha = super::painter_canvas_input::malha_sob_o_cursor(")
            && menu.contains(
                "ph2d_render::MeshUv::Use { u, v, .. } => [u * iw as f32, v * ih as f32]"
            ),
        "{MENU} abre o menu da alça pelo afim do quad, enquanto o botão PRIMÁRIO que arrasta a mesma \
         alça resolve pela malha: as duas metades do mesmo gesto com mapas diferentes."
    );
}

/// ⭐⭐⭐ **A TINTA DA MÁSCARA segue a arte — ela saiu do Vello para o passe de SPRITES.**
///
/// ⛔⛔ **Sem isto o resto desta wave tornava o removedor de fundo PIOR:** curado o ponteiro, a
/// máscara passa a ser pintada no texel certo — e uma tinta desenhada pelo afim do quad de repouso
/// mostra-a num sítio onde não está nem o dedo nem a arte. *Meia cura é pior que nenhuma quando as
/// duas metades concordavam por acidente.*
///
/// ⛔ **A alternativa foi MEDIDA e REFUTADA** (`skin_pieces_gpu_cost`, 2026-09-15): um recorte do
/// Vello por triângulo deixa costuras numa arte translúcida (`10 580` px fora da barra com `216`
/// peças; `41 732` com `3 456`) e dilatá-los **piora** o caso translúcido (`55 978`, pior `124`);
/// e acima disso os buffers do Vello são FIXOS e o que os estoura degrada em SILÊNCIO — que foi o
/// *«Smooth bugado quebrando a forma»* deste mesmo módulo.
#[test]
fn the_protection_tint_rides_the_sprite_pass_with_the_art_mesh() {
    const GPU: &str = "shells/desktop/src/render_loop/bgremoval_preview_gpu.rs";
    let src = fonte(GPU);
    // ⛔ A LEI ANTIGA proibida pelo nome: a tinta era um `draw_image_rgba_transformed` com o afim.
    assert!(
        !src.contains("draw_image_rgba_transformed"),
        "{GPU} voltou a desenhar a tinta da máscara pelo Vello com o afim do quad de repouso: numa \
         arte dobrada ela e a prévia que ela anota aparecem em sítios diferentes."
    );
    // ⚠️⚠️ **A agulha nomeia a LEI, nunca a VISIBILIDADE.** A 1.ª redacção deste gate pedia
    // `pub(super) fn tint_instances(` e ficou VERMELHA no mesmo dia, quando a função passou a
    // privada ao cortar o `dispatch` por responsabilidade — sem uma linha de comportamento mudar.
    // *É a quinta vez que este repo paga a mesma forma: um `pub` não é uma propriedade do produto.*
    assert!(
        src.contains("fn tint_instances(")
            && src.contains("ph2d_render::drawn_instance_of(present, gpu.entity_bits)")
            && src.contains("out.push(inst, malha)"),
        "{GPU} deixou de emitir a tinta como instância do passe de sprites COM a malha da arte."
    );
    // ⚠️ **O `sub_order` é o que a põe POR CIMA** — a chave de ordenação desempata por
    // `texture_id`, e o da ranhura da tinta tanto pode ser maior como menor que o da arte.
    assert!(
        src.contains("inst.sub_order = inst.sub_order.saturating_add(1);"),
        "{GPU} emite a tinta sem a pôr à frente da arte no MESMO fundo: dependendo da ordem em que \
         as ranhuras de textura foram pedidas, ela desaparece POR BAIXO dela."
    );

    // E o passe tem de a RECEBER — uma instância emitida que ninguém desenha é a metade muda.
    const PRESENT: &str = "shells/desktop/src/render_loop/present.rs";
    let present = fonte(PRESENT);
    assert!(
        present.contains("bgremoval_tint: &ph2d_render::LiftedInstances")
            && present.contains("for (i, inst) in bgremoval_tint.instances().iter().enumerate()"),
        "{PRESENT} não junta a tinta ao slot `extra` do passe de sprites: ela é produzida e nunca \
         desenhada."
    );
    // ⚠️ E o atalho «não copiar nada» tem de contar com ela: se ele só perguntar pelo Motion, um
    // quadro com fantasmas E tinta deita a tinta fora em silêncio.
    assert!(
        present
            .contains("let so_fantasmas = motion_slice.is_empty() && bgremoval_tint.is_empty();"),
        "{PRESENT} decide o atalho do slot `extra` sem contar com a tinta — e o atalho DEITA FORA \
         quem ele não conta."
    );
}

/// ⭐⭐⭐ **O CONTA-GOTAS LÊ O ECRÃ, e o ecrã tem DUAS metades.**
///
/// ⛔⛔⛔ **Report do dono, 2026-09-15: *«não funciona de maneira nenhuma e em nenhum lugar, com a
/// arte dobrada ou não — sempre fica com #00000000»*.** A leitura de reserva pedia só a intermédia
/// do Vello, que sobre o canvas é **transparente por construção** (os sprites vivem noutra
/// textura). ⇒ transparente É `#00000000`, e o patch que existia cobria UM canto (o Painter activo
/// sobre a sprite seleccionada). *A wave anterior curou ONDE ele amostra e não SE ele amostra — são
/// dois defeitos, e eu só tinha medido o primeiro.*
///
/// ⚠️ **E a fonte do mundo tem DOIS modos:** num quadro intercalado (ou com o vidro do *Edit
/// Prefab*) o mundo que o ecrã mostra é o ACUMULADOR, não a saída do tonemap. Ler sempre a segunda
/// devolveria a última faixa — certo no documento simples, errado em metade dos outros.
#[test]
fn the_eyedropper_reads_the_screen_and_the_screen_has_two_halves() {
    // ⚠️ **A escolha vive em DOIS ficheiros, e é de propósito** (corte por responsabilidade de
    // 2026-09-15): o EVENTO é drenado no `forwarding.rs`, e o que ele pede à shell — a amostra
    // autorada e o diálogo de paleta — mora no irmão. *Mover código parte gates, e este é o barato:
    // o que falha ALTO.*
    const F: &str = "shells/desktop/src/forwarding.rs";
    let src = fonte(F);
    // Controlo positivo: é ESTE o sítio que drena a escolha do conta-gotas.
    assert!(
        src.contains("WidgetEvent::EyedropperPick { parent, px, py }"),
        "{F} deixou de tratar a escolha do conta-gotas — este gate perdeu o sujeito"
    );
    assert!(
        src.contains("ph2d_render::screen_color("),
        "{F} voltou a ler UMA camada: sobre o canvas ela é transparente, e o artista recebe \
         `#00000000` em quase todo o ecrã."
    );
    assert!(
        src.contains("ph2d_render::world_source("),
        "{F} fixa a fonte do mundo em vez de perguntar qual delas o COMPOSITOR está a ler: num \
         quadro intercalado o conta-gotas devolve a última faixa em vez do que está no ecrã."
    );
    // ⚠️ E a metade AUTORADA fica: o ecrã passa pelo tonemap e pelo dither da descida, logo
    // escolher uma cor acabada de pintar e recebê-la com `±1` por canal não fecha o round-trip.
    const AUTORADA: &str = "shells/desktop/src/forwarding_picker.rs";
    assert!(
        fonte(AUTORADA).contains("painter.sample_composite_at_uv(su, sv)"),
        "{AUTORADA} perdeu a amostra AUTORADA do Painter: a leitura do ecrã responde em todo o lado, mas \
         só esta devolve exactamente a cor que o artista pousou."
    );
}
