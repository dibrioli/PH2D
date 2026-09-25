//! ⭐⭐⭐ **A TINTA MOLHADA DO PAINTER CONTINUA A ESCORRER NA PEÇA DEPOIS DE
//! LARGAR** (etapa 3), pelo caminho do produto — com a simulação da água a
//! correr na thread dela, em TEMPO REAL: por isso estes gates dormem entre
//! quadros, como o app.
//!
//! ⚠️ Durante a pincelada que escorre o plano de tinta fina está EMPRESTADO ao
//! traço (a peça tem `None`), logo a leitura é [`vivas`] e nunca `amostras`.

use super::super::{amostras, cena_52, tecla};
use super::{painter_vermelho, traco};
use crate::painter_na_malha::quadro;
use ph2d_editor_core::Tool;
use ph2d_tool_painter::{PaintMedia, PainterTool};
use std::time::{Duration, Instant};

/// As amostras de tinta fina ONDE ELAS ESTIVEREM — no traço que as tem
/// emprestadas, ou na peça.
fn vivas(s: &crate::Sculpt3dScene) -> Vec<[f32; 3]> {
    s.stroke
        .tinta_fina
        .as_ref()
        .map_or_else(|| amostras(s), |t| t.tinta().amostras().to_vec())
}

/// Um quadro do app: o relógio anda, o Painter bate, a costura pousa.
fn um_quadro(s: &mut crate::Sculpt3dScene, p: &mut PainterTool) {
    std::thread::sleep(Duration::from_millis(16));
    p.on_tick(16.7);
    quadro(Some(&mut *s), Some(&mut *p));
}

fn molhado() -> PainterTool {
    let mut p = painter_vermelho();
    p.set_paint_media(PaintMedia::WetPaint);
    p
}

/// ⭐⭐⭐ **GATE — a água que escorre depois de largar CHEGA à peça, e quando ela
/// pára a pincelada fecha num passo só de desfazer**, que devolve a peça ao
/// bit. ⚠️ Antes da etapa 3 o traço fechava no pen-up e a tela era limpa — o
/// que limpar a tela faz é MATAR a sessão da água, logo a peça recebia só o
/// depósito e nunca o escorrido.
#[test]
#[ignore = "precisa de adaptador e corre ~20 s em tempo real"]
fn a_tinta_molhada_escorre_na_peca_depois_de_largar_e_fecha_quando_para() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let antes = amostras(&s);
    assert!(!antes.is_empty(), "a cena abre com o plano armado");
    let passos = s.undo.len();
    let mut p = molhado();
    assert!(traco(&mut s, &mut p, 420.0), "o traço molhado");
    assert!(
        s.painter_escorre.is_some() && s.painter_tela.is_some(),
        "o traço fechou no pen-up com a água ainda a correr"
    );
    let no_pen_up = vivas(&s);
    for _ in 0..60 {
        um_quadro(&mut s, &mut p);
    }
    assert_ne!(
        vivas(&s),
        no_pen_up,
        "a água que escorreu depois de largar não chegou à peça"
    );
    assert!(
        s.painter_escorre.is_some(),
        "a pincelada fechou com a água ainda a correr (um traço assenta em ~19 s)"
    );
    let t0 = Instant::now();
    while s.painter_escorre.is_some() && t0.elapsed() < Duration::from_secs(60) {
        um_quadro(&mut s, &mut p);
    }
    assert!(s.painter_escorre.is_none(), "a pincelada nunca fechou");
    assert!(s.painter_tela.is_none(), "fechou sem largar a sessão");
    assert_eq!(
        s.undo.len(),
        passos + 1,
        "o traço e o escorrido são UM passo"
    );
    assert!(
        s.painter_molhada.is_some() && p.screen_canvas_is_wet(),
        "a sessão da água morreu no fecho: o traço seguinte já não se misturaria"
    );
    assert!(tecla(&mut s, false), "o Ctrl+Z tem de ser consumido");
    assert_eq!(amostras(&s), antes, "o Ctrl+Z não devolveu a peça ao bit");
}

/// ⭐⭐⭐ **GATE — um `Ctrl+Z` com a água a escorrer fecha a pincelada PRIMEIRO
/// e desfá-la inteira; a água que continua a correr na tela deixa de chegar à
/// peça.** ⚠️ Sem a porta que fecha, o desfazer tiraria o passo ANTERIOR e o
/// traço aberto continuaria a escrever por cima dele.
#[test]
#[ignore = "precisa de adaptador"]
fn um_ctrl_z_com_a_agua_a_correr_desfaz_o_traco_inteiro() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let antes = amostras(&s);
    let mut p = molhado();
    assert!(traco(&mut s, &mut p, 420.0));
    for _ in 0..20 {
        um_quadro(&mut s, &mut p);
    }
    assert!(
        s.painter_escorre.is_some(),
        "o CONTROLO: a água ainda corre"
    );
    assert!(tecla(&mut s, false), "o Ctrl+Z tem de ser consumido");
    assert!(
        s.painter_escorre.is_none(),
        "o Ctrl+Z não fechou a pincelada"
    );
    assert_eq!(amostras(&s), antes, "o Ctrl+Z não desfez o traço inteiro");
    for _ in 0..30 {
        um_quadro(&mut s, &mut p);
    }
    assert_eq!(
        amostras(&s),
        antes,
        "a água da tela continuou a chegar à peça depois do Ctrl+Z"
    );
}

/// ⭐⭐ **GATE — o `Digital` NÃO fica aberto depois de largar** (o CONTROLO da
/// etapa 3: só a água escorre).
#[test]
#[ignore = "precisa de adaptador"]
fn o_digital_fecha_no_pen_up() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let mut p = painter_vermelho();
    assert!(traco(&mut s, &mut p, 420.0));
    assert!(s.painter_escorre.is_none() && s.painter_tela.is_none());
}

/// ⭐⭐ **GATE — sem tinta fina a água escorre nos VÉRTICES e o traço também
/// fica aberto.** ⚠️ Ali cada pouso sobe o `edits` (a cor por vértice é
/// geometria para o desenho), e a rede de segurança da pincelada que escorre
/// pergunta justamente se o `edits` mudou: sem o pouso a actualizar a marca, o
/// traço fecharia no quadro seguinte ao primeiro pouso por se ler a si mesmo
/// como «outra mão».
#[test]
#[ignore = "precisa de adaptador"]
fn sem_tinta_fina_a_agua_escorre_nos_vertices_e_o_traco_fica_aberto() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.tinta_nivel = None;
    s.sync_mesh(&gpu.device, &gpu.queue);
    // ⚠️ **Um pincel LARGO, de propósito:** a água deixa uma linha de poucos
    // píxeis, e nesta peça grossa ela cai ENTRE as fileiras de vértices — medido,
    // a leitura nos três vértices sob o traço é a do retrato, ao bit. A linha
    // fina é o que a tinta fina existe para receber; aqui mede-se o fio.
    let mut p = molhado();
    p.set_brush_size_px(80.0);
    assert!(traco(&mut s, &mut p, 420.0));
    let e0 = s.edits;
    for _ in 0..60 {
        um_quadro(&mut s, &mut p);
    }
    assert!(s.edits > e0, "o CONTROLO: a água pousou nos vértices");
    assert!(
        s.painter_escorre.is_some(),
        "o traço fechou-se ao ler o próprio pouso como outra mão"
    );
}

/// ⭐⭐ **GATE — outra mão mexe na peça com a água a correr: o traço FECHA antes
/// do pouso seguinte.** É a rede de segurança por baixo das portas que fecham
/// explicitamente (o desfazer, as teclas, o painel, o clique da escultura).
#[test]
#[ignore = "precisa de adaptador"]
fn outra_mao_na_peca_fecha_a_pincelada_que_escorre() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let mut p = molhado();
    assert!(traco(&mut s, &mut p, 420.0));
    um_quadro(&mut s, &mut p);
    assert!(
        s.painter_escorre.is_some(),
        "o CONTROLO: a água ainda corre"
    );
    s.edits += 1;
    um_quadro(&mut s, &mut p);
    assert!(
        s.painter_escorre.is_none() && s.painter_tela.is_none(),
        "a pincelada continuou aberta depois de outra mão mexer na peça"
    );

    // ── E o desfazer chamado DIRECTAMENTE (não pela tecla, que já fecha antes) ──
    // ⚠️ Escrito por uma mutação SOBREVIVENTE (`E9`): o gate do `Ctrl+Z` entra
    // pelo teclado, que fecha a pincelada ANTES de chegar ao desfazer, logo a
    // porta do próprio desfazer era inobservável dali.
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let antes = amostras(&s);
    let mut p = molhado();
    assert!(traco(&mut s, &mut p, 420.0));
    um_quadro(&mut s, &mut p);
    assert!(
        s.painter_escorre.is_some(),
        "o CONTROLO: a água ainda corre"
    );
    assert!(s.undo_stroke(), "havia um passo a desfazer");
    assert!(
        s.painter_escorre.is_none(),
        "o desfazer não fechou a pincelada"
    );
    assert_eq!(
        amostras(&s),
        antes,
        "o desfazer não desfez o traço que escorria"
    );

    // ── E a vista muda de TAMANHO: a tela renasce transparente ──
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let mut p = molhado();
    assert!(traco(&mut s, &mut p, 420.0));
    um_quadro(&mut s, &mut p);
    assert!(
        s.painter_escorre.is_some(),
        "o CONTROLO: a água ainda corre"
    );
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 800.0, 600.0));
    um_quadro(&mut s, &mut p);
    assert!(
        s.painter_escorre.is_none() && s.painter_tela.is_none(),
        "a tela renasceu com a vista e a pincelada continuou aberta sobre ela"
    );
}

/// ⭐⭐⭐ **GATE — trocar a COR do pincel com a água a escorrer NÃO fecha a
/// pincelada**, pelas DUAS caixas de cor que o artista tem à mão: a do painel da
/// escultura (um `SetUi`) e a do Painter (o evento do selector partilhado).
///
/// Report do dono (24/09): *«a simulação seca (para) ao trocar a cor do
/// pincel»*. A 1.ª redacção fechava a pincelada em TODO intent do painel, e um
/// `SetUi` é um AJUSTE — não escreve um vértice. ⚠️ **O CONTROLO vem no fim:** um
/// GESTO do painel (limpar a máscara) continua a fechar, senão este gate ficaria
/// verde sobre uma porta que deixou de fechar tudo.
#[test]
#[ignore = "precisa de adaptador e corre em tempo real"]
fn trocar_a_cor_com_a_agua_a_correr_nao_fecha_a_pincelada() {
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_panel_sculpt3d::Sculpt3dIntent;
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let mut p = molhado();
    assert!(traco(&mut s, &mut p, 420.0), "o traço molhado");
    for _ in 0..5 {
        um_quadro(&mut s, &mut p);
    }
    assert!(
        s.painter_escorre.is_some(),
        "o CONTROLO: a água ainda corre"
    );

    // (1) A caixa de cor do painel da ESCULTURA — um `SetUi`.
    let mut ui = s.panel_snapshot(false, None).ui.clone();
    ui.brush.color = [0.0, 1.0, 0.0];
    let _ = s.apply_panel_intent(Sculpt3dIntent::SetUi(ui));
    assert!(
        s.painter_escorre.is_some(),
        "trocar a cor no painel da escultura fechou a pincelada (o report do dono)"
    );
    // (2) A caixa de cor do PAINTER — o evento que o selector partilhado emite.
    p.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_COLOR_THUMB,
        "0,0,255".to_string(),
    ));
    let depois_das_cores = vivas(&s);
    for _ in 0..30 {
        um_quadro(&mut s, &mut p);
    }
    assert!(
        s.painter_escorre.is_some(),
        "trocar a cor do Painter fechou a pincelada"
    );
    assert_ne!(
        vivas(&s),
        depois_das_cores,
        "a água deixou de chegar à peça depois de trocar a cor"
    );

    // O CONTROLO: um GESTO do painel continua a fechar.
    let _ = s.apply_panel_intent(Sculpt3dIntent::MaskClear);
    assert!(
        s.painter_escorre.is_none(),
        "um gesto do painel (limpar a máscara) deixou de fechar a pincelada"
    );
}

/// ⭐⭐⭐ **GATE — um SEGUNDO traço (com outra cor) não seca a água do primeiro.**
///
/// Report do dono (24/09): *«ao usar a segunda cor, a primeira cor ainda seca e
/// para»*. O pen-down do traço novo fechava a pincelada que escorria SEM guardar a
/// tela molhada, e o traço novo re-semeava a tela — o que MATA a sessão da água.
/// ⚠️ **A régua é a REGIÃO do 1.º traço**, e não o «a água corre?»: o 2.º traço
/// deposita água nova, logo uma sessão recém-nascida responderia «corre» sobre o
/// defeito. O que se mede é que as amostras que o 1.º traço pintou continuam a
/// mudar DEPOIS de o 2.º começar — longe dele (`120 px`).
#[test]
#[ignore = "precisa de adaptador e corre em tempo real"]
fn um_segundo_traco_nao_seca_a_agua_do_primeiro() {
    use ph2d_editor_core::tool::PanelEvent;
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let antes = amostras(&s);
    let mut p = molhado();
    assert!(traco(&mut s, &mut p, 420.0), "o 1.º traço molhado");
    for _ in 0..10 {
        um_quadro(&mut s, &mut p);
    }
    assert!(
        s.painter_escorre.is_some(),
        "o CONTROLO: a água do 1.º corre"
    );
    // As amostras que o 1.º traço (e o que ele já escorreu) pintou.
    let primeiro: Vec<usize> = vivas(&s)
        .iter()
        .zip(&antes)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect();
    assert!(!primeiro.is_empty(), "o 1.º traço não chegou à peça");

    p.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_COLOR_THUMB,
        "0,0,255".to_string(),
    ));
    assert!(traco(&mut s, &mut p, 300.0), "o 2.º traço, com a 2.ª cor");
    let no_pen_up = vivas(&s);
    for _ in 0..40 {
        um_quadro(&mut s, &mut p);
    }
    let depois = vivas(&s);
    let mexeram = primeiro
        .iter()
        .filter(|&&i| depois[i] != no_pen_up[i])
        .count();
    assert!(
        mexeram > 0,
        "a água do 1.º traço parou quando o 2.º começou (o report do dono): \
         0 de {} amostras dele mudaram",
        primeiro.len()
    );
}
