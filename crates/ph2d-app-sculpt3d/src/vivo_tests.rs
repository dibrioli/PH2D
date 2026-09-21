//! ⭐⭐⭐ **OS GATES DO CATAVENTO** — a promessa da rota B, afirmada no PIXEL do sprite.
//!
//! O `02.2` promete *«rodar um sprite e ver a luz acompanhar — o efeito que nenhum sprite
//! normal-mapeado comum consegue»*, e a §5.0 mediu que a frase é verdadeira **fora do plano do
//! ecrã** e falsa dentro dele. Estes gates afirmam a metade verdadeira, na saída que o artista vê.
//!
//! ```text
//! cargo test -p ph2d-app-sculpt3d --release catavento -- --ignored --nocapture --test-threads=1
//! ```

use ph2d_form_donation::baked_form::PassesDaLuz;
use ph2d_gpu::GpuContext;
use ph2d_light::LightRig;
use ph2d_render::{SpriteRenderer, TextureAtlas};

use crate::donation::PoseDaForma;
use crate::vivo::{AlvoVivo, FormaViva, ObjectoVivo, acende_um_quadro};

const LADO: u32 = 256;

fn placa() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static PARTILHADA: OnceLock<Option<GpuContext>> = OnceLock::new();
    PARTILHADA
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// O albedo: uma cor CHAPADA de propósito.
///
/// ⚠️ **Sem isso a régua media a arte e não a luz.** Um `base` com desenho faria dois quadros
/// diferirem pelos pixels do artista; chapado, **toda** a variação que sobra é a forma a acender.
fn base_chapado() -> Vec<u8> {
    let n = (LADO * LADO) as usize;
    let mut v = vec![0u8; n * 4];
    for p in v.as_chunks_mut::<4>().0 {
        *p = [200, 120, 90, 255];
    }
    v
}

/// Os pixels do sprite depois de um quadro da rota B, na pose pedida.
fn um_quadro(gpu: &GpuContext, malha: ph2d_mesh::Mesh, pose: PoseDaForma) -> Vec<u8> {
    let base = base_chapado();
    let atlas = TextureAtlas::new(gpu, LADO.max(256));
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let slot = renderer
        .acquire_individual(LADO, LADO, &base)
        .expect("slot do sprite");
    let mut cena = crate::Sculpt3dScene::new(&gpu.device, malha, 1.0);
    let mut passes = PassesDaLuz::default();
    let mut viva = FormaViva::garante(None, gpu, (LADO, LADO));
    acende_um_quadro(
        &mut cena,
        gpu,
        &mut renderer,
        &mut passes,
        &LightRig::default(),
        ObjectoVivo {
            alvo: AlvoVivo {
                size: (LADO, LADO),
                base: &base,
                texture_id: slot,
            },
            viva: &mut viva,
            pose,
        },
    )
    .expect("a rota B acende");
    let (_, _, px) = renderer.readback_individual(slot).expect("le de volta");
    px
}

/// Quantos componentes diferem por mais de um par de códigos, e o pior deles.
fn diferenca(a: &[u8], b: &[u8]) -> (usize, u8) {
    let mut fora = 0usize;
    let mut pior = 0u8;
    for (x, y) in a.iter().zip(b) {
        let d = x.abs_diff(*y);
        if d > 2 {
            fora += 1;
        }
        pior = pior.max(d);
    }
    (fora, pior)
}

/// ⭐⭐⭐ **VIRAR A FORMA FORA DO PLANO MUDA A LUZ — e numa esfera lisa não muda nada.**
///
/// ⛔⛔ **O CONTROLO é o que dá direito à leitura, e ele é a mesma fixtura do Bloco E:** uma esfera
/// lisa **não tem orientação** — o campo de normais dela visto de uma câmera não depende de como
/// ela está rodada —, logo virá-la tem de deixar o sprite **igual**. *Sem essa metade, um gate que
/// medisse «mudou» passaria com uma rota B que re-rasterizasse ruído.*
#[test]
#[ignore = "precisa de adapter"]
fn catavento_virar_a_forma_fora_do_plano_muda_a_luz() {
    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste (skip gracioso NÃO é verde)");
        return;
    };
    let reta = core::f32::consts::FRAC_PI_2;
    let com_relevo = || ph2d_mesh::shapes::uv_sphere_noisy(48, 64, 1.0, 0.18);
    let lisa = || ph2d_mesh::shapes::uv_sphere(48, 64, 1.0);

    let repouso = um_quadro(&gpu, com_relevo(), PoseDaForma::default());
    let virado = um_quadro(
        &gpu,
        com_relevo(),
        PoseDaForma {
            yaw: reta,
            pitch: 0.0,
        },
    );
    let (fora, pior) = diferenca(&repouso, &virado);
    let total = repouso.len();
    eprintln!("com relevo, virado 90°: {fora} de {total} componentes fora, pior {pior}");

    let c_repouso = um_quadro(&gpu, lisa(), PoseDaForma::default());
    let c_virado = um_quadro(
        &gpu,
        lisa(),
        PoseDaForma {
            yaw: reta,
            pitch: 0.0,
        },
    );
    let (c_fora, c_pior) = diferenca(&c_repouso, &c_virado);
    eprintln!("esfera lisa (CONTROLO): {c_fora} de {total} componentes fora, pior {c_pior}");

    assert!(
        c_fora * 200 < total,
        "o CONTROLO mexeu-se: uma esfera lisa não tem orientação, logo virá-la tem de deixar o \
         sprite igual — {c_fora} de {total} componentes fora, pior {c_pior}"
    );
    assert!(
        fora * 20 > total,
        "virar a forma fora do plano não mudou a luz: só {fora} de {total} componentes fora \
         (pior {pior}) — a rota B não está a fazer o que existe para fazer"
    );
}

/// ⭐⭐ **O CARIMBO POUPA A RASTERIZAÇÃO — e volta a pagá-la quando a pose muda.**
///
/// ⚠️ **A economia é INVISÍVEL a toda régua de valor:** com e sem carimbo a imagem é a mesma, e o
/// que muda é a CONTA. *Uma poupança que nenhum número mede é uma poupança que ninguém defende* —
/// daí o contador viver na [`FormaViva`] e não numa variável do teste.
///
/// ⛔ E a segunda metade é obrigatória: um carimbo que dissesse SEMPRE «nada mudou» passaria a
/// primeira, e entregaria um catavento que não gira.
#[test]
#[ignore = "precisa de adapter"]
fn catavento_o_carimbo_poupa_a_rasterizacao() {
    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste (skip gracioso NÃO é verde)");
        return;
    };
    let base = base_chapado();
    let atlas = TextureAtlas::new(&gpu, LADO.max(256));
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let slot = renderer
        .acquire_individual(LADO, LADO, &base)
        .expect("slot do sprite");
    let mut cena =
        crate::Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(32, 48, 1.0), 1.0);
    let mut passes = PassesDaLuz::default();
    let mut viva = FormaViva::garante(None, &gpu, (LADO, LADO));
    let rig = LightRig::default();
    let mut quadro = |viva: &mut FormaViva, pose: PoseDaForma| {
        acende_um_quadro(
            &mut cena,
            &gpu,
            &mut renderer,
            &mut passes,
            &rig,
            ObjectoVivo {
                alvo: AlvoVivo {
                    size: (LADO, LADO),
                    base: &base,
                    texture_id: slot,
                },
                viva,
                pose,
            },
        )
        .expect("a rota B acende");
    };

    quadro(&mut viva, PoseDaForma::default());
    assert_eq!(viva.rasterizacoes, 1, "o 1.º quadro tem de rasterizar");
    quadro(&mut viva, PoseDaForma::default());
    assert_eq!(
        viva.rasterizacoes, 1,
        "a pose não mudou e a forma foi re-rasterizada: o carimbo não está a ser lido"
    );
    quadro(
        &mut viva,
        PoseDaForma {
            yaw: 0.4,
            pitch: 0.0,
        },
    );
    assert_eq!(
        viva.rasterizacoes, 2,
        "a pose MUDOU e a forma não foi re-rasterizada: o carimbo está a dizer sempre «nada \
         mudou», e um catavento assim não gira"
    );
}
