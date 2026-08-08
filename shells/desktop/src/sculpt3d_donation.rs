//! **A DOAÇÃO** — o que a forma dá para a TINTA, e o interruptor que a governa.
//!
//! Módulo FILHO de [`super`] (`#[path]`), e a razão é o corte: o pai responde *o que o escultor
//! FAZ* (navegar, carimbar, desfazer, desenhar o barro); este responde *o que a forma DOA* — o
//! carimbo que decide se ela precisa ser re-rasterizada, a rasterização em si, e as três posições do
//! interruptor. As duas metades compartilham a `Sculpt3dScene`, então o filho alcança os campos
//! privados dela sem que nada precise virar `pub(crate)` para caber num cap de LOC.
//!
//! ⚠️ **Nada aqui sabe o que é uma camada do Painter.** O que sai é `Vec<f32>` — quatro floats por
//! texel — deixado num canal ([`crate::donated_form`]) que o `painter_bridge` consome. É essa
//! ignorância mútua que mantém a promessa do `docs/3D/02.3`: apagar o módulo 3D apaga este arquivo,
//! e o canal fica existindo, silencioso, exatamente como está hoje sem a feature.

use std::sync::Arc;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;

use super::Sculpt3dScene;
use crate::app_state::App;

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
pub(crate) fn spawn_canvas_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, AssetId>,
) -> Option<u64> {
    // Duas cenas querem a mesma tela branca, e a pergunta é feita UMA vez — ver
    // `scenes::wants_canvas`.
    if !super::wants_canvas() {
        return None;
    }
    match crate::image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        CANVAS_EDGE,
        // Branco opaco: a doação MULTIPLICA a tinta (o modelo é RELATIVO), então
        // sobre branco a luz da forma é o que se vê, sem cor competindo.
        2,
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok((label, bits)) => {
            if super::bake_scene() || super::reopen_scene() {
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
    camera: [u32; 7],
    size: (u32, u32),
}

/// O carimbo, como FUNÇÃO PURA das três entradas.
///
/// ⚠️ Separado do método por causa do GATE: uma `Sculpt3dScene` exige um `wgpu::Device` para
/// existir, então um teste do carimbo preso ao método só rodaria com adapter — e o que ele
/// verifica (*as três entradas movem o carimbo, e nada mais o move*) não tem nada a ver com a GPU.
fn stamp_of(edits: u64, camera: &ph2d_mesh_render::Camera3d, size: (u32, u32)) -> FormStamp {
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
        ],
        size,
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
        stamp_of(self.edits, &self.camera, size)
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
    ) -> Option<crate::donated_form::DonatedPlanes> {
        let stamp = self.form_stamp(size);
        if self.donated == Some(stamp) {
            return None;
        }
        // ⚠️ A malha tem de estar no device ANTES de rasterizar — e o upload vive no laço de
        // desenho, que num frame em modo LUZ nem roda. Perguntar aqui custa um `if` e é o que
        // impede a doação de descrever a malha de antes do traço.
        self.sync_mesh(device, queue);
        let planes = self
            .renderer
            .form_plane(device, queue, &self.camera, size, self.shade(), self.donation_ssao())?;
        self.donated = Some(stamp);
        Some(crate::donated_form::DonatedPlanes {
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
    pub(super) fn form_plane_for(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        size: (u32, u32),
    ) -> Option<ph2d_mesh_render::FormPlanes> {
        self.sync_mesh(&gpu.device, &gpu.queue);
        self.renderer.form_plane(
            &gpu.device,
            &gpu.queue,
            &self.camera,
            size,
            // ⚠️ **Os MESMOS knobs do viewport** (`Self::shade`), e não um default: a oclusão que a
            // doação carrega é função deles, e o artista afinou a cavidade olhando o barro.
            self.shade(),
            self.donation_ssao(),
        )
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

impl App {
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
    pub(crate) fn sculpt3d_donate_form(&mut self) {
        let Some(size) = self.donated_form.canvas else {
            // O Painter ainda não disse quão grande é o canvas — ou não há Painter. Sem tamanho não
            // há o que rasterizar, e INVENTAR um deixaria o plano do tamanho errado, que o tool
            // recusa em silêncio.
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(scene) = gfx.sculpt3d.as_mut() else {
            return;
        };
        if !scene.role.donates() {
            // ⚠️ **Desligar APAGA, não emudece.** Deixar o plano instalado com o interruptor em off
            // manteria a tinta acesa pela forma e o artista concluiria que o botão está quebrado. E
            // esvaziar o carimbo faz disto uma notícia só: a próxima vez que a forma mudar — ou que
            // o interruptor volte — a doação é re-rasterizada.
            if scene.donated.take().is_some() {
                self.donated_form.news = Some(None);
            }
            return;
        }
        // ⚠️ O `GpuContext` sai do `surface`, que vive no MESMO `gfx` que a cena — e a cena é
        // emprestada mutável. Clonar o `Arc` do device e da queue separa os dois empréstimos sem
        // copiar recurso nenhum.
        let (device, queue) = {
            let g = gfx.surface.gpu();
            (Arc::clone(&g.device), Arc::clone(&g.queue))
        };
        if let Some(plane) = scene.rasterise_form(&device, &queue, size) {
            self.donated_form.news = Some(Some(plane));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_mesh_render::Camera3d;

    /// **As TRÊS entradas movem o carimbo — e o gate existe porque esquecer uma é invisível.**
    ///
    /// Um carimbo que ignora a câmera deixa a tinta acesa pela forma vista de outro ângulo; um que
    /// ignora a malha, pela escultura de antes do traço; um que ignora o tamanho entrega um plano
    /// que o Painter recusa e a doação some sem dizer por quê. Nenhum dos três dá erro em lugar
    /// nenhum: a tela fica *plausível*.
    #[test]
    fn every_way_the_form_can_change_moves_the_stamp() {
        let base = Camera3d::default();
        let here = stamp_of(7, &base, (256, 128));
        assert_eq!(here, stamp_of(7, &base, (256, 128)), "premissa: é estável");

        assert_ne!(here, stamp_of(8, &base, (256, 128)), "a MALHA mudou");
        assert_ne!(here, stamp_of(7, &base, (512, 128)), "o CANVAS mudou");

        // Cada campo da câmera, um a um: um carimbo que só olha `yaw` passaria num teste que só
        // gira, e é justamente o `pan` (que move o `target`) o gesto mais fácil de esquecer.
        for (name, mutate) in [
            (
                "yaw",
                (|c: &mut Camera3d| c.yaw += 0.1) as fn(&mut Camera3d),
            ),
            ("pitch", |c| c.pitch += 0.1),
            ("distance", |c| c.distance *= 1.5),
            ("fov_y", |c| c.fov_y += 0.05),
            ("target.x", |c| c.target.x += 1.0),
            ("target.y", |c| c.target.y += 1.0),
            ("target.z", |c| c.target.z += 1.0),
        ] {
            let mut moved = base;
            mutate(&mut moved);
            assert_ne!(
                here,
                stamp_of(7, &moved, (256, 128)),
                "mexer em `{name}` tem de mover o carimbo"
            );
        }
    }

    /// **Uma câmera degenerada compara igual a si mesma.**
    ///
    /// ⚠️ É o gate do *"por BITS, nunca por valor"*: `NaN != NaN`, então um carimbo que comparasse
    /// `f32` por valor nunca diria "nada mudou" e a doação seria re-rasterizada **todo frame, para
    /// sempre** — uma leitura de volta bloqueante por quadro, sem nada na tela explicando por quê.
    #[test]
    fn a_degenerate_camera_still_compares_equal_to_itself() {
        let broken = Camera3d {
            yaw: f32::NAN,
            ..Camera3d::default()
        };
        let s = stamp_of(1, &broken, (64, 64));
        assert_eq!(s, stamp_of(1, &broken, (64, 64)));
    }

    /// **O interruptor CICLA, e cada posição é distinta.**
    ///
    /// Três voltas devolvem ao começo — sem isso o `D` viraria um caminho de mão única e o artista
    /// perderia o barro depois de doar uma vez.
    #[test]
    fn the_switch_cycles_through_all_three_and_comes_back() {
        let mut r = FormRole::Clay;
        let mut seen = Vec::new();
        for _ in 0..3 {
            seen.push(r.label());
            r = r.next();
        }
        assert_eq!(r, FormRole::Clay, "três toques voltam ao barro");
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 3, "as três posições têm rótulos distintos");
    }

    /// **Cada posição faz exatamente uma coisa, e `Off` não faz nenhuma.**
    ///
    /// ⚠️ A 1ª versão deste gate afirmava que os variants do enum são distintos entre si — o que o
    /// `derive(PartialEq)` garante. Ele **não podia falhar pelo motivo que alegava**, e teria
    /// ficado verde com `draws_clay` cravado em `true` (a malha desenhada por cima da tinta em
    /// todas as posições, a doação inalcançável). O oráculo tem de ser o COMPORTAMENTO das duas
    /// perguntas, não a identidade dos rótulos.
    #[test]
    fn each_position_answers_exactly_one_of_the_two_questions() {
        assert!(FormRole::Clay.draws_clay() && !FormRole::Clay.donates());
        assert!(FormRole::Light.donates() && !FormRole::Light.draws_clay());
        assert!(
            !FormRole::Off.draws_clay() && !FormRole::Off.donates(),
            "`Off` é o controle: nem barro na tela, nem forma na tinta"
        );
    }
}
