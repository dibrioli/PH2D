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

/// **A VISTA DA SONDA com um olhar escolhido** — o modo `Pbr`, que é a lei em prova.
fn vista_com(olhar: ph2d_mesh_render::Look) -> ph2d_mesh_render::Shade {
    ph2d_mesh_render::Shade {
        look: olhar,
        ..pbr_law_shade()
    }
}

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
fn residuo(vista: ph2d_mesh_render::Shade) -> Option<Residuo> {
    let gpu = gpu()?;
    let (mut renderer, camera, rig) = stage(&gpu);
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
    let Some(r) = residuo(vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA)) else {
        eprintln!("sem adapter: nada a medir");
        return;
    };
    let Some(cru) = residuo(vista_com(ph2d_mesh_render::Look::default())) else {
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
    let Some(r) = residuo(vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA)) else {
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
    let cru = residuo(vista_com(ph2d_mesh_render::Look::default())).expect("o adapter ja' existia");
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

/// ⭐⭐⭐⭐ **A RÉGUA QUE FALTAVA: cada MODO do visor contra a lei que assa.**
///
/// # Porque ela não existia, e porque o report do dono voltou duas vezes
///
/// O [`o_visor_acende_com_a_lei_que_assa`] entra pelo [`pbr_law_shade`], que **crava**
/// `Lighting::Pbr`. Ele afirma que o RAMO `Pbr` é a lei que assa — e é verdade. O que ele nunca
/// pergunta é **o que o app MOSTRA**, e é essa a pergunta do dono (*«o bake não é idêntico ao que
/// se vê em 3d»*).
///
/// ⛔⛔ **E a resposta é estrutural, não um epsilon:** o `BakedForm` não tem campo de modo de luz
/// **nem de material** — o bake corre SEMPRE a lei OpenPBR —, enquanto o visor corre o que o
/// `lighting` disser. Com o valor de fábrica ([`ph2d_mesh_render::DEFAULT_LIGHTING`]) a ser um
/// MATCAP, *o que se vê e o que se assa são duas leis diferentes por construção*, e nenhuma
/// paridade medida DENTRO do ramo `Pbr` o pode ver.
///
/// Corre-se com o filtro `mede_cada_modo` sobre esta crate, em release, com `--ignored --nocapture`.
#[test]
#[ignore = "precisa de adapter"]
fn mede_cada_modo_do_visor_contra_a_lei_que_assa() {
    println!("\n=== cada modo do visor contra a lei que assa (a MESMA forma, o MESMO rig) ===");
    println!("  {:<14} | {:>12} | {:>12} |", "modo", "médio", "pior");
    for modo in ph2d_mesh_render::Lighting::todos() {
        let vista = ph2d_mesh_render::Shade {
            lighting: modo,
            ..vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA)
        };
        let Some(r) = residuo(vista) else {
            eprintln!("sem adapter: nada a medir");
            return;
        };
        let fabrica = if modo == ph2d_mesh_render::DEFAULT_LIGHTING {
            "  <-- o que o app MOSTRA de fábrica"
        } else {
            ""
        };
        println!(
            "  {:<14} | {:>12.6} | {:>12.6} |{}",
            format!("{modo:?}"),
            r.medio,
            r.pior,
            fabrica
        );
    }
}

/// **O PREÇO de cada modo** — a diferença, porque só ela isola a LEI.
///
/// ⚠️ O [`super::render_live`] cria a textura e lê-a de volta, e as duas coisas custam o MESMO em
/// todo modo: um número absoluto aqui mediria a leitura de volta, não o sombreamento. O que esta
/// sonda publica é o **mínimo de N corridas por modo**, e o que se lê dela é a COLUNA DA DIFERENÇA
/// contra o matcap — *um `Δ` de dois relógios com o mesmo overhead é a única parte que é da lei*.
///
/// ⛔ Ela IMPRIME e não afirma: um gate de relógio é candidato à família de flakes de fan-out do
/// `CLAUDE.md` §5.0, e este número existe para uma DECISÃO de produto, não para uma catraca.
#[test]
#[ignore = "precisa de adapter"]
fn mede_o_preco_de_cada_modo() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a medir");
        return;
    };
    let (mut renderer, camera, rig) = stage(&gpu);
    let resolved = ph2d_light::resolve(&rig).expect("o rig default tem lampada acesa");
    let base = vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA);
    // ⚠️ O tamanho é o de uma VISTA e não o da sonda de paridade: o custo é por FRAGMENTO, logo
    // medi-lo num quadrado pequeno responde sobre um ecrã que ninguém tem.
    let size = (1600u32, 900u32);

    let mut medir = |modo| {
        let vista = ph2d_mesh_render::Shade {
            lighting: modo,
            ..base
        };
        let mut melhor = f64::MAX;
        for _ in 0..7 {
            let t = std::time::Instant::now();
            let _ = render_live(&gpu, &mut renderer, &camera, &resolved, size, vista);
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1000.0);
        }
        melhor
    };

    let matcap = medir(ph2d_mesh_render::Lighting::Matcap(0));
    println!(
        "\n=== o preço de cada modo a {}x{} (mínimo de 7) ===",
        size.0, size.1
    );
    println!("  {:<14} | {:>10} | {:>12}", "modo", "ms", "Δ vs matcap");
    for modo in ph2d_mesh_render::Lighting::todos() {
        let ms = medir(modo);
        println!(
            "  {:<14} | {:>10.3} | {:>+12.3}",
            format!("{modo:?}"),
            ms,
            ms - matcap
        );
    }
}

/// ⭐⭐⭐⭐ **O QUE O APP MOSTRA DE FÁBRICA É A LEI QUE ASSA — o gate que faltava, e que teria
/// apanhado o report do dono antes de ele o ver.**
///
/// # Porque o gate irmão não bastava
///
/// O [`o_visor_acende_com_a_lei_que_assa`] entra pelo [`pbr_law_shade`], que **crava**
/// `Lighting::Pbr`: ele prova que o RAMO existe e é a lei certa, e ficaria **verde para sempre** com
/// o app a abrir num matcap. *Um gate que escolhe o modo que quer medir afirma sobre um programa que
/// o artista não está a correr* — e o preço disso foi o mesmo report duas vezes.
///
/// ⇒ Este entra por [`ph2d_mesh_render::DEFAULT_LIGHTING`], que é **o que o app mostra**.
///
/// ⚠️ **A barra é a mesma [`MEIO_CODIGO`]**, e pela mesma razão: abaixo de meio código de oito bits
/// nenhum byte do sprite assado pode mudar, logo é a maior barra que ainda afirma *«o que o artista
/// vê é o que ele vai assar»*.
///
/// ⛔ **E ele leva o CONTROLO dentro:** um matcap tem de reprovar por uma ordem de grandeza. Sem
/// essa metade, o dia em que a sonda deixasse de ver a diferença entre duas leis (uma forma degenerada,
/// um render vazio) este gate ficaria verde **a afirmar nada** — que é exactamente como a família
/// anterior de réguas desta linha falhou.
#[test]
#[ignore = "precisa de adapter"]
fn o_que_o_app_mostra_de_fabrica_e_a_lei_que_assa() {
    let vista = ph2d_mesh_render::Shade {
        lighting: ph2d_mesh_render::DEFAULT_LIGHTING,
        ..vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA)
    };
    let Some(r) = residuo(vista) else {
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
        "o app abre em {:?}, que se afasta {:.6} da lei que assa (médio {:.6}) — \
         é o report do dono: «o bake não é idêntico ao que se vê em 3d»",
        ph2d_mesh_render::DEFAULT_LIGHTING,
        r.pior,
        r.medio
    );

    // ⭐⭐⭐ **O CONTROLO — a metade que torna a barra uma afirmação.** Um matcap é a luz do OLHO e
    // NÃO pode passar aqui: se passasse, esta sonda teria deixado de distinguir duas leis.
    let matcap = residuo(ph2d_mesh_render::Shade {
        lighting: ph2d_mesh_render::Lighting::Matcap(0),
        ..vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA)
    })
    .expect("o adapter ja' existia");
    assert!(
        matcap.pior > r.pior * 100.0,
        "controlo: um matcap tinha de divergir por uma ordem de grandeza, e leu {:.6} contra {:.6}",
        matcap.pior,
        r.pior
    );
}

/// **A OCLUSÃO DE TELA sobrevive ao ENQUADRAMENTO?** — a folga que esta sonda tinha e não via.
///
/// ⛔⛔ **O [`residuo`] pede a forma com `ssao: None`, logo ele é CEGO a este termo.** E ele não é
/// um detalhe: o valor de fábrica é [`ph2d_mesh_render::DEFAULT_SSAO_STRENGTH`] `= 1,0` — a oclusão
/// de tela entra INTEIRA de fábrica. Ela é medida no enquadramento de quem a mede: o visor mede-a
/// numa vista LARGA, e o bake mede-a no quadrado do SPRITE. *Duas medições da mesma sombra em dois
/// enquadramentos são duas respostas, e o `donation.rs` declara-o por escrito.*
///
/// ⚠️ **A régua é a média sobre a SILHUETA**, e não pixel a pixel: os dois planos têm resoluções
/// diferentes, logo comparar índices compararia sítios diferentes da peça. Se a lei for invariante
/// ao enquadramento, as duas médias têm de cair dentro de meio código.
///
/// Corre-se com o filtro `a_oclusao_de_tela_sobrevive_ao_enquadramento` sobre esta crate, com
/// `--ignored --nocapture`.
#[test]
#[ignore = "precisa de adapter"]
fn a_oclusao_de_tela_sobrevive_ao_enquadramento() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a medir");
        return;
    };
    let (mut renderer, camera, _rig) = stage(&gpu);
    let vista = vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA);

    // ⚠️ O `ssao` do `pbr_law_shade` está a ZERO (a vista da sonda desliga as ajudas), logo ele é
    // reposto AQUI no valor de FÁBRICA — é esse o programa que o artista corre.
    let vista = ph2d_mesh_render::Shade {
        ssao: ph2d_mesh_render::DEFAULT_SSAO_STRENGTH,
        ..vista
    };
    let params =
        ph2d_mesh_render::SsaoParams::for_bounds(ph2d_mesh::shapes::uv_sphere(8, 12, 1.0).bounds());

    let mut media = |size: (u32, u32)| {
        let planes = renderer
            .form_plane(&gpu.device, &gpu.queue, &camera, size, vista, Some(params))
            .expect("a malha esta la'");
        let (mut soma, mut n) = (0f64, 0usize);
        for (i, o) in planes.occlusion.iter().enumerate() {
            // Só dentro da silhueta: o alvo é limpo em BRANCO, e o fundo puxaria a média para 1.
            if planes.normal[i * 4 + 3] > 0.0 {
                soma += f64::from(*o);
                n += 1;
            }
        }
        (soma / n as f64, n)
    };

    let (largo, n_largo) = media((1600, 900));
    let (quadrado, n_quadrado) = media((1024, 1024));
    let dif = (largo - quadrado).abs();
    println!(
        "\n=== a oclusão de tela em dois enquadramentos (a MESMA câmera, a MESMA malha) ===\n  \
         vista LARGA  1600x900  : média {largo:.6} sobre {n_largo} texels\n  \
         sprite QUADRADO 1024²  : média {quadrado:.6} sobre {n_quadrado} texels\n  \
         diferença              : {dif:.6}  (meio código = {MEIO_CODIGO:.6})"
    );
}

/// ⭐⭐⭐⭐ **O VISOR contra a SPRITE QUE O PRODUTO DE FACTO ASSA — o report do dono, com número.**
///
/// # Porque nenhuma sonda desta linha o via
///
/// Todas as outras comparam o visor contra a [`acende_texel`], que é a lei **`Forma`**. E a lei que
/// o binário corre sai de [`ph2d_form_donation::lei_da_luz::do_ambiente`], que **sem a variável de
/// ambiente devolve [`ph2d_form_donation::lei_da_luz::Lei::Tinta`]** — o passe do Painter, que o
/// próprio módulo descreve como *«difuso envolvido mais um especular lido de uma tabela, sem GGX e
/// sem conservação de energia»*.
///
/// ⛔⛔ *Uma paridade medida contra uma lei que o produto não corre não afirma nada sobre o
/// produto.* É a 2.ª vez que esta linha o paga com o mesmo oráculo: da 1.ª vez eu li `0,055` aqui e
/// escrevi *«são duas leis diferentes»* — e era, à letra, o defeito que o dono reportava.
///
/// Corre-se com o filtro `o_visor_contra_a_sprite_que_o_produto_assa`, com `--ignored --nocapture`.
#[test]
#[ignore = "precisa de adapter"]
fn o_visor_contra_a_sprite_que_o_produto_assa() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a medir");
        return;
    };
    let (mut renderer, camera, rig) = stage(&gpu);
    let barro = [
        (CLAY[0] * 255.0 + 0.5) as u8,
        (CLAY[1] * 255.0 + 0.5) as u8,
        (CLAY[2] * 255.0 + 0.5) as u8,
    ];
    // O visor COMO ELE SHIPA desde que o `DEFAULT_LIGHTING` é a lei que assa.
    let c = super::compare(&gpu, &mut renderer, &camera, &rig, barro, pbr_law_shade());

    let (mut texels, mut soma, mut pior) = (0u64, 0f64, 0f32);
    let (mut vivo, mut assado) = (0f64, 0f64);
    for b in 0..c.count.len() {
        let n = c.count[b];
        if n == 0 {
            continue;
        }
        texels += n;
        soma += c.mean_diff[b] * n as f64;
        vivo += c.mean_live[b] * n as f64;
        assado += c.mean_bake[b] * n as f64;
        pior = pior.max(c.max_diff[b]);
    }
    let n = texels as f64;
    println!(
        "\n=== o visor (a lei que assa) contra a sprite que o PRODUTO assa hoje ===\n  \
         lei do bake            : {:?}  (ENV `{}`)\n  \
         texels                 : {texels}\n  \
         desvio médio por canal : {:.6}   (meio código = {MEIO_CODIGO:.6})\n  \
         pior                   : {:.6}\n  \
         média VIVA             : {:.6}\n  \
         média ASSADA           : {:.6}",
        ph2d_form_donation::lei_da_luz::do_ambiente(),
        ph2d_form_donation::lei_da_luz::ENV,
        soma / n,
        pior,
        vivo / n,
        assado / n
    );
}

/// ⭐⭐⭐⭐ **A LEI QUE ASSA É A LEI QUE O VISOR MOSTRA — as duas metades, num sítio só.**
///
/// # Porque este gate tem de existir, e porque nenhum dos outros o substitui
///
/// O produto tem **dois** valores de fábrica e eles são de crates diferentes: o que o visor MOSTRA
/// ([`ph2d_mesh_render::DEFAULT_LIGHTING`]) e a lei que ASSA
/// ([`ph2d_form_donation::lei_da_luz::Lei`], escolhida pela porta que o `light` chama). Enquanto
/// discordarem, *o que se vê não é o que se assa* — e foi isso, três vezes, o report do dono.
///
/// ⛔⛔ **Os gates de paridade não o veem, e a razão é estrutural:** o [`o_visor_acende_com_a_lei_que_assa`]
/// crava o modo, o [`o_que_o_app_mostra_de_fabrica_e_a_lei_que_assa`] mede o barro contra a
/// [`acende_texel`] — e a `acende_texel` é a lei `Forma`. *Se o produto assar pela `Tinta`, os dois
/// ficam verdes sobre um produto partido*, que é exactamente o estado em que esta linha esteve.
///
/// ⚠️ **Ele é PURO e corre SEMPRE** (sem adapter, sem ambiente): os gates de GPU são `#[ignore]` e o
/// CI nunca os corre, logo a amarra entre as duas metades não podia viver num deles. E ele lê
/// [`ph2d_form_donation::lei_da_luz::Lei::do_texto`] com `None` — *o caminho do produto sem variável
/// nenhuma* — em vez de `do_ambiente`, porque **um gate que lê o ambiente mede a máquina**.
#[test]
fn a_lei_que_assa_e_a_lei_que_o_visor_mostra() {
    use ph2d_form_donation::lei_da_luz::Lei;

    assert_eq!(
        Lei::do_texto(None),
        Lei::Forma,
        "sem variável nenhuma o produto tem de assar pela lei da FORMA (o OpenPBR); \
         com a `Tinta` aqui, a sprite sai de um modelo sem GGX e sem conservação de energia \
         enquanto o visor mostra o OpenPBR — o report do dono, medido em 0,055 por canal"
    );
    assert_eq!(
        ph2d_mesh_render::DEFAULT_LIGHTING,
        ph2d_mesh_render::Lighting::Pbr,
        "e o visor tem de ABRIR na mesma lei — é o `Lighting::Pbr` que corre o OpenPBR"
    );

    // ⭐ **O CONTROLO, e sem ele isto seriam duas constantes a olhar uma para a outra:** as duas
    // metades têm de ser SEPARÁVEIS. Se a bissecção não existisse, o gate acima estaria a afirmar
    // uma coincidência em vez de uma escolha — e ninguém poderia medir as duas leis lado a lado.
    assert_eq!(
        Lei::do_texto(Some("0")),
        Lei::Tinta,
        "a lei da tinta tem de continuar ALCANÇÁVEL para bissecar"
    );
    assert_ne!(
        ph2d_mesh_render::DEFAULT_LIGHTING,
        ph2d_mesh_render::Lighting::Matcap(0),
        "controlo: o matcap é a luz do OLHO e não pode ser o que abre — ele não é assável"
    );
}

/// ⭐⭐⭐⭐ **O VISOR CONTRA OS BYTES DA SPRITE — a prova de ponta a ponta que faltava.**
///
/// # O que ela mede que nenhuma outra media
///
/// As outras comparam o visor contra a [`acende_texel`], que é a lei num PONTO. Esta compara-o
/// contra [`ph2d_form_donation::baked_form::pixels_pela_forma_na_cpu`] — **os pixels da sprite**,
/// pela régua que o produto declara para a lei que ele assa, com o mesmo material, as mesmas
/// lâmpadas, os mesmos planos e o mesmo olhar. É o que o dono compara com os olhos.
///
/// ⚠️ **A barra é de DOIS códigos de oito bits**, e ela é composta e não escolhida: a quantização
/// para `u8` custa meio código por construção, e o resíduo de `f32` entre o visor e a lei mede
/// `0,000797` (≈ `0,2` de código). *Uma barra de meio código seria mais apertada que a própria
/// quantização, e reprovaria um produto correcto.*
///
/// ⛔ **O CONTROLO é o visor SEM luz:** com `Lighting::Flat` a mesma comparação tem de reprovar por
/// uma ordem de grandeza. Sem ele, o dia em que esta sonda deixasse de ver a diferença entre duas
/// leis ela ficaria verde a afirmar nada — que é exactamente como esta linha chegou aqui.
#[test]
#[ignore = "precisa de adapter"]
fn o_visor_e_os_bytes_da_sprite_sao_a_mesma_imagem() {
    use ph2d_form_donation::baked_form::{BakedForm, pixels_pela_forma_na_cpu};

    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a afirmar");
        return;
    };
    // ⭐ A lei que esta sonda usa como oráculo TEM de ser a que o produto assa — senão ela volta a
    // medir um programa que ninguém corre.
    assert_eq!(
        ph2d_form_donation::lei_da_luz::Lei::do_texto(None),
        ph2d_form_donation::lei_da_luz::Lei::Forma,
        "o oráculo desta sonda só descreve o produto se a lei de fábrica for a `Forma`"
    );

    let (mut renderer, camera, rig) = stage(&gpu);
    let size = (SIDE, SIDE);
    let vista = vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA);
    let planes = renderer
        .form_plane(&gpu.device, &gpu.queue, &camera, size, vista, None)
        .expect("a malha esta la'");
    let n = (SIDE * SIDE) as usize;

    // O `base` é o barro do shader, para o albedo ser o MESMO dos dois lados.
    let mut base = vec![0u8; n * 4];
    for px in base.as_chunks_mut::<4>().0.iter_mut() {
        px.copy_from_slice(&[
            (CLAY[0] * 255.0 + 0.5) as u8,
            (CLAY[1] * 255.0 + 0.5) as u8,
            (CLAY[2] * 255.0 + 0.5) as u8,
            255,
        ]);
    }
    let bake = BakedForm {
        size,
        base,
        form: planes.normal.clone(),
        form_occ: planes.occlusion.clone(),
        texture_id: 0,
        rig,
        lit_with: None,
    };
    let sprite = pixels_pela_forma_na_cpu(&bake, &rig).expect("o rig default tem lampada acesa");

    let resolved = ph2d_light::resolve(&rig).expect("o rig default tem lampada acesa");
    let profundidade = depth_from_edge(&planes.normal, SIDE, SIDE);

    let mut medir = |modo| {
        let vivo = render_live(
            &gpu,
            &mut renderer,
            &camera,
            &resolved,
            size,
            ph2d_mesh_render::Shade {
                lighting: modo,
                ..vista
            },
        );
        let (mut pior, mut dentro) = (0f32, 0usize);
        for i in 0..n {
            if profundidade[i] == u32::MAX {
                continue;
            }
            dentro += 1;
            for k in 0..3 {
                let byte_do_visor = (vivo[i * 4 + k].clamp(0.0, 1.0) * 255.0 + 0.5).floor();
                pior = pior.max((byte_do_visor - f32::from(sprite[i * 4 + k])).abs());
            }
        }
        (pior, dentro)
    };

    let (pior, dentro) = medir(ph2d_mesh_render::DEFAULT_LIGHTING);
    assert!(
        dentro > 40_000,
        "controlo: a silhueta tem de encher o quadro ({dentro} texels)"
    );
    assert!(
        pior <= 2.0,
        "o visor e a sprite diferem {pior} códigos de oito bits — o dono lê isso como \
         «o bake não é idêntico ao que se vê em 3d»"
    );

    // ⭐ **O CONTROLO**: sem luz a mesma régua tem de reprovar por uma ordem de grandeza.
    let (cru, _) = medir(ph2d_mesh_render::Lighting::Flat);
    assert!(
        cru > pior * 10.0,
        "controlo: o visor SEM luz tinha de divergir por uma ordem de grandeza, e leu {cru} \
         contra {pior}"
    );
}
