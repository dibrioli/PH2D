//! ⭐⭐⭐ **O DISPOSITIVO, CORRIDO** — a auditoria que o dono pediu em 2026-09-19 (*«já disse que a
//! presença da placa ao lado da esfera MUDA o SSS da esfera. Isso não faz sentido! Auditoria ou
//! outro agente!»*).
//!
//! # ⛔⛔⛔ O achado da auditoria: TODA coluna deste módulo chamada «DISPOSITIVO» era a CPU
//!
//! O [`super::Quadro::mole`] declara-se, por escrito, como *«`false` desenha o quadro como o
//! DISPOSITIVO o desenha»* — e ele pinta na **CPU** com o canal mole desligado. É um **SUCEDÂNEO**,
//! e o dispositivo nunca foi corrido por gate nenhum desta família. *Uma sonda que mede um
//! sucedâneo mede outro programa* — a lei que este repositório já tinha escrita, e que eu paguei
//! outra vez.
//!
//! # ⛔⛔⛔ E o que o sucedâneo escondia é MAIOR do que a diferença que ele media
//!
//! O caminho que o dono corre não é *«o dispositivo marcha e a CPU sombreia»*. É este, no
//! [`crate::smoke_draw_thread::traca`]:
//!
//! ```text
//! let pintado = if pelo_dispositivo && !p.refinar { gpu_frame::paint(…) } else { None };
//! if let Some(pintura) = pintado { …manda a imagem…; return; }   ⇐ DEVOLVE AQUI
//! …
//! g.curvature = curvatura::do_gbuffer(…)          ⇐ nunca alcançado
//! sh.set_soft(… sss_shadow::blur_por_material …)  ⇐ nunca alcançado
//! ```
//!
//! ⇒ **a §12 (a sombra de borda mole) e a cura de 19/09 (o raio por material) vivem em código que o
//! produto não executa** quando o dispositivo pinta — que é o caminho de OMISSÃO:
//!
//! | condição | valor de fábrica |
//! |---|---|
//! | `pelo_dispositivo` | `Shading::Render` + lâmpadas + a peça ser suportada |
//! | `!p.refinar` | ⭐ **verdadeiro**: o [`crate::preview::refines_occlusion`] exige o `PH2D_FIELD_AO`, que nasce DESLIGADO |
//!
//! ⚠️⚠️ *Era por isso que a cura que eu entreguei não mudava nada para ele:* ela era real, era
//! gateada ao bit, e estava **fora do caminho que o artista corre**.
//!
//! # ⭐⭐⭐ E EM 2026-09-19 A DÍVIDA FOI PAGA — a `W10`
//!
//! O `return` daquele ramo **continua lá e está certo**: a cura não foi voltar pelo caminho da CPU
//! (que traria o G-buffer pelo barramento, o que a `traca` evita de propósito) — foi o **dispositivo
//! passar a assar o canal ele próprio**, em duas passagens separáveis, com o gémeo do
//! `Surface::direct_sss` escrito em WGSL.
//!
//! | régua | antes | depois |
//! |---|---:|---:|
//! | quebra na banda, DISPOSITIVO, com a chapa | `9,20` | **`1,00`** |
//! | a mesma, REFERÊNCIA | `1,00` | `1,00` |
//! | paridade dos dois motores sobre a cena do report | *não existia* | **`100,000 %`**, pior byte `1` |
//!
//! ⛔ **E os DOIS gates deste ficheiro tinham a premissa morta**, cada um à sua maneira: o
//! estrutural dizia por escrito que a cura dele era ser apagado, e o de medição prometia o
//! *«dispositivo»* no nome e comparava a CPU consigo mesma. Os dois foram reescritos, com a morte
//! visível no diff.

use super::{Quadro, arranjo_do_dono, quadro, regua_da_banda, so_a_bola};

/// O jade do report — `Subsurface` no máximo, `Thin Walled: Solid`.
fn jade() -> ph2d_material::OpenPbr {
    ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    }
}

/// ⭐⭐⭐ **O GÉMEO DA BORDA MOLE ESTÁ LIGADO NO DISPOSITIVO** — a `W10`, fechada em 2026-09-19.
///
/// # ⛔⛔ Aqui viveu o gate OPOSTO, e a cura dele era apagá-lo
///
/// Até esta wave este ficheiro declarava, por escrito, que *«a borda mole é INALCANÇÁVEL quando o
/// dispositivo pinta»*: o ramo que a placa pinta devolve (`return`) **antes** da única chamada ao
/// [`ph2d_field_render::sss_shadow::blur_por_material`], logo no caminho de omissão a closure de
/// subsuperfície lia a visibilidade DURA e a chapa desenhava no jade a linha que o dono fotografou.
///
/// ⭐ **Aquele gate dizia de si mesmo que a cura era apagá-lo** (*«quando o dispositivo souber
/// entregar a visibilidade mole, esta linha sai»*), e é isso que este commit faz. ⚠️ **O `return`
/// continua lá e está CERTO:** a cura não foi voltar pelo caminho da CPU — foi o dispositivo passar
/// a assar o canal ele próprio, em duas passagens separáveis.
///
/// # O que este gate afirma no lugar dele
///
/// Que os **quatro elos** do gémeo existem. ⚠️ Ele é TEXTUAL porque medir isto a valer pede uma
/// placa, e um gate de placa é `#[ignore]` — *que é um gate que o CI nunca corre*. A medição existe
/// e é a [`super::super::paint_parity_luz_tests::a_borda_mole_da_sombra_e_a_mesma_nos_dois_motores`],
/// que lê `100,000 %` com `2 043` píxeis de canal afastado.
#[test]
fn o_gemeo_da_borda_mole_esta_ligado_no_dispositivo() {
    // ⚠️⚠️ **O corpo do pintor são TRÊS ficheiros desde 2026-09-19** (o tecto de LOC pediu a
    // fronteira, `docs/Render3d/10` §25) — e este gate lê a SOMA, porque é a soma que a placa
    // compila. ⛔ Ler só um deles fazia o gate ficar verde sobre um fragmento que ninguém junta.
    let corpo = concat!(
        include_str!("../../ph2d-field-gpu/src/paint_wgsl_sondas.rs"),
        include_str!("../../ph2d-field-gpu/src/paint_wgsl_mole.rs"),
    );
    let despacho = include_str!("../../ph2d-field-gpu/src/paint.rs");

    // (0) ⛔⛔ **E as TRÊS metades são de facto CONCATENADAS.** *Um fragmento declarado que o
    // `format!` não junta compila, passa em todo gate de texto, e não chega ao shader.*
    assert!(
        despacho.contains("{PINTOR}{PINTOR_SONDAS}{PINTOR_MOLE}"),
        "o corpo do pintor deixou de juntar as três metades — o fragmento da borda mole existe e \
         não entra no shader"
    );

    // (1) ⭐ As DUAS passagens existem — separáveis, logo duas.
    for entrada in ["fn borra_mole_h(", "fn borra_mole_v("] {
        assert!(
            corpo.contains(entrada),
            "o corpo do pintor não declara `{entrada}` — o gémeo da §12 não está escrito"
        );
    }

    // (2) ⛔⛔ E as DUAS são DESPACHADAS. *Um ponto de entrada que ninguém despacha compila,
    // fica verde em todo gate de texto, e não pinta um pixel* — é a lei do §24 deste módulo.
    for entrada in ["\"borra_mole_h\"", "\"borra_mole_v\""] {
        assert!(
            despacho.contains(entrada),
            "o {entrada} não é compilado pelo passe que pinta"
        );
    }
    assert!(
        despacho.contains("for p in [h_pass, v_pass] {"),
        "as duas passagens deixaram de ser despachadas em sequência — a segunda lê os vizinhos do \
         que a primeira escreveu, e sem as duas o canal fica a meio"
    );

    // (3) ⭐⭐⭐ E o PINTOR lê o canal — que é o elo que faz a lei chegar ao pixel.
    assert!(
        corpo.contains("mx_direct_sss(m, n, v, to_light, chega, chega_mole)"),
        "o laço das lâmpadas do pintor deixou de passar a radiância da subsuperfície à parte — ele \
         voltou ao `mx_direct` de UMA radiância, e a borda mole deixou de chegar ao ecrã"
    );
    assert!(
        corpo.contains("let bm = base_do_mole(i, l);"),
        "o pintor deixou de LER o canal mole — ele calcula-o e deita-o fora"
    );

    // (4) ⚠️ E as duas metades da decisão concordam: quem dimensiona o buffer e quem compila os
    // pipelines leem a MESMA origem. *Um `true` num lado com `None` no outro despacha duas
    // passagens sobre slots que o buffer não tem.*
    let ponte = include_str!("gpu_frame.rs");
    assert!(
        ponte.contains("mole: setup.mole.is_some(),"),
        "o `PaintSetup::mole` deixou de ser derivado do `MarchSetup::mole` — as duas metades da \
         decisão podem divergir"
    );
}

/// ⏱️⭐⭐⭐ **SONDA — o DISPOSITIVO a correr, contra a REFERÊNCIA, com e sem a chapa.**
///
/// É a medição que este módulo inteiro nunca fez: as colunas anteriores chamadas *«dispositivo»*
/// são a CPU com o canal mole desligado ([`super::Quadro::mole`]). Aqui a placa desenha.
///
/// Imprime, sobre os píxeis da ESFERA e com a **mesma** [`regua_da_banda`] das outras colunas:
///
/// - a quebra na banda (p99 da segunda diferença da luminância) e o contraste;
/// - no dispositivo e na referência;
/// - com a chapa e sem ela — que é o experimento do dono.
///
/// ⭐⭐⭐ **AS DUAS COLUNAS DA BANDA, para UMA cena** — a do dispositivo e a da referência, com a
/// MESMA régua, a MESMA luz e a MESMA câmera. Devolve `((dispositivo, contraste), (referência,
/// contraste))`.
///
/// ⚠️⚠️ **Ela existe porque a condição de fecho da [`W10`](../../../docs/Render3d/03_o_plano.md)
/// estava escrita como uma TABELA IMPRESSA** — *«a wave fecha quando as duas colunas lerem o
/// mesmo»* — e uma tabela que passa não é lida por ninguém. ⇒ a sonda imprime-a e o
/// [`as_duas_colunas_da_banda_leem_o_mesmo`] afirma-a: *uma lei, dois leitores*.
fn colunas_da_banda(
    t: &crate::gpu_frame::SharedTracer,
    doc: &ph2d_field::FieldDoc,
    cam: &ph2d_field_render::Orbit,
    onde: [f32; 3],
    luz: ph2d_field_ecs::FieldLight,
    chao: Option<ph2d_field_render::Ground>,
) -> Option<((f32, f32), (f32, f32))> {
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let m = jade();
    let mats = [m.prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &mats,
        owners: None,
    };
    let pontos = [ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: crate::lights::radiance_at_one(luz),
    }];
    let mundos = [onde];

    // ⭐ O G-buffer do MESMO dispositivo dá a máscara (quem é a esfera) e as normais que a régua lê.
    // A imagem vem do pintor do dispositivo, que é o que o produto entrega.
    let (g, _sh) =
        crate::gpu_frame::march(t, doc, &reg, cam, &mundos, chao, super::W, super::H, true)?;
    let pintura = crate::gpu_frame::paint(
        t,
        doc,
        &reg,
        cam,
        &pontos,
        &surfaces,
        &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
        [0, 0, 0, 0],
        chao,
        super::W,
        super::H,
        true,
    )?;
    let dispositivo = regua_da_banda(&g, &pintura.rgba, onde, cam);

    // A referência, com a MESMA régua e a mesma cena.
    let (g2, _, px2) = quadro(&Quadro {
        doc,
        m,
        cam,
        onde,
        luz,
        com_sombra: true,
        chao,
        sem_ceu: false,
        mole: true,
    });
    Some((dispositivo, regua_da_banda(&g2, &px2, onde, cam)))
}

/// ⭐⭐⭐ **A CONDIÇÃO DE FECHO DA `W10`, AFIRMADA** — as duas colunas lêem o mesmo.
///
/// Na cena do report (o jade com a chapa) a régua da banda lê, na árvore que ship, **`1,00` no
/// dispositivo contra `1,00` na referência**. Antes desta wave o dispositivo lia **`9,20`**, que é a
/// linha dura que o dono fotografou.
///
/// ⚠️⚠️ **A barra sai de um VALE MEDIDO, e ele é enorme:** `1,00` de um lado, `9,20` do outro — a
/// mesma cena, o mesmo commit, só o canal mole a chegar ou não ao pintor. ⇒ `≤ 2,0`, que é o dobro
/// do lado bom e menos de um quarto do mau.
///
/// ⛔⛔ **E este gate existe porque a PARIDADE não chegava:** com o dispositivo a NÃO pedir o canal,
/// o [`super::super::paint_parity_luz_tests::a_borda_mole_da_sombra_e_a_mesma_nos_dois_motores`] lia
/// `99,579 %` contra a barra de `99,5` — *passava*. A fracção afoga na média um fenómeno que ocupa
/// `10 %` dos píxeis; a régua da BANDA foi desenhada para o ver.
#[test]
#[ignore = "precisa de GPU"]
fn as_duas_colunas_da_banda_leem_o_mesmo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (com_placa, cam, onde, luz, chao) = arranjo_do_dono();
    let ((disp, c_disp), (refe, c_refe)) = colunas_da_banda(t, &com_placa, &cam, onde, luz, chao)
        .expect("a placa toma a cena do dono");
    println!(
        "  banda · dispositivo {disp:.2} (contraste {c_disp:.1}) · referência {refe:.2} (contraste {c_refe:.1})"
    );

    // ⚠️ **O CONTROLO vem primeiro:** sem contraste na banda a régua não tem sujeito, e as duas
    // colunas leriam `1,00` sobre uma imagem chata. *Uma régua que não vê o fenómeno acontecer não
    // prova que ele não aconteceu.*
    assert!(
        c_disp > 20.0 && c_refe > 20.0,
        "contraste {c_disp:.1}/{c_refe:.1} — a banda do terminador não tem sujeito nesta corrida"
    );
    assert!(
        disp <= 2.0,
        "a quebra na banda do DISPOSITIVO é {disp:.2} contra {refe:.2} da referência — o canal mole \
         não está a chegar ao pintor, e o artista vê a linha DURA que o report de 18/09 fotografou"
    );
    assert!(
        (disp - refe).abs() <= 1.0,
        "as duas colunas discordam ({disp:.2} contra {refe:.2}) — a condição de fecho da `W10` é \
         elas lerem o mesmo"
    );
}

/// ⭐ *Se a coluna do dispositivo cair quando a chapa sai, e a da referência não, então a linha é a
/// borda DURA da sombra da chapa, e a §12 é a cura que não a alcança.*
#[test]
#[ignore = "sonda: precisa de placa; imprime as quatro celulas, nao afirma"]
fn sonda_o_dispositivo_a_correr_contra_a_referencia() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("\n  ⚠️ SEM PLACA — esta sonda não tem sujeito.");
        return;
    };
    let (com_placa, cam, onde, luz, chao) = arranjo_do_dono();
    let sem_placa = so_a_bola();

    println!("\n  ── O DISPOSITIVO A CORRER, CONTRA A REFERÊNCIA ──");
    println!("    cena          ·  caminho      ·  quebra na banda  ·  contraste");

    for (nome, doc) in [("com a chapa", &com_placa), ("só a bola  ", &sem_placa)] {
        // ⭐ **A MESMA porta que o gate afirma** — [`colunas_da_banda`]. *Uma sonda que monta o
        // arranjo à mão ao lado de um gate que monta outro mede dois programas.*
        let Some(((disp, c_disp), (refe, c_refe))) =
            colunas_da_banda(t, doc, &cam, onde, luz, chao)
        else {
            println!("    {nome}  ·  DISPOSITIVO  ·  a placa recusou a peça");
            continue;
        };
        println!("    {nome}   ·  DISPOSITIVO  ·  {disp:>14.2}  ·  {c_disp:>9.1}");
        println!("    {nome}   ·  REFERÊNCIA   ·  {refe:>14.2}  ·  {c_refe:>9.1}");
    }
    println!(
        "\n    ⭐ As duas colunas lêem o mesmo desde a `W10` (2026-09-19), e isso é AFIRMADO pelo\n\
         \x20     `as_duas_colunas_da_banda_leem_o_mesmo`. ⛔ Antes dela o dispositivo lia `9,20`\n\
         \x20     com a chapa e `1,00` sem ela — a linha dura era a borda da sombra da chapa."
    );
}

/// ⭐⭐⭐ **OS DOIS CAMINHOS DESENHAM COISAS DIFERENTES, E A DIFERENÇA É SÓ A BORDA DA SOMBRA.**
///
/// # ⛔⛔ O report do dono (18/09), e porque nenhum gate o via
///
/// *«Por que a linha dura voltou em Solid? A luz está diferente?»* — com a foto do ecrã dele, onde
/// a cura da §12 **não aparece**. E, a seguir, o achado dele: ***«a presença da placa faz a linha
/// dura aparecer»***, que confirma a §11 (sem a placa, a bola sai lisa).
///
/// ⭐ **A luz NÃO está diferente**, e este gate é quem o prova: o **contraste** através da banda é o
/// mesmo nos dois caminhos (`61,2` contra `61,1`) — *a sombra está lá, com a mesma força, nos dois*.
/// O que muda é a **quebra**: `9,21` no dispositivo contra `1,00` na referência, `9,2×`.
///
/// A causa é que a §12 assou a borda mole no traçado de **CPU** e o **dispositivo ainda não tem o
/// gémeo** — ele calcula a visibilidade dentro da pintura. ⛔ Aquela secção declarou-o por escrito, e
/// **nada media a diferença**: as paridades CPU↔dispositivo ficam verdes porque *nenhuma delas assa
/// este canal* — elas comparam duas metades que concordam, e a metade em que discordam não entra.
///
/// # ⭐ Este gate reprova no dia em que o gémeo chegar, e isso é o desenho
///
/// Ele exige que a diferença **EXISTA**. Quando alguém escrever a passagem no dispositivo, ele cai —
/// e quem o curar tem de apagar a dívida declarada na §12 no mesmo gesto. *Uma diferença declarada e
/// não medida é uma nota que envelhece; uma com gate é uma propriedade com data de fim.*
///
/// ⚠️ **As três metades, e nenhuma chega sozinha:** sem (2) uma cura que apagasse a sombra dos dois
/// lados passaria em (1); sem (3) uma que a apagasse só de um lado também.
///
/// # ⛔⛔⛔ CORRECÇÃO DE 19/09: o NOME afirma o dispositivo e o CORPO nunca o correu
///
/// As duas colunas deste gate saem do [`super::quadro`], que pinta na **CPU** — a do «dispositivo» é o
/// [`super::Quadro::mole`] a `false`, um **SUCEDÂNEO**. O nome é uma afirmação sobre a placa e o corpo
/// media outro programa; *uma sonda que mede um sucedâneo mede outro programa*.
///
/// ⭐ **Corrido na placa a sério** ([`sonda_o_dispositivo_a_correr_contra_a_referencia`], 19/09), o sucedâneo **acerta nesta
/// cena**: `9,20` contra os `9,21` daqui, contraste `61,4` contra `61,1`. ⇒ este gate continua a
/// afirmar uma verdade, e passa a afirmá-la com a medição do lado.
///
/// ⚠️⚠️ **E a RAZÃO que o cabeçalho acima dá está incompleta.** Ele diz *«o dispositivo calcula a
/// visibilidade dentro da pintura»* — verdade, e a metade que falta é maior: no caminho de OMISSÃO
/// do produto a [`crate::smoke_draw_thread::traca`] **devolve** no ramo que o dispositivo pinta,
/// **antes** de o borrão da §12 poder correr. ⇒ a cura não está *«por escrever no dispositivo»*:
/// ela **existe, está gateada ao bit, e está fora do caminho que o artista corre**.
/// ⭐⭐⭐ **O CANAL MOLE MUDA A BORDA E NÃO MUDA A FORÇA DA SOMBRA** — o controlo da LEI.
///
/// # ⛔⛔ Ele chamava-se `o_dispositivo_ainda_desenha_a_borda_dura_e_a_referencia_nao`, e o nome MENTIA
///
/// As duas colunas que ele compara saem do [`super::Quadro::mole`], que pinta **na CPU** com o canal
/// ligado e desligado — um SUCEDÂNEO, como o cabeçalho deste módulo já dizia. ⇒ ele nunca correu o
/// dispositivo, e quando o gémeo chegou (2026-09-19) ele continuou VERDE sobre uma frase que passou
/// a ser falsa. *Um gate que mede um sucedâneo e promete o produto no nome fica verde no dia em que
/// a dívida é paga.*
///
/// ⭐ O que ele de facto mede é bom e fica: **o canal mole muda a BORDA (`≥ 3×`) e não muda a FORÇA
/// da sombra (`≤ 5 %`)** — que é a resposta ao dono: *«a luz é a mesma; o que muda é só a borda»*.
/// Quem corre o dispositivo é a
/// [`super::super::paint_parity_luz_tests::a_borda_mole_da_sombra_e_a_mesma_nos_dois_motores`].
#[test]
fn o_canal_mole_muda_a_borda_e_nao_a_forca_da_sombra() {
    let (doc, cam, onde, luz, chao) = arranjo_do_dono();
    let jade = jade();
    let (dura, contraste_duro) = super::quebra_na_banda(&doc, jade, &cam, onde, luz, chao, false);
    let (mole, contraste_mole) = super::quebra_na_banda(&doc, jade, &cam, onde, luz, chao, true);

    // (1) ⭐ O canal MUDA a borda — medido `9,21` contra `1,00`, e a barra é `3×` para não medir
    // ruído. ⚠️ **As duas colunas são a CPU**, com o canal desligado e ligado: é o efeito da LEI, e
    // não uma comparação entre motores.
    assert!(
        dura >= mole * 3.0,
        "o canal mole deixou de mudar a borda (duro {dura:.2} contra mole {mole:.2}) — a lei da \
         §12 ficou inerte, e com ela a cura que o dono aprovou em 18/09"
    );
    // (2) ⭐⭐⭐ A METADE QUE RESPONDE AO DONO: a LUZ é a mesma. O contraste através da banda mede a
    // FORÇA da sombra, e ele não se mexe — o que muda é só a borda dela.
    assert!(
        (contraste_duro - contraste_mole).abs() <= contraste_duro * 0.05,
        "o contraste da banda mudou entre os caminhos ({contraste_duro:.1} contra          {contraste_mole:.1}) — então não é só a BORDA que difere, e a resposta «a luz é a mesma»          deixou de ser verdade"
    );
    // (3) ⚠️ E os DOIS continuam a ter sombra: sem isto, um caminho que a apagasse leria a banda
    // lisíssima e passaria em (1) pelo motivo errado.
    assert!(
        contraste_duro >= 8.0 && contraste_mole >= 8.0,
        "a sombra desapareceu num dos caminhos ({contraste_duro:.1} · {contraste_mole:.1})"
    );
}
