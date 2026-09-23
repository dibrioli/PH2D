//! ⭐⭐⭐ **NENHUMA ESCOLHA DO APP TEM O NOME POR CIMA — medido pela GEOMETRIA, não pelo texto.**
//!
//! ⛔⛔ **Ordens do dono:** *«Label acima do campo numérico! Muito ruim!»* (2026-09-14), repetida
//! em 2026-09-15 para as outras onze linhas do Inspector, e *«quanto ao alinhamento precisamos
//! melhorar em todos os lugares»* (2026-09-21).
//!
//! # ⛔⛔⛔ Porque a régua é a GEOMETRIA
//!
//! O gate irmão ([`ph2d-panel-inspector`] `no_row_paints_its_name_above_its_control`) procura o
//! IDIOMA no fonte — avançar o `y` pela altura de um rótulo — e ficou cego **três** vezes pelo NOME
//! da variável: `label_h` (a 1.ª), `label_font` (2026-09-22, sobre a secção que o dono
//! fotografou) e **`font`** (o `seg_row` da TopDown, achado aqui). *Uma régua que depende de como o
//! autor chamou a variável mede o autor, não o painel.*
//!
//! ⇒ esta pergunta ao PRODUTO: pinta-se o painel pela porta do registo, colhem-se os grupos
//! segmentados que ele declara ([`ph2d_editor_core::widget::composto`]), e um grupo que COMEÇA na
//! borda esquerda do conteúdo — em vez de na coluna do valor — não tem um nome ao lado dele.
//! Ou o nome está por cima, ou não há nome.

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_ui_testkit::MockPanelHost;

use super::a_marca_tem_a_altura_da_linha::{abre_as_gavetas, viewport};

/// Um grupo segmentado encontrado a toda a largura: `(painel, x, y, largura, slug da 1.ª peça)`.
#[derive(Debug, Clone)]
pub(crate) struct Grupo {
    pub painel: &'static str,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub borda: f32,
    pub slug: String,
    pub pecas: usize,
    /// ⭐ A porta pintou-o (como PALETA) — `false` é um grupo a toda a largura montado À MÃO.
    pub da_porta: bool,
}

const FONTES: &[&str] = &[
    "../ph2d-panel-inspector/src",
    "../ph2d-panel-grid-snap/src",
    "../ph2d-panel-physics/src",
    "../ph2d-panel-sculpt3d/src",
    "../ph2d-panel-model3d/src",
    "../ph2d-panel-vector/src",
    "../ph2d-panel-flip/src",
    "../ph2d-panel-painter-layers/src",
    "../ph2d-panel-equalize-sizes/src",
    "../ph2d-panel-timeline/src",
    "../ph2d-panel-tokens/src",
    "../ph2d-panel-skeleton/src",
    "../ph2d-panel-tags/src",
    "../ph2d-panel-hierarchy/src",
    "../ph2d-editor-core/src/ids",
    "../ph2d-tool-painter/src",
    "../ph2d-tool-vector/src",
];

fn colhe(
    host: &mut MockPanelHost,
    painel: &mut ph2d_editor_core::panel::ErasedPanel,
    id: &'static str,
    nomes: &std::collections::BTreeMap<ph2d_editor_core::NodeId, String>,
    out: &mut Vec<Grupo>,
) {
    let ((_, grupos), escolhas) = ph2d_editor_core::property_row::escolha::medindo(|| {
        ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, viewport());
        })
    });
    let da_porta: std::collections::BTreeSet<u64> = escolhas.iter().map(|e| e.primeira.0).collect();
    let pintados = host.registos_da_ultima_pintura();
    if pintados.is_empty() {
        return;
    }
    // ⭐ A borda esquerda do CONTEÚDO lê-se do produto: o menor `x` do que o painel registou.
    let borda = pintados
        .iter()
        .map(|(_, r)| r.x)
        .fold(f32::INFINITY, f32::min);
    for g in grupos {
        if g.len() < 2 {
            continue;
        }
        let rects: Vec<_> = pintados
            .iter()
            .filter(|(n, _)| g.contains(n))
            .map(|(_, r)| *r)
            .collect();
        if rects.is_empty() {
            continue;
        }
        let x0 = rects.iter().map(|r| r.x).fold(f32::INFINITY, f32::min);
        let x1 = rects
            .iter()
            .map(|r| r.x + r.w)
            .fold(f32::NEG_INFINITY, f32::max);
        let y0 = rects.iter().map(|r| r.y).fold(f32::INFINITY, f32::min);
        if (x0 - borda).abs() > 1.5 {
            continue;
        }
        out.push(Grupo {
            painel: id,
            x: x0,
            y: y0,
            w: x1 - x0,
            borda,
            slug: nomes
                .get(&g[0])
                .cloned()
                .unwrap_or_else(|| format!("#{:x}", g[0].0)),
            pecas: g.len(),
            da_porta: g.iter().any(|n| da_porta.contains(&n.0)),
        });
    }
}

/// Toda escolha a toda a largura, em todo painel — de fábrica e armado.
pub(crate) fn a_toda_a_largura() -> Vec<Grupo> {
    let nomes = super::o_que_o_artista_nao_alcanca::nomes_de(FONTES, 400);
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut out = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            let mut host = MockPanelHost::new();
            painel.populate(host.store_mut());
            abre_as_gavetas(host.store_mut());
            colhe(&mut host, painel, id, &nomes, &mut out);
            if let Some(arm) = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id)
            {
                let mut host = MockPanelHost::new();
                (arm.arma)(host.store_mut());
                painel.populate(host.store_mut());
                abre_as_gavetas(host.store_mut());
                colhe(&mut host, painel, id, &nomes, &mut out);
                (arm.desarma)();
            }
        }
    });
    out.sort_by(|a, b| (a.painel, a.slug.as_str()).cmp(&(b.painel, b.slug.as_str())));
    out.dedup_by(|a, b| a.painel == b.painel && a.slug == b.slug);
    out
}

/// ⛔ **As exceções NOMEADAS do app** — grupos a toda a largura que NÃO são uma escolha com nome.
///
/// Cada uma diz porque não passa pela porta da ESCOLHA; e o gate exige que ela CONTINUE a existir
/// (censo de obsolescência), senão a lista vira licença.
const FORA: &[(&str, &str, &str)] = &[
    (
        "inspector",
        "insp_vis_layer_bit_0",
        "grelha de 32 bits das camadas: um MAPA de bits, não um-entre-N — cada célula liga e \
         desliga sozinha, e o nome dela (`Layers`) já vive na coluna",
    ),
    (
        "painter_layers",
        "painter_sidebar.toggle_dock",
        "as ABAS do painel (Brush · Layers): uma navegação entre vistas, pedida assim pelo dono \
         (2026-09-09), e não uma propriedade — um nome ao lado dela não diria nada",
    ),
];

/// ⭐⭐⭐ **NENHUMA ESCOLHA DO APP É MONTADA À MÃO** — todo grupo segmentado a toda a largura, em
/// todo painel, vem da porta [`ph2d_editor_core::property_row::paint_choice_row`] (que só o põe a
/// toda a largura quando a MEDIDA diz que ao lado do nome ele refluiria em duas fileiras ou mais).
///
/// ⭐ Nasceu em 2026-09-23 como gate do Inspector mais uma catraca de `33` nos outros painéis; a
/// catraca chegou a ZERO no ciclo seguinte (Vector e Esqueleto pelo `RowCtx::segmented`, a Física,
/// a Escultura, o Upscale e as sete fileiras do Modelo 3D, que não tinham nome nenhum) e morreu.
///
/// **Mutações que devem sangrar:** um painel voltar ao `paint_segmented_group_adaptive` com o nome
/// por cima · uma exceção deixar de existir no painel.
#[test]
fn nenhuma_escolha_do_app_e_montada_a_mao() {
    // ⛔⛔ **A guarda de âmbito** — a lição que a varredura das elisões pagou em 20/09: `flip`,
    //    `flip_frames`, `painter_layers` e `wet_tuning` não estão no `default` desta crate e só a
    //    unificação de features de um build de WORKSPACE os regista. Nesse âmbito pobre a exceção
    //    do Painter leria-se «obsoleta». ⇒ reprovar ALTO com a causa.
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut registados = std::collections::BTreeSet::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for p in reg.panels_mut() {
            registados.insert(p.manifest.id);
        }
    });
    assert!(
        registados.contains("painter_layers"),
        "o painel `painter_layers` nao esta' registado: esta corrida e' do ambito POBRE \
         (`-p ph2d-panel-registry-init`). O gate e' medido no ambito do APP ⇒ corra \
         `cargo nextest run --workspace -E 'test(nenhuma_escolha_do_app_e_montada_a_mao)'`."
    );
    let v = a_toda_a_largura();
    // ⛔ Piso de população: medido 48 grupos pela porta a toda a largura (2026-09-23). Sem ele,
    //    um arnês que deixasse de armar os painéis passaria por vácuo.
    let pela_porta = v.iter().filter(|g| g.da_porta).count();
    assert!(
        pela_porta >= 35,
        "so' {pela_porta} escolhas pela porta a toda a largura — o arnes deixou de armar paineis?"
    );
    let a_mao: Vec<String> = v
        .iter()
        .filter(|g| !g.da_porta && !FORA.iter().any(|(p, s, _)| *p == g.painel && *s == g.slug))
        .map(|g| {
            format!(
                "{} · {} ({} pecas, y={:.0})",
                g.painel, g.slug, g.pecas, g.y
            )
        })
        .collect();
    assert!(
        a_mao.is_empty(),
        "escolhas montadas A MAO a toda a largura (sem nome ao lado) — passe-as pela porta \
         `paint_choice_row`:\n  {}",
        a_mao.join("\n  ")
    );
    // ⛔ Censo de obsolescência: uma exceção que já não aparece é uma licença esquecida.
    for (painel, slug, porque) in FORA {
        assert!(
            v.iter()
                .any(|g| !g.da_porta && g.painel == *painel && g.slug == *slug),
            "a excecao «{painel} · {slug}» ({porque}) ja' nao aparece — apague-a de FORA"
        );
    }
}

/// ⭐⭐⭐ **A LEI DA FORMA da porta da escolha, pelo painel armado:** uma escolha que cabe numa
/// fileira ao lado do nome vai AO LADO; uma que não cabe vira PALETA — e as duas formas existem.
///
/// ⚠️ **As duas metades são o que a torna uma lei e não um gosto:** uma porta que pusesse tudo ao
/// lado faria torres de seis e sete fileiras (medido: a família de easing e os canais do Tween); uma
/// que pusesse tudo em paleta devolvia o nome por cima a quem cabia ao lado — o report do dono.
///
/// **Mutações que devem sangrar:** `ao_lado = true` · `ao_lado = false` · o limiar trocar de `1`.
#[test]
fn a_forma_da_escolha_sai_da_medida() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let escolhas = ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o Inspector esta' registado");
        super::o_inspector_armado::arma_tudo();
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        super::quantas_entradas_tem_cada_painel::abre_tudo(host.store_mut());
        let (_, escolhas) = ph2d_editor_core::property_row::escolha::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, viewport());
        });
        super::o_inspector_armado::desarma_tudo();
        escolhas
    });
    // ⛔ Pisos das DUAS populações: medido 2026-09-23, `4` ao lado e `18` em paleta no dock de
    //    omissão. Sem o piso de uma delas, a mutação que a apaga passaria verde.
    let ao_lado = escolhas.iter().filter(|e| e.ao_lado).count();
    let paleta = escolhas.len() - ao_lado;
    assert!(
        ao_lado >= 2,
        "so' {ao_lado} escolhas AO LADO do nome — a porta deixou de medir?"
    );
    assert!(
        paleta >= 10,
        "so' {paleta} escolhas em PALETA — a porta deixou de medir?"
    );
    let erradas: Vec<String> = escolhas
        .iter()
        .filter(|e| e.ao_lado != (e.fileiras <= 1))
        .map(|e| {
            format!(
                "{:?}: {} fileiras e ao_lado={}",
                e.primeira, e.fileiras, e.ao_lado
            )
        })
        .collect();
    assert!(
        erradas.is_empty(),
        "a forma nao obedece a' medida:\n  {}",
        erradas.join("\n  ")
    );
}

/// ⛔ SONDA — que escolhas começam na borda esquerda.
#[test]
#[ignore = "sonda de diagnóstico — corre à mão com --ignored --nocapture"]
fn diag_que_escolhas_comecam_na_borda() {
    let v = a_toda_a_largura();
    eprintln!("=== {} grupos segmentados a toda a largura ===", v.len());
    for g in &v {
        eprintln!(
            "{} {:<16} pecas={:2} x={:6.1} borda={:6.1} y={:7.1} w={:6.1}  {}",
            if g.da_porta { "PORTA " } else { "A_MAO " },
            g.painel,
            g.pecas,
            g.x,
            g.borda,
            g.y,
            g.w,
            g.slug
        );
    }
}

/// ⛔ SONDA — em quantas fileiras cada escolha do Inspector reflui AO LADO do nome.
#[test]
#[ignore = "sonda de diagnóstico — corre à mão com --ignored --nocapture"]
fn diag_quantas_fileiras_cada_escolha_ocupa_ao_lado() {
    let nomes = super::o_que_o_artista_nao_alcanca::nomes_de(FONTES, 400);
    let _ = ph2d_panel_registry_init::register_all_panels();
    for (vw, legenda) in [(1366.0, "omissao"), (1286.0, "minimo")] {
        let vp = ph2d_editor_core::zones::Rect {
            x: 0.0,
            y: 0.0,
            w: vw,
            h: 1024.0,
        };
        ph2d_editor_core::panel::with_registry(|reg| {
            let painel = reg
                .panels_mut()
                .iter_mut()
                .find(|p| p.manifest.id == "inspector")
                .expect("inspector");
            super::o_inspector_armado::arma_tudo();
            let mut host = MockPanelHost::new();
            painel.populate(host.store_mut());
            super::quantas_entradas_tem_cada_painel::abre_tudo(host.store_mut());
            let (_, escolhas) = ph2d_editor_core::property_row::escolha::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, vp);
            });
            eprintln!("=== {legenda}: {} escolhas pela porta ===", escolhas.len());
            let mut v: Vec<_> = escolhas
                .iter()
                .map(|e| {
                    (
                        e.fileiras,
                        e.pecas,
                        nomes.get(&e.primeira).cloned().unwrap_or_default(),
                    )
                })
                .collect();
            v.sort();
            v.dedup();
            for (f, p, s) in v {
                eprintln!("  fileiras={f} pecas={p:2}  {s}");
            }
            super::o_inspector_armado::desarma_tudo();
        });
    }
}
