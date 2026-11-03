//! **A régua da largura de uma FERRAMENTA** (J6 fecho B4): a imagem do que ela muda, a largura EQUIVALENTE (área ÷ pico
//! por coluna — contínua no raio) e as sementes do sorteio. A tabela (o gerador) e a promessa (o portão) passam as duas
//! por aqui: [`sementes_da_largura`] e a régua são as mesmas dos dois lados.

use super::*;

/// As sementes dos traços da régua para um pincel SEM sorteio — `0`, a do `PainterTool::default` (neutra ao byte).
const SEMENTE_UNICA: [u64; 1] = [0];

/// As sementes de uma FERRAMENTA com Jitter: a largura é a mediana delas. N = 3, medido 29/10 (`sonda_sorteio_da_largura`,
/// Smear Shattered, nova régua, 9 sementes, raios da promessa 2…300): UMA semente cai até ±10 % da mediana das 9; a
/// mediana das 3 primeiras fica a ≤ 4 % dela (26: 48,6 × 46,8; 13: 22,2 × 21,7; 300: 492 × 490), a das 5 primeiras
/// também (≤ 4 %) — 5 não compra precisão e custa 5/3 do gerador (cada semente repinta a lei inteira).
const SEMENTES_DO_SORTEIO: [u64; 3] = [0, 1, 2];

/// As sementes da régua da largura de `p` — a MESMA lista na tabela e na promessa. Uma ferramenta (o modo ≠ Paint) cuja
/// largura a semente MOVE ([`a_semente_move_a_largura`] — o Jitter, e também a textura que a semente desloca: o Blender
/// Textured tem jitter 0) mede a mediana de [`SEMENTES_DO_SORTEIO`] (Smear Shattered a raio 13 → 21–26 px, a 120 →
/// 214–267 conforme a semente; `sonda_sorteio_da_largura`, 29/10). Os pincéis (modo Paint) medem a mediana das colunas
/// de um traço longo, onde o sorteio já se média ao longo dele: uma semente.
pub(super) fn sementes_da_largura(p: &Preset) -> &'static [u64] {
    let ferramenta = p
        .parametros
        .paint_mode
        .as_deref()
        .is_some_and(|m| m != "Paint");
    if ferramenta && a_semente_move_a_largura(p) {
        &SEMENTES_DO_SORTEIO
    } else {
        &SEMENTE_UNICA
    }
}

/// O raio do MOTOR (lei crua — não depende da tabela, a mesma antes e depois de regerar) da prova de
/// [`a_semente_move_a_largura`]: onde o piso de 2 px da promessa e o píxel inteiro da régua dos pincéis já pesam menos
/// que os 2 %. Medido 29/10 (`sonda_sensibilidade_a_semente`, 44 presets do Wet, sementes 0‥2, dispersão = (máx − mín)
/// ÷ mediana): a 150 — Smear Shattered 7,7 %, Blender Dots 2,8 %, Blender Textured 2,2 %; os grafites e o Charcoal
/// 2,4–6,4 % (já do grão); Scratch 3,2 %; os outros pincéis ≤ 1,9 % (Dip Pen 1,93, Oil Pastel 1,80) e as outras 6
/// ferramentas 0. ⛔ Recusa MEDIDA: a 55 com o piso de 2 px o Blender Dots ficava de fora (1,64 px, 2,5 %) e a semente
/// 7 levava-o a −5,0 % a raio 300 (mutação (i)); a 13 nenhum passava.
const RAIO_DA_PROVA_DA_SEMENTE: f32 = 150.0;

/// ⭐ **A semente MOVE a largura de `p` além da tolerância fina** — MEDIDO, não lido da receita: a lei crua a
/// [`RAIO_DA_PROVA_DA_SEMENTE`] com cada uma das [`SEMENTES_DO_SORTEIO`]; a dispersão (máx − mín) ÷ mediana passa de
/// `TOLERANCIA_REL`. Uma régua não mede mais fino que o seu próprio ruído: o pincel ou a ferramenta assim é da classe
/// do GRÃO ([`tolerancia_da_largura`]) e, se for ferramenta, mede várias sementes ([`sementes_da_largura`]). Uma
/// medição por preset e por processo (a tabela e a promessa leem a mesma).
pub(super) fn a_semente_move_a_largura(p: &Preset) -> bool {
    use std::sync::{Arc, Mutex, OnceLock};
    static MEDIDAS: Mutex<BTreeMap<String, Arc<OnceLock<bool>>>> = Mutex::new(BTreeMap::new());
    let celula = MEDIDAS
        .lock()
        .unwrap()
        .entry(p.parametros.name.clone())
        .or_default()
        .clone();
    *celula.get_or_init(|| {
        let ls: Vec<f32> = std::thread::scope(|s| {
            let h: Vec<_> = SEMENTES_DO_SORTEIO
                .iter()
                .map(|&semente| {
                    s.spawn(move || {
                        let (px, w, h, regua) = imagem_da_largura(
                            p,
                            RAIO_DA_PROVA_DA_SEMENTE,
                            true,
                            Relogio::DeUmaVez,
                            semente,
                        );
                        regua(&px, w, h)
                    })
                })
                .collect();
            h.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let (lo, hi) = ls
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        hi - lo > TOLERANCIA_REL * mediana(ls)
    })
}

/// A tolerância da promessa de `p`: a do GRÃO quando a borda varia com ONDE o traço cai ([`borda_ruidosa`]) ou com a
/// semente ([`a_semente_move_a_largura`]); a fina nos outros.
pub(super) fn tolerancia_da_largura(p: &Preset) -> f32 {
    if borda_ruidosa(p) || a_semente_move_a_largura(p) {
        TOLERANCIA_REL_GRAO
    } else {
        TOLERANCIA_REL
    }
}

/// **A imagem da largura de uma FERRAMENTA** (J4 `mistura`; uma ferramenta não pinta — no papel branco mede 0): a do que ela
/// MUDA na tinta de [`base_da_ferramenta`]. A base é uma faixa cuja orla de BAIXO corre sobre a linha do traço (a
/// faixa cobre `[cy − 3r, cy]`, larga o bastante para a pegada caber nela): a ferramenta arrasta, dilui, seca ou tira
/// essa orla na largura da pegada. A largura é a de [`largura_que_se_ve`] sobre a DIFERENÇA da tela com e sem o traço
/// da ferramenta (a mesma água, os mesmos quadros) — por coluna, a largura EQUIVALENTE ([`largura_equivalente`]).
pub(super) fn imagem_da_ferramenta(
    p: &Preset,
    modo: &str,
    raio: f32,
    crua: bool,
    r: Relogio,
    semente: u64,
) -> (Vec<u8>, u32, u32, CenaDoRasto) {
    use ph2d_editor_core::tool::RasterEditTool;
    let base = crate::midias_de_fabrica::de_fabrica(base_da_ferramenta(modo));
    let escala = if crua { (raio / 500.0).max(1.0) } else { 1.0 };
    // A cena é a do raio do MOTOR (o mesmo nas duas leis): com a promessa, o da tabela do preset; crua, o pedido. Sem
    // isto a base (que acompanha o tamanho) saía outra na tabela e na promessa, e a largura medida também.
    let motor = if crua {
        raio
    } else {
        raio_do_motor_da_tabela(&p.parametros.width_table, raio)
    };
    // A tela e o traço de `traco_recto_alto` (a mesma altura, o mesmo gesto), com a base antes e o traço da ferramenta
    // só num dos dois lados: a faixa da base, ao raio 1,5·m (3m de largura), com a orla de baixo em `cy`, e a ferramenta
    // ao longo dela. O SMEAR arrasta no sentido do traço — ao longo da orla não muda quase nada (a tabela saía ~1 px a
    // todo o raio, janela 15) —: a faixa dele fica de PÉ (2m de largura, a orla direita em `cx`) e o traço dela ATRAVESSA
    // a orla para fora; a largura é a do rasto que ele puxa, de novo por coluna.
    // A ORLA da base: os dois pontos dela, a normal para DENTRO da faixa e o raio da faixa (a largura é o dobro).
    let (w, h, orla, dentro, ferramenta, raio_da_base) = if modo == "Smear" {
        let (w, h) = cena_da_ferramenta(modo, motor);
        // `cy` na grade INTEIRA (cura-smearwet §18): com `h · 0,5` a altura truncada `3m + 120` punha meia fase
        // escondida a cada inteiro de `3m` (5,94 → 6 trocava as fases ¼ e ¾); a fase sub-píxel é só a da cena.
        let (cx, cy) = (80.0 + 2.0 * motor, (h / 2) as f32);
        (
            w,
            h,
            [[cx, 20.0], [cx, h as f32 - 20.0]],
            [-1.0, 0.0],
            [[cx - motor, cy], [cx + 1.5 * motor, cy]],
            motor,
        )
    } else {
        let (w, h) = cena_da_ferramenta(modo, motor);
        // `cy` na grade INTEIRA, como o Smear ([`sonda_meia_fase_do_misturador`]: a meia fase movia a largura até 0,66 px).
        let cy = if MEIA_FASE_DO_MISTURADOR.get() {
            h as f32 * 0.5
        } else {
            (h / 2) as f32
        };
        let margem = MARGEM_DO_MISTURADOR.max(motor + 100.0);
        (
            w,
            h,
            [[margem - 40.0, cy], [w as f32 - margem + 40.0, cy]],
            [0.0, -1.0],
            [[margem, cy], [w as f32 - margem, cy]],
            1.5 * motor,
        )
    };
    let pinta = |com_base: bool, com_ferramenta: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
        t.set_wet_relogio_fixo(true);
        t.set_semente_dos_tracos(semente);
        assert!(t.wet_preset_aplicar(&direito(&base)).is_empty());
        t.set_brush_color_srgb8([191, 0, 18]);
        for faixa in faixas_da_base(orla, dentro, raio_da_base, t.raio_maximo())
            .into_iter()
            .filter(|_| com_base)
        {
            t.set_brush_size_px(faixa.1);
            crate::sonda_gestos::gesto(&mut t, &faixa.0, 3.0, |_| 1.0);
        }
        let orla = (modo == "Smear").then(|| orla_da_agua_da_base(&mut t, w, orla[0][0]));
        // Pela porta do navegador: ela põe a ferramenta do preset na mão (a interna escreve-o no slot dela e deixa o
        // Pincel na mão — a régua media o traço do Ink Brush).
        assert!(t.preset_aplicar(&direito(p)).unwrap().is_empty());
        if crua {
            t.set_wet_largura_crua(escala);
        }
        if com_ferramenta {
            // ⭐ A ferramenta atravessa as fases sub-píxel ao longo do traço, como o pincel
            // (`sonda_gestos::traco_que_desliza_com`): o centro desliza 1 px na normal da orla.
            let (fase, deslize) = super::FASE_DA_CENA.get();
            let [[x0, y0], [x1, y1]] = ferramenta;
            let ferramenta = [[x0, y0 + fase], [x1, y1 + fase + deslize]];
            t.set_brush_size_px(raio / escala);
            crate::sonda_gestos::gesto(&mut t, &ferramenta, 3.0, |_| 1.0);
        }
        passa_o_tempo(&mut t, QUADROS_DA_LEI, r);
        (t.run_full().0, orla)
    };
    let ((com, orla), (sem, _)) = (pinta(true, true), pinta(true, false));
    let diferenca: Vec<u8> = com
        .as_chunks::<4>()
        .0
        .iter()
        .zip(sem.as_chunks::<4>().0)
        .flat_map(|(a, b)| {
            let d = (0..3).map(|k| a[k].abs_diff(b[k])).max().unwrap_or(0);
            [255 - d, 255 - d, 255 - d, 255]
        })
        .collect();
    let cx = 80.0 + 2.0 * motor;
    let cena = CenaDoRasto {
        m: motor,
        x0: orla.map_or(cx, |o| o.max(cx)),
    };
    (diferenca, w, h, cena)
}

std::thread_local! {
    /// `true` = a orla da base do misturador em `cy = h·0,5` (a cena de antes de 03/11): com `h = 6m + 80` truncado, meia
    /// fase escondida a cada inteiro ímpar de `6m`. Só a [`sonda_meia_fase_do_misturador`] o liga, para comparar.
    static MEIA_FASE_DO_MISTURADOR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// **Sonda: a meia fase da orla da base do misturador** (aberto da rodada 02/11c, a mesma que enganou o Smear): os 5
/// presets do misturador (Blend, Wet) com a lei crua, a raios do motor com `⌊6m⌋` ÍMPAR (`cy = h·0,5` cai a meio
/// píxel), com a orla a meio píxel × no inteiro — as duas no MESMO processo, intercaladas. Medido 03/11: a meia fase
/// movia a largura até +0,66 px (Water Brush motor 7,2, 7,3 %) e 0,29 px nos Blender (13,2) — a régua saltava com a
/// paridade de `⌊6m⌋`; a orla foi para o inteiro.
#[test]
#[ignore = "sonda: a meia fase da cena do misturador (custo ~ minutos)"]
fn sonda_meia_fase_do_misturador() {
    const RAIOS: [f32; 5] = [3.2, 7.2, 13.2, 26.2, 55.2];
    let presets: Vec<Preset> = ficheiros_do_wet()
        .into_iter()
        .map(|(_, b)| ler(&b).unwrap())
        .filter(|p| {
            matches!(
                p.parametros.paint_mode.as_deref(),
                Some("Blend" | "Wet" | "Dry")
            )
        })
        .collect();
    let casos: Vec<(usize, f32, bool)> = (0..presets.len())
        .flat_map(|k| {
            RAIOS
                .iter()
                .flat_map(move |&r| [(k, r, true), (k, r, false)])
        })
        .collect();
    let custos: Vec<u64> = casos
        .iter()
        .map(|&(k, r, _)| super::memoria::custo_da_cena(&presets[k], r, true))
        .collect();
    let fios = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);
    let ls = super::memoria::em_paralelo_pela_memoria(&custos, fios, |j| {
        let (k, r, meia) = casos[j];
        MEIA_FASE_DO_MISTURADOR.set(meia);
        let (px, w, h, regua) = imagem_da_largura(&presets[k], r, true, Relogio::DeUmaVez, 0);
        regua(&px, w, h)
    });
    for (c, l) in casos.chunks(2).zip(ls.chunks(2)) {
        let (k, r, _) = c[0];
        println!(
            "[meia-fase] {:<16} motor {r:>5}: meia {:>8.3} · inteiro {:>8.3} · Δ {:+.3} px ({:+.2} %)",
            presets[k].parametros.name,
            l[0],
            l[1],
            l[0] - l[1],
            100.0 * (l[0] - l[1]) / l[1]
        );
    }
}

/// A cena do rasto do Smear que a régua precisa: o raio do motor EXACTO e a 1.ª coluna fora da água da base.
#[derive(Clone, Copy, Debug)]
pub(super) struct CenaDoRasto {
    pub(super) m: f32,
    pub(super) x0: f32,
}

/// ⭐ **Onde acaba a água da base** (cura-smearwet §15): a coluna (px) a partir da orla `cx` da faixa de pé em que a
/// grade do motor deixa de estar molhada (`wet` = 0), a mediana sobre as linhas da faixa. A água do Smear corre pelo
/// chão molhado (o termo `wet` do freio, §10), por isso as colunas antes daqui medem a ORLA da base, não o rasto.
fn orla_da_agua_da_base(t: &mut PainterTool, w: u32, cx: f32) -> f32 {
    let mut fins = Vec::new();
    t.wet_sonda_motor(|e| {
        let g = e.active_grid();
        let razao = w as f32 / g.w as f32;
        let celula = |x: f32, j: usize| ((x / razao) as usize + 1).min(g.w) + j * g.s;
        // Só as linhas onde a base está: a célula logo DENTRO da orla molhada.
        for j in (1..=g.h).filter(|&j| g.wet[celula((cx - 1.0).max(0.0), j)] > 0) {
            // A orla SUB-PÍXEL: cada célula molhada conta a sua COBERTURA, `wet ÷ o byte cheio do papel dela` (o
            // depósito escreve `ceil(byte · cobertura)`, cura-smearwet §17) — a meia célula conta meia, não uma.
            let (mut x, mut fim) = (cx.floor(), cx.floor());
            while x < w as f32 && g.wet[celula(x, j)] > 0 {
                let k = celula(x, j);
                let cheio = ph2d_wet_paint::grid::wet_byte_from_paper(f64::from(g.paper[k]));
                fim += razao * (f32::from(g.wet[k]) / f32::from(cheio.max(1))).min(1.0);
                x += 1.0;
            }
            fins.push(fim);
        }
    });
    mediana(fins)
}

/// A tela (w, h) da cena de [`imagem_da_ferramenta`] ao raio do motor `motor`: a do Smear (a faixa de pé) ou a do
/// misturador (o vão recto e a margem que cresce com o raio). A mesma conta dá o custo em [`px_da_cena`].
fn cena_da_ferramenta(modo: &str, motor: f32) -> (u32, u32) {
    if modo == "Smear" {
        ((motor * 4.5) as u32 + 160, (motor * 3.0) as u32 + 120)
    } else {
        let margem = MARGEM_DO_MISTURADOR.max(motor + 100.0);
        (
            (VAO_DO_MISTURADOR + 2.0 * margem) as u32,
            ((motor * 6.0) as u32 + 80).max(120),
        )
    }
}

/// Os píxeis da tela de UMA medição de [`imagem_da_largura`] (`p` a `raio`): a da ferramenta ao raio do motor, ou a de
/// `traco_recto_alto_com` (largura `sonda_gestos::W`, altura `6·raio + 80`). O custo de memória dela em voo.
pub(super) fn px_da_cena(p: &Preset, raio: f32, crua: bool) -> u64 {
    let (w, h) = match p.parametros.paint_mode.as_deref().filter(|m| *m != "Paint") {
        Some(modo) => {
            let motor = if crua {
                raio
            } else {
                raio_do_motor_da_tabela(&p.parametros.width_table, raio)
            };
            cena_da_ferramenta(modo, motor)
        }
        None => (crate::sonda_gestos::W, ((raio * 6.0) as u32 + 80).max(120)),
    };
    u64::from(w) * u64::from(h)
}

/// ⛔ **A promessa a `raio` passa do TECTO DO MOTOR** (02/11c): o motor que a tabela de `p` dá a `raio` passa de
/// [`ph2d_painter_brush::MAX_BRUSH_RADIUS_PX`] — o motor grampeia o raio e a largura medida ali não é a desse motor.
/// Devolve o motor. Entre o maior raio da lei (`RAIO_MAX`) e o tecto o refino MEDE o ponto (o recurso é a memória, que o
/// [`super::memoria::em_paralelo_pela_memoria`] gere): a Water Brush pede o motor 1 141,7 no raio 300.
pub(super) fn motor_passa_do_tecto(p: &Preset, raio: f32) -> Option<f32> {
    let motor = raio_do_motor_da_tabela(&p.parametros.width_table, raio);
    (motor > ph2d_painter_brush::MAX_BRUSH_RADIUS_PX).then_some(motor)
}

/// O vão RECTO do traço do misturador (px) e a margem mínima de cada lado: a 1000 px de tela (a `sonda_gestos::W` de
/// antes) os raios ≥ 400 eram só pontas. A margem cresce com o raio (`m + 100`), e a régua só lê as colunas do vão.
const VAO_DO_MISTURADOR: f32 = 800.0;
const MARGEM_DO_MISTURADOR: f32 = 100.0;

/// Os traços que pintam a faixa da base: o raio de cada um é `raio_da_base` até ao `maximo` do Size da base (o slider
/// pára em 512, `raio_maximo`), e acima dele empilham-se traços para DENTRO da orla, cada um a 1,5 raios do anterior,
/// até a faixa ter `2·raio_da_base`. ⛔ Medido 29/10: com UM traço a faixa ficava presa a 1 024 px e a orla dela fugia
/// da linha do traço — a lei crua do Blender Soft caía de 248 px a 0 entre o raio 723 e o 990 (a régua mentia).
fn faixas_da_base(
    orla: [[f32; 2]; 2],
    dentro: [f32; 2],
    raio_da_base: f32,
    maximo: f32,
) -> Vec<([[f32; 2]; 2], f32)> {
    let rb = raio_da_base.min(maximo);
    let mut d = rb;
    let mut v = Vec::new();
    loop {
        let em = |q: [f32; 2]| [q[0] + dentro[0] * d, q[1] + dentro[1] * d];
        v.push(([em(orla[0]), em(orla[1])], rb));
        if d + rb >= 2.0 * raio_da_base - 0.5 {
            break v;
        }
        d += 1.5 * rb;
    }
}

/// A diferença de uma coluna `x` da imagem de [`imagem_da_ferramenta`] (`0` = igual, `1` = o máximo).
fn mudanca(px: &[u8], w: u32, x: u32, y: u32) -> f32 {
    let i = ((y * w + x) * 4) as usize;
    f32::from(255 - px[i]) / 255.0
}

/// O pico abaixo do qual uma coluna não entra na mediana (o de [`crate::sonda_gestos::largura_mediana`]): é a coluna que
/// entra ou sai, não um lado dela — perto das pontas do traço, onde a mediana não mora.
const PISO_DA_COLUNA: f32 = 0.08;

/// ⭐ **A largura EQUIVALENTE da coluna `x`**: a área da mudança ÷ o pico dela (a largura do retângulo com a mesma área
/// e a mesma altura). Num perfil chato é a largura dele; num de sino, 1,06× a meia altura. É CONTÍNUA na imagem — um
/// lado que cresce de 0 entra com o seu peso, não de uma vez.
///
/// ⛔ Recusa MEDIDA (29/10): as larguras por LIMIAR saltavam no raio — a meia altura da coluna inteira (o lado de dentro
/// entrava ou não: Blender Soft 236 px a 254 → 481 a 282), cada lado à ½ do seu pico com piso 0,08 (o lado ligava num
/// degrau: Blender Textured motor 27,75 → 41 px, 27,99 → 60) e a 1.ª à última linha acima de ½ do pico no Smear (o
/// espalhar da água cruzava o limiar: Smear Wet 3,41 → 13, 3,5 → 27, 3,8 → 14).
pub(crate) fn largura_equivalente(px: &[u8], w: u32, h: u32, x: u32) -> Option<f32> {
    largura_equivalente_entre(px, w, x, 0..h)
}

/// A [`largura_equivalente`] da coluna `x` só nas linhas `ys`.
fn largura_equivalente_entre(px: &[u8], w: u32, x: u32, ys: std::ops::Range<u32>) -> Option<f32> {
    let (soma, pico) = ys
        .map(|y| mudanca(px, w, x, y))
        .fold((0.0f32, 0.0f32), |(s, m), c| (s + c, m.max(c)));
    (pico >= PISO_DA_COLUNA).then(|| soma / pico)
}

/// ⭐ **A largura de um misturador (Blend, Wet, Dry… — toda ferramenta menos o Smear)**: a pegada sobre a orla da base
/// muda os DOIS lados dela (a tinta que se mistura com o papel e a que vai para o papel); a largura é a mediana das
/// [`largura_equivalente`] das colunas — o lado mais fraco pesa o que muda.
pub(super) fn largura_do_misturador(px: &[u8], w: u32, h: u32) -> f32 {
    let x0 = ((w as f32 - VAO_DO_MISTURADOR) * 0.5) as u32;
    mediana(
        (x0..(x0 + VAO_DO_MISTURADOR as u32).min(w))
            .filter_map(|x| largura_equivalente(px, w, h, x))
            .collect(),
    )
}

/// ⭐ **A largura da ÁGUA (modos Wet e Dry, base Wet Round)**: a de [`largura_do_misturador`], com cada coluna só na
/// metade da faixa PERTO do traço — `cy ± 1,5·m` (o raio da base; a cena tem `h = 6m + 80`, `cy = h/2`). A água que
/// empurra corre pelo filme molhado até à orla de LÁ da faixa (`cy − 3m`) e mexe-a; essa orla é da cena, não do traço.
/// ⛔ Medido 03/11 (Water Brush, motor 298, só a base a variar 285/290/295/305): o pico da coluna inteira morava na orla
/// de lá (linha mediana 38–133, o traço em 934) e subia com os poros da base (0,31 → 0,45), com a área quase igual
/// (58 → 61): a largura lia 187 → 131 → 208. Na janela `±1,5m`: 219 / 195 / 184 / 210.
pub(super) fn largura_da_agua(px: &[u8], w: u32, h: u32) -> f32 {
    let m = (h as f32 - 80.0) / 6.0;
    let cy = (h / 2) as f32;
    let ys = ((cy - 1.5 * m).max(0.0) as u32)..((cy + 1.5 * m).ceil() as u32).min(h);
    let x0 = ((w as f32 - VAO_DO_MISTURADOR) * 0.5) as u32;
    mediana(
        (x0..(x0 + VAO_DO_MISTURADOR as u32).min(w))
            .filter_map(|x| largura_equivalente_entre(px, w, x, ys.clone()))
            .collect(),
    )
}

/// ⭐ **A largura do rasto do Smear**: a faixa de pé tem a orla direita em `cx` e o traço atravessa-a de `cx − m` a
/// `cx + 1,5·m` ([`imagem_da_ferramenta`]); o rasto que ele PUXA mede-se nas colunas por onde o CENTRO passou sobre o
/// papel (`cx … cx + 1,5·m`) — a ponta que se esvai depois dele e a orla esquerda da faixa ficam de fora (com elas o
/// Smear Shattered lia 597 px a 253,9 e 470 a 255,4). Por coluna, a [`largura_equivalente`]; a mediana.
/// (⛔ Medido 29/10: a área de todas as colunas ÷ a soma dos picos — contra o ruído das ~8 colunas a raio pequeno — saltava
/// MAIS no Smear Wet, queda 0,21 e 33,8 → 26,7 px a raio 6,3 → 6,7: as colunas de pico baixo e água larga mandavam.)
///
/// ⭐ cura-smearwet §15: as colunas começam depois de `cena.x0` e da [`FAIXA_DA_ORLA_DA_BASE`], fora da água da base ([`orla_da_agua_da_base`]) — lá a água
/// do Smear corre pela faixa molhada e a coluna mede a orla (25–80 px), não o rasto (10–18 px); a mediana das duas
/// populações saltava com a fase. `cena.m` é o motor EXACTO (o `(w − 160)/4,5` truncado lia 4,684 e 4,779 iguais).
pub(super) fn largura_do_rasto(px: &[u8], w: u32, h: u32, cena: CenaDoRasto) -> f32 {
    let (x0, x1) = colunas_do_rasto(w, cena);
    let fora_da_orla =
        FAIXA_DA_ORLA_DA_BASE.min(x1.saturating_sub(x0).saturating_sub(COLUNAS_MIN_DO_RASTO));
    mediana(
        (x0 + fora_da_orla..x1)
            .filter_map(|x| largura_equivalente(px, w, h, x))
            .collect(),
    )
}

/// As colunas por onde o centro passou sobre o papel, com a faixa da orla da base.
fn colunas_do_rasto(w: u32, cena: CenaDoRasto) -> (u32, u32) {
    let x0 = cena.x0 as u32;
    let x1 = ((80.0 + 3.5 * cena.m) as u32)
        .max(x0 + COLUNAS_MIN_DO_RASTO)
        .min(w);
    (x0, x1)
}

/// A [`largura_do_rasto`] COM a faixa da orla: o controlo e a régua da água que corre pela orla da base.
fn largura_do_rasto_com_orla(px: &[u8], w: u32, h: u32, cena: CenaDoRasto) -> f32 {
    let (x0, x1) = colunas_do_rasto(w, cena);
    mediana(
        (x0..x1)
            .filter_map(|x| largura_equivalente(px, w, h, x))
            .collect(),
    )
}

/// ⭐ **A faixa que não é rasto** (medido 03/11, colunas da cena a motor 4,5 · 4,75 · 5, cx = a orla): a água do Smear corre
/// ao longo da orla molhada nas 3 colunas depois dela (25 · 45 · 22 px) e o rasto começa logo a seguir (12,7 · 11,5 · 10,4),
/// o mesmo a motor 13 a 120. Com ~7 colunas na mediana a raio pequeno, as 3 mudavam a leitura de 12 para 17 px.
const FAIXA_DA_ORLA_DA_BASE: u32 = 3;

/// As colunas mínimas da mediana de [`largura_do_rasto`] (02/11c): a `m < 2` o vão `cx … cx + 1,5·m` tem menos de 3
/// colunas inteiras e a `m` ≈ 0,45–0,67 NENHUMA — a régua lia 0 (ou uma coluna só, a da orla da base) e a tabela do
/// Smear Wet guardava o 0,687 → 1,7. O rasto que o centro puxa cobre pelo menos a pegada depois do fim do traço:
/// três colunas a partir de `cx` caem nele a todo o raio ([`a_regua_do_rasto_le_um_traco_estreito`]).
const COLUNAS_MIN_DO_RASTO: u32 = 3;

/// ⭐ **A régua do rasto lê a largura REAL de um traço estreito a qualquer raio do motor** (02/11c): uma faixa sintética
/// de `k` = 1…4 px de altura sobre o rasto (`cx` → o fim do traço + 1 px, a cena de [`imagem_da_ferramenta`]) mede `k`
/// de m = 0,3 a 7. Controlo: sem [`COLUNAS_MIN_DO_RASTO`] a m = 0,5 o vão não tinha coluna nenhuma (lia 0).
#[test]
fn a_regua_do_rasto_le_um_traco_estreito() {
    for m in [0.3f32, 0.5, 0.6, 0.687, 1.0, 1.108, 1.5, 2.0, 3.25, 7.0] {
        let (w, h) = ((m * 4.5) as u32 + 160, (m * 3.0) as u32 + 120);
        let (cx, cy) = (80.0 + 2.0 * m, h / 2);
        let (xa, xb) = (cx as u32, (cx + 1.5 * m).ceil() as u32 + 1);
        for k in 1..=4u32 {
            let mut px = vec![255u8; (w * h * 4) as usize];
            for y in cy..cy + k {
                for x in xa..=xb {
                    let i = ((y * w + x) * 4) as usize;
                    px[i..i + 3].fill(0);
                }
            }
            let l = largura_do_rasto(&px, w, h, CenaDoRasto { m, x0: cx });
            assert!(
                (l - k as f32).abs() < 1e-4,
                "m {m}, traço de {k} px: a régua leu {l}"
            );
        }
    }
}

/// O salto admitido entre fases e raios vizinhos do rasto (px, sobre 20–26 de largura). Medido 03/11 com a umidade até à
/// orla: pior fase 1,67 · pior vizinho 1,50 — o MESMO ruído da lei de antes da cobertura (1,64 · 1,39). O 1,0 só passava
/// com o miolo seco pela orla do último dab, que era o defeito que o dono viu (o traço listrado, sem espalhar).
const SALTO_DO_RASTO_PX: f32 = 1.75;
/// O salto admitido COM a faixa da orla. Medido 03/11: o byte inteiro na orla 10,1 (17,6 → 25,0 → 13,9, a água corre
/// por ela nas fases a meia célula) · a cobertura de uma célula 4,7.
const SALTO_DA_ORLA_PX: f32 = 6.0;

/// ⭐ **O rasto do Smear Wet é contínuo no raio e não depende da fase** (cura-smearwet §15): a motor 4,5…6,5 (passo ¼) a
/// largura não salta mais de [`SALTO_DO_RASTO_PX`] entre raios vizinhos e entre as fases ¼ px da cena, fora da faixa da
/// orla; com ela, não mais de [`SALTO_DA_ORLA_PX`]. CONTROLO: com a faixa a mesma imagem salta acima do primeiro.
#[test]
fn o_rasto_do_smear_wet_e_continuo_e_nao_depende_da_fase() {
    let (_, b) = ficheiros_do_wet()
        .into_iter()
        .find(|(rel, _)| rel.ends_with("/Smear Wet.ph2d-brush.png"))
        .expect("o Smear Wet");
    let p = sem_real_oil(ler(&b).unwrap());
    let motores: Vec<f32> = (0..9).map(|i| 4.5 + 0.25 * i as f32).collect();
    let fases = [0.0f32, 0.25, 0.5, 0.75];
    let casos: Vec<(usize, usize)> = (0..motores.len())
        .flat_map(|i| (0..fases.len()).map(move |f| (i, f)))
        .collect();
    let medidas: Vec<(f32, f32, f32)> = std::thread::scope(|s| {
        let h: Vec<_> = casos
            .iter()
            .map(|&(i, f)| {
                let (p, motor, fase) = (&p, motores[i], fases[f]);
                s.spawn(move || {
                    super::FASE_DA_CENA.set((fase, 1.0));
                    let (px, w, h, cena) =
                        imagem_da_ferramenta(p, "Smear", motor, true, Relogio::DeUmaVez, 0);
                    (
                        largura_do_rasto(&px, w, h, cena),
                        largura_do_rasto_com_orla(&px, w, h, cena),
                        cena.x0 - (80.0 + 2.0 * motor),
                    )
                })
            })
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let tabela = |k: usize| -> Vec<Vec<f32>> {
        (0..motores.len())
            .map(|i| {
                (0..fases.len())
                    .map(|f| {
                        let m = medidas[i * fases.len() + f];
                        [m.0, m.1, m.2][k]
                    })
                    .collect()
            })
            .collect()
    };
    let falhas = |l: &[Vec<f32>]| -> Vec<String> {
        let mut v = Vec::new();
        for (i, li) in l.iter().enumerate() {
            let (lo, hi) = li
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            if hi - lo > SALTO_DO_RASTO_PX {
                v.push(format!("motor {}: fases {li:?}", motores[i]));
            }
            if let Some(lj) = l.get(i + 1) {
                for f in 0..fases.len() {
                    if (lj[f] - li[f]).abs() > SALTO_DO_RASTO_PX {
                        v.push(format!(
                            "fase {}: motor {} → {}: {} → {}",
                            fases[f],
                            motores[i],
                            motores[i + 1],
                            li[f],
                            lj[f]
                        ));
                    }
                }
            }
        }
        v
    };
    let (nova, com_orla, margem) = (tabela(0), tabela(1), tabela(2));
    for i in 0..motores.len() {
        println!(
            "[rasto smear wet] motor {:.2} · margem {:?} · rasto {:?} · com a orla {:?}",
            motores[i], margem[i], nova[i], com_orla[i]
        );
    }
    assert!(
        !falhas(&com_orla).is_empty(),
        "controlo: com a faixa da orla a régua também é contínua — a faixa não mede nada {com_orla:?}"
    );
    let orla_max = com_orla
        .windows(2)
        .flat_map(|w| w[0].iter().zip(&w[1]).map(|(a, b)| (a - b).abs()))
        .fold(0.0f32, f32::max);
    assert!(
        orla_max <= SALTO_DA_ORLA_PX,
        "a água corre pela orla da base conforme a fase (a cobertura da orla saiu): salto {orla_max} {com_orla:?}"
    );
    let f = falhas(&nova);
    assert!(
        f.is_empty(),
        "o rasto do Smear Wet salta:\n{}",
        f.join("\n")
    );
}

/// A queda máxima da largura entre dois raios VIZINHOS da varredura de [`sonda_monotonia_da_largura`] (fração), o
/// salto máximo ACIMA da razão dos raios, e a folga em px abaixo da qual nenhum dos dois conta (a 2–3 px de raio um
/// píxel é 20 %: Smear Soft 5,33 → 4,26 px a raio 2,52 → 2,67). As medições estão no doc-comment da sonda.
const QUEDA_MAX: f32 = 0.30;
const SALTO_MAX: f32 = 0.30;
const FOLGA_PX: f32 = 1.5;

/// **A lei crua de cada FERRAMENTA não salta** (J6 fecho B4): `PH2D_SONDA_RAIOS` (60) raios geométricos do motor de
/// `PH2D_SONDA_RAIO_MIN` (2) a `PH2D_SONDA_RAIO_MAX` (60) — a faixa dos degraus medidos (Blender Textured 27,75 → 27,99;
/// Smear Wet 3,41 → 3,8) —, nas ferramentas de fábrica (ou só `PH2D_SONDA_PRESET="A,B"`), `PH2D_LEI_FIOS` (8) de cada
/// vez. Entre dois raios vizinhos a largura não cai mais de [`QUEDA_MAX`] nem sobe mais de [`SALTO_MAX`] acima da razão
/// dos raios.
#[test]
#[ignore = "sonda: a lei crua das ferramentas é contínua no raio (custo ~ minutos)"]
fn sonda_monotonia_da_largura() {
    let var = |k: &str| std::env::var(k).ok();
    let n: usize = var("PH2D_SONDA_RAIOS").map_or(60, |v| v.parse().unwrap());
    let r0: f32 = var("PH2D_SONDA_RAIO_MIN").map_or(2.0, |v| v.parse().unwrap());
    let r1: f32 = var("PH2D_SONDA_RAIO_MAX").map_or(60.0, |v| v.parse().unwrap());
    let fios: usize = var("PH2D_LEI_FIOS").map_or(8, |v| v.parse().unwrap());
    let so = var("PH2D_SONDA_PRESET");
    let raios: Vec<f32> = (0..n)
        .map(|i| r0 * (r1 / r0).powf(i as f32 / (n - 1) as f32))
        .collect();
    let presets: Vec<Preset> = ficheiros_do_wet()
        .into_iter()
        .map(|(_, b)| ler(&b).unwrap())
        .filter(|p| {
            p.parametros
                .paint_mode
                .as_deref()
                .is_some_and(|m| m != "Paint")
        })
        .filter(|p| {
            so.as_deref()
                .is_none_or(|s| s.split(',').any(|n| n == p.parametros.name))
        })
        .collect();
    assert!(!presets.is_empty(), "nenhuma ferramenta casou");
    let cenas: Vec<(usize, usize)> = (0..presets.len())
        .flat_map(|k| (0..n).map(move |i| (k, i)))
        .collect();
    let custos: Vec<u64> = cenas
        .iter()
        .map(|&(k, i)| super::memoria::custo_da_cena(&presets[k], raios[i], true))
        .collect();
    let medidas = super::memoria::em_paralelo_pela_memoria(&custos, fios, |c| {
        let (k, i) = cenas[c];
        largura_de_com(&presets[k], raios[i], true, Relogio::DeUmaVez)
    });
    let mut l = vec![vec![0.0f32; n]; presets.len()];
    for (&(k, i), v) in cenas.iter().zip(medidas) {
        l[k][i] = v;
    }
    let mut falhas = Vec::new();
    for (p, l) in presets.iter().zip(&l) {
        let nome = &p.parametros.name;
        let (mut queda, mut salto) = ((0.0f32, 0.0f32), (0.0f32, 0.0f32));
        for i in 0..n - 1 {
            let (a, b) = (l[i], l[i + 1]);
            if a <= 0.0 {
                continue;
            }
            let esperado = a * raios[i + 1] / raios[i];
            // Abaixo da folga em px nem a queda nem o salto contam: a fração fica 0.
            let q = if a - b > FOLGA_PX { 1.0 - b / a } else { 0.0 };
            let sl = if b - esperado > FOLGA_PX {
                b / esperado - 1.0
            } else {
                0.0
            };
            if q > queda.0 {
                queda = (q, raios[i + 1]);
            }
            if sl > salto.0 {
                salto = (sl, raios[i + 1]);
            }
        }
        println!(
            "[monotonia] {nome:<17} queda máx {:.3} (raio {:.2}) · salto máx {:.3} (raio {:.2}) · {l:?}",
            queda.0, queda.1, salto.0, salto.1
        );
        if queda.0 > QUEDA_MAX || salto.0 > SALTO_MAX {
            falhas.push(format!(
                "{nome}: queda {:.3} a {:.2} · salto {:.3} a {:.2}",
                queda.0, queda.1, salto.0, salto.1
            ));
        }
    }
    println!("[monotonia] raios {raios:?}");
    assert!(falhas.is_empty(), "a lei salta:\n{}", falhas.join("\n"));
}

/// O raio do motor de `raio` pela tabela de largura de um preset — a MESMA conta de `LeiDaLargura::de_tabela` +
/// `raio_do_motor` (`wetpaint/largura.rs`): o fim de um patamar fica (`l >= l0`) e acima da tabela vale a razão do
/// último ponto que CRESCE. Tabela vazia = o raio pedido. ⛔ (02/11c) A cópia de antes tirava os patamares (`l > l0`) e
/// usava a razão do último ponto: a cena da régua saía de outro tamanho que o traço que o motor pintava.
pub(super) fn raio_do_motor_da_tabela(tabela: &[[f64; 2]], raio: f32) -> f32 {
    let mut pontos: Vec<(f64, f64)> = Vec::new();
    let mut sem_tinta_ate = 0.0f64;
    for &[r, l] in tabela {
        if !r.is_finite() || !l.is_finite() {
            continue;
        }
        if l <= 0.0 {
            if pontos.is_empty() {
                sem_tinta_ate = sem_tinta_ate.max(r);
            }
            continue;
        }
        if pontos.last().is_none_or(|&(r0, l0)| r > r0 && l >= l0) {
            pontos.push((r, l));
        }
    }
    if pontos.len() < 2 {
        return raio;
    }
    let alvo = 2.0 * f64::from(raio);
    let (r0, l0) = pontos[0];
    if alvo <= l0 {
        let r = r0 * alvo / l0;
        return (if r < sem_tinta_ate { r0 } else { r }) as f32;
    }
    for w in pontos.windows(2) {
        let ((ra, la), (rb, lb)) = (w[0], w[1]);
        if alvo <= lb {
            return (ra + (rb - ra) * (alvo - la) / (lb - la)) as f32;
        }
    }
    let (rn, ln) = pontos
        .windows(2)
        .rev()
        .find(|w| w[1].1 > w[0].1)
        .map_or(pontos[0], |w| w[1]);
    (rn * alvo / ln) as f32
}

/// ⭐ **A envolvente MONÓTONA DE BAIXO da lei medida** (02/11c): um ponto fica só se nenhum raio MAIOR mede menos (os de
/// largura 0 — sem traço — ficam, o `de_tabela` usa-os). A lei crua do Smear Wet sob a E10 não é monótona abaixo de
/// r ≈ 16 — a água espalha o rasto a ~13 px desde r 5 —, e um PICO isolado (3,5 → 20,3 · 4,3 → 16,9) deixado na tabela
/// fazia o `de_tabela` (que guarda o 1.º que cresce) largar o patamar inteiro depois dele: a largura 14 caía a r 3,25, no
/// meio do salto (3,41 → 13, 3,5 → 27), e media 17,0. Com a envolvente de baixo o pico sai e o raio cai onde a lei
/// já É a largura pedida.
pub(super) fn envolvente_de_baixo(t: &mut Vec<[f64; 2]>) {
    let mut minimo = f64::INFINITY;
    let mut fica = vec![true; t.len()];
    for (k, &[_, l]) in t.iter().enumerate().rev() {
        if l > 0.0 {
            fica[k] = l <= minimo;
            minimo = minimo.min(l);
        }
    }
    let mut k = 0;
    t.retain(|_| {
        k += 1;
        fica[k - 1]
    });
}

/// SONDA (02/11c): a promessa de `PH2D_SONDA_PRESET` (por omissão o Smear Wet) com a tabela do ficheiro e com a
/// [`envolvente_de_baixo`] dela — sem regerar: diz se a envolvente põe o raio do motor onde a lei já é a largura.
#[test]
#[ignore = "sonda: a promessa de um preset com a envolvente de baixo da tabela (custo ~ 1 min)"]
fn sonda_promessa_com_envolvente() {
    let nome = std::env::var("PH2D_SONDA_PRESET").unwrap_or_else(|_| "Smear Wet".into());
    let (_, b) = ficheiros_do_wet()
        .into_iter()
        .find(|(rel, _)| rel.ends_with(&format!("/{nome}.ph2d-brush.png")))
        .expect("o preset");
    let p = ler(&b).unwrap();
    let mut q = p.clone();
    envolvente_de_baixo(&mut q.parametros.width_table);
    for (rotulo, x) in [("ficheiro", &p), ("envolvente", &q)] {
        let motores: Vec<f32> = RAIOS_DA_PROMESSA
            .iter()
            .map(|&r| raio_do_motor_da_tabela(&x.parametros.width_table, r))
            .collect();
        let l = larguras_da_promessa(x, false);
        let fora = fora_da_promessa(&l, tolerancia_da_largura(x), f32::INFINITY);
        println!(
            "[envolvente] {nome} {rotulo}: motores {motores:?} · larguras {l:?} · fora {fora:?}"
        );
    }
}

/// ⭐ **A régua da largura não depende da FASE sub-píxel da cena** (cura-largura3, 02/11c): o traço da medição
/// desliza 1 px ao longo do comprimento (todas as fases), e deslocar a cena ¼ px move a largura ≤ 1 px. CONTROLO
/// (a régua de antes, o traço recto numa fase fixa): o Wet Dry Brush a motor 17,5 media 16 / 13 / 10 / 17 px.
#[test]
fn a_largura_nao_depende_da_fase_da_cena() {
    let (_, b) = ficheiros_do_wet()
        .into_iter()
        .find(|(rel, _)| rel.ends_with("/Wet Dry Brush.ph2d-brush.png"))
        .expect("o Wet Dry Brush");
    let p = ler(&b).unwrap();
    let larguras = |motor: f32, deslize: f32| -> Vec<f32> {
        std::thread::scope(|s| {
            let h: Vec<_> = [0.0f32, 0.25, 0.5, 0.75]
                .into_iter()
                .map(|fase| {
                    let p = &p;
                    s.spawn(move || {
                        super::FASE_DA_CENA.set((fase, deslize));
                        largura_de_com(p, motor, true, Relogio::DeUmaVez)
                    })
                })
                .collect();
            h.into_iter().map(|h| h.join().unwrap()).collect()
        })
    };
    let espalho = |l: &[f32]| {
        let (lo, hi) = l
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        hi - lo
    };
    let antes = larguras(17.5, 0.0);
    assert!(
        espalho(&antes) > 1.0,
        "controlo: a régua numa fase fixa já era estável {antes:?}"
    );
    let mut falhas = Vec::new();
    for motor in [7.25f32, 17.5, 17.75] {
        let l = larguras(motor, 1.0);
        if espalho(&l) > 1.0 {
            falhas.push(format!("motor {motor}: {l:?}"));
        }
    }
    assert!(
        falhas.is_empty(),
        "a largura muda com a fase da cena:\n{}",
        falhas.join("\n")
    );
}
