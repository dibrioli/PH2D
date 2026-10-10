//! A lei da apresentação do preview (relógio de fora) e a thread de verdade, ponta a ponta.

use super::*;
use ph2d_tool_painter::PaintMedia;

fn q(n: u8) -> Quadro {
    Arc::new(vec![n; 4])
}

fn ms(t0: Instant, n: u64) -> Instant {
    t0 + Duration::from_millis(n)
}

#[test]
fn uma_mudanca_isolada_anima_o_traco_uma_vez() {
    let t0 = Instant::now();
    let mut a = Apresentacao::nova();
    let g = a.mudou(t0);
    for n in 0..3 {
        a.chegou(g, q(n));
    }
    a.acabou(g);
    let passo = QUADRO_DA_ANIMACAO.as_millis() as u64;
    assert_eq!(a.quadro(ms(t0, 5)), Some(q(0)), "começa no primeiro quadro");
    assert_eq!(a.quadro(ms(t0, 5 + passo)), Some(q(1)));
    assert_eq!(a.quadro(ms(t0, 5 + 2 * passo)), Some(q(2)));
    assert_eq!(
        a.quadro(ms(t0, 5000)),
        Some(q(2)),
        "e pára no último: uma vez só"
    );
}

#[test]
fn enquanto_a_cena_nova_nao_tem_quadros_fica_o_que_estava() {
    let t0 = Instant::now();
    let mut a = Apresentacao::nova();
    let g = a.mudou(t0);
    a.chegou(g, q(7));
    a.acabou(g);
    assert_eq!(a.quadro(ms(t0, 2000)), Some(q(7)));
    let g2 = a.mudou(ms(t0, 3000));
    assert_eq!(
        a.quadro(ms(t0, 3010)),
        Some(q(7)),
        "sem piscar para o vazio"
    );
    a.chegou(g, q(9));
    assert_eq!(
        a.quadro(ms(t0, 3020)),
        Some(q(7)),
        "o quadro de uma cena velha deita-se fora"
    );
    a.chegou(g2, q(1));
    assert_eq!(a.quadro(ms(t0, 3030)), Some(q(1)));
}

#[test]
fn um_arrasto_mostra_o_traco_final_e_anima_quando_para() {
    let t0 = Instant::now();
    let mut a = Apresentacao::nova();
    let _ = a.mudou(t0);
    let g = a.mudou(ms(t0, 20)); // seguida: é um arrasto
    for n in 0..3 {
        a.chegou(g, q(n));
    }
    a.acabou(g);
    assert_eq!(
        a.quadro(ms(t0, 60)),
        Some(q(2)),
        "no arrasto vê-se o traço FINAL, ao vivo"
    );
    let sossego = 20 + SOSSEGO.as_millis() as u64;
    assert_eq!(
        a.quadro(ms(t0, sossego + 1)),
        Some(q(0)),
        "sossegou: a animação começa"
    );
    assert_eq!(
        a.quadro(ms(t0, sossego + 5000)),
        Some(q(2)),
        "e acaba no final"
    );
}

#[test]
fn a_thread_pinta_e_o_painel_recebe_a_faixa() {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let mut p = PreviewVivo::novo();
    let fim = Instant::now() + Duration::from_secs(20);
    while !p.apresentacao.completo && Instant::now() < fim {
        p.quadro(&t, Instant::now());
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        p.apresentacao.completo,
        "a thread não acabou a cena em 20 s"
    );
    let area =
        (ph2d_tool_painter::LARGURA_DO_PREVIEW * ph2d_tool_painter::ALTURA_DO_PREVIEW) as usize;
    let n = p.apresentacao.quadros.len();
    assert!(n >= 30, "{n} quadros");
    assert!(p.apresentacao.quadros.iter().all(|q| q.len() == area * 4));
    // Uma mudança: a geração nova pinta OUTRO traço.
    let antes = p.apresentacao.quadros.last().cloned().unwrap();
    t.set_brush_size_px(8.0);
    p.quadro(&t, Instant::now());
    let fim = Instant::now() + Duration::from_secs(20);
    while !p.apresentacao.completo && Instant::now() < fim {
        p.quadro(&t, Instant::now());
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_ne!(p.apresentacao.quadros.last().unwrap(), &antes);
}
