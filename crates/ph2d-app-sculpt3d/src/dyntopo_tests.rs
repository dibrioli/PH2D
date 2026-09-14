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
    s
}

/// ⭐⭐⭐ **A DENSIDADE AFINA A MALHA, E NUNCA A ENGROSSA.**
///
/// São **duas leis, não uma** (espec §3.2): *ele liga o colapso* **e** *ele não
/// liga o partir*. ⚠️ Um gate que só verificasse a primeira passaria com um
/// pincel que **também subdivide**, que é outro produto — e é por isso que a
/// segunda metade corre sobre uma malha GROSSA, onde partir teria o que fazer.
///
/// **Medido** (um dab, raio `160 px`):
///
/// | arranjo | verbo | vértices |
/// |---|---|---|
/// | malha fina (`48×72`), alvo grosso | `Density` | **`3 386 → 3 352`** |
/// | malha grossa (`8×12`), alvo fino | `Density` | **`86 → 86`** — ele nunca acrescenta |
/// | a MESMA, alvo fino | `Draw` | **`86 → 359`** — o controlo |
///
/// ⚠️ **A colheita da primeira linha é modesta (`34` vértices) e isso é a
/// NOSSA lei, não um defeito:** o nosso colapso tem quatro recusas
/// (`ph2d-mesh/src/collapse.rs`), entre elas *«algum dos quatro vértices está na
/// beira»* — mais dura que a do alvo, que em vez de recusar **escolhe o
/// sobrevivente**. A espec §3.8 declara isso uma **decisão de produto** com duas
/// frases e sem terceira saída, e o que shipa é a conservadora: *o `Density`
/// respeita as recusas que o nosso colapso já tem, e no bordo ele simplesmente
/// não come*.
#[test]
#[ignore]
fn a_densidade_afina_a_malha_e_nunca_a_engrossa() {
    let gpu = gpu_or_skip!();

    // (a) MALHA FINA, alvo GROSSO — há aresta curta de sobra, logo o colapso
    // tem o que comer.
    let fina = || ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    let mut s = cena_com(&gpu.device, Verb::Density, fina(), 0.15, 160.0);
    let antes = vertices(&s);
    um_dab(&mut s);
    let depois = vertices(&s);
    assert!(
        depois < antes,
        "a densidade não afinou nada ({antes} -> {depois}) — todo o efeito dela \
         é sobre o passe de topologia, e sem isto ela é um pincel inerte"
    );

    // (b) MALHA GROSSA, alvo FINO — aqui **partir** teria muito o que fazer, e
    // é isto que separa este pincel de um que também subdivide.
    let grossa = || ph2d_mesh::shapes::uv_sphere(8, 12, 1.0);
    let mut s = cena_com(&gpu.device, Verb::Density, grossa(), 1.0, 160.0);
    let antes = vertices(&s);
    um_dab(&mut s);
    let depois = vertices(&s);
    assert!(
        depois <= antes,
        "a densidade ACRESCENTOU superfície ({antes} -> {depois}) — ela liga o \
         colapso e NÃO liga o partir"
    );

    // ⭐ **O controlo positivo da metade (b):** o mesmo arranjo com um verbo que
    // liga as duas metades **cresce**. Sem ele, o `<=` acima ficaria verde sobre
    // um passe que nunca dispara.
    let mut s = cena_com(&gpu.device, Verb::Draw, grossa(), 1.0, 160.0);
    let antes = vertices(&s);
    um_dab(&mut s);
    let depois = vertices(&s);
    assert!(
        depois > antes,
        "o desenho não subdividiu a malha grossa ({antes} -> {depois}) — sem \
         isto a metade (b) não afirma nada"
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
