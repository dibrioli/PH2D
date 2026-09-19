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

/// **UM TRAÇO ANCORADO** (`Grip::Hold` / `Grip::Hook` / `Grip::Turn`), pela
/// sequência do pen-down que o `input.rs` percorre.
///
/// ⚠️ **Ele existe porque o `um_dab` mede o braço do CARIMBO**, e a metade do
/// report do dono que fala dos que *«deveriam criar subdivisões e não estão»* é
/// precisamente a dos verbos que **não passam por ali**. *Um arnês que só
/// percorre um dos dois caminhos é cego a metade da tabela.*
///
/// ⛔⛔ **E ELE TINHA O MESMO DEFEITO UM NÍVEL ABAIXO, achado em 14/09:** os
/// três grips ancorados entravam todos pelo `hook_step`, e **o `Grip::Hold` não
/// passa por lá no produto** — ali ele REGISTA (`pending_grab`) e quem carimba
/// é o quadro.
///
/// ⚠️⚠️ **E isto está MEDIDO, não argumentado — pelo contrafactual:** com o
/// arnês de antes **e** o `refine_for_dab` apagado do `grab_at`, este gate fecha
/// **VERDE**. Ou seja, o caminho que o agarrar e o polegar de facto tomam podia
/// ter o fio da topologia desligado sem uma linha vermelha em lado nenhum.
/// ⛔ *Uma mutação só no arnês não sangra, e é exactamente por isso que ela
/// precisa de ser feita: ela não corrige um defeito, torna um defeito
/// OBSERVÁVEL.* — este ficheiro já tinha pago a frase duas vezes, pelo pen-down
/// do desfazer e pelo da superfície de referência.
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
            // ⚠️ **As DUAS linhas do produto**: o evento regista, o quadro
            // drena. Chamar o `grab_at` direto saltaria o `pending_grab`, que é
            // metade do braço do `input.rs`.
            ph2d_sculpt3d::Grip::Hold => {
                s.pending_grab = Some((to[0], to[1]));
                s.flush_pending_grab();
            }
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
///
/// ⭐⭐⭐ **E DEPOIS O DONO CORREU O `=14` E JULGOU CINCO CÉLULAS** (14/09,
/// *«acho que layer, move/drag deve subdividir. Thumb se for possível, deveria
/// subdividir. Twist com dynamic topology fica com resultado muito ruim»*) — o
/// `Layer`, o `Move` e o `Thumb` passaram para o lado que **muda**, e a `Twist`
/// para o que **não muda**:
///
/// | verbo | caminho | e aqui |
/// |---|---|---|
/// | `Layer` | carimbo | a contagem **muda** |
/// | `Move` · `Thumb` | **quem SEGURA** — o fio novo desta wave | a contagem **muda** |
/// | `Twist` | quem gira | **controlo negativo** com o fio LIGADO — quem recusa é a tabela |
/// | `Pose` | quem segura | **controlo negativo** do fio novo — sem ele, responder pelo grip passaria |
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

    // ⭐⭐⭐ (5) **A DEMÃO MEXE — ordem do dono** (*«acho que layer … deve
    // subdividir»*, 14/09). ⚠️ Esta célula esteve nas duas posições no mesmo
    // dia: ela nasceu de manhã a afirmar `441 → 441` da referência medida, e o
    // dono viu-a com os olhos à tarde. *Uma referência responde o que outro
    // programa faz; o dono responde o que este produto tem de fazer.*
    let mut demao = cena_armada(&gpu.device, Verb::Layer);
    let antes = vertices(&demao);
    um_dab(&mut demao);
    assert!(
        vertices(&demao) > antes,
        "o `Layer` não refinou ({antes} -> {}) — ordem do dono (14/09)",
        vertices(&demao)
    );

    // ⭐⭐⭐ (6) **O AGARRAR MEXE — ordem do dono**, e ele é o TERCEIRO caminho:
    // quem SEGURA não percorre nem gira, logo não passava por nenhuma das duas
    // portas que a wave da manhã ligou.
    let mut agarrar = cena_armada(&gpu.device, Verb::Move);
    let antes = vertices(&agarrar);
    um_traco_ancorado(&mut agarrar);
    assert!(
        vertices(&agarrar) != antes,
        "o `Move` não mexeu na contagem ({antes} -> {}) — ordem do dono \
         (*«move/drag deve subdividir»*), e o fio dele é o `grab_at`",
        vertices(&agarrar)
    );

    // ⭐⭐⭐ (7) **O POLEGAR MEXE**, e é a célula que custou uma peça: ele é o
    // único verbo que CONGELA a pegada no pen-down, e um índice guardado não
    // sobrevive sozinho a um colapso. Ver `SculptStroke::grow_with` e a irmã.
    let mut polegar = cena_armada(&gpu.device, Verb::Thumb);
    let antes = vertices(&polegar);
    um_traco_ancorado(&mut polegar);
    assert!(
        vertices(&polegar) != antes,
        "o `Thumb` não mexeu na contagem ({antes} -> {}) — ordem do dono \
         (*«Thumb se for possível, deveria subdividir»*)",
        vertices(&polegar)
    );

    // ⛔ (8) **O CONTROLO NEGATIVO DO CAMINHO QUE GIRA: a TORÇÃO não muda.**
    // Veredito do dono no mesmo smoke — *«Twist com dynamic topology fica com
    // resultado muito ruim»* —, e ele desempata as duas referências a favor da
    // medida. ⭐ **Ela é o controlo CERTO justamente porque o fio dela está
    // ligado**: o `turn_at` chama a porta, e quem recusa é a TABELA. *Um
    // controlo negativo sobre um caminho desligado não afirma nada.*
    let mut torcao = cena_armada(&gpu.device, Verb::Twist);
    let antes = vertices(&torcao);
    um_traco_ancorado(&mut torcao);
    assert_eq!(
        vertices(&torcao),
        antes,
        "a `Twist` mudou a contagem — o dono julgou-a no `=14` e ela não adensa"
    );

    // ⛔ (9) **E O CONTROLO NEGATIVO DO CAMINHO QUE SEGURA**, que é o fio novo
    // desta wave: o `Pose` entra pelo MESMO `grab_at` do agarrar e do polegar, e
    // a tabela responde-lhe `false`. Sem ele, ligar a porta a todo `Grip::Hold`
    // passaria nos dois de cima.
    let mut pose = cena_armada(&gpu.device, Verb::Pose);
    let antes = vertices(&pose);
    um_traco_ancorado(&mut pose);
    assert_eq!(
        vertices(&pose),
        antes,
        "o `Pose` mudou a contagem — ele partilha o fio novo com o `Move` e o \
         `Thumb`, e sem este lado o fio passaria a responder pelo grip"
    );
}

/// ⭐⭐⭐ **GATE — O CARIMBO NÃO PENTEIA E O PASSE PENTEIA.**
///
/// A ordem do dono de 19/09 (*«vamos modificar completamente esse algoritmo»*)
/// mudou a lei do pente de SÍTIO: ela saiu do carimbo (o deslocamento pelo
/// centroide do anel, que é a lei do ALVO e que a bancada de paridade mede) e
/// passou a ser a **retícula**, no passe de topologia.
///
/// ⛔⛔ **As duas metades, porque cada uma sozinha MENTE:**
///
/// - só a primeira lê-se como *«o pente morreu»* — e ele não morreu, mudou de
///   dono;
/// - só a segunda lê-se como *«há pente»* — e deixaria o produto a correr as
///   DUAS leis ao mesmo tempo, uma a puxar para o centroide do anel e a outra
///   para o ponto da grelha.
///
/// ⚠️ **A primeira metade é TEXTUAL de propósito.** O `armed_brush_on` vive na
/// [`Sculpt3dScene`], que pede um `wgpu::Device` para nascer ⇒ um gate a sério
/// ali seria `#[ignore]` e **o CI nunca o correria**. A segunda, essa, mede o
/// BARRO, pela porta sem cena e sem device.
#[test]
fn o_carimbo_nao_penteia_e_o_passe_penteia() {
    // (1) O carimbo: o pincel armado leva `pente: 0.0`, e a porta que o
    // calculava **não é chamada** ali.
    let space = include_str!("space.rs");
    let armado = space
        .split("fn armed_brush_on")
        .nth(1)
        .expect("o `armed_brush_on` tem de existir — ele é quem monta o pincel do dab");
    let corpo = &armado[..armado.find("\n    /// ").unwrap_or(armado.len())];
    assert!(
        corpo.contains("pente: 0.0,"),
        "o pincel que o produto entrega ao `dab` voltou a levar pente — as duas \
         leis passariam a correr ao mesmo tempo"
    );
    assert!(
        !corpo.contains("pente: pente_do_traco("),
        "o `armed_brush_on` voltou a chamar o `pente_do_traco` — é a lei do \
         ALVO, e ela saiu do caminho do produto por ordem do dono"
    );

    // (2) O passe: com o knob em cima ele MOVE barro; com ele a zero, nada.
    let alvo = 0.06f32;
    let (centro, raio) = ([0.0, 0.0, 1.0], 0.34f32);
    let direccao = [0.28f32, 0.0, 0.0];
    let com = passe_com_pente(alvo, centro, raio, Some((direccao, 1.0)));
    let sem = passe_com_pente(alvo, centro, raio, None);
    let base = passe_com_pente(alvo, centro, raio, Some(([0.0; 3], 1.0)));

    assert!(
        com > 0.0,
        "o passe com o pente no tecto não moveu um vértice: {com:.3e}"
    );
    assert_eq!(
        sem, 0.0,
        "o passe SEM pente moveu barro — a retícula tem de ser inerte ali"
    );
    // ⛔ E a terceira: sem DIRECÇÃO não há retícula, que é a inércia que o
    // primeiro carimbo de todo traço percorre.
    assert_eq!(
        base, 0.0,
        "com a direcção nula a retícula moveu barro — ela tem de ser inerte no \
         primeiro carimbo, como as leis que substituiu"
    );
}

/// Corre o passe de topologia **sem cena e sem device** e devolve o maior
/// deslocamento que a retícula produziu.
///
/// ⚠️ **Ele mede o que SOBRA depois dos dois motores**, e por isso compara com
/// a mesma corrida sem pente: o colapso e o refino também mexem em posições, e
/// uma medição contra a malha de entrada leria o trabalho deles como sendo da
/// retícula.
fn passe_com_pente(alvo: f32, centro: [f32; 3], raio: f32, pente: Option<([f32; 3], f32)>) -> f32 {
    let mut malha = uv_sphere(32, 48, 1.0);
    malha.triangulate();
    let referencia = {
        let mut m = malha.clone();
        corre_o_passe(&mut m, alvo, centro, raio, None);
        m
    };
    corre_o_passe(
        &mut malha,
        alvo,
        centro,
        raio,
        pente.map(|(direccao, forca)| super::Pente {
            direccao,
            forca,
            queda: ph2d_sculpt3d::Falloff::Smooth,
        }),
    );
    if malha.vert_count() != referencia.vert_count() {
        panic!(
            "as duas corridas mudaram a contagem de maneiras diferentes \
             ({} contra {}) — a comparação por índice deixaria de afirmar nada",
            malha.vert_count(),
            referencia.vert_count()
        );
    }
    malha
        .positions()
        .iter()
        .zip(referencia.positions())
        .map(|(a, b)| {
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
        })
        .fold(0.0f32, f32::max)
}

fn corre_o_passe(
    malha: &mut ph2d_mesh::Mesh,
    alvo: f32,
    centro: [f32; 3],
    raio: f32,
    pente: Option<super::Pente>,
) {
    let mut remap = ph2d_mesh::Remap::default();
    let mut births = Vec::new();
    let mut region = ph2d_mesh::RegionScratch::default();
    let _ = super::passe_nos_motores(
        malha,
        Verb::Draw,
        alvo,
        centro,
        raio,
        super::Rascunho {
            remap: &mut remap,
            births: &mut births,
            region: &mut region,
        },
        pente,
    );
}
