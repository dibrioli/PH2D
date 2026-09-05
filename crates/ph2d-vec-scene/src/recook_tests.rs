//! Gates da porta única do re-cozimento em lugar ([`VecPath::replace_cooked`]).
//!
//! O gate que importa é o primeiro: ele nasceu **VERMELHO** contra o produto de 2026-07-22, onde
//! os dois re-cooks de texto faziam `*p = np` e a pilha de efeitos de um texto **desaparecia ao
//! digitar a letra seguinte**.

use crate::effect::{FxEntry, PathEffect};
use crate::fx_zigzag::ZigZagSpec;
use crate::{Contour, FillRule, Paint, Rgba8, StrokeSpec, VecPath, VecVertex};

fn zig() -> FxEntry {
    FxEntry::new(PathEffect::ZigZag(ZigZagSpec {
        amplitude: 12.0,
        ridges: 7.0,
        ..ZigZagSpec::default()
    }))
}

/// Um path com estilo e pilha, como o texto vivo tem depois de o artista aplicar um efeito.
fn authored() -> VecPath {
    VecPath {
        id: 77,
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([1.0, 0.0])],
        closed: false,
        fill: Some(Paint::solid(Rgba8::new(10, 20, 30, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.5)),
        subpaths: Vec::new(),
        fill_rule: FillRule::NonZero,
        effects: vec![zig()],
        // ⭐ **Fora do neutro de propósito** (v19): é o que dá dentes ao gate da sobrevivência
        // logo abaixo — no neutro, um `replace_cooked` que as levasse de `next` leria os mesmos
        // valores e o gate ficaria verde sobre o defeito.
        opacity: crate::Opacity::new(0.4),
        blend: ph2d_blend_mode::BlendMode::Multiply,
    }
}

/// O que um re-cozimento produz: geometria e estilo novos, `id` de ninguém e pilha VAZIA — é
/// exatamente a forma que `text_to_compound_path` devolve (`..Default::default()`).
fn freshly_cooked() -> VecPath {
    VecPath {
        id: 0,
        verts: vec![
            VecVertex::corner([5.0, 5.0]),
            VecVertex::corner([6.0, 7.0]),
            VecVertex::corner([8.0, 9.0]),
        ],
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(200, 100, 50, 255))),
        stroke: None,
        subpaths: vec![Contour {
            verts: vec![VecVertex::corner([1.0, 1.0])],
            closed: true,
        }],
        fill_rule: FillRule::EvenOdd,
        effects: Vec::new(),
        // Um cozimento nasce de um `..VecPath::default()`, logo NEUTRO nos dois.
        opacity: crate::Opacity::default(),
        blend: ph2d_blend_mode::BlendMode::default(),
    }
}

/// **O gate red-first.** Re-cozinhar a geometria de um path NÃO pode levar a pilha de efeitos
/// junto: `effects` é dado AUTORADO sobre a forma, e a forma cozida é a ENTRADA da pilha, não a
/// saída dela (ADR-0121).
///
/// Contra o produto anterior (`*p = np`) isto falha com `0` efeitos.
#[test]
fn a_recook_preserves_the_effects_stack() {
    let mut p = authored();
    p.replace_cooked(freshly_cooked());
    assert_eq!(
        p.effects,
        vec![zig()],
        "a pilha de efeitos tem de sobreviver a um re-cozimento de geometria"
    );
}

/// A identidade é do path que JÁ ESTÁ na cena — o `id` do recém-cozido é lixo de construtor.
/// Antes, cada chamador escrevia `np.id = id` à mão antes do `*p = np`; esquecê-lo trocava a
/// forma de identidade e a entidade/seleção/gizmo perdiam-na.
#[test]
fn a_recook_keeps_the_paths_own_identity() {
    let mut p = authored();
    p.replace_cooked(freshly_cooked());
    assert_eq!(
        p.id, 77,
        "o id do path na cena manda; o do cozido é descartado"
    );
}

/// ⭐⭐⭐ **A OPACIDADE E A MISTURA DO OBJECTO SOBREVIVEM** (v19) — a mesma lei da pilha de efeitos,
/// e o gate que a `line/Vector` deve ao destructuring exaustivo do módulo.
///
/// ⚠️ **O sintoma sem ele é mudo e caro:** um cozimento nasce de `VecPath::default()`, então lê-las
/// de `next` repunha o neutro — **reescrever o texto de uma forma a 40% devolvia-a opaca**, e o
/// artista teria de descobrir sozinho que foi a edição que apagou a autoria.
#[test]
fn a_recook_preserves_the_objects_opacity_and_blend() {
    let mut p = authored();
    p.replace_cooked(freshly_cooked());
    assert_eq!(
        p.opacity,
        crate::Opacity::new(0.4),
        "a opacidade do objecto e' autoria, nao produto do cozimento"
    );
    assert_eq!(
        p.blend,
        ph2d_blend_mode::BlendMode::Multiply,
        "e o modo de mistura tambem"
    );
}

/// A outra metade da lei: tudo o que o re-cozimento PRODUZ é substituído — senão a "cura" seria
/// um re-cook que não re-cozinha. Sem este gate, uma implementação que só copiasse `verts`
/// passaria no gate da pilha.
#[test]
fn a_recook_replaces_every_field_the_cooking_produces() {
    let mut p = authored();
    let next = freshly_cooked();
    p.replace_cooked(next.clone());
    assert_eq!(p.verts, next.verts, "geometria");
    assert_eq!(p.closed, next.closed, "fechamento");
    assert_eq!(p.subpaths, next.subpaths, "contornos extras");
    assert_eq!(p.fill_rule, next.fill_rule, "regra de preenchimento");
    assert_eq!(p.fill, next.fill, "preenchimento");
    assert_eq!(p.stroke, next.stroke, "traço");
}

/// Um path SEM pilha re-cozinha exatamente como antes — o caminho comum não muda de
/// comportamento. Pinado para que a cura não seja lida como mudança de produto.
#[test]
fn a_recook_of_a_path_without_effects_is_the_plain_replacement() {
    let mut p = authored();
    p.effects.clear();
    let next = freshly_cooked();
    p.replace_cooked(next.clone());

    let mut expected = next;
    expected.id = 77;
    // ⭐ v19: a autoria do OBJECTO vem de casa, como o id — a lista de quem sobrevive cresceu, e
    // este gate é onde ela se lê inteira.
    expected.opacity = crate::Opacity::new(0.4);
    expected.blend = ph2d_blend_mode::BlendMode::Multiply;
    assert_eq!(
        p, expected,
        "sem pilha, o resultado é o path cozido com o id, a opacidade e a mistura de casa"
    );
}
