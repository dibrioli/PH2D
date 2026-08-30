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
            chamfer: 0.0,
        },
        PrimitiveKind::Sphere => Primitive::Sphere { radius: 0.5 },
        PrimitiveKind::Cylinder => Primitive::Cylinder {
            radius: 0.4,
            half_height: 0.3,
            round: 0.08,
            chamfer: 0.0,
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
            chamfer: 0.0,
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
            chamfer: 0.0,
        },
        PrimitiveKind::Wedge => Primitive::Wedge {
            half: [0.45, 0.3, 0.35],
            round: 0.05,
            chamfer: 0.0,
        },
        // ⚠️ **Meia volta e um pouco**, de propósito: é o lado do `min` do sector, e um arco de menos
        // de meia volta nunca lá chegaria.
        PrimitiveKind::TorusArc => Primitive::TorusArc {
            major: 0.4,
            minor: 0.15,
            angle: std::f64::consts::PI as f32 * 1.3,
            round: 0.04,
            chamfer: 0.0,
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
            chamfer: 0.0,
        },
        PrimitiveKind::BoxFrame => Primitive::BoxFrame {
            half: [0.45, 0.35, 0.4],
            thickness: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        // ⚠️ **Os três semi-eixos DIFERENTES**: com dois iguais o campo é o de um esferóide, e a
        // razão `min/max` — que é exatamente o que esta forma subestima — só aparece num par.
        PrimitiveKind::Ellipsoid => Primitive::Ellipsoid {
            radii: [0.5, 0.2, 0.35],
        },
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **Cada representante escolhe o caso que EXERCITA a fórmula**, não o mais bonito: um
        // default cómodo esconde exactamente o termo que a forma acrescentou.
        PrimitiveKind::Octahedron => Primitive::Octahedron {
            radius: 0.45,
            round: 0.06,
            chamfer: 0.0,
        },
        // ⚠️ **Raios DIFERENTES**: com dois iguais ele degenera na cápsula, e o termo tangente —
        // que é tudo o que esta forma acrescenta — nunca seria exercitado.
        PrimitiveKind::RoundCone => Primitive::RoundCone {
            bottom: 0.35,
            top: 0.14,
            half_height: 0.3,
        },
        // ⚠️ Corte ACIMA do equador: em `cut = 0` a tampa é o disco maior e o caso é o mais fácil.
        PrimitiveKind::CutSphere => Primitive::CutSphere {
            radius: 0.45,
            cut: 0.15,
            round: 0.05,
            chamfer: 0.0,
        },
        PrimitiveKind::HollowDome => Primitive::HollowDome {
            radius: 0.45,
            cut: 0.1,
            thickness: 0.1,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Link => Primitive::Link {
            major: 0.3,
            minor: 0.1,
            length: 0.25,
        },
        PrimitiveKind::SolidAngle => Primitive::SolidAngle {
            radius: 0.45,
            angle: 0.7,
            round: 0.05,
            chamfer: 0.0,
        },
        // ⚠️ **Sete dentes**, que é ímpar pela razão da estrela: com um par, metade dos flancos cai
        // sobre a outra metade por simetria e uma costura entre dentes vizinhos nunca seria varrida
        // em ângulo genérico.
        PrimitiveKind::Gear => Primitive::Gear {
            teeth: 7,
            root: 0.32,
            outer: 0.45,
            tooth: 0.45,
            half_height: 0.15,
            round: 0.02,
            chamfer: 0.0,
        },
        PrimitiveKind::Cross => Primitive::Cross {
            arm: 0.45,
            width: 0.14,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Heart => Primitive::Heart {
            size: 0.3,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Moon => Primitive::Moon {
            radius: 0.45,
            bite: 0.4,
            offset: 0.2,
            half_height: 0.12,
            round: 0.02,
            chamfer: 0.0,
        },
        PrimitiveKind::Drop => Primitive::Drop {
            radius: 0.22,
            height: 0.55,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Pie => Primitive::Pie {
            radius: 0.45,
            angle: 1.0,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        // ⚠️ **Bases DIFERENTES**: iguais seria uma caixa, e o flanco inclinado — o `√(1+m²)` que
        // esta forma tem — não seria exercitado.
        PrimitiveKind::Trapezoid => Primitive::Trapezoid {
            bottom: 0.45,
            top: 0.2,
            half_width: 0.3,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Vesica => Primitive::Vesica {
            radius: 0.45,
            offset: 0.25,
            half_height: 0.12,
            round: 0.02,
            chamfer: 0.0,
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
        // ⛔⛔⛔ **MEDE-SE O ALCANCE DA PEÇA, e não se espera que uma amostra caia nele.**
        //
        // As duas versões anteriores deste gate falharam a mesma pergunta, cada uma à sua maneira,
        // e as duas ficaram VERDES sobre um defeito que chegou à tela (report do Enio, 30/08:
        // quatro setas para arcos pretos a atravessar uma cruz):
        //
        // | tentativa | por que ficou verde |
        // |---|---|
        // | `24×24` **direções**, campo `≥ 0` na esfera de raio `r` | exige **acertar na direção** do ponto mais afastado; a quina de um braço fino estava a `75,7°/17,3°` e a grelha passa a `7,5°` e `22,5°` — de raspão pelos dois lados |
        // | grelha de **pontos** na caixa | a saliência era de `0,003` e o passo da grelha `0,028` — *nenhuma amostra caiu lá dentro* |
        //
        // ⭐⭐⭐ *Uma fixtura de amostras prova o que amostrou.* ⇒ aqui **bissecta-se a superfície**
        // ao longo de cada direção e toma-se o **máximo** dos raios encontrados. Isso mede a
        // grandeza que a pergunta é — *até onde a peça chega* — em vez de esperar que uma amostra
        // caia por cima dela, e converge com a densidade de direções em vez de depender da sorte.
        const DIRS: usize = 96;
        let mut alcance = 0.0f64;
        let mut viu_peca = false;
        for i in 0..DIRS {
            for j in 0..(DIRS * 2) {
                let theta = std::f64::consts::PI * (f64::from(u32::try_from(i).unwrap()) + 0.5)
                    / DIRS as f64;
                let phi = std::f64::consts::TAU * (f64::from(u32::try_from(j).unwrap()) + 0.5)
                    / (DIRS * 2) as f64;
                let d = [
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ];
                let at = |t: f64| f.at(d[0] * t, d[1] * t, d[2] * t);
                // ⛔ O CONTROLE, colhido no mesmo varrimento: a peça tem de EXISTIR nalgum sítio.
                // ⚠️ Não se pergunta pelo centro — o de um toro é **vazio**, e a 1.ª escrita deste
                // controlo reprovou-o por isso.
                for n in 1..8 {
                    if at(r * f64::from(n) / 8.0) < 0.0 {
                        viu_peca = true;
                        break;
                    }
                }
                // Fora, à partida: se já está fora em `r`, esta direção não excede.
                if at(r) >= 0.0 {
                    // Bissecta entre 0 e r para achar onde a superfície está — mas só interessa
                    // o caso em que ela passa de r, então salta.
                    continue;
                }
                // ⚠️ Ainda DENTRO em `r` ⇒ a peça excede nesta direção. Acha até onde.
                let mut lo = r;
                let mut hi = r * 4.0;
                for _ in 0..40 {
                    let mid = 0.5 * (lo + hi);
                    if at(mid) < 0.0 { lo = mid } else { hi = mid }
                }
                alcance = alcance.max(hi);
            }
        }
        assert!(
            viu_peca,
            "«{}»: nenhuma direção encontrou peça — a varredura não tem sujeito",
            k.key()
        );
        assert!(
            alcance == 0.0,
            "«{}»: há peça a {alcance:.4} do centro e o bounding_radius diz {:.4} — a caixa do \
             mundo CORTA a peça, e o corte sai ESFÉRICO (um arco preto a atravessá-la)",
            k.key(),
            r / 1.001
        );
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

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ O CHANFRO — pedido do Enio, 2026-08-30
//
// > *«em todas as peças temos fillet para as bordas arredondadas mas não temos um slider para
// > chamfer. Poderíamos ter os 2, com chamfer antes de fillet para a possibilidade de arredondar as
// > bordas geradas por chamfer»*
//
// ⚠️ As três perguntas do censo valem para ele **sem uma linha de fixtura nova**, porque a lista é a
// mesma [`representative`] — que é a razão de o censo existir.
// ─────────────────────────────────────────────────────────────────────────────

/// Esta forma com o chanfro posto a `fracao` da parede que o documento declara.
fn com_chanfro(mut p: Primitive, fracao: f32) -> Option<Primitive> {
    let limite = ph2d_field::round_limit(&p)?;
    let alvo = limite * fracao;
    let i = ph2d_field::dims(&p)
        .iter()
        .position(|d| d.key == "field.dim.chamfer")?;
    ph2d_field::set_dim(&mut p, 0, i, alvo).ok()?;
    Some(p)
}

/// ⭐⭐⭐ **O CHANFRO MUDA TODA FORMA QUE O OFERECE** — e é a régua que separa um slider de um knob
/// morto.
///
/// ⚠️ **A lista é derivada** de [`PrimitiveKind::ALL`], então uma primitiva nova entra aqui sozinha.
/// ⛔ Uma lista escrita à mão teria deixado de fora exactamente a forma cujo construtor esqueceu de
/// ler o número — que é o defeito que este gate existe para apanhar, e que a W104 já pagou uma vez
/// com o `round` do cone e do prisma **inertes** (`+0,0 %` de volume, campo bit a bit igual).
#[test]
fn every_shape_that_offers_a_chamfer_is_changed_by_it() {
    let mut testadas = 0;
    let mut mudas = Vec::new();
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        let Some(chanfrada) = com_chanfro(p.clone(), 0.5) else {
            continue;
        };
        testadas += 1;
        let (vivo, cortado) = (inside_count(&p), inside_count(&chanfrada));
        // O chanfro **tira** material da quina: a contagem tem de descer, e não só mexer.
        let queda = f64::from(vivo.saturating_sub(cortado)) / f64::from(vivo.max(1)) * 100.0;
        if queda < 0.05 {
            mudas.push(format!("{k:?} ({vivo} -> {cortado}, {queda:.3} %)"));
        }
    }
    assert!(
        testadas >= 20,
        "o censo só chegou a {testadas} formas com aresta — a lista derivada partiu-se"
    );
    assert!(
        mudas.is_empty(),
        "estas formas oferecem o slider do chanfro e não são cortadas por ele: {mudas:?}"
    );
}

/// ⭐⭐⭐ **O CHANFRO HONRA A MARCHA, em toda forma que o oferece** — o produto `passo × ‖∇f‖`.
///
/// # ⛔ A 1.ª redacção deste gate media o gradiente CRU, e a resposta foi um achado
///
/// Ela afirmava que *o chanfro sozinho nunca infla* — «um `max` de funções 1-Lipschitz é
/// 1-Lipschitz» — e o censo refutou-a com o número que este arquivo já tinha escrito: **`1,1943` no
/// cone**, o `√(1 − cos φ)` do canto. *O plano do chanfro herda o ângulo das duas faces que ele
/// corta*, e a demonstração só vale enquanto as normais são ortogonais — que é precisamente o que
/// uma parede inclinada não é.
///
/// ⇒ a pergunta certa é a mesma que o [`every_primitive_honours_the_march`] já faz: **o produto**.
/// O `ph2d_field::fillet_inflates` passou a distinguir as quatro exactas (peças ortogonais, o
/// chanfro sozinho lê `1,0000`) das dezassete de parede inclinada (qualquer recuo infla).
#[test]
fn a_chamfer_honours_the_march_on_every_shape() {
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        let Some(mut chanfrada) = com_chanfro(p, 0.5) else {
            continue;
        };
        // ⭐ Sem filete: é o chanfro sozinho que está sob teste. ⚠️ E o zero **passa** desde a
        // `Span::WallFromZero` — antes dela esta linha era silenciosamente recusada e o gate media
        // os dois recuos juntos.
        let i = ph2d_field::dims(&chanfrada)
            .iter()
            .position(|d| d.key == "field.dim.round")
            .expect("uma forma com chanfro tem filete");
        ph2d_field::set_dim(&mut chanfrada, 0, i, 0.0).expect("o zero é a aresta viva");
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc_of(chanfrada.clone())));
        let g = worst_gradient(&field_of(chanfrada), 1.0, 24);
        assert!(
            passo * g <= SLACK,
            "{k:?}: passo {passo:.4} x ‖∇f‖ {g:.4} = {:.4} — a marcha atravessa a superfície",
            passo * g
        );
    }
}

/// ⭐⭐⭐ **TODA FORMA MARCHA EM SEGURANÇA COM OS DOIS RECUOS LIGADOS** — o gate que faltava.
///
/// # ⛔⛔ Ele nasceu de um report do Enio (2026-08-30), e o buraco era do CENSO
///
/// *«com um prisma veja que algumas arestas não receberam o fillet e ao rotacionar a aparência da
/// aresta muda»*. A segunda metade é a assinatura de um campo que **sobe mais depressa que a
/// distância**: a marcha atravessa a superfície, e o ponto em que ela pára passa a depender da
/// direcção do raio.
///
/// ⚠️ **O gate que existia media o par numa forma só — a caixa.** Medido depois do report, sobre as
/// **vinte** formas com aresta:
///
/// | forma | `passo × ‖∇f‖` antes | depois |
/// |---|---:|---:|
/// | `Octahedron` | **`3,4572`** | `0,8165` |
/// | `Wedge` | `1,8928` | `0,9981` |
/// | `Prism` | `1,4061` | `0,9860` |
/// | `Box` | `1,2237` | `1,0000` (passo **cheio**) |
/// | *(16 de 20 acima de `1`)* | | *(nenhuma)* |
///
/// ⇒ *um gate escrito para uma forma deixa as outras dezanove por medir*, que é a mesma família do
/// `the_fillet_reaches_every_edge_of_every_shape`.
///
/// # ⭐ A causa era a CONSTRUÇÃO, e não a feature
///
/// A 1.ª versão compunha chanfro-e-filete **misturando duas vezes**
/// (`intersection(intersection(a, plano, r), b, r)`), e cada nível encaixado soma um quadrado na lei
/// de Cauchy–Schwarz do [`ph2d_field_eval::gradient_bound`] — `√3` numa caixa, medido a `1,7306`.
/// Hoje é **encolher, chanfrar, deslocar**: um `max` de 1-Lipschitz é 1-Lipschitz.
#[test]
fn every_shape_marches_safely_with_both_recesses_on() {
    let mut furam = Vec::new();
    let mut testadas = 0;
    for k in PrimitiveKind::ALL {
        let Some(base) = representative(k) else {
            continue;
        };
        let Some(limite) = ph2d_field::round_limit(&base) else {
            continue;
        };
        let meio = limite * 0.5;
        let escreve = |p: &Primitive, chave: &str, v: f32| -> Option<Primitive> {
            let mut p = p.clone();
            let i = ph2d_field::dims(&p).iter().position(|d| d.key == chave)?;
            ph2d_field::set_dim(&mut p, 0, i, v).ok()?;
            Some(p)
        };
        let Some(par) = escreve(&base, "field.dim.round", meio)
            .and_then(|p| escreve(&p, "field.dim.chamfer", meio))
        else {
            continue;
        };
        testadas += 1;
        let d = doc_of(par.clone());
        let passo = f64::from(ph2d_field_eval::safe_march_step(&d));
        let g = worst_gradient(&field_of(par), 1.2, 30);
        if passo * g > SLACK {
            furam.push(format!("{k:?} {:.4}", passo * g));
        }
    }
    assert!(
        testadas >= 20,
        "só {testadas} formas com aresta — a lista derivada partiu-se"
    );
    assert!(
        furam.is_empty(),
        "estas formas marcham por cima da própria superfície com os dois recuos ligados: {furam:?}"
    );
}

/// ⭐⭐ **E as QUATRO EXACTAS andam o passo CHEIO com os dois recuos** — a caixa, o cilindro, a
/// extrusão e a moldura.
///
/// ⚠️ **A 1.ª versão desta wave dizia o contrário e tinha um gate a afirmá-lo**, porque a construção
/// misturada de facto inflava. *Um gate verde pode pinar um defeito*: aquele estava a defender o
/// preço de uma escrita errada como se fosse uma propriedade da forma.
#[test]
fn the_four_exact_shapes_keep_the_full_step_with_both_recesses() {
    let caixa = |round: f32, chamfer: f32| Primitive::Box {
        half: [0.4, 0.3, 0.25],
        round,
        chamfer,
    };
    for (r, c) in [(0.0, 0.0), (0.05, 0.0), (0.0, 0.05), (0.05, 0.05)] {
        let passo = ph2d_field_eval::safe_march_step(&doc_of(caixa(r, c)));
        assert!(
            (passo - 1.0).abs() < 1e-6,
            "filete {r} + chanfro {c}: a caixa arredonda por DESLOCAMENTO nos dois casos e tem de \
             andar o passo cheio, andou {passo}"
        );
    }
}

/// ⭐⭐⭐ **O RECUO ENTREGUE É O NÚMERO PEDIDO** — a régua que os quatro caracteres desta casa
/// partilham, medida na forma em que ela é inequívoca.
///
/// Uma caixa chanfrada em `c` tem de perder a quina a **exactamente** `c` de distância, ao longo de
/// cada uma das duas faces. ⚠️ Um chanfro que entregasse `0,71×` do pedido mentiria uma fracção
/// fixa, sempre — e trocar entre o chip *Chamfer* e este slider mudaria o tamanho da peça.
#[test]
fn the_chamfer_sets_back_exactly_what_it_promises() {
    const H: f64 = 0.4;
    for c in [0.05f64, 0.10, 0.15] {
        let f = field_of(Primitive::Box {
            half: [H as f32; 3],
            round: 0.0,
            chamfer: c as f32,
        });
        // Caminhamos na face `x = H` (fora dela o campo é positivo) e achamos onde a face acaba.
        let mut fim = 0.0f64;
        for i in 0..=4000 {
            let y = f64::from(i) / 4000.0 * H;
            // Na face plana o campo em `(H, y, 0)` é exactamente zero.
            if f.at(H, y, 0.0).abs() <= 1e-6 {
                fim = y;
            }
        }
        let recuo = H - fim;
        assert!(
            (recuo - c).abs() < 2e-3,
            "chanfro pedido {c:.3}: a face acaba a {recuo:.5} da quina, e tinha de acabar a {c:.3}"
        );
    }
}

/// ⭐⭐⭐ **O FILETE ARREDONDA a aresta do chanfro — ele não a MOVE.**
///
/// # ⛔⛔ Ele nasceu do 2.º report do Enio sobre a mesma feature (2026-08-30)
///
/// *«não funcionou. o fillet só muda a posição do chamfer»* — e ele estava literalmente certo. A
/// construção de então («encolher, chanfrar, deslocar») é a receita do filete da caixa, e ela
/// arredonda o que é **distância exacta**; um plano de chanfro é um **semiespaço**, e deslocar um
/// semiespaço dá outro semiespaço. É a lei que a W104 já tinha medido e escrito neste módulo, e eu
/// quebrei-a.
///
/// Medido, com o chanfro em `0,12`:
///
/// | filete | posição da quina | maior giro da normal |
/// |---:|---:|---:|
/// | `0,00` | `0,48083` | `40,2°` |
/// | `0,02` | `0,47255` | **`45,000°`** |
/// | `0,08` | `0,44770` | **`45,000°`** |
///
/// ⇒ o giro **cravado em `45°`** (uma quina perfeita) enquanto a posição desliza. *O gate que
/// existia media a MORDIDA — o volume que sai — e um chanfro deslocado tira volume na mesma.*
///
/// ⭐ A cura é arredondar as **três** superfícies de uma vez (as duas faces e o plano), com a
/// [`ph2d_field_eval::ops::intersection_round_n`]. Depois dela o giro cai a `1,3°` e `0,5°`.
#[test]
fn the_fillet_rounds_the_chamfer_edge_instead_of_moving_it() {
    let campo = |round: f32, chamfer: f32| {
        field_of(Primitive::Box {
            half: [0.4; 3],
            round,
            chamfer,
        })
    };
    // O maior giro da normal entre dois passos, ao longo da secção `z = 0` de um quadrante.
    let giro = |f: &Field| {
        let normal = |ang: f64| {
            let (c, s) = (ang.cos(), ang.sin());
            let (mut lo, mut hi) = (0.0f64, 2.0);
            for _ in 0..60 {
                let m = 0.5 * (lo + hi);
                if f.at(c * m, s * m, 0.0) <= 0.0 {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            let r = 0.5 * (lo + hi);
            let (x, y, e) = (c * r, s * r, 1.0e-4);
            let gx = f.at(x + e, y, 0.0) - f.at(x - e, y, 0.0);
            let gy = f.at(x, y + e, 0.0) - f.at(x, y - e, 0.0);
            let m = gx.hypot(gy).max(1.0e-12);
            (gx / m, gy / m)
        };
        let mut pior = 0.0f64;
        let mut ant = normal(0.0);
        for i in 1..=900 {
            let a = std::f64::consts::FRAC_PI_2 * f64::from(i) / 900.0;
            let cur = normal(a);
            pior = pior.max(
                (ant.0 * cur.0 + ant.1 * cur.1)
                    .clamp(-1.0, 1.0)
                    .acos()
                    .to_degrees(),
            );
            ant = cur;
        }
        pior
    };
    let vivo = giro(&campo(0.0, 0.12));
    assert!(
        vivo > 20.0,
        "⛔ o CONTROLE falhou: um chanfro sem filete TEM de deixar uma quina, e o giro leu \
         {vivo:.2}° — a sonda não está a ver quinas"
    );
    for r in [0.02f32, 0.05] {
        let g = giro(&campo(r, 0.12));
        assert!(
            g < 5.0,
            "com o chanfro em 0,12 e o filete em {r}, o giro da normal é {g:.3}° — o filete está a \
             MOVER a quina em vez de a arredondar (report do Enio, 2026-08-30)"
        );
    }
}

/// ⭐ **O FILETE POR CIMA morde as arestas que o chanfro criou** — que é literalmente o pedido.
#[test]
fn the_fillet_bites_the_edges_the_chamfer_made() {
    let caixa = |round: f32| Primitive::Box {
        half: [0.4; 3],
        round,
        chamfer: 0.12,
    };
    let (so_chanfro, com_filete) = (inside_count(&caixa(0.0)), inside_count(&caixa(0.05)));
    assert!(
        com_filete < so_chanfro,
        "o filete tem de tirar mais material nas arestas novas do chanfro: {so_chanfro} -> \
         {com_filete}"
    );
}

/// ⭐⭐⭐ **OS DOIS RECUOS DE UMA ARESTA VOLTAM A ZERO** — e este gate cura um defeito
/// **pré-existente** que o chanfro tornou impossível de ignorar.
///
/// # ⛔ O defeito, e a lei que já estava escrita a nomeá-lo
///
/// O filete usava [`ph2d_field::Span::Wall`], e o painel mapeia essa faixa para um slider que **começa em
/// zero**. A porta de escrita, porém, recusava o zero — `Wall` promete «positiva». ⇒ o artista
/// arredondava uma aresta e **não conseguia desarredondá-la**: o controle descia até ao fundo e o
/// número parava logo acima dele, sem dizer porquê.
///
/// ⚠️ O doc da [`ph2d_field::Span::Count`] já descrevia exactamente esta família — *«uma faixa que oferece
/// o que a porta recusa é uma affordance que mente»* — e foi por isso que ela ganhou um `min`. O que
/// faltava era aplicar a mesma lei ao outro lado da faixa.
///
/// ⭐ Para o **chanfro** isto não seria conforto: zero é o estado de nascimento dele, e um knob que
/// só liga é um knob que prende o artista.
///
/// ⚠️ **A lista é derivada** de [`PrimitiveKind::ALL`] — uma forma nova entra sozinha.
#[test]
fn both_recesses_of_an_edge_can_go_back_to_zero() {
    let mut presas = Vec::new();
    let mut testadas = 0;
    for k in PrimitiveKind::ALL {
        let mut p = match representative(k) {
            Some(p) => p,
            None => continue,
        };
        let mut tem_aresta = false;
        for chave in ["field.dim.chamfer", "field.dim.round"] {
            let Some(i) = ph2d_field::dims(&p).iter().position(|d| d.key == chave) else {
                continue;
            };
            tem_aresta = true;
            let limite = ph2d_field::round_limit(&p).expect("tem parede");
            // Primeiro liga-se o recuo, senão o gate mede o estado em que ele já estava.
            ph2d_field::set_dim(&mut p, 0, i, limite * 0.5).expect("ligar o recuo");
            if ph2d_field::set_dim(&mut p, 0, i, 0.0).is_err() {
                presas.push(format!("{k:?}/{chave}"));
            } else {
                assert!(
                    (ph2d_field::dims(&p)[i].value).abs() < 1e-9,
                    "{k:?}/{chave}: a porta disse Ok e o número não foi a zero"
                );
            }
        }
        if tem_aresta {
            testadas += 1;
        }
    }
    assert!(
        testadas >= 20,
        "só {testadas} formas com aresta — a lista derivada partiu-se"
    );
    assert!(
        presas.is_empty(),
        "estes recuos não voltam a zero, e o slider do painel oferece o zero: {presas:?}"
    );
}

/// ⭐⭐ **ESCALAR UMA FORMA ESCALA OS DOIS RECUOS DELA** — o filete **e** o chanfro.
///
/// ⚠️ Os dois são **comprimentos da peça**, e um deles fixo faria a aresta mudar de carácter ao
/// redimensionar: uma caixa de `1` com chanfro de `0,1` tem um corte de 10 % da face; a mesma caixa
/// levada a `4` teria um corte de 2,5 %, e o artista veria a quina «endurecer» sozinha.
///
/// ⛔ **O compilador apontou este defeito** — 17 avisos de `chamfer` não usado no `scale_primitive`,
/// um por forma. *Um aviso de variável não usada num `match` exaustivo é o compilador a dizer que
/// alguém acrescentou um campo e esqueceu metade do trabalho.*
///
/// ⚠️ **A lista é derivada** de [`PrimitiveKind::ALL`].
#[test]
fn scaling_a_shape_scales_both_recesses() {
    const FATOR: f32 = 3.0;
    let mut paradas = Vec::new();
    let mut testadas = 0;
    for k in PrimitiveKind::ALL {
        let Some(base) = representative(k) else {
            continue;
        };
        let Some(mut p) = com_chanfro(base, 0.5) else {
            continue;
        };
        testadas += 1;
        let antes: Vec<(&str, f32)> = ph2d_field::dims(&p)
            .iter()
            .filter(|d| d.key == "field.dim.round" || d.key == "field.dim.chamfer")
            .map(|d| (d.key, d.value))
            .collect();
        assert!(
            ph2d_field::scale_primitive(&mut p, FATOR),
            "{k:?}: a escala foi recusada"
        );
        let depois: Vec<(&str, f32)> = ph2d_field::dims(&p)
            .iter()
            .filter(|d| d.key == "field.dim.round" || d.key == "field.dim.chamfer")
            .map(|d| (d.key, d.value))
            .collect();
        for ((chave, a), (_, b)) in antes.iter().zip(depois.iter()) {
            // ⚠️ Um recuo que já era zero continua zero — a régua é a RAZÃO, e ela só existe
            // sobre um número que estava ligado.
            if *a <= 0.0 {
                continue;
            }
            if ((b / a) - FATOR).abs() > 1e-3 {
                paradas.push(format!("{k:?}/{chave} {a:.5} -> {b:.5} (x{:.3})", b / a));
            }
        }
    }
    assert!(
        testadas >= 20,
        "só {testadas} formas com aresta — a lista derivada partiu-se"
    );
    assert!(
        paradas.is_empty(),
        "estes recuos não acompanharam a escala da forma: {paradas:?}"
    );
}
