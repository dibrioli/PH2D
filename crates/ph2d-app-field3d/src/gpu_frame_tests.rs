//! Os gates da costura — ver [`super`].

use super::takes_the_frame;

/// ⭐⭐⭐ **AS DUAS CONDIÇÕES, e cada uma sozinha basta para o quadro ficar na CPU.**
///
/// # ⚠️⚠️ Eram TRÊS, e as duas que saíram eram DEFEITOS
///
/// **A do quadro ASSENTE** saiu em 2026-09-15 e era a causa directa do report do dono — *«e apagar
/// o AO ao rotacionar a tela»*: o sombreado de contacto só existe no caminho do dispositivo, logo
/// enquanto ela existiu ele desaparecia a cada gesto. O que a substitui não é uma cerca, é a lei da
/// W73 a viajar com o pedido (o `antialias` chega ao
/// [`ph2d_field_gpu::trace::MarchSetup`] e o dispositivo **salta o segundo despacho**).
///
/// **A da ESCULTURA** saiu no mesmo dia, e era uma cerca legítima que deixou de ter sujeito: a
/// escultura **atravessa** desde a wave da grade ([`ph2d_field_gpu::sculpt`]). O que sobra da
/// pergunta é mais estreita e verdadeira — *a folha amostrada sabe entregar a grade?*
///
/// ⚠️ *Uma cerca que perde o sujeito continua a recusar trabalho que já se sabe fazer.*
#[test]
fn o_dispositivo_so_toma_uma_peca_que_ele_sabe_desenhar() {
    let limpa = crate::smoke::scene(1);
    let com_escultura = crate::smoke::scene(6);
    let reg = crate::smoke::sampled_registry();
    let vazio = ph2d_field_eval::hybrid::Registry::new();

    // (1) sem adaptador não há dispositivo.
    assert!(!takes_the_frame(None, &limpa, &reg));

    let Some(t) = super::shared() else {
        println!("sem adaptador — as outras condições ficam por exercitar");
        return;
    };

    // ⭐ (2) **com a grade na mão, a peça com ESCULTURA é tomada** — é a wave da grade a shipar.
    assert!(
        takes_the_frame(Some(t), &com_escultura, &reg),
        "a peça com escultura ficou na CPU — a grade dela atravessa desde 2026-09-15"
    );
    // ⚠️ E o CONTROLO: sem a escultura no registo ela lê como espaço VAZIO nos dois motores, que é
    // o que o `ABSENT` significa — logo também é tomada, e por outra razão.
    assert!(
        takes_the_frame(Some(t), &com_escultura, &vazio),
        "um nome que o registo não conhece é espaço vazio nos DOIS motores"
    );

    // ⭐ O controlo de sempre: a peça sem escultura nenhuma.
    assert!(
        takes_the_frame(Some(t), &limpa, &reg),
        "com adaptador e peça limpa, o dispositivo TEM de tomar — senão esta wave não está ligada \
         a nada"
    );
}

/// ⭐⭐⭐⭐ **A PERGUNTA «ALGUÉM LÊ A CURVATURA?» TEM UMA PORTA, E SÓ UMA.**
///
/// ⛔⛔ **Este censo nasceu de uma auditoria (2026-09-23) que achou a união escrita DUAS vezes** —
/// uma no caminho de referência ([`crate::smoke_draw_thread`]) e outra no do dispositivo
/// ([`super`]), com o comentário do segundo a afirmar que era *«o MESMO predicado»*. *Era o mesmo
/// como EXPRESSÃO e não como mecanismo.*
///
/// ⚠️ **O modo de falha que ele impede é MUDO:** no dia em que nascer um terceiro leitor da
/// curvatura, se o lado do dispositivo ficar atrás, a fita inerte entra, o `curvatura_em` do shader
/// lê uma constante, e a imagem sai plausível e errada.
///
/// ⛔ **E nenhum gate de VALOR o pode apanhar:** o
/// `quem_le_o_campo_continua_a_leva_lo_no_shader` entra por **um** dos dois sítios, e a fixtura
/// dele usa a única propriedade que os dois já conheciam. *Um gate que entra por um dos dois nunca
/// os vê a discordar.*
///
/// ⚠️ **A agulha é MONTADA em runtime** e não escrita como literal: sem isso, um censo que um dia
/// passe a ler o próprio ficheiro acha sempre o que procura.
#[test]
fn a_pergunta_da_curvatura_tem_uma_porta_e_so_uma() {
    let agulha = format!(".any({}::Surface::reads_curvature)", "ph2d_material");
    let porta = "curvatura::alguem_le(";
    const FONTES: [(&str, &str); 2] = [
        ("gpu_frame.rs", include_str!("gpu_frame.rs")),
        ("smoke_draw_thread.rs", include_str!("smoke_draw_thread.rs")),
    ];
    for (nome, fonte) in FONTES {
        // ⭐ O PISO: sem ele um `include_str!` que apontasse para um ficheiro vazio deixava as duas
        // metades trivialmente verdadeiras.
        assert!(
            fonte.len() > 10_000,
            "o piso: {nome} tem {} bytes — este censo varreria quase nada",
            fonte.len()
        );
        assert!(
            fonte.contains(porta),
            "{nome} não lê a porta `{porta}` — ou ele deixou de perguntar pela curvatura, ou \
             escreveu a terceira resposta"
        );
        assert!(
            !fonte.contains(&agulha),
            "{nome} calcula a união da curvatura à mão — ela é uma PORTA \
             (`ph2d_field_render::curvatura::alguem_le`), e escrita duas vezes ela diverge no dia \
             do terceiro consumidor, com a imagem a sair plausível e errada"
        );
    }
}

/// ⭐⭐⭐⭐ **A CHAVE DA CACHE DO CHÃO NÃO LEVA A CÂMARA — porque a ASSADURA não a lê.**
///
/// ⛔⛔ **A premissa do gate que aqui vivia MORREU (2026-10-01).** Ele afirmava *«a chave leva a
/// precisão inteira»* (auditoria de 2026-09-23: a assadura lia `sharp.hit` e `sharp.normal`, e uma
/// chave com metade devolvia um campo com outro `ε`). Certo — e foi essa precisão na chave que fez
/// cada quadro de APROXIMAR re-assar o campo na CPU (`~88 ms` no nó, o report *«se aproximar ainda
/// fica lento»*). Hoje a assadura usa a precisão do MUNDO ([`ph2d_field_render::Sharpness::do_mundo`])
/// e a chave deixa a câmara de fora.
///
/// ⚠️ **As duas metades, porque cada uma sozinha mente:** a chave sem câmara só é honesta se a
/// assadura não a ler — a lei vive no `ph2d-field-render` (o gate de comportamento
/// `chao_sem_camara`, que assa a dois zooms e exige o mesmo campo bit a bit) e a FIAÇÃO lê-se aqui.
#[test]
fn a_chave_do_chao_nao_leva_a_camara_porque_a_assadura_nao_a_le() {
    let chave = include_str!("gpu_frame_chao.rs");
    let corpo = chave
        .split("let chave = ChaveDoChao {")
        .nth(1)
        .and_then(|r| r.split("};").next())
        .expect("a construção da chave");
    assert!(
        !corpo.contains("cam") && !corpo.contains("Sharpness"),
        "a chave do campo do chão voltou a ler a câmara — cada quadro de zoom re-assa o campo"
    );
    let assadura = include_str!("../../ph2d-field-render/src/ground_bounce.rs");
    assert!(
        assadura.contains("scene.sharp = crate::Sharpness::do_mundo();"),
        "a assadura do campo do chão voltou à precisão da câmara — a chave sem ela devolveria um \
         campo assado com outro ε"
    );
}

/// ⭐⭐⭐⭐ **O pedido do produto recorta o raio pela caixa da MARCHA DA CPU — e não assa grade.**
///
/// ⛔ A caixa crua (`Ball::aabb`) parecia a mesma coisa e não é: sem a margem da
/// [`ph2d_field_eval::bounds_clip::march_clip`] o dispositivo começava o raio noutro `t` que a CPU,
/// e a silhueta discordava em `12` pixels na cena `=29` (com ela, `0`). ⇒ as três metades: o pedido
/// traz o recorte · é a caixa DA MARCHA · e o CONTROLO de que a crua é mesmo outra (sem ele, um
/// `march_clip` sem margem deixaria este gate verde a comparar duas coisas iguais por acidente).
#[test]
fn o_pedido_recorta_pela_caixa_da_marcha_da_cpu() {
    let doc = crate::smoke::scene(29);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a cena tem bola");
    let (_, _, setup) = super::pedido(
        &doc,
        &reg,
        &cam,
        &[],
        None,
        ph2d_field_gpu::trace::MAX_LAMPS,
        super::Sonda::default(),
        192,
        108,
        None,
        false,
    )
    .expect("o pedido");
    let l = setup
        .longe
        .expect("o produto recorta o raio pela caixa da peça");
    let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
    assert_eq!(
        (l.lo, l.hi),
        (lo, hi),
        "a caixa do dispositivo não é a da marcha da CPU"
    );
    assert_eq!(
        l.res, 0,
        "a grade de longe está RECUSADA por medição — só o recorte shipa"
    );
    assert!(l.grade().is_none());
    let (alo, ahi) = bola.aabb();
    assert_ne!(
        (alo, ahi),
        (lo, hi),
        "CONTROLO: a caixa da marcha tem de ter margem sobre a crua"
    );
}
