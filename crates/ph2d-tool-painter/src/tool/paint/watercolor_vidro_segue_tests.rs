//! **O vidro da aguada segue a base que o borrão move** (B18.4, Watercolor Basic a 4K): com Smudge, o borrão
//! escreve a base no lugar e o `make_mut` move-a para um `Arc` novo; o `Weak` do vidro congelado morria, o
//! vidro recongelava a partir do plano JÁ escrito e copiava-o inteiro em todo quadro (58 MB a 3840×2160).
//! O oráculo é o caminho de antes (`VIDRO_SEGUE_A_BASE = false`) no mesmo processo.

use super::watercolor_vidro::VIDRO_SEGUE_A_BASE;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use std::sync::Arc;

const W: u32 = 256;
const H: u32 = 160;

fn cp(x: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, 80.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um traço com Smudge: os píxeis da tela depois de cada tique, quantas vezes o plano do vidro mudou de `Arc`
/// e quantas vezes a sessão do vidro congelou (ADR-0205 D3: recongelar já não copia o plano, recomeça a sessão).
fn traco(segue: bool) -> (Vec<Vec<u8>>, usize, usize, usize) {
    VIDRO_SEGUE_A_BASE.set(segue);
    super::watercolor_vidro::CONGELACOES.set(0);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(10.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.paint.brush.wet_smudge = 0.4;
    let mut telas = Vec::new();
    let (mut planos, mut trocas) = (None, 0usize);
    let mut ev = |t: &mut PainterTool, x: f32, phase| {
        t.on_canvas_pointer(cp(x, phase));
        t.on_tick(1000.0 / 60.0);
        telas.push(t.canvas_rgba.to_vec());
        let p = t
            .layers
            .active()
            .and_then(|c| t.vidros.get(&c))
            .map(Arc::as_ptr);
        if p.is_some() && planos.is_some() && p != planos {
            trocas += 1;
        }
        planos = p;
    };
    ev(&mut t, 30.0, PointerPhase::Down);
    let moves = 14;
    for k in 1..=moves {
        ev(&mut t, 30.0 + 12.0 * k as f32, PointerPhase::Move);
    }
    VIDRO_SEGUE_A_BASE.set(true);
    let congelacoes = super::watercolor_vidro::CONGELACOES.get();
    (telas, trocas, moves, congelacoes)
}

#[test]
fn o_vidro_segue_a_base_que_o_borrao_move_sem_copiar_o_plano() {
    let (antes, _, moves, cong_antes) = traco(false);
    let (cura, trocas_cura, _, cong_cura) = traco(true);
    assert!(
        cong_antes >= moves - 2,
        "controlo: o caminho de antes recongela o vidro em quase todo quadro ({cong_antes} de {moves})"
    );
    assert_eq!(
        cong_cura, 1,
        "a sessão do vidro recongelou {cong_cura} vezes num traço"
    );
    assert!(
        trocas_cura <= 1,
        "o plano do vidro foi recopiado {trocas_cura} vezes num traço (só o 1.º quadro o pode copiar)"
    );
    assert_eq!(
        antes, cura,
        "os píxeis da tela são os mesmos nos dois caminhos (o vidro só pesa no papel de cor)"
    );
}

/// O 1.º traço de uma camada sem vidro não COPIA o plano que acabou de nascer zerado (a 4K, 58 MB tocados no
/// 1.º toque). Controlo: sem a chave `VIDRO_PELO_JOURNAL` o 2.º traço copia (o desfazer segura o plano), e
/// a régua vê a cópia quando ela existe; com ela (ADR-0205) não copia.
#[test]
fn o_primeiro_traco_da_camada_nao_copia_o_vidro_que_nasce_zerado() {
    for journal in [false, true] {
        crate::undo::window::set_vidro_pelo_journal(journal);
        let (primeiro, segundo) = primeiro_e_segundo_traco();
        crate::undo::window::set_vidro_pelo_journal(true);
        assert_eq!(
            primeiro, 0,
            "chave={journal}: o 1.º traço copiou o vidro recém-nascido {primeiro} vez(es)"
        );
        if journal {
            assert_eq!(segundo, 0, "com a chave o 2.º traço copiou o vidro");
        } else {
            assert!(
                segundo >= 1,
                "controlo: sem a chave o 2.º traço copia o vidro que o desfazer segura ({segundo})"
            );
        }
    }
}

fn primeiro_e_segundo_traco() -> (usize, usize) {
    use super::watercolor_vidro::COPIAS_DO_VIDRO;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(10.0);
    let traco = |t: &mut PainterTool, y: f32| {
        COPIAS_DO_VIDRO.set(0);
        let p = |x: f32, phase| CanvasPointer {
            pos: [x, y],
            ..cp(x, phase)
        };
        t.on_canvas_pointer(p(30.0, PointerPhase::Down));
        t.on_tick(1000.0 / 60.0);
        for k in 1..=6 {
            t.on_canvas_pointer(p(30.0 + 12.0 * k as f32, PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(p(110.0, PointerPhase::Up));
        for _ in 0..3 {
            t.on_tick(1000.0 / 60.0);
        }
        COPIAS_DO_VIDRO.get()
    };
    let primeiro = traco(&mut t, 50.0);
    (primeiro, traco(&mut t, 110.0))
}

/// **O `vbase` é o vidro do INÍCIO da sessão** (ADR-0205 D3): três traços WB com Smudge na mesma sessão
/// molhada; cada janela que o quadro lê = o plano tirado inteiro na congelação, ao bit. Controlo: a sessão
/// atravessa traços e, nas janelas dos traços seguintes, o vivo já não é o início (o journal da sessão é que
/// responde; o do passo zerou no commit).
#[test]
fn o_vidro_da_base_da_sessao_e_o_do_inicio_da_sessao_ao_bit() {
    use super::watercolor_vidro::{GUARDA_O_INICIO_DA_SESSAO, JANELAS_DO_VBASE};
    GUARDA_O_INICIO_DA_SESSAO.set(true);
    JANELAS_DO_VBASE.with(|j| j.borrow_mut().clear());
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.paint.brush.wet_smudge = 0.4;
    let mut sessoes = Vec::new();
    let mut vivo_difere = 0usize;
    for (k, y) in [70.0f32, 80.0, 90.0].into_iter().enumerate() {
        let c = t.layers.active().expect("camada");
        let vivo_antes = t.vidros.get(&c).map(|p| p.as_ref().clone());
        let p = |x: f32, phase| CanvasPointer {
            pos: [x, y],
            ..cp(x, phase)
        };
        t.on_canvas_pointer(p(40.0, PointerPhase::Down));
        t.on_tick(1000.0 / 60.0);
        for i in 1..=8 {
            t.on_canvas_pointer(p(40.0 + 16.0 * i as f32, PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(p(170.0, PointerPhase::Up));
        for _ in 0..3 {
            t.on_tick(1000.0 / 60.0);
        }
        let janelas = JANELAS_DO_VBASE.with(|j| std::mem::take(&mut *j.borrow_mut()));
        assert!(!janelas.is_empty(), "traço {k}: nenhum quadro leu o vbase");
        for (j, inicio) in &janelas {
            sessoes.push((k, Arc::as_ptr(inicio)));
            let fw = W as usize;
            for (r, linha) in j.dados.chunks(j.w).enumerate() {
                let y = j.y0 + r;
                assert_eq!(
                    linha,
                    &inicio[y * fw + j.x0..y * fw + j.x0 + j.w],
                    "traço {k}: o vbase na linha {y} não é o vidro do início da sessão"
                );
                if let Some(v) = &vivo_antes
                    && linha != &v[y * fw + j.x0..y * fw + j.x0 + j.w]
                {
                    vivo_difere += 1;
                }
            }
        }
    }
    GUARDA_O_INICIO_DA_SESSAO.set(false);
    let atravessa = sessoes
        .iter()
        .any(|&(k, s)| k > 0 && sessoes.iter().any(|&(k0, s0)| k0 < k && s0 == s));
    assert!(
        atravessa,
        "controlo: a sessão molhada não atravessou traços ({sessoes:?})"
    );
    assert!(
        vivo_difere > 0,
        "controlo: em nenhuma janela o vivo de antes do traço diferia do início da sessão"
    );
}
