//! A ponte do HUD contra o LEDGER de verdade — o que ela decide é se um placar entra no ficheiro.

use ph2d_core::Vec2;
use ph2d_ecs::{Fit, SimWorld, Transform, UiCanvas};
use ph2d_hud::View;
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

use super::{canvas_count, drive_canvases};

/// Uma vista `64 × 36` centrada em `(10, 4)`.
fn vista() -> View {
    View {
        center: [10.0, 4.0],
        half: [32.0, 18.0],
    }
}

fn cena(cfg: UiCanvas, pose: Transform) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn((cfg, pose)).id();
    (sim, e)
}

/// ⭐ O canvas cola-se à vista: a translação é o centro, a escala é a da lei.
#[test]
fn o_canvas_cola_se_a_vista_do_jogo() {
    let (mut sim, e) = cena(
        UiCanvas {
            ref_w: 32.0,
            ref_h: 18.0,
            fit: Fit::Keep,
        },
        Transform::default(),
    );
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_canvases(&mut sim, Some(vista()), &mut drive), 1);
    let t = *sim.world().get::<Transform>(e).expect("pose");
    assert_eq!(t.translation, Vec2::new(10.0, 4.0), "o centro da vista");
    assert_eq!(t.scale, Vec2::new(2.0, 2.0), "64/32 = 36/18 = 2");
}

/// ⛔⛔ **Sem câmera de jogo, NADA é conduzido** — o canvas fica onde o artista o pôs.
#[test]
fn sem_camera_de_jogo_o_canvas_fica_onde_o_artista_o_pos() {
    let autorada = Transform {
        translation: Vec2::new(-3.0, 7.0),
        scale: Vec2::new(1.5, 1.5),
        ..Transform::default()
    };
    let (mut sim, e) = cena(UiCanvas::default(), autorada);
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_canvases(&mut sim, None, &mut drive), 0);
    assert_eq!(
        *sim.world().get::<Transform>(e).expect("pose"),
        autorada,
        "sem vista não há nada para onde colar"
    );
    assert!(drive.is_empty(), "e ninguém foi declarado ao ledger");
    // O CONTROLO: com vista, a MESMA cena é conduzida.
    assert_eq!(drive_canvases(&mut sim, Some(vista()), &mut drive), 1);
}

/// ⛔⛔ **A pose fica no LEDGER como pré-visualização** — é isto que a mantém fora do ficheiro e do
/// `Ctrl+Z`. Sem esta linha, «o HUD acompanhou a câmera» seria um passo de undo por quadro.
#[test]
fn a_pose_conduzida_e_previsualizacao_e_o_autorado_sobrevive() {
    let autorada = Transform {
        translation: Vec2::new(-3.0, 7.0),
        ..Transform::default()
    };
    let (mut sim, e) = cena(UiCanvas::default(), autorada);
    let mut drive = PreviewDrive::default();
    drive_canvases(&mut sim, Some(vista()), &mut drive);
    assert_eq!(
        drive.authored(e.to_bits(), Driver::CanvasPose),
        Some(Driven::CanvasPose(autorada)),
        "o ledger guarda o valor AUTORADO, que é o que a captura repõe"
    );
}

/// A rotação e o *skew* autorados SOBREVIVEM — só a posição e o tamanho são derivados.
#[test]
fn a_inclinacao_autorada_sobrevive_ao_passe() {
    let autorada = Transform {
        rotation: 0.35,
        skew_x: 0.1,
        ..Transform::default()
    };
    let (mut sim, e) = cena(UiCanvas::default(), autorada);
    let mut drive = PreviewDrive::default();
    drive_canvases(&mut sim, Some(vista()), &mut drive);
    let t = *sim.world().get::<Transform>(e).expect("pose");
    assert_eq!((t.rotation, t.skew_x), (0.35, 0.1));
    assert_ne!(
        t.translation, autorada.translation,
        "o CONTROLO: a pose MUDOU"
    );
}

/// Uma caixa impossível não conduz nada — e o canvas continua a existir para o painel a acusar.
#[test]
fn uma_caixa_de_lado_zero_nao_conduz_e_continua_a_contar() {
    let (mut sim, e) = cena(
        UiCanvas {
            ref_w: 0.0,
            ref_h: 18.0,
            fit: Fit::Keep,
        },
        Transform::default(),
    );
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_canvases(&mut sim, Some(vista()), &mut drive), 0);
    assert_eq!(
        *sim.world().get::<Transform>(e).expect("pose"),
        Transform::default(),
        "uma escala infinita levaria o HUD para fora de qualquer vista, em silêncio"
    );
    assert_eq!(canvas_count(&mut sim), 1, "o painel tem de o poder acusar");
}

/// ⚠️ **A linha do meio da tabela do ledger:** num quadro em que a vista não mudou, o motor
/// continua a conduzir — e o passe tem de o DECLARAR, senão o `settle` promove a pose da corrida a
/// documento.
#[test]
fn um_quadro_sem_mudanca_continua_a_declarar_a_conducao() {
    let (mut sim, e) = cena(UiCanvas::default(), Transform::default());
    let mut drive = PreviewDrive::default();
    drive_canvases(&mut sim, Some(vista()), &mut drive);
    drive.settle();
    // 2.º quadro, vista idêntica ⇒ a pose já é a certa e nada se mexe.
    assert_eq!(drive_canvases(&mut sim, Some(vista()), &mut drive), 1);
    drive.settle();
    assert!(
        drive.drives(e.to_bits()),
        "⛔ sem a declaração do quadro parado, o `settle` esqueceria o condutor e a pose \
         conduzida viraria documento"
    );
}

// ── QUAL botão o dedo tocou (`botao_sob_o_cursor`) ────────────────────────────────────────────

/// Monta `(mundo, botão elegível, filho do botão, forma solta, botão desligado)`.
fn cena_de_botoes() -> (
    SimWorld,
    ph2d_ecs::Entity,
    ph2d_ecs::Entity,
    ph2d_ecs::Entity,
    ph2d_ecs::Entity,
) {
    let mut sim = SimWorld::new();
    let botao = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_ecs::UiButton {
                signal: "disparar".into(),
                disabled: false,
            },
        ))
        .id();
    let rotulo = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::ChildOf(botao)))
        .id();
    let solta = sim.world_mut().spawn(Transform::IDENTITY).id();
    let desligado = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_ecs::UiButton {
                signal: "cinzento".into(),
                disabled: true,
            },
        ))
        .id();
    (sim, botao, rotulo, solta, desligado)
}

/// ⭐⭐⭐ **O caso que fez esta porta existir: o SEGUNDO candidato ganha quando o primeiro não é
/// um botão.**
///
/// ⚠️ **É esta a diferença entre a lei e a regra ingénua** (*«o primeiro candidato»*): com ela, um
/// botão feito de **sprite** por baixo de qualquer forma vectorial ficava inalcançável, e o dono
/// leria isso como *«o botão não funciona»*.
///
/// **Mutações que devem sangrar:** `for` → olhar só o primeiro · `continue` → `return None`.
#[test]
fn o_primeiro_candidato_que_sobe_ate_um_botao_ganha() {
    let (sim, botao, _, solta, _) = cena_de_botoes();
    let w = sim.world();
    assert_eq!(
        super::botao_sob_o_cursor(w, [solta.to_bits(), botao.to_bits()]),
        Some(botao),
        "uma forma que nao e' botao NAO pode engolir o candidato seguinte"
    );
    // ⚠️ **O CONTROLO:** sozinha, a forma solta não devolve botão nenhum — sem esta metade o gate
    // acima ficaria verde sobre uma lei que devolvesse sempre o último candidato.
    assert_eq!(super::botao_sob_o_cursor(w, [solta.to_bits()]), None);
}

/// ⚠️ **Um botão INELEGÍVEL também não bloqueia o seguinte** — *um botão desligado é decoração, e
/// decoração não engole um clique que era de outra coisa*.
#[test]
fn um_botao_desligado_nao_engole_o_candidato_seguinte() {
    let (sim, botao, _, _, desligado) = cena_de_botoes();
    let w = sim.world();
    assert_eq!(super::botao_sob_o_cursor(w, [desligado.to_bits()]), None);
    assert_eq!(
        super::botao_sob_o_cursor(w, [desligado.to_bits(), botao.to_bits()]),
        Some(botao)
    );
}

/// ⭐ **A subida da cadeia continua a valer** — o dedo toca o RÓTULO e o clique é do botão.
/// ⚠️ E as duas leis compõem: um rótulo é o candidato, e ele sobe.
#[test]
fn o_dedo_no_rotulo_e_um_clique_no_botao() {
    let (sim, botao, rotulo, _, _) = cena_de_botoes();
    assert_eq!(
        super::botao_sob_o_cursor(sim.world(), [rotulo.to_bits()]),
        Some(botao)
    );
}

/// ⚠️ **Sem candidatos, ninguém** — o piso de população desta família: um iterador vazio satisfaz
/// o laço em silêncio, e sem esta linha uma lei que devolvesse o primeiro botão do MUNDO passaria.
#[test]
fn sem_candidatos_nao_ha_botao() {
    let (sim, _, _, _, _) = cena_de_botoes();
    assert_eq!(super::botao_sob_o_cursor(sim.world(), []), None);
}
