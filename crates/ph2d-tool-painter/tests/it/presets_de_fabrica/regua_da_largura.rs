//! **A régua da largura de uma FERRAMENTA** (J6 fecho B4): a imagem do que ela muda, a largura EQUIVALENTE (área ÷ pico
//! por coluna — contínua no raio) e as sementes do sorteio. A tabela (o gerador) e a promessa (o portão) passam as duas
//! por aqui: [`sementes_da_largura`] e a régua são as mesmas dos dois lados.

use super::*;

/// As sementes dos traços da régua para um pincel SEM sorteio — `0`, a do `PainterTool::default` (neutra ao byte).
const SEMENTE_UNICA: [u64; 1] = [0];

/// As sementes de uma FERRAMENTA com Jitter: a largura é a mediana delas (a medição de N está em
/// [`sonda_monotonia_da_largura`] e no doc-comment de [`sementes_da_largura`]).
const SEMENTES_DO_SORTEIO: [u64; 5] = [0, 1, 2, 3, 4];

/// As sementes da régua da largura de `p` — a MESMA lista na tabela e na promessa. Uma ferramenta (o modo ≠ Paint) com
/// `jitter` > 0 na receita sorteia ONDE cada dab cai, e o rasto de uma semente só muda de largura conforme o sorteio
/// (medido 29/10, `sonda_sorteio_da_largura`, 9 sementes: Smear Shattered a raio 13 → 21–26 px, a 120 → 214–267): a
/// largura é a mediana de [`SEMENTES_DO_SORTEIO`]. Os pincéis (modo Paint) medem a mediana das colunas de um traço
/// longo, onde o sorteio já se média ao longo dele: uma semente.
pub(super) fn sementes_da_largura(p: &Preset) -> &'static [u64] {
    let ferramenta = p
        .parametros
        .paint_mode
        .as_deref()
        .is_some_and(|m| m != "Paint");
    let sorteia = p
        .parametros
        .brush
        .get("jitter")
        .and_then(Value::as_f64)
        .is_some_and(|j| j > 0.0);
    if ferramenta && sorteia {
        &SEMENTES_DO_SORTEIO
    } else {
        &SEMENTE_UNICA
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
    let (w, h, faixa, ferramenta, raio_da_base) = if modo == "Smear" {
        let (w, h) = ((motor * 4.5) as u32 + 160, (motor * 3.0) as u32 + 120);
        let (cx, cy) = (80.0 + 2.0 * motor, h as f32 * 0.5);
        (
            w,
            h,
            [[cx - motor, 20.0], [cx - motor, h as f32 - 20.0]],
            [[cx - motor, cy], [cx + 1.5 * motor, cy]],
            motor,
        )
    } else {
        let h = ((motor * 6.0) as u32 + 80).max(120);
        let cy = h as f32 * 0.5;
        (
            crate::sonda_gestos::W,
            h,
            [[60.0, cy - 1.5 * motor], [940.0, cy - 1.5 * motor]],
            [[100.0, cy], [900.0, cy]],
            1.5 * motor,
        )
    };
    let pinta = |com_ferramenta: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
        t.set_wet_relogio_fixo(true);
        t.set_semente_dos_tracos(semente);
        assert!(t.wet_preset_aplicar(&direito(&base)).is_empty());
        t.set_brush_size_px(raio_da_base);
        t.set_brush_color_srgb8([191, 0, 18]);
        crate::sonda_gestos::gesto(&mut t, &faixa, 3.0, |_| 1.0);
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
    mediana(
        (0..w)
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
        .filter(|p| p.parametros.paint_mode.as_deref().is_some_and(|m| m != "Paint"))
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
