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

/// O mesmo ficheiro **com os comentários**, para quem mede uma NOTA em vez de uma lei.
fn texto(rel: &str) -> String {
    let f = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join(rel);
    std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()))
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

/// ⭐⭐⭐⭐ **O CENSO É DERIVADO — e foi a falta disto que reprovou o smoke do dono.**
///
/// ⛔⛔⛔ **O gate irmão acima declara-se, por escrito, *«uma FAMÍLIA, não um sítio»* — e era uma
/// LISTA ESCRITA À MÃO.** Ele nomeava seis ficheiros (curva · linha · grelha · selos · gizmo de
/// selecção · gizmo), e **nenhuma sonda perguntava *«quem MAIS pinta um ponto autorado?»***. Medido
/// em 2026-09-15, quando o dono correu a cena do osso e fotografou o contorno da ELIPSE recto por
/// cima de arte dobrada: eram mais **quatro** desenhadores (elipse · polígono · stencil · simetria)
/// mais o gizmo de **Deform**, todos invisíveis ao censo — *e a elipse é exactamente a forma que o
/// roteiro do smoke manda arrastar*.
///
/// ⚠️⚠️ **É a MESMA armadilha que o gate irmão diz ter curado, um nível acima:** lá o piso de
/// população impede que um censo que varre MENOS se leia como *«não há mais nada»*; aqui o censo
/// não varria nada — ele recitava. *Uma lista escrita à mão não tem população para ter piso.*
///
/// ⇒ este varre **todos** os `painter_bridge*.rs`, proíbe a lei antiga pelo nome, e tem os DOIS
/// pisos (quantos ficheiros existem, e quantos de facto consultam a porta).
#[test]
fn the_canvas_chrome_census_is_derived_and_nobody_maps_an_authored_point_by_the_rest_quad() {
    /// ⛔ **Os isentos, cada um com a razão — e nenhum é «por agora».**
    const ISENTOS: &[(&str, &str)] = &[(
        "painter_bridge_brush_ring.rs",
        "O anel do pincel é ANCORADO NO CURSOR e usa só a parte LINEAR do afim; a forma dele já \
         carrega a curvatura da arte pela pegada do motor (`FootprintCurve`), que é uma cura \
         melhor que este mapa. ⛔ E a célula do Grid Stamp (`draw_grid_cell`) fica no quad de \
         propósito: a metade do DEDO (`grid_cell_under`) inverte o afim, e a porta que inverte \
         PELA MALHA (`ph2d_render::mesh_uv`) exige `&mut World` enquanto todo este caminho de \
         chrome tem `&World`. Curar só o desenho poria um rectângulo bem dobrado à volta da \
         célula ERRADA — *meia lei aplicada é pior que nenhuma*. Item ABERTO, com o bloqueador \
         nomeado: falta um inverso read-only na `DrawnMesh` (hoje ela só tem `world_at_uv`).",
    )];

    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("crates/ph2d-app-painter/src");
    let mut varridos = Vec::new();
    let mut consultam = Vec::new();
    let mut acusados = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("a pasta da família do Painter") {
        let path = entry.expect("entrada legível").path();
        let nome = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if !nome.starts_with("painter_bridge") || !nome.ends_with(".rs") {
            continue;
        }
        varridos.push(nome.clone());
        let src = std::fs::read_to_string(&path)
            .expect("ficheiro legível")
            .lines()
            .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
            .collect::<Vec<_>>()
            .join("\n");
        // ⚠️ **CONSULTAR, não CONSTRUIR.** A 1.ª redacção contava `CanvasMap::new(` e leu `7` onde
        // havia `8`: o `painter_bridge_gizmo.rs` **recebe** o mapa por parâmetro em vez de o montar.
        // *Uma agulha que nomeia o construtor mede quem MONTA, e a lei é sobre quem CONSULTA.*
        if src.contains("CanvasMap") {
            consultam.push(nome.clone());
        }
        // ⛔ **A LEI ANTIGA, proibida pelo NOME** — mapear um ponto autorado (px de IMAGEM) pelo
        // afim do quad de repouso. É ela que volta sozinha no primeiro desenhador novo.
        let mapeia_pelo_afim =
            src.contains("affine * Point::new") || src.contains("affine * ph2d_vector::Point::new");
        if mapeia_pelo_afim && !ISENTOS.iter().any(|(f, _)| *f == nome) {
            acusados.push(nome.clone());
        }
    }

    // ⚠️⚠️ **PISO 1 — a população varrida.** Um censo por prefixo que passe a varrer zero fica
    // trivialmente verde (HOWTO §2.7, o modo de falha MUDO). Este número é CONTADO hoje: 18.
    assert!(
        varridos.len() >= 16,
        "o censo varreu só {} ficheiros `painter_bridge*.rs`: ele deixou de ter sujeito, e um \
         `acusados.is_empty()` sobre uma lista vazia é trivialmente verdadeiro",
        varridos.len()
    );
    // ⚠️⚠️ **PISO 2 — quantos de facto CONSULTAM a porta.** Sem ele, apagar o `CanvasMap` de todos
    // os desenhadores deixaria este gate verde: a lei antiga também teria desaparecido.
    assert!(
        consultam.len() >= 8,
        "só {} desenhadores consultam o `CanvasMap` (eram 8 em 2026-09-15: curva · linha · grelha \
         · selos · gizmo de selecção · gizmo · overlays · gizmo de Deform). Um deles voltou a \
         pintar pelo quad de repouso: {consultam:?}",
        consultam.len()
    );
    assert!(
        acusados.is_empty(),
        "estes desenhadores de chrome do canvas mapeiam um ponto AUTORADO pelo afim do quad de \
         repouso, e sobre arte dobrada eles pintam longe da tinta (e longe de onde o dedo os \
         agarra, que já pergunta à malha): {acusados:?}\n\
         A porta é o `ph2d_app_painter::canvas_map::CanvasMap` — `point` para uma alça, `polyline` \
         para um contorno (com `fechada = true` numa caixa), `segment` para uma guia."
    );
    // ⚠️ **A metade de OBSOLESCÊNCIA da isenção** (CLAUDE.md §5.0: *uma catraca sem censo de
    // obsolescência vira LICENÇA*): um isento que já não estoura tem de sair da lista.
    for (f, _) in ISENTOS {
        assert!(
            varridos.iter().any(|v| v == f),
            "o isento `{f}` já não existe — a lista descreve um ficheiro que morreu"
        );
        let src = std::fs::read_to_string(dir.join(f)).expect("isento legível");
        assert!(
            src.contains("affine * Point::new"),
            "o isento `{f}` já NÃO mapeia nada pelo quad de repouso: a isenção deixou de descrever \
             alguma coisa e tem de ser apagada"
        );
    }
}

/// ⭐⭐⭐ **AS QUATRO FORMAS QUE O SMOKE DO DONO MANDA ARRASTAR pintam-se onde a arte desenha.**
///
/// ⚠️ O censo derivado acima responde *«há mais alguém?»*; este responde *«e o que cada um pede à
/// porta está certo?»* — porque consultar o mapa e continuar a ligar os cantos a direito passaria
/// o primeiro e desenharia exactamente as rectas que esta wave veio tirar.
#[test]
fn the_shape_overlays_ask_the_door_for_the_whole_outline() {
    // ⚠️ **As três formas mudaram de ficheiro no mesmo dia** (o teto de LOC por ficheiro pôs o
    // `painter_bridge_overlays.rs` a `709` de `700`, e a cura é MOVER): este gate falhou ALTO no
    // `include`, que é a espécie barata das três do HOWTO §2.7.
    const FORMAS: &str = "crates/ph2d-app-painter/src/painter_bridge_shape_overlays.rs";
    const OVERLAYS: &str = "crates/ph2d-app-painter/src/painter_bridge_overlays.rs";
    let src = fonte(FORMAS);
    // ⛔ **FECHADAS**: o `stroke_box` fecha o caminho, e sem o troço de fecho a última aresta sai
    // recta enquanto as outras seguem a arte — *as primeiras convencem o olho*.
    assert_eq!(
        src.matches("mapa.polyline(&overlay.perimeter, true)")
            .count(),
        2,
        "{FORMAS}: a elipse e o polígono têm de pedir a POLILINHA FECHADA à porta (eram 2 em \
         2026-09-15). Mapear ponto a ponto devolve o contorno recto que o dono fotografou."
    );
    assert!(
        src.contains("mapa.polyline(&overlay.corners, true)"),
        "{FORMAS}: a caixa do stencil deixou de fechar pela arte."
    );
    // ⛔⛔ **UMA GUIA É UM SEGMENTO, NÃO DUAS PONTAS.** A simetria atravessa o canvas inteiro: com
    // `move_to`/`line_to` sobre dois pontos mapeados ela sai recta por cima da dobra — e ela diz
    // ONDE o motor replica os traços, logo uma guia recta aponta para onde nada é espelhado.
    let pai = fonte(OVERLAYS);
    assert!(
        pai.contains("let pts = mapa.polyline(") && !pai.contains("path.line_to(map(cx"),
        "{OVERLAYS}: a guia de simetria voltou a ligar duas pontas a direito."
    );
    // O gizmo de Deform é o mesmo controlo, e entrou no censo na mesma wave.
    const DEFORM: &str = "crates/ph2d-app-painter/src/painter_bridge_deform_gizmo.rs";
    let deform = fonte(DEFORM);
    assert!(
        deform.contains("CanvasMap::new(")
            && deform.contains("mapa.polyline(&g.box_corners, true)"),
        "{DEFORM}: a caixa do gizmo de Deform deixou de fechar pela arte."
    );
}

/// ⭐⭐⭐ **PINTAR ACHATA A ARTE, e o quadro CONSULTA a porta em vez de a citar.**
///
/// Ordem do dono, 2026-09-15: *«inative a possibilidade de pintar sobre malha deformada por ossos;
/// ao entrar no Painter a imagem deixa a deformação, e ao sair ela retorna»*.
///
/// ⚠️ Que a suspensão FUNCIONE está medido na folha
/// (`ph2d-skeleton-live::skin_suspend_tests`); o que só o quadro pode dizer é que a resposta da
/// porta CHEGA ao produtor da malha — *chamá-la e deitar fora o que ela devolve compila, passa a
/// folha, e o dono continua a pintar sobre arte dobrada*.
///
/// ⛔ **Ele mora AQUI e não na `shells/desktop/tests/`** pela catraca `the_shell_only_shrinks`: um
/// gate na árvore da shell paga o tecto dela como qualquer outro ficheiro, e este lê um ficheiro do
/// repo por caminho — que é exactamente o que o [`fonte`] deste módulo já faz para os irmãos.
#[test]
fn painting_flattens_the_art_and_the_frame_passes_it_through() {
    const FASE: &str = "shells/desktop/src/render_loop/fase_sim_extract.rs";
    let src = fonte(FASE);
    let pergunta = src
        .find("skin_suspend::achata_e_avisa(")
        .unwrap_or_else(|| panic!("{FASE} deixou de perguntar QUEM o Painter esta' a achatar"));
    let produtor = src
        .find("skeleton_skin_image::attach_skin_meshes(")
        .unwrap_or_else(|| panic!("{FASE} deixou de por malha nenhuma"));
    assert!(
        pergunta < produtor,
        "{FASE} pergunta DEPOIS de por a malha: a suspensao chega um quadro atrasada"
    );
    // ⚠️⚠️ **A agulha nomeia a LEI, nunca a FORMATAÇÃO.** A 1.ª redacção casava a lista de
    // argumentos (`"px_por_metro, achatada,"`) e ficou VERMELHA no `cargo fmt` do mesmo dia, sem
    // uma linha de comportamento mudar. A propriedade é: dentro da chamada, a resposta entra.
    let chamada = &src[produtor..];
    let fim = chamada.find(");").unwrap_or(chamada.len());
    assert!(
        chamada[..fim].contains("achatada"),
        "{FASE} pergunta e NAO passa a resposta ao `attach_skin_meshes`: a arte continua deformada \
         por baixo do pincel"
    );
    // ⚠️ E a porta MUDA (`sprite_achatada`) achataria em silencio: o report seguinte seria «a arte saltou».
    assert!(
        !src.contains("skin_suspend::sprite_achatada("),
        "{FASE} chama a porta MUDA: a arte endireita-se sem uma palavra"
    );
}

/// ⭐⭐⭐ **A PORTA DORMENTE DIZ QUE DORME — e a nota MORRE com o achatamento.**
///
/// Decisão do dono, 2026-09-15: perguntado se devia limpar o chrome que segue a arte dobrada, ele
/// escolheu **deixar e marcar**. ⛔ E uma nota que diga *«dormente»* depois de o código acordar é a
/// forma mais cara de mentira deste repo — *uma catraca sem censo de obsolescência vira licença*.
///
/// ⇒ este gate ata as duas coisas nas **DUAS direcções**:
/// - enquanto o quadro ACHATAR (`skin_suspend::achata_e_avisa`), a porta e **todos** os
///   desenhadores que a consomem têm de carregar a marca;
/// - no dia em que o achatamento sair, **nenhum** pode carregá-la.
///
/// ⚠️ **O censo é DERIVADO, nunca uma lista escrita à mão** — é a lição que esta mesma jornada
/// pagou (o gate irmão declarava-se «uma família» e recitava seis nomes, deixando cinco
/// desenhadores planos, entre eles a elipse da foto do dono).
#[test]
fn the_dormant_door_says_so_and_the_note_dies_with_the_flattening() {
    const MARCA: &str = "DORMENTE";
    const PORTA: &str = "crates/ph2d-app-painter/src/canvas_map.rs";
    let achata = fonte("shells/desktop/src/render_loop/fase_sim_extract.rs")
        .contains("skin_suspend::achata_e_avisa(");

    assert_eq!(
        texto(PORTA).contains(MARCA),
        achata,
        "{PORTA}: a marca `{MARCA}` e o achatamento do quadro deixaram de concordar. Com \
         achatamento ela TEM de estar lá (senão o próximo leitor acredita que o chrome segue a \
         dobra); sem achatamento ela tem de SAIR (senão a nota mente ao contrário)."
    );

    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("dois pais")
        .join("crates/ph2d-app-painter/src");
    let (mut consumidores, mut marcados, mut faltam) = (0usize, 0usize, Vec::new());
    for entry in std::fs::read_dir(&dir).expect("a pasta da família do Painter") {
        let path = entry.expect("entrada legível").path();
        let nome = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if !nome.starts_with("painter_bridge") || !nome.ends_with(".rs") {
            continue;
        }
        let rel = format!("crates/ph2d-app-painter/src/{nome}");
        // ⚠️ Quem CONSOME lê-se no código (sem comentários); quem está MARCADO lê-se no texto cru.
        if !fonte(&rel).contains("CanvasMap") {
            continue;
        }
        consumidores += 1;
        if texto(&rel).contains(MARCA) {
            marcados += 1;
        } else {
            faltam.push(nome);
        }
    }
    // ⚠️⚠️ **Piso de população:** um censo que passe a varrer zero fica trivialmente verde. Eram
    // NOVE em 2026-09-15 (curva · linha · grelha · selos · gizmo de selecção · gizmo · overlays ·
    // formas · gizmo de Deform).
    assert!(
        consumidores >= 9,
        "o censo achou só {consumidores} desenhadores a consultar o `CanvasMap`: ele deixou de ter \
         sujeito, e um `faltam.is_empty()` sobre uma lista vazia é trivialmente verdadeiro"
    );
    assert_eq!(
        marcados,
        if achata { consumidores } else { 0 },
        "os desenhadores sem a marca `{MARCA}`: {faltam:?} (achatamento no quadro: {achata}). \
         Cada um deles AFIRMA no cabeçalho que segue a arte dobrada, e hoje isso não acontece \
         debaixo do pincel."
    );
}
