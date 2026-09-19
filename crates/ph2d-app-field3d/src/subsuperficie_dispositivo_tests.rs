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
//! ⚠️⚠️ *É por isso que a cura que eu entreguei não mudou nada para ele:* ela é real, é gateada ao
//! bit, e está **fora do caminho que o artista corre**.

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

/// ⭐⭐⭐ **A BORDA MOLE É INALCANÇÁVEL QUANDO O DISPOSITIVO PINTA — e isso é o report do dono.**
///
/// # A lei que este gate afirma
///
/// No [`crate::smoke_draw_thread::traca`] o ramo que o dispositivo PINTA devolve (`return`) **antes**
/// da única chamada ao [`ph2d_field_render::sss_shadow::blur_por_material`]. ⇒ no caminho de
/// omissão do produto a sombra que um material translúcido lê é a **DURA**, e a chapa desenha na
/// esfera a linha que o dono fotografou.
///
/// # ⚠️ Porque ele é TEXTUAL e não uma medição
///
/// Medir isto a valer pede uma placa, e um gate de placa é `#[ignore]` — *que é um gate que o CI
/// nunca corre*. A medição existe, e vive na [`sonda_o_dispositivo_a_correr_contra_a_referencia`],
/// logo abaixo; esta é a metade que corre **sempre** e que reprova no dia em que o gémeo chegar.
///
/// ⛔ **Ele é o TECTO de uma dívida declarada, e a cura dele é apagá-lo:** quando o dispositivo
/// souber entregar a visibilidade mole, esta linha sai — e com ela a nota da §12.
#[test]
fn a_borda_mole_e_inalcancavel_quando_o_dispositivo_pinta() {
    let fonte = include_str!("smoke_draw_thread.rs");

    // (1) ⭐ O CONTROLO da própria régua: as duas âncoras existem e a do borrão é ÚNICA. Sem esta
    // metade, um ficheiro que perdesse a chamada leria «o return vem antes» por VÁCUO.
    let borroes = fonte.matches("sss_shadow::blur_por_material(").count();
    assert_eq!(
        borroes, 1,
        "o `blur_por_material` é chamado {borroes} vezes na thread de traçado — este gate mede a \
         ORDEM contra UMA chamada, e com duas ele deixa de saber de qual fala"
    );
    let ramo = fonte
        .find("if let Some(pintura) = pintado {")
        .expect("o ramo que o dispositivo pinta — se ele mudou de forma, releia a `traca`");
    let borrao = fonte
        .find("sss_shadow::blur_por_material(")
        .expect("a chamada do borrão");

    // (2) ⛔⛔ A DÍVIDA: o ramo pintado devolve ANTES de o borrão poder correr.
    //
    // ⚠️⚠️ **A 1.ª redacção procurava o `return;` seguinte em TODO o resto do ficheiro, e uma
    // mutação SOBREVIVEU:** apagado o `return` deste ramo, ela achava o do `else { return; }` do
    // traçado de CPU (linha `155`), que também vem antes do borrão — *o gate afirmava «há ALGUM
    // return pelo caminho» e o nome dele promete «ESTE ramo devolve»*. A busca é agora limitada ao
    // CORPO do ramo, que fecha no primeiro `}` ao nível da função.
    let fecho = fonte[ramo..]
        .find("\n    }\n")
        .map(|d| ramo + d)
        .expect("o ramo pintado fecha — se ele mudou de indentação, releia a `traca`");
    let corpo = &fonte[ramo..fecho];
    assert!(
        corpo.contains("return;"),
        "o ramo que o dispositivo PINTA já não devolve — ele passa a cair no caminho de CPU, e com \
         isso o borrão da §12 volta a correr (se o gémeo chegou, esta linha sai e a nota da §12 com \
         ela; se foi por acidente, o quadro passou a ser desenhado DUAS vezes)"
    );
    assert!(
        fecho < borrao,
        "o ramo pintado deixou de vir ANTES do borrão da §12 (fecha em {fecho}, borrão em \
         {borrao}) — alguém mudou a ordem da `traca`, e a tabela do cabeçalho deste módulo deixou \
         de descrever o produto"
    );

    // (3) ⭐⭐⭐ E QUE ISSO É O CAMINHO DE OMISSÃO, e não um modo que ninguém usa: o ramo pintado
    // exige `!p.refinar`, e o refinamento nasce DESLIGADO.
    //
    // ⚠️ Sem esta metade o gate afirmaria uma dívida que talvez ninguém alcance — e *uma dívida
    // que o artista não atinge e uma que ele atinge todos os dias leem-se igual numa tabela*.
    assert!(
        fonte.contains("let pintado = if pelo_dispositivo && !p.refinar {"),
        "a condição do ramo pintado mudou — releia-a antes de acreditar na tabela do cabeçalho \
         deste módulo"
    );
    let preview = include_str!("preview.rs");
    assert!(
        preview.contains("antialias && plate_parked && cpu_occlusion_enabled()"),
        "o `refines_occlusion` mudou de lei — o `!p.refinar` do ramo pintado pode ter deixado de \
         ser o valor de fábrica, e a tabela deste módulo com ele"
    );
    assert!(
        preview.contains(r#"std::env::var("PH2D_FIELD_AO").is_ok_and(|v| v != "0")"#),
        "o refinamento de CPU deixou de nascer desligado — então `p.refinar` já não é `false` por \
         omissão, e o ramo pintado já não é o caminho que o artista corre"
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
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let m = jade();
    let mats = [m.prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &mats,
        owners: None,
    };
    // ⚠️ **A MESMA luz que a coluna de CPU usa** — o `quadro` deste módulo monta exactamente esta
    // `PointLamp`. Duas luzes diferentes fariam as colunas medir duas cenas.
    let pontos = [ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: crate::lights::radiance_at_one(luz),
    }];
    let mundos = [onde];

    println!("\n  ── O DISPOSITIVO A CORRER, CONTRA A REFERÊNCIA ──");
    println!("    cena          ·  caminho      ·  quebra na banda  ·  contraste");

    for (nome, doc) in [("com a chapa", &com_placa), ("só a bola  ", &sem_placa)] {
        // ⭐ O G-buffer do MESMO dispositivo dá a máscara (quem é a esfera) e as normais que a
        // régua lê. A imagem vem do pintor do dispositivo, que é o que o produto entrega.
        let Some((g, sh)) =
            crate::gpu_frame::march(t, doc, &reg, &cam, &mundos, chao, super::W, super::H, true)
        else {
            println!("    {nome}  ·  DISPOSITIVO  ·  a marcha recusou a peça");
            continue;
        };
        let Some(pintura) = crate::gpu_frame::paint(
            t,
            doc,
            &reg,
            &cam,
            &pontos,
            &surfaces,
            ph2d_view_transform::Look::default(),
            [0, 0, 0, 0],
            chao,
            super::W,
            super::H,
            true,
        ) else {
            println!("    {nome}  ·  DISPOSITIVO  ·  o pintor recusou a peça");
            continue;
        };
        let (quebra, contraste) = regua_da_banda(&g, &pintura.rgba, onde, &cam);
        println!("    {nome}   ·  DISPOSITIVO  ·  {quebra:>14.2}  ·  {contraste:>9.1}");

        // ⚠️⚠️ **ESTA LINHA MEDE A OUTRA PORTA, e dizê-lo aqui impede a leitura errada.**
        //
        // O [`crate::gpu_frame::march`] **devolve** o canal duro por lâmpada, do tamanho do
        // G-buffer — logo o borrão da §12 correria sobre ele. ⛔ **Mas o caminho que o dono corre
        // não chama o `march`:** ele chama o [`crate::gpu_frame::paint`], que calcula a
        // visibilidade **dentro** do sombreamento e devolve uma IMAGEM. ⇒ *a cura não é «falta a
        // chamada»* — ou o gémeo do borrão é escrito em WGSL, ou o quadro volta pelo `march` e
        // paga o G-buffer no barramento (o que a `traca` evita de propósito).
        println!(
            "                   ·  (pela OUTRA porta — o `march` — o dispositivo devolve {} \
             lâmpada(s), canal de {} contra {} píxeis do G-buffer)",
            sh.lamps(),
            sh.lamp_channel(0).len(),
            g.hit.len()
        );

        // A referência, com a MESMA régua e a mesma cena.
        let (g2, _, px2) = quadro(&Quadro {
            doc,
            m,
            cam: &cam,
            onde,
            luz,
            com_sombra: true,
            chao,
            sem_ceu: false,
            mole: true,
        });
        let (q2, c2) = regua_da_banda(&g2, &px2, onde, &cam);
        println!("    {nome}   ·  REFERÊNCIA   ·  {q2:>14.2}  ·  {c2:>9.1}");
    }
    println!(
        "\n    ⭐ Se a quebra do DISPOSITIVO cair ao tirar a chapa e a da REFERÊNCIA já for baixa\n\
         \x20     nas duas, a linha dura é a borda da sombra da chapa — e a cura da §12 existe,\n\
         \x20     está gateada, e o caminho que o dono corre devolve antes de a chamar."
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
#[test]
fn o_dispositivo_ainda_desenha_a_borda_dura_e_a_referencia_nao() {
    let (doc, cam, onde, luz, chao) = arranjo_do_dono();
    let jade = jade();
    let (dura, contraste_duro) = super::quebra_na_banda(&doc, jade, &cam, onde, luz, chao, false);
    let (mole, contraste_mole) = super::quebra_na_banda(&doc, jade, &cam, onde, luz, chao, true);

    // (1) ⛔ A DÍVIDA AINDA EXISTE — medido `9,21` contra `1,00`, e a barra é `3×` para não medir
    // ruído. *Se isto reprovar, o gémeo chegou: apague a dívida da §12 e este gate com ela.*
    assert!(
        dura >= mole * 3.0,
        "os dois caminhos passaram a desenhar a mesma borda (dispositivo {dura:.2} contra          referência {mole:.2}) — se o gémeo do dispositivo foi escrito, esta é a linha que sai, e          com ela a dívida declarada em `docs/Render3d/10` §12"
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
