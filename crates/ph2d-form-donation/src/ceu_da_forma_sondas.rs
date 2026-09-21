//! ⭐⭐⭐ **A SONDA DO CÉU DA FORMA** — quanto do objecto assado a lei nova deixa PRETO.
//!
//! ⚠️ **Ela é CPU pura e não precisa de placa**: a pergunta é a LEI, e a
//! [`crate::baked_form::pixels_pela_forma_na_cpu`] é a régua que a prova da placa já usa como
//! referência do dispositivo. *Uma sonda que precisa de adapter não se corre numa máquina com seis
//! linhas a trabalhar.*
//!
//! ```text
//! bash scripts/ph2d-run.sh scripts/cargo-test-narrow.sh ph2d-form-donation ceu_sondas
//! ```

use crate::baked_form::{BakedForm, pixels_pela_forma_na_cpu};
use ph2d_light::LightRig;

/// Lado da peça. O irmão da prova da placa, e pelo mesmo motivo: a pergunta é a LEI.
const LADO: u32 = 256;

/// **Uma bola com uma FRESTA** — e as duas metades da peça são de propósito.
///
/// ⚠️ A bola sozinha mede o **ambiente** (o terminador, onde `N·L` passa a zero); a fresta mede a
/// **oclusão**, que na peça da prova da placa é `1` em todo o lado — *uma fixtura sem oclusão não
/// pode dizer se a oclusão chega*.
///
/// ⚠️ **A fresta é SINTÉTICA e declara-se como tal**: o assador a sério escreve `cavidade × dois
/// AOs`, e reproduzi-lo aqui obrigaria esta folha a conhecer uma malha. O que a sonda pergunta é se
/// o canal **chega à lei**, e para isso um canal com a forma certa chega.
pub fn bola_com_fresta() -> BakedForm {
    let n = (LADO * LADO) as usize;
    let (mut base, mut form, mut occ) = (vec![0u8; n * 4], vec![0f32; n * 4], vec![1f32; n]);
    let meio = f64::from(LADO) * 0.5;
    let r = meio * 0.80;
    for y in 0..LADO {
        for x in 0..LADO {
            let i = (y * LADO + x) as usize;
            let (dx, dy) = (f64::from(x) + 0.5 - meio, f64::from(y) + 0.5 - meio);
            let d2 = dx * dx + dy * dy;
            // Um barro claro e dessaturado — o mesmo da escultura, para a FORMA aparecer.
            base[i * 4..i * 4 + 3].copy_from_slice(&[189u8, 179, 168]);
            base[i * 4 + 3] = 255;
            if d2 < r * r {
                let z = (r * r - d2).sqrt();
                let q = (d2 + z * z).sqrt();
                form[i * 4] = (dx / q) as f32;
                // ⛔⛔ **`+dy` e NÃO `-dy`** — ver [`a_peca_sintetica_acende_por_cima`]. O canal da
                // forma é o que o `canvas_normal` do barro ESCREVE: espaço de CANVAS, `y` para
                // BAIXO. Com o sinal de vista a bola acende por baixo.
                form[i * 4 + 1] = (dy / q) as f32;
                form[i * 4 + 2] = (z / q) as f32;
                form[i * 4 + 3] = 1.0;
                // A FRESTA: uma banda horizontal a meio da bola, fechada a `0,25`.
                let t = (dy / (meio * 0.16)).abs();
                if t < 1.0 {
                    occ[i] = 0.25 + 0.75 * (t * t) as f32;
                }
            } else {
                // ⚠️ Fora da silhueta o neutro é `[0,0,1]` com cobertura ZERO.
                form[i * 4 + 2] = 1.0;
            }
        }
    }
    BakedForm {
        size: (LADO, LADO),
        base,
        form,
        form_occ: occ,
        texture_id: 0,
        rig: LightRig::default(),
        lit_with: None,
        lei: crate::lei_da_luz::Lei::default(),
    }
}

/// A luminância de um texel, na régua de Rec.709.
fn lum(px: &[u8]) -> f64 {
    // ⛔⛔⛔ **DESCODIFICA ANTES DE PESAR, e sem isto uma RAZÃO daqui não é uma razão de LUZ.**
    //
    // Os pesos `0,2126 / 0,7152 / 0,0722` são Rec.709 e estão **definidos sobre luz linear**;
    // aplicados a códigos sRGB eles dão *luma* (a grandeza de vídeo), que não é proporcional à luz.
    //
    // ⚠️ **Isto mordeu em 2026-09-20, no dia em que o assado passou a CODIFICAR** (o passe escrevia
    // bytes crus e a distinção era invisível): a razão cima/baixo do
    // [`a_peca_sintetica_acende_por_cima`] caiu de **`2,6` para `1,4953`** e o gate reprovou por
    // `0,005` — **sobre um produto correcto**. A curva é compressiva, logo *toda* razão lida em
    // códigos encolhe, e uma barra calibrada num vale medido em luz deixa de descrever esse vale.
    //
    // ⭐ A cura não é baixar a barra — é devolver a grandeza à unidade em que ela foi calibrada.
    let lin = |b: u8| f64::from(ph2d_color::srgb::srgb_to_linear_byte(b));
    0.2126 * lin(px[0]) + 0.7152 * lin(px[1]) + 0.0722 * lin(px[2])
}

/// ⛔⛔⛔ **QUANTO DA PEÇA A LEI NOVA DEIXA PRETO** — o report do dono, em números.
///
/// A lei desta casa está escrita no doc do [`ph2d_light::AMBIENT`] e nomeia este defeito **antes de
/// ele acontecer**: *«os dois consumidores (tinta e forma) têm de dobrar a razão do MESMO jeito,
/// senão a mesma lâmpada deixaria a escultura mais escura na sombra que a pintura ao lado dela, e
/// ninguém saberia dizer por quê»*.
#[test]
#[ignore = "sonda: imprime uma tabela, nao afirma"]
fn diag_quanto_a_lei_nova_deixa_preto() {
    let bake = bola_com_fresta();
    let rig = LightRig::default();
    let resolvido = ph2d_light::resolve(&rig).expect("o rig de fabrica tem lampada");
    let acesos = resolvido.lamps().len();
    let luz = resolvido.lamps()[0].dir;
    let px = pixels_pela_forma_na_cpu(&bake, &rig).expect("a regua acende");

    let (mut n_forma, mut n_preto, mut n_sombra) = (0usize, 0usize, 0usize);
    let (mut soma_sombra, mut soma_luz) = (0f64, 0f64);
    let (mut soma_fresta, mut n_fresta) = (0f64, 0usize);
    for i in 0..(LADO * LADO) as usize {
        if bake.form[i * 4 + 3] <= 0.0 {
            continue;
        }
        n_forma += 1;
        let n = [bake.form[i * 4], bake.form[i * 4 + 1], bake.form[i * 4 + 2]];
        let ndl = n[0] * luz[0] + n[1] * luz[1] + n[2] * luz[2];
        let l = lum(&px[i * 4..i * 4 + 4]);
        if px[i * 4] == 0 && px[i * 4 + 1] == 0 && px[i * 4 + 2] == 0 {
            n_preto += 1;
        }
        if ndl <= 0.0 {
            n_sombra += 1;
            soma_sombra += l;
        } else {
            soma_luz += l;
        }
        if bake.form_occ[i] < 0.5 {
            n_fresta += 1;
            soma_fresta += l;
        }
    }
    let pc = |k: usize| 100.0 * k as f64 / n_forma as f64;
    let media = |s: f64, k: usize| s / k.max(1) as f64;
    println!();
    println!(
        "  lampadas acesas no rig de FABRICA ····· {acesos} de {}",
        ph2d_light::MAX_LIGHTS
    );
    println!("  texels de forma ······················· {n_forma}");
    println!(
        "  PRETOS ao bit ························· {n_preto} ({:.3} %)",
        pc(n_preto)
    );
    println!(
        "  na sombra (N·L <= 0) ·················· {n_sombra} ({:.3} %)",
        pc(n_sombra)
    );
    println!(
        "  luminancia media na SOMBRA ············ {:.3}",
        media(soma_sombra, n_sombra)
    );
    println!(
        "  luminancia media na LUZ ··············· {:.3}",
        media(soma_luz, n_forma - n_sombra)
    );
    println!(
        "  luminancia media na FRESTA (occ<0,5) ·· {:.3}  sobre {n_fresta} texels",
        media(soma_fresta, n_fresta)
    );
    println!();
    println!(
        "  a LEI da casa: a sombra vale `AMBIENT` = {:.2} do plano — nunca zero.",
        ph2d_light::AMBIENT
    );
    println!("  ANTES do ceu: PRETOS 8 243 (25,03 %), sombra 0,000, fresta 165,49.");
    println!("  ⇒ a fresta escurecer e' a OCLUSAO a chegar ao pixel pela 1.a vez.");
    println!();
}

/// **As médias das duas metades de cada BOLA**, sobre os texels que têm forma.
///
/// `e_cima` diz, para uma linha de imagem, se ela está na metade de CIMA da bola a que pertence.
///
/// ⛔⛔ **A partição tem de ser por LINHA DE IMAGEM e nunca pela normal, e as duas metades desta
/// frase são load-bearing.** Por linha porque é assim que o dono olha para a peça; e **nunca** pela
/// normal porque o canal da normal é exactamente o que está sob teste — partir por `form.y` faria a
/// partição virar JUNTO com o defeito, e o gate ficaria **verde nos dois sentidos**.
///
/// ⚠️ **E ela é POR BOLA e não pelo ecrã, o que a 1.ª redacção não era:** a fixtura da prova da placa
/// tem **quatro** bolas em quadrantes, logo um corte a meio do ecrã cai *entre* elas e compara os
/// ALBEDOS de duas contra duas — leu `85,22` contra `94,03` sobre uma peça correcta. *Uma partição
/// que não atravessa o fenómeno mede outra coisa.*
pub(crate) fn metades_da_bola(
    bake: &BakedForm,
    rig: &LightRig,
    e_cima: impl Fn(u32) -> bool,
) -> (f64, f64) {
    let (w, h) = bake.size;
    let px = pixels_pela_forma_na_cpu(bake, rig).expect("a regua acende");
    let (mut cima, mut n_cima, mut baixo, mut n_baixo) = (0f64, 0usize, 0f64, 0usize);
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if bake.form[i * 4 + 3] <= 0.0 {
                continue;
            }
            let l = lum(&px[i * 4..i * 4 + 4]);
            if e_cima(y) {
                cima += l;
                n_cima += 1;
            } else {
                baixo += l;
                n_baixo += 1;
            }
        }
    }
    assert!(
        n_cima > 0 && n_baixo > 0,
        "a particao tem de apanhar texels dos DOIS lados: cima {n_cima}, baixo {n_baixo}"
    );
    (cima / n_cima as f64, baixo / n_baixo as f64)
}

/// ⏱️ **SONDA — A MESMA RAZÃO, NAS DUAS UNIDADES.**
///
/// Ela existe porque uma mutação SOBREVIVEU: devolver o [`lum`] a pesar **códigos** deixa o
/// [`a_peca_sintetica_acende_por_cima`] VERDE, logo a corecção daquela régua não é hoje
/// discriminada por gate nenhum. ⚠️ *Uma mutação sobrevivente nomeia uma régua em falta — e o
/// primeiro passo é imprimir o número, para a decisão deixar de ser sobre uma intuição.*
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-form-donation --lib \
///   diag_a_razao_nas_duas_unidades -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medição: imprime a tabela, não afirma nada"]
fn diag_a_razao_nas_duas_unidades() {
    let rig = LightRig::default();
    let bake = bola_com_fresta();
    let px = pixels_pela_forma_na_cpu(&bake, &rig).expect("a regua acende");
    let (w, h) = bake.size;
    let mut soma = [[0f64; 2]; 2]; // [cima|baixo][luz|codigo]
    let mut n = [0usize; 2];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if bake.form[i * 4 + 3] <= 0.0 {
                continue;
            }
            let meta = usize::from(y >= LADO / 2);
            let p = &px[i * 4..i * 4 + 4];
            soma[meta][0] += lum(p);
            soma[meta][1] +=
                0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]);
            n[meta] += 1;
        }
    }
    let media = |m: usize, u: usize| soma[m][u] / n[m] as f64;
    println!();
    for (u, nome) in [(0usize, "LUZ (descodificado)"), (1, "CODIGO (cru)")] {
        println!(
            "  {nome:22}  cima {:9.4}  baixo {:9.4}  razao {:.4}",
            media(0, u),
            media(1, u),
            media(0, u) / media(1, u)
        );
    }
    println!("\n  a barra do gate e' 1,50 — e ela foi calibrada num vale medido em LUZ.\n");
}

/// ⛔⛔⛔ **UMA PEÇA SINTÉTICA ACENDE POR CIMA — e a fixtura da casa acendia por BAIXO.**
///
/// # O que se afirma
///
/// A lâmpada de fábrica ([`ph2d_light::Light::KEY`]) declara-se *«superior-esquerda a 30°»* e
/// resolve para `[−0,557, −0,663, +0,500]`. O canal da FORMA é escrito pelo `canvas_normal` do
/// `ph2d-mesh-render`, cuja última linha é `vec3(n.x, −n.y, n.z)` — **espaço de CANVAS, `y` para
/// BAIXO** — e o comentário ao lado dela diz exactamente o que esta fixtura fazia: *«sem esta
/// negação a mesma lâmpada acende a pintura por cima e a escultura por baixo»*.
///
/// ⇒ com o rig de fábrica, a metade de CIMA do ecrã tem de ser a mais clara.
///
/// # ⚠️ Porque nenhuma régua desta crate o podia ver
///
/// A paridade lê `100,000 %` dos bytes porque **os dois motores leem os MESMOS planos** — ela é uma
/// RELAÇÃO, e duas leis que acendem a peça de baixo concordam na mesma. E a calibração do
/// [`crate::lei_da_luz::OLHAR_DA_FORMA`] mede a **média do miolo**, que numa bola é praticamente
/// invariante a virar a luz ao contrário. *A grandeza que discrimina só passou a ser necessária
/// quando o ambiente ganhou DIRECÇÃO.*
///
/// # ⚠️ O CONTROLO está dentro
///
/// Sem ele o gate ficaria verde sobre uma cena que por acaso é mais clara em cima; com ele afirma-se
/// que **é o sinal do `y` que decide**, e a barra é o vale medido (razão `2,4×` para cada lado).
///
/// # ⛔⛔ MUTAÇÃO SOBREVIVENTE, NOMEADA com a medição — e o que ela compra é MARGEM
///
/// Devolver o [`lum`] a pesar **códigos** (o estado de antes de 2026-09-21) deixa este gate
/// **VERDE**, logo nenhuma régua desta casa discrimina hoje aquela correcção de unidades. ⚠️ *Isso
/// não a torna opcional — torna-a não-VERIFICADA*, e o número diz porquê
/// ([`diag_a_razao_nas_duas_unidades`]):
///
/// ```text
///   LUZ (descodificado)   cima 0,3808   baixo 0,1596   razao 2,3862   <- margem  +59 %
///   CODIGO (cru)          cima 161,33   baixo 107,04   razao 1,5071   <- margem +0,5 %
/// ```
///
/// ⛔ **Uma barra que passa por `0,0071` não é uma barra, é uma coincidência** — está a uma
/// re-corrida infeliz de virar flake. E o poder DISCRIMINANTE cai com ela: o vale entre o produto e
/// o controlo mede **`5,7×` em luz** contra `2,27×` em códigos.
///
/// ⚠️ **A régua que faltaria** é uma que afirme a SEPARAÇÃO (produto ÷ controlo) e não só o lado
/// certo — e ela fica por escrever de propósito: a barra dela não tem hoje fonte independente, e
/// escolher um número para a poder ter seria o palpite que o §0.0 proíbe.
#[test]
fn a_peca_sintetica_acende_por_cima() {
    let rig = LightRig::default();
    let luz = ph2d_light::resolve(&rig).expect("ha lampada").lamps()[0].dir;
    assert!(
        luz[1] < 0.0,
        "a lampada de fabrica aponta para CIMA no canvas (y<0); leu {:.3}",
        luz[1]
    );

    let bake = bola_com_fresta();
    let (cima, baixo) = metades_da_bola(&bake, &rig, |y| y < LADO / 2);
    assert!(
        cima > baixo * 1.5,
        "com a lampada superior-esquerda a metade de CIMA tem de ser a mais clara: \
         cima {cima:.2} contra baixo {baixo:.2}"
    );

    // **O CONTROLO**: virar o sinal do `y` da normal inverte a leitura. É o estado em que esta
    // fixtura viveu, e o que ele produz é uma peça iluminada de baixo.
    let mut virada = bola_com_fresta();
    for i in 0..(LADO * LADO) as usize {
        virada.form[i * 4 + 1] = -virada.form[i * 4 + 1];
    }
    let (c2, b2) = metades_da_bola(&virada, &rig, |y| y < LADO / 2);
    assert!(
        b2 > c2 * 1.5,
        "controlo: com o sinal do y trocado a peca tem de acender por BAIXO: \
         cima {c2:.2} contra baixo {b2:.2}"
    );
}

/// ⏱️ **SONDA — O QUE A INDIRECTA DO OpenPBR MUDA NO PIXEL, por material.**
///
/// A coluna **B3** trocou `albedo × E(n) × oclusão` (só a metade DIFUSA do céu) pela
/// [`ph2d_material::Surface::indirect`], que soma também a **espelhada pré-filtrada**. Esta sonda
/// mede o que isso vale, e a resposta **depende do material** — que é exactamente a razão de ela
/// varrer três.
///
/// ⚠️ **A lei de ONTEM é composta à mão aqui, e declara-se como tal**: ela é `luz + albedo × E(n) ×
/// occ`, a linha que a [`ph2d_form_pbr::acende_texel`] tinha antes de 2026-09-20. *Guardá-la num
/// `#[cfg(test)]` é o que torna «quanto é que isto mudou» uma pergunta respondível amanhã.*
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-form-donation --release \
///   diag_o_que_a_indirecta_muda -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn diag_o_que_a_indirecta_muda_no_pixel() {
    use ph2d_form_pbr::{Lampada, OpenPbr, Surface, Texel, VISTA};

    let bake = bola_com_fresta();
    let rig = LightRig::default();
    let lampadas = crate::baked_form::lampadas_do_rig(&rig).expect("o rig de fábrica resolve");
    let ceu = crate::baked_form::ceu_do_rig(&lampadas);
    let olhar = crate::lei_da_luz::OLHAR_DA_FORMA;

    // ⚠️ **A LEI DE ONTEM**, linha a linha — o ambiente lambertiano e mais nada.
    let de_ontem = |s: &Surface, t: &Texel, ls: &[Lampada]| -> [f32; 3] {
        let q: f32 = t.normal.iter().map(|c| c * c).sum();
        if q < 1e-12 {
            return t.albedo;
        }
        let inv = 1.0 / q.sqrt();
        let n = [t.normal[0] * inv, t.normal[1] * inv, t.normal[2] * inv];
        let s = s.at_base_color(t.albedo);
        let mut luz = [0.0f32; 3];
        for l in ls {
            let h: f32 = (0..3).map(|i| (VISTA[i] + l.para_a_luz[i]).powi(2)).sum();
            if h < 1e-12 {
                continue;
            }
            let r = s.direct(n, VISTA, l.para_a_luz, l.radiancia);
            for i in 0..3 {
                luz[i] += r[i];
            }
        }
        let e = ceu.irradiancia(n);
        let aceso = olhar.apply([0, 1, 2].map(|i| luz[i] + t.albedo[i] * e[i] * t.oclusao));
        let c = t.cobertura.clamp(0.0, 1.0);
        [0, 1, 2].map(|i| t.albedo[i] * (1.0 - c) + aceso[i] * c)
    };

    println!(
        "\n  o céu: base {:?}  inclinação {:?}",
        ceu.base, ceu.inclinacao
    );
    println!(
        "  {:<22} {:>8} {:>8} {:>8} {:>10}",
        "material", "Δ média", "Δ p99", "Δ máx", "texels ≠"
    );
    for (nome, m) in [
        ("barro (o de FÁBRICA)", OpenPbr::default()),
        (
            "dieléctrico polido",
            OpenPbr {
                specular_roughness: 0.15,
                ..Default::default()
            },
        ),
        (
            "metal polido",
            OpenPbr {
                base_metalness: 1.0,
                specular_roughness: 0.15,
                ..Default::default()
            },
        ),
    ] {
        let s = m.prepare();
        let (w, h) = bake.size;
        let mut deltas: Vec<u16> = Vec::new();
        let mut n_dif = 0usize;
        for i in 0..(w * h) as usize {
            if bake.form[i * 4 + 3] <= 0.0 {
                continue;
            }
            let t = Texel {
                normal: [bake.form[i * 4], bake.form[i * 4 + 1], bake.form[i * 4 + 2]],
                albedo: [0, 1, 2].map(|k| f32::from(bake.base[i * 4 + k]) / 255.0),
                cobertura: bake.form[i * 4 + 3],
                oclusao: bake.form_occ[i],
            };
            let byte = |c: [f32; 3]| c.map(|x| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as i32);
            let novo = byte(ph2d_form_pbr::acende_texel(&s, &t, &lampadas, ceu, olhar));
            let velho = byte(de_ontem(&s, &t, &lampadas));
            let d = (0..3)
                .map(|k| (novo[k] - velho[k]).abs())
                .max()
                .unwrap_or(0);
            if d > 0 {
                n_dif += 1;
            }
            deltas.push(u16::try_from(d).unwrap_or(u16::MAX));
        }
        deltas.sort_unstable();
        let media = f64::from(deltas.iter().map(|d| u32::from(*d)).sum::<u32>())
            / deltas.len().max(1) as f64;
        let p99 = deltas[deltas.len() * 99 / 100];
        let max = deltas.last().copied().unwrap_or(0);
        println!(
            "  {nome:<22} {media:>8.2} {p99:>8} {max:>8} {:>9.1} %",
            100.0 * n_dif as f64 / deltas.len().max(1) as f64
        );
    }
    // ⭐⭐⭐ **O CASO SEM LÂMPADA NENHUMA** — ali o ambiente é a imagem inteira, e é onde as duas
    // leis não se parecem nada.
    println!("\n  SEM LÂMPADA NENHUMA — só o céu (normal virada ao topo da TELA):");
    let t = Texel {
        normal: [0.0, -0.6, 0.8],
        albedo: [0.742, 0.702, 0.659],
        cobertura: 1.0,
        oclusao: 1.0,
    };
    for (nome, m) in [
        ("barro (o de FÁBRICA)", OpenPbr::default()),
        (
            "metal polido",
            OpenPbr {
                base_metalness: 1.0,
                specular_roughness: 0.15,
                ..Default::default()
            },
        ),
    ] {
        let s = m.prepare();
        let novo = ph2d_form_pbr::acende_texel(&s, &t, &[], ceu, olhar);
        let velho = de_ontem(&s, &t, &[]);
        println!("  {nome:<22} ontem {velho:?}\n  {:<22} hoje  {novo:?}", "");
    }
    println!(
        "\n  ⇒ a metade que chegou e' a ESPELHADA. ⛔ E a lei de ontem NAO desenhava um metal\n    \
         PRETO: ela dava a TODO material o mesmo ambiente LAMBERTIANO — inclusive a um metal, que\n    \
         no OpenPBR nao tem lobulo difuso NENHUM. *Ela nao errava a quantidade; errava a CLOSURE.*"
    );
}
