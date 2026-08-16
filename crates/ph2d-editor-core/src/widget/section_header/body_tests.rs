//! Gates da dobra do CORPO. O molde é o do irmão `fold.rs`: cada gate afirma **uma** propriedade
//! e traz ao lado a mutação que o faria sangrar.

use super::*;
use crate::interaction::{HitIndex, WidgetStore};

const X: f32 = 10.0;
const W: f32 = 280.0;
/// ⚠️ **Estes dois números são ESCOLHIDOS, e o primeiro par que escrevi não continha o
/// fenómeno.** `top + (bot − top)` só difere de `bot` em **0,644%** dos pares plausíveis (59,2 M
/// varridos); com `100,3 / 400,7` — a fixture original — as duas expressões dão o mesmo `f32`, e
/// a mutação que apaga o ramo do verbatim passava **verde**. Este par está nos 0,644%.
const TOP: f32 = 40.199_997;
const BOTTOM: f32 = 296.309_1;

fn store_with(id: NodeId, t: f32, remembered: Option<f32>) -> WidgetStore {
    let mut s = WidgetStore::with_capacity(8);
    s.set_section_open_live(id, t);
    if let Some(h) = remembered {
        s.remember_section_body_h(id, h);
    }
    s
}

/// **O REPOUSO ABERTO é byte a byte o mundo pré-wave** — nem camada empurrada nem aritmética
/// sobre o `y`. *Mutação: `finish` a devolver sempre `body_top + measured * t` ⇒ `400,70004`
/// contra `400,7`, um deslocamento de arredondamento em REPOUSO, para sempre.*
#[test]
fn an_open_section_at_rest_returns_the_cursor_verbatim() {
    let id = NodeId(1);
    let store = store_with(id, 1.0, Some(300.0));
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    let fold = SectionFold::begin(&store, id, X, W, TOP, &mut scene, &mut hit)
        .expect("uma secção aberta tem corpo");
    let out = fold.finish(&store, &mut scene, &mut hit, BOTTOM);
    assert_eq!(
        out.to_bits(),
        BOTTOM.to_bits(),
        "o `y` de saída tem de ser o MESMO f32, não um que arredonda para perto"
    );
}

/// **O REPOUSO FECHADO não pinta nem mede** — é o `if collapsed {{ return y + header_h; }}` de
/// sempre. *Mutação: `SHUT` a zero ⇒ `begin` devolve `Some` com `t = 0` e o corpo passa a ser
/// percorrido em toda secção fechada do app.*
#[test]
fn a_shut_section_has_no_body_at_all() {
    let id = NodeId(2);
    let store = store_with(id, 0.0, Some(300.0));
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    assert!(
        SectionFold::begin(&store, id, X, W, TOP, &mut scene, &mut hit).is_none(),
        "fechada e parada não abre escopo nenhum"
    );
}

/// **A meio, o `y` de saída ESCALA com o `t`** — é isto que faz tudo o que está por baixo subir
/// junto. *Mutação: devolver `cur_y` sempre ⇒ o corpo desliza e a secção de baixo fica parada.*
#[test]
fn a_folding_section_scales_the_cursor_it_hands_back() {
    let id = NodeId(3);
    let store = store_with(id, 0.5, Some(300.0));
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    let fold = SectionFold::begin(&store, id, X, W, TOP, &mut scene, &mut hit).expect("meio");
    let out = fold.finish(&store, &mut scene, &mut hit, BOTTOM);
    let expected = TOP + (BOTTOM - TOP) * 0.5;
    assert!(
        (out - expected).abs() < 1e-3,
        "a meia dobra o corpo ocupa metade: {out} != {expected}"
    );
}

/// **A altura é MEDIDA no quadro e LEMBRADA para o recorte do seguinte.** *Mutação: o
/// `remember_section_body_h` fora do `finish` ⇒ a memória nunca chega e o recorte fica em zero
/// para sempre — a secção abre de repente no fim, com o chevron a rodar sobre nada.*
#[test]
fn the_body_height_measured_this_frame_is_the_one_remembered() {
    let id = NodeId(4);
    let store = store_with(id, 1.0, None);
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    let fold = SectionFold::begin(&store, id, X, W, TOP, &mut scene, &mut hit).expect("aberta");
    fold.finish(&store, &mut scene, &mut hit, BOTTOM);
    let h = store.section_body_h(id).expect("mediu");
    assert!(
        (h - (BOTTOM - TOP)).abs() < 1e-3,
        "lembrou {h}, mediu {}",
        BOTTOM - TOP
    );
}

/// **Memória AUSENTE recorta a zero e mede na mesma** — a estreia de uma secção que nunca foi
/// pintada aberta custa um quadro invisível, não um salto. *Mutação: `unwrap_or(f32::MAX)` ⇒ a
/// estreia mostra o corpo INTEIRO num quadro e só depois encolhe, que é o pop que a dobra existe
/// para remover.*
#[test]
fn a_section_with_no_memory_clips_to_nothing_and_still_measures() {
    let id = NodeId(5);
    let store = store_with(id, 0.05, None);
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    let fold = SectionFold::begin(&store, id, X, W, TOP, &mut scene, &mut hit).expect("a abrir");
    // Uma row registada no meio do corpo: com recorte de altura ZERO ela não pode ser clicável.
    hit.register(NodeId(999), Rect::new(X, TOP + 50.0, W, 20.0));
    assert!(
        hit.hit(X + 5.0, TOP + 55.0).is_none(),
        "sem memória o recorte é zero, logo nada do corpo é clicável"
    );
    fold.finish(&store, &mut scene, &mut hit, BOTTOM);
    assert!(
        store.section_body_h(id).is_some(),
        "o quadro invisível TEM de deixar a medição para o seguinte"
    );
}

/// **O que a cena esconde, o rato também não alcança.** *Mutação: tirar o `hit_index.push_clip`
/// do `begin` ⇒ uma row invisível volta a responder nos vãos entre os widgets da secção de
/// baixo, que já subiu por cima dela.*
#[test]
fn a_row_below_the_fold_band_is_not_clickable() {
    let id = NodeId(6);
    let store = store_with(id, 0.5, Some(300.0));
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    let fold = SectionFold::begin(&store, id, X, W, TOP, &mut scene, &mut hit).expect("meio");
    // banda visível = [TOP, TOP + 150]
    hit.register(NodeId(11), Rect::new(X, TOP + 20.0, W, 20.0)); // dentro
    hit.register(NodeId(12), Rect::new(X, TOP + 200.0, W, 20.0)); // fora
    assert_eq!(
        hit.hit(X + 5.0, TOP + 25.0),
        Some(NodeId(11)),
        "a parte visível continua clicável"
    );
    assert!(
        hit.hit(X + 5.0, TOP + 205.0).is_none(),
        "a parte escondida não"
    );
    fold.finish(&store, &mut scene, &mut hit, BOTTOM);
    // e o recorte FECHA: o que vem depois da secção volta a ser registado inteiro
    hit.register(NodeId(13), Rect::new(X, TOP + 200.0, W, 20.0));
    assert_eq!(hit.hit(X + 5.0, TOP + 205.0), Some(NodeId(13)));
}

/// **Uma row a meio da banda é APARADA, não descartada** — a metade de cima continua a responder
/// enquanto a de baixo já não. *Mutação: `clipped` a devolver o rect original ⇒ a row inteira
/// responde, incluindo a metade que a cena já não mostra.*
#[test]
fn a_row_straddling_the_band_keeps_only_the_half_that_shows() {
    let id = NodeId(7);
    let store = store_with(id, 0.5, Some(100.0)); // banda = [TOP, TOP + 50]
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    let fold = SectionFold::begin(&store, id, X, W, TOP, &mut scene, &mut hit).expect("meio");
    hit.register(NodeId(21), Rect::new(X, TOP + 40.0, W, 40.0)); // atravessa a borda
    assert_eq!(
        hit.hit(X + 5.0, TOP + 45.0),
        Some(NodeId(21)),
        "acima da borda"
    );
    assert!(hit.hit(X + 5.0, TOP + 60.0).is_none(), "abaixo da borda");
    fold.finish(&store, &mut scene, &mut hit, BOTTOM);
}

/// **`has_body` e o `begin` concordam** — os pintores cujo corpo não é uma sequência de rows
/// perguntam ao primeiro, e não pode haver dois vereditos sobre o mesmo instante.
#[test]
fn has_body_agrees_with_whether_begin_opens_a_scope() {
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    for (i, t) in [0.0, 0.0005, 0.05, 0.5, 1.0].into_iter().enumerate() {
        let id = NodeId(100 + i as u64);
        let store = store_with(id, t, Some(50.0));
        let opened = SectionFold::begin(&store, id, X, W, TOP, &mut scene, &mut hit);
        assert_eq!(
            opened.is_some(),
            has_body(&store, id),
            "t = {t}: as duas portas têm de dar a mesma resposta"
        );
        if let Some(f) = opened {
            f.finish(&store, &mut scene, &mut hit, BOTTOM);
        }
    }
}

/// **O vão que segue uma secção dobra com ela.** *Mutação: devolver `gap` cru ⇒ uma secção
/// fechada continua a reservar o seu separador e a lista fica com um buraco.*
///
/// ⚠️ **A linha do `t = 1` nasceu UNILATERAL (`x - 8.0 < 1e-6`, sem o `.abs()`), e isso a tornava
/// satisfazível por QUALQUER valor abaixo de oito.** Com as outras duas a pinar só `t = 0` e
/// `t = 0,5`, um `clamp(0.0, 0.5)` no produto passava nos três — e os **onze** gates deste módulo
/// ficavam verdes com o vão a dobrar metade errado. A assimetria é o cheiro: as duas linhas
/// seguintes já traziam o `.abs()`, e a primeira era a que faltava.
#[test]
fn the_gap_after_a_section_folds_with_it() {
    let id = NodeId(8);
    assert!((folded_gap(&store_with(id, 1.0, None), id, 8.0) - 8.0).abs() < 1e-6);
    assert!(folded_gap(&store_with(id, 0.0, None), id, 8.0).abs() < 1e-6);
    assert!((folded_gap(&store_with(id, 0.5, None), id, 8.0) - 4.0).abs() < 1e-6);
}

/// **As duas portas são UMA lei.** O `begin` é o `begin_at` com o `t` que ele mesmo busca no
/// store — e é isso que impede o painel que fotografa a dobra de divergir do que não fotografa.
/// *Mutação: `begin` a clampar o `t` antes de delegar (ou a lê-lo de outro campo) ⇒ os dois
/// vereditos separam-se e o `assert_eq` sangra.*
#[test]
fn the_two_doors_are_one_law() {
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    for (i, t) in [0.0, 0.0005, 0.05, 0.5, 1.0].into_iter().enumerate() {
        let id = NodeId(200 + i as u64);
        let store = store_with(id, t, Some(120.0));
        let a = SectionFold::begin(&store, id, X, W, TOP, &mut scene, &mut hit);
        let looked_up = store.section_open_live(id);
        let b = SectionFold::begin_at(&store, id, looked_up, X, W, TOP, &mut scene, &mut hit);
        assert_eq!(
            a.is_some(),
            b.is_some(),
            "t = {t}: a porta que BUSCA e a que RECEBE têm de dar o mesmo veredito"
        );
        if let Some(f) = a {
            f.finish(&store, &mut scene, &mut hit, BOTTOM);
        }
        if let Some(f) = b {
            f.finish(&store, &mut scene, &mut hit, BOTTOM);
        }
    }
}

/// **O `t` que vem de fora é o que MANDA** — o store pode dizer outra coisa e a dobra honra o
/// argumento. É a propriedade inteira do `begin_at`: sem ela o painel que fotografa continuaria
/// a ver o veredito do store, que é o defeito que o gate do audio-editor mediu (aberto e fechado
/// com a MESMA altura). *Mutação: `begin_at` a reler o store ⇒ RED.*
#[test]
fn the_caller_supplied_t_wins_over_the_store() {
    let id = NodeId(210);
    let store = store_with(id, 1.0, Some(120.0)); // o STORE diz ABERTA
    let mut scene = VectorScene::new();
    let mut hit = HitIndex::new();
    assert!(
        SectionFold::begin_at(&store, id, 0.0, X, W, TOP, &mut scene, &mut hit).is_none(),
        "o `t` do chamador diz FECHADA e parada — nenhum corpo é percorrido"
    );
}
