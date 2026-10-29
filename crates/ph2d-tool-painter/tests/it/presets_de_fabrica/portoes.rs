//! Os PORTÕES da fábrica (o gerador, o censo, a receita, a miniatura, a largura prometida) — partidos de
//! `presets_de_fabrica.rs` pelo tecto de 700 linhas; os ajudantes moram no pai.

use super::memoria::pico_mib;
use super::*;

/// **A promessa na ponta da tabela**: a lei crua ONDULA de raio para raio e 64 pontos interpolam a ondulação (Knife
/// Oily raio 300: o motor 859 entre os pontos 803→573 e 892→616 pintava 614 px por 600, +2,3 %). Mede-se a promessa nos
/// [`RAIOS_DA_PROMESSA`]; onde passa de ¾ da tolerância, o ponto MEDIDO (raio do motor → largura que o slider deu) entra
/// na tabela, no seu lugar, e repete-se (≤ 3 rondas). Tabela que já cumpre fica como está.
fn refina_a_promessa(p: &mut Preset) {
    let rel = if borda_ruidosa(p) {
        TOLERANCIA_REL_GRAO
    } else {
        TOLERANCIA_REL
    };
    for _ in 0..3 {
        let medidas: Vec<(f32, f32, f32)> = std::thread::scope(|s| {
            let h: Vec<_> = RAIOS_DA_PROMESSA
                .iter()
                .map(|&r| {
                    let p = &*p;
                    s.spawn(move || {
                        let motor = raio_do_motor_da_tabela(&p.parametros.width_table, r);
                        (r, motor, largura_de_com(p, r, false, Relogio::DeUmaVez))
                    })
                })
                .collect();
            h.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut mudou = false;
        for (r, motor, l) in medidas {
            let pedido = 2.0 * r;
            if (l - pedido).abs() <= 0.75 * TOLERANCIA_PX.max(rel * pedido) {
                continue;
            }
            let t = &mut p.parametros.width_table;
            let m = f64::from(motor);
            if t.iter().any(|&[r0, _]| (r0 - m).abs() < 0.05) {
                continue;
            }
            let k = t.partition_point(|&[r0, _]| r0 < m);
            t.insert(
                k,
                [
                    format!("{motor}").parse().unwrap(),
                    format!("{l}").parse().unwrap(),
                ],
            );
            mudou = true;
        }
        if !mudou {
            break;
        }
    }
}

#[test]
#[ignore = "gerador: escreve assets/brushes/<motor do meio> (PH2D_REGERA_PRESETS=1)"]
fn gera_os_presets_de_fabrica() {
    let escreve = std::env::var("PH2D_REGERA_PRESETS").as_deref() == Ok("1");
    // `PH2D_REGERA_SO="Nome,Nome"`: só esses (cada preset do Wet custa ~2 min de lei da largura sob carga).
    let so = std::env::var("PH2D_REGERA_SO").ok();
    let pedido = |nome: &str| {
        so.as_deref()
            .is_none_or(|s| s.split(',').any(|n| n == nome))
    };
    if let Some(s) = so.as_deref() {
        let nomes: Vec<&str> = receitas()
            .map(|r| r.nome)
            .chain(receitas_dos_meios().map(|r| r.nome))
            .collect();
        let sem: Vec<&str> = s.split(',').filter(|n| !nomes.contains(n)).collect();
        assert!(sem.is_empty(), "PH2D_REGERA_SO: sem receita {sem:?}");
    }
    // Os outros meios: sem tabela de largura (a promessa do slider é do Wet), a receita é a verdade inteira.
    for r in receitas_dos_meios().filter(|r| pedido(r.nome)) {
        let preset = preset_do_meio(r);
        confere_receita_do_meio(r, &preset);
        let rel = relativo_do_meio(r);
        println!(
            "[fabrica] {:<14} miniatura fnv {:#018x} · {rel}",
            r.nome,
            fnv1a(&preset.miniatura.rgba)
        );
        if escreve {
            gravar(&raiz(), &rel, &preset).unwrap();
        }
    }
    for r in receitas().filter(|r| pedido(r.nome)) {
        let mut preset = Preset {
            parametros: parametros(r),
            origem: Origem::Fabrica,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![255; 4],
            },
            texturas: Default::default(),
        };
        // `PH2D_REGERA_SO_MINIATURA=1`: a tabela do ficheiro fica (só a miniatura, quando a lei da largura não mudou).
        if std::env::var("PH2D_REGERA_SO_MINIATURA").as_deref() != Ok("1") {
            preset.parametros.width_table = tabela_de_largura(&preset);
            refina_a_promessa(&mut preset);
        }
        if r.nome == "Wet Round" {
            // A tabela que o motor embute (`wetpaint/largura.rs::LARGURA_DO_WET_ROUND`), já limpa.
            let mut t = PainterTool::default();
            assert!(t.wet_preset_aplicar(&preset).is_empty());
            println!(
                "[fabrica] LARGURA_DO_WET_ROUND = {:?}",
                t.wet_largura_tabela()
            );
        }
        preset.miniatura = miniatura_do_preset(&preset);
        let larguras: Vec<String> = preset
            .parametros
            .width_table
            .iter()
            .map(|[r, l]| format!("{r}→{l}"))
            .collect();
        println!(
            "[fabrica] {:<14} miniatura fnv {:#018x} · pico {:.0} MiB · largura {}",
            r.nome,
            fnv1a(&preset.miniatura.rgba),
            pico_mib(),
            larguras.join(" ")
        );
        if escreve {
            let rel = caminho_relativo(MOTOR_DOS_PRESETS, r.meio, r.grupo, r.nome);
            gravar(&raiz(), &rel, &preset).unwrap();
        }
    }
    if !escreve {
        println!("[fabrica] nada escrito (PH2D_REGERA_PRESETS=1 para gravar)");
    }
}

/// ⭐ **A fábrica é CONTADA das receitas**: o conjunto de ficheiros em `assets/brushes/` (as raízes dos quatro meios) é
/// EXACTAMENTE o das receitas (a base + cada família, `WET` e `MEIOS`) — nem ficheiro sem receita, nem receita sem
/// ficheiro —, cada um na pasta do seu meio e com o nome do preset, e os NOMES são únicos nos quatro meios (a bancada,
/// o `de_fabrica` e o `PH2D_REGERA_SO` procuram pelo nome).
#[test]
fn the_factory_presets_are_counted() {
    let b = Biblioteca::carregar(&raiz(), None);
    assert!(
        b.erros.is_empty(),
        "ficheiros de fábrica ilegíveis: {:?}",
        b.erros
    );
    let mut esperados: Vec<String> = receitas()
        .map(|r| caminho_relativo(MOTOR_DOS_PRESETS, r.meio, r.grupo, r.nome))
        .chain(receitas_dos_meios().map(relativo_do_meio))
        .collect();
    esperados.sort();
    let n = esperados.len();
    esperados.dedup();
    assert_eq!(esperados.len(), n, "duas receitas no mesmo caminho");
    assert_eq!(n, PRESETS_DE_FABRICA);
    let achados: Vec<String> = b.entradas.iter().map(|e| e.relativo.clone()).collect();
    let sem_ficheiro: Vec<&String> = esperados.iter().filter(|r| !achados.contains(r)).collect();
    let sem_receita: Vec<&String> = achados.iter().filter(|r| !esperados.contains(r)).collect();
    assert!(
        sem_ficheiro.is_empty() && sem_receita.is_empty(),
        "a fábrica não é a das receitas — receita sem ficheiro (regere): {sem_ficheiro:?} · ficheiro sem receita: \
         {sem_receita:?}"
    );
    let mut nomes = Vec::new();
    for e in &b.entradas {
        let m = meio_do_preset(&e.preset.parametros.media)
            .unwrap_or_else(|| panic!("{}: meio desconhecido", e.relativo));
        assert_eq!(
            e.partes()[0],
            motor_do_meio(m),
            "{}: fora da pasta do meio",
            e.relativo
        );
        assert_eq!(e.preset.origem, Origem::Fabrica, "{}", e.relativo);
        assert_eq!(
            e.partes()[3],
            e.preset.parametros.name,
            "o nome do ficheiro é o do preset"
        );
        assert!(
            !nomes.contains(&e.preset.parametros.name),
            "{}: o nome {:?} já existe noutro preset de fábrica",
            e.relativo,
            e.preset.parametros.name
        );
        nomes.push(e.preset.parametros.name.clone());
    }
}

/// ⭐ **Os óleos de fábrica vêm com o Real Oil LIGADO** (dono 28/10, HANDOFF_W3b §Aberto 1; a caixa é do pincel, e
/// escolher um preset sem ela desligava-a): toda receita da pasta `Oil/` o declara, e todo ficheiro de fábrica sob
/// `WetPainter/Oil/` o grava — e só esses (fora dela a fábrica fica como era). O ficheiro e a receita, os dois: a
/// receita sem regerar acusa aqui antes do `cada_preset_de_fabrica_e_a_sua_receita`.
#[test]
fn os_oleos_de_fabrica_vem_com_o_real_oil() {
    let receitas_sem: Vec<&str> = receitas()
        .filter(|r| (r.meio == "Oil") != r.real_oil)
        .map(|r| r.nome)
        .collect();
    assert!(
        receitas_sem.is_empty(),
        "receita com o Real Oil fora da lei (Oil/ ⇔ ligado): {receitas_sem:?}"
    );
    let pasta = format!("{MOTOR_DOS_PRESETS}/Oil/");
    let mut oleos = 0;
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        let e_oleo = rel.starts_with(&pasta);
        if p.parametros.media != MEIO_DO_PRESET {
            assert!(!e_oleo, "{rel}: sob Oil/ mas não é do Wet Paint");
            continue;
        }
        // O que o artista recebe: o preset aplicado arma (ou não) a caixa.
        let mut t = PainterTool::default();
        assert!(t.wet_preset_aplicar(&p).is_empty(), "{rel}: aplicar falhou");
        let ligado = t.wet_oleo_real().is_some();
        oleos += usize::from(e_oleo);
        assert_eq!(ligado, e_oleo, "{rel}: Real Oil {ligado} (Oil/ ⇔ ligado)");
    }
    assert!(oleos > 0, "nenhum ficheiro sob {pasta}");
}

/// Os campos de topo (e as chaves de `engine`/`brush`) em que a receita e o ficheiro divergem — o veredito diz QUAL.
fn campos_que_diferem(
    receita: &Parametros,
    ficheiro: &Parametros,
    texturas_iguais: bool,
) -> Vec<String> {
    let (Value::Object(a), Value::Object(b)) = (
        serde_json::to_value(receita).unwrap(),
        serde_json::to_value(ficheiro).unwrap(),
    ) else {
        unreachable!("Parametros serializa como objecto")
    };
    let mut out = Vec::new();
    for k in a.keys().chain(b.keys().filter(|k| !a.contains_key(*k))) {
        let (x, y) = (a.get(k), b.get(k));
        if x == y {
            continue;
        }
        match (x, y) {
            (Some(Value::Object(xm)), Some(Value::Object(ym))) => {
                for c in xm.keys().chain(ym.keys().filter(|c| !xm.contains_key(*c))) {
                    if xm.get(c) != ym.get(c) {
                        out.push(format!(
                            "{k}.{c}: receita {:?} · ficheiro {:?}",
                            xm.get(c),
                            ym.get(c)
                        ));
                    }
                }
            }
            _ => out.push(format!("{k}: receita {x:?} · ficheiro {y:?}")),
        }
    }
    if !texturas_iguais {
        out.push("texturas".to_owned());
    }
    out
}

/// ⭐ **Cada preset de fábrica É a sua receita**: a receita de hoje, sem pintar, dá os parâmetros do ficheiro (no Wet,
/// com a tabela de largura do ficheiro, que é medida; nos outros meios também as texturas) — mudar uma receita sem
/// regerar acusa aqui (a miniatura acusa o motor; este, a receita).
#[test]
fn cada_preset_de_fabrica_e_a_sua_receita() {
    let mut mudaram = Vec::new();
    for r in receitas() {
        let rel = caminho_relativo(MOTOR_DOS_PRESETS, r.meio, r.grupo, r.nome);
        let f = ler(&std::fs::read(raiz().join(&rel)).unwrap()).unwrap();
        let mut p = parametros(r);
        p.width_table.clone_from(&f.parametros.width_table);
        if p != f.parametros {
            mudaram.push(rel);
        }
    }
    assert!(
        mudaram.is_empty(),
        "a receita mudou e o ficheiro não — regere: {mudaram:?}"
    );
    for r in receitas_dos_meios() {
        let rel = relativo_do_meio(r);
        let f = ler(&std::fs::read(raiz().join(&rel)).unwrap_or_else(|e| panic!("{rel}: {e}")))
            .unwrap();
        let (p, t) = parametros_do_meio(r);
        assert!(
            p == f.parametros && t == f.texturas,
            "{rel}: a receita mudou e o ficheiro não — regere (PH2D_REGERA_SO=\"{}\"); difere: {:?}",
            r.nome,
            campos_que_diferem(&p, &f.parametros, t == f.texturas)
        );
    }
}

/// ⭐ **A prova de ponta a ponta dos três meios** (J3), sem gravar: uma receita de cada meio (a do lápis, a da aguada e
/// a do empaste dos exemplos, `presets_dos_meios::EXEMPLOS`) vira um preset que arma o SEU meio, fixa o que a receita
/// diz (`confere_receita_do_meio`) e tem uma miniatura com tinta.
#[test]
fn uma_receita_de_cada_meio_vira_um_preset_desse_meio() {
    const PROVAS: [ReceitaMeio; 3] = [
        ReceitaMeio {
            pincel: &[("hardness", "0.3"), ("spacing", "0.05")],
            mao: &[
                ("pressureSize", 0.6),
                ("pressureOpacity", 0.5),
                ("tiltOpacity", 0.4),
            ],
            size_px: Some(12.0),
            opacity: Some(0.85),
            ..ReceitaMeio::digital("Pencil", "Graphite", "prova Digital")
        },
        ReceitaMeio {
            pincel: &[
                ("granulation", "0.4"),
                ("wet_dilution", "0.3"),
                ("depth", "1.6"),
            ],
            mao: &[("pressureSize", 0.3)],
            size_px: Some(30.0),
            ..ReceitaMeio::aquarela("Watercolor", "Round", "prova Watercolor")
        },
        ReceitaMeio {
            pincel: &[
                ("impasto_depth", "0.6"),
                ("impasto_body", "0.7"),
                ("dab_flatten", "0.4"),
            ],
            mao: &[("pressureOpacity", 0.4)],
            modo: Some("Knife"),
            ..ReceitaMeio::impasto("Oil", "Bristle", "prova Impasto")
        },
    ];
    let neutros: Vec<Vec<u8>> = [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
    ]
    .into_iter()
    .map(|m| {
        preset_do_meio(&ReceitaMeio::novo(m, "P", "G", "neutro"))
            .miniatura
            .rgba
    })
    .collect();
    for (r, neutro) in PROVAS.iter().zip(&neutros) {
        let p = preset_do_meio(r);
        assert_eq!(p.parametros.media, nome_do_meio(r.meio));
        assert!(
            p.parametros.width_table.is_empty(),
            "{}: tabela de largura fora do Wet",
            r.nome
        );
        confere_receita_do_meio(r, &p);
        let rgba = &p.miniatura.rgba;
        let tinta = rgba.chunks(4).filter(|c| c[..3] != rgba[..3]).count();
        println!(
            "[meios] {}: {} → {} px com tinta de {} · fnv {:#018x}",
            r.nome,
            relativo_do_meio(r),
            tinta,
            rgba.len() / 4,
            fnv1a(rgba)
        );
        assert!(
            tinta > 100,
            "{}: a miniatura não tem tinta ({tinta} px)",
            r.nome
        );
        assert!(
            rgba != neutro,
            "{}: a receita pinta a miniatura do pincel neutro",
            r.nome
        );
    }
}

/// A receita recusa um campo de OUTRO meio (o da aguada num Digital) — nunca um preset com um campo que ninguém lê.
#[test]
#[should_panic(expected = "não é do pincel do meio")]
fn a_receita_recusa_um_campo_de_outro_meio() {
    let _ = parametros_do_meio(&ReceitaMeio {
        pincel: &[("granulation", "0.4")],
        ..ReceitaMeio::novo(PaintMedia::Digital, "P", "G", "errado")
    });
}

/// A receita recusa uma linha da mão que o meio não lê (a pressão → opacidade na Watercolor).
#[test]
#[should_panic(expected = "não lê a linha da mão")]
fn a_receita_recusa_uma_mao_que_o_meio_nao_le() {
    let _ = parametros_do_meio(&ReceitaMeio {
        mao: &[("pressureOpacity", 0.5)],
        ..ReceitaMeio::novo(PaintMedia::Watercolor, "P", "G", "errado")
    });
}

/// Os aprovados cuja marca É o carimbo espaçado (Óleo Real W3, revisão cega): o Dabber 4 (Spacing 0,3 sem o
/// Continuous — toques soltos de esponja, `W3_P_Dabber.md`). Com o Spacing 0,1 a marca dele vira faixa contínua: outro
/// pincel. (O Metallic Splats, 0,25, saiu da fábrica no smoke do dono 27/10 — `EM_ESTUDO_W2`.)
const CARIMBOS_ESPACADOS: [&str; 1] = ["Dabber 4"];

/// ⭐ **A REGRA do Spacing** (dono 18/10, prompt 51): cada pincel de fábrica do WET PAINT — e o que nasce SEM preset —
/// leva o Spacing [`SPACING_DA_FABRICA`] com o Continuous ligado. Os outros meios não: a regra é a da cápsula e do
/// pigmento por comprimento do Wet (o lápis do Digital vive a 0,05).
/// Fora dela, pelo NOME, só os [`CARIMBOS_ESPACADOS`].
#[test]
fn a_regra_do_spacing_da_fabrica() {
    let cumpre = |b: &std::collections::BTreeMap<String, serde_json::Value>| {
        b.get("spacing")
            .and_then(serde_json::Value::as_f64)
            .is_some_and(|s| (s - SPACING_DA_FABRICA).abs() < 1e-6)
            && b.get("continuous_spacing") == Some(&serde_json::json!(true))
    };
    let fora: Vec<String> = Biblioteca::carregar(&raiz(), None)
        .entradas
        .into_iter()
        .filter(|e| e.preset.parametros.media == MEIO_DO_PRESET)
        .filter(|e| !CARIMBOS_ESPACADOS.contains(&e.preset.parametros.name.as_str()))
        .filter(|e| !cumpre(&e.preset.parametros.brush))
        .map(|e| e.relativo)
        .collect();
    assert!(fora.is_empty(), "fora da regra do Spacing: {fora:?}");
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let sem_preset = t
        .wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "g", "sem preset"))
        .0;
    assert!(
        cumpre(&sem_preset.brush),
        "o pincel sem preset: spacing {:?} · continuous {:?}",
        sem_preset.brush.get("spacing"),
        sem_preset.brush.get("continuous_spacing")
    );
}

#[test]
fn cada_preset_de_fabrica_volta_ao_byte() {
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        assert!(escrever(&p).unwrap() == b, "{rel}: escrever(ler(b)) != b");
    }
}

#[test]
fn a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje() {
    let mut vistas = Vec::new();
    // Todas as que mudaram, não só a primeira: o desenho de uma mudança de lei é QUAIS mudaram.
    let mut mudaram = Vec::new();
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        let hoje = miniatura_do_preset(&p);
        let diferem = hoje
            .rgba
            .chunks(4)
            .zip(p.miniatura.rgba.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        if hoje != p.miniatura {
            mudaram.push(format!(
                "{rel}: {diferem} px mudaram (fnv {:#018x} → {:#018x})",
                fnv1a(&p.miniatura.rgba),
                fnv1a(&hoje.rgba)
            ));
        }
        vistas.push((rel, hoje.rgba));
    }
    assert!(
        mudaram.is_empty(),
        "o motor mudou de lei. Se foi de propósito, regere a fábrica (cabeçalho deste ficheiro) e diga-o no \
         handoff:\n{}",
        mudaram.join("\n")
    );
    // O leitor DECIDE: dois presets diferentes pintam miniaturas diferentes.
    for (i, (ra, a)) in vistas.iter().enumerate() {
        for (rb, b) in &vistas[i + 1..] {
            assert_ne!(
                a, b,
                "{ra} e {rb} pintam igual: um knob do preset não tem leitor"
            );
        }
    }
}

/// **A LARGURA PROMETIDA (W2)**: em cada preset de fábrica, o traço a raio `r` mede `2·r ± 2 px` (± 2 %
/// acima de 100 px), de 2 a 300 — raios fora da tabela do preset. CONTROLO dentro: a lei CRUA (o raio direto ao motor, a de antes
/// da W2) reprova no mesmo pincel.
/// Um preset medido pelo [`a_largura_e_a_prometida`]: o ficheiro, as larguras com a promessa e (só o Wet Round) as crus.
type Medido = (Preset, Vec<f32>, Option<Vec<f32>>);

#[test]
fn a_largura_e_a_prometida() {
    let mut falhas = Vec::new();
    // Só o Wet: a tabela de largura é a promessa do slider dele (os outros meios não a gravam nem a leem).
    let ficheiros = ficheiros_do_wet();
    // Os presets em paralelo (cada um já pinta os seus 8 raios em paralelo), mas só `núcleos ÷ 8` de cada vez: todos
    // de uma vez eram 44 × 8 = 352 telas vivas, e com os Oil/* de Real Oil (dono 28/10; o campo do óleo ~+100 B por
    // célula, 466 contra 248 MiB a raio 300) a soma passava os 24G da fatia — OOM 2× (J6). Acima dos núcleos o
    // paralelismo não compra relógio, só memória.
    let de_cada_vez = std::thread::available_parallelism()
        .map_or(4, |n| (n.get() / RAIOS_DA_PROMESSA.len()).max(1));
    let proximo = std::sync::atomic::AtomicUsize::new(0);
    let mut medidos: Vec<Option<Medido>> = (0..ficheiros.len()).map(|_| None).collect();
    std::thread::scope(|s| {
        let h: Vec<_> = (0..de_cada_vez)
            .map(|_| {
                s.spawn(|| {
                    let mut feitos = Vec::new();
                    loop {
                        let k = proximo.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some((rel, b)) = ficheiros.get(k) else {
                            break feitos;
                        };
                        let p = ler(b).unwrap();
                        let (larguras, crua) = std::thread::scope(|s| {
                            let crua = rel
                                .ends_with("Wet Round.ph2d-brush.png")
                                .then(|| s.spawn(|| larguras_da_promessa(&p, true)));
                            let larguras = larguras_da_promessa(&p, false);
                            (larguras, crua.map(|h| h.join().unwrap()))
                        });
                        feitos.push((k, (p, larguras, crua)));
                    }
                })
            })
            .collect();
        for (k, m) in h.into_iter().flat_map(|h| h.join().unwrap()) {
            medidos[k] = Some(m);
        }
    });
    let medidos = medidos.into_iter().map(Option::unwrap);
    for ((rel, _), (p, larguras, crua)) in ficheiros.iter().zip(medidos) {
        println!("[largura] {rel}: {larguras:?} (pedidas {RAIOS_DA_PROMESSA:?} × 2)");
        let tol = if borda_ruidosa(&p) {
            TOLERANCIA_REL_GRAO
        } else {
            TOLERANCIA_REL
        };
        let tecto = p
            .parametros
            .brush
            .get("size_max_px")
            .and_then(serde_json::Value::as_f64)
            .map_or(f32::INFINITY, |d| d as f32);
        for f in fora_da_promessa(&larguras, tol, tecto) {
            falhas.push(format!("{rel}: {f}"));
        }
        if let Some(crua) = crua {
            assert!(
                !fora_da_promessa(&crua, TOLERANCIA_REL, f32::INFINITY).is_empty(),
                "controlo: a lei crua do Wet Round já cumpria a promessa — o portão não mede nada"
            );
        }
    }
    assert!(
        falhas.is_empty(),
        "a largura não é a do painel:\n{}",
        falhas.join("\n")
    );
}

/// O pincel que nasce SEM preset tem a lei da largura do Wet Round de fábrica (o `WetKnobs::DEFAULT` é
/// o Wet Round): a tabela embutida no motor é a do ficheiro.
#[test]
fn a_lei_de_omissao_e_a_do_wet_round() {
    let b = Biblioteca::carregar(&raiz(), None);
    let e = b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o Wet Round de fábrica");
    let mut t = PainterTool::default();
    assert!(t.wet_preset_aplicar(&e.preset).is_empty());
    let mut nasce = PainterTool::default();
    nasce.set_paint_media(PaintMedia::WetPaint);
    assert_eq!(
        nasce.wet_largura_tabela(),
        t.wet_largura_tabela(),
        "a LARGURA_DO_WET_ROUND do motor não é a do ficheiro: regere-a (o gerador imprime-a)"
    );
}
