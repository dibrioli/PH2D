//! ⭐⭐⭐ **O CENSO DOS CONTROLOS SÓ ENCOLHE** (doc 45 §3, doc 46 §2-4). Em cada meio, na fábrica,
//! cada controlo que a tela oferece é conduzido pela porta do ponteiro e risca-se o traço de fábrica
//! ([`super::censo_dos_controlos`]). O que mudou o ajuste e NÃO mudou a tinta tem de estar numa de
//! duas formas: **esmaecido** (a tela diz que depende de outro — o gate
//! `a_linha_esmaecida_e_a_que_nao_age` prova que só esmaece quem não age) ou **nesta lista, com o
//! motivo**. Controlo novo inerte sem motivo = vermelho; entrada que passou a agir ou saiu da tela =
//! vermelho (apague a linha: a lista só encolhe).

use super::censo_dos_controlos::{
    Bancada, Gesto, diferenca, em_paralelo, ensaio, gestos, nome, nomes,
};
use ph2d_a11y::NodeId;
use ph2d_tool_painter::PaintMedia;

/// `(meio, o controlo OU uma opção de menu, motivo)` — o id do menu cobre todas as opções dele.
type Inerte = (PaintMedia, NodeId, &'static str);

/// Os controlos que a tela oferece e que, na fábrica do meio, não mudam a tinta — cada um com o porquê.
/// Um «a fazer» aponta o item do plano que o faz agir (doc 46 §2); ao agir, a linha SAI daqui.
fn inertes_com_motivo() -> Vec<Inerte> {
    use PaintMedia::*;
    use ph2d_tool_painter::ids::*;
    let mut v: Vec<Inerte> = Vec::new();
    for m in [Digital, Watercolor, Impasto, WetPaint] {
        v.push((
            m,
            PAINTER_BRUSH_REPEAT_IMAGE,
            "não-pixel: a pré-visualização 3×3 da imagem que se repete",
        ));
        v.push((
            m,
            PAINTER_BRUSH_SYNC,
            "não-pixel: liga os ajustes desta ferramenta às outras",
        ));
    }
    let imagem =
        "a opção Image só age com uma imagem carregada (no app ela abre o seletor de ficheiro)";
    for m in [Digital, Impasto, WetPaint] {
        v.push((m, painter_brush_texture_kind_option_id(5), imagem));
    }
    for m in [Digital, WetPaint] {
        v.push((m, painter_shape_kind_option_id(5), imagem));
    }
    for luz in [
        PAINTER_IMPASTO_LIGHT_2,
        PAINTER_IMPASTO_LIGHT_3,
        PAINTER_IMPASTO_LIGHT_4,
    ] {
        v.push((Impasto, luz, "não-pixel: escolhe QUAL luz o painel edita"));
    }
    v.push((
        Impasto,
        PAINTER_IMPASTO_LIVE_EDIT,
        "por desenho: o Adjust Last Stroke age sobre a tinta JÁ pintada, e o censo muda-o antes dos traços",
    ));
    v.push((
        Watercolor,
        PAINTER_WATERCOLOR_WET_PREVIEW,
        "não-pixel: o véu de humidade sobre a tela",
    ));
    v.push((
        Watercolor,
        PAINTER_WATERCOLOR_DRY_TIME,
        "age entre traços separados por SEGUNDOS (cada poça seca no seu tempo, doc 46 §2-5) — os dois \
         traços do censo cabem em 1,3 s",
    ));
    let rodar = "o carimbo redondo sem Shape nem Grain que siga o carimbo não tem ângulo para mostrar. NÃO \
                 esmaece: o gate do esmaecido mediu a tinta a mudar por ACIDENTE em vários estados (o sorteio \
                 do ângulo desloca o fluxo das outras variações; a pegada rodada arredonda diferente)";
    for m in [Watercolor, WetPaint] {
        v.push((m, PAINTER_BRUSH_JITTER_ROTATE, rodar));
        v.push((m, PAINTER_BRUSH_JITTER_ROTATE_CHIP, rodar));
    }
    v.push((
        Impasto,
        painter_line_type_option_id(2),
        "age, mas não na FÁBRICA: o Reach de fábrica (1 diâmetro) costura dentro do próprio rastro, e o \
         rastro do Impasto é tinta OPACA da mesma cor (1 352 de 1 919 texels puros) — um fio por cima não \
         muda um byte; com Reach 3 ele sai do rastro e pinta (208 texels). O «só com Solid» de 2026-10-03 \
         era a corda do Solid a estragar o traço (curado: `a_corda_nao_deixa_rasto_nos_acumuladores_do_traco`)",
    ));
    v.push((
        Watercolor,
        PAINTER_SHAPE_WATERCOLOR_AUTO,
        "a porta das opções da Shape: desligá-lo põe o Falloff Watercolor, o MESMO carimbo de propósito \
         (`toggle_brush_watercolor_shape_auto`) — esmaecê-lo esconderia o caminho para escolher uma Shape",
    ));
    v.push((
        Watercolor,
        PAINTER_WATERCOLOR_SPREAD,
        "age, mas não PARA CIMA sem água: o aro usa `core_r = min(Spread, raio/2)` — na fábrica (raio 10, \
         Spread 7) baixá-lo a 1 muda 1 841 texels, subi-lo a 48 muda 0; acima de raio/2 só a água (Wet, \
         Dilution) o lê. Não é um dependente (não esmaece): o censo leva-o para cima (doc 46 §2-4)",
    ));
    v.push((
        Watercolor,
        PAINTER_LINE_SOLID,
        "a fazer: o Solid na aquarela (doc 46 §2-7)",
    ));
    v.push((
        Watercolor,
        painter_line_type_option_id(2),
        "age, mas não na FÁBRICA: o Reach de fábrica (1 diâmetro) costura dentro da própria aguada, e dentro \
         da tinta molhada o fio deposita só PIGMENTO da mesma cor (`watercolor_fios`: estender a cobertura ali \
         fazia o aro riscá-la de preto) — com Reach 3 a teia sai para o papel seco e pinta \
         (`o_sketchy_tinge_a_aguada_alem_do_rastro`)",
    ));
    for modo in [6u8, 7] {
        v.push((
            WetPaint,
            painter_brush_blend_option_id(modo),
            "Erase Alpha / Add Alpha mexem no ALFA da camada por baixo, e a água não a apaga: a borracha \
             dela é a ferramenta Erase, que tira tinta do fluido (`wetpaint::modo_da_sessao` → Mix)",
        ));
    }
    v.push((
        WetPaint,
        PAINTER_BRUSH_COMPOSITE_ENABLE,
        "a fazer: o Composite Brush no Wet Paint (doc 46 §2-8)",
    ));
    v.push((
        WetPaint,
        PAINTER_LINE_SOLID,
        "a fazer: o Solid no Wet Paint (doc 46 §2-9)",
    ));
    v.push((
        WetPaint,
        painter_line_type_option_id(2),
        "a fazer: os fios (Sketchy) no Wet Paint (doc 46 §2-9)",
    ));
    v.push((
        WetPaint,
        painter_line_type_option_id(3),
        "a fazer: os fios (Wire) no Wet Paint (doc 46 §2-9)",
    ));
    v.push((
        WetPaint,
        painter_brush_dab_handle_id(1),
        "o ângulo do carimbo (a pega do gizmo): o mesmo caso do Jitter Rotate — sem Shape nem Grain que \
         siga o carimbo, o carimbo redondo da água não tem ângulo para mostrar",
    ));
    v.push((
        WetPaint,
        PAINTER_WETPAINT_TUNING,
        "não-pixel: mostra o painel lateral dos números da água",
    ));
    v
}

/// O controlo de um gesto e, quando é a opção de um menu, o id da opção.
fn alvo(g: &Gesto) -> Option<(NodeId, Option<NodeId>)> {
    match g {
        Gesto::Opcao(menu, opcao, ..) => Some((*menu, Some(*opcao))),
        _ => g.id().map(|id| (id, None)),
    }
}

#[test]
fn o_censo_dos_controlos_so_encolhe() {
    let nomes = nomes();
    let lista = inertes_com_motivo();
    let meios = [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ];
    let falhas: Vec<Vec<String>> = em_paralelo(&meios, |&media| {
        let bancada = Bancada::nova(media, &[]);
        let esmaecida = |id: NodeId| bancada.host.store().tooltip_for(id).is_some();
        let (base, _) = ensaio(media, &[], &Gesto::Nenhum);
        let oferta = gestos(media, &[]);
        let medidas = em_paralelo(&oferta, |g| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ensaio(media, &[], g))).ok()
        });
        let mut out = Vec::new();
        let mut inertes: Vec<(NodeId, Option<NodeId>)> = Vec::new();
        let mut vivos: Vec<(NodeId, Option<NodeId>)> = Vec::new();
        for (g, medida) in oferta.iter().zip(medidas) {
            let Some(a) = alvo(g) else { continue };
            let rotulo = match (g, a.1) {
                (Gesto::Opcao(_, o, k, _), Some(_)) => {
                    format!("{}[{k}: {}]", nome(&nomes, a.0), nome(&nomes, *o))
                }
                _ => nome(&nomes, a.0),
            };
            let Some((px, entrega)) = medida else {
                out.push(format!("{media:?} {rotulo}: o gesto RENTOU (pânico)"));
                continue;
            };
            if !entrega.ajuste_mudou || entrega.ao_tool == 0 {
                continue; // não-pixel por desenho, ou a opção já escolhida
            }
            if diferenca(&base, &px).0 > 0 {
                vivos.push(a);
                continue;
            }
            inertes.push(a);
            let listado = lista
                .iter()
                .any(|(m, id, _)| *m == media && (*id == a.0 || Some(*id) == a.1));
            if !esmaecida(a.0) && !listado {
                out.push(format!(
                    "{media:?} {rotulo}: muda o ajuste e NÃO muda a tinta, sem esmaecer e sem motivo na lista"
                ));
            }
        }
        for (m, id, motivo) in lista.iter().filter(|(m, ..)| *m == media) {
            let casa = |a: &(NodeId, Option<NodeId>)| a.0 == *id || a.1 == Some(*id);
            let rotulo = nome(&nomes, *id);
            if vivos.iter().any(casa) {
                out.push(format!(
                    "{m:?} {rotulo}: AGORA MUDA A TINTA — apague a linha da lista («{motivo}»)"
                ));
            } else if !inertes.iter().any(casa) {
                out.push(format!(
                    "{m:?} {rotulo}: saiu da tela (ou deixou de mudar o ajuste) — apague a linha da lista («{motivo}»)"
                ));
            }
        }
        out
    });
    let falhas: Vec<String> = falhas.into_iter().flatten().collect();
    assert!(
        falhas.is_empty(),
        "{} linha(s) do censo fora da lei:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}
