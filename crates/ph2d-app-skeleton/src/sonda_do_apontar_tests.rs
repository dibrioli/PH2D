//! ⏱️ **UM OSSO JÁ SABE APONTAR PARA UM ALVO?** — a pergunta que a §5.0 do `CLAUDE.md` obriga a
//! fazer antes da primeira linha (*«antes de construir um item de lista aberta, MEÇA se a
//! composição já o exprime»*).
//!
//! A auditoria contra o Godot (MIT) nomeou o `SkeletonModification2DLookAt` como ausência nossa. E
//! a lei do alcance tem, escrito no corpo dela, um braço para **uma corrente de UM osso**
//! (*«um osso só: aponta, e o comprimento manda»*) — logo a pergunta não é se o motor existe: é se
//! ele é ALCANÇÁVEL pelo que a casa já monta, e **com que erro**.
//!
//! ⚠️ A sonda corre pelo caminho do PRODUTO (`goal::add` + `solve`), nunca chamando a lei pura: o
//! que se quer saber é se a cadeia inteira — corrente, mistura, limite e ledger — entrega o
//! apontar.

use super::*;
use crate::goal::osso;

/// Um esqueleto de **UM** osso na origem, deitado sobre o `+X` e com 10 de comprimento.
fn um_osso() -> (SimWorld, Entity) {
    let mut sim = SimWorld::default();
    let e = osso(&mut sim, "Aim", [0.0, 0.0], 10.0, None);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, e)
}

/// O ângulo (em graus) entre o eixo do osso e a direcção do alvo, depois de um quadro resolvido
/// com a corrente encurtada a `chain`.
fn erro_de_mira(chain: u32, alvo: [f64; 2]) -> f64 {
    let (mut sim, e) = um_osso();
    let t = crate::goal::add(&mut sim, e).expect("a ancora nasce");
    // A corrente que a sonda quer medir. ⚠️ Escrita no componente, que é o que o campo `IK Chain`
    // do painel escreve — a sonda não inventa um caminho próprio.
    if let Some(mut g) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::IkGoal>(e) {
        g.chain = chain;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o `Transform` da casa é f32; a geometria do documento é f64"
    )]
    if let Some(mut tt) = sim.world_mut().get_mut::<Transform>(t) {
        tt.translation = ph2d_core::Vec2::new(alvo[0] as f32, alvo[1] as f32);
    }
    let mut pv = ph2d_preview_drive::PreviewDrive::default();
    crate::goal::solve(&mut sim, &mut pv);
    let ponta = crate::bone_pick::tip_of(&sim, e.to_bits()).expect("o osso existe");
    let querido = alvo[1].atan2(alvo[0]);
    let teve = ponta[1].atan2(ponta[0]);
    ph2d_skeleton::wrap_pi(teve - querido).abs().to_degrees()
}

/// ⭐⭐⭐ **A MEDIÇÃO: com a corrente em UM, o osso aponta — e o erro é ruído de `f32`.**
///
/// | alvo | erro de mira |
/// |---|---|
/// | `(30, 0)` (em frente, fora do alcance) | `0,000000°` |
/// | `(0, 30)` (a norte) | `0,000000°` |
/// | `(−20, 15)` (atrás e acima) | `0,000000°` |
/// | `(5, −25)` (abaixo) | `0,000000°` |
/// | `(3, 2)` (**dentro** do alcance) | `0,000000°` |
///
/// ⇒ *o motor do LookAt já existe e já é alcançável; o que falta é o NOME e o desvio.* Esta sonda
/// é a razão de a wave do apontar ser um VERBO sobre a lei que já lá está, e não uma lei nova.
///
/// ⚠️ **A barra é DERIVADA e não escolhida:** a pose vive num `Transform` cuja rotação é `f32`, e o
/// ângulo reconstruído da ponta carrega esse arredondamento — `1e-3°` é folgado por três ordens de
/// grandeza sobre o que se mede, e é o que impede a sonda de virar um gate de último bit.
#[test]
fn com_a_corrente_em_um_o_osso_aponta_para_o_alvo() {
    for alvo in [
        [30.0, 0.0],
        [0.0, 30.0],
        [-20.0, 15.0],
        [5.0, -25.0],
        [3.0, 2.0],
    ] {
        let erro = erro_de_mira(1, alvo);
        assert!(
            erro < 1e-3,
            "com a corrente em UM o osso tinha de apontar a {alvo:?} e errou {erro:.6}°"
        );
    }
}

/// ⚠️ **O CONTROLO: com a corrente de fábrica (`2`) isto NÃO é apontar.**
///
/// Num esqueleto de um osso só a corrente real é `min(chain, cadeia)` = 1, logo as duas leituras
/// coincidem — *e é por isso que o controlo tem de ser num BRAÇO*, onde a corrente de `2` resolve
/// o par pela lei dos cossenos e a ponta do ombro vai parar a outro sítio.
///
/// ⛔ Sem esta metade, a sonda acima leria «aponta» sobre uma cena em que nada podia deixar de
/// apontar, e a conclusão — *o motor já existe* — seria fabricada.
#[test]
fn num_braco_a_corrente_de_dois_nao_e_apontar() {
    let (mut sim, [ombro, cotovelo]) = crate::goal::braco();
    let t = crate::goal::add(&mut sim, cotovelo).expect("a ancora nasce");
    // ⚠️ Os dois literais já são `f32` — não há conversão a isentar aqui, ao contrário do
    // `erro_de_mira`, que recebe o alvo em `f64`.
    if let Some(mut tt) = sim.world_mut().get_mut::<Transform>(t) {
        tt.translation = ph2d_core::Vec2::new(0.0_f32, 12.0_f32);
    }
    let mut pv = ph2d_preview_drive::PreviewDrive::default();
    crate::goal::solve(&mut sim, &mut pv);
    let ponta_do_ombro = crate::bone_pick::tip_of(&sim, ombro.to_bits()).expect("o ombro existe");
    let querido = 12.0_f64.atan2(0.0);
    let teve = ponta_do_ombro[1].atan2(ponta_do_ombro[0]);
    let erro = ph2d_skeleton::wrap_pi(teve - querido).abs().to_degrees();
    assert!(
        erro > 1.0,
        "a corrente de DOIS dobrou o cotovelo mas o ombro ficou a apontar ao alvo ({erro:.6}°): a \
         sonda irma esta' a medir uma cena onde tudo aponta, e a conclusao dela seria fabricada"
    );
}

/// ⭐⭐⭐ **O DESVIO EXISTE — e esta redacção substitui uma cuja PREMISSA MORREU no mesmo dia.**
///
/// A 1.ª versão deste teste dizia *«o desvio NÃO existe: o `IkGoal` tem cinco campos e nenhum é um
/// ângulo de offset»*, e mandava, por escrito, que ela morresse quando ele entrasse. Ele entrou —
/// e a morte fica visível no diff, que é a lei desta casa sobre premissas.
///
/// ⚠️ **O que ela afirma agora é a AUSÊNCIA que a justificou:** sem desvio, *«a cabeça olha para a
/// bola»* só funciona se o osso da cabeça tiver sido desenhado exactamente sobre o eixo em que ele
/// deve olhar — *uma condição de DESENHO a fingir de lei*. Medido: com o desvio a `30°`, o eixo do
/// osso fica **`30,0000°`** do alvo, e não sobre ele.
#[test]
fn o_desvio_roda_o_olhar_em_relacao_ao_eixo() {
    let alvo = [0.0, 30.0];
    let sem = pose_apontada(alvo, |_| {});
    for graus in [30.0_f64, -45.0, 180.0] {
        let com = pose_apontada(alvo, |g| g.offset = graus.to_radians());
        let d = ph2d_skeleton::wrap_pi(f64::from(com) - f64::from(sem)).to_degrees();
        assert!(
            (d - graus).abs() < 1e-3,
            "o desvio pedia {graus}° e o osso rodou {d:.6}°"
        );
    }
}

/// ⛔⛔ **E a MISTURA continua a desligar tudo, desvio incluído** — a metade que prende a ORDEM.
///
/// ⚠️ O desvio entra **antes** da mistura e do limite, e a ordem é a que os dois já declaram: a
/// mistura interpola entre a pose autorada e *o que a restrição quer*, e o limite apara *o que a
/// restrição pediu*. ⛔ Somá-lo **depois** faria o `Mix = 0` deixar de devolver a pose autorada —
/// o artista desligaria a restrição e o osso continuaria rodado, sem nada que o explicasse.
#[test]
fn com_a_mistura_a_zero_o_desvio_tambem_se_desliga() {
    let alvo = [0.0, 30.0];
    let parada = pose_apontada(alvo, |g| {
        g.mix = 0.0;
        g.offset = 60.0_f64.to_radians();
    });
    let autorada = pose_apontada(alvo, |g| g.mix = 0.0);
    assert_eq!(
        parada, autorada,
        "com a MISTURA a zero o desvio ainda rodou o osso: ele entra DEPOIS da mistura, e desligar \
         a restricao deixou de devolver a pose autorada"
    );
}

/// ⛔⛔ **E ele NÃO alcança uma corrente que ALCANÇA** — a cerca que faz a wave inteira ser segura.
///
/// Somar um desvio a uma corrente de dois ossos quebraria o alcance que ela acabou de resolver: a
/// mão deixaria de tocar aquilo que a restrição existe para tocar. ⇒ o campo é lido **só** quando a
/// corrente resolvida é de UM (`ph2d_skeleton_live::goal::aponta_com`).
///
/// ⚠️ **Sem esta metade, a de cima passaria com o desvio a alcançar tudo** — e o defeito só
/// apareceria num rig com braços.
#[test]
fn o_desvio_nao_toca_numa_corrente_que_alcanca() {
    let mover = |offset: f64| {
        let (mut sim, [_, cotovelo]) = crate::goal::braco();
        let t = crate::goal::add(&mut sim, cotovelo).expect("a ancora nasce");
        if let Some(mut g) = sim
            .world_mut()
            .get_mut::<ph2d_skeleton_ecs::IkGoal>(cotovelo)
        {
            g.offset = offset;
        }
        if let Some(mut tt) = sim.world_mut().get_mut::<Transform>(t) {
            tt.translation = ph2d_core::Vec2::new(4.0, 9.0);
        }
        let mut pv = ph2d_preview_drive::PreviewDrive::default();
        crate::goal::solve(&mut sim, &mut pv);
        crate::bone_pick::tip_of(&sim, cotovelo.to_bits()).expect("a ponta")
    };
    let sem = mover(0.0);
    let com = mover(45.0_f64.to_radians());
    assert_eq!(
        sem, com,
        "o desvio mexeu numa corrente de DOIS: a mao deixou de tocar o alvo que a restricao existe \
         para tocar"
    );
}

/// ⭐⭐ **E o verbo *Look At* nasce a APONTAR** — a corrente em UM, e a porta a confirmá-lo.
///
/// ⚠️ **As duas metades são dois defeitos:** um `add_look_at` que escrevesse a corrente de fábrica
/// seria um *Add IK* com outro rótulo; e um que não passasse pela porta do irmão perderia o alvo,
/// a marca e a semente do id.
#[test]
fn o_verbo_look_at_nasce_a_apontar() {
    let (mut sim, e) = um_osso();
    let alvo = ph2d_skeleton_live::goal::add_look_at(&mut sim, e).expect("o verbo cria o alvo");
    assert_eq!(
        sim.world()
            .get::<ph2d_skeleton_ecs::IkGoal>(e)
            .map(|g| g.chain),
        Some(1),
        "o Look At nasceu com outra corrente: ele e' um Add IK com outro rotulo"
    );
    assert!(
        ph2d_skeleton_live::goal::aponta(&sim, e),
        "a porta diz que esta ancora ALCANCA: o painel esconderia o desvio e o solver ignora'-lo-ia"
    );
    assert!(
        sim.world()
            .get::<ph2d_skeleton_ecs::IkTarget>(alvo)
            .is_some(),
        "o alvo nasceu sem a marca: ele apareceria com o anel do objecto vazio"
    );
}

/// ⭐⭐⭐ **CRIAR O APONTAR NÃO MOVE UM PIXEL** — a lei da casa (*todo motor novo é no-op no ponto
/// neutro*), e aqui ela sai de graça: o alvo nasce **na ponta do osso**, logo o osso já aponta
/// para ele.
#[test]
fn criar_o_apontar_nao_move_nada() {
    let (mut sim, e) = um_osso();
    let antes = ph2d_skeleton_live::skin_live::bone_segments(&sim);
    ph2d_skeleton_live::goal::add_look_at(&mut sim, e).expect("o verbo");
    let mut pv = ph2d_preview_drive::PreviewDrive::default();
    crate::goal::solve(&mut sim, &mut pv);
    assert_eq!(
        antes,
        ph2d_skeleton_live::skin_live::bone_segments(&sim),
        "criar o apontar moveu o osso: o artista perderia a pose que acabou de fazer"
    );
}

/// ⭐⭐⭐ **E QUAIS KNOBS DA ÂNCORA FICAM MORTOS COM A CORRENTE EM UM** — medido, não presumido.
///
/// | knob | com a corrente em `1` |
/// |---|---|
/// | *IK Mix* | **VIVO** (`0` devolve a pose autorada, `1` aponta) |
/// | *IK Softness* | **MORTO** — a lei de um osso põe a ponta a `comprimento` na direcção, e a
///   distância amortecida nunca entra na conta |
/// | *IK Bend* | **MORTO** — não há cotovelo, logo não há lado para dobrar |
///
/// ⛔ *Esconder um knob vivo e esconder um knob morto leem-se igual numa tabela; o que os separa é
/// a medição escrita ao lado* — e é esta. Sem ela, o painel do apontar ou promete dois controlos
/// que não fazem nada, ou apaga dois que fazem.
#[test]
fn com_a_corrente_em_um_a_suavidade_e_o_lado_sao_inertes_e_a_mistura_nao() {
    let alvo = [0.0, 30.0];
    let base = pose_apontada(alvo, |_| {});
    let com_suavidade = pose_apontada(alvo, |g| g.softness = 0.9);
    assert_eq!(
        base, com_suavidade,
        "a SUAVIDADE moveu o osso com a corrente em UM: ela nao e' inerte, e escondê-la apagaria \
         um controlo vivo"
    );
    for lado in ph2d_skeleton::BendSide::ALL {
        let com_lado = pose_apontada(alvo, |g| g.bend = lado);
        assert_eq!(
            base, com_lado,
            "o LADO DA DOBRA ({lado:?}) moveu o osso com a corrente em UM: ele nao e' inerte"
        );
    }
    let com_mistura = pose_apontada(alvo, |g| g.mix = 0.0);
    assert_ne!(
        base, com_mistura,
        "a MISTURA ficou inerte: ela e' o unico dos tres que continua a mandar, e sem ela o \
         apontar nao se desliga"
    );
}

/// A rotação do osso depois de um quadro, com a âncora afinada por `afina`.
fn pose_apontada(alvo: [f64; 2], afina: impl FnOnce(&mut ph2d_skeleton_ecs::IkGoal)) -> f32 {
    let (mut sim, e) = um_osso();
    let t = crate::goal::add(&mut sim, e).expect("a ancora nasce");
    if let Some(mut g) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::IkGoal>(e) {
        g.chain = 1;
        afina(&mut g);
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o `Transform` da casa é f32; a geometria do documento é f64"
    )]
    if let Some(mut tt) = sim.world_mut().get_mut::<Transform>(t) {
        tt.translation = ph2d_core::Vec2::new(alvo[0] as f32, alvo[1] as f32);
    }
    let mut pv = ph2d_preview_drive::PreviewDrive::default();
    crate::goal::solve(&mut sim, &mut pv);
    sim.world().get::<Transform>(e).expect("Transform").rotation
}
