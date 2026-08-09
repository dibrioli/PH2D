//! **O GIRO DO TRANSFORM, medido e gateado no gesto REAL.**
//!
//! Módulo irmão de teste do [`super`] (`#[path]`, `cfg(test)`), no molde do
//! `sculpt3d_bake_gesture_tests`: dirige o gesto do produto (`arm_transform` →
//! `begin_transform` → `transform_at`) e julga o resultado **na tela**, que é
//! onde o report do Enio foi feito — *"a direção da rotação do mouse está
//! invertida em relação à rot do objeto e é imprecisa (não consistente)"*.
//!
//! ⚠️ **O oráculo é a PROJEÇÃO de vértices, nunca um número interno.** Um gate
//! que lesse o `radians` do [`ph2d_sculpt3d::Gesture`] afirmaria que a medição
//! mediu o que ela mesma mediu — a forma de espelho que este repo já pagou
//! várias vezes. O que o artista vê é para onde o barro **girou**, então é isso
//! que se mede: o ângulo de tela que um vértice percorre em torno do pivô
//! projetado, contra o ângulo que o DEDO percorreu em torno do mesmo ponto.
//!
//! ## O que estas medições encontraram
//!
//! Com o eixo tirado do raio de PEN-DOWN e a varredura medida a partir do
//! pen-down (a v1 da W15):
//!
//! | dedo | peça | razão |
//! |---|---|---|
//! | 30° | −9,24° | **−0,308×** |
//! | 90° | −37,08° | **−0,412×** |
//! | 180° | −80,85° | **−0,449×** |
//!
//! Três defeitos numa tabela só: o **sinal** (o barro vai para o outro lado), a
//! **magnitude** (metade do que a mão pediu) e a **inconsistência** (a razão
//! muda com a amplitude, então não há um número que o artista aprenda). Com o
//! eixo pela reta olho→pivô e a varredura em torno do pivô projetado, armada no
//! pen-down: **1,000× em 30°, 90°, 180° e 360°**.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::transform::tests -- --ignored --nocapture
//! ```

use ph2d_mesh::shapes::uv_sphere;
use ph2d_sculpt3d::{MaskTransform, TransformKind};

use super::super::Sculpt3dScene;

const VW: u32 = 900;
const VH: u32 = 700;

/// O raio, em pixels, do círculo que o dedo percorre em volta do pivô.
///
/// Grande o bastante para a zona morta do acumulador (`TWIST_DEADZONE_PX`) não
/// entrar na conta, e pequeno o bastante para caber na viewport.
const DRAG_R: f32 = 220.0;

/// De quantos em quantos graus o dedo é amostrado.
///
/// ⚠️ **Ele é o tamanho do erro que o gate consegue VER**, e por isso não é
/// pequeno: com 5° por passo, um gesto que perdesse o primeiro incremento
/// entregaria 85° para 90° de dedo — 5,6%, folgado acima da tolerância. Um passo
/// de 0,5° tornaria esse defeito indistinguível de ruído.
const STEP_DEG: f32 = 5.0;

fn aspect() -> f32 {
    VW as f32 / VH as f32
}

/// Uma cena com uma esfera, a câmera enquadrada nela **e o pivô FORA do centro
/// da tela**.
///
/// ⚠️ **O pan é a metade da fixture que CONTÉM o fenômeno, e a primeira versão
/// não o tinha.** Enquadrada, a câmera põe o centroide no centro da imagem — e
/// ali a reta olho→pivô **é** o eixo óptico, que induz uma rotação de imagem
/// EXATA. O espalhamento entre quarenta vértices media `0,000°`, o que parece um
/// resultado esplêndido e é o gate a medir o caso em que toda escolha de eixo
/// paralela à vista acerta. Deslocado, os dois deixam de coincidir, e é aí que
/// um eixo errado tem para onde errar.
///
/// ⚠️ **E é também o caso REAL:** o pivô é o centroide da parte LIVRE, que sob
/// uma máscara macia quase nunca cai no meio da peça — quanto mais no meio da
/// tela, depois de o artista orbitar e deslocar a vista.
fn scene(device: &wgpu::Device) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, uv_sphere(48, 72, 1.0), aspect());
    // O `render` é quem publica o viewport no produto; isto aqui não desenha.
    s.viewport = (VW, VH);
    s.frame_all(aspect());
    // Fração da ALTURA da viewport (ver [`Camera3d::pan`]) — o suficiente para o
    // pivô sair umas duzentas colunas do centro, sem tirar a peça de quadro.
    s.camera.pan(0.28, 0.16);
    s
}

/// O ângulo de um ponto de tela em torno de `c`, no referencial que o
/// `swept_angle_about` usa: **`y` para CIMA**, anti-horário positivo.
fn screen_angle(c: (f32, f32), p: (f32, f32)) -> f32 {
    (c.1 - p.1).atan2(p.0 - c.0)
}

/// A diferença de dois ângulos, trazida para `(-pi, pi]`.
fn wrap(mut d: f32) -> f32 {
    while d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    while d <= -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    d
}

/// O pixel a `deg` graus no círculo de arrasto — **anti-horário na tela**, ou
/// seja `y` de janela DIMINUINDO (a janela cresce para baixo).
fn finger(c: (f32, f32), deg: f32) -> (f32, f32) {
    let t = deg.to_radians();
    (c.0 + DRAG_R * t.cos(), c.1 - DRAG_R * t.sin())
}

/// O pivô que o gesto vai usar, projetado.
///
/// ⚠️ Ele é função só da malha e da máscara, então é conhecível **antes** do
/// pen-down — que é o que deixa a fixture escolher um pixel de pen-down com raio
/// conhecido em volta dele. É a mesma porta que o produto usa.
fn pivot_on_screen(s: &Sculpt3dScene) -> (f32, f32) {
    let pivot_local = MaskTransform::begin(s.mesh())
        .expect("a esfera inteira está livre")
        .pivot();
    let world = s.pose().point_to_world(pivot_local);
    s.camera
        .project(world, s.viewport)
        .expect("o pivô está na tela")
}

/// Onde um vértice cai na tela, agora.
fn vert_on_screen(s: &Sculpt3dScene, v: usize) -> (f32, f32) {
    let w = s.pose().point_to_world(s.mesh().positions()[v]);
    s.camera.project(w, s.viewport).expect("na tela")
}

/// Os vértices com braço de alavanca suficiente para resolver um giro, espalhados
/// em torno de `c`.
///
/// ⚠️ **Espalhados, e não os N maiores:** os maiores braços de uma esfera caem
/// todos no mesmo anel, e um eixo inclinado gira lados OPOSTOS da silhueta por
/// ângulos diferentes — é essa discordância que o gate do eixo mede, e uma
/// amostra de um lado só seria cega a ela.
fn watched(s: &Sculpt3dScene, c: (f32, f32), want: usize) -> Vec<usize> {
    let mut far: Vec<usize> = (0..s.mesh().vert_count())
        .filter(|&v| {
            let p = vert_on_screen(s, v);
            (p.0 - c.0).hypot(p.1 - c.1) > DRAG_R * 0.4
        })
        .collect();
    let stride = (far.len() / want.max(1)).max(1);
    far = far.into_iter().step_by(stride).take(want).collect();
    assert!(far.len() >= want / 2, "a fixture não tem braço de alavanca");
    far
}

/// **Dirige o gesto REAL** e devolve, por vértice observado, o ângulo de tela
/// que ele percorreu em torno de `c` — **acumulado passo a passo**.
///
/// ⚠️ **Acumulado, e a razão é um defeito que esta medição já teve:** um ângulo
/// lido de uma foto única vive em `(-180°, 180°]`, e a primeira versão desta
/// fixture mediu 180° de dedo e imprimiu `−180°` — o mesmo ângulo com o sinal do
/// arredondamento, ou seja verde e vermelho indistinguíveis exatamente na
/// amplitude mais interessante.
fn drag_and_measure(
    s: &mut Sculpt3dScene,
    c: (f32, f32),
    sweep_deg: f32,
    watch: &[usize],
) -> Vec<f32> {
    let mut seen: Vec<f32> = watch
        .iter()
        .map(|&v| screen_angle(c, vert_on_screen(s, v)))
        .collect();
    let mut acc = vec![0.0f32; watch.len()];

    s.arm_transform(TransformKind::Rotate);
    let start = finger(c, 0.0);
    assert!(s.begin_transform(start.0, start.1), "a sessão abre");

    let steps = (sweep_deg / STEP_DEG).ceil() as i32;
    for k in 1..=steps {
        let p = finger(c, sweep_deg * (k as f32 / steps as f32));
        s.transform_at(p.0, p.1);
        for (i, &v) in watch.iter().enumerate() {
            let now = screen_angle(c, vert_on_screen(s, v));
            acc[i] += wrap(now - seen[i]);
            seen[i] = now;
        }
    }
    acc.iter_mut().for_each(|a| *a = a.to_degrees());
    acc
}

/// Abre a GPU, ou diz que não há nada a afirmar.
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

// ---------------------------------------------------------------- sondas ----

/// **O DEDO VARRE, A PEÇA GIRA — quanto, e para que lado.**
#[test]
#[ignore = "sonda: requer um adapter de GPU; rode com --ignored --nocapture"]
fn measure_what_the_finger_asks_and_what_the_piece_does() {
    let gpu = gpu_or_skip!();
    println!("\n=== O GIRO: o dedo pede, a peça faz ===");
    println!("(dedo ANTI-HORÁRIO na tela; peça medida pela PROJEÇÃO de um vértice)\n");
    println!("  sweep do dedo | giro da peça |   razão  | veredito");
    println!("  --------------+--------------+----------+----------------------------");
    for sweep in [30.0f32, 90.0, 180.0, 360.0] {
        let mut s = scene(&gpu.device);
        let c = pivot_on_screen(&s);
        let watch = watched(&s, c, 1);
        let piece = drag_and_measure(&mut s, c, sweep, &watch)[0];
        let ratio = piece / sweep;
        let verdict = if ratio < 0.0 {
            "INVERTIDO"
        } else if (ratio - 1.0).abs() > 0.05 {
            "mesmo sentido, magnitude errada"
        } else {
            "segue o dedo"
        };
        println!("  {sweep:>11.0}° | {piece:>11.2}° | {ratio:>7.3}× | {verdict}");
    }
    println!("\n  (razão 1,000× = manipulação direta: o barro acompanha a mão volta por volta)\n");
}

/// **O EIXO: quanto ele se inclina quando o pen-down sai de cima do pivô.**
///
/// O gesto gira em torno de uma reta que passa pelo pivô, e só uma direção faz o
/// giro parecer uma rotação na tela: a da reta que liga o pivô ao OLHO — ela
/// projeta num PONTO, então girar em volta dela deixa a silhueta a rodar em
/// torno daquele ponto. Qualquer outra projeta num segmento, e o que se vê é a
/// peça **cambalhotando**.
#[test]
#[ignore = "sonda: requer um adapter de GPU; rode com --ignored --nocapture"]
fn measure_how_far_the_pen_down_ray_tilts_from_the_eye_pivot_line() {
    let gpu = gpu_or_skip!();
    let s = scene(&gpu.device);
    let c = pivot_on_screen(&s);
    let pivot_world = {
        let l = MaskTransform::begin(s.mesh()).expect("livre").pivot();
        s.pose().point_to_world(l)
    };
    let eye = s.camera.eye();
    let ideal = {
        let v = [
            pivot_world[0] - eye.x,
            pivot_world[1] - eye.y,
            pivot_world[2] - eye.z,
        ];
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / l, v[1] / l, v[2] / l]
    };
    println!("\n=== O EIXO: inclinação do raio de pen-down ===");
    println!("  offset do pivô | inclinação vs a reta olho→pivô");
    println!("  ---------------+-------------------------------");
    for off in [0.0f32, 50.0, 120.0, 220.0, 340.0] {
        let d = s.ray_at(c.0 + off, c.1).dir();
        let dot = (d[0] * ideal[0] + d[1] * ideal[1] + d[2] * ideal[2]).clamp(-1.0, 1.0);
        println!("  {off:>13.0}px | {:>10.2}°", dot.acos().to_degrees());
    }
    println!("\n  (um eixo inclinado gira a peça PARA FORA do plano da tela)\n");
}

/// **A DISCORDÂNCIA entre vértices** — é ela que dá a barra do gate do eixo.
#[test]
#[ignore = "sonda: requer um adapter de GPU; rode com --ignored --nocapture"]
fn measure_how_much_the_screen_rotation_disagrees_between_vertices() {
    let gpu = gpu_or_skip!();
    println!("\n=== A DISCORDÂNCIA: o mesmo giro visto em pontos diferentes ===\n");
    println!("  sweep | menor giro | maior giro | espalhamento");
    println!("  ------+------------+------------+-------------");
    for sweep in [90.0f32, 180.0] {
        let mut s = scene(&gpu.device);
        let c = pivot_on_screen(&s);
        let watch = watched(&s, c, 40);
        let a = drag_and_measure(&mut s, c, sweep, &watch);
        let lo = a.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = a.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        println!(
            "  {sweep:>4.0}° | {lo:>9.3}° | {hi:>9.3}° | {:>10.3}°",
            hi - lo
        );
    }
    println!(
        "\n  (um eixo que não passa pelo olho gira lados opostos da silhueta por\n   \
         ângulos diferentes — é isso que se lê como cambalhota)\n"
    );
}

// ----------------------------------------------------------------- gates ----

/// **A PEÇA ACOMPANHA A MÃO, VOLTA POR VOLTA.**
///
/// O gate central da correção, e ele afirma as TRÊS metades do report de uma vez:
/// o **sinal** (razão positiva), a **magnitude** (1,0×) e a **consistência** (o
/// mesmo 1,0× em quatro amplitudes, inclusive além de meia volta).
///
/// ⚠️ **Ele nasceu VERMELHO sobre o produto de ontem**, com os números na tabela
/// do cabeçalho deste arquivo: −0,308× · −0,412× · −0,449×. Cada uma das três
/// mutações o mata sozinha — eixo virado para dentro da tela ⇒ razão negativa ·
/// varredura medida do pen-down ⇒ ~0,45× · varredura não armada no pen-down ⇒
/// um passo a menos, 0,944× em 90°.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_piece_turns_with_the_finger_turn_for_turn() {
    let gpu = gpu_or_skip!();
    for sweep in [30.0f32, 90.0, 180.0, 360.0] {
        let mut s = scene(&gpu.device);
        let c = pivot_on_screen(&s);
        let watch = watched(&s, c, 1);
        let piece = drag_and_measure(&mut s, c, sweep, &watch)[0];
        let ratio = piece / sweep;
        assert!(
            ratio > 0.0,
            "o dedo varreu {sweep:.0}° no anti-horário e a peça girou {piece:.2}° — \
             manipulação INVERTIDA (o eixo aponta para dentro da tela)"
        );
        // ⚠️ **5%, e o número saiu da medição de um fenômeno REAL:** com o pivô
        // fora do centro o giro na tela é uma homografia da rotação no mundo,
        // não uma rotação — um vértice longe do pivô percorre entre 0,984× e
        // 1,006× do que o dedo pediu. A barra é o teto DISSO, e continua muito
        // abaixo de tudo o que ela precisa pegar: 0,45× para a varredura medida
        // do pen-down, 0,83× no arrasto de 30° para a varredura não armada, e
        // negativo para o eixo virado.
        assert!(
            (ratio - 1.0).abs() <= 0.05,
            "o dedo varreu {sweep:.0}° e a peça girou {piece:.2}° ({ratio:.3}×) — \
             o barro não acompanha a mão"
        );
    }
}

/// **O GIRO É NO PLANO DA TELA: toda a silhueta roda pelo mesmo ângulo.**
///
/// O irmão do gate acima, e ele mede a outra metade do eixo. A razão de 1,0×
/// medida num vértice só sobrevive a um eixo levemente inclinado — o que a
/// inclinação faz é girar lados OPOSTOS da peça por ângulos diferentes, que é o
/// que o olho lê como cambalhota. Aqui a asserção é sobre o CONJUNTO.
///
/// ⚠️ **A barra saiu de DOIS números medidos, e fica entre eles**
/// (`measure_how_much_the_screen_rotation_disagrees_between_vertices`): com o
/// eixo pela reta olho→pivô o espalhamento entre quarenta vértices é **3,94°** a
/// 90° — e ele não é zero porque **não pode** ser: com o pivô fora do centro a
/// projeção de uma rotação é uma homografia, não uma rotação. Com o eixo do raio
/// de pen-down, que inclina 13,1° a 220 px, ele é **61,8°**.
///
/// ⚠️ **As DUAS amplitudes são afirmadas, e a de 180° sozinha seria fraca:** ali
/// o eixo certo dá `0,000°` exatos — a meia volta leva todo azimute ao antípoda
/// e a distorção da perspectiva cancela —, e um gate que só medisse esse ponto
/// estaria a medir uma degenerescência. A de 90° é a que carrega o fenômeno; a
/// de 180° entra porque é onde o eixo errado erra mais (102,6°).
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_whole_piece_turns_by_the_same_screen_angle() {
    let gpu = gpu_or_skip!();
    for sweep in [90.0f32, 180.0] {
        let mut s = scene(&gpu.device);
        let c = pivot_on_screen(&s);
        let watch = watched(&s, c, 40);
        let a = drag_and_measure(&mut s, c, sweep, &watch);
        let lo = a.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = a.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            hi - lo <= 15.0,
            "o mesmo giro de {sweep:.0}° foi visto entre {lo:.2}° e {hi:.2}° em \
             pontos diferentes da peça ({:.2}° de espalhamento) — o eixo não passa \
             pelo olho, e a peça cambalhota em vez de girar",
            hi - lo
        );
    }
}
