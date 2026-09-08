//! ⭐⭐⭐ **A MEDIÇÃO QUE ANTECEDE O FILTRO DE TECIDO** — *antes de construir um
//! item de lista aberta, MEÇA se a composição já o exprime* (`CLAUDE.md` §5.0).
//!
//! O filtro de tecido tem **cinco** tipos ([espec](../../../docs/3D/cleanroom/SPEC_cloth_brush.md)
//! §7): *Gravity · Inflate · Expand · Pinch · Scale*. Os oito modos do PINCEL já
//! shipam, e a pergunta desta bancada é **quantos dos cinco já estão escritos** —
//! respondida pelo OBSERVÁVEL (o que a malha faz depois de um passo em condições
//! de filtro), nunca por leitura de código.
//!
//! # As condições de filtro, e por que elas são exprimíveis
//!
//! A espec §7 diz que o filtro corre *sem pincel*: raio infinito, `w ≡ 1`, sem
//! banda, sem pino, restrições construídas uma vez para todos os vértices. Isso
//! escreve-se com o vocabulário que já existe — [`Area::Global`] (que devolve
//! `w = 1` por porta) mais [`Curva::Constante`] com um raio maior que a peça,
//! que faz o factor de pincel valer `1` em todo o vértice. ⇒ **a ÁREA do filtro
//! não é obra nova**; ver [`condicoes_de_filtro`].
//!
//! # ⛔ O que esta bancada NÃO decide
//!
//! Ela não tem lado APROVADO: o oráculo nunca correu o filtro (as `86` fixtures
//! de `docs/3D/cleanroom/fixtures/cloth/` são **todas** do pincel, e o §10 da
//! espec não traz um único vector do filtro). Então aqui não se calibra barra
//! nenhuma contra o alvo — mede-se só o que a NOSSA composição já produz, que é
//! a pergunta de arquitectura. *Uma barra calibrada sem o lado aprovado mede os
//! nossos próprios defeitos.*

use ph2d_cloth::V3;
use ph2d_cloth::verlet_gesto::{Accionamento, Area, Curva, Modo, Passo, Pincel, PincelTecido};

// ————————————————————————————————— as peças —————————————————————————————————

/// Uma grelha `n × n` no plano `z = 0`, com passo `h`.
fn grelha(n: usize, h: f64) -> (Vec<V3>, Vec<Vec<u32>>) {
    let meio = (n - 1) as f64 * h * 0.5;
    let mut p = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            p.push([i as f64 * h - meio, j as f64 * h - meio, 0.0]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * n + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            faces.push(vec![idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1)]);
        }
    }
    (p, faces)
}

/// Uma esfera UV de raio `1`, `nu` meridianos × `nv` paralelos.
///
/// ⚠️ **Ela é load-bearing e não decoração:** numa folha PLANA a normal da área
/// e um eixo de mundo apontam para o mesmo lado, e os dois lêem-se iguais. É
/// preciso uma peça CURVA para separar *«a direcção que a malha dita»* de *«a
/// direcção que o chamador dita»* — a pergunta inteira do tipo *Gravity*.
fn esfera(nu: usize, nv: usize) -> (Vec<V3>, Vec<Vec<u32>>) {
    let mut p = Vec::new();
    for j in 0..=nv {
        let v = j as f64 / nv as f64;
        let phi = v * std::f64::consts::PI;
        for i in 0..nu {
            let u = i as f64 / nu as f64;
            let th = u * std::f64::consts::TAU;
            p.push([phi.sin() * th.cos(), phi.sin() * th.sin(), phi.cos()]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * nu + (i % nu)).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..nv {
        for i in 0..nu {
            faces.push(vec![idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1)]);
        }
    }
    (p, faces)
}

fn aneis(n: usize, faces: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut a = vec![Vec::new(); n];
    for f in faces {
        for k in 0..f.len() {
            let (p, q) = (f[k] as usize, f[(k + 1) % f.len()] as usize);
            a[p].push(u32::try_from(q).unwrap_or(u32::MAX));
            a[q].push(u32::try_from(p).unwrap_or(u32::MAX));
        }
    }
    for l in &mut a {
        l.sort_unstable();
        l.dedup();
    }
    a
}

/// Normais por vértice, somando as normais de face dos polígonos incidentes.
fn normais(pos: &[V3], faces: &[Vec<u32>]) -> Vec<V3> {
    let mut n = vec![[0.0; 3]; pos.len()];
    for f in faces {
        let (a, b, c) = (
            pos[f[0] as usize],
            pos[f[1] as usize],
            pos[f[2] as usize],
        );
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fnv = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        for &i in f {
            for k in 0..3 {
                n[i as usize][k] += fnv[k];
            }
        }
    }
    for v in &mut n {
        let m = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if m > 1e-12 {
            for c in v.iter_mut() {
                *c /= m;
            }
        }
    }
    n
}

// ——————————————————————————— as condições de filtro ———————————————————————————

/// **O pincel em CONDIÇÕES DE FILTRO** (espec §7): malha inteira, `w ≡ 1`, sem
/// banda, sem pino, factor de pincel `1` em toda a parte.
///
/// ⚠️ O raio é grande de propósito — é ele que faz [`Curva::Constante`] devolver
/// `1` em vez de `0` (o corte `d >= r` do peso), e é ele que põe toda a malha na
/// lista de activos. *Um filtro que precise de um raio é um filtro que herda um
/// número que o artista nunca põe*, e é essa a dívida que esta bancada nomeia.
///
/// ⛔⛔ **E ela tem PREÇO MEDIDO, achado por esta bancada:** o *Push* põe o raio
/// DENTRO da magnitude (a espec §4.2 dá-lhe `2R`), então esconder «não há
/// pincel» atrás de um raio enorme faz o pico dele saltar de `~0,3` — a ordem de
/// grandeza dos outros sete — para **`598`**, `2000×`. *Um raio que finge ser
/// infinito não é neutro: ele é lido como comprimento por quem tem comprimento
/// na lei.* ⇒ o filtro precisa de porta própria, e não de um pincel disfarçado.
fn condicoes_de_filtro(modo: Modo, forca: f64) -> Pincel {
    Pincel {
        modo,
        area: Area::Global,
        curva: Curva::Constante,
        raio: 1e3,
        forca,
        dureza: 0.0,
        ..Pincel::default()
    }
}

/// **O pincel COMO FILTRO DE VERDADE** (espec §7) — pela porta
/// [`Accionamento::Filtro`], que a medição desta bancada obrigou a existir.
///
/// ⚠️ **O raio fica na omissão e não faz falta**: com este accionamento o factor
/// por vértice é `1 − máscara` (sem curva, sem corte de raio) e a área *Global*
/// põe toda a malha activa sem olhar ao alcance. *Quem não tem pincel não tem
/// raio* — que é precisamente o que a [`condicoes_de_filtro`] tinha de fingir.
fn filtro(modo: Modo, s: f64) -> Pincel {
    Pincel {
        modo,
        area: Area::Global,
        accionamento: Accionamento::Filtro { s },
        ..Pincel::default()
    }
}

/// O que UM passo em condições de filtro produziu.
struct Corrida {
    /// `x − posição de entrada`, por vértice.
    desloc: Vec<V3>,
    /// Algum `τ` saiu de zero?
    escreveu_tau: bool,
    /// Algum `σ` saiu de zero? (a assinatura dos modos de âncora)
    escreveu_ancora: bool,
    /// **O ALVO da âncora por vértice**, no fim do passo.
    ///
    /// ⚠️ **É ele que uma lei de âncora AFIRMA, e não o deslocamento** — a
    /// âncora é uma restrição com força `σ`, e o vértice acaba onde ela e as
    /// molas do tecido se equilibram. Medir a lei no deslocamento mede as duas.
    ancora: Vec<V3>,
    /// As posições de repouso (o `p⁰` a que a lei da Escala se refere).
    repouso: Vec<V3>,
}

/// Um quarto de volta em torno de `+Y`: `(x, y, z) → (z, y, −x)`.
///
/// ⚠️ **Exacta em `f64` por construção** (só troca e nega componentes), o que é
/// o que deixa a régua de equivariância pôr a barra perto de zero em vez de num
/// número escolhido.
fn roda(v: V3) -> V3 {
    [v[2], v[1], -v[0]]
}

/// Corre `passos` passos de gesto sobre a peça e devolve o observável.
///
/// `refresca` decide QUAL fotografia de normais cada passo recebe — é o
/// parâmetro que a espec §7 diz distinguir o *Inflate* do filtro do *Inflate* do
/// traço, e ele vive no CHAMADOR, não na lei.
///
/// `rodado` corre **a mesma corrida com a peça E o gesto rodados um quarto de
/// volta** — é o instrumento do gate da equivariância.
fn corre_com(
    pincel: Pincel,
    pos0: &[V3],
    faces: &[Vec<u32>],
    passos: usize,
    refresca: bool,
    rodado: bool,
) -> Corrida {
    let base: Vec<V3> = if rodado {
        pos0.iter().copied().map(roda).collect()
    } else {
        pos0.to_vec()
    };
    let gira = |v: V3| if rodado { roda(v) } else { v };
    let an = aneis(base.len(), faces);
    let n0 = normais(&base, faces);
    let inicio = gira([0.0, 0.0, 1.0]);
    let mut t = PincelTecido::pen_down(pincel, &base, inicio, Vec::new());
    let mut pos = base.clone();
    for k in 0..passos {
        let n = if refresca {
            normais(&pos, faces)
        } else {
            n0.clone()
        };
        let d = gira([0.02, 0.0, 0.0]);
        let c0 = [0.0, 0.0, 1.0];
        let p = Passo {
            cursor: gira([c0[0] + 0.02 * k as f64, c0[1], c0[2]]),
            delta: d,
            delta_3d: d,
            parado: false,
            vista: gira([0.0, 0.0, 1.0]),
            normais: &n,
            pressao: 1.0,
        };
        if t.passo(&pos, &|v| an[v as usize].clone(), &p) {
            pos.copy_from_slice(&t.sim.x);
        }
    }
    Corrida {
        desloc: pos
            .iter()
            .zip(&base)
            .map(|(a, b)| [a[0] - b[0], a[1] - b[1], a[2] - b[2]])
            .collect(),
        escreveu_tau: t.sim.tau.iter().any(|v| v.abs() > 0.0),
        escreveu_ancora: t.sim.sigma.iter().any(|v| v.abs() > 0.0),
        ancora: t.sim.ancora.clone(),
        repouso: t.sim.repouso.clone(),
    }
}

/// A corrida sem rotação — o caso de sempre.
fn corre(pincel: Pincel, pos0: &[V3], faces: &[Vec<u32>], passos: usize, refresca: bool) -> Corrida {
    corre_com(pincel, pos0, faces, passos, refresca, false)
}

fn norma(v: V3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// **Quão CONSTANTE é a direcção do deslocamento sobre a peça** — `1,0` = todos
/// os vértices que se moveram andaram para o mesmo lado; `0,0` = espalhados.
///
/// É a assinatura que separa uma força de MUNDO (a gravidade: uma direcção só,
/// ditada pelo chamador) de uma força que a MALHA dita (o *Inflate*, a normal de
/// cada vértice) ou que a POSIÇÃO dita (os apertos).
fn constancia(d: &[V3]) -> f64 {
    let mut soma = [0.0; 3];
    let mut n = 0.0;
    for v in d {
        let m = norma(*v);
        if m < 1e-9 {
            continue;
        }
        for k in 0..3 {
            soma[k] += v[k] / m;
        }
        n += 1.0;
    }
    if n == 0.0 {
        return 0.0;
    }
    norma(soma) / n
}

// —————————————————————————————— as medições ——————————————————————————————

/// ⭐⭐⭐ **O CENSO — que ARM da lei cada um dos oito modos escreve, e com que
/// assinatura de direcção.** É a tabela que decide quanto do filtro é obra nova.
#[test]
fn o_censo_dos_oito_modos_em_condicoes_de_filtro() {
    let (pos, faces) = esfera(24, 12);
    println!("\nmodo                | move | tau | anc | constancia");
    println!("--------------------|------|-----|-----|-----------");
    let mut escrevem_tau = Vec::new();
    let mut escrevem_ancora = Vec::new();
    for modo in [
        Modo::Arrastar,
        Modo::Empurrar,
        Modo::ApertarPonto,
        Modo::ApertarLinha,
        Modo::Inflar,
        Modo::Agarrar,
        Modo::Gancho,
        Modo::Expandir,
    ] {
        let c = corre(condicoes_de_filtro(modo, 1.0), &pos, &faces, 3, false);
        let movidos = c.desloc.iter().filter(|d| norma(**d) > 1e-9).count();
        println!(
            "{modo:<19?} | {movidos:>4} | {:>3} | {:>3} | {:>9.4}",
            u8::from(c.escreveu_tau),
            u8::from(c.escreveu_ancora),
            constancia(&c.desloc),
        );
        if c.escreveu_tau {
            escrevem_tau.push(modo);
        }
        if c.escreveu_ancora {
            escrevem_ancora.push(modo);
        }
    }
    // A partição tem de ser a da espec: UM modo muda o repouso, DOIS ancoram.
    assert_eq!(
        escrevem_tau,
        vec![Modo::Expandir],
        "so' o Expand muda o comprimento de repouso"
    );
    assert_eq!(
        escrevem_ancora,
        vec![Modo::Agarrar, Modo::Gancho],
        "so' os dois modos de ancora escrevem sigma"
    );
}

/// ⭐⭐⭐ **A FORÇA DO PINCEL É QUADRÁTICA E NÃO TEM SINAL — a do filtro é uma
/// RECTA COM SINAL.** É a medição que decide que o filtro precisa de porta
/// própria, e não de um [`Pincel`] disfarçado.
///
/// A espec §4.1 põe `B = 10 · força² · flip` no traço; a §7 põe
/// `S = força_base · Δpx · 0,001` no filtro — **linear**, e com o sinal a vir do
/// LADO para que a mão arrastou. Levar `S` num [`Pincel`] obrigaria a
/// `força = √|S/10|` e `flip = sinal(S)`, e o preço não é estético: perto de
/// `S = 0` a raiz tem derivada infinita, ou seja *os primeiros pixels de arrasto
/// mexeriam a peça muito mais que os últimos*.
#[test]
fn a_forca_do_traco_e_quadratica_logo_ela_nao_carrega_o_arrasto_do_filtro() {
    let (pos, faces) = grelha(21, 0.1);
    let mut medidas = Vec::new();
    for f in [0.25, 0.5, 1.0] {
        let c = corre(condicoes_de_filtro(Modo::Inflar, f), &pos, &faces, 2, false);
        let pico = c.desloc.iter().map(|d| norma(*d)).fold(0.0, f64::max);
        medidas.push((f, pico));
        println!("forca {f:.2} -> pico {pico:.6}");
    }
    let base = medidas[2].1;
    assert!(base > 1e-6, "a fixtura nao produz o fenomeno");
    // `alpha = forca²` ⇒ o pico tem de cair com o QUADRADO.
    for (f, pico) in &medidas {
        let esperado = base * f * f;
        let erro = (pico - esperado).abs() / base;
        println!("  forca {f:.2}: {pico:.6} contra {esperado:.6} (erro relativo {erro:.4})");
        assert!(
            erro < 0.02,
            "a forca do traco nao e' quadratica em {f} -- a premissa desta medicao caiu"
        );
    }
    // E o sinal: a força não o carrega (só o `flip` o faz).
    let neg = corre(
        condicoes_de_filtro(Modo::Inflar, -1.0),
        &pos,
        &faces,
        2,
        false,
    );
    let pico_neg = neg.desloc.iter().map(|d| norma(*d)).fold(0.0, f64::max);
    assert!(
        (pico_neg - base).abs() / base < 1e-9,
        "forca negativa mudou o resultado -- entao ela TEM sinal e esta medicao esta errada"
    );
    println!("forca -1,00 -> pico {pico_neg:.6} (identico ao de +1,00: o sinal vive no flip)");
}

/// ⭐⭐⭐ **O *Inflate* LÊ A FOTOGRAFIA QUE O CHAMADOR LHE DER** — a lei já exprime
/// as DUAS, e o que as separa é uma linha do adaptador, não um kernel novo.
///
/// A espec §7 avisa que *«a mesma palavra nomeia duas leis»*: o traço do pincel
/// lê as normais do início e o filtro refresca-as a cada passo. Esta bancada
/// mede que [`Passo::normais`] é o parâmetro que já as separa — e dá o NÚMERO da
/// divergência, para que ninguém a trate como detalhe.
///
/// ⚠️ **O CONTROLO é o passo único:** com um só passo simulado as duas
/// fotografias são a mesma, e o desvio tem de ser exactamente `0`. Sem ele, uma
/// divergência qualquer (ruído, ordem) leria-se como a lei.
#[test]
fn o_inflar_le_a_fotografia_de_normais_que_o_chamador_lhe_der() {
    let (pos, faces) = esfera(24, 12);
    let p = condicoes_de_filtro(Modo::Inflar, 1.0);

    // CONTROLO: um passo simulado (o 1.º nunca simula) ⇒ as duas fotografias
    // são a mesma, ao bit.
    let a1 = corre(p, &pos, &faces, 2, false);
    let b1 = corre(p, &pos, &faces, 2, true);
    let ctrl = a1
        .desloc
        .iter()
        .zip(&b1.desloc)
        .map(|(a, b)| norma([a[0] - b[0], a[1] - b[1], a[2] - b[2]]))
        .fold(0.0, f64::max);
    println!("controlo (1 passo simulado): desvio maximo {ctrl:.3e}");
    assert_eq!(ctrl, 0.0, "com um passo as duas fotografias tem de coincidir");

    // A LEI: com seis passos simulados a superfície já mudou, e as duas
    // fotografias divergem.
    let a = corre(p, &pos, &faces, 7, false);
    let b = corre(p, &pos, &faces, 7, true);
    let desvio = a
        .desloc
        .iter()
        .zip(&b.desloc)
        .map(|(x, y)| norma([x[0] - y[0], x[1] - y[1], x[2] - y[2]]))
        .fold(0.0, f64::max);
    let pico = a.desloc.iter().map(|d| norma(*d)).fold(0.0, f64::max);
    println!(
        "6 passos: congelada contra refrescada -> desvio maximo {desvio:.6} \
         ({:.2}% do proprio deslocamento de {pico:.6})",
        100.0 * desvio / pico.max(1e-12)
    );
    assert!(pico > 1e-6, "a fixtura nao produz o fenomeno");
    assert!(
        desvio > 0.0,
        "as duas fotografias deram o mesmo -- ou a peca nao se curva, \
         ou a lei nao le' `Passo::normais`, e nos dois casos esta bancada nao mede nada"
    );
}

/// ⭐⭐⭐ **OS OITO MODOS SÃO EQUIVARIANTES; UMA GRAVIDADE NÃO É** — é a medição
/// que diz que o tipo *Gravity* é expressão nova, e não composição.
///
/// ⛔⛔ **A 1.ª redacção deste gate mediu a coisa errada, e a medição apanhou-a:**
/// ela perguntava se a direcção é CONSTANTE sobre a peça, e acusou três modos —
/// o *Push* (`1,0000`), o *Grab* e o *Snake Hook* (`0,999`). Todos os três estão
/// certos e nenhum é uma gravidade: o Push empurra ao longo da normal da **área**
/// (que é **um** vector, não a normal de cada vértice — foi essa a premissa
/// errada), e os dois de âncora levam toda a gente ao longo do `δ` do cursor.
/// *Uma direcção única não é uma direcção de MUNDO.*
///
/// ⭐ **O discriminador certo é a EQUIVARIÂNCIA:** rode a peça **e** o gesto um
/// quarto de volta. Tudo o que a malha ou o cursor ditam roda junto — o
/// deslocamento novo é o antigo rodado. Uma força de mundo **não** roda: é
/// exactamente isso que a matriz de orientação da espec §7 compra, e é por isso
/// que ela existe lá. ⇒ *nenhum dos oito exprime uma gravidade*, e a prova é que
/// os oito passam neste gate.
#[test]
fn os_oito_modos_rodam_com_a_peca_logo_nenhum_e_uma_gravidade() {
    let (pos, faces) = esfera(24, 12);
    let mut pior = 0.0_f64;
    for modo in [
        Modo::Arrastar,
        Modo::Empurrar,
        Modo::ApertarPonto,
        Modo::ApertarLinha,
        Modo::Inflar,
        Modo::Agarrar,
        Modo::Gancho,
        Modo::Expandir,
    ] {
        let p = condicoes_de_filtro(modo, 1.0);
        let a = corre_com(p, &pos, &faces, 3, false, false);
        let b = corre_com(p, &pos, &faces, 3, false, true);
        let pico = a.desloc.iter().map(|d| norma(*d)).fold(0.0, f64::max);
        assert!(pico > 1e-9, "{modo:?} nao moveu nada -- a fixtura nao produz o fenomeno");
        // O deslocamento da corrida rodada tem de ser o da corrida normal, rodado.
        let desvio = a
            .desloc
            .iter()
            .zip(&b.desloc)
            .map(|(x, y)| {
                let r = roda(*x);
                norma([r[0] - y[0], r[1] - y[1], r[2] - y[2]])
            })
            .fold(0.0, f64::max)
            / pico;
        println!("{modo:<19?} desvio de equivariancia {desvio:.3e} (pico {pico:.6})");
        pior = pior.max(desvio);
        assert!(
            desvio < 1e-9,
            "{modo:?} NAO roda com a peca (desvio relativo {desvio:.3e}) -- \
             ou ele ja' exprime uma direccao de mundo, ou esta regua esta partida"
        );
    }
    println!("pior desvio dos oito: {pior:.3e}");
}

// ————————————————————— os portões das DUAS leis novas —————————————————————

/// ⭐⭐⭐ **O ARRASTO DO FILTRO É UMA RECTA COM SINAL** — o contrário exacto do
/// traço, que o gate irmão mede quadrático e sem sinal.
///
/// A espec §7 põe `S = força_base · Δpx · 0,001`: dobrar os pixels dobra o
/// efeito, e arrastar para o outro lado espelha-o. ⚠️ **A barra sai de uma
/// medição, não de uma escolha:** o solver relaxa restrições depois da
/// integração, e essa metade **não** é linear — por isso a linearidade afirma-se
/// sobre UM passo simulado, onde ela é a lei que acabou de correr.
#[test]
fn o_arrasto_do_filtro_e_uma_recta_com_sinal() {
    let (pos, faces) = grelha(21, 0.1);
    let pico = |s: f64| {
        let c = corre(filtro(Modo::Gravidade, s), &pos, &faces, 2, false);
        c.desloc.iter().map(|d| norma(*d)).fold(0.0, f64::max)
    };
    // O ZERO é exacto: sem arrasto não há filtro nenhum.
    assert_eq!(pico(0.0), 0.0, "com s = 0 a peca tem de ficar parada, ao bit");
    let base = pico(1.0);
    assert!(base > 1e-9, "a fixtura nao produz o fenomeno");
    for s in [0.25, 0.5, 1.0] {
        let m = pico(s);
        let erro = (m - base * s).abs() / base;
        println!("s {s:.2} -> pico {m:.6} contra a recta {:.6} (erro {erro:.2e})", base * s);
        assert!(erro < 1e-9, "o arrasto do filtro nao e' linear em {s}");
    }
    // ⭐ E o SINAL vive no próprio `s` — não num `flip` ao lado.
    let neg = corre(filtro(Modo::Gravidade, -1.0), &pos, &faces, 2, false);
    let pos_ = corre(filtro(Modo::Gravidade, 1.0), &pos, &faces, 2, false);
    let assimetria = neg
        .desloc
        .iter()
        .zip(&pos_.desloc)
        .map(|(a, b)| norma([a[0] + b[0], a[1] + b[1], a[2] + b[2]]))
        .fold(0.0, f64::max)
        / base;
    println!("s = -1 contra -(s = +1): assimetria relativa {assimetria:.2e}");
    assert!(
        assimetria < 1e-9,
        "arrastar para o outro lado nao espelhou o efeito (assimetria {assimetria:.2e})"
    );
}

/// ⭐⭐⭐ **A GRAVIDADE DO FILTRO NÃO RODA COM A PEÇA — e é isso que ela É.**
///
/// O gate irmão mede que os oito modos do traço rodam junto com a peça e o gesto
/// (equivariância `≤ 5,0e-11`), logo nenhum deles exprimia uma força de mundo.
/// Aqui prova-se o outro lado: com a peça e o gesto rodados um quarto de volta e
/// o eixo da gravidade PARADO, o deslocamento **não** acompanha.
///
/// ⚠️ **O CONTROLO corre no mesmo accionamento** (o *Inflate* como filtro): sem
/// ele, um desvio grande leria-se como *«a gravidade é de mundo»* quando podia
/// ser só a porta do filtro a estar partida.
#[test]
fn a_gravidade_do_filtro_nao_roda_com_a_peca_e_o_inflate_roda() {
    let (pos, faces) = esfera(24, 12);
    let equivariancia = |p: Pincel| {
        let a = corre_com(p, &pos, &faces, 3, false, false);
        let b = corre_com(p, &pos, &faces, 3, false, true);
        let pico = a.desloc.iter().map(|d| norma(*d)).fold(0.0, f64::max);
        assert!(pico > 1e-9, "a fixtura nao produz o fenomeno");
        (
            a.desloc
                .iter()
                .zip(&b.desloc)
                .map(|(x, y)| {
                    let r = roda(*x);
                    norma([r[0] - y[0], r[1] - y[1], r[2] - y[2]])
                })
                .fold(0.0, f64::max)
                / pico,
            pico,
        )
    };
    let (ctrl, pc) = equivariancia(filtro(Modo::Inflar, 1.0));
    println!("CONTROLO Inflate como filtro: desvio {ctrl:.3e} (pico {pc:.6})");
    assert!(
        ctrl < 1e-9,
        "o Inflate como filtro deixou de rodar com a peca -- a porta do filtro esta' partida, \
         e sem este controlo o gate da gravidade nao afirma nada"
    );
    let (grav, pg) = equivariancia(filtro(Modo::Gravidade, 1.0));
    println!("Gravidade: desvio {grav:.3e} (pico {pg:.6})");
    assert!(
        grav > 0.5,
        "a gravidade rodou com a peca (desvio {grav:.3e}) -- entao ela nao e' uma forca de MUNDO"
    );
}

/// ⭐⭐⭐ **A ESCALA DO FILTRO ANCORA EM `p⁰ + p⁰ · f` E HONRA OS EIXOS** (espec §7).
///
/// Duas afirmações numa fixtura só, porque elas são a mesma lei vista de dois
/// lados: com os três eixos ligados o deslocamento é **radial a partir da origem
/// do objecto**; com um eixo só, ele vive nesse eixo e em mais nenhum.
///
/// ⚠️ **A peça é uma esfera DESLOCADA da origem**, e isso é load-bearing: numa
/// esfera centrada, «radial a partir da origem» e «ao longo da normal» são a
/// MESMA direcção, e a fixtura não separaria a Escala do Inflate.
///
/// ⛔⛔ **A 1.ª redacção deste gate mediu o DESLOCAMENTO e reprovou com `0,889`
/// de cosseno — e a régua é que estava errada.** Uma âncora não teletransporta
/// o vértice: ela é uma restrição de força `0,01` que o solver relaxa CONTRA as
/// molas do tecido, e uma homotetia estica **toda** aresta, logo as molas puxam
/// de volta ao longo das arestas, que não são radiais. *O que a lei afirma é o
/// ALVO; onde o vértice acaba é a lei MAIS o tecido.* ⇒ o gate mede a âncora, e
/// o desvio do deslocamento fica ao lado como MEDIÇÃO, não como barra.
#[test]
fn a_escala_do_filtro_ancora_no_repouso_e_honra_os_eixos() {
    let (base, faces) = esfera(24, 12);
    let pos: Vec<V3> = base.iter().map(|p| [p[0] + 2.0, p[1] + 0.5, p[2]]).collect();

    // (a) os três eixos: a ÂNCORA é `p⁰ · (1 + f)`, radial a partir da origem.
    let c = corre(filtro(Modo::Escala, 1.0), &pos, &faces, 2, false);
    let mut pior_cos = 1.0_f64;
    let mut pico = 0.0_f64;
    for (a, p0) in c.ancora.iter().zip(&c.repouso) {
        let d = [a[0] - p0[0], a[1] - p0[1], a[2] - p0[2]];
        let (md, mp) = (norma(d), norma(*p0));
        pico = pico.max(md);
        if md < 1e-12 || mp < 1e-12 {
            continue;
        }
        pior_cos = pior_cos.min((d[0] * p0[0] + d[1] * p0[1] + d[2] * p0[2]) / (md * mp));
    }
    println!("tres eixos: pior cosseno da ANCORA contra o raio da origem {pior_cos:.9}");
    assert!(pico > 1e-9, "a ancora nao saiu do repouso -- a fixtura nao produz o fenomeno");
    assert!(
        pior_cos > 1.0 - 1e-12,
        "a ancora da Escala nao e' radial a partir da origem (pior cosseno {pior_cos:.9})"
    );
    // A MEDIÇÃO ao lado: quanto o tecido desvia o resultado da lei.
    let mut cos_desloc = 1.0_f64;
    for (d, p0) in c.desloc.iter().zip(&c.repouso) {
        let (md, mp) = (norma(*d), norma(*p0));
        if md < 1e-9 || mp < 1e-9 {
            continue;
        }
        cos_desloc = cos_desloc.min((d[0] * p0[0] + d[1] * p0[1] + d[2] * p0[2]) / (md * mp));
    }
    println!(
        "  e o DESLOCAMENTO desvia ate' {:.1}° do raio -- e' o tecido a resistir a` homotetia, \
         nao a lei a falhar",
        cos_desloc.clamp(-1.0, 1.0).acos().to_degrees()
    );

    // (b) só o eixo X: a âncora não sai desse eixo.
    let mut so_x = filtro(Modo::Escala, 1.0);
    so_x.referencial.activo = [true, false, false];
    let c = corre(so_x, &pos, &faces, 2, false);
    let (mut px, mut fuga) = (0.0_f64, 0.0_f64);
    for (a, p0) in c.ancora.iter().zip(&c.repouso) {
        let d = [a[0] - p0[0], a[1] - p0[1], a[2] - p0[2]];
        px = px.max(d[0].abs());
        fuga = fuga.max(d[1].abs().max(d[2].abs()));
    }
    println!("so' o eixo X: pico da ancora em x {px:.6}, fuga em y/z {fuga:.3e}");
    assert!(px > 1e-9, "com o eixo X ligado a ancora tinha de sair do repouso em x");
    assert_eq!(fuga, 0.0, "a ancora da Escala escapou dos eixos ligados");
}
