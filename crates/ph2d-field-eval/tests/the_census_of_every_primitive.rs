//! ⭐⭐⭐ **O CENSO DE TODA PRIMITIVA** (W101) — quatro perguntas, uma lista, **derivada**.
//!
//! | pergunta | gate |
//! |---|---|
//! | o campo ainda é uma distância? | `every_primitive_honours_the_march` |
//! | a caixa do mundo contém a peça? | `the_bounding_radius_contains_the_piece` |
//! | o filete que o documento aceita ainda deixa peça? | `the_biggest_fillet_still_leaves_a_body` |
//! | *(o controle das três)* | `the_probe_can_see_a_field_that_climbs_too_fast` |
//!
//! ⭐ **Uma lista só** ([`representative`]), e é isso que torna o censo barato de estender: uma
//! primitiva nova é **erro de compilação** ali, e as três perguntas passam a valer para ela sem uma
//! linha de mudança.
//!
//! # ⚠️ Três destes gates nasceram de MUTAÇÕES QUE SOBREVIVERAM
//!
//! A W101 gateou o **campo** (a forma sai certa) e não a **API do documento**. Sobreviveram, na
//! primeira ronda: o `bounding_radius` da cápsula trocado por uma hipotenusa, o `round_limit` do
//! cone sem o `√(1+m²)`, e o `set_round` a esquecer as formas novas. *Escrevi a guarda certa e não
//! a gateei* — pela terceira vez neste repo.
//!
//! # A afirmação da marcha
//!
//! A marcha de esferas anda `d · SAFE_STEP` e é segura enquanto `SAFE_STEP · ‖∇f‖ ≤ 1`. As três
//! formas da W101 ([`Primitive::Cone`], [`Primitive::Capsule`], [`Primitive::Prism`]) são
//! construídas por `max` de meias-fatias, e o argumento é geométrico e não empírico: **o máximo de
//! funções 1-Lipschitz é 1-Lipschitz**. Este gate mede-o em vez de o acreditar.
//!
//! # ⚠️ Por que ele existe, e o que ele substitui
//!
//! A sonda que já havia — `the_table_of_who_inflates_the_gradient` — é `#[ignore]` (imprime uma
//! tabela) **e percorre uma lista escrita à mão**. Um `Primitive` novo não aparecia nela, e nada
//! reprovava: a mesma família de defeito que a W53 pagou com uma feature inteira invisível, um
//! nível abaixo.
//!
//! ⭐ Aqui a lista sai de [`PrimitiveKind::ALL`], e o `match` de [`representative`] é **exaustivo**
//! ⇒ uma primitiva nova é **erro de compilação** até alguém dizer com que números ela se mede.
//!
//! # ⚠️ O que a sonda NÃO acusa
//!
//! Numa quina convexa a distância é exacta e `‖∇f‖ = 1`; num vinco côncavo a derivada não existe e
//! a diferença central lê **menos** que 1. O que se caça é o contrário — uma região **lisa** onde o
//! campo sobe mais depressa que a distância, que é o que faz a marcha atravessar a superfície.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, PrimitiveKind, Xform};
use ph2d_field_eval::Field;

/// A folga sobre `1`. ⚠️ Ela é do **instrumento**, não da forma: a diferença central com
/// `eps = 1e-4` sobre uma quina lê um pouco acima de 1 por amostragem, e o `SAFE_STEP = 1/√2` do
/// módulo já concede `1,414`. Um `1,02` aqui é 70× mais apertado do que o que a marcha tolera.
const SLACK: f64 = 1.02;

/// ⭐ **Uma peça representativa de cada família** — e o `match` é o que fecha a corrente.
///
/// ⚠️ Os números não são decorativos: cada um põe a forma dentro da caixa `[-1, 1]³` que a sonda
/// varre, com **inclinação a sério** onde ela existe (um cone quase-cilíndrico não testaria o
/// `√(1+m²)` da parede, que é precisamente o termo que a W101 acrescentou).
fn representative(k: PrimitiveKind) -> Option<Primitive> {
    Some(match k {
        PrimitiveKind::Box => Primitive::Box {
            half: [0.4, 0.3, 0.25],
            round: 0.08,
        },
        PrimitiveKind::Sphere => Primitive::Sphere { radius: 0.5 },
        PrimitiveKind::Cylinder => Primitive::Cylinder {
            radius: 0.4,
            half_height: 0.3,
            round: 0.08,
        },
        PrimitiveKind::Torus => Primitive::Torus {
            major: 0.4,
            minor: 0.15,
        },
        // ⚠️ As duas de PERFIL ficam de fora **desta** sonda, e não é omissão: elas precisam de um
        // contorno desenhado, a `the_table_of_who_inflates_the_gradient` já as mede com um, e o que
        // este gate defende é a lista de primitivas de FÓRMULA estar completa. `None` é uma
        // resposta declarada, não um esquecimento — e o `match` continua exaustivo.
        PrimitiveKind::Extrude | PrimitiveKind::Revolve => return None,
        PrimitiveKind::Cone => Primitive::Cone {
            bottom: 0.45,
            top: 0.12,
            half_height: 0.35,
            round: 0.06,
        },
        PrimitiveKind::Capsule => Primitive::Capsule {
            radius: 0.25,
            half_height: 0.4,
        },
        PrimitiveKind::Prism => Primitive::Prism {
            sides: 6,
            radius: 0.45,
            half_height: 0.3,
            round: 0.05,
        },
    })
}

/// O maior `‖∇f‖` sobre uma grelha densa da caixa `[-e, e]³`.
///
/// ⚠️ **Recebe o [`Field`], e não a primitiva**, e é isso que deixa o controle abaixo medir-se pelo
/// **mesmo** instrumento: uma sonda testada com um campo que ela própria não produz é uma sonda
/// cujo controle não controla nada.
fn worst_gradient(f: &Field, e: f64, steps: usize) -> f64 {
    let mut worst = 0.0f64;
    for i in 0..steps {
        for j in 0..steps {
            for k in 0..steps {
                let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / steps as f64;
                let g = f.gradient_norm(at(i), at(j), at(k), 1.0e-4);
                if g.is_finite() {
                    worst = worst.max(g);
                }
            }
        }
    }
    worst
}

fn field_of(p: Primitive) -> Field {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça");
    Field::new(&doc)
}

#[test]
fn every_primitive_honours_the_march() {
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let g = worst_gradient(&field_of(p), 1.0, 24);
        assert!(
            g <= SLACK,
            "«{}» tem ‖∇f‖ = {g:.4} — acima de 1 o campo sobe mais depressa que a distância, e a \
             marcha ATRAVESSA a superfície com o passo de hoje",
            k.key()
        );
    }
}

/// ⭐⭐⭐ **A CAIXA DO MUNDO CONTÉM A PEÇA** — e uma mutação sobrevivente pediu este gate.
///
/// # ⚠️ O defeito que ele apanha é SILENCIOSO
///
/// O [`ph2d_field::bounding_radius`] é o que dá o alcance ao traçado e à malha. Um valor **pequeno
/// demais** não falha: a peça sai **cortada nas pontas**, e o artista culpa a forma. A cápsula é o
/// caso: a ponta dela está a `h + r` no eixo, e uma hipotenusa `√(h²+r²)` — que é o que as outras
/// primitivas usam — dá **menos**. A mutação que a trocou passou a suíte inteira.
///
/// ⚠️ **A régua é o CAMPO, e não outra fórmula nossa**: amostra-se a esfera de raio `bounding` em
/// muitas direções e exige-se que o campo lá seja **positivo** (fora). Comparar duas contas nossas
/// seria cego a uma mutação que mexesse nas duas.
#[test]
fn the_bounding_radius_contains_the_piece() {
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        // ⚠️ **Um cabelo PARA FORA, e não o raio exacto.** Numa esfera o `bounding_radius` **é** a
        // superfície (0,5 para raio 0,5), e o campo ali é zero a menos de um ULP — pedir
        // estritamente positivo reprovaria a forma mais apertada e correcta que existe. A
        // afirmação verdadeira é *«nada da peça fica ALÉM do raio de contenção»*.
        let r = f64::from(ph2d_field::bounding_radius(&p)) * 1.001;
        let f = field_of(p);
        // ⚠️ **Uma grelha de direções, e não os seis eixos**: a ponta de uma cápsula está num eixo,
        // mas a quina de um prisma de 6 lados não — seis amostras aprovariam metade das formas por
        // acidente.
        for i in 0..24 {
            for j in 0..24 {
                let theta = std::f64::consts::PI * (f64::from(i) + 0.5) / 24.0;
                let phi = std::f64::consts::TAU * (f64::from(j) + 0.5) / 24.0;
                let (x, y, z) = (
                    r * theta.sin() * phi.cos(),
                    r * theta.sin() * phi.sin(),
                    r * theta.cos(),
                );
                assert!(
                    f.at(x, y, z) >= 0.0,
                    "«{}»: o campo a {r:.4} do centro (o próprio bounding_radius) ainda está DENTRO \
                     — a caixa do mundo corta a peça, e ninguém diz porquê",
                    k.key()
                );
            }
        }
    }
}

/// ⭐⭐⭐ **O MAIOR FILETE QUE O DOCUMENTO ACEITA AINDA DEIXA PEÇA** — o segundo gate que uma mutação
/// sobrevivente pediu.
///
/// # ⚠️ O que ele defende
///
/// O `round_limit` de um cone tem de contar a **inclinação** (`a/√(1+m²)`): a receita do filete
/// recua cada parede na **perpendicular**, e na parede inclinada isso baixa `a` de
/// `round·√(1+m²)`, não de `round`. Sem o `√`, o documento aceita um filete que **inverte a
/// parede** — e o que sai é uma peça **vazia**, aceite pela validação, sem uma palavra.
///
/// ⚠️ **A régua é «há peça», e não um número:** compara-se o campo no centro da forma, que tem de
/// estar dentro. É a pergunta mais fraca que ainda mata a mutação, e por isso a mais estável.
#[test]
fn the_biggest_fillet_still_leaves_a_body() {
    for k in PrimitiveKind::ALL {
        let Some(mut p) = representative(k) else {
            continue;
        };
        let Some(limite) = ph2d_field::round_limit(&p) else {
            continue;
        };
        // Um cabelo abaixo do limite — a validação recusa `>=`.
        let quase = limite * 0.999;
        // ⭐⭐⭐ **AS DUAS PORTAS, e são mesmo duas.** O filete escreve-se por
        // `ph2d_field::set_shape_radius` (o raio de uma forma, que o gizmo usa) **e** por
        // `dims::set_dim` no índice do filete (a linha do painel). ⚠️ Uma mutação que esvaziasse a
        // segunda **sobreviveu** a este gate quando ele só atravessava a primeira: *duas portas
        // para o mesmo número são duas coisas para gatear, e a que se esquece é a que o artista
        // usa.*
        // ⛔⛔ **CADA PORTA PARTE DO ORIGINAL, e a 1.ª versão deste laço não partia.**
        //
        // ⚠️ Ele fazia `p = escrita` no fim de cada volta, então a segunda porta já encontrava o
        // filete escrito pela primeira — e uma porta que **não escrevesse nada** passava, porque o
        // valor certo já lá estava. A mutação que esvazia o `dims::set_round` **sobreviveu a este
        // gate**, que era exactamente o gate escrito para a matar. *Um arnês que acumula estado
        // entre casos testa o primeiro caso duas vezes.*
        let original = p.clone();
        for porta in ["set_shape_radius", "set_dim"] {
            let mut escrita = original.clone();
            if porta == "set_shape_radius" {
                let mut shape = ph2d_field::NodeShape::Leaf(escrita.clone());
                ph2d_field::set_shape_radius(&mut shape, 0, quase).expect("aceite");
                let ph2d_field::NodeShape::Leaf(dentro) = shape else {
                    unreachable!("entrou folha, sai folha")
                };
                escrita = dentro;
            } else {
                let idx = ph2d_field::dims(&escrita)
                    .iter()
                    .position(|d| d.key == "field.dim.round")
                    .expect("uma forma com filete tem a linha dele");
                ph2d_field::set_dim(&mut escrita, 0, idx, quase).expect("aceite");
            }
            assert_eq!(
                ph2d_field::NodeShape::Leaf(escrita.clone()).radius(),
                Some(quase),
                "«{}» pela porta `{porta}`: o filete foi aceite e NÃO foi escrito — o controle \
                 mexe-se e não faz nada, sem deixar rasto",
                k.key()
            );
            p = escrita;
        }
        let f = field_of(p);
        assert!(
            f.at(0.0, 0.0, 0.0) < 0.0,
            "«{}»: com o maior filete que o documento aceita ({quase:.4}) o centro da peça está \
             FORA — a parede inverteu, e a validação deixou passar",
            k.key()
        );
    }
}

/// ⛔ **O CONTROLE, e sem ele o gate acima não vale nada.**
///
/// ⚠️ Uma sonda que devolvesse sempre um número pequeno passaria as nove famílias e não mediria
/// coisa nenhuma. Um campo **deliberadamente esticado** (a esfera multiplicada por dois) tem
/// `‖∇f‖ = 2` por construção — se esta sonda não o vir, ela não vê nada.
///
/// ⚠️ E ele é construído **fora** do `Primitive`, de propósito: não há forma de o exprimir com uma
/// primitiva legítima, que é exactamente o que o gate acima afirma.
#[test]
fn the_probe_can_see_a_field_that_climbs_too_fast() {
    use fidget::context::Tree;
    let esticada = (Tree::x().square() + Tree::y().square() + Tree::z().square())
        .max(1.0e-30)
        .sqrt()
        * Tree::constant(2.0)
        - Tree::constant(1.0);
    let g = worst_gradient(&Field::from_tree(&esticada), 1.0, 12);
    assert!(
        g > 1.5,
        "a sonda leu ‖∇f‖ = {g:.4} num campo que sobe ao DOBRO da distância — ela não mede nada"
    );
}
