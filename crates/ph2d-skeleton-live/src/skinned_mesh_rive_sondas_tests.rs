//! ⭐⭐ **AS SONDAS da jornada do Rive e do segundo corpo** — elas IMPRIMEM; as leis são o irmão
//! [`super::rive_tests`].
//!
//! ⚠️ **Saíram por tecto de LOC (`723` contra `700`) e o corte é por RESPONSABILIDADE** — o mesmo
//! que as ondulações já pagaram nesta crate: *uma medição exploratória e uma lei são coisas
//! diferentes, e juntá-las faz o ficheiro crescer a cada report sem lei nova.*

use super::ondulacao_regua_tests::{DENSO, ideal_denso};
use super::ouro_reguas_tests::*;
use super::serpentina_tests::serpentina_para_teste;

/// A mediana de `v`, ordenando-o em lugar.
fn p50(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v.get(v.len() / 2).copied().unwrap_or(0.0)
}

/// SONDA — as duas leis lado a lado, na dobra que o dono fotografou.
#[test]
fn diag_d_a_lei_do_rive_contra_a_de_hoje() {
    println!("\n{:=<92}", "");
    println!("SONDA · A LEI DO RIVE (só pontos de controlo) CONTRA A DE HOJE (ajuste das alças)");
    println!("{:=<92}", "");
    println!(
        "{:>6} | {:<26} | {:>10} | {:>9} {:>9} | {:>7} {:>7} | {:>7} {:>7}",
        "dobra", "lei", "serpent.", "ouro p90", "máx", "κ+ p90", "máx", "quebra", "máx"
    );
    for graus in [30.0_f32, 60.0, 90.0, 120.0] {
        let mut p = b_palco(true);
        p.reparte_com(1, false);
        p.lei_do_peso(false);
        p.dobra_em_s(graus);
        let pele = p.pele();
        let rest = b_amostra_com(&p.fonte, DENSO);
        let ouro = ideal_denso(&p, &pele, &rest, ph2d_vec_skin::curva::lei_c1_activa());
        let (chao, _, _) = b_chao_com(&p.fonte, &pele, &p.campo, &p.correcoes, DENSO);
        for (rot, path) in [
            ("HOJE (ajuste + conciliação)", p.produto(true, true)),
            ("RIVE (só pontos)", p.produto(false, true)),
        ] {
            let v = b_amostra_com(&path, DENSO);
            let (_, b, c) = b_perfil(&v, &ouro);
            let (kp, km) = b_excesso_de_curvatura(&rest, &v, &ouro);
            let mut q = b_quebra_nos(&path);
            let qmax = q.iter().copied().fold(0.0_f64, f64::max);
            println!(
                "{graus:>5}° | {rot:<26} | {:>10.6} | {b:>9.5} {c:>9.5} | {kp:>7.2} {km:>7.2} | {:>7.3} {qmax:>7.3}",
                serpentina_para_teste(&rest, &v),
                p50(&mut q),
            );
        }
        let (_, b, c) = b_perfil(&chao, &ouro);
        let (kp, km) = b_excesso_de_curvatura(&rest, &chao, &ouro);
        println!(
            "{graus:>5}° | {:<26} | {:>10.6} | {b:>9.5} {c:>9.5} | {kp:>7.2} {km:>7.2} | {:>7} {:>7}",
            "o CHÃO do modelo",
            serpentina_para_teste(&rest, &chao),
            "—",
            "—"
        );
    }
    println!("{:=<92}", "");
}

/// SONDA — o defeito que o AJUSTE foi construído para curar, re-medido com os nós de hoje.
///
/// Em 2026-09-19 a F30 mediu *«pintar peso entre os vértices não faz nada»* como `0,000000` pelo
/// caminho dos pontos de controlo. Isso foi medido com a contagem de nós de então.
#[test]
fn diag_d_uma_mancha_entre_dois_nos_move_a_arte() {
    use ph2d_skeleton_ecs::{CorreccaoDePeso, SkinBind};
    let base = b_palco(true);
    let cozido = base.fonte.cooked();
    let (v, _) = cozido.contour(0).expect("contorno");
    let n = v.len();
    // O espaçamento típico entre nós, e o MEIO da aresta mais comprida do miolo.
    let mut esp: Vec<f64> = (0..n - 1)
        .map(|k| (v[k + 1].anchor[0] - v[k].anchor[0]).hypot(v[k + 1].anchor[1] - v[k].anchor[1]))
        .collect();
    let (mut kmax, mut lmax) = (0usize, 0.0_f64);
    for (k, &d) in esp.iter().enumerate() {
        if d > lmax {
            (kmax, lmax) = (k, d);
        }
    }
    let meio = [
        f64::midpoint(v[kmax].anchor[0], v[kmax + 1].anchor[0]),
        f64::midpoint(v[kmax].anchor[1], v[kmax + 1].anchor[1]),
    ];
    println!("\n{:=<92}", "");
    println!(
        "SONDA · UMA MANCHA ENTRE DOIS NÓS — {n} nós, espaçamento p50 {:.4}, maior aresta {lmax:.4}",
        p50(&mut esp)
    );
    println!(
        "mancha no meio da aresta {kmax} em [{:.3}, {:.3}]",
        meio[0], meio[1]
    );
    println!("{:=<92}", "");
    println!(
        "{:>7} | {:<20} | {:>12} {:>12}",
        "raio", "lei", "máx move", "p50 move"
    );
    for raio in [0.20_f64, 0.40, 0.80, 1.60] {
        for (rot, curva) in [("RIVE (só pontos)", false), ("HOJE (ajuste)", true)] {
            let mut p = b_palco(true);
            p.reparte_com(1, false);
            p.lei_do_peso(false);
            p.dobra_em_s(90.0);
            let sem = b_amostra_com(&p.produto(curva, true), DENSO);
            let osso = {
                let b = p.sim.world().get::<SkinBind>(p.alvo).expect("pele");
                b.tendons[0].bone
            };
            {
                let mut b = p.sim.world_mut().get_mut::<SkinBind>(p.alvo).expect("pele");
                b.correcoes.push(CorreccaoDePeso {
                    bone: osso,
                    centro: meio,
                    raio,
                    especie: ph2d_skeleton::Especie::Soma(1.0),
                });
            }
            let com = b_amostra_com(&p.produto(curva, true), DENSO);
            let mut d: Vec<f64> = sem
                .iter()
                .zip(&com)
                .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                .collect();
            let mx = d.iter().copied().fold(0.0_f64, f64::max);
            println!("{raio:>7.2} | {rot:<20} | {mx:>12.6} {:>12.6}", p50(&mut d));
        }
    }
    println!("{:=<92}", "");
}

/// ⭐⭐ **SONDA — DESENHA as três leis, ampliadas, para o OLHO decidir ao lado da tabela.**
///
/// ⛔⛔ Ela existe porque a foto da cena **não consegue mostrar isto**: medido, a diferença entre
/// as duas leis na `PH2D_VEC_BONE_SMOKE=1` é de `37` píxeis numa janela de `1930×1012` — ela é
/// **sub-pixel** ao zoom de abertura, e o dono viu-a com a peça a encher o ecrã.
///
/// Escreve um SVG por lei em `$PH2D_SVG_DIR`; **sem essa porta ela não escreve nada**.
///
/// ⚠️ **E o que ela mostrou vale a nota:** a esta peça, `90°` em S, as três leis são quase
/// indistinguíveis nos troços LONGE das juntas — a serpentina mora *junto* delas, e `1,6 %` da
/// espessura só se vê com a peça grande no ecrã. *Um A/B feito ao zoom errado mostra duas imagens
/// iguais sobre uma diferença real.*
#[test]
fn diag_d_desenha_as_tres_leis() {
    // ⚠️ **Sem a porta ela não escreve nada** — um teste que deixa ficheiros no disco de toda
    // corrida da suíte é um efeito colateral, não uma sonda.
    let Ok(dir) = std::env::var("PH2D_SVG_DIR") else {
        println!("  (PH2D_SVG_DIR não está posta — nada escrito)");
        return;
    };
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, DENSO);
    let ouro = ideal_denso(&p, &pele, &rest, false);
    for (nome, v) in [
        ("ouro", ouro.clone()),
        ("hoje", b_amostra_com(&p.produto(true, true), DENSO)),
        ("rive", b_amostra_com(&p.produto(false, true), DENSO)),
    ] {
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for q in &v {
            x0 = x0.min(q[0]);
            y0 = y0.min(q[1]);
            x1 = x1.max(q[0]);
            y1 = y1.max(q[1]);
        }
        let (m, e) = (0.5_f64, 60.0_f64);
        let d: String = v
            .iter()
            .enumerate()
            .map(|(i, q)| {
                format!(
                    "{}{:.3} {:.3}",
                    if i == 0 { "M" } else { "L" },
                    (q[0] - x0 + m) * e,
                    (q[1] - y0 + m) * e
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let (w, h) = ((x1 - x0 + 2.0 * m) * e, (y1 - y0 + 2.0 * m) * e);
        let svg = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='{w:.0}' height='{h:.0}'>\
             <rect width='100%' height='100%' fill='#3a3a3a'/>\
             <path d='{d} Z' fill='#e8a33d' stroke='#1d2330' stroke-width='2'/>\
             <text x='14' y='34' font-family='sans-serif' font-size='26' fill='#fff'>{nome}</text>\
             </svg>"
        );
        let f = format!("{dir}/lei_{nome}.svg");
        std::fs::write(&f, svg).expect("svg");
        println!("  {f}");
    }
}

/// ⭐⭐⭐ **SONDA — O PREÇO DO SEGUNDO CORPO** (ordem do dono, 2026-09-20: *«vamos tentar dar à forma
/// presa dois corpos SE o custo em performance não for muito alto»*).
///
/// A condição é o preço, logo ele mede-se **antes** de a rota existir.
///
/// ⛔⛔ **A serpentina e o excesso de curvatura NÃO entram nesta tabela, e isso é uma propriedade
/// das réguas e não um esquecimento:** as duas passam pela [`b_no_passo`], que faz a
/// correspondência ser **MATERIAL** interpolando no espaço de ÍNDICE partilhado com o repouso.
/// *Um caminho com outra contagem de nós não tem esse espaço de índice.* ⇒ aqui a régua é o
/// desvio ao PADRÃO-OURO, que é geométrico (ponto a polilinha) e atravessa qualquer contagem.
#[test]
fn diag_d_o_preco_do_segundo_corpo() {
    use std::time::Instant;
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, DENSO);
    let ouro = ideal_denso(&p, &pele, &rest, false);
    let skin = p
        .sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(p.alvo)
        .expect("pele")
        .clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let pesos = skin.pesos_do_quadro(if g.valida() { &g.pesos } else { &[] });
    let correcoes = skin.correcoes_resolvidas();
    let diag = {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for v in p.fonte.verts_all() {
            for q in [v.anchor, v.in_handle, v.out_handle] {
                lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                hi = [hi[0].max(q[0]), hi[1].max(q[1])];
            }
        }
        (hi[0] - lo[0]).hypot(hi[1] - lo[1])
    };

    println!("\n{:=<98}", "");
    println!(
        "SONDA · O PREÇO DO SEGUNDO CORPO — barra a 90° em S, {} nós na fonte, diagonal {diag:.2}",
        p.fonte.verts_all().count()
    );
    println!("{:=<98}", "");
    println!(
        "{:>32} | {:>5} | {:>9} | {:>9} {:>9}",
        "lei", "nós", "µs/forma", "ouro p90", "máx"
    );
    let relogio = |f: &dyn Fn() -> ph2d_vec_scene::VecPath| -> f64 {
        // Aquece, depois mede por 250 ms — nunca uma corrida só.
        std::hint::black_box(f());
        let t = Instant::now();
        let mut n = 0u32;
        while t.elapsed().as_millis() < 250 {
            std::hint::black_box(f());
            n += 1;
        }
        t.elapsed().as_secs_f64() * 1e6 / f64::from(n.max(1))
    };
    let mostra = |rot: &str, path: &ph2d_vec_scene::VecPath, us: f64| {
        let v = b_amostra_com(path, DENSO);
        let (_, p90, max) = b_perfil(&v, &ouro);
        println!(
            "{rot:>32} | {:>5} | {us:>9.1} | {p90:>9.5} {max:>9.5}",
            path.verts_all().count()
        );
    };

    let hoje = || p.produto(true, true);
    mostra("HOJE (um corpo só)", &hoje(), relogio(&hoje));

    for (rot, c1) in [
        ("2.º corpo · campo C⁰", false),
        ("2.º corpo · campo C¹", true),
    ] {
        for frac in [0.003_f64, 0.001, 0.0003] {
            let f = || {
                ph2d_vec_skin::curva::refit_pela_curva(
                    &pele,
                    &g.path,
                    pesos,
                    &correcoes,
                    true,
                    ph2d_vec_skin::curva::LeituraDoCampo {
                        campo: g.campo.as_ref(),
                        c1,
                    },
                    frac * diag,
                )
            };
            mostra(
                &format!("{rot} · tol {:.2}%", frac * 100.0),
                &f(),
                relogio(&f),
            );
        }
    }
    // ⭐⭐⭐ **A IDEIA DO DONO: assar primeiro, ajustar depois.**
    println!("{:-<98}", "");
    for am in [8_usize, 16, 32] {
        for frac in [0.001_f64, 0.0003] {
            let f = || {
                ph2d_vec_skin::curva::refit_pelo_bake(
                    &pele,
                    &g.path,
                    pesos,
                    &correcoes,
                    true,
                    ph2d_vec_skin::curva::LeituraDoCampo {
                        campo: g.campo.as_ref(),
                        c1: false,
                    },
                    ph2d_vec_skin::curva::Bake {
                        amostras: am,
                        tolerancia: frac * diag,
                    },
                )
            };
            mostra(
                &format!("BAKE · {am} amostras/seg · tol {:.2}%", frac * 100.0),
                &f(),
                relogio(&f),
            );
        }
    }
    println!("{:-<98}", "");

    // ⭐⭐ **O CONTROLO que NOMEIA a causa:** sem o campo do domínio a lei é contínua, e o fitter
    // não acrescenta **um único nó**. Tudo o que ele acrescenta com o campo ligado é a perseguir
    // o bico de tangente que o elemento finito LINEAR deixa em cada aresta da malha.
    let sem = || {
        ph2d_vec_skin::curva::refit_pela_curva(
            &pele,
            &g.path,
            pesos,
            &correcoes,
            true,
            ph2d_vec_skin::curva::LeituraDoCampo {
                campo: None,
                c1: false,
            },
            0.001 * diag,
        )
    };
    mostra("CONTROLO · sem campo nenhum", &sem(), relogio(&sem));

    let (chao, _, _) = b_chao_com(&p.fonte, &pele, &p.campo, &p.correcoes, DENSO);
    let (_, p90, max) = b_perfil(&chao, &ouro);
    println!(
        "{:>32} | {:>5} | {:>9} | {p90:>9.5} {max:>9.5}",
        "o CHÃO (com estes nós)",
        p.fonte.verts_all().count(),
        "—"
    );

    // ⭐⭐⭐ **A COMPARAÇÃO QUE DECIDE: e se a fonte NÃO tivesse os nós que o bind põe?**
    //
    // O segundo corpo existe para ACRESCENTAR pontos. O `Bind` já acrescenta — uma vez, ao
    // prender ([`crate::subdivisao::DIVISOES_POR_OSSO`]) — e é por isso que a fonte tem `54` e não
    // `8`. ⇒ a pergunta honesta não é *«o refit é melhor que a lei de hoje?»*, é ***«acrescentar
    // pontos por QUADRO é melhor do que acrescentá-los UMA VEZ?»***
    println!("{:-<98}", "");
    let base = b_palco(false);
    let sb = base
        .sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(base.alvo)
        .expect("pele")
        .clone();
    let gb = crate::skinned_mesh::le(&sb.source).expect("fonte");
    println!(
        "{:>32} | {:>5} | {:>9} | {:>9} {:>9}",
        format!("a fonte SEM o bind: {} nós", gb.path.verts_all().count()),
        "",
        "",
        "",
        ""
    );
    let pesos_b = sb.pesos_do_quadro(if gb.valida() { &gb.pesos } else { &[] });
    let corr_b = sb.correcoes_resolvidas();
    let f8 = || {
        ph2d_vec_skin::curva::refit_pela_curva(
            &pele,
            &gb.path,
            pesos_b,
            &corr_b,
            true,
            ph2d_vec_skin::curva::LeituraDoCampo {
                campo: gb.campo.as_ref(),
                c1: true,
            },
            0.0003 * diag,
        )
    };
    mostra("2.º corpo sobre 8 nós · C¹", &f8(), relogio(&f8));
    // ⭐⭐⭐ **A PERGUNTA QUE MUDA A ARQUITECTURA:** e se o BAKE substituísse a subdivisão do bind?
    // O artista ficava com os `8` pontos que desenhou, e o desenho com os que a tolerância pedir.
    for am in [16_usize, 32, 64, 128] {
        let fb = || {
            ph2d_vec_skin::curva::refit_pelo_bake(
                &pele,
                &gb.path,
                pesos_b,
                &corr_b,
                true,
                ph2d_vec_skin::curva::LeituraDoCampo {
                    campo: gb.campo.as_ref(),
                    c1: false,
                },
                ph2d_vec_skin::curva::Bake {
                    amostras: am,
                    tolerancia: 0.0003 * diag,
                },
            )
        };
        mostra(
            &format!("BAKE sobre 8 nós · {am} amostras"),
            &fb(),
            relogio(&fb),
        );
    }
    let ingenua8 = || {
        let mut x = gb.path.clone();
        ph2d_vec_skin::curva::aplica_pela_curva_com(
            &pele,
            &mut x,
            pesos_b,
            &corr_b,
            true,
            gb.campo.as_ref(),
        );
        x
    };
    mostra("a lei de HOJE sobre 8 nós", &ingenua8(), relogio(&ingenua8));
    println!("{:=<98}", "");
    println!(
        "  um quadro a 60 Hz = 16 667 µs · loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// ⭐⭐⭐ **SONDA — QUANTO DE UM PONTO ERA A BUSCA.** A que nomeou o verdadeiro tecto do módulo.
///
/// ⚠️ **A contagem de amostras foi medida com contadores TEMPORÁRIOS** (um `thread_local` em
/// `ponto()` e outro na consulta) e eles saíram depois de darem o número — *um contador
/// permanente no laço do desenho, para um desenho que foi RECUSADO, não se paga*. O que eles
/// disseram, na barra a `90°` em S com `54` nós: a lei de hoje pede **`432`** amostras, o segundo
/// corpo pede **`17 947`** (campo `C⁰`) ou **`8 371`** (`C¹`), e a malha tem **`878`** triângulos.
#[test]
fn diag_d_quanto_e_a_varredura() {
    use std::time::Instant;
    let p = b_palco(true);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, 16);
    let mut w = pele.scratch();
    let n = rest.len();
    let cronometra = |rot: &str, f: &dyn Fn(usize)| {
        f(0);
        let t = Instant::now();
        let mut k = 0usize;
        while t.elapsed().as_millis() < 200 {
            f(k % n);
            k += 1;
        }
        #[expect(clippy::cast_precision_loss, reason = "contagem de iterações")]
        let us = t.elapsed().as_secs_f64() * 1e6 / k as f64;
        println!("  {rot:<38} {us:>9.4} µs/chamada");
        us
    };
    let idx = ph2d_vec_skin::pesos::IndiceDoCampo::novo(&p.campo.malha).expect("índice");
    let a = cronometra("campo.linha() — VARRE os 878 triângulos", &|i| {
        std::hint::black_box(p.campo.linha(rest[i]));
    });
    let ai = cronometra("campo.linha_com(índice)", &|i| {
        std::hint::black_box(p.campo.linha_com(rest[i], Some(&idx)));
    });
    let linha = p.campo.linha(rest[0]).expect("dentro");
    let b = cronometra("weights_corrected + blend", &|i| {
        let mut w2 = pele.scratch();
        pele.weights_corrected(rest[i], Some(&linha), &mut w2, &p.correcoes);
        std::hint::black_box(pele.blend(rest[i], &w2));
    });
    pele.weights_corrected(rest[0], Some(&linha), &mut w, &p.correcoes);
    println!(
        "  ⇒ a varredura é {:.0} % do custo de um ponto · o ÍNDICE é {:.1}× mais rápido que ela",
        100.0 * a / (a + b),
        a / ai
    );
    println!(
        "  ⇒ um ponto: {:.4} µs sem índice → {:.4} µs com ({:.1}×) · maior balde {} de {} triângulos",
        a + b,
        ai + b,
        (a + b) / (ai + b),
        idx.maior_balde(),
        p.campo.malha.tris.len()
    );
    println!(
        "  loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// ⭐⭐⭐ **SONDA — ONDE MORAM OS `3`–`5 ms` DO SEGUNDO CORPO.**
///
/// A conta não fechava: `8 371` amostras a `0,077 µs` são `645 µs`, e o refit media `3 095`.
/// *Uma atribuição que não soma é uma atribuição por fazer.*
#[test]
fn diag_d_onde_moram_os_milissegundos() {
    use std::time::Instant;
    let p = b_palco(true);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, 16);
    let n = rest.len();
    let crono = |rot: &str, f: &dyn Fn(usize)| {
        f(0);
        let t = Instant::now();
        let mut k = 0usize;
        while t.elapsed().as_millis() < 200 {
            f(k % n);
            k += 1;
        }
        #[expect(clippy::cast_precision_loss, reason = "contagem")]
        let us = t.elapsed().as_secs_f64() * 1e6 / k as f64;
        println!("  {rot:<44} {us:>10.4} µs");
        us
    };
    println!("\n  — o que se paga UMA VEZ por forma —");
    crono("IndiceDoCampo::novo", &|_| {
        std::hint::black_box(ph2d_vec_skin::pesos::IndiceDoCampo::novo(&p.campo.malha));
    });
    crono("CampoSuave::novo (deriva os gradientes)", &|_| {
        std::hint::black_box(ph2d_vec_skin::pesos_suave::CampoSuave::novo(&p.campo));
    });
    println!("  — o que se paga POR AMOSTRA —");
    let idx = ph2d_vec_skin::pesos::IndiceDoCampo::novo(&p.campo.malha).expect("índice");
    let suave = ph2d_vec_skin::pesos_suave::CampoSuave::novo(&p.campo).expect("suave");
    crono("campo.linha_com(índice)  [baricêntrico]", &|i| {
        std::hint::black_box(p.campo.linha_com(rest[i], Some(&idx)));
    });
    crono("suave.linha()            [C¹, indexado]", &|i| {
        std::hint::black_box(suave.linha(rest[i]));
    });
    let linha = p.campo.linha(rest[0]).expect("dentro");
    crono("weights_corrected + blend", &|i| {
        let mut w = pele.scratch();
        pele.weights_corrected(rest[i], Some(&linha), &mut w, &p.correcoes);
        std::hint::black_box(pele.blend(rest[i], &w));
    });
    println!(
        "  loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
