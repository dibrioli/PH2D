//! **A DOAÇÃO** — o que a forma dá para a TINTA, e o interruptor que a governa.
//!
//! Módulo FILHO de [`super`] (`#[path]`), e a razão é o corte: o pai responde *o que o escultor
//! FAZ* (navegar, carimbar, desfazer, desenhar o barro); este responde *o que a forma DOA* — o
//! carimbo que decide se ela precisa ser re-rasterizada, a rasterização em si, e as três posições do
//! interruptor. As duas metades compartilham a `Sculpt3dScene`, então o filho alcança os campos
//! privados dela sem que nada precise virar `pub(crate)` para caber num cap de LOC.
//!
//! ⚠️ **Nada aqui sabe o que é uma camada do Painter.** O que sai é `Vec<f32>` — quatro floats por
//! texel — deixado num canal ([`ph2d_form_donation::donated_form`]) que o `painter_bridge` consome. É essa
//! ignorância mútua que mantém a promessa do `docs/3D/02.3`: apagar o módulo 3D apaga este arquivo,
//! e o canal fica existindo, silencioso, exatamente como está hoje sem a feature.

use std::sync::Arc;

use ph2d_core::Vec2;

use super::Sculpt3dScene;

/// A tela da cena `=2`, em pixels de lado.
///
/// ⚠️ **1024, e o número é MEDIDO** (`measure_a_donation`, RTX, release) — a forma é rasterizada NO
/// tamanho do canvas e volta pela CPU, então a tela é o que decide o custo de cada
/// re-rasterização:
///
/// | canvas | uma doação | lidos |
/// |---|---|---|
/// | 512² | 1,54 ms | 4 MB |
/// | **1024²** | **5,94 ms** | 16 MB |
/// | 2048² | 27,72 ms | 64 MB |
/// | 4096² | 123,49 ms | 256 MB |
///
/// ⚠️ **A primeira versão desta nota dizia *"a 1024² são ~4 MB, que o artista não sente"* — e
/// errava DUAS vezes:** o plano é `[f32; 4]` = **16 B/texel**, não 4, e 5,94 ms é quase um terço de
/// um quadro de 60 fps. É o §0 ao pé da letra: o número que fica escrito é o que a medição deu.
///
/// **Isto é o que torna o CARIMBO o desenho inteiro**, e não uma otimização: sem ele toda a tabela
/// acima seria paga POR FRAME. Com ele, uma forma parada custa zero e o artista paga uma vez, ao
/// apertar `D`. Girar a câmera em modo LUZ ainda pagaria por frame — mas girar é o que se faz no
/// BARRO, onde não há doação.
const CANVAS_EDGE: u32 = 1024;

/// A tela branca da cena da doação (`PH2D_SCULPT3D_SMOKE=2`) — o mesmo gesto dos
/// smokes do impasto e do Wet Paint: nascer sobre uma superfície pronta em vez
/// de montar uma.
///
/// Devolve os bits da entidade para o chamador assentar a seleção nela.
pub struct TelaPedida {
    /// O lado, em pixels.
    pub edge: u32,
    /// Branco opaco: a doação MULTIPLICA a tinta (o modelo é RELATIVO), então sobre branco a
    /// luz da forma é o que se vê, sem cor competindo.
    pub bg: u8,
    /// O centro do mundo onde ela pousa.
    pub center: Vec2,
}

/// **A tela branca que este smoke quer** — a DECISÃO e os três números, com a razão de cada.
///
/// ⚠️⚠️ **A decisão fica aqui e a CAPACIDADE fica na shell, e o corte é o da regra 2** (W2/L3-B):
/// quem sabe *se* uma tela é precisa e *como* ela tem de ser é a família; quem sabe *fazer* uma é
/// o `image_import`, uma folha da shell com **41 consumidores** de famílias diferentes. Pedi-la
/// por closure seria arrastar seis parâmetros que nenhuma linha desta crate lê.
///
/// Duas cenas querem a mesma tela, e a pergunta é feita UMA vez — ver `scenes::wants_canvas`.
pub fn canvas_wanted() -> Option<TelaPedida> {
    super::wants_canvas().then(|| TelaPedida {
        edge: CANVAS_EDGE,
        bg: 2,
        center: Vec2::new(0.0, 0.0),
    })
}

/// ⭐⭐ **A TELA DESTA CENA É UM CATAVENTO?** — o componente que a `=52` quer na tela que acabou de
/// nascer, ou `None` em toda outra cena.
///
/// ⚠️ **A decisão é da FAMÍLIA e o gesto é da shell**, que é a mesma regra 2 do
/// [`canvas_wanted`] logo acima: quem sabe o que a cena quer é quem a escreveu; quem sabe pôr um
/// componente numa entidade é quem tem o mundo na mão. *Um `if cena == 52` escrito na shell seria
/// a lista que apodrece no dia da cena seguinte.*
///
/// ⛔ **O `piece` é `0` de propósito:** hoje a rota B rasteriza a peça ACTIVA da cena 3D, e o campo
/// existe para o dia em que houver mais de uma — declará-lo aqui com um índice inventado seria
/// prometer uma escolha que o passe ainda não faz.
#[must_use]
pub fn catavento_pedido() -> Option<ph2d_ecs::Mesh3D> {
    super::scenes::catavento_scene().then(catavento_da_cena)
}

/// **O catavento que a `=52` pede** — a LEI, separada da leitura da env.
///
/// ⚠️ **Ela é uma função própria porque esta crate proíbe `unsafe`**, e sem isso um gate não pode
/// armar a variável de ambiente para medir o que a cena pede: ele mediria o `None` e ficaria verde
/// a afirmar nada. ⭐ A metade *«e é a `=52` e não outra cena»* é medida pelo censo do roteador,
/// que varre os predicados desta crate à procura da forma `== Some("N")`.
#[must_use]
pub fn catavento_da_cena() -> ph2d_ecs::Mesh3D {
    ph2d_ecs::Mesh3D {
        piece: 0,
        yaw: 0.0,
        pitch: 0.0,
        spin: super::scenes::GIRO_DA_CENA,
    }
}

/// **A tela NASCEU (ou não)** — o log que ela merece, e os bits que o chamador assenta na selecção.
pub fn canvas_born(feito: Result<(String, u64), String>) -> Option<u64> {
    match feito {
        Ok((label, bits)) => {
            // ⚠️ **A `=52` entra AQUI e não no ramo de baixo**, e a razão foi medida numa foto
            // (21/09): a tela dela nascia a dizer *«esculpa, aperte D até ler LUZ, pegue o Painter
            // e pinte»* — o texto da DOAÇÃO — enquanto o roteiro dela manda `Shift+B`. *Uma cena
            // que imprime dois caminhos diferentes ensina o errado a metade de quem a lê.*
            if super::bake_scene() || super::reopen_scene() || super::scenes::catavento_scene() {
                eprintln!(
                    "[sculpt3d] sprite '{label}' ({CANVAS_EDGE}x{CANVAS_EDGE}) na mesa — ele e' o \
                     OBJETO que a forma vai acender"
                );
            } else {
                eprintln!(
                    "[sculpt3d] tela '{label}' ({CANVAS_EDGE}x{CANVAS_EDGE}) pronta — \
                     esculpa, aperte D ate ler LUZ, pegue o Painter e pinte"
                );
            }
            Some(bits)
        }
        Err(e) => {
            eprintln!("[sculpt3d] nao consegui abrir a tela: {e}");
            None
        }
    }
}

/// **O que a forma FAZ** — o interruptor da doação, e ele tem três posições porque são três
/// PERGUNTAS distintas que o artista precisa responder para julgar o módulo:
///
/// - [`Self::Clay`] — *como está a escultura?* O barro na tela. É esculpir.
/// - [`Self::Light`] — *como a tinta fica ACESA por ela?* O barro sai da tela e vira a luz da
///   tinta. É pintar sobre forma, e é a razão de o módulo existir.
/// - [`Self::Off`] — *como a tinta fica SEM ela?* O **controle** do A/B, sem o qual o artista vê
///   algo bonito e não sabe o que ganhou.
///
/// ⚠️ `Clay` e `Light` são exclusivos por CONSTRUÇÃO, não por política: a malha é desenhada por
/// cima do 2D (`LoadOp::Load`), então mostrar o barro esconde exatamente a tinta que a doação
/// existe para acender.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FormRole {
    Clay,
    Light,
    Off,
}

impl FormRole {
    /// A próxima posição. Um ciclo, e não três teclas: as três respondem à MESMA pergunta — *o que
    /// estou olhando?* — e um botão por resposta convidaria a combinações que não existem.
    pub(super) fn next(self) -> Self {
        match self {
            Self::Clay => Self::Light,
            Self::Light => Self::Off,
            Self::Off => Self::Clay,
        }
    }

    /// **O barro está na tela?** O fato puro que a cena delega — e que decide DUAS coisas: o passe
    /// de cor desenha, e o ponteiro é da cena. Uma pergunta, um dono.
    pub(super) fn draws_clay(self) -> bool {
        matches!(self, Self::Clay)
    }

    /// **A forma acende a tinta?** O outro fato, e ele NÃO é a negação do primeiro: `Off` não
    /// desenha barro *nem* doa, e é justamente essa terceira posição que dá o controle do A/B.
    pub(super) fn donates(self) -> bool {
        matches!(self, Self::Light)
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Clay => "BARRO (esculpir)",
            Self::Light => "LUZ (a forma acende a tinta)",
            Self::Off => "DESLIGADA (o controle do A/B)",
        }
    }
}

/// **O carimbo da forma** — o que decide se a doação precisa ser re-rasterizada.
///
/// Três entradas, e cada uma é uma maneira DISTINTA de a forma na tela mudar: a **malha** (um
/// traço), a **câmera** (um giro) e o **tamanho do canvas**. Faltar qualquer uma deixa a tinta acesa
/// por uma escultura que não está mais ali — e uma luz velha não se vê que é velha.
///
/// ⚠️ **A câmera entra por BITS, nunca por valor.** Um carimbo responde *"mudou?"*, e comparar
/// `f32` por valor faz um estado degenerado (`NaN`) nunca comparar igual a si mesmo — a doação seria
/// re-rasterizada todo frame, para sempre, sem nada na tela dizendo por quê. Bits são a identidade
/// honesta.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct FormStamp {
    edits: u64,
    camera: [u32; 8],
    size: (u32, u32),
    /// ⭐ **O ENQUADRAMENTO, por BITS** — sem ele um re-bake com o sprite noutro sítio do ecrã
    /// deixava a forma viva a mostrar o recorte de antes, porque as outras três entradas não se
    /// mexem num gesto de assar.
    recorte: [u32; 5],
}

/// ⭐⭐⭐ **A POSE 3D de uma forma viva — a orientação que o `Transform` 2D NÃO sabe exprimir.**
///
/// ⛔⛔ **Ela existe por medição, não por gosto.** O [`ph2d_ecs::Transform`] tem `rotation: f32` e
/// exprime **apenas** a rotação no plano do ecrã — e a §5.0 do catavento mediu que essa a rota A já
/// dá **exactamente** (`0,00°` de desacordo de normais contra `28,58°` de um plano fixo). *A
/// rotação que justifica a rota B é a que FALTA àquele campo*, logo ela tem de viajar no componente
/// da malha. Tabelas: `docs/Render3d/17_a_rota_b_o_catavento.md` §1.5.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PoseDaForma {
    /// A volta em torno do eixo vertical do ecrã — a pá do catavento a virar.
    pub yaw: f32,
    /// A inclinação para cima e para baixo.
    pub pitch: f32,
}

/// O carimbo, como FUNÇÃO PURA das três entradas.
///
/// ⚠️ Separado do método por causa do GATE: uma `Sculpt3dScene` exige um `wgpu::Device` para
/// existir, então um teste do carimbo preso ao método só rodaria com adapter — e o que ele
/// verifica (*as três entradas movem o carimbo, e nada mais o move*) não tem nada a ver com a GPU.
fn stamp_of(
    edits: u64,
    camera: &ph2d_mesh_render::Camera3d,
    size: (u32, u32),
    framing: ph2d_mesh_render::Framing,
) -> FormStamp {
    FormStamp {
        edits,
        camera: [
            camera.target.x.to_bits(),
            camera.target.y.to_bits(),
            camera.target.z.to_bits(),
            camera.distance.to_bits(),
            camera.yaw.to_bits(),
            camera.pitch.to_bits(),
            camera.fov_y.to_bits(),
            // ⚠️ **A LENTE entra aqui** (2026-09-21): trocar de convergente para paralela muda a
            // imagem inteira e **nenhuma** das outras sete entradas se mexe — a doação ficaria a
            // descrever a peça vista pela lente de antes, em silêncio.
            u32::try_from(camera.lens.index()).unwrap_or(0),
        ],
        size,
        recorte: [
            framing.aspect.to_bits(),
            framing.region.origin[0].to_bits(),
            framing.region.origin[1].to_bits(),
            framing.region.size[0].to_bits(),
            framing.region.size[1].to_bits(),
        ],
    }
}

impl Sculpt3dScene {
    /// **A MALHA MUDOU** — a porta única do shell para esse fato.
    ///
    /// ⚠️ Duas coisas dependem dele e elas TÊM de andar juntas: a GPU precisa reenviar os vértices,
    /// e a **DOAÇÃO** precisa ser re-rasterizada. Escrever uma sem a outra deixa a forma que ilumina
    /// a tinta descrevendo a escultura de antes do traço, em silêncio. Enumerar os sítios apodrece;
    /// uma porta, não.
    ///
    /// ⚠️ Toma os campos em vez de `&mut self` **por causa do chamador**: a lista de vértices vem de
    /// `self.stroke`, e um `&mut self` a obrigaria a ser copiada primeiro — uma alocação por dab, no
    /// laço mais quente do módulo, para satisfazer o borrow checker e não o produto.
    pub(super) fn mesh_changed(dirty: &mut Vec<u32>, edits: &mut u64, moved: &[u32]) {
        dirty.extend_from_slice(moved);
        *edits = edits.wrapping_add(1);
    }

    /// A malha foi RECONSTRUÍDA (o undo) — o upload incremental não serve, e a doação envelheceu
    /// igual. Irmã do [`Self::mesh_changed`].
    pub(super) fn mesh_rebuilt(&mut self) {
        if let Some(o) = self.obj_mut() {
            o.dirty.clear();
            o.uploaded = false;
        }
        self.edits = self.edits.wrapping_add(1);
    }

    /// O carimbo de HOJE, para o canvas pedido.
    fn form_stamp(&self, size: (u32, u32)) -> FormStamp {
        stamp_of(
            self.edits,
            &self.camera,
            size,
            // A doação escreve o canvas INTEIRO do Painter — ela não tem recorte nenhum.
            ph2d_mesh_render::Framing::whole(size),
        )
    }

    /// **A DOAÇÃO** — rasteriza a forma no tamanho do canvas do Painter e devolve o plano. `None`
    /// quando **nada mudou**, que é o caso comum.
    ///
    /// ⚠️ **A câmera é a do ESCULTOR, com o aspecto do CANVAS.** Não há enquadramento novo a
    /// inventar: a pose em que o artista deixou o modelo É a pose sobre a qual ele quer pintar. A
    /// consequência honesta é que um viewport 16:9 e um canvas 1:1 não mostram a mesma coisa — o FOV
    /// vertical é preservado e o horizontal segue o canvas, que é o que uma câmera em perspectiva
    /// faz.
    ///
    /// ⚠️ **Isto BLOQUEIA** (a leitura de volta espera o device). É por isso que o carimbo vem
    /// primeiro: num frame em que ninguém esculpiu nem girou, esta função não toca a GPU.
    fn rasterise_form(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
    ) -> Option<ph2d_form_donation::donated_form::DonatedPlanes> {
        let stamp = self.form_stamp(size);
        if self.donated == Some(stamp) {
            return None;
        }
        // ⚠️ A malha tem de estar no device ANTES de rasterizar — e o upload vive no laço de
        // desenho, que num frame em modo LUZ nem roda. Perguntar aqui custa um `if` e é o que
        // impede a doação de descrever a malha de antes do traço.
        self.sync_mesh(device, queue);
        let planes = self.renderer.form_plane(
            device,
            queue,
            &self.camera,
            size,
            self.shade(),
            self.donation_ssao(),
        )?;
        self.donated = Some(stamp);
        Some(ph2d_form_donation::donated_form::DonatedPlanes {
            normal: Arc::new(planes.normal),
            occlusion: Arc::new(planes.occlusion),
        })
    }

    /// **A forma, INCONDICIONALMENTE** — o G-buffer no tamanho pedido, sem carimbo.
    ///
    /// ⚠️ Mora aqui, ao lado do [`Self::rasterise_form`], porque as duas rotas têm de passar pela
    /// MESMA `form_plane`: a doação (que serve a tela do Painter) e o bake (que serve um sprite da
    /// cena) descrevem a mesma escultura, e uma segunda chamada com outra câmera ou outro `sync`
    /// daria dois G-buffers da mesma malha que discordam.
    ///
    /// E ela **não** carimba: o bake é um gesto explícito, então *"nada mudou"* não é uma resposta
    /// que ele aceite — o artista apertou a tecla, e o que ele espera é a forma de agora.
    ///
    /// ⚠️ **E ela recebe o ENQUADRAMENTO** desde 2026-09-21 (report do dono: *«o Bake não é feito
    /// projetando o objeto 3d exatamente como o posiciono sobre a sprite»*) — ver
    /// [`Self::enquadramento_do_sprite`], que é quem o deriva. Com
    /// [`ph2d_mesh_render::Framing::whole`] ela é, ao bit, a de antes desta wave.
    pub(super) fn form_plane_for(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        size: (u32, u32),
        framing: ph2d_mesh_render::Framing,
    ) -> Option<ph2d_mesh_render::FormPlanes> {
        self.sync_mesh(&gpu.device, &gpu.queue);
        self.renderer.form_plane_in(
            &gpu.device,
            &gpu.queue,
            &self.camera,
            size,
            // ⚠️ **Os MESMOS knobs do viewport** (`Self::shade`), e não um default: a oclusão que a
            // doação carrega é função deles, e o artista afinou a cavidade olhando o barro.
            self.shade(),
            self.donation_ssao(),
            framing,
        )
    }

    /// ⭐⭐⭐⭐ **O QUE SE VÊ É O QUE SE ASSA** — o enquadramento com que a forma é rasterizada
    /// dentro dos texels de um sprite.
    ///
    /// ⛔⛔ **O defeito que ela cura** (report do dono, 2026-09-21, com foto): a porta de assar
    /// escrevia o alvo INTEIRO, logo a peça ocupava, dentro do sprite, a mesma fracção que ocupava
    /// **da altura do VIEWPORT** — e o sprite é um rectângulo *dentro* dele. O assado saía
    /// deslocado e com a escala errada pela razão `altura da vista ÷ altura do sprite no ecrã`, que
    /// é exactamente *«um fundo deslocado do objeto 3d»*.
    ///
    /// `sub` é o rectângulo que a ARTE do sprite ocupa na janela (`[x, y, w, h]`, `y` do topo) — a
    /// shell é quem o sabe (ela tem a câmera 2D e o afim partilhado), e esta crate é quem sabe onde
    /// a vista 3D está.
    ///
    /// ⚠️⚠️ **`None` quer dizer *«não há vista publicada»*, e não *«o objecto está fora»*:** uma
    /// cena headless (todo gate de GPU desta casa) nunca publica canvas, e quem chama cai então na
    /// vista inteira — o comportamento de antes desta wave. *A alternativa, recusar o bake,
    /// trocaria um enquadramento aproximado por nenhum assado.*
    pub(super) fn enquadramento_do_sprite(
        &self,
        sub: Option<[f32; 4]>,
    ) -> Option<ph2d_mesh_render::Framing> {
        let sub = sub?;
        let vp = self.vp_screen(self.vp_active())?;
        let full = [vp.x as f32, vp.y as f32, vp.w as f32, vp.h as f32];
        let region = ph2d_mesh_render::ViewRegion::of_px(sub, full)?;
        Some(ph2d_mesh_render::Framing {
            // ⚠️ **O aspecto é o da VISTA e nunca o do sprite** — ver o doc do
            // [`ph2d_mesh_render::Framing`]: a forma do frustum é da vista, e o recorte só diz
            // que pedaço dela este alvo desenha.
            aspect: full[2] / full[3],
            region,
        })
    }

    /// ⭐⭐⭐ **A TERCEIRA saída da mesma rasterização: a forma VIVA, para vistas RESIDENTES.**
    ///
    /// A [`Self::rasterise_form`] devolve fatias da CPU com carimbo; a [`Self::form_plane_for`]
    /// devolve fatias sem carimbo. Esta **não devolve fatias nenhumas** — ela escreve directamente
    /// nas duas vistas que o chamador possui, e é isso que tira o readback do caminho: medido, ele
    /// vale **`31×`** a rasterização a `512²`, e é o preço inteiro da rota de hoje.
    ///
    /// ⭐⭐ **A POSE entra pela CÂMERA e não pela malha, e isso custa ZERO.** As normais do
    /// G-buffer são de **VISTA** (`canvas_normal(cam.view * model * n)`), logo orbitar a câmera em
    /// torno do alvo roda-as no referencial em que o rig vive — que é exactamente *«o objecto
    /// virou-se e a luz acompanhou»*. Rodar a MALHA daria a mesma imagem e pagaria um reenvio de
    /// vértices por quadro.
    ///
    /// ⚠️⚠️ **E o [`ph2d_mesh_render::Camera3d`] não tem ROLL — o que NÃO é um limite aqui.** O roll
    /// é a rotação **no plano do ecrã**, e a §5.0 mediu que essa a rota A já dá **exactamente**
    /// (`0,00°`): rodar a imagem e rodar as normais são duas operações 2D sobre o plano já assado.
    /// *A câmera não sabe exprimir precisamente aquilo de que esta rota não precisa.*
    ///
    /// Devolve `true` quando de facto rasterizou — `false` é o carimbo a dizer que nada mudou.
    pub(super) fn gbuffer_vivo(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        pose: PoseDaForma,
        size: (u32, u32),
        alvos: (&wgpu::TextureView, &wgpu::TextureView),
        carimbo: &mut Option<FormStamp>,
        // ⭐⭐⭐ **O enquadramento CONGELADO no bake** — ver
        // [`ph2d_form_donation::baked_form::Recorte`]. Sem ele a rota B rasterizaria a vista
        // inteira enquanto o assado por baixo dela tem um recorte, e a peça SALTAVA para encher o
        // sprite no primeiro quadro em que o relógio andasse.
        framing: ph2d_mesh_render::Framing,
    ) -> bool {
        let mut cam = self.camera;
        cam.aim(pose.yaw, pose.pitch);
        // ⚠️ O carimbo é do PAR (malha, câmera-do-objecto, tamanho, ENQUADRAMENTO) e não do da
        // cena: dois objectos vivos partilham a malha e têm poses — e recortes — diferentes, logo
        // um carimbo da cena diria «nada mudou» para o segundo depois de o primeiro ter
        // rasterizado.
        let agora = stamp_of(self.edits, &cam, size, framing);
        if *carimbo == Some(agora) {
            return false;
        }
        self.sync_mesh(&gpu.device, &gpu.queue);
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-app-sculpt3d forma viva"),
            });
        self.renderer.render_gbuffer_framed(
            &gpu.device,
            &gpu.queue,
            &mut enc,
            alvos.0,
            alvos.1,
            &cam,
            self.shade(),
            size,
            ph2d_mesh_render::ScreenRect::full(size),
            framing,
        );
        gpu.queue.submit(std::iter::once(enc.finish()));
        *carimbo = Some(agora);
        true
    }

    /// **A oclusão de tela que a DOAÇÃO mede** — os mesmos parâmetros do viewport, `None` quando o
    /// artista a desligou.
    ///
    /// ⚠️ **Ela é medida na rasterização da doação, nunca herdada do viewport**, e o motivo não é
    /// pureza: a doação tem outro tamanho (o do canvas) e outro aspecto. Reaproveitar a medição da
    /// tela desenharia a sombra de um enquadramento sobre a forma de outro — e o `render_gbuffer`
    /// CONSOME a frescura justamente para que isso não possa acontecer em silêncio.
    ///
    /// ⚠️ **E sem ela a wave entregaria nada no estado em que o artista está:** `DEFAULT_CAVITY` e
    /// `DEFAULT_AO_STRENGTH` são `0`, então a de TELA é a única oclusão acesa por padrão.
    fn donation_ssao(&self) -> Option<ph2d_mesh_render::SsaoParams> {
        (self.ssao > 0.0).then(|| self.ssao_params())
    }

    /// O interruptor avança uma posição. Devolve o rótulo do estado novo.
    pub(super) fn cycle_role(&mut self) -> &'static str {
        self.role = self.role.next();
        self.role.label()
    }

    /// O barro está na tela? Delega ao papel — ver [`FormRole::draws_clay`].
    pub(super) fn shows_clay(&self) -> bool {
        self.role.draws_clay()
    }
}

/// **A DOAÇÃO chega à TINTA** — rasteriza a forma no tamanho do canvas do Painter e deixa o
/// plano no canal que o `painter_bridge` consome.
///
/// Roda por frame e **quase sempre não faz nada**: sem cena armada sai no primeiro `if`, e com a
/// cena parada o carimbo responde *"nada mudou"* antes de a GPU ser tocada.
///
/// ⚠️ **Só esta função sabe que existe um módulo 3D.** O que ela escreve é `Vec<f32>`, e quem
/// instala (`painter_bridge`, o único sítio que pode fazer downcast para `PainterTool`) não
/// conhece malha nenhuma — as duas metades da promessa de removibilidade do `docs/3D/02.3` de
/// uma vez.
pub fn donate_form(
    scene: &mut Sculpt3dScene,
    canal: &mut ph2d_form_donation::donated_form::DonatedForm,
    gpu: &ph2d_gpu::GpuContext,
) {
    let Some(size) = canal.canvas else {
        // O Painter ainda não disse quão grande é o canvas — ou não há Painter. Sem tamanho não
        // há o que rasterizar, e INVENTAR um deixaria o plano do tamanho errado, que o tool
        // recusa em silêncio.
        return;
    };
    if !scene.role.donates() {
        // ⚠️ **Desligar APAGA, não emudece.** Deixar o plano instalado com o interruptor em off
        // manteria a tinta acesa pela forma e o artista concluiria que o botão está quebrado. E
        // esvaziar o carimbo faz disto uma notícia só: a próxima vez que a forma mudar — ou que
        // o interruptor volte — a doação é re-rasterizada.
        if scene.donated.take().is_some() {
            canal.news = Some(None);
        }
        return;
    }
    // ⚠️ **O `GpuContext` chega por PARÂMETRO, e é isso que dissolveu o empréstimo duplo**
    // (W2/L3-B). Ele e a cena viviam no mesmo `AppGfx`, e a versão anterior desta função
    // clonava os dois `Arc` só para separar os empréstimos; quem desmonta o `AppGfx` agora é
    // a shell, que é a dona dele, e aqui não sobra clone nenhum.
    if let Some(plane) = scene.rasterise_form(&gpu.device, &gpu.queue, size) {
        canal.news = Some(Some(plane));
    }
}

#[cfg(test)]
#[path = "donation_tests.rs"]
mod tests;
