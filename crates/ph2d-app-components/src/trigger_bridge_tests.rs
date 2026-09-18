//! Ver o cabeçalho de [`super`].

use super::amostras_das_accoes;
use ph2d_input::{ActionState, Binding, InputMap, InputState, Key};

/// O `Q`, que é a tecla que o smoke liga — ver [`crate::trigger_smoke::TECLA`].
const TECLA: Key = Key(0x51);
const ACCAO: &str = "fire";

/// Um mapa com UMA acção ligada à tecla. ⚠️ **Não é o `with_player_defaults`** — ele tem sete
/// acções e nenhuma é disparar, que é exactamente a medição que obriga o prólogo a criar esta.
fn mapa() -> InputMap {
    let mut m = InputMap::new();
    let id = m.create(ACCAO);
    if let Some(a) = m.get_mut(id) {
        a.bindings.push(Binding::Key(TECLA));
    }
    m
}

/// O estado das acções depois de resolver um tique por entrada de `premido`, do repouso.
///
/// ⚠️ **O percurso é uma SEQUÊNCIA e nunca um instante:** o `just_pressed` é a diferença entre o
/// tique de agora e o anterior, logo uma resolução só mede uma borda contra o VAZIO e não distingue
/// *«acabou de carregar»* de *«já estava em baixo»* — que é a propriedade inteira do `Press`.
fn estado(premido: &[bool]) -> ActionState {
    let m = mapa();
    let mut dev = InputState::new();
    let mut st = ActionState::new();
    for &p in premido {
        dev.begin_frame();
        if p {
            dev.keyboard.handle_key_down(TECLA);
        } else {
            dev.keyboard.handle_key_up(TECLA);
        }
        st.tick(&m, &dev);
    }
    st
}

/// ⭐⭐⭐ **A amostra chega do MAPA, com as três leituras certas, ao longo de um TOQUE INTEIRO.**
///
/// ⚠️⚠️ **O percurso tem quatro paragens de propósito, e cada uma separa DUAS leituras que num
/// instante só se lêem iguais:** premir (`pressed` = `just_pressed`), **segurar** (é aqui que essas
/// duas se separam), largar (`just_released` = `!pressed`) e **ficar solto** (é aqui que *estas* se
/// separam). *Uma régua medida num instante só aprova trocar uma leitura pela outra.*
///
/// ⚠️ **E a metade NEGATIVA é metade do valor:** sem ela a varredura podia devolver uma entrada
/// para TODA acção que alguém nomeasse, e um `Release` sobre um nome desconhecido dispararia em
/// **todo quadro** (`!pressed` é trivialmente verdade).
#[test]
fn as_amostras_saem_do_mapa_e_so_do_mapa() {
    // (o que se faz, as teclas até aqui, o esperado: pressed · just_pressed · just_released)
    let percurso = [
        ("premir", &[false, true][..], (true, true, false)),
        ("segurar", &[false, true, true][..], (true, false, false)),
        (
            "largar",
            &[false, true, true, false][..],
            (false, false, true),
        ),
        (
            "ficar solto",
            &[false, true, true, false, false][..],
            (false, false, false),
        ),
    ];
    for (nome, teclas, esperado) in percurso {
        let a = amostras_das_accoes(&mapa(), &estado(teclas));
        let s = a
            .get(ACCAO)
            .copied()
            .expect("a accao do mapa tem de estar la'");
        assert_eq!(
            (s.pressed, s.just_pressed, s.just_released),
            esperado,
            "{nome}: as tres leituras"
        );
    }

    let a = amostras_das_accoes(&mapa(), &estado(&[false, true]));
    assert!(
        !a.contains_key("fier"),
        "uma accao que o mapa nao conhece NAO pode ter entrada — senao o `Release` dela fala sempre"
    );
    assert_eq!(
        a.len(),
        1,
        "o mapa tem uma accao, a varredura tem de ter uma"
    );
}

/// ⭐⭐⭐ **O MAPA responde TRÊS coisas, e as duas mudas não são a mesma.**
///
/// ⚠️⚠️ **O `SemTecla` é o estado que faltava, e o gate prova as DUAS metades dele:** a acção
/// existe (logo não é órfã, e a cura não é criá-la) **e** ela não fala (as três leituras dão
/// `false`, tão calada como um nome errado). *Sem a segunda metade isto seria só uma etiqueta.*
#[test]
fn uma_accao_sem_tecla_existe_e_fica_calada_na_mesma() {
    use ph2d_editor_core::action_trigger_edits::NoMapa;

    let mut m = InputMap::new();
    let ligada = m.create("fire");
    if let Some(a) = m.get_mut(ligada) {
        a.bindings.push(Binding::Key(TECLA));
    }
    m.create("grab"); // declarada e por atribuir

    assert_eq!(super::no_mapa(&m, "fire"), NoMapa::Ligada);
    assert_eq!(super::no_mapa(&m, "grab"), NoMapa::SemTecla);
    assert_eq!(super::no_mapa(&m, "fier"), NoMapa::Desconhecida);

    // ⭐ A metade que torna o estado do meio uma LEI e não uma etiqueta: com a tecla em baixo, a
    // `grab` continua a ler `false` nas três — e é por isso que ela precisa de aviso próprio.
    let mut dev = InputState::new();
    let mut st = ActionState::new();
    for _ in 0..2 {
        dev.begin_frame();
        dev.keyboard.handle_key_down(TECLA);
        st.tick(&m, &dev);
    }
    let a = super::amostras_das_accoes(&m, &st);
    let g = a.get("grab").copied().expect("ela ESTÁ no mapa");
    assert_eq!(
        (g.pressed, g.just_pressed, g.just_released),
        (false, false, false),
        "uma accao sem ligacao nao fala — o mesmo silencio de um nome errado"
    );
    // O CONTROLO: a que tem tecla fala no mesmo instante.
    assert!(
        a.get("fire").copied().expect("no mapa").pressed,
        "controlo: sem isto o gate passaria com o teclado inerte"
    );
}
