//! Gates da PILHA (ADR-0132) — as propriedades de que tudo o resto depende:
//! o neutro não custa nada, a ordem importa, e cozinhar duas vezes é cozinhar uma.

use super::*;
use crate::{VecPath, VecVertex};

/// Um quadrado fechado com lados de 40 — números do produto, não `1.0`.
fn square() -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// O comprimento total do contorno primário de `p`, medido pelo motor de arco.
fn len_of(p: &VecPath) -> f64 {
    let n = p.verts.len();
    if n < 2 {
        return 0.0;
    }
    let segs = if p.closed { n } else { n - 1 };
    (0..segs)
        .map(|i| crate::arclen::arclen(&crate::corner_live::segment(&p.verts, i, n)))
        .sum()
}

/// **Pilha vazia = MESMO PONTEIRO.** É a propriedade que permitiu ligar o `cooked()` em
/// todo consumidor sem mudar comportamento, e ela não pode morrer com esta feature.
#[test]
fn an_empty_stack_still_borrows_the_source() {
    let p = square();
    let c = p.cooked();
    assert!(
        matches!(c, std::borrow::Cow::Borrowed(_)),
        "sem raio e sem efeito o `cooked()` tem de emprestar, não alocar"
    );
    assert!(std::ptr::eq(&*c, &p), "e tem de ser o MESMO ponteiro");
}

/// **Uma pilha inteiramente NEUTRA também empresta.**
///
/// Sem isto, abrir a seção "Effects" no painel e não configurar nada custaria uma alocação
/// por frame a todo documento — o efeito neutro tem de ser saltado, não executado.
#[test]
fn a_stack_of_neutral_effects_still_borrows() {
    let mut p = square();
    p.effects = vec![
        FxEntry::new(PathEffect::Trim(crate::fx_trim::TrimSpec::default())),
        FxEntry::new(PathEffect::Trim(crate::fx_trim::TrimSpec::default())),
    ];
    let c = p.cooked();
    assert!(
        matches!(c, std::borrow::Cow::Borrowed(_)),
        "dois Trims no ponto neutro não fazem nada e não podem custar nada"
    );
}

/// **A ORDEM da pilha muda o resultado** — e por isso "reordenar por arrastar" é uma
/// feature, não um enfeite.
///
/// Dois Trims bastam para provar: pegar a 1ª metade e depois a 2ª metade *dela* dá o
/// trecho `[0.25, 0.5]` do original; na ordem trocada dá `[0.5, 0.75]`. O oráculo é a
/// POSIÇÃO onde o trecho começa, não uma regra da implementação.
#[test]
fn the_order_of_the_stack_changes_the_geometry() {
    let first_half = PathEffect::Trim(crate::fx_trim::TrimSpec {
        start: 0.0,
        end: 0.5,
        offset: 0.0,
    });
    let second_half = PathEffect::Trim(crate::fx_trim::TrimSpec {
        start: 0.5,
        end: 1.0,
        offset: 0.0,
    });

    let mut a = square();
    a.effects = vec![
        FxEntry::new(first_half.clone()),
        FxEntry::new(second_half.clone()),
    ];
    let mut b = square();
    b.effects = vec![FxEntry::new(second_half), FxEntry::new(first_half)];

    let (ca, cb) = (a.cooked(), b.cooked());
    let (sa, sb) = (ca.verts[0].anchor, cb.verts[0].anchor);
    let apart = (sa[0] - sb[0]).hypot(sa[1] - sb[1]);
    assert!(
        apart > 1.0,
        "as duas ordens começam em {sa:?} e {sb:?} — a {apart} de distância. Se a pilha \
         fosse comutativa a ordem seria decoração, e o painel estaria a mentir ao oferecer \
         reordenação."
    );
}

/// **Cozinhar duas vezes é cozinhar uma.**
///
/// O `corner_live` garante isto zerando o raio do que emite; a pilha garante esvaziando a
/// si mesma na saída. Sem isso, um consumidor que chamasse `cooked()` sobre um resultado já
/// cozido encolheria a forma outra vez — e nada pareceria quebrado.
#[test]
fn cooking_the_cooked_path_changes_nothing() {
    let mut p = square();
    p.effects = vec![FxEntry::new(PathEffect::Trim(crate::fx_trim::TrimSpec {
        start: 0.1,
        end: 0.6,
        offset: 0.0,
    }))];
    let once = p.cooked().into_owned();
    let twice = once.cooked().into_owned();
    assert_eq!(once, twice, "a 2ª passagem tem de ser a identidade");
    assert!(
        once.effects.is_empty(),
        "a saída cozida não pode continuar a carregar a pilha"
    );
}

/// **O efeito é NÃO-DESTRUTIVO**: a fonte continua intacta depois de cozinhar.
#[test]
fn the_authored_source_survives_the_cook() {
    let mut p = square();
    p.effects = vec![FxEntry::new(PathEffect::Trim(crate::fx_trim::TrimSpec {
        start: 0.0,
        end: 0.25,
        offset: 0.0,
    }))];
    let before = p.verts.clone();
    let cooked = p.cooked().into_owned();
    assert_eq!(p.verts, before, "a fonte autorada não pode ser tocada");
    assert!(
        len_of(&cooked) < len_of(&p) * 0.5,
        "e mesmo assim o cozido tem de ser mais curto"
    );
}

/// **A ordem de DOIS TIPOS diferentes muda a geometria — e agora isto é o caso real.**
///
/// O gate acima usa dois Trims (a ordem importa, mas é a mesma operação). Com o Zig Zag a
/// pergunta fica concreta: **ondular e depois cortar** dá um pedaço da onda; **cortar e depois
/// ondular** ondula só o pedaço, e a contagem de cristas é a do caminho CURTO. As duas leituras
/// são legítimas — é por isso que a ordem é do artista e não nossa.
#[test]
fn zigzag_then_trim_is_not_trim_then_zigzag() {
    let zz = PathEffect::ZigZag(crate::fx_zigzag::ZigZagSpec {
        amplitude: 6.0,
        ridges: 8.0,
        smooth: false,
        rough_seed: None,
    });
    let trim = PathEffect::Trim(crate::fx_trim::TrimSpec {
        start: 0.0,
        end: 0.5,
        offset: 0.0,
    });

    let mut a = square();
    a.effects = vec![FxEntry::new(zz.clone()), FxEntry::new(trim.clone())];
    let mut b = square();
    b.effects = vec![FxEntry::new(trim), FxEntry::new(zz)];

    let (ca, cb) = (a.cooked().into_owned(), b.cooked().into_owned());
    assert_ne!(
        ca.verts, cb.verts,
        "ondular-depois-cortar e cortar-depois-ondular deram a MESMA geometria — se a pilha \
         comutasse, oferecer reordenação seria mentira"
    );
    // E os dois produzem geometria de verdade: um `assert_ne!` entre dois vazios passaria.
    assert!(ca.verts.len() > 3 && cb.verts.len() > 3);
}

/// **Um efeito NEUTRO no meio da pilha não muda nada** — nem a geometria, nem o resultado dos
/// vizinhos. É o que permite ao artista desarmar um efeito sem o remover.
#[test]
fn a_neutral_effect_in_the_middle_of_the_stack_is_transparent() {
    let zz = PathEffect::ZigZag(crate::fx_zigzag::ZigZagSpec {
        amplitude: 5.0,
        ridges: 6.0,
        smooth: true,
        rough_seed: None,
    });
    let sleeping = PathEffect::Trim(crate::fx_trim::TrimSpec::default()); // neutro

    let mut only = square();
    only.effects = vec![FxEntry::new(zz.clone())];
    let mut sandwiched = square();
    sandwiched.effects = vec![
        FxEntry::new(sleeping.clone()),
        FxEntry::new(zz),
        FxEntry::new(sleeping),
    ];

    assert_eq!(
        only.cooked().into_owned().verts,
        sandwiched.cooked().into_owned().verts,
        "um efeito no ponto neutro tem de ser INVISÍVEL para os vizinhos"
    );
}

/// **A tabela de tipos cobre TODOS os variants** — se alguém acrescentar um efeito e esquecer
/// a linha em `KINDS`, ele fica inalcançável pelo menu "Add" do painel: existe no motor, e o
/// artista nunca o vê. É o gate que substitui a rodada de costura que o Zig Zag custou.
#[test]
fn every_effect_kind_is_reachable_from_the_add_table() {
    for (i, name) in PathEffect::KINDS.iter().enumerate() {
        let fx = PathEffect::from_kind(i)
            .unwrap_or_else(|| panic!("KINDS[{i}] = {name}, mas `from_kind({i})` devolveu None"));
        assert_eq!(fx.kind_index(), i, "{name}: o índice não fecha a volta");
        assert_eq!(fx.label(), *name, "{name}: o rótulo diverge da tabela");
    }
    assert!(
        PathEffect::from_kind(PathEffect::KINDS.len()).is_none(),
        "um índice fora da tabela tem de devolver None, não o efeito 0"
    );
}

/// **Todo efeito nasce NEUTRO** — o clique em "Add" não pode mover um pixel.
#[test]
fn every_kind_is_born_neutral() {
    for i in 0..PathEffect::KINDS.len() {
        let fx = PathEffect::from_kind(i).expect("kind");
        assert!(
            fx.is_neutral(),
            "{} nasce a fazer alguma coisa — o clique em Add saltaria a forma",
            fx.label()
        );
    }
}

/// **Os parâmetros cabem no teto que o painel registra**, e cada um faz a volta
/// `set` → `get`. Sem isto, um parâmetro além do teto fica invisível e um `set` que escreve no
/// campo errado passa despercebido.
#[test]
fn every_parameter_round_trips_within_the_panel_ceiling() {
    for i in 0..PathEffect::KINDS.len() {
        let mut fx = PathEffect::from_kind(i).expect("kind");
        let params = fx.params();
        assert!(
            params.len() <= MAX_FX_PARAMS,
            "{} declara {} params e o painel só registra {MAX_FX_PARAMS}",
            fx.label(),
            params.len()
        );
        for (j, p) in params.iter().enumerate() {
            // Um valor que NÃO é o default nem o 0, para um `set` inerte não passar. Num
            // parâmetro de CONTAGEM o valor entra já redondo — o `set` arredonda de propósito,
            // e um meio-termo faria este gate reprovar a regra em vez de a verificar.
            let mid = p.min.midpoint(p.max);
            let v = if p.toggle {
                1.0
            } else if p.integer {
                mid.round()
            } else {
                mid
            };
            fx.set(j, v);
            assert!(
                (fx.get(j) - v).abs() < 1e-12,
                "{}::{}: escrevi {v}, li {}",
                fx.label(),
                p.name,
                fx.get(j)
            );
        }
    }
}

/// **Um parâmetro de CONTAGEM guarda um inteiro no documento.**
///
/// O gate acima verifica o round-trip *respeitando* a declaração; este verifica a declaração em
/// si. Sem ele, marcar `integer: true` seria uma etiqueta que ninguém honra: o slider entregaria
/// `37,42`, o chip mostraria `37` e a geometria desenharia `37` — três respostas para o mesmo
/// número, e o artista só veria a discordância ao arrastar.
#[test]
fn a_count_parameter_is_stored_rounded() {
    let mut found = 0;
    for i in 0..PathEffect::KINDS.len() {
        let mut fx = PathEffect::from_kind(i).expect("kind");
        for (j, p) in fx.params().iter().enumerate() {
            if !p.integer {
                continue;
            }
            found += 1;
            // Um valor com casas, dentro da faixa e longe da borda.
            let messy = p.min.midpoint(p.max) + 0.42;
            fx.set(j, messy);
            let got = fx.get(j);
            assert!(
                (got - got.round()).abs() < 1e-12,
                "{}::{}: escrevi {messy} e o documento guardou {got}, que não é inteiro",
                fx.label(),
                p.name
            );
            assert!(
                (got - messy.round()).abs() < 1e-12,
                "{}::{}: {messy} devia arredondar para {}, deu {got}",
                fx.label(),
                p.name,
                messy.round()
            );
        }
    }
    assert!(
        found > 0,
        "nenhum parâmetro de contagem — o gate estaria a dormir"
    );
}

/// Um índice de parâmetro fora da faixa é **no-op**, não pânico nem escrita no vizinho.
#[test]
fn an_out_of_range_parameter_index_is_inert() {
    let mut fx = PathEffect::from_kind(0).expect("kind");
    let before = fx.clone();
    fx.set(MAX_FX_PARAMS + 3, 0.7);
    assert_eq!(fx, before);
    assert_eq!(fx.get(MAX_FX_PARAMS + 3), 0.0);
}

/// **A caixa do gizmo segue o EFEITO, e um caminho sem geometria não tem caixa.**
///
/// `path_curve_bbox` semeava o min/max com a âncora **crua** e depois varria pontos
/// **cozidos** — duas fontes na mesma caixa. Com raio de quina vivo o erro era o canto
/// cortado; com a pilha, a âncora crua pode cair fora da forma inteira, e o gizmo passa a
/// abraçar espaço vazio. O oráculo é a APARÊNCIA: aparar metade do caminho tem de encolher a
/// caixa, porque metade do desenho deixou de existir.
#[test]
fn the_curve_box_hugs_the_cooked_shape_and_an_empty_one_has_none() {
    let full = square();
    let (lo0, hi0) = {
        let mut s = crate::VecScene::new();
        let id = s.push_path(full.clone());
        s.path_curve_bbox(id).expect("a forma cheia tem caixa")
    };
    assert!((hi0[0] - lo0[0] - 40.0).abs() < 0.5, "o quadrado mede 40");

    // ⚠️ A fixture tem de conter o fenómeno. A 1ª versão aparava `[0, 0.25]` — a aresta de
    // BAIXO, que começa exatamente na âncora crua `(0,0)`: a semente errada coincidia com a
    // geometria certa e o gate ficava verde sobre o bug. O trecho `[0.5, 0.75]` é a aresta de
    // CIMA, e a âncora crua está a 40 dela — a distância que a caixa esticava.
    let mut trimmed = square();
    trimmed.effects = vec![FxEntry::new(PathEffect::Trim(crate::fx_trim::TrimSpec {
        start: 0.5,
        end: 0.75,
        offset: 0.0,
    }))];
    let mut s = crate::VecScene::new();
    let id = s.push_path(trimmed);
    let (lo, hi) = s.path_curve_bbox(id).expect("um quarto ainda é geometria");
    assert!(
        hi[1] - lo[1] < 0.5,
        "a aresta de cima é horizontal — a caixa dela tem de ser plana, e mede {} de altura. \
         Com a semente crua ela esticava até y=0 e media os 40 inteiros.",
        hi[1] - lo[1]
    );

    // Aparar TUDO: sem ponto desenhado não há caixa. `Some(caixa invertida)` seria pior que
    // `None` — quem chama compara `lo <= hi` e recebe geometria impossível em silêncio.
    let mut gone = square();
    gone.effects = vec![FxEntry::new(PathEffect::Trim(crate::fx_trim::TrimSpec {
        start: 0.5,
        end: 0.5,
        offset: 0.0,
    }))];
    let mut s = crate::VecScene::new();
    let id = s.push_path(gone);
    assert_eq!(
        s.path_curve_bbox(id),
        None,
        "caminho sem geometria não tem caixa"
    );
}

/// **O painel alcança TODO tipo de efeito** — o menu "Add" registra `MAX_FX_KINDS` botões e pinta
/// só `.take(MAX_FX_KINDS)` da tabela, então um `KINDS` maior que o teto do painel deixa os
/// últimos tipos INVISÍVEIS (existem no motor, o artista nunca os vê). Foi o que a família Warp
/// arriscou: `KINDS` foi de 4 para 9 e o teto do painel estava em 8.
///
/// A crate do motor não alcança a do painel (vive de snapshots), então o gate compara contra o
/// número LITERAL e a mensagem diz onde está o outro lado — o mesmo padrão do teto de parâmetros.
#[test]
fn the_engine_and_panel_agree_on_the_kind_ceiling() {
    // `ph2d_editor_core::ids::MAX_FX_KINDS`, em ids/chrome/vector.rs.
    const PANEL_MAX_FX_KINDS: usize = 9;
    assert!(
        PathEffect::KINDS.len() <= PANEL_MAX_FX_KINDS,
        "o motor publica {} tipos e o menu Add do painel só regista {PANEL_MAX_FX_KINDS} \
         (`MAX_FX_KINDS` em ph2d-editor-core/src/ids/chrome/vector.rs) — os últimos ficam \
         inalcançáveis",
        PathEffect::KINDS.len()
    );
}

/// **O teto de parâmetros do motor e o do painel têm de CONCORDAR.**
///
/// ⚠️ O doc do `MAX_FX_ROW_PARAMS` afirmava *"há gate a exigir que os dois lados concordem"* e
/// **não havia** (achado numa auditoria, 2026-07-18). Baixar o do painel deixaria os últimos
/// parâmetros de um efeito registados no motor, invisíveis na tela e verdes em todo o lado — que
/// é a definição de um botão que falta sem ninguém dar por isso.
///
/// A crate do motor não alcança a do painel (ela vive de snapshots), então o gate compara contra
/// o número LITERAL, e a mensagem diz onde está o outro lado.
#[test]
fn the_engine_and_the_panel_agree_on_the_parameter_ceiling() {
    // `ph2d_editor_core::ids::MAX_FX_ROW_PARAMS`, em ids/chrome/vector.rs.
    const PANEL_MAX_FX_ROW_PARAMS: usize = 6;
    assert_eq!(
        MAX_FX_PARAMS, PANEL_MAX_FX_ROW_PARAMS,
        "o motor declara {MAX_FX_PARAMS} parâmetros por efeito e o painel regista \
         {PANEL_MAX_FX_ROW_PARAMS} (`MAX_FX_ROW_PARAMS` em ph2d-editor-core/src/ids/chrome/\
         vector.rs). O menor dos dois é quantos o artista consegue tocar."
    );
}

/// Um quadrado com um Trim ATIVO (revela só o 1º quarto do contorno) — a cena mínima em que
/// assar de facto MUDA a geometria.
fn scene_with_active_trim() -> (crate::VecScene, crate::VecPathId) {
    let mut scene = crate::VecScene::new();
    let id = scene.push_path(square());
    let p = scene.path_mut(id).unwrap();
    p.effects = vec![FxEntry::new(PathEffect::Trim(crate::fx_trim::TrimSpec {
        start: 0.0,
        end: 0.25,
        offset: 0.0,
    }))];
    (scene, id)
}

/// **Apply / bake:** assar a pilha congela a aparência (`verts` = cozido), esvazia a pilha, e o
/// que se via não muda — é o *Expand Appearance*, e a base do botão **Apply** e do **Convert to
/// Curves** sobre efeitos.
#[test]
fn baking_effects_freezes_the_cooked_geometry_and_clears_the_stack() {
    let (mut scene, id) = scene_with_active_trim();
    // O que o mundo VÊ antes de assar (o quarto revelado — mais curto que o quadrado inteiro).
    let cooked = scene.path(id).unwrap().cooked().into_owned();
    assert!(
        len_of(&cooked) < len_of(&square()) * 0.5,
        "pré-condição: o Trim tem de encurtar de facto, senão o teste não prova nada"
    );

    assert!(scene.bake_cooked(id), "havia efeito ativo a assar");
    let p = scene.path(id).unwrap();
    assert!(
        p.effects.is_empty(),
        "a pilha tem de sair vazia depois do bake"
    );
    assert_eq!(p.verts, cooked.verts, "a geometria autorada vira a cozida");
    assert_eq!(
        p.id, id,
        "o id sobrevive ao bake — assar não recria o objeto"
    );
    // A aparência não muda: cozinhar o assado é a identidade (a pilha já foi consumida).
    assert_eq!(
        p.cooked().into_owned().verts,
        cooked.verts,
        "assar não pode alterar o que o mundo desenha"
    );
}

/// **O bake preserva `id`, `fill` e `stroke`** — o que o `cooked()` carrega por clonagem. Um
/// bake que perdesse a cor seria "o efeito sumiu junto com o desenho" do ponto de vista do artista.
#[test]
fn baking_effects_preserves_identity_and_style() {
    let (mut scene, id) = scene_with_active_trim();
    let fill = Some(crate::Paint::Solid(crate::Rgba8::new(10, 20, 30, 255)));
    let stroke = Some(crate::StrokeSpec::new(crate::Rgba8::new(9, 8, 7, 255), 3.0));
    {
        let p = scene.path_mut(id).unwrap();
        p.fill = fill.clone();
        p.stroke = stroke;
    }
    assert!(scene.bake_cooked(id));
    let p = scene.path(id).unwrap();
    assert_eq!(p.fill, fill, "o fill tem de sobreviver ao bake");
    assert_eq!(p.stroke, stroke, "o stroke tem de sobreviver ao bake");
}

/// **Idempotente, e recusa a pilha vazia.** Assar o resultado de volta é um no-op (`false`), e
/// um caminho sem efeito nenhum não é tocado — é o que faz o botão "Apply" não ser oferecido
/// onde não há o que assar, sem depender de o painel ter razão.
#[test]
fn baking_an_empty_stack_is_a_refused_no_op() {
    let (mut scene, id) = scene_with_active_trim();
    let baked_once = scene.bake_cooked(id);
    assert!(baked_once, "o 1º bake tinha o que assar");
    let after = scene.path(id).unwrap().clone();

    assert!(
        !scene.bake_cooked(id),
        "a pilha já está vazia — o 2º bake não tem o que fazer"
    );
    assert_eq!(
        &after,
        scene.path(id).unwrap(),
        "um bake recusado não pode mexer no caminho"
    );

    // Um caminho recém-criado, sem efeitos, também é recusado.
    let mut clean = crate::VecScene::new();
    let clean_id = clean.push_path(square());
    assert!(!clean.bake_cooked(clean_id), "sem pilha, nada a assar");
}
