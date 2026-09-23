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
    /// ⭐ **A bandeira da W73** — *grosso a mexer, nítido ao assentar*.
    ///
    /// ⚠️⚠️ **Ela levava QUATRO passageiros e leva TRÊS desde a `W7c`** (2026-09-19,
    /// `docs/Render3d/12` §12): o contorno engrossado, a sombra directa da CPU e o ricochete com o
    /// campo do chão. ⛔ **O anti-serrilhado SAIU** — medido no caminho do pintor ele custa
    /// `1,03×`–`1,09×` do quadro de movimento, contra os `1,30×`–`1,40×` da tabela de CPU que o
    /// tinha posto fora dele, e a silhueta sem ele não tem **um único** pixel de cobertura parcial.
    ///
    /// *O nome mudou com o conteúdo: chamar-lhe `antialias` era descrever a bandeira pelo
    /// passageiro mais barato dos quatro.*
    pub assente: bool,
    pub shading: crate::shading::Shading,
    pub look: ph2d_view_transform::Look,
    /// ⭐⭐⭐ **A CAMADA DE ESTILO da cena** (`docs/Render3d/03`, a `W8`) — os botões da direcção de
    /// arte, que entram entre a física e o olhar.
    ///
    /// ⚠️ **Copiado já SANEADO** ([`ph2d_style::Style::sanitized`]), como tudo o que atravessa esta
    /// fronteira: a cerca é de QUADRO e correria por pixel se viajasse crua.
    pub style: ph2d_style::Style,
    /// ⭐ **O BRILHO da cena** (a `W7`) — copiado como o estilo, antes de a thread nascer.
    pub bloom: ph2d_field_render::Bloom,
    pub matcap: Arc<super::MatcapTexels>,
    pub materials: Option<Arc<crate::materials::Table>>,
    /// ⚠️ **As luzes ACESAS**, e não a lista do módulo: esta tem também as apagadas, porque o gizmo
    /// do canvas precisa de as desenhar para se poderem voltar a acender.
    pub lights: Vec<ph2d_field_render::PointLamp>,
    /// ⭐⭐⭐ **O CHÃO que só recebe** (`docs/Render3d/07`) — `None` fora do Render. Copiado da
    /// âncora do módulo ([`crate::floor`]) antes de a thread nascer, como as luzes.
    pub ground: Option<ph2d_field_render::Ground>,
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
    // ⭐⭐⭐ **A APRESENTAÇÃO DA CENA, montada UMA vez e entregue aos DOIS motores** — o olhar, o
    // estilo e a escala da peça (`docs/Render3d/03`, a `W8`).
    //
    // ⚠️⚠️ **Ela nasce ANTES do ramo do dispositivo de propósito, e a razão é o §24 do
    // `docs/Render3d/10`:** aquele defeito foi uma cura escrita no caminho de REFERÊNCIA enquanto o
    // caminho de OMISSÃO — este, o pintado — devolvia antes de a alcançar. *Duas variáveis montadas
    // em dois ramos são exactamente como isso volta a acontecer;* com uma só, o que o artista vê e o
    // que a régua mede são a mesma apresentação por construção.
    //
    // ⚠️ **Sem peça (documento vazio) o raio é `1`** e é inofensivo: sem tinta de curvatura e sem
    // subsuperfície ninguém o lê, porque a própria curvatura não chega a ser medida.
    let apresentacao = ph2d_field_render::Presentation {
        look: p.look,
        style: p.style,
        piece_radius: ph2d_field_eval::bounds::bounding_ball(&p.doc, &p.reg)
            .map_or(1.0, |b| b.radius),
        bloom: p.bloom,
    };
    // ⭐⭐⭐⭐ **E O MODO DE OMISSÃO JÁ VAI À PLACA** (report do dono, 2026-09-23: *«ao arrastar
    // fica grosseiro ainda»*). O `#[default]` do [`crate::shading::Shading`] é o **matcap**, e
    // enquanto só o `Render` era pintado no dispositivo o arrasto de uma cena recém-aberta era
    // **todo** de CPU: `90,17 ms` e o prévio a escolher `D=3` — um **nono** dos píxeis — contra
    // `16,63 ms` e `D=1`.
    //
    // ⚠️⚠️ **A condição junta DUAS perguntas e elas continuam separadas:** *«a placa sabe MARCHAR
    // esta peça?»* é GEOMETRIA e não tem modo ([`crate::gpu_frame::takes_the_frame`]); *«a placa
    // sabe PINTÁ-LA?»* depende da LEI de pintura, e há duas — o material (que precisa dos
    // materiais, do céu, do olhar e das lâmpadas) e o **matcap**, que precisa da normal e de uma
    // fotografia.
    //
    // ⛔⛔⛔ **E a primeira cura que eu desenhei era ABRIR A MARCHA ao matcap — ela PIORAVA.** O
    // [`crate::gpu_frame::march`] devolve o G-buffer pelo barramento (`49,8 MB` a `1920×1080`,
    // `119`–`123 ms`), que é **mais lento do que a CPU inteira**. *O ganho nunca foi a marcha estar
    // na placa; é a IMAGEM não atravessar o barramento* — e é por isso que a cura é um PASSE que
    // pinta, e não uma condição alargada.
    //
    // ⭐⭐ **O passe do matcap liga `8` armazéns contra os `9` do pintor de material**, que é o
    // piso garantido do WebGPU ⇒ *o modo de omissão corre em toda placa conforme, e o de material
    // só onde há folga* (ver [`ph2d_field_gpu::matcap`]).
    let pelo_dispositivo = matches!(p.shading, crate::shading::Shading::Render)
        && !mundos.is_empty()
        && crate::gpu_frame::takes_the_frame(p.gpu, &p.doc, &p.reg);
    // ⭐⭐⭐ **E O DISPOSITIVO TAMBÉM PINTA** (`docs/Render3d/05` §39): com as quatro leis
    // do pintor no dispositivo — o material, o céu, o olhar e o **dono** — o G-buffer
    // deixa de atravessar o barramento e o que volta é a IMAGEM.
    //
    // ⚠️ **Só quando o refinamento de CPU está desligado**, que é o caminho de omissão: com
    // ele ligado (a porta de bissecção da [`crate::preview`]) o quadro precisa do G-buffer
    // para p.refinar sobre ele, e essa é a única razão para o trazer de volta.
    // ⭐⭐⭐⭐ **O MATCAP, PINTADO NO DISPOSITIVO** — o caminho de omissão do modelador.
    //
    // ⚠️ **Sem `!p.refinar` aqui, e é MEDIDO e não esquecimento:** o refinamento é a oclusão a
    // assentar passagem a passagem (`ph2d_field_render::refine_hemisphere`), e ele vive **dentro**
    // do braço do `Render` — um matcap não lê oclusão nenhuma. *A cerca do material é sobre o
    // G-buffer que aquele refinamento precisa de ter na mão; aqui não há o que refinar.*
    //
    // ⚠️ **E ele NÃO precisa de lâmpada**: ver o `exige_luz` do [`crate::gpu_frame::pedido`].
    if matches!(p.shading, crate::shading::Shading::Matcap)
        && crate::gpu_frame::takes_the_frame(p.gpu, &p.doc, &p.reg)
        && let Some(t) = p.gpu.as_ref()
        && let Some(pintura) = crate::gpu_frame::pinta_matcap(
            t,
            &p.doc,
            &p.reg,
            &p.cam,
            &ph2d_field_gpu::matcap::MatcapSetup {
                rgb_linear: &p.matcap.rgb,
                side: p.matcap.side,
                chave: p.matcap.chave,
                stops: p.look.exposure_stops,
                view: ph2d_view_transform::wgsl::view_code(p.look.view),
                background: BACKGROUND,
            },
            p.tw,
            p.th,
        )
    {
        // ⚠️ **Os acertos contam-se na IMAGEM**, como no ramo do pintor de material: o G-buffer
        // ficou no dispositivo, e o alfa do fundo é o discriminador que já vive ali.
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
                &apresentacao,
                BACKGROUND,
                p.ground,
                p.tw,
                p.th,
                p.assente,
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
            crate::gpu_frame::march(
                t,
                &p.doc,
                &p.reg,
                &p.cam,
                &mundos,
                p.ground,
                p.tw,
                p.th,
                // ⭐⭐⭐ **EM TODO QUADRO** (`W7c`) — a bandeira do [`crate::gpu_frame::march`]
                // sempre significou só a segunda passagem, e desde 19/09 ela não é uma decisão do
                // quadro. A lei e a medição vivem na porta, e não neste `true`.
                crate::preview::re_amostra_a_silhueta(),
            )
        })
    } else {
        None
    };
    // Abandonado a meio: não se manda nada, e quem esperava já mudou de pedido.
    // ⚠️ O G-buffer **move-se**: ele tem milhões de pixels e não é `Clone` de propósito.
    let (mut g, sombras_do_gpu) = match do_gpu {
        Some((g, sh)) => (g, Some(sh)),
        None => {
            let Some(g) = ph2d_field_render::trace_cancellable(
                &p.doc,
                &p.reg,
                &p.cam,
                p.tw,
                p.th,
                &p.flag,
                // ⭐⭐⭐ **E o motor de REFERÊNCIA re-amostra pela mesma razão** — duas metades de
                // uma lei, uma em cada motor, é a forma como ela morre num deles. ⚠️ Aqui ela custa
                // o que a tabela do [`ph2d_field_render::trace_cancellable`] diz (`1,30×`–`1,40×`),
                // e é esse o preço de correr no caminho de recuo em vez de na placa.
                crate::preview::re_amostra_a_silhueta(),
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
            // ⚠️ **O `p.assente` É a bandeira** (`= !coarse`) — lido aqui, e não uma segunda
            // pergunta ao mesmo facto.
            // ⭐ **A sombra e a oclusão vêm do dispositivo quando ele traçou** — elas
            // saíram da MESMA marcha, e refazê-las aqui seria pagar duas vezes pela mesma
            // resposta.
            let mut sombras = match sombras_do_gpu {
                Some(sh) => Some(sh),
                None => p.assente.then(|| {
                    ph2d_field_render::shadow_pass_on(&p.doc, &p.reg, &p.cam, &g, &mundos, p.ground)
                }),
            };
            // ⭐⭐⭐ **A COR QUE A PEÇA DEVOLVE AO CHÃO** (`docs/Render3d/09`).
            //
            // O chão desenhava o que a peça TIRA (a sombra, o contacto) e não o que ela PÕE: um
            // vaso vermelho pousava numa sombra cinzenta. O campo é 2D — *o chão é um plano, logo
            // a resposta dele é função de `(x, z)` e não da câmera* — e por isso ele é barato:
            // `32² × 128 = 131 072` raios, **`1,6 %`** dos que a grelha de sondas da peça já paga.
            //
            // ⚠️ **Ele viaja na MESMA bandeira que a sombra** (`p.assente`, a lei «grosso a mexer,
            // nítido ao assentar» da W73): o quadro de MOVIMENTO
            // fica **byte-idêntico** ao de hoje, porque sem campo a consulta devolve `[0,0,0]` e o
            // pintor soma zero.
            if let (Some(sh), Some(chao)) = (sombras.as_mut(), p.ground) {
                sh.set_ground_bounce(ph2d_field_render::ground_bounce::bake_ground_bounce(
                    &p.doc,
                    &p.reg,
                    &p.cam,
                    chao,
                    &surfaces,
                    &p.lights,
                    ph2d_field_render::ground_bounce::GROUND_BOUNCE_GRID,
                    ph2d_field_render::ground_bounce::GROUND_BOUNCE_DIRS,
                    (p.tw.min(p.th)) as usize,
                ));
            }
            // ⭐⭐⭐ **A CURVATURA, quando alguém a lê** (`docs/Render3d/10` e a `W8`) — a grandeza
            // que a subsuperfície MACIÇA e a TINTA POR CURVATURA perguntam. ⚠️ Com a omissão
            // (`subsurface_weight = 0`, tintas brancas) as duas portas respondem `false` e o quadro
            // **não paga uma amostra de campo** — o gémeo exacto do `if` que o WGSL faz.
            //
            // ⚠️⚠️ **São DUAS portas somadas, e não uma**: se o censo perguntasse só ao material, o
            // artista mexeria na tinta de aresta e a peça não mudava um pixel — *a grandeza que o
            // botão escolhe nunca teria sido medida*, que é a forma de knob morto que esta casa mede
            // desde 30/08.
            //
            // ⚠️ **O passo é o da SEGUNDA diferença e sai da PEÇA**, nunca da vista: ver
            // [`ph2d_field_render::curvatura::eps_para`], onde a medição está.
            //
            // ⭐⭐⭐ **E SÃO DUAS ASSADURAS, porque são DUAS PERGUNTAS** (auditoria de 2026-09-19,
            // `docs/Render3d/11` §10). O material pede o **óptimo de PRECISÃO** (o vale do erro da
            // segunda diferença); o estilo pede uma **ESCALA ARTÍSTICA**, que é a única alavanca que
            // existe sobre a dureza da borda — o campo de curvatura é constante por troço, logo
            // nenhum multiplicador aplicado a ele pode produzir um gradiente.
            //
            // ⚠️ **Cada uma só corre se o consumidor DELA estiver vivo**, e é isso que faz o preço
            // ser zero no caminho de omissão e na cena de quem tinge sem jade: a segunda assadura
            // custa `5` avaliações de campo por pixel acertado, e só quando os dois lêem.
            let material_le = ph2d_field_render::curvatura::material_le(&surfaces);
            // ⚠️ **A bola é a MESMA que a apresentação já derivou** — ver o `piece_radius` lá em
            // cima. ⛔ Derivá-la outra vez aqui seria a segunda resposta à mesma pergunta.
            //
            // ⭐⭐⭐ **E a decisão inteira vive na PORTA** ([`ph2d_field_render::curvatura::assar_canais`]):
            // quem lê o quê, com que passo, e se vale a pena pagar. Escrita em linha aqui, ela
            // divergiu do arnês dos gates no dia em que nasceu — e um arnês que monta o estado à mão
            // mede outro programa.
            // ⚠️ **A UNIÃO é uma PORTA e não uma expressão** (auditoria de 2026-09-23): ela tem
            // dois leitores — este e o `gpu_frame`, que decide com ela se a fita da peça entra no
            // shader do pintor —, e escrita duas vezes ela divergia no dia do terceiro consumidor.
            if ph2d_field_render::curvatura::alguem_le(&surfaces, &apresentacao) {
                let mut eval = ph2d_field_eval::hybrid::Hybrid::new(&p.doc, &p.reg);
                ph2d_field_render::curvatura::assar_canais(
                    &mut eval,
                    &mut g,
                    material_le,
                    &apresentacao,
                );
            }
            // ⭐⭐⭐ **A SOMBRA COM A BORDA MOLE, que só um material TRANSLÚCIDO lê**
            // (`docs/Render3d/10` §12, o report de 2026-09-18).
            //
            // A luz que uma peça de jade devolve não entrou por ESTE ponto: entrou à volta dele e
            // espalhou-se por baixo da superfície ⇒ a borda de uma sombra nela é MOLE, e a do
            // especular ao lado continua dura. O comprimento da média é a distância de
            // espalhamento do material — o `subsurface_radius` por canal, em unidades do MUNDO —,
            // e é por ser **por canal** que a borda fica avermelhada.
            //
            // ⚠️ **Sem material translúcido não se assa nada** e o [`Shadows::soft_at`] devolve a
            // visibilidade DURA ⇒ o quadro é byte a byte o de sempre. É a mesma cerca da curvatura,
            // logo acima.
            //
            // ⭐⭐⭐ **O RAIO É DE CADA MATERIAL desde 19/09** (ordem do dono) — ver
            // [`ph2d_field_render::sss_shadow::blur_por_material`]. Até 18/09 era o **MÁXIMO da
            // cena**, e uma chapa gorda escolhia o espalhamento de uma esfera magra (`18×` medido).
            // ⚠️ Com **um** valor distinto o resultado é byte-idêntico, e é o caso de toda cena de
            // hoje: *paga-se a correcção exactamente quando se usa a capacidade.*
            if let Some(sh) = sombras.as_mut() {
                let canais: Vec<Vec<[f32; 3]>> = (0..p.lights.len())
                    .map(|l| {
                        ph2d_field_render::sss_shadow::blur_por_material(
                            &g,
                            sh.lamp_channel(l),
                            &surfaces,
                            &p.cam,
                            p.th,
                        )
                    })
                    .collect();
                sh.set_soft(canais);
            }
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
                    &apresentacao,
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
            // ⚠️ A LEI vive na porta ([`ph2d_field_render::refine_hemisphere`]); aqui
            // fica só o que é da thread: pintar, mandar, e dizer se vale a pena continuar.
            //
            // ⭐⭐⭐ **E desde a `W5` cada passagem avança as DUAS metades do hemisfério** — o céu,
            // que ATENUA, e o ricochete da cena, que SOMA (`docs/Render3d/08`). Elas andam com o
            // mesmo `k` porque saem do mesmo conjunto de direcções: *uma publicação com `k`
            // direcções de céu e outras tantas DIFERENTES de ricochete somaria dois hemisférios.*
            //
            // ⏱️ Medido (`--release`, CPU a `95 %` ociosa): uma passagem no caso do modelador custa
            // `4,88 ms` de céu mais `19,06` de ricochete a `1920×1080` — o refinamento inteiro
            // passa de `~0,23 s` para `~1,15 s`, publicando `48` vezes pelo caminho e **cancelável
            // em cada uma**. O quadro de MOVIMENTO fica byte-idêntico: ele não passa por aqui.
            let mut sh = sombras.take().unwrap_or_default();
            ph2d_field_render::refine_hemisphere(
                &p.doc,
                &p.reg,
                &p.cam,
                &g,
                &surfaces,
                &p.lights,
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
