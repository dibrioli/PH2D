//! ⭐⭐ **A lei da compactação, por pintura real** (dono 04/11, `docs/CartoesUI/05` §4). Pinta cada
//! painel do registo pela porta do produto, de fábrica e armado, todas as gavetas abertas, e lê o
//! censo da fundação (`panel::cards::censo`):
//!
//! - `lei_da_compactacao`: num cartão, linha de um só controlo curto seguida de outra que caberia
//!   ao lado reprova; distância entre linhas abaixo de `FIELD_ROW_GAP` reprova.
//! - `nenhum_texto_sai_do_botao`: em todo botão, segmento e caixa de marcar, o texto medido mais o
//!   respiro de `metrics` cabe na largura.
//!
//! `PENDENTES` só desce: é o alvo de zero. O número que desce TEM de ser escrito aqui. Os estados
//! semeados estão em [`super::censo_estados`]; `POPULACAO` impede que um zero seja só cegueira. O censo
//! vê a lista sob `paint_label_row`, o `aviso` e o corpo em captura (R4). Tela própria continua fora.
//! Sonda: `cargo test … diag_a_tabela_da_compactacao -- --ignored --nocapture`.

use std::collections::BTreeMap;

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::censo::{self, Censo};
use ph2d_ui_testkit::MockPanelHost;

use super::a_marca_tem_a_altura_da_linha::{abre_as_gavetas, viewport};

/// `(painel, irmãos separados, linhas grudadas, textos cortados)`, medido em 04/11.
///
/// `painter_layers` saiu a 04/11 (H8): as 5 últimas linhas grudadas vinham de Mixing, Symmetry e Tiling, que corriam o
/// corpo antigo só para saber se estavam à vista.
/// `widget_gallery` 4 -> 3 a 05/11 (F11): a 4.ª era o Solo (Idle, Walk, Jump). Sobram as três caixas de marcar que a
/// 218 px cortavam o texto (F10); a lei 7 ganha da lei 1. `widget_gallery` 3 -> 0 a 13/11: a caixa de marcar em
/// moldura à Godot (GEF, `249611a86a`) mede-se pela moldura com o nome em cima, e o par já não se separa.
const PENDENTES: &[(&str, usize, usize, usize)] = &[];

/// `(painel, fronteiras fora da lei do ritmo)`, medido em 04/11. Só desce: é o alvo de zero.
const PENDENTES_RITMO: &[(&str, usize)] = &[
    // Instance e Properties (família K de `18_REVISAO_DE_CAPRICHO`), e o nome dentro do chip da tag.
    ("inspector", 0),
];

/// Linhas medidas por painel, o piso (medido em 04/11): abaixo disto o zero já não é aprovação.
const POPULACAO: &[(&str, usize)] = &[
    ("vector", 100),
    ("skeleton", 12),
    ("inspector", 400),
    // 04/11, H8: 718 medidas com 12 estados semeados (`censo_estados`). Sem eles eram 138 em duas pinturas: o censo
    // estava CEGO para Máscara, Deform, Selection e Layers, e o piso de 250 aceitava-o. 692 desde a fatia q3: os
    // rótulos de grupo sobre um controlo deixaram de ser medidos como linha (são rótulos).
    ("painter_layers", 690),
    ("physics", 30),
    ("audio_editor", 10),
    #[cfg(feature = "panel-flip-frames")]
    ("flip_frames", 25),
];

/// `(painel, textos, controlos, fronteiras)` que a lei do ritmo tem de ver, o piso. Medido em 05/11 por
/// `diag_a_tabela_da_compactacao`, a ~5 % abaixo: vector 465/339/327 · skeleton 33/23/19 · inspector 1596/842/1233 ·
/// painter_layers 2754/1172/1837 · physics 152/124/124 · audio_editor 175/67/148 · flip_frames 57/63/46 ·
/// widget_gallery 568/334/455. O censo das linhas não vê estas peças: sem o ouvinte do texto em `grava`, o ritmo
/// ficaria verde sobre zero textos e o piso de [`POPULACAO`] nem notava.
const POPULACAO_RITMO: &[(&str, usize, usize, usize)] = &[
    ("vector", 440, 320, 310),
    ("skeleton", 31, 21, 18),
    ("inspector", 1500, 800, 1170),
    ("painter_layers", 2600, 1100, 1740),
    // ⬇️ 08/11 (GEF02 ondas 8/9a): a matriz (16 rótulos + 36 células) foi para a aba «Collision Layers»; medido 122/54/68.
    ("physics", 116, 51, 65),
    ("audio_editor", 166, 63, 140),
    #[cfg(feature = "panel-flip-frames")]
    ("flip_frames", 54, 59, 43),
    ("widget_gallery", 540, 317, 432),
];

/// O que um painel acusa: as listas de frases das três regras.
#[derive(Default)]
struct Acusa {
    irmaos: Vec<String>,
    grudadas: Vec<String>,
    cortados: Vec<String>,
    ritmo: Vec<String>,
    listas_baixas: Vec<String>,
    listas_soltas: usize,
    /// O tamanho da população medida: zero lê-se como aprovação, e pode ser só cegueira.
    linhas: usize,
    textos: usize,
    capturadas: usize,
    /// As peças que a lei do ritmo vê: o que o ouvinte do texto e o registo dos controlos lhe deram.
    pecas_texto: usize,
    pecas_controlo: usize,
    fronteiras: usize,
}

fn junta(a: &mut Acusa, c: &Censo) {
    let controlos = c
        .pecas
        .iter()
        .filter(|p| p.especie == censo::ritmo::Especie::Controlo)
        .count();
    a.pecas_controlo += controlos;
    a.pecas_texto += c.pecas.len() - controlos;
    a.fronteiras += c.distancias().len();
    a.linhas += c.linhas.len();
    a.textos += c.textos.len();
    a.capturadas += c.linhas.iter().filter(|l| l.em_captura).count();
    a.listas_soltas += c.listas_soltas.len();
    for frase in c.listas_soltas_baixas() {
        if !a.listas_baixas.contains(&frase) {
            a.listas_baixas.push(frase);
        }
    }
    for (de, para) in [
        (c.irmaos_separados(), &mut a.irmaos),
        (c.linhas_grudadas(), &mut a.grudadas),
        (c.textos_cortados(), &mut a.cortados),
        (c.ritmo_fora_da_lei(), &mut a.ritmo),
    ] {
        for frase in de {
            if !para.contains(&frase) {
                para.push(frase);
            }
        }
    }
}

/// As gavetas, e os dois cartões que nascem dobrados por decisão do painel (`set_collapsed_if_unchosen`): sem
/// escolha gravada eles voltavam a fechar, e Ordering e Material & Blend ficavam fora de todo censo.
pub(super) fn abre_tudo(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    abre_as_gavetas(store);
    for id in [
        ph2d_panel_inspector::CARD_ORDERING,
        ph2d_panel_inspector::CARD_MATERIAL,
    ] {
        store.set_collapsed(id, false);
    }
}

/// Os cartões que nascem dobrados só existem depois do 1.º quadro: abre, pinta, abre outra vez e mede.
pub(super) fn mede(
    host: &mut MockPanelHost,
    painel: &mut ph2d_editor_core::panel::ErasedPanel,
) -> Censo {
    abre_tudo(host.store_mut());
    let _ = host.medindo_a_pintura_do_registo(painel, viewport());
    abre_tudo(host.store_mut());
    censo::grava(|| host.medindo_a_pintura_do_registo(painel, viewport())).1
}

fn censo_de_todos() -> BTreeMap<&'static str, Acusa> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    let mut out: BTreeMap<&'static str, Acusa> = BTreeMap::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            let acusa = out.entry(id).or_default();
            let mut host = MockPanelHost::new().in_theme(ph2d_tokens::Theme::Dark);
            painel.populate(host.store_mut());
            junta(acusa, &mede(&mut host, painel));
            if let Some(arm) = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id)
            {
                let mut host = MockPanelHost::new().in_theme(ph2d_tokens::Theme::Dark);
                (arm.arma)(host.store_mut());
                painel.populate(host.store_mut());
                junta(acusa, &mede(&mut host, painel));
                (arm.desarma)();
            }
            for e in super::censo_estados::ESTADOS
                .iter()
                .filter(|e| e.painel == id)
            {
                let mut host = MockPanelHost::new().in_theme(ph2d_tokens::Theme::Dark);
                (e.arma)();
                painel.populate(host.store_mut());
                junta(acusa, &mede(&mut host, painel));
                (e.desarma)();
            }
        }
    });
    out
}

fn pendente(painel: &str) -> (usize, usize, usize) {
    PENDENTES
        .iter()
        .find(|p| p.0 == painel)
        .map_or((0, 0, 0), |p| (p.1, p.2, p.3))
}

/// A catraca: nem acima (defeito novo) nem abaixo (o número desceu e não foi escrito).
fn confere(medir: impl Fn(&Acusa) -> &Vec<String>, de: impl Fn((usize, usize, usize)) -> usize) {
    let tudo = censo_de_todos();
    assert!(
        tudo.len() > 10,
        "a varredura não mediu nada: {}",
        tudo.len()
    );
    let mut erros = Vec::new();
    for (painel, a) in &tudo {
        let lista = medir(a);
        let (tem, pode) = (lista.len(), de(pendente(painel)));
        if tem > pode {
            erros.push(format!("{painel}: {tem} contra {pode}: {lista:#?}"));
        } else if tem < pode {
            erros.push(format!(
                "{painel}: desceu para {tem}, escreva-o em PENDENTES"
            ));
        }
    }
    assert!(erros.is_empty(), "{}", erros.join("\n"));
}

/// ⭐ Leis 1 e 3 de `05 §4`.
#[test]
fn lei_da_compactacao() {
    confere(|a| &a.irmaos, |p| p.0);
    confere(|a| &a.grudadas, |p| p.1);
}

/// ⭐ A lista suspensa sem rótulo (o filtro por tag, o chip do Script) é uma linha do censo, à altura de
/// todo campo. *Mutação: `caixa_do_chip` a `ROW_H_PX` ⇒ vermelho; sem a anotação ⇒ a população cai a zero.*
#[test]
fn toda_lista_solta_tem_a_altura_de_um_campo() {
    let tudo = censo_de_todos();
    let baixas: Vec<_> = tudo
        .iter()
        .filter(|(_, a)| !a.listas_baixas.is_empty())
        .map(|(p, a)| format!("{p}: {:#?}", a.listas_baixas))
        .collect();
    assert!(baixas.is_empty(), "{}", baixas.join("\n"));
    let do_inspector = tudo.get("inspector").map_or(0, |a| a.listas_soltas);
    assert!(
        do_inspector >= 2,
        "só {do_inspector} listas soltas no Inspector: o censo está cego"
    );
}

/// ⭐ Lei 7 de `05 §4`: nunca «…» num botão.
#[test]
fn nenhum_texto_sai_do_botao() {
    confere(|a| &a.cortados, |p| p.2);
}

/// ⭐ **O ritmo vertical** (`05 §4`, dono 04/11, «Veja como o espaçamento é ruim»): por pintura real,
/// rótulo ↔ controlo, controlo ↔ frase e a fila de controlos lado a lado.
#[test]
fn lei_do_ritmo() {
    let tudo = censo_de_todos();
    let mut erros = Vec::new();
    for (painel, a) in &tudo {
        let pode = PENDENTES_RITMO
            .iter()
            .find(|p| p.0 == *painel)
            .map_or(0, |p| p.1);
        let tem = a.ritmo.len();
        if tem > pode {
            erros.push(format!("{painel}: {tem} contra {pode}: {:#?}", a.ritmo));
        } else if tem < pode {
            erros.push(format!(
                "{painel}: desceu para {tem}, escreva-o em PENDENTES_RITMO"
            ));
        }
    }
    for (painel, textos, controlos, fronteiras) in POPULACAO_RITMO {
        let a = &tudo[painel];
        let medido = (a.pecas_texto, a.pecas_controlo, a.fronteiras);
        if medido.0 < *textos || medido.1 < *controlos || medido.2 < *fronteiras {
            erros.push(format!(
                "{painel}: o ritmo mediu {medido:?} (textos, controlos, fronteiras) contra o piso \
                 {:?}: o zero não vale, a pintura deixou de chegar ao censo",
                (textos, controlos, fronteiras)
            ));
        }
    }
    assert!(erros.is_empty(), "{}", erros.join("\n"));
}

/// O zero de um painel só vale com população: os estados semeados continuam a pintar.
/// *Mutação: tirar um estado de `ESTADOS`, ou calar a captura ⇒ vermelho.*
#[test]
fn o_zero_tem_populacao() {
    let tudo = censo_de_todos();
    for (painel, piso) in POPULACAO {
        let a = &tudo[painel];
        assert!(
            a.linhas >= *piso,
            "{painel}: {} linhas contra {piso}",
            a.linhas
        );
    }
}

/// A tabela para o tracker: `(painel, irmãos, grudadas, cortados)` e as frases.
#[test]
#[ignore = "diagnóstico: imprime a tabela, não afirma nada"]
fn diag_a_tabela_da_compactacao() {
    for (painel, a) in censo_de_todos() {
        let (i, g, t) = (a.irmaos.len(), a.grudadas.len(), a.cortados.len());
        println!(
            "(\"{painel}\", {i}, {g}, {t}), // {} linhas ({} em captura), {} textos",
            a.linhas, a.capturadas, a.textos
        );
        for f in a.irmaos.iter().chain(&a.grudadas).chain(&a.cortados) {
            println!("    // {f}");
        }
        println!(
            "POPULACAO_RITMO (\"{painel}\", {}, {}, {}),",
            a.pecas_texto, a.pecas_controlo, a.fronteiras
        );
        println!("(\"{painel}\", {}),", a.ritmo.len());
        for f in &a.ritmo {
            println!("    RITMO {f}");
        }
    }
}

/// ⭐ A grade do 9-Slice no cartão (foto do dono, 04/11): três filas e três colunas de células da altura de
/// todo controlo, os dois atalhos numa fila própria, de ponta a ponta, e cada id responde ao clique no meio.
#[test]
fn a_grade_do_9_slice_anda_fila_a_fila() {
    use ph2d_card_stack::metrics as m;
    use ph2d_panel_inspector::ids as i;
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        let mut host = MockPanelHost::new().in_theme(ph2d_tokens::Theme::Dark);
        super::o_inspector_armado::arma_tudo();
        painel.populate(host.store_mut());
        let _ = mede(&mut host, painel);
        let regs = host.registos_da_ultima_pintura();
        let rect = |id| {
            regs.iter()
                .rev()
                .find(|(n, _)| *n == id)
                .map(|(_, r)| *r)
                .expect("id pintado")
        };
        let celulas: Vec<_> = i::INSP_SLICE_REGION
            .iter()
            .copied()
            .chain([i::INSP_SLICE_CENTRE])
            .collect();
        let mut ys: Vec<f32> = celulas.iter().map(|&c| rect(c).y).collect();
        let mut xs: Vec<f32> = celulas.iter().map(|&c| rect(c).x).collect();
        for v in [&mut ys, &mut xs] {
            v.sort_by(f32::total_cmp);
            v.dedup();
        }
        assert_eq!((ys.len(), xs.len()), (3, 3), "{ys:?} {xs:?}");
        let passo = m::FIELD_H + m::FIELD_ROW_GAP;
        assert!(
            ys.windows(2).all(|p| (p[1] - p[0] - passo).abs() < 0.5),
            "{ys:?}"
        );
        assert!(celulas.iter().all(|&c| rect(c).h == m::FIELD_H));
        let (tile, stretch) = (
            rect(i::INSP_SLICE_ALL_TILE),
            rect(i::INSP_SLICE_ALL_STRETCH),
        );
        assert_eq!((tile.y, tile.h), (stretch.y, stretch.h));
        assert_eq!(tile.h, m::FIELD_H);
        assert!((tile.y - (ys[2] + m::FIELD_H) - m::FIELD_ROW_GAP).abs() < 0.5);
        assert_eq!(tile.x, xs[0], "os atalhos começam na coluna da grade");
        assert!((tile.w - stretch.w).abs() < 0.5, "duas colunas iguais");
        for id in celulas
            .iter()
            .copied()
            .chain([i::INSP_SLICE_ALL_TILE, i::INSP_SLICE_ALL_STRETCH])
        {
            let r = rect(id);
            let no_meio = host.hit_index_mut().hit(r.x + r.w * 0.5, r.y + r.h * 0.5);
            assert_eq!(
                no_meio,
                Some(id),
                "o clique no meio de {id:?} chega a outro"
            );
        }
        super::o_inspector_armado::desarma_tudo();
    });
}

/// Pinta o painel no seu `n_estado`-ésimo estado de `censo_estados` e devolve os rects da última pintura.
#[cfg(all(feature = "panel-audio-editor", feature = "panel-vector"))]
fn rects_no_estado(
    painel_id: &str,
    n_estado: usize,
) -> (
    Vec<(ph2d_editor_core::NodeId, ph2d_editor_core::zones::Rect)>,
    Censo,
) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    let e = super::censo_estados::ESTADOS
        .iter()
        .filter(|e| e.painel == painel_id)
        .nth(n_estado)
        .expect("o estado");
    let (mut out, mut censo) = (Vec::new(), None);
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == painel_id)
            .expect("o painel");
        let mut host = MockPanelHost::new().in_theme(ph2d_tokens::Theme::Dark);
        (e.arma)();
        painel.populate(host.store_mut());
        censo = Some(mede(&mut host, painel));
        out = host.registos_da_ultima_pintura();
        (e.desarma)();
    });
    (out, censo.expect("pintou"))
}

#[track_caller]
fn rect_de(
    regs: &[(ph2d_editor_core::NodeId, ph2d_editor_core::zones::Rect)],
    id: ph2d_editor_core::NodeId,
) -> ph2d_editor_core::zones::Rect {
    regs.iter()
        .rev()
        .find(|(n, _)| *n == id)
        .map_or_else(|| panic!("{id:?} não foi pintado"), |(_, r)| *r)
}

/// ⭐ O Audio Editor num cartão: os botões do mesmo assunto são peças soltas, nas colunas e no vão da grelha, e o
/// `Export Set` não é empurrado por uma fileira vazia. *Mutação: `FIELD_ROW_GAP` ⇒ 0, ou o `+ gap` em dobro.*
#[cfg(all(feature = "panel-audio-editor", feature = "panel-vector"))]
#[test]
fn os_botoes_do_audio_ficam_soltos_na_grelha() {
    use ph2d_card_stack::metrics as m;
    use ph2d_panel_audio_editor as a;
    let (regs, _) = rects_no_estado("audio_editor", 0);
    let (play, stop, laco, carrega) = (
        rect_de(&regs, a::AEDIT_PLAY),
        rect_de(&regs, a::AEDIT_STOP),
        rect_de(&regs, a::AEDIT_LOOP),
        rect_de(&regs, a::AEDIT_LOAD),
    );
    for r in [play, stop, laco, carrega] {
        assert_eq!(r.h, m::FIELD_H);
    }
    assert!((stop.y - (play.y + play.h) - m::FIELD_ROW_GAP).abs() < 0.5);
    assert!((carrega.y - (stop.y + stop.h) - m::FIELD_ROW_GAP).abs() < 0.5);
    assert!((laco.x - (stop.x + stop.w) - m::FIELD_GAP).abs() < 0.5);
    assert!((stop.w - laco.w).abs() < 0.5, "duas colunas iguais");
    assert_eq!(play.x, stop.x, "a coluna do cartão");
    assert!((play.w - (stop.w + laco.w + m::FIELD_GAP)).abs() < 0.5);
    // Os quatro botões da cadeia, soltos e acabando na borda do cartão.
    let mais = rect_de(&regs, a::AEDIT_FX_ADD);
    let espera = play.x + play.w - 4.0 * mais.w - 3.0 * m::FIELD_GAP;
    assert!((mais.x - espera).abs() < 0.5, "{mais:?} contra {espera}");
    let codec = rect_de(&regs, a::AEDIT_CODEC_PREV);
    let seta = rect_de(&regs, a::AEDIT_CODEC_NEXT);
    assert!(
        (seta.x + seta.w - (play.x + play.w)).abs() < 0.5,
        "a seta de `◀ nome ▶` acaba na borda do cartão: {seta:?} {play:?}"
    );
    let qualidade = regs
        .iter()
        .rev()
        .find(|(n, _)| *n == a::AEDIT_EXPORT_SET)
        .map(|(_, r)| *r)
        .expect("Export Set");
    // Seletor, rótulo + barra, RAM e o botão: nenhuma fileira vazia no meio.
    let cabe = m::FIELD_H + 3.0 * m::LABEL_PX + 3.0 * m::FIELD_ROW_GAP;
    assert!(
        qualidade.y - (codec.y + codec.h) < cabe,
        "{} entre o seletor e o Export Set",
        qualidade.y - (codec.y + codec.h)
    );
}

/// ⭐ Vector: cada membro do Advanced abaixo do 1.º desce `GROUP_TOP` (grupo a grupo), Align e Width são as duas
/// colunas do cartão. *Mutação: tirar o vão de `membro`; `Choice` ⇒ `Icons` em `paragraph`.*
#[cfg(all(feature = "panel-audio-editor", feature = "panel-vector"))]
#[test]
fn o_advanced_separa_os_grupos_e_o_paragraph_tem_duas_colunas() {
    use ph2d_card_stack::metrics as m;
    use ph2d_panel_vector::ids as p;
    // `avancado`, o 6.º estado de `censo_estados`; o texto é o 5.º.
    let (_, censo) = rects_no_estado("vector", 5);
    let distancias = censo.distancias();
    for grupo in ["Morph", "Envelope"] {
        let d = distancias
            .iter()
            .find(|d| d.para == grupo)
            .unwrap_or_else(|| panic!("{grupo} não está no censo"));
        assert!(
            (d.vao - m::GROUP_TOP).abs() <= 1.5,
            "{grupo} a {} do controlo de cima",
            d.vao
        );
    }
    let (regs, _) = rects_no_estado("vector", 4);
    let (esq, dir, auto, fixa) = (
        rect_de(&regs, p::VECTOR_TEXT_ALIGN_LEFT),
        rect_de(&regs, p::VECTOR_TEXT_ALIGN_RIGHT),
        rect_de(&regs, p::VECTOR_TEXT_WRAP_AUTO),
        rect_de(&regs, p::VECTOR_TEXT_WRAP_FIXED),
    );
    assert_eq!(esq.y, auto.y, "a mesma fila");
    let largura_alinha = dir.x + dir.w - esq.x;
    let largura_quebra = fixa.x + fixa.w - auto.x;
    assert!(
        (largura_alinha - largura_quebra).abs() < 0.5,
        "{largura_alinha} × {largura_quebra}"
    );
}

/// ⭐ Pixel: os verbos do menu Select andam aos pares pela ordem (lei 1 de `05 §4`), e cada um responde ao clique no
/// meio. *Mutação: `n = 1` em `filas_de_botoes` ⇒ vermelho.*
#[cfg(feature = "panel-pixel-tools")]
#[test]
fn os_verbos_do_pixel_andam_aos_pares() {
    use ph2d_card_stack::metrics as m;
    use ph2d_panel_pixel_tools::ids::SELECT_ACTIONS;
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    let arm = super::paineis_armados::TABELA
        .iter()
        .find(|a| a.painel == "pixel_tools")
        .expect("a armação do Pixel");
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "pixel_tools")
            .expect("o painel");
        let mut host = MockPanelHost::new().in_theme(ph2d_tokens::Theme::Dark);
        (arm.arma)(host.store_mut());
        painel.populate(host.store_mut());
        let _ = mede(&mut host, painel);
        let regs = host.registos_da_ultima_pintura();
        let (tudo, inverso) = (
            rect_de(&regs, SELECT_ACTIONS[0].0),
            rect_de(&regs, SELECT_ACTIONS[3].0),
        );
        assert_eq!(tudo.y, inverso.y, "All e Inverse na mesma fila");
        assert!((inverso.x - (tudo.x + tudo.w) - m::FIELD_GAP).abs() < 0.5);
        assert!((tudo.w - inverso.w).abs() < 0.5);
        for (id, r) in [(SELECT_ACTIONS[0].0, tudo), (SELECT_ACTIONS[3].0, inverso)] {
            let no_meio = host.hit_index_mut().hit(r.x + r.w * 0.5, r.y + r.h * 0.5);
            assert_eq!(no_meio, Some(id));
        }
        (arm.desarma)();
    });
}
