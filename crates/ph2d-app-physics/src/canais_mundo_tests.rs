//! Os gates da [`super`] — a família MUNDO: a curva chega ao mundo vivo sem limpar o scrub e sem
//! tocar no autorado, e a mão ganha-lhe no quadro em que muda.

use std::collections::BTreeSet;

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, RigidBody};

/// Um corpo a cair livre, a Cena, e a ponte.
fn queda() -> (SimWorld, Entity, PhysicsBridge) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 50.0)),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider::default(),
        ))
        .id();
    ph2d_ecs::garante_cena(sim.world_mut());
    (sim, e, PhysicsBridge::default())
}

fn curva(sim: &mut SimWorld, campo: u16, v: f32) {
    let c = cena(sim).expect("cena");
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(c, k(campo), v);
}

fn lido(sim: &mut SimWorld, ponte: &PhysicsBridge, campo: u16) -> f32 {
    publicar(sim, ponte);
    let c = cena(sim).expect("cena");
    sim.world()
        .resource::<CanaisLidos>()
        .valor(c, k(campo))
        .expect("publicado")
}

fn y(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .expect("corpo")
        .translation
        .y
}

/// ⭐⭐⭐ A curva inverte a gravidade a MEIO da queda: o corpo trava (controlo: sem ela continua a
/// cair), o anel do scrub fica (o `set_settings` limpá-lo-ia — controlo da régua), e o autorado não
/// muda.
#[test]
fn a_curva_muda_a_gravidade_do_mundo_sem_limpar_o_scrub() {
    let correr = |com_curva: bool| {
        let (mut sim, e, mut ponte) = queda();
        let mut cond = ConducaoMundo::default();
        for t in 1..=60 {
            aplicar(&mut sim, &mut ponte, &mut cond);
            ponte.dispatch(&mut sim, true, t);
        }
        let anel = ponte.ring_stats().0;
        assert!(anel > 0, "o scrub guardou pontos");
        for t in 61..=150 {
            if com_curva {
                curva(&mut sim, m::GRAVIDADE_Y, 9.81);
            }
            aplicar(&mut sim, &mut ponte, &mut cond);
            // No quadro da mudança, ANTES do passo: um `clear` aqui seria tapado pelos passos seguintes.
            assert!(ponte.ring_stats().0 >= anel, "o anel foi limpo no tick {t}");
            ponte.dispatch(&mut sim, true, t);
        }
        assert_eq!(ponte.settings(), PhysicsSettings::default(), "o autorado");
        (y(&sim, e), ponte)
    };
    let (sem, _) = correr(false);
    let (com, ponte) = correr(true);
    assert!(com > sem + 5.0, "a curva travou a queda: {com} × {sem}");
    assert!((ponte.settings_vivos().gravity_y - 9.81).abs() < 1e-6);
    // Controlo da régua: a porta do painel limpa o anel.
    let mut p = ponte;
    p.set_settings(PhysicsSettings {
        gravity_y: 9.81,
        ..PhysicsSettings::default()
    });
    assert_eq!(p.ring_stats().0, 0, "o set_settings limpa — a régua mede");
}

/// ⭐ O que o save grava (`project_save.rs` lê `PhysicsBridge::settings`) é o AUTORADO, com uma
/// curva activa a mudar o mundo.
#[test]
fn o_save_grava_o_autorado_com_uma_curva_activa() {
    let (mut sim, _, mut ponte) = queda();
    let mut cond = ConducaoMundo::default();
    curva(&mut sim, m::GRAVIDADE_Y, 4.0);
    curva(&mut sim, m::SUBPASSOS, 8.0);
    assert!(aplicar(&mut sim, &mut ponte, &mut cond), "o mundo mudou");
    assert_eq!(ponte.settings_vivos().substeps, 8);
    assert_eq!(ponte.settings(), PhysicsSettings::default());
    // O eco: o Auto Key lê a curva, não uma diferença.
    assert!((lido(&mut sim, &ponte, m::GRAVIDADE_Y) - 4.0).abs() < 1e-6);
}

/// ⭐ A MÃO ganha à curva no quadro em que muda o autorado — no mundo e no que o Auto Key lê —, e
/// sem curva o mundo corre o autorado.
#[test]
fn a_mao_ganha_a_curva_no_quadro_em_que_muda() {
    let (mut sim, _, mut ponte) = queda();
    let mut cond = ConducaoMundo::default();
    curva(&mut sim, m::GRAVIDADE_Y, 5.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!((lido(&mut sim, &ponte, m::GRAVIDADE_Y) - 5.0).abs() < 1e-6);
    // O painel muda o autorado (fim do quadro); a curva ainda diz 5 no seguinte.
    ponte.set_settings(PhysicsSettings {
        gravity_y: 2.0,
        ..ponte.settings()
    });
    curva(&mut sim, m::GRAVIDADE_Y, 5.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!(
        (ponte.settings_vivos().gravity_y - 2.0).abs() < 1e-6,
        "o mundo é da mão"
    );
    assert!(
        (lido(&mut sim, &ponte, m::GRAVIDADE_Y) - 2.0).abs() < 1e-6,
        "o Auto Key vê a mão"
    );
    // A chave nasceu: a curva diz 2 — e volta a conduzir.
    curva(&mut sim, m::GRAVIDADE_Y, 3.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!((ponte.settings_vivos().gravity_y - 3.0).abs() < 1e-6);
    // Sem curva: o autorado.
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!((ponte.settings_vivos().gravity_y - 2.0).abs() < 1e-6);
    assert!((lido(&mut sim, &ponte, m::GRAVIDADE_Y) - 2.0).abs() < 1e-6);
}

/// Todo campo lê o que se escreveu; cada grupo tem nome traduzido e único.
#[test]
fn cada_campo_do_mundo_le_o_que_escreveu() {
    let mut nomes = BTreeSet::new();
    for campo in m::CAMPOS {
        let mut s = PhysicsSettings::default();
        let antes = ler(&s, campo).expect("leitura");
        let alvo = if m::ADORMECE == campo {
            1.0 - antes
        } else {
            antes + 2.0
        };
        escrever(&mut s, campo, alvo);
        assert_eq!(ler(&s.clamped(), campo), Some(alvo), "campo {campo}");
        let rotulo = k(campo).info().expect("info").rotulo;
        let nome = ph2d_i18n::tr(rotulo);
        assert_ne!(nome, rotulo, "{campo} sem tradução");
        if campo != m::GRAVIDADE_Y {
            assert!(nomes.insert(nome), "dois grupos com o nome {nome}");
        }
    }
}

/// ⭐⭐ **A curva liga e desliga uma célula da matriz** (F41): o mundo VIVO muda e o autorado não;
/// publicar põe as 36 células do autorado; e no quadro em que a MÃO muda a célula, a mão ganha.
#[test]
fn a_curva_conduz_uma_celula_da_matriz_e_a_mao_lhe_ganha() {
    let (mut sim, _, mut ponte) = queda();
    let mut cond = ConducaoMundo::default();
    let (i, a, b) = celulas().nth(4).expect("célula");
    let c = cena(&sim).expect("cena");
    let celula = |sim: &mut SimWorld, v: f32| {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(c, kc(i), v);
    };
    let liga = |s: PhysicsSettings| LayerMatrix::from_rows(s.layer_matrix).collides(a, b);
    assert!(liga(ponte.settings()), "controlo: de fábrica tudo colide");
    publicar(&mut sim, &ponte);
    let lidos = sim.world().resource::<CanaisLidos>();
    assert_eq!(
        (0..m::CELULAS)
            .filter(|&j| lidos.valor(c, kc(j)) == Some(1.0))
            .count(),
        usize::from(m::CELULAS)
    );
    aplicar(&mut sim, &mut ponte, &mut cond);
    celula(&mut sim, 0.0);
    assert!(aplicar(&mut sim, &mut ponte, &mut cond), "o mundo mudou");
    assert!(
        !liga(ponte.settings_vivos()),
        "a curva desligou o par no mundo vivo"
    );
    assert!(liga(ponte.settings()), "o autorado não muda pela curva");
    // A mão desliga-o no autorado e a curva manda ligar: neste quadro, a mão.
    let mut s = ponte.settings();
    let mut mx = LayerMatrix::from_rows(s.layer_matrix);
    mx.set(a, b, false);
    s.layer_matrix = mx.rows();
    ponte.set_settings(s);
    celula(&mut sim, 1.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!(
        !liga(ponte.settings_vivos()),
        "a mão ganhou no quadro em que mudou"
    );
    celula(&mut sim, 1.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!(liga(ponte.settings_vivos()), "no seguinte, a curva");
}

/// ⭐⭐ **O clique do painel sobre uma célula que a curva esconde é a MÃO** (smoke do dono F41).
/// O autorado já vale LIGADO, a curva desliga o par, o painel mostra o vivo e o clique pede ligado
/// (`o_clique_na_matriz_vira_o_par_que_se_ve_sob_a_curva`): o autorado não muda, e mesmo assim o
/// mundo e o que o Auto Key lê têm de ficar com o que o artista escolheu — pela porta do painel
/// (`panel_bridge::settings_do_painel`), a mesma que o `dispatch` usa.
#[test]
fn o_clique_sob_a_curva_e_a_mao_mesmo_sem_mudar_o_autorado() {
    let (mut sim, _, mut ponte) = queda();
    let mut cond = ConducaoMundo::default();
    let (i, a, b) = celulas().nth(4).expect("célula");
    let c = cena(&sim).expect("cena");
    let celula = |sim: &mut SimWorld, v: f32| {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(c, kc(i), v);
    };
    let liga = |s: PhysicsSettings| LayerMatrix::from_rows(s.layer_matrix).collides(a, b);
    aplicar(&mut sim, &mut ponte, &mut cond);
    celula(&mut sim, 0.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!(!liga(ponte.settings_vivos()), "controlo: a curva desligou");
    assert!(liga(ponte.settings()), "controlo: o autorado diz ligado");
    // O painel vira o par que se vê: LIGADO, o que o autorado já era.
    painel(&mut ponte, &mut cond, |s| {
        let mut mx = LayerMatrix::from_rows(s.layer_matrix);
        mx.set(a, b, true);
        PhysicsSettings {
            layer_matrix: mx.rows(),
            ..s
        }
    });
    celula(&mut sim, 0.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!(
        liga(ponte.settings_vivos()),
        "a mão ganhou no quadro do clique"
    );
    publicar(&mut sim, &ponte);
    assert_eq!(
        sim.world().resource::<CanaisLidos>().valor(c, kc(i)),
        Some(1.0),
        "o Auto Key lê o que o artista escolheu, não a curva"
    );
}

/// O painel escreve `editar` com o mundo a mostrar o vivo — a porta do `dispatch`.
fn painel(
    ponte: &mut PhysicsBridge,
    cond: &mut ConducaoMundo,
    editar: impl Fn(PhysicsSettings) -> PhysicsSettings,
) {
    let visto = ponte.settings_vivos();
    let toque = ph2d_panel_physics::Toque {
        visto,
        pedido: editar(visto),
    };
    let autorado = editar(ponte.settings());
    crate::panel_bridge::settings_do_painel(ponte, cond, autorado, toque);
}

/// ⭐⭐ **Todo controlo do painel é a mão sob a curva, mesmo sem mudar o autorado** (F41, a família
/// da matriz): a caixa de adormecer, o slider inteiro dos sub-passos levado ao valor autorado, e o
/// *Reset to Defaults* sobre um autorado que já é o de fábrica. A curva esconde o autorado, o
/// controlo pede-o, e o mundo e o Auto Key ficam com o que o artista escolheu.
#[test]
fn todo_controlo_do_painel_e_a_mao_sob_a_curva() {
    type Editar = fn(PhysicsSettings) -> PhysicsSettings;
    let casos: [(&str, u16, f32, Editar); 3] = [
        // O painel lê o vivo (desligado pela curva) e pede `!vivo` aos dois.
        ("adormecer", m::ADORMECE, 0.0, |s| {
            s.with_sleep_enabled(true)
        }),
        ("sub-passos", m::SUBPASSOS, 9.0, |s| PhysicsSettings {
            substeps: PhysicsSettings::default().substeps,
            ..s
        }),
        ("reset", m::GRAVIDADE_Y, 3.0, |_| PhysicsSettings::default()),
    ];
    for (nome, campo, na_curva, editar) in casos {
        let (mut sim, _, mut ponte) = queda();
        let mut cond = ConducaoMundo::default();
        let autorado = ler(&ponte.settings(), campo).expect("campo");
        assert_ne!(
            autorado, na_curva,
            "{nome}: controlo — a curva esconde o autorado"
        );
        aplicar(&mut sim, &mut ponte, &mut cond);
        curva(&mut sim, campo, na_curva);
        aplicar(&mut sim, &mut ponte, &mut cond);
        assert_eq!(
            ler(&ponte.settings_vivos(), campo),
            Some(na_curva),
            "{nome}: a curva"
        );
        painel(&mut ponte, &mut cond, editar);
        assert_eq!(
            ler(&ponte.settings(), campo),
            Some(autorado),
            "{nome}: autorado igual"
        );
        curva(&mut sim, campo, na_curva);
        aplicar(&mut sim, &mut ponte, &mut cond);
        assert_eq!(
            ler(&ponte.settings_vivos(), campo),
            Some(autorado),
            "{nome}: a mão ganhou no quadro do clique"
        );
        assert_eq!(
            lido(&mut sim, &ponte, campo),
            autorado,
            "{nome}: o Auto Key lê a mão"
        );
    }
}

/// ⭐ **Mexer num campo não toca nos outros que a curva conduz.** A gravidade muda pelo painel com
/// a curva a desligar uma célula da matriz: a célula continua da curva. (Comparar o pedido com o vivo
/// — e não com o que o painel MOSTRAVA — marcava todas as células conduzidas como mão.)
#[test]
fn mexer_num_campo_nao_toca_nos_outros_que_a_curva_conduz() {
    let (mut sim, _, mut ponte) = queda();
    let mut cond = ConducaoMundo::default();
    let (i, a, b) = celulas().nth(4).expect("célula");
    let c = cena(&sim).expect("cena");
    let celula = |sim: &mut SimWorld| {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(c, kc(i), 0.0);
    };
    let liga = |s: PhysicsSettings| LayerMatrix::from_rows(s.layer_matrix).collides(a, b);
    aplicar(&mut sim, &mut ponte, &mut cond);
    celula(&mut sim);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!(!liga(ponte.settings_vivos()), "controlo: a curva desligou");
    painel(&mut ponte, &mut cond, |s| PhysicsSettings {
        gravity_y: 1.5,
        ..s
    });
    celula(&mut sim);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert_eq!(ponte.settings_vivos().gravity_y, 1.5, "a mão na gravidade");
    assert!(!liga(ponte.settings_vivos()), "a célula continua da curva");
}

/// As chaves gravadas nas 36 células de antes apontam para o mesmo par: o índice é o triângulo por `max`.
/// A tabela é a do painel de 8 camadas (F41), escrita à mão, para não provar a fórmula com ela própria.
#[test]
fn as_36_celulas_antigas_ficam_no_mesmo_par_e_o_canal_novo_chega_ao_mundo() {
    let mut esperado = Vec::new();
    for a in 0..8usize {
        for b in 0..=a {
            esperado.push((a, b));
        }
    }
    for (i, a, b) in celulas().take(36) {
        assert_eq!(esperado[usize::from(i)], (a, b), "o canal {i}");
    }
    assert_eq!(celulas().nth(31).map(|c| (c.1, c.2)), Some((7, 3)));
    let (i, a, b) = celulas().last().expect("células");
    assert_eq!((usize::from(i), a, b), (527, 31, 31));

    // Uma chave numa célula alta conduz o mundo vivo, sem tocar no autorado.
    let (mut sim, _, mut ponte) = queda();
    let mut cond = ConducaoMundo::default();
    let alta = celulas()
        .find(|c| (c.1, c.2) == (25, 20))
        .expect("a célula")
        .0;
    publicar(&mut sim, &ponte);
    let c = cena(&sim).expect("cena");
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(c, kc(alta), 0.0);
    assert!(aplicar(&mut sim, &mut ponte, &mut cond), "o mundo mudou");
    let vivo = LayerMatrix::from_rows(ponte.settings_vivos().layer_matrix);
    assert!(!vivo.collides(25, 20) && !vivo.collides(20, 25));
    assert!(LayerMatrix::from_rows(ponte.settings().layer_matrix).collides(25, 20));
}

/// Sonda de custo (`--ignored --nocapture`): um quadro do funil, com a Cena e as 528 células.
#[test]
#[ignore = "mede: corre à mão"]
fn mede_o_quadro_das_celulas() {
    use std::time::Instant;
    let (mut sim, _, mut ponte) = queda();
    let mut cond = ConducaoMundo::default();
    let c = cena(&sim).expect("cena");
    let mut lidos = CanaisLidos::default();
    let (mut pub36, mut pub528, mut funil) = (f64::MAX, f64::MAX, f64::MAX);
    for _ in 0..200 {
        let t = Instant::now();
        for (i, _, _) in celulas().take(36) {
            lidos.publicar(c, kc(i), 1.0);
        }
        pub36 = pub36.min(t.elapsed().as_secs_f64());
        let t = Instant::now();
        for (i, _, _) in celulas() {
            lidos.publicar(c, kc(i), 1.0);
        }
        pub528 = pub528.min(t.elapsed().as_secs_f64());
        let t = Instant::now();
        publicar(&mut sim, &ponte);
        let _ = aplicar(&mut sim, &mut ponte, &mut cond);
        funil = funil.min(t.elapsed().as_secs_f64());
    }
    println!(
        "[canais] publicar 36 = {:.1} us · 528 = {:.1} us · quadro inteiro (publicar + aplicar, 528) = {:.1} us · \
         PhysicsSettings = {} B · entrada dos mapas = {} B",
        pub36 * 1e6,
        pub528 * 1e6,
        funil * 1e6,
        size_of::<PhysicsSettings>(),
        size_of::<((u64, FieldKey), f32)>(),
    );
}
