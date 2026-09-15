//! ⭐⭐⭐ **O QUE CORRE NA THREAD DE TRAÇADO** — do pedido à imagem.
//!
//! ⚠️ **Ele saiu do [`crate::smoke_draw`] em 2026-09-15 por TETO DE LINHAS**, e a fronteira é a que
//! o módulo já declarava por escrito: *o estado do módulo não atravessa a fronteira*. O que arma o
//! pedido fica lá; o que o **responde**, sem tocar no `Smoke`, fica aqui.
//!
//! ⛔⛔ **É por isso que o [`Pedido`] é uma struct e não uma lista de argumentos:** cada campo dele
//! foi COPIADO do módulo antes de a thread nascer, e o doc de cada um diz porquê. Passá-los soltos
//! deixaria a próxima adição livre de ler o módulo em vez de o copiar — que é exactamente o defeito
//! que a cópia existe para impedir (*«um olhar lido depois podia já não ser o do pedido que este
//! traçado responde»*).

use super::{BACKGROUND, Ready};
use ph2d_field::FieldDoc;
use ph2d_field_render::Matcap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::SyncSender;

/// ⭐⭐⭐ **O pedido que atravessa a fronteira da thread** — tudo copiado, nada lido depois.
pub(crate) struct Pedido {
    /// A peça, já com o contorno engrossado do quadro de movimento quando é o caso.
    pub doc: FieldDoc,
    pub reg: ph2d_field_eval::hybrid::Registry,
    pub cam: ph2d_field_render::Orbit,
    pub tw: u32,
    pub th: u32,
    /// ⭐ **A bandeira da W73** — *grosso a mexer, nítido ao assentar*. Ela governa o contorno, o
    /// anti-serrilhado, a sombra e o despacho de borda do dispositivo; **uma** pergunta, quatro
    /// passageiros.
    pub antialias: bool,
    pub shading: crate::shading::Shading,
    pub look: ph2d_view_transform::Look,
    pub matcap: Arc<super::MatcapTexels>,
    pub materials: Option<Arc<crate::materials::Table>>,
    /// ⚠️ **As luzes ACESAS**, e não a lista do módulo: esta tem também as apagadas, porque o gizmo
    /// do canvas precisa de as desenhar para se poderem voltar a acender.
    pub lights: Vec<ph2d_field_render::PointLamp>,
    pub tapes: Arc<ph2d_field_render::TapeCache>,
    pub usa_cache: bool,
    pub refinar: bool,
    pub flag: Arc<AtomicBool>,
    pub tx: SyncSender<Ready>,
    pub gpu: Option<&'static crate::gpu_frame::SharedTracer>,
}

/// ⭐⭐⭐ **A resposta a um pedido** — corre fora da thread que desenha.
#[allow(clippy::too_many_lines)] // é o quadro inteiro: o dispositivo, a CPU, os dois modos e o refinamento
pub(crate) fn traca(p: &Pedido) {
    let t0 = std::time::Instant::now();
    // ⭐⭐⭐ **O DISPOSITIVO, quando ele pode.** As três condições vivem numa porta
    // ([`crate::gpu_frame::takes_the_frame`]), e a que mais importa é a terceira: uma peça
    // com ESCULTURA fica na CPU, senão ela desapareceria em silêncio.
    let mundos: Vec<[f32; 3]> = p.lights.iter().map(|l| l.world).collect();
    let pelo_dispositivo = matches!(p.shading, crate::shading::Shading::Render)
        && !mundos.is_empty()
        && crate::gpu_frame::takes_the_frame(p.gpu, &p.doc);
    // ⭐⭐⭐ **E O DISPOSITIVO TAMBÉM PINTA** (`docs/Render3d/05` §39): com as quatro leis
    // do pintor no dispositivo — o material, o céu, o olhar e o **dono** — o G-buffer
    // deixa de atravessar o barramento e o que volta é a IMAGEM.
    //
    // ⚠️ **Só quando o refinamento de CPU está desligado**, que é o caminho de omissão: com
    // ele ligado (a porta de bissecção da [`crate::preview`]) o quadro precisa do G-buffer
    // para p.refinar sobre ele, e essa é a única razão para o trazer de volta.
    let padrao_gpu;
    let pintado = if pelo_dispositivo && !p.refinar {
        let surfaces = match &p.materials {
            Some(t) => t.surfaces_for(),
            None => {
                padrao_gpu = [ph2d_material::OpenPbr::default().prepare()];
                ph2d_field_render::Surfaces {
                    all: &padrao_gpu,
                    owners: None,
                }
            }
        };
        p.gpu.as_ref().and_then(|t| {
            crate::gpu_frame::paint(
                t,
                &p.doc,
                &p.reg,
                &p.cam,
                &p.lights,
                &surfaces,
                p.look,
                BACKGROUND,
                p.tw,
                p.th,
                p.antialias,
            )
        })
    } else {
        None
    };
    if let Some(pintura) = pintado {
        // ⚠️ **O `hits` do caminho pintado conta PIXELS COM TINTA**, e não acertos do
        // centro: o G-buffer ficou no dispositivo. As duas contagens divergem nas bordas
        // cujo centro falhou — é um número de diagnóstico, e dizê-lo aqui é mais honesto
        // do que trazer `49,8 MB` para o calcular.
        let hits = pintura
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| px[3] > BACKGROUND[3])
            .count();
        let _ = p.tx.try_send(Ready {
            rgba: pintura.rgba,
            width: p.tw,
            height: p.th,
            hits,
            edges: pintura.edges,
            millis: t0.elapsed().as_secs_f64() * 1000.0,
            passagem: 0,
            mais: false,
        });
        return;
    }
    let do_gpu = if pelo_dispositivo {
        p.gpu.as_ref().and_then(|t| {
            crate::gpu_frame::march(t, &p.doc, &p.reg, &p.cam, &mundos, p.tw, p.th, p.antialias)
        })
    } else {
        None
    };
    // Abandonado a meio: não se manda nada, e quem esperava já mudou de pedido.
    // ⚠️ O G-buffer **move-se**: ele tem milhões de pixels e não é `Clone` de propósito.
    let (g, sombras_do_gpu) = match do_gpu {
        Some((g, sh)) => (g, Some(sh)),
        None => {
            let Some(g) = ph2d_field_render::trace_cancellable(
                &p.doc,
                &p.reg,
                &p.cam,
                p.tw,
                p.th,
                &p.flag,
                p.antialias,
                p.usa_cache.then_some(&*p.tapes),
            ) else {
                return;
            };
            (g, None)
        }
    };
    let rgba = match p.shading {
        crate::shading::Shading::Matcap => ph2d_field_render::shade_with(
            &g,
            &Matcap {
                side: p.matcap.side,
                rgb_linear: &p.matcap.rgb,
            },
            p.look,
            BACKGROUND,
        ),
        crate::shading::Shading::Render => {
            // ⭐⭐⭐ **AS LUZES SÃO OBJECTOS DA CENA** (ordem do dono, 14/09) — e viajam com o
            // pedido pela MESMA razão que a tabela de materiais: o estado do módulo não
            // atravessa a fronteira, e uma lista lida depois podia já não ser a do pedido
            // que este traçado responde.
            //
            // ⛔ **O rig ancorado no ECRÃ da `ph2d-light` deixou de acender o Render.** Ele
            // continua a acender o **p.matcap** (que é sombreamento de vista, por definição) e
            // a tinta e a escultura, que são outros módulos. *Uma cena sem luz nenhuma sai
            // acesa só pelo céu — que é o que ela de facto tem.*
            let lamps: [ph2d_field_render::Lamp; 0] = [];
            let padrao;
            let surfaces = match &p.materials {
                Some(t) => t.surfaces_for(),
                // ⚠️ Sem tabela (o primeiro quadro de uma cena) a peça usa o material de
                // omissão — o mesmo que a ausência do componente significa.
                None => {
                    padrao = [ph2d_material::OpenPbr::default().prepare()];
                    ph2d_field_render::Surfaces {
                        all: &padrao,
                        owners: None,
                    }
                }
            };
            // ⭐⭐⭐ **A SOMBRA VIAJA NA BANDEIRA QUE JÁ EXISTE** (`coarse`, W73).
            //
            // O módulo tem desde a W73 uma lei escrita — *grosso a mexer, nítido ao
            // assentar* — com dois passageiros (o contorno engrossado e o anti-serrilhado
            // desligado) e uma nota a avisar que *«uma segunda pergunta para o mesmo facto
            // podia divergir dela»*. A sombra é o **terceiro**, e não uma pergunta nova.
            //
            // ⭐ **Porque é ela e não um knob** (medido 2026-09-14, `load 2,53`, três
            // cilindros cruzados — a tabela inteira em [`ph2d_field_render::shadow`]):
            // um raio de sombra custa `29,3` amostras contra `8,7` de um raio de câmera, e o
            // passe pesa `1,74×` o traçado a `640×360` e `1,97×` a `1920×1080`.
            //
            // | | traçado | + sombra |
            // |---|---:|---:|
            // | movimento (`640×360`, piso D=3) | `3,51 ms` | `9,60 ms` de `16,7` |
            // | assente (`1920×1080`) | `28,61 ms` | `84,84 ms` |
            //
            // ⇒ pendurá-la no quadro de movimento **cabia** nesta peça e comia metade da
            // folga que o laço do divisor tem para as pesadas — e o
            // [`crate::preview`] declara por escrito que uma peça que não caiba no piso
            // *«fica presa nele e a imagem fica lenta»*. Pendurada no quadro que assenta, o
            // de movimento fica **byte-idêntico** ao de hoje e o preço corre onde já
            // corriam `28,6 ms`, **noutra thread**, com a janela na mesma a 60 Hz.
            //
            // ⚠️ **O `p.antialias` É a bandeira** (`= !coarse`, linha 413) — lido aqui, e não
            // uma segunda pergunta ao mesmo facto.
            // ⭐ **A sombra e a oclusão vêm do dispositivo quando ele traçou** — elas
            // saíram da MESMA marcha, e refazê-las aqui seria pagar duas vezes pela mesma
            // resposta.
            let mut sombras = match sombras_do_gpu {
                Some(sh) => Some(sh),
                None => p
                    .antialias
                    .then(|| ph2d_field_render::shadow_pass(&p.doc, &p.reg, &p.cam, &g, &mundos)),
            };
            let pinta = |sh: Option<&ph2d_field_render::Shadows>| {
                ph2d_field_render::shade_render(
                    &g,
                    &p.cam,
                    &surfaces,
                    &ph2d_field_render::Lighting {
                        lamps: &lamps,
                        points: &p.lights,
                        sky: &crate::render_light::StudioSky,
                        shadows: sh,
                    },
                    p.look,
                    BACKGROUND,
                )
            };
            // ⭐⭐⭐ **A PASSAGEM `0`: a imagem com a sombra directa, já.**
            let _ = p.tx.try_send(Ready {
                rgba: pinta(sombras.as_ref()),
                width: p.tw,
                height: p.th,
                hits: g.hits(),
                edges: g.edges.len(),
                millis: t0.elapsed().as_secs_f64() * 1000.0,
                passagem: 0,
                mais: p.refinar,
            });
            if !p.refinar {
                return;
            }
            // ⭐⭐⭐ **E DEPOIS A OCLUSÃO REFINA, uma passagem de cada vez**
            // (`docs/Render3d/05` §30). O G-buffer já está pago e não se re-traça: cada
            // passagem é **um raio por pixel** mais uma pintura, e a imagem vai assentando
            // com a mão parada — o idioma de toda viewport de render.
            //
            // ⛔ Medido (§29.2): os `32` raios de uma só vez custam `1,35 s` a `1920×1080`.
            // Repartidos, cada passagem custa o que um quadro tolera e o artista vê a
            // peça a ganhar profundidade em vez de esperar por ela.
            // ⚠️ A LEI vive na porta ([`ph2d_field_render::refine_occlusion`]); aqui
            // fica só o que é da thread: pintar, mandar, e dizer se vale a pena continuar.
            let mut sh = sombras.take().unwrap_or_default();
            ph2d_field_render::refine_occlusion(
                &p.doc,
                &p.reg,
                &p.cam,
                &g,
                &mut sh,
                |sh, passagem| {
                    // ⛔⛔ **O cancelamento é visto ANTES da pintura**, e foi a segunda
                    // metade do report *«mover os objetos ficou muito lento»*: o
                    // `shade_render` corre em TODOS os núcleos, logo uma passagem já
                    // cancelada ainda roubava a máquina inteira ao quadro que a mão
                    // arrastava. *Largar depois de pagar não é largar.*
                    if p.flag.load(std::sync::atomic::Ordering::Relaxed) {
                        return false;
                    }
                    let ultima = passagem == ph2d_field_render::OCCLUSION_PASSES;
                    let _ = p.tx.try_send(Ready {
                        rgba: pinta(Some(sh)),
                        width: p.tw,
                        height: p.th,
                        hits: g.hits(),
                        edges: g.edges.len(),
                        millis: t0.elapsed().as_secs_f64() * 1000.0,
                        passagem,
                        mais: !ultima,
                    });
                    // ⚠️ **A mão voltou a mexer**: parar aqui é o que impede o refinamento
                    // de queimar um núcleo por uma imagem que já não se vê.
                    !p.flag.load(std::sync::atomic::Ordering::Relaxed)
                },
            );
            return;
        }
    };
    // O receptor pode ter sumido (janela fechada): descartar é a resposta certa.
    let _ = p.tx.try_send(Ready {
        rgba,
        width: p.tw,
        height: p.th,
        hits: g.hits(),
        edges: g.edges.len(),
        millis: t0.elapsed().as_secs_f64() * 1000.0,
        passagem: 0,
        mais: false,
    });
}
