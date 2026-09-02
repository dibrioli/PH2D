//! ⭐⭐⭐ **O PORTÃO QUE CONCORDA COM O OLHO DO DONO** — as réguas da ponta corridas sobre
//! os DOIS lados que ele julgou: a retopologia que **aprovou** e as saídas que **reprovou**.
//!
//! # ⛔⛔⛔ Por que este portão existe (handoff de 2026-09-01, §0)
//!
//! Uma jornada inteira entregou três curas cujas réguas diziam *«melhorou muito»* e o dono
//! disse *«absolutamente nenhuma melhoria»* — quatro vezes num dia, com foto. ⚠️ **Nenhuma
//! régua desta linha tinha alguma vez sido corrida sobre a malha que ele aprovou.** Corridas
//! em 2026-09-02, duas coisas apareceram:
//!
//! 1. **O piso do ápice (`0,55` do raio) escondia as pontas da foto** — as três de que ele
//!    se queixava estão a `0,43`–`0,47` do raio, e não eram medidas por régua nenhuma.
//! 2. **A barra da grade (`1,5`) foi calibrada só com a nossa saída** — a aprovada entrega
//!    `≤ 0,79` em todas as pontas, e as reprovadas `1,10`–`5,41`; a barra deixava passar
//!    exactamente o que ele via.
//!
//! ⇒ *Uma régua candidata tem de separar `Sculpt_Blender.obj` das nossas. Se não separa, não
//! é a régua* — e este ficheiro é essa frase em forma executável.
//!
//! # As fixturas (`tests/fixtures/pontas/`, proveniência no `README.md` ao lado)
//!
//! | ficheiro | o que é | veredito do dono |
//! |---|---|---|
//! | `sculpt_antes.obj` | a escultura de entrada nº 1 | — |
//! | `Sculpt_Blender.obj` | a retopologia dela pelo QRemeshify (Blender) | ✅ *«preserva as pontas»* (29/08) |
//! | `_base_sculpt.obj` | a escultura de entrada nº 2 | — |
//! | `_remesh_sculpt.obj` | a nossa saída, `Detail 0,75` (31/08) | ⛔ *«amputa uma ponta»* |
//! | `sculpt_Depois.obj` | a nossa saída (01/09, 16:56) | ⛔ *«não é bom»* (foto) |
//!
//! ⚠️ **As duas saídas nossas estão no referencial da ENTRADA** — o importador recentra a peça
//! e o exportador assa a pose (`(p − âncora) / escala + centro da caixa`); a transformação
//! está no `README.md`. *Comparar no referencial exportado é o erro que a jornada anterior
//! pagou quatro vezes.*
//!
//! ⚠️ **As duas entradas são peças DIFERENTES**, de propósito: uma régua normalizada pela
//! própria malha (unidade = aresta mediana da saída) tem de concordar com o olho em qualquer
//! peça, e um portão sobre uma peça só provaria uma constante.

use std::path::{Path, PathBuf};

use ph2d_mesh::Mesh;
use ph2d_quadfill::{
    CONE_MAX, TIP_DENSITY_MAX, TIP_GAP_MAX, TipDensity, TipDeviation, apices, median_edge,
    tip_density, tip_deviation,
};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pontas")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore — a gémea da que vive em
/// `ph2d-quadextract/tests/support`, pela mesma razão (o cabeçalho do gzip é de tamanho
/// variável e o rabo são oito bytes que o inflate não quer ver).
fn gunzip(raw: &[u8]) -> Vec<u8> {
    assert!(raw.len() > 18, "ficheiro curto demais para ser gzip");
    assert_eq!(&raw[..2], &[0x1f, 0x8b], "nao e' gzip");
    let flg = raw[3];
    let mut off = 10usize;
    if flg & 0b0000_0100 != 0 {
        let n = usize::from(u16::from_le_bytes([raw[off], raw[off + 1]]));
        off += 2 + n;
    }
    for bit in [0b0000_1000u8, 0b0001_0000] {
        if flg & bit != 0 {
            while raw[off] != 0 {
                off += 1;
            }
            off += 1;
        }
    }
    if flg & 0b0000_0010 != 0 {
        off += 2;
    }
    miniz_oxide::inflate::decompress_to_vec(&raw[off..raw.len() - 8]).expect("a fixtura nao inflou")
}

fn load(name: &str) -> Mesh {
    let path = fixture_dir().join(format!("{name}.obj.gz"));
    let raw = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let text = String::from_utf8(gunzip(&raw)).expect("a fixtura nao e' UTF-8");
    ph2d_mesh::import_obj(&text)
        .unwrap_or_else(|e| panic!("{name}: {e:?}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{name}: sem peca"))
        .mesh
}

/// As duas réguas, na unidade do produto (a aresta mediana da SAÍDA).
fn mede(tag: &str, input: &Mesh, output: &Mesh) -> (TipDeviation, TipDensity) {
    let unit = median_edge(output);
    let clock = std::time::Instant::now();
    let dev = tip_deviation(input, output, unit);
    let den = tip_density(input, output, unit);
    eprintln!(
        "{tag}: {} v entrada, {} quads saida, h = {unit:.4} | espinhos {} | AMPUTADAS {} \
         (pior gap {:.2} h, barra {TIP_GAP_MAX}) | GRADE pior {:.2} p50 {:.2} ({} acima de \
         {TIP_DENSITY_MAX}) | {} ms",
        input.vert_count(),
        output.face_count(),
        dev.tips,
        dev.cut,
        dev.apex_max,
        den.worst,
        den.p50,
        den.over,
        clock.elapsed().as_millis(),
    );
    assert_eq!(dev.tips, den.tips, "as duas reguas medem as MESMAS pontas");
    assert!(
        dev.tips > 0,
        "{tag}: nenhuma ponta medida -- «nao medido» nao e' «perfeito»"
    );
    (dev, den)
}

/// ⭐⭐⭐ **A retopologia que ele APROVOU passa nas duas barras, em TODAS as pontas.**
///
/// ⛔ É a metade que faltava a toda régua desta linha: sem ela, uma barra só mede a distância
/// entre os nossos próprios defeitos.
#[test]
fn a_retopologia_que_o_dono_aprovou_passa_em_todas_as_pontas() {
    let (dev, den) = mede("APROVADA", &load("sculpt_antes"), &load("Sculpt_Blender"));
    assert!(
        dev.tips >= 4,
        "a peca tem pelo menos quatro espinhos: {dev:?}"
    );
    assert_eq!(dev.cut, 0, "⛔ a aprovada nao tem ponta amputada: {dev:?}");
    assert_eq!(
        den.over, 0,
        "⛔ a aprovada nao tem grade grossa no bico: {den:?}"
    );
}

/// ⭐⭐⭐ **Cada saída que ele REPROVOU falha pelo menos uma ponta — e a da foto falha as
/// DUAS réguas.**
///
/// `sculpt_Depois.obj`: a ponta mais longa comida em `10` células e a agulha `15909` a `1,1`
/// (⛔ com `p50 0,84`, abaixo da barra da mediana — é por isso que o ápice se mede sozinho);
/// e cinco espinhos com a grade a engrossar para o bico (`1,40` a `4,50`).
#[test]
fn as_saidas_que_o_dono_reprovou_falham_pelo_menos_uma_ponta_cada() {
    let base = load("_base_sculpt");
    let (dev, den) = mede("REPROVADA 01/09", &base, &load("sculpt_Depois"));
    assert!(
        dev.cut >= 2,
        "⛔ a saida da foto tem DUAS pontas amputadas: {dev:?}"
    );
    assert!(den.over >= 2, "⛔ e a grade a engrossar em varias: {den:?}");
    let (dev, den) = mede("REPROVADA 31/08", &base, &load("_remesh_sculpt"));
    assert!(dev.cut >= 1, "⛔ «amputa uma ponta» (Enio, 31/08): {dev:?}");
    assert!(den.over >= 1, "⛔ e a grade termina antes do bico: {den:?}");
}

/// ⭐⭐ **As duas barras vivem num VAZIO, não em cima de um ponto medido.**
///
/// ⚠️ Uma barra colada ao pior valor aprovado (ou ao melhor reprovado) mudaria de veredito
/// com o ruído da próxima peça. Este gate exige margem para os dois lados — e é o gate que
/// morde se alguém «afinar» a barra para fazer uma candidata passar.
#[test]
fn as_barras_vivem_no_vazio_entre_o_aprovado_e_o_reprovado() {
    let (dev_ok, den_ok) = mede("APROVADA", &load("sculpt_antes"), &load("Sculpt_Blender"));
    let (dev_ko, den_ko) = mede("REPROVADA", &load("_base_sculpt"), &load("sculpt_Depois"));
    assert!(
        dev_ok.apex_max <= TIP_GAP_MAX * 0.6,
        "o pior gap aprovado ({:.2}) tem de ficar bem abaixo da barra {TIP_GAP_MAX}",
        dev_ok.apex_max
    );
    assert!(
        dev_ko.apex_max >= TIP_GAP_MAX * 2.0,
        "o pior gap reprovado ({:.2}) tem de ficar bem acima da barra {TIP_GAP_MAX}",
        dev_ko.apex_max
    );
    assert!(
        den_ok.worst <= TIP_DENSITY_MAX - 0.1,
        "a pior grade aprovada ({:.2}) tem de ficar abaixo da barra {TIP_DENSITY_MAX} com margem",
        den_ok.worst
    );
    assert!(
        den_ko.worst >= TIP_DENSITY_MAX + 0.1,
        "a pior grade reprovada ({:.2}) tem de ficar acima da barra {TIP_DENSITY_MAX} com margem",
        den_ko.worst
    );
}

/// ⭐⭐⭐ **O piso antigo (`0,55` do raio) NÃO VIA as pontas da foto** — e o novo vê-as sem
/// chamar «ponta» às bossas.
///
/// Na escultura do dono há `42` máximos locais de raio acima de `0,25`; os espinhos AFIADOS
/// são `5` (`9663` · `12074` · `15909` · `3138` · `10230`), as cúpulas (`1463` · `1943` ·
/// `15341`) entram só na resolução em que são cónicas, e os outros `34` são bossas do corpo
/// com cone `≥ 1,21`.
#[test]
fn o_piso_antigo_nao_via_as_pontas_da_foto_e_o_novo_nao_ve_bossas() {
    let base = load("_base_sculpt");
    let out = load("sculpt_Depois");
    let unit = median_edge(&out);
    let (mid, apex) = apices(&base, unit);
    let pos = base.positions();
    let r = |i: usize| {
        let p = pos[i];
        let d = [p[0] - mid[0], p[1] - mid[1], p[2] - mid[2]];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
    };
    let far = pos
        .iter()
        .enumerate()
        .map(|(i, _)| r(i))
        .fold(0.0f32, f32::max);
    let curtos = apex.iter().filter(|&&i| r(i) < 0.55 * far).count();
    eprintln!(
        "espinhos {} (cone <= {CONE_MAX}), dos quais {curtos} abaixo do piso antigo de 0,55: {:?}",
        apex.len(),
        apex.iter()
            .map(|&i| format!("{i}@{:.2}", r(i) / far))
            .collect::<Vec<_>>()
    );
    assert!(
        (4..=12).contains(&apex.len()),
        "a peca tem 5 espinhos afiados (mais as cupulas conforme a resolucao) e 42 maximos \
         locais; a lei devolveu {}",
        apex.len()
    );
    assert!(
        curtos >= 2,
        "⛔ as pontas da foto (`3138` a 0,47 e `10230` a 0,51 do raio) tem de entrar: {curtos}"
    );
    // ⚠️ **Por RAIO e não por índice:** o importador renumera os vértices (o `3138` do
    // ficheiro é o `15622` da malha), e um índice do `.obj` num assert mediria o importador.
    assert!(
        apex.iter().any(|&i| (r(i) / far - 0.47).abs() < 0.02),
        "⛔ o espinho da foto (a 0,47 do raio, cone 0,63) tem de estar no censo"
    );
    assert!(
        apices(&base, 0.0).1.is_empty(),
        "sem unidade nao ha' cone, e sem cone a lista seria de 42 «pontas»"
    );
}
