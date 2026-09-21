//! **O esfregão numa CURVA, e a TROCA que ele esconde** — o report do dono de 2026-09-21
//! (*«dois aspectos no mesmo círculo»*), com a pilha dele declarada a seguir: *«todas as camadas
//! ativas, com tamanhos diferentes, uma textura em Shape; Jitter 0»*.
//!
//! ⛔⛔⛔ **A conclusão desta medição é uma RECUSA, não uma cura.** O esfregão perde `15,6 %` da
//! tinta de uma figura fechada, e a causa é a corrente `D(p) = v + D(p − v)` alcançar muito além
//! dos dabs que tocaram cada texel. O tecto que a cura existe
//! ([`ph2d_painter_brush::TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`]) e **mata o transporte longo que o
//! dono exigiu duas vezes** — *«as fronteiras não são vencidas. o relevo não é levado além»*.
//!
//! ⇒ Este módulo não afirma que o defeito está curado. Ele prende as **duas metades da troca**,
//! para que ninguém adopte o tecto sem ver o preço nem apague o instrumento sem refazer a
//! medição.

use super::composite::CompositeOp;
use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const L: u32 = 700;
const C: [f32; 2] = [350.0, 350.0];
const R: f32 = 215.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn cena(esfregao: f32, metodo: ph2d_painter_brush::StrokeMethod) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
    t.paint.brush.radius_px = 30.0;
    t.paint.brush.color = [0.75, 0.12, 0.12];
    t.paint.brush.space_attenuation = false;
    t.paint.brush.stroke_method = metodo;
    t.paint.composite_enabled = true;
    for (op, s) in [(CompositeOp::Smear, esfregao), (CompositeOp::Brush, 1.0)] {
        t.acrescenta_camada(op.to_u8());
        let pos = t.composite_len() - 1;
        t.set_composite_layer_strength(pos, s);
    }
    t
}

fn anel(t: &mut PainterTool) {
    t.on_canvas_pointer(cp(C, PointerPhase::Down));
    for i in 1..=8 {
        let u = i as f32 / 8.0;
        t.on_canvas_pointer(cp([C[0] + R * u, C[1]], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([C[0] + R, C[1]], PointerPhase::Up));
}

fn tinta(t: &PainterTool) -> f64 {
    (0..(L * L) as usize)
        .map(|i| f64::from(t.canvas_rgba[i * 4 + 3]))
        .sum::<f64>()
        / 255.0
}

/// A fracção da tinta de um anel que sobrevive ao esfregão, com o tecto pedido.
fn anel_guardado(tecto: f32) -> f64 {
    super::smear_warp::espia::poe_tecto(tecto);
    let mut com = cena(1.0, ph2d_painter_brush::StrokeMethod::Ellipse);
    anel(&mut com);
    super::smear_warp::espia::poe_tecto(tecto);
    let mut sem = cena(0.0, ph2d_painter_brush::StrokeMethod::Ellipse);
    anel(&mut sem);
    let (a, b) = (tinta(&com), tinta(&sem));
    assert!(b > 10_000.0, "a fixtura não pintou o anel ({b:.0})");
    100.0 * a / b
}

/// **Até onde a faca leva a tinta** — a propriedade que o dono exigiu, em píxeis.
///
/// Uma mancha curta à esquerda, e depois um arrasto LONGO do esfregão sozinho por cima dela. O
/// que se mede é o `x` mais distante que ficou com tinta: é isso que *«o relevo é levado além»*
/// quer dizer, medido no canal da cor, que sai pela MESMA porta que o relevo.
fn ate_onde_leva(tecto: f32) -> u32 {
    super::smear_warp::espia::poe_tecto(tecto);
    let mut t = cena(0.0, ph2d_painter_brush::StrokeMethod::Space);
    // A mancha: uma coluna curta em x ≈ 100.
    t.on_canvas_pointer(cp([100.0, 320.0], PointerPhase::Down));
    for i in 1..=12 {
        t.on_canvas_pointer(cp([100.0, 320.0 + i as f32 * 5.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([100.0, 380.0], PointerPhase::Up));

    // Só o esfregão, num arrasto longo para a direita.
    t.set_composite_layer_strength(0, 1.0);
    t.set_composite_layer_strength(1, 0.0);
    super::smear_warp::espia::poe_tecto(tecto);
    t.on_canvas_pointer(cp([100.0, 350.0], PointerPhase::Down));
    for i in 1..=250 {
        t.on_canvas_pointer(cp([100.0 + i as f32 * 2.0, 350.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([600.0, 350.0], PointerPhase::Up));

    (0..L)
        .rev()
        .find(|&x| (0..L).any(|y| t.canvas_rgba[((y * L + x) * 4 + 3) as usize] > 4))
        .unwrap_or(0)
}

/// ⭐⭐⭐ **A TROCA, nas duas metades** — o tecto cura a curva **e** mata o transporte longo.
///
/// ⚠️ Nenhuma metade sozinha diz a verdade: *«o tecto cura a curva»* leria-se como uma cura por
/// aplicar, e *«o tecto mata o transporte»* leria-se como um instrumento inútil. É a tabela
/// inteira que é o achado, e é ela que impede que a próxima janela adopte um lado sem ver o
/// outro.
///
/// **Mutações que sangram:** apagar o grampo do `accumulate_dab_smear` · pôr o produto a passar
/// o tecto em vez de `SEM_TECTO`.
#[test]
fn o_tecto_cura_a_curva_e_mata_o_transporte_longo() {
    let tecto = ph2d_painter_brush::TECTO_MEDIDO_E_RECUSADO_EM_RAIOS;
    let sem = ph2d_painter_brush::SEM_TECTO;

    let (curva_com, curva_sem) = (anel_guardado(tecto), anel_guardado(sem));
    let (leva_com, leva_sem) = (ate_onde_leva(tecto), ate_onde_leva(sem));
    super::smear_warp::espia::poe_tecto(sem);

    // (a) o que o tecto COMPRA: a figura fechada deixa de perder tinta.
    assert!(
        curva_com > 99.0 && curva_sem < 90.0,
        "a metade da CURVA não reproduz: com tecto {curva_com:.1} %, sem tecto {curva_sem:.1} %"
    );
    // (b) o que o tecto CUSTA: a faca deixa de levar a tinta além.
    assert!(
        leva_sem > leva_com + 100,
        "a metade do TRANSPORTE não reproduz: com tecto a tinta chega a x={leva_com}, \
         sem tecto a x={leva_sem}"
    );
}

/// ⭐⭐ **E o PRODUTO shipa sem tecto** — a metade que impede que a recusa seja aplicada às
/// escondidas, e que um `SEM_TECTO` trocado por engano passe calado.
///
/// ⚠️ A régua é o BARRO e não a constante: correr o anel pela porta do produto tem de reproduzir
/// a perda que o dono relatou.
#[test]
fn o_produto_shipa_sem_tecto() {
    // ⚠️⚠️ **Este gate NÃO PÕE o tecto — ele MEDE o que o produto usaria.** A 1.ª redacção
    // chamava `poe_tecto(SEM_TECTO)` aqui, e a mutação que trocava o valor de FÁBRICA do espião
    // **SOBREVIVEU**: o gate escrevia o valor que devia estar a medir. *Uma régua que arma o
    // sujeito dela não o mede.*
    assert_eq!(
        super::smear_warp::espia::tecto(),
        ph2d_painter_brush::SEM_TECTO,
        "o valor de FÁBRICA do transporte deixou de ser `SEM_TECTO`"
    );
    let mut com = cena(1.0, ph2d_painter_brush::StrokeMethod::Ellipse);
    anel(&mut com);
    let mut sem = cena(0.0, ph2d_painter_brush::StrokeMethod::Ellipse);
    anel(&mut sem);
    let guardado = 100.0 * tinta(&com) / tinta(&sem);
    assert!(
        (80.0..90.0).contains(&guardado),
        "o produto deixou de shipar sem tecto (ou a lei mudou): o anel guarda {guardado:.1} %, \
         e a medição de 2026-09-21 é 84,4 %"
    );

    // ⚠️⚠️ **A metade TEXTUAL, e sem ela a de cima mede o ESPIÃO e não o produto.** Fora do teste
    // o valor não vem do `espia` — vem da linha `cfg(not(test))`, que nenhuma corrida de teste
    // percorre. *Um gate que não consegue ver o ramo do produto afirma sobre código que não
    // corre.*
    let fonte = include_str!("smear_warp.rs");
    assert!(
        fonte.contains(
            "#[cfg(not(test))]\n        let tecto_em_raios = ph2d_painter_brush::SEM_TECTO;"
        ),
        "o ramo do PRODUTO deixou de passar `SEM_TECTO` ao esfregão"
    );
}

/// ⛔⛔⛔ **O PASSO DE VOLTA PELO ARCO NÃO CURA — a recusa, com o controlo que a torna honesta.**
///
/// A hipótese era que a corrente sai do traço porque cada elo é uma CORDA; a cura seria o passo
/// de volta rodar em torno do centro de curvatura ([`super::arco_do_caminho`]). Ela funciona como
/// geometria e **não move o barro**.
///
/// ⚠️ **O CONTROLO é obrigatório e foi ele que apanhou a 1.ª redacção:** medida à mão livre, a
/// cura lia `84,4 %` com e sem — e a causa era o arco **nunca armar** (um dab por lote, logo nunca
/// há passo anterior). *Uma comparação entre dois lados em que um deles nunca corre não é uma
/// medição.* Aqui o gate EXIGE que o arco arme na maioria dos dabs antes de comparar.
#[test]
fn o_passo_pelo_arco_nao_cura_a_perda() {
    // ⚠️⚠️ **O lado do PRODUTO não põe nada — ele MEDE o valor de fábrica.** A 1.ª redacção
    // chamava `poe_arco(false)` aqui, e a mutação que trocava o valor de fábrica do espião
    // **SOBREVIVEU** (a mesma armadilha que o gate do tecto pagou duas horas antes): *uma régua
    // que arma o sujeito dela não o mede.*
    let medir = |arco: Option<bool>| -> (f64, f32) {
        if let Some(v) = arco {
            super::smear_warp::espia::poe_arco(v);
        }
        super::smear_warp::espia::zera_arcos();
        let mut com = cena(1.0, ph2d_painter_brush::StrokeMethod::Ellipse);
        anel(&mut com);
        let (a, b) = super::smear_warp::espia::arcos();
        let armou = 100.0 * a as f32 / (a + b).max(1) as f32;
        let mut sem = cena(0.0, ph2d_painter_brush::StrokeMethod::Ellipse);
        anel(&mut sem);
        (100.0 * tinta(&com) / tinta(&sem), armou)
    };

    let (recto, armou_recto) = medir(None);
    let (curvo, armou_curvo) = medir(Some(true));
    super::smear_warp::espia::poe_arco(false);

    // CONTROLO: o arco tem de ARMAR de um lado e não do outro, senão as duas colunas são a mesma
    // corrida e o gate afirma o nada.
    assert!(
        armou_curvo > 90.0,
        "o arco mal armou ({armou_curvo:.1} %) — o gate estaria a comparar duas corridas iguais"
    );
    assert!(
        armou_recto < 1.0,
        "o produto armou o arco ({armou_recto:.1} %) — ele está RECUSADO"
    );
    // A RECUSA: com o arco a lei fica onde estava.
    assert!(
        (curvo - recto).abs() < 1.0,
        "o passo pelo arco mudou a perda — reavalie a recusa: recto {recto:.1} %, arco {curvo:.1} %"
    );
    assert!(
        recto < 90.0 && curvo < 90.0,
        "a fixtura não contém o fenómeno: recto {recto:.1} %, arco {curvo:.1} %"
    );

    // ⚠️ A metade TEXTUAL: fora do teste o valor não vem do espião, vem da linha
    // `cfg(not(test))`, que corrida de teste nenhuma percorre.
    let fonte = include_str!("smear_warp.rs");
    assert!(
        fonte.contains("#[cfg(not(test))]\n        let espia_do_arco = || false;"),
        "o ramo do PRODUTO deixou de recusar o arco"
    );
}
