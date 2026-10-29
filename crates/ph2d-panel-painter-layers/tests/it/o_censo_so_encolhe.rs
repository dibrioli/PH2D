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
    // O Wet Paint saiu a 13/10: ele nasce com as cerdas no Shape (plano 27 W2), e o Image sem imagem
    // troca-as pela ponta lisa.
    v.push((Digital, painter_shape_kind_option_id(5), imagem));
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
    // O Wet Paint saiu a 13/10: as cerdas do Shape giram com o Rotate (plano 27, 27 §6.6).
    v.push((Watercolor, PAINTER_BRUSH_JITTER_ROTATE, rodar));
    // As linhas da MARCA (L4) que viram ou achatam a PONTA: na ponta redonda de fábrica não têm lado para
    // mostrar. NÃO esmaecem pela razão do `rodar` (medido 20/10: o dab que leva a forma sai pela rota por dab e
    // arredonda diferente da cache — a tinta muda por ACIDENTE em vários estados). Na Watercolor com a Shape
    // «Automatic» o carimbo é o redondo dela (doc 47 §6); no Wet as cerdas só leem o ROTOR (o espelho Y não
    // lhes chega, o X sim pela meia-volta).
    let ponta = "a ponta redonda de fábrica não tem lado para virar nem eixo para achatar; NÃO esmaece pela \
                 razão do Jitter Rotate (a rota por dab arredonda diferente da cache: a tinta muda por ACIDENTE)";
    // Desde 23/10 (dono: «todos sliders») cada uma é uma barra com o chip ligado — os DOIS gestos entram, como o
    // par do Jitter Rotate acima.
    let (vira_x, vira_y, redonda, angulo) = (3, 4, 2, 5);
    for (m, id) in [
        (Digital, vira_x),
        (Digital, vira_y),
        (Digital, angulo),
        (Watercolor, vira_x),
        (Watercolor, vira_y),
        (Watercolor, redonda),
        (Watercolor, angulo),
        (Impasto, vira_x),
        (Impasto, vira_y),
        (WetPaint, vira_y),
    ] {
        v.push((m, PAINTER_MARCA_FIELDS[id], ponta));
        v.push((m, PAINTER_MARCA_CHIPS[id], ponta));
    }
    // ⛔ A velocidade→opacidade (L12) no Wet SAIU daqui (J3 canetas 2.ª onda): a L12 lê a velocidade da MÃO, não a do
    // traço atrasado pelo estabilizador, e o traço do censo já passa dos 300 px/s — a linha age.
    v.push((Watercolor, PAINTER_BRUSH_JITTER_ROTATE_CHIP, rodar));
    // ⛔ O tipo de linha do Impasto (`painter_line_type_option_id(2)`) SAIU a 28/10: o 3.º traço do censo (as duas
    // pontas dentro da tela) dá ao fio do Reach de fábrica onde sair do rastro — a linha age.
    // Saíram a 23/10 (a fábrica da Aquarela passou a ser a do painel do dono: Rewet 0,5, Bleed 4, Charge
    // 0,5…): o Automatic, o Spread e a teia (Sketchy) passaram a mudar a tinta da fábrica.
    for modo in [6u8, 7] {
        v.push((
            WetPaint,
            painter_brush_blend_option_id(modo),
            "Erase Alpha / Add Alpha mexem no ALFA da camada por baixo, e a água não a apaga: a borracha \
             dela é a ferramenta Erase, que tira tinta do fluido (`wetpaint::modo_da_sessao` → Mix)",
        ));
    }
    // O cartão Pen (L2). A rotação do carimbo: o mesmo porquê do Jitter Rotate. (Na Watercolor de fábrica — Shape
    // «Automatic» — ela nem está no cartão: o carimbo redondo dela não a lê, `LinhaDaMao::lida`.)
    for m in [Digital, Impasto] {
        v.push((m, PAINTER_PEN_ROTATION, rodar));
    }
    for m in [Digital, Watercolor, Impasto, WetPaint] {
        v.push((
            m,
            PAINTER_PEN_FIELDS[13],
            "a caneta deitada alisa o GRÃO (`grain_depth × (1 − smooth)`, `spec_frame::at_dab`) e a fábrica não \
             tem Grain. NÃO esmaece: ligá-la põe o dab na rota deitada (por dab), e com uma Shape a tinta muda \
             por arredondamento — medido pelo gate do esmaecido (20/10, L2)",
        ));
    }
    v.push((
        WetPaint,
        PAINTER_WETPAINT_TUNING,
        "não-pixel: mostra o painel lateral dos números da água",
    ));
    v
}

/// Os controlos DESENHADOS cujo gesto não chega a ninguém (nem à ferramenta, nem à shell, nem muda a tela)
/// — cada um com o porquê. Fora desta lista, um clique assim é o clique MORTO (o «Same as Paper», 28/10).
fn nao_chegam_com_motivo() -> Vec<Inerte> {
    // Vazia: hoje todo gesto desenhado chega a alguém (ferramenta, shell) ou muda a própria tela (revela, esconde,
    // dobra a gaveta, abre o seletor de cor; a pega da secção arrasta).
    Vec::new()
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
    let lista_muda = nao_chegam_com_motivo();
    let meios = [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ];
    let falhas: Vec<Vec<String>> = em_paralelo(&meios, |&media| {
        let bancada = Bancada::nova(media, &[]);
        let esmaecida = |id: NodeId| {
            bancada
                .host
                .store()
                .tooltip_for(id)
                .is_some_and(ph2d_panel_painter_layers::e_dica_de_inercia)
        };
        let (base, _) = ensaio(media, &[], &Gesto::Nenhum);
        let oferta = gestos(media, &[]);
        let medidas = em_paralelo(&oferta, |g| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ensaio(media, &[], g))).ok()
        });
        let mut out = Vec::new();
        let mut inertes: Vec<(NodeId, Option<NodeId>)> = Vec::new();
        let mut vivos: Vec<(NodeId, Option<NodeId>)> = Vec::new();
        let mut mudos: Vec<(NodeId, Option<NodeId>)> = Vec::new();
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
            // ⛔ Um gesto numa caixa DESENHADA que não chega a ninguém (nem à ferramenta nem à shell) é o
            // clique morto — foi assim que o «Same as Paper» ficou verde (28/10). Só passa com o porquê.
            if entrega.ao_tool == 0
                && entrega.revelados.is_empty()
                && entrega.escondidos.is_empty()
                && !entrega.dobrou
                && entrega.outras.is_empty()
            {
                // A PEGA de uma secção: o gesto dela é o ARRASTO (`as_seccoes_arrastam_e_tem_tema`), o clique parado
                // não faz nada por desenho.
                let pega = ph2d_editor_core::ids::section_of_grip(a.0).is_some()
                    || bancada
                        .ids()
                        .into_iter()
                        .any(|s| ph2d_editor_core::ids::grip_de(s) == a.0);
                if pega {
                    continue;
                }
                if lista_muda
                    .iter()
                    .any(|(m, id, _)| *m == media && *id == a.0)
                {
                    mudos.push(a);
                } else {
                    out.push(format!(
                        "{media:?} {rotulo}: o gesto na caixa desenhada NÃO chega à ferramenta, sem motivo na lista"
                    ));
                }
                continue;
            }
            if !entrega.ajuste_mudou || entrega.ao_tool == 0 {
                continue; // a opção já escolhida, ou um gesto da própria tela (abre secção, chama a shell)
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
        for (m, id, motivo) in lista_muda.iter().filter(|(m, ..)| *m == media) {
            if !mudos.iter().any(|a| a.0 == *id) {
                out.push(format!(
                    "{m:?} {}: AGORA CHEGA À FERRAMENTA (ou saiu da tela) — apague a linha da lista dos mudos («{motivo}»)",
                    nome(&nomes, *id)
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
