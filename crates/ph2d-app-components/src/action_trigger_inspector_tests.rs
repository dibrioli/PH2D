//! Os gates da PONTE do gatilho — a tradução de ida-e-volta e a coluna da acção órfã.

use super::*;

/// ⭐⭐ **A tradução é uma BIJECÇÃO** — e a metade da VOLTA é o que prova que toda aresta que a lei
/// resolve é **alcançável** pelo painel. Sem ela, um `_ => Press` no caminho de ida passaria.
#[test]
fn a_traducao_da_aresta_fecha_nos_dois_sentidos() {
    for (i, e) in ActionEdge::ALL.iter().enumerate() {
        let v = u8::try_from(i).expect("três cabem num u8");
        assert_eq!(edge_de_u8(v), *e, "ida: o índice {v} tem de dar {e:?}");
        assert_eq!(u8_de_edge(*e), v, "volta: {e:?} tem de dar o índice {v}");
    }
    assert_eq!(
        arestas(),
        ActionEdge::ALL.len(),
        "a porta que o gate da shell lê deixou de contar a população real"
    );
}

/// ⚠️⚠️ **Um nome VAZIO não é «desconhecido»**, e a diferença tem consequência visível: contá-lo
/// como órfão poria o título a dizer «1 broken» num gatilho acabado de acrescentar, que é o estado
/// normal de quem está a escrever. O painel tem uma frase própria para o vazio.
///
/// ⭐⭐⭐ **E o estado do meio é o que este gate passou a afirmar (2026-09-18):** uma acção que
/// EXISTE e não tem tecla nenhuma fica tão calada como um nome errado — e ela **não conta como
/// órfã**, porque a cura é outra (ligar uma tecla, não criar a acção). *Duas causas, o mesmo
/// silêncio, dois avisos.*
#[test]
fn um_nome_vazio_nao_conta_como_orfao_e_um_desconhecido_conta() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn(SignalOnAction(vec![
            ActionTriggerRow {
                action: String::new(),
                ..Default::default()
            },
            ActionTriggerRow {
                action: "fire".into(),
                ..Default::default()
            },
            ActionTriggerRow {
                action: "fier".into(),
                ..Default::default()
            },
            ActionTriggerRow {
                action: "grab".into(),
                ..Default::default()
            },
        ]))
        .id();
    let info = build_info(&sim, e.to_bits(), true, 1, &|n| match n {
        "fire" => NoMapa::Ligada,
        "grab" => NoMapa::SemTecla,
        _ => NoMapa::Desconhecida,
    })
    .expect("tem o componente");
    let estados: Vec<NoMapa> = info.rows.iter().map(|r| r.no_mapa).collect();
    assert_eq!(
        estados,
        [
            NoMapa::Ligada,
            NoMapa::Ligada,
            NoMapa::Desconhecida,
            NoMapa::SemTecla
        ],
        "o vazio lê-se como Ligada (não acusa nada); o resto é o que o mapa disse"
    );
    assert_eq!(info.orfas(), 1, "só a linha com o nome errado é órfã");
    assert_eq!(info.sem_tecla(), 1, "e só a `grab` está por ligar");
    // ⛔ **O CONTROLO da porta `fala()`:** ela tem de ser `false` nos DOIS estados mudos — senão
    // um consumidor que só queira *«isto vai funcionar?»* aprova metade do silêncio.
    let fala: Vec<bool> = estados.iter().map(|e| e.fala()).collect();
    assert_eq!(fala, [true, true, false, false]);
}

/// ⭐ **Escrever o MESMO valor não é uma mudança** — devolver `true` aqui faria cada quadro com o
/// campo focado marcar o componente como sujo, e o undo regista por diff.
#[test]
fn reescrever_o_mesmo_valor_nao_suja_o_mundo() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn(SignalOnAction(vec![ActionTriggerRow {
            action: "fire".into(),
            edge: ActionEdge::Press,
            signal: "shoot".into(),
        }]))
        .id();
    let bits = e.to_bits();
    assert!(
        !apply_all(&mut sim, &[(bits, E::Action(0, "fire".into()))]),
        "o mesmo nome não é uma mudança"
    );
    assert!(
        !apply_all(&mut sim, &[(bits, E::Edge(0, 0))]),
        "a mesma aresta não é uma mudança"
    );
    // ⚠️ **Os TRÊS campos, e não dois:** a guarda é escrita uma vez por braço, logo um gate que
    // cubra dois deixa o terceiro sem régua — e foi assim que a mutação do `signal` sobreviveu à
    // 1.ª redacção deste ficheiro.
    assert!(
        !apply_all(&mut sim, &[(bits, E::Signal(0, "shoot".into()))]),
        "o mesmo sinal não é uma mudança"
    );
    // O controlo, também nos três: um valor DIFERENTE muda.
    assert!(
        apply_all(&mut sim, &[(bits, E::Edge(0, 2))]),
        "o controlo: outra aresta TEM de sujar o mundo"
    );
    assert!(
        apply_all(&mut sim, &[(bits, E::Action(0, "jump".into()))]),
        "o controlo: outro nome de acção TEM de sujar o mundo"
    );
    assert!(
        apply_all(&mut sim, &[(bits, E::Signal(0, "bang".into()))]),
        "o controlo: outro sinal TEM de sujar o mundo"
    );
}

/// ⚠️ **O tecto mora na ponte e não só no painel** — uma edição pode vir de um barramento drenado
/// tarde, quando o `+` já não é pintado.
#[test]
fn o_tecto_do_modelo_e_honrado_pela_ponte() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(SignalOnAction(Vec::new())).id();
    let bits = e.to_bits();
    for _ in 0..ACTION_TRIGGERS_MAX {
        assert!(apply_all(&mut sim, &[(bits, E::Add)]));
    }
    assert!(
        !apply_all(&mut sim, &[(bits, E::Add)]),
        "a ponte deixou passar do tecto do modelo"
    );
    let n = sim
        .world()
        .get::<SignalOnAction>(e)
        .expect("componente")
        .0
        .len();
    assert_eq!(n, ACTION_TRIGGERS_MAX);
}
