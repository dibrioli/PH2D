//! **Gap Closure** — a feature que faz o balde "funcionar de primeira".
//!
//! Line-art desenhado à mão quase nunca fecha: as pontas passam perto uma da outra e
//! o preenchimento escapa pela fresta. O Grease Pencil resolve prolongando as PONTAS
//! na tangente até que elas colidam com alguma linha (ou com outra extensão), e é
//! isso que portamos.
//!
//! **O twist do Harmony (`04 §3`), adotado:** o fechamento que funcionou **vira um
//! traço INVISÍVEL persistente** no desenho — não um estado efêmero da ferramenta.
//! Consequência prática enorme: re-preencher depois de editar a cor, preencher o
//! quadro vizinho, ou reabrir o arquivo amanhã **não depende de a ferramenta estar
//! com os mesmos parâmetros**. O gap ficou fechado.
//!
//! Três fontes de fechamento (as duas do GP + a ponte do Harmony):
//! 1. **Pontas** — cada extremidade de um traço aberto se prolonga na tangente.
//! 2. **Quinas mid-stroke** — onde o traço vira mais apertado que a própria espessura
//!    (raio de curvatura < espessura), o GP também estende. É por isso que ele fecha
//!    cantos em "V" que outros baldes não fecham: num "V" as duas pernas se cruzam
//!    *visualmente*, mas o vértice fica fora do preenchimento.
//! 3. **Pontas EMPARELHADAS** (BUGS #23, a régua honesta do slider): duas pontas que
//!    apontam uma para a outra a `dist ≤ reach` fecham pela RETA entre elas. Sem isto o
//!    vão CANÔNICO — o traço feito em dois tempos, pontas colineares frente a frente —
//!    era invisível: `ray_hit` trata colinear como PARALELO (`denom ≈ 0` ⇒ `None`),
//!    então as extensões se atravessavam sem "colidir" e o vão só fechava por acidente,
//!    quando o raio alcançava alguma parede DISTANTE (o "4× o vão" medido era isso — a
//!    quina do outro lado da caixa, a 2,5 do vão de 1,0). Com o par, **`reach` = o VÃO**
//!    no caso que o artista mede na tela — o que o rótulo do slider sempre prometeu.
//!    ⚠️ **O guard de direção é quem impede a hachura de virar tubo**: pontas lado a
//!    lado (traços paralelos) têm o vetor entre elas PERPENDICULAR às tangentes — cada
//!    uma tem de apontar PARA a outra (`d·(o₂−o₁) > 0` dos dois lados).
//!
//! **Corte por colisão:** uma extensão para onde encosta em outra linha (ou noutra
//! extensão). Sem isso, as extensões varam o desenho e cortam regiões ao meio. (O par
//! ponta-a-ponta não corta: ele liga dois pontos REAIS do desenho.)

use ph2d_core::Vec2;

/// Um segmento de fechamento: a linha invisível que tapa o vão.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Closure {
    pub a: Vec2,
    pub b: Vec2,
}

/// Uma polilinha de fronteira (um traço do desenho), como o solver a vê.
pub struct Boundary<'a> {
    pub points: &'a [Vec2],
    pub closed: bool,
}

/// Distância mínima (unidades do documento) para uma extensão valer a pena.
const MIN_EXTENSION: f32 = 1e-4;

/// **Gera os fechamentos** para os traços dados, com alcance `reach` (unidades do
/// documento — o slider "Gap Closure" do painel; `0` = desligado).
///
/// Cada extensão é cortada onde encosta em qualquer OUTRA linha (ou noutra extensão),
/// e só sobrevive se de fato colidiu: uma ponta que se estende no vazio não fecha vão
/// nenhum, e uma linha solta atravessando o desenho só faria mal.
#[must_use]
pub fn closures(strokes: &[Boundary<'_>], reach: f32) -> Vec<Closure> {
    if reach <= MIN_EXTENSION {
        return Vec::new();
    }
    // As candidatas: os raios (origem + direção), com o índice do traço-dono (para não
    // colidirem com a própria linha na origem). As PONTAS (não as quinas) também são as
    // candidatas ao pareamento ponta-a-ponta — anotamos onde elas terminam na lista.
    let mut rays: Vec<(usize, Vec2, Vec2)> = Vec::new();
    for (si, s) in strokes.iter().enumerate() {
        if s.closed || s.points.len() < 2 {
            continue; // cíclica nunca ganha extensão (é o GP; e ela já fecha)
        }
        let n = s.points.len();
        // Ponta inicial: da 2ª ponto para o 1º (para FORA).
        if let Some(d) = dir(s.points[1], s.points[0]) {
            rays.push((si, s.points[0], d));
        }
        // Ponta final.
        if let Some(d) = dir(s.points[n - 2], s.points[n - 1]) {
            rays.push((si, s.points[n - 1], d));
        }
    }
    let tip_count = rays.len();
    for (si, s) in strokes.iter().enumerate() {
        if s.closed || s.points.len() < 2 {
            continue;
        }
        let n = s.points.len();
        // **Quinas apertadas** (o "V"): onde a virada é mais fechada que ~60°, a
        // bissetriz EXTERNA vira um raio. É o que fecha o vértice de um "V" cujo bico
        // ficou de fora da região.
        for i in 1..n - 1 {
            let (Some(d0), Some(d1)) = (
                dir(s.points[i - 1], s.points[i]),
                dir(s.points[i], s.points[i + 1]),
            ) else {
                continue;
            };
            let cos = d0.x * d1.x + d0.y * d1.y;
            if cos > -0.5 {
                continue; // virada < 120°: não é uma quina apertada
            }
            // A bissetriz EXTERNA — a direção do "bico". O ângulo interno é a soma das
            // duas direções que SAEM da quina pelas pernas (`-d0` e `d1`); o bico é o
            // oposto disso, `d0 - d1`. (Trocar o sinal aponta o raio para DENTRO da
            // cunha, onde ele colide com a própria linha e não fecha vão nenhum.)
            if let Some(b) = dir(Vec2::new(0.0, 0.0), Vec2::new(d0.x - d1.x, d0.y - d1.y)) {
                rays.push((si, s.points[i], b));
            }
        }
    }

    // **Passe 1** — corta cada raio na 1ª colisão com as linhas ORIGINAIS. Fazer os dois
    // passes contra o estado original (e não contra o resultado parcial) deixa o
    // resultado independente da ordem dos traços — determinismo (HR-5).
    let cut_against_lines = |owner: usize, origin: Vec2, far: Vec2| -> f32 {
        let mut best = f32::INFINITY;
        for (si, s) in strokes.iter().enumerate() {
            let n = s.points.len();
            if n < 2 {
                continue;
            }
            let last = if s.closed { n } else { n - 1 };
            for i in 0..last {
                let (p, q) = (s.points[i], s.points[(i + 1) % n]);
                // A própria linha, perto da origem, não conta (senão toda ponta colide
                // consigo mesma no primeiro pixel).
                if si == owner && near(origin, p, q) {
                    continue;
                }
                if let Some(t) = ray_hit(origin, far, p, q)
                    && t > 1e-3
                    && t < best
                {
                    best = t;
                }
            }
        }
        best
    };

    // O raio inteiro de cada extensão, com o corte contra as linhas já aplicado.
    let stretched: Vec<(usize, Vec2, Vec2, f32)> = rays
        .iter()
        .map(|&(owner, origin, d)| {
            let far = Vec2::new(origin.x + d.x * reach, origin.y + d.y * reach);
            (owner, origin, far, cut_against_lines(owner, origin, far))
        })
        .collect();

    // **Passe 2 — extensão contra EXTENSÃO.** Sem ele, uma quina em "L" aberta cujas
    // duas pontas se cruzam NO AR (cada uma passa longe da *linha* da outra) não fechava
    // vão nenhum: `closures()` devolvia vazio e o balde vazava. É justamente a quina que
    // o GP fecha, e a razão de o Extend existir.
    let mut out = Vec::new();
    for (i, &(_, origin, far, best_lines)) in stretched.iter().enumerate() {
        let mut best = best_lines;
        for (j, &(_, o2, f2, b2)) in stretched.iter().enumerate() {
            if i == j {
                continue;
            }
            // A outra extensão, já cortada onde ela de fato termina.
            let end2 = if b2.is_finite() && b2 <= 1.0 {
                Vec2::new(o2.x + (f2.x - o2.x) * b2, o2.y + (f2.y - o2.y) * b2)
            } else {
                f2
            };
            if let Some(t) = ray_hit(origin, far, o2, end2)
                && t > 1e-3
                && t < best
            {
                best = t;
            }
        }
        // Só vale se COLIDIU dentro do alcance: uma extensão para o nada não fecha vão.
        if best.is_finite() && best <= 1.0 {
            out.push(Closure {
                a: origin,
                b: Vec2::new(
                    origin.x + (far.x - origin.x) * best,
                    origin.y + (far.y - origin.y) * best,
                ),
            });
        }
    }

    // **Passe 3 — pontas EMPARELHADAS** (a fonte 3 do doc, BUGS #23): duas pontas que se
    // apontam a `dist ≤ reach` fecham pela RETA entre elas — é o que torna o rótulo do
    // slider verdadeiro (`reach` = o vão que se mede na tela). Só PONTAS pareiam (as
    // quinas seguem por colisão); pontas coincidentes (quinas que se tocam) não geram
    // fechamento degenerado; e o guard de direção — cada uma apontando PARA a outra —
    // é o que separa um vão (frente a frente) de uma hachura (lado a lado: o vetor
    // entre as pontas é PERPENDICULAR às tangentes, e o par não fecha).
    for (i, &(_, o1, d1)) in rays.iter().take(tip_count).enumerate() {
        for &(_, o2, d2) in rays.iter().take(tip_count).skip(i + 1) {
            let v = o2 - o1;
            let dist2 = v.x * v.x + v.y * v.y;
            if dist2 > reach * reach || dist2 < MIN_EXTENSION * MIN_EXTENSION {
                continue;
            }
            if d1.x * v.x + d1.y * v.y <= 0.0 || d2.x * v.x + d2.y * v.y >= 0.0 {
                continue;
            }
            out.push(Closure { a: o1, b: o2 });
        }
    }
    out
}

/// Direção unitária `a → b`, ou `None` se coincidem (nunca `normalize(0)` = NaN — a
/// lição do ponto duplicado, `BUGS_flip.md` #4).
fn dir(a: Vec2, b: Vec2) -> Option<Vec2> {
    let d = b - a;
    let len = (d.x * d.x + d.y * d.y).sqrt();
    (len > 1e-6).then(|| Vec2::new(d.x / len, d.y / len))
}

/// O segmento `p→q` passa a menos de um cabelo da origem do raio? (a exclusão do
/// próprio segmento de origem).
fn near(o: Vec2, p: Vec2, q: Vec2) -> bool {
    let d = q - p;
    let len2 = d.x * d.x + d.y * d.y;
    let t = if len2 < 1e-9 {
        0.0
    } else {
        (((o.x - p.x) * d.x + (o.y - p.y) * d.y) / len2).clamp(0.0, 1.0)
    };
    let c = Vec2::new(p.x + t * d.x, p.y + t * d.y);
    let e = o - c;
    e.x * e.x + e.y * e.y < 1e-6
}

/// Interseção do segmento `o→f` com o segmento `p→q`. Devolve o `t ∈ [0,1]` ao longo
/// do raio, ou `None`. Puramente algébrico (sem transcendental — HR-5).
fn ray_hit(o: Vec2, f: Vec2, p: Vec2, q: Vec2) -> Option<f32> {
    let r = f - o;
    let s = q - p;
    let denom = r.x * s.y - r.y * s.x;
    if denom.abs() < 1e-9 {
        return None; // paralelos
    }
    let op = p - o;
    let t = (op.x * s.y - op.y * s.x) / denom;
    let u = (op.x * r.y - op.y * r.x) / denom;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(a: (f32, f32), b: (f32, f32)) -> Vec<Vec2> {
        vec![Vec2::new(a.0, a.1), Vec2::new(b.0, b.1)]
    }

    /// **O caso de uso:** duas linhas que quase se encontram. A ponta de uma se
    /// estende e ENCOSTA na outra — o vão fecha.
    #[test]
    fn a_tip_extends_until_it_hits_the_other_line() {
        // Horizontal em y=0 (de x=0 a x=10) e vertical em x=12 (de y=-5 a y=5).
        // A ponta direita da horizontal aponta para a vertical, a 2 unidades.
        let h = line((0.0, 0.0), (10.0, 0.0));
        let v = line((12.0, -5.0), (12.0, 5.0));
        let strokes = [
            Boundary {
                points: &h,
                closed: false,
            },
            Boundary {
                points: &v,
                closed: false,
            },
        ];
        let cl = closures(&strokes, 5.0);
        // A ponta da horizontal alcança a vertical em (12, 0).
        let hit = cl
            .iter()
            .find(|c| (c.a - Vec2::new(10.0, 0.0)).x.abs() < 1e-3)
            .expect("a ponta da horizontal tinha de fechar");
        assert!(
            (hit.b.x - 12.0).abs() < 1e-3 && hit.b.y.abs() < 1e-3,
            "o fechamento para EM CIMA da outra linha: {:?}",
            hit.b
        );
    }

    /// Um alcance curto demais não fecha nada — e, principalmente, **não deixa uma
    /// linha solta atravessando o desenho**: a extensão só sobrevive se COLIDIU.
    #[test]
    fn an_extension_that_hits_nothing_is_discarded() {
        let h = line((0.0, 0.0), (10.0, 0.0));
        let v = line((12.0, -5.0), (12.0, 5.0));
        let strokes = [
            Boundary {
                points: &h,
                closed: false,
            },
            Boundary {
                points: &v,
                closed: false,
            },
        ];
        let cl = closures(&strokes, 1.0); // alcance 1 < os 2 de distância
        assert!(
            cl.iter().all(|c| (c.a.x - 10.0).abs() > 1e-3),
            "a ponta não alcança: não pode virar linha solta"
        );
    }

    /// Alcance zero = feature desligada.
    #[test]
    fn zero_reach_is_off() {
        let h = line((0.0, 0.0), (10.0, 0.0));
        let strokes = [Boundary {
            points: &h,
            closed: false,
        }];
        assert!(closures(&strokes, 0.0).is_empty());
    }

    /// Uma curva FECHADA nunca ganha extensão (já fecha; estendê-la só cortaria a
    /// região ao meio).
    #[test]
    fn a_closed_stroke_gets_no_extensions() {
        let sq = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let strokes = [Boundary {
            points: &sq,
            closed: true,
        }];
        assert!(closures(&strokes, 5.0).is_empty());
    }

    /// **A quina em "V"**: as duas pernas se cruzam visualmente, mas o BICO fica de
    /// fora. A quina apertada gera um raio pela bissetriz externa — é o que fecha o
    /// vértice (e é a razão de o balde do GP fechar "V"s que outros não fecham).
    #[test]
    fn a_tight_corner_emits_a_ray_along_the_outer_bisector() {
        // Um "V" agudo: desce até (0,0) e volta a subir. A virada é de ~160°.
        let v = vec![
            Vec2::new(-1.0, 10.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 10.0),
        ];
        // Um teto em y=-2 para a extensão da quina colidir (senão ela é descartada).
        let roof = line((-5.0, -2.0), (5.0, -2.0));
        let strokes = [
            Boundary {
                points: &v,
                closed: false,
            },
            Boundary {
                points: &roof,
                closed: false,
            },
        ];
        let cl = closures(&strokes, 6.0);
        let from_corner = cl.iter().find(|c| c.a == Vec2::new(0.0, 0.0));
        let hit = from_corner.expect("a quina apertada tinha de emitir um raio");
        assert!(
            (hit.b.y + 2.0).abs() < 1e-3,
            "o raio da quina desce e encosta no teto: {:?}",
            hit.b
        );
    }

    /// 🔴 **O vão CANÔNICO — pontas colineares frente a frente — fecha com `reach` = o
    /// VÃO** (BUGS #23). É o traço feito em dois tempos: a mão levanta e volta na mesma
    /// linha. Antes do pareamento este caso era INVISÍVEL: `ray_hit` trata colinear como
    /// paralelo, as extensões se atravessavam sem "colidir", e o vão de 1,0 só fechava
    /// com reach 4,0 — quando o raio alcançava uma parede DISTANTE por acidente. Mutação
    /// que sangra: remover o passe 3 (o par não nasce e o vão volta a ser cego).
    #[test]
    fn facing_collinear_tips_close_at_the_reach_that_names_the_gap() {
        // Duas metades da mesma linha vertical, vão de 1,0 entre (2,-0.5) e (2,0.5).
        let a = line((2.0, -2.0), (2.0, -0.5));
        let b = line((2.0, 0.5), (2.0, 2.0));
        let strokes = [
            Boundary {
                points: &a,
                closed: false,
            },
            Boundary {
                points: &b,
                closed: false,
            },
        ];
        let cl = closures(&strokes, 1.0); // reach = o vão, exatamente
        assert!(
            cl.iter().any(|c| {
                let (lo, hi) = if c.a.y < c.b.y {
                    (c.a, c.b)
                } else {
                    (c.b, c.a)
                };
                (lo - Vec2::new(2.0, -0.5)).y.abs() < 1e-3
                    && (hi - Vec2::new(2.0, 0.5)).y.abs() < 1e-3
            }),
            "o par ponta-a-ponta tinha de fechar o vao colinear: {cl:?}"
        );
        // E abaixo do vão continua aberto (o slider não mente para o outro lado).
        assert!(
            closures(&strokes, 0.9).is_empty(),
            "reach menor que o vao nao pode fechar"
        );
    }

    /// 🔴 **Hachura não vira tubo**: pontas LADO A LADO (traços paralelos terminando
    /// juntos) não pareiam — o vetor entre elas é perpendicular às tangentes, e o guard
    /// de direção (cada ponta apontando PARA a outra) as recusa. Mutação que sangra:
    /// remover o guard (todo fim de hachura fecharia num pente selado).
    #[test]
    fn side_by_side_hatching_tips_do_not_pair() {
        // Três traços de hachura paralelos, terminando alinhados a 0,5 um do outro.
        let h1 = line((0.0, 0.0), (10.0, 0.0));
        let h2 = line((0.0, 0.5), (10.0, 0.5));
        let h3 = line((0.0, 1.0), (10.0, 1.0));
        let strokes = [
            Boundary {
                points: &h1,
                closed: false,
            },
            Boundary {
                points: &h2,
                closed: false,
            },
            Boundary {
                points: &h3,
                closed: false,
            },
        ];
        assert!(
            closures(&strokes, 2.0).is_empty(),
            "pontas lado a lado nao sao um vao — fechar aqui selaria a hachura"
        );
    }

    /// **Pontas COINCIDENTES (quinas que se tocam) não geram fechamento degenerado** —
    /// o desenho comum tem traços emendados ponta na ponta, e um `Closure` de
    /// comprimento zero por emenda seria ruído para todo consumidor.
    #[test]
    fn coincident_tips_do_not_pair() {
        let a = line((0.0, 0.0), (10.0, 0.0));
        let b = line((10.0, 0.0), (10.0, 10.0)); // começa ONDE a outra termina
        let strokes = [
            Boundary {
                points: &a,
                closed: false,
            },
            Boundary {
                points: &b,
                closed: false,
            },
        ];
        assert!(
            closures(&strokes, 5.0)
                .iter()
                .all(|c| (c.a - c.b).x.abs() > 1e-3 || (c.a - c.b).y.abs() > 1e-3),
            "emenda ponta-na-ponta nao e' vao"
        );
    }

    /// **Duas extensões que se cruzam NO AR fecham o vão.**
    ///
    /// Quina em "L" aberta: a ponta horizontal aponta para +x, a vertical para +y, e as
    /// duas se cruzam em (12, 0). Nenhuma das duas encosta na *linha* da outra — só na
    /// extensão dela. Cortando os raios apenas contra as linhas originais (o 1º corte),
    /// `closures` devolvia VAZIO e o balde vazava justamente na quina que o Extend
    /// existe para fechar.
    #[test]
    fn two_extensions_that_cross_in_mid_air_close_the_gap() {
        let a = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)]; // termina apontando +x
        let b = [Vec2::new(12.0, -10.0), Vec2::new(12.0, -2.0)]; // termina apontando +y
        let lines = [
            Boundary {
                points: &a,
                closed: false,
            },
            Boundary {
                points: &b,
                closed: false,
            },
        ];
        let cs = closures(&lines, 6.0);
        assert!(
            !cs.is_empty(),
            "as duas extensoes se cruzam em (12,0): tem de sair um fechamento"
        );
    }
}
