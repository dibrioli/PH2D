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
