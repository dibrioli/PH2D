//! ⭐⭐ **OS GATES DA FIAÇÃO DA LEI** — o que se pode afirmar sem um `Device`.
//!
//! ⚠️ **A acendida em si precisa de placa** (`upload_rgba` e `copy_texture_into_individual` são
//! `wgpu`), logo a comparação de PIXELS entre as duas leis é um gate `#[ignore]` que só corre com
//! adapter. O que mora aqui é a metade que é aritmética: *que a porta com a lei dita existe, que a
//! porta do produto a chama, e que a lei de fábrica continua a ser a de sempre.*

/// ⛔⛔ **A PORTA DO PRODUTO PERGUNTA AO OBJECTO** — e sem isto o `light()` podia ficar com uma
/// terceira redacção da escolha.
///
/// # ⚠️⚠️ A premissa que MORREU, e ela era o nome deste gate
///
/// Ele chamava-se `a_porta_do_produto_pergunta_ao_ambiente_e_delega` e exigia, por escrito, que o
/// corpo do `light` contivesse `crate::lei_da_luz::do_ambiente()`. Isso era **a lei** enquanto a
/// escolha era global; desde 2026-09-21 ela é um **campo do objecto** e o ambiente só se
/// **SOBREPÕE** ⇒ o corpo tem de conter a porta que compõe os dois (`efectiva`) e o campo que ela
/// lê (`bake.lei`). *Um gate que exige o nome da função antiga defende o desenho antigo.*
///
/// ⚠️ A régua é o TEXTO porque a função pede um `GpuContext`, um `SpriteRenderer` e um passe — ela
/// **não é alcançável de um teste** sem placa, e é exactamente a família de defeito que esta casa
/// já pagou quatro vezes: *um motor com a lei certa e a porta a não a ligar lê-se como um motor sem
/// a lei*.
#[test]
fn a_porta_do_produto_pergunta_ao_objecto_e_delega() {
    let fonte = include_str!("baked_form.rs");

    // ⚠️⚠️ **A agulha é lida DENTRO do corpo da `light`, e em DOIS pedaços** — a 1.ª redacção
    // procurava `"acende_com(crate::lei_da_luz::do_ambiente()"` como uma corrida só, e o `cargo fmt`
    // partiu a chamada em sete linhas. *Uma agulha textual que o formatador pode partir mede o
    // formatador.* Ela falhou ALTO, que é a sorte da história; a cura é não depender do espaçamento.
    let i = fonte
        .find("pub fn light(")
        .expect("controlo: a porta do produto tem de existir com este nome");
    let corpo = &fonte[i..i + fonte[i..]
        .find("\npub fn acende_com(")
        .expect("controlo: a irmã vem a seguir")];
    for pedaco in ["acende_com(", "crate::lei_da_luz::efectiva(bake.lei)"] {
        assert!(
            corpo.contains(pedaco),
            "o corpo do `light` tem de conter `{pedaco}` — ele é quem pergunta ao OBJECTO, com o \
             bissector do ambiente por cima"
        );
    }

    // ⛔ **E ele NÃO pode voltar a perguntar directamente ao ambiente:** com `do_ambiente` ali, a
    // lei gravada de cada peça deixava de ser lida e todo objecto acendia pela mesma lei outra vez.
    assert!(
        !corpo.contains("lei_da_luz::sobreposicao()"),
        "a porta do produto não lê a sobreposição à mão — quem a compõe é a `efectiva`"
    );

    // E os DOIS braços têm de estar na porta que recebe a lei, não espalhados.
    for braco in [
        "Lei::Tinta => acende_pela_tinta",
        "Lei::Forma => acende_pela_forma",
    ] {
        assert!(fonte.contains(braco), "falta o braço `{braco}`");
    }

    // **O CONTROLO da régua**: uma agulha que *não* está lá tem de falhar, senão um `contains`
    // sobre um ficheiro que mudou de nome passaria por vácuo.
    assert!(
        !fonte.contains("Lei::Vidro =>"),
        "controlo: a régua tem de poder dizer NÃO"
    );
}

/// ⚠️ **A lei nova NÃO entra pela rota que o passe da tinta usa** — ela não toca no
/// `ImpastoLightPass`, e é isso que garante que ligar uma não pode mudar a outra.
#[test]
fn a_lei_nova_nao_toca_no_passe_da_tinta() {
    let fonte = include_str!("baked_form.rs");
    let i = fonte
        .find("fn acende_pela_forma(")
        .expect("controlo: a função tem de existir com este nome");
    let j = fonte[i..]
        .find("\nfn acende_pela_tinta(")
        .expect("controlo: a irmã tem de vir a seguir");
    let corpo = &fonte[i..i + j];
    assert!(
        !corpo.contains("ImpastoLightPass") && !corpo.contains("SpecLut"),
        "o caminho da forma não pode mencionar o passe da tinta"
    );
    // **O CONTROLO**: o caminho da TINTA menciona-o — senão a asserção acima seria sobre um
    // ficheiro onde aquele nome já não existe em lado nenhum.
    assert!(
        fonte[i + j..].contains("ImpastoLightPass"),
        "controlo: o caminho da tinta TEM de usar o passe da tinta"
    );
}

/// ⭐⭐⭐ **A OCLUSÃO DE FORMA CHEGA AO PIXEL — e este gate é o IRMÃO INVERTIDO do que aqui estava.**
///
/// # A premissa que morreu, e onde ela estava escrita
///
/// O gate anterior chamava-se `a_oclusao_de_forma_e_inerte_enquanto_o_ambiente_for_zero` e acabava
/// com esta frase: *«no dia em que esta lei ganhar ambiente, a metade de cima reprova e a premissa
/// morre à vista no diff»*. **O dia é este.** As duas metades continuam as mesmas e trocaram de
/// papel: o que era a afirmação passou a ser o CONTROLO.
///
/// ⚠️ **A inércia nunca foi da oclusão — era do AMBIENTE.** A [`ph2d_form_pbr::acende_texel`] aplica
/// a oclusão a **um** termo (`albedo × E(n) × oclusão`), e enquanto `E(n)` foi `[0,0,0]` o canal era
/// multiplicado por zero antes de chegar a um pixel; foi por isso que a mutação que apaga a leitura
/// da textura de oclusão no [`super::passe_da_forma`] **sobreviveu** à paridade com `pior = 0`
/// bytes. ⇒ *a cavidade × os dois AOs que o objecto assado guarda desde que existe passam a ser
/// lidos, e isso não custou uma linha de lei nova: custou o céu.*
///
/// ⚠️ **E a oclusão continua a NÃO tocar a luz directa**, que é a lei do
/// `a_oclusao_nao_toca_a_luz_directa` na folha da óptica — uma lâmpada que o artista apontou chega
/// onde ele a apontou.
#[test]
fn a_oclusao_de_forma_chega_ao_pixel() {
    let lado = 16u32;
    let n = (lado * lado) as usize;
    let base = vec![200u8; n * 4];
    // Uma forma virada ao ecrã, com cobertura cheia — o regime em que a lei de facto acende.
    let mut form = vec![0.0f32; n * 4];
    for t in form.as_chunks_mut::<4>().0 {
        *t = [0.0, 0.0, 1.0, 1.0];
    }
    let cheia = vec![1.0f32; n];
    // ⚠️ Uma oclusão que VARIA — uma constante diferente de `1` seria indistinguível de um albedo
    // mais escuro, e o que se mede é se o CANAL chega.
    let variada: Vec<f32> = (0..n).map(|i| (i % 8) as f32 / 8.0).collect();

    let material = crate::lei_da_luz::material_da_forma();
    let rig = ph2d_light::LightRig::default();
    let lampadas = super::lampadas_do_rig(&rig).expect("o rig de fábrica tem lâmpada acesa");
    let acende = |occ: &[f32], ceu: ph2d_form_pbr::Ceu| {
        ph2d_form_pbr::imagem::acende_imagem(
            &material,
            &ph2d_form_pbr::imagem::Planos {
                size: (lado, lado),
                base: &base,
                form: &form,
                form_occ: occ,
            },
            &lampadas,
            ceu,
            crate::lei_da_luz::OLHAR_DA_FORMA,
        )
        .expect("acende")
    };

    assert_ne!(
        acende(&cheia, super::ceu_do_rig(&lampadas)),
        acende(&variada, super::ceu_do_rig(&lampadas)),
        "com o céu da casa a oclusão de forma TEM de mover pixels"
    );
    // **O CONTROLO, e ele é a premissa que morreu**: sem céu ela continua inerte — a oclusão pesa o
    // AMBIENTE e mais nada, logo um ambiente nulo torna o canal invisível. Sem esta metade alguém
    // leria a de cima como *«a oclusão passou a pesar a directa»*, que é outra lei e seria errada.
    assert_eq!(
        acende(&cheia, ph2d_form_pbr::Ceu::PRETO),
        acende(&variada, ph2d_form_pbr::Ceu::PRETO),
        "controlo: sem céu a oclusão não pode mover um único byte — ela pesa SÓ o ambiente"
    );
}

/// ⭐⭐⭐ **A SOMBRA VALE `AMBIENT` DO PLANO — a lei da casa, agora exacta nesta lei também.**
///
/// # O que se afirma, e porque ele é o gate desta wave
///
/// O doc do [`ph2d_light::AMBIENT`] declara-o como *«o que uma face totalmente virada PARA LONGE da
/// luz ainda devolve»* — uma fracção da resposta PLANA — e diz, por escrito, que *«os dois
/// consumidores (tinta e forma) têm de dobrar a razão do MESMO jeito»*. A lei da forma é ABSOLUTA e
/// a de sempre é RELATIVA, logo a frase não se cumpre copiando o número: cumpre-se pela
/// **conversão** que o [`super::ceu_do_rig`] faz (`f = A/(1 − A)`), e é ela que este gate mede.
///
/// ⚠️ **O material é o [`ph2d_form_pbr::Surface::matte`]**, e não o de omissão: a conversão é exacta
/// para o lóbulo DIFUSO, e o especular do OpenPBR soma um termo que o modelo relativo não tem. *Uma
/// barra tirada com o material de omissão mediria o destaque, não a lei.*
///
/// ⚠️ **A régua é o ACESO e não o pixel**, e a razão é o `Look`: a exposição e a curva de vista são
/// não-lineares, logo a razão entre dois pixels não é a razão entre duas radiâncias. Este gate corre
/// com a [`ph2d_view_transform::Look::default`] (a identidade) de propósito.
#[test]
fn a_sombra_vale_ambient_do_plano() {
    let rig = ph2d_light::LightRig::default();
    let lampadas = super::lampadas_do_rig(&rig).expect("o rig de fábrica tem lâmpada acesa");
    let ceu = super::ceu_do_rig(&lampadas);
    let s = ph2d_form_pbr::OpenPbr::default().prepare().matte();

    let acende = |n: [f32; 3]| {
        ph2d_form_pbr::acende_texel(
            &s,
            &ph2d_form_pbr::Texel {
                normal: n,
                albedo: [1.0; 3],
                cobertura: 1.0,
                oclusao: 1.0,
            },
            &lampadas,
            ceu,
            ph2d_view_transform::Look::default(),
        )
    };

    // O PLANO: a normal da vista. A face virada PARA LONGE: `N·L < 0`, ainda a olhar o observador.
    //
    // ⛔⛔ **A `y` da face escura tem de ser ZERO, e isso NÃO é escolher a fixtura que passa.** A
    // rampa do céu vale `ENV_BASE` no horizonte e **redistribui** à volta dele — uma face virada
    // para BAIXO recebe menos céu (medido: `0,730` do horizonte, e a razão lê `0,255`) e uma virada
    // para cima recebe mais. *A conversão `f = A/(1 − A)` é exacta sobre o horizonte; o que a rampa
    // faz a partir dali é precisamente o que ela existe para fazer* — e a metade de baixo mede-o.
    let l = lampadas[0].para_a_luz;
    let plano = acende([0.0, 0.0, 1.0]);
    let longe = acende([0.8, 0.0, 0.6]);
    assert!(
        l[0] * 0.8 + l[2] * 0.6 < 0.0,
        "controlo: a face escura tem mesmo de estar virada para longe da lâmpada"
    );

    let razao = f64::from(longe[1]) / f64::from(plano[1]);
    let alvo = f64::from(ph2d_light::AMBIENT);
    assert!(
        (razao - alvo).abs() < 0.01,
        "a sombra tem de valer `AMBIENT` ({alvo:.3}) do plano; leu {razao:.3}"
    );

    // **A RAMPA REDISTRIBUI** — e sem esta metade um céu chapado passaria na de cima.
    let cima = f64::from(acende([0.0, -0.8, 0.6])[1]) / f64::from(plano[1]);
    let baixo = f64::from(acende([0.0, 0.8, 0.6])[1]) / f64::from(plano[1]);
    assert!(
        cima > razao && razao > baixo,
        "o céu é o topo da TELA: virada para cima {cima:.3} > horizonte {razao:.3} > baixo {baixo:.3}"
    );

    // **O CONTROLO**: sem céu a face virada para longe devolve ZERO — que é o defeito que esta wave
    // curou, e sem esta metade alguém leria a de cima como uma propriedade que já existia.
    let sem_ceu = ph2d_form_pbr::acende_texel(
        &s,
        &ph2d_form_pbr::Texel {
            normal: [0.8, 0.0, 0.6],
            albedo: [1.0; 3],
            cobertura: 1.0,
            oclusao: 1.0,
        },
        &lampadas,
        ph2d_form_pbr::Ceu::PRETO,
        ph2d_view_transform::Look::default(),
    );
    // ⚠️ **`< 1e-6` e não `== 0`**: a óptica prende o `N·L` num `EPS` em vez de o deixar chegar a
    // zero, logo uma face virada para longe devolve `3,2e-9` e não o zero exacto. *Escrito com
    // `assert_eq!` este controlo reprovava sobre produto correcto* — e reprovou, que é o gate a
    // funcionar. A barra está `300×` acima do medido e `10 000×` abaixo do que o céu entrega.
    assert!(
        sem_ceu.iter().all(|c| *c < 1e-6),
        "controlo: sem céu a sombra é PRETA; leu {sem_ceu:?}"
    );
}

/// ⭐⭐ **AS DUAS PORTAS DO «PLANO» CONCORDAM** — a que lê o rig resolvido e a que lê as lâmpadas da
/// óptica.
///
/// A conversão do piso relativo em rampa mudou-se para a [`ph2d_light::env_ramp`] quando o barro
/// vivo passou a ser o segundo consumidor dela. A entrada dessa conversão — a **resposta plana** —
/// ficou calculada nos dois sítios, porque cada consumidor tem o seu vocabulário de lâmpada.
///
/// ⚠️ **Duas contas iguais escritas em dois tipos são exactamente como uma lei diverge**, e a única
/// coisa que o impede é este gate: ele constrói o MESMO rig dos dois lados e exige o mesmo `f32`.
///
/// ⭐ **E o CONTROLO é o que lhe dá direito:** um rig com todas as lâmpadas apagadas tem de ler
/// `[0, 0, 0]` — senão a igualdade acima seria entre dois zeros e não afirmaria nada.
///
/// **Mutação que deve sangrar:** tirar o `max(0.0)` de um dos lados.
#[test]
fn as_duas_portas_do_plano_concordam() {
    let rig = ph2d_light::LightRig::default();
    let resolvido = ph2d_light::resolve(&rig).expect("o rig de fábrica tem uma lâmpada acesa");

    let pela_luz = ph2d_light::flat_response(&resolvido);
    let lampadas = crate::baked_form::lampadas_do_rig(&rig).expect("o rig resolve");
    let mut pela_optica = [0.0f32; 3];
    for l in &lampadas {
        let ndl = l.para_a_luz[2].max(0.0);
        for (p, r) in pela_optica.iter_mut().zip(l.radiancia) {
            *p += ndl * r;
        }
    }
    assert_eq!(
        pela_luz, pela_optica,
        "as duas portas da resposta plana divergiram — a rampa do céu passa a ser outra no barro \
         vivo e no sprite assado, e o sintoma é «o assado não está igual ao vivo»"
    );

    // ⭐ **O CONTROLO**: a fixtura tem de ter luz, senão a igualdade é entre dois zeros.
    assert!(
        pela_luz.iter().any(|c| *c > 1.0e-3),
        "controlo: o rig de fábrica tem de dar resposta plana ({pela_luz:?})"
    );

    // ⛔ E a metade que prova que o céu SEGUE o rig: sem lâmpada nenhuma a rampa é toda zero, e a
    // lei que a lê volta a ser byte a byte a de antes de haver céu.
    let (base, inc) = ph2d_light::env_ramp([0.0; 3]);
    assert_eq!((base, inc), ([0.0; 3], [0.0; 3]));
}
