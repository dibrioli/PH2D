//! As provas da cena — a metade que se mede sem uma GPU e sem uma janela.

use super::*;

/// **A IMAGEM tem de ter CONTRASTE**, senão a máscara é indistinguível de mais intensidade.
///
/// ⚠️ Este gate existe porque o defeito é invisível num teste de estrutura: uma síntese que
/// devolvesse cinzento uniforme monta a cena inteira, passa o grafo, e o smoke mostraria um halo
/// parejo — o produto correcto e o defeito leem-se igual.
#[test]
fn the_dirt_image_is_mostly_dark_with_bright_patches() {
    let px = dirt_pixels();
    assert_eq!(px.len(), (DIRT_PX * DIRT_PX * 4) as usize);
    // ⚠️ **Em LINEAR, que é o que o shader vê.** Os bytes estão codificados em sRGB (ver
    // `dirt_pixels`), então medir `byte/255` mediria a codificação e não a luz.
    let lin = |b: u8| ph2d_color::srgb::srgb_to_linear_byte(b);
    let lum: Vec<f32> = px
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| (lin(c[0]) + lin(c[1]) + lin(c[2])) / 3.0)
        .collect();
    let dark = lum.iter().filter(|v| **v < 0.1).count();
    let bright = lum.iter().filter(|v| **v > 0.5).count();
    let total = lum.len();
    assert!(
        dark * 2 > total,
        "a mascara tinha de ser MAIS de metade escura (é onde o halo fica o de sempre): {dark}/{total}"
    );
    assert!(
        bright * 100 > total,
        "e ter pelo menos 1% de manchas claras, senao nao ha' o que ver: {bright}/{total}"
    );
    // E o alfa é opaco em toda parte — a máscara é lida por RGB, e um alfa variável seria um
    // segundo canal a decidir a mesma coisa.
    assert!(px.as_chunks::<4>().0.iter().all(|c| c[3] == 255));
}

/// A imagem é COLORIDA — a metade da referência que um cinzento não mostra.
#[test]
fn the_dirt_image_carries_colour_not_just_brightness() {
    let px = dirt_pixels();
    let coloured = px
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|c| {
            let (r, g, b) = (i32::from(c[0]), i32::from(c[1]), i32::from(c[2]));
            (r - b).abs() > 24 && r.max(g).max(b) > 64
        })
        .count();
    assert!(
        coloured > 1000,
        "quase nenhum pixel tem matiz: a sujidade so' somaria brilho ({coloured})"
    );
}

/// **A imagem é a MESMA em toda máquina.** Sem isto, duas fotos do smoke não se comparam e um
/// report do Enio deixa de ser reproduzível.
#[test]
fn the_dirt_image_is_deterministic() {
    assert_eq!(dirt_pixels(), dirt_pixels());
}

/// **O grafo monta, e o nó do halo sai com a máscara JÁ escolhida.**
///
/// ⚠️ **As duas metades**: o `f32` e o param de TEXTO. Uma cena que pusesse só a intensidade
/// abriria com o knob alto e nenhuma imagem — que é exactamente o estado que a identidade do
/// quadro tem de sobreviver, e não o que este smoke quer mostrar.
#[test]
fn the_scene_wires_the_glow_with_the_mask_already_chosen() {
    let mut g = Graph::new();
    let out = build(&mut g).expect("a cena monta");
    assert_eq!(
        g.nodes()
            .iter()
            .find(|n| n.id == out)
            .map(|n| n.type_name.as_str()),
        Some("motion.output")
    );
    let glow = g
        .nodes()
        .iter()
        .find(|n| n.type_name == "fx.glow")
        .expect("ha' um fx.glow");
    assert_eq!(
        ph2d_node_fx_glow::dirt::source(&g).as_deref(),
        Some(DIRT_NAME),
        "o campo «Dirt Texture» nao nasceu preenchido"
    );
    let ov = g.node_param_overrides(glow.id).expect("overrides");
    assert_eq!(
        ov.get(ph2d_node_fx_glow::dirt::DIRT_INTENSITY).copied(),
        Some(DIRT_INTENSITY)
    );
    // E o limiar tem de estar ABAIXO de 1: com o de fábrica as peças da grade (que nascem em
    // 1,0) nao acendem, e o smoke mostraria um ecra limpo sobre produto correcto.
    let threshold = ov.get("threshold").copied().expect("threshold autorado");
    assert!(threshold < 1.0, "nada acenderia: threshold {threshold}");
}

/// O nome que a sprite leva é EXACTAMENTE o que o nó procura — as duas pontas saem da mesma
/// constante, e este gate é o que impede alguém de as separar.
#[test]
fn the_sprite_name_and_the_node_field_are_one_constant() {
    let mut g = Graph::new();
    build(&mut g).expect("monta");
    assert_eq!(
        ph2d_node_fx_glow::dirt::source(&g).as_deref(),
        Some(DIRT_NAME)
    );
    assert!(!DIRT_NAME.trim().is_empty());
    // E ele não cai no namespace reservado do editor (`$`), que o publicador recusa.
    assert!(!ph2d_nodegraph::external::is_reserved(DIRT_NAME));
}

/// **TODO PARAM QUE A CENA AUTORA TEM DE EXISTIR NO MANIFESTO DO NÓ.**
///
/// ⚠️ **Este gate nasceu de um defeito real desta própria cena:** ela escrevia `spacing_x` /
/// `spacing_y` no `motion.grid`, cujo manifesto declara `gap_x` / `gap_y`. Um `set_param` com
/// nome errado **não falha** — ele guarda um override que ninguém lê —, então a cena montava, o
/// campo saía com o espaçamento de fábrica, e não havia uma linha vermelha em parte nenhuma.
/// *Um nome de param errado é indistinguível de um valor mal escolhido a olho*, e a diferença
/// entre os dois é onde se vai procurar.
///
/// Ele varre a cena inteira contra o registry, então cobre também os nós que ela venha a ganhar.
#[test]
fn the_scene_only_authors_params_the_manifests_declare() {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");
    let mut g = Graph::new();
    build(&mut g).expect("monta");
    // ⚠️ **E o `validate` do próprio grafo** — descoberto depois, ao montar a cena `=107`: ele
    // devolve `UnknownParam` para exactamente esta classe, e cobre também os TIPOS das arestas.
    // A varredura abaixo fica porque diz QUAL param e QUAIS estão declarados, que é o que se
    // quer ler quando ela reprova; o `validate` é a rede mais larga.
    g.validate(&reg).expect("a cena e' bem-tipada");
    let mut checked = 0;
    for node in g.nodes() {
        let manifest = reg
            .manifests()
            .find(|m| m.name == node.type_name)
            .unwrap_or_else(|| panic!("a cena usa um no' que nao existe: {}", node.type_name));
        let Some(ov) = g.node_param_overrides(node.id) else {
            continue;
        };
        for name in ov.keys() {
            assert!(
                manifest.params.iter().any(|p| p.name == name),
                "a cena escreve «{name}» em {}, que nao o declara. Declarados: {:?}",
                node.type_name,
                manifest.params.iter().map(|p| p.name).collect::<Vec<_>>()
            );
            checked += 1;
        }
    }
    // Controle positivo: se a varredura não viu nenhum override, ela não provou nada.
    assert!(
        checked >= 6,
        "a varredura so' viu {checked} params autorados"
    );
}

/// **A IMAGEM É CODIFICADA EM sRGB, E ISSO NÃO É COSMÉTICA.**
///
/// ⚠️ **Sem este passo a máscara é quase preta e a feature parece não existir.** O átlas
/// partilhado é `Rgba8UnormSrgb`, então o amostrador decodifica o byte para linear antes de o
/// composite o ver: escrever o linear `0,04` como byte cru (`10`) chega ao shader como
/// **`0,004`** — dez vezes menos. Medido na aplicação real: com bytes crus, o pixel mediano do
/// halo mudava `1,01×` com o knob no máximo, e o report foi *«não percebi nenhuma mudança»*.
///
/// O gate afirma a ida-e-volta: o byte que sai daqui, decodificado como o hardware o decodifica,
/// tem de devolver o valor LINEAR que a síntese autorou.
#[test]
fn the_image_survives_the_srgb_round_trip_of_the_atlas() {
    let px = dirt_pixels();
    let lin = |b: u8| ph2d_color::srgb::srgb_to_linear_byte(b);
    // ⚠️ **O PISO da linha de cima, não um pixel escolhido a dedo** — o `(0,0)` parecia fundo e
    // é uma mota de pó (o hash do pó é `0` ali). O canal AZUL da 1.ª linha é `a * 0.55`, e o
    // menor `a` daquela linha é o fundo autorado, `0,04`.
    let base_blue = (0..DIRT_PX as usize)
        .map(|x| lin(px[x * 4 + 2]))
        .fold(f32::INFINITY, f32::min);
    assert!(
        (base_blue - 0.04 * 0.55).abs() < 0.004,
        "o fundo devia voltar a {:.4} em linear e voltou a {base_blue:.4} — os bytes nao estao \
         codificados em sRGB",
        0.04 * 0.55
    );
    // E o CONTROLE que nomeia o defeito: se os bytes fossem crus, o mesmo pixel leria ~10× menos.
    let raw_would_be = ph2d_color::srgb::srgb_to_linear_unit(0.04 * 0.55);
    assert!(
        raw_would_be < base_blue / 4.0,
        "o controle nao separa as duas escritas ({raw_would_be:.5} vs {base_blue:.5})"
    );
}
