//! **O CATÁLOGO DIZ A VERDADE** — os gates do NOME e do ORÁCULO EXTERNO.
//!
//! O irmão `brush_tests.rs` já pina as nove leis contra a transcrição do
//! `brush.cc`, e ⚠️ **ele era CEGO ao defeito que esta wave curou**: aquele gate
//! nomeia a VARIANTE, então uma lei do Blender escondida sob um identificador
//! que o painel nunca mostra o satisfaz igualmente. O que faltava era a outra
//! metade — *o rótulo que o artista lê é o que o Blender dá àquela lei* — e o
//! oráculo que só o Blender A CORRER pode dar.

use super::Falloff;

/// **O RÓTULO É O DO BLENDER, E ERA ELE QUE MENTIA.**
///
/// Até 2026-08-16 o painel pintava **"Smooth"** sobre `(1 − t²)²` e
/// **"Sharper"** sobre `(1 − t²)⁴` — duas curvas que referência nenhuma tem —
/// enquanto as leis do Blender viviam sob `Smoothstep` e `Pow4`, identificadores
/// que ele não usa em lugar nenhum da UI dele (`rna_brush.cc:86` rotula o
/// `BRUSH_CURVE_POW4` de *"Sharper"*).
///
/// ⚠️ **A mutação que este gate existe para matar é a barata:** devolver a lei
/// certa sob o rótulo errado. O gate das nove fórmulas fica **VERDE** sob ela —
/// ele pergunta *"esta lei existe no catálogo?"*, e a pergunta que decide o que
/// o artista escolhe é *"o nome em cima dela é o certo?"*.
#[test]
fn the_label_the_artist_reads_is_the_one_blender_gives_that_law() {
    // (rótulo na tela, a fórmula do `brush.cc` em `u = 1 − t`).
    #[allow(clippy::type_complexity)]
    let pairs: [(&str, fn(f32) -> f32); 4] = [
        ("Smooth", |u| 3.0 * u * u - 2.0 * u * u * u),
        ("Sharper", |u| u * u * u * u),
        ("Sharp", |u| u * u),
        ("Linear", |u| u),
    ];
    for (label, law) in pairs {
        let found = Falloff::ALL
            .into_iter()
            .find(|f| f.label() == label)
            .unwrap_or_else(|| panic!("nenhuma curva se chama {label}"));
        for k in 0..200 {
            let t = k as f32 / 200.0;
            let (got, want) = (found.weight(t), law(1.0 - t));
            assert!(
                (got - want).abs() < 1e-6,
                "a curva rotulada {label} em t={t}: {got} contra a lei do Blender {want}"
            );
        }
    }
}

/// **O QUE O BLENDER A CORRER DEPOSITA — o oráculo EXTERNO.**
///
/// A tabela abaixo é transcrita à mão de uma corrida de
/// `docs/3D/ferramentas/blender_sculpt_oracle.py` sobre o **Blender 5.2**:
/// pincel `DRAW` de fábrica, `unprojected_size = 1,0` (⚠️ **diâmetro** — o campo
/// foi renomeado no 5.x e o significado mudou junto ⇒ `R = 0,5`),
/// `strength = 1`, `hardness = 0`, malha plana subdividida 64. O pico medido é
/// `0,500000` em toda a varredura, então a coluna é `dz / 0,5`.
///
/// ⚠️ **É ele que justifica o `profile_b` declarar [`Falloff::Smooth`]:** a
/// leitura ESTÁTICA do `brush.cc` dizia que um pincel nasce
/// `BRUSH_CURVE_CUSTOM` com uma *curvemapping* semeada, logo *"nenhuma das
/// nove"* — e o Blender a correr reporta `curve_distance_falloff_preset =
/// SMOOTH`. *Um pincel não nasce zero-inicializado; ele nasce do arquivo de
/// startup.*
///
/// ⚠️ **A barra é `1e-4` e o DISCRIMINADOR está três ordens acima dela:** o
/// spline de quatro pontos que a leitura estática previa daria **0,940** em
/// `r/R ≈ 0,258` contra os **0,835** medidos, e a curva que este repo chamava de
/// "Smooth" até esta wave daria **0,871**. O resíduo de `~3e-5` que sobra é a
/// impressão da tabela (o `r` vem com cinco casas), não desacordo de lei.
#[test]
fn the_factory_curve_is_what_blender_running_deposits() {
    // (r, dz) medidos; R = 0,5 e o pico é 0,5.
    const OBSERVED: [(f32, f32); 12] = [
        (0.088_39, 0.458_649),
        (0.128_85, 0.417_503),
        (0.168_29, 0.368_206),
        (0.197_64, 0.327_388),
        (0.225_35, 0.286_860),
        (0.251_95, 0.247_082),
        (0.296_46, 0.181_107),
        (0.338_02, 0.123_425),
        (0.377_60, 0.075_226),
        (0.411_03, 0.041_860),
        (0.434_14, 0.023_741),
        (0.489_14, 0.000_697),
    ];
    let mut worst = 0.0f32;
    for (r, dz) in OBSERVED {
        let (t, want) = (r / 0.5, dz / 0.5);
        worst = worst.max((Falloff::Smooth.weight(t) - want).abs());
    }
    assert!(
        worst < 1e-4,
        "o perfil de fábrica do Blender não é a nossa `Smooth`: pior desvio {worst}"
    );
    // O CONTROLE: as duas curvas que a leitura estática e o catálogo anterior
    // ofereciam falham a MESMA tabela, e por muito.
    let spline_e_a_antiga = [
        ("o spline de 4 pontos", 0.940_f32),
        ("a `(1 − t²)²` que se chamava Smooth", 0.870_9),
    ];
    let medido = 0.417_503 / 0.5;
    for (nome, valor) in spline_e_a_antiga {
        assert!(
            (valor - medido).abs() > 100.0 * 1e-4,
            "{nome} teria de estar longe do medido para o gate discriminar"
        );
    }
}

/// **A CURVA DE FÁBRICA SEGUE O MODO, e o modo governava só metade.**
///
/// ⚠️ `default_falloff` lia `RefMode::S` cravado, então escolher `b-mode` no
/// painel deixava o pincel vestindo a quártica do SculptGL — *a curva que o
/// Blender de facto usa era inalcançável pelo produto*, por mais que o perfil
/// dele passasse a declará-la.
#[test]
fn the_factory_curve_follows_the_mode() {
    use crate::{RefMode, Verb};
    assert_eq!(
        Verb::Draw.default_falloff(RefMode::S),
        Falloff::Plateau,
        "o `s-mode` é a quártica do SculptGL"
    );
    assert_eq!(
        Verb::Draw.default_falloff(RefMode::B),
        Falloff::Smooth,
        "o `b-mode` é a smoothstep que o Blender a correr deposita"
    );
    // ⚠️ **E a resposta do `B` tem de vir da DECLARAÇÃO, não do recuo.** A
    // asserção acima sobrevive com o `profile_b` mudo, porque o
    // `unwrap_or(Falloff::Smooth)` devolve o MESMO valor — medido por mutação:
    // apagar a linha do perfil não sangrava gate nenhum. *Duas rotas que dão o
    // mesmo número não são distinguíveis por quem só lê o número.*
    assert_eq!(
        Verb::Draw.profile(RefMode::B).and_then(|p| p.falloff),
        Some(Falloff::Smooth),
        "o perfil do `b-mode` tem de DECLARAR a curva, não herdá-la do recuo"
    );
}

/// **O que a família `(1 − t²)ⁿ` TEM e as do Blender não: derivada zero no
/// centro.**
///
/// ⚠️ **A propriedade, nunca um número escolhido.** Um dab pousa como DOMO
/// quando `f'(0) = 0` (o vértice do meio sobe quase igual aos vizinhos) e como
/// CONE quando não (um vértice puxado para longe do anel dele — a definição de
/// espetado sobre malha discreta). Este gate divide as doze por essa fronteira
/// e afirma que as duas metades existem: um catálogo só de cones não tem o que
/// o artista pedia, e um só de domos perde o detalhe fino do Blender.
///
/// A derivada é por diferença finita **para a frente** de propósito: `weight`
/// devolve `0` fora de `[0, 1)`, então a central atravessaria a guarda em `t=0`.
#[test]
fn the_dome_family_lands_flat_and_the_blender_family_lands_on_a_point() {
    const H: f32 = 1e-4;
    let slope = |f: Falloff| (f.weight(H) - f.weight(0.0)) / H;

    // Os domos: derivada indistinguível de zero na régua da própria diferença.
    for f in [
        Falloff::Dome,
        Falloff::Dome4,
        Falloff::Sphere,
        Falloff::Plateau,
        Falloff::InvSquare,
    ] {
        assert!(
            slope(f).abs() < 0.01,
            "{} devia pousar CHATO e tem f'(0) = {}",
            f.label(),
            slope(f)
        );
    }
    // E os cones, com o `Sharper` — o mais agudo das doze — nomeado.
    assert!(
        slope(Falloff::Sharper) < -3.5,
        "o `Sharper` de hoje é `(1 − t)⁴` e cai a −4 do centro; medido {}",
        slope(Falloff::Sharper)
    );
    assert!(slope(Falloff::Sharp) < -1.5, "o `Sharp` cai a −2");
    assert!(slope(Falloff::Linear) < -0.9, "a rampa cai a −1");
}

/// **As duas leis que voltaram são EXATAMENTE as que saíram.**
///
/// O oráculo é a expressão escrita à mão — chamar a função sob teste para
/// computar o que se espera é o gate sempre-verde que esta casa já documentou.
///
/// ⚠️ **E ele tem de reproduzir a ASSOCIAÇÃO, não só a álgebra.** A primeira
/// versão escreveu `u*u*u*u` — três multiplicações da esquerda para a direita —
/// contra o `(u*u)*(u*u)` da lei, e as duas divergem por **um ULP** já em
/// `t = 0,02` (1065326389 contra 1065326388). *Num gate byte-a-byte a ordem dos
/// produtos é parte do oráculo*, e foi a lei que estava certa.
#[test]
fn the_dome_curves_are_the_laws_the_parity_wave_displaced() {
    for k in 0..=100 {
        let t = k as f32 / 100.0;
        let u = 1.0 - t * t;
        let u2 = u * u;
        let (want2, want4) = if t >= 1.0 { (0.0, 0.0) } else { (u2, u2 * u2) };
        assert_eq!(
            Falloff::Dome.weight(t).to_bits(),
            want2.to_bits(),
            "Dome em t={t}"
        );
        assert_eq!(
            Falloff::Dome4.weight(t).to_bits(),
            want4.to_bits(),
            "Dome 4 em t={t}"
        );
    }
}
