//! Gates das opções de vista.

use super::*;

/// **O rig é o `0`, e nenhum matcap pousa nele.**
///
/// ⚠️ É o gate do sentinela: o shader lê `matcap > 0` para escolher o caminho,
/// então um material que empacotasse em zero seria *invisível* — o artista
/// escolheria o `Clay Warm` e veria a luz do documento, sem nada dizendo por quê.
///
/// ⚠️ **A premissa do rig é DECLARADA, e ela já foi herdada em silêncio uma
/// vez:** esta linha era `Shade::default()`, o que só dizia *"o rig é zero"*
/// enquanto o default FOSSE o rig. Quando ele virou o matcap do SculptGL
/// (2026-08-10) o gate ficou vermelho — a sorte de ele ter sido escrito com um
/// `assert_eq` e não com um `assert_ne`. Uma fixture que chega ao estado pelo
/// default inverte de sentido no dia em que o default anda, e continua verde
/// testando o oposto.
#[test]
fn the_rig_is_zero_and_no_matcap_lands_there() {
    assert_eq!(
        ShadeRaw::pack(Shade {
            matcap: None,
            ..Shade::default()
        })
        .matcap,
        0
    );
    for i in 0..MATCAPS.len() {
        let packed = ShadeRaw::pack(Shade {
            matcap: Some(u8::try_from(i).expect("a tabela cabe num u8")),
            ..Shade::default()
        });
        assert_ne!(
            packed.matcap, 0,
            "o material {i} empacotou como \"sem matcap\""
        );
        assert_eq!(packed.matcap, i as u32 + 1);
    }
}

/// **Um índice fora da tabela é PRESO no último, nunca deixado passar.**
///
/// ⚠️ **A razão mudou com a wave da imagem, e o gate FICA.** Antes o `switch` do
/// shader tinha um braço `default` que era um material legítimo (a cera), e um
/// índice inválido pousava lá. Hoje quem lê o índice é a CPU, para escolher qual
/// PNG decodificar — e um índice fora da tabela ali seria um `panic` no meio de
/// um frame. Prender no último mantém *"pediu o que não existe"* como uma
/// resposta plausível em vez de uma queda, que é a mesma política do
/// [`crate::matcap::decode`] (gate irmão: `an_index_past_the_end_is_clamped_not_a_panic`).
#[test]
fn an_index_past_the_table_is_pinned_to_the_last_material() {
    let last = ShadeRaw::pack(Shade {
        matcap: Some(u8::try_from(MATCAPS.len() - 1).expect("cabe")),
        ..Shade::default()
    });
    for i in [MATCAPS.len() as u8, 200, u8::MAX] {
        assert_eq!(
            ShadeRaw::pack(Shade {
                matcap: Some(i),
                ..Shade::default()
            })
            .matcap,
            last.matcap,
            "o índice {i} escapou da tabela"
        );
    }
}

/// **A cavidade é clampada na porta.**
#[test]
fn the_cavity_is_clamped_at_the_door() {
    for (given, want) in [(-1.0, 0.0), (0.0, 0.0), (0.5, 0.5), (3.0, 1.0)] {
        assert!(
            (ShadeRaw::pack(Shade {
                cavity: given,
                ..Shade::default()
            })
            .cavity
                - want)
                .abs()
                < 1e-6
        );
    }
}

/// **O wireframe NÃO viaja no uniform** — ele é um passe, não um termo.
///
/// ⚠️ Gate de AUSÊNCIA, e ele é o que impede a próxima wave de "resolver" o
/// wireframe com um `if` no fragment: armá-lo não pode mover um byte do que o
/// shader lê, e é isso que se afirma.
#[test]
fn arming_the_wireframe_does_not_move_a_byte_of_the_uniform() {
    let off = ShadeRaw::pack(Shade::default());
    let on = ShadeRaw::pack(Shade {
        wireframe: true,
        ..Shade::default()
    });
    assert_eq!(bytemuck::bytes_of(&off), bytemuck::bytes_of(&on));
}

/// **O MATCAP DO SHADER É A IMAGEM, e não sobrou uma segunda lei.**
///
/// ⚠️ **Este gate SUBSTITUI o `the_shader_has_exactly_one_arm_per_named_material`,
/// que a wave de 2026-08-10 dissolveu.** Ele contava os braços `case`/`default`
/// de um `fn material(id)` no WGSL contra o tamanho de [`MATCAPS`], porque
/// enquanto um matcap era um punhado de números havia DUAS listas que podiam
/// divergir em tamanho. Hoje a identidade de um matcap é a textura residente, o
/// `id` nem chega ao shader, e aquela contagem não tem objeto — apagar o gate
/// sem pôr nada no lugar é que teria sido a perda.
///
/// O que ele afirma agora são as duas metades que sobraram do mesmo risco:
///
/// 1. **o fragment de fato AMOSTRA** a imagem (um shader que voltasse a computar
///    a cor deixaria os nove PNGs decorativos, com a fileira de chips inteira
///    fazendo a mesma coisa);
/// 2. **nenhuma lei analítica sobreviveu** — um `fn material(` de volta seria a
///    segunda resposta a *"como este material é"*, e ela divergiria da imagem no
///    único lugar onde ninguém lê um número: uma screenshot.
#[test]
fn the_shader_reads_the_matcap_image_and_keeps_no_second_law() {
    let src = crate::pipeline::MESH_WGSL;
    assert!(
        src.contains("textureSampleLevel(matcap_tex, sss_samp, matcap_uv(n)"),
        "o fragment tem de amostrar a imagem do matcap"
    );
    assert!(
        src.contains("@group(3) @binding(2) var matcap_tex: texture_2d<f32>;"),
        "a imagem tem de estar declarada no grupo do SSS"
    );
    assert!(
        !src.contains("fn material(id: u32)"),
        "o seletor analítico de material voltou — ele é a segunda resposta que a \
         imagem substituiu"
    );
}

/// **O BARRO E A DOAÇÃO PERGUNTAM A MESMA COISA À MESMA FUNÇÃO.**
///
/// A oclusão de forma — cavidade × os dois AOs — é o que o `docs/3D/05.2` leva à tinta 2D, e ela é
/// composta no shader. Duas expressões seriam duas respostas a *"quão escura é esta fresta?"*, e
/// elas divergiriam no único lugar onde ninguém lê um número de volta: uma escultura que escurece
/// de um jeito no viewport e de outro na tinta que ela acende.
///
/// ⚠️ **O oráculo é ESTRUTURAL, e é isso que o torna imune a um refactor honesto:** ele não procura
/// a expressão, procura que os dois fragments CHAMEM a porta e que ninguém mais componha os três
/// canais por fora dela. Um gate que casasse com o texto da fórmula ficaria vermelho no dia em que
/// alguém renomeasse uma variável, e verde no dia em que alguém copiasse a fórmula.
#[test]
fn the_clay_and_the_donation_ask_the_same_door_how_dark_a_crevice_is() {
    let src = crate::pipeline::MESH_WGSL;
    assert_eq!(
        src.matches("fn form_occlusion(").count(),
        1,
        "a porta é uma"
    );
    for entry in ["fs_main", "fs_gbuffer"] {
        let body = src
            .split_once(&format!("fn {entry}("))
            .expect("o fragment existe")
            .1
            .split_once("\n}")
            .expect("ele fecha")
            .0;
        assert!(
            body.contains("form_occlusion("),
            "`{entry}` tem de perguntar à porta, não compor a oclusão por conta própria"
        );
    }
    // A metade que impede a divergência de VOLTAR: os três ingredientes são nomeados UMA vez, dentro
    // da porta. Se um deles reaparecer noutro lugar, alguém está compondo a oclusão de novo.
    //
    // ⚠️ **O CÓDIGO, sem os comentários** — e este gate nasceu VERMELHO por isso, sobre um shader
    // correto: a primeira versão contava `shade.ao` no texto inteiro e achava duas ocorrências, uma
    // delas a PROSA que explica o termo. *Um gate que varre fonte conta o token na documentação
    // também*, e o modo de falha é uma reprovação que manda mexer no código certo.
    let code: String = src
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(before, _)| before))
        .collect::<Vec<_>>()
        .join("\n");
    for ingredient in ["shade.cavity", "shade.ao", "shade.ssao"] {
        assert_eq!(
            code.matches(ingredient).count(),
            1,
            "`{ingredient}` só pode ser lido dentro de `form_occlusion` — \
             duas leituras são duas leis"
        );
    }
}

/// **O G-BUFFER ESCREVE OS DOIS ALVOS**, e o segundo é a oclusão.
///
/// ⚠️ Sem este gate, apagar o `@location(1)` é uma mudança que **compila**: o pipeline declara dois
/// alvos, o fragment escreve um, e o wgpu aceita — o segundo alvo simplesmente fica com o valor de
/// limpeza. Como o valor de limpeza é BRANCO (o neutro), o sintoma seria a doação carregar *"nada
/// oclui em lugar nenhum"*, que é indistinguível de uma escultura lisa.
#[test]
fn the_gbuffer_writes_the_occlusion_as_its_second_target() {
    let src = crate::pipeline::MESH_WGSL;
    let body = src
        .split_once("fn fs_gbuffer(")
        .expect("o fragment existe")
        .1
        .split_once("\n}")
        .expect("ele fecha")
        .0;
    assert!(
        body.contains("out.occlusion = form_occlusion("),
        "o segundo alvo tem de receber a oclusão da porta"
    );
    assert!(
        src.contains("@location(1) occlusion: f32"),
        "e o struct de saída tem de declará-lo"
    );
}
