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
