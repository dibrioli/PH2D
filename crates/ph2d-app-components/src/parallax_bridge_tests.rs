//! A ponte da PARALAXE contra o LEDGER de verdade, e contra a TABELA MEDIDA no oráculo.
//!
//! ⚠️ **A tabela do `o_declive_e_um_menos_k` não é uma escolha nossa** — ela foi medida no binário
//! da Godot 4.7.2 (MIT), pela sonda versionada
//! [`godot_parallax_probe.gd`](../../../docs/Components/ferramentas/godot_parallax_probe.gd), e a
//! pesquisa está em [`23_pesquisa_paralaxe.md`](../../../docs/Components/23_pesquisa_paralaxe.md).
//! ⛔ **O `0,5` não discrimina** (`1 − 0,5 = 0,5`): quem decide são o `0`, o `1` e o `2`.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Fit, ScrollFactor, ScrollLimits, ScrollMotion, ScrollRepeat, SimWorld, Transform, UiCanvas,
};

/// ⚠️ **A meia-vista de quem NÃO confina.** Ela entrou na assinatura com a W3 e é irrelevante para
/// as leis da W1 e da W2 — escrevê-la por um nome diz isso, e um `[0.0, 0.0]` solto em vinte sítios
/// leria-se como um número que importa.
pub(super) const SEM_LIMITE: [f32; 2] = [0.0, 0.0];
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

/// ⚠️ **O instante em que a deriva da W4 é INERTE.** Toda a bancada das W1–W3 corre aqui, e é isso
/// que mantém aquelas leis medidas SOZINHAS — `velocidade × 0` é zero seja qual for a velocidade.
pub(super) const PARADO: f64 = 0.0;

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
    drive_parallax(sim, Some((centro, SEM_LIMITE)), PARADO, drive);
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
    assert_eq!(drive_parallax(&mut sim, Some(([400.0, 300.0], SEM_LIMITE)), PARADO, &mut drive), 0);
    assert_eq!(
        *sim.world().get::<Transform>(e).expect("pose"),
        autorada,
        "um objecto do mundo não se desloca"
    );
    assert!(drive.is_empty(), "e ninguém foi declarado ao ledger");
    // O CONTROLO: a MESMA cena com `k` de fundo É conduzida.
    let (mut sim2, _) = cena([0.5, 0.5], autorada);
    let mut drive2 = PreviewDrive::default();
    assert_eq!(drive_parallax(&mut sim2, Some(([400.0, 300.0], SEM_LIMITE)), PARADO, &mut drive2), 1);
}

/// ⛔⛔ **Sem câmera de jogo, NADA é conduzido** — o fundo fica onde o artista o pôs.
#[test]
fn sem_camera_de_jogo_o_fundo_fica_onde_o_artista_o_pos() {
    let autorada = pose_em(-3.0, 7.0);
    let (mut sim, e) = cena([0.25, 0.25], autorada);
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_parallax(&mut sim, None, PARADO, &mut drive), 0);
    assert_eq!(*sim.world().get::<Transform>(e).expect("pose"), autorada);
    assert!(drive.is_empty());
    // O CONTROLO: com vista, a MESMA cena é conduzida.
    assert_eq!(drive_parallax(&mut sim, Some(([100.0, 0.0], SEM_LIMITE)), PARADO, &mut drive), 1);
}

/// ⛔⛔ **A pose fica no LEDGER como pré-visualização** — é isto que a mantém fora do ficheiro e do
/// `Ctrl+Z`. Sem esta linha, panhar a câmera num fundo seria um passo de undo por quadro.
#[test]
fn a_pose_deslocada_e_previsualizacao_e_o_autorado_sobrevive() {
    let autorada = pose_em(-3.0, 7.0);
    let (mut sim, e) = cena([0.25, 0.25], autorada);
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some(([400.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
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
    assert_eq!(drive_parallax(&mut sim, Some(([400.0, 0.0], SEM_LIMITE)), PARADO, &mut drive), 0);
    assert_eq!(*sim.world().get::<Transform>(e).expect("pose"), autorada);
    // O CONTROLO: o mesmo objecto SEM o canvas é conduzido.
    let (mut sim2, _) = cena([0.5, 0.5], autorada);
    let mut drive2 = PreviewDrive::default();
    assert_eq!(drive_parallax(&mut sim2, Some(([400.0, 0.0], SEM_LIMITE)), PARADO, &mut drive2), 1);
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
    drive_parallax(&mut sim, Some(([400.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
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
            neutro.deslocamento(centro),
            [0.0, 0.0],
            "o NEUTRO desloca com a vista em {centro:?}: ele deixou de querer dizer \
             «este objecto e' do MUNDO»"
        );
        assert_eq!(
            ScrollFactor::default().deslocamento(centro),
            [0.0, 0.0],
            "um `ScrollFactor` acabado de criar desloca: o componente anexado e nao tocado deixa \
             de ser inerte"
        );
    }
    // O CONTROLO: um `k` de fundo move-se — sem ele as asserções acima passariam sobre uma lei
    // que nunca desloca nada.
    assert_ne!(
        ScrollFactor { k: [0.5, 0.5] }.deslocamento([400.0, 0.0]),
        [0.0, 0.0]
    );
    let _ = autorada;
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
    assert_eq!(drive_parallax(&mut sim, Some((centro, SEM_LIMITE)), PARADO, &mut drive), 2);

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
    drive_parallax(&mut sim2, Some(([0.0, 0.0], SEM_LIMITE)), PARADO, &mut drive2);
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
        drive_parallax(&mut sim, Some((centro, SEM_LIMITE)), PARADO, &mut drive);
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

// ══ A REPETIÇÃO INFINITA (plano 24, W2) ════════════════════════════════════════════════════════

fn cena_rep(k: [f32; 2], tile: [f32; 2], pose: Transform) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((ScrollFactor { k }, ScrollRepeat { tile }, pose))
        .id();
    (sim, e)
}

/// ⭐⭐⭐ **A CORRECÇÃO É UM NÚMERO INTEIRO DE LADRILHOS** — a lei medida no alvo, e a única metade
/// dela que a sonda de facto gravou.
///
/// ⚠️ **É isto que faz a costura não poder abrir:** a imagem a seguir ao salto é a mesma. ⛔ Somar
/// um RESTO faria o erro de `f32` acumular, e ao décimo milésimo ladrilho a costura estaria aberta.
#[test]
fn a_correccao_e_um_numero_inteiro_de_ladrilhos() {
    let tile = 256.0_f32;
    let rep = ScrollRepeat {
        tile: [tile, tile],
    };
    let mut corrigiu = 0;
    for i in -400_i16..=400 {
        let d = f32::from(i) * 37.0; // ⚠️ um passo que NÃO divide o ladrilho, de propósito
        let e = rep.envolve([d, -d]);
        for (bruto, envolvido) in [(d, e[0]), (-d, e[1])] {
            let n = (bruto - envolvido) / tile;
            assert!(
                (n - n.round()).abs() < 1e-3,
                "a correccao de {bruto} nao e' um inteiro de ladrilhos: {n}"
            );
            if n.abs() > 0.5 {
                corrigiu += 1;
            }
        }
    }
    // O CONTROLO: a varredura de facto passou por ladrilhos — sem ele as asserções acima passariam
    // sobre uma lei que nunca corrige nada.
    assert!(corrigiu > 700, "so' {corrigiu} correccoes: a varredura nao sai do 1.o ladrilho");
}

/// ⭐⭐⭐ **A FASE é a mesma ao DÉCIMO MILÉSIMO ladrilho** — o gate que o plano encomendou.
///
/// Com `k = 0,5` e um ladrilho de `256`, a câmera avança `512` por ladrilho. A varredura vai a
/// `10 240` unidades (**vinte** ladrilhos) e a pose tem de cair na mesma fase — ⚠️ **módulo o
/// ladrilho**, que é o que «a mesma fase» quer dizer: `+128` e `−128` são o MESMO ponto de uma
/// imagem que se repete a cada `256`, e uma régua que os comparasse como números crus acusaria a
/// lei certa.
///
/// # ⭐ E o tecto de `f32` está MEDIDO, e não é da lei
///
/// A fase mantém-se exacta até câmeras de `5 × 10¹⁰` para uma fase de zero; a que se perde primeiro
/// é a **coordenada da câmera** — a `5,12 × 10⁹` um `f32` já não consegue guardar um desvio de
/// `256` sobre ela, e o `+256` do meio-ladrilho evapora ANTES de chegar à lei. *O limite não é o
/// envolvimento; é representar onde a câmera está.*
#[test]
fn a_fase_e_a_mesma_ao_decimo_milesimo_ladrilho() {
    let tile = 256.0_f32;
    let autorada = pose_em(-3.0, 7.0);
    for fase_inicial in [0.0_f32, 256.0, 61.0] {
        let mut poses = Vec::new();
        for n in 0_i16..=20 {
            let (mut sim, e) = cena_rep([0.5, 1.0], [tile, 0.0], autorada);
            let mut drive = PreviewDrive::default();
            let centro = [fase_inicial + 512.0 * f32::from(n), 0.0];
            drive_parallax(&mut sim, Some((centro, SEM_LIMITE)), PARADO, &mut drive);
            poses.push(sim.world().get::<Transform>(e).expect("pose").translation.x);
        }
        for (n, p) in poses.iter().enumerate() {
            let delta = ph2d_ecs::envolve_eixo(p - poses[0], tile);
            assert!(
                delta.abs() < 1e-3,
                "fase {fase_inicial}: ao ladrilho {n} a pose e' {p} contra {} — a costura ABRIU \
                 (desvio de {delta} dentro do ladrilho)",
                poses[0]
            );
            // ⭐⭐⭐ **A METADE ABSOLUTA, e ela nasceu de uma MUTAÇÃO SOBREVIVENTE:** apagar a
            // repetição da ponte deixava a asserção de cima VERDE, porque a varredura anda
            // exactamente um ladrilho por passo e a diferença envolve para zero. *Uma régua que
            // compara a fase MÓDULO o ladrilho não distingue «a costura fechou» de «o fundo fugiu
            // um número inteiro de ladrilhos»* — e fugir é precisamente o que a repetição existe
            // para impedir. ⚠️ O controlo do fim mede outra CENA (sem o componente): ele prova que
            // a varredura mexe, nunca que é o componente que a segura.
            assert!(
                (p - autorada.translation.x).abs() <= tile / 2.0 + 1e-3,
                "fase {fase_inicial}: ao ladrilho {n} a pose FUGIU para {p} — o fundo deixou de \
                 ser infinito (autorada {}, meio ladrilho {})",
                autorada.translation.x,
                tile / 2.0
            );
        }
        // ⭐ O CONTROLO: **sem** a repetição a mesma varredura afasta-se `5 120` unidades. Sem ele
        // este gate passaria sobre uma paralaxe que nunca desloca nada.
        let (mut sim, e) = cena([0.5, 1.0], autorada);
        let mut drive = PreviewDrive::default();
        drive_parallax(&mut sim, Some(([fase_inicial + 512.0 * 20.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
        let solto = sim.world().get::<Transform>(e).expect("pose").translation.x;
        assert!(
            (solto - poses[0]).abs() > 1000.0,
            "sem repeticao a pose devia ter fugido, e leu {solto} contra {}",
            poses[0]
        );
    }
}

/// ⛔ **A ausência e o `0` são a mesma coisa** — a convenção do alvo, e é ela que permite repetir
/// só em X, que é o caso de quase todo fundo.
#[test]
fn um_ladrilho_zero_nao_corrige_e_o_eixo_livre_desloca() {
    let autorada = pose_em(0.0, 0.0);
    let centro = [4000.0_f32, 4000.0];
    // Repete em X (ladrilho 256) e NÃO em Y.
    let (mut sim, e) = cena_rep([0.5, 0.5], [256.0, 0.0], autorada);
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some((centro, SEM_LIMITE)), PARADO, &mut drive);
    let t = *sim.world().get::<Transform>(e).expect("pose");
    assert!(
        t.translation.x.abs() <= 128.0 + 1e-3,
        "o eixo REPETIDO fugiu do ladrilho: x = {}",
        t.translation.x
    );
    assert!(
        (t.translation.y - 2000.0).abs() < 1e-2,
        "o eixo LIVRE deixou de deslocar: y = {} contra 2000",
        t.translation.y
    );
    // E o CONTROLO da própria convenção: sem o componente os dois eixos fogem.
    let (mut sim2, e2) = cena([0.5, 0.5], autorada);
    let mut drive2 = PreviewDrive::default();
    drive_parallax(&mut sim2, Some((centro, SEM_LIMITE)), PARADO, &mut drive2);
    let t2 = *sim2.world().get::<Transform>(e2).expect("pose");
    assert!((t2.translation.x - 2000.0).abs() < 1e-2, "x = {}", t2.translation.x);
}

/// ⭐⭐ **A repetição não toca na POSE AUTORADA** — o artista continua a poder arrastar o fundo, e o
/// que ele arrasta é o documento.
///
/// ⚠️ É esta a razão de a lei envolver o DESLOCAMENTO e não a soma: envolver a soma envolveria
/// também o que ele autorou, e o fundo saltaria para a origem assim que ele o puxasse para além de
/// meio ladrilho.
#[test]
fn a_repeticao_nao_envolve_a_pose_autorada() {
    let tile = 256.0_f32;
    // ⚠️ A pose autorada está MUITO para lá de meio ladrilho, que é onde a lei errada morde.
    let autorada = pose_em(4000.0, 0.0);
    let (mut sim, e) = cena_rep([0.5, 1.0], [tile, 0.0], autorada);
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
    assert_eq!(
        *sim.world().get::<Transform>(e).expect("pose"),
        autorada,
        "com a camera na origem a repeticao mexeu na pose que o artista autorou"
    );
    // E com a câmera longe, o que se vê é a autorada mais uma fase — nunca a autorada envolvida.
    drive_parallax(&mut sim, Some(([10_000.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
    let x = sim.world().get::<Transform>(e).expect("pose").translation.x;
    assert!(
        (x - 4000.0).abs() <= 128.0 + 1e-3,
        "a pose fugiu do ladrilho a` volta do AUTORADO: {x} contra 4000 ± 128"
    );
    let Some(Driven::ParallaxPose(memo)) = drive.authored(e.to_bits(), Driver::ParallaxPose) else {
        panic!("tinha de continuar a ser conduzido");
    };
    assert_eq!(memo, autorada, "o autorado foi envolvido");
}

// ══ O CONFINAMENTO (plano 24, W3) ══════════════════════════════════════════════════════════════

/// A cena da tabela medida no alvo: região `−600..600`, k `0,5`.
fn cena_lim(k: f32, min: f32, max: f32) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            ScrollFactor { k: [k, 1.0] },
            ScrollLimits {
                min: [min, 0.0],
                max: [max, 0.0],
            },
            pose_em(0.0, 0.0),
        ))
        .id();
    (sim, e)
}

fn pose_com_vista(k: f32, min: f32, max: f32, cam: f32, meia: f32) -> f32 {
    let (mut sim, e) = cena_lim(k, min, max);
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some(([cam, 0.0], [meia, 0.0])), PARADO, &mut drive);
    sim.world().get::<Transform>(e).expect("pose").translation.x
}

/// ⭐⭐⭐ **A CURVA COM OS DOIS JOELHOS, pedaço a pedaço** — a tabela medida no alvo, reproduzida.
///
/// ```text
/// cam.x  −1200 … −360   declive +1,0000     ← a camada CONGELA no ecrã
/// cam.x   −240 …  +240   declive +0,5000     ← a paralaxe autorada
/// cam.x   +360 … +1200   declive +1,0000     ← congela outra vez
/// ```
///
/// ⚠️⚠️ **O gate amostra os PEDAÇOS e nunca a média, e isso não é estilo:** duas leis foram
/// construídas e REFUTADAS antes desta porque eu media um declive MÉDIO sobre uma curva que tem
/// joelhos. *Um clamp não tem um declive; tem pedaços* — e a média de dois pedaços não é nenhum
/// deles. Uma régua média aqui lê `~0,8` e aprova qualquer lei que passe pelos extremos.
#[test]
fn a_curva_do_confinamento_tem_os_dois_joelhos() {
    let (k, min, max, meia) = (0.5_f32, -600.0_f32, 600.0_f32, 360.0_f32);
    let declive = |a: f32, b: f32| {
        (pose_com_vista(k, min, max, b, meia) - pose_com_vista(k, min, max, a, meia)) / (b - a)
    };
    for (a, b, esperado, nome) in [
        (-1200.0_f32, -360.0_f32, 1.0_f32, "congelada a` esquerda"),
        (-240.0, 240.0, 0.5, "a paralaxe autorada"),
        (360.0, 1200.0, 1.0, "congelada a` direita"),
    ] {
        let m = declive(a, b);
        assert!(
            (m - esperado).abs() < 1e-4,
            "{nome} ({a} .. {b}): declive {m} contra {esperado} — a tabela do alvo"
        );
    }
    // ⭐ E o CONTROLO que prova que a régua vê os pedaços: a MÉDIA da curva inteira não é nenhum
    // deles, e é ela que aprovaria a lei errada.
    let media = declive(-1200.0, 1200.0);
    assert!(
        (media - 0.5).abs() > 0.1 && (media - 1.0).abs() > 0.1,
        "a media da curva inteira ({media}) caiu em cima de um dos pedacos: este gate deixou de \\
         poder distinguir a lei da media dela"
    );
}

/// ⭐⭐⭐ **O JOELHO é `(região − ecrã)/2`** — o ponto em que a borda da VISTA alcança a da REGIÃO.
///
/// ⚠️ **É por isto que a meia-janela tem de atravessar** (e é a premissa da W1 que morreu): o joelho
/// move-se com o ZOOM, e uma lei que só visse o centro poria-o sempre no mesmo sítio.
#[test]
fn o_joelho_esta_onde_a_borda_da_vista_alcanca_a_regiao() {
    for (largura, ecra) in [(1200.0_f32, 720.0_f32), (1200.0, 200.0), (400.0, 100.0)] {
        let (min, max, meia) = (-largura / 2.0, largura / 2.0, ecra / 2.0);
        let joelho = (largura - ecra) / 2.0;
        // ⚠️ Mede-se DENTRO e FORA do joelho, com uma margem, porque o ponto exacto é a fronteira
        // dos dois pedaços e não pertence a nenhum.
        let m_dentro = (pose_com_vista(0.5, min, max, joelho - 1.0, meia)
            - pose_com_vista(0.5, min, max, joelho - 21.0, meia))
            / 20.0;
        let m_fora = (pose_com_vista(0.5, min, max, joelho + 21.0, meia)
            - pose_com_vista(0.5, min, max, joelho + 1.0, meia))
            / 20.0;
        assert!(
            (m_dentro - 0.5).abs() < 1e-3,
            "largura {largura} ecra {ecra}: a {} unidades o declive e' {m_dentro} e devia ser a \\
             paralaxe (0,5) — o joelho chegou CEDO",
            joelho - 11.0
        );
        assert!(
            (m_fora - 1.0).abs() < 1e-3,
            "largura {largura} ecra {ecra}: a {} unidades o declive e' {m_fora} e devia ser o \\
             congelamento (1,0) — o joelho chegou TARDE",
            joelho + 11.0
        );
    }
}

/// ⛔ **Sem limites a saída é BYTE-IDÊNTICA** — o segundo termo é `k · 0`, e é essa a forma que a
/// lei tem escrita. ⚠️ A forma equivalente `centro − k·confinado` diferiria por um ULP, e mudava o
/// que a W1 e a W2 já shipam.
#[test]
fn sem_limites_a_saida_e_byte_identica() {
    for centro in [[400.0_f32, -250.0], [0.1, 3.7], [-1e4, 1e4]] {
        for k in [[0.3_f32, 0.7], [0.0, 2.0], [-0.5, 1.0]] {
            let autorada = pose_em(-3.0, 7.0);
            let (mut a, ea) = cena(k, autorada);
            let mut da = PreviewDrive::default();
            drive_parallax(&mut a, Some((centro, [360.0, 360.0])), PARADO, &mut da);

            let mut b = SimWorld::default();
            // ⚠️ Uma região VAZIA (`max == min`) é a omissão — o componente presente e inerte.
            let eb = b
                .world_mut()
                .spawn((
                    ScrollFactor { k },
                    ScrollLimits::default(),
                    autorada,
                ))
                .id();
            let mut db = PreviewDrive::default();
            drive_parallax(&mut b, Some((centro, [360.0, 360.0])), PARADO, &mut db);

            assert_eq!(
                a.world().get::<Transform>(ea).expect("pose").translation,
                b.world().get::<Transform>(eb).expect("pose").translation,
                "com limites INERTES a pose mudou (centro {centro:?}, k {k:?}) — o segundo termo \\
                 deixou de ser `k · 0`"
            );
        }
    }
}

/// ⛔⛔ **Uma região mais ESTREITA que a vista fixa-a no CENTRO dela** — e não entra em pânico.
///
/// ⚠️ Sem esta metade o clamp seria `clamp(c, min+h, max−h)` com o limite de baixo ACIMA do de
/// cima, e o `f32::clamp` do Rust **entra em pânico** ali. *É o mesmo caso que a câmera do jogo já
/// pagou* (uma cerca mais estreita que a janela), com a mesma cura.
#[test]
fn uma_regiao_mais_estreita_que_a_vista_nao_entra_em_panico() {
    // Região de `100` com uma vista de `720`: ela cabe inteira, e o sítio honesto é o centro dela.
    let x = pose_com_vista(0.5, 150.0, 250.0, 5000.0, 360.0);
    // O centro da região é `200`; o confinado é `200` ⇒ `d = 5000·0,5 + 0,5·(5000 − 200) = 4900`.
    assert!((x - 4900.0).abs() < 1e-2, "x = {x} contra 4900");
    // E o CONTROLO: com a vista a caber na região o resultado é outro.
    let y = pose_com_vista(0.5, -5000.0, 5000.0, 0.0, 360.0);
    assert!(y.abs() < 1e-3, "y = {y}");
}

/// ⭐⭐ **A ordem DECLARADA: confinar e depois envolver.**
///
/// ⚠️ Os dois juntos são uma cena que se contradiz — *um fundo que repete não tem borda para
/// esconder* —, e o que este gate fixa é o que ela DÁ: a repetição vem por último, logo o
/// deslocamento final cai sempre dentro de meio ladrilho, mesmo com a camada congelada.
#[test]
fn com_limites_e_repeticao_a_repeticao_e_a_ultima() {
    let tile = 256.0_f32;
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            ScrollFactor { k: [0.5, 1.0] },
            ScrollLimits {
                min: [-600.0, 0.0],
                max: [600.0, 0.0],
            },
            ScrollRepeat { tile: [tile, 0.0] },
            pose_em(0.0, 0.0),
        ))
        .id();
    let mut drive = PreviewDrive::default();
    // Muito para lá do joelho: sem a repetição a camada teria fugido `~4 900`.
    drive_parallax(&mut sim, Some(([5000.0, 0.0], [360.0, 0.0])), PARADO, &mut drive);
    let x = sim.world().get::<Transform>(e).expect("pose").translation.x;
    assert!(
        x.abs() <= tile / 2.0 + 1e-3,
        "a repeticao deixou de ser a ULTIMA: x = {x}, fora de meio ladrilho"
    );
}

/// ⭐⭐⭐ **A FORMA da composição, e ela é BYTE-IDÊNTICA à da W1 — com o CONTROLO que prova que a
/// forma alternativa não é.**
///
/// # ⛔⛔ Porque este gate precisou de existir: o irmão comparava o mutante consigo próprio
///
/// O `sem_limites_a_saida_e_byte_identica` mede *«uma cena com limites inertes dá a mesma pose que
/// uma sem o componente»* — e a mutação que troca a lei por `centro − k·confinado` **SOBREVIVEU**,
/// porque os **dois** lados dela passam pela lei mutada e apanham o mesmo ULP. *Um gate que compara
/// duas corridas do mesmo código não pode ver a forma desse código mudar.*
///
/// ⇒ a régua é a IDENTIDADE entre as duas portas: com `confinado == centro`, a
/// `deslocamento_confinado` tem de devolver a `deslocamento` **ao bit**, porque o segundo termo é
/// `k · 0`.
#[test]
fn a_composicao_do_confinamento_e_byte_identica_a_lei_da_w1() {
    // ⚠️ O corpus CONTÉM os pares discriminantes — medido: das `70` células, `8` dão números
    // diferentes nas duas formas (`(3,7 · 0,3)` lê `2,5899999` contra `2,5900002`). *Um corpus sem
    // eles aprovaria as duas.*
    let centros = [0.1_f32, 3.7, 400.0, -250.0, 1e4, -1e4, 7.0, -3.0, 1234.567];
    let ks = [0.3_f32, 0.7, 0.5, 2.0, -0.5, 0.25, 0.1234];
    let mut discriminantes = 0;
    for c in centros {
        for k in ks {
            let f = ScrollFactor { k: [k, k] };
            let centro = [c, -c];
            assert_eq!(
                f.deslocamento_confinado(centro, centro),
                f.deslocamento(centro),
                "com a vista DENTRO da regiao a lei deixou de ser a da W1 (centro {c}, k {k}) — o \
                 segundo termo deixou de ser `k · 0`"
            );
            // O CONTROLO: a forma alternativa `centro − k·confinado` dá outro número aqui.
            if c * (1.0 - k) != c - k * c {
                discriminantes += 1;
            }
        }
    }
    assert!(
        discriminantes >= 5,
        "so' {discriminantes} pares discriminam as duas formas: este gate deixou de poder ver a \
         diferenca que ele existe para guardar"
    );
}

// ══ O MOVIMENTO PRÓPRIO (plano 24, W4) ═════════════════════════════════════════════════════════

fn cena_mov(k: [f32; 2], v: [f32; 2], pose: Transform) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((ScrollFactor { k }, ScrollMotion { velocity: v }, pose))
        .id();
    (sim, e)
}

fn pose_em_t(k: [f32; 2], v: [f32; 2], centro: [f32; 2], t: f64) -> Vec2 {
    let (mut sim, e) = cena_mov(k, v, pose_em(0.0, 0.0));
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some((centro, SEM_LIMITE)), t, &mut drive);
    sim.world().get::<Transform>(e).expect("pose").translation
}

/// ⭐⭐⭐ **A deriva é `velocidade × playhead`, e ela é PURA** — o mesmo instante dá a mesma pose,
/// e a ordem em que os instantes são pedidos não conta.
///
/// ⚠️ **É esta propriedade que se compra**, e não o efeito: o *autoscroll* do alvo move `+0,000` nos
/// quatro observáveis (medido, §4.6) porque vive no caminho de DESENHO. O nosso é medível, e é isso
/// que faz este gate existir.
#[test]
fn a_deriva_e_velocidade_vezes_o_playhead() {
    let v = [3.0_f32, -1.5];
    for t in [0.0_f64, 0.5, 2.0, 10.0, 3600.0] {
        let p = pose_em_t(ScrollFactor::NEUTRO, v, [0.0, 0.0], t);
        #[allow(clippy::cast_possible_truncation)]
        let esperado = Vec2::new((f64::from(v[0]) * t) as f32, (f64::from(v[1]) * t) as f32);
        assert_eq!(p, esperado, "t = {t}");
    }

    // ⭐⭐⭐ **O PRODUTO é feito em `f64`, e o corpus de cima NÃO o discriminava** — uma mutação
    // que o fizesse em `f32` SOBREVIVEU, porque `3,0 × 3 600` é exacto nas duas larguras.
    //
    // ⚠️ **A deriva é a única grandeza desta família que cresce sem limite com o tempo**, e é ela
    // que paga o `f64`: ao fim de uma hora de relógio o ULP de um `f32` é `2,4e-4`, e arredondar o
    // PRODUTO em vez do resultado é o que o dobra. Os pares abaixo são **medidos** — de `72`
    // células, `10` separam as duas larguras, e a pior é `3,9e-3` (uma nuvem a `12,3 m/s` ao fim de
    // uma hora). *Um corpus de números redondos não distingue duas aritméticas.*
    let mut discriminantes = 0;
    for (v, t, esperado) in [
        (12.345_f32, 3600.1_f64, 44443.234_375_f32),
        (3.0, 3600.1, 10800.299_804_687_5),
        (12.345, 1234.567_8, 15240.740_234_375),
        (0.3, 1234.567_8, 370.370_361_328_125),
    ] {
        let p = pose_em_t(ScrollFactor::NEUTRO, [v, 0.0], [0.0, 0.0], t);
        assert_eq!(p.x, esperado, "v = {v}, t = {t} — o produto deixou de ser feito em `f64`");
        #[allow(clippy::cast_possible_truncation)]
        if v * (t as f32) != esperado {
            discriminantes += 1;
        }
    }
    assert_eq!(
        discriminantes, 4,
        "os pares deixaram de separar as duas larguras: este gate voltou a nao poder ver a \
         diferenca que ele existe para guardar"
    );
}

/// ⭐⭐⭐ **SCRUBBAR PARA TRÁS DESFAZ A DERIVA** — a propriedade que o alvo não tem, e a razão de a
/// lei não ter estado.
///
/// ⚠️ Um acumulador (`pos += v·dt`) daria uma nuvem que **continua a andar** quando o artista puxa a
/// régua para trás, e duas máquinas com quadros diferentes veriam nuvens diferentes. *Este gate é a
/// diferença entre uma deriva e um contador.*
#[test]
fn um_scrub_para_tras_desfaz_a_deriva() {
    let v = [3.0_f32, 0.0];
    let (mut sim, e) = cena_mov(ScrollFactor::NEUTRO, v, pose_em(0.0, 0.0));
    let mut drive = PreviewDrive::default();
    let mut pose_em_instante = |t: f64| {
        drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), t, &mut drive);
        sim.world().get::<Transform>(e).expect("pose").translation.x
    };
    // ⚠️ A régua percorre os instantes FORA de ordem, de propósito: uma lei com acumulador passa
    // a subir e nunca a descer, e uma varredura monótona não a distinguiria da pura.
    let ida = pose_em_instante(10.0);
    let volta = pose_em_instante(2.0);
    let repete = pose_em_instante(10.0);
    let zero = pose_em_instante(0.0);
    assert!((ida - 30.0).abs() < 1e-3, "t=10 leu {ida}");
    assert!((volta - 6.0).abs() < 1e-3, "t=2 depois de t=10 leu {volta} — a deriva ACUMULOU");
    assert!((repete - 30.0).abs() < 1e-3, "voltar a t=10 leu {repete}");
    assert!(zero.abs() < 1e-3, "rebobinar leu {zero} — a nuvem nao voltou ao principio");
}

/// ⭐⭐ **A deriva SOMA-SE à paralaxe** — ela não é um segundo condutor.
///
/// ⚠️ Medido no [`super::w4_probe`]: dois motores sobre o mesmo `Transform` entram no ledger com
/// chaves diferentes, e esta ponte leria a escrita do outro como um arrasto do artista.
#[test]
fn a_deriva_soma_se_ao_deslocamento_da_camera() {
    let k = [0.5_f32, 0.5];
    let v = [3.0_f32, 0.0];
    let centro = [400.0_f32, 0.0];
    let so_camera = pose_em_t(k, [0.0, 0.0], centro, 10.0);
    let so_deriva = pose_em_t(k, v, [0.0, 0.0], 10.0);
    let ambas = pose_em_t(k, v, centro, 10.0);
    assert!((so_camera.x - 200.0).abs() < 1e-3, "so' camera: {}", so_camera.x);
    assert!((so_deriva.x - 30.0).abs() < 1e-3, "so' deriva: {}", so_deriva.x);
    assert!(
        (ambas.x - 230.0).abs() < 1e-3,
        "as duas juntas leem {} e nao a SOMA (230)",
        ambas.x
    );
}

/// ⛔⛔ **Um objecto com deriva é conduzido MESMO com `k` neutro** — a metade que o salto do neutro
/// esconderia.
///
/// ⚠️ `k = 1` diz *«não guardo nada do movimento da CÂMERA»*, e a deriva não é movimento da câmera.
/// Sem esta metade uma nuvem que anda sozinha num plano normal ficaria parada — e o painel diria
/// que está a andar.
#[test]
fn a_deriva_acorda_um_objecto_de_k_neutro() {
    let (mut sim, e) = cena_mov(ScrollFactor::NEUTRO, [3.0, 0.0], pose_em(0.0, 0.0));
    let mut drive = PreviewDrive::default();
    assert_eq!(
        drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), 10.0, &mut drive),
        1,
        "um objecto com deriva e `k` neutro nao foi conduzido"
    );
    assert!((sim.world().get::<Transform>(e).expect("pose").translation.x - 30.0).abs() < 1e-3);
    // ⭐ E o CONTROLO da omissão: `k` neutro **sem** deriva continua a ser saltado, byte-idêntico.
    let (mut sim2, e2) = cena_mov(ScrollFactor::NEUTRO, [0.0, 0.0], pose_em(-3.0, 7.0));
    let mut drive2 = PreviewDrive::default();
    assert_eq!(
        drive_parallax(&mut sim2, Some(([400.0, 0.0], SEM_LIMITE)), 10.0, &mut drive2),
        0
    );
    assert_eq!(
        *sim2.world().get::<Transform>(e2).expect("pose"),
        pose_em(-3.0, 7.0)
    );
    assert!(drive2.is_empty());
}

/// ⭐ **A deriva entra ANTES da repetição** — quem anda para sempre é precisamente quem tem de
/// envolver, e envolver antes de somar deixaria a nuvem a fugir.
#[test]
fn uma_nuvem_que_deriva_e_repete_nao_foge() {
    let tile = 256.0_f32;
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            ScrollFactor { k: ScrollFactor::NEUTRO },
            ScrollMotion { velocity: [3.0, 0.0] },
            ScrollRepeat { tile: [tile, 0.0] },
            pose_em(0.0, 0.0),
        ))
        .id();
    let mut drive = PreviewDrive::default();
    for t in [0.0_f64, 60.0, 600.0, 6000.0] {
        drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), t, &mut drive);
        let x = sim.world().get::<Transform>(e).expect("pose").translation.x;
        assert!(
            x.abs() <= tile / 2.0 + 1e-3,
            "a t = {t} a nuvem fugiu para {x} — a repeticao deixou de a apanhar"
        );
    }
    // O CONTROLO: sem a repetição, aos `6 000 s` ela está a `18 000` do sítio.
    let solta = pose_em_t(ScrollFactor::NEUTRO, [3.0, 0.0], [0.0, 0.0], 6000.0);
    assert!((solta.x - 18_000.0).abs() < 1.0, "solta: {}", solta.x);
}

// ══ O DOLLY — a multiplano (plano 24, W5 · §2) ══════════════════════════════════════════════════

/// Uma cena com câmera de jogo, que é de onde o dolly sai.
fn cena_dolly(k: [f32; 2], dolly: f32, pose: Transform) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    sim.world_mut().spawn((
        ph2d_ecs::GameCamera {
            dolly,
            ..ph2d_ecs::GameCamera::default()
        },
        ph2d_ecs::StableId(1),
        Transform::default(),
    ));
    let e = sim.world_mut().spawn((ScrollFactor { k }, pose)).id();
    (sim, e)
}

/// ⭐⭐⭐ **A ESCALA do dolly é `(1 − δ)/(1 − k·δ)`, e ela foi verificada em aritmética EXACTA.**
///
/// A tabela sai de uma varredura em `Fraction` do Python contra a definição geométrica
/// (`[z/(z−d)] ÷ [z₀/(z₀−d)]`), com `12` células a concordarem ao racional — a fórmula fechada é a
/// definição e não uma aproximação dela.
#[test]
fn a_escala_do_dolly_bate_a_geometria() {
    for (k, delta, esperado) in [
        (1.0_f32, 0.0_f32, 1.0_f32),
        (1.0, 0.5, 1.0),
        (1.0, -1.0, 1.0),
        (0.5, 0.5, 2.0 / 3.0),
        (0.5, -1.0, 4.0 / 3.0),
        (0.25, 0.5, 4.0 / 7.0),
        (0.25, -1.0, 1.6),
        (0.01, 0.5, 0.502_512_58),
    ] {
        let e = ScrollFactor::escala_do_dolly(k, delta).expect("a camera nao atravessa");
        assert!(
            (e - esperado).abs() < 1e-6,
            "k = {k}, delta = {delta}: {e} contra {esperado}"
        );
    }
}

/// ⛔⛔ **A DEGENERESCÊNCIA QUE O PLANO PUBLICA ESTÁ REFUTADA PELA FÓRMULA DELE** — e este gate é a
/// refutação, não uma nota.
///
/// O §2 do plano diz *«`k = 0` é `z = ∞` ⇒ escala ≡ 1 para todo `d` (o que está infinitamente longe
/// nunca muda de tamanho)»*. ⚠️ **O parêntesis é verdade e a conclusão não:** o tamanho ABSOLUTO não
/// muda, e a `escala` desta lei é RELATIVA ao plano focal — que CRESCEU. A fórmula fechada dá
/// `1 − δ`, e o limite numérico confirma-o (`k = 1/10` → `0,5263`; `1/10⁴` → `0,50003`).
///
/// ⭐ **E não há um braço `if k == 0`:** a fórmula fechada já contém a lei, logo ela não pode
/// divergir do limite. *Uma degenerescência escrita num ramo é a segunda resposta à mesma pergunta.*
#[test]
fn o_ceu_encolhe_relativamente_ao_plano_focal_e_o_plano_dizia_o_contrario() {
    let delta = 0.5_f32;
    let ceu = ScrollFactor::escala_do_dolly(0.0, delta).expect("o ceu nao e' atravessado");
    assert!(
        (ceu - (1.0 - delta)).abs() < 1e-6,
        "o ceu leu {ceu} e a formula da' 1 − delta = {}",
        1.0 - delta
    );
    assert!(
        (ceu - 1.0).abs() > 0.4,
        "o ceu leu {ceu}, que e' o que o plano prometia — a refutacao evaporou"
    );
    // ⭐ O LIMITE: a fórmula fechada e o `k` a tender para zero encontram-se.
    for n in [10.0_f32, 100.0, 10_000.0] {
        let quase = ScrollFactor::escala_do_dolly(1.0 / n, delta).expect("nao atravessa");
        assert!(
            (quase - ceu).abs() < 2.0 / n,
            "k = 1/{n}: {quase} nao converge para {ceu}"
        );
    }
}

/// ⭐⭐⭐ **A MULTIPLANO: dois planos, um dolly, e a razão dos tamanhos bate a fórmula** — o gate que
/// o plano encomendou.
#[test]
fn dois_planos_com_um_dolly_dao_a_razao_da_formula() {
    let delta = 0.5_f32;
    let mut sim = SimWorld::default();
    sim.world_mut().spawn((
        ph2d_ecs::GameCamera {
            dolly: delta,
            ..ph2d_ecs::GameCamera::default()
        },
        ph2d_ecs::StableId(1),
        Transform::default(),
    ));
    let perto = sim
        .world_mut()
        .spawn((ScrollFactor { k: [1.0, 1.0] }, pose_em(0.0, 0.0)))
        .id();
    let longe = sim
        .world_mut()
        .spawn((ScrollFactor { k: [0.25, 0.25] }, pose_em(0.0, 0.0)))
        .id();
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
    let sp = sim.world().get::<Transform>(perto).expect("pose").scale.x;
    let sl = sim.world().get::<Transform>(longe).expect("pose").scale.x;
    assert!((sp - 1.0).abs() < 1e-5, "o plano focal mudou de tamanho: {sp}");
    assert!((sl - 4.0 / 7.0).abs() < 1e-5, "o fundo leu {sl} contra 4/7");
    // ⭐ **É isto que nenhum motor 2D faz:** um zoom multiplicaria os dois pelo MESMO número.
    assert!(
        (sp / sl - 7.0 / 4.0).abs() < 1e-4,
        "a razao entre os dois planos e' {} e nao 7/4 — isto virou um zoom",
        sp / sl
    );
}

/// ⛔⛔ **Com `dolly = 0` a saída é BYTE-IDÊNTICA à da W1** — a exigência do plano, e a lei desta
/// casa para tudo o que é novo.
#[test]
fn com_dolly_zero_a_saida_e_byte_identica() {
    let autorada = Transform {
        translation: Vec2::new(-3.0, 7.0),
        rotation: 0.35,
        scale: Vec2::new(2.0, 3.0),
        ..Transform::default()
    };
    for k in [[0.3_f32, 0.7], [0.0, 2.0], [-0.5, 1.0]] {
        let (mut a, ea) = cena(k, autorada);
        let mut da = PreviewDrive::default();
        drive_parallax(&mut a, Some(([400.0, -250.0], SEM_LIMITE)), PARADO, &mut da);

        let (mut b, eb) = cena_dolly(k, 0.0, autorada);
        let mut db = PreviewDrive::default();
        drive_parallax(&mut b, Some(([400.0, -250.0], SEM_LIMITE)), PARADO, &mut db);

        assert_eq!(
            *a.world().get::<Transform>(ea).expect("pose"),
            *b.world().get::<Transform>(eb).expect("pose"),
            "com dolly ZERO a pose mudou (k {k:?}) — a omissao da W5 deixou de ser inerte"
        );
    }
}

/// ⛔⛔ **A câmera a ATRAVESSAR a camada RECUSA, e o objecto fica onde o artista o pôs.**
///
/// ⚠️ `1 − k·δ ≤ 0` é `d ≥ z`: a câmera passou para lá do fundo, e não há tamanho aparente nenhum.
/// *Um clamp ali entregaria um número plausível para uma cena impossível.*
#[test]
fn a_camera_a_atravessar_a_camada_recusa() {
    // `k = 0,5` ⇒ `z = 2·z₀`; um dolly de `2,5` passa para lá dela.
    let autorada = pose_em(-3.0, 7.0);
    let (mut sim, e) = cena_dolly([0.5, 0.5], 2.5, autorada);
    let mut drive = PreviewDrive::default();
    assert_eq!(
        drive_parallax(&mut sim, Some(([400.0, 0.0], SEM_LIMITE)), PARADO, &mut drive),
        0,
        "a camada atravessada foi conduzida"
    );
    assert_eq!(*sim.world().get::<Transform>(e).expect("pose"), autorada);
    assert!(ScrollFactor::escala_do_dolly(0.5, 2.5).is_none());
    // ⭐ O CONTROLO: um dolly de `1,9` ainda está aquém dela e conduz.
    let (mut sim2, _) = cena_dolly([0.5, 0.5], 1.9, autorada);
    let mut drive2 = PreviewDrive::default();
    assert_eq!(
        drive_parallax(&mut sim2, Some(([400.0, 0.0], SEM_LIMITE)), PARADO, &mut drive2),
        1
    );
}

/// ⭐⭐ **A escala multiplica a AUTORADA e nunca a viva** — senão ela COMPÕE a cada quadro e o fundo
/// cresce sem limite.
#[test]
fn a_escala_do_dolly_nao_compoe_entre_quadros() {
    let autorada = Transform {
        scale: Vec2::new(2.0, 2.0),
        ..pose_em(0.0, 0.0)
    };
    let (mut sim, e) = cena_dolly([0.25, 0.25], 0.5, autorada);
    let mut drive = PreviewDrive::default();
    let mut escalas = Vec::new();
    for _ in 0..5 {
        drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
        escalas.push(sim.world().get::<Transform>(e).expect("pose").scale.x);
    }
    let esperado = 2.0 * (4.0 / 7.0);
    for (i, s) in escalas.iter().enumerate() {
        assert!(
            (s - esperado).abs() < 1e-5,
            "ao quadro {i} a escala leu {s} contra {esperado} — ela COMPOS"
        );
    }
}

/// ⭐⭐⭐ **O DOLLY MUDA A FRACÇÃO, e não só o tamanho** — a metade da multiplano que um zoom também
/// não faz.
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE:** apagar o dolly do deslocamento (usar `k₀` em
/// vez de `k(δ)`) passava por toda a bancada, porque **nenhum gate da tabela do declive tem uma
/// câmera de jogo na cena** — sem ela o dolly é `0` e as duas leituras coincidem. *Um corpus onde o
/// parâmetro está no ponto neutro não testa esse parâmetro*, a forma que este repo já conta.
#[test]
fn o_dolly_muda_a_velocidade_e_nao_so_o_tamanho() {
    let (k, delta) = ([0.25_f32, 0.25], 0.5_f32);
    // `k(δ) = k · (1−δ)/(1−kδ) = 0,25 · (4/7) = 1/7` ⇒ declive `1 − 1/7 = 6/7`.
    let declive = |d: f32| {
        let medir = |cam: f32| {
            let (mut sim, e) = cena_dolly(k, d, pose_em(0.0, 0.0));
            let mut drive = PreviewDrive::default();
            drive_parallax(&mut sim, Some(([cam, 0.0], SEM_LIMITE)), PARADO, &mut drive);
            sim.world().get::<Transform>(e).expect("pose").translation.x
        };
        (medir(200.0) - medir(-200.0)) / 400.0
    };
    let sem = declive(0.0);
    let com = declive(delta);
    assert!((sem - 0.75).abs() < 1e-4, "sem dolly o declive e' {sem} contra 1 − 0,25");
    assert!(
        (com - 6.0 / 7.0).abs() < 1e-4,
        "com dolly {delta} o declive leu {com} contra 6/7 — a fraccao deixou de mudar, e o dolly \
         virou um zoom que so' mexe no tamanho"
    );
}

/// ⛔⛔ **O dolly sai da câmera ACTIVA, e não de qualquer uma.**
///
/// ⚠️ **Também nasceu de uma mutação sobrevivente:** todas as cenas do dolly têm UMA câmera, e ali
/// «a activa» e «a primeira» são a mesma. *Uma fixtura com um só candidato não testa uma escolha.*
#[test]
fn o_dolly_sai_da_camera_activa_e_nao_de_qualquer_uma() {
    let mut sim = SimWorld::default();
    // ⚠️⚠️ **A DESLIGADA nasce PRIMEIRO, e a ordem é load-bearing:** com a activa à frente, *«a
    // activa»* e *«a primeira que a consulta devolve»* são a mesma entidade, e a mutação que troca
    // a porta por uma consulta crua **SOBREVIVE**. *Uma fixtura em que duas respostas coincidem
    // não distingue as duas perguntas* — e foi assim que esta prova nasceu.
    sim.world_mut().spawn((
        ph2d_ecs::GameCamera {
            dolly: 0.5,
            priority: 99,
            active: false,
            ..ph2d_ecs::GameCamera::default()
        },
        ph2d_ecs::StableId(2),
        Transform::default(),
    ));
    // A ACTIVA, sem dolly.
    sim.world_mut().spawn((
        ph2d_ecs::GameCamera {
            dolly: 0.0,
            priority: 10,
            ..ph2d_ecs::GameCamera::default()
        },
        ph2d_ecs::StableId(1),
        Transform::default(),
    ));
    let autorada = Transform {
        scale: Vec2::new(2.0, 2.0),
        ..pose_em(0.0, 0.0)
    };
    let e = sim
        .world_mut()
        .spawn((ScrollFactor { k: [0.25, 0.25] }, autorada))
        .id();
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
    assert_eq!(
        sim.world().get::<Transform>(e).expect("pose").scale,
        autorada.scale,
        "uma camera DESLIGADA mandou no dolly: a escolha da activa deixou de passar pela porta"
    );
}
