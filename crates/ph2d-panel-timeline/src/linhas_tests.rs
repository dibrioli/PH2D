//! Tests for [`super`] (`linhas.rs`) — as linhas de grupo do dope-sheet.

use super::*;
use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_editor_core::interaction::{GestureMods, GesturePhase, TimelineGesture};
use ph2d_host::PointerButton;
use ph2d_timeline::{AnimTarget, Extrap, Interp, KeyId, KeyView, TimelineIntent};

use crate::state::{self, TimelinePanelState};

const GH: f32 = 132.0;
const FILL_R: u16 = v::TINTA_PREENCH + v::OFF_SOLIDA;

fn field(campo: u16, i: u16, j: u16) -> PropKind {
    PropKind::Field(FieldKey::new(VETOR, campo, i, j))
}

/// Uma faixa com chaves nos instantes `ts` (id da chave = `target * 100 + n`).
fn track(target: u64, entity: u64, prop: PropKind, ts: &[f64]) -> TrackView {
    TrackView {
        target: AnimTarget::new(target),
        prop,
        entity,
        missing: false,
        keys: ts
            .iter()
            .enumerate()
            .map(|(n, &t)| KeyView {
                id: KeyId::new(target * 100 + n as u64),
                t_seconds: t,
                value: 0.0,
                interp: Interp::Linear,
                selected: false,
                roving: false,
            })
            .collect(),
        buffer_ghost: None,
        pre: Extrap::Hold,
        post: Extrap::Hold,
        expr: None,
    }
}

/// O que o autokey grava numa cor de preenchimento: os 4 canais da cor chapada, chave em 0 e 1 —
/// ligados FORA de ordem (A, R, B, G), entre um X e uma largura de traço (grupo de um só).
fn cena() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        tracks: vec![
            track(1, 9, PropKind::TranslationX, &[0.0]),
            track(2, 9, field(FILL_R + 3, 0, 0), &[0.0, 1.0]),
            track(3, 9, field(FILL_R, 0, 0), &[0.0, 1.0]),
            track(4, 9, field(FILL_R + 2, 0, 0), &[0.0, 1.0]),
            track(5, 9, field(FILL_R + 1, 0, 0), &[0.0, 1.0]),
            track(6, 9, field(v::TRACO_LARGURA, 0, 0), &[0.0]),
        ],
        ..TimelineViewSnapshot::default()
    }
}

fn linhas(snap: &TimelineViewSnapshot, abertos: &[GrupoChave]) -> Vec<Linha> {
    layout(snap, Tab::Keys, &[], abertos, GH, 0.0, 0.0)
        .bandas
        .iter()
        .map(|b| b.linha)
        .collect()
}

fn chave_fill() -> GrupoChave {
    GrupoChave::de(&cena().tracks[1]).expect("a cor é um grupo")
}

#[test]
fn a_cor_inteira_e_uma_linha_so() {
    assert_eq!(
        linhas(&cena(), &[]),
        vec![
            Linha::Faixa { i: 0, grupo: None },
            Linha::Grupo(0),
            Linha::Faixa { i: 5, grupo: None },
        ],
        "X · Fill (os 4 canais numa linha) · Stroke Width (grupo de um: sem cabeçalho)"
    );
}

#[test]
fn abrir_o_grupo_mostra_os_membros_na_ordem_de_leitura() {
    let s = cena();
    let l = linhas(&s, &[chave_fill()]);
    let membros: Vec<_> = l[2..6]
        .iter()
        .map(|x| match *x {
            Linha::Faixa { i, grupo: Some(0) } => i,
            o => panic!("membro esperado, veio {o:?}"),
        })
        .collect();
    assert_eq!(
        membros,
        vec![2, 4, 3, 1],
        "R, G, B, A — não a ordem das ligações"
    );
    let lay = layout(&s, Tab::Keys, &[], &[chave_fill()], GH, 0.0, 0.0);
    let nomes: Vec<_> = membros
        .iter()
        .map(|&i| rotulo_membro(&s, &lay.grupos[0], i))
        .collect();
    assert_eq!(nomes, vec!["R", "G", "B", "A"]);
    assert_eq!(
        l.len(),
        7,
        "o cabeçalho continua, os membros vêm por baixo dele"
    );
}

#[test]
fn o_mesmo_grupo_em_dois_objetos_sao_duas_linhas() {
    let mut s = cena();
    s.tracks.push(track(7, 8, field(FILL_R, 0, 0), &[0.5]));
    s.tracks.push(track(8, 8, field(FILL_R + 1, 0, 0), &[0.5]));
    let grupos = linhas(&s, &[])
        .into_iter()
        .filter(|l| matches!(l, Linha::Grupo(_)))
        .count();
    assert_eq!(grupos, 2);
}

#[test]
fn o_ponto_da_forma_le_se_pelo_indice_e_o_contorno_so_quando_ha_dois() {
    let mut s = TimelineViewSnapshot::default();
    for (n, (i, j)) in [(0, 0), (0, 1)].into_iter().enumerate() {
        s.tracks
            .push(track(n as u64 + 1, 9, field(v::ANCORA_X, i, j), &[0.0]));
    }
    s.tracks.push(track(9, 9, field(v::FECHADO, 0, 0), &[0.0]));
    let lay = layout(&s, Tab::Keys, &[], &[], GH, 0.0, 0.0);
    let g = &lay.grupos[0];
    let nome = |i| rotulo_membro(&s, g, i);
    assert_eq!(
        (nome(0), nome(1), nome(2)),
        ("Point X 1".into(), "Point X 2".into(), "Closed".into())
    );

    s.tracks
        .push(track(10, 9, field(v::ANCORA_X, 1, 0), &[0.0]));
    let lay = layout(&s, Tab::Keys, &[], &[], GH, 0.0, 0.0);
    assert_eq!(rotulo_membro(&s, &lay.grupos[0], 3), "Point X 2.1");
}

fn gesto(kind: TimelineHitKind, phase: GesturePhase) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind,
        phase,
        x: 0.0,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    }
}

/// O losango do grupo agarra as chaves dos MEMBROS naquele instante — não a do X, que está no
/// mesmo instante noutra linha. (Mutação: `Escopo::Tudo` no braço `GroupKey` ⇒ entra a chave do X.)
#[test]
fn o_losango_do_grupo_seleciona_so_os_membros() {
    let s = cena();
    let mut st = TimelinePanelState::default();
    let _ = state::drain_intents();
    let kind = TimelineHitKind::GroupKey {
        group: 2,
        t_bits: 0.0_f64.to_bits(),
    };
    crate::interact::dispatch_primary(&mut st, 0.0, 120.0, &s, gesto(kind, GesturePhase::Begin));
    let mut sel: Vec<_> = state::drain_intents()
        .into_iter()
        .filter_map(|i| match i {
            TimelineIntent::AddToSelection(k) => Some(k.target.get()),
            _ => None,
        })
        .collect();
    sel.sort_unstable();
    assert_eq!(sel, vec![2, 3, 4, 5]);
}

#[test]
fn o_triangulo_do_grupo_abre_e_fecha_os_membros() {
    let s = cena();
    let mut st = TimelinePanelState::default();
    let kind = TimelineHitKind::GroupTwirl { group: 4 };
    crate::interact::dispatch_primary(&mut st, 0.0, 120.0, &s, gesto(kind, GesturePhase::Click));
    assert_eq!(st.grupos_abertos, vec![chave_fill()]);
    crate::interact::dispatch_primary(&mut st, 0.0, 120.0, &s, gesto(kind, GesturePhase::Click));
    assert!(st.grupos_abertos.is_empty());
}

/// O retângulo de seleção sobre a linha do grupo leva as chaves dos membros dentro dele.
#[test]
fn o_retangulo_sobre_a_linha_do_grupo_leva_os_membros() {
    let s = cena();
    let st = TimelinePanelState::default();
    let view = TimeView {
        time_x: 100.0,
        view_start: 0.0,
        px_per_s: 100.0,
        right: 1000.0,
    };
    let rows = Rect::new(0.0, 0.0, 1000.0, 400.0);
    // O Summary é a linha 0, o X a 1, o grupo a 2: o centro dele está em 2,5 linhas.
    let cy = ROW_H_PX * 2.5;
    let sel = Rect::new(150.0, cy - 2.0, 100.0, 4.0); // só o instante 1 (x = 200)
    let mut got: Vec<_> = keys_in_rect(rows, view, &st, &s, sel)
        .into_iter()
        .map(|k| k.target.get())
        .collect();
    got.sort_unstable();
    assert_eq!(got, vec![2, 3, 4, 5]);
}

/// Todo campo que o vocabulário do vetor declara tem um nome de membro na tabela de strings — sem
/// isso a linha do membro pinta o identificador cru. (Controlo: mais de 60 campos varridos.)
#[test]
fn todo_campo_do_vetor_tem_nome_de_membro_traduzido() {
    let mut n = 0;
    for campo in 0..=u16::from(u8::MAX) {
        let k = FieldKey::new(VETOR, campo, 0, 0);
        if k.info().is_none() {
            continue;
        }
        n += 1;
        let m = k.membro().expect("campo declarado sem nome de membro");
        assert_ne!(ph2d_i18n::tr(m), m, "campo {campo}: «{m}» sem tradução");
    }
    assert!(n > 60, "só {n} campos — o varrimento leu o sítio errado");
}

/// A tinta da camada 1 da pilha lê-se «Fill 1», a de base só «Fill».
#[test]
fn a_tinta_de_uma_camada_da_pilha_leva_o_numero_dela() {
    let base = crate::tracks::track_label(Some("Star"), 9, field(FILL_R, 0, 0));
    let camada = crate::tracks::track_label(Some("Star"), 9, field(FILL_R, 1, 0));
    assert_eq!(
        (base.as_str(), camada.as_str()),
        ("Star · Fill", "Star · Fill 1")
    );
}
