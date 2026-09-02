//! ⭐⭐⭐ **OS SÓLIDOS QUE UM CATÁLOGO VETORIAL NÃO PODIA TER** (W107) — as formas que só existem em
//! três dimensões, e que a fila deste módulo **nunca chegou a contar**.
//!
//! # Por que este arquivo existe, e o que ele corrige
//!
//! A auditoria de 29/08 ([doc 08 §4](../../../docs/3DModeling/08_formas_por_formula.md)) leu as
//! **47 formas do catálogo vetorial** e concluiu, com razão, que quase todas já se exprimem por
//! composição. O §5 do mesmo doc declarou então *«a fila fechou»*.
//!
//! ⛔⛔⛔ **Ela fechou contra a lista errada.** O §2.4 daquele documento tem uma **segunda** lista —
//! *«as 3D que o catálogo vetorial nem podia ter»*, quinze formas, todas classe **A** — e ela nunca
//! foi auditada: a §4 respondeu *«quantas das 47»*, o §5 fechou contra a resposta da §4, e as 3D
//! ficaram por contar. *Uma auditoria responde a lista que leu; fechar uma fila contra ela exige
//! provar que era a fila toda.*
//!
//! ⚠️ **E o argumento que cortou as outras é do tipo errado para um MENU.** *«Já se faz por
//! composição»* responde *«o motor consegue exprimir?»*; a pergunta de uma paleta é *«a pessoa
//! ACHA?»*. Quem abre *Add Shape* à procura de uma engrenagem não quer descobrir que precisa de
//! modelar um dente e encontrar o modificador radial. ⭐ **A lei já estava escrita neste módulo** —
//! o gate `field3d_reach_tests` afirma que *o painel oferece exactamente o que o gesto faz* — e a
//! auditoria passou ao lado dela. E o critério de entrada estava escrito no §3.1: *«quantas vezes o
//! artista as quer, e **não o preço**»*, logo antes de quarenta serem cortadas pelo preço.
//!
//! # A fonte, e as três coisas que mudam em relação a ela
//!
//! As *3D distance functions* de Inigo Quilez — a mesma que o [`crate::ops`] já cita para a caixa,
//! a esfera e o toro. **Porte, não invenção.**
//!
//! 1. ⚠️ Toda raiz passa pelo [`crate::ops::safe_sqrt`]: uma que não passe reintroduz `NaN` no
//!    gradiente, e o sintoma aparece na malha três camadas abaixo como uma quina serrilhada.
//! 2. ⚠️ **Nenhuma ramifica sobre a POSIÇÃO.** A referência escreve `if (3*p.x < m) … else …`; um
//!    `if` em Rust sobre uma coordenada seria uma segunda forma escondida dentro da primeira,
//!    porque o campo aqui é uma **árvore** avaliada em lote. Onde a referência ramifica, este
//!    arquivo ou reescreve sem ramo (com prova), ou adopta a construção da casa (interseção de
//!    meias-fatias, como o [`crate::ops::sd_prism`]).
//! 3. ⚠️ **O `round` segue a receita da casa** — e **só onde a fonte é distância EXATA**. A W104
//!    mediu que `offset(max(A,B), r)` sobre semiespaços é **inerte**: dilatar um semiespaço dá
//!    outro semiespaço, sem canto para arredondar. Por isso as arestas destas formas fecham por
//!    [`intersection`] com [`Blended::Exact`], que é o operador que de facto arredonda.

use fidget::context::Tree;

use crate::ops::{length2, length3};

/// ⭐⭐ **OCTAEDRO regular** — `radius` é a distância do centro a um VÉRTICE.
///
/// ⚠️ **Circunraio e não apótema**, pela mesma razão que o [`crate::ops::sd_prism`]: assim o
/// octaedro **inscreve-se** na esfera do mesmo raio, e trocar um pelo outro nunca faz a peça
/// crescer.
///
/// ⭐ **Construído como o prisma: a interseção das oito faces**, fechadas duas a duas por
/// [`Blended::Exact`]. ⛔ **A fórmula publicada foi lida e NÃO portada**, e o motivo é o ponto 2 do
/// cabeçalho: ela ramifica sobre qual coordenada domina (`if 3*p.x < m`), e a alternativa sem ramo
/// não existe — o `min` das três projecções **sobrestima** na região da face (cada uma é a distância
/// a um ponto do bordo, logo um limite superior), e o `min` com o plano **subestima** na região da
/// aresta. *Duas combinações erradas em direcções opostas não fazem uma certa.*
///
/// ⇒ fica a construção da casa: exacta dentro, subestimador junto às quinas de fora, `‖∇f‖ ≤ 1` por
/// definição — **a mesma troca que o prisma e o cone já fazem**, e o `round` funciona nela.
///
/// ⚠️ As normais de duas faces vizinhas fazem `70,53°` (`cos ψ = 1/3`), logo o diedro interno é
/// **obtuso** (`109,5°`) — e desde a W107 o operador arredonda-o com o arco VERDADEIRO, que num
/// obtuso é *menor* do que a lei ortogonal entregava. Ver [`crate::ops::union_round_at`].
pub fn sd_octahedron(radius: f64, round: f64, chamfer: f64) -> Tree {
    // A face é `x + y + z = radius` no octante positivo; normalizada, `(x+y+z−r)/√3`.
    let r = radius;
    let inv = 1.0 / 3.0_f64.sqrt();
    // ⭐ **As normais de duas faces vizinhas fazem `cos ψ = 1/3`** — e é esse o número que o
    // operador quer, sem passar por ângulo nenhum (ver [`crate::ops::union_round_at`]).
    const COS_FACES: f64 = 1.0 / 3.0;
    let sinais: Vec<[f64; 3]> = [1.0_f64, -1.0]
        .iter()
        .flat_map(|&sx| {
            [1.0_f64, -1.0]
                .iter()
                .flat_map(move |&sy| [1.0_f64, -1.0].iter().map(move |&sz| [sx, sy, sz]))
        })
        .collect();
    let face = |s: [f64; 3]| {
        (Tree::x() * Tree::constant(s[0])
            + Tree::y() * Tree::constant(s[1])
            + Tree::z() * Tree::constant(s[2])
            - Tree::constant(r))
            * Tree::constant(inv)
    };
    if chamfer <= 0.0 {
        // ⭐ **O caminho de sempre, ao bit** — ver a nota da assimetria abaixo.
        let mut faces: Option<Tree> = None;
        for &s in &sinais {
            let f = face(s);
            faces = Some(faces.map_or_else(
                || f.clone(),
                // ⚠️ **O ÂNGULO vale para o filete e não para o chanfro**, e a assimetria é
                // geométrica: o operador de filete media a distância com Pitágoras e por isso
                // precisava de saber o ângulo (`ops::union_round_at`), enquanto o plano do
                // chanfro **é** exacto — ele recua `c` em cada face seja qual for o ângulo.
                |w: Tree| {
                    crate::ops_joint::intersection_joint(
                        &w,
                        &f,
                        crate::ops_joint::Edge::at(round, chamfer, COS_FACES),
                    )
                },
            ));
        }
        return faces.unwrap_or_else(|| Tree::constant(0.0));
    }
    // ⭐⭐⭐ **AS OITO FACES E AS DOZE ARESTAS NUMA MISTURA SÓ** — ver
    // [`crate::ops_joint::intersection_joint_n`]. Dobrar as faces duas a duas fazia cada junta
    // receber a composta das anteriores, e a costura dela aflorava na aresta seguinte: medido
    // `19,9°` de giro da normal contra `2,3°` só com filete.
    //
    // ⚠️ **Duas faces de um octaedro partilham uma aresta quando os sinais diferem em EXACTAMENTE
    // uma componente** — `8 × 3 / 2 = 12`, que é o número de arestas que a forma tem. Uma lista
    // escrita à mão aqui seria a segunda resposta a *«quais são as arestas»*.
    let corpo: Vec<Tree> = sinais.iter().map(|&s| face(s)).collect();
    let mut arestas: Vec<(Tree, Tree)> = Vec::new();
    for (i, a) in sinais.iter().enumerate() {
        for (j, b) in sinais.iter().enumerate().skip(i + 1) {
            let difere = (0..3).filter(|&k| (a[k] - b[k]).abs() > 0.5).count();
            if difere == 1 {
                arestas.push((corpo[i].clone(), corpo[j].clone()));
            }
        }
    }
    debug_assert_eq!(arestas.len(), 12, "um octaedro tem doze arestas");
    // ⚠️ O ângulo viaja, e a mistura n-ária ainda NÃO o lê — ver `ops_joint::intersection_joint`.
    crate::ops_joint::intersection_joint_n(
        &corpo,
        &arestas,
        crate::ops_joint::Edge::at(round, chamfer, COS_FACES),
    )
}

/// ⭐⭐⭐ **CONE DE PONTAS ARREDONDADAS** — o casco convexo de duas esferas, e a forma mais útil
/// deste lote para quem modela algo vivo: um membro, um dedo, um chifre, um tronco.
///
/// Raio `bottom` em `z = −half_height`, `top` em `z = +half_height`. ⚠️ **Não é o
/// [`crate::ops::sd_cone`] com filete**: ali as tampas são planas e o aro é que arredonda; aqui
/// **não há tampa** — a superfície fecha nas duas calotas e a parede é **tangente** às duas esferas.
///
/// ⭐ **Com `bottom == top` degenera na cápsula, ao bit** — o `b` abaixo fica zero e a expressão
/// vira a distância ao segmento. É a mesma família do par cone/tronco: uma fórmula, dois defaults.
///
/// ⚠️ **Sem `round`, e a ausência é a forma** — como na [`crate::ops::sd_capsule`] e na esfera: ela
/// já é toda arco, e um segundo raio não teria onde agir.
///
/// # ⭐⭐ A escrita SEM RAMO, e por que ela é exacta
///
/// A referência ramifica em três regimes (calota de baixo · parede · calota de cima). Aqui a forma
/// é escrita como o **mínimo sobre a esfera que desliza**:
///
/// `d(p) = min ₜ∈[0,1] ( ‖p − c(t)‖ − r(t) )`, com `c` e `r` a interpolar linearmente.
///
/// ⭐ O objectivo é **convexo em `t`** (uma norma composta com uma afim, menos uma afim), então o
/// mínimo com `t` preso a `[0,1]` é o mínimo livre **grampeado** — e o mínimo livre resolve-se em
/// fechado: `t* = (a·z + b·ρ)/(a·H)`. ⇒ um `clamp` substitui os três ramos, e a igualdade é exacta,
/// não uma aproximação.
///
/// ⚠️ **`|bottom − top| < 2·half_height` é obrigatório** — acima disso uma esfera contém a outra, a
/// tangente comum não existe (`a = 0`) e a divisão explode. O documento recusa antes de chegar aqui.
pub fn sd_round_cone(bottom: f64, top: f64, half_height: f64) -> Tree {
    let h = 2.0 * half_height;
    let b = (bottom - top) / h;
    let a = (1.0 - b * b).max(1.0e-9).sqrt();
    // Eixo em Z com a esfera `bottom` na origem local.
    let z = Tree::z() + Tree::constant(half_height);
    let rho = length2(&Tree::x(), &Tree::y());
    // `t*` livre, já em [0,1] depois do grampo.
    //
    // ⚠️⚠️ **O SINAL do termo radial** — a 1.ª escrita somava-o, e o gate apanhou-a: no equador da
    // esfera de baixo (um ponto que está na superfície por construção, porque a esfera inteira
    // pertence ao casco) ela devolvia **`+0,027`** em vez de zero. *Um mínimo não pode valer mais do
    // que uma amostra do próprio objectivo* — foi essa desigualdade que nomeou o defeito.
    //
    // A conta: com `u = z − tH`, anular a derivada dá `u/√(ρ²+u²) = b`, logo `u = bρ/a` e
    // `t = (z − bρ/a)/H` ⇒ o termo radial **subtrai**.
    let k = z.clone() * Tree::constant(a) - rho.clone() * Tree::constant(b);
    let t = (k * Tree::constant(1.0 / (a * h))).max(0.0).min(1.0);
    let centro_z = t.clone() * Tree::constant(h);
    let raio = Tree::constant(bottom) + t * Tree::constant(top - bottom);
    length2(&rho, &(z - centro_z)) - raio
}

/// ⭐⭐ **ESFERA CORTADA por um plano** — uma cúpula, um botão, uma bolha assente numa mesa.
///
/// `radius` é a esfera; `cut` é a altura do corte em Z. `cut = 0` é a meia-esfera, `cut = −radius`
/// é a esfera inteira, e o documento recusa `cut >= radius` (não sobraria peça).
///
/// ⭐ **As duas fontes são distância EXATA** — a esfera e o semiespaço —, e é por isso que o
/// `round` funciona aqui pela porta da casa: *a receita nunca foi «encolher e deslocar», era
/// «encolher uma distância EXATA e deslocar»* (W104).
///
/// ⚠️ **A fórmula publicada tem um bordo de raio fixo; esta tem um KNOB.** Ela resolve a aresta com
/// `length(q − bordo)`, que é um arco de raio zero; aqui a aresta é a única viva da forma e o
/// artista escolhe o raio dela.
pub fn sd_cut_sphere(radius: f64, cut: f64, round: f64, chamfer: f64) -> Tree {
    let esfera = length3(&Tree::x(), &Tree::y(), &Tree::z()) - Tree::constant(radius);
    let plano = Tree::z() - Tree::constant(cut);
    crate::ops_joint::intersection_joint(
        &esfera,
        &plano,
        crate::ops_joint::Edge::square(round, chamfer),
    )
}

/// ⭐⭐ **CÚPULA OCA** — uma tigela, um capacete, uma antena.
///
/// A casca esférica de raio médio `radius` e espessura `thickness`, cortada em `z = cut`.
///
/// ⚠️ **Não é a [`sd_cut_sphere`] menos outra esfera** — seria, e daria **duas** entidades na
/// Hierarquia para uma forma que é uma, com o artista a mexer em dois raios para engrossar uma
/// parede. É a mesma razão que fez a moldura de caixa ser primitiva (W103): *compor é a resposta
/// certa quando a composição é o que o artista pensa; aqui ele pensa «tigela».*
///
/// ⭐ **A casca é distância EXATA** (`| ‖p‖ − r | − t/2`), e o plano também ⇒ os dois bordos
/// arredondam pela porta da casa.
///
/// ⛔ **A fórmula publicada foi lida e NÃO portada.** Ela ramifica sobre a posição
/// (`h*q.x < w*q.y ? … : …`) e o `min` dos dois ramos **não** a reproduz: num ponto alto sobre o
/// eixo, o ramo do bordo dá `√(w² + (z−h)²)` e o da casca dá `|z − r|`, e como `h < r` o **segundo é
/// menor** — o `min` escolheria o ramo errado exactamente onde a referência escolhe o outro.
/// *Verifiquei num ponto antes de acreditar na simplificação.*
pub fn sd_cut_hollow_sphere(
    radius: f64,
    cut: f64,
    thickness: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let casca = (length3(&Tree::x(), &Tree::y(), &Tree::z()) - Tree::constant(radius)).abs()
        - Tree::constant(thickness * 0.5);
    let plano = Tree::z() - Tree::constant(cut);
    crate::ops_joint::intersection_joint(
        &casca,
        &plano,
        crate::ops_joint::Edge::square(round, chamfer),
    )
}

/// ⭐⭐⭐ **ELO DE CORRENTE** — a forma que nada neste catálogo exprime, e a que mais se nota quando
/// falta: uma corrente, um mosquetão, uma argola.
///
/// Um toro **esticado**: o círculo do eixo vira um estádio — duas semicircunferências de raio
/// `major` ligadas por dois segmentos rectos de comprimento `2·length` —, engrossado em `minor`.
///
/// ⚠️ **Não é composição.** Duas meias-argolas mais dois cilindros dão a silhueta e deixam **quatro**
/// objectos na Hierarquia, duas costuras de campo onde as peças se encontram, e uma espessura que
/// passa a ser quatro números que têm de concordar.
///
/// ⭐ **É exacta, e a ideia é a da cápsula:** alongar o eixo é subtrair a `y` o seu próprio valor
/// preso ao intervalo — a mesma manobra que transforma uma esfera em cápsula transforma um toro em
/// elo. Sem `round`: como o toro, ela já é toda arco.
pub fn sd_link(major: f64, minor: f64, length: f64) -> Tree {
    let y = Tree::y().abs() - Tree::constant(length);
    let q = length2(&Tree::x(), &y.max(0.0)) - Tree::constant(major);
    length2(&q, &Tree::z()) - Tree::constant(minor)
}

/// ⭐ **ÂNGULO SÓLIDO** — a fatia cónica de uma esfera: um farol, um cone de visão, um gomo.
///
/// `radius` é a esfera e `angle` a meia-abertura em radianos medida do eixo `+Z`.
///
/// ⚠️ **Não é o [`crate::ops::sd_cone`] intersectado com uma esfera** enquanto ÁRVORE de duas
/// entidades: isso dá a silhueta certa, duas linhas na Hierarquia e dois números que têm de
/// concordar. Aqui é **uma** folha, e a aresta circular entre a calota e a parede é arredondável.
///
/// ⭐ **A parede do cone é um SEMIESPAÇO normalizado**, não uma distância a construir: no plano
/// meridiano, `ρ·cos θ − z·sin θ` tem gradiente `(cos θ, −sin θ)`, de norma **1** — logo é a
/// distância exacta ao cone infinito, com o sinal certo (negativa dentro). ⇒ a forma inteira é
/// `esfera ∩ cone`, pela mesma porta que fecha o prisma.
///
/// ⛔ **A projecção com o pé PRESO ao arco (a da referência) foi escrita e descartada**: ela dá a
/// distância exacta à aresta, e `length` é sempre positiva — faltava-lhe o **sinal**, que só voltava
/// com um `compare` sobre a posição. *Uma expressão exacta sem sinal não é um campo com sinal.*
pub fn sd_solid_angle(radius: f64, angle: f64, round: f64, chamfer: f64) -> Tree {
    let (s, c) = (angle.sin(), angle.cos());
    let rho = length2(&Tree::x(), &Tree::y());
    let esfera = length3(&Tree::x(), &Tree::y(), &Tree::z()) - Tree::constant(radius);
    let cone = rho * Tree::constant(c) - Tree::z() * Tree::constant(s);
    crate::ops_joint::intersection_joint(
        &esfera,
        &cone,
        crate::ops_joint::Edge::square(round, chamfer),
    )
}

#[cfg(test)]
#[path = "ops_solids_tests.rs"]
mod tests;
