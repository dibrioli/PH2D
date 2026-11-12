//! ⭐ **A instância acompanha a peça** (smoke do dono 11/11): o container mede o conteúdo, então a
//! instância que mostrava a peça INTEIRA continua a mostrá-la quando a peça muda — à mesma
//! velocidade; a recortada e a de comprimento fixo ficam; em Loop só a fatia muda; o undo desfaz tudo.

use ph2d_core::Playhead;
use ph2d_timeline::{
    ClipStrip, EnterStep, StackHost, StripId, StripLoop, StripSource, TimelineIntent as I,
    TimelineState, apply_intent, instance_map,
};

const EPS: f64 = 1e-9;

struct Cena {
    st: TimelineState,
    c: usize,
    /// As duas instâncias da lane Base da cena: `[1, 3)` e `[4, 6)`, velocidade 1.
    a: StripId,
    b: StripId,
    /// Dentro: jump `[1, 2)` na lane Base (o fim da peça).
    jump: StripId,
}

/// A montagem do `PH2D_NEST_SMOKE=4` em miniatura: walk `[0, 1.25)` + jump `[1, 2)`; 2 s.
fn cena() -> Cena {
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let c = doc.add_container("Run & jump".into());
    let dentro = StackHost::Container(c);
    doc.add_lane_in(dentro, "Base".into()).unwrap();
    let clip = StripSource::Clip(0);
    doc.add_strip_to(dentro, 0, clip, 0.0, 1.25).unwrap();
    let jump = doc.add_strip_to(dentro, 0, clip, 1.0, 2.0).unwrap();
    doc.add_lane("Base".into()).unwrap();
    let src = StripSource::Container(u16::try_from(c).unwrap());
    let a = doc
        .add_strip_to(StackHost::Document, 0, src, 1.0, 3.0)
        .unwrap();
    let b = doc
        .add_strip_to(StackHost::Document, 0, src, 4.0, 6.0)
        .unwrap();
    Cena { st, c, a, b, jump }
}

fn fora(k: &Cena, id: StripId) -> ClipStrip {
    k.st.doc
        .strip_in(StackHost::Document, 0, id)
        .unwrap()
        .clone()
}

fn fora_mut(st: &mut TimelineState, id: StripId) -> &mut ClipStrip {
    st.doc.strip_in_mut(StackHost::Document, 0, id).unwrap()
}

fn aplica(k: &mut Cena, i: I) {
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(&mut k.st, &mut ph, i);
}

/// Move o jump para `t` (tempo do container) pela peça aberta no lugar em `a`.
fn move_jump_no_lugar(k: &mut Cena, t_container: f64) {
    let t_cena = instance_map(&k.st.doc, k.a).unwrap().scene(t_container);
    let (a, jump) = (k.a, k.jump);
    aplica(
        k,
        I::NoLugar {
            instancia: a,
            intent: Box::new(I::MoveStrip {
                lane: 0,
                to_lane: 0,
                id: jump,
                t_start: t_cena,
            }),
        },
    );
}

fn assert_inteira(k: &Cena, id: StripId, len: f64, span: f64) {
    let s = fora(k, id);
    assert!(
        (s.speed - 1.0).abs() < EPS,
        "a velocidade fica: {}",
        s.speed
    );
    assert!(
        s.src_in.abs() < EPS && (s.src_out - len).abs() < EPS,
        "mostra a peça inteira: [{}, {}] de {len}",
        s.src_in,
        s.src_out
    );
    assert!((s.span() - span).abs() < EPS, "span {} ≠ {span}", s.span());
}

#[test]
fn no_lugar_acompanha_o_jump_para_la_do_fim_estica_as_duas_instancias() {
    let mut k = cena();
    move_jump_no_lugar(&mut k, 2.5); // jump [2.5, 3.5): a peça tem 3,5 s
    let len = k.st.doc.container_length_seconds(k.c);
    assert!((len - 3.5).abs() < EPS, "{len}");
    assert_inteira(&k, k.a, len, len);
    assert_inteira(&k, k.b, len, len);
}

#[test]
fn no_lugar_acompanha_tambem_dentro_do_container_entrado() {
    let mut k = cena();
    k.st.edit_path = vec![EnterStep {
        container: k.c,
        lane: 0,
        strip: Some(k.a),
    }];
    let jump = k.jump;
    aplica(
        &mut k,
        I::MoveStrip {
            lane: 0,
            to_lane: 0,
            id: jump,
            t_start: 0.5,
        },
    ); // jump [0.5, 1.5): a peça encolhe para 1,5 s (o walk acaba em 1,25)
    let len = k.st.doc.container_length_seconds(k.c);
    assert!((len - 1.5).abs() < EPS, "{len}");
    assert_inteira(&k, k.a, len, len);
    assert_inteira(&k, k.b, len, len);
}

#[test]
fn no_lugar_acompanha_a_mesma_velocidade_e_a_recortada_fica() {
    let mut k = cena();
    // `a` a meia velocidade (span 4); `b` recortada a `[0, 1]`.
    let s = fora_mut(&mut k.st, k.a);
    (s.t_end, s.speed) = (5.0, 0.5);
    let s = fora_mut(&mut k.st, k.b);
    (s.src_out, s.t_end) = (1.0, 5.0);
    move_jump_no_lugar(&mut k, 2.0); // jump [2, 3): a peça tem 3 s
    let len = k.st.doc.container_length_seconds(k.c);
    let a = fora(&k, k.a);
    assert!((a.speed - 0.5).abs() < EPS && (a.src_out - len).abs() < EPS);
    assert!(
        (a.span() - len / 0.5).abs() < EPS,
        "Once: o span segue à mesma velocidade: {}",
        a.span()
    );
    let b = fora(&k, k.b);
    assert_eq!(
        (b.src_in, b.src_out, b.t_start, b.t_end, b.speed),
        (0.0, 1.0, 4.0, 5.0, 1.0),
        "a recortada não se mexe"
    );
}

#[test]
fn no_lugar_acompanha_em_loop_so_a_fatia() {
    let mut k = cena();
    fora_mut(&mut k.st, k.b).loop_mode = StripLoop::Loop;
    move_jump_no_lugar(&mut k, 2.5);
    let b = fora(&k, k.b);
    assert!(
        (b.src_out - 3.5).abs() < EPS && (b.t_start, b.t_end) == (4.0, 6.0),
        "Loop: a fatia cresce, o span fica: [{}, {}) fatia {}",
        b.t_start,
        b.t_end,
        b.src_out
    );
}

#[test]
fn no_lugar_acompanha_nao_com_comprimento_fixo() {
    let mut k = cena();
    k.st.doc.set_container_length_override(k.c, Some(2.0));
    move_jump_no_lugar(&mut k, 2.5);
    assert_eq!(k.st.doc.container_length_seconds(k.c), 2.0);
    assert_inteira(&k, k.a, 2.0, 2.0);
}

#[test]
fn no_lugar_acompanha_o_container_dentro_de_outro() {
    let mut k = cena();
    // `Pack` toca o Run & jump inteiro `[0, 2)`; a cena toca o Pack inteiro em `[10, 12)`.
    let doc = &mut k.st.doc;
    let pack = doc.add_container("Pack".into());
    let dentro = StackHost::Container(pack);
    doc.add_lane_in(dentro, "Base".into()).unwrap();
    let run = StripSource::Container(u16::try_from(k.c).unwrap());
    let r = doc.add_strip_to(dentro, 0, run, 0.0, 2.0).unwrap();
    doc.add_lane("Pack".into()).unwrap();
    let p = StripSource::Container(u16::try_from(pack).unwrap());
    let fora_p = doc
        .add_strip_to(StackHost::Document, 1, p, 10.0, 12.0)
        .unwrap();
    move_jump_no_lugar(&mut k, 2.5);
    let doc = &k.st.doc;
    let r = doc.strip_in(dentro, 0, r).unwrap();
    assert!((r.src_out - 3.5).abs() < EPS && (r.span() - 3.5).abs() < EPS);
    assert!((doc.container_length_seconds(pack) - 3.5).abs() < EPS);
    let fp = doc.strip_in(StackHost::Document, 1, fora_p).unwrap();
    assert!(
        (fp.src_out - 3.5).abs() < EPS && (fp.t_end - 13.5).abs() < EPS,
        "o hospedeiro também acompanha: {} {}",
        fp.src_out,
        fp.t_end
    );
}

#[test]
fn no_lugar_acompanha_o_undo_desfaz_as_duas_metades() {
    let mut k = cena();
    let antes = (fora(&k, k.a), fora(&k, k.b));
    move_jump_no_lugar(&mut k, 2.5);
    assert!((fora(&k, k.a).src_out - 3.5).abs() < EPS);
    aplica(&mut k, I::Undo);
    assert!((k.st.doc.container_length_seconds(k.c) - 2.0).abs() < EPS);
    assert_eq!((fora(&k, k.a), fora(&k, k.b)), antes, "um passo de undo");
    aplica(&mut k, I::Redo);
    assert!((fora(&k, k.b).src_out - 3.5).abs() < EPS);
}

#[test]
fn no_lugar_acompanha_nao_quando_a_edicao_fixa_o_comprimento() {
    let mut k = cena();
    let c = k.c;
    aplica(
        &mut k,
        I::SetContainerLength {
            container: c,
            len: Some(3.0),
        },
    );
    assert_eq!(k.st.doc.container_length_seconds(k.c), 3.0);
    assert_inteira(&k, k.a, 2.0, 2.0);
}
