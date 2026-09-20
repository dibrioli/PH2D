//! Gates da porta única do re-cozimento em lugar ([`VecPath::replace_cooked`]).
//!
//! O gate que importa é o primeiro: ele nasceu **VERMELHO** contra o produto de 2026-07-22, onde
//! os dois re-cooks de texto faziam `*p = np` e a pilha de efeitos de um texto **desaparecia ao
//! digitar a letra seguinte**.

use crate::effect::{FxEntry, PathEffect};
use crate::fx_zigzag::ZigZagSpec;
use crate::{Contour, FillRule, Paint, Rgba8, StrokeSpec, VecPath, VecVertex};

/// Um contorno EXTRA — a camada que a v20 acrescentou à forma.
fn segundo_traco() -> crate::PaintEntry {
    crate::PaintEntry::stroke(StrokeSpec::new(Rgba8::new(250, 250, 250, 255), 4.0))
}

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
        // ⭐ **Idem para a PILHA DE APARÊNCIA** (v20), e pela mesma razão: um contorno extra aqui
        // é o que deixa o gate da sobrevivência ver a diferença entre «sobreviveu» e «leu o
        // vazio de `next`».
        paints: vec![segundo_traco()],
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
        paints: Vec::new(),
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
    // ⭐ v20: e a PILHA DE APARÊNCIA também. ⚠️ **Este gate é o censo de quem vem de casa** — cada
    // campo novo que sobreviva ao cozimento tem de aparecer aqui, e é ele que impede a lista de
    // crescer sem ninguém a ler.
    expected.paints = vec![segundo_traco()];
    assert_eq!(
        p, expected,
        "sem pilha de efeitos, o resultado e' o path cozido com o id, a opacidade, a mistura e a \
         aparencia de casa"
    );
}

/// ⭐⭐⭐ **A PILHA DE APARÊNCIA SOBREVIVE ao re-cozimento** (v20) — a terceira lei da mesma família
/// (efeitos, opacidade/mistura, aparência), e a que o destructuring exaustivo obrigou a escrever.
///
/// ⚠️ Sem ela o sintoma é mudo: **reescrever o texto de uma forma com dois contornos devolve-a com
/// um só**, porque o cozimento nasce de um `VecPath::default()` e a lista dele está vazia.
#[test]
fn a_recook_preserves_the_appearance_stack() {
    let mut p = authored();
    p.replace_cooked(freshly_cooked());
    assert_eq!(
        p.paints,
        vec![segundo_traco()],
        "a pilha de aparencia e' autoria, nao produto do cozimento"
    );
}

/// ⭐⭐⭐ **A PORTA DA GEOMETRIA LEVA AS POSIÇÕES E NÃO TOCA NO ESTILO** — o gate red-first do report
/// do dono de 2026-09-19 (*«num vector linkado aos ossos não consigo mudar a espessura do stroke»*).
///
/// ⚠️ **Ele é o irmão INVERSO do [`a_recook_preserves_the_effects_stack`]**, e as duas metades são
/// obrigatórias: sem a 1.ª asserção, uma [`VecPath::replace_geometry`] que não escrevesse nada
/// passaria; sem a 2.ª, a de ontem — que trazia o estilo da fotografia do `Bind` — passaria
/// também. *Uma porta que se define pelo que NÃO faz precisa de um gate que veja as duas coisas.*
///
/// ⚠️ **A fixtura tem os dois lados FORA do neutro de propósito** (a autorada tem traço de `0,5` e
/// preenchimento, a cozida tem `stroke: None` e outro preenchimento): com os dois iguais este gate
/// ficaria verde sobre a porta errada.
#[test]
fn replace_geometry_carries_the_points_and_leaves_the_style_alone() {
    let mut p = authored();
    let antes = p.clone();
    p.replace_geometry(freshly_cooked());

    // (1) a GEOMETRIA veio — senão isto seria um no-op a passar por lei.
    let cozido = freshly_cooked();
    assert_eq!(p.verts, cozido.verts, "as posicoes tinham de vir do `next`");
    assert_eq!(p.closed, cozido.closed, "o fecho e' geometria");
    assert_eq!(
        p.subpaths, cozido.subpaths,
        "os contornos extra sao geometria"
    );

    // (2) e NADA de estilo se mexeu — é a metade que o report do dono é.
    assert_eq!(
        p.stroke, antes.stroke,
        "o traco e' autoria do OBJECTO: uma re-escrita de posicoes que o leve desfaz, no quadro \
         seguinte, toda edicao de espessura que o artista fizer numa forma presa aos ossos"
    );
    assert_eq!(p.fill, antes.fill, "o preenchimento idem");
    assert_eq!(
        p.fill_rule, antes.fill_rule,
        "a regra de preenchimento idem"
    );
    assert_eq!(
        p.id, antes.id,
        "quem manda na identidade e' o path que ja' esta' na cena"
    );
    assert_eq!(p.effects, antes.effects, "a pilha de efeitos idem");
    assert_eq!(p.paints, antes.paints, "a pilha de aparencia idem");
    assert_eq!(p.opacity, antes.opacity, "a opacidade idem");
    assert_eq!(p.blend, antes.blend, "a mistura idem");
}
