//! ⏱️⭐⭐⭐ **A RÉGUA DO TERMINADOR** — o report do dono de 2026-09-18 (*«em `Thin Walled: Solid` não
//! há transição suave entre a área iluminada e a área sombreada da esfera, mas uma linha dura»*,
//! duas fotos da cena `=33` com a seta em cima do vinco).
//!
//! # ⚠️ A fixtura é a CENA QUE ELE CORREU, e a régua DECOMPÕE em vez de julgar
//!
//! A 1.ª redacção desta sonda montava uma bola sozinha na origem e binava a luminância por `N·L`.
//! Ela leu o perfil a atravessar `N·L = 0` **liso** e o pior salto a `0,78` — ⇒ *a fixtura não
//! continha o fenómeno*, que é a lei que esta casa já pagou cinco vezes.
//!
//! O que esta sonda faz: reconstrói o quadro do dono (a `=33`, o chão, o céu, a sombra, a
//! curvatura) e, sobre os píxeis da ESFERA, acha o pixel de maior **segunda diferença** da
//! luminância e imprime a vizinhança dele com **todos os canais lado a lado** — `N·L` · a
//! curvatura que o campo entrega · a visibilidade da lâmpada · a luminância.
//!
//! ⭐ *O canal cujo salto COINCIDE com o da luminância é a causa; os outros ficam ilibados com
//! número.*

use ph2d_field_render::{Orbit, Surfaces};

/// O material maciço com que o dono a viu: `Subsurface` no máximo, `Thin Walled: Solid`.
fn macico() -> ph2d_material::OpenPbr {
    ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
//  O GATE — três metades, e cada uma é um defeito MEDIDO
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **O CÉU APAGADO** — a condição em que a comparação com o oráculo tem UMA incógnita só.
///
/// Com o céu ligado há duas coisas a casar (o ambiente e a lâmpada) e a nossa `StudioSky` é um
/// gradiente que o oráculo não reproduz. Com ele apagado sobra **só a lâmpada**, e as duas leis de
/// queda são a mesma (`1/r²`) ⇒ **um único factor de escala** liga os dois lados, e tudo o que
/// sobrar depois de o ajustar é a LEI. *Uma comparação com duas incógnitas livres não afirma nada.*
struct CeuPreto;

impl ph2d_material::Environment for CeuPreto {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        [0.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        [0.0; 3]
    }
}

struct Quadro<'a> {
    doc: &'a ph2d_field::FieldDoc,
    m: ph2d_material::OpenPbr,
    cam: &'a Orbit,
    onde: [f32; 3],
    luz: ph2d_field_ecs::FieldLight,
    com_sombra: bool,
    chao: Option<ph2d_field_render::Ground>,
    /// ⭐ Apaga o céu — a condição da comparação com o oráculo. Ver [`CeuPreto`].
    sem_ceu: bool,
    /// ⭐⭐⭐ **A sombra tem a borda MOLE?** `false` desenha o quadro como o DISPOSITIVO o desenha.
    ///
    /// # ⛔⛔ Ele existe porque as duas metades do produto não desenham o mesmo
    ///
    /// A cura da §12 (*a visibilidade que uma closure translúcida lê é a média da vizinhança*) foi
    /// assada no traçado de **CPU** e o dispositivo **ainda não tem o gémeo** — ele calcula a
    /// visibilidade dentro da pintura, e dá-la mole pede a passagem que a escreve. O §12 declarou-o
    /// por escrito e **nada media a diferença**: as paridades CPU↔dispositivo ficam verdes porque
    /// nenhuma delas assa este canal.
    ///
    /// ⇒ *uma diferença declarada e não medida é uma nota que envelhece* — com este campo ela é um
    /// número, e o gate que o lê reprova no dia em que o gémeo chegar (que é quando ele deve
    /// reprovar: para alguém apagar a nota).
    ///
    /// ⚠️ **`false` NÃO é «sem sombra»** — é a sombra com a borda DURA, que é o que o dispositivo
    /// entrega hoje e o que o dono fotografou em 18/09.
    ///
    /// # ⛔⛔⛔ Ele é um SUCEDÂNEO, e até 2026-09-19 nada o tinha conferido
    ///
    /// Esta linha dizia *«desenha o quadro como o DISPOSITIVO o desenha»* e **pinta na CPU**: toda
    /// coluna deste módulo rotulada «dispositivo» era esta, e a placa **nunca foi corrida**.
    /// *Uma sonda que mede um sucedâneo mede outro programa* — e um sucedâneo por conferir é uma
    /// afirmação sobre um programa que ninguém observou.
    ///
    /// ⭐ **Conferido em 19/09 contra a placa a sério** ([`dispositivo`], RTX 5060 Ti), na
    /// cena `=33` com o jade do report:
    ///
    /// | | dispositivo REAL | este sucedâneo |
    /// |---|---:|---:|
    /// | quebra na banda | **`9,20`** | `9,21` |
    /// | contraste | `61,4` | `61,1` |
    ///
    /// ⇒ ele descreve bem o que a placa desenha **nesta cena**. ⚠️ O que ele **não** descreve é a
    /// razão: o caminho do dono nem chega ao borrão (ver o cabeçalho de [`dispositivo`]),
    /// e é por isso que a concordância deste número não autoriza a próxima afirmação sobre a placa
    /// a ser feita sem a correr.
    mole: bool,
}

/// A vista é a mesma nos dois gates — a cena é que muda.
const W: u32 = 320;
const H: u32 = 240;

/// O quadro do dono em CPU: devolve `(gbuffer, sombras, bytes)`.
fn quadro(
    q: &Quadro<'_>,
) -> (
    ph2d_field_render::Gbuffer,
    ph2d_field_render::Shadows,
    Vec<u8>,
) {
    let (doc, m, cam, onde, luz, com_sombra, chao) =
        (q.doc, q.m, q.cam, q.onde, q.luz, q.com_sombra, q.chao);
    let (w, h) = (W, H);
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let mut g = ph2d_field_render::trace(doc, &reg, cam, w, h);
    let mats = [m.prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    if surfaces
        .all
        .iter()
        .any(ph2d_material::Surface::reads_curvature)
        && let Some(b) = ph2d_field_eval::bounds::bounding_ball(doc, &reg)
    {
        let mut eval = ph2d_field_eval::hybrid::Hybrid::new(doc, &reg);
        g.curvature = ph2d_field_render::curvatura::do_gbuffer(
            &mut eval,
            &g,
            ph2d_field_render::curvatura::eps_para(b.radius),
        );
    }
    let uma = [onde];
    let sh = ph2d_field_render::shadow_pass_on(
        doc,
        &reg,
        cam,
        &g,
        if com_sombra { &uma[..] } else { &[][..] },
        chao,
    );
    // ⭐⭐⭐ **A BORDA MOLE, pela MESMA porta que o app usa** — ⚠️ a 1.ª redacção desta sonda
    // derivava a distância de espalhamento aqui, o que a fazia a segunda resposta à mesma pergunta.
    let mut sh = sh;
    if q.mole {
        // ⚠️ **As lâmpadas que EXISTEM**, e não `0..1`: com a sombra desligada não há canal
        // nenhum, e um `0..1` pedia o canal de uma lâmpada que o passe não escreveu.
        let canais = (0..sh.lamps())
            .map(|l| {
                ph2d_field_render::sss_shadow::blur_por_material(
                    &g,
                    sh.lamp_channel(l),
                    &surfaces,
                    cam,
                    h,
                )
            })
            .collect();
        sh.set_soft(canais);
    }
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    let preto = CeuPreto;
    let estudio = crate::render_light::StudioSky;
    let ceu: &(dyn ph2d_material::Environment + Sync) = if q.sem_ceu { &preto } else { &estudio };
    let px = ph2d_field_render::shade_render(
        &g,
        cam,
        &surfaces,
        &ph2d_field_render::Lighting {
            lamps: &sem_ecra,
            points: &[ph2d_field_render::PointLamp {
                world: onde,
                radiance_at_one: crate::lights::radiance_at_one(luz),
            }],
            sky: ceu,
            shadows: Some(&sh),
        },
        &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
        [0, 0, 0, 0],
    );
    (g, sh, px)
}

/// ⭐ **O ARRANJO DO DONO** — a cena `=33` com a câmera, a luz e o chão de abertura.
///
/// ⚠️ Ele é uma porta porque o gate da sombra e a sonda dos dois caminhos têm de medir **a mesma
/// cena**: *duas montagens do mesmo arranjo divergem no dia em que uma delas ganhar uma linha.*
fn arranjo_do_dono() -> (
    ph2d_field::FieldDoc,
    Orbit,
    [f32; 3],
    ph2d_field_ecs::FieldLight,
    Option<ph2d_field_render::Ground>,
) {
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena do dono");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = Orbit::default();
    let (onde, luz) = crate::lights::opening_light(&cam);
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    (doc, cam, onde, luz, chao)
}

/// ⭐⭐⭐ **A BOLA SOZINHA** — o experimento que o DONO fez, e que decide o que a linha É.
///
/// *«Descobri que a presença da placa faz a linha dura aparecer»* (18/09). Tirada a lâmina da cena,
/// no **mesmo** enquadramento e com a **mesma** luz, a bola sai lisa ⇒ a linha é a borda da SOMBRA
/// que a placa lança, e não o terminador.
///
/// ⚠️ Ela é uma PORTA e não uma variável de ambiente numa sonda: *o experimento que decidiu o
/// diagnóstico é o que um gate tem de poder repetir.*
fn so_a_bola() -> ph2d_field::FieldDoc {
    ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.42 },
            ph2d_field::Xform {
                translation: [0.55, 0.0, 0.0],
                ..ph2d_field::Xform::IDENTITY
            },
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a bola sozinha")
}

/// Os bytes do quadro do dono — a metade que o gate compara **ao bit**.
fn bytes_do_quadro(
    doc: &ph2d_field::FieldDoc,
    m: ph2d_material::OpenPbr,
    cam: &Orbit,
    onde: [f32; 3],
    luz: ph2d_field_ecs::FieldLight,
    chao: Option<ph2d_field_render::Ground>,
    mole: bool,
) -> Vec<u8> {
    quadro(&Quadro {
        doc,
        m,
        cam,
        onde,
        luz,
        com_sombra: true,
        chao,
        sem_ceu: false,
        mole,
    })
    .2
}

/// ⭐⭐⭐ **A QUEBRA NA BANDA onde a sombra da placa corta a bola, e o CONTRASTE que lá vive.**
///
/// Devolve `(p99 da segunda diferença da luminância, claro − escuro)`, sobre os píxeis da ESFERA
/// cujo `N·L` está a menos de `0,15` de zero — que é a banda do terminador, e não a bola inteira.
///
/// ⚠️ **A segunda diferença e não a primeira:** o terminador tem um gradiente legítimo, e é a
/// CURVATURA dele que diz se há um degrau. *Uma primeira diferença acusaria toda a banda.*
///
/// ⭐ `mole = false` desenha o quadro **como o dispositivo o desenha** — ver [`Quadro::mole`].
fn quebra_na_banda(
    doc: &ph2d_field::FieldDoc,
    m: ph2d_material::OpenPbr,
    cam: &Orbit,
    onde: [f32; 3],
    luz: ph2d_field_ecs::FieldLight,
    chao: Option<ph2d_field_render::Ground>,
    mole: bool,
) -> (f32, f32) {
    let (g, _, px) = quadro(&Quadro {
        doc,
        m,
        cam,
        onde,
        luz,
        com_sombra: true,
        chao,
        sem_ceu: false,
        mole,
    });
    regua_da_banda(&g, &px, onde, cam)
}

/// ⭐⭐⭐ **A RÉGUA, sobre um quadro JÁ PINTADO** — devolve `(p99 da segunda diferença da luminância
/// na banda do terminador, claro − escuro)`.
///
/// # ⛔⛔ Ela é uma PORTA desde 2026-09-19, e a razão é um defeito MEDIDO
///
/// Até aqui a régua vivia dentro da [`quebra_na_banda`], que pinta **na CPU** — logo toda coluna
/// deste módulo rotulada *«DISPOSITIVO»* era a CPU com o canal mole desligado, um **SUCEDÂNEO**, e
/// o dispositivo nunca foi corrido. *Uma sonda que mede um sucedâneo mede outro programa.*
///
/// ⭐ Com a régua aqui, o leitor do dispositivo ([`dispositivo`]) mede a imagem que o
/// produto de facto entrega **com a mesma régua** — e duas colunas medidas por duas funções
/// diferentes não são uma comparação.
fn regua_da_banda(
    g: &ph2d_field_render::Gbuffer,
    px: &[u8],
    onde: [f32; 3],
    cam: &Orbit,
) -> (f32, f32) {
    let (wu, hu) = (W as usize, H as usize);
    let (right, up, fwd) = cam.basis();
    let da_bola = |i: usize| g.hit[i] && g.point[i][0] > 0.1;
    let lum = |i: usize| {
        let b = i * 4;
        0.2126 * f32::from(px[b]) + 0.7152 * f32::from(px[b + 1]) + 0.0722 * f32::from(px[b + 2])
    };
    let ndl = |i: usize| {
        let p = g.point[i];
        let d = [onde[0] - p[0], onde[1] - p[1], onde[2] - p[2]];
        let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let n = [0, 1, 2]
            .map(|k| g.normal[i][0] * right[k] + g.normal[i][1] * up[k] + g.normal[i][2] * fwd[k]);
        (n[0] * d[0] + n[1] * d[1] + n[2] * d[2]) / r.max(1e-9)
    };
    let mut saltos: Vec<f32> = Vec::new();
    let (mut claro, mut escuro) = (0.0f32, f32::MAX);
    for y in 0..hu {
        for x in 1..wu - 1 {
            let (a, b, c) = (y * wu + x - 1, y * wu + x, y * wu + x + 1);
            if !da_bola(a) || !da_bola(b) || !da_bola(c) || ndl(b).abs() > 0.15 {
                continue;
            }
            saltos.push((lum(a) - 2.0 * lum(b) + lum(c)).abs());
            claro = claro.max(lum(b));
            escuro = escuro.min(lum(b));
        }
    }
    assert!(saltos.len() > 2_000, "só {} px na banda", saltos.len());
    saltos.sort_by(f32::total_cmp);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let k = ((saltos.len() as f32 - 1.0) * 0.99).round() as usize;
    (saltos[k], claro - escuro)
}

/// ⛔⛔⛔ **RECUSA MEDIDA — «marchar o raio de costas» foi construída, fotografada e REVERTIDA.**
///
/// O passe de sombra escreve `vis = 1,0` para todo ponto de costas para a luz, o que **trunca** no
/// terminador a sombra que um vizinho projecta. Isso é um defeito real, e o comentário desse filtro
/// previa-o por escrito. A cura — lançar o raio na mesma e só contar o que estiver depois de ele
/// SAIR do próprio corpo, com o `t` do estimador de penumbra recontado a partir da saída — foi
/// construída inteira, com o gémeo em WGSL, as 6 paridades verdes e 4 mutações a sangrar.
///
/// ⛔ **E a FOTO reprovou-a.** Na cena `=33`, com o enquadramento do dono:
///
/// | | o que se vê |
/// |---|---|
/// | antes | a borda da sombra é **limpa**, embora dura |
/// | com a cura | a borda alarga **e ganha um FIO escuro SERRILHADO** por cima |
///
/// ⚠️ O serrilhado é a assinatura da causa: **um `if` por pixel** (`N·L <= 0`) escolhia entre duas
/// maneiras de calcular a mesma grandeza, e *a fronteira entre elas desenha-se*. Perto do
/// terminador o raio de costas rasa a própria peça durante `~√(2R·ε)` antes de sair, logo o `t`
/// dele reconta tarde e a penumbra sai mais dura que a do vizinho de frente.
///
/// ⛔ **E apagar o ramo (um só caminho, todo raio a partir do ponto) é PIOR:** sem o ergue pela
/// normal volta a **acne** — a foto mostra riscos claros ao longo do terminador, e a banda passa de
/// `p99 3,71` para `13,06`.
///
/// ⇒ *a truncagem é um defeito INVISÍVEL nesta cena e o fio é VISÍVEL*, logo shipa-se a truncagem.
/// A cura de fundo é outra e está nomeada no [`10` §11.6]: **a visibilidade que um termo
/// TRANSMISSIVO lê tem de ser borrada pela distância de espalhamento** — é isso que faz a sombra
/// num jade ter a borda mole, e nenhuma das duas referências o escreve.
///
/// Os dois gates abaixo ficam: eles são as propriedades que aquela cura teria partido, e é por eles
/// que uma segunda tentativa sabe onde bate.
/// ⭐⭐⭐ **A FOLHA COM A LUZ ATRÁS NÃO SE APAGA** — a metade que a cura óbvia partia.
///
/// Marchar o raio de costas como os outros **cura a esfera e mata a folha**: ele atravessa a
/// própria lâmina, lê-a como obstáculo, e a luminância média cai de `83,7` para **`73,8`** com a
/// `vis` mínima em `0,000` (medido). ⇒ o raio só conta o que estiver **depois de ele sair**.
///
/// ⚠️ A barra é `1 %` da média porque a resposta certa é a IGUALDADE: não há nada entre a folha e
/// a luz senão ela própria.
#[test]
fn a_folha_com_a_luz_atras_nao_se_apaga() {
    let cam = Orbit::default();
    let (_, luz) = crate::lights::opening_light(&cam);
    let (_, _, fwd) = cam.basis();
    let o = crate::lights::opening_place(&cam);
    let k = o[0] * fwd[0] + o[1] * fwd[1] + o[2] * fwd[2];
    let atras = [0, 1, 2].map(|i| o[i] - 2.0 * k * fwd[i]);
    let folha = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Box {
                half: [0.45, 0.42, 0.015],
                round: 0.012,
                chamfer: 0.0,
            },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a folha");
    let fina = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: true,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    let media = |com_sombra: bool| {
        let (g, _, px) = quadro(&Quadro {
            doc: &folha,
            m: fina,
            cam: &cam,
            onde: atras,
            luz,
            com_sombra,
            chao: None,
            sem_ceu: false,
            mole: true,
        });
        let (mut soma, mut n) = (0.0f64, 0usize);
        for i in 0..g.hit.len() {
            if !g.hit[i] {
                continue;
            }
            let b = i * 4;
            soma += f64::from(
                0.2126 * f32::from(px[b])
                    + 0.7152 * f32::from(px[b + 1])
                    + 0.0722 * f32::from(px[b + 2]),
            );
            n += 1;
        }
        assert!(n > 5_000, "só {n} pixels de folha — a fixtura não a mostra");
        #[allow(clippy::cast_precision_loss)]
        let m = soma / n as f64;
        m
    };
    let (sem, com) = (media(false), media(true));
    assert!(
        (com - sem).abs() <= sem * 0.01,
        "a folha com a luz ATRÁS lê {com:.1} com sombra contra {sem:.1} sem — ela está a ler o \
         próprio corpo como obstáculo, e é isso que apaga a luz que a atravessa"
    );
}

/// ⭐⭐⭐ **UM CORPO CONVEXO NÃO SE TAPA A SI PRÓPRIO** — o piso que as outras duas não vêem.
///
/// ⚠️ Sem esta metade, uma «cura» que pusesse toda a metade escura de toda peça em sombra passaria
/// nas outras duas. Medido: `0` de `12 924` pixels com `vis < 0,99` numa esfera sozinha, antes e
/// depois — *a cura não pode ganhar isto escurecendo o mundo.*
#[test]
fn um_corpo_convexo_nao_se_tapa_a_si_proprio() {
    let cam = Orbit::default();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (onde, _) = crate::lights::opening_light(&cam);
    let bola = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.42 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a bola sozinha");
    let g = ph2d_field_render::trace(&bola, &reg, &cam, 320, 240);
    let sh = ph2d_field_render::shadow_pass_on(&bola, &reg, &cam, &g, &[onde], None);
    let (mut tapados, mut total) = (0usize, 0usize);
    for i in 0..g.hit.len() {
        if !g.hit[i] {
            continue;
        }
        total += 1;
        if sh.at(0, i) < 0.99 {
            tapados += 1;
        }
    }
    assert!(
        total > 5_000,
        "só {total} pixels de peça — a fixtura não a mostra"
    );
    assert_eq!(
        tapados, 0,
        "{tapados} de {total} pixels de uma esfera SOZINHA vêm sombreados — um convexo não se \
         pode tapar a si próprio, logo isto é o corpo a ser lido como obstáculo de si mesmo"
    );
}

/// ⭐⭐⭐ **A BORDA DA SOMBRA NUM JADE É MOLE, E A DO OPACO AO LADO CONTINUA DURA.**
///
/// # O report de 2026-09-18 e o experimento que o diagnosticou
///
/// O dono apontou uma linha dura na esfera com `Thin Walled: Solid`. O que a decidiu foi tirar a
/// LÂMINA da cena: **sem ela a bola sai lisa** ⇒ a linha é a borda da SOMBRA que a placa lança, e
/// ela é dura porque a luz é um **ponto**. ⛔ Certo como geometria, errado como produto: num jade a
/// luz entra fora da sombra e espalha-se por baixo da superfície **para dentro** dela.
///
/// # As três metades, e porque nenhuma chega sozinha
///
/// 1. **o jade amacia** — a quebra na banda do terminador cai;
/// 2. **o OPACO não se mexe, ao bit** — sem isto, borrar a visibilidade de toda a gente passaria
///    aqui e apagaria a sombra do app inteiro;
/// 3. **o jade continua a TER sombra** — sem isto, uma «cura» que pusesse `vis = 1` em todo o lado
///    lia a banda lisíssima e passava.
#[test]
fn a_borda_da_sombra_num_jade_e_mole_e_a_do_opaco_continua_dura() {
    let (doc, cam, onde, luz, chao) = arranjo_do_dono();
    let base = ph2d_material::OpenPbr {
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    let jade = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        ..base
    };
    // ⚠️ **A régua é a PORTA**, com dois chamadores (este gate e a sonda dos dois caminhos) —
    // *duas cópias de uma régua medem dois programas*.
    let medida = |m: ph2d_material::OpenPbr| {
        let (quebra, contraste) = quebra_na_banda(&doc, m, &cam, onde, luz, chao, true);
        (
            quebra,
            contraste,
            bytes_do_quadro(&doc, m, &cam, onde, luz, chao, true),
        )
    };

    let (dura_op, _, bytes_op) = medida(base);
    let (mole_jade, contraste, _) = medida(jade);

    // (1) o jade amacia — medido `9,21` antes e `1,36` depois; a barra sai do vale.
    assert!(
        mole_jade <= 4.0,
        "a quebra na banda do jade lê p99 {mole_jade:.2} contra a barra 4,0 — a borda da sombra \
         voltou a ser dura (o opaco, que não tem espalhamento, lê {dura_op:.2})"
    );
    // (2) ⚠️ o OPACO não pode ter mudado UM BYTE — ele não tem espalhamento, logo não assa canal.
    let sem_sss = ph2d_material::OpenPbr {
        subsurface_weight: 0.0,
        ..base
    };
    let (_, _, controlo) = medida(sem_sss);
    assert_eq!(
        bytes_op, controlo,
        "o material sem subsuperfície mudou de bytes — a borda mole está a alcançar quem não a pediu"
    );
    // (3) e o jade CONTINUA a ter sombra: sem isto, `vis = 1` em todo o lado passaria em (1).
    assert!(
        contraste >= 8.0,
        "o contraste através da banda do jade é {contraste:.1} — a sombra foi apagada em vez de \
         amaciada, e a metade (1) não distingue as duas"
    );
}

/// ⭐⭐⭐ **A RÉGUA LÊ ~ZERO NUM GRADIENTE LISO — e é isso que a prende à SEGUNDA diferença.**
///
/// # ⛔⛔ Ela nasceu de uma mutação que SOBREVIVEU a DOIS gates
///
/// Trocar a segunda diferença (`a − 2b + c`) pela primeira (`a − c`) deixava verdes tanto o gate da
/// borda mole (barra ABSOLUTA) como o dos dois caminhos (uma RAZÃO) — o segundo por construção, já
/// que ele compara **dois renders com a mesma régua** e por isso é invariante ao operador dela.
///
/// ⚠️⚠️ **E a troca não é inofensiva:** um terminador tem um gradiente LEGÍTIMO, e uma primeira
/// diferença acusa-o inteiro. Os números desta página (`9,21` · `1,00` · a barra `4,0`) passariam a
/// medir a inclinação da banda em vez do DEGRAU nela, e ninguém saberia.
///
/// ⭐ **O discriminador é o experimento do DONO**, virado do avesso: na [`so_a_bola`] — sem a placa,
/// logo sem sombra a cortar — a banda é um gradiente puro. Ali a segunda diferença lê **~0** e a
/// primeira lê a inclinação toda. *Uma régua de degrau que acusa uma rampa não é uma régua de
/// degrau.*
///
/// ⚠️ **A barra é a RAZÃO contra a cena com placa**, e não um número escolhido: o que se afirma é
/// que a régua **separa** as duas situações, não que ela devolve um valor.
#[test]
fn a_regua_da_banda_le_quase_zero_num_gradiente_sem_degrau() {
    let (com_placa, cam, onde, luz, chao) = arranjo_do_dono();
    let jade = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    // ⚠️ O caminho DURO nos dois lados: é ele que tem o degrau, e é sobre ele que a pergunta é feita.
    let (com, _) = quebra_na_banda(&com_placa, jade, &cam, onde, luz, chao, false);
    let (sem, _) = quebra_na_banda(&so_a_bola(), jade, &cam, onde, luz, chao, false);
    assert!(
        sem * 4.0 <= com,
        "a régua lê {sem:.2} na bola SOZINHA (um gradiente sem degrau nenhum) contra {com:.2} na          cena com a placa — ela deixou de separar um DEGRAU de uma RAMPA, e é o que acontece se a          segunda diferença virar primeira"
    );
    // ⭐ O CONTROLO: a cena com placa tem mesmo o degrau, senão o teste acima passa por vácuo.
    assert!(
        com >= 4.0,
        "a cena com a placa lê {com:.2} — o degrau que o dono fotografou desapareceu da fixtura, e          este gate deixou de ter sujeito"
    );
}

/// ⭐⭐⭐ **A BANCADA DO ORÁCULO EXTERNO** — irmão de ASSUNTO, cortado daqui em 2026-09-18 pelo tecto
/// de LOC (`1455` contra `700`). ⛔ *Split por responsabilidade, nunca uma entrada no
/// `FILE_OVERAGE_OK`.*
///
/// ⚠️ **Ele é um FILHO e não um irmão de `lib.rs`**, e isso é o que mantém a régua UMA: ele vê o
/// [`quadro`] deste módulo e os dois filhos abaixo veem os auxiliares dele (`le_pfm`, `razao_rb`,
/// `casa_a_populacao`), sem que nada abra visibilidade para a crate. *Três colunas medidas por três
/// funções diferentes não são uma comparação.*
/// ⏱️ **AS SONDAS** — as quatro réguas que decompõem o report, todas `#[ignore]`. Irmão de
/// assunto, cortado daqui pelo tecto de LOC: *um ficheiro onde uma sonda e uma lei se leem
/// iguais é onde uma lei passa a `#[ignore]` sem ninguém dar por isso.*
/// ⭐⭐⭐ **POR ONDE UMA PEÇA ALCANÇA OUTRA** — os três caminhos pelos quais a chapa chega à
/// esfera, dois deles vazamentos medidos e não curados (report do dono, 18/09).
#[path = "subsuperficie_alcance_tests.rs"]
mod alcance;

/// ⭐⭐⭐ **O DISPOSITIVO, CORRIDO** — a metade que faltava a este módulo inteiro.
#[path = "subsuperficie_dispositivo_tests.rs"]
mod dispositivo;

#[path = "subsuperficie_sondas_tests.rs"]
mod sondas;

#[path = "subsuperficie_oraculo_tests.rs"]
mod oraculo;

/// ⭐ A UNREAL como TERCEIRO contendor — ela mede-se com a régua **deste** módulo, de propósito.
///
/// ⚠️ É um módulo-filho por `#[path]` e não uma crate nem um irmão de `lib.rs`: assim ele vê o
/// `quadro` daqui e o `le_pfm`, a `razao_rb` e a `casa_a_populacao` do [`oraculo`] **sem que nada
/// abra visibilidade** — e, sobretudo, *sem uma segunda cópia da régua*. Três colunas medidas por
/// três funções diferentes não são uma comparação.
#[path = "unreal_contendor_tests.rs"]
mod unreal_contendor;

/// ⭐ **A LEI DA COR** — ordem do dono depois do veredito a quatro colunas: *«atacamos agora a cor»*.
///
/// Mesmo desenho do irmão: módulo-filho por `#[path]`, para usar a régua deste módulo sem abrir
/// visibilidade e sem uma segunda cópia dela.
#[path = "cor_da_profundidade_tests.rs"]
mod cor_da_profundidade;
