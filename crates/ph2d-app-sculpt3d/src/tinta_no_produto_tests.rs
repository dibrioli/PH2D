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

/// Sonda: ONDE mora o plano em cada passo de dois traços de cor.
#[test]
#[ignore = "precisa de adaptador"]
fn diag_onde_mora_o_plano_entre_dois_tracos() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    let (on, _) = s.toggle_dyntopo();
    eprintln!("[diag] interruptor armado = {on}");
    s.sync_mesh(&gpu.device, &gpu.queue);
    fn onde(s: &Sculpt3dScene, quando: &str) {
        eprintln!(
            "[diag] {quando:<28} peca={:?} traco={:?} nivel={:?} topo={:?}",
            s.objects[s.active]
                .tinta
                .as_ref()
                .map(|t| t.amostras().len()),
            s.stroke.tinta_fina.as_ref().map(|t| t.tocadas().len()),
            s.tinta_nivel,
            topologia(s),
        );
    }
    onde(&s, "inicio");
    let a = traco(&mut s, 400.0);
    onde(&s, "A: pen-up, pre-sync");
    s.sync_mesh(&gpu.device, &gpu.queue);
    onde(&s, "A: pos-sync");
    let b = traco(&mut s, 250.0);
    onde(&s, "B: pen-up, pre-sync");
    s.sync_mesh(&gpu.device, &gpu.queue);
    onde(&s, "B: pos-sync");
    eprintln!("[diag] pen-down aceite: A={a} B={b}");
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
