//! ⭐⭐⭐ **OS GATES DA ESCULTURA NO DISPOSITIVO** — a grade, a pose, a pintura e a residência.
//!
//! ⚠️ **Irmão por ASSUNTO do [`super::smoke_gpu_tests`], e não por tamanho:** aquele mede o quadro
//! das cenas do roteador; estes medem a folha que **não é uma expressão** — a única do documento
//! cuja lei vive fora da árvore ([`ph2d_field_gpu::sculpt`]).

/// ⭐⭐⭐ **A ESCULTURA COM POSE** — o gate que a cena `6` não consegue ser.
///
/// # ⛔⛔ Porque ele existe: duas mutações SOBREVIVERAM
///
/// A lei da escultura no dispositivo desfaz a pose (`q = R⁻¹(p − t)/s`) e devolve o valor
/// **multiplicado pela escala** — sem esse `· s` o campo deixa de ser uma distância assim que houver
/// escala, e todo raio de filete mente.
///
/// ⚠️ Apagar as duas coisas passou o gate da cena `6`: ali a escultura está na **identidade**, com
/// `t = 0` e `s = 1`. *Um corpus no ponto NEUTRO de um knob não testa esse knob* — e esta é a
/// terceira vez que esta linha paga a mesma forma.
///
/// ⇒ a MESMA grade, posta com translação, rotação e escala, comparada com a CPU pela porta do
/// produto (o [`ph2d_field_render::trace`], que corre o avaliador híbrido).
mod escultura_posta {
    use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Xform};
    use ph2d_field_render::{Orbit, Screen, trace};
    const W: u32 = 192;
    const H: u32 = 108;

    /// A escultura da cena `6`, posta longe da identidade.
    fn peca(escala: f32) -> FieldDoc {
        // ⚠️ Uma rotação a sério (eixo `(1,1,1)`, `40°`): com o quaternião identidade a matriz
        // inversa é a identidade, e os nove números dela ficariam por medir.
        let a = 40.0f32.to_radians() * 0.5;
        let e = 1.0 / 3.0f32.sqrt();
        FieldDoc::new(
            vec![Node {
                xform: Xform {
                    translation: [0.17, -0.09, 0.11],
                    rotation: [e * a.sin(), e * a.sin(), e * a.sin(), a.cos()],
                    scale: escala,
                },
                kind: NodeKind::Sampled { key: "blob".into() },
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a escultura posta")
    }

    #[test]
    #[ignore = "precisa de GPU"]
    fn a_escultura_posta_e_a_mesma_nos_dois_motores() {
        // ⚠️ A cena `6` é quem REGISTA a `blob` — sem esta linha o registo vem vazio e os dois
        // motores desenhariam espaço vazio, que é o buraco que este ficheiro acabou de tapar.
        let _ = crate::smoke::scene(6);
        let reg = crate::smoke::sampled_registry();
        assert!(
            reg.contains_key("blob"),
            "a cena 6 não registou a escultura"
        );

        let Some(t) = crate::gpu_frame::shared() else {
            println!("sem adaptador — saltado");
            return;
        };
        let cam = Orbit::default();
        let (right, up, fwd) = cam.basis();
        let screen = Screen::new(W, H, cam.half_extent);

        for escala in [1.0f32, 0.55, 1.7] {
            let doc = peca(escala);
            let g = trace(&doc, &reg, &cam, W, H);
            let acertos = g.hit.iter().filter(|h| **h).count();
            assert!(
                acertos * 100 > (W * H) as usize,
                "a escala {escala} desenhou só {acertos} pixels — sem sujeito não há comparação"
            );

            let campo = ph2d_field_eval::device::DeviceField::new(&doc, &reg)
                .expect("a escultura compila para o dispositivo");
            let fita = campo.tape_wgsl().expect("a fita");
            let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
                .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
            let passo = ph2d_field_eval::safe_march_step(&doc);
            let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
            let sharp = ph2d_field_render::Sharpness::for_frame(cam.half_extent, W.min(H) as usize);
            let setup = ph2d_field_gpu::trace::MarchSetup {
                half_extent: cam.half_extent,
                half_px: screen.half(),
                target: cam.target,
                right,
                up,
                fwd,
                ortho_start: ph2d_field_render::ORTHO_START,
                eye_distance: cam.eye_distance().unwrap_or(0.0),
                hit_eps: sharp.hit,
                normal_eps: sharp.normal,
                lamps: [[0.0; 3]; ph2d_field_gpu::trace::MAX_LAMPS],
                n_lamps: 0,
                ball_center: bola.center,
                ball_radius: bola.radius,
                ao_rays: 0,
                ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
                ground: None,
                antialias: false,
                edge_cos: ph2d_field_render::EDGE_COS,
                step: passo,
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
                    / passo.clamp(f32::EPSILON, 1.0))
                .ceil() as u32,
                t_max: ph2d_field_render::T_MAX,
            };
            let dev = t
                .lock()
                .expect("o traçador")
                .frame(&fita, campo.sculpts(), setup, W, H);

            let mut difere = 0usize;
            let mut pior_dt = 0.0f32;
            for i in 0..g.hit.len() {
                if g.hit[i] != dev.hit(i) {
                    difere += 1;
                    continue;
                }
                if !g.hit[i] {
                    continue;
                }
                #[allow(clippy::cast_precision_loss)]
                let (sx, sy) = ((i % W as usize) as f32 + 0.5, (i / W as usize) as f32 + 0.5);
                let (u, v) = screen.plane_at(sx, sy);
                let (o, d) = cam.ray_at_plane(u, v);
                let p = g.point[i];
                let t_cpu = (p[0] - o[0]) * d[0] + (p[1] - o[1]) * d[1] + (p[2] - o[2]) * d[2];
                pior_dt = pior_dt.max((t_cpu - dev.t[i]).abs());
            }
            #[allow(clippy::cast_precision_loss)]
            let frac = difere as f64 / f64::from(W * H) * 100.0;
            println!(
                "  escala {escala:>4} · {acertos} pixels · silhueta {frac:.3} % · pior Δt {pior_dt:.2e}"
            );
            // ⚠️ **A barra da silhueta é `0,5 %` e a do Δt é `1e-3`**: a `f64` da matriz inversa
            // desce a `f32` aqui (a mesma divergência declarada da fita), e uma grade de `24³` tem
            // células de `~0,058` — um desacordo de meia célula na borda é o chão desta régua.
            assert!(
                frac < 0.5,
                "a escala {escala} moveu a silhueta {frac:.3} % — a pose não bate"
            );
            assert!(
                pior_dt < 1.0e-3,
                "a escala {escala} deu Δt {pior_dt:.3e} — o campo não é a mesma distância"
            );
        }
    }

    /// ⭐⭐⭐ **E A ESCULTURA PINTA-SE NO DISPOSITIVO** — a imagem inteira, não só a geometria.
    ///
    /// ⚠️ **É um gate à parte do de cima, e o de cima não o cobre:** aquele compara o G-BUFFER, e
    /// uma grade ligada ao passe da marcha e **não** ao do pintor daria silhueta certa e imagem
    /// preta. *O pintor lê a mesma grade por um `BindGroup` diferente, e um binding em falta ali é
    /// mudo para quem só olha a forma.*
    #[test]
    #[ignore = "precisa de GPU"]
    fn a_escultura_pinta_se_no_dispositivo() {
        let _ = crate::smoke::scene(6);
        let reg = crate::smoke::sampled_registry();
        let Some(t) = crate::gpu_frame::shared() else {
            println!("sem adaptador — saltado");
            return;
        };
        let doc = peca(1.2);
        let cam = Orbit::default();
        let luz = [ph2d_field_render::PointLamp {
            world: {
                let (right, up, toward_eye) = cam.basis();
                let e = [-0.55f32, 0.66, 0.5];
                let r = 2.0 * cam.half_extent;
                [0, 1, 2].map(|i| {
                    cam.target[i] + r * (e[0] * right[i] + e[1] * up[i] + e[2] * toward_eye[i])
                })
            },
            radiance_at_one: [3.0, 3.0, 3.0],
        }];
        let materiais = [ph2d_material::OpenPbr {
            base_color: [0.70, 0.55, 0.35],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare()];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let olhar = ph2d_view_transform::Look::default();
        const BG: [u8; 4] = [0, 0, 0, 0];

        let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();
        let (g, sh) = crate::gpu_frame::march(t, &doc, &reg, &cam, &mundos, None, W, H, true)
            .expect("a marcha da escultura");
        let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
        let cpu = ph2d_field_render::shade_render(
            &g,
            &cam,
            &surfaces,
            &ph2d_field_render::Lighting {
                lamps: &sem_ecra,
                points: &luz,
                sky: &crate::render_light::StudioSky,
                shadows: Some(&sh),
            },
            olhar,
            BG,
        );
        let gpu = crate::gpu_frame::paint(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, None, W, H, true,
        )
        .expect("o pintor da escultura")
        .rgba;

        // ⭐ A população primeiro: sem ela duas imagens pretas leriam zero de desvio.
        let pintados = cpu.as_chunks::<4>().0.iter().filter(|p| p[3] > 0).count();
        assert!(
            pintados * 100 > (W * H) as usize,
            "a escultura pintou só {pintados} pixels — o gate não tem sujeito"
        );
        let pior = cpu
            .iter()
            .zip(gpu.iter())
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        #[allow(clippy::cast_precision_loss)]
        let frac = cpu
            .iter()
            .zip(gpu.iter())
            .filter(|(a, b)| a.abs_diff(**b) <= 1)
            .count() as f64
            / cpu.len() as f64;
        println!(
            "  escultura pintada · {pintados} pixels · ≤1 nível em {:.3} % · pior {pior}",
            frac * 100.0
        );
        assert!(
            frac >= 0.995 && pior <= 2,
            "a escultura pintada diverge: {:.3} % a ≤1 nível, pior {pior}",
            frac * 100.0
        );
    }
}

/// ⏱️⭐⭐⭐ **A GRADE SOBE UMA VEZ, e um arrasto não a reenvia.**
///
/// # ⛔⛔ Porque isto precisa de gate e não de confiança
///
/// Uma grade de `128³` são `8 MB`. Reenviá-la por quadro custaria **mais barramento do que a imagem
/// inteira** que o passe do pintor veio poupar (`8,3 MB`) — isto é, a escultura teria desfeito a
/// wave anterior sem mover um número que alguém estivesse a olhar.
///
/// ⚠️ **E o custo é INVISÍVEL na imagem**: os dois caminhos desenham exactamente o mesmo pixel. *Um
/// custo que nenhuma sonda conta é um custo que nenhuma mutação mata.*
mod grade_residente {
    /// Quantos quadros de um «arrasto» este gate encena.
    const QUADROS: usize = 6;

    #[test]
    #[ignore = "precisa de GPU"]
    fn um_arrasto_nao_reenvia_a_escultura() {
        let doc = crate::smoke::scene(6);
        let reg = crate::smoke::sampled_registry();
        // ⛔ **Um traçador PRÓPRIO, e não o partilhado** (2026-09-16). O contador de envios vive no
        // traçador, e o `gpu_frame::shared()` é o de TODOS os gates de placa desta crate: na bateria
        // com threads em paralelo os outros sobem grelhas pelo mesmo contador, e este gate leu
        // `[15, 18, 19, 20, 22, 24]` sobre um produto que, sozinho, lê `[1, 1, 1, 1, 1, 1]` (3 de 3).
        // *Um contador atrás de estado partilhado conta quem mais o partilha.*
        let Some(proprio) = ph2d_field_gpu::trace::Tracer::new() else {
            println!("sem adaptador — saltado");
            return;
        };
        let t = &std::sync::Arc::new(std::sync::Mutex::new(proprio));
        let base = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&base)];
        let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();

        let mut enviadas = Vec::new();
        for i in 0..QUADROS {
            // ⚠️ **A câmera MEXE-SE a cada quadro** — é isso que faz disto um arrasto e não uma
            // repetição: sem o movimento, um cache que comparasse a cena inteira também passaria.
            #[allow(clippy::cast_precision_loss)]
            let cam = ph2d_field_render::Orbit {
                half_extent: base.half_extent,
                target: base.target,
                lens: base.lens,
                ..ph2d_field_render::Orbit::from_yaw_pitch(0.05 * i as f32, 0.1)
            };
            crate::gpu_frame::march(t, &doc, &reg, &cam, &mundos, None, 96, 54, false)
                .expect("a marcha da escultura");
            enviadas.push(t.lock().expect("o traçador").grades_enviadas());
        }
        println!("  grades enviadas por quadro: {enviadas:?}");
        // ⭐ **O CONTROLO vem primeiro:** a primeira tem de subir, senão o gate abaixo passaria com
        // o caminho da escultura inteiro desligado.
        assert!(
            enviadas[0] >= 1,
            "a grade nunca subiu — a escultura não está a chegar ao dispositivo"
        );
        assert_eq!(
            enviadas[0],
            *enviadas.last().expect("há quadros"),
            "a grade voltou a subir durante o arrasto: {enviadas:?}"
        );
    }
}

/// ⏱️⭐⭐⭐ **O QUE A ESCULTURA CUSTA nos dois motores** — a cena da PONTE, quadro assente.
///
/// ⚠️ **Ela imprime o `/proc/loadavg` ao lado de cada número**, e o mínimo de sete corridas: nenhuma
/// leitura de relógio desta máquina vale nada acima de `load ~5`.
#[cfg(test)]
mod relogio_da_escultura {
    #[test]
    #[ignore = "medição — precisa de GPU e de máquina calma"]
    fn mede_a_ponte_nos_dois_motores() {
        const LW: u32 = 1920;
        const LH: u32 = 1080;
        const CORRIDAS: usize = 7;

        let doc = crate::smoke::scene(6);
        let reg = crate::smoke::sampled_registry();
        let Some(t) = crate::gpu_frame::shared() else {
            println!("sem adaptador — saltado");
            return;
        };
        let cam = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();
        let materiais = [ph2d_material::OpenPbr::default().prepare()];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let olhar = ph2d_view_transform::Look::default();
        const BG: [u8; 4] = [0, 0, 0, 0];

        let mede = |mut f: Box<dyn FnMut()>| -> (f64, f64) {
            let mut v: Vec<f64> = Vec::with_capacity(CORRIDAS);
            for _ in 0..CORRIDAS {
                let t0 = std::time::Instant::now();
                f();
                v.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            v.sort_by(f64::total_cmp);
            (v[0], v[CORRIDAS / 2])
        };
        // ⚠️ Uma corrida de aquecimento fora da conta: a primeira compila o pipeline e sobe a grade.
        let _ = crate::gpu_frame::paint(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, None, LW, LH, true,
        );

        let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
        let (cpu_min, cpu_med) = mede(Box::new(|| {
            let g = ph2d_field_render::trace(&doc, &reg, &cam, LW, LH);
            let sh = ph2d_field_render::shadow_pass(&doc, &reg, &cam, &g, &mundos);
            let px = ph2d_field_render::shade_render(
                &g,
                &cam,
                &surfaces,
                &ph2d_field_render::Lighting {
                    lamps: &sem_ecra,
                    points: &luz,
                    sky: &crate::render_light::StudioSky,
                    shadows: Some(&sh),
                },
                olhar,
                BG,
            );
            std::hint::black_box(px.len());
        }));
        let (gpu_min, gpu_med) = mede(Box::new(|| {
            let p = crate::gpu_frame::paint(
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, None, LW, LH, true,
            )
            .expect("o pintor da escultura");
            std::hint::black_box(p.rgba.len());
        }));
        println!(
            "\n  A PONTE · {LW}×{LH} · load {}",
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
        println!("  caminho                 min    mediana");
        println!("  CPU inteira         {cpu_min:7.1}  {cpu_med:7.1} ms");
        println!("  dispositivo         {gpu_min:7.2}  {gpu_med:7.2} ms");
        println!("  ganho               {:7.1}×", cpu_min / gpu_min);
    }
}
