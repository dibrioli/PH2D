//! A MEMÓRIA da lei da largura (Dabber, L13) e as cópias do Wet (spray, simetria) — partidas de
//! `presets_de_fabrica.rs` pelo tecto de 700 linhas.

use super::*;

/// O pico de memória do processo (`VmHWM`), em MiB.
pub(super) fn pico_mib() -> f64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find_map(|l| l.strip_prefix("VmHWM:"))
        .and_then(|r| r.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0)
        / 1024.0
}

/// A memória residente do processo AGORA (`VmRSS`), em bytes.
fn rss() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find_map(|l| l.strip_prefix("VmRSS:"))
        .and_then(|r| r.split_whitespace().next()?.parse::<u64>().ok())
        .unwrap_or(0)
        * 1024
}

/// O tecto de memória do processo (bytes): o menor `memory.max` do cgroup dele e dos pais (a fatia do `ph2d-run.sh`),
/// menos o que ele já ocupa; sem cgroup com tecto, a `MemAvailable` da máquina.
fn memoria_livre() -> u64 {
    let ler = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
    let campo = |txt: &str, k: &str| -> Option<u64> {
        txt.lines()
            .find_map(|l| l.strip_prefix(k))
            .and_then(|r| r.split_whitespace().next()?.parse::<u64>().ok())
    };
    let mut grupo = ler("/proc/self/cgroup")
        .lines()
        .find_map(|l| l.strip_prefix("0::").map(str::to_owned))
        .unwrap_or_default();
    let mut livre = None::<u64>;
    loop {
        let d = format!("/sys/fs/cgroup{grupo}");
        if let (Ok(max), Some(atual)) = (
            ler(&format!("{d}/memory.max")).trim().parse::<u64>(),
            ler(&format!("{d}/memory.current"))
                .trim()
                .parse::<u64>()
                .ok(),
        ) {
            let l = max.saturating_sub(atual);
            livre = Some(livre.map_or(l, |v| v.min(l)));
        }
        match grupo.rfind('/') {
            Some(k) if !grupo.is_empty() => grupo.truncate(k),
            _ => break,
        }
    }
    livre.unwrap_or_else(|| campo(&ler("/proc/meminfo"), "MemAvailable:").unwrap_or(0) * 1024)
}

/// Os bytes por píxel de CENA de uma medição da largura em voo (o pico do processo ÷ os píxeis da tela), medido 03/11
/// num processo por preset, um raio de cada vez (`sonda_monotonia_da_largura`, `PH2D_LEI_FIOS=1`, `VmHWM`, motor 1 100,
/// cena 3 200 × 6 680 = 21,4 Mpx): Water Brush 7 746 MiB = 380 B/px (a 1 099: 7 585). Arredondado para cima: a
/// estimativa erra só para o lado seguro. Os 64 raios de uma vez somam 150 Mpx = 7× o maior (≈ 53 GiB + o fixo): o OOM.
const BYTES_POR_PX_DA_CENA: u64 = 400;
/// A parte da [`memoria_livre`] que as medições em voo podem somar (o resto: o binário, as tabelas, a miniatura).
const FRACAO_DO_LIVRE: f64 = 0.7;

/// O custo (bytes) de UMA medição da largura de `p` a `raio` em voo: [`regua_da_largura::px_da_cena`] ×
/// [`BYTES_POR_PX_DA_CENA`].
pub(super) fn custo_da_cena(p: &Preset, raio: f32, crua: bool) -> u64 {
    super::regua_da_largura::px_da_cena(p, raio, crua) * BYTES_POR_PX_DA_CENA
}

/// ⭐ **As medições da largura em paralelo PELA MEMÓRIA** (02/11c): `f(i)` para cada `i`, no máximo `fios` de cada vez,
/// do maior custo para o menor; o resultado é por índice e cada `f(i)` é independente, por isso o que se grava é o
/// mesmo com qualquer concorrência. Uma medição entra (a primeira em voo entra sempre) se:
/// - a soma dos `custos` estimados em voo fica ≤ [`FRACAO_DO_LIVRE`] da [`memoria_livre`] do início; e
/// - a memória livre REAL do cgroup cobre a nova, mais TUDO o que está em voo voltar a crescer, mais 5 %.
///
/// ⛔ Medido 03/11 (Water Brush, `sonda_monotonia_da_largura`): um fio por raio (64 de uma vez) pôs a lei crua a 66 G.
/// Só com o estimado (1.ª cura) morreu a 48 G: os seis maiores em voo picaram 33,7 G para 33,2 estimados, mas
/// depois de saírem o `VmRSS` ficou em 23,4 G com zero em voo — o malloc guarda as páginas na arena de cada fio, e a
/// medição seguinte, noutro fio, não as reaproveita. A medida real conta essa memória guardada; e quem pega a
/// medição seguinte é, em geral, o fio que acabou de soltar a sua (já acordado; a arena dele tem as páginas).
pub(super) fn em_paralelo_pela_memoria<T: Send>(
    custos: &[u64],
    fios: usize,
    f: impl Fn(usize) -> T + Sync,
) -> Vec<T> {
    use std::sync::{Condvar, Mutex};
    let livre0 = memoria_livre();
    let orcamento = (livre0 as f64 * FRACAO_DO_LIVRE) as u64;
    let reserva = livre0 / 20;
    // A 1.ª em voo entra sempre: a que SOZINHA passa da fatia morria por OOM — falha alto, com a conta.
    if let Some((i, &c)) = custos
        .iter()
        .enumerate()
        .find(|&(_, &c)| c + reserva > livre0)
    {
        panic!(
            "[memoria] a medição {i} sozinha não cabe na fatia: {} MiB estimados ({BYTES_POR_PX_DA_CENA} B/px) + \
             {} MiB de reserva > {} MiB livres — suba PH2D_MEM_MAX",
            c >> 20,
            reserva >> 20,
            livre0 >> 20
        );
    }
    let cabe = |em_voo: u64, c: u64| {
        em_voo == 0 || (em_voo + c <= orcamento && memoria_livre() >= c + em_voo + reserva)
    };
    let mut ordem: Vec<usize> = (0..custos.len()).collect();
    ordem.sort_by_key(|&i| std::cmp::Reverse(custos[i]));
    // (o próximo da `ordem`, a soma estimada em voo)
    let estado = (Mutex::new((0usize, 0u64)), Condvar::new());
    // Devolve o custo mesmo se `f` entrar em pânico (senão os outros fios esperavam para sempre).
    struct Vaga<'a>(&'a (Mutex<(usize, u64)>, Condvar), u64);
    impl Drop for Vaga<'_> {
        fn drop(&mut self) {
            let mut g = self
                .0
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            g.1 -= self.1;
            self.0.1.notify_all();
        }
    }
    let mut saida: Vec<Option<T>> = (0..custos.len()).map(|_| None).collect();
    std::thread::scope(|s| {
        let h: Vec<_> = (0..fios.clamp(1, custos.len().max(1)))
            .map(|_| {
                s.spawn(|| {
                    let mut feitas = Vec::new();
                    loop {
                        let mut g = estado.0.lock().unwrap();
                        let i = loop {
                            let Some(&i) = ordem.get(g.0) else {
                                return feitas;
                            };
                            if cabe(g.1, custos[i]) {
                                break i;
                            }
                            g = estado.1.wait(g).unwrap();
                        };
                        g.0 += 1;
                        g.1 += custos[i];
                        eprintln!(
                            "[memoria] entra {i}: {} MiB, {} MiB em voo, {} MiB livres, VmRSS {} MiB",
                            custos[i] >> 20,
                            g.1 >> 20,
                            memoria_livre() >> 20,
                            rss() >> 20
                        );
                        drop(g);
                        let _vaga = Vaga(&estado, custos[i]);
                        feitas.push((i, f(i)));
                    }
                })
            })
            .collect();
        for (i, v) in h.into_iter().flat_map(|h| h.join().unwrap()) {
            saida[i] = Some(v);
        }
    });
    saida.into_iter().map(Option::unwrap).collect()
}

/// **Sonda da MEMÓRIA da lei da largura** (J2 Dabber, L13): UM raio por processo (`VmHWM` é do processo), o traço
/// de [`largura_de`] com `PH2D_SONDA_PRESET` (Dabber), `PH2D_SONDA_RAIO`, `PH2D_SONDA_CRUA` (1), `PH2D_SONDA_JITTER`
/// (o do preset) e `PH2D_SONDA_QUADROS` (os da lei; `0` = só o gesto). Imprime o pico depois do gesto e depois da
/// água, as lanes e as janelas. O portão [`a_memoria_do_dabber_cabe`] corre-a num processo filho.
#[test]
#[ignore = "sonda: a memória da lei da largura, um raio por processo"]
fn sonda_memoria_da_lei() {
    use ph2d_editor_core::{Tool, tool::RasterEditTool};
    let var = |k: &str| std::env::var(k).ok();
    let nome = var("PH2D_SONDA_PRESET").unwrap_or_else(|| "Dabber".into());
    let raio: f32 = var("PH2D_SONDA_RAIO").map_or(300.0, |v| v.parse().unwrap());
    let crua = var("PH2D_SONDA_CRUA").as_deref() != Some("0");
    let quadros: u32 = var("PH2D_SONDA_QUADROS").map_or(QUADROS_DA_LEI, |v| v.parse().unwrap());
    let mut p = ficheiros_do_wet()
        .into_iter()
        .map(|(_, b)| ler(&b).unwrap())
        .find(|p| p.parametros.name == nome)
        .expect("o preset de fábrica");
    // `PH2D_SONDA_RECEITA=1`: o preset da RECEITA de hoje (antes de o regerar), com a tabela do ficheiro.
    if var("PH2D_SONDA_RECEITA").as_deref() == Some("1") {
        let r = receitas().find(|r| r.nome == nome).expect("a receita");
        let tabela = std::mem::take(&mut p.parametros.width_table);
        p.parametros = parametros(r);
        p.parametros.width_table = tabela;
    }
    // `PH2D_SONDA_OLEO=0`: o mesmo preset sem a caixa Real Oil (J6: o custo do óleo é a diferença).
    if var("PH2D_SONDA_OLEO").as_deref() == Some("0") {
        p.parametros.engine.remove(CHAVE_REAL_OIL);
    }
    if let Some(j) = var("PH2D_SONDA_JITTER") {
        p.parametros.brush.insert(
            "jitter".into(),
            serde_json::json!(j.parse::<f64>().unwrap()),
        );
    }
    let escala = if crua { (raio / 500.0).max(1.0) } else { 1.0 };
    let h = ((raio * 6.0) as u32 + 80).max(120);
    let w = crate::sonda_gestos::W;
    let cy = h as f32 * 0.5;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&direito(&p)).is_empty());
    if crua {
        t.set_wet_largura_crua(escala);
    }
    t.set_brush_size_px(raio / escala);
    t.set_brush_color_srgb8(crate::sonda_gestos::AZUL);
    let t0 = std::time::Instant::now();
    crate::sonda_gestos::gesto(&mut t, &[[100.0, cy], [900.0, cy]], 3.0, |_| 1.0);
    let gesto = pico_mib();
    let mut janelas = Vec::new();
    t.wet_sonda_motor(|e| {
        janelas = e
            .lane_trails
            .iter()
            .map(ph2d_wet_paint::trail::Trail::window_half_for_measure)
            .collect();
    });
    for _ in 0..quadros {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, pw, ph) = t.run_full();
    println!(
        "[memoria] {nome} raio {raio} crua {crua} · tela {w}×{h} · gesto {gesto:.0} MiB · água {:.0} MiB · lanes {} \
         · meias-larguras {janelas:?} · largura {:.1} · {:.1} s",
        pico_mib(),
        janelas.len(),
        largura_que_se_ve(&px, pw, ph),
        t0.elapsed().as_secs_f64()
    );
}

/// ⭐ O binário que corre AGORA, para o re-executar num filho. Nunca o CAMINHO (`current_exe`): o cargo religa o mesmo
/// `it-<hash>` (outra corrida, o `exame.sh` ao lado) gravando um ficheiro novo por cima, o inode deste processo fica
/// «(deleted)» e o filho dava `NotFound` — ou, pior, corria o código NOVO. `/proc/self/exe` abre o inode que corre.
fn o_proprio_binario() -> std::path::PathBuf {
    if cfg!(target_os = "linux") {
        "/proc/self/exe".into()
    } else {
        std::env::current_exe().unwrap()
    }
}

/// O pico do GESTO (MiB) e as lanes da [`sonda_memoria_da_lei`] num processo filho (o `VmHWM` é do processo).
fn memoria_num_filho(raio: f32) -> (f64, usize) {
    let out = std::process::Command::new(o_proprio_binario())
        .args([
            "presets_de_fabrica::memoria::sonda_memoria_da_lei",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env("PH2D_SONDA_RAIO", raio.to_string())
        .env("PH2D_SONDA_QUADROS", "0")
        .env("MALLOC_ARENA_MAX", "1")
        .env_remove("PH2D_SONDA_JITTER")
        .env_remove("PH2D_SONDA_PRESET")
        .output()
        .expect("o processo filho da sonda");
    let txt = String::from_utf8_lossy(&out.stdout);
    let linha = txt
        .split("[memoria]")
        .nth(1)
        .unwrap_or_else(|| panic!("a sonda não respondeu a raio {raio}:\n{txt}"));
    let num = |k: &str| -> f64 {
        linha
            .split(k)
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap()
    };
    println!("[memoria]{}", linha.lines().next().unwrap_or(""));
    (num(" gesto "), num(" lanes ") as usize)
}

/// Os tectos do PICO do gesto do Dabber na lei crua da largura (a do gerador), em MiB, medidos 20/10 (L13, smoke,
/// um processo por raio; 03_MOTORES §4 L13):
///
/// | raio | limiar `r` (o mutante: 4 lanes) | uma ponta = uma lane | tecto |
/// |---|---|---|---|
/// | 300  | 298 | 165 | 230 |
/// | 1000 | 2 966 | 1 221 | 1 800 |
///
/// (O Dabber de antes — lâmina, Jitter 0,5 — com o limiar `r`: 467 a raio 300 e 3 955 a 1 100, 6–7 lanes.)
///
/// J6 (dono 28/10): o Dabber de fábrica traz o **Real Oil** — o campo do óleo soma ~+60 B por célula ao gesto. Medido
/// 28/10 (smoke, um processo por raio, `PH2D_SONDA_OLEO`):
///
/// | raio | sem óleo | com óleo | mutante (limiar `r`) com óleo | tecto |
/// |---|---|---|---|---|
/// | 300  | 152 | 270 | 309 (2 lanes) | 290 |
/// | 1000 | 940 | 1 513 | 2 029 (2 lanes) | 1 800 |
///
/// ⚠️ Essa tabela é do perfil `smoke`; o portão corre no `ci-test` (`nextest-impacted.sh`), onde o MESMO código pesa mais
/// (em `d94e19a57e`, o commit do tecto 290: 303 MiB — o portão nasceu vermelho no perfil dele). Medido 29/10 no HEAD da
/// J6 (RAM do pico do gesto, um processo por raio):
///
/// | raio | ci-test sem óleo | ci-test com óleo | smoke com óleo | mutante (limiar `r`) ci-test | tecto |
/// |---|---|---|---|---|---|
/// | 300  | 165 | 295 | 286 | 322 (2 lanes) | 310 |
/// | 1000 | 968 | 1 563 | 1 497 | 2 058 (2 lanes) | 1 800 |
///
/// E10 (`e8f35b5710`, a lei única de pigmento, ADR-0197): bissecção na linha (integ 02/11, ci-test, raio 300, sonda sem
/// óleo) — `171` MiB no pai, `317` nele; nada mais mexe o pico de forma comparável. O Dabber migrou para Medium = 1 (o
/// óleo VIVE na célula: a grade do óleo existe sem a caixa Real Oil) e a `OleoCelula` passou de 16 para 24 B. É o preço
/// da lei aprovada pelo dono, não um rascunho por preguiçar. Medido 02/11 (ci-test, um processo por raio, 2 rodadas
/// intercaladas com o binário do merge-base `422fec2027`, load 11–22; o RECURSO é a RAM do processo, `VmHWM`):
///
/// | raio | base sem óleo | base com óleo | integ (`2795e01fe4`+) sem óleo | integ com óleo | tecto |
/// |---|---|---|---|---|---|
/// | 300  | 158 | 278 | 222–225 | 320–325 | 340 |
/// | 1000 | — | — | 1 140 | 1 861 | 1 950 |
///
/// 03/11: com as arenas do glibc o pico não é uma medida. Raio 1000, mesma lei: 1 673–2 053 MiB, 3 de 13 corridas acima
/// do tecto 1 950, inclusive a load 12 e no `main`. Com `MALLOC_ARENA_MAX=1` no filho, intercalado, load 4–9, 5/5:
/// 305–306 MiB a raio 300 e 1 607–1 608 a raio 1000; seis cópias em paralelo, load até 84, 12/12: 306–308 e 1 608–1 610.
/// O tecto é o medido calmo + 5 %.
///
/// ⚠️ O tecto deixou de separar o mutante do limiar `r` (322 MiB em 29/10 < o pico de hoje): quem o pega é a asserção
/// `lanes == 1` (o mutante parte o traço em 2 lanes), que corre ANTES do tecto.
///
/// Dono 05/11: o Smooth Edges 0,3 de fábrica do Dabber aloca o campo da aparência e o plano do alisamento. Remedido 3×,
/// load 10–13: 367–368 MiB (era 305–306) e 1 801–1 802 (era 1 607–1 610). O tecto é esse + 5 %.
///
/// 07/11 (integração da tinta viva): o pico do gesto passou a 490 MiB a raio 300 (2 corridas, +1 na suíte: 490–492) e
/// 2 071 a raio 1000 (2 corridas, idênticas), `MALLOC_ARENA_MAX=1`, load 3–8, `--ignored` pelo binário `it` do `ci-test`.
/// Causa (o registo medido; que a base não o abria é leitura do código, a base não foi remedida): o 1.º traço de uma sessão
/// passou a abrir o registo do undo no pen-down (`wet_acorda` antes de `wet_guarda_o_passo`, `stroke_lifecycle.rs`; antes
/// não havia sessão nesse instante e o passo não guardava nada). O registo do gesto retém 117 MiB a raio 300
/// (`bytes_do_historico` depois do gesto; a grade tem 1 000 × 1 880 células × ~63 B = 118 MiB). É o preço de o desfazer do
/// 1.º traço não matar a tinta. O recurso é a RAM do processo (`VmHWM`). O tecto é o medido + 5 % (515 e 2 175); se o custo
/// se cura (marcar só a caixa do que o traço escreve) o tecto desce — decisão do dono.
///
/// 14/11 (integração 13/11, missão 4K do Painter): o rascunho do campo do Real Oil (B14.A1, `2f22e2dd1e` +
/// `2f580f0225`, `RascunhoCampo` nos `PlanosDaOtica`) RETÉM os planos `f64` do campo entre imagens em vez de os
/// devolver ao SO — é o ganho medido da linha (campo 1440², faixas do rayon: 44,0–47,4 → 22,2–28,9 ms por imagem).
/// Sem a caixa Real Oil (`PH2D_SONDA_OLEO=0`) a integração não paga nada (400 × 405 MiB no main). O recurso é a RAM
/// do processo (`VmHWM`). Medido intercalado com o binário do main `4f46a3573c`, smoke, `MALLOC_ARENA_MAX=1`, load 20–40:
///
/// | raio | main com óleo | integ com óleo | main sem óleo | integ sem óleo | tecto |
/// |---|---|---|---|---|---|
/// | 300  | 478–482 | 532–573 | 405–406 | 400 | 605 |
/// | 1000 | 2 045–2 048 | 2 232–2 233 | — | — | 2 345 |
///
/// O tecto é o maior medido + 5 %; o pico varia com a janela do campo retida no fim do gesto (532 × 573).
///
/// 14/11 (rodada 4): a raio 1000 o pico SOBE COM A CARGA — o rascunho é retido por operário do rayon, e sob contenção
/// mais operários tocam o campo. Não é o papel em duas leituras (`5618e94ace`): isolado, o HEAD dá o mesmo da tabela acima.
/// Binário `it` do `ci-test` (o perfil da suíte), `MALLOC_ARENA_MAX=1` no filho:
///
/// | raio | isolado, smoke (load 14) | isolado, ci-test (load 9) | 3 em paralelo, ci-test (load 6–9) | suíte (load 30+) | tecto |
/// |---|---|---|---|---|---|
/// | 300  | 536 | 539 | 565–570 | — | 605 |
/// | 1000 | 2 186 | 2 234 | 2 261–2 286 | 2 359 | 2 477 |
///
/// ABERTO (BUGS_painter #115): reter um rascunho por gesto, não por operário, devolve o pico ao isolado e o tecto desce.
const TECTO_DA_MEMORIA_MIB: [(f32, f64); 2] = [(300.0, 605.0), (1000.0, 2477.0)];

/// ⭐ **A MEMÓRIA do Dabber cabe** (L13, EXAME §3.6): o Jitter espalha UMA ponta — o traço inteiro numa lane — e o
/// pico do gesto a raio 300 e 1000 fica abaixo do tecto medido. Mutação: o limiar da lane sem o Jitter
/// (`BrushSpec::limiar_da_lane` = `r`) parte o traço em lanes e passa os dois tectos.
#[test]
fn a_memoria_do_dabber_cabe() {
    for (raio, tecto) in TECTO_DA_MEMORIA_MIB {
        let (pico, lanes) = memoria_num_filho(raio);
        println!("MEMORIA raio {raio}: pico {pico:.0} MiB, tecto {tecto:.0}");
        assert_eq!(
            lanes, 1,
            "raio {raio}: o Jitter partiu o traço em {lanes} lanes"
        );
        assert!(
            pico <= tecto,
            "raio {raio}: pico do gesto {pico:.0} MiB > tecto {tecto:.0}"
        );
    }
}

/// O que [`wet_com_copias`] mede: a tinta por metade (esquerda, direita), as lanes, os dabs e os que pousaram com
/// peso ZERO (a opacidade do dab no motor, `Dab::opacity`, já com o pigmento por comprimento).
struct Copias {
    tinta: [f64; 2],
    lanes: usize,
    dabs: usize,
    sem_pigmento: usize,
}

/// O Wet Round (o pincel que nasce sem preset: `pigmentSpacing` > 0, o pigmento por comprimento) com Jitter 0,5,
/// o Count e a simetria dados, um traço de `pts` a raio 16.
fn wet_com_copias(count: u32, simetria: bool, pts: &[[f32; 2]]) -> Copias {
    use ph2d_editor_core::{Tool, tool::RasterEditTool};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (w, h) = (crate::sonda_gestos::W, crate::sonda_gestos::H);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_paint_media(PaintMedia::WetPaint);
    let mut p = t
        .wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "g", "copias"))
        .0;
    p.brush.insert("jitter".into(), serde_json::json!(0.5));
    p.brush
        .insert("spray_count".into(), serde_json::json!(count));
    let preset = Preset {
        parametros: p,
        origem: Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
        texturas: Default::default(),
    };
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&preset).is_empty());
    if simetria {
        t.toggle_symmetry_enabled();
    }
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8(crate::sonda_gestos::AZUL);
    let (dabs, zero) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (d2, z2) = (Arc::clone(&dabs), Arc::clone(&zero));
    t.wet_sonda_motor(|e| {
        e.on_dab = Some(Box::new(move |_b, d| {
            d2.fetch_add(1, Ordering::Relaxed);
            if d.opacity <= 1e-9 {
                z2.fetch_add(1, Ordering::Relaxed);
            }
        }));
    });
    crate::sonda_gestos::gesto(&mut t, pts, 3.0, |_| 1.0);
    let mut lanes = 0;
    t.wet_sonda_motor(|e| lanes = e.lane_trails.len());
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, pw, _) = t.run_full();
    let mut tinta = [0.0; 2];
    for (i, c) in px.as_chunks::<4>().0.iter().enumerate() {
        tinta[usize::from(i as u32 % pw >= pw / 2)] += f64::from(255 - c[0]);
    }
    Copias {
        tinta,
        lanes,
        dabs: dabs.load(Ordering::Relaxed),
        sem_pigmento: zero.load(Ordering::Relaxed),
    }
}

/// ⭐ **Cada marca do spray é uma ponta** (L13): no Wet com Count 3 e Jitter, as três marcas de cada ponto
/// pousam pigmento — cada uma na sua lane. Com a lane só pela distância (`r + 2·alcance`) as três caíam
/// na mesma e a 2.ª e a 3.ª, sem caminho andado, pousavam com peso 0 (o mutante: 3 → 1 lane, 250 de 2 100 dabs sem
/// pigmento; a tinta do traço ×2,50 → ×2,23 da do Count 1, porque o rasto leva a tinta de uma marca à outra).
#[test]
fn cada_marca_do_spray_pousa_no_wet() {
    let pts = [[150.0, 320.0], [850.0, 320.0]];
    let (um, tres) = (
        wet_com_copias(1, false, &pts),
        wet_com_copias(3, false, &pts),
    );
    let razao = (tres.tinta[0] + tres.tinta[1]) / (um.tinta[0] + um.tinta[1]);
    println!(
        "[copias] spray: Count 1 {} lane(s) · Count 3 {} lanes, {} de {} dabs sem pigmento · tinta ×{razao:.2}",
        um.lanes, tres.lanes, tres.sem_pigmento, tres.dabs
    );
    assert_eq!(um.lanes, 1, "Count 1 com Jitter: uma ponta, uma lane");
    assert_eq!(tres.lanes, 3, "Count 3: uma lane por marca");
    assert_eq!(
        tres.sem_pigmento, 0,
        "{} de {} marcas não pousaram pigmento",
        tres.sem_pigmento, tres.dabs
    );
    assert!(razao >= 2.0, "Count 3 pousa ×{razao:.2} a tinta do Count 1");
}

/// ⭐ **As duas cópias da simetria pintam as duas** (L13): o traço vertical a 12 px do eixo, com Jitter — as cópias
/// caem a < `r + 2·alcance` uma da outra e, só pela distância, partilhavam a lane: o rasto saltava entre elas e
/// dabs pousavam com peso 0. Mutação (a cópia ignorada): 1 lane, 71 de 790 dabs sem pigmento — sangra.
#[test]
fn as_copias_da_simetria_pintam_as_duas_no_wet() {
    let x = crate::sonda_gestos::W as f32 / 2.0 + 12.0;
    let c = wet_com_copias(1, true, &[[x, 120.0], [x, 520.0]]);
    let r = c.tinta[0].min(c.tinta[1]) / c.tinta[0].max(c.tinta[1]);
    println!(
        "[copias] simetria: {} lanes, {} de {} dabs sem pigmento · tinta esquerda {:.0} · direita {:.0} (razão {r:.3})",
        c.lanes, c.sem_pigmento, c.dabs, c.tinta[0], c.tinta[1]
    );
    assert_eq!(c.lanes, 2, "a simetria: uma lane por cópia");
    assert_eq!(
        c.sem_pigmento, 0,
        "{} de {} dabs da simetria não pousaram pigmento",
        c.sem_pigmento, c.dabs
    );
    // As cópias sobrepõem-se a 12 px do eixo e a 2.ª pousa sobre a tinta molhada da 1.ª: a razão é física, não 1.
    assert!(r >= 0.85, "uma cópia da simetria pinta {r:.3} da outra");
}
