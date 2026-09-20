//! ⭐⭐⭐ **A ORDEM das ÂNCORAS DO HUD dentro do quadro** (TOP-20 #20).
//!
//! A `fase_hud` resolve a vista da câmera do jogo, **conduz a pose da raiz** do canvas e guarda a
//! vista no estado da família; o passe das âncoras lê-a bem mais abaixo, no produtor de
//! `LiveGeometry`, para saber contra que rectângulo um filho se cola.
//!
//! ⚠️⚠️ **As duas metades são load-bearing, e por razões DIFERENTES:**
//!
//! 1. a vista tem de ser **GUARDADA antes de LIDA**, senão as âncoras deste quadro usam a do
//!    anterior — o placar ficaria um quadro atrás da janela em todo redimensionamento, que é
//!    exactamente a classe de defeito que esta shell já paga em quatro sítios;
//! 2. a raiz tem de estar **CONDUZIDA antes de o filho ser medido**, porque a caixa de mundo do
//!    filho atravessa a pose da raiz — medi-la antes daria a razão local→mundo do quadro passado.
//!
//! ⚠️ **A lente é o TEXTO EMENDADO do quadro** (`frame_text::render_frame`), nunca um ficheiro: a
//! fase que corre primeiro pode morar no ficheiro que vem depois, e um gate que lesse um `mod.rs`
//! mediria a ordem dos FICHEIROS.

/// **Mutação que deve sangrar:** mover a linha `self.layout_live.vista = …` para antes do
/// `self.fase_hud(camera_rect)`.
#[test]
fn o_hud_conduz_a_raiz_antes_de_as_ancoras_a_lerem() {
    let src = crate::frame_text::render_frame();

    let guarda = src
        .find("self.components.hud.vista = vista;")
        .expect("a `fase_hud` deixou de guardar a vista — as ancoras ficariam sem moldura");
    let le = src
        .find("self.layout_live.vista = self.components.hud.vista;")
        .expect("o passe de layout deixou de receber a vista — as ancoras do HUD ficam INERTES");
    assert!(
        guarda < le,
        "as ancoras leem a vista ANTES de a `fase_hud` a guardar: o placar fica um quadro atras \
         da janela em todo redimensionamento"
    );

    // E a pose da raiz é conduzida no MESMO sítio em que a vista é guardada — as duas saem da
    // mesma linha de código, e é isso que as impede de divergirem.
    let conduz = src
        .find("hud_bridge::drive_canvases(")
        .expect("a raiz do canvas deixou de ser conduzida");
    assert!(
        conduz < guarda,
        "a vista e' guardada antes de conduzir a raiz — o gate quer o valor que DE FACTO a conduziu"
    );
    assert!(
        conduz < le,
        "o filho e' medido antes de a raiz ser conduzida: a razao local->mundo seria a do quadro \
         passado"
    );
}

/// ⛔ **E a vista NÃO é re-lida da câmera no passe das âncoras.**
///
/// ⚠️ Metade NEGATIVA, e é a que guarda a lei que o `fase_frame_open` escreve por extenso: *«uma
/// segunda leitura da câmera seria a segunda resposta a "qual é a vista?", e as duas divergiriam no
/// dia em que uma delas mudasse»*. Sem esta metade, alguém “consertaria” a ordem chamando o
/// `fase_game_camera` outra vez — e o gate de cima ficaria verde sobre duas verdades.
#[test]
fn as_ancoras_nao_fazem_uma_segunda_leitura_da_camera() {
    let src = include_str!("../../src/layout_live_anchors.rs");
    assert!(
        !src.contains("fase_game_camera"),
        "o passe das ancoras chama a camera outra vez — a vista tem UMA resposta por quadro"
    );
    assert!(
        src.contains("self.vista"),
        "o passe deixou de ler a vista guardada"
    );
}

/// ⛔⛔⛔ **A vista do HUD é o ECRÃ DO JOGADOR, nunca a banda do chrome — e isto é uma DECISÃO.**
///
/// # ⚠️⚠️ Este gate existe para tornar a cura ERRADA barulhenta
///
/// O item *«no EDITOR um HUD colado às bordas cai atrás dos painéis»* lê-se como um defeito com uma
/// cura óbvia: alimentar a fase com a **banda** entre os painéis (a `scene_window`, que existe e é
/// a lei de todo mapeamento ecrã↔mundo do chrome desde 2026-09-17). ⛔ **Ela está errada**, e o
/// `todo_aponte_passa_pela_janela_da_cena` já declara porquê na partição
/// `A_JANELA_E_O_ASSUNTO`: *a câmera do JOGO mede o ecrã do jogador, não a banda do chrome*.
///
/// ⭐ **O que o artista vê no editor é onde a peça vai estar PARA QUEM JOGA.** Um HUD enquadrado
/// pela banda ficaria bonito no editor e **mentiria sobre o jogo**: a peça colada ao canto sairia
/// para dentro do ecrã do jogador, e o defeito só apareceria em quem jogasse. *Entre mostrar a
/// verdade parcialmente tapada e mostrar uma mentira inteira, esta casa mostra a verdade* — e a
/// área segura no editor é **decisão de produto**, não a troca da régua.
///
/// **Mutações que devem sangrar:** trocar o `camera_rect` da fase pela `scene_window` · fazer a
/// fase calcular a própria meia-janela em vez de receber a que a câmera devolveu.
#[test]
fn a_vista_do_hud_vem_da_camera_do_jogo_e_nao_da_banda_do_chrome() {
    let fase = include_str!("../../src/render_loop/fase_hud.rs");
    for proibido in ["scene_window", "surface.size()", "aspect_of"] {
        assert!(
            !fase.contains(proibido),
            "a `fase_hud` passou a medir `{proibido}`: ela deixaria de enquadrar o ecra' do \
             JOGADOR e o HUD mentiria sobre o jogo — a area segura no editor e' DECISAO de \
             produto, nunca a troca desta regua"
        );
    }
    assert!(
        fase.contains("camera_rect"),
        "a fase deixou de receber o rectangulo da camera — ela nao tem outra fonte legitima"
    );

    // ⚠️ **A metade que prova que a partição ainda abriga isto** — uma entrada que já não descreve
    // nada é a catraca a virar licença, e a régua larga apanharia a câmera como falso positivo.
    let camera = include_str!("../../src/render_loop/fase_game_camera.rs");
    assert!(
        camera.contains("aspect_of(surface.size())"),
        "a camera do JOGO deixou de medir a janela — se isso foi deliberado, a entrada da \
         particao `A_JANELA_E_O_ASSUNTO` ficou obsoleta e a premissa DESTE gate morreu com ela"
    );
}
