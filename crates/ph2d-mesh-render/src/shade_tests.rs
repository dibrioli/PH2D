//! Gates das opções de vista.

use super::*;

/// ⭐⭐⭐ **A ESCADA DA LUZ É EXACTA: plano `0`, rig `1`, matcap `2 + i`.**
///
/// ⛔⛔ **Este gate chamava-se `the_rig_is_zero_and_no_matcap_lands_there` e a
/// premissa dele MORREU em 2026-09-20**, por ordem do dono (*«precisamos como no
/// blender modos de shaders além do matcap para pintar»*): o `0` deixou de ser o
/// rig e passou a ser o modo PLANO. A morte está à vista no diff, que é a lei
/// desta casa para uma premissa que cai.
///
/// ⚠️ **E o `0` ser o PLANO é deliberado:** quem esquecer um sítio na conversão
/// produz uma peça sem luz, que se vê na primeira olhada — *um valor esquecido
/// que é silencioso é um defeito que ninguém conserta*.
///
/// ⚠️ **A premissa do rig continua DECLARADA e não herdada do default** (a
/// lição que este gate já pagou uma vez: uma fixtura que chega ao estado pelo
/// `Shade::default()` inverte de sentido no dia em que o default anda).
#[test]
fn a_escada_da_luz_e_exacta_e_o_zero_e_o_plano() {
    assert_eq!(
        ShadeRaw::pack(Shade {
            lighting: Lighting::Flat,
            ..Shade::default()
        })
        .lighting,
        LIGHTING_FLAT
    );
    assert_eq!(
        ShadeRaw::pack(Shade {
            lighting: Lighting::Rig,
            ..Shade::default()
        })
        .lighting,
        LIGHTING_RIG
    );
    for i in 0..MATCAPS.len() {
        let packed = ShadeRaw::pack(Shade {
            lighting: Lighting::Matcap(u8::try_from(i).expect("a tabela cabe num u8")),
            ..Shade::default()
        });
        assert_eq!(packed.lighting, LIGHTING_FIRST_MATCAP + i as u32);
        // O CONTROLO: nenhum material pousa num dos dois degraus fixos, senão
        // ele seria invisível — o artista escolheria uma cera e veria outra luz.
        assert!(
            packed.lighting != LIGHTING_FLAT && packed.lighting != LIGHTING_RIG,
            "o material {i} empacotou como um dos modos SEM matcap"
        );
    }
}

/// ⭐⭐ **A ESCADA É A MESMA NO SHADER** — e ela está escrita duas vezes de
/// propósito, porque um uniform não partilha constantes com Rust.
///
/// ⛔ *Duas cópias sem gate divergem na primeira wave que acrescentar um modo*,
/// e o modo de falha é MUDO: o artista escolhe «plano» e vê um matcap.
#[test]
fn a_escada_da_luz_concorda_com_o_shader() {
    const WGSL: &str = include_str!("shaders/mesh.wgsl");
    for (nome, valor) in [
        ("LIGHTING_FLAT", LIGHTING_FLAT),
        ("LIGHTING_RIG", LIGHTING_RIG),
        ("LIGHTING_FIRST_MATCAP", LIGHTING_FIRST_MATCAP),
    ] {
        let agulha = format!("const {nome}: u32 = {valor}u;");
        assert!(
            WGSL.contains(&agulha),
            "o shader não declara `{agulha}` — as duas escadas divergiram"
        );
    }
    // O CONTROLO da própria extracção: uma agulha que o shader NÃO tem tem de
    // falhar, senão este gate ficaria verde sobre um ficheiro vazio.
    assert!(!WGSL.contains("const LIGHTING_FLAT: u32 = 99u;"));
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
        lighting: Lighting::Matcap(u8::try_from(MATCAPS.len() - 1).expect("cabe")),
        ..Shade::default()
    });
    for i in [MATCAPS.len() as u8, 200, u8::MAX] {
        assert_eq!(
            ShadeRaw::pack(Shade {
                lighting: Lighting::Matcap(i),
                ..Shade::default()
            })
            .lighting,
            last.lighting,
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
    let src = crate::fonte::MESH_WGSL;
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
    let src = crate::fonte::MESH_WGSL;
    assert_eq!(
        src.matches("fn form_occlusion(").count(),
        1,
        "a porta é uma"
    );
    // ⚠️ **O corpo do sombreamento mudou de casa em 2026-09-20 e este gate
    //   reprovou — que é ele a funcionar.** O `fs_main` passou a delegar num
    //   `fs_core(in, vcolor)` para que a tinta fina não trouxesse uma SEGUNDA
    //   lei de luz, logo é o `fs_core` que tem de perguntar à porta. A metade
    //   nova, logo abaixo, é a que impede alguém de re-embutir um corpo.
    for entry in ["fs_core", "fs_gbuffer"] {
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
    // ⭐ E as DUAS entradas de cor delegam no MESMO corpo — sem isto, a tinta
    // fina podia ganhar uma cópia do sombreamento e a peça acenderia de duas
    // maneiras conforme o plano estivesse armado.
    for entry in ["fs_main", "fs_main_tinta"] {
        let fonte = crate::fonte::mesh_wgsl(true);
        let body = fonte
            .split_once(&format!("fn {entry}("))
            .unwrap_or_else(|| panic!("`{entry}` existe"))
            .1
            .split_once("\n}")
            .expect("ele fecha")
            .0;
        assert!(
            body.contains("fs_core("),
            "`{entry}` tem de delegar no corpo único, não compor a luz por conta própria"
        );
        assert!(
            !body.contains("form_occlusion("),
            "`{entry}` voltou a compor a oclusão — o corpo é do `fs_core`"
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
    let src = crate::fonte::MESH_WGSL;
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
