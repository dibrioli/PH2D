//! Gates da câmera orbital — headless, sem GPU.

use super::*;
use ph2d_mesh::Aabb;

fn boxy(min: [f32; 3], max: [f32; 3]) -> Aabb {
    Aabb { min, max }
}

/// Projeta um ponto de mundo para NDC. `None` se ficar atrás da câmera.
fn to_ndc(cam: &Camera3d, aspect: f32, p: Vec3) -> Option<Vec3> {
    let clip = cam.view_proj(aspect) * p.extend(1.0);
    (clip.w > 0.0).then(|| clip.truncate() / clip.w)
}

/// A quina `i` da caixa, na convenção de bits do `frame`.
fn corner(b: Aabb, i: usize) -> Vec3 {
    Vec3::new(
        if i & 1 == 0 { b.min[0] } else { b.max[0] },
        if i & 2 == 0 { b.min[1] } else { b.max[1] },
        if i & 4 == 0 { b.min[2] } else { b.max[2] },
    )
}

/// O maior |x| e |y| em NDC entre as oito quinas — quão perto do fim da tela o
/// modelo chega.
fn worst_ndc(cam: &Camera3d, aspect: f32, b: Aabb) -> f32 {
    (0..8)
        .map(|i| {
            let ndc = to_ndc(cam, aspect, corner(b, i)).expect("quina atrás da câmera");
            assert!((0.0..=1.0).contains(&ndc.z), "quina fora dos planos: {ndc}");
            ndc.x.abs().max(ndc.y.abs())
        })
        .fold(0.0f32, f32::max)
}

/// ⚠️ **O gate que justifica a conta do `frame` — e ele é DOIS gates num, por
/// um erro que eu cometi.** A primeira versão afirmava só *toda quina cai dentro
/// do frustum*, e passava sobre um enquadramento que desperdiçava `√3`: o modelo
/// ocupava 16,9% da tela e o gate ficava verde, porque **contenção não vê
/// folga** — qualquer distância grande o bastante a satisfaz. Agora ele afirma
/// as duas metades: cabe **e** encosta.
///
/// A varredura de aspecto é obrigatória porque é a janela ESTREITA que a versão
/// só-vertical corta: o modo de falha aparece de lado, não de frente.
#[test]
fn framing_fits_the_bounds_and_actually_fills_the_screen() {
    let b = boxy([-1.0, -0.5, -2.0], [3.0, 4.0, 1.0]);
    for aspect in [0.25f32, 0.5, 1.0, 1.7778, 4.0] {
        for (yaw, pitch) in [(0.0, 0.0), (1.2, 0.7), (-2.5, -1.1), (3.0, 1.4)] {
            let mut cam = Camera3d {
                yaw,
                pitch,
                fov_y: core::f32::consts::FRAC_PI_4,
                ..Camera3d::default()
            };
            cam.frame(b, aspect);
            let worst = worst_ndc(&cam, aspect, b);
            assert!(
                worst <= 1.0,
                "aspect {aspect}, yaw {yaw}: o modelo saiu da tela ({worst:.3})"
            );
            assert!(
                worst > 0.80,
                "aspect {aspect}, yaw {yaw}: sobra tela demais ({worst:.3}) — \
                 o enquadramento está frouxo, e é assim que o desperdicio de √3 passou"
            );
        }
    }
}

/// ⚠️ **O preço da escolha, pinado.** O enquadramento é do ângulo ATUAL, então
/// girar depois pode encostar a caixa na borda — é o mesmo comportamento do
/// *frame selected* do Blender, e o trade contra a versão invariante que
/// desperdiçava `√3`. O gate afirma o limite: mesmo no pior giro, a folga
/// perdida é modesta.
#[test]
fn orbiting_after_a_fit_can_crop_a_little_and_this_is_the_documented_price() {
    let b = boxy([-1.0, -0.5, -2.0], [3.0, 4.0, 1.0]);
    let mut cam = Camera3d {
        fov_y: core::f32::consts::FRAC_PI_4,
        ..Camera3d::default()
    };
    cam.frame(b, 1.0);
    let mut worst = 0.0f32;
    for k in 0..32 {
        cam.yaw = k as f32 * 0.2;
        cam.pitch = (k as f32 * 0.11).sin();
        worst = worst.max(worst_ndc(&cam, 1.0, b));
    }
    assert!(
        worst < 1.6,
        "girar depois do fit cortou demais ({worst:.2}) — o fit está apertado \
         demais para a caixa mais desfavorável"
    );
}

/// O alvo pousa no CENTRO da tela — é isso que "orbitar em torno de" significa,
/// e é o que se vê quebrar quando a matriz de vista está errada.
#[test]
fn the_target_lands_at_the_centre_of_the_screen() {
    let mut cam = Camera3d::framing(boxy([-1.0; 3], [1.0; 3]), 0.8, 1.6);
    for (yaw, pitch) in [(0.0, 0.0), (2.0, -1.0), (-1.3, 1.2)] {
        cam.yaw = yaw;
        cam.pitch = pitch;
        let ndc = to_ndc(&cam, 1.6, cam.target).unwrap();
        assert!(ndc.x.abs() < 1e-4 && ndc.y.abs() < 1e-4, "alvo em {ndc}");
    }
}

/// A distância ao alvo é invariante da órbita: girar não aproxima.
#[test]
fn orbiting_never_changes_the_distance_to_the_target() {
    let mut cam = Camera3d::default();
    let d0 = (cam.eye() - cam.target).length();
    for _ in 0..50 {
        cam.orbit(0.37, 0.21);
        let d = (cam.eye() - cam.target).length();
        assert!((d - d0).abs() < 1e-3, "{d} contra {d0}");
    }
}

/// ⚠️ O polo é o caso degenerado que produz `NaN` numa `look_at`, e o clamp
/// existe para torná-lo **inalcançável**. Mil passos para cima não chegam lá.
#[test]
fn the_pitch_never_reaches_the_pole_so_the_view_matrix_never_degenerates() {
    let mut cam = Camera3d::default();
    for _ in 0..1000 {
        cam.orbit(0.0, 0.5);
    }
    assert!(cam.pitch < core::f32::consts::FRAC_PI_2);
    assert!(
        cam.view().is_finite(),
        "a matriz de vista virou NaN no polo"
    );
    for _ in 0..2000 {
        cam.orbit(0.0, -0.5);
    }
    assert!(cam.pitch > -core::f32::consts::FRAC_PI_2);
    assert!(cam.view().is_finite());
}

/// O zoom é MULTIPLICATIVO: o mesmo gesto vale a mesma razão em qualquer
/// distância. Um passo aditivo daria saltos longe e paralisia perto.
#[test]
fn the_dolly_is_multiplicative_so_the_gesture_feels_the_same_at_any_distance() {
    let mut near = Camera3d {
        distance: 0.1,
        ..Camera3d::default()
    };
    let mut far = Camera3d {
        distance: 100.0,
        ..Camera3d::default()
    };
    let (n0, f0) = (near.distance, far.distance);
    near.dolly(3.0);
    far.dolly(3.0);
    let (rn, rf) = (near.distance / n0, far.distance / f0);
    assert!((rn - rf).abs() < 1e-4, "razões {rn} e {rf} divergiram");
    assert!(rn < 1.0, "dolly positivo tem de APROXIMAR");
}

/// A distância nunca vira zero nem negativa — os dois colapsam a câmera para
/// dentro do modelo e a matriz de projeção junto.
#[test]
fn the_distance_stays_positive_however_hard_the_wheel_is_spun() {
    let mut cam = Camera3d::default();
    for _ in 0..10_000 {
        cam.dolly(10.0);
    }
    assert!(cam.distance > 0.0, "distância {}", cam.distance);
    assert!(cam.view().is_finite());
    for _ in 0..10_000 {
        cam.dolly(-10.0);
    }
    assert!(cam.distance.is_finite());
}

/// O pan move o alvo no PLANO DA TELA: puxar para a direita move o modelo para
/// a direita, sem componente na direção da vista.
#[test]
fn panning_slides_the_target_across_the_screen_plane_only() {
    let mut cam = Camera3d {
        yaw: 0.9,
        pitch: 0.3,
        ..Camera3d::default()
    };
    let before = cam.target;
    let forward = (cam.target - cam.eye()).normalize();
    cam.pan(0.25, 0.0);
    let delta = cam.target - before;
    assert!(delta.length() > 1e-3, "o pan não moveu nada");
    assert!(
        delta.dot(forward).abs() < 1e-4,
        "o pan andou {} na direção da vista",
        delta.dot(forward)
    );
    // Na tela, o alvo saiu do centro para a ESQUERDA (o modelo foi para a direita).
    let ndc = to_ndc(&cam, 1.0, before).unwrap();
    assert!(
        ndc.x > 0.0,
        "o mundo devia ter ido para a direita, ndc {ndc}"
    );
}

/// O pan é proporcional à distância: perto move pouco de mundo, longe move
/// muito — em fração de TELA, o gesto é o mesmo.
#[test]
fn the_pan_is_measured_in_screen_fractions_not_world_units() {
    let mut near = Camera3d {
        distance: 1.0,
        ..Camera3d::default()
    };
    let mut far = Camera3d {
        distance: 10.0,
        ..Camera3d::default()
    };
    let (n0, f0) = (near.target, far.target);
    near.pan(0.1, 0.0);
    far.pan(0.1, 0.0);
    let ratio = (far.target - f0).length() / (near.target - n0).length();
    assert!((ratio - 10.0).abs() < 0.05, "razão {ratio}, esperada ~10");
}

/// Enquadrar uma caixa vazia não faz nada — não há o que enquadrar, e um
/// `NaN` daqui envenenaria a matriz de vista do frame inteiro.
#[test]
fn framing_an_empty_box_leaves_the_camera_alone() {
    let before = Camera3d::default();
    let mut cam = before;
    cam.frame(Aabb::EMPTY, 1.5);
    assert_eq!(cam, before);
    assert!(cam.view_proj(1.5).is_finite());
}

/// ⚠️ **O gate que uma mutação sobrevivente pediu.** Trocar `perspective_rh`
/// pela variante `_gl` passou por TODOS os gates de GPU: no enquadramento
/// normal o modelo fica longe do plano near, onde os dois mapeamentos caem
/// ambos dentro de `[0,1]`, e a imagem sai idêntica. O defeito só aparece
/// **perto** — o clip do wgpu é `z ∈ [0,1]` e o do GL é `[-1,1]`, então metade
/// da faixa de profundidade cai atrás do near e some. É o gesto de aproximar
/// para trabalhar um detalhe, que é o gesto central de esculpir.
///
/// A afirmação é sobre a CONVENÇÃO, direto: o plano near mapeia em `z = 0`.
#[test]
fn the_projection_uses_the_wgpu_clip_range_so_close_geometry_is_not_clipped_away() {
    let cam = Camera3d::default();
    let (near, far) = cam.clip_planes();
    let fwd = (cam.target - cam.eye()).normalize();

    let at_near = to_ndc(&cam, 1.0, cam.eye() + fwd * near).unwrap();
    assert!(
        at_near.z.abs() < 1e-3,
        "o plano near tem de cair em z=0 (wgpu), e caiu em {} — \
         a projeção está na convenção do OpenGL e corta o que estiver perto",
        at_near.z
    );
    let at_far = to_ndc(&cam, 1.0, cam.eye() + fwd * far).unwrap();
    assert!((at_far.z - 1.0).abs() < 1e-3, "o far caiu em {}", at_far.z);

    // E o meio do caminho fica ESTRITAMENTE dentro — nada de geometria útil
    // nascendo fora do volume.
    let mid = to_ndc(&cam, 1.0, cam.target).unwrap();
    assert!((0.0..1.0).contains(&mid.z), "o alvo caiu em z={}", mid.z);
}

/// Os planos de corte acompanham a distância, então a RAZÃO entre eles — que é
/// de onde vem a precisão do depth-buffer — é constante em qualquer zoom.
#[test]
fn the_clip_planes_follow_the_distance_so_depth_precision_is_scale_free() {
    let mut ratios = Vec::new();
    for d in [0.01f32, 1.0, 1000.0] {
        let cam = Camera3d {
            distance: d,
            ..Camera3d::default()
        };
        let (near, far) = cam.clip_planes();
        assert!(near > 0.0 && far > near);
        assert!(near < d && far > d, "o modelo tem de caber entre os planos");
        ratios.push(far / near);
    }
    let first = ratios[0];
    for r in &ratios {
        assert!((r - first).abs() / first < 1e-3, "razões {ratios:?}");
    }
}
