//! A janela LINT por gesto real, sobre a fixture `vicios_p13` do `ph2d-pixel-lint`: abrir ⇒ seis
//! grupos; clicar no Doubles ⇒ o foco centra a câmera na célula e as células ficam marcadas; pintar a
//! correção ⇒ o grupo some na corrida seguinte (e nunca corre a cada quadro).

#[path = "../../../ph2d-pixel-lint/tests/it/fixture.rs"]
#[allow(dead_code)]
mod fixture;

use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_lint::canal::{self, LintView};
use ph2d_panel_pixel_lint::{PixelLintPanel, PixelLintPanelState, ids};
use ph2d_pixel_core::{Op, Patch, Rect as PRect, Sprite, apply};
use ph2d_pixel_lint::{Regra, Regras};
use ph2d_ui_testkit::mesa::Mesa;
use ph2d_vector::{Affine, VectorScene};

const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 1400.0);

fn regras_da_fixture() -> Regras {
    // 8 cores na fixture: com 7, a mais rara (a do órfão) passa do limite.
    Regras {
        limite_cores: 7,
        ..Regras::default()
    }
}

fn mesa() -> Mesa<PixelLintPanel> {
    canal::corre(None, true);
    canal::poe_regras(regras_da_fixture());
    let mut m = Mesa::nova(PixelLintPanelState);
    m.host.set_panel_visible(PixelLintPanel::ID, true);
    m.assenta_em(JANELA);
    m
}

fn corre(s: &Sprite, stamp: u64) {
    canal::corre(Some((7, s, s.frames()[0].id, stamp)), true);
}

fn apaga(s: &mut Sprite, x: u32, y: u32) {
    let op = Op::Patch(Patch {
        layer: s.layers()[0].id,
        frame: s.frames()[0].id,
        bbox: Some(PRect::new(x as i32, y as i32, 1, 1)),
        rect: PRect::new(x as i32, y as i32, 1, 1),
        pixels: vec![0; 4],
    });
    apply(s, op).expect("patch");
}

fn regras_vistas(v: &LintView) -> Vec<Regra> {
    v.grupos.iter().map(|g| g.regra).collect()
}

#[test]
fn abrir_o_lint_na_fixture_da_seis_grupos_pintados() {
    let mut m = mesa();
    let (t, _) = fixture::vicios_p13();
    let s = t.sprite();
    corre(&s, 1);
    m.assenta_em(JANELA);
    assert_eq!(regras_vistas(&canal::view()), Regra::TODAS.to_vec());
    for r in Regra::TODAS {
        assert!(
            m.rect(ids::grupo(r)).is_some(),
            "{r:?}: grupo sem linha pintada"
        );
        assert!(
            m.rect(ids::regra(r)).is_some(),
            "{r:?}: caixa sem linha pintada"
        );
    }
    assert!(m.rect(ids::COLOR_LIMIT).is_some());
}

#[test]
fn clicar_no_doubles_centra_o_canvas_na_celula_e_corrigir_tira_o_grupo() {
    let mut m = mesa();
    let (t, _) = fixture::vicios_p13();
    let mut s = t.sprite();
    corre(&s, 1);
    m.assenta_em(JANELA);
    let _ = m.clica(ids::grupo(Regra::CantoDuplo));
    m.assenta_em(JANELA);
    assert_eq!(canal::marcadas(), vec![[7, 3], [7, 4]]);
    // O afim célula → ecrã do último desenho: 10 px por célula, origem em (100, 50). A câmera da
    // família, aqui, é a identidade ecrã = mundo.
    let afim = Affine::translate((100.0, 50.0)) * Affine::scale(10.0);
    canal::desenha(
        7,
        afim,
        ph2d_vector::Color::WHITE,
        1.0,
        &mut VectorScene::new(),
    );
    let area = [0.0, 0.0, 800.0, 600.0];
    let d = canal::foca(area, |p| [p.0, p.1]);
    // A célula (7, 3) tem o meio em (175, 85): somar (175 − 400, 85 − 300) ao centro põe-na no meio.
    assert_eq!(d, [175.0 - 400.0, 85.0 - 300.0]);
    assert_eq!(
        canal::foca(area, |p| [p.0, p.1]),
        [0.0, 0.0],
        "o foco drena-se uma vez"
    );
    // Pintar a correção: o canto do Z sai, a linha fica um degrau diagonal.
    apaga(&mut s, 7, 3);
    corre(&s, 2);
    m.assenta_em(JANELA);
    let v = canal::view();
    assert!(!regras_vistas(&v).contains(&Regra::CantoDuplo), "{v:?}");
    assert!(m.rect(ids::grupo(Regra::CantoDuplo)).is_none());
    assert!(
        canal::marcadas().is_empty(),
        "a marca do aviso corrigido some com ele"
    );
}

#[test]
fn o_lint_nao_corre_a_cada_quadro_nem_durante_o_traco_nem_fechado() {
    let mut m = mesa();
    let (t, _) = fixture::vicios_p13();
    let s = t.sprite();
    corre(&s, 1);
    let n = canal::corridas();
    for _ in 0..10 {
        corre(&s, 1);
        m.assenta_em(JANELA);
    }
    assert_eq!(canal::corridas(), n, "o mesmo carimbo não repete o lint");
    canal::corre(Some((7, &s, s.frames()[0].id, 2)), false);
    assert_eq!(canal::corridas(), n, "traço em curso: espera o soltar");
    corre(&s, 2);
    assert_eq!(canal::corridas(), n + 1, "soltar o traço corre");
    canal::poe_aberto(false);
    corre(&s, 3);
    assert_eq!(canal::corridas(), n + 1, "janela fechada: não corre");
}

#[test]
fn a_caixa_da_regra_cala_o_grupo_dela() {
    let mut m = mesa();
    let (t, _) = fixture::vicios_p13();
    let s = t.sprite();
    corre(&s, 1);
    m.assenta_em(JANELA);
    let _ = m.clica(ids::regra(Regra::Faixa));
    m.assenta_em(JANELA);
    assert!(!canal::regras().faixa);
    corre(&s, 1);
    let v = canal::view();
    assert!(!regras_vistas(&v).contains(&Regra::Faixa), "{v:?}");
    assert_eq!(v.grupos.len(), 5);
}

/// O ecrã do smoke (`smoke_P13_cena33`, 1 600 × 872, sem a timeline: a janela Lint tinha ~460 px):
/// com as regras em cima, só 1–2 grupos se viam. Aqui o mock não tem timeline, logo um ecrã de 620 px
/// dá a mesma janela. A lista vem primeiro, e cada linha de grupo e de aviso fica INTEIRA dentro dela.
#[test]
fn num_ecra_baixo_cada_grupo_e_aviso_cabe_inteiro_na_janela() {
    let baixa = Rect::new(0.0, 0.0, 1600.0, 620.0);
    let mut m = mesa();
    let (t, _) = fixture::vicios_p13();
    let s = t.sprite();
    corre(&s, 1);
    m.assenta_em(baixa);
    let janela = m
        .host
        .store()
        .panel_rect(PixelLintPanel::NODE_ID)
        .expect("janela publicada");
    assert!(
        janela.h < 480.0,
        "a janela tem de ser a do smoke: {janela:?}"
    );
    let v = canal::view();
    let dentro = |r: Rect| r.y >= janela.y && r.y + r.h <= janela.y + janela.h + 0.5;
    let mut vistos = 0;
    for g in &v.grupos {
        let ids_do_grupo = std::iter::once(ids::grupo(g.regra))
            .chain((0..g.avisos.len()).map(|i| ids::aviso(g.regra, i)));
        for id in ids_do_grupo {
            let r = m
                .rect(id)
                .unwrap_or_else(|| panic!("{:?}: linha sem rect", g.regra));
            assert!(dentro(r), "{:?}: {r:?} sai da janela {janela:?}", g.regra);
            vistos += 1;
        }
    }
    assert!(vistos >= 10, "{vistos}");
}

/// «1 issues» era o plural colado no código: a frase inteira vem da tabela.
#[test]
fn o_facto_do_grupo_diz_issue_no_singular() {
    let um = ph2d_i18n::tr_with("pixel.lint.count", &[("n", &1), ("severity", &"Low")]);
    let tres = ph2d_i18n::tr_with("pixel.lint.count", &[("n", &3), ("severity", &"Low")]);
    assert_eq!(um, "1 issue \u{b7} Low");
    assert_eq!(tres, "3 issues \u{b7} Low");
}

/// «Orphan pi…»: na largura de FÁBRICA (o artista nunca alargou), o nome de cada regra lê-se
/// inteiro na linha do grupo, ao lado do facto.
#[test]
fn na_largura_de_fabrica_o_nome_de_cada_regra_le_se_inteiro() {
    let mut m = mesa();
    let (t, _) = fixture::vicios_p13();
    let s = t.sprite();
    corre(&s, 1);
    m.assenta_em(JANELA);
    let nomes: Vec<&str> = Regra::TODAS
        .iter()
        .map(|&r| ph2d_i18n::tr(ph2d_panel_pixel_lint::chave_da_regra(r)))
        .collect();
    let medidos = m
        .host
        .medindo_a_pintura::<PixelLintPanel>(&mut m.st, JANELA);
    let cortados: Vec<_> = medidos
        .iter()
        .filter(|md| nomes.contains(&md.texto.as_str()) && md.pintado != md.texto)
        .map(|md| (&md.texto, &md.pintado, md.largura))
        .collect();
    assert!(cortados.is_empty(), "{cortados:?}");
}
