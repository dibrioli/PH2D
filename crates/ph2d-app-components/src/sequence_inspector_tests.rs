//! Os gates do instantâneo e do dreno da secção SEQUENCE (TOP-20 #19, W3).

use ph2d_ecs::timer::{Timer, TimerRuntime, TimerState, Timers};
use ph2d_ecs::{SequencePlayer, SimWorld, Transform};
use ph2d_editor_core::sequence_edits::SequenceFieldEdit as E;

use super::{Cutscene, Relogios, apply, build_info};

/// A cena a correr, na vista que deixa a cutscene andar.
const A_CORRER: Relogios = Relogios {
    clock_playing: true,
    vista_deixa_correr: true,
};

/// As cutscenes do documento, com a ISCA à frente — a mesma lei da fixtura da fase: com **uma** só,
/// o índice é sempre `0` e uma resolução partida fica inobservável.
const CUTSCENES: [Cutscene<'static>; 2] = [
    Cutscene {
        nome: "Isca",
        duracao: 2.0,
    },
    Cutscene {
        // ⚠️⚠️ **Com o espaço, e ele é o gate.** A lei (`SequencePlayer::resolve`) apara os DOIS
        // lados, e um `==` cru escrito à mão no instantâneo responderia igual em toda fixtura
        // limpa — *um corpus sem o fenómeno aprova a mutação que o apaga*, e foi o que aconteceu
        // na 1.ª corrida da prova de W3.
        nome: "Porta ",
        duracao: 3.0,
    },
];

/// Um objecto que toca `nome`, com um relógio de `dur_s` segundos, decorrido `t_s`.
fn cena(nome: &str, dur_s: f64, t_s: f64, a_correr: bool) -> (SimWorld, u64) {
    let mut sim = SimWorld::default();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "fixtura"
    )]
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            SequencePlayer {
                container: nome.to_owned(),
            },
            Timers(vec![Timer {
                name: "clock".into(),
                duration_us: (dur_s * 1_000_000.0) as u64,
                repeat: false,
                autostart: false,
                signal: String::new(),
            }]),
            TimerRuntime(vec![TimerState {
                elapsed_us: (t_s * 1_000_000.0) as u64,
                running: a_correr,
            }]),
        ))
        .id();
    let bits = e.to_bits();
    (sim, bits)
}

/// ⛔ **Um objecto sem o componente não tem secção** — ADR-0166.
#[test]
fn um_objecto_sem_cutscene_nao_tem_seccao() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(Transform::default()).id();
    assert!(build_info(&sim, e.to_bits(), &CUTSCENES, A_CORRER, 1).is_none());
}

/// ⭐⭐ **O nome RESOLVE para o índice do documento** — e o gate corre com a ISCA à frente, senão
/// um `position` partido devolveria `0` e passaria.
///
/// **Mutação que deve sangrar:** trocar o `resolve` por `Some(0)`.
#[test]
fn o_nome_resolve_para_o_indice_do_documento() {
    let (sim, bits) = cena("Porta", 3.0, 1.2, true);
    let i = build_info(&sim, bits, &CUTSCENES, A_CORRER, 1).expect("tem secção");
    assert_eq!(i.escolhido, Some(1), "a «Porta» é a SEGUNDA do documento");
    assert!((i.duracao_da_cutscene - 3.0).abs() < 1e-9);
    assert!(!i.orfao());
}

/// ⛔ **Um nome que não existe fica ÓRFÃO e o nome FICA** — a cura é do artista, não do app.
///
/// **Mutação que deve sangrar:** limpar o `container` quando ele não resolve.
#[test]
fn um_nome_que_nao_existe_fica_orfao_com_o_nome_dentro() {
    let (sim, bits) = cena("Ponte", 3.0, 0.0, false);
    let i = build_info(&sim, bits, &CUTSCENES, A_CORRER, 1).expect("tem secção");
    assert_eq!(i.escolhido, None);
    assert!(i.orfao(), "um nome escrito que não resolve é um ÓRFÃO");
    assert_eq!(i.container, "Ponte", "o nome guardado não se apaga");
    assert!(
        (i.duracao_da_cutscene).abs() < 1e-9,
        "sem cutscene escolhida não há duração a mostrar"
    );
}

/// ⚠️ **Em branco NÃO é órfão** — as duas frases do painel são diferentes, e a cura também.
#[test]
fn em_branco_nao_e_orfao() {
    let (sim, bits) = cena("", 3.0, 0.0, false);
    let i = build_info(&sim, bits, &CUTSCENES, A_CORRER, 1).expect("tem secção");
    assert!(!i.orfao());
    assert_eq!(i.nome(), None);
}

/// ⭐⭐⭐ **O relógio mais curto que a cutscene é DITO** — ela nunca chega ao fim, e é o defeito
/// mais caro deste componente.
///
/// ⚠️ **Com o CONTROLO ao lado:** um relógio igual ou mais longo cala-se.
///
/// **Mutação que deve sangrar:** inverter a comparação, ou tirar a folga de um milissegundo.
#[test]
fn um_relogio_mais_curto_que_a_cutscene_e_dito() {
    let (sim, bits) = cena("Porta", 1.0, 0.0, true);
    let curto = build_info(&sim, bits, &CUTSCENES, A_CORRER, 1).expect("tem secção");
    assert!(
        curto.relogio_curto(),
        "1 s de relógio para 3 s de cutscene: ela nunca acaba"
    );

    let (sim, bits) = cena("Porta", 3.0, 0.0, true);
    let exacto = build_info(&sim, bits, &CUTSCENES, A_CORRER, 1).expect("tem secção");
    assert!(
        !exacto.relogio_curto(),
        "um empate EXACTO é o que o artista escreveu, não um aviso"
    );
}

/// ⛔⛔ **A FOLGA de um milissegundo é o gate, e a 1.ª redacção não a continha.**
///
/// ⚠️ Os dois números vêm de unidades diferentes — o relógio em microssegundos INTEIROS, o
/// container em `f64` de segundos somado de tempos racionais —, logo um empate que o artista
/// escreveu de propósito chega aqui como uma diferença de meio milissegundo. Sem a folga, ele
/// recebia um aviso sobre uma cutscene perfeita. *Um empate EXACTO não contém o fenómeno: a
/// mutação que apaga a folga sobrevive a ele.*
///
/// **Mutação que deve sangrar:** apagar o `+ 1e-3`.
#[test]
fn um_empate_a_menos_de_um_milissegundo_nao_e_um_aviso() {
    // O relógio tem 2,0000 s e a cutscene 2,0005 — meio milissegundo de arredondamento.
    let (sim, bits) = cena("Porta", 2.0, 0.0, true);
    let quase = [
        CUTSCENES[0],
        Cutscene {
            nome: "Porta ",
            duracao: 2.0005,
        },
    ];
    let i = build_info(&sim, bits, &quase, A_CORRER, 1).expect("tem secção");
    assert_eq!(i.escolhido, Some(1));
    assert!(
        !i.relogio_curto(),
        "meio milissegundo de arredondamento não é «o relógio acaba antes»"
    );
}

/// ⛔ **Sem cutscene escolhida a pergunta do relógio curto nem se põe** — senão todo objecto recém
/// anexado abriria com um aviso sobre uma cutscene que ele não toca.
#[test]
fn sem_cutscene_o_aviso_do_relogio_cala_se() {
    let (sim, bits) = cena("", 0.1, 0.0, true);
    let i = build_info(&sim, bits, &CUTSCENES, A_CORRER, 1).expect("tem secção");
    assert!(!i.relogio_curto());
}

/// ⚠️ **Sem `Timers` o objecto é INERTE, e o instantâneo di-lo** — o descritor exige o componente,
/// e o artista pode removê-lo.
#[test]
fn sem_timers_o_instantaneo_diz_que_nao_ha_relogio() {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            SequencePlayer {
                container: "Porta".into(),
            },
        ))
        .id();
    let i = build_info(&sim, e.to_bits(), &CUTSCENES, A_CORRER, 1).expect("tem secção");
    assert!(!i.tem_relogio);
    assert!(!i.a_correr);
    assert!(!i.relogio_curto(), "sem relógio não há relógio curto");
}

/// ⭐ **O instante VIVO é o do relógio `0`** — a mesma conversão do `em_corrida`, em `f64`.
#[test]
fn o_instante_vivo_e_o_do_relogio_zero() {
    let (sim, bits) = cena("Porta", 3.0, 1.25, true);
    let i = build_info(&sim, bits, &CUTSCENES, A_CORRER, 1).expect("tem secção");
    assert!((i.t - 1.25).abs() < 1e-9);
    assert!(i.a_correr);
}

/// ⭐⭐ **A edição escreve o NOME no componente.**
#[test]
fn a_edicao_escreve_o_nome_no_componente() {
    let (mut sim, bits) = cena("", 3.0, 0.0, false);
    assert!(apply(&mut sim, bits, &E::Container("Porta".into())));
    let e = ph2d_ecs::Entity::from_bits(bits);
    assert_eq!(
        sim.world().get::<SequencePlayer>(e).expect("tem").container,
        "Porta"
    );
}

/// ⛔⛔ **Escrever o MESMO nome não é uma mudança** — o `get_mut` do bevy marca o componente por
/// ser PEDIDO, e um clique na cutscene que já estava escolhida entraria no `Ctrl+Z`.
///
/// **Mutação que deve sangrar:** apagar a comparação de igualdade.
#[test]
fn escrever_o_mesmo_nome_nao_e_uma_mudanca() {
    let (mut sim, bits) = cena("Porta", 3.0, 0.0, false);
    assert!(!apply(&mut sim, bits, &E::Container("Porta".into())));
}

/// ⭐ **Largar a cutscene é escrever o vazio** — o caminho de volta, e ele é uma mudança.
#[test]
fn largar_a_cutscene_escreve_o_vazio() {
    let (mut sim, bits) = cena("Porta", 3.0, 0.0, false);
    assert!(apply(&mut sim, bits, &E::Container(String::new())));
    let e = ph2d_ecs::Entity::from_bits(bits);
    assert!(
        sim.world()
            .get::<SequencePlayer>(e)
            .expect("tem")
            .container
            .is_empty()
    );
}

/// ⛔ **Uma edição sobre quem não tem o componente não escreve nada.**
#[test]
fn uma_edicao_sobre_quem_nao_tem_o_componente_nao_escreve() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(Transform::default()).id();
    assert!(!apply(&mut sim, e.to_bits(), &E::Container("Porta".into())));
}
