//! **O GESTO DO FILTRO, medido no produto.**
//!
//! Módulo irmão de teste do [`super`] (`#[path]`, `cfg(test)`), no molde do
//! `sculpt3d_transform_tests`: um gesto exige uma cena e uma cena exige um
//! device, então estes gates são `#[ignore]` + `gpu_or_skip!`.
//!
//! ⚠️ **A LEI já tem gate no kernel** (`stroke_filter_tests`, dez asserções e
//! duas mutações). O que se afirma AQUI é o que o kernel não pode ver: *o que a
//! mão na tela quer dizer* — o SINAL do arrasto, a exclusão contra o transform,
//! o arm que só está vivo onde o painel o mostra, e o gesto inteiro custar UM
//! passo de undo.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::filter::tests -- --ignored --nocapture
//! ```

use ph2d_mesh::shapes::uv_sphere;
use ph2d_sculpt3d::{TransformKind, Verb};

use super::super::Sculpt3dScene;

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

/// Quantos pixels o dedo anda. Grande o bastante para a força sair do ruído
/// (`FILTER_DRAG_PER_PX` é `0,001`), e é a régua da referência que decide isto —
/// não um número escolhido aqui.
const DRAG_PX: f32 = 400.0;

/// Uma cena com uma esfera e o verbo pedido em mãos.
fn scene(device: &wgpu::Device, verb: Verb) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, uv_sphere(24, 36, 1.0), 1.0);
    s.viewport = (900, 700);
    s.brush.verb = verb;
    s
}

/// A distância média dos vértices ao centroide — a régua de *inflou ou murchou*.
///
/// ⚠️ **Ela é medida contra o CENTROIDE VIVO e não contra a origem**, senão uma
/// lei que apenas transladasse a esfera inteira mediria como inflação.
fn mean_radius(s: &Sculpt3dScene) -> f32 {
    let p = s.mesh().positions();
    let n = p.len() as f32;
    let c = p.iter().fold([0.0f32; 3], |a, v| {
        [a[0] + v[0] / n, a[1] + v[1] / n, a[2] + v[2] / n]
    });
    p.iter()
        .map(|v| ((v[0] - c[0]).powi(2) + (v[1] - c[1]).powi(2) + (v[2] - c[2]).powi(2)).sqrt() / n)
        .sum()
}

/// Roda o gesto inteiro: arma, pousa em `x0`, arrasta até `x1`, solta.
fn drag(s: &mut Sculpt3dScene, x0: f32, x1: f32) {
    assert!(s.arm_filter(), "a fixture nao conseguiu armar o filtro");
    assert!(s.begin_filter(x0), "o begin recusou um verbo que filtra");
    s.filter_at(x1);
    s.close_stroke();
}

/// ⭐ **O SINAL do arrasto é o da referência: DIREITA é positivo.**
///
/// O `sculpt_filter_mesh.cc:2299` faz `len = prev − mouse` e depois
/// `strength = start · −len · 0.001`, que é `(mouse − prev) · 0.001`. Um sinal
/// trocado aqui **não daria erro nenhum** — o filtro funcionaria, murchando onde
/// o artista pede para inflar, e nenhum gate de lei do kernel veria (lá o
/// `amount` chega pronto).
///
/// ⚠️ **O oráculo é o RAIO MÉDIO, e não o `amount`**: perguntar o número que a
/// própria fiação calculou seria o espelho que este repo já pagou várias vezes.
/// E o verbo é o **Inflate** de propósito — é o único cuja direção é
/// inequívoca na tela (um Smooth encolhe nos dois sentidos do arrasto).
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn dragging_right_inflates_and_dragging_left_deflates() {
    let gpu = gpu_or_skip!();

    let mut s = scene(&gpu.device, Verb::Inflate);
    let before = mean_radius(&s);
    drag(&mut s, 400.0, 400.0 + DRAG_PX);
    let right = mean_radius(&s);

    let mut s = scene(&gpu.device, Verb::Inflate);
    drag(&mut s, 400.0, 400.0 - DRAG_PX);
    let left = mean_radius(&s);

    eprintln!("[filtro] raio medio: antes {before:.6} | direita {right:.6} | esquerda {left:.6}");
    assert!(
        right > before,
        "arrastar para a DIREITA tinha de inflar (a lei do sculpt_filter_mesh.cc:2299): \
         {before:.6} -> {right:.6}"
    );
    assert!(
        left < before,
        "arrastar para a ESQUERDA tinha de murchar: {before:.6} -> {left:.6}"
    );
}

/// **Os dois arms reivindicam o MESMO botão, então nunca estão acesos juntos.**
///
/// ⚠️ **Nas DUAS direções:** armar o filtro apaga o transform e armar o
/// transform apaga o filtro. Uma metade só deixaria a ordem dos cliques decidir
/// o que o botão esquerdo faz.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_two_arms_never_claim_the_left_button_together() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Smooth);

    s.arm_transform(TransformKind::Move);
    assert!(s.transform_arm().is_some(), "o transform nao armou");
    assert!(s.arm_filter(), "o filtro nao armou");
    assert!(
        s.transform_arm().is_none(),
        "armar o filtro deixou o transform aceso: os dois disputam o botao esquerdo"
    );

    s.arm_transform(TransformKind::Move);
    assert!(
        !s.filter_arm(),
        "armar o transform deixou o filtro aceso: os dois disputam o botao esquerdo"
    );

    // CONTROLE: o toggle desliga o que ele mesmo ligou.
    let mut s = scene(&gpu.device, Verb::Smooth);
    assert!(s.arm_filter(), "1o clique arma");
    assert!(!s.arm_filter(), "2o clique desarma");
    assert!(!s.filter_arm(), "o desarme nao pegou");
}

/// ⭐ **VISÍVEL ⇔ VIVO** — o arm apaga com um verbo sem lei, e SOBREVIVE à volta.
///
/// O painel só oferece o interruptor a `Verb::filters_mesh`. Um arm que
/// sobrevivesse a pegar o Draw ficaria **aceso e invisível**: o botão esquerdo
/// pararia de esculpir sem nada na tela dizendo por quê.
///
/// ⚠️ **A segunda metade é a que impede a cura preguiçosa.** Apagar o flag na
/// troca de verbo satisfaria a primeira asserção e esqueceria a escolha do
/// artista — voltar ao Smooth teria de re-armar à mão.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_arm_goes_dark_without_a_law_and_comes_back_with_one() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Smooth);
    assert!(s.arm_filter() && s.filter_arm(), "armado com o Smooth");

    s.brush.verb = Verb::Draw;
    assert!(
        !Verb::Draw.filters_mesh(),
        "a fixture perdeu a premissa: o Draw passou a filtrar"
    );
    assert!(
        !s.filter_arm(),
        "o arm ficou VIVO com um verbo que nao filtra -- o botao esquerdo pararia de esculpir com \
         o painel sem mostrar por que"
    );
    assert!(
        !s.begin_filter(100.0),
        "o gesto abriu sobre um verbo sem lei"
    );

    s.brush.verb = Verb::Smooth;
    assert!(
        s.filter_arm(),
        "voltar ao verbo que filtra tinha de devolver a escolha do artista"
    );
}

/// **O gesto inteiro custa UM passo de undo, e ele DESFAZ.**
///
/// ⚠️ **O fecho é o `close_stroke`, a porta do traço** — este gate é o que prova
/// que ela serve: se o `filter_begin` deixasse de preencher os dois arrays que
/// ela grava, o desfazer devolveria a malha errada (ou nada) em silêncio.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_whole_drag_is_one_undo_step() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Inflate);
    let before: Vec<[f32; 3]> = s.mesh().positions().to_vec();

    assert!(s.arm_filter());
    assert!(s.begin_filter(400.0));
    // ⚠️ **Cinco eventos, um gesto** — é aqui que um filtro que compusesse
    // incrementos em vez de reler a pose congelada se separaria de um que não
    // compõe, e é o mesmo caminho que o rato real percorre.
    for k in 1..=5 {
        s.filter_at(400.0 + DRAG_PX * k as f32 / 5.0);
    }
    s.close_stroke();

    let moved = s
        .mesh()
        .positions()
        .iter()
        .zip(&before)
        .filter(|(a, b)| a != b)
        .count();
    assert!(moved > 0, "o gesto nao moveu vertice nenhum");

    assert!(s.undo_stroke(), "o gesto nao gravou passo de undo");
    assert_eq!(
        s.mesh().positions(),
        &before[..],
        "UM desfazer tinha de devolver a malha inteira -- o gesto gravou mais de um passo, ou \
         gravou a janela errada"
    );
    assert!(
        !s.undo_stroke(),
        "o gesto gravou MAIS de um passo: o artista teria de desfazer N vezes um arrasto so'"
    );
}
