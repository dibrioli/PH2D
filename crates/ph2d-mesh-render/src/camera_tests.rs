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

#[test]
fn the_ray_through_a_pixel_lands_where_that_pixel_draws() {
    // O oráculo do pick: pegue um ponto do mundo, PROJETE-o com a mesma
    // `view_proj` que o renderer usa, converta para pixel, e dispare o raio de
    // volta por esse pixel — ele tem de passar pelo ponto. É este ida-e-volta
    // que prende o cursor à imagem; sem ele, o raio e a projeção podem discordar
    // por um ângulo pequeno e constante, e o sintoma é *"o pincel pinta ao lado
    // de onde eu aponto"* — que nenhum teste de nenhuma das duas metades vê.
    let size = (1280u32, 720u32);
    let aspect = f32::from(size.0 as u16) / f32::from(size.1 as u16);
    let cam = Camera3d {
        target: glam::Vec3::new(0.3, -0.2, 0.1),
        distance: 4.0,
        yaw: 0.9,
        pitch: -0.35,
        fov_y: core::f32::consts::FRAC_PI_4,
    };
    let _ = aspect;
    let mut checked = 0;
    for k in 0..24 {
        let t = k as f32 / 23.0;
        let p = glam::Vec3::new(-1.0 + 2.0 * t, (t * 7.0).sin() * 0.8, (t * 5.0).cos() * 0.8);
        // ⚠️ Pela PORTA `project`, não por uma conversão escrita aqui: uma
        // segunda conta NDC→pixel no teste é a segunda resposta de que o defeito
        // precisa para nascer, e ela concordaria com o erro em vez de o expor.
        let Some((px, py)) = cam.project(p.into(), size) else {
            continue;
        };
        if !(0.0..size.0 as f32).contains(&px) || !(0.0..size.1 as f32).contains(&py) {
            continue;
        }
        let ray = cam.ray_through(px, py, size);
        // Distância do ponto à reta do raio.
        let o = glam::Vec3::from(ray.origin());
        let d = glam::Vec3::from(ray.dir());
        let w = p - o;
        let perp = w - d * w.dot(d);
        assert!(
            perp.length() < 2e-3,
            "ponto {p:?} caiu a {} do raio do pixel ({px:.1}, {py:.1})",
            perp.length()
        );
        checked += 1;
    }
    assert!(checked > 12, "só {checked} pontos ficaram na tela");
}

#[test]
fn the_ray_through_the_centre_of_the_screen_aims_at_the_target() {
    let cam = Camera3d {
        target: glam::Vec3::new(1.0, 2.0, -3.0),
        distance: 5.0,
        yaw: -1.2,
        pitch: 0.6,
        ..Camera3d::default()
    };
    let size = (800u32, 600u32);
    let ray = cam.ray_through(400.0, 300.0, size);
    let to_target = (cam.target - glam::Vec3::from(ray.origin())).normalize();
    let d = glam::Vec3::from(ray.dir());
    assert!(
        d.dot(to_target) > 0.9999,
        "o centro da tela não mira o alvo: {:?} vs {to_target:?}",
        d
    );
}

/// Um ponto na face do modelo que está voltada para a câmera.
fn front_of_model(cam: &Camera3d, radius: f32) -> glam::Vec3 {
    let dir = (cam.eye() - cam.target).normalize();
    cam.target + dir * radius
}

#[test]
fn dragging_right_turns_the_model_right_and_dragging_down_shows_its_top() {
    // ⚠️ **O gate mede o modelo NA TELA, não o sinal do ângulo.** Argumentar
    // sobre `yaw += dx` é como o erro entrou: `yaw` positivo leva o OLHO para
    // `+X`, e a câmera indo para a direita faz o modelo *parecer* ir para a
    // esquerda. A pergunta que o artista faz é *"o modelo segue a minha mão?"*,
    // e é essa que se afirma aqui — via [`Camera3d::project`], a porta que o
    // produto usa.
    let size = (1000u32, 800u32);
    let base = Camera3d {
        target: glam::Vec3::ZERO,
        distance: 4.0,
        yaw: 0.3,
        pitch: 0.2,
        ..Camera3d::default()
    };
    let mark = front_of_model(&base, 1.0);
    let before = base
        .project(mark.into(), size)
        .expect("o ponto está na tela");

    // Arrastar para a DIREITA: o ponto da frente tem de andar para a direita.
    let mut right = base;
    right.orbit(-0.25, 0.0);
    let after = right.project(mark.into(), size).expect("continua na tela");
    assert!(
        after.0 > before.0 + 20.0,
        "arrastar para a direita moveu o modelo de x={} para x={}",
        before.0,
        after.0
    );

    // Arrastar para BAIXO: vê-se o TOPO, então o ponto da frente desce na tela.
    let mut down = base;
    down.orbit(0.0, 0.25);
    let after = down.project(mark.into(), size).expect("continua na tela");
    assert!(
        after.1 > before.1 + 20.0,
        "arrastar para baixo moveu o modelo de y={} para y={}",
        before.1,
        after.1
    );
}

#[test]
fn project_is_the_exact_inverse_of_ray_through() {
    // As duas portas TÊM de concordar, e o gate as compara direto em vez de
    // reimplementar a projeção no teste — uma segunda conta aqui seria a
    // segunda resposta que o defeito precisa para nascer.
    let size = (1280u32, 720u32);
    let cam = Camera3d {
        target: glam::Vec3::new(0.3, -0.2, 0.1),
        distance: 4.0,
        yaw: 0.9,
        pitch: -0.35,
        fov_y: core::f32::consts::FRAC_PI_4,
    };
    let mut checked = 0;
    for k in 0..40 {
        let (x, y) = ((k % 8) as f32 * 160.0 + 20.0, (k / 8) as f32 * 140.0 + 20.0);
        let ray = cam.ray_through(x, y, size);
        // Um ponto a 3 unidades ao longo do raio tem de projetar de volta no
        // MESMO pixel.
        let p = ray.at(3.0);
        let (bx, by) = cam.project(p, size).expect("à frente do olho");
        assert!(
            (bx - x).abs() < 1e-2 && (by - y).abs() < 1e-2,
            "pixel ({x}, {y}) voltou como ({bx}, {by})"
        );
        checked += 1;
    }
    assert_eq!(checked, 40);
}

/// A câmera de referência dos gates de raio-de-tela: fora dos eixos nos três
/// ângulos, para que nenhum deles seja zero por acidente.
fn oblique() -> Camera3d {
    Camera3d {
        target: glam::Vec3::new(0.2, -0.1, 0.4),
        distance: 5.0,
        yaw: 0.8,
        pitch: -0.3,
        fov_y: core::f32::consts::FRAC_PI_4,
    }
}

/// Os eixos da câmera em mundo — `right` é perpendicular ao eixo de vista, então
/// andar ao longo dele **preserva a profundidade**, que é a premissa do oráculo.
fn screen_axes(cam: &Camera3d) -> (Vec3, Vec3) {
    let axis = (cam.eye() - cam.target).normalize();
    let right = Vec3::Y.cross(axis).normalize();
    (right, axis)
}

/// ⚠️ **O oráculo é o `project`, não a fórmula.** O gate converte `px` pixels em
/// mundo, anda esse tanto na tela, projeta de volta e exige de novo `px` — então
/// ele mede a PROPRIEDADE que a porta promete sem conhecer a conta dela. Uma
/// asserção escrita com `2·d·tan(fov/2)/h` seria a fórmula conferindo a si
/// mesma, e ficaria verde sobre qualquer erro comum às duas cópias.
#[test]
fn a_screen_radius_measures_the_pixels_it_names() {
    let size = (1280u32, 720u32);
    let cam = oblique();
    let (right, _) = screen_axes(&cam);
    let mut checked = 0;
    // Vários pontos, incluindo FORA do eixo da tela: a conversão não pode
    // depender de onde o ponto cai lateralmente, só da profundidade.
    for at in [
        cam.target,
        cam.target + right * 1.2,
        cam.target - right * 0.8 + Vec3::Y * 0.5,
        cam.target + Vec3::Y * 1.5,
    ] {
        for px in [4.0f32, 20.0, 96.0] {
            let r = cam.world_radius_for_screen_px(at.into(), px, size);
            assert!(r > 0.0, "raio nulo em {at} para {px} px");
            let (ax, ay) = cam.project(at.into(), size).expect("à frente");
            let (bx, by) = cam
                .project((at + right * r).into(), size)
                .expect("à frente");
            let moved = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
            assert!(
                (moved - px).abs() < 0.05,
                "{px} px viraram {r} de mundo, que a tela lê como {moved} px"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 12);
}

/// **A ENTREGA da wave, enunciada como propriedade:** o pincel mede o mesmo
/// tanto de TELA a qualquer distância, logo o raio de MUNDO tem de acompanhar a
/// distância. Sem isto, aproximar a câmera faz o pincel engolir o modelo — que é
/// o defeito que o item 6b existe para fechar.
#[test]
fn the_world_radius_tracks_the_distance_so_the_screen_size_holds() {
    let size = (1024u32, 768u32);
    let near = Camera3d {
        distance: 2.0,
        ..oblique()
    };
    let far = Camera3d {
        distance: 8.0,
        ..oblique()
    };
    let at: [f32; 3] = oblique().target.into();
    let rn = near.world_radius_for_screen_px(at, 40.0, size);
    let rf = far.world_radius_for_screen_px(at, 40.0, size);
    let ratio = rf / rn;
    assert!(
        (ratio - 4.0).abs() < 0.01,
        "4x a distância devia dar 4x o raio de mundo, e deu {ratio}x ({rn} → {rf})"
    );
}

/// Um ponto ATRÁS do olho não tem pixel, então não tem conversão. O caso não
/// acontece no produto (um `Hit` está sempre à frente), e a porta não pode
/// devolver um raio negativo por isso.
#[test]
fn a_point_behind_the_eye_has_no_screen_radius() {
    let size = (800u32, 600u32);
    let cam = oblique();
    let behind = cam.eye() + (cam.eye() - cam.target).normalize() * 2.0;
    assert_eq!(
        cam.world_radius_for_screen_px(behind.into(), 30.0, size),
        0.0
    );
}

/// **A BASE DA TELA É ORTONORMAL E SEGUE A ÓRBITA** — a porta que o estêncil do
/// alpha e o Grab compartilham.
///
/// ⚠️ **O gate mede as três propriedades que os dois consumidores usam** —
/// unitária, perpendicular entre si e perpendicular à direção de quem olha —,
/// porque uma base que perdesse qualquer uma delas ainda compilaria e sairia
/// como um carimbo cisalhado que ninguém consegue nomear.
#[test]
fn the_screen_basis_is_orthonormal_and_follows_the_orbit() {
    let mut c = Camera3d::default();
    let len = |v: Vec3| v.length();
    let (r0, u0) = c.screen_basis();
    let axis0 = (c.eye() - c.target).normalize();
    assert!((len(r0) - 1.0).abs() < 1e-5, "a direita não é unitária");
    assert!((len(u0) - 1.0).abs() < 1e-5, "o cima não é unitário");
    assert!(r0.dot(u0).abs() < 1e-5, "a base não é ortogonal");
    assert!(
        r0.dot(axis0).abs() < 1e-5,
        "a direita não está no plano da tela"
    );

    c.orbit(0.7, 0.3);
    let (r1, u1) = c.screen_basis();
    assert!(
        (r1 - r0).length() > 1e-3 || (u1 - u0).length() > 1e-3,
        "a base não acompanhou a órbita — um estêncil preso a ela ficaria preso \
         ao BARRO, que é o oposto do que ele é"
    );
    let axis1 = (c.eye() - c.target).normalize();
    assert!((len(r1) - 1.0).abs() < 1e-5 && r1.dot(axis1).abs() < 1e-5);
}

/// **O GRAB E O ESTÊNCIL LEEM A MESMA TELA** — o gate da porta única.
///
/// ⚠️ Sem ele, a extração que criou a [`Camera3d::screen_basis`] poderia ter
/// deixado o Grab com uma base própria: as duas responderiam à mesma pergunta e
/// só divergiriam no dia em que a convenção de *para cima* mudasse — com o barro
/// indo para um lado e o carimbo para o outro.
#[test]
fn the_grab_and_the_stencil_read_the_same_screen() {
    let c = Camera3d::default();
    let size = (800, 600);
    let at = [0.0, 0.0, 0.0];
    let (right, up) = c.screen_basis();
    let per_px = c.world_radius_for_screen_px(at, 1.0, size);
    let want = right * (3.0 * per_px) + up * (5.0 * per_px);
    let got = Vec3::from(c.screen_delta_to_world(at, 3.0, -5.0, size));
    assert!(
        (got - want).length() < 1e-5,
        "o deslocamento do Grab deixou de sair da base da tela"
    );
}
