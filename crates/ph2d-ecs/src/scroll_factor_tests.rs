//! Gates da LEI da paralaxe sem ledger nenhum — a [`saida_eixo`] e as guardas que a auditoria 26
//! pediu. Os gates da PONTE (o ledger, a captura, a condução) vivem na `ph2d-app-components`.

use super::*;
use crate::{ScrollMotion, meia_da_camada};

/// ⛔⛔ **A repetição fica presa à VISTA, e é congruente com a antiga módulo um ladrilho** (auditoria
/// 26, §1.1). ⚠️ A régua é `saída − c` (a posição no ECRÃ) — a que a memória de 22/09 gravou,
/// `|saída − autorada|`, é a grandeza ERRADA e ficava verde sobre o defeito.
#[test]
fn a_repeticao_fica_presa_a_vista_e_nao_ao_mundo() {
    let (k, tile, autorada) = (0.65_f32, 3.5_f32, 0.8_f32);
    let mut c = 0.0_f32;
    while c < 200.0 {
        let s = saida_eixo(k, autorada, c, c, 0.0, 1.0, Some(tile));
        let no_ecra = s - c;
        assert!(
            (no_ecra - autorada).abs() <= tile * 0.5 + 1e-3,
            "c = {c}: a camada saiu da vista ({no_ecra} contra a autorada {autorada})"
        );
        // A imagem de um padrão repetido é a MESMA: a diferença para a lei sem repetição é um
        // número inteiro de ladrilhos.
        let sem = autorada + (c * (1.0 - k));
        let n = (s - sem) / tile;
        assert!(
            (n - n.round()).abs() < 1e-3,
            "c = {c}: a correcção não é inteira ({n} ladrilhos)"
        );
        c += 0.37;
    }
}

/// ⭐ **O oráculo** (a tabela do cabeçalho da `ScrollRepeat`): com a câmera a varrer `0 → 768`, a
/// origem RELATIVA AO ECRÃ fica numa janela de um ladrilho. A janela dele não é a nossa (divergência
/// declarada); a LIMITAÇÃO é.
#[test]
fn a_origem_no_ecra_fica_numa_janela_de_um_ladrilho_como_no_oraculo() {
    let (k, tile, autorada) = (0.5_f32, 256.0_f32, -436.0_f32);
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for i in 0..=48 {
        let c = i as f32 * 16.0;
        let s = saida_eixo(k, autorada, c, c, 0.0, 1.0, Some(tile));
        lo = lo.min(s - c);
        hi = hi.max(s - c);
    }
    assert!(
        hi - lo <= tile + 1e-2,
        "a origem no ecrã variou {} ≥ um ladrilho",
        hi - lo
    );
}

/// ⛔⛔ **O dolly escala à volta do CENTRO DA VISTA** (auditoria 26, §1.4). O oráculo é a pinhole
/// escrita por extenso — uma fórmula diferente da que o código usa, que é o que a torna régua.
#[test]
fn o_dolly_escala_a_volta_do_centro_da_vista() {
    // A pinhole: plano do mundo em `z₀ = 1`, camada em `z = 1/k`, a câmera avança `δ`.
    let pinhole = |k: f32, a: f32, c: f32, delta: f32| {
        let (z0, z) = (1.0_f64, 1.0 / f64::from(k));
        let x = f64::from(a) / f64::from(k); // o ponto que, com a câmera na origem, se vê em `a`
        let d = f64::from(delta);
        (f64::from(c) + (x - f64::from(c)) * (z0 - d) / (z - d)) as f32
    };
    for (k, a, c, delta) in [
        (0.12_f32, 3.2_f32, 0.0_f32, 0.9_f32),
        (0.5, -2.0, 7.5, 0.5),
        (2.0, 1.0, -3.0, 0.3),
        (0.35, 1.2, 12.0, -1.0),
    ] {
        let esc = ScrollFactor::escala_do_dolly(k, delta).expect("a câmera não atravessa");
        let s = saida_eixo(k, a, c, c, 0.0, esc, None);
        let p = pinhole(k, a, c, delta);
        assert!(
            (s - p).abs() < 1e-4,
            "k {k} a {a} c {c} δ {delta}: {s} contra a pinhole {p}"
        );
    }
}

/// ⭐ **Sem repetição e sem dolly a saída é a de sempre AO BIT** — a paridade da W1 com o oráculo e
/// as W3/W4 dependem disto.
#[test]
fn sem_repeticao_nem_dolly_a_saida_e_a_de_sempre_ao_bit() {
    for (k, a, c, conf, o) in [
        (0.5_f32, 1.25_f32, 400.0_f32, 400.0_f32, 0.0_f32),
        (0.35, -7.0, 13.3, 9.0, 2.7),
        (-0.5, 0.1, -3.0, -3.0, -1.0),
    ] {
        let f = ScrollFactor { k: [k, k] };
        let d = f.deslocamento_confinado([c, c], [conf, conf])[0];
        let antiga = a + (d + o);
        let s = saida_eixo(k, a, c, conf, o, 1.0, None);
        assert_eq!(s.to_bits(), antiga.to_bits(), "k {k}: {s} contra {antiga}");
    }
}

/// ⛔ **`δ ≥ 1` é recusa, não uma escala a zero** — uma escala `0` escrita no `Transform` não tem
/// volta (auditoria 26, §3).
#[test]
fn a_camera_no_plano_focal_e_recusa() {
    assert!(ScrollFactor::escala_do_dolly(0.5, 1.0).is_none());
    assert!(ScrollFactor::escala_do_dolly(0.5, 1.5).is_none());
    assert!(ScrollFactor::escala_do_dolly(0.5, f32::NAN).is_none());
    // O controlo: dentro do domínio há resposta.
    assert!(ScrollFactor::escala_do_dolly(0.5, 0.9).is_some());
}

/// ⭐ **A meia-vista da camada devolve a MESMA faixa de conteúdo com e sem dolly** (auditoria 26,
/// §2.3). A régua é a faixa de conteúdo que a vista chega a mostrar enquanto a câmera percorre
/// tudo — calculada pela SAÍDA, não pela fórmula da meia-vista.
#[test]
fn com_dolly_a_cerca_mostra_a_mesma_faixa_de_conteudo() {
    let (min, max, h, k) = (-600.0_f32, 600.0_f32, 200.0_f32, 0.5_f32);
    let faixa = |delta: f32| {
        let esc = ScrollFactor::escala_do_dolly(k, delta).expect("dentro");
        let m = meia_da_camada(h, k, esc);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        let mut c = -2000.0_f32;
        while c <= 2000.0 {
            let conf = crate::confina_eixo(c, m, min, max);
            // O conteúdo `P` que cai nas bordas da vista `c ± h`: `c ± h = c + esc·(P − k·conf)`.
            lo = lo.min(k * conf - h / esc);
            hi = hi.max(k * conf + h / esc);
            c += 10.0;
        }
        (lo, hi)
    };
    let (lo0, hi0) = faixa(0.0);
    for delta in [0.3_f32, 0.6, -0.5] {
        let (lo, hi) = faixa(delta);
        assert!(
            (lo - lo0).abs() < 0.5 && (hi - hi0).abs() < 0.5,
            "δ {delta}: a vista mostra [{lo}, {hi}] contra [{lo0}, {hi0}] sem dolly"
        );
    }
    // O controlo: sem a meia-vista da camada a faixa ALARGA e a borda entra.
    let esc = ScrollFactor::escala_do_dolly(k, 0.6).expect("dentro");
    let conf = crate::confina_eixo(2000.0, h, min, max);
    assert!(
        k * conf + h / esc > hi0 + 1.0,
        "o controlo não contém o fenómeno"
    );
}

/// ⛔ **Uma deriva não-finita é inerte**, eixo a eixo (auditoria 26, §3).
#[test]
fn uma_deriva_nao_finita_e_inerte() {
    let m = ScrollMotion {
        velocity: [f32::NAN, 2.0],
    };
    assert_eq!(m.deslocamento(3.0), [0.0, 6.0]);
    assert!(!m.e_inerte(), "o eixo finito continua a andar");
    let so_nan = ScrollMotion {
        velocity: [f32::INFINITY, f32::NAN],
    };
    assert!(so_nan.e_inerte());
    assert_eq!(so_nan.deslocamento(3.0), [0.0, 0.0]);
}

/// ⭐⭐ **Gravar → abrir devolve os QUATRO componentes e o `dolly`** (auditoria 26, §3).
///
/// ⚠️ Os gates da lei correm todos num mundo que nunca atravessou um ficheiro; o que a auditoria
/// pediu é a ida-e-volta pela porta que o `.ph2dproj` usa — o snapshot, o postcard e o restauro,
/// com o **registo real** (`register_ecs_components`), que é onde um componente esquecido no
/// registo evapora **em silêncio** (o snapshot só leva o que o registo conhece).
///
/// **Mutações que devem sangrar:** tirar um dos quatro do registo da câmera · o `dolly` a não
/// viajar (um campo `#[serde(skip)]`).
#[test]
fn gravar_e_abrir_devolve_os_quatro_componentes_e_o_dolly() {
    use crate::scene::{
        ComponentRegistry, WorldSnapshot, register_ecs_components, snapshot_to_world,
        world_to_snapshot,
    };
    use crate::transform::{TransformPropagationState, WorklistBuf};
    use crate::{GameCamera, ScrollLimits, ScrollRepeat, SimWorld, Transform};

    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let mut sim = SimWorld::new();
    let factor = ScrollFactor { k: [0.35, 0.8] };
    let repete = ScrollRepeat { tile: [3.5, 0.0] };
    let cerca = ScrollLimits {
        min: [-30.0, -2.0],
        max: [30.0, 2.0],
    };
    let deriva = ScrollMotion {
        velocity: [0.35, -0.1],
    };
    let camera = GameCamera {
        dolly: 0.4,
        ..GameCamera::default()
    };
    sim.world_mut()
        .spawn((Transform::default(), factor, repete, cerca, deriva));
    sim.world_mut()
        .spawn((Transform::default(), camera.clone()));

    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut snap).expect("grava");
    let bytes = postcard::to_allocvec(&snap).expect("serializa");
    let lido: WorldSnapshot = postcard::from_bytes(&bytes).expect("le");

    let mut outro = SimWorld::new();
    snapshot_to_world(outro.world_mut(), &lido, &reg).expect("abre");
    let w = outro.world_mut();
    let fundos: Vec<_> = w
        .query::<(&ScrollFactor, &ScrollRepeat, &ScrollLimits, &ScrollMotion)>()
        .iter(w)
        .map(|(a, b, c, d)| (*a, *b, *c, *d))
        .collect();
    assert_eq!(fundos, vec![(factor, repete, cerca, deriva)]);
    let cams: Vec<GameCamera> = w.query::<&GameCamera>().iter(w).cloned().collect();
    assert_eq!(cams, vec![camera], "o dolly nao atravessou o ficheiro");
}

/// ⛔ **Um ladrilho `0` (ou não-finito) NÃO corrige** — a ausência de repetição é a omissão.
///
/// ⚠️ Pela porta do produto este caso já não chega aqui (a `saida_eixo` só envolve com `tile > 0`),
/// e foi por isso que a prova de mutação da guarda SOBREVIVIA quando apontava para o gate da ponte:
/// *uma guarda defensiva mede-se na porta que a tem*. Sem ela, `d − 0·(d/0).round()` é `NaN`.
#[test]
fn um_ladrilho_zero_ou_nao_finito_nao_corrige() {
    for d in [5.0_f32, 0.0, -3.25] {
        assert_eq!(
            crate::envolve_eixo(d, 0.0).to_bits(),
            d.to_bits(),
            "tile 0, d {d}"
        );
        assert_eq!(
            crate::envolve_eixo(d, f32::NAN).to_bits(),
            d.to_bits(),
            "tile NaN, d {d}"
        );
        assert_eq!(
            crate::envolve_eixo(d, -8.0).to_bits(),
            d.to_bits(),
            "tile < 0, d {d}"
        );
    }
    // O CONTROLO: com um ladrilho a sério a correcção acontece.
    assert_eq!(crate::envolve_eixo(300.0, 256.0), 44.0);
}
