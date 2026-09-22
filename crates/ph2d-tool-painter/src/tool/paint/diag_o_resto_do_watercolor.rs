//! **O RESÍDUO DO DESCASQUE, NO WATERCOLOR** — a sonda da auditoria de 2026-09-22 (ordem do dono:
//! *«agora que sabemos como resolver vários tipos de artefatos no painter, quero uma auditoria do
//! modo watercolor buscando por problemas similares»*).
//!
//! ## A hipótese, lida do código antes de medir
//!
//! O caminho de figura do watercolor tem o descasque dele
//! ([`super::stamp_preview::PainterTool::stamp_drag_preview_watercolor`]), e ali **a caixa que se
//! SALVA e a janela que se ESCREVE são calculadas por duas expressões diferentes** — a mesma forma
//! de defeito que o [Bug #24](../../../../../docs/Painter/BUGS_painter.md) curou no composite:
//!
//! | | quem calcula | expressão |
//! |---|---|---|
//! | **SALVA** | `watercolor_preview_footprint` | `edge_spread·2 + warp + 4`, do pincel **ACTUAL** |
//! | **ESCREVE** | `watercolor_render/window.rs` | `reach + warp + 2`, com `reach` das **MÁXIMAS DA SESSÃO** e `union(wet_cum_dirty, frame)` |
//!
//! ⇒ duas divergências alcançáveis por este caminho, e **nenhuma delas é uma opinião**:
//!
//! 1. as **máximas da sessão** (`session_maxima`) contra o pincel actual — um knob BAIXADO entre
//!    dois re-carimbos deixa a escrita no valor VELHO e a caixa no NOVO;
//! 2. a janela é **CUMULATIVA** (`union(wet_cum_dirty, frame)`) e a caixa é a do quadro; e o
//!    `wet_cum_dirty` só é limpo em `open_stroke` / `pour_canvas_wet_inner` / `wet_canvas_now` —
//!    **nenhum deles corre por re-carimbo de figura**.
//!
//! ⚠️ A 3.ª divergência lida no código (o **alcance da RESERVA**) **não é alcançável aqui** e está
//! medida: ela entra atrás de `stroke_deplete.len() == n`, e o caminho da figura nunca abre um
//! traço molhado ⇒ o plano não existe. *Uma divergência que o produto não alcança é uma nota, não
//! um defeito* — e é por isso que a coluna `deplete` é impressa.
//!
//! ## O oráculo: INDEPENDÊNCIA DO CAMINHO
//!
//! ⭐ *A tela não pode lembrar-se de um raio nem de um spread por onde a mão apenas PASSOU.* Duas
//! corridas que acabam no **mesmo** estado — uma que passa por um valor maior, outra que vai
//! direita ao final — têm de deixar a tela **byte-idêntica**. Tudo o que diferir é o que o
//! descasque não alcançou.
//!
//! ⛔⛔ **A FIXTURA TEM DE PARQUEAR A FIGURA, e isso foi MEDIDO e não escolhido.** A 1.ª redacção
//! desta sonda deixava a elipse **EM VOO** e leu `MEXEU = 0` em todas as células — **o controlo
//! digital incluído**. A causa é a 1.ª instrução do
//! [`super::stroke_multi::PainterTool::restamp_shapes_preview`]: com `draft_stamp()` verdadeiro
//! (que é *todo* o gesto até ao `Up` — `shape_draft = ev.phase != Up`) ele **descasca e volta sem
//! carimbar nada**, em QUALQUER meio. *Uma fixtura que não larga o ponteiro mede um programa que
//! nunca pinta, e `0` é o mesmo byte de um produto limpo.*
//!
//! ⚠️ **A tela leva ARTE por baixo**, e não é decoração: a §1 da
//! [auditoria da pilha](../../../../../docs/Painter/40_auditoria_da_pilha_2026-09-21.md) mediu que
//! sobre tela vazia estes caminhos movem `0` pixels — *uma fixtura vazia lê `0` e não afirma nada*.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
//!     diag_o_resto_do_watercolor -- --ignored --nocapture --test-threads=1
//! ```

use super::diag_auditoria_da_pilha::{cp, tela_com_arte};
use super::*;

const S: u32 = 1024;
const C: [f32; 2] = [512.0, 512.0];

/// A cena: arte por baixo, meio **Watercolor** armado, composite DESLIGADO (ele nem chega a correr
/// no watercolor — o `stamp_dabs_dispatch` devolve antes do `stamp_dabs_routed`).
fn cena_wc(raio: f32, spread: f32, rewet: f32, charge: f32) -> PainterTool {
    let mut t = tela_com_arte(raio, 255);
    t.paint.composite_enabled = false;
    t.paint.brush.watercolor = true;
    t.paint.brush.radius_px = raio;
    t.paint.brush.edge_spread = spread;
    t.paint.brush.warp = 0.0;
    t.paint.brush.wet_rewet = rewet;
    t.paint.brush.wet_charge = charge;
    t
}

/// Parqueia UMA elipse — a rota que de facto carimba (o idioma de `tests/parked_shapes.rs`).
fn parqueia(t: &mut PainterTool, centro: [f32; 2], r: f32) {
    t.paint.parked_shapes.clear();
    t.paint.parked_shapes.push(stroke_multi::StrokeShape {
        state: crate::undo::ShapeEditState::Ellipse(crate::undo::EllipseState {
            center: centro,
            u: [1.0, 0.0],
            rx: r,
            ry: r,
            editing: false,
            seed: 1,
        }),
        op: stroke_multi::StrokeOp::Overlay,
    });
}

/// Os texels em que DUAS telas diferem, com a distância ao centro da figura.
fn difere(a: &[u8], b: &[u8]) -> Vec<(u32, u32, u8, f32)> {
    let mut v = Vec::new();
    for y in 0..S {
        for x in 0..S {
            let i = ((y * S + x) * 4) as usize;
            let d = (0..4).map(|k| a[i + k].abs_diff(b[i + k])).max().unwrap_or(0);
            if d > 0 {
                let dx = x as f32 - C[0];
                let dy = y as f32 - C[1];
                v.push((x, y, d, (dx * dx + dy * dy).sqrt()));
            }
        }
    }
    v
}

/// Imprime uma linha com os CONTROLOS POSITIVOS à frente do veredito.
fn linha(nome: &str, a: &PainterTool, b: &PainterTool, limpa: &PainterTool, extra: String) {
    let mexeu = difere(&limpa.canvas_rgba, &b.canvas_rgba).len();
    let v = difere(&a.canvas_rgba, &b.canvas_rgba);
    let pior = v.iter().map(|t| t.2).max().unwrap_or(0);
    let r_min = v.iter().map(|t| t.3).fold(f32::INFINITY, f32::min);
    let r_max = v.iter().map(|t| t.3).fold(0.0f32, f32::max);
    println!(
        "{nome:<38} | vivo={:<5} MEXEU={mexeu:>7} armou={:<5} base={:<5} deplete={:<5} \
         | difere={:>7} pior={pior:>3} banda=[{:>6.1},{:>6.1}] | {extra}",
        limpa.watercolor_render_active(),
        a.paint.wet_shape_active,
        a.paint.watercolor_base.is_some(),
        a.paint.stroke_deplete.len() == (S * S) as usize,
        v.len(),
        if v.is_empty() { 0.0 } else { r_min },
        r_max,
    );
}

/// **(1) O KNOB BAIXADO:** re-carimbar com `spread` alto e depois com o final, contra ir direito ao
/// final. A caixa salva segue o pincel ACTUAL; a janela de escrita segue a MÁXIMA DA SESSÃO.
fn celula_knob(nome: &str, raio: f32, alto: f32, final_: f32, rewet: f32, charge: f32) {
    let mut a = cena_wc(raio, alto, rewet, charge);
    parqueia(&mut a, C, 160.0);
    a.restamp_shapes_preview(&[]);
    a.paint.brush.edge_spread = final_;
    a.restamp_shapes_preview(&[]);

    let mut b = cena_wc(raio, final_, rewet, charge);
    parqueia(&mut b, C, 160.0);
    b.restamp_shapes_preview(&[]);

    let limpa = cena_wc(raio, final_, rewet, charge);
    let salva = final_.round().clamp(0.0, 48.0) * 2.0 + 4.0;
    linha(nome, &a, &b, &limpa, format!("salva_pad={salva:>5.1} sessao_pad={:>5.1}", alto.round().clamp(0.0, 48.0) * 2.0 + 4.0));
}

/// **(2) A FIGURA QUE ANDA:** re-carimbar numa posição e depois noutra, contra ir direito à segunda.
/// A janela de escrita é `union(wet_cum_dirty, frame)` e o `wet_cum_dirty` nunca é limpo aqui.
fn celula_move(nome: &str, raio: f32, spread: f32, dx: f32, rewet: f32, charge: f32) {
    let mut a = cena_wc(raio, spread, rewet, charge);
    parqueia(&mut a, [C[0] - dx, C[1]], 160.0);
    a.restamp_shapes_preview(&[]);
    parqueia(&mut a, C, 160.0);
    a.restamp_shapes_preview(&[]);

    let mut b = cena_wc(raio, spread, rewet, charge);
    parqueia(&mut b, C, 160.0);
    b.restamp_shapes_preview(&[]);

    let limpa = cena_wc(raio, spread, rewet, charge);
    linha(nome, &a, &b, &limpa, format!("dx={dx:>5.0}"));
}

/// **(3) A ABLAÇÃO que ATRIBUI:** re-carimbar duas vezes, mas largando o PLANO DA RESERVA entre as
/// duas (`stroke_deplete = Vec::new()`), que é o que faz o guarda do `watercolor_accum` re-correr o
/// ramo de alocação — o único sítio que SEMEIA o `stroke_deplete_prox` e faz o backfill `= 255` sob
/// a cobertura. ⚠️ Isto NÃO é uma cura: é a medição que separa *«o plano existe e está a zero»* de
/// todas as outras explicações. Se o desvio for a zero, a causa está nomeada.
fn celula_ablacao(nome: &str, raio: f32, spread: f32, rewet: f32, charge: f32, largar: bool) {
    let mut a = cena_wc(raio, spread, rewet, charge);
    parqueia(&mut a, C, 160.0);
    a.restamp_shapes_preview(&[]);
    if largar {
        a.paint.stroke_deplete = Vec::new();
        a.paint.stroke_deplete_prox = Vec::new();
    }
    a.restamp_shapes_preview(&[]);

    let mut b = cena_wc(raio, spread, rewet, charge);
    parqueia(&mut b, C, 160.0);
    b.restamp_shapes_preview(&[]);

    let limpa = cena_wc(raio, spread, rewet, charge);
    linha(nome, &a, &b, &limpa, format!("largou_o_plano={largar}"));
}

/// **(4) A BISSECÇÃO:** o mesmo par de re-carimbos, repondo UM campo de estado entre os dois. O
/// campo cuja reposição leva o desvio a `0` é a causa. ⚠️ Medição, não cura.
fn celula_campo(nome: &str, repor: impl Fn(&mut PainterTool)) {
    let mut a = cena_wc(200.0, 7.0, 0.0, 0.5);
    parqueia(&mut a, C, 160.0);
    a.restamp_shapes_preview(&[]);
    repor(&mut a);
    a.restamp_shapes_preview(&[]);

    let mut b = cena_wc(200.0, 7.0, 0.0, 0.5);
    parqueia(&mut b, C, 160.0);
    b.restamp_shapes_preview(&[]);

    let v = difere(&a.canvas_rgba, &b.canvas_rgba);
    let pior = v.iter().map(|t| t.2).max().unwrap_or(0);
    println!("  repondo {nome:<34} -> difere={:>7} pior={pior:>3}", v.len());
}

/// **(0) O DESCASQUE É EXACTO?** Carimbar, descascar à mão, e comparar com a tela intocada. Se isto
/// for `0`, a caixa salva cobre tudo o que a janela escreveu e o defeito NÃO é geométrico.
fn celula_peel(nome: &str, rewet: f32, charge: f32) {
    let mut a = cena_wc(200.0, 7.0, rewet, charge);
    parqueia(&mut a, C, 160.0);
    a.restamp_shapes_preview(&[]);
    a.peel_drag_preview();
    let limpa = cena_wc(200.0, 7.0, rewet, charge);
    let v = difere(&a.canvas_rgba, &limpa.canvas_rgba);
    let pior = v.iter().map(|t| t.2).max().unwrap_or(0);
    let r_max = v.iter().map(|t| t.3).fold(0.0f32, f32::max);
    println!("  descasque {nome:<32} -> resto={:>7} pior={pior:>3} r_max={r_max:>6.1}", v.len());
}

/// **(5) A ESCADA:** `n` re-carimbos contra UM. Um artista mexe num knob dezenas de vezes; se o
/// desvio CRESCER com `n`, a aguada de uma figura depende de quantas vezes lhe tocaram.
fn celula_escada(n: usize, charge: f32) {
    let mut a = cena_wc(200.0, 7.0, 0.0, charge);
    parqueia(&mut a, C, 160.0);
    for _ in 0..n {
        a.restamp_shapes_preview(&[]);
    }
    let mut b = cena_wc(200.0, 7.0, 0.0, charge);
    parqueia(&mut b, C, 160.0);
    b.restamp_shapes_preview(&[]);
    let v = difere(&a.canvas_rgba, &b.canvas_rgba);
    let pior = v.iter().map(|t| t.2).max().unwrap_or(0);
    let i = ((C[1] as u32 * S + C[0] as u32) * 4) as usize;
    println!(
        "  n={n:<2} charge={charge:<4} -> difere={:>7} pior={pior:>3} centro_a={:?} centro_b={:?}",
        v.len(),
        &a.canvas_rgba[i..i + 3],
        &b.canvas_rgba[i..i + 3],
    );
}

/// A TABELA. ⚠️ Ela não conserta nada — ela mede, e cada linha é uma ablação de UM termo.
#[test]
#[ignore = "sonda de auditoria: imprime a tabela, não afirma uma barra"]
fn diag_o_resto_do_watercolor() {
    println!("\n== o mesmo estado final por DOIS caminhos; tudo o que difere é resíduo ==\n");

    // CONTROLO NEGATIVO: sem watercolor, o caminho é o do composite — curado em 2026-09-21.
    {
        let mut a = tela_com_arte(200.0, 255);
        a.paint.composite_enabled = false;
        parqueia(&mut a, [C[0] - 120.0, C[1]], 160.0);
        a.restamp_shapes_preview(&[]);
        parqueia(&mut a, C, 160.0);
        a.restamp_shapes_preview(&[]);
        let mut b = tela_com_arte(200.0, 255);
        b.paint.composite_enabled = false;
        parqueia(&mut b, C, 160.0);
        b.restamp_shapes_preview(&[]);
        let mut limpa = tela_com_arte(200.0, 255);
        limpa.paint.composite_enabled = false;
        let mexeu = difere(&limpa.canvas_rgba, &b.canvas_rgba).len();
        let v = difere(&a.canvas_rgba, &b.canvas_rgba);
        println!(
            "{:<38} | {:>52} | difere={:>7} MEXEU={mexeu:>7} (CONTROLO: digital, sem watercolor)",
            "digital, figura que ANDA", "", v.len()
        );
    }

    println!();
    // (1) o knob BAIXADO entre dois re-carimbos.
    celula_knob("wc, spread 40 -> 7, sem mixer", 200.0, 40.0, 7.0, 0.0, 1.0);
    celula_knob("wc, spread 40 -> 7, MIXER armado", 200.0, 40.0, 7.0, 0.0, 0.5);
    celula_knob("wc, spread 40 -> 7, REWET alto", 200.0, 40.0, 7.0, 1.0, 0.5);
    celula_knob("wc, spread  7 ->  7 (controlo)", 200.0, 7.0, 7.0, 0.0, 1.0);
    // ⭐⭐ O CONTROLO QUE DISCRIMINA: o MESMO spread nos dois re-carimbos, com o mixer ARMADO.
    // Se ele diferir, o defeito NÃO e o knob — e o dois re-carimbos que nao fecham.
    celula_knob("wc, spread  7 ->  7, MIXER (ctrl)", 200.0, 7.0, 7.0, 0.0, 0.5);
    celula_knob("wc, spread  7 ->  7, REWET (ctrl)", 200.0, 7.0, 7.0, 1.0, 0.5);

    println!();
    // (2) a figura que ANDA (o `wet_cum_dirty` cumulativo).
    celula_move("wc, figura ANDA 120 px", 200.0, 7.0, 120.0, 0.0, 1.0);
    celula_move("wc, figura ANDA 120 px, MIXER", 200.0, 7.0, 120.0, 0.0, 0.5);
    celula_move("wc, figura ANDA   0 px (controlo)", 200.0, 7.0, 0.0, 0.0, 1.0);
    celula_move("wc, figura ANDA   0 px, MIXER (ctrl)", 200.0, 7.0, 0.0, 0.0, 0.5);

    println!();
    // (3) a ABLAÇÃO: o mesmo par, largando o plano da reserva entre os dois re-carimbos.
    celula_ablacao("wc, 2 re-carimbos, MIXER", 200.0, 7.0, 0.0, 0.5, false);
    celula_ablacao("wc, 2 re-carimbos, MIXER, LARGA plano", 200.0, 7.0, 0.0, 0.5, true);
    celula_ablacao("wc, 2 re-carimbos, REWET, LARGA plano", 200.0, 7.0, 1.0, 0.5, true);

    println!("\n-- (0) o DESCASQUE deixa resto? --");
    celula_peel("sem mixer", 0.0, 1.0);
    celula_peel("MIXER armado", 0.0, 0.5);
    celula_peel("REWET alto", 1.0, 0.5);

    println!("\n-- (4) bisseccao por CAMPO (mixer armado; 377756 = sem cura) --");
    celula_campo("nada (controlo)", |_| {});
    celula_campo("wet_mix", |t| t.paint.wet_mix = Default::default());
    celula_campo("wet_styles", |t| t.paint.wet_styles = Default::default());
    celula_campo("stroke_water", |t| t.paint.stroke_water = Vec::new());
    celula_campo("stroke_density", |t| t.paint.stroke_density = Vec::new());
    celula_campo("wet_soak(+pos,+active)", |t| {
        t.paint.wet_soak = Vec::new();
        t.paint.wet_soak_pos = None;
        t.paint.wet_soak_active = false;
    });
    celula_campo("wet_shape_active=false", |t| t.paint.wet_shape_active = false);
    celula_campo("wet_backdrop", |t| t.paint.wet_backdrop = None);
    celula_campo("wet_session_canvas", |t| t.paint.wet_session_canvas = None);
    celula_campo("wet_level_smear_pos", |t| t.paint.wet_level_smear_pos = None);

    println!("\n-- (5) a ESCADA: n re-carimbos contra UM --");
    for n in [1usize, 2, 3, 5, 10, 30] {
        celula_escada(n, 0.5);
    }
    celula_escada(10, 1.0); // CONTROLO: mixer desarmado

    println!("\n(banda = distâncias ao centro onde há diferença; salva_pad = o que a caixa salva\n \
              cobre além do dab no pincel FINAL; sessao_pad = o que ela cobriria no valor VELHO)\n");
}
