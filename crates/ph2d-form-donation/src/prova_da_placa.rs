//! ⭐⭐⭐ **AS DUAS LEIS, SOBRE A MESMA FORMA, NA PLACA** — a prova que o dono julga.
//!
//! ⚠️ **Ela é `#[ignore]` porque precisa de adapter**, e é a única metade que não se pode afirmar
//! sem ele: os dois caminhos acabam em `wgpu`. O que é aritmética já está gateado sem placa
//! (`baked_form_lei_tests.rs`).
//!
//! ```text
//! PH2D_LEI_DUMP=/tmp/leis bash scripts/ph2d-run.sh env PH2D_GPU=1 \
//!     cargo test -p ph2d-form-donation --release prova_da_placa -- --ignored --nocapture
//! ```
//!
//! Escreve dois `.ppm` (que o `magick` converte) e imprime a **razão do destaque** de cada lei em
//! cada quadrante de cor — *uma imagem que ninguém mede é uma opinião com pixels*.

use super::*;
use crate::lei_da_luz::Lei;
use ph2d_form_pbr::imagem::{Look, ViewTransform};
use ph2d_render::TextureAtlas;

/// Lado da peça sintética. ⚠️ Pequeno de propósito: a pergunta é a LEI, e uma peça grande só faz a
/// sonda demorar — o custo por tamanho tem o diagnóstico dele na `ph2d-form-pbr`.
const LADO: u32 = 256;

fn placa() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static PARTILHADA: OnceLock<Option<GpuContext>> = OnceLock::new();
    PARTILHADA
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// **A peça sintética: QUATRO bolas, uma por cor.**
///
/// ⛔⛔ **A 1.ª redacção era UMA bola cortada em quatro quadrantes, e ela não discriminava nada:**
/// as duas leis liam `7,25` contra `7,43` no vermelho. A causa é geométrica — *um destaque
/// especular vive num SÍTIO*, e numa bola só ele cai dentro de um quadrante e os outros três medem
/// difuso puro. ⇒ cada cor precisa da bola dela, para cada uma ter o seu destaque.
///
/// ⚠️ **As quatro cores são a régua**, não decoração: o defeito que esta wave curou só é visível
/// num albedo COM MATIZ — num cinzento as duas composições dão a mesma razão entre canais, e uma
/// peça cinzenta aprovaria as duas leis. A quarta bola é **cinzenta**, e é o controlo.
///
/// # ⛔⛔⛔ E a bola AZUL tem uma FRESTA, porque sem ela a paridade não media a OCLUSÃO
///
/// Esta fixtura escrevia `occ = 1` em todo o lado, e a consequência foi medida **duas vezes**: a
/// mutação que crava a leitura da textura de oclusão a `1.0` no shader **SOBREVIVEU** à paridade no
/// pixel, com `pior = 0`. Da 1.ª vez a causa era a LEI (o ambiente era zero, logo o canal era
/// multiplicado por zero); curada a lei, ela sobreviveu **outra vez** — e aí a causa era a FIXTURA.
/// *Uma paridade só afirma sobre os canais que a peça faz VARIAR.*
///
/// ⚠️ **A fresta vive só na bola AZUL, e isso é deliberado:** a bola CINZENTA é o alvo do
/// `miolo_cinzento`, que é a régua da calibração do `OLHAR_DA_FORMA` — escurecê-la moveria o número
/// que ela existe para escolher, *e uma régua que muda com a fixtura que mede deixa de ser uma
/// régua*.
fn peca() -> BakedForm {
    let n = (LADO * LADO) as usize;
    let (mut base, mut form, mut occ) = (vec![0u8; n * 4], vec![0f32; n * 4], vec![1f32; n]);
    let meio = f64::from(LADO) * 0.5;
    let r = meio * 0.42;
    for y in 0..LADO {
        for x in 0..LADO {
            let i = (y * LADO + x) as usize;
            let (qx, qy) = (u32::from(x >= LADO / 2), u32::from(y >= LADO / 2));
            // O centro da bola DESTE quadrante.
            let cx = f64::from(qx) * meio + meio * 0.5;
            let cy = f64::from(qy) * meio + meio * 0.5;
            let (dx, dy) = (f64::from(x) + 0.5 - cx, f64::from(y) + 0.5 - cy);
            let d2 = dx * dx + dy * dy;
            let cor: [u8; 3] = match (qx, qy) {
                (0, 0) => [204, 26, 26], // vermelho
                (1, 0) => [26, 204, 26], // verde
                (0, 1) => [26, 26, 230], // azul
                _ => [204, 204, 204],    // ⭐ o CONTROLO
            };
            base[i * 4..i * 4 + 3].copy_from_slice(&cor);
            base[i * 4 + 3] = 255;
            if d2 < r * r {
                // A normal de uma esfera, no espaço em que o canal da FORMA vive.
                //
                // ⛔⛔ **CANVAS, `y` para BAIXO — e esta linha dizia «espaço de vista» e escrevia
                // `-dy`.** Quem escreve este canal no produto é o `canvas_normal` do
                // `ph2d-mesh-render`, cuja última linha é `vec3(n.x, -n.y, n.z)` com o comentário
                // *«sem esta negação a mesma lâmpada acende a pintura por cima e a escultura por
                // baixo»* — ou seja o plano assado JÁ vem virado, e a fixtura voltava a virá-lo.
                //
                // ⚠️ **A paridade nunca o podia ver**: os dois motores leem os MESMOS planos, logo
                // `100,000 %` é verdade sobre uma peça acesa ao contrário. Quem o vê é o
                // [`crate::ceu_sondas::a_peca_sintetica_acende_por_cima`], e ele só passou a ser
                // preciso quando o ambiente ganhou DIRECÇÃO — até aí a luz vinha só das lâmpadas e
                // uma bola simétrica acesa de baixo mede o mesmo nível que uma acesa de cima.
                let z = (r * r - d2).sqrt();
                let q = (d2 + z * z).sqrt();
                form[i * 4] = (dx / q) as f32;
                form[i * 4 + 1] = (dy / q) as f32;
                form[i * 4 + 2] = (z / q) as f32;
                form[i * 4 + 3] = 1.0;
                // ⭐⭐⭐ **A FRESTA, e SÓ na bola AZUL** — ver o doc da [`peca`].
                if (qx, qy) == (0, 1) {
                    let t = (dy / (r * 0.35)).abs();
                    if t < 1.0 {
                        occ[i] = 0.25 + 0.75 * (t * t) as f32;
                    }
                }
            } else {
                // ⚠️ Fora da silhueta o neutro é `[0,0,1]` com cobertura ZERO — um zero em todo o
                // lado seria uma superfície virada de lado, que é outra coisa.
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

/// ⛔⛔ **A PEÇA DAS QUATRO BOLAS ACENDE POR CIMA** — o irmão do
/// [`crate::ceu_sondas::a_peca_sintetica_acende_por_cima`], sobre a fixtura DESTE ficheiro.
///
/// ⚠️ Ele existe porque a correcção de sinal foi feita em **duas** fixturas e *uma lei escrita em
/// dois sítios ainda não é uma lei*: sem este gate, alguém que reescrevesse a `peca` a partir do
/// comentário antigo (*«espaço de vista»*) devolvia o defeito sem nada acusar — a paridade da placa
/// continua a ler `100,000 %` com a peça acesa ao contrário.
///
/// ⭐ Ele é CPU pura: **não é `#[ignore]`** e corre no CI, ao contrário de tudo o resto deste módulo.
#[test]
fn a_peca_das_quatro_bolas_acende_por_cima() {
    let rig = LightRig::default();
    // ⚠️ **Por QUADRANTE**: cada bola vive no meio do seu, logo a metade de cima de uma bola
    // é o quarto de cima do quadrante dela.
    let (cima, baixo) =
        crate::ceu_sondas::metades_da_bola(&peca(), &rig, |y| y % (LADO / 2) < LADO / 4);
    assert!(
        cima > baixo * 1.2,
        "com a lampada superior-esquerda a metade de CIMA tem de ser a mais clara: \
         cima {cima:.2} contra baixo {baixo:.2}"
    );
}

/// A razão `R/B` **no DESTAQUE** de um quadrante — o `TOPO` por cento mais brilhante, só onde há
/// forma.
///
/// ⛔⛔ **A 1.ª redacção media a MÉDIA do quadrante inteiro, e não discriminava nada:** ela leu
/// `7,764` contra `7,738` para as duas leis sobre a mesma bola vermelha. A causa é que a média é
/// dominada pelo **difuso**, que é tingido pelo albedo nas DUAS leis — e correctamente. *O destaque
/// é meia dúzia de por cento dos pixels, e uma média não vê meia dúzia de por cento.*
///
/// ⚠️ Esta é a mesma forma de defeito que esta casa regista meia dúzia de vezes (o `edge_max` cego
/// ao quad fino, o `χ` cego à almofada, o `Q` que é uma média): *a régua um nível abaixo do
/// fenómeno*.
fn razao_no_destaque(px: &[u8], forma: &[f32], qx: u32, qy: u32) -> f64 {
    /// Que fracção do quadrante conta como destaque. ⚠️ Não é escolhido por gosto: com
    /// `specular_roughness = 0,3` o lóbulo é largo, e uma fracção muito menor mediria o ruído do
    /// pixel mais brilhante em vez do destaque.
    const TOPO: f64 = 0.03;

    let mut dentro: Vec<(f64, f64, f64)> = Vec::new();
    for y in (qy * LADO / 2)..((qy + 1) * LADO / 2) {
        for x in (qx * LADO / 2)..((qx + 1) * LADO / 2) {
            let i = (y * LADO + x) as usize;
            if forma[i * 4 + 3] > 0.5 {
                let (r, g, b) = (
                    f64::from(px[i * 4]),
                    f64::from(px[i * 4 + 1]),
                    f64::from(px[i * 4 + 2]),
                );
                dentro.push((r + g + b, r, b));
            }
        }
    }
    assert!(
        !dentro.is_empty(),
        "controlo: o quadrante ({qx},{qy}) tem de ter forma"
    );
    dentro.sort_by(|a, b| b.0.total_cmp(&a.0));
    let n = ((dentro.len() as f64 * TOPO) as usize).max(1);
    let (r, b) = dentro[..n]
        .iter()
        .fold((0f64, 0f64), |(ar, ab), s| (ar + s.1, ab + s.2));
    (r / n as f64) / (b / n as f64).max(1e-9)
}

/// ⚠️⚠️ **A COLUNA QUE FALTAVA À TABELA DO `R/B`: quantos por cento do destaque estão a SATURAR.**
///
/// Sem ela a tabela é interpretável ao contrário. Um destaque que bate no `255` num canal e não no
/// outro empurra a razão **para longe de `1`**, e isso lê-se exactamente como *«esta lei tinge mais
/// o destaque»* — que é o oposto do que está a acontecer. *Uma razão entre dois números cortados
/// não mede a lei que os produziu.*
fn saturados_no_destaque(px: &[u8], forma: &[f32], qx: u32, qy: u32) -> f64 {
    const TOPO: f64 = 0.03;
    let mut dentro: Vec<(f64, bool)> = Vec::new();
    for y in (qy * LADO / 2)..((qy + 1) * LADO / 2) {
        for x in (qx * LADO / 2)..((qx + 1) * LADO / 2) {
            let i = (y * LADO + x) as usize;
            if forma[i * 4 + 3] > 0.5 {
                let c = &px[i * 4..i * 4 + 3];
                let soma = f64::from(c[0]) + f64::from(c[1]) + f64::from(c[2]);
                dentro.push((soma, c.contains(&255)));
            }
        }
    }
    dentro.sort_by(|a, b| b.0.total_cmp(&a.0));
    let n = ((dentro.len() as f64 * TOPO) as usize).max(1);
    100.0 * dentro[..n].iter().filter(|s| s.1).count() as f64 / n as f64
}

/// A média do canal verde no MIOLO da bola cinzenta — o nível, sem a matiz.
///
/// ⚠️ **O miolo e não a bola inteira:** a borda de uma esfera tem `N·L → 0` e a média dela mede a
/// silhueta. O corte é `0,6 R`, e o piso de população está no `assert`.
fn miolo_cinzento(px: &[u8], forma: &[f32]) -> f64 {
    let meio = f64::from(LADO) * 0.5;
    let (cx, cy) = (meio + meio * 0.5, meio + meio * 0.5);
    let r = meio * 0.42 * 0.6;
    let (mut soma, mut n) = (0f64, 0usize);
    for y in (LADO / 2)..LADO {
        for x in (LADO / 2)..LADO {
            let i = (y * LADO + x) as usize;
            let (dx, dy) = (f64::from(x) + 0.5 - cx, f64::from(y) + 0.5 - cy);
            if dx * dx + dy * dy < r * r && forma[i * 4 + 3] > 0.5 {
                soma += f64::from(px[i * 4 + 1]);
                n += 1;
            }
        }
    }
    assert!(
        n > 500,
        "controlo: o miolo cinzento tem de ter pixels ({n})"
    );
    soma / n as f64
}

/// Só a lei nova, na CPU, com o olhar dito — a escada não precisa da placa.
fn so_a_forma(bake: &BakedForm, olhar: Look) -> Vec<u8> {
    let planos = ph2d_form_pbr::imagem::Planos {
        size: bake.size,
        base: &bake.base,
        form: &bake.form,
        form_occ: &bake.form_occ,
    };
    let material = ph2d_form_pbr::OpenPbr::default().prepare();
    let lampadas: Vec<ph2d_form_pbr::Lampada> = ph2d_light::resolve(&bake.rig)
        .expect("rig aceso")
        .lamps()
        .iter()
        .map(|l| ph2d_form_pbr::Lampada {
            para_a_luz: l.dir,
            radiancia: l.tint,
        })
        .collect();
    ph2d_form_pbr::imagem::acende_imagem(
        &material,
        &planos,
        &lampadas,
        ceu_do_rig(&lampadas),
        olhar,
    )
    .expect("acende")
}

fn escreve_ppm(dir: &str, nome: &str, px: &[u8]) {
    let mut v = format!("P6\n{LADO} {LADO}\n255\n").into_bytes();
    v.extend(
        px.as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]]),
    );
    let caminho = format!("{dir}/{nome}.ppm");
    std::fs::write(&caminho, v).expect("escreve");
    println!("  escrito: {caminho}");
}

/// ⭐⭐⭐ **A MESMA forma, as duas leis.** Ver o cabeçalho.
#[test]
#[ignore = "precisa de adapter; escreve imagens quando PH2D_LEI_DUMP aponta uma pasta"]
fn as_duas_leis_sobre_a_mesma_forma() {
    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste (skip gracioso NÃO é verde)");
        return;
    };
    let bake = peca();
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);

    let mut saida = Vec::new();
    for (nome, lei) in [("tinta", Lei::Tinta), ("forma", Lei::Forma)] {
        // ⚠️ **Um slot NOVO por lei, e não um re-uso:** acender ESCREVE no slot, logo a segunda lei
        // leria os pixels que a primeira deixou. *O `base` é a fonte de toda re-acendida, e essa é a
        // lei que o próprio `BakedForm` já declara.*
        let mut b = BakedForm {
            texture_id: renderer
                .acquire_individual(LADO, LADO, &bake.base)
                .expect("slot"),
            base: bake.base.clone(),
            form: bake.form.clone(),
            form_occ: bake.form_occ.clone(),
            ..bake
        };
        b.rig = LightRig::default();
        // ⚠️ **Uma ranhura NOVA por lei**, e ela é a de sempre: cada lei constrói o pipeline
        // dela na primeira acendida, e partilhá-la entre as duas não provaria nada a mais.
        let mut passes = PassesDaLuz::default();
        acende_com(lei, &gpu, &mut renderer, &mut passes, &b.rig, &b)
            .unwrap_or_else(|e| panic!("a lei `{nome}` recusou: {e}"));
        let (_, _, px) = renderer
            .readback_individual(b.texture_id)
            .expect("le de volta");
        saida.push((nome, px));
    }

    println!(
        "\n  quadrante            R/B no destaque: TINTA   FORMA   |  a 255 (%): TINTA  FORMA"
    );
    for (rotulo, qx, qy) in [
        ("vermelho", 0u32, 0u32),
        ("verde", 1, 0),
        ("azul", 0, 1),
        ("cinzento (controlo)", 1, 1),
    ] {
        println!(
            "  {rotulo:<20} {:>8.3}          {:>8.3}   |         {:>6.1} {:>6.1}",
            razao_no_destaque(&saida[0].1, &bake.form, qx, qy),
            razao_no_destaque(&saida[1].1, &bake.form, qx, qy),
            saturados_no_destaque(&saida[0].1, &bake.form, qx, qy),
            saturados_no_destaque(&saida[1].1, &bake.form, qx, qy),
        );
    }

    // ⛔ **O CONTROLO que dá direito à tabela:** no quadrante CINZENTO as duas leis têm de ler
    // `~1`. Sem ele, uma sonda que devolvesse lixo produziria uma tabela igualmente convincente.
    for (nome, px) in &saida {
        let g = razao_no_destaque(px, &bake.form, 1, 1);
        assert!(
            (g - 1.0).abs() < 0.05,
            "controlo: no cinzento a lei `{nome}` tem de ler R/B ~1 (leu {g:.3})"
        );
    }

    // ⭐⭐⭐ **A ESCADA DA EXPOSIÇÃO** — o número que falta ao `OLHAR_DA_FORMA`.
    //
    // A lei de sempre é RELATIVA e a nova é ABSOLUTA ⇒ elas não estão na mesma escala, e a pergunta
    // não é «qual é mais bonita» mas «que exposição põe as duas no mesmo NÍVEL». ⚠️ A régua é a
    // média do MIOLO da bola cinzenta (o controlo), porque ali as duas leis concordam na matiz e o
    // que resta é só o nível.
    let alvo = miolo_cinzento(&saida[0].1, &bake.form);
    println!("\n  stops   media do miolo cinzento   contra a tinta ({alvo:.1})");
    let mut melhor = (f64::INFINITY, 0.0f32);
    // ⚠️ A escada passa do candidato de PROPÓSITO: um mínimo na BORDA de uma varredura não é um
    // mínimo — é o fim da lista.
    for stops in [2.0f32, 2.5, 2.75, 3.0, 3.1, 3.25, 3.5, 4.0] {
        let px = so_a_forma(
            &bake,
            Look {
                exposure_stops: stops,
                view: ViewTransform::Standard,
            },
        );
        let m = miolo_cinzento(&px, &bake.form);
        let erro = (m - alvo).abs();
        if erro < melhor.0 {
            melhor = (erro, stops);
        }
        println!("  {stops:>5.2}   {m:>21.1}   {:>+14.1}", m - alvo);
    }
    println!(
        "\n  ⇒ a exposicao que iguala o nivel: {:.2} stops\n",
        melhor.1
    );

    if let Ok(dir) = std::env::var("PH2D_LEI_DUMP") {
        std::fs::create_dir_all(&dir).expect("pasta");
        for (nome, px) in &saida {
            escreve_ppm(&dir, nome, px);
        }
    } else {
        println!("\n  (sem PH2D_LEI_DUMP: só a tabela)");
    }
}

/// ⭐⭐⭐⭐ **A PARIDADE DO LAÇO — a placa calcula a mesma resposta que a régua.**
///
/// Obra **3** da §7 do `docs/Render3d/15`. ⭐ **A da ÓPTICA já estava paga** pelo
/// `ph2d_field_gpu::material_parity`: o `mx_direct` que este shader chama é, pela `naga`, o do
/// `ph2d-material` — o que faltava medir eram as ~8 linhas do LAÇO (normalizar a normal, somar as
/// lâmpadas, pesar o ambiente pela oclusão, aplicar a vista, misturar pela cobertura) mais a
/// montagem que as alimenta.
///
/// ⚠️ **As duas metades entram pela PORTA DO PRODUTO**: a régua é a
/// [`super::pixels_pela_forma_na_cpu`] e a placa é o [`super::acende_com`] com
/// [`Lei::Forma`], que é o que o app corre. *Um arnês que montasse a sua própria chamada continuaria
/// a passar depois de a do produto ficar torta* — a lei que esta linha já pagou quatro vezes.
///
/// # A barra, e de onde ela sai
///
/// ⛔ **Byte-idêntico não é exigível e dizê-lo é honestidade, não folga:** um backend pode contrair
/// `a*b + c` num `fma` (que é *mais* exacto, logo diferente), o `pow` do WGSL não é o `powf` do
/// Rust, e os subnormais podem ser esvaziados — os três estão medidos e escritos no cabeçalho da
/// `ph2d-style`. ⇒ a barra é **um byte** de `255`, que é o degrau da quantização: abaixo dele as
/// duas leis escreveriam o mesmo pixel.
///
/// ⚠️ **E a FRACÇÃO não é a régua — o PIOR é.** Uma fracção afoga um defeito que ocupa 10 % da
/// imagem (a lição que a `W8` do modelador pagou); aqui afirma-se o **máximo** e imprime-se a
/// distribuição ao lado.
/// **Quantos bytes podem divergir, em percentagem.** ⚠️ **Medido, não escolhido:** nesta placa a
/// paridade é **exacta** (`0,000 %` de `262 144` bytes, `2026-09-20`), e a mutação que troca o
/// arredondamento por truncagem lê a ordem de grandeza do outro lado. A barra fica no vale entre os
/// dois, longe das duas pontas.
const POPULACAO_MAXIMA: f64 = 0.5;

#[test]
#[ignore = "precisa de adapter"]
fn a_placa_e_a_regua_concordam_no_pixel() {
    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste (skip gracioso NÃO é verde)");
        return;
    };
    let bake = peca();
    let rig = LightRig::default();

    // A RÉGUA — a lei em Rust, pela porta do produto.
    let cpu = pixels_pela_forma_na_cpu(&bake, &rig).expect("a régua acende");

    // A PLACA — o caminho que o app corre.
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let mut b = BakedForm {
        texture_id: renderer
            .acquire_individual(LADO, LADO, &bake.base)
            .expect("slot"),
        base: bake.base.clone(),
        form: bake.form.clone(),
        form_occ: bake.form_occ.clone(),
        ..bake
    };
    b.rig = rig;
    let mut passes = PassesDaLuz::default();
    acende_com(Lei::Forma, &gpu, &mut renderer, &mut passes, &b.rig, &b).expect("a placa acende");
    let (_, _, placa_px) = renderer
        .readback_individual(b.texture_id)
        .expect("le de volta");

    assert_eq!(
        cpu.len(),
        placa_px.len(),
        "as duas telas têm de medir o mesmo"
    );

    // ⚠️ **O ALFA entra na conta.** Ele atravessa intacto nos dois caminhos por LEI, e uma
    // divergência ali seria o RECORTE do objecto a mudar — pior que um desvio de luz.
    let mut hist = [0u32; 5];
    let mut pior = 0i32;
    let mut onde = 0usize;
    for (i, (a, b)) in cpu.iter().zip(placa_px.iter()).enumerate() {
        let d = i32::from(*a) - i32::from(*b);
        let d = d.abs();
        hist[(d.min(4)) as usize] += 1;
        if d > pior {
            pior = d;
            onde = i;
        }
    }
    let n = cpu.len();
    println!("\n  == a placa contra a régua, {LADO}x{LADO} ==");
    for (d, c) in hist.iter().enumerate() {
        let rotulo = if d == 4 {
            "4+".to_string()
        } else {
            d.to_string()
        };
        println!(
            "  |Δ| = {rotulo:<3} {c:>9} bytes  ({:>6.3} %)",
            100.0 * f64::from(*c) / n as f64
        );
    }
    println!(
        "  pior: {pior} byte(s), no canal {} do texel {}\n",
        onde % 4,
        onde / 4
    );
    // ⛔⛔ **DUAS barras, e a segunda existe porque a primeira deixou uma mutação SOBREVIVER.**
    //
    // Apagar o `+ 0.5` do shader (arredondar → truncar) desloca metade da tela por UM byte, e um
    // tecto de `1` aceita isso — *uma barra larga não é só uma afirmação fraca: é o sítio onde uma
    // régua errada sobrevive*. O que separa os dois casos não é a MAGNITUDE, é a POPULAÇÃO:
    //
    // | causa | pior | quantos bytes |
    // |---|---|---|
    // | contracção `fma` no backend | `1` | os que caem a ~1 ULP de uma fronteira de quantização |
    // | uma LEI diferente (a truncagem) | `1` | **todos** os que não são exactos |
    //
    // ⇒ *um desvio de um byte SISTEMÁTICO é um defeito de lei; um esporádico é representação.*
    let divergentes = n - hist[0] as usize;
    let fraccao = 100.0 * divergentes as f64 / n as f64;
    assert!(
        pior <= 1,
        "a placa e a régua divergem {pior} bytes — a barra é UM, o degrau da quantização"
    );
    assert!(
        fraccao <= POPULACAO_MAXIMA,
        "{fraccao:.3} % dos bytes divergem — acima de {POPULACAO_MAXIMA} % isto é uma LEI \
         diferente e não a representação (ver a tabela acima)"
    );
}

/// ⭐⭐⭐⭐ **O QUE A PLACA CUSTA** — a segunda coluna da tabela que a §7 do `docs/Render3d/15` abriu.
///
/// A primeira foi medida na [`ph2d_form_pbr::imagem`] e é o que obrigou este passe a existir: em
/// **paralelo**, a `1024²`, o corredor de referência custa `11,1 ms` com uma lâmpada e `34,1 ms` com
/// quatro, contra um orçamento de quadro de `16,7 ms`.
///
/// ⚠️ **O relógio inclui o `poll`**, e sem ele isto mediria a fila de submissão em vez do trabalho:
/// o `submit` devolve antes de a placa ter feito nada, e uma tabela tirada assim leria microssegundos
/// para qualquer tamanho — *a maneira mais fácil de publicar um número que é só o custo de pedir.*
///
/// ⚠️ **E ele inclui os uploads**, que é o que o produto paga: os três canais sobem a cada acendida
/// (a forma é `Rgba32Float`, ou seja `16 bytes` por texel). *Medir só o despacho responderia sobre
/// um programa que ninguém corre.*
///
/// ```text
/// bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo test -p ph2d-form-donation --release \
///     diag_quanto_a_placa_custa -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_quanto_a_placa_custa() {
    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste");
        return;
    };
    let material = crate::lei_da_luz::material_da_forma();
    let uma = vec![ph2d_form_pbr::Lampada {
        para_a_luz: [0.0, 0.3, 0.95],
        radiancia: [1.0; 3],
    }];
    let quatro = vec![uma[0]; 4];

    println!("\n  lado    lâmpadas    ms (mínimo de 9)   % de um quadro de 16,7 ms");
    for lado in [256u32, 512, 1024, 2048] {
        let n = (lado * lado) as usize;
        let base = vec![180u8; n * 4];
        let mut form = vec![0.0f32; n * 4];
        for t in form.as_chunks_mut::<4>().0 {
            *t = [0.0, 0.0, 1.0, 1.0];
        }
        let occ = vec![1.0f32; n];
        let planos = ph2d_form_pbr::imagem::Planos {
            size: (lado, lado),
            base: &base,
            form: &form,
            form_occ: &occ,
        };
        for (rotulo, lampadas) in [("1", &uma), ("4", &quatro)] {
            let mut passe = passe_da_forma::PasseDaForma::new(&gpu);
            let mut melhor = f64::MAX;
            for _ in 0..9 {
                let t0 = std::time::Instant::now();
                passe
                    .acende(
                        &gpu,
                        &material,
                        &planos,
                        lampadas,
                        ceu_do_rig(lampadas),
                        crate::lei_da_luz::OLHAR_DA_FORMA,
                    )
                    .expect("acende");
                let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                melhor = melhor.min(t0.elapsed().as_secs_f64() * 1000.0);
            }
            println!(
                "  {lado:>5}    {rotulo:>8}    {melhor:>14.2}   {:>22.1}",
                100.0 * melhor / 16.7
            );
        }
    }
    println!();
}

/// ⭐⭐⭐ **A ESCADA DO OLHAR, RE-TIRADA COM O CÉU** — o número do
/// [`crate::lei_da_luz::OLHAR_DA_FORMA`] foi calibrado com o ambiente a ZERO e envelheceu no dia em
/// que ele deixou de o ser.
///
/// ⚠️ **Ela é CPU pura, ao contrário da escada original**, que vivia dentro do teste de placa: o que
/// se mede é o NÍVEL que a lei da forma entrega, e isso é a régua — não precisa de adapter. O alvo
/// (`186,1`, a média do miolo da bola CINZENTA pela lei da TINTA) é o número que aquela corrida
/// gravou, e ⭐ ele sobrevive à correcção de sinal do `y`: numa bola simétrica a média sobre a peça
/// inteira não muda ao virar a luz ao contrário.
#[test]
#[ignore = "sonda: imprime uma tabela, nao afirma"]
fn diag_a_escada_do_olhar_com_ceu() {
    /// A média do miolo cinzento pela lei da TINTA, medida na placa em 2026-09-20.
    const ALVO: f64 = 186.1;
    let bake = peca();
    println!();
    println!("  stops   media do miolo cinzento   contra a tinta ({ALVO:.1})");
    for stops in [
        0.0f32, 1.30, 1.40, 1.45, 1.48, 1.50, 1.52, 1.55, 1.60, 1.70, 2.10,
    ] {
        let olhar = Look {
            exposure_stops: stops,
            view: ViewTransform::Standard,
        };
        let m = miolo_cinzento(&so_a_forma(&bake, olhar), &bake.form);
        println!(
            "   {stops:.2}                   {m:7.1}          {:+7.1}",
            m - ALVO
        );
    }
    println!();
}
