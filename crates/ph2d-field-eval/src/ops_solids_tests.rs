//! ⭐⭐⭐ **AS SEIS FÓRMULAS, PROVADAS ANTES DE SEREM LIGADAS** (W106).
//!
//! # Por que estes gates vêm ANTES da ligação
//!
//! Ligar uma primitiva nova custa ~13 sítios (o enum, o `kind`, as dimensões do painel, os limites
//! de filete, o tamanho característico, o raio delimitador, a escala, a rotulagem, a paleta, o
//! censo). ⚠️ **Nenhum deles diz se a FÓRMULA está certa** — o censo mede que a marcha é segura e
//! que o filete deixa corpo, não que a forma é a que se pediu. *Um erro de geometria descoberto
//! depois de treze ligações é treze ligações a refazer.*
//!
//! ⇒ cada fórmula responde aqui, contra pontos escolhidos **onde a resposta é conhecida sem a
//! fórmula**: um vértice, um centro, um ponto a uma distância medida com a régua.
//!
//! # A régua, e o que ela NÃO é
//!
//! `Field::from_tree(..).at(x,y,z)` — o mesmo avaliador do produto. ⚠️ **Não há relógio em nenhum
//! destes gates**, então nada aqui pertence à família de flakes de carga do `CLAUDE.md` §5.0.
//!
//! ⚠️ **A barra é geométrica, não um epsilon de gosto.** Onde a construção da casa é um
//! subestimador conhecido (a interseção de meias-fatias, como no prisma e no cone), o gate afirma a
//! **desigualdade** que a construção promete — nunca uma igualdade que ela não pode dar.

use super::*;
use crate::Field;

/// Avalia uma árvore num ponto.
fn at(t: &fidget::context::Tree, p: [f64; 3]) -> f64 {
    Field::from_tree(t).at(p[0], p[1], p[2])
}

/// A maior norma do gradiente sobre uma grelha — a régua da marcha.
///
/// ⚠️ Amostra **fora** da peça e longe das quinas, que é onde a construção da casa promete `≤ 1`;
/// nas quinas ela subestima de propósito, e isso **não** é violação (subestimar é seguro).
fn pior_gradiente(t: &fidget::context::Tree, raio: f64, n: i32) -> f64 {
    let f = Field::from_tree(t);
    let mut pior: f64 = 0.0;
    for i in -n..=n {
        for j in -n..=n {
            for k in -n..=n {
                let p = [
                    f64::from(i) / f64::from(n) * raio,
                    f64::from(j) / f64::from(n) * raio,
                    f64::from(k) / f64::from(n) * raio,
                ];
                pior = pior.max(f.gradient_norm(p[0], p[1], p[2], 1.0e-4));
            }
        }
    }
    pior
}

/// ⛔ **O CONTROLO DE TODOS OS OUTROS.** Se a régua não distingue dentro de fora numa forma cuja
/// resposta toda a gente sabe, nenhum gate abaixo quer dizer nada.
#[test]
fn the_ruler_can_tell_inside_from_outside() {
    let t = sd_octahedron(1.0, 0.0, 0.0);
    assert!(at(&t, [0.0, 0.0, 0.0]) < 0.0, "o centro tem de ser dentro");
    assert!(at(&t, [5.0, 0.0, 0.0]) > 0.0, "longe tem de ser fora");
}

/// ⭐⭐ **O OCTAEDRO tem o vértice ONDE O NÚMERO DIZ** — `radius` é o circunraio.
///
/// ⚠️ Esta é a afirmação que separa circunraio de apótema, e ela é visível: com o apótema o vértice
/// ficaria a `radius·√3`, e trocar uma esfera por um octaedro faria a peça **crescer 73 %**.
#[test]
fn the_octahedron_puts_its_vertex_at_the_radius() {
    let r = 1.3;
    let t = sd_octahedron(r, 0.0, 0.0);
    for v in [[r, 0.0, 0.0], [-r, 0.0, 0.0], [0.0, r, 0.0], [0.0, 0.0, -r]] {
        assert!(
            at(&t, v).abs() < 1.0e-6,
            "o vertice {v:?} devia estar na superficie; deu {}",
            at(&t, v)
        );
    }
    // ⭐ E o centro da face é o ponto `(r/3, r/3, r/3)` — que dista `r/√3` da origem.
    //
    // ⚠️ **A 1.ª escrita deste gate punha o PONTO em `(r/√3, r/√3, r/√3)`**, confundindo a
    // distância com a coordenada, e reprovou uma fórmula que estava certa. *Um gate errado sobre
    // produto certo custa o mesmo tempo que um bug — e engana na direcção contrária.*
    let c = r / 3.0;
    assert!(
        at(&t, [c, c, c]).abs() < 1.0e-6,
        "o centro da face devia estar na superficie; deu {}",
        at(&t, [c, c, c])
    );
    // E ele está mais perto do centro do que o vértice — é isso que faz a face ser plana.
    assert!((c * 3.0_f64.sqrt() - r / 3.0_f64.sqrt()).abs() < 1.0e-12);
    // ⛔ O ponto a meio caminho entre dois vértices está DENTRO — é o que faz dele um sólido de
    // faces planas e não uma esfera.
    assert!(at(&t, [r * 0.5, r * 0.5, 0.0]).abs() < 1.0e-6);
    assert!(at(&t, [r * 0.7, r * 0.7, 0.0]) > 0.0);
}

/// ⭐⭐⭐ **O CONE DE PONTAS ARREDONDADAS degenera na CÁPSULA quando os dois raios são iguais** — e
/// não «aproximadamente»: a mesma expressão, o mesmo valor.
///
/// ⚠️ **É o controlo que prova que a escrita sem ramo está certa**, porque a cápsula é uma fórmula
/// independente, escrita noutro arquivo e há muito no produto. *Duas construções que concordam num
/// caso conhecido é o que uma prova de porte precisa.*
#[test]
fn a_round_cone_with_equal_radii_is_exactly_the_capsule() {
    let (r, h) = (0.4, 0.9);
    let redondo = sd_round_cone(r, r, h);
    let capsula = crate::ops::sd_capsule(r, h);
    let mut pior: f64 = 0.0;
    for i in -8..=8 {
        for j in -8..=8 {
            for k in -8..=8 {
                let p = [
                    f64::from(i) / 8.0 * 2.0,
                    f64::from(j) / 8.0 * 2.0,
                    f64::from(k) / 8.0 * 2.0,
                ];
                pior = pior.max((at(&redondo, p) - at(&capsula, p)).abs());
            }
        }
    }
    assert!(
        pior < 1.0e-6,
        "o cone redondo de raios iguais devia SER a capsula; maior desvio {pior}"
    );
}

/// ⭐⭐ **E com raios DIFERENTES ele toca as duas esferas** — o pólo de cada calota está na
/// superfície, e a parede é tangente às duas.
#[test]
fn a_round_cone_touches_both_of_its_spheres() {
    let (r1, r2, h) = (0.5, 0.2, 0.8);
    let t = sd_round_cone(r1, r2, h);
    // O pólo de baixo: a esfera `r1` centrada em `−h`.
    assert!(
        at(&t, [0.0, 0.0, -h - r1]).abs() < 1.0e-6,
        "o polo de baixo devia estar na superficie"
    );
    assert!(
        at(&t, [0.0, 0.0, h + r2]).abs() < 1.0e-6,
        "o polo de cima devia estar na superficie"
    );
    // ⭐⭐⭐ **E os DOIS equadores NÃO são iguais — é isto que separa esta forma de um tronco de
    // cone**, e a 1.ª escrita deste gate afirmava que os dois estavam na superfície.
    //
    // A tangente comum toca cada esfera à altura `b·r` acima do centro dela (`b = (r1−r2)/H > 0`).
    // ⇒ no equador da esfera GRANDE ainda não há parede a cobrir, e o ponto está na superfície;
    // no da PEQUENA a parede já passou por fora — ela vale `r2/a > r2` àquela altura —, e o ponto
    // está **dentro**. *Uma simetria assumida entre as duas pontas de uma forma assimétrica.*
    let a = (1.0_f64 - ((r1 - r2) / (2.0 * h)).powi(2)).sqrt();
    assert!(
        at(&t, [r1, 0.0, -h]).abs() < 1.0e-6,
        "o equador da esfera GRANDE esta' na superficie; deu {}",
        at(&t, [r1, 0.0, -h])
    );
    assert!(
        at(&t, [r2, 0.0, h]) < -1.0e-6,
        "o equador da esfera PEQUENA esta' coberto pela parede, logo DENTRO; deu {}",
        at(&t, [r2, 0.0, h])
    );
    // ⭐ E os dois pontos de TANGÊNCIA estão os dois na superfície — a afirmação que o gate devia
    // ter feito desde o princípio.
    let b = (r1 - r2) / (2.0 * h);
    for (r, zc) in [(r1, -h), (r2, h)] {
        let p = [a * r, 0.0, zc + b * r];
        assert!(
            at(&t, p).abs() < 1.0e-6,
            "o ponto de tangencia {p:?} devia estar na superficie; deu {}",
            at(&t, p)
        );
    }
    // ⛔ E a parede é TANGENTE, logo o ponto médio dos dois equadores fica FORA (a parede inclina
    // para dentro). Uma parede que ligasse os equadores em linha recta poria este ponto na
    // superfície — é a diferença entre um tronco de cone e o casco convexo de duas esferas.
    let meio = [(r1 + r2) * 0.5, 0.0, 0.0];
    assert!(
        at(&t, meio) < 0.0,
        "o meio da parede devia estar DENTRO (a tangente comum passa por fora da corda); deu {}",
        at(&t, meio)
    );
}

/// ⭐ **A ESFERA CORTADA perde exactamente a calota acima do corte.**
#[test]
fn a_cut_sphere_keeps_what_is_below_the_cut() {
    let (r, cut) = (1.0, 0.3);
    let t = sd_cut_sphere(r, cut, 0.0, 0.0);
    // O pólo de baixo continua na superfície; o de cima foi-se.
    assert!(at(&t, [0.0, 0.0, -r]).abs() < 1.0e-6);
    assert!(at(&t, [0.0, 0.0, r]) > 0.0, "o polo de cima foi cortado");
    // A tampa é plana em `z = cut`, e o centro dela está na superfície.
    assert!(at(&t, [0.0, 0.0, cut]).abs() < 1.0e-6);
    // ⭐ O raio da tampa é `√(r²−cut²)` — o bordo está na superfície, e um pouco além está fora.
    let w = (r * r - cut * cut).sqrt();
    assert!(at(&t, [w, 0.0, cut]).abs() < 1.0e-5);
    assert!(at(&t, [w + 0.1, 0.0, cut]) > 0.0);
}

/// ⭐⭐ **A CÚPULA OCA é uma CASCA: o miolo é VAZIO.**
///
/// ⚠️ Este é o gate que a distingue da [`sd_cut_sphere`], e sem ele as duas passariam nos mesmos
/// testes de silhueta — *duas formas com a mesma silhueta e miolos diferentes leem-se iguais em
/// qualquer régua que só olhe o contorno.*
#[test]
fn a_hollow_dome_is_hollow() {
    let (r, cut, t_esp) = (1.0, 0.0, 0.15);
    let t = sd_cut_hollow_sphere(r, cut, t_esp, 0.0, 0.0);
    // O centro da esfera está VAZIO (fora do sólido).
    assert!(
        at(&t, [0.0, 0.0, 0.0]) > 0.0,
        "o miolo de uma tigela tem de estar vazio; deu {}",
        at(&t, [0.0, 0.0, 0.0])
    );
    // A parede, no raio médio e abaixo do corte, está DENTRO.
    assert!(
        at(&t, [0.0, 0.0, -r]) < 0.0,
        "o fundo da tigela e' parede; deu {}",
        at(&t, [0.0, 0.0, -r])
    );
    // E acima do corte não há nada.
    assert!(at(&t, [0.0, 0.0, r]) > 0.0, "acima do corte foi removido");
    // ⭐ A espessura é a que se pediu: as duas faces da parede estão a `t/2` do raio médio.
    assert!(at(&t, [0.0, 0.0, -(r + t_esp * 0.5)]).abs() < 1.0e-6);
    assert!(at(&t, [0.0, 0.0, -(r - t_esp * 0.5)]).abs() < 1.0e-6);
}

/// ⭐⭐⭐ **O ELO é um ESTÁDIO, não um círculo** — e a prova é que o buraco tem lados RECTOS.
#[test]
fn a_link_is_a_stretched_torus() {
    let (major, minor, len) = (0.5, 0.15, 0.4);
    let t = sd_link(major, minor, len);
    // No plano do meio, a secção do tubo à direita: centro do tubo em `x = major`, `y = ±len`.
    for y in [-len, 0.0, len] {
        assert!(
            at(&t, [major, y, 0.0]) < 0.0,
            "o centro do tubo em y={y} devia estar dentro"
        );
        assert!(
            (at(&t, [major, y, minor])).abs() < 1.0e-6,
            "o topo do tubo em y={y} devia estar na superficie"
        );
    }
    // ⭐ O BURACO: no eixo, e em toda a extensão recta, está VAZIO.
    for y in [-len, 0.0, len] {
        assert!(
            at(&t, [0.0, y, 0.0]) > 0.0,
            "o buraco do elo em y={y} tem de estar vazio"
        );
    }
    // ⛔ E é isto que o distingue de um toro: **fora** do trecho recto ele fecha em arco, então um
    // ponto no eixo, para lá da ponta, está DENTRO do arco de fecho — não vazio.
    assert!(
        at(&t, [0.0, len + major, 0.0]) < 0.0,
        "a ponta do elo fecha em arco e o eixo la' passa dentro do tubo"
    );
}

/// ⭐ **O ÂNGULO SÓLIDO é a fatia, e o resto da esfera não vem junto.**
#[test]
fn a_solid_angle_keeps_only_its_cone() {
    let (r, ang) = (1.0_f64, 0.6_f64);
    let t = sd_solid_angle(r, ang, 0.0, 0.0);
    // No eixo, dentro do raio: dentro. Fora do raio: fora.
    assert!(at(&t, [0.0, 0.0, r * 0.5]) < 0.0);
    assert!(at(&t, [0.0, 0.0, r * 1.5]) > 0.0);
    // ⭐ Do lado de fora do cone, ao mesmo raio: FORA — é a metade que um `max` com a esfera dá e
    // uma esfera sozinha não daria.
    let fora = [
        r * 0.5 * (ang * 2.0).sin(),
        0.0,
        r * 0.5 * (ang * 2.0).cos(),
    ];
    assert!(
        at(&t, fora) > 0.0,
        "um ponto fora da abertura devia estar fora; deu {}",
        at(&t, fora)
    );
    // E o oposto do eixo é sempre fora, por mais perto que esteja.
    assert!(at(&t, [0.0, 0.0, -r * 0.1]) > 0.0);
}

/// ⭐⭐⭐ **A MARCHA É SEGURA em todas as seis** — `‖∇f‖ ≤ 1` com folga de amostragem.
///
/// ⚠️ É a propriedade de que o traçador depende: um campo que suba mais depressa que `1` faz a
/// marcha **atravessar** a superfície, e o sintoma é um buraco na peça. O censo do produto mede
/// isto por primitiva ligada; aqui mede-se **antes** de ligar.
#[test]
fn every_new_solid_marches_safely() {
    let casos: [(&str, fidget::context::Tree); 6] = [
        ("octaedro", sd_octahedron(0.8, 0.0, 0.0)),
        ("cone redondo", sd_round_cone(0.5, 0.2, 0.6)),
        ("esfera cortada", sd_cut_sphere(0.9, 0.2, 0.0, 0.0)),
        ("cupula oca", sd_cut_hollow_sphere(0.9, 0.0, 0.12, 0.0, 0.0)),
        ("elo", sd_link(0.5, 0.15, 0.35)),
        ("angulo solido", sd_solid_angle(0.9, 0.7, 0.0, 0.0)),
    ];
    // A folga é de AMOSTRAGEM: a norma sai de diferenças finitas com `eps = 1e-4`, e numa quina o
    // quociente lê ligeiramente acima de 1 sem que o campo o esteja.
    const TETO: f64 = 1.02;
    for (nome, t) in casos {
        let g = pior_gradiente(&t, 2.0, 12);
        assert!(
            g <= TETO,
            "{nome}: o campo sobe a {g} por unidade — a marcha atravessaria a superficie"
        );
        // ⛔ O controlo: um campo constante daria `0` e passaria. Tem de haver superfície.
        assert!(
            g > 0.5,
            "{nome}: gradiente {g} e' baixo demais para ser um campo de distancia"
        );
    }
}
