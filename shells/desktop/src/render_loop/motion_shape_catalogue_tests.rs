//! **O CATÁLOGO das formas** — que geometria cada `kind` cozinha, que rótulo o
//! menu mostra, e que knobs cada uma acende.
//!
//! Cortado do irmão `motion_shape_gen_tests.rs` no teto de LOC do shell, por
//! ASSUNTO: lá mora a PONTE (a chave de conteúdo, o store, o traço, o publish e o
//! que o nó lê de volta), aqui *o que cada forma É*. Declarado pelo pai como um
//! `#[path]`, então `super` é `render_loop::motion_shape_gen`.

use super::*;

/// O descritor NEUTRO — lido do MANIFESTO, nunca escrito à mão.
///
/// ⚠️ **Um literal aqui é a forma exacta de um param novo escapar a estes gates:** ele
/// compila até alguém acrescentar um campo, e nesse dia a correcção mais rápida é copiar
/// um número, o que prende o teste a um valor que o manifesto pode ter mudado. Lido pela
/// `ShapeParams::read` — a mesma porta do nó e do shell — o neutro é, por construção, o
/// que um documento sem nada autorado cozinha.
fn neutral() -> ShapeParams {
    ShapeParams::read(manifest_default)
}
use ph2d_node_motion_shape::{ALL_KINDS, KIND_LABELS, ShapeKind, ShapeParams};

/// **A que decide se a wave pode existir.** Rotear tudo pelo `cook()` só é seguro se
/// as oito formas que shipavam saírem iguais — um grafo salvo guarda o índice do
/// `kind`, e uma forma que mudasse calada reescreveria a arte de quem já a autorou.
///
/// O oráculo é o construtor CONGELADO (`build_shape_path_as_it_shipped`), não uma
/// re-derivação: comparar o `cook()` com uma segunda escrita da mesma receita
/// provaria que eu sei somar duas vezes, e não que o produto não mudou.
///
/// ⚠️ **E a igualdade NÃO é bit a bit, porque nunca foi — este gate achou uma
/// divergência que já existia.** O círculo tinha DUAS derivações do mesmo número:
/// `ellipse()` usa a constante literal `KAPPA` e `ellipse_sweep` calcula
/// `(4/3)·tan(α/4)`, que o doc do `round.rs` declara ser *"a generalização do
/// `KAPPA` (que é esse valor para 90°)"*. São o mesmo valor por dois caminhos, e
/// eles tinham deslizado no último bit: **1,7e-12 de handle**, ou 3e-10 relativo.
/// Esta wave colapsa as duas portas numa; o número mede o quanto elas estavam
/// separadas, não uma regressão introduzida.
///
/// Então a ESTRUTURA é afirmada exata (contagem, fechamento, regra de
/// preenchimento, espécie de vértice — onde um erro de tradução apareceria) e a
/// GEOMETRIA a uma barra MEDIDA. Um `assert_eq` de bits aqui teria reprovado uma
/// wave correta; uma tolerância folgada teria deixado passar um round-rect com o
/// canto errado.
///
/// A varredura é adversarial de propósito — `corner` no máximo (onde o round-rect
/// e o polígono divergiriam se os campos apendados não fossem neutros), `aspect`
/// nos dois lados de 1, `sides` nos extremos.
#[test]
fn the_eight_that_shipped_cook_exactly_what_they_cooked() {
    let (mut checked, mut worst) = (0usize, 0.0f64);
    for kind in [
        ShapeKind::Circle,
        ShapeKind::Square,
        ShapeKind::Ellipse,
        ShapeKind::Rectangle,
        ShapeKind::Polygon,
        ShapeKind::Star,
        ShapeKind::Heart,
        ShapeKind::Gear,
    ] {
        // ⚠️ **O `3.0` entrou quando a engrenagem passou a varrer só `corner = 0`** (a nota
        // acima): sem ele a varredura caía de 648 para 594 células e o piso de 600 reprovava —
        // e baixar o piso teria trocado uma perda de cobertura por um número menor. Um eixo a
        // mais custa nada e devolve mais do que a nota tirou.
        for size in [0.01f32, 1.0, 3.0, 7.5] {
            for aspect in [0.05f32, 1.0, 3.25] {
                for sides in [3u32, 6, 32] {
                    // ⚠️ **A engrenagem varre só `corner = 0`, e isso é uma NOTA, não uma
                    // folga.** Até 2026-08-19 o `corner` era inerte nela **e escondido pelo
                    // painel**, então nenhum documento pôde autorá-lo — a única rota era
                    // trocar o `kind` de uma caixa que já o tinha. Com o arredondamento geral
                    // ele passou a mover a engrenagem, e varrê-lo aqui seria comparar contra
                    // um construtor congelado que nunca soube dele. O que continua prendido
                    // é o que o artista de facto tinha: `corner = 0`.
                    let corners: &[f32] = if kind == ShapeKind::Gear {
                        &[0.0]
                    } else {
                        &[0.0, 0.37, 1.0]
                    };
                    for &corner in corners {
                        let p = ShapeParams {
                            kind,
                            size,
                            aspect,
                            sides,
                            corner,
                            ..neutral()
                        };
                        let now = build_shape_path(&p);
                        let then = build_shape_path_as_it_shipped(&p);
                        assert_eq!(
                            now.verts.len(),
                            then.verts.len(),
                            "{kind:?} size={size} aspect={aspect} sides={sides} corner={corner}"
                        );
                        assert_eq!(now.closed, then.closed, "{kind:?}");
                        assert_eq!(now.fill_rule, then.fill_rule, "{kind:?}");
                        // O FURO da engrenagem viaja num subcontorno: comparar so o
                        // contorno principal deixaria uma engrenagem macica passar
                        // por uma furada.
                        assert_eq!(
                            now.subpaths.len(),
                            then.subpaths.len(),
                            "{kind:?}: numero de subcontornos"
                        );
                        let (gn, gt) = (all_geometry(&now), all_geometry(&then));
                        assert_eq!(gn.len(), gt.len(), "{kind:?}: total de coordenadas");
                        for (i, (x, y)) in gn.iter().zip(&gt).enumerate() {
                            let d = (x - y).abs() / f64::from(size).max(1e-3);
                            worst = worst.max(d);
                            assert!(
                                d < 1e-8,
                                "{kind:?} coord {i} size={size} aspect={aspect} corner={corner}: \
                                 desvio relativo {d:e} — isto e traducao errada, nao arredondamento"
                            );
                        }
                        for (i, (a, b)) in now.verts.iter().zip(&then.verts).enumerate() {
                            assert_eq!(
                                a.kind, b.kind,
                                "{kind:?} vert {i}: a ESPECIE do vertice tem de bater"
                            );
                        }
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(
        checked >= 600,
        "a varredura tem de ser larga: {checked} celulas"
    );
    // O numero fica PINADO: se ele subir, alguem trocou uma receita e nao so um
    // arredondamento, e a mensagem diz de quanto.
    assert!(
        worst < 1e-8,
        "pior desvio relativo do catalogo antigo: {worst:e}"
    );
}

/// **Toda etiqueta do dropdown desenha alguma coisa.** As duas listas do nó (os
/// rótulos e as espécies) e a tradução do shell são três coisas que têm de
/// concordar, e nada no compilador as alinha — um rótulo a mais é uma linha do menu
/// que cozinha a forma do vizinho, e um a menos é uma forma inalcançável.
///
/// Também prova que a wave ENTREGA: uma contagem de espécies DISTINTAS do
/// `ph2d-vec-scene`, porque oito rótulos apontando para a mesma elipse seriam um
/// catálogo que só parece grande.
#[test]
fn every_label_names_a_kind_and_every_kind_draws() {
    use std::collections::BTreeSet;
    assert_eq!(
        KIND_LABELS.len(),
        ALL_KINDS.len(),
        "rotulo sem especie por tras (ou o contrario)"
    );
    let mut vec_kinds = BTreeSet::new();
    for (i, k) in ALL_KINDS.iter().enumerate() {
        assert_eq!(
            ShapeKind::from_index(i as f32),
            *k,
            "o indice {i} nao devolve a especie {k:?}"
        );
        assert_eq!(k.index(), i, "e a volta tem de fechar");
        let p = ShapeParams {
            kind: *k,
            aspect: 1.3,
            corner: 0.2,
            ..neutral()
        };
        let path = build_shape_path(&p);
        assert!(
            path.verts.len() >= 2,
            "{k:?} ({}) nao desenhou nada",
            KIND_LABELS[i]
        );
        assert!(
            path.closed,
            "{k:?} e ABERTA — so as preenchiveis entram nesta wave (as cinco de traco esperam)"
        );
        assert!(
            path.verts
                .iter()
                .all(|v| v.anchor[0].is_finite() && v.anchor[1].is_finite()),
            "{k:?} produziu coordenada nao-finita"
        );
        vec_kinds.insert(format!("{:?}", vec_recipe(&p).0));
    }
    assert!(
        vec_kinds.len() >= 41,
        "o catalogo tem de ser de especies DISTINTAS, e sao {}",
        vec_kinds.len()
    );
}

/// **Nenhuma espécie esconde um controlo vivo nem mostra um morto.** Os `ParamGate`
/// decidem que sliders o painel pinta por espécie, e a lista é escrita à mão — a
/// forma exata que apodrece quando o catálogo cresce.
///
/// O oráculo não conhece a tabela: para cada espécie e cada param gateado, ele MEXE
/// no número e olha se a geometria mudou. Se mudou, o param tem de estar visível
/// (senão é um controlo vivo escondido, e o artista conclui que a forma não se
/// ajusta); se não mudou, não pode estar (o botão morto que este codebase recusa).
#[test]
fn no_kind_hides_a_live_knob_or_shows_a_dead_one() {
    let base = |kind: ShapeKind| ShapeParams { kind, ..neutral() };
    // Um valor claramente diferente por param — o suficiente para a geometria
    // responder se ela responde de todo.
    /// O nome do param e o empurrão que ele leva — nomeado porque a tupla crua
    /// dispara o `type_complexity` do clippy, e um `✗` de lint bloqueia o ship.
    /// O nome do param, o CONTEXTO em que ele é medido, e o empurrão que ele leva.
    ///
    /// ⚠️ **O contexto corre nos DOIS lados da comparação, e sem ele o gate atribui mal.**
    /// A suavização só tem o que suavizar sobre um canto REDONDO, então o empurrão dela
    /// precisa de um `corner > 0` ao lado — mas se esse `corner` entrasse só no lado
    /// empurrado, o gate mediria *"corner + smoothing"* e concluía que a suavização muda o
    /// POLÍGONO (que responde ao `corner` e não a ela). Foi exactamente o que ele acusou.
    /// Nomeado porque a tupla crua dispara o `type_complexity` do clippy.
    type Nudge = (&'static str, fn(&mut ShapeParams), fn(&mut ShapeParams));
    fn nothing(_: &mut ShapeParams) {}
    let nudge: &[Nudge] = &[
        ("aspect", nothing, |p| p.aspect = 2.5),
        ("sides", nothing, |p| p.sides = 11),
        ("corner", nothing, |p| p.corner = 0.6),
        ("star_depth", nothing, |p| p.star_depth = 0.85),
        ("cleft", nothing, |p| p.cleft = 0.42),
        ("tooth_depth", nothing, |p| p.tooth_depth = 0.55),
        ("hole", nothing, |p| p.hole = 0.9),
        ("sweep", nothing, |p| p.sweep = 140.0),
        ("start", nothing, |p| p.start = 55.0),
        ("inner", nothing, |p| p.inner = 0.5),
        ("corner_tr", nothing, |p| p.corner_offsets[0] = 0.4),
        ("corner_br", nothing, |p| p.corner_offsets[1] = 0.4),
        ("corner_bl", nothing, |p| p.corner_offsets[2] = 0.4),
        ("smoothing", |p| p.corner = 0.5, |p| p.smoothing = 0.8),
    ];
    let same = |a: &ph2d_vec_scene::VecPath, b: &ph2d_vec_scene::VecPath| {
        all_geometry(a) == all_geometry(b)
    };
    for (i, kind) in ALL_KINDS.iter().enumerate() {
        for (name, context, apply) in nudge {
            let mut before = base(*kind);
            context(&mut before);
            let untouched = build_shape_path(&before);
            let mut p = before;
            apply(&mut p);
            let live = !same(&build_shape_path(&p), &untouched);
            let shown = param_gate_shows(name, i);
            assert_eq!(
                live,
                shown,
                "{} ({:?}): o slider `{name}` {} e o painel {}",
                KIND_LABELS[i],
                kind,
                if live { "MUDA a forma" } else { "nao faz nada" },
                if shown { "mostra-o" } else { "esconde-o" },
            );
        }
    }
}

/// Toda a geometria de um path como uma lista chata de números — os vértices do
/// contorno principal E os dos SUBCONTORNOS.
///
/// ⚠️ **Ler só `verts` foi um oráculo cego, e ele já me mentiu uma vez.** O furo de
/// uma engrenagem e o miolo de uma rosquinha viajam em `subpaths` (compound path),
/// não no contorno principal, então um comparador que os ignora reporta *"mexer no
/// `hole` não muda nada"* sobre um knob perfeitamente VIVO — e, pior, deixaria
/// passar o dia em que o furo desaparecesse de verdade.
fn all_geometry(p: &ph2d_vec_scene::VecPath) -> Vec<f64> {
    let mut out = Vec::new();
    let mut push = |vs: &[ph2d_vec_scene::VecVertex]| {
        for v in vs {
            out.extend([
                v.anchor[0],
                v.anchor[1],
                v.in_handle[0],
                v.in_handle[1],
                v.out_handle[0],
                v.out_handle[1],
            ]);
        }
    };
    push(&p.verts);
    for c in &p.subpaths {
        push(&c.verts);
    }
    out
}

/// O painel mostra `name` para a espécie de índice `idx`? Lê a MESMA tabela que o
/// registry entrega ao painel — uma cópia aqui seria a segunda lista a driftar.
fn param_gate_shows(name: &str, idx: usize) -> bool {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_motion_shape::register(&mut reg).unwrap();
    let gates = reg
        .param_gates(ph2d_node_motion_shape::MANIFEST.id)
        .unwrap_or(&[]);
    match gates.iter().find(|g| g.param == name) {
        // Sem porta = sempre visível.
        None => true,
        Some(g) => g.values.contains(&(idx as i32)),
    }
}

/// **OS SETE KNOBS NOVOS, INTOCADOS, COZINHAM O QUE A BIBLIOTECA JÁ COZINHAVA** (doc 89
/// folha 14 — as linhas do *sweep/start/inner* e do *raio por canto*).
///
/// ⚠️ **A byte-identidade não é automática aqui, e é por isso que este gate existe:** o
/// `sweep` tem uma SENTINELA (`0` = *"como a forma nasce"*) porque as espécies discordam
/// sobre o que é neutro — a elipse passava `0` (que a biblioteca lê como volta inteira) e a
/// pizza passava o ângulo canónico dela. Um default único teria quebrado uma das duas, **e
/// em silêncio**: uma pizza que vira círculo continua a desenhar uma forma bonita.
///
/// O oráculo é a própria biblioteca: para toda espécie do catálogo, os valores que a receita
/// monta a partir do descritor NEUTRO têm de ser exactamente os `defaults()` daquela espécie.
#[test]
fn the_seven_new_knobs_are_neutral_at_their_defaults() {
    for kind in ALL_KINDS {
        let (vk, _, _, v) = vec_recipe(&ShapeParams {
            kind: *kind,
            ..neutral()
        });
        // As oito primeiras têm braço próprio na receita (a `Circle` passa `[0,0,0]`, que a
        // biblioteca lê como volta inteira, e o round-rect passa `[r,0,0,0,0]`); o gate
        // `the_eight_that_shipped_cook_exactly_what_they_cooked` é quem as prende, contra o
        // construtor congelado. Aqui interessam as do CATÁLOGO, que corriam nos `defaults()`.
        if matches!(
            kind,
            ShapeKind::Circle
                | ShapeKind::Ellipse
                | ShapeKind::Square
                | ShapeKind::Rectangle
                | ShapeKind::Polygon
                | ShapeKind::Star
                | ShapeKind::Heart
                | ShapeKind::Gear
        ) {
            continue;
        }
        assert_eq!(
            v,
            vk.defaults().to_vec(),
            "{kind:?} deixou de correr nos valores canonicos da biblioteca"
        );
    }
}

/// **E o CONTROLE de que os knobs novos não são inertes na pizza e na corda** — sem ele o
/// gate acima é satisfeito por uma receita que ignora os params por completo.
///
/// ⚠️ É o par exigido pela lei desta casa: *ausência E presença*. A folha 14 dizia
/// *"a FORMA chegou, o CONTROLO não"*, e é precisamente a metade "controlo" que este mede.
#[test]
fn the_pie_and_the_segment_answer_to_the_new_knobs() {
    /// O nome do knob e o empurrão — nomeado porque a tupla crua dispara o `type_complexity`.
    type Move = (&'static str, fn(&mut ShapeParams));
    for kind in [ShapeKind::Pie, ShapeKind::Segment] {
        let base = build_shape_path(&ShapeParams { kind, ..neutral() });
        let moves: &[Move] = &[
            ("sweep", |p| p.sweep = 200.0),
            ("start", |p| p.start = 70.0),
        ];
        for (name, apply) in moves {
            let mut p = ShapeParams { kind, ..neutral() };
            apply(&mut p);
            assert_ne!(
                all_geometry(&build_shape_path(&p)),
                all_geometry(&base),
                "{kind:?}: o `{name}` tem de mover a forma"
            );
        }
    }
}

/// **SONDA — em que espécies o `corner` de facto MEXE?** É de onde sai a lista do
/// `ParamGate` dele, e ela é grande demais para se escrever de cabeça: a rota geral de
/// arredondamento (as Live Corners aplicadas depois do `cook`) tornou o knob vivo em quase
/// todo o catálogo, mas **não em todo** — uma elipse, uma rosquinha e uma pílula não têm
/// quina nenhuma.
///
/// Imprime `SIM`/`nao` por espécie. Rode com
/// `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop -- which_kinds_the_corner_moves --ignored --nocapture`.
#[test]
#[ignore = "sonda: imprime a tabela de que o ParamGate do `corner` e' derivado"]
fn which_kinds_the_corner_moves() {
    let mut live: Vec<&str> = Vec::new();
    let mut dead: Vec<&str> = Vec::new();
    for (i, kind) in ALL_KINDS.iter().enumerate() {
        let base = ShapeParams {
            kind: *kind,
            ..neutral()
        };
        let bumped = ShapeParams {
            corner: 0.4,
            ..base
        };
        if all_geometry(&build_shape_path(&bumped)) == all_geometry(&build_shape_path(&base)) {
            dead.push(KIND_LABELS[i]);
        } else {
            live.push(KIND_LABELS[i]);
        }
    }
    eprintln!("\n[corner] VIVO em {} especies:\n  {live:?}", live.len());
    eprintln!("\n[corner] morto em {} especies:\n  {dead:?}", dead.len());
    eprintln!(
        "\n  LEITURA: a lista MORTA e' a que o `ParamGate` do `corner` tem de EXCLUIR — e ela
  e' curta de proposito: sao as formas sem quina nenhuma."
    );
}
