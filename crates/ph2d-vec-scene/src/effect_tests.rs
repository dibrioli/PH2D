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
            // Um valor que NÃO é o default nem o 0, para um `set` inerte não passar.
            let v = if p.toggle { 1.0 } else { p.min.midpoint(p.max) };
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

/// Um índice de parâmetro fora da faixa é **no-op**, não pânico nem escrita no vizinho.
#[test]
fn an_out_of_range_parameter_index_is_inert() {
    let mut fx = PathEffect::from_kind(0).expect("kind");
    let before = fx.clone();
    fx.set(MAX_FX_PARAMS + 3, 0.7);
    assert_eq!(fx, before);
    assert_eq!(fx.get(MAX_FX_PARAMS + 3), 0.0);
}
