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
    modo: ph2d_sculpt3d::DensityModo,
    malha: ph2d_mesh::Mesh,
    detalhe: f32,
    raio_px: f32,
) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, malha, 1.0);
    // ⚠️ **A direcção é do PINCEL e entra pelo arnês**, nunca por omissão: as
    // duas células que a tabela-verdade da espec separa diferem SÓ neste campo,
    // e lê-las do default deixaria metade do gate a medir o outro caso.
    s.brush.density_modo = modo;
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
    s
}

/// ⭐⭐⭐ **A TABELA-VERDADE DO PASSE, célula a célula** (espec §3.2).
///
/// ⛔⛔ **Este gate nasceu a afirmar UMA CÉLULA AO CONTRÁRIO, e quem o desmentiu
/// foi o dono, pelo produto:** *«por que não pode aumentar a densidade
/// também?»* (2026-09-14). A redacção anterior chamava-se *«a densidade afina a
/// malha e NUNCA a engrossa»* e escrevia, com o comentário ao lado, que *«um
/// pincel que também subdividisse é outro produto»*.
///
/// **Não é.** A espec diz que este pincel **ACRESCENTA a bandeira de colapso**
/// ao modo do passe — ele não **RETIRA** a de partir. Quem decide o partir é o
/// ajuste ([`ph2d_sculpt3d::DensityModo`]), e a espec mede as duas células **na
/// mesma malha grossa**: `81 → 81` com o ajuste em «só colapsar» e **`81 → 101`**
/// com «partir + colapsar».
///
/// ⚠️ **A recusa medida da espec continua de pé e é OUTRA pergunta:** o pincel
/// não pode **FORÇAR** o partir, como força o colapso. A célula (b) é
/// exactamente esse controlo.
///
/// **Medido** (um dab, raio `160 px`):
///
/// | arranjo | ajuste | vértices |
/// |---|---|---|
/// | malha fina (`48×72`), alvo grosso | `Afinar` | **`3 386 → 3 352`** |
/// | malha grossa (`8×12`), alvo fino | `Afinar` | **`86 → 86`** — ele não FORÇA o partir |
/// | a MESMA, alvo fino | **`Igualar`** | **cresce** — a célula que o dono pediu |
/// | a MESMA, alvo fino | `Draw` | **`86 → 359`** — o controlo |
///
/// ⚠️ **A colheita da primeira linha é modesta (`34` vértices) e isso é a
/// NOSSA lei, não um defeito:** o nosso colapso tem quatro recusas
/// (`ph2d-mesh/src/collapse.rs`), entre elas *«algum dos quatro vértices está na
/// beira»* — mais dura que a do alvo, que em vez de recusar **escolhe o
/// sobrevivente**. A espec §3.8 declara isso uma **decisão de produto** com duas
/// frases e sem terceira saída, e o que shipa é a conservadora: *o `Density`
/// respeita as recusas que o nosso colapso já tem, e no bordo ele simplesmente
/// não come*. ⭐ A magnitude que o dono vê está no gate irmão
/// [`a_densidade_tira_uma_fraccao_visivel_e_nao_um_punhado`], que mede o
/// percurso dele inteiro e não um dab.
#[test]
#[ignore]
fn a_densidade_obedece_a_tabela_verdade_do_passe() {
    use ph2d_sculpt3d::DensityModo;
    let gpu = gpu_or_skip!();

    // (a) MALHA FINA, alvo GROSSO — há aresta curta de sobra, logo o colapso
    // tem o que comer. ⭐ **E ele corre nos DOIS ajustes**: é a lei do pincel
    // (a coluna *colapsar* está a `sim` nas três linhas da tabela da espec), e
    // não um ajuste.
    let fina = || ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    for modo in DensityModo::ALL {
        let mut s = cena_com(&gpu.device, Verb::Density, modo, fina(), 0.15, 160.0);
        let antes = vertices(&s);
        um_dab(&mut s);
        let depois = vertices(&s);
        assert!(
            depois < antes,
            "a densidade não afinou nada em `{}` ({antes} -> {depois}) — o \
             colapso é a LEI deste pincel e corre nos dois ajustes",
            modo.label()
        );
    }

    // (b) MALHA GROSSA, alvo FINO, ajuste `Afinar` — aqui **partir** teria muito
    // o que fazer, e é isto que prova que o pincel não o FORÇA.
    let grossa = || ph2d_mesh::shapes::uv_sphere(8, 12, 1.0);
    let mut s = cena_com(
        &gpu.device,
        Verb::Density,
        DensityModo::Afinar,
        grossa(),
        1.0,
        160.0,
    );
    let antes = vertices(&s);
    um_dab(&mut s);
    let so_afina = vertices(&s);
    assert!(
        so_afina <= antes,
        "a densidade ACRESCENTOU superfície com o ajuste em `Thin Only` \
         ({antes} -> {so_afina}) — ali ela só colapsa"
    );

    // ⭐⭐⭐ (c) **A MESMA malha e o MESMO alvo, com o ajuste de OMISSÃO: ela
    // CRESCE.** É a célula `81 → 101` da espec, e é o report do dono.
    let mut s = cena_com(
        &gpu.device,
        Verb::Density,
        DensityModo::Igualar,
        grossa(),
        1.0,
        160.0,
    );
    let antes_ig = vertices(&s);
    um_dab(&mut s);
    let iguala = vertices(&s);
    assert_eq!(
        antes_ig, antes,
        "os dois arranjos têm de partir da MESMA malha, senão a comparação \
         abaixo não é entre ajustes"
    );
    assert!(
        iguala > so_afina,
        "o ajuste de omissão não adensou nada ({antes_ig} -> {iguala}, contra \
         {so_afina} em `Thin Only`) — é exactamente o report do dono: *«por que \
         não pode aumentar a densidade também?»*"
    );

    // ⭐ **O controlo positivo:** o mesmo arranjo com um verbo que liga as duas
    // metades cresce também. Sem ele, a célula (b) ficaria verde sobre um passe
    // que nunca dispara.
    let mut s = cena_com(
        &gpu.device,
        Verb::Draw,
        DensityModo::Afinar,
        grossa(),
        1.0,
        160.0,
    );
    let antes = vertices(&s);
    um_dab(&mut s);
    let depois = vertices(&s);
    assert!(
        depois > antes,
        "o desenho não subdividiu a malha grossa ({antes} -> {depois}) — sem \
         isto a célula (b) não afirma nada"
    );
}

/// ⭐⭐⭐ **A PISTA DO DETALHE CHEGA AO MOTOR — e a régua é a MALHA, nunca o
/// campo.**
///
/// ⚠️⚠️ **É o ponto cego que o §5.0 do roteador nomeia:** *nenhum instrumento
/// deste repo pergunta se o VALOR chega a um consumidor*. Um slider pode estar
/// pintado, registado, vivo sob o ponteiro e varrido pela costura — e o número
/// dele nunca sair do painel. As três metades aqui são:
///
/// 1. o painel **publica** o que a cena tem (ida);
/// 2. a cena **escreve** o que o painel mandou (volta);
/// 3. ⭐ **duas posições da pista dão malhas DIFERENTES** no mesmo gesto — a
///    única das três que prova que o número atravessou até ao motor.
///
/// ⛔ Sem a terceira, cravar o `detail` numa constante dentro do passe deixaria
/// as duas primeiras verdes.
#[test]
#[ignore]
fn a_pista_do_detalhe_chega_ao_motor() {
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(10, 14, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));

    // (1) IDA — o retrato publica o que a cena tem.
    s.dyntopo.detail = 0.15;
    assert!(
        (s.panel_snapshot(false).ui.dyn_detail - 0.15).abs() < 1e-6,
        "o retrato não publica o detalhe da cena — a pista nasceria a mostrar \
         outro número que o do motor"
    );

    // (2) VOLTA — a cena escreve o que o painel mandou.
    //
    // ⚠️ **Pela PORTA DO PRODUTO** (`apply_panel_intent`), e não pelo `apply_ui`
    // privado: é este o caminho que um arrasto de pista toma, e um gate que
    // chamasse o ajudante interno afirmaria sobre código que o painel não usa —
    // a mesma armadilha que este repo já registou como *nomear a VISIBILIDADE
    // em vez da lei*.
    let mut ui = s.panel_snapshot(false).ui;
    ui.dyn_detail = 0.9;
    s.apply_panel_intent(ph2d_panel_sculpt3d::Sculpt3dIntent::SetUi(ui));
    assert!(
        (s.dyntopo.detail - 0.9).abs() < 1e-6,
        "a pista não chega ao campo da cena: {} — um slider morto",
        s.dyntopo.detail
    );

    // (3) ⭐ **E O NÚMERO CHEGA AO MOTOR:** o mesmo gesto, na mesma malha, com
    // duas posições da pista, tem de dar contagens DIFERENTES. É esta metade
    // que uma constante cravada no passe faria sangrar.
    let conta_com = |detalhe: f32| {
        let mut s = cena_com(
            &gpu.device,
            Verb::Density,
            ph2d_sculpt3d::DensityModo::Igualar,
            ph2d_mesh::shapes::uv_sphere(10, 14, 1.0),
            detalhe,
            160.0,
        );
        um_dab(&mut s);
        vertices(&s)
    };
    let grosso = conta_com(0.0);
    let fino = conta_com(1.0);
    assert!(
        fino > grosso,
        "as duas pontas da pista dão a mesma malha ({grosso} e {fino}) — o \
         número não atravessa até ao motor"
    );
}

/// ⭐⭐ **COM O PASSE DESARMADO ELA É INTEIRAMENTE INERTE, e o `0` é exacto.**
///
/// Espec §3.1: a diferença máxima de posição é **`0` exactamente**, não
/// «pequena» — ela não desloca vértice nenhum, e a única metade que ela arma
/// está fora de jogo. É o análogo do *Detailing* em **Manual** do alvo.
#[test]
#[ignore]
fn com_o_passe_desarmado_a_densidade_nao_move_um_vertice() {
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(48, 72, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Density;
    // ⚠️ **SEM `toggle_dyntopo`**: é esta ausência que o gate mede.
    let antes = s.objects[s.active].stack.mesh().positions().to_vec();
    um_dab(&mut s);
    let depois = s.objects[s.active].stack.mesh().positions().to_vec();

    assert_eq!(antes.len(), depois.len(), "a contagem não pode mudar");
    let pior = antes
        .iter()
        .zip(&depois)
        .map(|(a, b)| {
            (a[0] - b[0])
                .abs()
                .max((a[1] - b[1]).abs())
                .max((a[2] - b[2]).abs())
        })
        .fold(0.0f32, f32::max);
    assert_eq!(
        pior, 0.0,
        "a densidade moveu barro com o passe desarmado (pior delta {pior:e}) — \
         ela não tem lei por-vértice nenhuma"
    );

    // ⛔⛔ **E ela nem CHEGA A OLHAR para um vértice — esta metade é a que
    // discrimina, e a primeira redacção não a tinha.**
    //
    // ⚠️ Medido por mutação: apagar o desvio do `stroke_symmetry` deixava o
    // `assert_eq!(pior, 0.0)` acima **verde**, porque o `stroke_target` tem um
    // braço `Verb::Density => live` — a resposta defensiva para *«e se alguém
    // chegar aqui mesmo assim?»*, a mesma que o tecido e a pose têm. *Duas
    // respostas à mesma pergunta, e a de baixo mascarava a de cima.*
    //
    // ⇒ a régua passa a ser a JANELA DO TRAÇO: o `dab_core` fotografa (`capture`)
    // todo vértice ao alcance antes de decidir o que fazer com ele, logo um
    // `touched` não-vazio prova que a cadeia de peso correu — mesmo quando ela
    // não move nada.
    assert!(
        s.stroke.touched().is_empty(),
        "a densidade fotografou {} vértices — ela desvia ANTES do laço \
         por-vértice, e um `touched` não-vazio quer dizer que a cadeia de peso \
         correu à mesma",
        s.stroke.touched().len()
    );
}

/// **SONDA:** o percurso do dono na `=14` — adensar com o `Draw` e depois afinar
/// com o `Density`, dab a dab.
#[test]
#[ignore]
fn diag_o_percurso_do_dono() {
    let gpu = gpu_or_skip!();
    // A malha da própria cena `=14`.
    let malha = ph2d_mesh::shapes::uv_sphere(10, 14, 1.0);
    let mut s = Sculpt3dScene::new(&gpu.device, malha, 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    let (ligado, tri) = s.toggle_dyntopo();
    assert!(ligado);
    println!("cena =14: {} verts (triangulou {tri} faces)", vertices(&s));

    s.brush.verb = Verb::Draw;
    s.dyntopo.detail = 1.0;
    for i in 0..8 {
        um_dab(&mut s);
        println!("  Draw fino dab {i}: {} verts", vertices(&s));
    }
    s.brush.verb = Verb::Density;
    for modo in ph2d_sculpt3d::DensityModo::ALL {
        s.brush.density_modo = modo;
        for detalhe in [0.15f32, 0.5] {
            s.dyntopo.detail = detalhe;
            for i in 0..10 {
                um_dab(&mut s);
                println!(
                    "  Density {} detalhe {detalhe} dab {i}: {} verts",
                    modo.label(),
                    vertices(&s)
                );
            }
        }
    }

    // ⭐⭐ **O GESTO NOVO — adensar com o próprio pincel de densidade**, que é a
    // metade que o report de 14/09 pediu (*«por que não pode aumentar a
    // densidade também?»*). Parte da malha CRUA da cena, sem `Draw` nenhum.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(10, 14, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    assert!(s.toggle_dyntopo().0);
    s.brush.verb = Verb::Density;
    s.brush.density_modo = ph2d_sculpt3d::DensityModo::Igualar;
    s.dyntopo.detail = 1.0;
    println!("adensar com Density: {} verts (cru)", vertices(&s));
    for i in 0..6 {
        um_dab(&mut s);
        println!("  Density Equalise fino dab {i}: {} verts", vertices(&s));
    }
}

/// ⭐⭐⭐ **ELA TIRA UMA FRACÇÃO VISÍVEL, e não um punhado de vértices.**
///
/// ⛔⛔ **Este gate nasceu de um report do dono — *«não vejo efeito com
/// density»* — e o que falhou foi a RÉGUA, não o pincel.** O gate irmão
/// afirmava `depois < antes`: uma **direcção**. Com ele verde, a colheita
/// medida era de `34` vértices em `3 386` — **1 %**, que é invisível a olho nu.
/// *Uma régua que só vê o SINAL não vê a MAGNITUDE*, e é a mesma família do
/// `edge_max` cego ao quad fino e da contagem cega a QUAIS vértices se movem.
///
/// ⇒ a barra é uma **fracção**, e ela sai do percurso do próprio dono medido
/// dab a dab: com o detalhe em `grosso`, dez toques levam a peça de `822` para
/// `399` vértices (**−51 %**). A barra fica em `−25 %`, com margem de `2×`.
#[test]
#[ignore]
fn a_densidade_tira_uma_fraccao_visivel_e_nao_um_punhado() {
    let gpu = gpu_or_skip!();
    // O percurso do dono: a malha da `=14`, adensada com o `Draw` no fino.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(10, 14, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    assert!(s.toggle_dyntopo().0);
    s.brush.verb = Verb::Draw;
    s.dyntopo.detail = 1.0;
    for _ in 0..8 {
        um_dab(&mut s);
    }
    let adensada = vertices(&s);
    assert!(
        adensada > 700,
        "o arranjo não adensou o suficiente para a pergunta ter sentido: {adensada}"
    );

    // ⚠️ **E o detalhe em GROSSO é parte do gesto, não do arnês:** o alvo de
    // aresta é `raio × f(detalhe)`, logo num detalhe fino não há o que colapsar.
    // É exactamente isto que a queixa do passe diz ao artista.
    s.brush.verb = Verb::Density;
    s.dyntopo.detail = 0.15;
    for _ in 0..10 {
        um_dab(&mut s);
    }
    let afinada = vertices(&s);
    let fraccao = 1.0 - (afinada as f32) / (adensada as f32);
    assert!(
        fraccao >= 0.25,
        "a densidade tirou só {:.1} % ({adensada} -> {afinada}) — uma colheita \
         que o dono não vê é indistinguível de um pincel partido",
        fraccao * 100.0
    );
}

/// ⭐⭐⭐ **QUANDO ELA NÃO FAZ NADA, ELA DIZ PORQUÊ.**
///
/// ⛔ Um verbo cujo efeito inteiro é sobre a topologia **parece partido** sempre
/// que o passe não corre, e há **três** razões diferentes que o artista vê
/// **iguais**: nada acontece. *Foi assim que o report nasceu.*
///
/// ⚠️ **O controlo negativo está dentro:** um `Draw` que não parte nada é o caso
/// **normal** (a malha já tem a densidade pedida ali) e tem de ficar **calado** —
/// senão o log enche-se e deixa de ser lido.
#[test]
#[ignore]
fn quando_a_densidade_nao_faz_nada_ela_diz_porque() {
    let gpu = gpu_or_skip!();

    // (a) O modo DESLIGADO — a razão mais comum, e a que o dono encontrou.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(24, 36, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Density;
    um_dab(&mut s);
    assert!(
        s.dyn_queixa_dita,
        "a densidade ficou muda com o modo desligado — o artista vê um pincel \
         partido e não tem como saber que falta o `P`"
    );

    // (b) ARMADA mas sem nada a fazer (detalhe fino numa malha grossa).
    //
    // ⚠️⚠️ **O ajuste é `Thin Only`, e é ele que CRIA a pergunta.** Desde que o
    // pincel também adensa (report do dono, 14/09), no ajuste de omissão este
    // arranjo **parte** arestas — ou seja, o passe faz alguma coisa e não há
    // queixa nenhuma a dar. *É a metade do report que já está curada.* A queixa
    // continua a existir para quem escolhe só afinar, que é exactamente onde o
    // artista pode pedir uma coisa que a malha não tem.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(10, 14, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    assert!(s.toggle_dyntopo().0);
    s.brush.verb = Verb::Density;
    s.brush.density_modo = ph2d_sculpt3d::DensityModo::Afinar;
    s.dyntopo.detail = 1.0;
    um_dab(&mut s);
    assert!(
        s.dyn_queixa_dita,
        "a densidade ficou muda sem aresta ao alcance — é a razão mais difícil \
         de adivinhar de fora, porque depende do DETALHE e não do pincel"
    );

    // ⭐ **O CONTROLO NEGATIVO: o desenho no MESMO caminho fica CALADO.**
    //
    // ⚠️⚠️ **A primeira redacção deste controlo era VÁCUO, e uma mutação
    // provou-o:** ela usava uma esfera densa com o detalhe grosso, onde o
    // `Draw` **colapsa** — logo ele nunca chegava ao caminho da queixa, e
    // tornar a queixa universal não o reprovava. *Um controlo negativo tem de
    // percorrer o MESMO caminho que a metade positiva*, senão ele está a
    // afirmar sobre código que não corre.
    //
    // ⇒ ele usa agora o caminho (a) — o modo **desligado** —, que é onde a
    // queixa de facto nasce.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(24, 36, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Draw;
    um_dab(&mut s);
    assert!(
        !s.dyn_queixa_dita,
        "o desenho queixou-se — um passe que não corre é o caso NORMAL nele (ele \
         esculpe na mesma), e uma linha por dab é um log que ninguém lê"
    );
}
