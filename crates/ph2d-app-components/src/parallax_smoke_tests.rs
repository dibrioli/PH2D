//! Os gates da cena da PARALAXE (plano 24, W7).
//!
//! ⚠️⚠️ **Eles perguntam o que a cena ENSINA, e não só se ela monta.** Os irmãos desta crate
//! pagaram três vezes a mesma forma — uma cena com as peças todas e o sujeito fora do ecrã, um
//! herói sem corpo, uma bomba fora da banda — e em todas os gates liam os DADOS e nenhum perguntava
//! *«o dono consegue ver/alcançar isto?»*. ⇒ cada gate aqui é uma frase do roteiro.

use super::*;
use ph2d_ecs::{ChildOf, SimWorld};
use ph2d_preview_drive::PreviewDrive;

fn montada(nivel: u32) -> (SimWorld, Montada) {
    let mut sim = SimWorld::default();
    let m = montar(sim.world_mut(), nivel);
    (sim, m)
}

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    entidade_por_nome(sim.world_mut(), nome)
}

/// ⭐ **`CENAS` é CONTADO do `montar`** — e um nível fora da escada cai no `=1`, nunca em nada.
#[test]
fn as_cenas_declaradas_sao_as_que_o_montar_monta() {
    assert_eq!(CENAS, 2);
    for n in 1..=CENAS {
        assert_eq!(
            montada(n).1.nivel,
            n,
            "o nível {n} tem de montar-se a si mesmo"
        );
    }
    assert_eq!(
        montada(99).1.nivel,
        1,
        "um nível fora da escada cai na cena `=1`"
    );
}

/// ⭐⭐⭐ **A frase (1) do roteiro: UM número separa os planos.** Dois vizinhos a menos de `0,2`
/// leem-se como um só borrão — o vale que o cabeçalho das constantes mede.
#[test]
fn os_planos_vizinhos_diferem_o_bastante_para_se_lerem() {
    let (mut sim, _) = montada(1);
    let k = |sim: &mut SimWorld, n: &str| {
        let e = por_nome(sim, n);
        sim.world().get::<ScrollFactor>(e).expect(n).k
    };
    let ks = [
        1.0, // o chão: sem componente, a `k = 1` por construção
        k(&mut sim, "Arvores")[0],
        k(&mut sim, "Colinas")[0],
        k(&mut sim, "Ceu")[0],
    ];
    for w in ks.windows(2) {
        assert!(
            w[0] - w[1] >= 0.2,
            "dois planos vizinhos a {:.2} e {:.2} não se distinguem",
            w[0],
            w[1]
        );
    }
    let chao = por_nome(&mut sim, "Chao");
    assert!(
        sim.world().get::<ScrollFactor>(chao).is_none(),
        "o chão É o mundo — um `ScrollFactor` nele seria o neutro escrito outra vez"
    );
}

/// ⭐⭐ **A régua do chão existe:** sem nada a `k = 1` a passar pelo herói, os fundos leem-se como
/// *«o mundo inteiro anda devagar»*.
#[test]
fn os_postes_do_chao_sao_a_regua() {
    let (mut sim, _) = montada(1);
    let postes = sim
        .world_mut()
        .query::<(&Name, &Transform)>()
        .iter(sim.world())
        .filter(|(n, _)| n.0.starts_with("Poste "))
        .count();
    assert!(
        postes >= 20,
        "só {postes} postes — a régua do passo é o que torna a cena legível"
    );
}

/// ⭐⭐⭐ **A câmera fica PRESA em Y** — a região da cerca tem a altura EXACTA da vista, logo a lei
/// devolve o meio. ⚠️ A régua é a do `height_world` de fábrica, e não a constante sozinha: sem
/// ela, subir o `height_world` deixava a cerca mais estreita que a vista em silêncio.
#[test]
fn a_camera_fica_presa_em_y() {
    let (mut sim, _) = montada(1);
    let cam = por_nome(&mut sim, "Camera");
    let lim = sim.world().get::<CameraLimits>(cam).expect("a cerca");
    let alt = sim
        .world()
        .get::<GameCamera>(cam)
        .expect("a câmera")
        .height_world;
    assert!(
        (lim.max[1] - lim.min[1] - alt).abs() < 1e-6,
        "a cerca tem de medir a vista"
    );
    assert!((alt / 2.0 - MEIA_VISTA_Y).abs() < 1e-6);
}

/// Os filhos de uma camada, em coordenadas do MUNDO (a pose do pai + a local), com o tamanho.
fn pecas(sim: &mut SimWorld, pai: Entity) -> Vec<([f32; 2], [f32; 2])> {
    let base = sim.world().get::<Transform>(pai).expect("pai").translation;
    sim.world_mut()
        .query::<(&ChildOf, &Transform, &Sprite)>()
        .iter(sim.world())
        .filter(|(c, ..)| c.0 == pai)
        .map(|(_, t, s)| ([base.x + t.translation.x, base.y + t.translation.y], s.size))
        .collect()
}

/// A meia-largura mais larga que a cena tem de aguentar: um ecrã `21:9`.
const MEIA_LARGURA_MAX: f32 = MEIA_VISTA_Y * 21.0 / 9.0;

/// ⭐⭐⭐ **A frase (2): o céu e as árvores NUNCA acabam** — a fileira tem de cobrir a vista mais a
/// janela do embrulho (`±ladrilho/2`), senão a costura entra no ecrã num ultrawide.
#[test]
fn a_fileira_que_repete_cobre_a_vista_e_a_janela_do_embrulho() {
    let (mut sim, _) = montada(1);
    for nome in ["Arvores", "Ceu"] {
        let e = por_nome(&mut sim, nome);
        let tile = sim.world().get::<ScrollRepeat>(e).expect(nome).tile[0];
        assert!(
            sim.world().get::<ScrollLimits>(e).is_none(),
            "{nome} repete e não tem cerca"
        );
        let p = pecas(&mut sim, e);
        let esq = p
            .iter()
            .map(|(c, s)| c[0] - s[0] / 2.0)
            .fold(f32::MAX, f32::min);
        let dir = p
            .iter()
            .map(|(c, s)| c[0] + s[0] / 2.0)
            .fold(f32::MIN, f32::max);
        let precisa = MEIA_LARGURA_MAX + tile / 2.0;
        assert!(
            -esq >= precisa && dir >= precisa,
            "{nome}: a fileira cobre [{esq:.1}, {dir:.1}] e precisa de ±{precisa:.1}"
        );
        // ⚠️ E o PASSO entre peças é o ladrilho — senão a repetição salta para uma peça que não
        // está onde a anterior estava.
        let mut xs: Vec<f32> = p.iter().map(|(c, _)| c[0]).collect();
        xs.sort_by(f32::total_cmp);
        for w in xs.windows(2) {
            assert!(
                (w[1] - w[0] - tile).abs() < 1e-4,
                "{nome}: passo {} ≠ ladrilho {tile}",
                w[1] - w[0]
            );
        }
    }
}

/// ⭐⭐⭐ **A frase (2), a outra metade: as colinas PARAM na borda** — e o contraste só ensina se
/// elas não repetirem.
#[test]
fn as_colinas_tem_cerca_e_nao_repetem() {
    let (mut sim, _) = montada(1);
    let e = por_nome(&mut sim, "Colinas");
    assert!(sim.world().get::<ScrollRepeat>(e).is_none());
    assert!(sim.world().get::<ScrollLimits>(e).is_some());
}

/// ⭐⭐⭐ **O que a cerca promete: a borda da serra nunca entra em cena.** Corre a PONTE (a porta do
/// produto) com a câmera muito para lá da cerca e mede a serra contra a vista.
///
/// ⚠️⚠️ **E a metade que torna a primeira honesta:** com a cerca ALARGADA (o passo (5) do roteiro)
/// a borda ENTRA — senão o gate passaria com uma serra infinita e a cerca não mediria nada.
#[test]
fn a_borda_da_serra_nunca_entra_em_cena_e_entra_sem_a_cerca() {
    for (alarga, deve_cobrir) in [(false, true), (true, false)] {
        let (mut sim, _) = montada(1);
        let e = por_nome(&mut sim, "Colinas");
        if alarga {
            sim.world_mut()
                .get_mut::<ScrollLimits>(e)
                .expect("cerca")
                .max[0] = 80.0;
        }
        let meia = [MEIA_LARGURA_MAX, MEIA_VISTA_Y];
        let mut drive = PreviewDrive::default();
        let mut cobre = true;
        for c in [0.0_f32, 15.0, 30.0, 60.0, 70.0] {
            drive_parallax(&mut sim, Some(([c, 0.0], meia)), 0.0, &mut drive);
            let p = pecas(&mut sim, e);
            let dir = p
                .iter()
                .map(|(q, s)| q[0] + s[0] / 2.0)
                .fold(f32::MIN, f32::max);
            cobre &= dir >= c + meia[0];
        }
        assert_eq!(cobre, deve_cobrir, "cerca alargada = {alarga}");
    }
}

use crate::parallax_bridge::drive_parallax;

/// ⭐⭐⭐ **A frase (1), medida pela PONTE: longe anda MENOS no ecrã.** O deslocamento no ecrã de
/// uma camada é `k · Δcâmera` — o chão passa o passo inteiro, o céu quase nada.
#[test]
fn no_ecra_o_mais_longe_anda_menos() {
    let (mut sim, _) = montada(1);
    let meia = [MEIA_LARGURA_MAX, MEIA_VISTA_Y];
    let mut drive = PreviewDrive::default();
    let pose = |sim: &mut SimWorld, n: &str| {
        let e = por_nome(sim, n);
        sim.world().get::<Transform>(e).expect(n).translation.x
    };
    drive_parallax(&mut sim, Some(([0.0, 0.0], meia)), 0.0, &mut drive);
    let antes: Vec<f32> = ["Arvores", "Colinas", "Ceu"]
        .iter()
        .map(|n| pose(&mut sim, n))
        .collect();
    // ⚠️ Um passo MENOR que meio ladrilho de todas: senão o embrulho mistura-se na régua.
    let passo = 2.0_f32;
    drive_parallax(&mut sim, Some(([passo, 0.0], meia)), 0.0, &mut drive);
    let depois: Vec<f32> = ["Arvores", "Colinas", "Ceu"]
        .iter()
        .map(|n| pose(&mut sim, n))
        .collect();
    // no ecrã = mundo − câmera
    let ecra: Vec<f32> = antes
        .iter()
        .zip(&depois)
        .map(|(a, d)| passo - (d - a))
        .collect();
    assert!(
        (ecra[0] - K_ARVORES * passo).abs() < 1e-4,
        "árvores: {}",
        ecra[0]
    );
    assert!(
        (ecra[1] - K_COLINAS * passo).abs() < 1e-4,
        "colinas: {}",
        ecra[1]
    );
    assert!((ecra[2] - K_CEU * passo).abs() < 1e-4, "céu: {}", ecra[2]);
    assert!(passo > ecra[0] && ecra[0] > ecra[1] && ecra[1] > ecra[2]);
}

/// ⭐⭐ **A frase (4): as nuvens andam SOZINHAS** — com a câmera parada, o relógio move o céu e só
/// o céu.
#[test]
fn com_a_camera_parada_so_o_ceu_anda() {
    let (mut sim, _) = montada(1);
    let meia = [MEIA_LARGURA_MAX, MEIA_VISTA_Y];
    let mut drive = PreviewDrive::default();
    let x = |sim: &mut SimWorld, n: &str| {
        let e = por_nome(sim, n);
        sim.world().get::<Transform>(e).expect(n).translation.x
    };
    drive_parallax(&mut sim, Some(([0.0, 0.0], meia)), 0.0, &mut drive);
    let a = [x(&mut sim, "Arvores"), x(&mut sim, "Ceu")];
    drive_parallax(&mut sim, Some(([0.0, 0.0], meia)), 2.0, &mut drive);
    let d = [x(&mut sim, "Arvores"), x(&mut sim, "Ceu")];
    assert!((d[0] - a[0]).abs() < 1e-6, "as árvores não têm deriva");
    assert!(
        (d[1] - a[1] - 2.0 * DERIVA_CEU).abs() < 1e-4,
        "o céu deriva {} em 2 s",
        d[1] - a[1]
    );
}

/// ⭐⭐⭐ **Tudo cabe na vista em Y**, com folga para a barra do topo — a armadilha que três waves
/// desta linha pagaram (o sujeito do passo fora do ecrã, com os gates todos verdes).
#[test]
fn toda_peca_cabe_na_vista_em_y() {
    let (mut sim, _) = montada(1);
    let limite = MEIA_VISTA_Y * 0.9;
    let mut pior = 0.0_f32;
    let raizes: Vec<Entity> = sim
        .world_mut()
        .query::<(Entity, &Transform)>()
        .iter(sim.world())
        .map(|(e, _)| e)
        .collect();
    for e in raizes {
        let w = sim.world();
        let Some(s) = w.get::<Sprite>(e) else {
            continue;
        };
        let t = w.get::<Transform>(e).expect("pose").translation;
        let y = w
            .get::<ChildOf>(e)
            .and_then(|c| w.get::<Transform>(c.0))
            .map_or(t.y, |p| p.translation.y + t.y);
        pior = pior
            .max((y + s.size[1] / 2.0).abs())
            .max((y - s.size[1] / 2.0).abs());
    }
    assert!(
        pior <= limite,
        "a peça mais alta chega a ±{pior:.2}, e a vista útil é ±{limite:.2}"
    );
}

/// ⭐⭐ **O herói ANDA** — o corpo cinemático e o collider, a lição do report de 19/09.
#[test]
fn o_heroi_tem_corpo_cinematico() {
    let (mut sim, _) = montada(1);
    let h = por_nome(&mut sim, "Heroi");
    let w = sim.world();
    assert!(w.get::<ph2d_physics_ecs::TopDownPlayer>(h).is_some());
    assert!(matches!(
        w.get::<RigidBody>(h).map(|b| b.kind),
        Some(BodyKind::Kinematic)
    ));
    assert!(w.get::<Collider>(h).is_some());
}

/// ⭐⭐ **Quem nasce escolhido é o sujeito do roteiro:** as Árvores na `=1`, a câmera na `=2`.
#[test]
fn quem_nasce_escolhido_e_o_sujeito_do_roteiro() {
    let (sim, m) = montada(1);
    let e = Entity::from_bits(m.escolhido);
    assert_eq!(
        sim.world().get::<Name>(e).map(|n| n.0.as_str()),
        Some("Arvores")
    );
    assert!(sim.world().get::<ScrollFactor>(e).is_some());
    let (sim, m) = montada(2);
    let e = Entity::from_bits(m.escolhido);
    assert!(sim.world().get::<GameCamera>(e).is_some());
}

/// ⭐⭐ **A `=2` é o DOLLY e mais nada:** ninguém anda e a câmera não segue ninguém.
#[test]
fn na_cena_do_dolly_ninguem_anda() {
    let (mut sim, _) = montada(2);
    let herois = sim
        .world_mut()
        .query::<&ph2d_physics_ecs::TopDownPlayer>()
        .iter(sim.world())
        .count();
    assert_eq!(herois, 0);
    let cam = por_nome(&mut sim, "Camera");
    assert!(
        sim.world()
            .get::<CameraFollow>(cam)
            .expect("follow")
            .target
            .is_empty()
    );
}

/// ⭐⭐⭐ **A frase (2) da `=2`, medida pela PONTE:** com o dolly a `½` o chão fica do mesmo tamanho
/// e os três fundos ENCOLHEM, o mais longe mais.
#[test]
fn o_dolly_encolhe_os_fundos_e_nao_o_chao() {
    let (mut sim, _) = montada(2);
    let cam = por_nome(&mut sim, "Camera");
    sim.world_mut()
        .get_mut::<GameCamera>(cam)
        .expect("câmera")
        .dolly = 0.5;
    let mut drive = PreviewDrive::default();
    drive_parallax(
        &mut sim,
        Some(([0.0, 0.0], [MEIA_LARGURA_MAX, MEIA_VISTA_Y])),
        0.0,
        &mut drive,
    );
    let esc = |sim: &mut SimWorld, n: &str| {
        let e = por_nome(sim, n);
        sim.world().get::<Transform>(e).expect(n).scale.x
    };
    let (a, c, s) = (
        esc(&mut sim, "Arvores"),
        esc(&mut sim, "Colinas"),
        esc(&mut sim, "Ceu"),
    );
    let chao = esc(&mut sim, "Chao");
    assert!((chao - 1.0).abs() < 1e-6, "o chão mudou de tamanho: {chao}");
    assert!(
        1.0 > a && a > c && c > s,
        "árvores {a} · colinas {c} · céu {s}"
    );
}

/// ⭐⭐⭐ **O roteiro nomeia o que o painel PINTA** — as chaves que ele lê existem, e as da secção
/// são as MESMAS que o pintor da secção usa (lido do fonte dele, que é a outra metade da afirmação).
#[test]
fn o_roteiro_nomeia_o_que_o_painel_pinta() {
    let chaves = [
        "panel.inspector.camera.camera",
        "panel.inspector.camera.dolly",
        "panel.inspector.parallax.parallax",
        "panel.inspector.parallax.scroll_factor",
        "panel.inspector.parallax.repeat_m",
        "panel.inspector.parallax.drift_m_s",
        "panel.inspector.parallax.limit_max_m",
    ];
    let fonte = include_str!("parallax_smoke.rs");
    let pintor = include_str!("../../ph2d-panel-inspector/src/sections/parallax.rs");
    let camara = include_str!("../../ph2d-panel-inspector/src/sections/camera.rs");
    for k in chaves {
        assert_ne!(ph2d_i18n::tr(k), k, "a chave {k} não tem texto");
        assert!(fonte.contains(k), "o roteiro já não lê {k}");
        assert!(
            pintor.contains(k) || camara.contains(k),
            "nenhum pintor pinta {k}"
        );
    }
}
