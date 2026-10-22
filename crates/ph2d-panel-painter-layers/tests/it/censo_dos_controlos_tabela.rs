// O censo dos controlos (`censo_dos_controlos.rs`, que o inclui): a tabela meio × controlo e as
// pré-condições que revelam cada controlo escondido.

#[test]
#[ignore = "sonda: a tabela meio × controlo (o entregável (a) da missão de 2026-10-02)"]
fn sonda_a_tabela_dos_controlos() {
    let nomes = nomes();
    let so: Option<String> = std::env::var("CENSO_MEIO").ok();
    let rotulo = |g: &Gesto| -> String {
        let n = nome(&nomes, g.id().unwrap_or(NodeId(0)));
        match g {
            Gesto::Opcao(m, _, k, t) => format!("{}[{k}/{t}]", nome(&nomes, *m)),
            _ => n,
        }
    };
    for media in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        if so.as_ref().is_some_and(|m| *m != format!("{media:?}")) {
            continue;
        }
        let t_meio = std::time::Instant::now();
        let mut coberto: Vec<NodeId> = Vec::new();
        let mut candidatos: Vec<(Vec<Gesto>, Gesto)> = Vec::new();
        let mut fila: std::collections::VecDeque<Vec<Gesto>> = [Vec::new()].into();
        while let Some(armar) = fila.pop_front() {
            let estado = if armar.is_empty() {
                "fábrica".to_owned()
            } else {
                armar.iter().map(&rotulo).collect::<Vec<_>>().join(" + ")
            };
            let t0 = std::time::Instant::now();
            let (base, _) = mede(media, &armar, &Gesto::Nenhum);
            // O controlo do ruído corre-se UMA vez, na fábrica: um ensaio do Wet Paint custa 2 s.
            let de_novo = if armar.is_empty() {
                mede(media, &armar, &Gesto::Nenhum).0
            } else {
                base.clone()
            };
            let ruido = diferenca(&base, &de_novo);
            eprintln!(
                "CENSO\t{media:?}\tCONTROLO\t-\t{}\t{}\t{}\t-\t-\t-\testado={estado}\t({:.2} s por ensaio)",
                ruido.0,
                ruido.1,
                ruido.2,
                t0.elapsed().as_secs_f32() / 2.0
            );
            let mut deste: Vec<Gesto> = Vec::new();
            for g in gestos(media, &armar) {
                let id = g.id().unwrap_or(NodeId(0));
                if !coberto.contains(&id) {
                    coberto.push(id);
                    deste.push(g);
                }
            }
            let medidas = em_paralelo(&deste, |g| {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mede(media, &armar, g)))
                    .ok()
            });
            for (g, medida) in deste.iter().zip(medidas) {
                let Some((px, entrega)) = medida else {
                    eprintln!(
                        "CENSO\t{media:?}\t{}\tPÂNICO\t-\t-\t-\t-\t-\t-\testado={estado}\t{g:?}",
                        rotulo(g)
                    );
                    continue;
                };
                let (n, soma, max) = diferenca(&base, &px);
                eprintln!(
                    "CENSO\t{media:?}\t{}\t{}\t{n}\t{soma}\t{max}\t{}\t{}\t{}\testado={estado}\t{g:?}",
                    rotulo(g),
                    tipo_do_gesto(g),
                    entrega.ao_tool,
                    if entrega.ajuste_mudou {
                        "mudou"
                    } else {
                        "igual"
                    },
                    entrega.outras.join(",")
                );
                if n == 0 && entrega.ao_tool > 0 && entrega.ajuste_mudou {
                    candidatos.push((armar.clone(), g.clone()));
                }
                let novos = entrega.revelados.iter().any(|i| !coberto.contains(i));
                if novos && armar.len() < PROFUNDIDADE && entrega.ao_tool > 0 {
                    let mut mais = armar.clone();
                    mais.push(g.clone());
                    fila.push_back(mais);
                }
            }
        }
        eprintln!("CENSO-COBERTO {media:?}: {} gestos", coberto.len());
        eprintln!(
            "CENSO-TEMPO\t{media:?}\texploração\t{} candidatos\t{:.1} s",
            candidatos.len(),
            t_meio.elapsed().as_secs_f32()
        );
        if std::env::var("CENSO_ARMAR").is_ok() {
            procura_as_pre_condicoes(media, &candidatos, &rotulo);
        }
    }
}

/// Quão longe (px, na vertical) um vizinho pode estar para contar como «do mesmo cartão».
const VIZINHANCA_PX: f32 = 160.0;

/// ⭐ **A PRÉ-CONDIÇÃO PROCURA-SE, não se adivinha.** Para cada candidato a morto (o ajuste mudou e a
/// imagem não), tenta como pré-condição cada gesto VIZINHO do mesmo estado (os controlos do mesmo
/// cartão, do mais perto para o mais longe) e um traço já pintado na tela. O primeiro que o faz mudar
/// a imagem — contra a base armada com o MESMO vizinho — é a pré-condição; nenhum ⇒ `MORTO`.
fn procura_as_pre_condicoes(
    media: PaintMedia,
    candidatos: &[(Vec<Gesto>, Gesto)],
    rotulo: &dyn Fn(&Gesto) -> String,
) {
    let t_procura = std::time::Instant::now();
    let resultados = em_paralelo(candidatos, |(armar, g)| {
        let t0 = std::time::Instant::now();
        let mut feitas = 0usize;
        let mut total = 0usize;
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let bancada = Bancada::nova(media, armar);
            let y_de = |id: NodeId| {
                bancada
                    .rects
                    .iter()
                    .find(|(i, _)| *i == id)
                    .map(|(_, r)| r.y)
            };
            let ancora = match g {
                Gesto::Opcao(menu, ..) => y_de(*menu),
                _ => g.id().and_then(y_de),
            };
            let todos = std::env::var("CENSO_ARMAR").is_ok_and(|v| v == "2");
            let mut vizinhos: Vec<(f32, Gesto)> = gestos(media, armar)
                .into_iter()
                .filter(|v| v.id() != g.id())
                .filter(|v| !matches!((v, g), (Gesto::Opcao(a, ..), Gesto::Opcao(b, ..)) if a == b))
                .filter_map(|v| {
                    let y = match &v {
                        Gesto::Opcao(menu, ..) => y_de(*menu),
                        _ => v.id().and_then(y_de),
                    }?;
                    let d = (y - ancora?).abs();
                    // Longe do cartão, de cada menu só as duas primeiras opções que não são a 0: os
                    // menus de 29 entradas (Shape, Grain, Paper) eram a maior parte do custo.
                    let poucas = !matches!(v, Gesto::Opcao(_, _, k, _) if k > 2);
                    (d <= VIZINHANCA_PX || (todos && poucas)).then_some((d, v))
                })
                .collect();
            vizinhos.sort_by(|a, b| a.0.total_cmp(&b.0));
            // A tinta na tela entra em CADA posição da sequência: o relevo que um esculpir precisa
            // pinta-se com o pincel de depósito, ANTES de a ferramenta de esculpir ser escolhida.
            let mut tentativas: Vec<(Gesto, Vec<Gesto>)> = (0..=armar.len())
                .rev()
                .map(|k| {
                    let mut mais = armar.clone();
                    mais.insert(k, Gesto::Traco);
                    (Gesto::Traco, mais)
                })
                .collect();
            tentativas.extend(vizinhos.into_iter().map(|(_, v)| {
                let mut mais = armar.clone();
                mais.push(v.clone());
                (v, mais)
            }));
            total = tentativas.len();
            for (t, mais) in tentativas {
                feitas += 1;
                let Ok((base, _)) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    mede(media, &mais, &Gesto::Nenhum)
                })) else {
                    continue;
                };
                let Ok((px, _)) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    mede(media, &mais, g)
                })) else {
                    continue;
                };
                let (n, ..) = diferenca(&base, &px);
                if n > 0 {
                    return Some((t, n));
                }
            }
            None
        }))
        .unwrap_or(None);
        eprintln!(
            "ARMA-TEMPO\t{media:?}\t{feitas}/{total}\t{:.1} s\t{} armado(s)\t{g:?}",
            t0.elapsed().as_secs_f32(),
            armar.len()
        );
        r
    });
    eprintln!(
        "CENSO-TEMPO\t{media:?}\tprocura\t{} candidatos\t{:.1} s",
        candidatos.len(),
        t_procura.elapsed().as_secs_f32()
    );
    for ((armar, g), r) in candidatos.iter().zip(resultados) {
        let estado = if armar.is_empty() {
            "fábrica".to_owned()
        } else {
            armar.iter().map(rotulo).collect::<Vec<_>>().join(" + ")
        };
        match r {
            Some((t, n)) => eprintln!(
                "ARMA\t{media:?}\t{}\tINERTE-ATÉ\t{}\tn={n}\testado={estado}",
                rotulo(g),
                if matches!(t, Gesto::Traco) {
                    "tinta na tela".to_owned()
                } else {
                    rotulo(&t)
                }
            ),
            None => eprintln!(
                "ARMA\t{media:?}\t{}\tMORTO\t-\t-\testado={estado}",
                rotulo(g)
            ),
        }
    }
}

/// Um alvo da sonda das pré-condições: um id (o gesto é o que a tela armada oferece para ele) ou um
/// gesto escrito à mão (quando o sentido do movimento importa).
enum Alvo {
    Id(NodeId),
    Gesto(Gesto),
}

/// `(meio, a pré-condição em palavras, os gestos que a ARMAM, os alvos, medir DEPOIS do traço?)`.
type Linha = (PaintMedia, &'static str, Vec<Gesto>, Vec<Alvo>, bool);

fn opcao(menu: NodeId, opcao: NodeId) -> Gesto {
    Gesto::Opcao(menu, opcao, 0, 0)
}

/// As PRÉ-CONDIÇÕES que a 1.ª passagem do censo acusou e que a leitura do código nomeou. Cada linha
/// arma a condição pela porta do painel e mede os alvos contra a base JÁ armada.
fn pre_condicoes() -> Vec<Linha> {
    use PaintMedia::*;
    use ph2d_tool_painter::ids::*;
    let mut v: Vec<Linha> = Vec::new();
    for m in [Digital, Impasto, WetPaint] {
        v.push((
            m,
            "Taper com comprimento (a pega da curva arrastada)",
            vec![Gesto::Arrasto(painter_taper_handle_id(0), 0.6, 0.4)],
            vec![
                Alvo::Id(PAINTER_TAPER_TIP_START),
                Alvo::Id(PAINTER_TAPER_OPACITY),
            ],
            false,
        ));
        v.push((
            m,
            "Space Attenuation ligado",
            vec![Gesto::Clique(PAINTER_BRUSH_SPACE_ATTEN)],
            vec![Alvo::Id(PAINTER_BRUSH_ACCUMULATE)],
            false,
        ));
        v.push((
            m,
            "Accumulate ligado",
            vec![Gesto::Clique(PAINTER_BRUSH_ACCUMULATE)],
            vec![Alvo::Id(PAINTER_BRUSH_SPACE_ATTEN)],
            false,
        ));
        v.push((
            m,
            "Accumulate ligado e Strength 0,4",
            vec![
                Gesto::Clique(PAINTER_BRUSH_ACCUMULATE),
                Gesto::Slider(PAINTER_BRUSH_STRENGTH_SLIDER, 0.4),
            ],
            vec![Alvo::Id(PAINTER_BRUSH_SPACE_ATTEN)],
            false,
        ));
        v.push((
            m,
            "papel com relevo (Relief 0,6)",
            vec![Gesto::Numero(PAINTER_SUBSTRATE_RELIEF, 0.6)],
            vec![
                Alvo::Gesto(opcao(
                    PAINTER_WATERCOLOR_PAPER_KIND,
                    painter_paper_kind_option_id(1),
                )),
                Alvo::Gesto(opcao(
                    PAINTER_WATERCOLOR_PAPER_KIND,
                    painter_paper_kind_option_id(2),
                )),
                Alvo::Id(PAINTER_SUBSTRATE_ROUGHNESS),
            ],
            false,
        ));
        v.push((
            m,
            "papel com relevo e um papel escolhido",
            vec![
                Gesto::Numero(PAINTER_SUBSTRATE_RELIEF, 0.6),
                opcao(
                    PAINTER_WATERCOLOR_PAPER_KIND,
                    painter_paper_kind_option_id(1),
                ),
            ],
            vec![Alvo::Id(PAINTER_SUBSTRATE_ROUGHNESS)],
            false,
        ));
        v.push((
            m,
            "Shape não redonda",
            // No Wet Paint o Noise deixa a tela VAZIA (BUGS_painter #47): o Checker pinta.
            vec![opcao(
                PAINTER_SHAPE_KIND,
                painter_shape_kind_option_id(if m == WetPaint { 2 } else { 1 }),
            )],
            vec![Alvo::Id(PAINTER_BRUSH_JITTER_ROTATE)],
            false,
        ));
    }
    v.push((
        Digital,
        "Strength 0,4 (o Accumulate tem tecto a ultrapassar)",
        vec![Gesto::Slider(PAINTER_BRUSH_STRENGTH_SLIDER, 0.4)],
        vec![Alvo::Id(PAINTER_BRUSH_ACCUMULATE)],
        false,
    ));
    for m in [Digital, Watercolor, Impasto, WetPaint] {
        // Na Aquarela o menu do Grain só está na tela com «Same as Paper» desligado (dono 22/10).
        let grao_na_tela = |mut g: Vec<Gesto>| {
            if m == Watercolor {
                g.insert(0, Gesto::Clique(PAINTER_WATERCOLOR_GRAN_SAME));
            }
            g
        };
        v.push((
            m,
            "Grain Noise (Detail de fábrica: uma oitava)",
            grao_na_tela(vec![opcao(
                PAINTER_BRUSH_TEXTURE_KIND,
                painter_brush_texture_kind_option_id(1),
            )]),
            vec![Alvo::Id(PAINTER_BRUSH_TEXTURE_PARAMS[3])],
            false,
        ));
        v.push((
            m,
            "Grain Noise com Detail 0,6",
            grao_na_tela(vec![
                opcao(
                    PAINTER_BRUSH_TEXTURE_KIND,
                    painter_brush_texture_kind_option_id(1),
                ),
                Gesto::Numero(PAINTER_BRUSH_TEXTURE_PARAMS[2], 0.6),
            ]),
            vec![Alvo::Id(PAINTER_BRUSH_TEXTURE_PARAMS[3])],
            false,
        ));
        // A profundidade do grão por dab (L9): o mínimo e o controlo só agem com o jitter acima de 0.
        v.push((
            m,
            "Grain Noise com Depth jitter 0,6",
            grao_na_tela(vec![
                opcao(
                    PAINTER_BRUSH_TEXTURE_KIND,
                    painter_brush_texture_kind_option_id(1),
                ),
                Gesto::Numero(PAINTER_PONTA_FIELDS[3], 0.6),
            ]),
            vec![
                Alvo::Id(PAINTER_PONTA_FIELDS[2]),
                Alvo::Id(PAINTER_PONTA_FIELDS[4]),
            ],
            false,
        ));
    }
    for m in [Digital, Watercolor, Impasto, WetPaint] {
        v.push((
            m,
            "Jitter 0,8",
            vec![Gesto::Slider(PAINTER_BRUSH_JITTER, 0.8)],
            vec![Alvo::Gesto(opcao(
                PAINTER_BRUSH_JITTER_UNIT,
                painter_brush_jitter_unit_option_id(1),
            ))],
            false,
        ));
        v.push((
            m,
            "Randomize Hue 0,8",
            vec![Gesto::Slider(PAINTER_BRUSH_COLOR_JITTER_HUE, 0.8)],
            vec![Alvo::Id(PAINTER_BRUSH_COLOR_JITTER_PER_STROKE)],
            false,
        ));
        v.push((
            m,
            "Dash ligado (Ratio 0,3)",
            vec![Gesto::Slider(PAINTER_BRUSH_DASH_RATIO, 0.3)],
            vec![
                Alvo::Id(PAINTER_BRUSH_DASH_LENGTH),
                Alvo::Id(PAINTER_BRUSH_DASH_LENGTH_CHIP),
            ],
            false,
        ));
    }
    v.push((
        Watercolor,
        "papel escolhido",
        vec![opcao(
            PAINTER_WATERCOLOR_PAPER_KIND,
            painter_paper_kind_option_id(1),
        )],
        vec![
            Alvo::Id(PAINTER_SUBSTRATE_ROUGHNESS),
            Alvo::Id(PAINTER_SUBSTRATE_RELIEF),
        ],
        false,
    ));
    v.push((
        Watercolor,
        "papel escolhido e Relief 0,6",
        vec![
            opcao(
                PAINTER_WATERCOLOR_PAPER_KIND,
                painter_paper_kind_option_id(1),
            ),
            Gesto::Numero(PAINTER_SUBSTRATE_RELIEF, 0.6),
        ],
        vec![Alvo::Id(PAINTER_SUBSTRATE_ROUGHNESS)],
        false,
    ));
    for (nome, arma) in [
        (
            "Wet 0,8, Dry Time para o MÍNIMO",
            Gesto::Numero(PAINTER_WATERCOLOR_WET, 0.8),
        ),
        (
            "Smudge 0,8, Dry Time para o MÍNIMO",
            Gesto::Numero(PAINTER_WATERCOLOR_SMUDGE, 0.8),
        ),
        (
            "Charge 0,4, Dry Time para o MÍNIMO",
            Gesto::Numero(PAINTER_WATERCOLOR_CHARGE, 0.4),
        ),
    ] {
        v.push((
            Watercolor,
            nome,
            vec![arma],
            vec![Alvo::Gesto(Gesto::Numero(PAINTER_WATERCOLOR_DRY_TIME, 2.0))],
            false,
        ));
    }
    v.push((
        Watercolor,
        "Same as Paper DESLIGADO",
        vec![Gesto::Clique(PAINTER_WATERCOLOR_GRAN_SAME)],
        (1..=4)
            .map(|k| {
                Alvo::Gesto(opcao(
                    PAINTER_BRUSH_TEXTURE_KIND,
                    painter_brush_texture_kind_option_id(k),
                ))
            })
            .collect(),
        false,
    ));
    // As Grain Colors (dono 22/10): a linha «Use Color Ramp» com um Grain escolhido, em cada meio —
    // viva onde o meio lê o grão; esmaecida no Wet Paint fora da ferramenta Paint; na Aquarela com «Same
    // as Paper» ligado ela SAI da tela com o grão próprio (`seam_same_as_paper`).
    let grao = opcao(
        PAINTER_BRUSH_TEXTURE_KIND,
        painter_brush_texture_kind_option_id(1),
    );
    let ferramenta_wet = |k: usize| PAINTER_WETPAINT_TOOL_IDS[k];
    // Na Aquarela de fábrica «Same as Paper» está ligado e o menu do Grain nem está na tela: desliga-se antes.
    let papel = || vec![Gesto::Clique(PAINTER_WATERCOLOR_GRAN_SAME)];
    for (m, nome, antes, mais) in [
        (Digital, "Grain escolhido", Vec::new(), Vec::new()),
        (Impasto, "Grain escolhido", Vec::new(), Vec::new()),
        (WetPaint, "Grain escolhido", Vec::new(), Vec::new()),
        (
            Watercolor,
            "Grain escolhido e depois Same as Paper ligado (sai da tela)",
            papel(),
            papel(),
        ),
        (
            Watercolor,
            "Grain escolhido com Same as Paper DESLIGADO",
            papel(),
            Vec::new(),
        ),
        (
            WetPaint,
            "Grain escolhido, tinta molhada na tela e a ferramenta Smear",
            Vec::new(),
            vec![Gesto::Traco, Gesto::Clique(ferramenta_wet(2))],
        ),
    ] {
        v.push((
            m,
            nome,
            antes
                .into_iter()
                .chain(std::iter::once(grao.clone()))
                .chain(mais)
                .collect(),
            vec![Alvo::Id(PAINTER_BRUSH_TEXTURE_RAMP_ENABLE)],
            false,
        ));
    }
    v.push((
        Watercolor,
        "Charge 0,4 (o pincel volta a apanhar)",
        vec![Gesto::Numero(PAINTER_WATERCOLOR_CHARGE, 0.4)],
        vec![Alvo::Id(PAINTER_WATERCOLOR_PULL)],
        false,
    ));
    v.push((
        Watercolor,
        "Wet 0,8 (a borda molhada)",
        vec![Gesto::Numero(PAINTER_WATERCOLOR_WET, 0.8)],
        vec![Alvo::Id(PAINTER_WATERCOLOR_SPREAD)],
        false,
    ));
    v.push((
        Watercolor,
        "Ragged 24 (a borda manchada)",
        vec![Gesto::Numero(PAINTER_WATERCOLOR_WARP, 24.0)],
        vec![Alvo::Id(PAINTER_WATERCOLOR_SPREAD)],
        false,
    ));
    v.push((
        Watercolor,
        "fábrica, Dry Time para o MÍNIMO",
        Vec::new(),
        vec![Alvo::Gesto(Gesto::Numero(PAINTER_WATERCOLOR_DRY_TIME, 2.0))],
        false,
    ));
    v.push((
        Watercolor,
        "Automatic DESLIGADO e Shape não redonda",
        vec![
            Gesto::Clique(PAINTER_SHAPE_WATERCOLOR_AUTO),
            opcao(PAINTER_SHAPE_KIND, painter_shape_kind_option_id(1)),
        ],
        vec![
            Alvo::Id(PAINTER_BRUSH_JITTER_ROTATE),
            Alvo::Id(PAINTER_SHAPE_RAMP_ENABLE),
        ],
        false,
    ));
    v.push((
        Impasto,
        "um Grain escolhido",
        vec![opcao(
            PAINTER_BRUSH_TEXTURE_KIND,
            painter_brush_texture_kind_option_id(1),
        )],
        vec![Alvo::Id(PAINTER_IMPASTO_SOURCE_GRAIN)],
        false,
    ));
    v.push((
        Impasto,
        "a luz 2 seleccionada",
        vec![Gesto::Clique(PAINTER_IMPASTO_LIGHT_2)],
        vec![Alvo::Id(PAINTER_IMPASTO_LIGHT_ON)],
        true,
    ));
    v.push((
        Impasto,
        "Adjust Last Stroke DESLIGADO (fábrica)",
        Vec::new(),
        vec![
            Alvo::Id(PAINTER_IMPASTO_DEPTH),
            Alvo::Id(PAINTER_IMPASTO_BODY),
            Alvo::Id(PAINTER_IMPASTO_SHINE),
            Alvo::Id(PAINTER_IMPASTO_LIGHT_ANGLE),
        ],
        true,
    ));
    v.push((
        Impasto,
        "Adjust Last Stroke LIGADO",
        vec![Gesto::Clique(PAINTER_IMPASTO_LIVE_EDIT)],
        vec![
            Alvo::Id(PAINTER_IMPASTO_DEPTH),
            Alvo::Id(PAINTER_IMPASTO_BODY),
            Alvo::Id(PAINTER_IMPASTO_SHINE),
            Alvo::Id(PAINTER_IMPASTO_LIGHT_ANGLE),
        ],
        true,
    ));
    // A Shape Color (prompt 41): cada controlo da secção sobre um Shape (Voronoi), com as duas camadas
    // (a fábrica) e com a Falloff Color desligada — o Wet Paint responde como o Digital (medido 16/10:
    // controlo a controlo, os mesmos vivos e os mesmos inertes).
    for (m, sem_queda) in [Digital, Watercolor, Impasto, WetPaint]
        .into_iter()
        .flat_map(|m| [(m, false), (m, true)])
    {
        let mut armar = vec![
            Gesto::Clique(PAINTER_SHAPE_RAMP_ENABLE),
            opcao(PAINTER_SHAPE_KIND, painter_shape_kind_option_id(3)),
        ];
        if m == Watercolor {
            armar.insert(0, Gesto::Clique(PAINTER_SHAPE_WATERCOLOR_AUTO));
        }
        if sem_queda {
            armar.push(Gesto::Clique(PAINTER_SHAPE_FALLOFF_COLOR));
        }
        v.push((
            m,
            if sem_queda {
                "Shape Color sem a Falloff Color, sobre um Shape Voronoi"
            } else {
                "Shape Color ligada sobre um Shape Voronoi"
            },
            armar,
            [
                PAINTER_SHAPE_TEXTURE_COLOR,
                PAINTER_SHAPE_FALLOFF_COLOR,
                PAINTER_SHAPE_COLOR_SWAP,
                PAINTER_SHAPE_COLOR_BLEND,
                PAINTER_SHAPE_TEXTURE_OPACITY,
                PAINTER_SHAPE_FALLOFF_OPACITY,
                PAINTER_SHAPE_RAMP_MODE,
                PAINTER_SHAPE_RAMP_INTERP,
                PAINTER_SHAPE_RAMP_BW,
                PAINTER_SHAPE_RAMP_INVERT,
                PAINTER_SHAPE_RAMP_ADD,
                PAINTER_SHAPE_RAMP_REMOVE,
                PAINTER_SHAPE_RAMP_STOP_POS,
                PAINTER_SHAPE_FALLOFF_RAMP_MODE,
                PAINTER_SHAPE_FALLOFF_RAMP_INTERP,
                PAINTER_SHAPE_FALLOFF_RAMP_INVERT,
                PAINTER_SHAPE_FALLOFF_RAMP_ADD,
                PAINTER_SHAPE_FALLOFF_RAMP_REMOVE,
            ]
            .into_iter()
            .map(Alvo::Id)
            .collect(),
            false,
        ));
    }
    // «Use Texture Colors» (26 C1): só aparece com uma ponta de imagem colorida; a água não lê a cor
    // da textura (o motor deposita a cor do pincel pela silhueta).
    for m in [Digital, Impasto, WetPaint] {
        v.push((
            m,
            "ponta de imagem colorida (Use as Brush Shape)",
            vec![Gesto::PontaColorida],
            vec![Alvo::Id(PAINTER_SHAPE_PER_LAYER_COLOR)],
            false,
        ));
    }
    let ferramenta_wet = |k: usize| PAINTER_WETPAINT_TOOL_IDS[k];
    v.push((
        WetPaint,
        "tinta molhada na tela e a ferramenta Erase",
        vec![Gesto::Traco, Gesto::Clique(ferramenta_wet(1))],
        vec![Alvo::Id(PAINTER_WETPAINT_ERASE)],
        false,
    ));
    for (k, nome) in [
        (2, "tinta molhada na tela e a ferramenta Smear"),
        (3, "tinta molhada na tela e a ferramenta Blend"),
    ] {
        v.push((
            WetPaint,
            nome,
            vec![Gesto::Traco, Gesto::Clique(ferramenta_wet(k))],
            vec![Alvo::Id(PAINTER_WETPAINT_PICKUP)],
            false,
        ));
    }
    v.push((
        WetPaint,
        "fábrica, Pickup medido DEPOIS do traço",
        Vec::new(),
        vec![
            Alvo::Id(PAINTER_WETPAINT_PICKUP),
            Alvo::Id(PAINTER_WETPAINT_ERASE),
        ],
        true,
    ));
    v
}

#[test]
#[ignore = "sonda: os candidatos a morto com a PRÉ-CONDIÇÃO armada"]
fn sonda_as_pre_condicoes() {
    let nomes = nomes();
    let so: Option<String> = std::env::var("CENSO_MEIO").ok();
    for (media, condicao, armar, alvos, depois) in pre_condicoes() {
        if so.as_ref().is_some_and(|m| *m != format!("{media:?}")) {
            continue;
        }
        let corre = |g: &Gesto| {
            if depois {
                ensaio_depois(media, &armar, g)
            } else {
                ensaio(media, &armar, g)
            }
        };
        let armada = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (corre(&Gesto::Nenhum).0, gestos(media, &armar))
        }));
        let Ok((base, oferta)) = armada else {
            eprintln!("PRE\t{media:?}\t{condicao}\t-\tPRÉ-CONDIÇÃO-FORA-DA-TELA");
            continue;
        };
        for alvo in alvos {
            let g = match alvo {
                Alvo::Gesto(g) => g,
                Alvo::Id(id) => match oferta.iter().find(|g| g.id() == Some(id)) {
                    Some(g) => g.clone(),
                    None => {
                        eprintln!(
                            "PRE\t{media:?}\t{condicao}\t{}\tNÃO-PINTADO",
                            nome(&nomes, id)
                        );
                        continue;
                    }
                },
            };
            let corrida = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| corre(&g)));
            let Ok((px, entrega)) = corrida else {
                eprintln!("PRE\t{media:?}\t{condicao}\t{g:?}\tPÂNICO");
                continue;
            };
            let (n, soma, max) = diferenca(&base, &px);
            // Quanto cada imagem PINTOU (texels diferentes da tela de partida) — distingue «o
            // controlo muda o traço» de «o controlo apaga o traço».
            let (pintou_base, ..) = diferenca(&tela(), &base);
            let (pintou, ..) = diferenca(&tela(), &px);
            eprintln!(
                "PRE\t{media:?}\t{condicao}\t{}\t{}\tn={n}\tsoma={soma}\tmax={max}\tajuste={}\tdepois={depois}\tpintou={pintou_base}->{pintou}",
                nome(&nomes, g.id().unwrap_or(NodeId(0))),
                tipo_do_gesto(&g),
                if entrega.ajuste_mudou {
                    "mudou"
                } else {
                    "igual"
                },
            );
        }
    }
}

