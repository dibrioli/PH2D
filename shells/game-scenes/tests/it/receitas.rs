//! `PH2D_SFX_SMOKE=2`: os sons da receita pelo caminho do produto. A sonda é o [`sonda_cada_receita_sai_do_mixer`]:
//! o que o Preview toca é o que `render` devolveu, e os números da mensagem ao dono saem dela.

use ph2d_app_audio::audio_2d::play_target_with;
use ph2d_app_audio::scene::SceneAudio;
use ph2d_audio::{AudioEngine, AudioFormat};
use ph2d_ecs::{
    AudioSource2D, Entity, Name, PickStrategy, ProjectSounds, SignalActions, SignalVerb, SimWorld,
    SoundSet, assign_missing_stable_ids,
};
use ph2d_sfx_dsp::render;
use ph2d_sfx_recipe::{ENGINE_V1, SfxLibrary};
use ph2d_sfx_scene as s;
use s::receitas as r;

fn cena() -> SimWorld {
    let mut sim = SimWorld::new();
    let jogador = crate::comp::montar(&mut sim, s::NIVEL_DO_JOGO);
    crate::comp::com_portas(&crate::comp::registo(), |p| {
        r::veste(&mut sim, p.menu.anexa, p.menu.raiz, jogador)
    });
    assign_missing_stable_ids(sim.world_mut());
    sim
}

fn objeto(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    w.query::<(Entity, &Name)>()
        .iter(w)
        .find(|(_, n)| n.as_str() == nome)
        .unwrap_or_else(|| panic!("sem o objeto {nome}"))
        .0
}

#[test]
fn a_cena_tem_um_objeto_por_receita_e_as_moedas_apontam_para_o_conjunto() {
    let mut sim = cena();
    for nome in r::ORDEM {
        let e = objeto(&mut sim, nome);
        let som = sim
            .world()
            .get::<AudioSource2D>(e)
            .expect("fonte")
            .sound
            .clone();
        let sons = sim.world().resource::<ProjectSounds>();
        assert!(
            sons.get(&som).is_some(),
            "{nome}: «{som}» não está no projeto"
        );
    }
    assert_eq!(
        sim.world().resource::<SfxLibrary>().recipes.len(),
        r::ORDEM.len()
    );

    let c = objeto(&mut sim, r::CONJUNTO);
    let set = sim.world().get::<SoundSet>(c).expect("conjunto");
    assert_eq!(set.strategy, PickStrategy::Sequence);
    assert_eq!(set.sounds.len(), r::ORDEM.len());

    let w = sim.world_mut();
    let mut moedas = 0;
    for (n, acoes) in w.query::<(&Name, &SignalActions)>().iter(w) {
        if !n.as_str().starts_with("Coin ") {
            continue;
        }
        moedas += 1;
        let toca = acoes.0.iter().find(|a| a.verb == SignalVerb::PlaySound);
        assert_eq!(
            toca.expect("a moeda toca").target,
            r::CONJUNTO,
            "{}",
            n.as_str()
        );
    }
    assert_eq!(moedas, 5);
}

/// **A sonda:** cada objeto, pelo mesmo `play_target` do botão Preview, e o que sai do MIXER contra o buffer da receita.
#[test]
fn sonda_cada_receita_sai_do_mixer() {
    let mut sim = cena();
    let lib = sim.world().resource::<SfxLibrary>().clone();
    const BLOCO: usize = 256;
    println!("som\tduração\tpico dBFS\tresíduo contra o render, dB");
    for nome in r::ORDEM {
        let receita = &lib.recipes[nome];
        let t = std::time::Instant::now();
        let esperado = render(receita, receita.seed, ENGINE_V1, &lib.takes).unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        assert!(esperado.missing.is_empty(), "{nome}");

        let e = objeto(&mut sim, nome);
        let (mut livro, (mut motor, mut mixer)) = (
            SceneAudio::default(),
            AudioEngine::new(AudioFormat::stereo(48_000)),
        );
        assert!(
            play_target_with(&mut sim, &mut livro, &mut motor, e),
            "{nome}"
        );
        let mut esquerda = Vec::new();
        while esquerda.len() < esperado.samples.len() {
            let mut out = vec![0.0f32; BLOCO * 2];
            mixer.render(&mut out, BLOCO);
            esquerda.extend(out.chunks(2).map(|c| c[0]));
        }
        esquerda.truncate(esperado.samples.len());

        // Em silêncio não há o que ouvir, e o ganho do mixer é um só: o resto do sinal é o erro.
        let cruzado: f64 = esquerda
            .iter()
            .zip(&esperado.samples)
            .map(|(a, b)| f64::from(*a) * f64::from(*b))
            .sum();
        let energia: f64 = esperado.samples.iter().map(|b| f64::from(*b).powi(2)).sum();
        assert!(energia > 0.0, "{nome}: a receita saiu muda");
        let ganho = cruzado / energia;
        let residuo: f64 = esquerda
            .iter()
            .zip(&esperado.samples)
            .map(|(a, b)| (f64::from(*a) - ganho * f64::from(*b)).powi(2))
            .sum();
        let db = 10.0 * (residuo / (ganho * ganho * energia)).max(1.0e-30).log10();
        let pico = esperado.samples.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        println!(
            "{nome}\t{:.2} s, {ms:.2} ms\t{:.1}\t{db:.1}",
            receita.seconds,
            20.0 * pico.log10()
        );
        assert!(
            ganho > 0.1,
            "{nome}: o mixer quase não tocou (ganho {ganho})"
        );
        assert!(db < -60.0, "{nome}: saiu outro som ({db:.1} dB)");
    }
}
