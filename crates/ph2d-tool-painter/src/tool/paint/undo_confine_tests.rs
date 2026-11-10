//! **UM CTRL+Z REPINTA SÓ O QUE ELE MUDOU** — os gates do confinamento (doc 28 §5.63).
//!
//! # O que pode dar errado, e qual gate pega
//!
//! O confinamento tem **dois** modos de falha e eles não têm o mesmo preço:
//!
//! * **reivindicar de menos** (`None` onde cabia um retângulo) — custa um repaint. Pego pelo gate de
//!   COMPORTAMENTO, que exige a pista parcial.
//! * **reivindicar de mais** (um retângulo que não cobre tudo o que mudou) — deixa na tela a figura
//!   ANTERIOR fora dele. **Nenhum gate de conteúdo pega isso**, porque dentro do retângulo está tudo
//!   certo. Pego pelo **ORÁCULO**, que compara a tela contra a rota de repaint inteiro.
//!
//! É por isso que o oráculo é o gate central desta wave, e por isso a ablação
//! ([`crate::undo::UndoController::confine`]) existe: sem uma segunda rota não há contra quem comparar.

use super::measure_stroke_owners::{armed, stroke};
use super::*;

/// Pinta `n` traços e devolve o tool.
fn painted(side: u32, n: u8, confine: bool) -> PainterTool {
    let mut t = armed(side);
    t.undo.confine = confine;
    for k in 0..n {
        stroke(&mut t, 200.0 + f32::from(k) * 40.0);
    }
    let _ = t.take_preview_arc();
    t
}

/// **O gate de COMPORTAMENTO: desfazer um traço não repinta a tela inteira.**
///
/// ⚠️ O oráculo é a pista que a drenagem tomou **mais** o retângulo publicado, e as duas metades são
/// necessárias: a pista sozinha ficaria verde com um retângulo do tamanho do canvas (que é repaint
/// inteiro vestindo o nome de parcial), e o retângulo sozinho ficaria verde com a drenagem o ignorando
/// — que é exatamente o estado em que esta wave passou uma hora (o confinamento disparava, o
/// `restore_selection` derrubava o cache, e o quadro seguia em 381 ms).
#[test]
fn an_undone_stroke_repaints_only_the_window_it_touched() {
    let mut t = painted(1024, 2, true);
    // ⚠️ O diagnóstico é capturado ANTES do passo, porque depois dele a entrada já saiu da pilha — e
    // ele entra nas mensagens de falha em vez de viver como helper sem chamador. Um `None` mudo manda a
    // próxima pessoa reconstruir o instrumento que esta wave já pagou (doc 28 §5.63).
    let why = t.undo.peek_confine_diagnosis(false).unwrap_or_default();
    assert!(t.undo_last());
    let rect = t
        .dirty_rect_now()
        .unwrap_or_else(|| panic!("o undo tem de publicar a janela que reescreveu — {why}"));
    assert!(
        t.composited_is_some(),
        "o cache de composite tem de SOBREVIVER a um undo confinado (quem o derruba manda a tela \
         inteira ser refeita, e o retangulo publicado vira decoracao) — {why}"
    );
    t.paint_tick(1.0 / 60.0);
    let _ = t.take_preview_arc();
    assert_eq!(
        t.last_drain_branch,
        crate::tool::DrainBranch::PartialComposite,
        "a drenagem tem de tomar a pista PARCIAL depois de um undo confinado — {why}"
    );
    // ⚠️ A conta é em PIXELS dos dois lados. A 1ª versão deste gate multiplicava a janela por 4
    // (elementos de um plano RGBA) e a comparava com a tela em pixels — reprovando um retângulo de
    // 72.922 px contra uma tela de 262.144. *Um gate que mistura unidades falha pelo motivo errado.*
    let (w, h) = t.source_size;
    let (win, screen) = (
        u64::from(rect.w) * u64::from(rect.h),
        u64::from(w) * u64::from(h),
    );
    assert!(
        win * 4 < screen,
        "a janela ({}x{} = {win} px) tem de ser uma FRAÇÃO da tela ({w}x{h} = {screen} px) — um \
         retângulo do tamanho do canvas é repaint inteiro vestindo o nome de parcial",
        rect.w,
        rect.h
    );
}

/// **O ORÁCULO: a pista parcial mostra EXATAMENTE o que o repaint inteiro mostra.**
///
/// Duas cópias do mesmo tool, dirigidas pelo mesmo roteiro, uma com o confinamento ablacionado. Os
/// bytes que a tela recebe têm de ser **idênticos** — no undo E no redo, porque as duas direções
/// escrevem regiões diferentes e uma delas pode reivindicar de mais sozinha.
///
/// ⚠️ **Este é o único gate que pode ver "reivindicou de mais"**, e ele não sabe nada sobre janelas: ele
/// compara duas TELAS. Um gate que afirmasse propriedades do retângulo estaria espelhando a regra que
/// julga.
#[test]
fn a_confined_undo_shows_exactly_what_a_full_repaint_shows() {
    let (mut fast, mut slow) = (painted(256, 3, true), painted(256, 3, false));
    let screen = |t: &mut PainterTool| {
        t.paint_tick(1.0 / 60.0);
        t.take_preview_arc().map(|(px, _, _)| px)
    };
    for step in 0..3u8 {
        assert!(
            fast.undo_last() && slow.undo_last(),
            "a fila tem de ter passos"
        );
        let (a, b) = (screen(&mut fast), screen(&mut slow));
        assert_eq!(
            a, b,
            "undo {step}: a pista parcial divergiu do repaint inteiro"
        );
    }
    for step in 0..3u8 {
        assert!(
            fast.redo_last() && slow.redo_last(),
            "a fila de redo tem de ter passos"
        );
        let (a, b) = (screen(&mut fast), screen(&mut slow));
        assert_eq!(
            a, b,
            "redo {step}: a pista parcial divergiu do repaint inteiro"
        );
    }
    // ⚠️ Controle: sem isto o gate ficaria verde comparando duas telas que a ablação nunca separou.
    assert!(
        fast.undo.confine && !slow.undo.confine,
        "a ablacao nao esta armada — os dois bracos rodam o MESMO caminho e o gate e' verde por vacuo"
    );
}

/// **Uma edição ESTRUTURAL não é confinada** — a cerca de Chesterton do `invalidate_composite`, que
/// esta wave estreita em vez de derrubar. Mudar a opacidade de uma camada muda a figura em TODA parte.
///
/// ⚠️ **O passo tem de tocar pixels TAMBÉM, e a 1ª versão deste gate não tocava.** Sem escrita de
/// canvas os dezenove planos saem `Untouched`, o acumulador termina sem região e o veredito já é `None`
/// **pela metade dos PLANOS** — então a mutação *"os metadados são sempre confinados"* passava, e o
/// gate era verde por vácuo sobre a metade que ele diz julgar. Com a escrita, os planos dizem
/// *confinado* e só a metade dos metadados pode recusar.
#[test]
fn a_structural_edit_is_not_confined() {
    let mut t = painted(256, 1, true);
    let id = t.layers.active().expect("camada ativa");
    let before = t.snapshot_model();
    t.set_layer_opacity(id, 0.5);
    // Uma escrita de canvas pequena, para os planos declararem uma janela e a prova cair inteira na
    // metade dos metadados.
    {
        let buf = std::sync::Arc::make_mut(&mut t.canvas_rgba);
        for px in buf.as_chunks_mut::<4>().0.iter_mut().skip(600).take(40) {
            px[0] = px[0].wrapping_add(37);
        }
    }
    t.commit_structural_edit(before);
    assert!(
        t.undo.peek_confined_region(false).is_none(),
        "uma mudanca de opacidade muda o composite fora de retangulo nenhum"
    );
}

/// O documento do smoke 4K, em pequeno: a sprite nascida «Textured Paper» aberta no `meio`.
fn papel_texturizado(
    side: u32,
    meio: crate::tool::paint::media::PaintMedia,
    confine: bool,
) -> PainterTool {
    let mut t = PainterTool::default();
    let px = PainterTool::pixeis_do_papel_texturizado(side, side);
    t.abre_a_sprite_nova(7, px, side, side);
    t.set_paint_media(meio);
    t.set_brush_size_px(9.0);
    t.undo.confine = confine;
    stroke(&mut t, 120.0);
    let _ = t.take_preview_arc();
    t
}

/// **B10c: o Ctrl+Z do 1.º traço sobre o papel texturizado é confinado no Digital, Watercolor e Impasto** — e
/// a tela da pista parcial é a do repaint inteiro ao bit (undo e redo) nos 4 meios. Era `None` no Watercolor
/// (o vidro aparece: `v=WHOLE`) e no Impasto (o relevo aparece e o `has_relief` vira), dobra e prévia da
/// TELA no app. O 1.º traço Wet (`tintas_molhadas`) segue não confinado: só o oráculo o julga.
#[test]
fn o_ctrl_z_de_um_traco_no_papel_e_confinado_em_cada_meio() {
    use crate::tool::paint::media::PaintMedia::{Digital, Impasto, Watercolor, WetPaint};
    let screen = |t: &mut PainterTool| {
        t.paint_tick(1.0 / 60.0);
        t.take_preview_arc().map(|(px, _, _)| px)
    };
    let mut falhas = Vec::new();
    for meio in [Digital, Impasto, Watercolor, WetPaint] {
        let (mut fast, mut slow) = (
            papel_texturizado(256, meio, true),
            papel_texturizado(256, meio, false),
        );
        let why = fast.undo.peek_confine_diagnosis(false).unwrap_or_default();
        if meio == Watercolor {
            assert!(
                why.contains("v=WHOLE"),
                "controlo: o vidro aparece neste passo — {why}"
            );
        }
        let deve = matches!(meio, Digital | Watercolor | Impasto);
        if deve && fast.undo.peek_confined_region(false).is_none() {
            falhas.push(format!("{meio:?}: {why}"));
        }
        assert!(
            fast.undo_last() && slow.undo_last(),
            "{meio:?}: há o que desfazer"
        );
        assert_eq!(
            screen(&mut fast),
            screen(&mut slow),
            "{meio:?}: undo ≠ repaint inteiro"
        );
        assert!(
            fast.redo_last() && slow.redo_last(),
            "{meio:?}: há o que refazer"
        );
        assert_eq!(
            screen(&mut fast),
            screen(&mut slow),
            "{meio:?}: redo ≠ repaint inteiro"
        );
    }
    assert!(falhas.is_empty(), "não confinados: {falhas:#?}");
}

/// As texturas de planos da pista da PLACA: persistem e cada quadro escreve só a janela dobrada
/// (`compose_light_premul` em `ph2d-app-painter`). Vazia até a 1.ª dobra da tela (`planes_seeded`).
#[derive(Default)]
struct TexturasDaPlaca {
    relevo: Vec<u32>,
    cobertura: Vec<u8>,
    mat0: Vec<u8>,
    mat1: Vec<u8>,
}

impl TexturasDaPlaca {
    /// Um quadro da pista: drena o sujo, dobra a janela (a tela se ainda não semeada) e escreve-a.
    /// Devolve a janela dobrada.
    fn quadro(&mut self, t: &mut PainterTool) -> (u32, u32, u32, u32) {
        let _ = t.take_preview_dirty();
        let (w, h) = t.source_size;
        let janela = t
            .preview_gpu_region()
            .filter(|_| !self.relevo.is_empty())
            .unwrap_or((0, 0, w, h));
        let Some(p) = t.impasto_gpu_planes_in(janela) else {
            return janela;
        };
        if self.relevo.is_empty() {
            let n = (w * h) as usize;
            *self = Self {
                relevo: vec![0; n],
                cobertura: vec![0; n],
                mat0: vec![0; n * 4],
                mat1: vec![0; n * 4],
            };
        }
        let (x, y, pw, ph) = p.region;
        for r in 0..ph as usize {
            let (de, para) = (r * pw as usize, (y as usize + r) * w as usize + x as usize);
            let n = pw as usize;
            for (d, s) in self.relevo[para..para + n]
                .iter_mut()
                .zip(&p.relief[de..de + n])
            {
                *d = s.to_bits();
            }
            self.cobertura[para..para + n].copy_from_slice(&p.cover[de..de + n]);
            self.mat0[para * 4..(para + n) * 4].copy_from_slice(&p.mat0[de * 4..(de + n) * 4]);
            self.mat1[para * 4..(para + n) * 4].copy_from_slice(&p.mat1[de * 4..(de + n) * 4]);
        }
        janela
    }

    /// O que a placa ilumina é a dobra da tela inteira, ao bit (quando há luz).
    fn e_a_dobra_inteira(&self, t: &PainterTool, caso: &str) {
        let Some(p) = t.impasto_gpu_planes() else {
            return;
        };
        let bits: Vec<u32> = p.relief.iter().map(|v| v.to_bits()).collect();
        assert!(
            bits == self.relevo,
            "{caso}: relevo da placa ≠ dobra inteira"
        );
        assert!(
            p.cover == self.cobertura,
            "{caso}: cobertura da placa ≠ dobra inteira"
        );
        assert!(p.mat0 == self.mat0, "{caso}: mat0 da placa ≠ dobra inteira");
        assert!(p.mat1 == self.mat1, "{caso}: mat1 da placa ≠ dobra inteira");
    }
}

/// **B10c: a pista da PLACA no Ctrl+Z do 1.º traço Impasto** (com e sem papel): a janela que ela dobra é a do
/// traço, não a tela (o quadro de 80 ms), e as texturas persistentes remendadas por ela são a dobra inteira ao
/// bit, no undo e no redo. Reivindicar de menos (uma linha de relevo fora da janela) reprova aqui.
#[test]
fn a_placa_dobra_so_a_janela_do_ctrl_z_do_1o_traco_impasto() {
    use crate::tool::paint::media::PaintMedia::Impasto;
    for papel in [true, false] {
        let mut t = if papel {
            let mut t = PainterTool::default();
            t.abre_a_sprite_nova(
                7,
                PainterTool::pixeis_do_papel_texturizado(512, 512),
                512,
                512,
            );
            t
        } else {
            armed(512)
        };
        t.set_paint_media(Impasto);
        t.set_brush_size_px(9.0);
        let mut placa = TexturasDaPlaca::default();
        placa.quadro(&mut t);
        stroke(&mut t, 120.0);
        placa.quadro(&mut t);
        placa.e_a_dobra_inteira(&t, &format!("papel={papel} traço"));
        assert!(
            !placa.relevo.is_empty(),
            "papel={papel}: o traço Impasto acende a luz"
        );
        for passo in ["undo", "redo"] {
            let feito = if passo == "undo" {
                t.undo_last()
            } else {
                t.redo_last()
            };
            assert!(feito, "papel={papel}: há {passo}");
            let (w, h) = t.source_size;
            let (_, _, jw, jh) = placa.quadro(&mut t);
            assert!(
                u64::from(jw) * u64::from(jh) * 4 < u64::from(w) * u64::from(h),
                "papel={papel} {passo}: a placa dobra a TELA ({jw}x{jh})"
            );
            placa.e_a_dobra_inteira(&t, &format!("papel={papel} {passo}"));
        }
    }
}

/// **Dois planos VAZIOS não azedam o veredito** — a lição que custou a wave (doc 28 §5.63).
///
/// O `split` manda para `Whole` tudo o que não sabe medir, e comprimento zero não é medível; as seis
/// superfícies da sessão de Sculpt são vazias num traço de pigmento comum. Lidas como `Whole`, elas
/// tornavam **todo** passo do produto não-confinado.
#[test]
fn two_empty_planes_do_not_sour_the_confinement() {
    use crate::undo_delta::StoredPlane;
    use crate::undo_delta::confine::PlaneReach;
    let empty: StoredPlane<f32> = StoredPlane::Whole {
        before: std::sync::Arc::new(Vec::new()),
        after: std::sync::Arc::new(Vec::new()),
    };
    assert_eq!(empty.reach(), PlaneReach::Untouched);
    let full: StoredPlane<f32> = StoredPlane::Whole {
        before: std::sync::Arc::new(vec![0.0; 4]),
        after: std::sync::Arc::new(vec![1.0; 4]),
    };
    assert_eq!(
        full.reach(),
        PlaneReach::Whole,
        "um plano com conteudo segue Whole"
    );
}
