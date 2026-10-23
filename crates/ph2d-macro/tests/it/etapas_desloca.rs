//! **A etapa que anda sozinha** (W16, `docs/Macros/14_W16_plano.md` §2.1): o losango arrastado move só
//! aquela etapa; as outras ficam, o relógio fica (um relógio por quadro), a sobreposição toca nas faixas.
//!
//! Na gravação: o gesto 0 ocupa `4Q..=6Q`, o 1 `12Q..=14Q`, o 2 `19Q..=21Q`; o último evento é em `21Q`.

use super::*;

fn desloca(g: &Track, k: usize, us: i64) -> Track {
    com(
        g.clone(),
        EdicaoDeEtapa {
            ancora: ancora(g, k),
            desloca_us: us,
            ..Default::default()
        },
    )
}

/// `(t, tipo, faixa)` dos eventos que não são relógio, nem a marca de saída, nem a da faixa.
fn gestos(ev: &[Event]) -> Vec<(u64, u8, u8)> {
    ev.iter()
        .filter(|e| !matches!(e.payload[0], 0 | 7 | 9))
        .map(|e| (e.t.0 / Q, e.payload[0], e.payload.get(9).copied().unwrap_or(0)))
        .collect()
}

fn ate(r: Vec<(MacroTime, Vec<u8>)>, q: u64) -> Vec<(MacroTime, Vec<u8>)> {
    r.into_iter().filter(|(t, _)| *t <= MacroTime(q * Q)).collect()
}

fn relogio(ev: &[Event]) -> Vec<(MacroTime, Vec<u8>)> {
    ev.iter()
        .filter(|e| e.payload[0] == 0)
        .map(|e| (e.t, e.payload.clone()))
        .collect()
}

/// Mutações que sangram: deslocar também as seguintes (ripple) · deslocar o relógio da etapa · sem faixa.
#[test]
fn deslocar_move_so_a_etapa_e_o_relogio_fica() {
    let g = gravacao();
    let ev = efetiva(&desloca(&g, 2, -8 * Q as i64), &Brinquedo).expect("sobrepor é aceite");
    assert_eq!(
        relogio(&ev),
        ate(relogio(&g.events), 14),
        "um relógio só, o da gravação — até ao fim novo (a última etapa no tempo é a 1, até 14Q)"
    );
    assert_eq!(
        gestos(&ev),
        vec![
            (4, 1, 0),
            (5, 2, 0),
            (6, 3, 0),
            (11, 4, 0),
            (11, 1, 3),
            (12, 1, 0),
            (12, 2, 3),
            (13, 2, 0),
            (13, 3, 3),
            (14, 3, 0),
        ],
        "o gesto 2 começa a meio do intervalo (depois do que já lá estava) e acaba a meio do gesto 1, na faixa 3; o resto não anda"
    );
    assert!(ev.windows(2).all(|w| w[0].t <= w[1].t), "o tempo nunca recua");
    let marca = ev.iter().position(|e| e.payload[0] == 9).expect("sai da gravação");
    assert_eq!(ev[marca].t, MacroTime(11 * Q), "no menor instante que mudou");
    assert!(
        ev[..marca].iter().all(|e| e.t < MacroTime(11 * Q)),
        "a marca vem antes de tudo o que toca nesse instante"
    );
    let pincel: Vec<_> = ev
        .iter()
        .enumerate()
        .filter(|(_, e)| e.payload[0] == 7)
        .map(|(i, e)| (i, e.t.0 / Q, e.payload[9]))
        .collect();
    assert_eq!(pincel.len(), 1, "uma marca de faixa por etapa deslocada");
    let (i, t, n) = pincel[0];
    assert_eq!(
        (t, n),
        (14, 3),
        "no instante GRAVADO do gesto 2 (19Q), puxado para o fim novo da trilha, para a faixa 3"
    );
    assert!(
        ev[i + 1..].iter().all(|e| e.t > ev[i].t),
        "depois de tudo o que já estava nesse instante"
    );
}

/// Passar à frente: o gesto 2 levado para antes do gesto 0 — a ordem troca-se e nada mais muda.
#[test]
fn uma_etapa_passa_a_frente_das_outras() {
    let g = gravacao();
    let ev = efetiva(&desloca(&g, 2, -19 * Q as i64), &Brinquedo).expect("aceite");
    let primeiros: Vec<_> = gestos(&ev).into_iter().take(3).collect();
    assert_eq!(primeiros, vec![(0, 1, 3), (1, 2, 3), (2, 3, 3)]);
    assert_eq!(relogio(&ev), ate(relogio(&g.events), 14));
}

#[test]
fn antes_do_zero_recusa_se() {
    let g = gravacao();
    assert_eq!(
        efetiva(&desloca(&g, 0, -5 * Q as i64), &Brinquedo),
        Err(ErroDeEdicao::ComecaAntesDoZero(ancora(&g, 0)))
    );
    assert!(efetiva(&desloca(&g, 0, -4 * Q as i64), &Brinquedo).is_ok(), "até ao 0 vai");
}

/// Um Desfazer gravado (tipo 5) logo a seguir ao gesto 1 desfaz «o último»: deslocar pode tudo, desde que
/// a última etapa a acabar antes dele continue a ser a mesma.
#[test]
fn um_desfazer_gravado_continua_a_desfazer_a_mesma_etapa() {
    let mut g = gravacao();
    let fim1 = Brinquedo.etapas(&g.events)[1].fim;
    let t = g.events[fim1].t;
    g.events.insert(
        fim1 + 1,
        Event {
            t,
            payload: carga(5, t.0),
        },
    );
    let a1 = ancora(&g, 1);
    assert_eq!(
        efetiva(&desloca(&g, 1, 2 * Q as i64), &Brinquedo),
        Err(ErroDeEdicao::MudaOQueODesfazerDesfaz(a1)),
        "o gesto 1 levado para depois do Desfazer: ele desfaria o gesto 0"
    );
    for (k, d, porque) in [
        (2, -7, "o gesto 2 a acabar no mesmo instante do Desfazer: ambíguo"),
        (2, -8, "o gesto 2 a tocar com o gesto 1: o Desfazer levaria os dois"),
        (0, 9, "o gesto 0 a tocar com o gesto 1: idem"),
    ] {
        assert_eq!(
            efetiva(&desloca(&g, k, d * Q as i64), &Brinquedo),
            Err(ErroDeEdicao::MudaOQueODesfazerDesfaz(ancora(&g, k))),
            "{porque}"
        );
    }
    for (k, d) in [(1, -2), (0, 2), (2, -1)] {
        assert!(
            efetiva(&desloca(&g, k, d * Q as i64), &Brinquedo).is_ok(),
            "gesto {k} Δ {d}Q: o Desfazer continua a desfazer só o gesto 1"
        );
    }
}

/// Para lá do fim: o relógio estende-se (a família sintetiza-o) até ao fim da etapa.
#[test]
fn para_la_do_fim_o_relogio_estende_se() {
    let g = gravacao();
    let ev = efetiva(&desloca(&g, 0, 20 * Q as i64), &Brinquedo).expect("aceite");
    let ultimo = gestos(&ev).last().copied().expect("gestos");
    assert_eq!(ultimo, (26, 3, 1), "o gesto 0 acaba em 26Q");
    let quadros_depois = relogio(&ev)
        .iter()
        .filter(|(t, _)| *t > MacroTime(21 * Q))
        .count();
    assert_eq!(quadros_depois, 4, "22Q..=25Q sintetizados");
    assert!(ev.windows(2).all(|w| w[0].t <= w[1].t));
}

/// Apagar vence deslocar; a espera de outra etapa empurra a deslocada com as seguintes.
#[test]
fn deslocar_convive_com_espera_e_apagar() {
    let g = gravacao();
    let mut tr = desloca(&g, 2, -(Q as i64));
    tr.edicao_mut(ancora(&g, 1)).espera_us = 2 * Q as i64;
    let ev = efetiva(&tr, &Brinquedo).expect("aceite");
    assert!(gestos(&ev).contains(&(20, 1, 3)), "19Q + 2Q de espera − 1Q");
    tr.edicao_mut(ancora(&g, 2)).apagada = true;
    let ev = efetiva(&tr, &Brinquedo).expect("aceite");
    assert!(gestos(&ev).iter().all(|g| g.2 == 0), "a apagada não toca");
}

#[test]
fn o_ficheiro_guarda_o_deslocamento_e_exige_quem_o_aplique() {
    let mut doc = editado();
    let a = doc.tracks[0].edicoes[0].ancora;
    doc.tracks[0].edicao_mut(a).desloca_us = -7 * Q as i64;
    let bytes = codec::write(&doc);
    assert_eq!(&bytes[9..13], &[0x01, 0x00, 0x02, 0x00], "1.2");
    let back = codec::read(&bytes).expect("lê");
    assert_eq!(back, doc);
    assert_eq!(codec::write(&back), bytes);
    let exige = b"etapa.desloca";
    assert!(
        bytes.windows(exige.len()).any(|w| w == exige),
        "um leitor que não o saiba aplicar recusa em vez de tocar no sítio velho"
    );
    assert!(
        !codec::write(&editado())
            .windows(exige.len())
            .any(|w| w == exige),
        "sem deslocamento não se exige"
    );
}

/// A cauda anda com a última etapa no tempo: a última levada para trás encurta a trilha (o relógio que
/// sobra sai), e o fim volta a ser o fim da última que acaba.
#[test]
fn a_ultima_etapa_levada_para_tras_encurta_a_trilha() {
    let g = gravacao();
    let ev = efetiva(&desloca(&g, 2, -8 * Q as i64), &Brinquedo).expect("aceite");
    assert_eq!(
        ev.last().map(|e| e.t),
        Some(MacroTime(14 * Q)),
        "o fim é o do gesto 1 (14Q): a gravação acabava no fim do gesto 2"
    );
    assert!(relogio(&ev).iter().all(|(t, _)| *t <= MacroTime(14 * Q)));
    assert!(ev.windows(2).all(|w| w[0].t <= w[1].t));
    let ev = efetiva(&desloca(&g, 0, 2 * Q as i64), &Brinquedo).expect("aceite");
    assert_eq!(
        ev.last().map(|e| e.t),
        Some(MacroTime(21 * Q)),
        "deslocar uma do meio não mexe no fim"
    );
}
