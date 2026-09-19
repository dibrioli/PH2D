//! ⭐⭐⭐ **A LEI DA COR — porque a matiz de um jade muda com a profundidade, e a nossa não.**
//!
//! Ordem do dono, 2026-09-18, depois do veredito a quatro colunas da
//! [`§16`](../../../docs/Render3d/10_a_luz_que_atravessa_a_peca.md): *«atacamos agora a cor»*.
//!
//! # ⛔⛔ O mecanismo da nossa surdez, e ele está numa DIVISÃO
//!
//! O [`ph2d_material::subsurface::integrate_burley`] devolve `Σ(R·w) / Σ R` — um quociente em que
//! o perfil `R` aparece **em cima e em baixo**. Tudo o que o perfil sabe sobre a profundidade (a
//! forma `1/mfp`, as duas exponenciais, a escala) **cancela-se na divisão**, e o que sobra é uma
//! média direccional pura. A cor sai depois por `sss = subsurface_color × isso`.
//!
//! ⇒ *a informação da profundidade não se perde por aproximação: ela é DIVIDIDA FORA por
//! construção.* Com o `mfp` partilhado pelos três canais o quociente é o mesmo nos três, e a matiz
//! que sai **é** a que o artista escreveu no painel — a qualquer profundidade.
//!
//! # ⭐⭐⭐ E o mecanismo da VERDADE é um expoente, não um multiplicador
//!
//! Num passeio aleatório, cada evento de espalhamento multiplica a luz pelo albedo do canal. Um
//! `mfp` curto ⇒ **muitos** eventos antes de sair ⇒ a cor é multiplicada muitas vezes ⇒ **saturada**.
//! Um `mfp` da ordem da peça ⇒ **poucos** eventos ⇒ a cor mal é multiplicada ⇒ **lava para o
//! branco**. ⇒ a matiz que volta é `albedo^p`, com `p` a **cair** com a profundidade.
//!
//! Esta sonda mede esse `p` nas DUAS verdades, e mede o nosso pela porta do material.
//! ⛔ **Ela não propõe lei nenhuma** — a lei escreve-se depois de as duas verdades concordarem
//! sobre a curva, e não antes.
//!
//! # ⚠️ A cerca que as réguas de hoje NÃO têm: a ORIENTAÇÃO
//!
//! As linhas de um PFM vêm de **baixo para cima**. A [`super::oraculo::razao_rb`] e a
//! [`super::oraculo::casa_a_populacao`] somam sobre um limiar, logo são **invariantes a um espelho
//! vertical** — a janela E notou isso por conta própria e passou a reportar a posição do píxel mais
//! brilhante. Uma **máscara geométrica fixa**, que é o que esta sonda usa, *não* é invariante: um
//! ficheiro ao contrário mediria o fundo. ⇒ a cerca é medida, não assumida.

use super::oraculo::le_pfm;
use super::{H, Quadro, W, quadro};
use ph2d_field_render::Orbit;

/// O raio da esfera da cena `=33`, em unidades do mundo — ⛔ leia-o da cena, não daqui, se ela mudar.
const RAIO_DA_PECA: f32 = 0.42;

/// A cor autorada das duas fixturas (base e subsuperfície).
const COR: [f32; 3] = [0.75, 0.35, 0.35];

/// As profundidades onde **as duas** verdades existem, na família de raios IGUAIS.
///
/// ⭐ É a família de raios iguais porque ela **isola a pergunta**: com os três canais a viajar a
/// mesma distância, toda mudança de matiz é da PROFUNDIDADE e nenhuma é de os canais viajarem
/// distâncias diferentes.
const PROFUNDIDADES: [(&str, f32); 4] = [
    ("g003", 0.03),
    ("g010", 0.10),
    ("g030", 0.30),
    ("g100", 1.00),
];

/// ⏱️⛔⛔⛔ **QUANDO É QUE A CURA SE VÊ? — a sonda que o report *«não vejo diferença»* obrigou.**
///
/// O dono correu as duas metades lado a lado e leu a **mesma imagem**. ⚠️ Antes de procurar um fio
/// partido, a pergunta certa é *de que tamanho é o efeito NAS DEFINIÇÕES DELE* — porque a lei mexe
/// na saturação **relativa entre canais**, e a cena `=33` abre com `subsurface_color` no valor de
/// omissão, que é **cinzento** (`0,8 · 0,8 · 0,8`).
///
/// ⛔ *Uma lei que muda o contraste entre canais quase não tem o que fazer num material onde os três
/// canais já são iguais* — e a medição de onde ela veio (§17) usou um jade **saturado**
/// (`0,75 · 0,35 · 0,35`), com a subsuperfície a `1` e o especular a `0`.
///
/// Esta sonda varre as combinações e imprime o que o **OLHO** recebe (bytes), não a lei nua.
#[test]
#[ignore = "sonda: imprime uma tabela, não afirma"]
fn sonda_quando_a_cura_se_ve() {
    let olhar = ph2d_view_transform::Look::default();
    let bytes = |c: [f32; 3]| -> [i32; 3] {
        olhar.apply(c).map(|v| {
            #[allow(clippy::cast_possible_truncation)]
            {
                (v.clamp(0.0, 1.0) * 255.0 + 0.5) as i32
            }
        })
    };
    println!(
        "\n  ── QUANDO A CURA SE VÊ (bytes do olho, luz branca de π, κ = 1/0,42) ──\n    \
         cor da subsup.  · peso · espec ·  N·L ·      SEM a cura ·     COM a cura · Δ máx"
    );
    for (nome, cor) in [
        ("CINZENTO .8", [0.8f32, 0.8, 0.8]),
        ("o JADE §17 ", [0.75, 0.35, 0.35]),
        ("VERMELHO 1 ", [1.0, 0.0, 0.0]),
        ("âmbar      ", [0.9, 0.55, 0.2]),
    ] {
        for (peso_ss, espec) in [(0.514f32, 1.0f32), (1.0, 0.0)] {
            for ndl in [0.4f32, 0.0, -0.3] {
                let faz = |cura: f32| {
                    let s = ph2d_material::OpenPbr {
                        subsurface_weight: peso_ss,
                        subsurface_color: cor,
                        subsurface_radius: 1.0,
                        subsurface_radius_scale: [1.0, 0.5, 0.25],
                        subsurface_depth_hue: cura,
                        specular_weight: espec,
                        geometry_thin_walled: false,
                        ..ph2d_material::OpenPbr::default()
                    }
                    .prepare()
                    .at_curvature(1.0 / RAIO_DA_PECA);
                    let l = [(1.0 - ndl * ndl).max(0.0).sqrt(), 0.0, ndl];
                    bytes(s.direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], l, [1.0; 3]))
                };
                let (a, b) = (faz(0.0), faz(1.0));
                let d = (0..3).map(|k| (a[k] - b[k]).abs()).max().unwrap_or(0);
                println!(
                    "    {nome} · {peso_ss:.2} ·  {espec:.0}   · {ndl:>4.1} · {a:>3?} · {b:>3?} · \
                     {d:>3}{}",
                    if d >= 3 { "  ⭐ VISÍVEL" } else { "" }
                );
            }
        }
    }
    println!(
        "\n    ⚠️ Um byte de diferença é INVISÍVEL num ecrã. A lei precisa de canais com valores\n \
         \x20     DIFERENTES entre si para ter o que mover — num material cinzento ela quase não age."
    );
}

/// ⏱️⭐⭐⭐ **O QUADRO INTEIRO, os dois lados — quantos bytes o ECRÃ de facto muda.**
///
/// ⛔⛔ A [`sonda_quando_a_cura_se_ve`] mede a lei com radiância unitária, que é a lei e não a CENA:
/// os bytes dela são todos baixos porque a lâmpada do produto é outra. *Prometer visibilidade a
/// partir daquela tabela seria prometer sobre outro programa* — esta renderiza a `=33` de verdade,
/// pelo mesmo caminho do smoke, e conta píxeis.
#[test]
#[ignore = "sonda: imprime uma tabela, não afirma"]
fn sonda_o_ecra_com_e_sem_a_cura() {
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let (onde, luz) = crate::lights::opening_light(&cam);
    println!("\n  ── O QUADRO INTEIRO da `=33`, com e sem a cura ──");
    println!("    material            · peso · Δ médio · Δ p99 · Δ MÁX · px mudados");
    for (nome, cor, peso_ss, espec) in [
        ("cinzento (a cena)  ", [0.8f32, 0.8, 0.8], 0.514f32, 1.0f32),
        ("cinzento, sss=1    ", [0.8, 0.8, 0.8], 1.0, 0.0),
        ("JADE (a medição)   ", [0.75, 0.35, 0.35], 1.0, 0.0),
        ("JADE, como a cena  ", [0.75, 0.35, 0.35], 0.514, 1.0),
        ("âmbar, sss=1       ", [0.9, 0.55, 0.2], 1.0, 0.0),
    ] {
        let faz = |cura: f32| {
            let (_, _, px) = quadro(&Quadro {
                doc: &doc,
                m: ph2d_material::OpenPbr {
                    subsurface_weight: peso_ss,
                    subsurface_color: cor,
                    base_color: cor,
                    subsurface_radius: 1.0,
                    subsurface_radius_scale: [1.0, 0.5, 0.25],
                    subsurface_depth_hue: cura,
                    specular_weight: espec,
                    geometry_thin_walled: false,
                    ..ph2d_material::OpenPbr::default()
                },
                cam: &cam,
                onde,
                luz,
                com_sombra: true,
                chao: None,
                sem_ceu: false,
                mole: true,
            });
            px
        };
        let (a, b) = (faz(0.0), faz(1.0));
        let mut deltas: Vec<i32> = Vec::new();
        for i in 0..(W as usize) * (H as usize) {
            let q = i * 4;
            let soma = u32::from(a[q]) + u32::from(a[q + 1]) + u32::from(a[q + 2]);
            if soma > 30 {
                deltas.push(
                    (0..3)
                        .map(|k| i32::from(a[q + k]).abs_diff(i32::from(b[q + k])) as i32)
                        .max()
                        .unwrap_or(0),
                );
            }
        }
        deltas.sort_unstable();
        let n = deltas.len().max(1);
        #[allow(clippy::cast_precision_loss)]
        let media = deltas.iter().sum::<i32>() as f32 / n as f32;
        let p99 = deltas[(n * 99 / 100).min(n - 1)];
        let maximo = deltas.last().copied().unwrap_or(0);
        let mudados = deltas.iter().filter(|d| **d >= 2).count();
        println!(
            "    {nome} · {peso_ss:.2} · {media:>7.2} · {p99:>5} · {maximo:>5} · {mudados:>5} de {n}"
        );
    }
    println!(
        "\n    ⚠️ Lado a lado, um olho treinado apanha `2`–`3` bytes numa área grande; abaixo disso\n \
         \x20     a promessa de «vai ver a diferença» é falsa, e é ela que este número julga."
    );
}

/// `R/B` **linear** sobre uma máscara fixa.
///
/// ⭐ Em linear o quociente é **invariante à exposição** (os dois canais escalam juntos), logo aqui
/// não há exposição a casar — e é por isso que este é o espaço certo para a pergunta da matiz. Ver
/// a [`§16.3`](../../../docs/Render3d/10_a_luz_que_atravessa_a_peca.md), onde a régua de bytes
/// chega a **inverter a ordem** de duas colunas.
fn rb_linear(px: &[[f32; 3]], mascara: &[bool]) -> f32 {
    let (mut r, mut b) = (0.0f64, 0.0f64);
    for (i, &m) in mascara.iter().enumerate() {
        if m {
            r += f64::from(px[i][0]);
            b += f64::from(px[i][2]);
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((r / b.max(1e-12)) as f32)
}

/// Onde está o píxel mais brilhante — a **cerca de orientação**.
fn mais_brilhante(px: &[[f32; 3]]) -> (usize, usize) {
    let w = W as usize;
    let i = px
        .iter()
        .enumerate()
        .max_by(|a, b| {
            let s = |c: &[f32; 3]| c[0] + c[1] + c[2];
            s(a.1).total_cmp(&s(b.1))
        })
        .map_or(0, |p| p.0);
    (i % w, i / w)
}

/// O expoente da matiz: `R/B = (0,75/0,35)^p`.
///
/// ⭐ `p = 1` quer dizer *«a cor que sai é a cor autorada»*; `p = 0` quer dizer *«branco»*.
fn expoente(rb: f32) -> f32 {
    rb.max(1e-6).ln() / (COR[0] / COR[2]).ln()
}

/// ⏱️⭐⭐⭐ **A CURVA DA MATIZ CONTRA A PROFUNDIDADE, nas duas verdades e na nossa lei.**
///
/// ```text
/// PH2D_UNREAL=/var/tmp/ph2d-oraculo-unreal/out \
/// PH2D_VERDADE2=/var/tmp/ph2d-verdade-cycles \
///   cargo test -p ph2d-app-field3d --lib sonda_a_lei_da_cor -- --ignored --nocapture
/// ```
///
/// ⛔ **Ela mede e não cura.** Uma lei escrita antes de as duas verdades concordarem sobre a curva
/// seria um ajuste a três pontos com cara de mecanismo — que é exactamente o que este repo chama de
/// *«uma recusa medida responde UMA pergunta»* ao contrário.
#[test]
#[ignore = "sonda: precisa dos dois oráculos em $PH2D_UNREAL e $PH2D_VERDADE2"]
fn sonda_a_lei_da_cor_da_profundidade() {
    let (Ok(dir_u), Ok(dir_v)) = (std::env::var("PH2D_UNREAL"), std::env::var("PH2D_VERDADE2"))
    else {
        println!("sem $PH2D_UNREAL e/ou $PH2D_VERDADE2 — saltado");
        return;
    };

    // ── A MÁSCARA: a silhueta iluminada da bola OPACA, a mesma para todas as células ────────────
    // ⚠️ Fixa de propósito. A janela E mediu que uma máscara derivada da própria imagem (um limiar
    // sobre um percentil) **anda com o ruído**: o traçado dela lia `1,2170 → 1,3122 → 1,3516` ao
    // subir amostras, e o que se movia era a régua. Com máscara fixa, `0,117 %`.
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let (onde, luz) = crate::lights::opening_light(&cam);
    let (_, _, opaco) = quadro(&Quadro {
        doc: &doc,
        m: ph2d_material::OpenPbr {
            subsurface_weight: 0.0,
            base_color: COR,
            specular_weight: 0.0,
            ..ph2d_material::OpenPbr::default()
        },
        cam: &cam,
        onde,
        luz,
        com_sombra: true,
        chao: None,
        sem_ceu: true,
        mole: true,
    });
    let n_px = (W as usize) * (H as usize);
    let mascara: Vec<bool> = (0..n_px)
        .map(|i| {
            let q = i * 4;
            u32::from(opaco[q]) + u32::from(opaco[q + 1]) + u32::from(opaco[q + 2]) > 30
        })
        .collect();
    let vivos = mascara.iter().filter(|m| **m).count();
    println!("\n  máscara fixa (silhueta iluminada do opaco): {vivos} px de {n_px}");

    // ── A CERCA DE ORIENTAÇÃO ───────────────────────────────────────────────────────────────────
    let nosso_pico = {
        let w = W as usize;
        let i = (0..n_px)
            .max_by_key(|&i| {
                u32::from(opaco[i * 4]) + u32::from(opaco[i * 4 + 1]) + u32::from(opaco[i * 4 + 2])
            })
            .unwrap_or(0);
        (i % w, i / w)
    };
    println!("  pico do NOSSO opaco: {nosso_pico:?}");

    // ── A CURVA ─────────────────────────────────────────────────────────────────────────────────
    println!(
        "\n  ── A MATIZ CONTRA A PROFUNDIDADE (raios IGUAIS, LINEAR, máscara fixa) ──\n    \
         mfp   · mfp/raio ·   CYCLES R/B ·   p ·  UNREAL-PT R/B ·   p"
    );
    let mut anterior: Option<f32> = None;
    let mut verdade: Vec<Option<f32>> = Vec::new();
    for (tag, mfp) in PROFUNDIDADES {
        let ler = |caminho: String| -> Option<f32> {
            let (_, _, px) = le_pfm(&caminho)?;
            let pico = mais_brilhante(&px);
            // ⛔ Um ficheiro ao contrário põe o pico a ~`H − y` daqui, e a máscara mediria o fundo.
            let dy = pico.1.abs_diff(nosso_pico.1);
            assert!(
                dy < 40,
                "ORIENTAÇÃO: o pico de {caminho} está em {pico:?} e o nosso em {nosso_pico:?} \
                 (Δy = {dy}) — o ficheiro pode estar espelhado na vertical"
            );
            Some(rb_linear(&px, &mascara))
        };
        let cy = ler(format!("{dir_v}/ref_jade_{tag}_e5.pfm"));
        let un = ler(format!("{dir_u}/jade_{tag}_pathtracer.pfm"));
        let sat = un.is_some_and(|u| {
            anterior.is_some_and(|a| (u - a).abs() <= super::unreal_contendor::INDISTINGUIVEL * a)
        });
        anterior = un;
        let f = |o: Option<f32>| {
            o.map_or_else(
                || "         —      —".to_string(),
                |v| format!("{v:>9.4} · {:>5.3}", expoente(v)),
            )
        };
        println!(
            "    {mfp:>5.2} ·  {:>6.3}  · {} · {}{}",
            mfp / RAIO_DA_PECA,
            f(cy),
            f(un),
            if sat { "  ⛔ SATURADO" } else { "" }
        );
        verdade.push(cy);
    }

    // ── A NOSSA LEI, medida pela PORTA DO MATERIAL e não pela imagem ─────────────────────────────
    //
    // ⭐⭐ Aqui a resposta é EXACTA e não precisa de um quadro linear: `direct` devolve radiância
    // linear, e com luz branca o quociente é a matiz da lei. *A surdez deixa de ser um argumento e
    // passa a ser uma corrida.*
    println!("\n  ── A NOSSA LEI, pela porta do material (luz branca, radiância linear) ──");
    println!("    mfp   · N·L=0,9 · N·L=0,4 · N·L=0,0 · N·L=−0,3   ‖ MAGNITUDE (canal R)");
    let mut todos: Vec<f32> = Vec::new();
    // ⚠️⚠️ **A MAGNITUDE ao lado da matiz, e ela responde a OUTRA pergunta.** A matiz é constante
    // por álgebra; a magnitude é o que o `integrate_burley` de facto devolve. ⛔ E há uma cerca a
    // testar: aquele integral faz `max(mfp, 0.1)` — um piso em unidades **absolutas de mundo** —, e
    // a peça desta cena tem raio `0,42`. *Se duas profundidades diferentes derem a MESMA magnitude,
    // o botão do artista está inerte naquela faixa*, e isso decide o domínio da lei nova.
    //
    // ⛔⛔⛔ **E há uma armadilha de RÉGUA aqui, paga em 18/09:** a 1.ª redacção imprimia a
    // magnitude **só em `N·L = 0,4`** e leu `1,03×` sobre `33×` de profundidade — eu quase publiquei
    // *«o botão está quase morto»*. Medidos os QUATRO ângulos, ela lê `1,76×` no terminador e
    // **`3,50×`** do lado escuro. ⇒ *o `0,4` é o PIVÔ da redistribuição, o único sítio onde esta lei
    // por construção quase não se mexe* — e a razão de ela existir é o terminador, que é onde eu
    // não estava a olhar. **Uma régua que amostra um ângulo só mede o sítio onde o fenómeno não
    // está**, e é por isso que os quatro ficam impressos.
    let mut magnitudes: Vec<[f32; 4]> = Vec::new();
    for (_, mfp) in PROFUNDIDADES {
        let s = ph2d_material::OpenPbr {
            subsurface_weight: 1.0,
            geometry_thin_walled: false,
            subsurface_color: COR,
            base_color: COR,
            specular_weight: 0.0,
            subsurface_radius: mfp,
            subsurface_radius_scale: [1.0, 1.0, 1.0],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare()
        .at_curvature(1.0 / RAIO_DA_PECA);
        let n = [0.0, 0.0, 1.0];
        let v = [0.0, 0.0, 1.0];
        let mut linha = format!("    {mfp:>5.2} ·");
        let mut mags = [0.0f32; 4];
        for (j, ndl) in [0.9f32, 0.4, 0.0, -0.3].into_iter().enumerate() {
            let l = [(1.0 - ndl * ndl).max(0.0).sqrt(), 0.0, ndl];
            let c = s.direct(n, v, l, [1.0, 1.0, 1.0]);
            let rb = c[0] / c[2].max(1e-12);
            todos.push(rb);
            linha += &format!(" {rb:>8.5} ·");
            mags[j] = c[0];
        }
        magnitudes.push(mags);
        println!(
            "{linha}  ‖ {:>9.6} {:>9.6} {:>9.6} {:>9.6}",
            mags[0], mags[1], mags[2], mags[3]
        );
    }
    // ⭐⭐⭐ **A razão de existir desta lei é AMACIAR O TERMINADOR**, logo a resposta tem de aparecer
    // em `N·L ≈ 0` e do lado de lá — medir só de frente mede o sítio onde ela não tem trabalho.
    println!("\n    ⇒ quanto a MAGNITUDE se mexe sobre as quatro profundidades, por ângulo:");
    for (j, ndl) in ["N·L=0,9", "N·L=0,4", "N·L=0,0", "N·L=−0,3"]
        .into_iter()
        .enumerate()
    {
        let v: Vec<f32> = magnitudes.iter().map(|m| m[j]).collect();
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        println!(
            "       {ndl:<9} · {lo:.6} .. {hi:.6} · balanço {:.3}×",
            hi / lo.max(1e-12)
        );
    }
    for (i, (tag, mfp)) in PROFUNDIDADES.iter().enumerate().skip(1) {
        let (a, b) = (magnitudes[i - 1], magnitudes[i]);
        if (0..4).all(|j| (a[j] - b[j]).abs() <= 1e-7 * a[j].abs().max(1e-9)) {
            println!(
                "    ⛔⛔ {tag} (mfp {mfp}) é IDÊNTICO ao anterior nos QUATRO ângulos — o botão está \
                 INERTE nesta faixa (o `max(mfp, 0.1)` do integral é um piso em unidades ABSOLUTAS \
                 de mundo, e a peça desta cena tem raio 0,42)"
            );
        }
    }
    let (lo, hi) = todos
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    println!(
        "\n    ⇒ a nossa matiz varre [{lo:.5} .. {hi:.5}] · balanço {:.4}× · p = {:.4}\n       (a cor \
         autorada é {:.5}; o OPACO lê o mesmo)",
        hi / lo.max(1e-12),
        expoente(lo),
        COR[0] / COR[2]
    );

    // ── ⭐⭐⭐ O QUE A CURA COMPRA ────────────────────────────────────────────────────────────────
    //
    // A mesma porta do material, com o `subsurface_depth_hue` a `0` (a aproximação publicada) e a
    // `1` (a lei nova). ⚠️ A matiz é a mesma em todo ângulo — a lei toca na COR e não na forma —,
    // logo um ângulo chega, e os quatro acima já provaram isso.
    println!("\n  ── ⭐ O QUE A CURA COMPRA (matiz R/B contra a VERDADE) ──");
    println!("    mfp/raio ·  VERDADE ·  NÓS hoje ·   erro ·  NÓS c/ cura ·   erro");
    let (mut pior_hoje, mut pior_cura) = (0.0f32, 0.0f32);
    for (i, (_, mfp)) in PROFUNDIDADES.iter().enumerate() {
        let Some(v) = verdade[i] else { continue };
        let matiz = |peso: f32| -> f32 {
            let s = ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: false,
                subsurface_color: COR,
                base_color: COR,
                specular_weight: 0.0,
                subsurface_radius: *mfp,
                subsurface_radius_scale: [1.0, 1.0, 1.0],
                subsurface_depth_hue: peso,
                ..ph2d_material::OpenPbr::default()
            }
            .prepare()
            .at_curvature(1.0 / RAIO_DA_PECA);
            let c = s.direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.6, 0.0, 0.8], [1.0; 3]);
            c[0] / c[2].max(1e-12)
        };
        let (h, cura) = (matiz(0.0), matiz(1.0));
        let (eh, ec) = (100.0 * (h / v - 1.0), 100.0 * (cura / v - 1.0));
        pior_hoje = pior_hoje.max(eh.abs());
        pior_cura = pior_cura.max(ec.abs());
        println!(
            "     {:>7.4} · {v:>8.4} · {h:>9.4} · {eh:>+6.1} % · {cura:>12.4} · {ec:>+6.1} %",
            mfp / RAIO_DA_PECA
        );
    }
    println!(
        "\n    ⇒ pior erro de matiz: HOJE {pior_hoje:.1} %  ·  COM A CURA {pior_cura:.1} %  \
         ({:.1}× melhor)",
        pior_hoje / pior_cura.max(1e-6)
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
//  OS GATES DA CENA `=34` — e o primeiro deles é o que teria apanhado o erro de 18/09
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **A CENA CONTÉM O FENÓMENO QUE PROMETE — e este gate nasceu de um report do dono.**
///
/// A lei da matiz foi construída, medida e ligada a um interruptor, e o smoke que eu escrevi
/// mandava o dono olhar para a `=33` — que abre com o material de omissão, **CINZENTO**. Uma lei que
/// redistribui saturação **entre canais** não tem o que fazer quando os três já são iguais, logo ele
/// correu as duas metades e viu a mesma imagem. *O defeito era do smoke, não da lei.*
///
/// ⛔⛔ **Nenhum gate deste repo fazia esta pergunta:** havia gates a provar que a cena constrói,
/// que ela é ela própria, que o roteiro é anunciado — e **nenhum** a perguntar *«a cena mostra o que
/// o roteiro diz que ela mostra?»*. É a família do `CLAUDE.md` §5.0 (*uma cena que ensina o
/// contrário é pior que uma cena ausente*) num degrau acima: aqui a cena não ensinava o contrário,
/// **não podia ensinar nada**.
///
/// ⚠️ A barra é `0,15` de matiz (`15 %`) porque no quadro inteiro o jade mede `23` bytes de média
/// contra `1` do cinzento — *uma barra que o cinzento passasse não afirmaria nada*, e a segunda
/// metade deste gate é exactamente esse controlo.
#[test]
fn a_cena_da_cor_contem_o_fenomeno_que_promete() {
    let pedidos = crate::smoke::scenes::materiais_da_cena(34).expect("a `=34` declara materiais");
    assert_eq!(pedidos.len(), 4, "são as quatro profundidades do oráculo");

    let matiz = |m: &ph2d_field_ecs::FieldMaterial, cura: f32| -> f32 {
        let s = ph2d_material::OpenPbr {
            subsurface_weight: m.subsurface_weight,
            subsurface_color: m.subsurface_color,
            base_color: m.base_color,
            subsurface_radius: m.subsurface_radius,
            subsurface_radius_scale: m.subsurface_radius_scale,
            specular_weight: m.specular_weight,
            subsurface_depth_hue: cura,
            geometry_thin_walled: false,
            ..ph2d_material::OpenPbr::default()
        }
        .prepare()
        .at_curvature(1.0 / 0.30); // o raio das esferas desta cena
        let c = s.direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.6, 0.0, 0.8], [1.0; 3]);
        c[0] / c[2].max(1e-12)
    };

    // (a) O interruptor MEXE em cada esfera.
    let mut pior = 0.0f32;
    for m in &pedidos {
        let (sem, com) = (matiz(m, 0.0), matiz(m, 1.0));
        pior = pior.max((com / sem - 1.0).abs());
    }
    assert!(
        pior >= 0.15,
        "a cena mal reage ao interruptor ({:.1} %) — ela não contém o fenómeno",
        100.0 * pior
    );

    // (b) ⭐ E as quatro esferas têm de ser DIFERENTES ENTRE SI com a cura ligada — é isso que faz
    //     o fenómeno ver-se DENTRO de uma imagem, e não só entre duas corridas.
    let com: Vec<f32> = pedidos.iter().map(|m| matiz(m, 1.0)).collect();
    let (lo, hi) = com
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    assert!(
        hi / lo >= 1.15,
        "com a cura as quatro esferas quase não diferem ({hi:.3} contra {lo:.3}) — o gradiente não \
         se vê numa imagem só"
    );

    // (c) ⛔ O CONTROLO, e é ele que dá sentido à barra: o material de OMISSÃO (cinzento) tem de
    //     REPROVAR na mesma régua. Sem isto, uma barra frouxa passaria com a cena que o dono viu.
    let cinzento = ph2d_field_ecs::FieldMaterial {
        subsurface_weight: 1.0,
        ..ph2d_field_ecs::FieldMaterial::default()
    };
    let (sem, com_) = (matiz(&cinzento, 0.0), matiz(&cinzento, 1.0));
    assert!(
        (com_ / sem - 1.0).abs() < 0.15,
        "o cinzento reagiu {:.1} % — o controlo deixou de separar o material que mostra do que não \
         mostra",
        100.0 * (com_ / sem - 1.0).abs()
    );
}

/// ⭐⭐ **O que a cena declara é o que a MEDIÇÃO usou** — ⛔ não uma cor bonita.
///
/// ⚠️ E a escala por canal tem de ser **IGUAL nos três**: com a de omissão (`1 · 0,5 · 0,25`) a
/// matiz também mudaria por cada canal viajar a sua distância, e a cena deixaria de responder a uma
/// pergunta só.
#[test]
fn a_cena_da_cor_declara_o_material_da_medicao() {
    let pedidos = crate::smoke::scenes::materiais_da_cena(34).expect("a `=34` declara materiais");
    for (m, esperado) in pedidos
        .iter()
        .zip(crate::smoke::scenes::edge::PROFUNDIDADES_DA_COR)
    {
        assert!(
            (m.subsurface_radius - esperado).abs() < 1e-6,
            "a profundidade saiu da tabela do oráculo"
        );
        assert_eq!(m.subsurface_color, COR, "a cor é a das fixturas");
        assert_eq!(
            m.subsurface_radius_scale, [1.0; 3],
            "raios IGUAIS nos três canais"
        );
        assert!(
            m.subsurface_weight >= 1.0,
            "subsuperfície pura, como a medição"
        );
        assert!(
            m.specular_weight <= 0.0,
            "sem realce a lavar o que se quer ver"
        );
    }
    // ⚠️ E nenhuma OUTRA cena pede material — o mecanismo nasceu para esta e um segundo consumidor
    // silencioso mudaria uma cena que alguém já aprovou.
    for n in 1..=crate::smoke::scenes::CENAS {
        assert_eq!(
            crate::smoke::scenes::materiais_da_cena(n).is_some(),
            n == 34,
            "a cena {n} mudou de material sem ninguém dizer"
        );
    }
}

/// ⭐⭐⭐ **O MATERIAL CHEGA ÀS ESFERAS — pelo caminho do produto, não pela tabela.**
///
/// ⛔⛔ As duas metades acima provam que a cena **declara** o material certo e que esse material
/// **contém** o fenómeno. As duas são cegas ao elo do meio: *alguém tem de PÔR o material nas
/// entidades*. Um `Vec` declarado e deitado fora lê-se exactamente como um `Vec` aplicado — e é o
/// mesmo buraco que o `CLAUDE.md` §5.0 nomeia sobre si mesmo (*«nenhum instrumento pergunta se o
/// VALOR chega a um consumidor»*).
///
/// ⚠️ Ele entra pela [`crate::scene::sync_scene_and_birth`], que é a porta que o smoke percorre, e
/// não por uma montagem à mão do mundo.
#[test]
fn o_material_da_cena_chega_as_esferas() {
    let doc = crate::smoke::scenes::edge::cena_34().expect("a cena `=34`");
    let pedidos = crate::smoke::scenes::materiais_da_cena(34).expect("os materiais dela");
    crate::smoke::set_armed_by_panel(true);
    crate::smoke::with_smoke(|s| s.seed_materials = Some(pedidos.clone()));
    let mut sim = ph2d_ecs::SimWorld::new();
    crate::scene::sync_scene_and_birth(&mut sim, Some(&doc), &[], 0.0, &crate::scene::no_drawing());
    let world = sim.world_mut();
    let mut q = world.query::<&ph2d_field_ecs::FieldMaterial>();
    let mut raios: Vec<f32> = q.iter(world).map(|m| m.subsurface_radius).collect();
    let cores: Vec<[f32; 3]> = {
        let mut q2 = world.query::<&ph2d_field_ecs::FieldMaterial>();
        q2.iter(world).map(|m| m.subsurface_color).collect()
    };
    crate::smoke::set_armed_by_panel(false);

    assert_eq!(
        raios.len(),
        4,
        "só {} das 4 esferas receberam material — o elo do meio está partido",
        raios.len()
    );
    raios.sort_by(f32::total_cmp);
    let mut alvo = crate::smoke::scenes::edge::PROFUNDIDADES_DA_COR;
    alvo.sort_by(f32::total_cmp);
    for (a, b) in raios.iter().zip(alvo) {
        assert!(
            (a - b).abs() < 1e-6,
            "as profundidades que chegaram ({raios:?}) não são as que a cena pediu ({alvo:?})"
        );
    }
    // ⚠️ E a COR também — um material com o raio certo e a cor de omissão voltaria a ser cinzento,
    // que é exactamente o defeito que esta cena existe para não ter.
    for c in &cores {
        assert_eq!(*c, COR, "uma esfera ficou com a cor de omissão");
    }
}
