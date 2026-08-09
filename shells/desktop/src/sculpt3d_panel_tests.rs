//! **ARMAR UM PADRÃO SEMEIA, NUNCA IMPÕE** — os gates de `set_alpha_image` (ADR-0150).
//!
//! Módulo irmão de teste do [`super`] (`#[path]`, `cfg(test)`), no molde do `sculpt3d_mode_tests`:
//! `set_alpha_image` é método da CENA, e uma cena exige um device ⇒ `#[ignore]` + `gpu_or_skip!`.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::panel::tests -- --ignored --nocapture
//! ```

use ph2d_mesh::shapes::uv_sphere;
use ph2d_sculpt3d::{Alpha, AlphaImage, Brush};

use super::super::Sculpt3dScene;

/// Abre a GPU, ou diz que não há nada a afirmar. (Cópia local dos irmãos: um macro exportado entre
/// módulos de teste seria acoplamento por conveniência.)
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

/// Um "mármore": bandas DIAGONAIS, para linha e coluna serem distinguíveis.
///
/// ⚠️ Uma imagem de bandas de UM eixo deixaria os dois gates de eixo verdes por vácuo — a
/// degeneração que eles medem é *uma das coordenadas do plano nunca varia*, e um padrão que já é
/// constante naquele eixo não pode revelá-la.
fn marble() -> AlphaImage {
    let (w, h) = (64usize, 64usize);
    let rgba: Vec<u8> = (0..w * h)
        .flat_map(|i| {
            let (x, y) = ((i % w) as f32, (i / w) as f32);
            #[expect(clippy::cast_possible_truncation, reason = "gerando bytes de fixture")]
            #[expect(clippy::cast_sign_loss, reason = "o seno deslocado é sempre positivo")]
            let v = (((x * 0.19 + y * 0.07).sin() * 0.5 + 0.5) * 255.0) as u8;
            [v, v, v, 255]
        })
        .collect();
    #[expect(clippy::cast_possible_truncation, reason = "64 cabe em u32")]
    AlphaImage::from_rgba(w as u32, h as u32, &rgba).expect("a fixture descreve uma imagem")
}

/// Quantas linhas e colunas DISTINTAS a fatia `z = 0` — a que o swatch do painel desenha — produz.
///
/// ⚠️ **Esta é a mesma pergunta que o preview do painel faz**, feita aqui sobre a porta pública
/// (`Brush::alpha_weight` + `Brush::alpha_frame`): o swatch amostra `weight_at([px, py, 0.0], …)`,
/// e é por o campo ser CONSTANTE ao longo de `z` (com o eixo encarando a vista) que essa fatia é
/// exatamente o que todo ponto do modelo recebe.
fn slice_variety(b: &Brush) -> (usize, usize) {
    let f = b.alpha_frame();
    let n = 32usize;
    let grid: Vec<Vec<u8>> = (0..n)
        .map(|r| {
            (0..n)
                .map(|c| {
                    #[expect(clippy::cast_precision_loss, reason = "32 cabe em f32")]
                    let px = -1.0 + 2.0 * (c as f32 + 0.5) / n as f32;
                    #[expect(clippy::cast_precision_loss, reason = "32 cabe em f32")]
                    let py = 1.0 - 2.0 * (r as f32 + 0.5) / n as f32;
                    #[expect(clippy::cast_possible_truncation, reason = "peso clampado em [0,1]")]
                    #[expect(clippy::cast_sign_loss, reason = "peso clampado em [0,1]")]
                    let v = (b.alpha_weight([px, py, 0.0], &f).clamp(0.0, 1.0) * 255.0) as u8;
                    v
                })
                .collect()
        })
        .collect();
    let rows: std::collections::BTreeSet<_> = grid.iter().cloned().collect();
    let cols: std::collections::BTreeSet<Vec<u8>> = (0..n)
        .map(|c| (0..n).map(|r| grid[r][c]).collect())
        .collect();
    (rows.len(), cols.len())
}

/// **A ESCALA QUE O ARTISTA AUTOROU SOBREVIVE AO BOTÃO** — o report *"as configurações da textura
/// não foram obedecidas"* numa asserção.
///
/// ⚠️ **Semear e impor são coisas diferentes, e a porta irmã sempre soube disso:** o chip do painel
/// (`ph2d-panel-sculpt3d/src/event.rs`) só semeia enquanto o valor ainda é o
/// [`ph2d_sculpt3d::DEFAULT_ALPHA_SCALE`], que é a razão de aquela constante existir — o doc dela
/// diz que o trabalho REAL dela é ser **sentinela**. Este botão escrevia incondicionalmente.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn arming_an_image_keeps_the_scale_the_artist_authored() {
    let gpu = gpu_or_skip!();
    let mut scene = Sculpt3dScene::new(&gpu.device, uv_sphere(24, 36, 1.0), 1.0);

    // O artista escolheu um tamanho de poro. Um número que NÃO é o sentinela.
    let chosen = 0.123_45_f32;
    scene.brush.alpha_scale = chosen;
    let after = scene.set_alpha_image(marble());
    assert!(
        (after - chosen).abs() < 1e-6,
        "armar um padrao descartou a escala autorada ({chosen} -> {after}): o botao IMPOE onde \
         devia SEMEAR, e a porta irma (o chip) obedece ao sentinela"
    );

    // E a metade oposta: intocado, ele SEMEIA — senão o padrão nasce num tamanho que este modelo
    // não comporta, o defeito que um smoke já reprovou (*"os poros são gigantescos"*).
    let mut fresh = Sculpt3dScene::new(&gpu.device, uv_sphere(24, 36, 1.0), 1.0);
    assert!(
        (fresh.brush.alpha_scale - ph2d_sculpt3d::DEFAULT_ALPHA_SCALE).abs() < 1e-6,
        "a cena nao nasce no sentinela — o resto deste gate nao afirma nada"
    );
    let seeded = fresh.set_alpha_image(marble());
    assert!(
        (seeded - ph2d_sculpt3d::DEFAULT_ALPHA_SCALE).abs() > 1e-6,
        "a semeadura nao disparou num pincel intocado: o padrao nasce num tamanho que a malha \
         nao comporta"
    );
}

/// **UM CARIMBO ENCARA QUEM O APLICA** — o report *"o Preview no painel ainda não está correto"*,
/// e o modelo junto.
///
/// ⚠️ **UM número respondia DUAS perguntas.** Para os nove padrões procedurais o eixo diz *para que
/// lado os estratos empilham*, e o `elev = 0` de fábrica é o certo (o doc do `Brush::default`
/// explica: com `elev = 90` o artista veria uma camada só). Para uma IMAGEM o eixo é a **direção de
/// PROJEÇÃO** — `Alpha::Image` lê `q[0]`/`q[1]`, o plano perpendicular a ele —, então o mesmo
/// `elev = 0` projeta o carimbo **de lado** e ele degenera em listras.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn arming_an_image_turns_the_axis_to_face_the_view() {
    let gpu = gpu_or_skip!();
    let mut scene = Sculpt3dScene::new(&gpu.device, uv_sphere(24, 36, 1.0), 1.0);
    let factory = Brush::default();
    assert_eq!(
        (
            scene.brush.alpha_az_deg,
            scene.brush.alpha_elev_deg,
            factory.alpha_elev_deg,
        ),
        (factory.alpha_az_deg, factory.alpha_elev_deg, 0),
        "a premissa deste gate e' que a cena nasce no eixo de FABRICA, e que ele deita no plano"
    );

    // CONTROLE: com o eixo de fábrica, a fatia que o swatch desenha é degenerada.
    let mut deitado = Brush {
        alpha_scale: 0.5,
        ..factory.clone()
    };
    deitado.alpha = Some(Alpha::Image(std::sync::Arc::new(marble())));
    let (rows_flat, _) = slice_variety(&deitado);
    assert!(
        rows_flat <= 4,
        "o CONTROLE nao contem o fenomeno: com o eixo deitado a fatia deveria colapsar em poucas \
         linhas distintas, e mediu {rows_flat}"
    );

    // E o que o botão arma resolve a imagem inteira.
    scene.set_alpha_image(marble());
    scene.brush.alpha_scale = 0.5;
    let (rows, cols) = slice_variety(&scene.brush);
    assert!(
        rows > 8 && cols > 8,
        "armar uma imagem deixou o eixo DEITADO: a fatia que o painel desenha (e que o modelo \
         recebe) colapsou em {rows} linhas x {cols} colunas distintas — as listras do report"
    );
}

/// **MAS UM EIXO AUTORADO SOBREVIVE** — a mesma lei do sentinela da escala, no outro campo.
///
/// ⚠️ Sem esta metade a semeadura seria imposição, e um artista que apontou o carimbo de propósito
/// veria a escolha dele evaporar ao trocar de imagem.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn an_authored_axis_survives_arming_another_image() {
    let gpu = gpu_or_skip!();
    let mut scene = Sculpt3dScene::new(&gpu.device, uv_sphere(24, 36, 1.0), 1.0);
    scene.brush.alpha_az_deg = 33;
    scene.brush.alpha_elev_deg = 12;
    scene.set_alpha_image(marble());
    assert_eq!(
        (scene.brush.alpha_az_deg, scene.brush.alpha_elev_deg),
        (33, 12),
        "armar um padrao reescreveu um eixo AUTORADO: semear virou impor"
    );
}
