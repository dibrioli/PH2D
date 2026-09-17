//! ⭐⭐⭐ **TODO NOME QUE O MOTOR PUBLICA PARA ESTE PAINEL TEM CHAVE NA TABELA** — a metade do HR-15
//! que a régua lexical do painel não alcança (o literal mora nas crates do motor).
//!
//! Varre as quatro tabelas que a shell publica (`fase_selection_mirror_*`): os efeitos de caminho e
//! os parâmetros de cada um — incluindo as variantes que um interruptor acorda (o `Roughen` do Zig
//! Zag) —, os filtros raster com os rótulos dos controlos e dos modos, as leis de mistura que um
//! degrau oferece e os presets da gaiola.

use ph2d_panel_vector::nomes_do_motor as n;
use ph2d_tool_vector::shapes::{self, FieldDesc};
use ph2d_vec_scene::effect::PathEffect;

/// Os campos que o CONECTOR publica — ver o controlo derivado no corpo do gate.
const CONECTOR: &[&FieldDesc] = &[
    &ph2d_tool_vector::connector::ROUTE,
    &ph2d_tool_vector::connector::JETTY,
    &ph2d_tool_vector::connector::SPREAD,
    &ph2d_tool_vector::connector::CORNER,
    &ph2d_tool_vector::connector::CURVE,
];

/// Os campos que as PONTAS de traço publicam — idem.
const MARCADOR: &[&FieldDesc] = &[
    &ph2d_tool_vector::params::MARKER_SCALE,
    &ph2d_tool_vector::params::MARKER_ROUND,
    &ph2d_tool_vector::params::BOTH_ENDS,
];

fn faltas(
    familia: &str,
    rotulos: &[&'static str],
    chave: fn(&str) -> Option<&'static str>,
) -> Vec<String> {
    let mut out: Vec<String> = rotulos
        .iter()
        .filter(|r| chave(r).is_none())
        .map(|r| format!("{familia}\t{r}"))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Os rótulos dos efeitos e dos parâmetros — cada tipo, e cada variante que um interruptor acorda.
fn efeitos_e_parametros() -> (Vec<&'static str>, Vec<&'static str>) {
    let (mut efeitos, mut params) = (Vec::new(), Vec::new());
    for kind in 0..PathEffect::KINDS.len() {
        let fx = PathEffect::from_kind(kind).expect("todo índice da tabela constrói um efeito");
        efeitos.push(fx.label());
        for (i, p) in fx.params().iter().enumerate() {
            params.push(p.name);
            if p.toggle {
                let mut ligado = fx.clone();
                ligado.set(i, 1.0);
                efeitos.push(ligado.label());
                params.extend(ligado.params().iter().map(|q| q.name));
            }
        }
    }
    efeitos.extend(PathEffect::KINDS.iter().copied());
    (efeitos, params)
}

#[test]
fn every_name_the_engine_publishes_has_a_key() {
    let (efeitos, params) = efeitos_e_parametros();
    let warps: Vec<&'static str> = ph2d_warp_style::WarpStyle::ALL
        .iter()
        .map(|w| w.label())
        .collect();
    let specs = ph2d_fx_op::FxOp::SPECS;
    let filtros: Vec<&'static str> = specs.iter().map(|s| s.name).collect();
    let mut controlos = Vec::new();
    let mut modos = Vec::new();
    for s in &specs {
        controlos.extend(s.radius_label);
        controlos.extend(s.color_label);
        controlos.extend(s.color_b_label);
        controlos.extend(s.grow_label);
        if let Some((a, b)) = s.offset_labels {
            controlos.extend([a, b]);
        }
        if let Some((a, b, c)) = s.noise_labels {
            controlos.extend([a, b, c]);
        }
        if let Some((a, b, c)) = s.adjust_labels {
            controlos.extend([a, b, c]);
        }
        modos.extend(s.modes.iter().copied());
    }
    let misturas: Vec<&'static str> = (0..ph2d_fx_op::FxOp::BLEND_KINDS)
        .map(|m| ph2d_blend_mode::BlendMode::from_u8(m).name())
        .collect();
    // ⛔ Controlo de vacuidade: as tabelas medidas em 2026-09-16, menos folga para encolher.
    assert!(
        efeitos.len() >= 20 && params.len() >= 20 && filtros.len() >= 10,
        "as tabelas do motor vieram vazias"
    );
    // ⭐⭐⭐ **AS CINCO TABELAS DO CATÁLOGO** (2026-09-17) — o resto do que o motor do vector
    //    publicava cru: as FORMAS e os PARÂMETROS de cada uma (mais os valores de um campo de
    //    escolha), o CONECTOR com as rotas, as MOLDURAS e as PONTAS de traço.
    //
    // ⚠️ **A contagem sai das tabelas, nunca de uma lista escrita aqui** — é o que faz uma forma
    //    nova nascer vermelha em vez de nascer muda.
    let formas: Vec<&'static str> = shapes::SHAPES.iter().map(|d| d.label).collect();
    let mut campos: Vec<&'static str> = Vec::new();
    for d in shapes::SHAPES {
        for f in d.fields {
            campos.push(f.label);
            if let shapes::FieldUnit::Choice(opcoes) = f.unit {
                campos.extend(opcoes.iter().copied());
            }
        }
    }
    let mut conector: Vec<&'static str> = CONECTOR.iter().map(|f| f.label).collect();
    conector.extend(ph2d_tool_vector::connector::ROUTE_NAMES.iter().copied());
    let molduras: Vec<&'static str> = ph2d_tool_vector::frames::DEVICE_PRESETS
        .iter()
        .map(|p| p.label)
        .collect();
    let mut marcador: Vec<&'static str> = MARCADOR.iter().map(|f| f.label).collect();
    marcador.extend(ph2d_tool_vector::params::BOTH_ENDS_NAMES.iter().copied());
    // ⛔⛔ **As duas listas acima são ESCRITAS À MÃO, e por isso trazem um controlo DERIVADO:** o
    //    motor não publica um `ALL_FIELDS` para elas (as formas publicam o `SHAPES`), e a crate é
    //    de outra linha — acrescentar-lhe uma const seria editar o módulo de outrem. ⇒ o gate CONTA
    //    os `pub const …: FieldDesc` no fonte e exige que a lista os cubra: *uma lista escrita à mão
    //    sem um controlo que a confronte com a fonte é uma lista que envelhece em silêncio.*
    for (ficheiro, lista) in [
        ("connector.rs", CONECTOR.len()),
        ("params.rs", MARCADOR.len()),
    ] {
        let caminho = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates/")
            .join("ph2d-tool-vector/src")
            .join(ficheiro);
        let fonte = std::fs::read_to_string(&caminho).expect("o fonte do motor existe");
        assert_eq!(
            fonte
                .lines()
                .filter(|l| l.trim_start().starts_with("pub const ") && l.contains(": FieldDesc"))
                .count(),
            lista,
            "o `{ficheiro}` do motor declara um número de `FieldDesc` diferente do que a lista \
             deste gate enumera — um campo novo do motor ficaria sem chave e sem acusação"
        );
    }
    let pontas: Vec<&'static str> = ph2d_vec_scene::ALL_MARKERS
        .iter()
        .map(|m| m.label())
        .collect();
    // ⛔ Controlo de vacuidade das cinco novas — medidas em 2026-09-17.
    assert!(
        formas.len() >= 40
            && campos.len() >= 60
            && conector.len() >= 6
            && molduras.len() >= 4
            && marcador.len() >= 4
            && pontas.len() >= 6,
        "uma das tabelas novas do motor veio vazia: formas={} campos={} conector={} molduras={} \
         marcador={} pontas={}",
        formas.len(),
        campos.len(),
        conector.len(),
        molduras.len(),
        marcador.len(),
        pontas.len()
    );

    let mut todas = Vec::new();
    todas.extend(faltas("forma", &formas, n::chave_da_forma));
    todas.extend(faltas("campo", &campos, n::chave_do_campo));
    todas.extend(faltas("conector", &conector, n::chave_do_conector));
    todas.extend(faltas("moldura", &molduras, n::chave_da_moldura));
    todas.extend(faltas("marcador", &marcador, n::chave_do_marcador));
    todas.extend(faltas("ponta", &pontas, n::chave_da_ponta));
    todas.extend(faltas("efeito", &efeitos, n::chave_do_efeito));
    todas.extend(faltas("efeito", &warps, n::chave_do_efeito));
    todas.extend(faltas("parametro", &params, n::chave_do_parametro));
    todas.extend(faltas("filtro", &filtros, n::chave_do_filtro));
    todas.extend(faltas("controlo", &controlos, n::chave_do_controlo));
    todas.extend(faltas("modo", &modos, n::chave_do_modo));
    todas.extend(faltas("mistura", &misturas, n::chave_da_mistura));
    assert!(
        todas.is_empty(),
        "nomes publicados pelo motor sem chave em `nomes_do_motor` (o painel pintá-los-ia crus):\n{}",
        todas.join("\n")
    );
}

/// ⭐ **E todo ponto de ESCRITA passa pela correspondência** — a outra metade.
///
/// ⚠️ Em inglês a tabela diz o mesmo que o motor, então apagar a tradução de um setter não muda um
/// pixel e nenhum gate de glifos o veria. Este lê os quatro sítios por onde os nomes entram no
/// painel (e o da montagem dos chips de modo, que é uma fatia `'static` do motor).
#[test]
fn every_door_the_names_enter_by_translates_them() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ler = |f: &str| {
        let t = std::fs::read_to_string(src.join(f)).unwrap_or_else(|e| panic!("{f}: {e}"));
        // só código: a prosa deste repo cita as funções que explica
        t.lines()
            .map(|l| l.split("//").next().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
    };
    for (ficheiro, chamadas) in [
        ("state_effects.rs", &["n::efeito(", "n::parametro("][..]),
        (
            "state_filters.rs",
            &["n::filtro(", "n::controlo", "nomes_do_motor::mistura"][..],
        ),
        ("state_envelope.rs", &["nomes_do_motor::efeito("][..]),
        ("paint_filters.rs", &["nomes_do_motor::modo("][..]),
        // ⭐ As PORTAS do catálogo (2026-09-17) — cada uma é o único sítio por onde aquela família
        //   entra no painel. ⚠️ Em inglês a tabela diz o mesmo que o motor, logo apagar uma destas
        //   chamadas não muda um pixel e nenhum gate de glifos a veria.
        ("paint_catalog.rs", &["nomes_do_motor::forma("][..]),
        ("paint_modes.rs", &["nomes_do_motor::campo("][..]),
        ("paint_connector.rs", &["nomes_do_motor::conector("][..]),
        ("paint_frame.rs", &["nomes_do_motor::moldura("][..]),
        (
            "paint_markers.rs",
            &["nomes_do_motor::marcador(", "nomes_do_motor::ponta("][..],
        ),
    ] {
        let codigo = ler(ficheiro);
        for c in chamadas {
            assert!(
                codigo.contains(c),
                "`{ficheiro}` deixou de chamar `{c}` — os nomes do motor voltariam a ser pintados crus"
            );
        }
    }
}
