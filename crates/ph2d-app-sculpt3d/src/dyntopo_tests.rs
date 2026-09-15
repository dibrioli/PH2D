//! **QUEM MUDA A TOPOLOGIA EM DYNAMIC TOPOLOGY** — medido no gesto, na cena.
//!
//! Módulo irmão de teste do [`super`] (`#[path]`, `cfg(test)`), no molde do
//! `undo_tests`: um gesto exige uma cena e uma cena exige um device, então estes
//! gates são `#[ignore]` + `gpu_or_skip!`.
//!
//! ```text
//! cargo test -p ph2d-app-sculpt3d --lib dyntopo::tests -- --ignored --nocapture
//! ```
//!
//! # ⚠️ Porque este ficheiro nasce com UM verbo só
//!
//! O report do dono (2026-09-14) é sobre a tabela inteira — *«algumas tools que
//! não deveriam subdividir estão fazendo (como smooth) enquanto algumas que
//! deveriam não estão»* — e a tabela é uma pergunta de **ORÁCULO**
//! (`docs/3D/22`). ⛔ **Uma só célula dela não precisa de alvo nenhum: a
//! MÁSCARA.** Ela pinta um canal por-vértice e não move um único vértice, logo
//! refinar debaixo dela muda a topologia da peça num gesto que não toca na
//! geometria. *Um gesto que não escreve posição não tem porque mudar a
//! topologia* — e essa frase não depende do que outro programa faz.
//!
//! ⇒ este gate afirma **essa** célula, com o controlo positivo ao lado. As
//! outras ficam como estão até o estudo, e o [`ph2d_sculpt3d::Verb`] tem hoje a
//! porta onde elas vão morar.

use ph2d_mesh::shapes::uv_sphere;
use ph2d_sculpt3d::Verb;

use super::Sculpt3dScene;

/// Abre a GPU, ou diz que não há nada a afirmar. (Cópia local dos irmãos: um
/// macro exportado entre módulos de teste seria acoplamento por conveniência.)
macro_rules! gpu_or_skip {
    () => {
        match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

/// O meio do viewport — onde a esfera enquadrada está.
const CENTRE: (f32, f32) = (450.0, 350.0);

/// Uma cena com uma esfera, o verbo pedido, e o dyntopo **ARMADO no detalhe
/// mais fino**.
///
/// ⚠️ **O detalhe mais fino é escolha MEDIDA e não zelo:** um refino que só
/// aparece no extremo fino lê-se como *«não refina»* num corpus grosso — é a
/// armadilha que o plano `docs/3D/22 §4` nomeia para o estudo, e ela vale
/// igualmente aqui.
fn cena_armada(device: &wgpu::Device, verb: Verb) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, uv_sphere(24, 36, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = verb;
    let (ligado, _) = s.toggle_dyntopo();
    assert!(ligado, "o dyntopo tinha de ligar");
    s.dyntopo.detail = 1.0;
    s
}

fn vertices(s: &Sculpt3dScene) -> usize {
    s.objects[s.active].stack.mesh().positions().len()
}

/// Um traço de UM dab no meio da peça, pela sequência do pen-down.
fn um_dab(s: &mut Sculpt3dScene) {
    assert!(s.aim(CENTRE.0, CENTRE.1), "o raio errou a peça enquadrada");
    s.stroke.begin(s.objects[s.active].stack.mesh());
    // ⚠️⚠️ **O PEN-DOWN também abre o traço de topologia, e este arnês não o
    // fazia** (achado em 14/09): é ali que a malha de antes é fotografada — sem
    // ela o `close_stroke` nunca grava a entrada `Remeshed` — e, desde a ordem
    // do dono, é ali que a peça é triangulada para quem corre sem o
    // interruptor. *Um arnês a que falta um passo do produto mede outro
    // programa*, e o que ele media aqui era um pincel sem desfazer.
    s.open_dyntopo_stroke();
    // ⚠️⚠️ **E a superfície de REFERÊNCIA, que é a SEGUNDA metade do pen-down
    // que este arnês não fazia.** A primeira (a foto do desfazer) mordeu horas
    // antes, com o pincel de densidade; esta mordeu com o apagador, que sem ela
    // move **zero** vértices mesmo com a pilha montada. *Um arnês a que falta
    // um passo do produto mede outro programa — e o que ele media aqui era um
    // pincel inerte.*
    s.open_reference_stroke();
    assert!(
        s.sculpt_at(CENTRE.0, CENTRE.1),
        "o dab não pegou a malha: a fixtura não contém o fenómeno"
    );
    s.close_stroke();
}

/// ⭐⭐⭐ **A MÁSCARA NÃO MUDA A TOPOLOGIA, E O DESENHO MUDA.**
///
/// As duas metades são necessárias: sem o controlo positivo, um `assert_eq!` de
/// contagem ficaria verde num arranjo em que o refino **nunca** dispara (o
/// detalhe grosso, a esfera já fina, o raio errado) e não estaria a afirmar
/// nada. *Uma régua que não vê o fenómeno acontecer não prova que ele não
/// aconteceu.*
#[test]
#[ignore]
fn a_mascara_nao_muda_a_topologia_e_o_desenho_muda() {
    let gpu = gpu_or_skip!();

    let mut mascara = cena_armada(&gpu.device, Verb::Mask);
    let antes = vertices(&mascara);
    um_dab(&mut mascara);
    let depois = vertices(&mascara);
    assert_eq!(
        depois, antes,
        "a MÁSCARA mudou a contagem de vértices ({antes} -> {depois}): ela não \
         move um único vértice, e refinar debaixo dela muda a topologia da peça \
         num gesto que não toca na geometria"
    );

    // ⭐ O controlo: o mesmo arranjo, com um verbo que ESCREVE posição.
    let mut desenho = cena_armada(&gpu.device, Verb::Draw);
    let antes = vertices(&desenho);
    um_dab(&mut desenho);
    let depois = vertices(&desenho);
    assert!(
        depois > antes,
        "o DESENHO não refinou ({antes} -> {depois}) — sem isto a metade de \
         cima não afirma nada: ela ficaria verde sobre um dyntopo inerte"
    );
}

/// ⚠️ **E a máscara continua a MASCARAR** — a cura é sobre a topologia, não
/// sobre o efeito do pincel.
///
/// *Curar um defeito desligando o verbo é a forma mais barata de o esconder*, e
/// esta é a asserção que o impede.
#[test]
#[ignore]
fn a_mascara_continua_a_pintar_o_canal() {
    let gpu = gpu_or_skip!();
    let mut s = cena_armada(&gpu.device, Verb::Mask);
    let antes: f32 = s.objects[s.active]
        .stack
        .mesh()
        .masks()
        .map_or(0.0, |m| m.iter().sum());
    um_dab(&mut s);
    let depois: f32 = s.objects[s.active]
        .stack
        .mesh()
        .masks()
        .map_or(0.0, |m| m.iter().sum());
    assert!(
        depois > antes,
        "a máscara deixou de pintar ({antes:.4} -> {depois:.4})"
    );
}

/// Uma cena com a malha e o detalhe pedidos, e o passe **armado**.
fn cena_com(
    device: &wgpu::Device,
    verb: Verb,
    malha: ph2d_mesh::Mesh,
    detalhe: f32,
    raio_px: f32,
) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, malha, 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = verb;
    // ⚠️⚠️ **O raio do pincel vive em PIXELS DE ECRÃ, e o `Brush::radius` é
    // DERIVADO dele a cada dab** (`armed_brush_on` reescreve-o com
    // `world_radius_for_screen_px`). *Escrever `s.brush.radius` aqui não teria
    // efeito nenhum* — a primeira redacção deste arnês fê-lo e a fixtura não
    // continha o fenómeno, com `3 386 -> 3 386` a ler-se como «o pincel é
    // inerte» quando o que estava inerte era o arnês.
    s.radius_px = raio_px;
    let (ligado, _) = s.toggle_dyntopo();
    assert!(ligado, "o dyntopo tinha de ligar");
    s.dyntopo.detail = detalhe;
    // ⚠️⚠️ **OS DOIS sliders levam o mesmo valor, e é de propósito.** Desde a
    // ordem do dono de 14/09 há um `Detail` da CENA (a topologia dinâmica) e um
    // do PINCEL de densidade; um arnês que só escrevesse um deixaria metade dos
    // gates a medir o **default** do outro. ⭐ *Quem prova que cada verbo lê o
    // SEU é o gate dedicado* (`cada_gesto_le_o_seu_proprio_slider`), que os põe
    // em valores DIFERENTES — aqui eles concordam para que todos os outros
    // gates continuem a medir a densidade que pedem.
    s.brush.density_detail = detalhe;
    s
}

/// **OS GATES DO PINCEL DE DENSIDADE** — ver [`densidade`].
///
/// ⚠️ **O corte é de ASSUNTO e foi forçado pelo tecto de 700 LOC** (o ficheiro
/// chegou a `827` com os três reports do dono de 14/09): aqui ficam o ARNÊS e a
/// cura da **máscara**, lá os gates do pincel que não tem lei por-vértice. ⛔
/// Curado por corte e nunca por uma entrada no `FILE_OVERAGE_OK`.
#[path = "densidade_tests.rs"]
mod densidade;

/// **OS GATES DO APAGADOR DE DESLOCAMENTO** — ver [`apagador`].
///
/// ⚠️ O corte é o dos vizinhos: a **LEI** dele mora na `ph2d-sculpt3d` (sem
/// GPU); aqui fica o que só uma **pilha de multiresolução** a sério pode
/// afirmar — a recusa quando não há, e que ele não come a forma.
#[path = "apagador_tests.rs"]
mod apagador;

/// **OS GATES DO ESFREGÃO DE DESLOCAMENTO** — ver [`esfregao`].
///
/// ⚠️ Mesmo corte do vizinho, e um passo a mais: o traço deles **ANDA**, porque
/// um arnês de UM dab deixa o arrasto inerte por lei.
#[path = "esfregao_tests.rs"]
mod esfregao;

/// **UM TRAÇO ANCORADO** (`Grip::Hook` / `Grip::Turn`), pela sequência do
/// pen-down que o `input.rs` percorre.
///
/// ⚠️ **Ele existe porque o `um_dab` mede o braço do CARIMBO**, e a metade do
/// report do dono que fala dos que *«deveriam criar subdivisões e não estão»* é
/// precisamente a dos verbos que **não passam por ali**. *Um arnês que só
/// percorre um dos dois caminhos é cego a metade da tabela.*
fn um_traco_ancorado(s: &mut Sculpt3dScene) {
    assert!(s.aim(CENTRE.0, CENTRE.1), "o raio errou a peça enquadrada");
    s.stroke.begin(s.objects[s.active].stack.mesh());
    s.open_dyntopo_stroke();
    s.open_reference_stroke();
    assert!(
        s.take_hold(CENTRE.0, CENTRE.1),
        "o pen-down ancorado não pegou a malha"
    );
    s.stroke_anchor = [CENTRE.0, CENTRE.1];
    let mut prev = [CENTRE.0, CENTRE.1];
    for k in 1..=8 {
        let to = [CENTRE.0 + 9.0 * k as f32, CENTRE.1];
        match s.brush.verb.grip() {
            ph2d_sculpt3d::Grip::Turn(kind) => s.turn_at(kind, to[0], to[1]),
            _ => s.hook_step(prev, to),
        }
        prev = to;
    }
    s.close_stroke();
}

/// ⭐⭐⭐ **O REPORT DO DONO, MEDIDO NOS DOIS SENTIDOS** (2026-09-14):
///
/// > *«algumas tools que não deveriam fazer a subdivisão de polígonos no modo
/// > Dynamic Topology estão fazendo (como smooth) enquanto algumas que deveriam
/// > criar subdivisões com Dynamic Topology não estão criando.»*
///
/// ⭐ **As duas metades foram confirmadas por um oráculo LIVRE** (o SculptGL é
/// **MIT**, logo lê-se e porta-se com atribuição — §0.9: *a triagem pára na
/// primeira porta ABERTA*), e este gate mede que o PRODUTO as honra:
///
/// | verbo | o oráculo | e aqui |
/// |---|---|---|
/// | **Smooth** | não chama a topologia | a contagem **não muda** |
/// | **Snake Hook** | chama-a | a contagem **muda** |
/// | `Draw` | chama-a | **controlo positivo** — sem ele o gate ficaria verde sobre um dyntopo inerte |
/// | `Move` (agarrar) | não chama | **controlo negativo** — sem ele, desligar tudo passaria |
///
/// ⚠️⚠️ **O `Draw` é obrigatório e não é zelo:** um `assert_eq!` de contagem
/// fica trivialmente verde num arranjo em que o refino **nunca** dispara (o
/// detalhe grosso, a esfera já fina, o raio errado). *Uma régua que não vê o
/// fenómeno acontecer não prova que ele não aconteceu* — e este módulo já pagou
/// essa frase seis vezes.
#[test]
#[ignore]
fn o_smooth_deixou_de_subdividir_e_o_gancho_passou_a_subdividir() {
    let gpu = gpu_or_skip!();

    // ⭐ (1) O CONTROLO POSITIVO: o `Draw` refina, senão o arranjo é inerte.
    let mut draw = cena_armada(&gpu.device, Verb::Draw);
    let antes = vertices(&draw);
    um_dab(&mut draw);
    let depois = vertices(&draw);
    assert!(
        depois > antes,
        "o `Draw` não refinou ({antes} -> {depois}): o arranjo não contém o \
         fenómeno, e o resto deste gate mediria vácuo"
    );

    // ⛔ (2) O SMOOTH NÃO MUDA A CONTAGEM — a 1.ª metade do report.
    let mut smooth = cena_armada(&gpu.device, Verb::Smooth);
    let antes = vertices(&smooth);
    let pos_antes: Vec<[f32; 3]> = smooth.mesh().positions().to_vec();
    um_dab(&mut smooth);
    assert_eq!(
        vertices(&smooth),
        antes,
        "o `Smooth` mudou a contagem de vértices — o oráculo livre não chama a \
         topologia dinâmica de lado nenhum do alisador, e o dono nomeou-o à letra"
    );
    // ⭐⭐ **E ELE CONTINUA A ALISAR** — *curar um defeito desligando o verbo é a
    // forma mais barata de o esconder*, e com a contagem igual nos dois casos um
    // `Smooth` inerte passaria a metade de cima sem se mexer.
    let moveu = pos_antes
        .iter()
        .zip(smooth.mesh().positions())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        moveu > 0,
        "o `Smooth` não moveu um único vértice — a contagem igual passou a ser \
         a afirmação trivial de que um pincel morto não muda a topologia"
    );

    // ⭐⭐⭐ (3) O GANCHO PASSOU A MUDAR A CONTAGEM — a 2.ª metade do report, e
    // ela é a que precisou de FIAÇÃO: ele tem âncora, logo não passa pelo braço
    // do carimbo.
    let mut gancho = cena_armada(&gpu.device, Verb::SnakeHook);
    let antes = vertices(&gancho);
    um_traco_ancorado(&mut gancho);
    let depois = vertices(&gancho);
    assert!(
        depois != antes,
        "o `Snake Hook` não mexeu na contagem ({antes} -> {depois}) — ele \
         TRANSPORTA matéria e é dos que mais produzem aresta longa; o oráculo \
         livre chama a topologia dinâmica nele"
    );

    // ⭐ (4) **O EMPURRÃO**, que a referência MEDIDA respondeu (`441 → 2 853`):
    // ele é irmão do gancho e também tem âncora, logo também precisou da
    // fiação.
    let mut empurrao = cena_armada(&gpu.device, Verb::Nudge);
    let antes = vertices(&empurrao);
    um_traco_ancorado(&mut empurrao);
    assert!(
        vertices(&empurrao) != antes,
        "o `Nudge` não mexeu na contagem ({antes} -> {}) — a referência medida \
         diz que ele mexe, e ele entra pela mesma porta que o gancho",
        vertices(&empurrao)
    );

    // ⛔ (5) **A DEMÃO deixou de mexer** — `441 → 441` nos dois extremos do
    // slider na referência medida. Ela é de CARIMBO, logo o caminho por onde ela
    // deixou de refinar é o oposto do dos ancorados: aqui a porta é alcançada e
    // a TABELA é que responde `false`.
    let mut demao = cena_armada(&gpu.device, Verb::Layer);
    let antes = vertices(&demao);
    um_dab(&mut demao);
    assert_eq!(
        vertices(&demao),
        antes,
        "o `Layer` mudou a contagem — a referência medida devolve 441 -> 441 \
         nos dois extremos do slider"
    );

    // ⛔ (6) O CONTROLO NEGATIVO: o agarrar NÃO muda, e o oráculo concorda.
    let mut agarrar = cena_armada(&gpu.device, Verb::Move);
    let antes = vertices(&agarrar);
    um_traco_ancorado(&mut agarrar);
    assert_eq!(
        vertices(&agarrar),
        antes,
        "o `Move` mudou a contagem — sem este lado, ligar a porta a TODO gesto \
         ancorado passaria neste gate"
    );
}
