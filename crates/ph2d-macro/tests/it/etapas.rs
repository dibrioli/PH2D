//! **As etapas** (W5, `docs/Macros/02_plano_W5.md` §2/§4): a trilha que toca é uma função pura da
//! gravação (imutável) e das edições; uma espera é relógio a CORRER, encurtar só tira relógio; e o
//! ficheiro de uma macro editada exige quem saiba aplicá-la.
//!
//! A família de brinquedo: carga `[tipo, tempo:u64le]` — `0` relógio (o tempo do playhead dentro, como o
//! `Quadro` do Motion), `1..=3` início/meio/fim de um gesto, `4` uma propriedade solta, `9` a marca de
//! saída da gravação.

use ph2d_macro::codec::{self, CodecError};
use ph2d_macro::etapa::efetiva;
use ph2d_macro::{
    EdicaoDeEtapa, ErroDeEdicao, Etapa, Event, LeitorDaFamilia, MacroDoc, MacroTime, RefRect, Track,
};

const Q: u64 = 10_000;

fn carga(tipo: u8, tempo: u64) -> Vec<u8> {
    let mut v = vec![tipo];
    v.extend_from_slice(&tempo.to_le_bytes());
    v
}

fn tempo(c: &[u8]) -> u64 {
    u64::from_le_bytes(c[1..9].try_into().expect("9 bytes"))
}

struct Brinquedo;

impl LeitorDaFamilia for Brinquedo {
    fn etapas(&self, eventos: &[Event]) -> Vec<Etapa> {
        let mut out = Vec::new();
        let mut inicio = None;
        for (i, e) in eventos.iter().enumerate() {
            match e.payload[0] {
                1 => inicio = Some(i),
                3 => {
                    if let Some(s) = inicio.take() {
                        out.push(Etapa { inicio: s, fim: i });
                    }
                }
                _ => {}
            }
        }
        out
    }
    fn e_relogio(&self, c: &[u8]) -> bool {
        c[0] == 0
    }
    fn espera(&self, ev: &[Event], antes_de: usize, t0: MacroTime, us: u64, ja: i64) -> Vec<Event> {
        let ultimo = ev[..antes_de]
            .iter()
            .rev()
            .find(|e| e.payload[0] == 0)
            .map_or(0, |e| tempo(&e.payload));
        (0..us / Q)
            .map(|j| Event {
                t: MacroTime(t0.0 + j * Q),
                payload: carga(0, ultimo.saturating_add_signed(ja) + (j + 1) * Q),
            })
            .collect()
    }
    fn desloca(&self, c: &[u8], us: i64) -> Option<Vec<u8>> {
        (c[0] == 0).then(|| carga(0, tempo(c).saturating_add_signed(us)))
    }
    fn sai_da_gravacao(&self) -> Option<Vec<u8>> {
        Some(carga(9, 0))
    }
}

/// Relógio a cada `Q`; três gestos de 3 quadros com 4 quadros de relógio entre eles; uma propriedade
/// solta no 2.º intervalo.
fn gravacao() -> Track {
    let mut events = Vec::new();
    let mut t = 0;
    let mut quadro = |tipo: u8, events: &mut Vec<Event>| {
        events.push(Event {
            t: MacroTime(t),
            payload: carga(tipo, t),
        });
        if tipo == 0 {
            t += Q;
        }
    };
    for g in 0..3 {
        for _ in 0..4 {
            quadro(0, &mut events);
        }
        if g == 1 {
            quadro(4, &mut events);
            quadro(0, &mut events);
        }
        for tipo in [1, 2, 3] {
            quadro(tipo, &mut events);
            quadro(0, &mut events);
        }
    }
    Track {
        family: "brinquedo".into(),
        target: 0,
        initial: vec![],
        events,
        edicoes: vec![],
    }
}

fn com(mut tr: Track, ed: EdicaoDeEtapa) -> Track {
    tr.edicoes.push(ed);
    tr
}

fn ancora(tr: &Track, k: usize) -> usize {
    Brinquedo.etapas(&tr.events)[k].inicio
}

fn tipos(ev: &[Event]) -> Vec<u8> {
    ev.iter().map(|e| e.payload[0]).collect()
}

#[test]
fn sem_edicoes_a_trilha_que_toca_e_a_gravacao_ao_bit() {
    let tr = gravacao();
    assert_eq!(
        Brinquedo.etapas(&tr.events).len(),
        3,
        "a fixture tem 3 gestos"
    );
    assert_eq!(efetiva(&tr, &Brinquedo), Ok(tr.events.clone()));
}

#[test]
fn uma_espera_e_relogio_a_correr_e_tudo_o_que_vem_depois_anda_com_ela() {
    let g = gravacao();
    let a = ancora(&g, 1);
    let tr = com(
        g.clone(),
        EdicaoDeEtapa {
            ancora: a,
            espera_us: 5 * Q as i64,
            ..Default::default()
        },
    );
    let ev = efetiva(&tr, &Brinquedo).expect("aplica");
    assert_eq!(
        &ev[..a],
        &g.events[..a],
        "antes da etapa: a gravação, ao bit"
    );
    assert_eq!(
        ev[a].payload,
        carga(9, 0),
        "a marca de saída, uma vez, no início da espera"
    );
    assert_eq!(tipos(&ev).iter().filter(|&&k| k == 9).count(), 1);
    let ultimo = tempo(&g.events[a - 1].payload);
    for j in 0..5 {
        let e = &ev[a + 1 + j];
        assert_eq!(e.t, MacroTime(g.events[a].t.0 + j as u64 * Q));
        assert_eq!(
            tempo(&e.payload),
            ultimo + (j as u64 + 1) * Q,
            "o relógio da espera continua a cadência gravada"
        );
    }
    for (i, e) in g.events[a..].iter().enumerate() {
        let f = &ev[a + 6 + i];
        assert_eq!(
            f.t.0,
            e.t.0 + 5 * Q,
            "a etapa e as seguintes andam a espera inteira"
        );
        if e.payload[0] == 0 {
            assert_eq!(
                tempo(&f.payload),
                tempo(&e.payload) + 5 * Q,
                "e o tempo de dentro também"
            );
        } else {
            assert_eq!(f.payload, e.payload);
        }
    }
    assert!(
        ev.windows(2).all(|w| w[0].t <= w[1].t),
        "o tempo nunca recua"
    );
}

#[test]
fn encurtar_tira_so_relogio_e_nunca_passa_a_etapa_anterior() {
    let g = gravacao();
    let a = ancora(&g, 1);
    let ed = |us: i64| EdicaoDeEtapa {
        ancora: a,
        espera_us: us,
        ..Default::default()
    };
    let ev = efetiva(&com(g.clone(), ed(-3 * Q as i64)), &Brinquedo).expect("cabe");
    let relogios = |v: &[Event]| v.iter().filter(|e| e.payload[0] == 0).count();
    assert_eq!(
        relogios(&ev),
        relogios(&g.events) - 3,
        "três quadros de relógio a menos"
    );
    assert!(
        tipos(&ev).contains(&4),
        "a propriedade solta NÃO é relógio: fica"
    );
    let p = ev.iter().position(|e| e.payload[0] == 4).expect("está");
    assert_eq!(
        ev[p].t,
        ev[p + 1].t,
        "e cola-se ao início da etapa encurtada"
    );
    assert!(
        ev.windows(2).all(|w| w[0].t <= w[1].t),
        "o tempo nunca recua"
    );
    let fim_anterior = Brinquedo.etapas(&g.events)[0].fim;
    let intervalo = g.events[a].t.0 - g.events[fim_anterior].t.0;
    assert!(efetiva(&com(g.clone(), ed(-(intervalo as i64))), &Brinquedo).is_ok());
    assert_eq!(
        efetiva(&com(g, ed(-(intervalo as i64) - 1)), &Brinquedo),
        Err(ErroDeEdicao::EsperaMaiorQueOIntervalo(a))
    );
}

#[test]
fn trocar_e_acrescentar_cargas_nao_mexe_no_tempo() {
    let g = gravacao();
    let a = ancora(&g, 2);
    let tr = com(
        g.clone(),
        EdicaoDeEtapa {
            ancora: a,
            trocas: vec![(a + 2, carga(2, 777))],
            insercoes: vec![(a, carga(4, 1))],
            ..Default::default()
        },
    );
    let ev = efetiva(&tr, &Brinquedo).expect("aplica");
    assert_eq!(ev.len(), g.events.len() + 1);
    assert_eq!(&ev[..a], &g.events[..a]);
    assert_eq!(
        ev[a],
        Event {
            t: g.events[a].t,
            payload: carga(4, 1)
        }
    );
    assert_eq!(ev[a + 3].payload, carga(2, 777));
    assert!(
        !tipos(&ev).contains(&9),
        "sem mudar o tempo, a trilha não sai da gravação"
    );
    let sai = com(
        g.clone(),
        EdicaoDeEtapa {
            ancora: a,
            sai_da_gravacao: true,
            ..Default::default()
        },
    );
    let ev = efetiva(&sai, &Brinquedo).expect("aplica");
    assert_eq!(ev[a].payload, carga(9, 0), "…a não ser que a edição o peça");
}

#[test]
fn uma_edicao_que_nao_cabe_na_gravacao_recusa_se() {
    let g = gravacao();
    let a = ancora(&g, 0);
    let ed = |ancora| EdicaoDeEtapa {
        ancora,
        ..Default::default()
    };
    assert_eq!(
        efetiva(&com(g.clone(), ed(a + 1)), &Brinquedo),
        Err(ErroDeEdicao::AncoraSemEtapa(a + 1))
    );
    assert_eq!(
        efetiva(&com(com(g.clone(), ed(a)), ed(a)), &Brinquedo),
        Err(ErroDeEdicao::AncoraRepetida(a))
    );
    let fora = EdicaoDeEtapa {
        ancora: a,
        trocas: vec![(g.events.len(), vec![])],
        ..Default::default()
    };
    assert_eq!(
        efetiva(&com(g.clone(), fora), &Brinquedo),
        Err(ErroDeEdicao::ForaDaGravacao(g.events.len()))
    );
}

#[test]
fn apagar_uma_etapa_fecha_o_tempo_dela() {
    let g = gravacao();
    let es = Brinquedo.etapas(&g.events);
    let (a, f) = (es[1].inicio, es[1].fim);
    let dur = g.events[f].t.0 - g.events[a].t.0;
    assert!(dur > 0, "a fixture: a etapa dura");
    let tr = com(
        com(
            g.clone(),
            EdicaoDeEtapa {
                ancora: es[0].inicio,
                insercoes: vec![(a + 1, carga(4, 42))],
                ..Default::default()
            },
        ),
        EdicaoDeEtapa {
            ancora: a,
            apagada: true,
            // As outras edições de uma etapa apagada não valem.
            espera_us: 5 * Q as i64,
            trocas: vec![(f + 1, carga(4, 7))],
            ..Default::default()
        },
    );
    let ev = efetiva(&tr, &Brinquedo).expect("aplica");
    assert_eq!(
        &ev[..a],
        &g.events[..a],
        "antes da etapa: a gravação, ao bit"
    );
    assert_eq!(
        ev[a].payload,
        carga(9, 0),
        "a trilha sai da gravação onde a etapa estava"
    );
    assert_eq!(ev[a].t, g.events[a].t);
    assert_eq!(
        (ev[a + 1].t, &ev[a + 1].payload),
        (g.events[a].t, &carga(4, 42)),
        "a inserção de OUTRA etapa num evento apagado toca onde ele fechou"
    );
    let depois = &ev[a + 2..];
    assert_eq!(
        depois.len(),
        g.events.len() - f - 1,
        "os eventos da etapa (e o relógio de dentro) saem"
    );
    for (e, o) in depois.iter().zip(&g.events[f + 1..]) {
        assert_eq!(
            e.t.0,
            o.t.0 - dur,
            "o que vem depois recua a duração da etapa"
        );
        if o.payload[0] == 0 {
            assert_eq!(
                tempo(&e.payload),
                tempo(&o.payload) - dur,
                "e o tempo de dentro também"
            );
        } else {
            assert_eq!(e.payload, o.payload);
        }
    }
    assert_eq!(Brinquedo.etapas(&ev).len(), 2, "fica uma etapa a menos");
    assert_eq!(tipos(&ev).iter().filter(|&&k| k == 9).count(), 1);
    assert!(
        ev.windows(2).all(|w| w[0].t <= w[1].t),
        "o tempo nunca recua"
    );
    let mut so = g.clone();
    assert!(so.apaga_etapa(&Brinquedo, 1));
    assert!(!so.apaga_etapa(&Brinquedo, 3), "não há etapa 3");
    assert_eq!(so.edicoes.len(), 1);
    assert!(so.edicoes[0].apagada && so.edicoes[0].ancora == a);
}

fn editado() -> MacroDoc {
    let g = gravacao();
    let g0 = g.clone();
    let (a, b) = (ancora(&g, 1), ancora(&g, 2));
    let mut tr = g;
    tr.edicoes = vec![
        EdicaoDeEtapa {
            ancora: a,
            nome: Some("Pincelada azul".into()),
            dica: Some("Pinte o céu primeiro".into()),
            espera_us: -12_345,
            trocas: vec![(a + 2, vec![7, 7])],
            insercoes: vec![(a, vec![])],
            sai_da_gravacao: true,
            apagada: false,
        },
        EdicaoDeEtapa {
            ancora: ancora(&g0, 0),
            apagada: true,
            ..Default::default()
        },
        EdicaoDeEtapa {
            ancora: b,
            espera_us: i64::MAX / 3,
            ..Default::default()
        },
    ];
    MacroDoc {
        reference: RefRect { w: 1.0, h: 1.0 },
        tracks: vec![tr],
    }
}

#[test]
fn as_edicoes_vao_e_voltam_ao_byte_e_o_ficheiro_exige_quem_as_aplique() {
    let doc = editado();
    let bytes = codec::write(&doc);
    assert_eq!(
        &bytes[9..13],
        &[0x01, 0x00, 0x01, 0x00],
        "1.1: as edições sobem o minor"
    );
    let back = codec::read(&bytes).expect("lê");
    assert_eq!(back, doc);
    assert_eq!(codec::write(&back), bytes);
    let mut sem = doc;
    sem.tracks[0].edicoes.clear();
    assert_eq!(
        &codec::write(&sem)[9..13],
        &[0x01, 0x00, 0x00, 0x00],
        "sem edições: 1.0"
    );
}

#[test]
fn um_ficheiro_que_exige_o_que_este_leitor_nao_sabe_recusa_se() {
    // v1.0 vazio (sem trilhas) com `exige` = "voar", escrito à mão.
    let b = [
        &b"PH2DMACRO"[..],
        &[0x01, 0x00, 0x01, 0x00],
        &[0x01, 0x08, 0, 0, 0x80, 0x3F, 0, 0, 0x80, 0x3F],
        &[0x03, 0x04, b'v', b'o', b'a', b'r'],
        &[0x00],
    ]
    .concat();
    assert_eq!(codec::read(&b), Err(CodecError::Exige("voar".into())));
    let mut sabe = b.clone();
    sabe.splice(23..29, [0x03, 0x06, b'e', b't', b'a', b'p', b'a', b's']);
    assert!(codec::read(&sabe).is_ok(), "e o que sabe lê-se");
}

/// A cadência exacta: `k` quadros de 60 Hz arredondados a µs cabem `k` vezes, mesmo quando a divisão em
/// vírgula flutuante fica um nada abaixo do inteiro (50 000 / 16 666,67 = 2,999…).
#[test]
fn uma_espera_de_k_quadros_arredondada_a_us_tem_k_quadros_e_nenhum_resto() {
    use ph2d_macro::etapa::{no_quadro, quadros_da_espera};
    let q = 1e6 / 60.0;
    for k in 1..=600 {
        assert_eq!(quadros_da_espera(no_quadro(k, q), q), (k, 0), "{k} quadros");
    }
}
