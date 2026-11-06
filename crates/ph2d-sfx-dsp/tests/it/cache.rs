//! **O cache por camada dá os mesmos bytes do render sem cache**, e refaz só a camada editada.

use ph2d_sfx_dsp::{RenderCache, pcm_hash, render};
use ph2d_sfx_recipe::{ENGINE_V1, Recipe, Source, Takes, Tone, Wave, factory};

fn impacto() -> Recipe {
    factory::all()
        .into_iter()
        .find(|(n, _)| *n == "Impact")
        .unwrap()
        .1
}

fn igual_ao_render_sem_cache(cache: &mut RenderCache, r: &Recipe) -> usize {
    let t = Takes::default();
    let com = cache.render(r, r.seed, ENGINE_V1, &t).unwrap();
    let sem = render(r, r.seed, ENGINE_V1, &t).unwrap();
    assert_eq!(pcm_hash(&com.samples), pcm_hash(&sem.samples));
    cache.last_rendered
}

#[test]
fn o_cache_da_os_mesmos_bytes_e_refaz_so_a_camada_editada() {
    let mut r = impacto();
    let mut c = RenderCache::default();
    let n = r.layers.len();
    assert_eq!(igual_ao_render_sem_cache(&mut c, &r), n, "cache vazio");
    assert_eq!(igual_ao_render_sem_cache(&mut c, &r), 0, "nada mudou");

    // Só o ganho: a soma muda, o kernel não corre.
    r.layers[0].gain *= 0.5;
    assert_eq!(igual_ao_render_sem_cache(&mut c, &r), 0, "só o ganho");

    // Uma fonte editada: uma camada.
    r.layers[1].source = Source::Tone(Tone {
        wave: Wave::Square,
        hz: 300.0,
        hz_end: 300.0,
        glide: 0.0,
    });
    assert_eq!(igual_ao_render_sem_cache(&mut c, &r), 1, "uma fonte");

    // Um envelope editado: uma camada.
    r.layers[2].envelope = ph2d_sfx_recipe::Envelope::hit(0.01, 0.2, 0.01);
    assert_eq!(igual_ao_render_sem_cache(&mut c, &r), 1, "um envelope");

    // A semente da receita muda toda camada.
    r.seed ^= 1;
    assert_eq!(igual_ao_render_sem_cache(&mut c, &r), n, "a semente");

    // A duração refaz todas, e o buffer novo tem o tamanho novo.
    r.seconds = 0.3;
    assert_eq!(igual_ao_render_sem_cache(&mut c, &r), n, "a duração");

    // Apagar não pede cálculo, e a camada apagada sai do cache.
    r.layers.remove(0);
    assert_eq!(igual_ao_render_sem_cache(&mut c, &r), 0, "apagar");
}

/// A ordem de soma é a do id: reordenar na tela não muda um bit, com ou sem cache.
#[test]
fn reordenar_a_pilha_nao_muda_um_bit() {
    let r = impacto();
    let mut virada = r.clone();
    virada.layers.reverse();
    let t = Takes::default();
    let a = render(&r, r.seed, ENGINE_V1, &t).unwrap().samples;
    let b = render(&virada, r.seed, ENGINE_V1, &t).unwrap().samples;
    assert_eq!(pcm_hash(&a), pcm_hash(&b));
}

/// O início de uma camada só desloca: não refaz nada se o trecho couber igual.
#[test]
fn o_inicio_desloca_a_camada_sem_mudar_os_bytes_dela() {
    let (_, mut r) = factory::all().remove(1);
    let t = Takes::default();
    r.seconds = 1.0;
    let a = render(&r, r.seed, ENGINE_V1, &t).unwrap().samples;
    r.layers[0].start = 0.25;
    let b = render(&r, r.seed, ENGINE_V1, &t).unwrap().samples;
    assert_eq!(a[..1_000], b[12_000..13_000]);
    assert!(b[..12_000].iter().all(|s| *s == 0.0));
}
