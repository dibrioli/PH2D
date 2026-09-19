//! ⭐⭐⭐ **A BANCADA DO ORÁCULO EXTERNO** — como esta cena se mede contra um traçado de caminhos
//! CONVERGIDO, e como se comparam duas imagens que nasceram de programas diferentes.
//!
//! Irmão de ASSUNTO do [`super`], cortado dele em 2026-09-18 por tecto de LOC (`1455` contra `700`)
//! — ⛔ *split por responsabilidade, nunca uma entrada no `FILE_OVERAGE_OK`* (`CLAUDE.md` §2). O
//! vizinho responde *«a nossa luz atravessa a peça como deve?»*; este responde *«e comparada com
//! quê, medida como?»*.
//!
//! # ⚠️ As leis desta bancada, que são o valor dela
//!
//! - **O oráculo é alimentado pelas PORTAS do produto** (`Orbit::basis`, `eye_distance`,
//!   `lights::opening_light`), nunca por números re-derivados num script: *um oráculo alimentado
//!   com números re-derivados mede outro programa*, e esta linha pagou isso quatro vezes em dois
//!   dias.
//! - **O PFM tem a escala com SINAL** (`> 0` ⇒ big-endian) e as linhas guardadas de baixo para
//!   cima — ⛔ as duas foram lidas ao contrário uma vez, e o sintoma foi um «desvio de 96 %» que
//!   não existia.
//! - **`R/B` compara-se em LINEAR**, onde é invariante à exposição; em BYTES a transformação de
//!   vista é não-linear e duas colunas casadas por exposição podem trocar de ordem.
//! - **A população é casada antes de comparar** ([`casa_a_populacao`]): duas imagens de programas
//!   diferentes não partilham escala de exposição, e comparar bytes sem isso mede o tonemap.
//!
//! ⚠️ **As sondas daqui são todas `#[ignore]`**: elas pedem ficheiros que vivem FORA da árvore
//! (`$PH2D_VERDADE`, `$PH2D_VERDADE2`), porque o arnês de um oráculo amuralhado não mora no repo.

use ph2d_field_render::Orbit;

// ⚠️ O quadro e a moldura dele vivem no PAI: a bancada compara imagens, não as produz.
use super::{H, Quadro, W};

/// ⏱️⭐⭐⭐ **OS NÚMEROS DA NOSSA CENA, para um oráculo externo os reproduzir.**
///
/// ⚠️ Eles saem das PORTAS (`Orbit::basis`, `eye_distance`, `lights::opening_light`), nunca
/// re-derivados num script — *um oráculo alimentado com números re-derivados mede outro programa*,
/// que é o erro que esta linha já pagou quatro vezes em dois dias.
#[test]
#[ignore = "sonda: imprime o enquadramento para o oráculo externo"]
fn sonda_os_numeros_da_cena() {
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let (right, up, fwd) = cam.basis();
    let dist = cam.eye_distance();
    let olho = dist.map(|d| [0, 1, 2].map(|i| cam.target[i] + fwd[i] * d));
    let (onde, luz) = crate::lights::opening_light(&cam);
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    println!("CAM_TARGET={:?}", cam.target);
    println!("CAM_HALF_EXTENT={}", cam.half_extent);
    println!("CAM_RIGHT={right:?}");
    println!("CAM_UP={up:?}");
    println!("CAM_FWD={fwd:?}");
    println!("CAM_EYE={olho:?}");
    println!("CAM_EYE_DIST={dist:?}");
    println!("LAMP_POS={onde:?}");
    println!("LAMP_INTENSITY={}", luz.intensity);
    println!("LAMP_COLOR={:?}", luz.color);
    println!("LAMP_RADIANCE={:?}", crate::lights::radiance_at_one(luz));
    println!("GROUND_Y={:?}", ph2d_field_render::lowest_point(&doc, &reg));
    println!("W={W} H={H}");
}

/// ⭐⭐ **Lê um PFM** — e as DUAS convenções dele partem um consumidor em SILÊNCIO.
///
/// ⛔⛔ **A 1.ª redacção desta função tinha os dois defeitos ao mesmo tempo, e a saída dela era
/// lixo que passava por número:** ela lia `f32::from_le_bytes` sobre dados **BIG-endian**, e o
/// laço que parseia o cabeçalho parava aos **três** campos (`PF`, largura, altura) ⇒ a linha da
/// **escala nunca era lida**, o offset dos dados ficava errado, e o «desvio de forma de `96 %`»
/// que esta sonda imprimiu durante uma jornada inteira era isso.
///
/// As convenções, medidas pelo agente E sobre os ficheiros que ele produziu:
/// - **escala `> 0` ⇒ BIG-endian** (`< 0` ⇒ little);
/// - **as linhas vêm de BAIXO para CIMA** — a primeira linha do ficheiro é a de baixo da imagem.
///
/// ⚠️ **Ela RECUSA em voz alta** em vez de devolver lixo: magia errada, cabeçalho curto, tamanho
/// que não fecha, ou valores não-finitos ⇒ `None`. *Um leitor que devolve lixo plausível é pior que
/// um que falha.*
pub(super) fn le_pfm(caminho: &str) -> Option<(usize, usize, Vec<[f32; 3]>)> {
    let bytes = std::fs::read(caminho).ok()?;
    // Os QUATRO campos do cabeçalho, e não três.
    let mut campos: Vec<String> = Vec::new();
    let mut i = 0usize;
    while campos.len() < 4 {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let ini = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if ini == i {
            return None;
        }
        campos.push(String::from_utf8_lossy(&bytes[ini..i]).to_string());
    }
    i += 1; // o único byte branco a seguir à escala
    if campos[0] != "PF" {
        return None;
    }
    let (w, h): (usize, usize) = (campos[1].parse().ok()?, campos[2].parse().ok()?);
    let escala: f32 = campos[3].parse().ok()?;
    let big = escala > 0.0;
    if i + w * h * 12 != bytes.len() {
        return None;
    }
    let mut px = vec![[0.0f32; 3]; w * h];
    for y in 0..h {
        // ⚠️ A linha `0` do FICHEIRO é a de BAIXO da imagem.
        let linha_no_ficheiro = h - 1 - y;
        for x in 0..w {
            let b = i + (linha_no_ficheiro * w + x) * 12;
            px[y * w + x] = [0, 1, 2].map(|k| {
                let q = [
                    bytes[b + k * 4],
                    bytes[b + k * 4 + 1],
                    bytes[b + k * 4 + 2],
                    bytes[b + k * 4 + 3],
                ];
                if big {
                    f32::from_be_bytes(q)
                } else {
                    f32::from_le_bytes(q)
                }
            });
        }
    }
    px.iter()
        .all(|c| c.iter().all(|v| v.is_finite() && *v >= 0.0))
        .then_some((w, h, px))
}

/// ⏱️⭐⭐⭐ **NÓS CONTRA A VERDADE**, nas condições em que a comparação tem UMA incógnita.
///
/// # ⭐⭐⭐ O desenho, e porque cada cerca está lá
///
/// A 1.ª tentativa comparou as duas imagens com o céu ligado, o chão posto e o especular vivo, e
/// leu `96 %` de desvio de forma — *uma comparação com três incógnitas livres não afirma nada*.
/// Aqui:
///
/// | cerca | porquê |
/// |---|---|
/// | **céu PRETO nos dois** | sobra só a lâmpada, e as duas leis de queda são a MESMA (`1/r²`) |
/// | **sem chão** | o ricochete do chão é luz que o oráculo distribui de outra maneira |
/// | **especular MORTO** | o lóbulo GGX nosso e o do oráculo não são o mesmo, e não é ele que se mede |
/// | **uma escala só** | com o resto casado, um único factor liga os dois lados — *o que sobrar é a LEI* |
///
/// ⚠️ **O oráculo entra pelo NOSSO olhar** (`Look::apply`): comparar um linear com um já tonemapado
/// não afirma nada sobre a forma. E o factor de escala É a `exposure_stops`, logo ajustá-la é
/// ajustar a escala — não há segundo botão.
///
/// ⚠️ A varredura é por **COLUNA** (a borda corre quase na horizontal — §13.4 do `docs/Render3d/10`)
/// e a borda localiza-se pelo canal que a produz (a visibilidade), com o limiar no **meio da faixa
/// medida**: a penumbra da placa nunca chega ao preto, logo `0,5` não é atravessado.
///
/// ⚠️ **As fixturas são acto do agente E** (`$PH2D_VERDADE`), nunca desta janela — INC-R1.
#[test]
#[ignore = "sonda: precisa do oráculo em $PH2D_VERDADE"]
fn sonda_nos_contra_a_verdade() {
    let dir = std::env::var("PH2D_VERDADE").ok();
    let base = ph2d_material::OpenPbr {
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        // ⚠️ O especular morre dos DOIS lados — ver a tabela acima.
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    };
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let (onde, luz) = crate::lights::opening_light(&cam);
    let dump = std::env::var("PH2D_TERM_DUMP").ok();

    for (nome, m) in [
        ("opaco", base),
        (
            "jade",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: false,
                ..base
            },
        ),
    ] {
        let (g, sh, nossos) = super::quadro(&Quadro {
            doc: &doc,
            m,
            cam: &cam,
            onde,
            luz,
            com_sombra: true,
            chao: None,
            sem_ceu: true,
        });
        let (w, h) = (W as usize, H as usize);
        if let Some(d) = &dump {
            let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
            for i in 0..w * h {
                ppm.extend_from_slice(&nossos[i * 4..i * 4 + 3]);
            }
            let _ = std::fs::write(format!("{d}/calib_{nome}.ppm"), ppm);
        }
        let na_bola = |i: usize| g.hit[i] && g.point[i][0] > 0.1;
        const COLUNA: usize = 160;
        let nosso: Vec<(usize, f32)> = (0..h)
            .filter(|&y| na_bola(y * w + COLUNA))
            .map(|y| {
                let b = (y * w + COLUNA) * 4;
                (
                    y,
                    0.2126 * f32::from(nossos[b])
                        + 0.7152 * f32::from(nossos[b + 1])
                        + 0.0722 * f32::from(nossos[b + 2]),
                )
            })
            .collect();
        let (lo, hi) = nosso.iter().fold((f32::MAX, f32::MIN), |(a, b), p| {
            let v = sh.at(0, p.0 * w + COLUNA);
            (a.min(v), b.max(v))
        });
        println!(
            "  {nome}: {} px na coluna · vis de {lo:.3} a {hi:.3} · luminância de {:.1} a {:.1}",
            nosso.len(),
            nosso.iter().map(|p| p.1).fold(f32::MAX, f32::min),
            nosso.iter().map(|p| p.1).fold(f32::MIN, f32::max)
        );
        let Some(dir) = &dir else { continue };
        let meio = 0.5 * (lo + hi);
        let Some(centro) = (hi - lo > 0.05)
            .then(|| {
                nosso.windows(2).find_map(|par| {
                    let (a, b) = (par[0].0, par[1].0);
                    ((sh.at(0, a * w + COLUNA) - meio).signum()
                        != (sh.at(0, b * w + COLUNA) - meio).signum())
                    .then_some(b)
                })
            })
            .flatten()
        else {
            println!("    a coluna não atravessa a borda da sombra");
            continue;
        };
        const MEIA: usize = 45;
        let janela: Vec<usize> = nosso
            .iter()
            .map(|p| p.0)
            .filter(|y| y.abs_diff(centro) <= MEIA)
            .collect();
        let nossa_em = |y: usize| nosso.iter().find(|p| p.0 == y).map_or(0.0, |p| p.1);
        // ⭐ Varre as energias que o E produziu e fica com a que melhor casa: a energia do oráculo
        // e a nossa escala são a MESMA incógnita, e resolvê-la duas vezes seria dar-lhe dois botões.
        let mut melhor: Option<(f32, String, f32, Vec<f32>)> = None;
        for e in [5, 10, 20, 40] {
            let caminho = format!("{dir}/ref_{nome}_e{e}.pfm");
            let Some((ow, oh, linear)) = le_pfm(&caminho) else {
                continue;
            };
            assert_eq!((ow, oh), (w, h), "o oráculo {caminho} tem outro tamanho");
            for passo in -96..96 {
                #[allow(clippy::cast_precision_loss)]
                let stops = passo as f32 * 0.125;
                let olhar = ph2d_view_transform::Look {
                    exposure_stops: stops,
                    ..ph2d_view_transform::Look::default()
                };
                let vals: Vec<f32> = janela
                    .iter()
                    .map(|&y| {
                        let d = olhar.apply(linear[y * w + COLUNA]);
                        255.0
                            * (0.2126 * d[0].clamp(0.0, 1.0)
                                + 0.7152 * d[1].clamp(0.0, 1.0)
                                + 0.0722 * d[2].clamp(0.0, 1.0))
                    })
                    .collect();
                let erro: f32 = janela
                    .iter()
                    .zip(&vals)
                    .map(|(&y, v)| (v - nossa_em(y)).abs())
                    .sum();
                if melhor.as_ref().is_none_or(|b| erro < b.0) {
                    melhor = Some((erro, format!("e{e} {stops:+.2}st"), stops, vals));
                }
            }
        }
        // ⭐ A imagem do oráculo pelo NOSSO olhar, para se poder OLHAR lado a lado — a foto é o
        // árbitro, e foi ela que apanhou as duas curas refutadas desta jornada.
        if let (Some(d), Some((_, _, stops, _))) = (&dump, melhor.as_ref()) {
            for e in [5, 10, 20, 40] {
                let Some((_, _, linear)) = le_pfm(&format!("{dir}/ref_{nome}_e{e}.pfm")) else {
                    continue;
                };
                let olhar = ph2d_view_transform::Look {
                    exposure_stops: *stops,
                    ..ph2d_view_transform::Look::default()
                };
                let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
                for cru in linear.iter().take(w * h) {
                    for canal in olhar.apply(*cru) {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        ppm.push((canal.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
                    }
                }
                let _ = std::fs::write(format!("{d}/verdade_{nome}_e{e}.ppm"), ppm);
            }
        }
        let Some((_, como, _, verdade)) = melhor else {
            println!("    sem oráculo para `{nome}` em {dir}");
            continue;
        };
        let cru_nosso: Vec<f32> = janela.iter().map(|&y| nossa_em(y)).collect();
        let largura = |v: &[f32]| -> f32 {
            let (lo, hi) = v
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            let onde = |f: f32| {
                let alvo = lo + f * (hi - lo);
                v.iter()
                    .enumerate()
                    .min_by(|p, q| (p.1 - alvo).abs().total_cmp(&(q.1 - alvo).abs()))
                    .map_or(0, |p| p.0)
            };
            #[allow(clippy::cast_precision_loss)]
            let d = (onde(0.9) as f32 - onde(0.1) as f32).abs();
            d
        };
        let normaliza = |v: &[f32]| -> Vec<f32> {
            let (lo, hi) = v
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            let d = (hi - lo).max(1e-6);
            v.iter().map(|x| (x - lo) / d).collect()
        };
        let (a, b) = (normaliza(&cru_nosso), normaliza(&verdade));
        let forma: f32 = a
            .iter()
            .zip(&b)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f32::max);
        let bytes: f32 = cru_nosso
            .iter()
            .zip(&verdade)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f32::max);
        println!(
            "    borda em y={centro} · casou com {como} · largura 10–90%: NÓS {:.0} px · VERDADE \
             {:.0} px · desvio da FORMA {:.1} % · pior byte {bytes:.1}",
            largura(&cru_nosso),
            largura(&verdade),
            100.0 * forma
        );
    }
}

/// ⏱️⭐⭐⭐ **A VARREDURA DA COR: nós contra a verdade, em três profundidades.**
///
/// O primeiro veredito mediu UM ponto (`Subsurface Radius = 1,0`) e leu os dois a mover a cor em
/// sentidos opostos. ⛔ *Um desvio medido num ponto é um ponto, não uma lei* ⇒ esta sonda varre
/// três profundidades, e em duas famílias:
///
/// - **`r`**: o raio por canal do produto (`1 : 0,5 : 0,25` escalado) — o que o artista tem;
/// - **`g`**: o mesmo raio nos **três** canais — o controlo que separa *«a cor muda com a
///   PROFUNDIDADE»* de *«a cor muda porque cada canal viaja o seu»*.
///
/// ⚠️ **Os dois lados são lidos no MESMO espaço** (o nosso olhar, com a exposição ajustada por
/// célula): `R/B` em linear e em ecrã não é o mesmo número, e comparar um com o outro seria a
/// quinta régua mal calibrada desta jornada.
///
/// ⚠️ **Piso de ruído do oráculo, medido pelo agente E: `~1e-4` absoluto** (`≈0,2 %` da média) — o
/// Cycles em CPU **não é bit-reprodutível entre invocações**. Nada abaixo disso é lei.
/// ⭐⭐⭐ **A RÉGUA DA COR — `R/B` da região iluminada, em BYTES do NOSSO olhar.**
///
/// ⛔⛔ **Ela NÃO se mede em linear, e a diferença é grande o suficiente para inverter um
/// veredito.** Uma esfera lambertiana de `base_color = (0,75 · 0,35 · 0,35)` sob luz branca devolve
/// `R/B = 0,75/0,35 = 2,143` em LINEAR — foi exactamente o que a janela da Unreal mediu, a quatro
/// dígitos, no controlo opaco dela. O nosso controlo lê **`1,33`** porque a transformação de vista
/// comprime a razão antes de ela chegar ao ecrã. *Comparar um com o outro seria medir a curva de
/// exibição e chamar-lhe material.*
///
/// ⚠️ A máscara é `soma dos três bytes > 30` — a região **iluminada**, e a mesma dos dois lados.
pub(super) fn razao_rb(px: &[u8]) -> (f32, usize) {
    let (mut r, mut b, mut n) = (0.0f64, 0.0f64, 0usize);
    for i in 0..(W as usize) * (H as usize) {
        let q = i * 4;
        let soma = u32::from(px[q]) + u32::from(px[q + 1]) + u32::from(px[q + 2]);
        if soma > 30 {
            r += f64::from(px[q]);
            b += f64::from(px[q + 2]);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((r / b.max(1e-9)) as f32, n)
}

/// Um quadro linear pelo NOSSO olhar, com a exposição dada.
pub(super) fn em_bytes(linear: &[[f32; 3]], stops: f32) -> Vec<u8> {
    let (w, h) = (W as usize, H as usize);
    let olhar = ph2d_view_transform::Look {
        exposure_stops: stops,
        ..ph2d_view_transform::Look::default()
    };
    let mut bytes = vec![0u8; w * h * 4];
    for (i, cru) in linear.iter().take(w * h).enumerate() {
        for (k, canal) in olhar.apply(*cru).into_iter().enumerate() {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                bytes[i * 4 + k] = (canal.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
        }
    }
    bytes
}

/// ⭐⭐ **Põe um quadro de oráculo no nosso olhar com a MESMA POPULAÇÃO iluminada que o nosso.**
///
/// ⛔ Sem isto a máscara mede **regiões diferentes** dos dois lados, e o `R/B` de cada uma é de
/// outra coisa — foi esse o defeito que fez a §14.3 escrever *«a cor move-se no sentido oposto»*
/// sobre um ponto só, e que a §14.4 corrigiu. A exposição é a **única** incógnita livre da
/// comparação (o céu está apagado, e as duas leis de queda são `1/r²`), logo ajustá-la é honesto e
/// ajustar qualquer outra coisa não seria.
///
/// ⚠️ Devolve também os `stops` escolhidos: *um ajuste que não se imprime é um grau de liberdade
/// escondido*.
pub(super) fn casa_a_populacao(linear: &[[f32; 3]], alvo_n: usize) -> (Vec<u8>, f32) {
    let mut melhor = (usize::MAX, 0.0f32);
    for passo in -96..96 {
        #[allow(clippy::cast_precision_loss)]
        let stops = passo as f32 * 0.125;
        let (_, n) = razao_rb(&em_bytes(linear, stops));
        if n.abs_diff(alvo_n) < melhor.0 {
            melhor = (n.abs_diff(alvo_n), stops);
        }
    }
    (em_bytes(linear, melhor.1), melhor.1)
}
#[test]
#[ignore = "sonda: precisa do lote 2 do oráculo em $PH2D_VERDADE2"]
fn sonda_a_varredura_da_cor() {
    let Ok(dir) = std::env::var("PH2D_VERDADE2") else {
        println!("sem $PH2D_VERDADE2 — saltado");
        return;
    };
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let (onde, luz) = crate::lights::opening_light(&cam);
    println!("  raio · família ·   NOSSO R/B ·  VERDADE R/B · o que isso quer dizer");
    for (tag, raio, escala) in [
        ("r010", 0.1f32, [1.0f32, 0.5, 0.25]),
        ("r030", 0.3, [1.0, 0.5, 0.25]),
        ("r100", 1.0, [1.0, 0.5, 0.25]),
        ("g010", 0.1, [1.0, 1.0, 1.0]),
        ("g030", 0.3, [1.0, 1.0, 1.0]),
        ("g100", 1.0, [1.0, 1.0, 1.0]),
    ] {
        let m = ph2d_material::OpenPbr {
            subsurface_weight: 1.0,
            geometry_thin_walled: false,
            subsurface_color: [0.75, 0.35, 0.35],
            base_color: [0.75, 0.35, 0.35],
            specular_weight: 0.0,
            subsurface_radius: raio,
            subsurface_radius_scale: escala,
            ..ph2d_material::OpenPbr::default()
        };
        let (_, _, nossos) = super::quadro(&Quadro {
            doc: &doc,
            m,
            cam: &cam,
            onde,
            luz,
            com_sombra: true,
            chao: None,
            sem_ceu: true,
        });
        let (nosso_rb, nosso_n) = razao_rb(&nossos);
        let Some((_, _, linear)) = le_pfm(&format!("{dir}/ref_jade_{tag}_e5.pfm")) else {
            println!("  {tag}: sem oráculo");
            continue;
        };
        let (bytes, _stops) = casa_a_populacao(&linear, nosso_n);
        let (verdade_rb, verdade_n) = razao_rb(&bytes);
        println!(
            "  {:.2} · {} · {nosso_rb:>10.2} · {verdade_rb:>11.2} · {} ({nosso_n} vs {verdade_n} px)",
            raio,
            if escala[1] < 1.0 {
                "por canal"
            } else {
                "IGUAIS  "
            },
            if (nosso_rb - verdade_rb).abs() < 0.15 {
                "concordam"
            } else if (nosso_rb - 1.0).signum() == (verdade_rb - 1.0).signum() {
                "mesmo sentido, magnitude diferente"
            } else {
                "SENTIDOS OPOSTOS"
            }
        );
    }
}
