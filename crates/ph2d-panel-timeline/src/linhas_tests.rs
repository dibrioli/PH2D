//! Tests for [`super`] (`linhas.rs`) — as linhas de grupo do dope-sheet.

use super::*;
use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_editor_core::interaction::{GestureMods, GesturePhase, TimelineGesture};
use ph2d_host::PointerButton;
use ph2d_timeline::{AnimTarget, Extrap, Interp, KeyId, KeyView, TimelineIntent};

use crate::state::{self, TimelinePanelState};

const GH: f32 = 132.0;
const FILL_R: u16 = v::TINTA_PREENCH + v::OFF_SOLIDA;

fn field(campo: u16, i: u16, j: u16) -> PropKind {
    PropKind::Field(FieldKey::new(VETOR, campo, i, j))
}

/// Uma faixa com chaves nos instantes `ts` (id da chave = `target * 100 + n`).
fn track(target: u64, entity: u64, prop: PropKind, ts: &[f64]) -> TrackView {
    TrackView {
        target: AnimTarget::new(target),
        prop,
        entity,
        missing: false,
        keys: ts
            .iter()
            .enumerate()
            .map(|(n, &t)| KeyView {
                id: KeyId::new(target * 100 + n as u64),
                t_seconds: t,
                value: 0.0,
                interp: Interp::Linear,
                selected: false,
                roving: false,
            })
            .collect(),
        buffer_ghost: None,
        pre: Extrap::Hold,
        post: Extrap::Hold,
        expr: None,
    }
}

/// O que o autokey grava numa cor de preenchimento: os 4 canais da cor chapada, chave em 0 e 1 —
/// ligados FORA de ordem (A, R, B, G), entre um X e uma largura de traço (grupo de um só).
fn cena() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        tracks: vec![
            track(1, 9, PropKind::TranslationX, &[0.0]),
            track(2, 9, field(FILL_R + 3, 0, 0), &[0.0, 1.0]),
            track(3, 9, field(FILL_R, 0, 0), &[0.0, 1.0]),
            track(4, 9, field(FILL_R + 2, 0, 0), &[0.0, 1.0]),
            track(5, 9, field(FILL_R + 1, 0, 0), &[0.0, 1.0]),
            track(6, 9, field(v::TRACO_LARGURA, 0, 0), &[0.0]),
        ],
        ..TimelineViewSnapshot::default()
    }
}

/// O agrupamento é o mesmo nos dois caminhos; estes testes leem-no na disposição plana (sem o
/// cabeçalho do objecto dos cartões, que tem os testes dele no fim do ficheiro).
fn classico() {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(false));
}

fn linhas(snap: &TimelineViewSnapshot, abertos: &[GrupoChave]) -> Vec<Linha> {
    classico();
    layout(snap, Tab::Keys, &[], abertos, GH, 0.0, 0.0)
        .bandas
        .iter()
        .map(|b| b.linha)
        .collect()
}

fn chave_fill() -> GrupoChave {
    GrupoChave::de(&cena().tracks[1]).expect("a cor é um grupo")
}

#[test]
fn a_cor_inteira_e_uma_linha_so() {
    assert_eq!(
        linhas(&cena(), &[]),
        vec![
            Linha::Faixa { i: 0, grupo: None },
            Linha::Grupo(0),
            Linha::Faixa { i: 5, grupo: None },
        ],
        "X · Fill (os 4 canais numa linha) · Stroke Width (grupo de um: sem cabeçalho)"
    );
}

#[test]
fn abrir_o_grupo_mostra_os_membros_na_ordem_de_leitura() {
    let s = cena();
    let l = linhas(&s, &[chave_fill()]);
    let membros: Vec<_> = l[2..6]
        .iter()
        .map(|x| match *x {
            Linha::Faixa { i, grupo: Some(0) } => i,
            o => panic!("membro esperado, veio {o:?}"),
        })
        .collect();
    assert_eq!(
        membros,
        vec![2, 4, 3, 1],
        "R, G, B, A — não a ordem das ligações"
    );
    let lay = layout(&s, Tab::Keys, &[], &[chave_fill()], GH, 0.0, 0.0);
    let nomes: Vec<_> = membros
        .iter()
        .map(|&i| rotulo_membro(&s, &lay.grupos[0], i))
        .collect();
    assert_eq!(nomes, vec!["R", "G", "B", "A"]);
    assert_eq!(
        l.len(),
        7,
        "o cabeçalho continua, os membros vêm por baixo dele"
    );
}

#[test]
fn o_mesmo_grupo_em_dois_objetos_sao_duas_linhas() {
    let mut s = cena();
    s.tracks.push(track(7, 8, field(FILL_R, 0, 0), &[0.5]));
    s.tracks.push(track(8, 8, field(FILL_R + 1, 0, 0), &[0.5]));
    let grupos = linhas(&s, &[])
        .into_iter()
        .filter(|l| matches!(l, Linha::Grupo(_)))
        .count();
    assert_eq!(grupos, 2);
}

#[test]
fn o_ponto_da_forma_le_se_pelo_indice_e_o_contorno_so_quando_ha_dois() {
    let mut s = TimelineViewSnapshot::default();
    for (n, (i, j)) in [(0, 0), (0, 1)].into_iter().enumerate() {
        s.tracks
            .push(track(n as u64 + 1, 9, field(v::ANCORA_X, i, j), &[0.0]));
    }
    s.tracks.push(track(9, 9, field(v::FECHADO, 0, 0), &[0.0]));
    let lay = layout(&s, Tab::Keys, &[], &[], GH, 0.0, 0.0);
    let g = &lay.grupos[0];
    let nome = |i| rotulo_membro(&s, g, i);
    assert_eq!(
        (nome(0), nome(1), nome(2)),
        ("Point X 1".into(), "Point X 2".into(), "Closed".into())
    );

    s.tracks
        .push(track(10, 9, field(v::ANCORA_X, 1, 0), &[0.0]));
    let lay = layout(&s, Tab::Keys, &[], &[], GH, 0.0, 0.0);
    assert_eq!(rotulo_membro(&s, &lay.grupos[0], 3), "Point X 2.1");
}

fn gesto(kind: TimelineHitKind, phase: GesturePhase) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind,
        phase,
        x: 0.0,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    }
}

/// O losango do grupo agarra as chaves dos MEMBROS naquele instante — não a do X, que está no
/// mesmo instante noutra linha. (Mutação: `Escopo::Tudo` no braço `GroupKey` ⇒ entra a chave do X.)
#[test]
fn o_losango_do_grupo_seleciona_so_os_membros() {
    let s = cena();
    let mut st = TimelinePanelState::default();
    let _ = state::drain_intents();
    let kind = TimelineHitKind::GroupKey {
        group: 2,
        t_bits: 0.0_f64.to_bits(),
    };
    crate::interact::dispatch_primary(&mut st, 0.0, 120.0, &s, gesto(kind, GesturePhase::Begin));
    let mut sel: Vec<_> = state::drain_intents()
        .into_iter()
        .filter_map(|i| match i {
            TimelineIntent::AddToSelection(k) => Some(k.target.get()),
            _ => None,
        })
        .collect();
    sel.sort_unstable();
    assert_eq!(sel, vec![2, 3, 4, 5]);
}

#[test]
fn o_triangulo_do_grupo_abre_e_fecha_os_membros() {
    let s = cena();
    let mut st = TimelinePanelState::default();
    let kind = TimelineHitKind::GroupTwirl { group: 4 };
    crate::interact::dispatch_primary(&mut st, 0.0, 120.0, &s, gesto(kind, GesturePhase::Click));
    assert_eq!(st.grupos_abertos, vec![chave_fill()]);
    crate::interact::dispatch_primary(&mut st, 0.0, 120.0, &s, gesto(kind, GesturePhase::Click));
    assert!(st.grupos_abertos.is_empty());
}

/// O retângulo de seleção sobre a linha do grupo leva as chaves dos membros dentro dele.
#[test]
fn o_retangulo_sobre_a_linha_do_grupo_leva_os_membros() {
    classico();
    let s = cena();
    let st = TimelinePanelState::default();
    let view = TimeView {
        time_x: 100.0,
        view_start: 0.0,
        px_per_s: 100.0,
        right: 1000.0,
    };
    let rows = Rect::new(0.0, 0.0, 1000.0, 400.0);
    // O Summary é a linha 0, o X a 1, o grupo a 2: o centro dele está em 2,5 linhas.
    let cy = ROW_H_PX * 2.5;
    let sel = Rect::new(150.0, cy - 2.0, 100.0, 4.0); // só o instante 1 (x = 200)
    let mut got: Vec<_> = keys_in_rect(rows, view, &st, &s, sel)
        .into_iter()
        .map(|k| k.target.get())
        .collect();
    got.sort_unstable();
    assert_eq!(got, vec![2, 3, 4, 5]);
}

/// Todo campo que o vocabulário do vetor declara tem um nome de membro na tabela de strings — sem
/// isso a linha do membro pinta o identificador cru. (Controlo: mais de 60 campos varridos.)
#[test]
fn todo_campo_do_vetor_tem_nome_de_membro_traduzido() {
    let mut n = 0;
    for campo in 0..=u16::from(u8::MAX) {
        let k = FieldKey::new(VETOR, campo, 0, 0);
        if k.info().is_none() {
            continue;
        }
        n += 1;
        let m = k.membro().expect("campo declarado sem nome de membro");
        assert_ne!(ph2d_i18n::tr(m), m, "campo {campo}: «{m}» sem tradução");
    }
    assert!(n > 60, "só {n} campos — o varrimento leu o sítio errado");
}

/// A tinta da camada 1 da pilha lê-se «Fill 1», a de base só «Fill».
#[test]
fn a_tinta_de_uma_camada_da_pilha_leva_o_numero_dela() {
    let base = crate::tracks::track_label(Some("Star"), 9, field(FILL_R, 0, 0));
    let camada = crate::tracks::track_label(Some("Star"), 9, field(FILL_R, 1, 0));
    assert_eq!(
        (base.as_str(), camada.as_str()),
        ("Star · Fill", "Star · Fill 1")
    );
}

/// ⭐ Doc 15 onda 2 (maquete B1): nos cartões cada objecto abre com UMA linha de cabeçalho, e as
/// faixas dele vêm por baixo; um segundo objecto abre o seu, e as faixas de um objecto juntam-se mesmo chegando intercaladas.
#[test]
fn nos_cartoes_o_objeto_e_o_cabecalho_das_suas_faixas() {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let mut s = cena();
    s.tracks
        .insert(1, track(7, 11, PropKind::Rotation, &[0.0, 1.0]));
    let l: Vec<_> = layout(&s, Tab::Keys, &[], &[], GH, 0.0, 0.0)
        .bandas
        .iter()
        .map(|b| b.linha)
        .collect();
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    assert_eq!(
        l,
        vec![
            Linha::Objeto(9),
            Linha::Faixa { i: 0, grupo: None },
            Linha::Grupo(0),
            Linha::Faixa { i: 6, grupo: None },
            Linha::Objeto(11),
            Linha::Faixa { i: 1, grupo: None },
        ]
    );
}

/// ⭐ A faixa sob o cabeçalho diz só a propriedade, inteira — o «Herói · Translat…» cortado era o nome
/// do objecto repetido a comer a coluna (smoke do dono, onda 1).
#[test]
fn a_faixa_diz_so_a_propriedade_e_o_cabecalho_o_objeto() {
    let mut s = cena();
    s.object_names.insert(9, "Hero".to_owned());
    let u =
        crate::tracks_value::Unidades::of(&ph2d_editor_core::project::ProjectSettings::default());
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
    cartoes(Some(true));
    let p = crate::tracks::propriedade_snap(&s, PropKind::TranslationX, u);
    let r = crate::tracks::propriedade_snap(&s, PropKind::Rotation, u);
    cartoes(Some(false));
    let classica = crate::tracks::propriedade_snap(&s, PropKind::TranslationX, u);
    cartoes(None);
    let tx = crate::tracks::prop_label(PropKind::TranslationX);
    assert_eq!(tx, "Position X", "a palavra do Inspector (dono 10/11)");
    // O `Transform` guarda metros e radianos: nos cartões a faixa diz a unidade do projecto, como o
    // Inspector; o caminho clássico fica como era.
    assert_eq!(p, format!("{tx} ({})", u_px(u)));
    assert!(r.ends_with("(°)") || r.ends_with("(rad)"), "{r}");
    assert_eq!(classica, tx);
    assert!(!p.contains("Hero"));
    assert_eq!(crate::tracks::objeto_snap(&s, 9), "Hero");
}

fn u_px(u: crate::tracks_value::Unidades) -> &'static str {
    crate::tracks_value::sufixo_da_unidade(
        PropKind::Field(FieldKey::new(VETOR, v::TRACO_LARGURA, 0, 0)),
        u,
    )
    .map_or("", str::trim)
}

/// Todos os temas do menu — uma lista à mão deixava quatro de fora.
const TEMAS: [ph2d_tokens::Theme; 12] = ph2d_tokens::Theme::ALL;

/// Uma cor no plano OKLab, e a distância — a régua do `BoneHandle`.
fn lab_de(c: ph2d_tokens::Color) -> (f64, f64, f64) {
    let (l, ch, h) = ph2d_tokens::srgb_to_oklch(c.r, c.g, c.b);
    (l, ch * h.to_radians().cos(), ch * h.to_radians().sin())
}

fn dist(x: (f64, f64, f64), y: (f64, f64, f64)) -> f64 {
    ((x.0 - y.0).powi(2) + (x.1 - y.1).powi(2) + (x.2 - y.2).powi(2)).sqrt()
}

/// O pior caso, em todos os temas, da distância entre cada `a` e cada `b`.
fn pior(a: &[TimelineTinta], b: &[TimelineTinta]) -> f64 {
    let mut p = f64::MAX;
    for t in TEMAS {
        for &x in a {
            for &y in b {
                if x != y {
                    p = p.min(dist(lab_de(x.resolve(t)), lab_de(y.resolve(t))));
                }
            }
        }
    }
    p
}

use ph2d_tokens::TimelineTinta;

/// ⭐ A barra entre duas chaves veste a interpolação da chave de SAÍDA: curva ciano, reta violeta,
/// segura tracejada — as da maquete B1.
#[test]
fn a_barra_veste_a_interpolacao_da_chave_de_saida() {
    assert_eq!(cor_da_barra(Interp::Hold), TimelineTinta::Ponto);
    assert_eq!(cor_da_barra(Interp::Linear), TimelineTinta::BarraReta);
    assert_eq!(
        cor_da_barra(Interp::Bezier {
            x1: 0.4,
            y1: 0.0,
            x2: 0.6,
            y2: 1.0
        }),
        TimelineTinta::BarraCurva
    );
}

/// ⭐ **As três barras leem-se diferentes em todo tema** (as cores da maquete B1, onda 2g). Medido:
/// o pior par em todos os temas é 0,107; a barra fica em 0,10, a de antes.
#[test]
fn as_tres_barras_leem_se_diferentes_nos_oito_temas() {
    let barras = [
        TimelineTinta::BarraCurva,
        TimelineTinta::BarraReta,
        TimelineTinta::Ponto,
    ];
    let p = pior(&barras, &barras);
    eprintln!("pior distancia entre barras: {p:.3}");
    assert!(p >= 0.10, "duas barras a {p:.3}");
}

/// ⭐ Dono 09/11: a linha do OBJECTO tem cara própria — na coluna e na fita, um degrau acima das
/// propriedades em todo tema (os tons da maquete B1); ícone do tipo vindo da Hierarquia;
/// propriedades com o ícone do gizmo. O caminho clássico guarda o `fundo_da_linha` de antes.
#[test]
fn o_cabecalho_do_objeto_tem_fundo_e_icones_proprios() {
    use TimelineTinta as T;
    use ph2d_editor_core::icons::IconId;
    assert_eq!(
        tinta_da_linha(Linha::Objeto(9)),
        (T::NomesCabecalho, T::FitaCabecalho)
    );
    assert_eq!(
        tinta_da_linha(Linha::Faixa { i: 0, grupo: None }),
        (T::NomesPropriedade, T::FitaPropriedade)
    );
    let p = pior(&[T::NomesCabecalho], &[T::NomesPropriedade])
        .min(pior(&[T::FitaCabecalho], &[T::FitaPropriedade]));
    eprintln!("cabecalho x propriedade: {p:.3}");
    // A maquete B1 separa-os por 0,015 (o degrau dela; o negrito e o ícone fazem o resto).
    assert!(p >= 0.014, "cabeçalho a {p:.3} das propriedades");
    for n in [0, 1] {
        assert_eq!(
            fundo_da_linha(Linha::Objeto(9), n),
            Some(ColorToken::TimelineObjectRow)
        );
    }
    assert_eq!(
        icone_da_propriedade(PropKind::TranslationX),
        Some(IconId::Move)
    );
    assert_eq!(
        icone_da_propriedade(PropKind::Rotation),
        Some(IconId::Rotate)
    );
    assert_eq!(icone_da_propriedade(PropKind::ScaleY), Some(IconId::Scale));
    assert_eq!(
        crate::hier::icone_do_objeto(424_242),
        IconId::Cube,
        "sem declaração: objecto vazio"
    );
    crate::hier::publica_icone(424_242, IconId::Sprite);
    assert_eq!(crate::hier::icone_do_objeto(424_242), IconId::Sprite);
}

/// ⭐ Dono 09/11: o Summary tem cor PRÓPRIA — forma C (célula do nome em destaque, fita tingida), cores
/// da maquete B1 (o azul do chip escolhido): célula e fita longe de todo fundo vizinho em todo tema,
/// o texto e o losango âmbar legíveis por cima. Números medidos no handoff (2g).
#[test]
fn o_summary_tem_cor_propria_nos_oito_temas() {
    use TimelineTinta as T;
    let summary = [T::SummaryCelula, T::SummaryFita];
    // A célula encosta na coluna de nomes; a fita, na fita (cabeçalho, propriedades, régua).
    let fundo = pior(
        &[T::SummaryCelula],
        &[T::NomesCabecalho, T::NomesPropriedade],
    )
    .min(pior(
        &[T::SummaryFita],
        &[T::FitaCabecalho, T::FitaPropriedade, T::FitaRegua],
    ));
    let mut leg = f64::MAX;
    for t in TEMAS {
        for s in summary {
            for c in [
                T::TextoCabecalho.resolve(t),
                ColorToken::TimelineSummaryKey.resolve(t),
            ] {
                leg = leg.min(dist(lab_de(s.resolve(t)), lab_de(c)));
            }
        }
    }
    eprintln!("summary: fundo {fundo:.3} legivel {leg:.3}");
    assert!(
        fundo >= BARRA_SUMMARY,
        "o Summary a {fundo:.3} de um fundo vizinho"
    );
    assert!(
        leg >= LEGIVEL_SUMMARY,
        "texto/losango a {leg:.3} do fundo do Summary"
    );
}

/// Medido (onda 2g, cores da maquete): fundo 0,045 · texto/losango 0,249; as barras logo abaixo.
const BARRA_SUMMARY: f64 = 0.044;
const LEGIVEL_SUMMARY: f64 = 0.24;

/// ⭐ Dono 09/11 («os nomes não estão centrados»): em toda linha da coluna (Summary, cabeçalho do
/// objecto, grupo, propriedade e o valor), o meio das maiúsculas que o pintor desenha fica a ≤ 1 px
/// do meio da linha — onde estão o losango, a barra, o ícone e o ▶/▾. Reconstrói a conta do pintor
/// (`ph2d_text`: o tamanho desenhado e o meio das maiúsculas). *Mutação: a conta `(linha − letra)/2`
/// de antes ⇒ ~2,5 px ⇒ vermelho.*
#[test]
fn o_texto_da_coluna_fica_no_meio_da_linha() {
    use ph2d_tokens::TypeToken;
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let y = 300.0_f32;
    let meio = y + ROW_H_PX * 0.5;
    let losango = meio; // `paint_lane_keys`, `paint_grupo`, `paint_objeto`: cy = y + ROW_H_PX/2
    let icone = y + (ROW_H_PX - ICONE_PX) * 0.5 + ICONE_PX * 0.5;
    for font in [TypeToken::Sm.px(), TypeToken::Xs.px()] {
        let topo = y_do_texto(y, font) - (ph2d_text::displayed_font_px(font) - font) * 0.5;
        let maiusculas = topo + ph2d_text::cap_centre_px(font);
        for (alvo, o_que) in [(meio, "linha"), (losango, "losango"), (icone, "ícone")] {
            assert!(
                (maiusculas - alvo).abs() <= 1.0,
                "letra {font}: texto a {:.2} px do meio do(a) {o_que}",
                maiusculas - alvo
            );
        }
    }
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ Dono 09/11 («uma outline como tem na referência»): a moldura da área das trilhas existe com o
/// Auto Key desligado e ligado, muda de tinta com ele (neutra → vermelha) em todo tema, lê-se contra
/// a folha, e cabe inteira na área: não invade a barra de rolagem nem passa da última linha.
#[test]
fn a_moldura_da_fita_existe_e_acende_com_o_auto_key() {
    use TimelineTinta as T;
    let g = geom::resolve(Rect::new(0.0, 0.0, 1200.0, 400.0), 60.0, 180.0, 56.0);
    let (r0, t0) = moldura(&g, false);
    let (r1, t1) = moldura(&g, true);
    assert_eq!((t0, t1), (T::Moldura, T::Gravando));
    assert_eq!(
        r0, r1,
        "a moldura é a mesma nos dois estados; só a tinta muda"
    );
    let meia = ph2d_tokens::TIMELINE_FRAME_PX * 0.5;
    assert!(r0.w > 0.0 && r0.h > 0.0, "a moldura existe");
    assert!(
        r0.x + r0.w + meia <= g.scrollbar.x + 0.01,
        "não cobre a barra de rolagem"
    );
    assert!(
        r0.y + r0.h + meia <= g.rows.y + g.rows.h + 0.01,
        "não passa da última linha"
    );
    assert!(r0.x - meia >= g.region.x - 0.01 && r0.y - meia >= g.region.y - 0.01);
    let mudou = pior(&[T::Moldura], &[T::Gravando]);
    let le_se = pior(
        &[T::Moldura],
        &[T::FitaPropriedade, T::NomesPropriedade, T::FitaRegua],
    );
    eprintln!("moldura: muda {mudou:.3} · contra a folha {le_se:.3}");
    assert!(mudou >= 0.10, "o Auto Key muda a moldura só {mudou:.3}");
    assert!(le_se >= 0.05, "a moldura some na folha ({le_se:.3})");
}

/// ⭐ Doc 15 onda 2i (dono 09/11: «as abas são a navegação principal»): em todo tema, a pílula e o
/// texto da aba ativa destacam-se do fundo da barra MAIS do que a caixa do Auto Key desligada (a de
/// antes, `CardField`, e a de agora, sem fundo) e o texto dela.
#[test]
fn a_aba_ativa_manda_mais_que_a_caixa_do_auto_key() {
    let lab = |c: ph2d_vector::Color| {
        let [r, g, b, _] = c.to_rgba8().to_u8_array();
        lab_de(ph2d_tokens::Color {
            r,
            g,
            b,
            a: u8::MAX,
        })
    };
    for t in TEMAS {
        let fundo = lab(resolve(ColorToken::KanbanList, t));
        let tons = crate::transport::cartoes::tons_da_barra(t);
        let aba = dist(lab(tons.ativa_fundo), fundo);
        let caixa_antes = dist(lab(resolve(ColorToken::CardField, t)), fundo);
        assert!(
            aba > caixa_antes && aba > 0.0,
            "{t:?}: aba {aba:.3} × caixa {caixa_antes:.3}"
        );
        let txt_aba = dist(lab(tons.ativa_texto), fundo);
        let txt_caixa = dist(lab(tons.discreta_texto), fundo);
        assert!(
            txt_aba > txt_caixa,
            "{t:?}: texto da aba {txt_aba:.3} × da caixa {txt_caixa:.3}"
        );
    }
}
