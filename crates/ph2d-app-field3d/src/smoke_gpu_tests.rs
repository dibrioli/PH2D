//! ⭐⭐⭐ **OS GATES DE PARIDADE DO DISPOSITIVO** — o quadro do modelador nos dois motores.
//!
//! ⚠️ **Eles saíram do [`super`] em 2026-09-15 por TETO DE LINHAS**, e a fronteira não é arbitrária:
//! o `smoke.rs` é o roteador das cenas e estes são os gates que as **medem** contra a GPU. *Uma
//! régua e o sujeito dela não são a mesma responsabilidade.*

#[cfg(test)]
mod gpu_parity {
    /// ⭐⭐⭐ **DOIS MOTORES, UMA LEI: o campo do DISPOSITIVO responde o mesmo que o da CPU.**
    ///
    /// Percorre **todas** as cenas vivas do smoke, gera o WGSL da fita de cada uma, corre-a na GPU
    /// sobre uma grelha de pontos e compara com o [`ph2d_field_eval::Field::at`].
    ///
    /// ⚠️ **A barra não é zero, e a razão é declarada:** a fita da CPU é `f64` e o dispositivo é
    /// `f32`. O que se exige é que o erro seja o da **representação**, e não o de uma lei diferente
    /// — um opcode traduzido ao contrário (a ordem do `atan2`, o sinal do `Mod`) dá erros de
    /// unidades de mundo, não de `1e-6`.
    ///
    /// ⚠️ `#[ignore]`: precisa de adaptador, como todo gate de GPU desta casa.
    #[test]
    #[ignore = "precisa de GPU"]
    fn o_campo_do_dispositivo_responde_como_o_da_cpu() {
        // Uma grelha dentro do enquadramento da peça, mais os eixos — pontos que caem DENTRO,
        // FORA e sobre a superfície.
        let mut pontos = Vec::new();
        for i in 0..11 {
            for j in 0..11 {
                for k in 0..11 {
                    let f = |n: i32| (n as f32 / 10.0) * 2.4 - 1.2;
                    pontos.push([f(i), f(j), f(k)]);
                }
            }
        }

        let mut piores: Vec<(u32, f64, f64)> = Vec::new();
        for n in 0..crate::smoke::scenes::CENAS {
            if crate::smoke::scenes::PODADAS.contains(&n) {
                continue;
            }
            let doc = crate::smoke::scene(n);
            let Some(p) = ph2d_field_gpu::parity::compare(&doc, &pontos) else {
                println!("cena {n}: sem GPU ou sem fita — saltada");
                continue;
            };
            // ⚠️ **NaN de um lado tem de ser NaN do outro** — um campo que responde `NaN` onde o
            // outro responde um número é uma lei diferente, e a subtração esconde-o.
            let discordam_nan = p
                .gpu
                .iter()
                .zip(&p.cpu)
                .filter(|(g, c)| g.is_nan() != c.is_nan())
                .count();
            assert_eq!(
                discordam_nan, 0,
                "cena {n}: {discordam_nan} pontos em que um motor diz NaN e o outro não"
            );
            piores.push((n, p.worst(), p.rms()));
        }
        assert!(!piores.is_empty(), "nenhuma cena foi comparada");

        println!("  cena ·   pior desvio ·    RMS");
        for (n, w, r) in &piores {
            println!("  {n:4} · {w:13.3e} · {r:9.3e}");
        }
        let pior = piores.iter().fold(0.0f64, |m, (_, w, _)| m.max(*w));
        // ⚠️ **A barra é a da REPRESENTAÇÃO.** As peças vivem em `±1,2` de mundo, e um `f32` tem
        // `~7` dígitos: um erro acumulado ao longo de uma fita de centenas de operações fica na
        // casa de `1e-4`. ⛔ Uma lei diferente (uma ordem de `atan2` trocada, um sinal de `Mod`)
        // dá desvios de unidades de MUNDO — três ordens de grandeza acima disto.
        assert!(
            pior < 1e-3,
            "o pior desvio entre os dois motores é {pior:.3e} — acima do erro de representação de \
             um `f32` sobre uma peça de `±1,2`. Não é precisão: é uma LEI diferente."
        );
    }
}

#[cfg(test)]
mod gpu_coarse_law {
    /// ⭐⭐⭐ **A BANDEIRA DA BORDA CHEGA AO SEGUNDO DESPACHO** — desligá-la re-amostra `0` pixels.
    ///
    /// # ⛔⛔⛔ A PREMISSA DESTE GATE MORREU em 2026-09-19, e ele fica pelo que sobra
    ///
    /// Ele chamava-se `sem_anti_serrilhado_o_dispositivo_nao_reamostra_borda_nenhuma` e o doc dele
    /// afirmava que **a lei da W73 manda o quadro de movimento não pagar o segundo despacho**.
    /// A `W7c` mediu esse pagamento no caminho do pintor, a `1920×1080`: **`1,03×`–`1,09×`** do
    /// quadro de movimento (`+0,18` a `+2,66 ms`), contra os `1,30×`–`1,40×` que a tabela de CPU do
    /// [`ph2d_field_render::trace_cancellable`] media a `640×360` — e sem ele a silhueta que a mão
    /// arrasta não tem **um único** pixel de cobertura parcial, que é o que o dono lê como a peça a
    /// *«ferver»* (`docs/Render3d/12` §12). ⇒ **a segunda passagem saiu da bandeira** e corre em
    /// todo quadro; quem a desliga é a [`crate::gpu_frame::Sonda::bordas`], que não é produto.
    ///
    /// ⚠️ *O que este gate continua a afirmar é a FIAÇÃO, não a lei:* a bandeira do
    /// [`crate::gpu_frame::march`] sempre significou só o segundo despacho, e ela tem de lá chegar
    /// — senão a porta de bissecção de um report não bissecta nada.
    #[test]
    #[ignore = "precisa de GPU"]
    fn a_bandeira_da_borda_chega_ao_segundo_despacho() {
        let doc = crate::smoke::scene(1);
        let reg = ph2d_field_eval::hybrid::Registry::new();
        let cam = ph2d_field_render::Orbit::default();
        let luz = [super::gpu_gbuffer_parity::luz_do_rig(&cam)];
        let Some(t) = crate::gpu_frame::shared() else {
            println!("sem adaptador — saltado");
            return;
        };

        let bordas = |re_amostra: bool| {
            crate::gpu_frame::march(t, &doc, &reg, &cam, &luz, None, 192, 108, re_amostra)
                .map(|(g, _)| g.edges.len())
        };
        let nitido = bordas(true).expect("o dispositivo tem de marchar a peça limpa");
        let grosso = bordas(false).expect("o dispositivo tem de marchar a peça limpa");

        // ⭐ O controlo vem PRIMEIRO: sem ele, `0 == 0` passaria com o passe inteiro partido.
        assert!(
            nitido > 50,
            "com anti-serrilhado o dispositivo devolveu só {nitido} bordas — a fixtura não tem              contorno, e o gate abaixo não estaria a afirmar nada"
        );
        assert_eq!(
            grosso, 0,
            "com a bandeira em baixo o dispositivo ainda re-amostrou {grosso} bordas — a porta de              bissecção da `W7c` não alcança o segundo despacho"
        );
    }
}

#[cfg(test)]
mod gpu_gbuffer_parity {
    /// ⭐⭐⭐ **O G-BUFFER DO DISPOSITIVO É O DA CPU** — a silhueta, a profundidade e a normal.
    ///
    /// ⚠️ **É este gate que vigia a única coisa escrita DUAS vezes**: a câmera. Mandar os raios
    /// prontos seriam `50 MB` por quadro, logo o WGSL reconstrói o `ray_at_plane` — e uma
    /// divergência ali move o ponto de acerto em **unidades de mundo**, que é o que as colunas
    /// medem.
    /// A lâmpada onde a wave da §25 a põe.
    pub(super) fn luz_do_rig(cam: &ph2d_field_render::Orbit) -> [f32; 3] {
        let (right, up, toward_eye) = cam.basis();
        let ecra = [-0.5566703_f32, 0.6634139, 0.5];
        let r = 2.0 * cam.half_extent;
        [0, 1, 2].map(|i| {
            cam.target[i] + r * (ecra[0] * right[i] + ecra[1] * up[i] + ecra[2] * toward_eye[i])
        })
    }

    #[test]
    #[ignore = "precisa de GPU"]
    fn o_gbuffer_do_dispositivo_e_o_da_cpu() {
        use ph2d_field_render::{Orbit, Screen, trace};
        const W: u32 = 192;
        const H: u32 = 108;

        let cam = Orbit::default();
        let (right, up, fwd) = cam.basis();
        let screen = Screen::new(W, H, cam.half_extent);

        println!("  cena · silhueta ·       Dt ·  Dnormal ·   a variacao da PROPRIA peca · razao");
        let mut piores = (0usize, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f64);
        let mut vistas = 0;
        let mut populacao_ceu = 0usize;
        for n in 0..crate::smoke::scenes::CENAS {
            if crate::smoke::scenes::PODADAS.contains(&n) {
                continue;
            }
            let doc = crate::smoke::scene(n);
            // ⛔⛔⛔ **O REGISTO LÊ-SE DEPOIS DE A CENA NASCER, e até 2026-09-15 não era assim.**
            //
            // Ele era um `Registry::new()` do topo do gate — **vazio** —, e quem enche o registo é
            // o construtor da cena (`register_sampled`). ⇒ a cena da PONTE era comparada com os
            // DOIS motores a desenhar espaço vazio, e a linha dela lia `0,000 %` em todas as
            // colunas. *Um zero de «igual» e um de «nenhum dos dois desenhou nada» são o mesmo
            // byte* — e o piso de população abaixo é o que os separa.
            let reg = crate::smoke::sampled_registry();
            // ⭐⭐⭐ **A peça compila COM A ESCULTURA DENTRO** — ver [`ph2d_field_eval::device`].
            // ⚠️ Até 2026-09-15 esta linha era o `Field::new`, que traduz uma escultura para espaço
            // VAZIO: a cena da ponte era comparada com o dispositivo a desenhar uma peça **sem
            // ela**, e os dois lados concordavam sobre um buraco.
            let Some(campo) = ph2d_field_eval::device::DeviceField::new(&doc, &reg) else {
                continue;
            };
            let Some(fita) = campo.tape_wgsl() else {
                continue;
            };
            let g = trace(&doc, &reg, &cam, W, H);
            let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
                .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
            let passo = ph2d_field_eval::safe_march_step(&doc);
            let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
            let setup = ph2d_field_gpu::trace::MarchSetup {
                half_extent: cam.half_extent,
                half_px: screen.half(),
                target: cam.target,
                right,
                up,
                fwd,
                ortho_start: ph2d_field_render::ORTHO_START,
                eye_distance: cam.eye_distance().unwrap_or(0.0),
                hit_eps: ph2d_field_render::Sharpness::for_frame(
                    cam.half_extent,
                    W.min(H) as usize,
                )
                .hit,
                normal_eps: ph2d_field_render::Sharpness::for_frame(
                    cam.half_extent,
                    W.min(H) as usize,
                )
                .normal,
                lamps: {
                    // ⚠️ A cauda fica a zero: só as `n_lamps` primeiras são lidas.
                    let mut v = [[0.0f32; 3]; ph2d_field_gpu::trace::MAX_LAMPS];
                    v[0] = luz_do_rig(&cam);
                    v
                },
                n_lamps: 1,
                ball_center: bola.center,
                ball_radius: bola.radius,
                ao_rays: ph2d_field_render::OCCLUSION_PASSES,
                ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
                ceu_passo: 1,
                ground: None,
                antialias: true,
                edge_cos: ph2d_field_render::EDGE_COS,
                mole: None,
                // ⭐⭐⭐⭐ **O recorte do PRODUTO, pela porta do produto** — até 2026-09-24 este gate
                // corria com `None` e comparava um dispositivo SEM recorte contra uma CPU COM ele:
                // mediu a divergência do ponto de partida (`8` pixels de silhueta na cena `=29`), e
                // não a do produto (`0`).
                longe: ph2d_field_eval::bounds::bounding_ball(&doc, &reg).and_then(|b| {
                    crate::gpu_frame::a_caixa_da_marcha(b, crate::gpu_frame::Sonda::default().longe)
                }),
                step: passo,
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
                    / passo.clamp(f32::EPSILON, 1.0))
                .ceil() as u32,
                t_max: ph2d_field_render::T_MAX,
            };
            let Some(t) = crate::gpu_frame::shared() else {
                println!("sem GPU — saltada");
                return;
            };
            let dev = t
                .lock()
                .expect("o traçador")
                .frame(&fita, campo.sculpts(), setup, W, H);
            vistas += 1;

            // ⭐⭐⭐ **O PISO DE POPULAÇÃO: uma cena que não desenha nada não compara nada.**
            //
            // ⚠️ **Sem ele este gate leu `0,000 %` sobre a cena da PONTE durante toda a wave da
            // GPU**, porque o registo de esculturas estava vazio e os dois motores desenhavam
            // espaço vazio. *Um gate que não sabe se o sujeito dele existe não afirma nada sobre
            // ele.* O número é `1 %` da tela — qualquer cena do roteador enche muito mais.
            let acertos = g.hit.iter().filter(|h| **h).count();
            assert!(
                acertos * 100 > (W * H) as usize,
                "a cena {n} desenhou só {acertos} de {} pixels — ela não tem sujeito, e a linha \
                 dela leria zero de desvio por não haver nada que possa divergir",
                W * H
            );

            // ⚠️ **A silhueta compara-se por CONTAGEM de pixels em desacordo, não por igualdade**:
            // na borda um raio decide por um `epsilon`, e os dois motores são `f32` com ordens de
            // soma diferentes. O que não pode é a peça mudar de tamanho.
            let mut difere = 0usize;
            let mut dts: Vec<f32> = Vec::new();
            let mut angs: Vec<f32> = Vec::new();
            // ⭐ O Δt por pixel fica guardado: é ele que diz onde a oclusão pode ser comparada.
            let mut dt_por_pixel = vec![f32::INFINITY; g.hit.len()];
            // O laço indexa SEIS sequências pelo mesmo `i` (as duas silhuetas, os dois pontos, as
            // duas normais) — um `enumerate` sobre uma delas escondia as outras cinco.
            #[allow(clippy::needless_range_loop)]
            for i in 0..g.hit.len() {
                if g.hit[i] != dev.hit(i) {
                    difere += 1;
                    continue;
                }
                if !g.hit[i] {
                    continue;
                }
                // O `t` da CPU não é guardado; o ponto é. A distância entre os dois pontos É o Δt.
                let (sx, sy) = ((i % W as usize) as f32 + 0.5, (i / W as usize) as f32 + 0.5);
                let (u, v) = screen.plane_at(sx, sy);
                let (o, d) = cam.ray_at_plane(u, v);
                let p = g.point[i];
                let t_cpu = (p[0] - o[0]) * d[0] + (p[1] - o[1]) * d[1] + (p[2] - o[2]) * d[2];
                dt_por_pixel[i] = (t_cpu - dev.t[i]).abs();
                dts.push(dt_por_pixel[i]);
                let (a, b) = (g.normal[i], dev.normal[i]);
                let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
                angs.push(dot.acos().to_degrees());
            }
            // ⚠️⚠️ **O EXTREMO E A POPULAÇÃO respondem a perguntas DIFERENTES, e aqui a que
            // interessa é a segunda.** Num VINCO a derivada não existe (o módulo já o tem escrito),
            // logo um pixel que caia exactamente lá dá normais muito diferentes a partir de uma
            // diferença de campo de `1e-7` — e as formas por fórmula (rosca, polígono, triângulo)
            // são feitas de vincos. *Um opcode traduzido ao contrário move MILHARES de pixels; um
            // vinco move um punhado.* ⇒ a barra é o `p99`, e o máximo fica na tabela para se ver.
            dts.sort_by(f32::total_cmp);
            angs.sort_by(f32::total_cmp);
            let q = |v: &[f32], f: f64| -> f32 {
                if v.is_empty() {
                    0.0
                } else {
                    v[((v.len() - 1) as f64 * f) as usize]
                }
            };
            // ⭐⭐⭐ **A RÉGUA DA NORMAL É A VARIAÇÃO DA PRÓPRIA PEÇA, e não um ângulo escolhido.**
            //
            // ⚠️ Duas cenas — a ROSCA e as CURVAS — dão `p99` de `12,8°` e `9,9°` contra `≤ 0,5°`
            // das outras catorze, e o campo concorda a `1e-7` nas três. A diferença não é a lei:
            // é o **CONDICIONAMENTO**. Numa ranhura de passo fino a normal roda dezenas de graus
            // de um pixel para o vizinho, logo um deslocamento de `1e-7` no ponto move-a muito.
            //
            // ⇒ a barra é a **variação entre pixels VIZINHOS da CPU**: os dois motores têm de
            // concordar tanto quanto a geometria permite a um pixel concordar com o do lado. *Uma
            // barra em graus absolutos ou isentava a rosca ou acusava as outras quinze.*
            let mut vizinhos: Vec<f32> = Vec::new();
            for y in 0..H as usize {
                for x in 0..W as usize - 1 {
                    let (a_i, b_i) = (y * W as usize + x, y * W as usize + x + 1);
                    if !g.hit[a_i] || !g.hit[b_i] {
                        continue;
                    }
                    let (a, b) = (g.normal[a_i], g.normal[b_i]);
                    let d = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
                    vizinhos.push(d.acos().to_degrees());
                }
            }
            vizinhos.sort_by(f32::total_cmp);
            let pct = 100.0 * difere as f64 / g.hit.len() as f64;
            let (ang99, viz99) = (q(&angs, 0.99), q(&vizinhos, 0.99));
            let razao = ang99 / viz99.max(1e-6);
            println!(
                "  {n:4} · {pct:7.3} % · p99 {:8.2e} · p99 {ang99:6.2}° · o VIZINHO varia {viz99:6.2}° · {razao:5.2}x",
                q(&dts, 0.99)
            );
            piores.0 = piores.0.max(difere);
            piores.1 = piores.1.max(q(&dts, 0.99));
            piores.2 = piores.2.max(razao);

            // ⭐⭐⭐ **AS TRÊS COISAS NOVAS, cada uma com a sua régua.**
            let sh = ph2d_field_render::shadow_pass(&doc, &reg, &cam, &g, &[luz_do_rig(&cam)]);
            let ao = ph2d_field_render::occlusion(
                &doc,
                &reg,
                &cam,
                &g,
                ph2d_field_render::OCCLUSION_PASSES,
            );
            let mut d_sombra: Vec<f32> = Vec::new();
            let mut d_ceu: Vec<f32> = Vec::new();
            let mut d_ceu_todos: Vec<f32> = Vec::new();
            for (j, ceu) in ao.iter().enumerate() {
                if !g.hit[j] || !dev.hit(j) {
                    continue;
                }
                d_sombra.push((sh.at(0, j) - dev.shadow[j]).abs());
                d_ceu_todos.push((ceu - dev.ambient[j]).abs());
                // ⭐⭐⭐ **A OCLUSÃO SÓ SE COMPARA ONDE A NORMAL CONCORDA, e não é conveniência.**
                //
                // Desde 2026-09-15 a oclusão é **função de `(ponto, normal)`** — há gate na
                // `ph2d-field-render` a afirmá-lo, ao bit. ⇒ onde os dois motores entregam normais
                // a `9,9°` uma da outra (cena 30, num vinco, e a coluna ao lado mede-o), eles TÊM
                // de entregar oclusões diferentes: isso é a consequência da divergência da normal,
                // que **já tem barra própria duas linhas acima**, e não uma lei de oclusão
                // diferente. *Gatear a mesma divergência duas vezes não a mede melhor — mede o
                // acoplamento e chama-lhe defeito do segundo passe.*
                let (a, b) = (g.normal[j], dev.normal[j]);
                let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
                if dot.acos().to_degrees() < 1.0 && dt_por_pixel[j] < 3e-5 {
                    d_ceu.push((ceu - dev.ambient[j]).abs());
                }
            }
            d_ceu_todos.sort_by(f32::total_cmp);
            d_sombra.sort_by(f32::total_cmp);
            d_ceu.sort_by(f32::total_cmp);

            // ⚠️ **A BORDA compara-se como CONJUNTO.** Os dois motores decidem por um `cos` sobre
            // normais em `f32`, logo um pixel de fronteira pode cair de qualquer lado — o que não
            // pode é a população ser outra. *A régua é a sobreposição, não a igualdade.*
            let cpu_b: std::collections::BTreeSet<u32> = g.edges.iter().map(|e| e.pixel).collect();
            let gpu_b: std::collections::BTreeSet<u32> =
                dev.edges.iter().map(|e| e.pixel).collect();
            let comuns = cpu_b.intersection(&gpu_b).count();
            let uniao = cpu_b.union(&gpu_b).count();
            let sobrep = if uniao == 0 {
                1.0
            } else {
                comuns as f64 / uniao as f64
            };
            println!(
                "         sombra p99 {:7.4} · ceu p99 {:7.4} (todos {:7.4}) · bordas CPU {} / GPU {} · sobrepoem {:5.1} %",
                q(&d_sombra, 0.99),
                q(&d_ceu, 0.99),
                q(&d_ceu_todos, 0.99),
                cpu_b.len(),
                gpu_b.len(),
                100.0 * sobrep
            );
            piores.3 = piores.3.max(q(&d_sombra, 0.99));
            piores.4 = piores.4.max(q(&d_ceu, 0.99));
            populacao_ceu += d_ceu.len();
            piores.5 = piores.5.min(sobrep);
        }
        assert!(vistas > 10, "só {vistas} cenas foram comparadas");

        let pct = 100.0 * piores.0 as f64 / (W * H) as f64;
        // ⚠️ **As três barras são do MESMO tipo: erro de representação, não de lei.** Uma câmera
        // divergente move o ponto de acerto em unidades de MUNDO e vira a silhueta inteira; um
        // estêncil trocado põe a normal a dezenas de graus. *As barras estão onde o vale medido
        // está, não onde o defeito seria confortável.*
        assert!(
            pct < 1.0,
            "{pct:.3} % dos pixels discordam sobre haver peça — na borda um `epsilon` decide, mas \
             1 % é a peça a mudar de TAMANHO, e isso é a câmera escrita duas vezes a divergir"
        );
        // ⭐⭐⭐ **A barra do Δt desceu de `1e-3` para `1e-5` em 2026-09-24, e sai de um VALE
        // medido:** com o recorte pela caixa da marcha (o produto) o pior `p99` das cenas é
        // `5,96e-7`; sem recorte nenhum era `1,8e-4`, e com a caixa CRUA `3,7e-4` — os dois
        // motores a partir de sítios diferentes. ⇒ `1e-5` fica `17×` acima do produto e `18×`
        // abaixo da regressão mais pequena. ⛔ A de `1e-3` deixava passar as duas.
        assert!(
            piores.1 < 1e-5,
            "o Δt do p99 é {:.3e} — a marcha do dispositivo está a parar noutro sítio (o recorte \
             é o da `march_clip`, a mesma caixa da CPU?)",
            piores.1
        );
        // ⭐ **A sombra e a oclusão são AO BIT comparáveis** — os dois motores correm a mesma
        // sequência de amostragem, logo o que sobra é `f32`. ⛔ Uma sequência só «equivalente»
        // obrigaria a descer a uma média, que é a régua que a §31 mostrou ser cega.
        assert!(
            piores.3 < 0.05,
            "a sombra dos dois motores difere {:.4} no p99 — com o MESMO amostrador, isso já não \
             é ruído",
            piores.3
        );
        // ⚠️⚠️⚠️ **A BARRA DA OCLUSÃO FOI RE-DERIVADA em 2026-09-15, e a anterior media uma
        // grandeza que deixou de existir.** Ela era `2 / OCCLUSION_PASSES` — *dois raios do
        // quantum binário*. Com CONES não há quantum: a resposta é contínua, e aquela fórmula
        // passou a devolver `0,0417` sem nomear recurso nenhum. *Um tecto derivado da grandeza
        // errada lê-se como generoso.*
        //
        // ⭐ O recurso é a **representação em `f32` propagada pelo estimador**, e a conta fecha
        // com a medição. O cone lê `d / (t · cos)` e a primeira amostra cai em `t₀ = 4 · hit_eps`;
        // com `hit_eps ≈ 7e-3` nesta resolução, o campo dos dois motores a concordar a `1e-4` (a
        // barra da fita) e o `Δt` do filtro a `3e-5`, o pior caso é
        // `(1e-4 + 3e-5) / (3e-2 · 0,3) ≈ 0,014`. Medido: **`0,0153`** na pior cena.
        //
        // ⭐⭐ **O filtro foi VARRIDO e a dependência é do `Δt`, o que confirma o mecanismo:**
        //
        // | `Δt` do filtro | população | pior `p99` |
        // |---|---:|---:|
        // | `1e-5` | `8 807` | `0,0103` |
        // | **`3e-5`** | **`24 391`** | **`0,0153`** |
        // | `1e-4` | `58 215` | `0,0163` |
        //
        // ⇒ `3e-5` é onde a população deixa de ser um punhado sem a barra deixar de apertar.
        //
        // ⛔ Uma lei diferente — a esfera de Fibonacci desalinhada, o peso sem cosseno, a cerca da
        // bola só num motor — move **décimas**: a cerca a faltar na CPU leu `0,1450` no dia em que
        // este gate a apanhou, `9,5×` esta barra.
        assert!(
            populacao_ceu > 20_000,
            "só {populacao_ceu} pixels passaram o filtro de geometria — a barra da oclusão está a              medir quase nada, e um `p99` sobre um punhado de pixels aprova qualquer coisa"
        );
        assert!(
            piores.4 < 0.025,
            "a oclusão difere {:.4} no p99 ONDE A GEOMETRIA CONCORDA — isso já não é a `f32` da              fita a propagar-se, é a lei do cone a divergir entre os dois motores",
            piores.4
        );
        // ⚠️ A borda é uma decisão de `cos` sobre `f32`: um pixel de fronteira pode cair de
        // qualquer lado. *O que não pode é a POPULAÇÃO ser outra.*
        assert!(
            piores.5 > 0.9,
            "as listas de borda só se sobrepõem {:.1} % — o critério de aresta divergiu",
            100.0 * piores.5
        );
        assert!(
            piores.2 < 1.0,
            "a normal dos dois motores difere {:.2}x mais do que um pixel difere do VIZINHO — \
             isso ja nao e condicionamento: e o estencil ou a base de vista a divergirem",
            piores.2
        );
    }
}

#[cfg(test)]
mod gpu_frame_clock {
    /// ⭐⭐⭐ **O QUADRO COMPLETO NO DISPOSITIVO** — traçado, normal, sombra, oclusão e bordas, com
    /// a leitura de volta dentro. É o relógio que o artista vai sentir.
    #[test]
    #[ignore = "precisa de GPU"]
    fn measure_the_device_frame() {
        use std::time::Instant;
        let reg = ph2d_field_eval::hybrid::Registry::new();
        let cam = ph2d_field_render::Orbit::default();
        let (right, up, fwd) = cam.basis();
        let doc = crate::smoke::scene(1);
        let campo = ph2d_field_eval::Field::new(&doc);
        let fita = campo.tape_wgsl().expect("a fita");
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
            .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
        let passo = ph2d_field_eval::safe_march_step(&doc);
        let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
        let ecra = [-0.5566703_f32, 0.6634139, 0.5];
        let r = 2.0 * cam.half_extent;
        let luz = [0, 1, 2]
            .map(|i| cam.target[i] + r * (ecra[0] * right[i] + ecra[1] * up[i] + ecra[2] * fwd[i]));

        println!(
            "carga: {}",
            std::fs::read_to_string("/proc/loadavg").unwrap().trim()
        );
        println!("  px        · DISPOSITIVO · a CPU faz · ganho");
        for (w, h) in [(640_u32, 360_u32), (1920, 1080)] {
            let screen = ph2d_field_render::Screen::new(w, h, cam.half_extent);
            let sharp = ph2d_field_render::Sharpness::for_frame(cam.half_extent, w.min(h) as usize);
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
                step: passo,
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
                    / passo.clamp(f32::EPSILON, 1.0))
                .ceil() as u32,
                t_max: ph2d_field_render::T_MAX,
                lamps: {
                    // ⚠️ A cauda fica a zero: só as `n_lamps` primeiras são lidas.
                    let mut v = [[0.0f32; 3]; ph2d_field_gpu::trace::MAX_LAMPS];
                    v[0] = luz;
                    v
                },
                n_lamps: 1,
                ball_center: bola.center,
                ball_radius: bola.radius,
                ao_rays: ph2d_field_render::OCCLUSION_PASSES,
                ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
                ceu_passo: 1,
                ground: None,
                antialias: true,
                edge_cos: ph2d_field_render::EDGE_COS,
                mole: None,
                longe: None,
            };
            // ⛔⛔ **O TRAÇADOR VIVE ENTRE QUADROS, e a 1.ª redacção desta sonda usava a porta que
            // abre o dispositivo a cada chamada** — ela leu `130 ms` a `640×360`, *mais lento que a
            // CPU*, medindo a abertura e a compilação em vez do quadro. O doc daquela porta já
            // dizia «é a forma de sonda», e eu usei-a como relógio na mesma.
            let Some(mut tr) = ph2d_field_gpu::trace::Tracer::new() else {
                println!("sem GPU");
                return;
            };
            // A 1.ª corrida COMPILA o shader (§33); fica de fora.
            let _ = tr.frame(&fita, &[], setup, w, h);
            let mut v: Vec<f64> = (0..5)
                .map(|_| {
                    let t = Instant::now();
                    let g = tr.frame(&fita, &[], setup, w, h);
                    std::hint::black_box(g.edges.len());
                    t.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            v.sort_by(f64::total_cmp);
            let cpu = if w == 640 { 13.15 } else { 102.64 };
            println!(
                "{w:5}x{h:<4} · {:8.2} ms · {cpu:6.2} ms · {:5.1}x",
                v[0],
                cpu / v[0]
            );
        }
    }
}

#[cfg(test)]
mod gpu_recusa {
    /// ⭐⭐⭐ **UMA PEÇA COM ESCULTURA VAI PARA O DISPOSITIVO** — e até 2026-09-15 ela não ia.
    ///
    /// # ⚠️⚠️ A redacção anterior deste gate afirmava o CONTRÁRIO, e estava certa na altura
    ///
    /// Ela dizia *«uma peça com escultura NÃO vai»*, e era o gate mais importante daquela wave:
    /// uma escultura compilava para `Tree::constant(ABSENT)` — espaço vazio —, logo sem a recusa a
    /// GPU desenharia a peça **sem ela**, em silêncio e com o resto perfeito. ⛔ E o gate da
    /// paridade não o veria: ele compara a fita com a fita, e as duas concordam que ali não há
    /// nada. *Um zero de «igual» e um de «nenhum dos dois sabe» são o mesmo byte.*
    ///
    /// ⭐ Hoje a escultura **atravessa** ([`ph2d_field_gpu::sculpt`]): ela entra na árvore como uma
    /// variável e o shader amostra a grade. ⇒ o que este gate passa a afirmar é a outra metade da
    /// mesma lei — que a recusa **encolheu** para a pergunta verdadeira (*a folha sabe entregar a
    /// grade?*) e não desapareceu.
    #[test]
    fn uma_peca_com_escultura_vai_para_o_dispositivo() {
        // A cena 6 é a PONTE: uma escultura de 8 192 triângulos virada campo.
        let com = crate::smoke::scene(6);
        assert!(
            com.nodes()
                .iter()
                .any(|n| matches!(n.kind, ph2d_field::NodeKind::Sampled { .. })),
            "a fixtura deixou de ter escultura — este gate passou a não afirmar nada"
        );
        let reg = crate::smoke::sampled_registry();
        assert!(
            ph2d_field_gpu::supports(&com, &reg),
            "a cena da ponte foi recusada — a grade dela atravessa desde 2026-09-15"
        );

        // ⭐⭐ **O CONTROLO: uma folha amostrada SEM grade continua a ser recusada.** É ele que
        // impede o `supports` de virar um `true` constante — e a lei que ele prende é a que faz
        // uma escultura de outra espécie cair na CPU em vez de desaparecer.
        struct SemGrade;
        impl ph2d_field_eval::hybrid::Sampled for SemGrade {
            fn at(&self, _p: [f32; 3]) -> f32 {
                0.0
            }
            fn bounding_radius(&self) -> f32 {
                1.0
            }
            // ⚠️ Sem `grid`: fica o default do trait, que é `None`.
        }
        let mut cego = ph2d_field_eval::hybrid::Registry::new();
        for k in reg.keys() {
            cego.insert(k.clone(), std::sync::Arc::new(SemGrade));
        }
        assert!(
            !cego.is_empty(),
            "o registo da cena está vazio — o controlo não tem sujeito"
        );
        assert!(
            !ph2d_field_gpu::supports(&com, &cego),
            "uma folha amostrada SEM grade foi aceite — ela desapareceria da peça"
        );

        // ⭐ E as OUTRAS cenas continuam a ir.
        let mut aceites = 0;
        for n in 0..crate::smoke::scenes::CENAS {
            if crate::smoke::scenes::PODADAS.contains(&n) {
                continue;
            }
            if ph2d_field_gpu::supports(&crate::smoke::scene(n), &reg) {
                aceites += 1;
            }
        }
        assert!(
            aceites > 10,
            "só {aceites} cenas foram aceites — a porta está a reclamar tudo"
        );
    }
}

/// ⭐⭐⭐ **Os gates da ESCULTURA no dispositivo** — ver [`sculpt`].
#[path = "smoke_gpu_sculpt_tests.rs"]
mod sculpt;
