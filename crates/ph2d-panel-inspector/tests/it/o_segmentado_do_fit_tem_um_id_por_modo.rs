//! ⛔⛔⛔ **UM id por SEGMENTO — é a contagem do array que decide o que o artista VÊ.**
//!
//! Report do dono, 2026-09-20: *«só tem as opções de keep e Stretch. Não expand»*. E o pintor já
//! passava os **três** rótulos — o que faltava era o terceiro **id**: o segmentado pinta um
//! segmento por entrada do `INSP_HUD_FIT`, logo o rótulo a mais era **ignorado em silêncio**.
//!
//! ⚠️⚠️ **O gate que existia era CEGO a isto:** ele contava os RÓTULOS no fonte do pintor
//! (`3 == 3`) e nunca olhava para o array. *Uma régua que conta rótulos não vê quantos SEGMENTOS
//! são pintados.*
//!
//! ⭐ É a mesma forma que o `SignalVerb::ALL` pagou (foi a `9` com o array parado em `8`) e que o
//! chip do `Density` da escultura pagou antes dele — **a terceira vez nesta linha**.
//!
//! ⚠️ Ele mora AQUI e não na crate da família porque a régua é a `ph2d_hud::Fit::ALL`, e este
//! painel **pode** vê-la: a `ph2d-hud` é uma FOLHA (só `serde`), como a `ph2d-shake` que a secção
//! do abanão já lê pela mesma razão — *a parede do ADR-0029 é contra a `ph2d-ecs`*.

use ph2d_hud::Fit;
use ph2d_panel_inspector::ids;

/// **Mutação que deve sangrar:** tirar a última entrada do `INSP_HUD_FIT`.
#[test]
fn o_segmentado_do_fit_tem_um_id_por_modo() {
    assert_eq!(
        ids::INSP_HUD_FIT.len(),
        Fit::ALL.len(),
        "o segmentado do `Fit` tem {} ids e a lei tem {} modos — ele pinta UM segmento por id, \
         logo os modos a mais existem, tem lei, tem gates, e o artista NAO lhes chega",
        ids::INSP_HUD_FIT.len(),
        Fit::ALL.len()
    );
}

/// ⛔ **E os ids são DISTINTOS** — dois segmentos com o mesmo id partilhariam foco e clique.
///
/// ⚠️ Metade que a contagem não dá: um array de três com uma entrada repetida tem o tamanho certo
/// e **dois segmentos indistinguíveis** para o despachante.
#[test]
fn os_ids_do_segmentado_sao_distintos() {
    let mut v = ids::INSP_HUD_FIT.to_vec();
    v.sort_unstable();
    v.dedup();
    assert_eq!(
        v.len(),
        ids::INSP_HUD_FIT.len(),
        "dois segmentos do `Fit` partilham id — o clique de um acende o outro"
    );
}
