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
    }
}

/// A luminância de um texel, na régua de Rec.709.
fn lum(px: &[u8]) -> f64 {
    0.2126 * f64::from(px[0]) + 0.7152 * f64::from(px[1]) + 0.0722 * f64::from(px[2])
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
    println!("  a lei da TINTA e o barro VIVO dobram-na assim; esta dobra-a a ZERO.");
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
/// que **é o sinal do `y` que decide**, e a barra é o vale medido (razão `2,6×` para cada lado).
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
