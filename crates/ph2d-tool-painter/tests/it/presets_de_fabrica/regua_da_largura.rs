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
) -> (Vec<u8>, u32, u32) {
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
        let (w, h) = ((motor * 4.5) as u32 + 160, (motor * 3.0) as u32 + 120);
        let (cx, cy) = (80.0 + 2.0 * motor, h as f32 * 0.5);
        (
            w,
            h,
            [[cx, 20.0], [cx, h as f32 - 20.0]],
            [-1.0, 0.0],
            [[cx - motor, cy], [cx + 1.5 * motor, cy]],
            motor,
        )
    } else {
        let h = ((motor * 6.0) as u32 + 80).max(120);
        let (cy, margem) = (h as f32 * 0.5, MARGEM_DO_MISTURADOR.max(motor + 100.0));
        let w = (VAO_DO_MISTURADOR + 2.0 * margem) as u32;
        (
            w,
            h,
            [[margem - 40.0, cy], [w as f32 - margem + 40.0, cy]],
            [0.0, -1.0],
            [[margem, cy], [w as f32 - margem, cy]],
            1.5 * motor,
        )
    };
    let pinta = |com_ferramenta: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
        t.set_wet_relogio_fixo(true);
        t.set_semente_dos_tracos(semente);
        assert!(t.wet_preset_aplicar(&direito(&base)).is_empty());
        t.set_brush_color_srgb8([191, 0, 18]);
        for faixa in faixas_da_base(orla, dentro, raio_da_base, t.raio_maximo()) {
            t.set_brush_size_px(faixa.1);
            crate::sonda_gestos::gesto(&mut t, &faixa.0, 3.0, |_| 1.0);
        }
        // Pela porta do navegador: ela põe a ferramenta do preset na mão (a interna escreve-o no slot dela e deixa o
        // Pincel na mão — a régua media o traço do Ink Brush).
        assert!(t.preset_aplicar(&direito(p)).unwrap().is_empty());
        if crua {
            t.set_wet_largura_crua(escala);
        }
        if com_ferramenta {
            t.set_brush_size_px(raio / escala);
            crate::sonda_gestos::gesto(&mut t, &ferramenta, 3.0, |_| 1.0);
        }
        passa_o_tempo(&mut t, QUADROS_DA_LEI, r);
        t.run_full().0
    };
    let (com, sem) = (pinta(true), pinta(false));
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
    (diferenca, w, h)
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
fn largura_equivalente(px: &[u8], w: u32, h: u32, x: u32) -> Option<f32> {
    let (soma, pico) = (0..h)
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

/// ⭐ **A largura do rasto do Smear**: a faixa de pé tem a orla direita em `cx` e o traço atravessa-a de `cx − m` a
/// `cx + 1,5·m` ([`imagem_da_ferramenta`]); o rasto que ele PUXA mede-se nas colunas por onde o CENTRO passou sobre o
/// papel (`cx … cx + 1,5·m`) — a ponta que se esvai depois dele e a orla esquerda da faixa ficam de fora (com elas o
/// Smear Shattered lia 597 px a 253,9 e 470 a 255,4). Por coluna, a [`largura_equivalente`]; a mediana.
/// (⛔ Medido 29/10: a área de todas as colunas ÷ a soma dos picos — contra o ruído das ~8 colunas a raio pequeno — saltava
/// MAIS no Smear Wet, queda 0,21 e 33,8 → 26,7 px a raio 6,3 → 6,7: as colunas de pico baixo e água larga mandavam.)
pub(super) fn largura_do_rasto(px: &[u8], w: u32, h: u32) -> f32 {
    // A cena: `w = 4,5·m + 160`, `cx = 80 + 2·m` ([`imagem_da_ferramenta`]).
    let m = (w as f32 - 160.0) / 4.5;
    let (x0, x1) = ((80.0 + 2.0 * m) as u32, ((80.0 + 3.5 * m) as u32).min(w));
    mediana(
        (x0..x1)
            .filter_map(|x| largura_equivalente(px, w, h, x))
            .collect(),
    )
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
    use std::sync::atomic::{AtomicUsize, Ordering};
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
    let proxima = AtomicUsize::new(0);
    let mut l = vec![vec![0.0f32; n]; presets.len()];
    std::thread::scope(|s| {
        let h: Vec<_> = (0..fios)
            .map(|_| {
                s.spawn(|| {
                    let mut feitas = Vec::new();
                    while let Some(&(k, i)) = cenas.get(proxima.fetch_add(1, Ordering::Relaxed)) {
                        let v = largura_de_com(&presets[k], raios[i], true, Relogio::DeUmaVez);
                        feitas.push((k, i, v));
                    }
                    feitas
                })
            })
            .collect();
        for (k, i, v) in h.into_iter().flat_map(|h| h.join().unwrap()) {
            l[k][i] = v;
        }
    });
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
