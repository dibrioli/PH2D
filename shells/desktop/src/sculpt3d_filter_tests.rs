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
use ph2d_sculpt3d::{FilterKind, TransformKind, Verb};

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

/// ⭐ **O VERBO SEMEIA A LEI, E NUNCA A REESCREVE** — a premissa que a W9b
/// derrubou, com a metade que sobreviveu a ela.
///
/// ⚠️ **O gate ANTERIOR afirmava o oposto** (*"o arm apaga com um verbo sem
/// lei"*), e estava certo enquanto a lei era DERIVADA do verbo em mãos: sem
/// verbo que filtrasse não havia lei, então um arm aceso ficava **invisível**.
/// Com a lei ESCOLHIDA num selector isso deixa de valer — a row é sempre
/// pintada, um arm aceso é sempre visível, e o que resta a defender é o
/// contrário: **pegar outra ferramenta não pode apagar a escolha do artista.**
///
/// ⚠️ **A metade do SEED é a que impede as duas curas preguiçosas:** re-semear
/// a cada troca de verbo faria o selector ser sobrescrito por um gesto que não
/// fala sobre ele, e não semear nunca obrigaria a escolher de novo o que a
/// ferramenta na mão já diz.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_verb_seeds_the_law_and_never_rewrites_it() {
    let gpu = gpu_or_skip!();

    // Armar com um verbo que TEM lei semeia com a dele.
    let mut s = scene(&gpu.device, Verb::Smooth);
    s.filter_kind = FilterKind::Sphere;
    assert!(s.arm_filter() && s.filter_arm(), "armado com o Smooth");
    assert_eq!(
        s.filter_kind,
        Verb::Smooth.filter_kind().expect("o Smooth filtra"),
        "armar com um verbo que filtra tinha de semear a lei DELE"
    );

    // Trocar de verbo com o filtro ACESO nao mexe em nada.
    s.brush.verb = Verb::Draw;
    assert!(
        !Verb::Draw.filters_mesh(),
        "a fixture perdeu a premissa: o Draw passou a filtrar"
    );
    assert!(
        s.filter_arm(),
        "o arm apagou ao pegar outro verbo -- a lei e ESCOLHIDA, nao derivada"
    );
    assert!(
        s.begin_filter(100.0),
        "o gesto recusou sobre um verbo sem lei propria, e a lei nao vem dele"
    );
    assert_eq!(
        s.filter_kind,
        Verb::Smooth.filter_kind().expect("o Smooth filtra"),
        "trocar de verbo com o filtro aceso REESCREVEU a escolha do artista"
    );

    // E armar com um verbo SEM lei deixa a ultima escolha de pe.
    let mut s = scene(&gpu.device, Verb::Draw);
    s.filter_kind = FilterKind::Random;
    assert!(s.arm_filter());
    assert_eq!(
        s.filter_kind,
        FilterKind::Random,
        "um verbo sem lei propria apagou a escolha ao armar"
    );
}

/// ⭐ **AS TRÊS LEIS SEM VERBO SÃO ALCANÇÁVEIS** — a razão de a wave existir.
///
/// ⚠️ Não há pincel de `Scale`, de `Sphere` nem de `Random`, então enquanto a
/// lei vinha do verbo elas eram **inexprimíveis por gesto nenhum**. O oráculo é
/// o produto: cada uma tem de MOVER a malha por um caminho que o artista de
/// facto percorre (armar → escolher → arrastar).
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_verbless_laws_are_reachable_from_a_gesture() {
    let gpu = gpu_or_skip!();
    for kind in [FilterKind::Scale, FilterKind::Sphere, FilterKind::Random] {
        // O verbo em maos e o Draw: ele NAO filtra, e e esse o ponto.
        //
        // ⚠️ **A malha e um ELIPSOIDE, e nao a esfera das outras fixtures:** o
        // `Sphere` puxa cada ponto para a esfera unitaria, e sobre uma esfera
        // unitaria centrada na origem ele e a IDENTIDADE -- a fixture nao
        // conteria o fenomeno, e o gate acusaria a lei de nao chegar ao motor
        // sobre um produto correto. Achatar em X da-lhe o que corrigir; para o
        // `Scale` e o `Random` a troca e inofensiva.
        let mut s = scene(&gpu.device, Verb::Draw);
        for p in s.objects[s.active].stack.mesh_mut().positions_mut() {
            p[0] *= 3.0;
        }
        let before: Vec<[f32; 3]> = s.mesh().positions().to_vec();

        assert!(s.arm_filter(), "{kind:?}: o arm recusou");
        s.filter_kind = kind;
        assert!(s.begin_filter(400.0), "{kind:?}: o gesto recusou");
        s.filter_at(700.0);

        let worst = s
            .mesh()
            .positions()
            .iter()
            .zip(&before)
            .map(|(a, b)| {
                let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            })
            .fold(0.0f32, f32::max);
        assert!(
            worst > 1e-3,
            "{kind:?} nao moveu a malha ({worst:.6}) -- a lei escolhida nao chegou ao motor"
        );
    }
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

/// ⭐ **O FILTRO DESFAZ COM QUALQUER VERBO EM MÃOS** — o report do Enio de
/// 2026-08-18 (*"não temos undo para Filter"*).
///
/// ⚠️ **O que a fixture do irmão [`the_whole_drag_is_one_undo_step`] não continha
/// era o VERBO, e não a porta de entrada.** Ele arrasta com o `Inflate` em mãos e
/// fica verde; o que quebra é o `Mask`, porque o registo perguntava *"o verbo
/// pinta máscara?"* em vez de *"este gesto mudou máscaras?"* — e um filtro só
/// escreve POSIÇÕES, seja qual for o pincel que estava na mão quando o artista
/// armou. Com o `Verb::Mask` a entrada saía gravada como se fosse um gesto de
/// máscara: o Ctrl+Z trocava o canal errado, a geometria ficava onde estava, e
/// do lado do artista isso é exactamente *não tem undo*.
///
/// ⚠️ **E ele é alcançável de propósito**: desde o picker da W9b a lei do filtro
/// não vem do verbo — quem tem o `Mask` na mão pode armar o filtro e escolher
/// `Inflate`, e é o caminho que o smoke `=34` percorre.
///
/// ⚠️ **A rota do PONTEIRO (`sculpt3d_pointer_down/move/up`) NÃO é alcançável de
/// um teste**, e a nota que pedia este gate ali estava errada sobre o preço:
/// aquelas três funções vivem no `App`, cujo `AppGfx` segura um
/// `SurfaceContext` (uma surface wgpu presa a uma janela real,
/// `app_state.rs:51`). O que elas fazem com o filtro em voo é literalmente esta
/// sequência — `aim` + `begin_filter` no braço `if scene.filter_arm()`
/// (`sculpt3d_input.rs:106-111`), `filter_at` no move, e `close_stroke` no
/// `was == Some(Drag::Filter)` do up (`sculpt3d_input.rs:243-244`) —, e é ela
/// que este gate dirige.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_filter_undoes_the_geometry_whatever_verb_is_in_hand() {
    let gpu = gpu_or_skip!();
    // O pincel de MÁSCARA em mãos, e a lei do filtro escolhida no picker.
    let mut s = scene(&gpu.device, Verb::Mask);
    let before: Vec<[f32; 3]> = s.mesh().positions().to_vec();
    let masks_before = s.mesh().masks().map(<[f32]>::to_vec);

    assert!(s.arm_filter(), "a fixture nao conseguiu armar o filtro");
    s.filter_kind = FilterKind::Inflate;
    assert!(
        s.begin_filter(400.0),
        "o begin recusou: a lei do filtro voltou a vir do verbo em maos"
    );
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
    assert!(
        moved > 0,
        "premissa: o gesto nao moveu vertice nenhum, e sem isso o resto nao afirma nada"
    );

    assert!(s.undo_stroke(), "o gesto nao gravou passo de undo");
    assert_eq!(
        s.mesh().positions(),
        &before[..],
        "o Ctrl+Z NAO devolveu a geometria: a entrada foi gravada como gesto de MASCARA porque o \
         pincel em maos era o Mask, e o desfazer trocou o canal que o filtro nunca escreveu"
    );
    // ⚠️ **E o canal de máscara não pode ter sido tocado** — inclusive não pode
    // ter NASCIDO: `masks_mut()` cria o plano, então o desfazer errado inventava
    // uma máscara numa malha que nunca teve uma.
    assert_eq!(
        s.mesh().masks().map(<[f32]>::to_vec),
        masks_before,
        "o desfazer mexeu no canal de MASCARA, que este gesto nunca escreveu"
    );
}

/// **Desfazer o filtro AVISA A TELA** — o segundo defeito do mesmo braço.
///
/// ⚠️ **Ele é independente do irmão acima, e é a mutação que o separa:** tirar o
/// `mesh_rebuilt()` do desfazer deixa a malha certa na memória e a GPU a mostrar
/// a de antes. O artista aperta Ctrl+Z e **nada acontece na tela** — que é a
/// forma do report, e a razão de o defeito ter passado por *"não tem undo"* em
/// vez de *"o undo desfaz a coisa errada"*.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn undoing_a_filter_tells_the_screen() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Mask);

    assert!(s.arm_filter(), "a fixture nao conseguiu armar o filtro");
    s.filter_kind = FilterKind::Inflate;
    assert!(s.begin_filter(400.0), "o begin recusou");
    s.filter_at(400.0 + DRAG_PX);
    s.close_stroke();

    // A subida do gesto já aconteceu; o que se mede é a do DESFAZER.
    s.objects[s.active].uploaded = true;
    let edits = s.edits;

    assert!(s.undo_stroke(), "o gesto nao gravou passo de undo");
    assert!(
        !s.objects[s.active].uploaded,
        "o desfazer nao invalidou o upload: a malha voltou na memoria e a tela continua a mostrar \
         a de antes"
    );
    assert_ne!(
        s.edits, edits,
        "o desfazer nao contou como edicao: a doacao ao Painter serve um carimbo velho"
    );
}
