//! Os gates da ponte da barra — ver o cabeçalho de [`super`].

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::Transform;

const DT: f32 = 1.0 / 60.0;
const UV: [f32; 4] = [0.1, 0.2, 0.3, 0.4];

fn estado() -> HealthBarsState {
    HealthBarsState {
        uv: UV,
        ..HealthBarsState::new()
    }
}

fn vida(max: f32) -> Health {
    Health {
        max,
        start: max,
        ..Health::default()
    }
}

/// Um inimigo com vida `max`, a vida de agora `agora` (`None` = antes do 1.º tique) e uma barra.
fn inimigo(
    sim: &mut SimWorld,
    pos: [f32; 2],
    max: f32,
    agora: Option<f64>,
    barra: HealthBar,
) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Inimigo"),
        Transform::from_translation(Vec2::new(pos[0], pos[1])),
        vida(max),
        barra,
    ));
    if let Some(p) = agora {
        e.insert(HealthNow {
            pontos: p,
            escudo: 0.0,
            morta: p <= 0.0,
        });
    }
    e.id()
}

fn pontos(sim: &mut SimWorld, e: Entity, p: f64) {
    sim.world_mut().entity_mut(e).insert(HealthNow {
        pontos: p,
        escudo: 0.0,
        morta: p <= 0.0,
    });
}

/// `(centro, tamanho, cor)` de cada instância, arredondados para comparar.
fn faixas(s: &HealthBarsState) -> Vec<([f32; 2], [f32; 2], [f32; 4])> {
    s.instances
        .iter()
        .map(|i| (i.world_pos, i.size, i.tint))
        .collect()
}

fn perto(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

/// ⭐⭐⭐ **A barra do PRÓPRIO objecto:** fundo inteiro, vida da esquerda, acima do centro, no
/// ladrilho branco e à frente de tudo.
///
/// **Mutações que devem sangrar:** a vida medida contra o `start` em vez do `max` · a faixa da vida
/// centrada em vez de crescer da esquerda · o deslocamento ignorado.
#[test]
fn a_barra_mostra_a_vida_do_proprio_objecto() {
    let mut sim = SimWorld::new();
    let b = HealthBar::default();
    let e = inimigo(&mut sim, [2.0, 3.0], 100.0, Some(60.0), b.clone());
    let mut s = estado();
    s.frame(&mut sim, DT);
    let f = faixas(&s);
    assert_eq!(f.len(), 3, "fundo, rasto e vida: {f:?}");
    let y = 3.0 + b.offset_y;
    assert_eq!(
        f[0],
        ([2.0, y], [1.0, b.height], b.back),
        "o fundo é a vida inteira"
    );
    let (c, t, cor) = f[2];
    assert!(
        perto(c[0], 1.8) && perto(t[0], 0.6),
        "a vida a 60 % cresce da esquerda: {c:?} {t:?}"
    );
    assert_eq!(cor, b.fill);
    assert!(
        s.instances
            .iter()
            .all(|i| i.atlas_uv == UV && i.z_order == u32::MAX)
    );
    assert_eq!(
        s.instances.iter().map(|i| i.sub_order).collect::<Vec<_>>(),
        [0, 1, 2],
        "fundo → rasto → vida"
    );
    assert_eq!(
        s.rasto_of(e.to_bits()),
        Some(0.6),
        "o rasto nasce colado à vida"
    );
}

/// ⭐⭐⭐ **O rasto:** um golpe deixa o pedaço perdido visível, e ele escorre depois do atraso.
///
/// **Mutação que deve sangrar:** o estado do rasto não sobreviver ao quadro (nascer sempre).
#[test]
fn um_golpe_deixa_o_rasto_e_ele_escorre() {
    let mut sim = SimWorld::new();
    let b = HealthBar::default();
    let e = inimigo(&mut sim, [0.0, 0.0], 100.0, Some(100.0), b.clone());
    let mut s = estado();
    s.frame(&mut sim, DT);
    pontos(&mut sim, e, 50.0);
    s.frame(&mut sim, DT);
    assert_eq!(
        s.rasto_of(e.to_bits()),
        Some(1.0),
        "o rasto segura no golpe"
    );
    let rasto = faixas(&s)[1];
    assert!(
        perto(rasto.1[0], 1.0),
        "a faixa do rasto ainda mostra a vida de antes: {rasto:?}"
    );
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let quadros = ((b.trail_delay_s + 1.0 / b.trail_speed) / DT) as usize + 5;
    for _ in 0..quadros {
        s.frame(&mut sim, DT);
    }
    assert_eq!(s.rasto_of(e.to_bits()), Some(0.5), "o rasto chegou à vida");
    // E com o relógio parado ele espera.
    pontos(&mut sim, e, 20.0);
    s.frame(&mut sim, 0.0);
    s.frame(&mut sim, 0.0);
    assert_eq!(
        s.rasto_of(e.to_bits()),
        Some(0.5),
        "o relógio parado andou o rasto"
    );
}

/// ⭐⭐ **A barra de OUTRO objecto, pelo NOME** — a do herói no placar. Ela desenha-se onde está a
/// entidade que a CARREGA, e mostra a vida de quem ela nomeia; entre homónimos, nunca um molde.
///
/// **Mutação que deve sangrar:** ler a vida de quem carrega a barra em vez do alvo.
#[test]
fn a_barra_do_placar_mostra_a_vida_de_quem_ela_nomeia() {
    let mut sim = SimWorld::new();
    // Um MOLDE homónimo, com a vida cheia — se fosse escolhido a barra leria 100 %.
    // ⚠️ **Nasce PRIMEIRO e com `Transform`, e é o que faz a fixtura conter o fenómeno:** a
    // identidade mais baixa ganha o empate, e só quem tem `Transform` (ou pai) recebe uma. Com o
    // molde depois do herói a cerca dele era invisível — a prova de mutação apanhou-a viva.
    sim.world_mut().spawn((
        Name::new("Heroi"),
        Transform::IDENTITY,
        vida(10.0),
        MasterPiece,
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let heroi = sim
        .world_mut()
        .spawn((
            Name::new("Heroi"),
            Transform::from_translation(Vec2::new(-9.0, -9.0)),
            vida(10.0),
            HealthNow {
                pontos: 2.5,
                escudo: 0.0,
                morta: false,
            },
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let placar = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(5.0, 4.0)),
            HealthBar {
                target: "Heroi".into(),
                offset_y: 0.0,
                ..HealthBar::default()
            },
        ))
        .id();
    let mut s = estado();
    s.frame(&mut sim, DT);
    let f = faixas(&s);
    assert_eq!(f.len(), 3, "{f:?}");
    assert_eq!(
        f[0].0,
        [5.0, 4.0],
        "desenha-se onde está o placar, não o herói"
    );
    assert!(
        perto(f[2].1[0], 0.25),
        "mostra os 25 % do HERÓI: {:?}",
        f[2]
    );
    assert_eq!(s.rasto_of(placar.to_bits()), Some(0.25));
    let _ = heroi;
    // Um nome que ninguém tem não desenha nada.
    sim.world_mut().entity_mut(placar).insert(HealthBar {
        target: "Ninguem".into(),
        ..HealthBar::default()
    });
    s.frame(&mut sim, DT);
    assert!(
        s.instances.is_empty(),
        "um alvo inexistente desenhou uma barra"
    );
}

/// ⭐ **A barra segue a ESCALA de quem a carrega e nunca a rotação.**
///
/// **Mutação que deve sangrar:** a escala ignorada.
#[test]
fn a_barra_segue_a_escala_e_nao_roda() {
    let mut sim = SimWorld::new();
    let b = HealthBar::default();
    let e = inimigo(&mut sim, [0.0, 0.0], 100.0, Some(100.0), b.clone());
    sim.world_mut().entity_mut(e).insert(Transform {
        translation: Vec2::new(1.0, 1.0),
        rotation: std::f32::consts::FRAC_PI_2,
        scale: Vec2::new(2.0, 2.0),
        ..Transform::IDENTITY
    });
    let mut s = estado();
    s.frame(&mut sim, DT);
    let fundo = s.instances[0];
    assert_eq!(fundo.basis, [1.0, 0.0, 0.0, 1.0], "a barra rodou");
    assert!(
        perto(fundo.size[0], 2.0),
        "a escala não chegou à largura: {:?}",
        fundo.size
    );
    assert!(
        perto(fundo.world_pos[1], 1.0 + 2.0 * b.offset_y) && perto(fundo.world_pos[0], 1.0),
        "o deslocamento é na vertical do MUNDO e escalado: {:?}",
        fundo.world_pos
    );
}

/// ⚠️ **Quem NÃO desenha:** um molde, um objecto escondido, e a vida cheia com `hide_when_full` —
/// com o CONTROLO de que o mesmo objecto, sem essas condições, desenha.
#[test]
fn moldes_escondidos_e_vidas_cheias_escondidas_nao_desenham() {
    let mut sim = SimWorld::new();
    let e = inimigo(
        &mut sim,
        [0.0, 0.0],
        100.0,
        Some(100.0),
        HealthBar::default(),
    );
    let mut s = estado();
    s.frame(&mut sim, DT);
    assert_eq!(
        s.instances.len(),
        3,
        "controlo: a vida cheia desenha as três faixas"
    );

    sim.world_mut().entity_mut(e).insert(HealthBar {
        hide_when_full: true,
        ..HealthBar::default()
    });
    s.frame(&mut sim, DT);
    assert!(
        s.instances.is_empty(),
        "hide_when_full com a vida cheia desenhou"
    );
    pontos(&mut sim, e, 99.0);
    s.frame(&mut sim, DT);
    assert!(
        !s.instances.is_empty(),
        "com um arranhão ela tem de aparecer"
    );

    sim.world_mut().entity_mut(e).insert(Visibility::hidden());
    s.frame(&mut sim, DT);
    assert!(
        s.instances.is_empty(),
        "um objecto escondido desenhou a barra"
    );
    sim.world_mut()
        .entity_mut(e)
        .insert((Visibility::visible(), MasterPiece));
    s.frame(&mut sim, DT);
    assert!(s.instances.is_empty(), "um molde desenhou a barra");
    // ⚠️ E antes do 1.º passe que DERIVA o `MasterPiece`, a raiz só tem o `MasterRoot`.
    sim.world_mut()
        .entity_mut(e)
        .remove::<MasterPiece>()
        .insert(ph2d_ecs::MasterRoot);
    s.frame(&mut sim, DT);
    assert!(
        s.instances.is_empty(),
        "a raiz de um molde desenhou a barra antes de o passe a marcar"
    );
}

/// ⭐ **Antes do 1.º tique a barra lê a vida de NASCENÇA** — vazia leria-se como um inimigo morto;
/// e com `max = 0` («sem máximo») ela mede contra ela.
#[test]
fn antes_do_play_a_barra_le_a_vida_de_nascenca() {
    let mut sim = SimWorld::new();
    let e = inimigo(&mut sim, [0.0, 0.0], 40.0, None, HealthBar::default());
    let mut s = estado();
    s.frame(&mut sim, DT);
    assert_eq!(s.rasto_of(e.to_bits()), Some(1.0));
    sim.world_mut().entity_mut(e).insert(Health {
        max: 0.0,
        start: 40.0,
        ..Health::default()
    });
    pontos(&mut sim, e, 10.0);
    s.frame(&mut sim, DT);
    assert!(
        perto(faixas(&s)[2].1[0], 0.25),
        "sem máximo, mede contra a vida de nascença"
    );
    sim.world_mut().entity_mut(e).insert(Health {
        max: 0.0,
        start: 0.0,
        ..Health::default()
    });
    s.frame(&mut sim, DT);
    assert!(s.instances.is_empty(), "sem denominador não há barra");
}

/// ⭐ **Rebobinar é renascer:** o rasto volta colado à vida.
#[test]
fn rebobinar_faz_o_rasto_nascer_colado_a_vida() {
    let mut sim = SimWorld::new();
    let e = inimigo(
        &mut sim,
        [0.0, 0.0],
        100.0,
        Some(100.0),
        HealthBar::default(),
    );
    let mut s = estado();
    s.frame(&mut sim, DT);
    pontos(&mut sim, e, 30.0);
    s.frame(&mut sim, DT);
    assert_eq!(s.rasto_of(e.to_bits()), Some(1.0));
    assert_eq!(s.rewind(), 1);
    assert!(s.instances.is_empty());
    s.frame(&mut sim, DT);
    assert_eq!(
        s.rasto_of(e.to_bits()),
        Some(0.3),
        "o rasto do passado sobreviveu ao rebobinar"
    );
}

/// ⭐ **A SONDA do custo — «quanto custa uma barra por inimigo?»** (plano 28, W4: a wave abre com
/// esta medição). Corre-se à mão, calma e em `--release`:
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-app-components --release --lib mede_o_custo -- --ignored --nocapture`.
/// ⛔ **Sem tecto no produto:** o número é o que diz se um tecto é preciso; ele sai daqui, nunca
/// de um palpite (§0.0). A cena de alvos com rasto a escorrer é o pior caso (o rasto trabalha).
#[test]
#[ignore = "sonda de relógio: corre-se à mão com a máquina calma"]
fn mede_o_custo_de_n_barras() {
    for n in [100usize, 1_000, 10_000] {
        let mut sim = SimWorld::new();
        let alvos: Vec<Entity> = (0..n)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let x = i as f32;
                inimigo(&mut sim, [x, 0.0], 100.0, Some(100.0), HealthBar::default())
            })
            .collect();
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        let mut s = estado();
        s.frame(&mut sim, DT);
        for &e in &alvos {
            pontos(&mut sim, e, 40.0);
        }
        let quadros = 60u32;
        let t0 = std::time::Instant::now();
        for _ in 0..quadros {
            s.frame(&mut sim, DT);
        }
        let ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(quadros);
        let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
        println!(
            "barras {n:>6}: {ms:>8.3} ms/quadro · {:>6.1} % de 16,67 ms · loadavg {}",
            ms / 16.67 * 100.0,
            carga.split_whitespace().next().unwrap_or("?")
        );
        assert_eq!(s.instances.len(), 3 * n);
    }
}

/// ⭐⭐ **O quadro das barras envelhece o IMPACTO, e o rebobinar esquece-o** (plano 28, W5) — o
/// impacto mora aqui para renascer junto com os rastos, e é esta a prova de que o faz.
///
/// **Mutações que devem sangrar:** apagar o `impacto.anda` do `frame` · o `impacto.rewind` do
/// `rewind`.
#[test]
fn o_quadro_envelhece_o_impacto_e_o_rebobinar_esquece_o() {
    use ph2d_physics_ecs::{HealthEvent, HealthEventKind};
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            Health {
                numbers: true,
                ..Health::default()
            },
        ))
        .id();
    let mut st = estado();
    let golpe = HealthEvent {
        target: e,
        source: e,
        kind: HealthEventKind::Damaged { amount: 5.0 },
    };
    st.impacto.ouve(&sim, &[golpe]);
    assert_eq!(st.impacto.numeros().len(), 1);
    st.frame(&mut sim, 1.0);
    assert!(
        st.impacto.numeros().is_empty(),
        "o quadro não envelheceu o número"
    );
    st.impacto.ouve(&sim, &[golpe]);
    st.impacto.pausa.pede(0.1);
    let _ = st.rewind();
    assert!(st.impacto.numeros().is_empty());
    assert_eq!(
        st.impacto.pausa.resta_s(),
        0.0,
        "o rebobinar não esqueceu a pausa"
    );
}
