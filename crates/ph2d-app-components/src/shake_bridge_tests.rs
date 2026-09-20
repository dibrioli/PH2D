//! Os gates da ponte do abanão.
//!
//! ⚠️ **Todos entram pela PORTA DO PRODUTO** ([`super::drive_camera_shake`]) e medem o OFFSET ou o
//! TRAUMA — nunca uma tabela de leis. *Uma tabela é um resumo do produto, e um resumo não tem de
//! conter tudo* (a lição que o `Clay Strips` da escultura pagou).

use super::*;
use ph2d_ecs::{GameCamera, Name, ShakeSource, SignalFrom, StableId};
use ph2d_runtime::Signal;

const DT: f32 = 1.0 / 60.0;

/// Uma cena com câmera em `(0,0)` que treme, e nada mais.
fn cena() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let cam = sim
        .world_mut()
        .spawn((
            Name::new("Camera"),
            Transform::IDENTITY,
            StableId(1),
            GameCamera::default(),
            CameraShake::default(),
        ))
        .id();
    (sim, cam)
}

/// Põe uma bomba em `x` com uma fonte que ouve `nome`.
fn bomba(sim: &mut SimWorld, id: u64, x: f32, nome: &str, de: SignalFrom) -> Entity {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    sim.world_mut()
        .spawn((
            Name::new(format!("Bomba{id}")),
            t,
            StableId(id),
            ShakeEmitter(vec![ShakeSource {
                on: nome.to_string(),
                de,
                ..ShakeSource::default()
            }]),
        ))
        .id()
}

/// Publica e corre um quadro.
fn quadro(sim: &mut SimWorld, out: &mut SignalOutbox, r: &mut SignalReader) -> AbanaoReport {
    drive_camera_shake(sim, out, r, DT)
}

// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **O caminho de omissão é BYTE-IDÊNTICO ao de antes desta wave**, e a régua tem as DUAS
/// metades: sem câmera **e** com câmera sem o componente.
#[test]
fn sem_abanao_a_vista_nao_se_mexe_e_o_cursor_nao_anda() {
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::new();
    out.publish(Signal::from_control("boom"));

    // (a) Cena sem câmera nenhuma.
    let mut sim = SimWorld::new();
    assert_eq!(quadro(&mut sim, &mut out, &mut r), AbanaoReport::default());
    assert_eq!(
        r.missed(),
        0,
        "o cursor não pode acumular perdas numa cena sem câmera"
    );

    // (b) Câmera SEM o componente — a cena de todo projecto que já existe.
    let (mut sim, cam) = cena();
    sim.world_mut().entity_mut(cam).remove::<CameraShake>();
    bomba(&mut sim, 2, 0.0, "boom", SignalFrom::Anyone);
    assert_eq!(quadro(&mut sim, &mut out, &mut r), AbanaoReport::default());
}

/// **Um estrondo abana a vista.** ⚠️ E a régua é o OFFSET e não o trauma: *um trauma que não chega
/// a pixel nenhum lê-se exactamente como o abanão a funcionar.*
#[test]
fn um_estrondo_mexe_a_vista() {
    let (mut sim, _cam) = cena();
    bomba(&mut sim, 2, 0.0, "boom", SignalFrom::Anyone);
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_control("boom"));

    let rep = quadro(&mut sim, &mut out, &mut r);
    assert_eq!(rep.impulsos, 1, "o impulso tem de CHEGAR");
    assert!(rep.trauma > 0.5, "trauma {:.3}", rep.trauma);
    // O offset move-se ao longo de alguns quadros (o ruído pode passar por zero num instante).
    let mut maior: f32 = rep.offset[0].abs().max(rep.offset[1].abs());
    for _ in 0..20 {
        let rep = quadro(&mut sim, &mut out, &mut r);
        maior = maior.max(rep.offset[0].abs()).max(rep.offset[1].abs());
    }
    assert!(maior > 0.05, "a vista mal se mexeu: {maior:.4} m");
}

/// ⭐⭐⭐ **A cerca de quem falou é a razão de o `SignalFrom` estar aqui:** dez bombas iguais, UMA
/// explode, e só ela abana. ⛔ Sem ela a única cena que este componente serve bem é a que tem um
/// objecto só — o defeito que a sonda do suplente #24 já tinha medido para a tabela de acções.
#[test]
fn dez_bombas_iguais_e_so_uma_abana() {
    // (a) `Myself`: dez bombas, o estrondo é da terceira.
    let (mut sim, _cam) = cena();
    let mut ids = Vec::new();
    for i in 0..10u64 {
        ids.push(bomba(&mut sim, 10 + i, 0.0, "boom", SignalFrom::Myself));
    }
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_death("boom", ids[2].to_bits()));
    let fechado = quadro(&mut sim, &mut out, &mut r);
    assert_eq!(fechado.impulsos, 1, "só a bomba que gritou pode contar");

    // (b) O CONTROLO: as mesmas dez com a cerca aberta contam dez.
    let (mut sim, _cam) = cena();
    let mut ids = Vec::new();
    for i in 0..10u64 {
        ids.push(bomba(&mut sim, 10 + i, 0.0, "boom", SignalFrom::Anyone));
    }
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_death("boom", ids[2].to_bits()));
    let aberto = quadro(&mut sim, &mut out, &mut r);
    assert_eq!(aberto.impulsos, 10, "com a cerca aberta todas ouvem");
}

/// ⚠️ **Um sinal SEM sujeito nunca passa uma cerca fechada** — `None` não é curinga. Lê-lo como
/// *«qualquer um»* faria uma cerca FECHADA deixar passar tudo, que é o modo de falha mais caro que
/// uma cerca pode ter.
#[test]
fn um_sinal_sem_sujeito_nao_passa_a_cerca_fechada() {
    let (mut sim, _cam) = cena();
    bomba(&mut sim, 2, 0.0, "boom", SignalFrom::Myself);
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    // ⚠️ O `Control` é uma das TRÊS origens sem sujeito (medido na sonda do §5.0).
    out.publish(Signal::from_control("boom"));
    let rep = quadro(&mut sim, &mut out, &mut r);
    assert_eq!(rep.impulsos, 0);
    assert_eq!(rep.trauma, 0.0);
}

/// ⭐⭐ **A distância ATENUA**, e a régua mede o trauma dos dois lados numa cena só de cada vez.
#[test]
fn a_mesma_explosao_abana_menos_de_longe() {
    fn trauma_a(x: f32) -> f32 {
        let (mut sim, _cam) = cena();
        bomba(&mut sim, 2, x, "boom", SignalFrom::Anyone);
        let mut out = SignalOutbox::new();
        let mut r = SignalReader::at(&out);
        out.publish(Signal::from_control("boom"));
        quadro(&mut sim, &mut out, &mut r).trauma
    }
    let perto = trauma_a(0.0);
    let meio = trauma_a(7.0);
    let longe = trauma_a(20.0);
    assert!(
        perto > meio,
        "perto {perto:.4} tem de abanar mais que meio {meio:.4}"
    );
    assert!(meio > 0.0, "a meio caminho ainda tem de abanar: {meio:.4}");
    assert_eq!(longe, 0.0, "para lá do raio externo não chega nada");
}

/// ⚠️ **«ninguém gritou» e «gritaram longe demais» são DOIS factos** — sem a coluna `longe`, um
/// artista com o emissor ligado e a câmera longe lê exactamente o mesmo que um com o nome errado.
#[test]
fn longe_demais_conta_na_coluna_certa() {
    let (mut sim, _cam) = cena();
    bomba(&mut sim, 2, 40.0, "boom", SignalFrom::Anyone);
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_control("boom"));
    let rep = quadro(&mut sim, &mut out, &mut r);
    assert_eq!((rep.impulsos, rep.longe), (0, 1));

    // ⭐ O CONTROLO: um nome que NINGUÉM ouve não conta em coluna nenhuma.
    let (mut sim, _cam) = cena();
    bomba(&mut sim, 2, 40.0, "outro", SignalFrom::Anyone);
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_control("boom"));
    let rep = quadro(&mut sim, &mut out, &mut r);
    assert_eq!((rep.impulsos, rep.longe), (0, 0));
}

/// ⚠️ **Uma fonte com o nome VAZIO é calada** — o que uma acabada de anexar pela paleta é.
#[test]
fn uma_fonte_sem_nome_e_calada() {
    let (mut sim, _cam) = cena();
    bomba(&mut sim, 2, 0.0, "", SignalFrom::Anyone);
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_control(""));
    out.publish(Signal::from_control("boom"));
    let rep = quadro(&mut sim, &mut out, &mut r);
    assert_eq!(rep.impulsos, 0);
}

/// ⭐ **O abanão ACABA, e a vista volta EXACTAMENTE ao sítio.** ⚠️ As duas metades: ele tem de
/// acabar (senão a vista nunca assenta) e tem de acabar em `[0, 0]` **ao bit** (senão toda cena com
/// uma câmera fica com um desvio permanente depois do primeiro estrondo).
#[test]
fn o_abanao_acaba_e_a_vista_volta_ao_sitio() {
    let (mut sim, _cam) = cena();
    bomba(&mut sim, 2, 0.0, "boom", SignalFrom::Anyone);
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_control("boom"));

    // O de fábrica dura `1 / 2,0` = meio segundo ⇒ 30 quadros a 60 Hz.
    let mut ultimo = quadro(&mut sim, &mut out, &mut r);
    for _ in 0..40 {
        ultimo = quadro(&mut sim, &mut out, &mut r);
    }
    assert_eq!(ultimo.trauma, 0.0, "o trauma tem de chegar a ZERO");
    assert_eq!(
        ultimo.offset,
        [0.0, 0.0],
        "e a vista tem de voltar ao sítio"
    );
}

/// ⭐⭐⭐ **REBOBINAR É RENASCER** — a metade que o [`ph2d_ecs::rewind_runtime`] paga. ⛔ Sem ela,
/// rebobinar a meio de uma explosão deixaria a vista a acabar de tremer o abanão da corrida
/// anterior, com o relógio no zero.
#[test]
fn rebobinar_apaga_o_trauma_e_o_relogio() {
    let (mut sim, cam) = cena();
    bomba(&mut sim, 2, 0.0, "boom", SignalFrom::Anyone);
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_control("boom"));
    for _ in 0..5 {
        quadro(&mut sim, &mut out, &mut r);
    }
    let vivo = *sim.world().get::<CameraShakeRuntime>(cam).unwrap();
    assert!(
        vivo.trauma > 0.0 && vivo.t > 0.0,
        "o arranjo tem de conter o fenómeno: {vivo:?}"
    );

    let tocados = ph2d_ecs::rewind_runtime::rewind_runtime_state(
        sim.world_mut(),
        ph2d_ecs::rewind_runtime::Renascimento::Rebobinar,
    );
    assert!(tocados > 0);
    let depois = *sim.world().get::<CameraShakeRuntime>(cam).unwrap();
    assert_eq!(depois, CameraShakeRuntime::default(), "{depois:?}");
    // ⭐ E a vista volta ao sítio no quadro seguinte.
    assert_eq!(quadro(&mut sim, &mut out, &mut r).offset, [0.0, 0.0]);
}

/// ⭐⭐ **Dois estrondos seguidos NÃO têm a mesma cara**, e é o relógio que não se repõe que o
/// compra. ⚠️ Sem esta propriedade o abanão é uma animação enlatada: a mesma explosão dez vezes
/// desenharia sempre o mesmo caminho.
#[test]
fn dois_estrondos_seguidos_abanam_de_maneiras_diferentes() {
    fn um_estrondo(
        sim: &mut SimWorld,
        out: &mut SignalOutbox,
        r: &mut SignalReader,
    ) -> Vec<[f32; 2]> {
        out.publish(Signal::from_control("boom"));
        (0..30).map(|_| quadro(sim, out, r).offset).collect()
    }
    let (mut sim, _cam) = cena();
    bomba(&mut sim, 2, 0.0, "boom", SignalFrom::Anyone);
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);

    let primeiro = um_estrondo(&mut sim, &mut out, &mut r);
    // Deixa assentar por completo.
    for _ in 0..40 {
        quadro(&mut sim, &mut out, &mut r);
    }
    let segundo = um_estrondo(&mut sim, &mut out, &mut r);

    // ⚠️⚠️ **A 1.ª redacção desta régua CONTAVA AS CAUDAS DE ZEROS e reprovava sobre produto
    // correcto** (`13` de `30` idênticos): a `forca` de fábrica é `0,6`, logo um estrondo dura
    // `0,6 / 2,0 = 0,3 s` = **18** quadros, e os `12` seguintes são `[0, 0]` **nos dois** — *uma
    // régua que compara caudas de zeros mede o DECAIMENTO, não a fase.* ⇒ ela compara só onde as
    // duas se mexem, **com piso de população**: sem o piso, uma lei que não abanasse nada daria
    // conjunto vazio e o gate ficaria verde por VÁCUO.
    let vivos: Vec<_> = primeiro
        .iter()
        .zip(&segundo)
        .filter(|(a, b)| **a != [0.0, 0.0] && **b != [0.0, 0.0])
        .collect();
    assert!(
        vivos.len() >= 15,
        "o arranjo tem de CONTER o fenómeno: só {} quadros com as duas a abanar",
        vivos.len()
    );
    let iguais = vivos.iter().filter(|(a, b)| a == b).count();
    assert!(
        iguais < 3,
        "{iguais} de {} quadros idênticos — o abanão está enlatado",
        vivos.len()
    );
}

/// ⚠️ **O trauma SATURA:** vinte estrondos no mesmo quadro não abanam vinte vezes mais.
#[test]
fn vinte_estrondos_juntos_nao_abanam_vinte_vezes_mais() {
    let (mut sim, _cam) = cena();
    for i in 0..20u64 {
        bomba(&mut sim, 10 + i, 0.0, "boom", SignalFrom::Anyone);
    }
    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_control("boom"));
    let rep = quadro(&mut sim, &mut out, &mut r);
    assert_eq!(rep.impulsos, 20);
    assert_eq!(rep.trauma.max(0.0), ph2d_shake::decai(1.0, 2.0, DT));
    let pico = rep.offset[0].abs().max(rep.offset[1].abs());
    assert!(
        pico <= CameraShake::default().amplitude + 1e-6,
        "a vista saltou {pico:.3} m contra a amplitude declarada"
    );
}

/// ⭐⭐ **A distância sai de QUEM GRITOU quando o sinal traz o sujeito** — uma bomba que ouve o
/// estrondo de outra abana a partir de **onde a outra está**, não de onde ela própria está.
#[test]
fn a_distancia_sai_de_quem_gritou_e_nao_do_ouvinte() {
    let (mut sim, _cam) = cena();
    // O OUVINTE está em cima da câmera; quem grita está muito longe.
    bomba(&mut sim, 2, 0.0, "boom", SignalFrom::Anyone);
    let mut t = Transform::IDENTITY;
    t.translation.x = 40.0;
    let longe = sim
        .world_mut()
        .spawn((Name::new("Longe"), t, StableId(3)))
        .id();

    let mut out = SignalOutbox::new();
    let mut r = SignalReader::at(&out);
    out.publish(Signal::from_death("boom", longe.to_bits()));
    let rep = quadro(&mut sim, &mut out, &mut r);
    assert_eq!(
        (rep.impulsos, rep.longe),
        (0, 1),
        "o estrondo é a 40 m: ele não pode chegar por o OUVINTE estar perto"
    );
}
