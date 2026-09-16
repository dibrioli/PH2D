//! A cena do SCRIPT, headless: o que o dono vai ver, medido antes de ele o ver — e ALCANÇÁVEL
//! (a lição do #15: uma cena certa como dados pode ser impossível como gesto).

use std::sync::atomic::{AtomicU32, Ordering};

use ph2d_ecs::{Entity, Name, SimWorld, Transform, Visibility};
use ph2d_preview_drive::PreviewDrive;
use ph2d_render::Sprite;
use ph2d_script::{LuauScript, ScriptHost, ScriptValue};

use super::{BOB_LUAU, BOB_SIZE, BOB_X, BOB_Y, CENAS, FILE_NAME, LAMP_Y, TALL_AMPLITUDE, montar};

fn pasta() -> std::path::PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    std::env::temp_dir().join(format!(
        "ph2d_script_smoke_{}_{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ))
}

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    w.query::<(Entity, &Name)>()
        .iter(w)
        .find(|(_, n)| n.as_str() == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("`{nome}` não está na cena"))
}

fn montada() -> (SimWorld, std::path::PathBuf) {
    let dir = pasta();
    let mut sim = SimWorld::new();
    let m = montar(sim.world_mut(), 1, &dir).expect("monta");
    assert_eq!(m.nivel, 1);
    // ⭐ Quem abre escolhido é o «Bob (tall)» — o que tem a secção mais cheia.
    assert_eq!(
        sim.world()
            .get::<Name>(Entity::from_bits(m.escolhido))
            .map(Name::as_str),
        Some("Bob (tall)")
    );
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, dir)
}

#[test]
fn o_roteador_nunca_devolve_acima_do_tecto() {
    for nivel in 0..=3 {
        let mut sim = SimWorld::new();
        let n = montar(sim.world_mut(), nivel, &pasta())
            .expect("monta")
            .nivel;
        assert!(n >= 1 && n <= CENAS, "nível {nivel} devolveu {n}");
    }
}

#[test]
fn a_cena_escreve_o_ficheiro_e_os_tres_bonecos_apontam_para_ele() {
    let (mut sim, dir) = montada();
    let path = dir.join(FILE_NAME);
    assert_eq!(std::fs::read_to_string(&path).expect("lê"), BOB_LUAU);
    for nome in ["Bob", "Bob (tall)", "Bob (fast)"] {
        let e = por_nome(&mut sim, nome);
        let cfg = sim.world().get::<LuauScript>(e).expect("tem script");
        assert_eq!(cfg.source, path.to_string_lossy(), "{nome}");
    }
}

/// ⛔ **A lição do #15: o que tem script tem CORPO** — uma sprite visível, onde o dono carrega.
#[test]
fn todo_boneco_com_script_se_desenha_e_se_apanha_no_centro() {
    let (mut sim, _) = montada();
    for (i, nome) in ["Bob", "Bob (tall)", "Bob (fast)"].iter().enumerate() {
        let e = por_nome(&mut sim, nome);
        let w = sim.world();
        let s = w.get::<Sprite>(e).expect("tem corpo");
        assert!(s.size[0] > 0.0 && s.size[1] > 0.0, "{nome}");
        assert!(w.get::<Visibility>(e).is_none_or(|v| !v.hidden), "{nome}");
        let t = w.get::<Transform>(e).expect("pose");
        assert_eq!((t.translation.x, t.translation.y), (BOB_X[i], BOB_Y));
        assert_eq!(s.size, BOB_SIZE);
    }
    // E nenhum dos três se sobrepõe a outro: o dedo não tem desempate a fazer.
    assert!(BOB_X[1] - BOB_X[0] > BOB_SIZE[0] && BOB_X[2] - BOB_X[1] > BOB_SIZE[0]);
}

/// ⭐⭐⭐ **O §0 do plano, à vista** — o mesmo script, três comportamentos; e o grito acende a
/// lâmpada pela tabela de acções.
#[test]
fn tres_bonecos_tres_amplitudes_e_o_rapido_acende_a_lampada() {
    let (mut sim, _) = montada();
    let mut host = ScriptHost::new().expect("vm");
    let mut drive = PreviewDrive::default();
    let bobs = ["Bob", "Bob (tall)", "Bob (fast)"].map(|n| por_nome(&mut sim, n));
    let lampada = por_nome(&mut sim, "Lamp");
    let mut max_dy = [0.0f32; 3];
    let mut gritos = 0;
    let mut acesa_alguma_vez = false;
    for _ in 0..(60 * 4) {
        let f =
            crate::script_bridge::frame(&mut host, &mut sim, &mut drive, true, 1, 1.0 / 60.0, &[]);
        assert!(f.failed.is_empty(), "{:?}", f.failed);
        let nomes: Vec<&str> = f.emitted.iter().map(|(_, n)| n.as_str()).collect();
        gritos += nomes.len();
        let efeitos = ph2d_ecs::resolve_signal_actions(
            sim.world_mut(),
            &ph2d_tags::TagTree::default(),
            &nomes,
        );
        for fx in &efeitos {
            if let Some(mut v) = sim.world_mut().get_mut::<Visibility>(fx.target) {
                v.hidden = !v.hidden;
            }
        }
        acesa_alguma_vez |= sim
            .world()
            .get::<Visibility>(lampada)
            .is_some_and(|v| !v.hidden);
        drive.settle();
        for (i, &e) in bobs.iter().enumerate() {
            let y = sim.world().get::<Transform>(e).expect("pose").translation.y;
            max_dy[i] = max_dy[i].max((y - BOB_Y).abs());
        }
    }
    assert!(
        (max_dy[0] - 1.0).abs() < 0.05,
        "Bob segue o ficheiro: {max_dy:?}"
    );
    assert!(
        (f64::from(max_dy[1]) - TALL_AMPLITUDE).abs() < 0.1,
        "Bob (tall) sobe o dobro: {max_dy:?}"
    );
    // ⚠️ E a lâmpada nunca é tapada pelo boneco que a acende: o topo dele fica abaixo dela.
    assert!(BOB_Y + max_dy[2] + BOB_SIZE[1] * 0.5 < LAMP_Y - 0.4);
    assert!(
        (max_dy[2] - 1.0).abs() < 0.05,
        "Bob (fast) tem a amplitude do ficheiro: {max_dy:?}"
    );
    // speed 6 rad/s em 4 s ≈ 3,8 voltas ⇒ 3 ou 4 topos.
    assert!((3..=4).contains(&gritos), "gritos: {gritos}");
    assert!(acesa_alguma_vez, "a lâmpada nunca acendeu");
}

/// ⭐ **O órfão de propósito** chega ao painel com o que o script quer.
#[test]
fn o_bob_alto_mostra_o_orfao_height() {
    let (mut sim, _) = montada();
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    let e = por_nome(&mut sim, "Bob (tall)");
    let info = crate::script_inspector::build_info(&sim, Some(&host), e.to_bits(), 1, true)
        .expect("tem secção");
    assert_eq!(info.orphans.len(), 1);
    assert_eq!(info.orphans[0].name, "height");
    assert_eq!(info.orphans[0].wants, None);
    let amp = info
        .props
        .iter()
        .find(|p| p.name == "amplitude")
        .expect("declarada");
    assert!(amp.own);
}

/// ⭐⭐ **O ficheiro é VIVO** — mudar o default e gravar muda só quem não tem número próprio (Q1/Q2).
#[test]
fn mudar_o_default_no_ficheiro_muda_so_o_bob_que_o_segue() {
    let (mut sim, dir) = montada();
    let mut host = ScriptHost::new().expect("vm");
    let mut drive = PreviewDrive::default();
    let bob = por_nome(&mut sim, "Bob");
    let alto = por_nome(&mut sim, "Bob (tall)");
    let novo = BOB_LUAU.replace(
        r#"ph2d.property("amplitude", 1.0, { min = 0, max = 4 })"#,
        r#"ph2d.property("amplitude", 1.5, { min = 0, max = 4 })"#,
    );
    assert_ne!(novo, BOB_LUAU, "a substituição casou");
    // ⚠️ **A cena corre ANTES de o ficheiro mudar** — é o gesto do dono (gravar com a cena a
    // tocar), e é o que prova a RECARGA; gravar antes do primeiro quadro só provaria a carga.
    for _ in 0..60 {
        crate::script_bridge::frame(&mut host, &mut sim, &mut drive, true, 1, 1.0 / 60.0, &[]);
        drive.settle();
    }
    std::fs::write(dir.join(FILE_NAME), novo).expect("grava");
    let mut max_dy = [0.0f32; 2];
    for _ in 0..240 {
        crate::script_bridge::frame(&mut host, &mut sim, &mut drive, true, 1, 1.0 / 60.0, &[]);
        drive.settle();
        for (i, e) in [bob, alto].into_iter().enumerate() {
            let y = sim.world().get::<Transform>(e).expect("pose").translation.y;
            max_dy[i] = max_dy[i].max((y - BOB_Y).abs());
        }
    }
    assert!(
        (max_dy[0] - 1.5).abs() < 0.05,
        "o Bob seguiu o default novo: {max_dy:?}"
    );
    assert!(
        (f64::from(max_dy[1]) - TALL_AMPLITUDE).abs() < 0.1,
        "o alto guardou o dele: {max_dy:?}"
    );
    let cfg = sim.world().get::<LuauScript>(alto).expect("cfg");
    assert_eq!(
        cfg.own.get("amplitude"),
        Some(&ScriptValue::Number(TALL_AMPLITUDE))
    );
}
