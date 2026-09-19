//! ⭐⭐⭐ **O CENSO DOS BOTÕES DO MATERIAL — qual deles chega a um CONSUMIDOR, e em que caminho.**
//!
//! Pergunta do dono, 2026-09-18, depois do smoke da `=34`: *«SS Anisotropy está morto?»*
//!
//! # ⛔⛔ Porque isto é um CENSO e não uma resposta
//!
//! O `CLAUDE.md` §5.0 diz de si mesmo que *«nenhum instrumento do repo pergunta se o VALOR chega a
//! um consumidor»* — e a caça de 2026-08-30 achou **34 controlos mortos** sobre ~504 seguidos até
//! ao efeito. Responder a UM botão por leitura de código deixaria os outros trinta e dois por
//! perguntar, e a leitura é o método pior: ela diz *como*, e a pergunta é *o quê*.
//!
//! ⇒ para cada campo de material, o mesmo material com **duas** posições do botão, comparado **ao
//! bit** — nos dois caminhos da subsuperfície, porque a `=33` e a `=34` mostram que o caminho muda
//! a resposta.
//!
//! # ⚠️ O que este censo NÃO vê, e está declarado
//!
//! Ele mede a [`ph2d_material::Surface::direct`] — a luz de uma lâmpada. ⛔ Fica de fora a
//! **emissão** (que sai por uma porta própria) e a luz do **céu** (que precisa de um
//! `Environment`). *Um botão que este censo diga morto e que só viva ali está vivo, e é por isso
//! que a lista de fora é nomeada em vez de silenciada.*
//!
//! ⚠️ E o material de base tem **tudo armado** (verniz, especular, subsuperfície a meio) de
//! propósito: um botão cujo dono está desligado mede-se morto sem o ser, e essa é a forma mais
//! barata de fabricar uma lista de dívida.

use ph2d_field_ecs::FieldMaterial;

/// O material em que **tudo está armado** — ver o cabeçalho.
fn base() -> FieldMaterial {
    FieldMaterial {
        base_weight: 1.0,
        base_color: [0.75, 0.35, 0.35],
        base_diffuse_roughness: 0.3,
        metalness: 0.0,
        specular_weight: 1.0,
        specular_color: [1.0; 3],
        roughness: 0.3,
        specular_ior: 1.5,
        coat: 0.5,
        coat_color: [1.0; 3],
        coat_roughness: 0.2,
        coat_ior: 1.6,
        coat_darkening: 1.0,
        emission: 0.1,
        emission_color: [1.0; 3],
        // ⭐ A MEIO de propósito: com `1` a difusa desaparece e metade dos botões da base mede-se
        // morta sem o ser; com `0` desaparece a subsuperfície inteira.
        subsurface_weight: 0.5,
        subsurface_color: [0.75, 0.35, 0.35],
        subsurface_radius: 0.3,
        subsurface_radius_scale: [1.0, 1.0, 1.0],
        subsurface_scatter_anisotropy: 0.0,
        ..FieldMaterial::default()
    }
}

/// O índice do `thin_walled` — o **selector de caminho**, e por isso ele não se varre: ele É o eixo.
const CAMPO_DA_PAREDE_FINA: u8 = ph2d_field::MATERIAL_FIELDS - 1;

/// A resposta do material a uma luz, em várias geometrias, como bits.
fn resposta(m: FieldMaterial) -> Vec<u32> {
    let s = crate::materials::surface_of(m).at_curvature(1.0 / 0.3);
    let mut out = Vec::new();
    for ndl in [0.9f32, 0.4, 0.0, -0.3, -0.8] {
        let l = [(1.0 - ndl * ndl).max(0.0).sqrt(), 0.0, ndl];
        for v in [[0.0, 0.0, 1.0], [0.5, 0.0, 0.866]] {
            out.extend(
                s.direct([0.0, 0.0, 1.0], v, l, [1.0; 3])
                    .into_iter()
                    .map(f32::to_bits),
            );
        }
    }
    out
}

/// Um campo está vivo neste caminho se mexê-lo muda a resposta **em algum bit**.
fn vivo(campo: u8, parede_fina: bool) -> bool {
    let mut a = base();
    a.set(CAMPO_DA_PAREDE_FINA, if parede_fina { 1.0 } else { 0.0 });
    let mut b = a;
    // ⚠️ `0,25` e `0,75` e não `0` e `1`: os extremos de vários destes botões são degenerados (um
    // peso a zero apaga o ramo inteiro), e um par degenerado mede a AUSÊNCIA do ramo em vez do
    // botão.
    a.set(campo, 0.25);
    b.set(campo, 0.75);
    resposta(a) != resposta(b)
}

/// ⏱️ **A TABELA** — o que está vivo em que caminho.
#[test]
#[ignore = "sonda: imprime uma tabela, não afirma"]
fn sonda_o_censo_dos_knobs_do_material() {
    println!("\n  ── OS BOTÕES DO MATERIAL, por caminho (medido ao BIT) ──");
    println!("    campo · nome                          · SOLID · THIN WALLED");
    for k in 0..ph2d_field::MATERIAL_FIELDS {
        if k == CAMPO_DA_PAREDE_FINA {
            continue;
        }
        let nome = ph2d_field_ecs::material_key(k).unwrap_or("???");
        let (s, t) = (vivo(k, false), vivo(k, true));
        let marca = |b: bool| if b { "vivo " } else { "MORTO" };
        println!(
            "     {k:>4} · {nome:<36} · {} · {}{}",
            marca(s),
            marca(t),
            if s == t { "" } else { "   ⭐ DIFERE" }
        );
    }
    println!(
        "\n    ⚠️ Este censo mede a luz de uma LÂMPADA. A emissão e a luz do céu saem por portas\n \
         \x20     próprias e não entram aqui — um botão morto nesta tabela pode viver numa delas."
    );
}

/// ⭐⭐⭐ **A RESPOSTA À PERGUNTA DO DONO: o `SS Anisotropy` NÃO está morto — ele é do caminho da
/// PAREDE FINA, e no `Solid` não tem consumidor.**
///
/// ⛔ E isso é o **porte fiel** e não um defeito nosso: o modelo publicado lê a anisotropia só nos
/// dois factores da parede fina (`subsurface_thin_walled_brdf_factor` e `..._btdf_factor`), e o
/// caminho maciço dele nem a recebe como argumento.
///
/// ⚠️⚠️ **Mas isto É um controlo morto no sentido do `CLAUDE.md` §5.0** — *«o painel escreve onde ·
/// quem lê · o leitor DECIDE, ou entrega a alguém que descarta?»*. Com `Thin Walled: Solid` o painel
/// oferece uma fileira que o barro não sente, que é a mesma forma do `Strength` do `Density` na
/// família do esculpir. ⇒ **a cura tem dono e é de produto**: esconder a fileira no `Solid`, ou
/// pintá-la desactivada com a razão à vista.
///
/// ⭐ Este gate FIXA a medição para que essa decisão não se perca: no dia em que alguém ligar a
/// anisotropia ao caminho maciço — ou esconder a fileira — ele reprova e obriga a dizer o que
/// mudou.
/// ⭐⭐⭐ **E não é UM botão: é uma PARTIÇÃO — cinco controlos, e cada modo lê metade.**
///
/// | botão | `Solid` | `Thin Walled` | porquê |
/// |---|---|---|---|
/// | `subsurface_anisotropy` | **morto** | vivo | a fase só entra nos dois factores da parede fina |
/// | `subsurface_radius` | vivo | **morto** | a parede fina **não sabe nada sobre a forma da peça** |
/// | `subsurface_scale_r/g/b` | vivo | **morto** | idem — sem profundidade, não há distância por canal |
///
/// ⭐ *A parede fina é a lambertiana do lado de lá: ela não tem profundidade, logo um caminho livre
/// médio não lhe diz nada.* E a maciça integra um perfil isotrópico, logo uma fase não lhe diz
/// nada. **As duas metades são o porte fiel**, e é por isso que este gate as FIXA em vez de as
/// curar.
///
/// ⚠️⚠️ **Mas as cinco SÃO controlos mortos no sentido do `CLAUDE.md` §5.0** — *«o painel escreve
/// onde · quem lê · o leitor DECIDE, ou entrega a alguém que descarta?»*. O painel mostra a UNIÃO e
/// a lei lê uma PARTIÇÃO ⇒ em qualquer modo há fileiras que o barro não sente. É a mesma forma do
/// `Strength` do `Density` na família do esculpir, e a cura é a mesma e **é de produto**: esconder,
/// ou pintar desactivado com a razão à vista.
#[test]
fn os_botoes_da_subsuperficie_sao_uma_particao_entre_os_dois_caminhos() {
    // ⛔ A lista é de ÍNDICES com o nome CONFERIDO, e não de nomes escritos à mão: se a tabela do
    // documento se mexer, este gate reprova a dizer que se mexeu, em vez de medir outro campo.
    let so_macico = [
        (27u8, "field.dim.subsurface_radius"),
        (28, "field.dim.subsurface_scale_r"),
        (29, "field.dim.subsurface_scale_g"),
        (30, "field.dim.subsurface_scale_b"),
    ];
    let so_parede_fina = [(31u8, "field.dim.subsurface_anisotropy")];
    for (k, nome) in so_macico.iter().chain(so_parede_fina.iter()) {
        assert_eq!(
            ph2d_field_ecs::material_key(*k),
            Some(*nome),
            "o índice {k} deixou de ser o `{nome}` — a tabela mexeu-se debaixo deste gate"
        );
    }
    for (k, nome) in so_macico {
        assert!(
            vivo(k, false),
            "o `{nome}` ficou morto TAMBÉM no maciço — isso seria um defeito, não uma lei"
        );
        assert!(
            !vivo(k, true),
            "o `{nome}` passou a mexer na PAREDE FINA — se foi de propósito, esta é a linha que tem \
             de mudar, e com ela a fileira do painel"
        );
    }
    for (k, nome) in so_parede_fina {
        assert!(
            vivo(k, true),
            "o `{nome}` ficou morto TAMBÉM na parede fina — isso seria um defeito, não uma lei"
        );
        assert!(
            !vivo(k, false),
            "o `{nome}` passou a mexer no MACIÇO — se foi de propósito, esta é a linha que tem de \
             mudar, e com ela a fileira do painel"
        );
    }
    // ⭐ E o CONTROLO que impede a leitura preguiçosa: os botões da subsuperfície que NÃO estão na
    // partição têm de estar vivos nos DOIS. Sem isto, «cinco partidos» leria-se como «a
    // subsuperfície inteira é uma partição», que é falso e mandaria esconder o painel todo.
    for (k, nome) in [
        (23u8, "field.dim.subsurface_weight"),
        (24, "field.dim.subsurface_r"),
    ] {
        assert_eq!(ph2d_field_ecs::material_key(k), Some(nome));
        assert!(
            vivo(k, false) && vivo(k, true),
            "o `{nome}` devia viver nos dois caminhos"
        );
    }
}

/// ⭐⭐ **E o censo tem PISO e CONTROLO** — sem eles ele podia estar a medir o nada.
///
/// ⛔ Um `vivo()` que respondesse sempre `false` (uma resposta constante, um arnês partido) faria a
/// tabela inteira ler *«MORTO»* e leria-se como um achado enorme. ⇒ a maioria dos campos tem de
/// estar viva, e um campo que sabemos vivo tem de o dizer.
#[test]
fn o_censo_dos_knobs_mede_alguma_coisa() {
    let vivos = (0..ph2d_field::MATERIAL_FIELDS)
        .filter(|k| *k != CAMPO_DA_PAREDE_FINA)
        .filter(|k| vivo(*k, false) || vivo(*k, true))
        .count();
    let total = usize::from(ph2d_field::MATERIAL_FIELDS) - 1;
    assert!(
        vivos * 2 > total,
        "só {vivos} de {total} campos mexem alguma coisa — o arnês deste censo está partido, e uma \
         tabela toda MORTA lê-se como um achado"
    );
    // ⭐ O controlo positivo nomeado: a cor da base muda a resposta nos DOIS caminhos, sempre.
    let cor = 1u8; // `base_r`
    assert_eq!(
        ph2d_field_ecs::material_key(cor),
        Some("field.dim.base_r"),
        "o índice do controlo positivo mexeu-se"
    );
    assert!(vivo(cor, false) && vivo(cor, true), "o controlo positivo não acusou");
}
