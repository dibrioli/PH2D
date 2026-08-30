//! ⭐ **O quadro do smoke** — o que se vê na área da janela.
//!
//! ⚠️ É um **módulo-filho** de [`super`] e não um irmão de topo: `field3d_smoke::draw` continua a
//! ser o caminho, pelo re-export, e o estado (`Smoke`, `Grip`, `Drag`) fica onde estava. O corte é
//! por **assunto** — o irmão guarda o estado e responde perguntas sobre ele; este pinta.

/// ⚠️ **A cache de fitas entre quadros está ligada?** `PH2D_FIELD_TAPE_CACHE=0` desliga-a — a porta
/// de bissecção da W82. *Um interruptor de bissecção é a diferença entre «piorou» e «piorou por
/// causa disto».*
pub(crate) fn tape_cache_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_FIELD_TAPE_CACHE").as_deref() != Ok("0"))
}

use super::*;

/// ⭐ **O diagnóstico do laço do preview** (W24): imprime cada traçado com o tamanho que ele teve.
///
/// ⚠️ É a única forma de ver o laço a decidir — o divisor não aparece em lado nenhum na tela, e é
/// suposto não aparecer: o artista vê a peça, não a régua.
fn trace_log() -> bool {
    std::env::var("PH2D_FIELD_TRACE_LOG").is_ok()
}

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

        // ⚠️ **Peça vazia é uma resposta, não um erro** — e a tela tem de a mostrar. Guardar o
        // último quadro válido faria a imagem mentir sobre a cena, que é a irmã exacta do
        // congelador que este módulo já pagou no cache do traçado.
        let Some(doc) = smoke.doc.clone() else {
            for v in &mut smoke.vps {
                v.frame = None;
                v.requested = None;
            }
            return;
        };

        // ⭐⭐⭐ **A DIVISÃO** (W90) — os retângulos saem da porta única
        // ([`crate::field3d_layout`]), em pixels inteiros, e **todo** consumidor lê de lá: o
        // traçado, o desenho e o *«este clique é meu?»*.
        let layout = crate::field3d_layout::rects(area, smoke.split);
        // ⚠️ **Copiados para a pilha antes do laço**: o passe mexe em `smoke`, e um empréstimo do
        // layout vivo lá dentro seria um empréstimo de `smoke` também.
        let mut quadros = [ph2d_editor::zones::Rect::new(0.0, 0.0, 0.0, 0.0); 4];
        let n = layout.as_slice().len();
        quadros[..n].copy_from_slice(layout.as_slice());
        crate::field3d_smoke::ensure_viewports(smoke, n);
        // ⭐⭐⭐ **A ACTIVA PASSA PRIMEIRO** (W90c), e a ordem é a lei — não uma preferência.
        //
        // ⛔ A guarda de prioridade dentro do passe pergunta *«a activa tem traçado em voo?»*, e
        // numa ordem natural (`0..n`) as vistas que correm **antes** dela escapam-lhe no primeiro
        // tique: naquele instante a activa ainda não começou, então a resposta é «não». *Uma
        // prioridade que depende de quem chega primeiro não é uma prioridade* — e foi o gate
        // `the_active_viewport_gets_its_image_first` que o disse, com as duas a empatar no tique 2.
        //
        // ⚠️ A ordem de PINTURA é indiferente: os retângulos não se sobrepõem (é o que o gate
        // `the_four_pieces_tile_the_area_exactly` garante).
        let activa = smoke.active.min(n - 1);
        for i in std::iter::once(activa).chain((0..n).filter(|k| *k != activa)) {
            viewport_pass(smoke, i, quadros[i], &doc, scene_out);
        }
        // ⭐⭐ **O RÓTULO DE CADA VISTA** (W90d) — só com a divisão aberta: com uma vista só, a
        // pergunta *«qual é qual?»* não existe, e um rótulo permanente seria ruído sobre a peça.
        if n > 1 {
            for (i, r) in quadros[..n].iter().enumerate() {
                let key = crate::field3d_views::label_key(&smoke.vps[i].cam);
                crate::field3d_gizmo_paint::paint_view_label(scene_out, text, *r, key, theme);
            }
        }

        // ⭐⭐⭐ **AS COSTURAS E A MOLDURA DO ACTIVO** (W90) — por cima das imagens e por baixo do
        // gizmo, que é onde uma moldura de janela vive.
        crate::field3d_gizmo_paint::paint_split(
            scene_out,
            &quadros[..n],
            smoke.active.min(n - 1),
            theme,
        );

        // ⭐ **O chrome é do viewport ACTIVO**, e daqui para baixo `area` é o retângulo dele.
        //
        // ⚠️ **Um gizmo por quadrante seria quatro respostas à mesma pergunta.** O gesto acontece
        // num viewport de cada vez (`Smoke::active`), e as alças são a projecção do gesto: pintá-las
        // nas quatro vistas convidaria a agarrar a de uma vista e a arrastar noutra.
        let area = quadros[smoke.active.min(n - 1)];

        // ⭐⭐ **O GIZMO DE NAVEGAÇÃO** (W49), na quina superior direita — como no Blender e no Unity.
        //
        // ⚠️ Ele é pintado **sempre**, e não dentro da guarda de seleção que vem a seguir: ele diz de
        // que lado do modelo se está a olhar, e essa pergunta não depende de haver algo escolhido.
        {
            let safe = crate::field3d_smoke::safe_of(smoke);
            let balls = crate::field3d_navball::balls(&smoke.vp().cam, area, safe);
            crate::field3d_navball_paint::paint(
                scene_out,
                &balls,
                smoke.nav_hot,
                theme,
                [area.x, area.y],
                crate::field3d_navball::centre_in(area, safe),
            );
        }

        // ⭐⭐ **A MOLDURA DO LAÇO** (W58) — e ela é pintada **sempre**, como o gizmo de navegação
        // logo acima e pela mesma razão: ela diz **o que a mão está a fazer**, e essa pergunta não
        // depende de haver algo escolhido.
        //
        // ⛔ **A 1.ª versão pô-la DENTRO da guarda de seleção que vem a seguir** (Enio, 2026-08-24:
        // *"o desenho do retângulo de seleção deixou de aparecer"*): sem nada selecionado não há
        // âncora de gizmo, o bloco inteiro é saltado, e o laço mais comum de todos — o primeiro,
        // com a peça acabada de abrir — desenhava **nada**. ⚠️ *O parágrafo do navball, uma linha
        // acima, já escrevia esta lei; eu pus o código do outro lado dela.*
        //
        // ⚠️ Ela sai do campo do **GESTO** (`smoke.lasso`), nunca do pedido: pintar a partir do
        // pedido faria a moldura sobreviver ao dedo por um quadro.
        if let Some((from, to)) = smoke.lasso {
            scene_out.push_clip(&ph2d_vector::Rect::new(
                f64::from(area.x),
                f64::from(area.y),
                f64::from(area.x + area.w),
                f64::from(area.y + area.h),
            ));
            crate::field3d_gizmo_paint::paint_lasso(scene_out, from, to, theme, [area.x, area.y]);
            scene_out.pop_layer();
        }

        // ⭐ **O gizmo por cima da peça**, e no referencial da área.
        //
        // ⚠️ Ele é desenhado **depois** do quadro traçado e **sem teste de profundidade**: uma alça
        // escondida por trás da superfície que ela move seria inalcançável exatamente quando o
        // artista precisa dela. É o que todo modelador faz, e a razão é essa.
        if let Some(anchor) = smoke.gizmo {
            // ⚠️ **A projeção é a da ÁREA e vem do dono dela** ([`crate::field3d_input::handles`]) —
            // nunca uma segunda conta a partir do tamanho do traçado. Desde a W24 os dois números
            // são diferentes enquanto a mão mexe, e uma cópia aqui poria as alças a um terço do
            // tamanho: o gizmo agarraria longe da superfície, **só durante o movimento**.
            let Some(screen) = crate::field3d_input::area_screen(smoke) else {
                return;
            };
            let handles = crate::field3d_input::handles(smoke);
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
                let Some((o2, _)) = smoke.vp().cam.project(anchor.origin, screen) else {
                    return;
                };
                let at = [area.x + o2[0], area.y + o2[1]];
                // ⭐ **A digitar, a ficha mostra o que está a ser ESCRITO** (W26) — e não o valor.
                //
                // ⚠️ Enquanto se escreve `-0.` não há número nenhum, e uma ficha que saltasse para
                // `0,000` mentiria sobre o que a tecla seguinte vai fazer. Assim que o texto é um
                // número, o mundo já o aplicou — as duas metades dizem a mesma coisa.
                match (
                    smoke.typed.as_deref(),
                    crate::field3d_input::hot_handle(smoke),
                ) {
                    (Some(t), Some(handle)) => crate::field3d_gizmo_paint::paint_readout_text(
                        scene_out,
                        text,
                        &crate::field3d_typed::label(handle, t),
                        at,
                        theme,
                    ),
                    _ => crate::field3d_gizmo_paint::paint_readout(
                        scene_out,
                        text,
                        grip.applied,
                        at,
                        theme,
                    ),
                }
            }
            scene_out.pop_layer();
        }
    });
}

/// ⭐⭐⭐ **O PASSE DE UM VIEWPORT** (W90) — colher, decidir, pedir, e pôr a imagem no sítio.
///
/// ⚠️ **Era o corpo do [`draw`]**, escrito uma vez para a única vista que o módulo tinha. O que
/// mudou não foi a lei — é a mesma escada de `next_trace`, o mesmo cancelamento, o mesmo laço
/// fechado da resolução —, foi o **sujeito**: ela passa a ser sobre `vps[i]`, e não sobre «a»
/// câmera.
fn viewport_pass(
    smoke: &mut crate::field3d_smoke::Smoke,
    i: usize,
    area: ph2d_editor::zones::Rect,
    doc: &FieldDoc,
    scene_out: &mut VectorScene,
) {
    // Colhe o traçado que ficou pronto, se ficou.
    if let Some(job) = &smoke.vps[i].inflight {
        match job.rx.try_recv() {
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
                smoke.vps[i].last_trace_ms = r.millis as f32;
                // ⭐ **A medição que fecha o laço** (W24): o tempo **com** os pixels a que foi
                // medido. O pedido seguinte sai daqui, e é por isso que este módulo não precisa
                // de saber em que máquina corre.
                smoke.vps[i].measured = Some(crate::field3d_preview::Measured {
                    pixels: u64::from(r.width) * u64::from(r.height),
                    millis: r.millis as f32,
                });
                if trace_log() {
                    println!(
                        "[field-smoke] traçado {}x{} em {:.1} ms ({} px de peça)",
                        r.width, r.height, r.millis, r.hits
                    );
                }
                // ⭐⭐⭐ **O handle nasce AQUI — uma vez por TRAÇADO, não uma vez por quadro.**
                // É a linha inteira da cura do atlas persistente: o id da imagem passa a mudar
                // quando os **pixels** mudam, que é a única altura em que ele devia mudar.
                // ⚠️ `premultiplied` porque é o que o `shade` produz — o mesmo tipo de alfa que a
                // porta crua declarava. Entrar pelo `from_rgba` faria o Vello pré-multiplicar
                // **outra vez**, e a borda da peça escureceria.
                // ⚠️ `None` é inalcançável (o `shade` devolve `w*h*4` por construção) e ainda
                // assim não se desembrulha: um `expect` aqui derrubaria a janela por um traçado
                // malformado, e a resposta certa a *«a imagem não presta»* é a de sempre — a
                // anterior fica, esticada.
                smoke.vps[i].frame = ph2d_vector::StableImage::from_rgba_premultiplied(
                    Arc::new(r.rgba),
                    r.width,
                    r.height,
                );
                smoke.vps[i].inflight = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => smoke.vps[i].inflight = None,
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

    smoke.vps[i].area = Some(area);

    if !smoke.vps[i].manual && smoke.vps[i].inflight.is_none() {
        let dt = smoke.vps[i].since.elapsed().as_secs_f32();
        smoke.vps[i].since = std::time::Instant::now();
        // Trava o passo: se a janela ficou minimizada meia hora, a peça não dá vinte voltas
        // de uma vez.
        // ⚠️ Em torno do Y do **MUNDO**, e não de um eixo da câmera: um prato giratório é
        // exatamente isso — a peça a girar sobre a mesa, com o horizonte parado. É o único
        // sítio deste módulo onde um eixo do mundo entra na conta, e é de propósito.
        smoke
            .vp_mut()
            .cam
            .turn_world([0.0, 1.0, 0.0], -SPIN_RATE * dt.min(0.25));
    }

    // ⭐ **Só se traça o que MUDOU — e o TAMANHO do que se traça sai da medição** (W24).
    //
    // Uma requisição em voo por vez. Quando a câmera ou o documento mudam, o pedido sai no
    // tamanho que **cabe num quadro** segundo o que o último traçado custou; quando nada muda e
    // o que está na tela ainda é grosso, sai o **cheio**. Uma cena parada e já nítida custa
    // **zero** — senão re-traçaria o mesmo quadro para sempre, queimando um núcleo por nada.
    let ask = crate::field3d_preview::next_trace(
        // ⛔⛔ **ERA `smoke.vp()` — o viewport ACTIVO** (report do Enio, 27/08: *«apenas a janela
        // activa fica com o objecto liso, as demais ficam no modo de baixa resolução»*). Cada
        // viewport perguntava *«a câmera mudou?»* comparando a câmera DELE com o pedido de OUTRO
        // ⇒ para toda vista não-activa a resposta era **sempre sim**, e ela ficava presa no quadro
        // de movimento para sempre, sem nunca subir os dois degraus do assentar.
        smoke.vps[i]
            .requested
            .as_ref()
            .map(|(c, w, h, d, k)| (c, *w, *h, d, *k)),
        &smoke.vps[i].cam,
        doc,
        (tw, th),
        smoke.vps[i].measured,
        smoke.vps[i].frame.is_some(),
        MIN_TRACE,
    );
    // ⭐ **E um REFINAMENTO cede à mão** (W32): se o que está em voo é o quadro cheio e o que se
    // pede agora é mais grosso, a mão voltou a mexer — abandona-se o refinamento em vez de o
    // esperar (até **121 ms** medidos). ⛔ O contrário nunca: um traçado de movimento corre até
    // ao fim, senão numa órbita contínua ele seria cancelado a cada quadro e a imagem
    // **congelava**. Ver `field3d_preview::cancels_the_inflight`.
    if let (Some(job), Some((_, _, ac))) = (&smoke.vps[i].inflight, ask)
        && crate::field3d_preview::cancels_the_inflight(job.refinement, ac)
    {
        job.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        smoke.vps[i].inflight = None;
    }
    // ⭐⭐⭐ **A VISTA ACTIVA TEM PRIORIDADE** (W90c) — e o número que a exige está medido.
    //
    // Quatro traçados a correr ao mesmo tempo **não ganham nada** por serem quatro: cada um já
    // satura a máquina com o `rayon`, então eles só se fatiam. Medido a `load 1,7`
    // (`the_price_of_four_views.rs`, uma edição a `1280×720`):
    //
    // | | ms |
    // |---|---:|
    // | uma vista, área inteira | `156,2` |
    // | uma vista, um quarto | `64,5` |
    // | **quatro ao mesmo tempo** | **`253,7`** (`3,93×` uma sozinha) |
    //
    // ⇒ sem esta guarda, a vista onde a mão do artista está espera pelas **outras três**: `254 ms`
    // em vez de `64`. *Trabalho total igual, latência percebida `3,9×` pior — e a fatia que importa
    // é sempre a mesma.*
    //
    // ⚠️ **Não há fome:** as vistas nomeadas são estáticas, então elas só têm trabalho quando o
    // DOCUMENTO muda — e nesse instante a activa também tem, e acaba primeiro. O caso em que elas
    // esperariam muito (a mão a orbitar **enquanto** o documento muda a cada quadro) é o caso em que
    // o que elas mostrariam já estaria velho.
    let activa_em_voo = i != smoke.active.min(smoke.vps.len() - 1)
        && smoke.vps[smoke.active.min(smoke.vps.len() - 1)]
            .inflight
            .is_some();
    if let (None, false, Some((tw, th, coarse))) = (&smoke.vps[i].inflight, activa_em_voo, ask) {
        // ⚠️ **O comparado é o documento REAL.** Guardar aqui o engrossado faria a cena parecer
        // mudada em toda alternância entre grosso e fino, e o laço re-traçaria para sempre.
        smoke.vps[i].requested = Some((smoke.vps[i].cam, tw, th, doc.clone(), coarse));
        // ⭐⭐⭐ **E o que vai para o traçador leva o contorno GROSSO enquanto a mão mexe**
        // (2026-08-26) — a mesma lei que já baixava os pixels, aplicada onde o custo estava: o
        // traçado paga `0,22 ms` por **aresta do contorno**, e esse custo é **cego aos pixels**.
        // Ver [`crate::field3d_preview::coarse_doc`].
        // ⭐ E os dois cortes do quadro de movimento saem da MESMA bandeira (W73): o contorno
        // grosso e o anti-serrilhado desligado são a mesma lei — *grosso a mexer, nítido ao
        // assentar* —, e uma segunda pergunta para o mesmo facto podia divergir dela.
        // ⭐⭐⭐ **Os DOIS quadros engrossam o contorno** (W85) — com orçamentos de erro
        // diferentes, e não com leis diferentes. O que ship até à W84 era «engrossa a mexer,
        // autoral ao parar»; medido, o autoral acima de `0,5°` de erro de normal compra
        // `≤3/255` no pixel e custa o dobro. Ver `field3d_preview::SETTLED_NORMAL_ERR_DEG`.
        let doc = crate::field3d_preview::coarse_doc(doc, coarse).unwrap_or_else(|| doc.clone());
        let antialias = !coarse;
        let (tx, rx) = channel::<Ready>();
        let cam = smoke.vps[i].cam;
        let matcap = Arc::clone(&smoke.matcap);
        let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = Arc::clone(&cancel);
        // ⚠️ O registo de esculturas atravessa a fronteira da thread como **cópia dos `Arc`** —
        // o `thread_local` que o guarda não existe do outro lado.
        let reg = crate::field3d_smoke::sampled_registry();
        // ⭐⭐⭐ **As fitas já compiladas atravessam a fronteira da thread** (W82) — ver
        // [`ph2d_field_render::TapeCache`]. É um `Arc`: o que viaja é o ponteiro.
        let tapes = std::sync::Arc::clone(&smoke.vps[i].tapes);
        // ⚠️ **A porta de bissecção** — `PH2D_FIELD_TAPE_CACHE=0` traça sem cache nenhuma, que é
        // o que o app fazia até à W82. Ela existe porque um report de *«piorou muito»* não diz
        // **qual** mudança o causou, e duas corridas dizem.
        let usa_cache = tape_cache_enabled();
        std::thread::spawn(move || {
            let t0 = std::time::Instant::now();
            // Abandonado a meio: não se manda nada, e quem esperava já mudou de pedido.
            let Some(g) = ph2d_field_render::trace_cancellable(
                &doc,
                &reg,
                &cam,
                tw,
                th,
                &flag,
                antialias,
                usa_cache.then_some(&*tapes),
            ) else {
                return;
            };
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
        smoke.vps[i].inflight = Some(crate::field3d_smoke::InFlight {
            rx,
            cancel,
            refinement: !coarse,
        });
    }

    if let Some(frame) = &smoke.vps[i].frame {
        // O quadro cobre a área toda — ele foi traçado com a proporção dela. Enquanto o
        // primeiro traçado do tamanho novo não chega, o anterior estica; é um quadro só, e
        // esticar é melhor do que piscar.
        //
        // ⭐ **`draw_stable_image` e não a porta crua** — o handle veio pronto do sítio onde o
        // traçado chegou, e redesenhá-lo com o **mesmo id** faz o atlas persistente acertar em vez
        // de re-enviar a imagem inteira. O `dest` em rectângulo compõe exactamente o mesmo afim
        // que aqui estava escrito à mão (`translate` × `scale_non_uniform`).
        scene_out.draw_stable_image(
            frame,
            (
                f64::from(area.x),
                f64::from(area.y),
                f64::from(area.x) + f64::from(area.w),
                f64::from(area.y) + f64::from(area.h),
            ),
            // ⚠️ **Bilinear, não bicúbico.** No caso normal o mapeamento é 1:1 e os dois são a
            // identidade — mas o bicúbico **toca** (*ringing*) numa aresta de alto contraste, e
            // agora que a aresta sai anti-serrilhada do traçador, um halo posto pelo filtro
            // seria o próprio artefato que se acabou de remover.
            ImageQuality::Medium,
        );
    }
}
