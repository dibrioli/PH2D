//! ⭐⭐⭐ **O VISOR ACENDE COM A LEI QUE ASSA** — a sonda e o gate do modo `Pbr`.
//!
//! Irmão (`#[path]`) do [`super`] pelo mesmo motivo dos outros deste módulo: o corte é de ASSUNTO.
//!
//! # O report que a pediu
//!
//! *«O que se vê no objeto 3d não é o que se vê na sprite cozida. Já temos o material do módulo
//! Modelling. porque não trazer para o sculpt?»* (o dono, 2026-09-20) — e a seguir *«A luz 3d também
//! deveria ser trazida»*.
//!
//! O que a medição respondeu antes da primeira linha de produto: **a LUZ já era a mesma** (as duas
//! metades leem o mesmo [`ph2d_light::LightRig`] e, desde a coluna B3, o mesmo céu por
//! [`ph2d_light::env_ramp`]). O que era diferente era o **MATERIAL** — o visor shipava um barro com
//! três números cravados no shader e o sprite já era OpenPBR — e o **OLHAR**, que o visor não
//! aplicava de todo.
//!
//! # ⛔⛔ O ORÁCULO É A REFERÊNCIA EM CPU, e a escolha custou uma medição errada
//!
//! A primeira redacção desta sonda comparava o visor contra o que o [`super::compare`] produz — e
//! **aquele harness acende o sprite pelo [`ph2d_render::ImpastoLightPass`], que é a lei da TINTA**,
//! não a [`ph2d_form_pbr::acende_texel`]. O nome dele (*«as duas luzes sobre a mesma forma»*) não
//! diz **qual**, e a tabela que ele imprimiu foi lida por mim como *«as duas implementações da mesma
//! lei divergem `0,055`»* quando o que ela media eram **duas leis diferentes**.
//!
//! ⚠️ *Um oráculo escolhido pelo harness que estava à mão mede o que aquele harness mede.* Aqui o
//! oráculo é a **função** que assa um texel, chamada com o que a forma doa — sem passe de tinta, sem
//! viagem por oito bits, sem uma segunda calibração pelo meio.
//!
//! # O que SOBRA, e é declarado
//!
//! 1. o visor escreve **HDR** e a lei devolve display-referred já cortado — o `vt_to_display` é o
//!    mesmo dos dois lados, logo isto não é uma fonte de desvio aqui;
//! 2. a normal viaja pelo **G-buffer** (`rgba8`) e a lei normaliza-a; o visor tem a normal do
//!    fragmento em `f32`. É a maior fonte de resíduo desta sonda, e é o que a barra mede;
//! 3. a vista do escultor é em **PERSPECTIVA** e a tela é ortográfica — as duas metades lêem
//!    `(0,0,1)` de propósito (o `PBR_VIEW`), logo esta sonda não a mede.
//!
//! ```text
//! cargo test -p ph2d-app-sculpt3d --release o_visor_acende_com_a_lei_que_assa -- --ignored --nocapture
//! ```

use super::shade::pbr_law_shade;
use super::{CLAY, SIDE, depth_from_edge, gpu, render_live, stage};
use ph2d_form_donation::lei::{Ceu, Lampada, Texel, acende_texel};

/// O albedo dos dois lados: o `CLAY * vcolor` que o shader usa, com `vcolor = 1`.
///
/// ⚠️ **Ele é o do SHADER e não o byte do sprite.** A sonda irmã veste o sprite com `[189, 179, 168]`
/// porque ali o albedo entra por uma textura de oito bits; aqui a lei é chamada directamente e o
/// controlo honesto é o MESMO `f32` que o visor multiplica — *um albedo arredondado a bytes poria
/// `0,16 %` de desvio na tabela sem que uma linha de lei divergisse*.
const ALBEDO: [f32; 3] = CLAY;

/// ⭐⭐⭐ **A BARRA, DERIVADA DO RECURSO: meio código de um canal de oito bits.**
///
/// ⚠️ **Ela não é um epsilon escolhido.** O sprite é `unorm8`: um desvio abaixo de meio código NÃO
/// PODE mudar um byte da imagem assada, logo é a maior barra que ainda afirma *«o que o artista vê
/// no visor é o que ele vai assar»*. Medido, o visor fica em `0,000797` — **um quinto** dela.
///
/// ⛔ Uma barra folgada é onde uma régua errada sobrevive: a 1.ª redacção desta sonda media a lei da
/// TINTA e lia `0,146`, e qualquer número redondo acima disso a teria deixado passar.
const MEIO_CODIGO: f32 = 1.0 / 510.0;

/// O que a sonda devolve: o pior e o médio desvio por canal, e quantos texels entraram.
struct Residuo {
    medio: f64,
    pior: f32,
    dentro: usize,
}

/// **O visor, pixel a pixel, contra a [`acende_texel`].**
///
/// ⚠️ O `olhar` é PARÂMETRO e não uma constante lida aqui dentro: é ele que faz o CONTROLO desta
/// sonda existir — *uma medição que só sabe correr a configuração certa não pode mostrar que ela é
/// a certa*.
fn residuo(olhar: ph2d_mesh_render::Look) -> Option<Residuo> {
    let gpu = gpu()?;
    let (mut renderer, camera, rig) = stage(&gpu);
    let vista = ph2d_mesh_render::Shade {
        look: olhar,
        ..pbr_law_shade()
    };
    let size = (SIDE, SIDE);
    let resolved = ph2d_light::resolve(&rig).expect("o rig default tem lampada acesa");

    let vivo = render_live(&gpu, &mut renderer, &camera, &resolved, size, vista);
    let forma = renderer
        .form_plane(&gpu.device, &gpu.queue, &camera, size, vista, None)
        .expect("a malha esta la'");

    // ⭐ **As lâmpadas e o céu saem das MESMAS portas que o sprite atravessa** — a tradução do
    // vocabulário do rig e a rampa do ambiente. Escrevê-las aqui faria desta sonda a segunda
    // resposta a *«que luz é esta»*, e ela mediria a si própria.
    let lampadas: Vec<Lampada> = ph2d_form_donation::baked_form::lampadas_do_rig(&rig)
        .expect("o rig default tem lampada acesa");
    let ceu: Ceu = ph2d_form_donation::baked_form::ceu_do_rig(&lampadas);
    let material = ph2d_form_donation::lei_da_luz::material_da_forma();
    // ⛔ **O ORÁCULO corre SEMPRE com o olhar da casa** — ele é a lei que assa, e a sprite não tem
    // outra. É a VISTA que varia, e é por isso que o controlo mede alguma coisa.
    let olhar_da_lei = ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA;

    let profundidade = depth_from_edge(&forma.normal, SIDE, SIDE);
    let n = (SIDE * SIDE) as usize;
    let (mut soma, mut pior, mut dentro) = (0f64, 0f32, 0usize);
    for i in 0..n {
        if profundidade[i] == u32::MAX {
            continue;
        }
        dentro += 1;
        let esperado = acende_texel(
            &material,
            &Texel {
                normal: [
                    forma.normal[i * 4],
                    forma.normal[i * 4 + 1],
                    forma.normal[i * 4 + 2],
                ],
                albedo: ALBEDO,
                cobertura: 1.0,
                oclusao: forma.occlusion[i],
            },
            &lampadas,
            ceu,
            olhar_da_lei,
        );
        let mut d = 0f32;
        for k in 0..3 {
            d = d.max((vivo[i * 4 + k] - esperado[k]).abs());
        }
        soma += f64::from(d);
        pior = pior.max(d);
    }
    Some(Residuo {
        medio: soma / dentro as f64,
        pior,
        dentro,
    })
}

/// **A TABELA que escolheu a barra.**
#[test]
#[ignore = "precisa de adapter"]
fn mede_o_visor_contra_a_lei_que_assa() {
    let Some(r) = residuo(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA) else {
        eprintln!("sem adapter: nada a medir");
        return;
    };
    let Some(cru) = residuo(ph2d_mesh_render::Look::default()) else {
        return;
    };
    println!(
        "  CONTROLO (o visor SEM o olhar): medio {:.6} · pior {:.6}",
        cru.medio, cru.pior
    );
    println!(
        "\n=== o visor (Lighting::Pbr) contra a acende_texel ===\n  \
         texels dentro da silhueta: {}\n  desvio medio por canal: {:.6}\n  pior: {:.6}",
        r.dentro, r.medio, r.pior
    );
}

/// ⭐⭐⭐ **O MODO `Pbr` DO VISOR É A LEI QUE ASSA, e não uma segunda redacção dela.**
///
/// ⚠️ **A barra não é um epsilon escolhido: ela sai do RESÍDUO NOMEADO** — a normal que a lei recebe
/// vem do G-buffer em `rgba8` e a que o visor usa é a do fragmento em `f32`. Um texel a meio caminho
/// entre dois códigos de oito bits move a normal `~1/255` por eixo, e num lóbulo especular isso é
/// o maior termo desta tabela.
///
/// ⛔ **Uma barra larga é onde uma régua errada sobrevive** — foi assim que a redacção anterior
/// desta sonda passou a comparar o visor contra a lei da TINTA sem que nada acusasse. Por isso ela
/// leva o CONTROLO ao lado: a MESMA medição com o olhar NEUTRO tem de reprovar por uma ordem de
/// grandeza, senão a sonda não está a ver o último acto da lei.
#[test]
#[ignore = "precisa de adapter"]
fn o_visor_acende_com_a_lei_que_assa() {
    let Some(r) = residuo(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA) else {
        eprintln!("sem adapter: nada a afirmar");
        return;
    };
    assert!(
        r.dentro > 40_000,
        "controlo: a silhueta tem de encher o quadro ({} texels)",
        r.dentro
    );
    assert!(
        r.pior < MEIO_CODIGO,
        "o pior texel afasta-se {:.6} da lei que assa (medido {:.6})",
        r.pior,
        r.medio
    );

    // ⭐⭐⭐ **O CONTROLO, e ele é a metade que torna a barra uma afirmação.** Com o olhar NEUTRO o
    // visor entrega a radiância CRUA e a lei continua a entregar a exposta — se esta sonda não visse
    // o último acto da lei, as duas colunas leriam o mesmo e a barra acima estaria a aprovar o nada.
    let cru = residuo(ph2d_mesh_render::Look::default()).expect("o adapter ja' existia");
    assert!(
        cru.pior > r.pior * 100.0,
        "controlo: sem o olhar o desvio tinha de EXPLODIR, e leu {:.6} contra {:.6}",
        cru.pior,
        r.pior
    );
}

/// ⭐⭐⭐ **E O PRODUTO LIGA AS DUAS PORTAS — a metade que o gate de paridade NÃO pode ver.**
///
/// ⚠️⚠️ **O [`o_visor_acende_com_a_lei_que_assa`] entra pelo [`super::shade::pbr_law_shade`], que é
/// uma vista de SONDA.** Ele afirma que a LEI do ramo `Pbr` é a que assa — e ficaria verde no dia em
/// que o `view.rs` escrevesse um `OpenPbr::default()` seu ou deixasse o olhar neutro, porque a sonda
/// nunca percorre a rota do produto. *Um motor com a lei certa e a shell a não a ligar lê-se como um
/// motor sem a lei* — é a 5.ª vez que esta casa o paga, e o preço aqui seria o report do dono a
/// voltar com a paridade verde por cima.
///
/// ⛔ **Por TEXTO e não por chamada:** a `Sculpt3dScene::new` pede um `wgpu::Device`, logo um gate
/// que construísse a cena nasceria `#[ignore]` e **o CI nunca o correria**. Este corre sempre.
///
/// **Mutações que devem sangrar:** trocar o `openpbr_da_forma()` por `OpenPbr::default()`; trocar o
/// `OLHAR_DA_FORMA` por `Look::default()`; apagar qualquer uma das duas linhas.
#[test]
fn o_visor_le_o_material_e_o_olhar_da_lei_que_assa() {
    let fonte = include_str!("view.rs");
    for agulha in [
        "material: ph2d_form_donation::lei_da_luz::openpbr_da_forma()",
        "look: ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA",
    ] {
        assert!(
            fonte.contains(agulha),
            "o `shade()` do visor deixou de ler a porta da lei que assa: `{agulha}` sumiu"
        );
    }
    // ⭐ **O CONTROLO NEGATIVO:** o `view.rs` não pode voltar a escrever um material seu. Sem esta
    // metade as duas agulhas acima ficariam verdes com uma SEGUNDA linha ao lado a sobrepor-se.
    //
    // ⛔⛔ **E o CÓDIGO é varrido sem os COMENTÁRIOS, porque a 1.ª redacção acusou o doc que EXPLICA
    // a cura.** O `view.rs` diz, na linha acima da cura, *«esta linha era `OpenPbr::default()` e era
    // a SEGUNDA resposta»* — e a varredura crua leu a prosa como sendo o defeito. É a forma que a
    // Fase B da física já registou (*«uma régua textual a varrer lê o doc-comment que explica a cura
    // e acusa 93 ficheiros de 133»*), e a cura não é apagar a memória: é a régua olhar para CÓDIGO.
    let codigo: String = fonte
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !codigo.contains("OpenPbr::default()"),
        "o `view.rs` voltou a escrever o material dele — a porta da lei passou a ser a segunda resposta"
    );
}
