//! A ponte da PARALAXE contra o LEDGER de verdade, e contra a TABELA MEDIDA no oráculo.
//!
//! ⚠️ **A tabela do `o_declive_e_um_menos_k` não é uma escolha nossa** — ela foi medida no binário
//! da Godot 4.7.2 (MIT), pela sonda versionada
//! [`godot_parallax_probe.gd`](../../../docs/Components/ferramentas/godot_parallax_probe.gd), e a
//! pesquisa está em [`23_pesquisa_paralaxe.md`](../../../docs/Components/23_pesquisa_paralaxe.md).
//! ⛔ **O `0,5` não discrimina** (`1 − 0,5 = 0,5`): quem decide são o `0`, o `1` e o `2`.

use ph2d_core::Vec2;
use ph2d_ecs::{Fit, ScrollFactor, SimWorld, Transform, UiCanvas};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

use super::{drive_parallax, parallax_count};

fn cena(k: [f32; 2], pose: Transform) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn((ScrollFactor { k }, pose)).id();
    (sim, e)
}

fn pose_em(x: f32, y: f32) -> Transform {
    Transform {
        translation: Vec2::new(x, y),
        ..Transform::default()
    }
}

/// Corre um quadro com a câmera em `centro` e devolve a translação resultante.
fn com_camera(sim: &mut SimWorld, drive: &mut PreviewDrive, centro: [f32; 2]) -> Vec2 {
    drive_parallax(sim, Some(centro), drive);
    let e = sim
        .world_mut()
        .query::<(ph2d_ecs::Entity, &ScrollFactor)>()
        .iter(sim.world())
        .map(|(e, _)| e)
        .next()
        .expect("um objecto");
    sim.world().get::<Transform>(e).expect("pose").translation
}

/// ⭐⭐⭐ **A LEI, contra a tabela MEDIDA no alvo:** o declive da pose é `1 − k`.
///
/// ⚠️ A régua é o DECLIVE (Δpose ÷ Δcâmera) e não uma posição absoluta, pela mesma razão que a
/// sonda teve de ser reescrita: uma posição absoluta não distingue *«a lei não fez nada»* de
/// *«a minha medição não a disparou»*.
#[test]
fn o_declive_e_um_menos_k() {
    for (k, declive) in [(0.0_f32, 1.0_f32), (0.25, 0.75), (0.5, 0.5), (1.0, 0.0), (2.0, -1.0)] {
        let (mut sim, _) = cena([k, k], pose_em(0.0, 0.0));
        let mut drive = PreviewDrive::default();
        let a = com_camera(&mut sim, &mut drive, [-200.0, 0.0]);
        let b = com_camera(&mut sim, &mut drive, [200.0, 0.0]);
        let medido = (b.x - a.x) / 400.0;
        assert!(
            (medido - declive).abs() < 1e-5,
            "k = {k}: declive {medido} contra {declive} (a tabela do alvo)"
        );
    }
}

/// ⭐ **Por EIXO**, que é a lei do alvo e o caso das nuvens que correm de lado e mal sobem.
#[test]
fn os_dois_eixos_sao_independentes() {
    let (mut sim, _) = cena([0.2, 0.8], pose_em(0.0, 0.0));
    let mut drive = PreviewDrive::default();
    let p = com_camera(&mut sim, &mut drive, [100.0, 100.0]);
    assert!((p.x - 80.0).abs() < 1e-4, "x: 100 · (1 − 0,2) = 80, deu {}", p.x);
    assert!((p.y - 20.0).abs() < 1e-4, "y: 100 · (1 − 0,8) = 20, deu {}", p.y);
}

/// ⭐⭐⭐ **`k = 1` não escreve um bit e NÃO se declara** — anexar o componente e não lhe tocar deixa
/// a cena byte-idêntica, e é isso que faz a omissão desta wave ser inerte.
///
/// ⚠️ A segunda metade é a que importa: sem ela, toda cena com o componente passaria a ter uma
/// entrada viva no ledger, e a captura pagaria uma varredura por nada.
#[test]
fn o_neutro_nao_escreve_nem_declara() {
    let autorada = pose_em(-3.0, 7.0);
    // ⚠️⚠️ **O `k` é o LITERAL e nunca `ScrollFactor::NEUTRO`.** A 1.ª redacção lia a constante
    // que este gate existe para medir: mutá-la para `[0,0]` movia os DOIS lados e a prova
    // SOBREVIVEU — o gate auto-referente que esta casa já pagou na arma (a barra derivada da const
    // que a cena lê). Quem afirma o valor da constante é o irmão abaixo, pelo EFEITO dela.
    let (mut sim, e) = cena([1.0, 1.0], autorada);
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_parallax(&mut sim, Some([400.0, 300.0]), &mut drive), 0);
    assert_eq!(
        *sim.world().get::<Transform>(e).expect("pose"),
        autorada,
        "um objecto do mundo não se desloca"
    );
    assert!(drive.is_empty(), "e ninguém foi declarado ao ledger");
    // O CONTROLO: a MESMA cena com `k` de fundo É conduzida.
    let (mut sim2, _) = cena([0.5, 0.5], autorada);
    let mut drive2 = PreviewDrive::default();
    assert_eq!(drive_parallax(&mut sim2, Some([400.0, 300.0]), &mut drive2), 1);
}

/// ⛔⛔ **Sem câmera de jogo, NADA é conduzido** — o fundo fica onde o artista o pôs.
#[test]
fn sem_camera_de_jogo_o_fundo_fica_onde_o_artista_o_pos() {
    let autorada = pose_em(-3.0, 7.0);
    let (mut sim, e) = cena([0.25, 0.25], autorada);
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_parallax(&mut sim, None, &mut drive), 0);
    assert_eq!(*sim.world().get::<Transform>(e).expect("pose"), autorada);
    assert!(drive.is_empty());
    // O CONTROLO: com vista, a MESMA cena é conduzida.
    assert_eq!(drive_parallax(&mut sim, Some([100.0, 0.0]), &mut drive), 1);
}

/// ⛔⛔ **A pose fica no LEDGER como pré-visualização** — é isto que a mantém fora do ficheiro e do
/// `Ctrl+Z`. Sem esta linha, panhar a câmera num fundo seria um passo de undo por quadro.
#[test]
fn a_pose_deslocada_e_previsualizacao_e_o_autorado_sobrevive() {
    let autorada = pose_em(-3.0, 7.0);
    let (mut sim, e) = cena([0.25, 0.25], autorada);
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some([400.0, 0.0]), &mut drive);
    assert_eq!(
        drive.authored(e.to_bits(), Driver::ParallaxPose),
        Some(Driven::ParallaxPose(autorada)),
        "o ledger guarda o valor AUTORADO, que é o que a captura repõe"
    );
}

/// ⭐⭐⭐ **O caso que decide se esta ponte está CERTA: o artista ARRASTA o fundo.**
///
/// A lei genérica do ledger tomaria a pose DESLOCADA como o autorado novo, e o fundo saltaria
/// `centro · (1 − k)` no quadro seguinte. A ponte reconhece a própria escrita e RECUPERA o
/// autorado subtraindo o mesmo deslocamento.
///
/// ⚠️ **A régua é o DELTA**: o artista arrastou `+5`, logo o autorado tem de andar `+5` — e a pose
/// vista tem de ficar onde o dedo a largou, sem saltar.
#[test]
fn arrastar_o_fundo_move_o_autorado_pelo_mesmo_delta() {
    let autorada = pose_em(0.0, 0.0);
    let (mut sim, e) = cena([0.5, 0.5], autorada);
    let mut drive = PreviewDrive::default();

    // Um quadro com a câmera longe da origem: o fundo desloca-se 200.
    let vista = [400.0, 0.0];
    let p0 = com_camera(&mut sim, &mut drive, vista);
    assert!((p0.x - 200.0).abs() < 1e-4, "400 · (1 − 0,5) = 200, deu {}", p0.x);

    // O artista arrasta +5 na tela.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation.x += 5.0;
    }

    // O quadro seguinte, com a MESMA câmera: nada pode saltar.
    let p1 = com_camera(&mut sim, &mut drive, vista);
    assert!(
        (p1.x - (p0.x + 5.0)).abs() < 1e-4,
        "a pose vista fica onde o dedo a largou: {} contra {}",
        p1.x,
        p0.x + 5.0
    );
    let Some(Driven::ParallaxPose(novo)) = drive.authored(e.to_bits(), Driver::ParallaxPose) else {
        panic!("o ledger tem de continuar a conduzir");
    };
    assert!(
        (novo.translation.x - 5.0).abs() < 1e-4,
        "e o AUTORADO andou o mesmo delta: {} contra 5,0",
        novo.translation.x
    );
}

/// ⛔⛔ **Quem tem `UiCanvas` fica de fora** — a raiz de um HUD já é conduzida pela ponte dele, e
/// dois motores sobre o mesmo `Transform` escreveriam um por cima do outro.
#[test]
fn um_hud_nao_e_tocado_por_esta_ponte() {
    let autorada = pose_em(1.0, 2.0);
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            ScrollFactor { k: [0.5, 0.5] },
            UiCanvas {
                ref_w: 32.0,
                ref_h: 18.0,
                fit: Fit::Keep,
            },
            autorada,
        ))
        .id();
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_parallax(&mut sim, Some([400.0, 0.0]), &mut drive), 0);
    assert_eq!(*sim.world().get::<Transform>(e).expect("pose"), autorada);
    // O CONTROLO: o mesmo objecto SEM o canvas é conduzido.
    let (mut sim2, _) = cena([0.5, 0.5], autorada);
    let mut drive2 = PreviewDrive::default();
    assert_eq!(drive_parallax(&mut sim2, Some([400.0, 0.0]), &mut drive2), 1);
    assert_eq!(parallax_count(&mut sim2), 1);
}

/// ⭐ **A rotação e a escala autoradas SOBREVIVEM** — esta ponte só escreve a translação.
#[test]
fn so_a_translacao_e_derivada() {
    let autorada = Transform {
        translation: Vec2::new(0.0, 0.0),
        rotation: 0.35,
        scale: Vec2::new(2.0, 3.0),
        skew_x: 0.1,
        ..Transform::default()
    };
    let (mut sim, e) = cena([0.5, 0.5], autorada);
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some([400.0, 0.0]), &mut drive);
    let t = *sim.world().get::<Transform>(e).expect("pose");
    assert_eq!(t.rotation, 0.35);
    assert_eq!(t.scale, Vec2::new(2.0, 3.0));
    assert_eq!(t.skew_x, 0.1);
}

/// ⭐⭐⭐ **O NEUTRO declarado é a IDENTIDADE da lei** — e é medido pelo EFEITO, nunca comparando
/// dois literais.
///
/// ⚠️ `ScrollFactor::NEUTRO` diz *«este objecto é do MUNDO: ele anda com a câmera, logo não se
/// desloca»*. Um gate que escrevesse `assert_eq!(NEUTRO, [1.0, 1.0])` afirmaria que a constante é
/// ela própria; este passa-a pela lei e exige o **ponto fixo**, com o `Default` ao lado — se os
/// dois divergirem, um objecto acabado de criar deixa de ser do mundo.
#[test]
fn o_neutro_declarado_e_a_identidade_da_lei() {
    let autorada = [-3.0_f32, 7.0];
    for centro in [[0.0_f32, 0.0], [400.0, -250.0], [-1.5, 0.25]] {
        let neutro = ScrollFactor {
            k: ScrollFactor::NEUTRO,
        };
        assert_eq!(
            neutro.desloca(autorada, centro),
            autorada,
            "o NEUTRO desloca com a vista em {centro:?}: ele deixou de querer dizer \
             «este objecto e' do MUNDO»"
        );
        assert_eq!(
            ScrollFactor::default().desloca(autorada, centro),
            autorada,
            "um `ScrollFactor` acabado de criar desloca: o componente anexado e nao tocado deixa \
             de ser inerte"
        );
    }
    // O CONTROLO: um `k` de fundo move-se — sem ele as asserções acima passariam sobre uma lei
    // que nunca desloca nada.
    assert_ne!(
        ScrollFactor { k: [0.5, 0.5] }.desloca(autorada, [400.0, 0.0]),
        autorada
    );
}

/// ⭐⭐⭐ **A REFERÊNCIA é a ORIGEM DO MUNDO, e não a peça** — o gate que separa esta lei da forma
/// que o Flip usa.
///
/// # ⛔⛔ Porque ele precisou de existir: a fixtura estava no ponto NEUTRO da referência
///
/// As outras provas autoram o fundo em `(0, 0)`, e ali `autorada + centro·(1−k)` e
/// `autorada + (centro − autorada)·(1−k)` dão **o mesmo número** — a mutação que troca uma pela
/// outra SOBREVIVEU a seis gates verdes. *Um corpus na origem não testa de onde a lei mede.*
///
/// ⚠️ **A régua é o DESLOCAMENTO de duas peças com o MESMO `k` e poses autoradas diferentes:** na
/// nossa lei ele é o mesmo para as duas (é função só da vista), e na forma do Flip ele cresce com
/// a distância da peça à câmera. É isso que faz um fundo autorado num canto ser **TELEPORTADO**
/// para o centro da vista, que é o defeito que o cabeçalho da lei nomeia.
#[test]
fn o_deslocamento_nao_depende_da_pose_autorada() {
    let mut sim = SimWorld::default();
    let k = [0.4_f32, 0.4];
    let a = pose_em(-50.0, 30.0);
    let b = pose_em(120.0, -80.0);
    sim.world_mut().spawn((ScrollFactor { k }, a));
    sim.world_mut().spawn((ScrollFactor { k }, b));
    let mut drive = PreviewDrive::default();
    let centro = [400.0_f32, -250.0];
    assert_eq!(drive_parallax(&mut sim, Some(centro), &mut drive), 2);

    let poses: Vec<(Transform, Transform)> = {
        let world = sim.world_mut();
        world
            .query::<(ph2d_ecs::Entity, &Transform)>()
            .iter(world)
            .map(|(e, t)| {
                let Some(Driven::ParallaxPose(memo)) =
                    drive.authored(e.to_bits(), Driver::ParallaxPose)
                else {
                    panic!("as duas tinham de ser conduzidas");
                };
                (memo, *t)
            })
            .collect()
    };
    let mut deltas: Vec<Vec2> = poses
        .iter()
        .map(|(memo, vivo)| vivo.translation - memo.translation)
        .collect();
    assert_eq!(deltas.len(), 2);
    let d1 = deltas.pop().expect("duas");
    let d0 = deltas.pop().expect("duas");
    // ⚠️ **A barra não é `assert_eq!` e o vale está MEDIDO:** os dois deslocamentos são
    // DERIVADOS (`vivo − autorado`) e `f32` perde um bit em magnitudes diferentes — medido,
    // `240,00002` contra `240,0`, ou seja **`2e-5`**. O defeito que este gate existe para apanhar
    // vale `(120 − (−50)) · 0,6 = 102` nesta fixtura: **seis ordens de grandeza** de vale, e a
    // barra de `1e-2` fica no meio dele.
    let desvio = (d0 - d1).length();
    assert!(
        desvio < 1e-2,
        "o deslocamento depende de ONDE a peca foi autorada (desvio {desvio}): a referencia \
         deixou de ser a origem do mundo, e um fundo posto num canto e' teleportado para o centro \
         da vista"
    );
    // E o número: `centro · (1 − k)`, que é o que a tabela do alvo diz.
    assert!((d0.x - centro[0] * 0.6).abs() < 1e-3, "dx = {}", d0.x);
    assert!((d0.y - centro[1] * 0.6).abs() < 1e-3, "dy = {}", d0.y);

    // ⭐ **A metade que o artista sente:** com a câmera na ORIGEM nada se desloca, seja onde for
    // que ele pôs o fundo. Na forma do Flip a peça saltaria para `autorada · k`.
    let mut sim2 = SimWorld::default();
    sim2.world_mut().spawn((ScrollFactor { k }, a));
    let mut drive2 = PreviewDrive::default();
    drive_parallax(&mut sim2, Some([0.0, 0.0]), &mut drive2);
    let world = sim2.world_mut();
    let e = world
        .query_filtered::<ph2d_ecs::Entity, bevy_ecs::prelude::With<ScrollFactor>>()
        .iter(world)
        .next()
        .expect("a peca");
    assert_eq!(
        *sim2.world().get::<Transform>(e).expect("pose"),
        a,
        "com a camera na origem o fundo saiu do sitio onde o artista o pos"
    );
}

/// ⭐⭐⭐ **O DESLOCAMENTO NÃO DEPENDE DO ÂNGULO** — a §4.4 da pesquisa, e a única das leis do alvo
/// que eu escrevi ao contrário antes de a medir.
///
/// # ⚠️ A nota que esta prova corrige
///
/// A 1.ª redacção da pesquisa dizia que *«o alvo ignorar a rotação da câmera é uma limitação»*.
/// **É FALSO, e a física decide:** a paralaxe nasce da TRANSLAÇÃO — dois planos a profundidades
/// diferentes separam-se porque o olho ANDA. Uma câmera que só **roda** vê os dois planos girarem
/// juntos e não produz paralaxe nenhuma; deslocar por um ângulo seria inventar um efeito que
/// nenhuma óptica tem.
///
/// ⚠️ **Do lado da lei isto é estrutural** (a `desloca` recebe um centro e mais nada), e é por isso
/// que a régua tem de ser o PRODUTO: a peça é rodada e escalada, e o deslocamento medido é o mesmo
/// ao bit. Sem esta prova, «não depende do ângulo» é uma frase num cabeçalho.
#[test]
fn o_deslocamento_nao_depende_do_angulo_da_peca() {
    let k = [0.3_f32, 0.3];
    let centro = [400.0_f32, -250.0];
    let base = pose_em(0.0, 0.0);
    let mut deltas = Vec::new();
    for (rotation, scale) in [
        (0.0_f32, Vec2::new(1.0, 1.0)),
        (0.9, Vec2::new(1.0, 1.0)),
        (-2.4, Vec2::new(3.0, 0.25)),
    ] {
        let (mut sim, e) = cena(
            k,
            Transform {
                rotation,
                scale,
                ..base
            },
        );
        let mut drive = PreviewDrive::default();
        drive_parallax(&mut sim, Some(centro), &mut drive);
        let t = *sim.world().get::<Transform>(e).expect("pose");
        deltas.push(t.translation - base.translation);
    }
    assert_eq!(
        deltas[0], deltas[1],
        "rodar a peca mudou o deslocamento: a paralaxe passou a ler um angulo, e ela nasce da \
         TRANSLACAO — uma camera que so' roda nao produz paralaxe nenhuma"
    );
    assert_eq!(
        deltas[0], deltas[2],
        "escalar a peca mudou o deslocamento: a lei passou a ler o tamanho dela"
    );
    // O CONTROLO: o deslocamento não é zero — sem ele as duas asserções acima passariam sobre uma
    // lei que nunca desloca nada.
    assert!(deltas[0].length() > 1.0, "delta = {:?}", deltas[0]);
}
