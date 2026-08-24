//! Os gates do `CHANNEL_POSITION_XY` — **um nó para a posição inteira** (doc 89 folha 03).

//! Os gates do [`super::CHANNEL_POSITION_XY`] — **um nó para a posição inteira**
//! (doc 89 folha 03).

use super::*;

fn row() -> Stream {
    Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]))
}

/// Corre `ticks` tiques e devolve a pose final.
///
/// ⚠️ **O primeiro tique é na ORIGEM e só depois o alvo SALTA.** A primeira versão desta
/// fixture punha o alvo já em `target` desde o tique 0 — e a mola semeia *no* alvo, por
/// desenho (nada de snap), então ela não tinha o que perseguir e a pose saía exactamente
/// no alvo. *Uma fixture que não contém o fenómeno acusa produto correcto.*
fn chase(channel: i32, target: [f32; 2], ticks: usize) -> Stream {
    let mut state = Stream::new(0);
    let mut out = row();
    for k in 0..ticks {
        let input = if k == 0 {
            row()
        } else {
            Stream::new(2).with(
                "P",
                Column::Vec2(vec![target, [target[0] + 1.0, target[1] + 1.0]]),
            )
        };
        out = step(&input, &state, channel, 120.0, 12.0, k as f32 / 60.0);
        state = out.clone();
    }
    out
}

fn pos(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// ⭐ **Um nó só persegue nos DOIS eixos** — e um canal escalar só persegue no dele.
#[test]
fn the_position_channel_chases_on_both_axes_where_a_scalar_one_chases_on_its_own() {
    let target = [4.0_f32, 3.0];
    let only_x = pos(&chase(0, target, 3));
    let both = pos(&chase(CHANNEL_POSITION_XY, target, 3));
    // O canal X move o x e deixa o y no alvo cru (a mola nem o toca).
    assert!(
        (only_x[0][0] - target[0]).abs() > 1e-3,
        "o X esta' a caminho, nao no alvo: {:?}",
        only_x[0]
    );
    assert!(
        (only_x[0][1] - target[1]).abs() < 1e-6,
        "e o Y do canal X sai CRU: {:?}",
        only_x[0]
    );
    // O canal XY está a caminho nos DOIS.
    assert!(
        (both[0][0] - target[0]).abs() > 1e-3 && (both[0][1] - target[1]).abs() > 1e-3,
        "o XY tinha de estar a caminho nos dois eixos: {:?}",
        both[0]
    );
}

/// ⭐ **O eixo X do canal XY é EXACTAMENTE o do canal X** — a mesma lei, não uma parecida.
#[test]
fn the_x_of_the_pair_is_bit_for_bit_the_x_of_the_scalar_channel() {
    let target = [4.0_f32, 3.0];
    let a = pos(&chase(0, target, 5));
    let b = pos(&chase(CHANNEL_POSITION_XY, target, 5));
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert_eq!(
            x[0].to_bits(),
            y[0].to_bits(),
            "elemento {i}: o x divergiu ({:?} contra {:?})",
            x[0],
            y[0]
        );
    }
}

/// ⭐ **E o eixo Y do par é EXACTAMENTE o do canal Y** — o gêmeo do gate acima, e ele
/// existe porque a mutação que o pedia **SOBREVIVEU**.
///
/// ⚠️ **A afirmação era maior que a máquina:** eu escrevi que *"o estado do Y é
/// append-only"* e provei-o só pela PRESENÇA das colunas. Fazer o `solve` do Y **ler** as
/// colunas do X passava a suíte inteira — o Y continuava *a caminho* do alvo (só que com a
/// memória errada), e nenhum dos três gates olhava para o valor dele. *Um gate que mede
/// que a coluna existe não mede quem a leu.*
#[test]
fn the_y_of_the_pair_is_bit_for_bit_the_y_of_the_scalar_channel() {
    // ⚠️ O alvo é ASSIMÉTRICO de propósito: com `x == y` as duas molas percorrem o mesmo
    // número e a troca de estado passaria despercebida por simetria.
    let target = [4.0_f32, -7.5];
    let a = pos(&chase(1, target, 5));
    let b = pos(&chase(CHANNEL_POSITION_XY, target, 5));
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert_eq!(
            x[1].to_bits(),
            y[1].to_bits(),
            "elemento {i}: o y divergiu ({:?} contra {:?}) -- o par nao esta' a usar o \
             estado dele",
            x[1],
            y[1]
        );
    }
    // CONTROLE: os dois eixos de facto percorreram números diferentes, senão o gate
    // ficaria verde por simetria.
    assert!(
        (b[0][0] - b[0][1]).abs() > 1.0,
        "CONTROLE: a fixtura tem de ser assimetrica ({:?})",
        b[0]
    );
}

/// ⚠️ **O estado do Y é APPEND-ONLY** — os quatro canais escalares nunca escrevem as
/// colunas dele, então uma cena já autorada não ganha estado que ninguém lê.
#[test]
fn the_scalar_channels_never_mint_the_pairs_state() {
    for ch in 0..4 {
        let s = chase(ch, [4.0, 3.0], 2);
        assert!(
            s.get("spring_value_y").is_none() && s.get("spring_vel_y").is_none(),
            "o canal {ch} cunhou o estado do par"
        );
    }
    let pair = chase(CHANNEL_POSITION_XY, [4.0, 3.0], 2);
    assert!(
        pair.get("spring_value_y").is_some(),
        "e o par escreve o dele"
    );
    assert!(
        pair.get("spring_value").is_some(),
        "sem deixar de escrever o de sempre -- o `pairing` pergunta por ele"
    );
}
