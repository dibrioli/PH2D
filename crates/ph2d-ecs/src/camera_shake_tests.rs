//! Os gates dos dois componentes do abanão. ⚠️ A LEI tem a bancada dela na folha `ph2d-shake`;
//! aqui mede-se o que é **desta** crate: a porta para a lei, os valores de fábrica e a cerca.

use super::*;

/// ⭐ **A porta ÚNICA para a lei** — ida e volta, campo a campo. ⚠️ Escrita por `..Default`, ela
/// deixaria de reprovar no dia do sexto número; escrita **por nome**, um campo novo na `Lei` é
/// **erro de compilação** aqui.
#[test]
fn a_porta_para_a_lei_leva_os_cinco_numeros() {
    let c = CameraShake {
        amplitude: 0.5,
        frequencia: 13.0,
        decaimento: 3.0,
        expoente: 3,
        semente: 0xF00D,
    };
    let ph2d_shake::Lei {
        amplitude,
        frequencia,
        decaimento,
        expoente,
        semente,
    } = c.lei();
    assert_eq!(amplitude, 0.5);
    assert_eq!(frequencia, 13.0);
    assert_eq!(decaimento, 3.0);
    assert_eq!(expoente, 3);
    assert_eq!(semente, 0xF00D);
}

/// ⚠️ **Os valores de fábrica têm de ser UTILIZÁVEIS**, e cada asserção lê a tabela do doc.
#[test]
fn os_valores_de_fabrica_abanam_de_verdade() {
    let c = CameraShake::default();
    assert!(
        c.semente != 0,
        "`0` é o «por semear» do splitmix — uma semente nula é um defeito silencioso"
    );
    assert!(
        (ph2d_shake::EXPOENTE_MIN..=ph2d_shake::EXPOENTE_MAX).contains(&c.expoente),
        "o expoente de fábrica tem de estar na faixa que o painel oferece"
    );
    // ⭐ A régua é o BARRO: com trauma cheio a vista tem de se mexer de verdade.
    let l = c.lei();
    let mut maior: f32 = 0.0;
    for i in 0..600 {
        let [x, y] = ph2d_shake::deslocamento(&l, 1.0, i as f32 * 0.002);
        maior = maior.max(x.abs()).max(y.abs());
    }
    assert!(
        maior > c.amplitude * 0.8,
        "o abanão de fábrica mal chega a {maior:.3} m contra a amplitude {:.3}",
        c.amplitude
    );
    // ⚠️ …e a duração é a que o doc promete: `1 / decaimento`.
    let vida = 1.0 / c.decaimento;
    assert!(
        (vida - 0.5).abs() < 1e-6,
        "o abanão de fábrica dura {vida:.3} s e o doc diz meio segundo"
    );
}

/// ⚠️ **Uma fonte acabada de anexar é CALADA** — a lei dos contactos da física. Sem isto, anexar o
/// componente pela paleta poria a câmera a tremer com um sinal que ninguém escolheu.
#[test]
fn uma_fonte_nova_e_calada_e_o_emissor_nasce_vazio() {
    assert_eq!(ShakeEmitter::default().0.len(), 0);
    assert!(ShakeSource::default().on.is_empty());
    // ⭐ E o CONTROLO: os raios de fábrica dela não são degenerados.
    let s = ShakeSource::default();
    assert!(s.fora > s.dentro, "{:.1} contra {:.1}", s.fora, s.dentro);
    assert!(s.forca > 0.0);
}

/// ⭐⭐ **O que a distância de fábrica compra, medido:** uma fonte no ecrã abana, uma fora dele
/// cala-se. ⚠️ A vista de fábrica tem `11,25` m de altura ⇒ `±5,625` até à borda de cima.
#[test]
fn os_raios_de_fabrica_cobrem_o_ecra_e_nao_mais() {
    let s = ShakeSource::default();
    assert_eq!(
        ph2d_shake::atenuacao(0.0, s.dentro, s.fora),
        1.0,
        "à queima-roupa o impulso chega inteiro"
    );
    assert!(
        ph2d_shake::atenuacao(5.6, s.dentro, s.fora) > 0.3,
        "na borda do ecrã ainda tem de abanar"
    );
    assert_eq!(
        ph2d_shake::atenuacao(12.0, s.dentro, s.fora),
        0.0,
        "uma explosão a uma vista de distância não abana nada"
    );
}

/// ⛔ **A cerca de quem falou é a do suplente #24, reutilizada INTEIRA** — e o que se afirma aqui é
/// que ela é a mesma porta, não uma segunda cópia com a mesma cara.
#[test]
fn a_cerca_de_quem_falou_e_a_do_gatilho() {
    let mut w = World::new();
    let a = w.spawn_empty().id();
    let b = w.spawn_empty().id();
    assert!(SignalFrom::Anyone.deixa_passar(None, a));
    assert!(SignalFrom::Myself.deixa_passar(Some(a), a));
    assert!(!SignalFrom::Myself.deixa_passar(Some(b), a));
    assert!(
        !SignalFrom::Myself.deixa_passar(None, a),
        "um sinal SEM sujeito nunca passa uma cerca fechada"
    );
}

/// ⚠️ **O vivo nasce a ZERO**, e é isso que o [`crate::rewind_runtime`] repõe.
#[test]
fn o_vivo_nasce_sem_trauma_e_sem_relogio() {
    let rt = CameraShakeRuntime::default();
    assert_eq!(rt.trauma, 0.0);
    assert_eq!(rt.t, 0.0);
}
