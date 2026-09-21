//! ⭐⭐⭐ **OS GATES DO AJUSTE DAS ALÇAS** — que ele é o ÓPTIMO da cúbica, e que conciliar SAI dele.
//!
//! ⛔⛔⛔ **Este ficheiro chamava-se *«os gates da CONCILIAÇÃO»* e a premissa dele morreu em
//! 2026-09-20.** Ele nasceu do report de 2026-09-19 (*«muitas irregularidades … certamente um mau
//! tratamento das alças dos handles»*) e afirmava que conciliar as duas alças de um nó era a cura.
//! O report seguinte do dono — *«não fica bom. Muito curvado»*, com foto — mediu-se contra a
//! conciliação: ela custava **`9,3×`** de serpentina e **`1,6×`** de fidelidade para comprar
//! `13,65°` de quebra de tangente que **o CHÃO do modelo paga igual**. A tabela está em
//! [`super::aplica_pela_curva`]; o passe foi apagado.
//!
//! ⇒ o que se afirma aqui agora é o **contrário**: que o ajuste livre é o melhor que uma cúbica
//! pode fazer, e que conciliar — reimplementado dentro do gate, à letra como ele era — **piora**.
//! *Uma recusa medida que não é executável envelhece; esta corre em todo portão.*
//!
//! ⛔ Ele vive ao lado do [`super::tests`] e não dentro dele por **tecto de LOC**: o irmão estava a
//! `882` linhas. *O ficheiro é uma unidade de manutenção; a lei é a mesma.*

use super::*;
use ph2d_skeleton::{SkinBone, Xform};
use ph2d_vec_scene::{ShapeKind, VecPath, cook};

use super::tests::{forma, pele};

/// A mesma barra, **elíptica** — uma forma cujos nós têm alças a sério.
///
/// ⛔⛔ **A [`forma`] não serve a um gate de TANGENTE e isso está medido:** um rectângulo tem as
/// alças **em cima das âncoras**, logo a lei ingénua não deixa tangente nenhuma para comparar e a
/// régua lê `0,000°` dos dois lados — *verde por vácuo sobre o defeito*.
///
/// ⛔⛔ **E a `RoundRect` também não serve, por um motivo que engana:** ali o arredondamento é um
/// `corner_radius` **dentro do vértice** (Live Corners, ADR-0121), resolvido só no `cooked()` — a
/// FONTE, que é sobre quem esta lei corre, continua a ser um rectângulo de alças degeneradas.
/// *Uma forma que parece curva na tela pode ser recta na fonte.*
fn forma_curva() -> VecPath {
    cook(ShapeKind::Ellipse, [0.0, 0.0], [40.0, 10.0], &[])
}

/// A tabela do padrão-ouro DERIVADA da forma: o 1.º osso manda na metade esquerda, o 2.º na direita.
///
/// ⚠️ **Derivada e não escrita à mão**, porque a [`forma_curva`] tem mais nós do que o rectângulo e
/// uma tabela com a contagem errada é **ignorada em silêncio** (cai na lei derivada).
fn tabela_de(p: &VecPath) -> Vec<f64> {
    let mut out = Vec::new();
    for v in p.verts_all() {
        let linha = if v.anchor[0] < 20.0 {
            [1.0, 0.0]
        } else {
            [0.0, 1.0]
        };
        for _ in 0..3 {
            out.extend_from_slice(&linha);
        }
    }
    out
}

/// O ajuste, o óptimo e a lei ingénua num segmento — o maior afastamento de cada um à curva
/// VERDADEIRA, medido nos mesmos `t`.
///
/// ⭐ O **óptimo** é calculado aqui de raiz e não pela porta do produto: mínimos quadrados dos dois
/// pontos de controlo (não das correcções) sobre `256` amostras, com as pontas presas na verdade.
/// *Uma régua derivada da função que ela julga não a pode julgar.*
fn erros_do_segmento(k: &Skin, fonte: &VecPath, t: &[f64], seg: usize) -> (f64, f64, f64) {
    let n = fonte.verts.len();
    let s = super::SegmentoDaPele {
        src: super::cubica(&fonte.verts, seg, n),
        pele: k,
        ra: super::linha(t, 2, seg),
        rb: super::linha(t, 2, (seg + 1) % n),
        correcoes: &[],
        rigido: true,
        campo: None,
        indice: None,
        suave: None,
    };
    const N: usize = 256;
    let ts: Vec<f64> = (0..=N).map(|i| i as f64 / N as f64).collect();
    let verdade: Vec<kurbo::Point> = ts.iter().map(|&u| s.ponto(u)).collect();

    // O ÓPTIMO: `min ‖verdade(t) − B₀P₀ − B₁P₁ − B₂P₂ − B₃P₃‖²` em `P₁, P₂`, com `P₀`/`P₃` presos.
    let (p0, p3) = (verdade[0], verdade[N]);
    let (mut a11, mut a12, mut a22) = (0.0_f64, 0.0_f64, 0.0_f64);
    let (mut b1, mut b2) = (kurbo::Vec2::ZERO, kurbo::Vec2::ZERO);
    for (i, &u) in ts.iter().enumerate() {
        let v = 1.0 - u;
        let (w0, w1, w2, w3) = (v * v * v, 3.0 * v * v * u, 3.0 * v * u * u, u * u * u);
        let d = verdade[i] - (p0.to_vec2() * w0 + p3.to_vec2() * w3).to_point();
        a11 = w1.mul_add(w1, a11);
        a12 = w1.mul_add(w2, a12);
        a22 = w2.mul_add(w2, a22);
        b1 += d * w1;
        b2 += d * w2;
    }
    let det = a12.mul_add(-a12, a11 * a22);
    let melhor = kurbo::CubicBez::new(
        p0,
        ((b1 * a22 - b2 * a12) / det).to_point(),
        ((b2 * a11 - b1 * a12) / det).to_point(),
        p3,
    );

    // O AJUSTE do produto e a lei INGÉNUA, sobre a mesma fonte.
    let mut ing = fonte.clone();
    crate::aplica_corrigido(k, &mut ing, t, &[]);
    let mut aj = fonte.clone();
    aplica_pela_curva(k, &mut aj, t, &[]);
    let (ci, ca) = (
        super::cubica(&ing.verts, seg, n),
        super::cubica(&aj.verts, seg, n),
    );
    let mut e = (0.0_f64, 0.0_f64, 0.0_f64);
    for (i, &u) in ts.iter().enumerate() {
        e.0 = e.0.max((verdade[i] - ca.eval(u)).hypot());
        e.1 = e.1.max((verdade[i] - melhor.eval(u)).hypot());
        e.2 = e.2.max((verdade[i] - ci.eval(u)).hypot());
    }
    e
}

/// ⭐⭐⭐ **O AJUSTE É O MELHOR QUE UMA CÚBICA PODE FAZER — e o CONTROLO é a lei do Rive.**
///
/// A lei ingénua ([`crate::aplica_corrigido`]) é, à letra, o que o `rive-runtime` faz: um afim por
/// vértice aplicado às três metades dele (`src/shapes/cubic_vertex.cpp`, MIT). Ela erra no interior
/// de cada segmento porque a pele é um mapa **não-afim**, e é esse erro que o ajuste come.
///
/// # As três metades
///
/// 1. **O ajuste bate o ÓPTIMO** — o melhor par de alças possível para aquele segmento, calculado
///    aqui de raiz sobre `256` amostras. Se o ajuste se afastasse dele, algum passe a jusante
///    estaria a estragar a solução.
/// 2. **A lei ingénua é muito pior** — sem esta metade a primeira ficaria verde numa fixtura em que
///    o mapa por acaso é afim e os três erros são zero.
/// 3. ⛔⛔⛔ **CONCILIAR SAI DO ÓPTIMO, e é por isso que o passe foi apagado.** A conciliação está
///    reimplementada abaixo **à letra como ela era** (as duas alças rodadas para a média dos
///    desvios aos eixos, pesada pelo comprimento), e o que se afirma é que ela **piora**. *Uma
///    recusa medida que não corre é uma nota; esta é um gate.*
#[test]
fn o_ajuste_e_o_optimo_da_cubica_e_conciliar_sai_dele() {
    let k = pele(1.2);
    let fonte = forma_curva();
    let t = tabela_de(&fonte);
    let n = fonte.verts.len();
    let (mut pior_aj, mut pior_op, mut pior_in) = (0.0_f64, 0.0_f64, 0.0_f64);
    for seg in 0..n {
        let (aj, op, ing) = erros_do_segmento(&k, &fonte, &t, seg);
        pior_aj = pior_aj.max(aj);
        pior_op = pior_op.max(op);
        pior_in = pior_in.max(ing);
    }
    eprintln!(
        "[alcas] pior erro à curva verdadeira: ajuste {pior_aj:.6} · óptimo {pior_op:.6} · \
         ingénua (Rive) {pior_in:.6}"
    );
    assert!(
        pior_in > pior_op * 4.0,
        "a lei ingénua erra so' {pior_in} contra {pior_op} do óptimo — a fixtura nao contem o \
         fenomeno, e a asserção do ajuste passa a ser trivial"
    );
    assert!(
        pior_aj < pior_op * 1.05,
        "o ajuste ({pior_aj}) afastou-se do ÓPTIMO da cúbica ({pior_op}): algum passe a jusante \
         esta' a mexer nas alças depois de elas estarem certas"
    );
    // ⭐ A CONCILIAÇÃO, reimplementada à letra como ela era em 2026-09-19.
    let mut ing = fonte.clone();
    crate::aplica_corrigido(&k, &mut ing, &t, &[]);
    let mut conc = fonte.clone();
    aplica_pela_curva(&k, &mut conc, &t, &[]);
    for no in 0..n {
        let a = kurbo::Point::new(conc.verts[no].anchor[0], conc.verts[no].anchor[1]);
        let hs = [
            kurbo::Point::new(conc.verts[no].in_handle[0], conc.verts[no].in_handle[1]) - a,
            kurbo::Point::new(conc.verts[no].out_handle[0], conc.verts[no].out_handle[1]) - a,
        ];
        // O EIXO de cada metade: a imagem, pelo afim do nó, da tangente que a FONTE tinha.
        let ia = kurbo::Point::new(ing.verts[no].anchor[0], ing.verts[no].anchor[1]);
        let es = [
            kurbo::Point::new(ing.verts[no].in_handle[0], ing.verts[no].in_handle[1]) - ia,
            kurbo::Point::new(ing.verts[no].out_handle[0], ing.verts[no].out_handle[1]) - ia,
        ];
        let (mut soma, mut desvio) = ((0.0_f64, 0.0_f64), [0.0_f64; 2]);
        for i in 0..2 {
            if es[i].hypot() <= 0.0 {
                continue;
            }
            let l = hs[i].hypot();
            desvio[i] = (hs[i].atan2() - es[i].atan2()).rem_euclid(std::f64::consts::TAU);
            if desvio[i] > std::f64::consts::PI {
                desvio[i] -= std::f64::consts::TAU;
            }
            soma = (desvio[i].mul_add(l, soma.0), soma.1 + l);
        }
        if soma.1 <= 0.0 {
            continue;
        }
        let comum = soma.0 / soma.1;
        for i in 0..2 {
            if es[i].hypot() <= 0.0 {
                continue;
            }
            let (si, co) = (comum - desvio[i]).sin_cos();
            let g = kurbo::Vec2::new(
                si.mul_add(-hs[i].y, co * hs[i].x),
                si.mul_add(hs[i].x, co * hs[i].y),
            );
            let q = [a.x + g.x, a.y + g.y];
            if i == 0 {
                conc.verts[no].in_handle = q;
            } else {
                conc.verts[no].out_handle = q;
            }
        }
    }
    let mut pior_conc = 0.0_f64;
    for seg in 0..n {
        let s = super::SegmentoDaPele {
            src: super::cubica(&fonte.verts, seg, n),
            pele: &k,
            ra: super::linha(&t, 2, seg),
            rb: super::linha(&t, 2, (seg + 1) % n),
            correcoes: &[],
            rigido: true,
            campo: None,
            indice: None,
            suave: None,
        };
        let c = super::cubica(&conc.verts, seg, n);
        for i in 0..=256 {
            let u = f64::from(i) / 256.0;
            pior_conc = pior_conc.max((s.ponto(u) - c.eval(u)).hypot());
        }
    }
    eprintln!("[alcas] conciliada {pior_conc:.6} (o ajuste sozinho é {pior_aj:.6})");
    assert!(
        pior_conc > pior_aj * 1.20,
        "conciliar as alças ({pior_conc}) nao piorou o ajuste ({pior_aj}) — ou a reimplementação \
         acima deixou de ser a que foi apagada, ou o ajuste deixou de ser o óptimo"
    );
}

/// ⭐⭐⭐ **NUMA FORMA DE ARESTAS RECTAS O AJUSTE ARQUEIA A ARESTA — e é para isto que ele existe.**
///
/// ⛔⛔⛔ **O gate que estava aqui afirmava o CONTRÁRIO — *«o passe não toca em nada»* — e era sobre
/// a conciliação, que foi apagada.** Deixá-lo de pé seria ficar **verde a afirmar nada**: sem aquele
/// passe, *«ele não toca em nada»* é trivialmente verdadeiro.
///
/// A lei que fica é a que justifica o ajuste existir: a imagem verdadeira de uma recta sob a pele
/// **arqueia**, e a lei ingénua entrega uma recta. Um rectângulo tem as quatro alças em cima das
/// âncoras, logo é a fixtura mais dura que há para isto.
///
/// ⚠️ **E ela não pode arquear em REPOUSO**, que é a metade que impede a cura barata de bulir com
/// arte parada.
#[test]
fn numa_forma_de_arestas_rectas_o_ajuste_arqueia_a_aresta() {
    let k = pele(1.2);
    let fonte = forma();
    let t = tabela_de(&fonte);
    let n = fonte.verts.len();
    let (mut pior_aj, mut pior_in) = (0.0_f64, 0.0_f64);
    for seg in 0..n {
        let (aj, _, ing) = erros_do_segmento(&k, &fonte, &t, seg);
        pior_aj = pior_aj.max(aj);
        pior_in = pior_in.max(ing);
    }
    eprintln!("[alcas] recta: ajuste {pior_aj:.6} · ingénua {pior_in:.6}");
    assert!(
        pior_in > 0.01,
        "a imagem da recta desviou so' {pior_in} da recta — a fixtura nao contem o fenomeno"
    );
    assert!(
        pior_aj < pior_in / 4.0,
        "numa forma de arestas rectas o ajuste ({pior_aj}) nao arqueia a aresta para seguir a \
         verdade ({pior_in}) — sem isto a F30 nao vale nada na arte que a caneta desenha"
    );
    let repouso = pele(0.0);
    let mut parada = fonte.clone();
    aplica_pela_curva(&repouso, &mut parada, &t, &[]);
    for (a, b) in fonte.verts.iter().zip(&parada.verts) {
        assert_eq!(
            (a.in_handle, a.out_handle),
            (b.in_handle, b.out_handle),
            "em repouso o ajuste mexeu numa alça — a diferença é exactamente zero ali, logo o \
             segundo membro do sistema tem de ser zero"
        );
    }
}

/// ⭐⭐ **O `NaN` não chega ao desenho com um osso de escala zero num eixo.**

#[test]
fn um_osso_colapsado_nao_devolve_nan() {
    // ⭐⭐ **Um eixo colapsado, e o TENDÃO posto à mão** — sem o tendão os dois ossos partilham a
    // coluna `0` da tabela e a lei nunca chega ao osso morto.
    let mut vivo = SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        10.0,
        2.0,
        Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular");
    vivo.tendon = 0;
    let mut morto = SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        10.0,
        2.0,
        // ⚠️ O `y` colapsa e o `x` não — é essa a célula que mata a cerca. Com a pele INTEIRA
        // colapsada o mapa é constante, a alça cai em cima da âncora e a [`reconcilia`] salta o nó
        // **antes** de olhar para o eixo: o `NaN` é calculado e nunca chega ao desenho.
        Xform([1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular");
    morto.tendon = 1;
    let k = Skin::new(vec![vivo, morto]).expect("2 ossos");

    let mut p = cook(ShapeKind::Ellipse, [0.0, 0.0], [10.0, 5.0], &[]);
    // O nó da ESQUERDA (alças verticais) fica com o osso morto; os outros com o vivo.
    let mut t = Vec::new();
    for v in p.verts_all() {
        let linha = if v.anchor[0] < 1.0 {
            [0.0, 1.0]
        } else {
            [1.0, 0.0]
        };
        for _ in 0..3 {
            t.extend_from_slice(&linha);
        }
    }
    aplica_pela_curva(&k, &mut p, &t, &[]);
    for v in p.verts_all() {
        for q in [v.anchor, v.in_handle, v.out_handle] {
            assert!(
                q[0].is_finite() && q[1].is_finite(),
                "a forma virou NaN com um osso de escala zero num eixo — a cerca do vector nulo \
                 saiu, e uma alça revivida pela correcção livre encontrou um eixo `NaN`"
            );
        }
    }
}
