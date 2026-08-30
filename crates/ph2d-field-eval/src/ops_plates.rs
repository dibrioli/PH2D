//! ⭐⭐⭐ **AS CHAPAS** (W106) — contornos 2D puxados em Z, a família que tinha **UMA** entrada.
//!
//! # O que uma chapa é, neste módulo
//!
//! O molde já existia desde a W103: um contorno fechado no plano XY, fechado em Z pela laje do
//! [`crate::ops::slab_and_walls`], com o **aro** (a aresta entre a parede e a tampa) arredondado
//! pelo mesmo `round` de toda a casa. A [`crate::ops::sd_star`] é a primeira, e este arquivo é o
//! resto da família.
//!
//! ⚠️ **Cada uma responde à pergunta «porque não é composição?»** — e para várias delas a resposta
//! honesta é *«é, e isso não importa»*: o critério de entrada de uma paleta é **alcance**, não
//! expressividade. Uma cruz é duas caixas em união, e um artista que abre *Add Shape* à procura de
//! uma cruz não quer duas caixas: quer uma cruz, com uma **largura de braço** que é um número só.
//! *Uma forma que exige montagem é uma forma que não está no menu.*
//!
//! ⭐ E onde a composição deixaria **N** entidades na Hierarquia por uma forma que é **uma**, a
//! primitiva não é conveniência: é a diferença entre engrossar um braço mexendo num número e
//! mexendo em quatro que têm de concordar. Foi essa a razão da moldura de caixa (W103), e vale
//! igual aqui.
//!
//! # ⚠️ O que TODAS partilham, e que um leitor distraído desfaz
//!
//! 1. **A laje entra pela porta** ([`crate::ops::slab_and_walls`]) — nunca um `max` com `|z|−h`
//!    escrito à mão, senão o aro deixa de arredondar (W104: `offset(max(A,B), r)` é **inerte**).
//! 2. **As paredes chegam NORMALIZADAS** (`‖∇‖ = 1`), senão o `round` mede um raio que não é o que
//!    o artista pediu.
//! 3. **As quinas convexas fecham por [`intersection`] com [`Blended::Exact`]**, e as côncavas por
//!    [`union`] com o mesmo — o dual de De Morgan do mesmo arco.

use fidget::context::Tree;

use crate::ops::{Blended, intersection, length2, slab_and_walls, union};

/// Um disco de raio `r` centrado na origem do plano XY, já normalizado.
fn disco(r: f64) -> Tree {
    length2(&Tree::x(), &Tree::y()) - Tree::constant(r)
}

/// Um disco deslocado.
fn disco_em(cx: f64, cy: f64, r: f64) -> Tree {
    length2(
        &(Tree::x() - Tree::constant(cx)),
        &(Tree::y() - Tree::constant(cy)),
    ) - Tree::constant(r)
}

/// Um rectângulo 2D **com as quatro quinas arredondadas** em `r`.
///
/// ⚠️⚠️ **A receita é a da caixa, e é a ÚNICA que funciona aqui:** encolher uma distância **exacta**
/// e deslocá-la. A W104 mediu que `offset` sobre um `max` de semiespaços é **inerte** — dilatar um
/// semiespaço dá outro semiespaço, sem canto para arredondar. O [`rect`] é exacto, logo dilatá-lo
/// **é** o rectângulo de quinas redondas.
///
/// ⭐ **Existe porque a sonda de arestas o exigiu:** a cruz lia `4,7 %` da superfície sobre um vinco
/// de `88°` com o filete a metade do limite. As quinas verticais dos braços estavam **órfãs** — o
/// `slab_and_walls` arredonda o **aro** (parede↔tampa) e não as arestas do contorno. É a mesma
/// pedra que a W104 apanhou no prisma: *uma divisão de responsabilidade copiada de outra forma é uma
/// aresta órfã quando o segundo dono não existe* (no `Extrude` o dono é o editor vetorial; aqui não
/// há nenhum).
fn rect_round(hx: f64, hy: f64, r: f64) -> Tree {
    let r = r.min(hx * 0.999).min(hy * 0.999).max(0.0);
    crate::ops::offset(&rect(hx - r, hy - r), r)
}

/// Um rectângulo 2D de meias-extensões `(hx, hy)` — distância **exacta**, dentro e fora.
fn rect(hx: f64, hy: f64) -> Tree {
    let dx = Tree::x().abs() - Tree::constant(hx);
    let dy = Tree::y().abs() - Tree::constant(hy);
    let fora = length2(&dx.max(0.0), &dy.max(0.0));
    let dentro = dx.max(dy).min(0.0);
    fora + dentro
}

/// ⭐⭐⭐ **ENGRENAGEM** — `teeth` dentes, e foi a forma que o Enio nomeou.
///
/// ⛔⛔ **Ela tinha sido CORTADA da fila** com o argumento *«é um dente mais o modificador radial,
/// que já existe»* — e essa frase responde *«o motor consegue?»*, não *«a pessoa acha?»*. Quem abre
/// a paleta à procura de uma engrenagem não vai modelar um dente. ⇒ é **uma** forma, com um número
/// de dentes, exactamente como a estrela tem um número de pontas.
///
/// `root` é o raio do corpo, `outer` a ponta do dente, `tooth` a fracção do passo que o dente ocupa
/// (`0` = sem dente, `1` = dente encostado no vizinho).
///
/// ⭐ **A construção é a da estrela, com um dente TRAPEZOIDAL em vez de uma pipa**: o corpo (um
/// disco de raio `root`) unido a `teeth` dentes, cada um a interseção de três meias-fatias — as
/// duas flancos e a ponta. ⚠️ **O corpo TEM de sobrepor o dente**, senão a união toca sem se
/// cruzar e nasce uma superfície fantasma no interior (a lição que a estrela pagou na W104-bis).
///
/// ⚠️ **O flanco é RECTO, não uma evolvente.** Uma engrenagem que engrena de verdade tem perfil
/// evolvente, e ele **não** é uma distância fechada — seria classe C. Para desenhar, um flanco
/// recto é o que toda a biblioteca de formas usa; para transmitir binário, não é. *A cerca fica
/// nomeada em vez de a forma prometer o que não faz.*
pub fn sd_gear(
    teeth: u32,
    root: f64,
    outer: f64,
    tooth: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let n = teeth.max(3);
    let passo = std::f64::consts::TAU / f64::from(n);
    // Meia-abertura angular do dente na base.
    let meia = passo * 0.5 * tooth.clamp(0.05, 0.95);
    let corpo = disco(root);
    let mut dentes: Option<Tree> = None;
    for k in 0..n {
        let phi = passo * f64::from(k);
        // Os dois flancos, como meias-fatias normalizadas que passam pela origem: manter o lado
        // interno do sector `[phi−meia, phi+meia]`.
        let (a1, a2) = (phi - meia, phi + meia);
        let f1 = Tree::x() * Tree::constant(a1.sin()) - Tree::y() * Tree::constant(a1.cos());
        let f2 = Tree::x() * Tree::constant(-a2.sin()) + Tree::y() * Tree::constant(a2.cos());
        // A ponta: o semiplano perpendicular à direcção do dente, a `outer` do centro.
        let ponta = Tree::x() * Tree::constant(phi.cos()) + Tree::y() * Tree::constant(phi.sin())
            - Tree::constant(outer);
        let dente = intersection(
            &intersection(&f1, &f2, Blended::Exact(0.0)),
            &ponta,
            Blended::Exact(round),
        );
        dentes = Some(dentes.map_or_else(
            || dente.clone(),
            |w: Tree| union(&w, &dente, Blended::Exact(round)),
        ));
    }
    let dentes = dentes.unwrap_or_else(|| Tree::constant(0.0));
    // ⚠️ O corpo entra por `union` ARREDONDADA: é ali que nasce o vale entre dois dentes, e ele é
    // uma quina **côncava** — a que o artista mais vê numa engrenagem.
    slab_and_walls(
        &union(&corpo, &dentes, Blended::Exact(round)),
        half_height,
        crate::ops_joint::Edge { round, chamfer },
    )
}

/// ⭐ **CRUZ / mais** — `arm` é o meio-comprimento do braço e `width` a meia-largura.
///
/// ⚠️ **É composição — duas caixas — e entra na mesma.** O que a primitiva compra é o **número**: a
/// largura do braço é um valor, e com duas caixas são dois que têm de concordar. E as quatro quinas
/// **côncavas** arredondam de uma vez, com o mesmo `round` das convexas.
pub fn sd_cross(arm: f64, width: f64, half_height: f64, round: f64, chamfer: f64) -> Tree {
    // ⚠️ **Os braços chegam JÁ arredondados** — ver [`rect_round`]: as oito quinas verticais deles
    // não são aro, e o `slab_and_walls` não lhes toca.
    let horizontal = rect_round(arm, width, round);
    let vertical = rect_round(width, arm, round);
    slab_and_walls(
        &union(&horizontal, &vertical, Blended::Exact(round)),
        half_height,
        crate::ops_joint::Edge { round, chamfer },
    )
}

/// ⭐⭐ **CORAÇÃO** — `size` é o meio-lado do losango que forma a ponta de baixo.
///
/// ⭐ **A construção clássica e exacta:** um quadrado rodado 45° com dois semicírculos assentes nas
/// duas arestas de cima. Cada peça é distância exacta e a união de convexos por `min` é exacta no
/// exterior ⇒ o campo é bom sem uma aproximação.
///
/// ⚠️ **A cova entre os dois lóbulos é uma quina CÔNCAVA**, e é ela que dá o carácter da forma — por
/// isso a união é arredondada e não crua.
pub fn sd_heart(size: f64, half_height: f64, round: f64, chamfer: f64) -> Tree {
    // ⚠️⚠️ **O losango é um QUADRADO RODADO, e não um `|x|+|y|` dobrado** — e a diferença é o
    // filete. A forma dobrada é um `max` de quatro semiespaços, e a W104 mediu que dilatar isso é
    // **inerte**: as quatro quinas ficavam vivas (a sonda leu `2,8 %` da superfície sobre um vinco
    // de `86°`). Num referencial rodado 45° a mesma região é um **rectângulo exacto**, e aí a
    // receita da caixa funciona.
    //
    // ⭐ A rotação é uma isometria, então a distância no referencial rodado É a distância.
    let inv = 1.0 / 2.0_f64.sqrt();
    let (rx, ry) = (
        (Tree::x() + Tree::y()) * Tree::constant(inv),
        (Tree::x() - Tree::y()) * Tree::constant(inv),
    );
    let lado = size * inv;
    let r = round.min(lado * 0.5);
    let losango = crate::ops::offset(
        &{
            let dx = rx.abs() - Tree::constant(lado - r);
            let dy = ry.abs() - Tree::constant(lado - r);
            length2(&dx.max(0.0), &dy.max(0.0)) + dx.max(dy).min(0.0)
        },
        r,
    );
    // ⭐ A aresta de cima do losango vai de `(−s, 0)` a `(0, s)`: mede `s·√2`, logo o
    // semicírculo assente nela tem raio `s/√2` e centro no ponto médio `(±s/2, s/2)`.
    let r = size / 2.0_f64.sqrt();
    let c = size * 0.5;
    let esquerdo = disco_em(-c, c, r);
    let direito = disco_em(c, c, r);
    let corpo = union(
        &union(&losango, &esquerdo, Blended::Exact(0.0)),
        &direito,
        Blended::Exact(round),
    );
    slab_and_walls(
        &corpo,
        half_height,
        crate::ops_joint::Edge { round, chamfer },
    )
}

/// ⭐ **LUA / crescente** — o disco `radius` menos um disco `bite` deslocado de `offset` em `+X`.
///
/// ⚠️ **É composição — e o que a primitiva compra é o GESTO.** Com dois objectos, mudar a espessura
/// do crescente exige mover um e redimensionar o outro em concordância; aqui é **um** número.
///
/// ⚠️ As duas pontas do crescente são quinas agudas onde os dois círculos se cruzam, e o `round`
/// alcança-as: a subtracção arredondada é o dual do mesmo arco.
pub fn sd_moon(
    radius: f64,
    bite: f64,
    offset: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let cheio = disco(radius);
    let mordida = disco_em(offset, 0.0, bite);
    let crescente = intersection(&cheio, &crate::ops::neg(&mordida), Blended::Exact(round));
    slab_and_walls(
        &crescente,
        half_height,
        crate::ops_joint::Edge { round, chamfer },
    )
}

/// ⭐ **GOTA / ovo** — um disco de raio `radius` e uma ponta a `height` acima dele.
///
/// ⭐ **Um disco unido a um triângulo TANGENTE.** ⚠️ Se o triângulo apenas tocasse o disco, a união
/// teria a costura fantasma que a estrela pagou; aqui os flancos são as **tangentes** ao disco a
/// partir da ponta, e por isso a junção é lisa **por geometria** e não por mistura.
pub fn sd_drop(radius: f64, height: f64, half_height: f64, round: f64, chamfer: f64) -> Tree {
    let bolha = disco(radius);
    let h = height.max(radius * 1.01);
    // As tangentes ao círculo a partir de `(0, h)`: o ângulo entre a tangente e o eixo é `asin(r/h)`.
    let s = (radius / h).clamp(0.0, 0.999);
    let c = (1.0 - s * s).sqrt();
    // ⚠️⚠️ **O SINAL do termo axial** — a 1.ª escrita punha `−s·(y−h)` nas duas e o gate apanhou-a: o
    // ápice ficava **na** superfície e o interior do cone **fora** dela, ou seja o cone abria para
    // CIMA. A normal exterior da tangente esquerda é `(−c, s)`, e a verificação é numa linha: a
    // distância dela à origem é `|s·(0−h)| = s·h = r` ⇒ ela **é** tangente ao círculo.
    let esquerda =
        Tree::x() * Tree::constant(-c) + (Tree::y() - Tree::constant(h)) * Tree::constant(s);
    let direita =
        Tree::x() * Tree::constant(c) + (Tree::y() - Tree::constant(h)) * Tree::constant(s);
    let cone = intersection(&esquerda, &direita, Blended::Exact(0.0));
    // ⛔⛔ **E O CONE TEM DE SER CORTADO EM BAIXO.** Ele é infinito para lá do ápice, e uma união
    // crua com a bolha acrescentaria à peça uma cunha que desce para sempre — a gota ficaria com um
    // rabo infinito, que **nenhum gate de silhueta no plano do equador veria**.
    //
    // ⭐ O corte certo é a altura de TANGÊNCIA, `y = r·s = r²/h`: ali a meia-largura do cone e a da
    // bolha são **exactamente iguais** (é o que «tangente» quer dizer), então as duas peças
    // encontram-se sem degrau; abaixo dela a bolha é mais larga e manda; acima, o cone. ⇒ as duas
    // **sobrepõem-se** em vez de se tocarem, e não há a superfície fantasma que a estrela pagou.
    let corte = Tree::constant(radius * s) - Tree::y();
    let bico = intersection(&cone, &corte, Blended::Exact(0.0));
    // ⚠️ A união é CRUA de propósito: as tangentes encontram o círculo **sem quina**, e arredondar
    // ali abriria um sulco onde não há aresta.
    slab_and_walls(
        &bolha.min(bico),
        half_height,
        crate::ops_joint::Edge { round, chamfer },
    )
}

/// ⭐ **FATIA / sector de disco** — `radius` e a meia-abertura `angle`, centrada em `+Y`.
///
/// ⚠️ **A ramificação vive em RUST**, como no [`crate::ops::sd_torus_arc`]: até `π` o sector é a
/// **interseção** dos dois semiplanos, acima é a **união** deles. Uma ramificação dentro do campo
/// seria uma segunda forma escondida na primeira.
pub fn sd_pie(radius: f64, angle: f64, half_height: f64, round: f64, chamfer: f64) -> Tree {
    let d = disco(radius);
    let a = angle.clamp(0.01, std::f64::consts::PI - 0.01);
    // Os dois semiplanos que limitam o sector, com a bissectriz em `+Y`.
    let e1 = Tree::x() * Tree::constant(a.cos()) - Tree::y() * Tree::constant(a.sin());
    let e2 = Tree::x() * Tree::constant(-a.cos()) - Tree::y() * Tree::constant(a.sin());
    // ⚠️⚠️ **O ÁPICE CORTA A SECO** — e o censo do módulo é que o disse: com ele arredondado o
    // campo subia a `1,50` por unidade contra o `1,41` que a marcha de um nível de inflação
    // comporta, e o traçador **atravessaria a superfície**.
    //
    // ⭐ A causa é o encaixe: três interseções arredondadas uma dentro da outra (os dois
    // semiplanos · o disco · a laje) compõem a inflação de cada uma. É a mesma lei que a
    // [`crate::ops::sd_star`] já tinha escrito para o corte de sector dela, e por uma razão de
    // desenho igualmente boa: num ângulo apertado o ápice é um **ponto**, e um arco ali não é o que
    // se vê — o que se vê são as duas quinas onde a face plana encontra o arco, e essas ficam
    // arredondadas.
    let sector = if a <= std::f64::consts::FRAC_PI_2 {
        e1.max(e2)
    } else {
        e1.min(e2)
    };
    slab_and_walls(
        &intersection(&d, &sector, Blended::Exact(round)),
        half_height,
        crate::ops_joint::Edge { round, chamfer },
    )
}

/// ⭐ **TRAPÉZIO** — `bottom` e `top` são as meias-larguras das duas bases, `half_width` a meia-altura.
///
/// ⚠️ **Não é o prisma de 4 lados estreitado**: aquele estreita nos **dois** eixos (é uma pirâmide
/// truncada), e um trapézio estreita **num** — a secção continua a ser um rectângulo em Z.
pub fn sd_trapezoid(
    bottom: f64,
    top: f64,
    half_width: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    // Os dois flancos inclinados, normalizados, mais as duas bases.
    let m = (top - bottom) / (2.0 * half_width);
    let norm = 1.0 / (1.0 + m * m).sqrt();
    let a = (bottom + top) * 0.5;
    let flanco = (Tree::x().abs() - Tree::constant(a) - Tree::y() * Tree::constant(m))
        * Tree::constant(norm);
    let bases = Tree::y().abs() - Tree::constant(half_width);
    slab_and_walls(
        &intersection(&flanco, &bases, Blended::Exact(round)),
        half_height,
        crate::ops_joint::Edge { round, chamfer },
    )
}

/// ⭐ **VESICA / lente** — a interseção de dois discos de raio `radius` afastados de `2·offset`.
///
/// As duas pontas são quinas agudas, e é delas que a forma vive.
pub fn sd_vesica(radius: f64, offset: f64, half_height: f64, round: f64, chamfer: f64) -> Tree {
    let a = disco_em(-offset, 0.0, radius);
    let b = disco_em(offset, 0.0, radius);
    slab_and_walls(
        &intersection(&a, &b, Blended::Exact(round)),
        half_height,
        crate::ops_joint::Edge { round, chamfer },
    )
}

#[cfg(test)]
#[path = "ops_plates_tests.rs"]
mod tests;
