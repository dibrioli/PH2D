use super::*;
use ph2d_editor_core::ids;
use std::collections::BTreeSet;

/// B35.7: todo knob liga/desliga visível (`0..1`, degrau `1`) que não é uma ESCOLHA é uma caixa de marcar.
#[test]
fn interruptor_nao_e_slider() {
    for r in rows() {
        let liga_desliga = r.min == 0.0 && r.max == 1.0 && r.step == 1.0 && r.options.is_none();
        assert_eq!(r.interruptor(), liga_desliga, "{}", r.key);
    }
    for k in INTERRUPTORES {
        assert!(rows().iter().any(|r| r.key == k), "{k} não é linha visível");
    }
}

/// B35.5: o ↺ da linha (o `default` da linha) é a fábrica da ferramenta, nunca o boot do motor (Edge 50 × 12).
#[test]
fn a_linha_do_tuning_mostra_a_fabrica() {
    for r in rows() {
        let f = ph2d_tool_painter::WetKnobs::fabrica(KNOB_DEFS[r.knob].knob);
        assert_eq!(r.default.to_bits(), f.to_bits(), "{}", r.key);
    }
}

/// Cada unidade nomeia uma linha VISÍVEL — uma chave com erro de grafia deixava o número sem unidade.
#[test]
fn cada_unidade_nomeia_uma_linha_visivel() {
    for (k, u) in UNIDADES {
        let r = rows()
            .iter()
            .find(|r| r.key == k)
            .expect("a chave é de uma linha visível");
        assert_eq!(r.unit, Some(u));
    }
}

/// The dynamic id family cannot collide — across faces, across knobs,
/// and against the section's static ids (the hand table in
/// `node_id_collisions` cannot see runtime-derived ids, so the family
/// proves itself here, over the REAL keys).
#[test]
fn wet_tuning_ids_dont_collide() {
    let mut seen: BTreeSet<u64> = BTreeSet::new();
    let mut put = |id: NodeId, what: &str| {
        assert!(seen.insert(id.0), "NodeId collision at {what} ({id:?})");
    };
    for r in rows() {
        put(r.slider, r.key);
        put(r.chip, r.key);
        put(r.reset, r.key);
        for (_, o) in r.options.iter().flatten() {
            put(*o, r.key);
        }
    }
    for s in SECTIONS {
        put(s.header, s.label);
        put(s.reset, s.label);
    }
    for id in [
        ids::WET_TUNING_PANEL,
        ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[4],
        ph2d_tool_painter::ids::WET_TUNING_KM_GLAZE,
        ph2d_tool_painter::ids::WET_TUNING_DRAG_HANDLE,
        ph2d_tool_painter::ids::WET_TUNING_RESIZE_HANDLE,
        ph2d_tool_painter::ids::WET_TUNING_RESIZE_HANDLE_BL,
        ph2d_tool_painter::ids::PAINTER_WETPAINT_TUNING,
    ] {
        put(id, "static chrome");
    }
}

/// 44 visible knobs (25 Paint — the 12 of the pen in the brush (W6, doc 47) + incl. Staining, the brush Glaze of prompt 42 (o Dust do prompt 43 foi para o cartão Wet Paint a 19/10), the brush
/// Pressure size/opacity of prompt 44 and its opacity curve of prompt 45, promoted out of Hidden
/// by doc 23; as 4 cerdas saíram para o Shape a 13/10, plano 27 W2 — + 7 Water + 6 Physics (Leveling, Level Clamp, Capillary e Brake foram para o cartão Wet Paint a 13/10) + 5 Tools — incl. the doc-23
/// Rewet lift — + 1 Relief) — the registry's visible surface, whole. O PAPER (6) saiu
/// na W1e (F2); do grupo dele só o relevo da tinta (Emboss) voltou, a 11/10. O Spacing saiu
/// para o Hidden a 11/10 (o do pincel é o do Stroke).
/// Mais 40 avançados a 25/10 (dono: os escondidos do Wet Paint vão para o Tuning): 19 Bristles · 7 Stroke
/// · 7 Wet effects · 5 Look, e o WaterPush/DryerEdge no Tools (5 → 7). Menos 5 no mesmo dia (dono, R3: o que está
/// no cartão Wet Paint não se repete aqui): Backruns, Granulation, Pigment diffusion, Wet bloom (Wet effects
/// 7 → 3) e Tip pickup (Stroke 7 → 6). Menos 4 a 25/10 (a linha da ferramenta da água no cartão): Blend force
/// (Paint 25 → 24), Dryer, Blow e Smear (Tools 7 → 4). Mais 11 a 27/10 (Óleo Real W2 E3): o rebordo da faca no
/// Stroke (6 → 7) e os 10 do Material da tinta (o Principled por célula e o Lock). Menos 1 a
/// 27/10 (W3): o Material são os 8 do Principled simplificado e o Lock (10 → 9). Mais 2 a 27/10 (W3 K2): a textura
/// que acompanha o tamanho do pincel (expoente e diâmetro de referência) no Stroke (7 → 9). Mais 1 a 27/10 (W3 K3a):
/// a faca que nivela (`knifeLevel`) no Stroke (9 → 10). Mais 1 a 27/10 (W3 K5): o expoente da textura através
/// do traço (`textureSizeExpAcross`) no Stroke (10 → 11). Mais 1 a 27/10 (dono): a Transparency (`1 − Alpha`)
/// no Material (9 → 10). Mais 2 (W3b): a Transmission e o Relevo do papel (10 → 12). Menos 5 a 27/10 (W3b item 3):
/// o grupo Look inteiro (controlos mortos: só o `render_region`, sem chamador no produto, os lia). Mais 1 a 28/10
/// (dono): o modo da Transmission, um seletor (12 → 13). Mais 2 a 28/10 (dono): o Depth e o Gloss do Óleo Real,
/// vindos do cartão Real Oil (13 → 15). Mais 2 a 28/10 (J6 aguarela): o domo do filme no Stroke (11 → 12) e a
/// borda da mancha molhada no Wet effects (3 → 4). Mais 1 (J6): as tampas do filme no Stroke (12 → 13). Mais 1 (J6 mistura): o
/// nível do secador no Tools (+1). Mais 1 (J6 mistura, lacuna 6): o alcance do misturador no Tools (+1). Mais 1 (J6 guache, lacuna 7): a tinta que cobre a molhada no
/// Wet effects (4 → 5). Mais 1 (J6 cerdas): os fios que crescem com o pincel no Bristles (19 → 20).
/// Mais 1 a 28/10 (dono): o Paper Press no Material (15 → 16). Mais 1 a 31/10: a borda graduada no Water (7 → 8). Contagem refeita do código a 31/10: 98 → 95 (o cartão Wet Paint levou Staining, Glaze, Cover e Wet soften de Paint/Stroke/Wet effects para Hidden a 30/10 sem este teste; Paint 24 → 22, Stroke 13 → 12, Wet effects 5 → 4).
/// Mais 1 a 31/10: o arco dos tufos do leque (`tuftArc`) no Bristles (20 → 21).
/// B35.2: o `gateSaturation` pinta-se na gaveta do solver (Paint 3 → 2, Physics 5 → 6).
/// Menos 19 a 10/11 (B35.1, [`ph2d_tool_painter::ids::NO_INSPECTOR`]): 88 → 69 (Paint 19 → 3, Water 7 → 5, Physics 6 → 5).
/// Menos 1 (B35.9): o Eraser fundiu-se no Erase Strength do cartão (Tools 6 → 5; 69 → 68).
/// Menos 1 (B35.10): o Base evaporation fundiu-se no Dry Speed (Water 5 → 4; 68 → 67).
/// Menos 2 (B35.12): Paint drag e Emboss subiram ao cartão Wet (Paint 2 → 1, Paper/Relief 1 → 0; 67 → 65).
/// Menos 17 (B35.13, dono 07/11): as cerdas moram no cartão Shape (Bristles 17 → 0; 65 → 48).
#[test]
fn every_visible_knob_has_a_row() {
    assert_eq!(rows().len(), 48);
    for g in [
        (KnobGroup::Paint, 1),
        (KnobGroup::Water, 4),
        (KnobGroup::Physics, 6),
        (KnobGroup::Tools, 5),
        (KnobGroup::Paper, 0),
        (KnobGroup::Bristles, 0),
        (KnobGroup::Stroke, 12),
        (KnobGroup::WetEffects, 4),
        (KnobGroup::Material, 16),
    ] {
        let n = rows().iter().filter(|r| r.group == g.0).count();
        assert_eq!(n, g.1, "group {:?} row count", g.0);
    }
}

/// ⭐ B35.1 (R3 do dono, 25/10): um knob com linha no Inspector não tem segunda linha no Tuning — os 15 da caneta
/// são os `Knob` de [`ph2d_tool_painter::LINHAS_DA_MAO`] e os 4 do cartão Wet os que ele escreve.
#[test]
fn nenhum_knob_tem_duas_linhas_no_mesmo_painel() {
    let caneta: BTreeSet<&str> = ph2d_tool_painter::LINHAS_DA_MAO
        .iter()
        .filter_map(|l| match l {
            ph2d_tool_painter::LinhaDaMao::Knob(k) => Some(KNOB_DEFS[*k as usize].key),
            _ => None,
        })
        .collect();
    let cartao_wet = [
        "pickup",
        "evaporation",
        "edgeDarkening",
        "gravity",
        "drag",
        "emboss",
    ];
    let no_inspector: BTreeSet<&str> = caneta.iter().copied().chain(cartao_wet).collect();
    let lista: BTreeSet<&str> = ph2d_tool_painter::ids::NO_INSPECTOR.into_iter().collect();
    assert_eq!(no_inspector, lista, "a lista é a caneta + o cartão Wet");
    for r in rows() {
        assert!(
            !lista.contains(r.key),
            "{}: já tem linha no Inspector",
            r.key
        );
    }
}

/// ⭐ **O que fica FORA do Tuning, e porquê** (25/10) — o censo CONTA o grupo `Hidden` do motor contra
/// esta lista: um knob novo nasce aqui (e alguém decide onde mora) ou num grupo à vista.
#[test]
fn os_escondidos_sao_so_os_de_outra_superficie_ou_de_calibracao() {
    const NOUTRA_SUPERFICIE: [&str; 44] = [
        // fundidos noutro controlo do cartão Wet: Eraser no Erase Strength (B35.9), Base evaporation no Dry Speed (B35.10)
        "eraser",
        "baseEvaporation",
        // a Shape › Bristles (plano 27 W2)
        "felt",
        "bristleCount",
        "bristleStrength",
        "bristleSize",
        // a Shape › Bristle Combing (dono 07/11)
        "bristleLaneWidth",
        "bristleHeight",
        "bristleTipRagged",
        "bristleLaneSizeExp",
        // o Spacing do Stroke manda (dono 11/10)
        "spacing",
        // o cartão Wet Paint do Painter (dono 13/10 e 19/10)
        "leveling",
        "capillary",
        "brake",
        "levelClamp",
        "dust",
        // os do pincel que mais decidem o preset, só no cartão (dono 25/10, R2 e R3)
        "extBackrun",
        "extGranulation",
        "extDiffusion",
        "wetOpen",
        "tipReservoir",
        // a linha da ferramenta da água na mão, no cartão Wet Paint (dono 25/10, o painel do Rebelle)
        "blendForce",
        "dryer",
        "blow",
        "smear",
        "smearWater",
        // a linha do Wet (31/10): o Drag
        "waterDrag",
        // o cartão Wet Paint do pincel (ordem do dono 30/10): Staining, Glaze, Cover, Wet soften e a Drag (J10)
        "extStaining",
        "glaze",
        "wetCover",
        // e o Dry cover ao lado do Cover (31/10)
        "dryCover",
        "extWetSoften",
        "rewetPaint",
        "solubility",
        "rewetDrag",
        // os controlos de lei da E10 (F3 §9): os volumes do topo e o cartão Wet Paint
        "medium",
        "dilution",
        "volume",
        "body",
        "smoothEdges",
        // e os fundidos neles: a mesma grandeza que um controlo acima já escreve (o Hardness escreve o paperGate)
        "pigmentPerDab",
        "waterPerDab",
        "intensity",
        "paperGate",
    ];
    const CALIBRACAO: [&str; 9] = [
        // normaliza o pigmento pelo comprimento (o preset diz a que Spacing foi afinado)
        "pigmentSpacing",
        // passo, tecto e arredondamento do sub-dab da cápsula: amostragem e custo
        "capsuleSpacing",
        "capsuleMaxPx",
        "capsuleRoundNearest",
        // leis de ESCALA com o diâmetro, afinadas contra o Rebelle
        "embossSizeExp",
        "embossSizeRef",
        "thinStrokeFade",
        "thinStrokeRef",
        // a tinta opaca (31/10): o ponto no segmento das duas tintas, declarado pelo preset de fábrica
        "opaque",
    ];
    let hidden: BTreeSet<&str> = KNOB_DEFS
        .iter()
        .filter(|d| d.group == KnobGroup::Hidden)
        .map(|d| d.key)
        .collect();
    let lista: BTreeSet<&str> = NOUTRA_SUPERFICIE
        .iter()
        .chain(CALIBRACAO.iter())
        .copied()
        .collect();
    assert_eq!(hidden, lista);
}

/// ⭐ U5 (auditoria W3b): o grupo Material mostra os 8 do Principled, os três PESOS da tinta juntos (Transparency ·
/// Transmission · Paper Relief), o modo da Transmission logo abaixo dela (dono 28/10) e o «Lock material» no FIM;
/// o Depth e o Gloss do Óleo Real (dono 28/10, vindos do cartão) à cabeça.
#[test]
fn a_ordem_do_material_junta_os_pesos_e_o_lock_no_fim() {
    let m: Vec<&str> = rows()
        .iter()
        .filter(|r| r.group == KnobGroup::Material)
        .map(|r| r.key)
        .collect();
    assert_eq!(
        m,
        [
            "oilDepth",
            "oilGloss",
            "metallic",
            "roughness",
            "iorLevel",
            "coat",
            "coatRoughness",
            "sheen",
            "subsurface",
            "subsurfaceRadius",
            "transparency",
            "transmission",
            "transmissionMode",
            "paperRelief",
            "paperPress",
            "lockMaterial",
        ]
    );
    let escolhas: Vec<&str> = rows()
        .iter()
        .filter(|r| r.options.is_some())
        .map(|r| r.key)
        .collect();
    assert_eq!(
        escolhas.len(),
        CHOICES.len(),
        "os seletores são os de CHOICES"
    );
    assert!(CHOICES.iter().all(|k| escolhas.contains(k)));
    for k in CHOICES {
        let d = KNOB_DEFS.iter().find(|d| d.key == k).expect("knob");
        assert_eq!(
            (d.min, d.max, d.step),
            (0.0, 1.0, 1.0),
            "{k}: um seletor de 2 é 0..1 de passo 1"
        );
    }
}

/// Cada secção repõe o grupo que pinta — a tabela da ferramenta e a do painel são a mesma.
#[test]
fn o_repor_de_cada_seccao_repoe_o_grupo_dela() {
    let resets = ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS;
    // Os repores do Relief (B35.12) e do Bristles (B35.13), secções que saíram, ficam nos ids: as macros gravaram-nos.
    assert_eq!(SECTIONS.len() + 2, resets.len());
    for s in SECTIONS {
        let i = resets.iter().position(|r| *r == s.reset).expect("reset");
        assert_eq!(ph2d_tool_painter::ids::WET_TUNING_RESET_GROUPS[i], s.group);
    }
}

/// ⭐ B35.13 (dono 07/11: «tudo o que importa das cerdas mora no Shape»): nenhuma cerda tem linha no Tuning.
#[test]
fn todas_as_cerdas_moram_no_shape() {
    let no_tuning: Vec<&str> = rows()
        .iter()
        .filter(|r| r.group == KnobGroup::Bristles || r.key.starts_with("bristle"))
        .map(|r| r.key)
        .collect();
    assert!(no_tuning.is_empty(), "cerdas no Tuning: {no_tuning:?}");
    let do_motor: BTreeSet<&str> = KNOB_DEFS
        .iter()
        .filter(|d| d.group == KnobGroup::Bristles)
        .map(|d| d.key)
        .collect();
    let no_shape: BTreeSet<&str> = ph2d_tool_painter::ids::CERDAS_NO_SHAPE
        .into_iter()
        .collect();
    assert_eq!(do_motor, no_shape, "toda cerda do motor mora no Shape");
    // A regra do pai (B35.3) mudou-se com elas: sem a ponta Bristles nenhuma; filha só com o pai > 0.
    let mut b = ph2d_tool_painter::PainterTool::default().brush_settings();
    b.shape_kind = ph2d_tool_painter::TextureKind::None.to_u8();
    assert!(b.wet_knobs.cerdas(&b).is_empty());
    b.shape_kind = ph2d_tool_painter::TextureKind::Bristles.to_u8();
    for (filha, pai) in ph2d_tool_painter::ids::PAIS_DAS_CERDAS {
        let i = KNOB_DEFS.iter().position(|d| d.key == pai).expect(pai);
        let a_mostra = |b: &ph2d_tool_painter::BrushSettings| {
            b.wet_knobs.cerdas(b).iter().any(|l| l.chave == filha)
        };
        b.wet_knobs.knobs[i] = 0.0;
        assert!(!a_mostra(&b), "{filha}: {pai} a zero => escondida");
        b.wet_knobs.knobs[i] = KNOB_DEFS[i].max.min(1.0);
        assert!(a_mostra(&b), "{filha}: {pai} ligado => à vista");
    }
}
