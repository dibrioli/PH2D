//! ⭐ **O quadro do smoke** — o que se vê na área da janela.
//!
//! ⚠️ É um **módulo-filho** de [`super`] e não um irmão de topo: `field3d_smoke::draw` continua a
//! ser o caminho, pelo re-export, e o estado (`Smoke`, `Grip`, `Drag`) fica onde estava. O corte é
//! por **assunto** — o irmão guarda o estado e responde perguntas sobre ele; este pinta.

use super::*;

/// Desenha o smoke sobre a área dada. No-op silencioso quando a variável não está posta.
pub(crate) fn draw(
    area: EditorRect,
    theme: ph2d_tokens::Theme,
    text: &mut ph2d_text::TextSystem,
    scene_out: &mut VectorScene,
) {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        let smoke = slot.get_or_insert_with(boot);
        let Some(smoke) = smoke.as_mut() else {
            return;
        };

        // Colhe o traçado que ficou pronto, se ficou.
        if let Some(rx) = &smoke.inflight {
            match rx.try_recv() {
                Ok(r) => {
                    if !smoke.announced {
                        smoke.announced = true;
                        // ⚠️ Uma linha, uma vez. É ela que separa "o smoke subiu" de "o smoke
                        // DESENHOU": o boot já imprime acima, e um boot sem quadro é exatamente o
                        // modo de falha em que a janela fica vazia e ninguém sabe de quem é a culpa.
                        // Zero pixels aqui = a peça está fora do quadro ou o campo saiu vazio.
                        println!(
                            "[field-smoke] primeiro quadro desenhado — {}x{}, {} pixels de peça, \
                             {} de borda re-amostrada, {:.1} ms",
                            r.width, r.height, r.hits, r.edges, r.millis
                        );
                    }
                    smoke.last_trace_ms = r.millis as f32;
                    smoke.frame = Some((Arc::new(r.rgba), r.width, r.height));
                    smoke.inflight = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => smoke.inflight = None,
            }
        }

        // ⚠️ **O traçado sai no tamanho REAL da área** (correção do smoke de 19/08: *"render
        // pixelado"*). Antes ele era fixo em 640×480 e o desenho reamostrava para a área — e uma
        // imagem reamostrada é uma imagem com metade da informação, num módulo cuja razão de
        // existir é a nitidez da aresta. *A resolução do traçado é a da tela, não a de um número
        // que alguém escolheu.*
        let (tw, th) = (
            (area.w.round().max(1.0) as u32).max(MIN_TRACE),
            (area.h.round().max(1.0) as u32).max(MIN_TRACE),
        );

        smoke.area = Some(area);

        // ⚠️ **Peça vazia é uma resposta, não um erro** — e a tela tem de a mostrar. Guardar o
        // último quadro válido faria a imagem mentir sobre a cena, que é a irmã exacta do
        // congelador que este módulo já pagou no cache do traçado.
        let Some(doc) = smoke.doc.clone() else {
            smoke.frame = None;
            smoke.requested = None;
            return;
        };

        if !smoke.manual && smoke.inflight.is_none() {
            let dt = smoke.since.elapsed().as_secs_f32();
            smoke.since = std::time::Instant::now();
            // Trava o passo: se a janela ficou minimizada meia hora, a peça não dá vinte voltas
            // de uma vez.
            // ⚠️ Em torno do Y do **MUNDO**, e não de um eixo da câmera: um prato giratório é
            // exatamente isso — a peça a girar sobre a mesa, com o horizonte parado. É o único
            // sítio deste módulo onde um eixo do mundo entra na conta, e é de propósito.
            smoke
                .cam
                .turn_world([0.0, 1.0, 0.0], -SPIN_RATE * dt.min(0.25));
        }

        // ⭐ **Só se traça o que MUDOU.** Uma requisição em voo por vez, e só quando a câmera ou o
        // tamanho já não são os do último pedido — senão uma cena parada re-traçaria o mesmo quadro
        // para sempre, queimando um núcleo por nada. Com o prato a girar a câmera muda todo quadro,
        // então isto é invisível ali; com a mão no controlo, uma peça parada custa **zero**.
        let stale = needs_trace(
            smoke.requested.as_ref(),
            &smoke.cam,
            tw,
            th,
            &doc,
            smoke.frame.is_some(),
        );
        if smoke.inflight.is_none() && stale {
            smoke.requested = Some((smoke.cam, tw, th, doc.clone()));
            let (tx, rx) = channel::<Ready>();
            let cam = smoke.cam;
            let matcap = Arc::clone(&smoke.matcap);
            // ⚠️ O registo de esculturas atravessa a fronteira da thread como **cópia dos `Arc`** —
            // o `thread_local` que o guarda não existe do outro lado.
            let reg = crate::field3d_smoke::sampled_registry();
            std::thread::spawn(move || {
                let t0 = std::time::Instant::now();
                let g = trace(&doc, &reg, &cam, tw, th);
                let rgba = shade(
                    &g,
                    &Matcap {
                        side: matcap.side,
                        rgb_linear: &matcap.rgb,
                    },
                    BACKGROUND,
                );
                // O receptor pode ter sumido (janela fechada): descartar é a resposta certa.
                let _ = tx.send(Ready {
                    rgba,
                    width: tw,
                    height: th,
                    hits: g.hits(),
                    edges: g.edges.len(),
                    millis: t0.elapsed().as_secs_f64() * 1000.0,
                });
            });
            smoke.inflight = Some(rx);
        }

        if let Some((frame, fw, fh)) = &smoke.frame {
            // O quadro cobre a área toda — ele foi traçado com a proporção dela. Enquanto o
            // primeiro traçado do tamanho novo não chega, o anterior estica; é um quadro só, e
            // esticar é melhor do que piscar.
            scene_out.draw_image_rgba_premultiplied_transformed(
                frame,
                *fw,
                *fh,
                Affine::translate((f64::from(area.x), f64::from(area.y)))
                    * Affine::scale_non_uniform(
                        f64::from(area.w) / f64::from(*fw),
                        f64::from(area.h) / f64::from(*fh),
                    ),
                // ⚠️ **Bilinear, não bicúbico.** No caso normal o mapeamento é 1:1 e os dois são a
                // identidade — mas o bicúbico **toca** (*ringing*) numa aresta de alto contraste, e
                // agora que a aresta sai anti-serrilhada do traçador, um halo posto pelo filtro
                // seria o próprio artefato que se acabou de remover.
                ImageQuality::Medium,
            );
        }

        // ⭐ **O gizmo por cima da peça**, e no referencial da área.
        //
        // ⚠️ Ele é desenhado **depois** do quadro traçado e **sem teste de profundidade**: uma alça
        // escondida por trás da superfície que ela move seria inalcançável exatamente quando o
        // artista precisa dela. É o que todo modelador faz, e a razão é essa.
        if let Some(anchor) = smoke.gizmo {
            let screen = ph2d_field_render::Screen::new(tw, th, smoke.cam.half_extent);
            let handles =
                crate::field3d_gizmo::project(anchor, &smoke.cam, screen, smoke.gizmo_mode);
            let hot = crate::field3d_input::hot_handle(smoke);
            scene_out.push_clip(&ph2d_vector::Rect::new(
                f64::from(area.x),
                f64::from(area.y),
                f64::from(area.x + area.w),
                f64::from(area.y + area.h),
            ));
            crate::field3d_gizmo_paint::paint(scene_out, &handles, hot, theme, [area.x, area.y]);
            // ⭐ **O NÚMERO do gesto**, ao lado do gizmo — só durante o arrasto.
            //
            // ⚠️ Ele sai do que o mundo **aplicou** (`Grip::applied`), nunca de uma segunda conta a
            // partir do cursor: com o gesto preso à grelha, as duas discordariam e a ficha diria
            // `0,503` enquanto a peça pousou em `0,500`. É a lei que o `gizmo/readout.rs` da casa já
            // escreveu, e o gate `the_readout_is_the_pose_the_world_took` prende-a aqui.
            if let Some(grip) = smoke.drag_grip {
                let Some((o2, _)) = smoke.cam.project(anchor.origin, screen) else {
                    return;
                };
                crate::field3d_gizmo_paint::paint_readout(
                    scene_out,
                    text,
                    grip.applied,
                    [area.x + o2[0], area.y + o2[1]],
                    theme,
                );
            }
            scene_out.pop_layer();
        }
    });
}
