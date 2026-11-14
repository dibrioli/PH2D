//! **Sonda (ignorada): o travamento do começo do 1.º traço** (dono 05/11, Watercolor Basic e Wet Wash). Cronometra cada
//! evento do 1.º e do 2.º traço, com o quadro composto: ms do maior e dos 5 primeiros. `PH2D_SONDA_TELA=LxA`.

use super::offthread::otica::forca_no_teste;
use super::sonda_bordas_longe::por_nome;
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use std::time::Instant;

/// As faltas de página menores do PROCESSO (`/proc/self/stat`, campo 10): × 4 KiB = memória nova tocada.
fn faltas() -> u64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let resto = s.rsplit_once(") ").map_or("", |(_, r)| r);
    resto
        .split(' ')
        .nth(7)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn traco(t: &mut PainterTool, y: f32) -> Vec<f64> {
    traco_de(t, y, 100.0, 12.0, 30)
}

/// Um traço em `y` de `x0`, `n` passos de `passo` px.
fn traco_de(t: &mut PainterTool, y: f32, x0: f32, passo: f32, n: usize) -> Vec<f64> {
    let mut ms = Vec::new();
    let mut ev = |t: &mut PainterTool, x: f32, phase| {
        let f0 = faltas();
        // `PH2D_SONDA_PARTES_DOWN`: o pen-down por dentro (`down.*` em `stroke_lifecycle.rs`).
        let partes = phase == PointerPhase::Down && std::env::var("PH2D_SONDA_PARTES_DOWN").is_ok();
        if partes {
            crate::sonda_partes::liga();
        }
        let t0 = Instant::now();
        t.on_canvas_pointer(CanvasPointer {
            pos: [x, y],
            pressure: 0.8,
            tilt: [0.0, 0.0],
            phase,
        });
        if partes {
            crate::sonda_partes::parte("down.fim");
            let v: Vec<String> = crate::sonda_partes::tira()
                .into_iter()
                .filter(|(_, ms, _)| *ms >= 0.3)
                .map(|(n, ms, mb)| format!("{n} {ms:.1} ({mb:.0} MB)"))
                .collect();
            println!("  DOWN y={y} {}", v.join(" · "));
        }
        let p = t0.elapsed().as_secs_f64() * 1e3;
        t.on_tick(1000.0 / 60.0);
        let q = t0.elapsed().as_secs_f64() * 1e3;
        let _ = t.take_preview_arc();
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
        if ms.len() <= 3 && std::env::var("PH2D_SONDA_PARTES").is_ok() {
            println!(
                "  ev{} ponteiro {p:.1} tique {:.1} saída {:.1} · {:.1} MB novos",
                ms.len() - 1,
                q - p,
                ms[ms.len() - 1] - q,
                (faltas() - f0) as f64 * 4096.0 / 1e6
            );
        }
    };
    ev(t, x0, PointerPhase::Down);
    for k in 1..=n {
        ev(t, x0 + passo * k as f32, PointerPhase::Move);
    }
    ev(t, x0 + passo * (n + 1) as f32, PointerPhase::Up);
    ms
}

#[test]
#[ignore = "sonda: corre à mão, no perfil smoke"]
fn sonda_primeiro_traco() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let (w, h): (usize, usize) = std::env::var("PH2D_SONDA_TELA")
        .ok()
        .and_then(|v| {
            let (a, b) = v.split_once('x')?;
            Some((a.parse().ok()?, b.parse().ok()?))
        })
        .unwrap_or((1920, 1080));
    forca_no_teste(Some(true));
    // `PH2D_SONDA_VIDRO_ANTIGO`: o borrão deixa o vidro recongelar em todo evento (o de antes do B18.4).
    super::super::watercolor_vidro::VIDRO_SEGUE_A_BASE
        .set(std::env::var("PH2D_SONDA_VIDRO_ANTIGO").is_err());
    // `PH2D_SONDA_CAPTURA_ANTIGA`: o desfazer copia a tela inteira por bytes (o de antes do n.º 6).
    super::super::plane_fork::CAPTURA_PARTILHADA
        .set(std::env::var("PH2D_SONDA_CAPTURA_ANTIGA").is_err());
    super::super::plane_fork::CAPTURA_PELA_CAIXA
        .set(std::env::var("PH2D_SONDA_CAPTURA_ANTIGA").is_err());
    ph2d_wet_paint::solver::CHAO_SEM_REAL_SALTA.store(
        std::env::var("PH2D_SONDA_CHAO_ANTIGO").is_err(),
        std::sync::atomic::Ordering::Relaxed,
    );
    super::super::watercolor_backdrop::FUNDO_EM_CACHE
        .set(std::env::var("PH2D_SONDA_FUNDO_ANTIGO").is_err());
    super::preparo::liga_no_teste(std::env::var("PH2D_SONDA_PREPARO").is_ok());
    // n.º 10 F5j (B18.7): `PH2D_SONDA_ANTES=<n>` dá à sessão o fenómeno (n traços largos antes do «+», a tela de lado a
    // lado); `PH2D_SONDA_COMPRIME_MB=<m>` baixa o limiar do fio só aqui; `PH2D_SONDA_PAUSA_MS=<ms>` troca o preparo de
    // 60 quadros entre o «+» e o toque por esta pausa (em quadros de 16 ms), para o traço cair DENTRO do fio.
    let antes: usize = std::env::var("PH2D_SONDA_ANTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if let Some(mb) = std::env::var("PH2D_SONDA_COMPRIME_MB")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        super::adormecida::COMPRIME_A_PARTIR_NO_TESTE.set(Some(mb << 20));
    }
    let pausa_q: Option<usize> = std::env::var("PH2D_SONDA_PAUSA_MS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .map(|ms| ms.div_ceil(16));
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let nomes = [
        "Watercolor Basic",
        "Wet Wash",
        "Wet Round",
        "Wet Liner",
        "Marker Bullet",
    ];
    // `PH2D_SONDA_VIDRO`: `1` (omissão) a chave `VIDRO_PELO_JOURNAL` ligada, `0` desligada, `ab` as duas
    // intercaladas em blocos rodados (`PH2D_SONDA_REPETE` blocos) no MESMO binário (ADR-0205, A×B).
    let repete: usize = std::env::var("PH2D_SONDA_REPETE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    // `PH2D_SONDA_MAIS` (mesmos valores): o eixo A×B passa a ser a chave `MAIS_SEM_DIFF` do «+» (n.º 7 item 3).
    let eixo_mais = std::env::var("PH2D_SONDA_MAIS").is_ok();
    // `PH2D_SONDA_JANELA` (mesmos valores): o eixo é a chave `JANELA_DA_TINTA_POR_PALAVRAS` (o «adormeceu» do «+»).
    let eixo_janela = std::env::var("PH2D_SONDA_JANELA").is_ok();
    // `PH2D_SONDA_TELA_ZERO` (mesmos valores): o eixo é a chave `TELA_NOVA_PELO_MMAP` (a tela do «+»).
    let eixo_tela = std::env::var("PH2D_SONDA_TELA_ZERO").is_ok();
    // `PH2D_SONDA_ENCOLHE` (mesmos valores): o eixo é a chave `ENCOLHE_NO_SILENCIO` (o fio da compressão no «+»).
    let eixo_encolhe = std::env::var("PH2D_SONDA_ENCOLHE").is_ok();
    // `PH2D_SONDA_AQUECE` (mesmos valores): o eixo é a chave `FUNDO_AQUECIDO_NA_TROCA` (o 1.º pen-down da aguada).
    let eixo_aquece = std::env::var("PH2D_SONDA_AQUECE").is_ok();
    // `PH2D_SONDA_EM_FUNDO` (mesmos valores): o eixo é a chave `FUNDO_AQUECIDO_EM_FUNDO` (o aquecimento num fio, 3f).
    let eixo_em_fundo = std::env::var("PH2D_SONDA_EM_FUNDO").is_ok();
    let eixo = if eixo_em_fundo {
        "PH2D_SONDA_EM_FUNDO"
    } else if eixo_aquece {
        "PH2D_SONDA_AQUECE"
    } else if eixo_encolhe {
        "PH2D_SONDA_ENCOLHE"
    } else if eixo_tela {
        "PH2D_SONDA_TELA_ZERO"
    } else if eixo_janela {
        "PH2D_SONDA_JANELA"
    } else if eixo_mais {
        "PH2D_SONDA_MAIS"
    } else {
        "PH2D_SONDA_VIDRO"
    };
    let rotas: Vec<bool> = match std::env::var(eixo).as_deref() {
        Ok("0") => vec![false; repete],
        Ok("ab") => (0..repete)
            .flat_map(|r| {
                if r % 2 == 0 {
                    [true, false]
                } else {
                    [false, true]
                }
            })
            .collect(),
        _ => vec![true; repete],
    };
    let corridas = nomes
        .iter()
        .filter(|n| so.as_deref().is_none_or(|s| s == **n))
        .flat_map(|n| rotas.iter().map(move |&r| (n, r)));
    for (n, rota) in corridas {
        if eixo_em_fundo {
            super::super::watercolor_aquece::FUNDO_AQUECIDO_EM_FUNDO.set(rota);
            println!("ROTA\t{n}\tfundo_em_fundo={rota}");
        } else if eixo_aquece {
            super::super::watercolor_vidro::FUNDO_AQUECIDO_NA_TROCA.set(rota);
            println!("ROTA\t{n}\tfundo_aquecido={rota}");
        } else if eixo_encolhe {
            super::adormecida::ENCOLHE_NO_SILENCIO.set(rota);
            println!("ROTA\t{n}\tencolhe_no_silencio={rota}");
        } else if eixo_tela {
            crate::tool::layers::TELA_NOVA_PELO_MMAP.set(rota);
            println!("ROTA\t{n}\ttela_pelo_mmap={rota}");
        } else if eixo_janela {
            crate::undo_delta::confine::JANELA_DA_TINTA_POR_PALAVRAS.with(|c| c.set(rota));
            println!("ROTA\t{n}\tjanela_paralela={rota}");
        } else if eixo_mais {
            crate::undo_planes::MAIS_SEM_DIFF.set(rota);
            println!("ROTA\t{n}\tmais_sem_diff={rota}");
        } else {
            crate::undo::window::set_vidro_pelo_journal(rota);
            println!("ROTA\t{n}\tvidro_pelo_journal={rota}");
        }
        let mut t = PainterTool::default();
        // `PH2D_SONDA_PAPEL`: o Textured Paper do dono (sem ele a dobra do relevo não existe).
        if std::env::var("PH2D_SONDA_PAPEL").is_ok() {
            t.agenda_papel_texturizado(7);
            t.bind_document(7, vec![255u8; w * h * 4], w as u32, h as u32);
        } else {
            t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
        }
        t.set_paint_tool_mode("wetpaint");
        t.preset_aplicar(por_nome(&lib, n)).expect("preset");
        t.sonda_raio(60.0);
        for _ in 0..3 {
            t.on_tick(1000.0 / 60.0);
            let _ = t.take_preview_arc();
        }
        // O artista leva mais de um quadro a encostar a caneta.
        if std::env::var("PH2D_SONDA_PREPARO").is_ok() {
            for _ in 0..60 {
                std::thread::sleep(std::time::Duration::from_millis(16));
                t.on_tick(1000.0 / 60.0);
                let _ = t.take_preview_arc();
            }
        }
        if std::env::var("PH2D_SONDA_NASCE").is_ok() {
            let t0 = Instant::now();
            t.ensure_wet_session();
            println!(
                "  nascer a sessão {:.1} ms",
                t0.elapsed().as_secs_f64() * 1e3
            );
            for _ in 0..3 {
                t.on_tick(1000.0 / 60.0);
                let _ = t.take_preview_arc();
            }
        }
        let (mut um, mut dois) = (traco(&mut t, 200.0), traco(&mut t, 500.0));
        for i in 0..antes {
            let y = 80.0 + (i as f32 + 0.5) * (h as f32 - 160.0) / antes as f32;
            let _ = traco_de(&mut t, y, 60.0, (w as f32 - 120.0) / 61.0, 60);
        }
        // `PH2D_SONDA_NOVA`: o 1.º e o 2.º traço numa camada NOVA (aba Layers · «+»), como o dono (B18.4).
        if std::env::var("PH2D_SONDA_NOVA").is_ok() {
            println!(
                "TRACO\t{n}\tbase 1.º máx {:.1} · 2.º máx {:.1}",
                um.iter().copied().fold(0.0, f64::max),
                dois.iter().copied().fold(0.0, f64::max)
            );
            let _ = super::adormecida::ULTIMO_FIO.take();
            let retem = t
                .paint
                .wetpaint
                .session
                .as_ref()
                .map_or(0, |s| s.bytes_retidos());
            println!(
                "  a sessão retém {:.1} MB antes do «+» ({antes} traços largos a mais)",
                retem as f64 / 1e6
            );
            let (f0, t0) = (faltas(), Instant::now());
            crate::sonda_partes::liga();
            let nova = t.add_raster_layer("nova").expect("a camada nova");
            crate::sonda_partes::parte("mais.fim");
            t.select_layer(nova);
            crate::sonda_partes::parte("select_layer");
            for _ in 0..3 {
                t.on_tick(1000.0 / 60.0);
                crate::sonda_partes::parte("tique");
                let _ = t.take_preview_arc();
                crate::sonda_partes::parte("preview");
            }
            if std::env::var("PH2D_SONDA_PARTES").is_ok() {
                for (nome, ms, mb) in crate::sonda_partes::tira() {
                    println!("    «+» {nome} {ms:.2} ms · {mb:.1} MB do fio");
                }
            } else {
                let _ = crate::sonda_partes::tira();
            }
            println!(
                "  «+» rota={rota} {:.1} ms · {:.1} MB novos",
                t0.elapsed().as_secs_f64() * 1e3,
                (faltas() - f0) as f64 * 4096.0 / 1e6
            );
            let fio = super::adormecida::ULTIMO_FIO.take();
            let ms =
                |i: Option<Instant>| i.map_or(-1.0, |i| i.duration_since(t0).as_secs_f64() * 1e3);
            if fio.is_none() {
                println!("  fio da compressão: nenhum (a sessão retém menos do que o limiar)");
            }
            // O artista leva mais de um quadro do «+» ao toque: o preparo corre nesse tempo.
            if pausa_q.is_some() || std::env::var("PH2D_SONDA_PREPARO").is_ok() {
                for _ in 0..pausa_q.unwrap_or(60) {
                    std::thread::sleep(std::time::Duration::from_millis(16));
                    t.on_tick(1000.0 / 60.0);
                    let _ = t.take_preview_arc();
                }
            }
            let t1 = Instant::now();
            (um, dois) = (traco(&mut t, 800.0), traco(&mut t, 1100.0));
            if let Some(h) = &fio {
                let (a, b) = *h.lock().expect("as horas");
                println!(
                    "  fio da compressão: início {:.1} · fim {:.1} ms depois do «+»; os traços na nova de {:.1} a {:.1}",
                    ms(a),
                    ms(b),
                    t1.duration_since(t0).as_secs_f64() * 1e3,
                    t0.elapsed().as_secs_f64() * 1e3
                );
            }
        }
        // `PH2D_SONDA_TROCA` (n.º 10 F5c, B18.3): trocar da camada activa para a de baixo e voltar, as partes de cada.
        if std::env::var("PH2D_SONDA_TROCA").is_ok() {
            let ida = t.layers.active().expect("a activa");
            let baixo = (t.layers.all_ids())
                .find(|&id| id != ida && t.images.contains_key(&id))
                .expect("a de baixo");
            for (rot, alvo) in [
                ("troca", baixo),
                ("volta", ida),
                ("troca", baixo),
                ("volta", ida),
            ] {
                let (f0, t0) = (faltas(), Instant::now());
                crate::sonda_partes::liga();
                t.select_layer(alvo);
                crate::sonda_partes::parte("troca.fim");
                t.on_tick(1000.0 / 60.0);
                crate::sonda_partes::parte("tique");
                let _ = t.take_preview_arc();
                crate::sonda_partes::parte("preview");
                let partes: Vec<String> = crate::sonda_partes::tira()
                    .into_iter()
                    .map(|(nome, ms, mb)| format!("{nome} {ms:.2} ms/{mb:.0} MB"))
                    .collect();
                println!(
                    "TROCA\t{n}\t{rot}\t{:.1} ms · {:.1} MB novos\t{}",
                    t0.elapsed().as_secs_f64() * 1e3,
                    (faltas() - f0) as f64 * 4096.0 / 1e6,
                    partes.join(" · ")
                );
            }
        }
        // Para o perfil: só pen-downs, com o motor na mão.
        let toques: usize = std::env::var("PH2D_SONDA_TOQUES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        for k in 0..toques {
            let p = |phase| CanvasPointer {
                pos: [100.0 + (k % 40) as f32 * 20.0, 700.0],
                pressure: 0.8,
                tilt: [0.0, 0.0],
                phase,
            };
            t.on_canvas_pointer(p(PointerPhase::Down));
            t.on_canvas_pointer(p(PointerPhase::Up));
        }
        let fmt = |v: &[f64]| {
            let c: Vec<String> = v.iter().take(5).map(|x| format!("{x:.1}")).collect();
            let mx = v.iter().copied().fold(0.0, f64::max);
            let mut o = v.to_vec();
            o.sort_by(f64::total_cmp);
            format!(
                "[{}] máx {mx:.1} mediana {:.1}",
                c.join(" "),
                o[o.len() / 2]
            )
        };
        println!(
            "TRACO\t{n}\tjournal={rota}\t1.º {}\t2.º {}",
            fmt(&um),
            fmt(&dois)
        );
    }
    println!(
        "preparo usado (motores, papéis): {:?}",
        super::preparo::usados_no_teste()
    );
    super::preparo::liga_no_teste(false);
    forca_no_teste(None);
    crate::undo::window::set_vidro_pelo_journal(true);
    crate::undo_planes::MAIS_SEM_DIFF.set(true);
    crate::undo_delta::confine::JANELA_DA_TINTA_POR_PALAVRAS.with(|c| c.set(true));
    crate::tool::layers::TELA_NOVA_PELO_MMAP.set(true);
    super::adormecida::ENCOLHE_NO_SILENCIO.set(true);
    super::super::watercolor_vidro::FUNDO_AQUECIDO_NA_TROCA.set(true);
    super::super::watercolor_aquece::FUNDO_AQUECIDO_EM_FUNDO.set(true);
}
