//! Gates do empacotamento das lâmpadas.
//!
//! Estes são os que rodam sem device. O que só um render pode responder — *de que lado a luz cai* —
//! está em `tests/gpu_render.rs`, porque um número que descreve uma direção não prova de que lado a
//! forma acende.

use super::*;
use ph2d_light::{Light, LightRig};

/// O uniform copia o rig RESOLVIDO, lâmpada por lâmpada.
///
/// Se algum dia alguém "otimizar" isto reconstruindo `dir` a partir de graus, o número muda em toda
/// lâmpada — e a escultura passa a ser acesa de um lugar levemente diferente da pintura, o que ninguém
/// vê e ninguém consegue explicar.
#[test]
fn the_uniform_carries_the_resolved_lamps_not_a_re_derivation() {
    let mut rig = LightRig::default();
    rig.lights[1] = Light {
        on: true,
        angle_deg: 71,
        elev_deg: 44,
        intensity: 0.6,
        color: [0.9, 0.95, 1.0],
    };
    let resolved = ph2d_light::resolve(&rig).expect("duas acesas");
    let raw = RigRaw::pack(Some(&resolved));
    assert_eq!(raw.n, 2);
    for (i, l) in resolved.lamps().iter().enumerate() {
        assert_eq!(raw.lamps[i].dir, [l.dir[0], l.dir[1], l.dir[2], 0.0]);
        assert_eq!(raw.lamps[i].hlf, [l.half[0], l.half[1], l.half[2], 0.0]);
        assert_eq!(raw.lamps[i].tint, [l.tint[0], l.tint[1], l.tint[2], 0.0]);
    }
    // E as vagas não usadas ficam zeradas — o shader itera até `n`, mas lixo num uniform é a coisa que
    // aparece no dia em que alguém troca a condição do laço.
    for j in raw.n as usize..MAX_LIGHTS {
        assert_eq!(raw.lamps[j].dir, [0.0; 4]);
        assert_eq!(raw.lamps[j].tint, [0.0; 4]);
    }
}

/// Sem lâmpada acesa não há rig, e o uniform diz isso com `n = 0` em vez de mandar lixo.
#[test]
fn no_lit_lamp_packs_as_zero_lamps() {
    let mut dark = LightRig::default();
    for l in &mut dark.lights {
        l.on = false;
    }
    assert!(ph2d_light::resolve(&dark).is_none());
    let raw = RigRaw::pack(None);
    assert_eq!(raw.n, 0);
    assert_eq!(raw.lamps[0].tint, [0.0; 4]);
}

/// O tamanho que o buffer reserva é o tamanho que o WGSL espera.
///
/// `4 × 48` (as lâmpadas: três `vec4` cada) `+ 16` (o contador e o padding que o alinhamento de 16 B
/// exige). Se isto mudar sem o shader mudar junto, a leitura sai deslocada e o resultado é uma cena
/// iluminada por números que ninguém escreveu.
#[test]
fn the_uniform_is_the_size_the_shader_reads() {
    assert_eq!(RigRaw::SIZE, MAX_LIGHTS * 48 + 16);
    assert_eq!(std::mem::size_of::<LampRaw>(), 48);
    assert_eq!(RigRaw::SIZE % 16, 0, "um uniform alinha em 16 B");
}

/// **A malha dobra a razão pelo MESMO piso ambiente que a tinta.**
///
/// O piso é lei do modelo relativo, não material: é *"o que uma face virada para longe da luz ainda
/// devolve"*, e duas cópias dariam uma escultura mais escura na sombra que a pintura ao lado dela, sob
/// a MESMA lâmpada, sem que ninguém soubesse dizer por quê.
///
/// ⚠️ A string é DERIVADA da constante, e não escrita à mão. Uma string literal aqui pegaria o shader
/// driftando e seria cega à outra direção — o número em Rust mudar e o WGSL ficar parado —, que é
/// exatamente a direção que passa a existir quando um segundo shader entra na conta. (O irmão deste
/// gate, do lado da tinta, é `impasto_light_shader_constants_match_the_cpu_pass`.)
#[test]
fn the_clay_folds_the_ratio_by_the_same_ambient_floor() {
    let decl = format!("const AMBIENT: f32 = {};", ph2d_light::AMBIENT);
    assert!(
        crate::pipeline::MESH_WGSL.contains(&decl),
        "mesh.wgsl tem de declarar `{decl}` — o piso é `ph2d_light::AMBIENT`"
    );
    // E o array de lâmpadas do uniform é o do rig, não um número escolhido.
    assert!(
        crate::pipeline::MESH_WGSL.contains(&format!("array<Lamp, {}>", ph2d_light::MAX_LIGHTS)),
        "o array do uniform é `ph2d_light::MAX_LIGHTS`"
    );
}

/// **O AMBIENTE COM DIREÇÃO é o do rig, e não uma segunda cópia.**
///
/// Irmão do gate acima, e pela mesma razão: o piso do barro e o da tinta têm de
/// ser o MESMO estúdio. ⚠️ **E aqui a segunda cópia é mais perigosa que no
/// escalar**, porque um vetor erra em silêncio de mais maneiras — um canal
/// trocado deixa a sombra verde em vez de fria, e ninguém consegue nomear isso
/// olhando uma escultura.
///
/// As strings são DERIVADAS das constantes, nas duas direções: o WGSL driftar e
/// o Rust driftar quebram os dois igual.
#[test]
fn the_clay_lights_the_shadow_with_the_rigs_environment() {
    for (name, v) in [
        ("ENV_BASE", ph2d_light::ENV_BASE),
        ("ENV_SLOPE", ph2d_light::ENV_SLOPE),
    ] {
        let decl = format!(
            "const {name}: vec3<f32> = vec3<f32>({}, {}, {});",
            v[0], v[1], v[2]
        );
        assert!(
            crate::pipeline::MESH_WGSL.contains(&decl),
            "mesh.wgsl tem de declarar `{decl}` — o ambiente é o de `ph2d_light`"
        );
    }
    // ⚠️ **E o SINAL, que é o que um referencial trocado leva embora.** O céu é o
    // topo da TELA e neste frame o topo é `-y`, então o gradiente SUBTRAI. Um
    // `+` aqui põe o céu no chão, e o barro passa a parecer iluminado de um
    // porão — o FATO é medido por um render (`gpu_render.rs`), este é o proxy
    // que sobrevive numa máquina sem adapter.
    assert!(
        crate::pipeline::MESH_WGSL.contains("ENV_BASE - ENV_SLOPE * n.y"),
        "o gradiente tem de SUBTRAIR: o céu é o topo da tela, que aqui é `-y`"
    );
}

/// **A conversão de espaço está NO shader, e ela é a única.**
///
/// O rig é autorado em espaço de tela (`y` para baixo) e a normal chega em espaço de vista (`y` para
/// cima). Sem a negação a mesma lâmpada acende a pintura por cima e a escultura por baixo.
///
/// ⚠️ Isto é um PROXY deliberado — o FATO (de que lado a forma acende) é medido por um render, em
/// `tests/gpu_render.rs::the_key_light_falls_where_the_artist_put_it`, porque uma direção não prova
/// aparência. O proxy existe para a linha não ser apagada por acidente numa máquina sem adapter, onde
/// o gate de verdade faz *skip* e o skip não é verde.
#[test]
fn the_normal_crosses_into_the_rigs_space() {
    assert!(
        crate::pipeline::MESH_WGSL.contains("vec3<f32>(n.x, -n.y, n.z)"),
        "o fragment tem de negar o `y` ao entrar no espaço do rig"
    );
}
