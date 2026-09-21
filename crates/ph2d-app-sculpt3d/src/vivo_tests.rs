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
            recorte: None,
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
                recorte: None,
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

/// ⭐⭐⭐ **A FASE ACENDE QUEM TEM O COMPONENTE, LARGA A VRAM DE QUEM O PERDEU, E CARIMBA O ASSADO.**
///
/// ⚠️ **As três metades e nenhuma é decoração:**
/// - *acende quem tem* — sem ela a fase é um laço vazio, e a suíte inteira fica verde;
/// - *larga quem não tem* — uma [`FormaViva`] órfã é **memória de placa que ninguém mais tem a
///   quem perguntar**, e um vazamento aqui é invisível a toda régua de imagem;
/// - *carimba o assado* — é o que impede a irmã `relight_stale` de re-acender por cima no quadro
///   seguinte, e o carimbo diz a VERDADE (*ele foi aceso com este rig*).
///
/// ⚠️ **O `BakedForm` da fixtura nasce com `form`/`form_occ` VAZIOS de propósito, e isso é a
/// afirmação mais forte do gate:** a rota B **não lê** os canais gravados — ela lê as texturas
/// residentes. *Se a fase caísse no caminho assado, ela estouraria aqui em vez de passar.*
///
/// **Mutações que devem sangrar:** apagar o `vivas.retain(…)`; `assado.lit_with = Some(…)` →
/// `= None`.
#[test]
#[ignore = "precisa de adapter"]
fn a_fase_acende_larga_e_carimba() {
    use std::collections::BTreeMap;

    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste (skip gracioso NÃO é verde)");
        return;
    };
    let base = base_chapado();
    let atlas = TextureAtlas::new(&gpu, LADO.max(256));
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let mut cena =
        crate::Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(32, 48, 1.0), 1.0);
    let mut passes = PassesDaLuz::default();

    let mut sim = ph2d_ecs::SimWorld::default();
    let com = sim
        .world_mut()
        .spawn(ph2d_ecs::Mesh3D {
            piece: 0,
            yaw: 0.0,
            pitch: 0.0,
            spin: 0.0,
        })
        .id();
    let sem = sim.world_mut().spawn(()).id();

    let mut assados: BTreeMap<u64, ph2d_form_donation::baked_form::BakedForm> = BTreeMap::new();
    for e in [com, sem] {
        let slot = renderer
            .acquire_individual(LADO, LADO, &base)
            .expect("slot do sprite");
        assados.insert(
            e.to_bits(),
            ph2d_form_donation::baked_form::BakedForm {
                size: (LADO, LADO),
                base: base.clone(),
                // Ver o doc: a rota B não os lê, e é esta linha que o afirma.
                form: Vec::new(),
                form_occ: Vec::new(),
                texture_id: slot,
                rig: LightRig::default(),
                lit_with: None,
                lei: ph2d_form_donation::lei_da_luz::Lei::default(),
                recorte: None,
            },
        );
    }

    // A órfã: uma forma viva de uma entidade que não está no mundo.
    let mut vivas: BTreeMap<u64, FormaViva> = BTreeMap::new();
    vivas.insert(u64::MAX, FormaViva::garante(None, &gpu, (LADO, LADO)));

    let conta = crate::vivo_fase::acende_os_cataventos(
        &mut cena,
        &mut assados,
        &mut vivas,
        crate::vivo_fase::Bancada {
            gpu: &gpu,
            renderer: &mut renderer,
            passes: &mut passes,
        },
        &mut sim,
        0.0,
    );

    assert_eq!(conta.acesos, 1, "so' a entidade com Mesh3D pode acender");
    assert_eq!(conta.largadas, 1, "a forma viva ORFA tem de ser largada");
    assert!(
        !vivas.contains_key(&u64::MAX),
        "a VRAM da orfa ficou no mapa — um vazamento que nenhuma regua de imagem ve'"
    );
    assert!(
        vivas.contains_key(&com.to_bits()),
        "a entidade com Mesh3D tem de ficar com a forma viva dela"
    );
    assert!(
        assados[&com.to_bits()].lit_with.is_some(),
        "o assado do catavento nao foi carimbado: a irma `relight_stale` re-acende por cima"
    );
    assert!(
        assados[&sem.to_bits()].lit_with.is_none(),
        "a fase carimbou um objecto que ela nao acendeu — o carimbo deixaria de dizer a verdade"
    );
}
