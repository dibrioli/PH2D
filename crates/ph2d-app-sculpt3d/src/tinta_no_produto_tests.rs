//! ⭐⭐⭐⭐ **A TINTA FINA PELO CAMINHO QUE O ARTISTA PERCORRE** — pen-down,
//! movimentos, pen-up e o QUADRO, com um adaptador a sério.
//!
//! ⛔⛔ **Porque este ficheiro existe:** os gates puros da
//! [`crate::tinta_da_peca`] provam a lei do plano e os da
//! [`ph2d_sculpt3d::tinta_fina`] provam a lei do dab — e o report do dono de
//! 2026-09-20 (*«traços posteriores estão reduzindo a resolução dos traços em
//! alta resolução anteriores; como se voltasse para o modo mesh»*) vivia
//! **entre os dois**, no despacho do gesto e na reconciliação do quadro. *Um
//! recurso de arrastar prova-se pelo gesto, não pela função que o gesto chama.*

use ph2d_sculpt3d::Verb;

use crate::Sculpt3dScene;

macro_rules! gpu_or_skip {
    () => {
        match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("sem adaptador nesta máquina — nada a afirmar");
                return;
            }
        }
    };
}

/// A shell mínima que as portas de ponteiro perguntam (o molde do
/// [`crate::undo_plano_tests`]).
struct HostDeTeste {
    ponteiro: (f32, f32),
}
impl ph2d_app_host::AppHost for HostDeTeste {
    fn pointer(&self) -> (f32, f32) {
        self.ponteiro
    }
    fn mods(&self) -> ph2d_app_host::HostMods {
        ph2d_app_host::HostMods {
            shift: false,
            control: false,
            alt: false,
            super_key: false,
        }
    }
    fn pointer_over_chrome(&self, _x: f32, _y: f32) -> bool {
        false
    }
    fn modal_takes_the_pointer(&self) -> bool {
        false
    }
    fn note_authored_change(&mut self) {}
}

/// A cena `=52` armada como o roteiro a entrega, mais o degrau da lição.
fn cena_52(device: &wgpu::Device) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, crate::scenes::tinta_fina::peca(), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    ph2d_panel_sculpt3d::state::switch_verb_parts(
        &mut s.verb_slots,
        &mut s.brush,
        &mut s.radius_px,
        Verb::Paint,
    );
    s.brush.color = [1.0, 0.0, 0.0];
    s.tinta_nivel = crate::scenes::tinta_fina::DEGRAU_DA_LICAO.nivel();
    s
}

/// Um traço pelo despacho REAL, a partir de `x0`.
fn traco(s: &mut Sculpt3dScene, x0: f32) -> bool {
    let mut host = HostDeTeste {
        ponteiro: (x0, 350.0),
    };
    let ok = crate::input_down::pointer_down(&mut host, s, winit::event::MouseButton::Left);
    for k in 1..=8u8 {
        crate::input::pointer_move(s, x0 + 6.0 * f32::from(k), 350.0);
    }
    crate::input::pointer_up(s);
    ok
}

fn topologia(s: &Sculpt3dScene) -> (usize, usize) {
    (s.mesh().vert_count(), s.mesh().faces().len())
}

fn amostras(s: &Sculpt3dScene) -> Vec<[f32; 3]> {
    s.objects[s.active]
        .tinta
        .as_ref()
        .map(|t| t.amostras().to_vec())
        .unwrap_or_default()
}

/// ⭐⭐⭐⭐ **GATE — UM TRAÇO DE COR NÃO APAGA O DETALHE DOS ANTERIORES, NEM COM
/// A TOPOLOGIA DINÂMICA ARMADA.** O report do dono de 2026-09-20, reproduzido
/// pelo caminho do produto.
///
/// ⛔⛔ **O mecanismo:** com o interruptor ligado cada dab de cor chamava o
/// passe, a contagem de faces mudava, a `concorda_com` passava a `false` e a
/// [`crate::tinta_da_peca::garante`] reconstruía o plano **SEMEADO da cor por
/// vértice** — que é, à letra, *«voltar para o modo mesh»*.
///
/// ⚠️ **O CONTROLO é a metade que o torna uma medição:** um verbo de FORMA no
/// mesmo arranjo **muda** a topologia e o plano **é** reconstruído. Sem ele
/// este gate ficaria verde sobre uma cena em que o passe nunca arma — que é
/// exactamente como ele ficaria verde hoje se alguém desarmasse o interruptor.
#[test]
#[ignore = "precisa de adaptador"]
fn dois_tracos_de_cor_com_dyntopo_nao_perdem_o_detalhe_do_primeiro() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    let (on, _) = s.toggle_dyntopo();
    assert!(on, "a fixtura precisa do interruptor LIGADO");
    s.sync_mesh(&gpu.device, &gpu.queue);
    let topo_0 = topologia(&s);
    assert!(
        !amostras(&s).is_empty(),
        "o plano não nasceu: o degrau da lição não chegou ao produto"
    );

    assert!(traco(&mut s, 400.0), "o pen-down de A não foi da cena");
    s.sync_mesh(&gpu.device, &gpu.queue);
    let depois_de_a = amostras(&s);
    let pintadas: Vec<usize> = depois_de_a
        .iter()
        .enumerate()
        .filter(|(_, c)| **c != [1.0, 1.0, 1.0])
        .map(|(i, _)| i)
        .collect();
    assert!(
        !pintadas.is_empty(),
        "o traço A não pintou uma amostra: a fixtura não contém o fenómeno"
    );

    // ⚠️ **`560` e nao `250`, e o numero foi MEDIDO** (`diag_onde_mora_o_plano`):
    // a peca ocupa `x ∈ [280, 610]` a `y = 350` e o raio e `50 px`, logo o
    // `250` que aqui esteve **ERRAVA A PEÇA** — o traço B nao pintava nada e
    // este gate media dois traços de que so um existia. ⛔ E nao pode
    // sobrepor A (centros `400..448`, pegada `[350, 498]`), senao o
    // `mudou == 0` la em baixo reprova sobre produto CERTO: `560` cobre
    // `[510, 658]`, do outro lado da peca.
    assert!(traco(&mut s, 560.0), "o pen-down de B não foi da cena");
    s.sync_mesh(&gpu.device, &gpu.queue);
    let depois_de_b = amostras(&s);

    assert_eq!(
        topologia(&s),
        topo_0,
        "a topologia mudou debaixo do plano durante DOIS traços de COR -- é o \
         report de 20/09, e o plano é paramétrico nas FACES"
    );
    assert_eq!(
        depois_de_a.len(),
        depois_de_b.len(),
        "o plano foi RECONSTRUÍDO entre os dois traços"
    );
    let mudou = pintadas
        .iter()
        .filter(|i| depois_de_a[**i] != depois_de_b[**i])
        .count();
    assert_eq!(
        mudou,
        0,
        "das {} amostras que o traço A pintou, {mudou} mudaram por causa do \
         traço B -- o detalhe fino do primeiro foi reescrito",
        pintadas.len()
    );

    // ⭐⭐ **O CONTROLO: um verbo de FORMA no mesmo arranjo MUDA a topologia.**
    ph2d_panel_sculpt3d::state::switch_verb_parts(
        &mut s.verb_slots,
        &mut s.brush,
        &mut s.radius_px,
        Verb::Draw,
    );
    assert!(
        traco(&mut s, 400.0),
        "o pen-down do controlo não foi da cena"
    );
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_ne!(
        topologia(&s),
        topo_0,
        "CONTROLO: um verbo de FORMA com o interruptor ligado não mexeu na \
         topologia -- a fixtura não arma o passe, e o gate acima afirma nada"
    );
}

/// ⭐⭐⭐ **GATE — LIGAR A TOPOLOGIA DINÂMICA NÃO TRIANGULA A PEÇA ENQUANTO A
/// TINTA FINA ESTÁ ARMADA.**
///
/// ⛔ Triangular parte cada quad em duas faces, logo a contagem muda e o plano
/// é reconstruído — *o artista perdia o detalhe fino no gesto de carregar num
/// interruptor*. A triangulação **não desapareceu**: ela mudou-se para o
/// pen-down do gesto que de facto vai mexer na topologia.
///
/// ⚠️ **O CONTROLO é a mesma peça SEM plano**, onde o interruptor continua a
/// triangular ao ligar — senão esta cura leria como *«a triangulação morreu»*.
#[test]
#[ignore = "precisa de adaptador"]
fn o_interruptor_nao_triangula_a_peca_com_a_tinta_fina_armada() {
    let gpu = gpu_or_skip!();

    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert!(
        s.tinta_fina_armada(),
        "a fixtura precisa do plano armado antes do interruptor"
    );
    let faces = s.mesh().faces().len();
    let quads = s
        .mesh()
        .faces()
        .iter()
        .filter(|f| f.verts().len() > 3)
        .count();
    assert!(quads > 0, "a fixtura não tem quads: nada a triangular");
    let (on, trianguladas) = s.toggle_dyntopo();
    assert!(on);
    assert_eq!(
        (trianguladas, s.mesh().faces().len()),
        (0, faces),
        "o interruptor triangulou a peça com o plano armado -- o detalhe fino \
         morre no gesto de o ligar"
    );

    // ⭐ CONTROLO — sem plano, ele continua a triangular.
    let mut sem = cena_52(&gpu.device);
    sem.tinta_nivel = None;
    sem.sync_mesh(&gpu.device, &gpu.queue);
    assert!(!sem.tinta_fina_armada(), "o CONTROLO não devia ter plano");
    let (on, trianguladas) = sem.toggle_dyntopo();
    assert!(on);
    assert!(
        trianguladas > 0,
        "CONTROLO: o interruptor deixou de triangular também sem plano -- a \
         cura apagou a lei em vez de a condicionar"
    );
}

/// ⭐⭐⭐⭐ **GATE — UM CLIQUE FORA DA PEÇA NÃO LEVA O PLANO COM ELE.**
///
/// ⛔⛔ **O defeito que isto impede foi o que sobrou do report de 21/09 depois
/// da cura da premissa, e é MAIOR do que ela:** o pen-down empresta o plano à
/// peça por um `take` **antes** de saber se o gesto vai pegar (o primeiro dab
/// precisa dele). Se o raio ERRA o modelo, o gesto vira `Drag::Orbit` — e o
/// `close_stroke`, que é quem devolve, só corre no pen-up de um
/// `Drag::Sculpt`/`Filter` ⇒ **o plano morria dentro do traço abandonado**, a
/// peça ficava sem ele, e o `garante` do quadro seguinte reconstruía-o
/// **semeado da cor por vértice** — *«voltou para o modo mesh»*, à letra.
///
/// ⚠️⚠️ **E ele NÃO precisa da topologia dinâmica** — é por isso que este gate
/// não a liga: basta UM clique no vazio, que é o gesto que o próprio
/// `input_down` chama de *«o mais comum do mundo»*.
///
/// ⭐ **O CONTROLO é a primeira metade:** um clique que ACERTA deixa a peça com
/// o plano. Sem ele, um produto que nunca empresta nada passaria aqui.
#[test]
#[ignore = "precisa de adaptador"]
fn um_pen_down_que_erra_a_peca_nao_fica_com_o_plano() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let nasceu = amostras(&s).len();
    assert!(nasceu > 0, "o plano não nasceu: a fixtura não tem fenómeno");

    // (CONTROLO) um traço que ACERTA devolve o plano à peça.
    assert!(
        traco(&mut s, 400.0),
        "o pen-down do controlo não foi da cena"
    );
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        amostras(&s).len(),
        nasceu,
        "CONTROLO: um traço que acerta tem de devolver o plano à peça"
    );

    // ⛔⛔ **A PREMISSA DESTE GATE MORREU EM 21/09, e a fixtura teve de mudar
    // com ela.** Desde a cura do §10 (*«permita pintar mesmo se [não] tocar um
    // vertex»*) um pincel de COR que erra o pen-down **já não é abandonado** —
    // ele abre o traço na mesma. ⇒ *com o `Verb::Paint` da cena esta metade
    // deixaria de conter o fenómeno e o gate ficaria VERDE a afirmar nada*
    // (a mutação `M25`, que apaga o `close_stroke` do braço da recusa, passaria
    // a sobreviver ao comportamento). A população que ainda ORBITA é a dos
    // verbos de FORMA, e é nela que o vazamento se mede.
    ph2d_panel_sculpt3d::state::switch_verb_parts(
        &mut s.verb_slots,
        &mut s.brush,
        &mut s.radius_px,
        Verb::Draw,
    );
    // ⚠️ `120` está FORA da peça, e o número é medido: ela ocupa `x ∈ [280, 610]`.
    assert!(
        traco(&mut s, 120.0),
        "o pen-down no vazio tem de ser consumido (ele vira ÓRBITA)"
    );
    assert!(
        s.stroke.tinta_fina.is_none(),
        "o traço ABANDONADO ficou com o plano dentro -- é o report de 21/09: \
         o pen-down empresta por um `take` e só o `close_stroke` devolve"
    );
    assert_eq!(
        amostras(&s).len(),
        nasceu,
        "a peça perdeu o plano por causa de um clique FORA dela"
    );

    // E o quadro seguinte não o reconstrói semeado (= não voltou ao modo mesh).
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        amostras(&s).len(),
        nasceu,
        "o quadro a seguir ao clique no vazio reconstruiu o plano"
    );
}

/// Um gesto pelo despacho REAL: pen-down em `x0`, `n` passos ate' `x1`, pen-up.
fn gesto(s: &mut Sculpt3dScene, x0: f32, x1: f32, n: u16) -> bool {
    let mut host = HostDeTeste {
        ponteiro: (x0, 350.0),
    };
    let ok = crate::input_down::pointer_down(&mut host, s, winit::event::MouseButton::Left);
    for k in 1..=n {
        let t = f32::from(k) / f32::from(n);
        crate::input::pointer_move(s, x0 + (x1 - x0) * t, 350.0);
    }
    crate::input::pointer_up(s);
    ok
}

/// ⭐⭐⭐⭐ **GATE — UM TRAÇO DE COR QUE COMEÇA FORA DA PEÇA PINTA.**
///
/// ⛔⛔ **Report do dono (21/09, o SEGUNDO sobre o mesmo sintoma):** *«se começar
/// a pintar sem tocar um vertex acontece mais vezes de sumir a pintura»*.
/// Medido, o que sumia **não era a tinta de antes** — o plano ficou em `47 106`
/// amostras o tempo todo. Era **o traço que ele acabara de fazer**: o
/// `sculpt_at` do pen-down errava a peça, o gesto inteiro virava `Drag::Orbit`,
/// e o dedo entrava na peça sem traço nenhum aberto para carimbar.
///
/// ⚠️⚠️ **A régua é a CONTAGEM DE AMOSTRAS PINTADAS, e é ela que os gates
/// irmãos não tinham:** o [`um_pen_down_que_erra_a_peca_nao_fica_com_o_plano`]
/// mede a sobrevivência do PLANO — e o plano sobrevivia, logo ele ficava VERDE
/// sobre isto —, e o [`dois_tracos_de_cor_com_dyntopo_nao_perdem_o_detalhe_do_primeiro`]
/// começa os dois traços DENTRO da peça, logo a fixtura dele **não contém o
/// fenómeno**. *A régua vizinha, pela terceira vez nesta queixa.*
///
/// ⭐ **O CONTROLO é a segunda metade, e sem ele esta cura seria uma
/// REGRESSÃO:** um verbo de FORMA que erra a peça **continua a orbitar** e não
/// move um vértice — é a afordância que a referência define (*arrastar no
/// vazio*), e a troca do dono só é barata porque ela fica intacta onde foi
/// medida.
#[test]
#[ignore = "precisa de adaptador"]
fn um_traco_de_cor_que_comeca_fora_da_peca_pinta() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let pintadas = |s: &Sculpt3dScene| {
        amostras(s)
            .iter()
            .filter(|c| **c != [1.0, 1.0, 1.0])
            .count()
    };
    assert_eq!(pintadas(&s), 0, "a fixtura tem de começar por pintar");

    // ⚠️ `150` está FORA da peça (ela ocupa `x ∈ [280, 610]`, medido) e `480`
    // está dentro: é o gesto do report, à letra.
    assert!(
        gesto(&mut s, 150.0, 480.0, 20),
        "o pen-down não foi da cena"
    );
    s.sync_mesh(&gpu.device, &gpu.queue);
    let n = pintadas(&s);
    assert!(
        n > 0,
        "o traço COMEÇOU FORA da peça e não pintou uma única amostra -- é o \
         report de 21/09: o gesto virou ÓRBITA e o dedo entrou na peça sem \
         traço nenhum aberto"
    );

    // ⭐ CONTROLO: um verbo de FORMA que erra a peça continua a ORBITAR.
    ph2d_panel_sculpt3d::state::switch_verb_parts(
        &mut s.verb_slots,
        &mut s.brush,
        &mut s.radius_px,
        Verb::Draw,
    );
    let antes: Vec<[f32; 3]> = s.mesh().positions().to_vec();
    assert!(
        gesto(&mut s, 150.0, 480.0, 20),
        "o pen-down do controlo não foi da cena"
    );
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        s.mesh().positions(),
        &antes[..],
        "CONTROLO: um verbo de FORMA que começa fora da peça esculpiu -- a \
         afordância «arrastar no vazio = órbita» é da referência e tem de ficar \
         intacta para quem muda a FORMA"
    );
}

/// Quantas amostras estão pintadas (≠ branco) no plano da peça activa.
fn pintadas(s: &Sculpt3dScene) -> usize {
    amostras(s)
        .iter()
        .filter(|c| **c != [1.0, 1.0, 1.0])
        .count()
}

/// Varre o ecrã de 5 em 5 píxeis com UM clique em cada sítio e devolve em
/// quantos deles alguma coisa foi pintada.
fn sitios_que_pintam(gpu: &ph2d_gpu::GpuContext, raio: f32, nivel: Option<u8>) -> (usize, usize) {
    let mut s = cena_52(&gpu.device);
    s.tinta_nivel = nivel;
    s.radius_px = raio;
    s.sync_mesh(&gpu.device, &gpu.queue);
    let (mut acertos, mut sitios) = (0usize, 0usize);
    let mut antes = pintadas(&s);
    let mut x = 300.0f32;
    while x <= 600.0 {
        gesto(&mut s, x, x, 1);
        s.sync_mesh(&gpu.device, &gpu.queue);
        let agora = pintadas(&s);
        sitios += 1;
        if agora > antes {
            acertos += 1;
        }
        antes = agora;
        x += 5.0;
    }
    (acertos, sitios)
}

/// ⭐⭐⭐⭐ **A TINTA FINA NÃO PRECISA DE UM VÉRTICE DEBAIXO DO PINCEL** —
/// report do dono, 21/09: *«a tinta só é depositada se o pincel está sobre um
/// vertex»*.
///
/// ⛔⛔ **A régua é a fracção de SÍTIOS que pintam, e não quanto pintam.** O
/// defeito era um mapa em ILHAS — a tinta caía onde a rede de vértices estava
/// e em mais lado nenhum —, e uma contagem de amostras somada sobre a varredura
/// esconde isso atrás dos sítios que funcionavam.
///
/// ⚠️ **O CONTROLO é o MESMO gesto com a tinta no modo `Mesh`**, onde a cor
/// mora nos vértices e precisar de um é a LEI. Sem ele este gate passaria com
/// um pincel que pinta a peça inteira a cada clique.
#[test]
#[ignore = "precisa de adaptador"]
fn a_tinta_fina_nao_precisa_de_um_vertice_debaixo_do_pincel() {
    let gpu = gpu_or_skip!();
    let fino = crate::scenes::tinta_fina::DEGRAU_DA_LICAO.nivel();
    assert!(fino.is_some(), "premissa: o degrau da lição arma o plano");
    for raio in [6.0f32, 12.0] {
        let (com, n) = sitios_que_pintam(&gpu, raio, fino);
        assert_eq!(
            com,
            n,
            "raio {raio}: a tinta fina deixou {} de {n} sítios sem pintar — \
             é o report de 21/09, e o mapa das que pintam é a rede de vértices",
            n - com
        );
        let (sem, n2) = sitios_que_pintam(&gpu, raio, None);
        assert_eq!(n, n2, "as duas varreduras têm de ter os mesmos sítios");
        assert!(
            sem < com,
            "CONTROLO: no modo `Mesh` a cor mora nos VÉRTICES e precisar de um é \
             a lei — se ele também pinta em {sem} de {n2} sítios, esta régua não \
             mede a tinta fina"
        );
    }
}

/// ⭐⭐⭐⭐ **A FOLHA QUE O OLHO VÊ VALE PARA A AMOSTRA** — a lei que a máscara
/// de alcance aplica ao VÉRTICE desde 2026-09-19, na unidade que a tinta fina
/// escreve.
///
/// ⛔⛔ Antes desta wave ela era **inerte** para a cor fina, e isso está
/// medido: o mesmo traço na barbatana pintava o mesmo número de amostras com a
/// máscara armada e desarmada.
///
/// ⚠️ **O CONTROLO é o raio PEQUENO**: ali a esfera do dab não alcança as
/// costas, logo não há o que cortar e as duas colunas TÊM de ler igual. Sem
/// ele, uma cerca que cortasse por engano em todo lado passaria.
#[test]
#[ignore = "precisa de adaptador"]
fn a_folha_que_o_olho_ve_vale_para_a_amostra() {
    let gpu = gpu_or_skip!();
    let conta = |mascara: bool, raio: f32| -> usize {
        let mut s = Sculpt3dScene::new(&gpu.device, crate::scenes::parede_fina::barbatana(), 1.0);
        s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
        ph2d_panel_sculpt3d::state::switch_verb_parts(
            &mut s.verb_slots,
            &mut s.brush,
            &mut s.radius_px,
            Verb::Paint,
        );
        s.brush.color = [1.0, 0.0, 0.0];
        s.brush.surface_only = mascara;
        s.tinta_nivel = crate::scenes::tinta_fina::DEGRAU_DA_LICAO.nivel();
        s.radius_px = raio;
        s.sync_mesh(&gpu.device, &gpu.queue);
        gesto(&mut s, 420.0, 470.0, 8);
        s.sync_mesh(&gpu.device, &gpu.queue);
        pintadas(&s)
    };
    // ⚠️ A barbatana tem `0,06` de espessura: um pincel GORDO alcança as costas
    // e um FINO não. É essa a fronteira que separa as duas metades do gate.
    let (armada, livre) = (conta(true, 64.0), conta(false, 64.0));
    assert!(livre > 0, "premissa: o traço tem de pintar alguma coisa");
    assert!(
        armada * 10 < livre * 8,
        "a máscara não cortou nada de substancial na tinta fina: {armada} contra \
         {livre} — antes desta wave ela era INERTE aqui, e a lei da folha tem de \
         valer para a AMOSTRA e não só para o vértice"
    );
    let (perto_a, perto_l) = (conta(true, 10.0), conta(false, 10.0));
    assert_eq!(
        perto_a, perto_l,
        "CONTROLO: com o pincel fino a esfera não alcança as costas da folha, \
         logo não há o que cortar e as duas colunas têm de ler igual"
    );
}

/// A tecla, pela porta que a shell chama.
fn tecla(s: &mut Sculpt3dScene, shift: bool) -> bool {
    use winit::keyboard::KeyCode as K;
    let mut req = crate::Sculpt3dRequests::default();
    let factos = crate::keys::keys_delete::DeleteFacts {
        clay_on_screen: true,
        text_focused: false,
        over_panel: false,
        vector_has_selection: false,
    };
    crate::keys::key(
        s,
        &mut req,
        None,
        crate::keys::KeyPress {
            code: K::KeyZ,
            ctrl: true,
            shift,
        },
        &factos,
        "",
    )
}

/// ⭐⭐⭐⭐ **GATE — O `Ctrl+Z` DESFAZ A TINTA FINA, E O `Ctrl+Shift+Z` REFÁ-LA.**
///
/// ⛔⛔ **A medição que o encomendou** (sonda `diag_o_ctrl_z_desfaz_a_tinta_fina`,
/// 21/09): pintar `1010` amostras e carregar em `Ctrl+Z` deixava **`1010`**. A
/// entrada de um traço era a janela de VÉRTICES tocados, e a tinta fina escreve
/// AMOSTRAS.
///
/// ⚠️ **As duas metades, e nenhuma basta:** sem o refazer, um desfazer que
/// pintasse tudo de branco passaria; sem o desfazer não há wave nenhuma. E o
/// refazer tem de devolver o plano **ao bit**, que é o que separa uma troca
/// involutiva de duas leis que por acaso concordam num sentido.
#[test]
#[ignore = "precisa de adaptador"]
fn o_ctrl_z_desfaz_a_tinta_fina() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let virgem = amostras(&s);
    assert_eq!(pintadas(&s), 0, "a peça abre por pintar");

    assert!(
        gesto(&mut s, 380.0, 470.0, 10),
        "o pen-down não foi da cena"
    );
    s.sync_mesh(&gpu.device, &gpu.queue);
    let depois = amostras(&s);
    let n = pintadas(&s);
    assert!(
        n > 100,
        "a fixtura não contém o fenómeno: pintou {n} amostras"
    );

    assert!(tecla(&mut s, false), "o Ctrl+Z tem de ser consumido");
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        amostras(&s),
        virgem,
        "o Ctrl+Z não devolveu o plano de antes do traço — foi {n} contra \
         {} amostras pintadas",
        pintadas(&s)
    );

    assert!(tecla(&mut s, true), "o Ctrl+Shift+Z tem de ser consumido");
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        amostras(&s),
        depois,
        "o refazer não devolveu o plano de depois, ao bit"
    );
}

/// ⭐⭐⭐⭐ **GATE — UM TRAÇO QUE NÃO TOCA UM VÉRTICE DEIXA ENTRADA DE DESFAZER.**
///
/// ⛔⛔ Este é o portão que a wave do vértice deixou a contar a população
/// errada: o `close_stroke` devolvia cedo com `touched()` vazio, e depois da
/// cura de 21/09 um dab fino escreve amostras **sem um único vértice na
/// pegada**. *Sem entrada não há o que desfazer, e o Ctrl+Z gastava o passo
/// ANTERIOR — que é pior do que não fazer nada.*
///
/// ⚠️ **A fixtura sai de uma MEDIÇÃO** (`diag_onde_a_pegada_de_vertices_fica_vazia`):
/// a `raio 6 px` a pegada de vértices lê `0` em `30` dos `31` sítios varridos, e
/// o `x = 400` é um deles. A 1.ª asserção é o CONTROLO de que ela ainda contém
/// o fenómeno — *um raio maior torna este gate verde a medir outra coisa*.
#[test]
#[ignore = "precisa de adaptador"]
fn um_traco_de_cor_sem_vertice_debaixo_do_pincel_deixa_desfazer() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.radius_px = 6.0;
    s.sync_mesh(&gpu.device, &gpu.queue);
    let virgem = amostras(&s);
    let antes = s.undo.len();
    assert!(gesto(&mut s, 400.0, 400.0, 1), "o pen-down não foi da cena");
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert!(pintadas(&s) > 0, "a fixtura não pintou uma amostra");
    assert_eq!(s.undo.len(), antes + 1, "o traço tem de deixar UMA entrada");

    match s.undo.last().map(|e| &e.undo) {
        Some(crate::StrokeUndo::Stroke { verts, finas, .. }) => {
            assert!(
                verts.is_empty(),
                "CONTROLO: a fixtura deixou de conter o fenómeno — a pegada de \
                 vértices tem {} e este gate passaria pelo canal errado",
                verts.len()
            );
            assert!(finas.is_some(), "e a entrada tem de trazer a janela FINA");
        }
        outra => panic!("a entrada não é a de um traço: {}", outra.is_some()),
    }

    assert!(tecla(&mut s, false), "o Ctrl+Z tem de ser consumido");
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        amostras(&s),
        virgem,
        "o Ctrl+Z não devolveu o plano de antes de um traço SEM vértice"
    );
}

/// ⭐ **AS SONDAS vivem no irmão** — ver [`sondas`]. O corte foi por
/// responsabilidade quando o par cruzou o tecto de LOC: *um gate AFIRMA e uma
/// sonda MEDE*, e as que medem imprimem tabelas sem barra nenhuma.
#[path = "tinta_no_produto_sondas.rs"]
mod sondas;
