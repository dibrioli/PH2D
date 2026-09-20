//! ⭐⭐⭐ **A LEI QUE MISTURA OS OSSOS — o vinco, o tecto das três leis, e onde cada uma PARTE.**
//!
//! A auditoria de 2026-09-20 (ordem do dono) mediu o caminho vectorial contra um padrão-ouro e o
//! veredito foi contra a suspeita: **o vector copia o ouro a `≤ 0,46 %` da espessura até `90°`,
//! com os nós exactos ao bit — e o OURO tem o mesmo vinco.** ⇒ o defeito dominante não é a curva
//! nem o peso: é a lei que mistura as rotações, e é ela que estas sondas medem.
//!
//! ⚠️ **Saiu do irmão [`super::ouro_tests`] por tecto de LOC** (`1 358` contra `700`), e o corte é
//! por RESPONSABILIDADE: lá pergunta-se *«quão fiel é o vector ao ouro?»* e aqui *«qual LEI usar?»*.

use super::ouro_reguas_tests::*;

/// O ÂNGULO MÉDIO `θ̄` que a [`ph2d_skeleton::Skin::blend`] aplica num ponto, e a COERÊNCIA da
/// média em círculo.
fn b_theta(
    pele: &ph2d_skeleton::Skin,
    campo: &ph2d_vec_skin::pesos::CampoDoDominio,
    correcoes: &[ph2d_skeleton::Correccao],
    p: [f64; 2],
) -> (f64, f64) {
    let mut w = pele.scratch();
    let linha = campo.linha(p).unwrap_or_else(|| b_mais_proximo(campo, p));
    pele.weights_corrected(p, Some(&linha), &mut w, correcoes);
    let (mut sx, mut sy) = (0.0_f64, 0.0_f64);
    for (bn, &pw) in pele.bones().iter().zip(w.iter()) {
        let t = bn.angulo_da_pose();
        sx = pw.mul_add(t.cos(), sx);
        sy = pw.mul_add(t.sin(), sy);
    }
    (sy.atan2(sx), sx.hypot(sy))
}

/// ⭐⭐⭐ **SONDA B4 — A LEI DO VINCO.** A previsão geométrica, medida.
///
/// # A previsão
///
/// A [`ph2d_skeleton::Skin::blend`] roda **em torno da JUNTA** (`c`) e translada pela mistura
/// linear DELA. Num par pai→filho os dois ossos partilham a junta, logo `M₁(c) = M₂(c)` e a
/// translação é a MESMA para todo peso: perto da junta a lei é *«rodar em torno de `J` por um
/// ângulo `θ̄(p)` que varia com o peso»*.
///
/// Derivando ao longo do contorno (parâmetro de arco `s`), a tangente da imagem é
/// `R(θ̄)·[t̂(s) + θ̄′(s)·perp(p − c)]`. Na aresta INTERIOR do cotovelo `perp(p − c)` aponta **contra**
/// a marcha e tem módulo `r` (a meia-espessura) ⇒ **o esticão é `|1 − θ̄′·r|`, e ele chega a ZERO
/// quando `θ̄′ = 1/r`.** Num ponto de esticão zero a curva tem uma CÚSPIDE; acima dele, ela dobra
/// sobre si mesma e o contorno passa a CRUZAR-SE.
#[test]
fn diag_b_a_lei_do_vinco() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let n = rest.len();
    // O arco de repouso acumulado — o `s` da derivada.
    let cum = b_cum(&rest);
    const R: f64 = 0.5; // meia-espessura da barra

    println!("\n{:=<126}", "");
    println!("SONDA B4 · A LEI DO VINCO — a cúspide prevista, medida");
    println!("{:=<126}", "");
    println!(
        "previsão: esticão = |1 − θ̄′·r| com r = {R} ⇒ CÚSPIDE quando θ̄′ = {:.3} rad/u",
        1.0 / R
    );
    println!(
        "{:>6} | {:>9} {:>9} | {:>9} {:>9} | {:>10} {:>10} | {:>8} {:>6} | {:>5}",
        "graus",
        "estic p50",
        "estic MIN",
        "θ̄′ máx",
        "θ̄′·r máx",
        "prev. estic",
        "medido",
        "κ máx",
        "quina°",
        "X"
    );
    for graus in [
        20.0_f32, 45.0, 60.0, 70.0, 80.0, 85.0, 90.0, 95.0, 100.0, 110.0, 120.0, 150.0,
    ] {
        p.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let thetas: Vec<f64> = rest
            .iter()
            .map(|&x| b_theta(&pele, &p.campo, &p.correcoes, x).0)
            .collect();

        let mut estica = Vec::new();
        let mut dtheta = Vec::new();
        let mut min_e = (f64::MAX, 0usize);
        let mut max_d = (0.0_f64, 0usize);
        for i in 0..n {
            let j = (i + 1) % n;
            let dr = cum[i + 1] - cum[i];
            if dr <= 1e-9 {
                continue;
            }
            let dd = (ouro[i][0] - ouro[j][0]).hypot(ouro[i][1] - ouro[j][1]);
            let e = dd / dr;
            estica.push(e);
            if e < min_e.0 {
                min_e = (e, i);
            }
            // A derivada do ângulo médio ao longo do arco de REPOUSO — com a volta em ±π tratada.
            let mut dt = thetas[j] - thetas[i];
            while dt > std::f64::consts::PI {
                dt -= std::f64::consts::TAU;
            }
            while dt < -std::f64::consts::PI {
                dt += std::f64::consts::TAU;
            }
            let d = (dt / dr).abs();
            dtheta.push(d);
            if d > max_d.0 {
                max_d = (d, i);
            }
        }
        let (e50, ..) = b_pct(&mut estica.clone());
        let kt = b_menger(&ouro, B_H);
        let rectas = b_rectas(&rest);
        let (_, _, kmax) = b_pct(&mut rectas.iter().map(|&i| kt[i]).collect::<Vec<_>>());
        println!(
            "{graus:>6.0} | {e50:>9.4} {:>9.4} | {:>9.4} {:>9.4} | {:>10.4} {:>10.4} | {kmax:>8.3} {:>6.1} | {:>5}",
            min_e.0,
            max_d.0,
            max_d.0 * R,
            (1.0 - max_d.0 * R).abs(),
            min_e.0,
            b_quina(kmax, B_H),
            b_auto(&ouro, 1e-7)
        );
    }
    println!(
        "\n  (a coluna `prev. esticão` é |1 − θ̄′·r| com o θ̄′ MÁXIMO; a coluna `medido` é o esticão \
         MÍNIMO do contorno. Elas concordam ⇒ a lei está identificada.)"
    );

    // ── ONDE, e o retrato local ────────────────────────────────────────────────────────
    println!(
        "\n── O RETRATO LOCAL a 90° — a aresta INTERIOR do cotovelo 2 (junta em x = -3.93) {:─<45}",
        ""
    );
    p.dobra(90.0);
    let pele = p.pele();
    let ouro = p.ouro(&pele, &rest);
    let mut alvo: Vec<(f64, usize)> = (0..n)
        .filter(|&i| (rest[i][1] - 3.0).abs() < 1e-9 && (rest[i][0] + 3.93).abs() < 0.7)
        .map(|i| (rest[i][0], i))
        .collect();
    alvo.sort_by(|a, b| a.0.total_cmp(&b.0));
    println!(
        "{:>9} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "rest x", "def x", "def y", "θ̄ (graus)", "coer", "passo"
    );
    let mut ant: Option<[f64; 2]> = None;
    for (x, i) in alvo.iter().step_by(2) {
        let (t, c) = b_theta(&pele, &p.campo, &p.correcoes, rest[*i]);
        let passo = ant.map_or(0.0, |a| (a[0] - ouro[*i][0]).hypot(a[1] - ouro[*i][1]));
        println!(
            "{x:>9.3} {:>10.4} {:>10.4} {:>10.2} {c:>10.4} {passo:>10.5}",
            ouro[*i][0],
            ouro[*i][1],
            t.to_degrees()
        );
        ant = Some(ouro[*i]);
    }
    // Qual aresta é a de DENTRO?
    for (rot, nome) in [(3.0_f64, "y = 3 (topo)"), (2.0, "y = 2 (base)")] {
        let idx: Vec<usize> = (0..n)
            .filter(|&i| (rest[i][1] - rot).abs() < 1e-9)
            .collect();
        let mut e: Vec<f64> = idx
            .iter()
            .filter_map(|&i| {
                let j = (i + 1) % n;
                let dr = cum[i + 1] - cum[i];
                (dr > 1e-9).then(|| (ouro[i][0] - ouro[j][0]).hypot(ouro[i][1] - ouro[j][1]) / dr)
            })
            .collect();
        let mn = e.iter().copied().fold(f64::MAX, f64::min);
        let (e50, ..) = b_pct(&mut e);
        println!(
            "  {nome}: esticão p50 = {e50:.4} · MIN = {mn:.4}  ⇒  {}",
            if mn < 0.5 {
                "é a de DENTRO (comprime)"
            } else {
                "é a de FORA (estica)"
            }
        );
    }
    println!(
        "\nloadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<126}", "");
}

/// Os ângulos das poses DESDOBRADOS ao longo da cadeia — cada um escolhido na volta mais próxima
/// do anterior.
///
/// ⚠️ **`angulo_da_pose` é um `atan2` e vive em `(−π, π]`**: numa cadeia que dobra `90°` por junta,
/// o terceiro osso lê `±180°` e o sinal é um sorteio de último bit.
fn b_desdobra(pele: &ph2d_skeleton::Skin) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::with_capacity(pele.len());
    let mut ant = 0.0_f64;
    for b in pele.bones() {
        let mut t = b.angulo_da_pose();
        while t - ant > std::f64::consts::PI {
            t -= std::f64::consts::TAU;
        }
        while t - ant < -std::f64::consts::PI {
            t += std::f64::consts::TAU;
        }
        out.push(t);
        ant = t;
    }
    out
}

/// ⭐⭐⭐ **A lei DESDOBRADA foi para o PRODUTO** ([`ph2d_skeleton::MisturaDoAngulo::Desdobrado`]).
///
/// ⛔⛔ Esta sonda tinha aqui uma cópia dela escrita à mão (`b_blend_ang_linear`), e essa cópia era
/// uma medição de **outro programa** no instante em que a lei shipou — a família de defeitos que
/// esta casa já pagou meia dúzia de vezes. Hoje o `modo` é um parâmetro do
/// [`ph2d_skeleton::Skin::blend_com`], que é a porta que o quadro atravessa.
/// ⭐⭐⭐ **SONDA B5 — O TECTO: as TRÊS leis de mistura, medidas lado a lado.**
///
/// `LINEAR` = a mistura de posições (`blend_linear`, o *candy-wrapper* que a [`ph2d_skeleton`]
/// guarda como CONTROLO) · `CÍRCULO` = o que o produto ship hoje ([`ph2d_skeleton::Skin::blend`],
/// média em círculo) · `ÂNGULO` = a mesma lei com o ângulo médio LINEAR sobre ângulos desdobrados.
#[test]
fn diag_b_o_tecto_das_tres_leis() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let n = rest.len();
    let cum = b_cum(&rest);
    let rectas = b_rectas(&rest);
    let area_rest = b_area(&rest);
    const R: f64 = 0.5;

    println!("\n{:=<128}", "");
    println!("SONDA B5 · O TECTO — as três leis de mistura na MESMA barra, com os MESMOS pesos");
    println!("{:=<128}", "");
    println!(
        "{:>6} {:<9} | {:>9} {:>9} | {:>9} | {:>8} {:>7} | {:>8} {:>8} | {:>4}",
        "graus",
        "lei",
        "estic p50",
        "estic MIN",
        "θ̄′·r máx",
        "κ máx",
        "quina°",
        "área %",
        "larg min",
        "X"
    );
    for graus in [45.0_f32, 70.0, 90.0, 110.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let desd = b_desdobra(&pele);
        if (graus - 90.0).abs() < 0.5 {
            println!(
                "         (a 90° os ângulos CRUS das poses são {:?} e os DESDOBRADOS {:?})",
                pele.bones()
                    .iter()
                    .map(|b| (b.angulo_da_pose().to_degrees() * 10.0).round() / 10.0)
                    .collect::<Vec<_>>(),
                desd.iter()
                    .map(|t| (t.to_degrees() * 10.0).round() / 10.0)
                    .collect::<Vec<_>>()
            );
        }
        for (nome, modo) in [("LINEAR", 0u8), ("CÍRCULO", 1), ("ÂNGULO", 2)] {
            use ph2d_skeleton::MisturaDoAngulo;
            let lei = if modo == 2 {
                MisturaDoAngulo::Desdobrado
            } else {
                MisturaDoAngulo::Circulo
            };
            let mut thetas = vec![0.0_f64; n];
            let saida: Vec<[f64; 2]> = rest
                .iter()
                .enumerate()
                .map(|(i, &x)| {
                    let mut w = pele.scratch();
                    let linha = p
                        .campo
                        .linha(x)
                        .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                    pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                    let (mut sx, mut sy, mut soma) = (0.0_f64, 0.0_f64, 0.0_f64);
                    for (k, &pw) in w.iter().enumerate() {
                        sx = pw.mul_add(desd[k].cos(), sx);
                        sy = pw.mul_add(desd[k].sin(), sy);
                        soma = pw.mul_add(desd[k], soma);
                    }
                    thetas[i] = if modo == 2 { soma } else { sy.atan2(sx) };
                    if modo == 0 {
                        pele.blend_linear(x, &w)
                    } else {
                        // ⭐ A PORTA DO PRODUTO, com a lei como PARÂMETRO.
                        pele.blend_com(x, &w, lei)
                    }
                })
                .collect();
            let mut estica = Vec::new();
            let mut maxd = 0.0_f64;
            for i in 0..n {
                let j = (i + 1) % n;
                let dr = cum[i + 1] - cum[i];
                if dr <= 1e-9 {
                    continue;
                }
                estica.push((saida[i][0] - saida[j][0]).hypot(saida[i][1] - saida[j][1]) / dr);
                let mut dt = thetas[j] - thetas[i];
                while dt > std::f64::consts::PI {
                    dt -= std::f64::consts::TAU;
                }
                while dt < -std::f64::consts::PI {
                    dt += std::f64::consts::TAU;
                }
                maxd = maxd.max((dt / dr).abs());
            }
            let emin = estica.iter().copied().fold(f64::MAX, f64::min);
            let (e50, ..) = b_pct(&mut estica);
            let kt = b_menger(&saida, B_H);
            let (_, _, kmax) = b_pct(&mut rectas.iter().map(|&i| kt[i]).collect::<Vec<_>>());
            let mut l = b_larguras(&rest, &saida);
            let lmin = l.iter().copied().fold(f64::MAX, f64::min);
            l.clear();
            println!(
                "{:>6} {nome:<9} | {e50:>9.4} {emin:>9.4} | {:>9.4} | {kmax:>8.3} {:>7.1} | {:>8.2} {lmin:>8.4} | {:>4}",
                if modo == 0 {
                    format!("{graus:.0}")
                } else {
                    String::new()
                },
                maxd * R,
                b_quina(kmax, B_H),
                b_area(&saida) / area_rest * 100.0,
                b_auto(&saida, 1e-7)
            );
        }
        println!("{:-<128}", "");
    }
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<128}", "");
}

/// ⭐⭐⭐ **SONDA B8 — ONDE CADA LEI PARTE**, varrida a `1°` na barra da cena do dono.
///
/// A [`diag_b_o_tecto_das_tres_leis`] mede seis dobras e mostra que a troca não é um ganho limpo
/// em toda a faixa. Esta responde à pergunta que decide: **até que ângulo cada lei aguenta?**
///
/// Três marcos, cada um com a régua ao lado:
///
/// - **BICO** — a 1.ª dobra em que `θ̄′·r ≥ 1`. Ali a aresta de dentro tem comprimento ZERO e a
///   forma cria um bico; é a forma fechada do cabeçalho do [`ph2d_skeleton::centro`].
/// - **DOBRA** — a 1.ª dobra em que o contorno se **cruza a si mesmo**. Depois dela a arte está
///   virada do avesso nalgum sítio, e nenhuma régua de suavidade o descreve.
/// - **`< 25 %`** — a 1.ª dobra em que a aresta de dentro cai abaixo de um quarto do comprimento
///   de repouso, que é onde o esmagamento passa a ver-se.
///
/// ⚠️⚠️ **Ler a coluna da QUINA sozinha inverte o veredito entre os dois bicos:** passado o bico
/// dela, a lei de CÍRCULO re-abre e volta a ler quinas pequenas — *com o contorno já cruzado*. Uma
/// régua de suavidade sobre uma forma dobrada sobre si mesma mede a suavidade do ERRO.
/// Os marcos de uma lei na barra da cena do dono: `(bico, dobra, aresta < 25 %, (aresta, quina) a 90°)`.
///
/// ⚠️ **Uma PORTA e não o corpo da sonda:** o gate que julga estes números e a sonda que os imprime
/// têm de medir a MESMA coisa, e duas cópias divergem no dia em que uma delas for afinada.
fn b_marcos(
    p: &mut BPalco,
    lei: ph2d_skeleton::MisturaDoAngulo,
) -> (Option<i32>, Option<i32>, Option<i32>, (f64, f64)) {
    const R: f64 = 0.5;
    let rest = b_amostra(&p.fonte);
    let n = rest.len();
    let cum = b_cum(&rest);
    let rectas = b_rectas(&rest);
    let (mut bico, mut dobra, mut esmaga) = (None, None, None);
    let mut a90 = (0.0_f64, 0.0_f64);
    for g in 60..=170 {
        p.dobra(g as f32);
        let pele = p.pele();
        let mut thetas = vec![0.0_f64; n];
        let saida: Vec<[f64; 2]> = rest
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let mut w = pele.scratch();
                let linha = p
                    .campo
                    .linha(x)
                    .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                thetas[i] = b_theta_da_lei(&pele, &w, lei);
                pele.blend_com(x, &w, lei)
            })
            .collect();
        let (mut emin, mut maxd) = (f64::MAX, 0.0_f64);
        for i in 0..n {
            let j = (i + 1) % n;
            let dr = cum[i + 1] - cum[i];
            if dr <= 1e-9 {
                continue;
            }
            emin = emin.min((saida[i][0] - saida[j][0]).hypot(saida[i][1] - saida[j][1]) / dr);
            let mut dt = thetas[j] - thetas[i];
            while dt > std::f64::consts::PI {
                dt -= std::f64::consts::TAU;
            }
            while dt < -std::f64::consts::PI {
                dt += std::f64::consts::TAU;
            }
            maxd = maxd.max((dt / dr).abs());
        }
        if bico.is_none() && maxd * R >= 1.0 {
            bico = Some(g);
        }
        if dobra.is_none() && b_auto(&saida, 1e-7) > 0 {
            dobra = Some(g);
        }
        if esmaga.is_none() && emin < 0.25 {
            esmaga = Some(g);
        }
        if g == 90 {
            let kt = b_menger(&saida, B_H);
            let (_, _, kmax) = b_pct(&mut rectas.iter().map(|&i| kt[i]).collect::<Vec<_>>());
            a90 = (emin, b_quina(kmax, B_H));
        }
    }
    (bico, dobra, esmaga, a90)
}

#[test]
fn diag_b_onde_cada_lei_parte() {
    use ph2d_skeleton::MisturaDoAngulo;
    let mut p = b_palco(true);
    println!("\n{:=<96}", "");
    println!("SONDA B8 · ONDE CADA LEI PARTE — barra da cena do dono, varrida a 1°");
    println!("{:=<96}", "");
    println!(
        "{:<12} | {:>10} | {:>10} | {:>12} | {:>18}",
        "lei", "BICO", "DOBRA", "aresta < 25 %", "a 90°: aresta/quina"
    );
    for (nome, lei) in [
        ("CÍRCULO", MisturaDoAngulo::Circulo),
        ("DESDOBRADO", MisturaDoAngulo::Desdobrado),
    ] {
        let (bico, dobra, esmaga, a90) = b_marcos(&mut p, lei);
        let d = |x: Option<i32>| x.map_or("—".into(), |g| format!("{g}°"));
        println!(
            "{nome:<12} | {:>10} | {:>10} | {:>12} | {:>8.4} {:>8.1}°",
            d(bico),
            d(dobra),
            d(esmaga),
            a90.0,
            a90.1
        );
    }
    println!("{:=<96}", "");
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// ⭐⭐⭐ **GATE — A LEI DESDOBRADA EMPURRA O BICO E A DOBRA, na barra da cena do dono.**
///
/// A medição de 2026-09-20, pela porta do produto ([`ph2d_skeleton::Skin::blend_com`]):
///
/// | lei | BICO | DOBRA | aresta `< 25 %` | a `90°`: aresta / quina |
/// |---|---:|---:|---:|---:|
/// | `Circulo` (o que ship) | `93°` | `93°` | `77°` | `0,0496` / `180,0°` |
/// | **`Desdobrado`** | **`121°`** | **`108°`** | **`91°`** | **`0,2524`** / **`67,5°`** |
///
/// ⚠️⚠️ **A coluna que decide é a DOBRA e não a quina.** Passado o bico dela, a lei de círculo
/// **re-abre** e volta a ler quinas pequenas (`57,6°` a `110°`) — *com o contorno já cruzado*. Uma
/// régua de suavidade sobre uma forma dobrada sobre si mesma mede a suavidade do ERRO, e ler só
/// aquela coluna inverte o veredito entre os dois bicos.
///
/// ⛔ **As barras são bandas à volta do medido e não o número exacto:** a fixtura é a cena do dono
/// e ela pode ganhar um nó; o que não pode mudar sem alguém reparar é a ORDEM dos marcos.
///
/// ⛔⛔⛔ **E ESTE GATE NÃO DIZ O TAMANHO DO QUE SE VÊ.** O `5,1×` da aresta é um MÍNIMO sobre o
/// contorno — um extremo LOCAL —, e o DESENHO move-se `4,3 %` da espessura a `90°`, com o `p50` a
/// **`0,0000`**. Quem cita este gate cita também a [`diag_b_quanto_a_lei_nova_move_o_desenho`], que
/// é a régua do tamanho; eu reportei o `5,1×` ao dono sem ela, e foi a imagem que me corrigiu.
#[test]
fn a_lei_desdobrada_empurra_o_bico_e_a_dobra() {
    use ph2d_skeleton::MisturaDoAngulo;
    let mut p = b_palco(true);
    let (bc, dc, ec, a90c) = b_marcos(&mut p, MisturaDoAngulo::Circulo);
    let (bd, dd, ed, a90d) = b_marcos(&mut p, MisturaDoAngulo::Desdobrado);
    let (bc, dc, ec) = (
        bc.expect("o círculo bica"),
        dc.expect("dobra"),
        ec.expect("esmaga"),
    );
    let (bd, dd, ed) = (
        bd.expect("a desdobrada bica"),
        dd.expect("dobra"),
        ed.expect("esmaga"),
    );
    println!("  bico {bc}° → {bd}° · dobra {dc}° → {dd}° · esmaga {ec}° → {ed}°");
    println!(
        "  a 90°: aresta {:.4} → {:.4} · quina {:.1}° → {:.1}°",
        a90c.0, a90d.0, a90c.1, a90d.1
    );

    assert!(
        (90..=96).contains(&bc),
        "o bico da lei de CÍRCULO leu {bc}° e a medição dá 93°"
    );
    assert!(
        (116..=126).contains(&bd),
        "o bico da lei DESDOBRADA leu {bd}° e a medição dá 121°"
    );
    assert!(
        dd >= dc + 10,
        "a lei desdobrada devia adiar a DOBRA em pelo menos 10° e foi de {dc}° para {dd}°"
    );
    assert!(
        ed >= ec + 10,
        "a lei desdobrada devia adiar o esmagamento visível e foi de {ec}° para {ed}°"
    );
    // ⚠️ E o que o dono VÊ a 90°: a aresta de dentro e a quina numa aresta que em repouso é RECTA.
    assert!(
        a90d.0 >= a90c.0 * 4.0,
        "a 90° a aresta de dentro devia MULTIPLICAR-SE e foi de {:.4} para {:.4}",
        a90c.0,
        a90d.0
    );
    assert!(
        a90c.1 > 170.0 && a90d.1 < 90.0,
        "a 90° a quina devia ir de um BICO (~180°) para menos de 90°, e leu {:.1}° → {:.1}°",
        a90c.1,
        a90d.1
    );
}

/// O `θ̄` que uma LEI resolve para estes pesos — o mesmo que o [`ph2d_skeleton::Skin::blend_com`]
/// usa, medido pelo par que ele devolve em vez de reescrito.
fn b_theta_da_lei(
    pele: &ph2d_skeleton::Skin,
    w: &[f64],
    lei: ph2d_skeleton::MisturaDoAngulo,
) -> f64 {
    // ⚠️ Dois pontos a distância 1 da origem: a diferença deles É o `(cos θ̄, sin θ̄)`, e sai da
    // PORTA — reescrever a média aqui seria a segunda resposta que esta sonda acabou de apagar.
    let o = pele.blend_com([0.0, 0.0], w, lei);
    let u = pele.blend_com([1.0, 0.0], w, lei);
    (u[1] - o[1]).atan2(u[0] - o[0])
}
