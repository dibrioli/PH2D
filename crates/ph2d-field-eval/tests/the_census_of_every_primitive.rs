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
        // ⚠️ **ESTREITADO**, e não o prisma recto: um prisma de paredes verticais não testaria o
        // `√(1+m²)` da parede inclinada, que é o termo que a W102 acrescentou.
        PrimitiveKind::Prism => Primitive::Prism {
            sides: 6,
            bottom: 0.45,
            top: 0.18,
            half_height: 0.3,
            round: 0.05,
        },
        PrimitiveKind::Wedge => Primitive::Wedge {
            half: [0.45, 0.3, 0.35],
            round: 0.05,
        },
        // ⚠️ **Meia volta e um pouco**, de propósito: é o lado do `min` do sector, e um arco de menos
        // de meia volta nunca lá chegaria.
        PrimitiveKind::TorusArc => Primitive::TorusArc {
            major: 0.4,
            minor: 0.15,
            angle: std::f64::consts::PI as f32 * 1.3,
        },
        // ⚠️ **CINCO pontas**, que é o número ímpar: com um par, metade das paredes cai sobre a
        // outra metade por simetria, e uma costura entre pipas vizinhas nunca seria varrida em
        // ângulo genérico.
        PrimitiveKind::Star => Primitive::Star {
            points: 5,
            outer: 0.45,
            inner: 0.18,
            half_height: 0.25,
            round: 0.03,
        },
        PrimitiveKind::BoxFrame => Primitive::BoxFrame {
            half: [0.45, 0.35, 0.4],
            thickness: 0.12,
            round: 0.03,
        },
        // ⚠️ **Os três semi-eixos DIFERENTES**: com dois iguais o campo é o de um esferóide, e a
        // razão `min/max` — que é exatamente o que esta forma subestima — só aparece num par.
        PrimitiveKind::Ellipsoid => Primitive::Ellipsoid {
            radii: [0.5, 0.2, 0.35],
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

fn doc_of(p: Primitive) -> FieldDoc {
    FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça")
}

fn field_of(p: Primitive) -> Field {
    Field::new(&doc_of(p))
}

/// ⭐⭐⭐ **O produto `passo × ‖∇f‖` é o que fura, e é ele que este gate mede.**
///
/// # ⛔ A barra era `‖∇f‖ ≤ 1,02`, e a W103 mostrou que essa pergunta é a errada
///
/// Enquanto **toda** primitiva era 1-Lipschitz, medir só o gradiente dava no mesmo. Deixou de dar
/// no dia em que o filete do cone passou a existir: ele arredonda por **interseção arredondada** (a
/// única saída com paredes não-ortogonais), e isso infla — `1,1943` medido, que é o `√(1 − cos φ)`
/// do canto. Baixar a barra seria **afrouxar o gate**; a resposta certa é perguntar ao módulo qual
/// o passo que ele vai de facto usar nesta peça ([`ph2d_field_eval::safe_march_step`]) e exigir que
/// o **produto** seja seguro.
///
/// ⭐ Isto é estritamente MAIS forte do que a barra antiga: uma primitiva que inflasse **sem** o
/// declarar ao [`ph2d_field::fillet_inflates`] passaria a barra de `1,02`… não passaria — mas
/// também não passa aqui, e agora com a causa nomeada (o passo continua em `1,0`).
#[test]
fn every_primitive_honours_the_march() {
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let doc = doc_of(p.clone());
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
        let g = worst_gradient(&Field::new(&doc), 1.0, 24);
        assert!(
            passo * g <= SLACK,
            "«{}»: passo {passo:.4} × ‖∇f‖ {g:.4} = {:.4} — acima de 1 a marcha ATRAVESSA a \
             superfície",
            k.key(),
            passo * g
        );
    }
}

/// ⛔ **O CONTROLE do gate acima, e sem ele o produto passaria por ser pequeno.**
///
/// ⚠️ Se `safe_march_step` devolvesse um número minúsculo para tudo, o produto seria seguro em toda
/// a linha e o gate deixaria de medir. Esta metade exige o contrário: quem **não** infla tem de
/// andar a passo INTEIRO — é o `CLAUDE.md` §0 aplicado ao passo (*o caminho lento não define o teto
/// do rápido*), e é a propriedade que a W56f construiu.
#[test]
fn the_shapes_that_do_not_inflate_still_march_at_full_step() {
    let mut inteiros = 0;
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let passo = ph2d_field_eval::safe_march_step(&doc_of(p.clone()));
        if ph2d_field::fillet_inflates(&p) {
            assert!(
                passo < 1.0,
                "«{}» infla e mesmo assim anda a passo inteiro",
                k.key()
            );
        } else {
            assert!(
                (passo - 1.0).abs() < 1.0e-6,
                "«{}» não infla e anda a {passo:.4} — o caminho lento a definir o teto do rápido",
                k.key()
            );
            inteiros += 1;
        }
    }
    assert!(
        inteiros >= 4,
        "o controle perdeu o sujeito: {inteiros} formas a passo inteiro"
    );
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
        // ⛔⛔ **A PERGUNTA MUDOU NA W103, e a antiga estava ERRADA — não «apertada demais».**
        //
        // ⚠️ Ela era `f(0,0,0) < 0`: *«o centro da peça está dentro»*. Isso é uma afirmação sobre
        // sólidos **maciços**, e o [`Primitive::BoxFrame`] tem o miolo **vazio de propósito** — o
        // gate reprovou-o a dizer *«a parede inverteu, e a validação deixou passar»*, que é uma
        // causa que não existia. *Uma sonda que amostra UM ponto escolhido a olho carrega, sem o
        // dizer, a forma que o autor tinha em mente.*
        //
        // ⭐ A pergunta derivada faz o mesmo trabalho sem essa premissa: **conta** quantas amostras
        // de uma grelha caem dentro, com filete zero e com o filete máximo. `n1 > 0` é «ainda há
        // peça»; `n1 <= n0` é «o filete comeu, não cresceu» — e é este segundo lado que apanha a
        // parede invertida que a redação antiga NOMEAVA e nunca media.
        let n1 = inside_count(&p);
        let n0 = {
            // ⚠️ Pela porta do RAIO e não pelo `set_dim`: o zero é a aresta viva, e é ela que dá a
            // referência contra a qual o filete tem de tirar material.
            let mut shape = ph2d_field::NodeShape::Leaf(original.clone());
            ph2d_field::set_shape_radius(&mut shape, 0, 0.0).expect("aresta viva é sempre válida");
            let ph2d_field::NodeShape::Leaf(sem) = shape else {
                unreachable!("entrou folha, sai folha")
            };
            inside_count(&sem)
        };
        assert!(
            n1 > 0,
            "«{}»: com o maior filete que o documento aceita ({quase:.4}) NÃO sobrou peça nenhuma \
             — a validação deixou passar um filete que apaga a forma",
            k.key()
        );
        // ⛔⛔⛔ **ESTRITAMENTE MENOR, e é esta desigualdade que apanha o filete INERTE.**
        //
        // ⚠️ A W101 e a W102 shiparam o cone, o prisma, a cunha e a estrela com um `round` que
        // **não fazia nada** (`+0,0 %` de volume, campo bit a bit igual) ou que fazia a peça
        // **crescer** (`+41,0 %` na cunha). Nenhum gate reprovava, porque nenhum comparava a peça
        // COM o filete contra a peça SEM ele — mediam-se propriedades da forma filetada, e um campo
        // que ignora o filete continua a ser uma forma correcta. *Um `<=` teria deixado passar as
        // duas inertes; a inércia é precisamente a igualdade.*
        assert!(
            n1 < n0,
            "«{}»: o filete de {quase:.4} não tirou NADA ({n0} amostras dentro sem ele, {n1} com \
             ele) — ou ele é inerte, ou o recuo da fonte tem o sinal trocado",
            k.key()
        );
    }
}

/// Quantas amostras de uma grelha do bordo da peça caem **dentro** dela.
///
/// ⚠️ **A grelha sai do [`ph2d_field::bounding_radius`]**, e não de um número escrito aqui: uma
/// caixa fixa mediria a fração da caixa em peças de tamanhos diferentes, e a comparação de duas
/// formas deixaria de querer dizer alguma coisa.
fn inside_count(p: &Primitive) -> u32 {
    use fidget::shape::EzShape;
    // ⚠️ **A finura sai do FILETE, não do gosto** — e a primeira versão (`24`) reprovou a gaiola por
    // uma razão que não existia: com a viga a `0,12` e o passo a `0,060`, cabiam **duas** amostras
    // na secção, e um filete que come `21 %` dela não movia nenhuma. *Uma grelha que não resolve a
    // feature mede a grelha.* A `64` o passo é `0,023`, e a diferença aparece.
    const N: i32 = 64;
    // ⚠️ **Em FATIA, e não ponto a ponto**: a mesma grelha por [`Field::at`] custava **18 s** neste
    // gate, e um teste que ficou lento é uma medição de custo que ninguém pediu. A fita é a mesma; o
    // que muda é quantas vezes se atravessa a fronteira do avaliador.
    let r = f64::from(ph2d_field::bounding_radius(p)) * 1.05;
    let c = |v: i32| ((f64::from(v) / f64::from(N)).mul_add(2.0, -1.0) * r) as f32;
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..=N {
        for j in 0..=N {
            for k in 0..=N {
                xs.push(c(i));
                ys.push(c(j));
                zs.push(c(k));
            }
        }
    }
    let shape = ph2d_field_eval::Engine::from(ph2d_field_eval::compile(&doc_of(p.clone())));
    let tape = shape.ez_float_slice_tape();
    let mut eval = ph2d_field_eval::Engine::new_float_slice_eval();
    let out = eval.eval(&tape, &xs, &ys, &zs).expect("avalia a grelha");
    u32::try_from(out.iter().filter(|v| **v < 0.0).count()).unwrap_or(u32::MAX)
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
