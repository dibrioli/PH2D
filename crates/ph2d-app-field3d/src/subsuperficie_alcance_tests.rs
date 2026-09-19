//! ⭐⭐⭐ **POR ONDE UMA PEÇA ALCANÇA OUTRA** — o report do dono de 2026-09-18: *«a chapa ainda
//! influencia o SSS da esfera. Por que isso? Não faz sentido.»*
//!
//! # ⭐ Faz sentido, e são TRÊS caminhos — só um deles é o que ele vê
//!
//! | caminho | legítimo? | medido |
//! |---|---|---|
//! | **(A) a SOMBRA** que a chapa lança | ⭐ **sim** — e ela TEM de alcançar a subsuperfície | é o que se vê |
//! | **(B) o PASSO DA CURVATURA** sai da bola do DOCUMENTO | ⛔ não | `2,54×` no `ε`, e **`1` byte** na imagem |
//! | **(C) o RAIO DO BORRÃO** é o MÁXIMO da cena | ⛔ não | `18×` no raio — **dormente** enquanto a vizinha for opaca |
//!
//! ⭐⭐⭐ **Porque (A) é legítimo, e é a resposta ao «não faz sentido»:** o termo de subsuperfície é
//! *a luz que entrou PERTO e saiu aqui*. Se a chapa impede a luz de entrar perto, sai menos — logo
//! uma sombra sobre uma peça translúcida **tem** de a escurecer. ⚠️ É exactamente por isso que a
//! cura da §12 **borra** a visibilidade que aquela closure lê, em vez de a remover.
//!
//! # ⛔⛔ E os outros dois são vazamentos REAIS, medidos e NÃO curados
//!
//! Eles ficam com gate no tamanho MEDIDO de hoje — *um vazamento nomeado com número é uma dívida;
//! um sem número é uma nota que envelhece*. Curá-los muda toda imagem que já ship (o `ε`) ou custa
//! `n` passagens de borrão (o raio), e isso é **decisão do dono**.

use super::{Quadro, arranjo_do_dono, quadro, quebra_na_banda, so_a_bola};

/// ⭐⭐⭐ **(B) O PASSO DA CURVATURA DE UMA PEÇA SAI DA BOLA DO DOCUMENTO INTEIRO** — e não dela.
///
/// ⚠️⚠️ **A [`ph2d_field_render::curvatura::eps_para`] diz de si mesma, por escrito, que `escala` é
/// *«o tamanho da PEÇA (o raio da bola que a envolve)»*** — e quem a chama passa
/// `bounding_ball(doc)`, que é a **cena**. ⇒ pôr uma chapa ao lado da esfera muda o passo com que a
/// curvatura dela é medida, e a curvatura é o que o caminho maciço da subsuperfície lê.
///
/// ⭐ **Medido, e é pequeno:** o `ε` muda `2,54×` e a imagem move **`1` byte** de `255` sobre
/// `18 315` píxeis da esfera, **com a sombra desligada** — que é o que isola este caminho dos
/// outros dois. *Real, e não é o que o dono vê.*
///
/// ⛔ **Fica NÃO CURADO de propósito:** um `ε` por peça muda toda imagem que já ship e move a
/// paridade com o dispositivo, que deriva o dele da mesma bola. Este gate é o tecto da dívida —
/// ele reprova se ela CRESCER, que é quando ela passa a ser visível.
#[test]
fn a_chapa_move_a_curvatura_da_esfera_e_isso_nao_passa_de_um_byte() {
    let (com_placa, cam, onde, luz, chao) = arranjo_do_dono();
    let jade = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    // ⚠️ **A SOMBRA DESLIGADA nos dois** — sem isto o gate media (A), que é legítimo, e passaria a
    // dizer que o vazamento é enorme.
    let pixeis = |doc: &ph2d_field::FieldDoc| {
        quadro(&Quadro {
            doc,
            m: jade,
            cam: &cam,
            onde,
            luz,
            com_sombra: false,
            chao,
            sem_ceu: false,
            mole: true,
        })
    };
    let (gc, _, pc) = pixeis(&com_placa);
    let (gs, _, ps) = pixeis(&so_a_bola());
    let (mut pior, mut n) = (0u8, 0usize);
    for i in 0..gc.hit.len() {
        if !gc.hit[i] || !gs.hit[i] || gc.point[i][0] <= 0.1 {
            continue;
        }
        n += 1;
        for k in 0..3 {
            pior = pior.max(pc[i * 4 + k].abs_diff(ps[i * 4 + k]));
        }
    }
    assert!(
        n > 10_000,
        "só {n} píxeis da esfera — a fixtura perdeu o sujeito"
    );
    // (1) ⭐⭐⭐ **A ENTRADA do vazamento EXISTE, e é grande:** o `ε` da esfera muda `2,54×` só por
    // haver uma chapa ao lado. *Sem esta metade, um `eps_para` que ignorasse a escala passaria a
    // metade (2) trivialmente — e o gate ficaria a afirmar que não há vazamento nenhum.*
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let eps = |d: &ph2d_field::FieldDoc| {
        ph2d_field_eval::bounds::bounding_ball(d, &reg)
            .map_or(0.0, |b| ph2d_field_render::curvatura::eps_para(b.radius))
    };
    let (ec, es) = (eps(&com_placa), eps(&so_a_bola()));
    assert!(
        ec >= es * 2.0,
        "o passo da curvatura deixou de depender de quem está ao lado ({ec:.6} com a chapa contra \
         {es:.6} sem ela) — se alguém o passou a derivar da PEÇA, este gate sai e a dívida do \
         `eps_para` com ele"
    );
    // (2) ⚠️ **O tecto é o NÚMERO QUE A MEDIÇÃO DEU** (`1`), e não `2`: uma folga de `2×` sobre
    // uma medição é uma licença que ninguém pediu (`CLAUDE.md` §0.0).
    //
    // ⛔⛔ **E ele é um tecto SOBRE ESTA FIXTURA, não um limite do defeito** — medido: tornar o
    // `eps_para` quadrático na escala **não** move este byte, porque a curvatura de uma ESFERA é
    // robusta ao passo. *Numa peça com detalhe fino um `ε` `2,54×` mais grosso apagaria feição, e
    // aí o vazamento seria visível* — o que este número diz é que a cena do dono não o mostra, e
    // não que ele é pequeno em geral.
    assert!(
        pior <= 1,
        "a chapa move a esfera em {pior} bytes com a SOMBRA DESLIGADA (medido `1` em 18/09) — o \
         vazamento do passo da curvatura cresceu e passou a ser visível; ou alguém o curou, e então \
         esta linha sai"
    );
}

/// ⭐⭐⭐ **(C) O RAIO DO BORRÃO DA SOMBRA É O MÁXIMO DA CENA** — a chapa escolhe o raio da esfera.
///
/// ⛔⛔ **E a razão declarada no código estava ERRADA no ponto que decide:** ela diz que é *«uma
/// diferença que só se vê onde as duas peças se tocam»*. Não é — o raio é a **largura** com que
/// TODA borda de sombra da imagem é amaciada, e a sombra que a chapa lança **sobre a esfera** é
/// precisamente onde o raio da chapa apareceria. *Uma divergência declarada com o mecanismo errado
/// é pior que uma não declarada: ela convence quem a lê a não a medir.*
///
/// ⭐ **Medido:** uma esfera de raio `0,05` ao lado de uma chapa de `0,90` desenha a borda dela com
/// `0,90` — **`18×`**.
///
/// ⚠️⚠️ **E é por isso que o dono NÃO o vê na cena `=33`:** a lâmina dele está **opaca**, e um
/// material sem subsuperfície devolve `0` de espalhamento ⇒ não entra no máximo. *O vazamento é
/// real e está dormente, e a diferença entre as duas coisas é uma peça translúcida ao lado.*
#[test]
fn a_chapa_escolhe_o_raio_do_borrao_da_esfera_e_so_dorme_por_ela_ser_opaca() {
    let jade = |raio: f32| ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_radius: raio,
        ..ph2d_material::OpenPbr::default()
    };
    // ⚠️⚠️ **A vizinha opaca tem de ter um RAIO GORDO**, e a 1.ª redacção não tinha: com o raio a
    // zero, tirar a guarda do `reads_curvature` era **invisível** e a mutação SOBREVIVEU. *Uma
    // fixtura cujo valor «mau» é zero não testa o filtro que o deita fora.*
    //
    // ⭐ E é a situação REAL: a lâmina da `=33` tem um número no slider do raio e o **peso** a
    // zero. O que a mantém fora do máximo é o PESO, não o raio — e é isso que este gate afirma.
    let opaca = ph2d_material::OpenPbr {
        subsurface_weight: 0.0,
        subsurface_radius: 0.90,
        ..ph2d_material::OpenPbr::default()
    };
    let maior = |m: &[ph2d_material::Surface]| {
        crate::materials::maior_espalhamento(&ph2d_field_render::Surfaces {
            all: m,
            owners: None,
        })
    };
    let magra = [jade(0.05).prepare()];
    let par_translucido = [jade(0.05).prepare(), jade(0.90).prepare()];
    let par_opaco = [jade(0.05).prepare(), opaca.prepare()];

    let so = maior(&magra).expect("a esfera é translúcida")[0];
    let com = maior(&par_translucido).expect("as duas são translúcidas")[0];
    let dorme = maior(&par_opaco).expect("a esfera é translúcida")[0];

    // (1) ⛔ O VAZAMENTO EXISTE, e é grande: a vizinha gorda escolhe o raio da magra.
    assert!(
        com >= so * 10.0,
        "a vizinha translúcida deixou de mandar no raio da esfera ({so:.3} sozinha contra \
         {com:.3} acompanhada) — se alguém passou a fazer um borrão por material, esta linha sai e \
         a nota do `maior_espalhamento` com ela"
    );
    // (2) ⭐ E ELE DORME quando a vizinha é OPACA — é isto que explica por que a `=33` não o mostra.
    assert!(
        (dorme - so).abs() <= f32::EPSILON,
        "uma vizinha OPACA passou a entrar no máximo ({so:.3} contra {dorme:.3}) — então o \
         vazamento deixou de estar dormente na cena do dono, e ele vai vê-lo"
    );
    // (3) ⭐⭐⭐ **E UMA CENA SÓ DE OPACOS DEVOLVE `None`** — *esta* é a metade que o filtro de
    // `reads_curvature` compra, e a que faz um quadro sem subsuperfície não pagar NADA e sair byte
    // a byte o de sempre.
    //
    // ⛔⛔ Ela nasceu de uma mutação que SOBREVIVEU: apagar aquele filtro não move (1) nem (2),
    // porque a [`ph2d_material::Surface::scatter_distance`] tem a **mesma** guarda dentro dela e
    // devolve `0`. *O que o filtro decide sozinho não é o número — é o `Some` contra o `None`.*
    assert!(
        maior(&[opaca.prepare(), opaca.prepare()]).is_none(),
        "uma cena SÓ de materiais opacos devolveu um espalhamento — então o borrão passa a correr \
         sobre um quadro que não o pediu, e ele deixa de ser byte a byte o de sempre"
    );
}

/// ⏱️⭐⭐⭐ **SONDA — POR QUE A CHAPA INFLUENCIA O SSS DA ESFERA** (report do dono, 2026-09-18:
/// *«a chapa ainda influencia o SSS da esfera. Por que isso? Não faz sentido.»*).
///
/// Ela separa os **três** caminhos pelos quais uma peça alcança outra, e diz qual deles está a
/// falar — porque *dizer «é a sombra» sem medir é a resposta que a §11 já deu para OUTRA pergunta*.
#[test]
#[ignore = "sonda: imprime os tres caminhos, nao afirma"]
fn sonda_por_onde_a_chapa_alcanca_a_esfera() {
    let (com_placa, cam, onde, luz, chao) = arranjo_do_dono();
    let so_bola = so_a_bola();
    let jade = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    let reg = ph2d_field_eval::hybrid::Registry::new();

    println!("\n  ── POR ONDE A CHAPA ALCANCA A ESFERA ──");

    // ── (A) A SOMBRA ─────────────────────────────────────────────────────────────────────────
    println!("\n  (A) A SOMBRA que ela lanca — LEGITIMA: um objecto tapa a luz de outro.");

    // ── (B) O PASSO DA CURVATURA ─────────────────────────────────────────────────────────────
    // ⚠️ O `eps_para` diz de si que `escala` e' «o tamanho da PECA (o raio da bola que a envolve)»
    // — e quem o chama passa a bola do DOCUMENTO INTEIRO.
    let raio = |d: &ph2d_field::FieldDoc| {
        ph2d_field_eval::bounds::bounding_ball(d, &reg).map_or(0.0, |b| b.radius)
    };
    let (rc, rs) = (raio(&com_placa), raio(&so_bola));
    let (ec, es) = (
        ph2d_field_render::curvatura::eps_para(rc),
        ph2d_field_render::curvatura::eps_para(rs),
    );
    println!(
        "  (B) O PASSO DA CURVATURA sai da bola do DOCUMENTO:\n               com a chapa  raio {rc:.4} ⇒ eps {ec:.6}\n               so' a bola   raio {rs:.4} ⇒ eps {es:.6}   ({:.2}x)",
        ec / es.max(1e-9)
    );

    // ── (C) O RAIO DO BORRAO ─────────────────────────────────────────────────────────────────
    let magro = ph2d_material::OpenPbr {
        subsurface_radius: 0.05,
        ..jade
    };
    let gordo = ph2d_material::OpenPbr {
        subsurface_radius: 0.90,
        ..jade
    };
    let so = [magro.prepare()];
    let par = [magro.prepare(), gordo.prepare()];
    let ver = |m: &[ph2d_material::Surface]| {
        crate::materials::maior_espalhamento(&ph2d_field_render::Surfaces {
            all: m,
            owners: None,
        })
        .map_or(0.0, |e| e[0])
    };
    println!(
        "  (C) O RAIO DO BORRAO da sombra e' o MAXIMO da cena:\n               so' a esfera magra        ⇒ {:.4}\n               esfera magra + chapa gorda ⇒ {:.4}   ⛔ a chapa escolheu o raio da esfera",
        ver(&so),
        ver(&par)
    );

    // ── O QUE SOBRA COM A SOMBRA DESLIGADA ───────────────────────────────────────────────────
    // ⭐ Sem sombra nenhuma, tudo o que diferir e' (B) — e e' a medicao que decide.
    let pixeis = |doc: &ph2d_field::FieldDoc| {
        quadro(&Quadro {
            doc,
            m: jade,
            cam: &cam,
            onde,
            luz,
            com_sombra: false,
            chao,
            sem_ceu: false,
            mole: true,
        })
    };
    let (gc, _, pc) = pixeis(&com_placa);
    let (gs, _, ps) = pixeis(&so_bola);
    let (mut pior, mut n, mut curv) = (0i32, 0usize, 0.0f32);
    for i in 0..gc.hit.len() {
        // So' os pixeis da ESFERA, nos dois quadros.
        if !gc.hit[i] || !gs.hit[i] || gc.point[i][0] <= 0.1 {
            continue;
        }
        n += 1;
        curv = curv.max((gc.curvature[i] - gs.curvature[i]).abs());
        for k in 0..3 {
            pior = pior.max(i32::from(pc[i * 4 + k]).abs_diff(i32::from(ps[i * 4 + k])) as i32);
        }
    }
    println!(
        "\n  COM A SOMBRA DESLIGADA, sobre {n} pixeis da esfera:\n               pior diferenca de byte  {pior}\n               pior diferenca de curvatura  {curv:.4}"
    );
    println!(
        "\n    ⭐ Se o `pior byte` for > 0 com a sombra DESLIGADA, a chapa alcanca a esfera por\n          \x20     um caminho que NAO e' a sombra — e (B) e' o unico candidato que resta."
    );
}

/// ⏱️⭐⭐⭐ **SONDA — «SSRadius tira a linha dura»** (report do dono, 2026-09-18).
///
/// Ele mexeu no `Subsurface Radius` e a linha amaciou. Esta sonda varre o botão nos **dois**
/// caminhos e diz *por que mecanismo* — porque eles não são o mesmo:
///
/// | caminho | o que o raio muda |
/// |---|---|
/// | REFERÊNCIA | a **LARGURA DO BORRÃO** da visibilidade (a §12): o raio **é** a distância de espalhamento |
/// | DISPOSITIVO | só o **PERFIL** de Burley, que lava o terminador — não há borrão nenhum lá |
#[test]
#[ignore = "sonda: varre o botao nos dois caminhos, nao afirma"]
fn sonda_o_ssradius_contra_a_linha_dura() {
    let (doc, cam, onde, luz, chao) = arranjo_do_dono();
    println!("\n  ── A QUEBRA NA BANDA CONTRA O `Subsurface Radius` ──");
    println!("    raio  ·  DISPOSITIVO  ·  REFERENCIA");
    for raio in [0.05f32, 0.15, 0.30, 0.50, 0.76, 1.00] {
        let m = ph2d_material::OpenPbr {
            subsurface_weight: 1.0,
            geometry_thin_walled: false,
            subsurface_radius: raio,
            subsurface_color: [0.75, 0.35, 0.35],
            base_color: [0.75, 0.35, 0.35],
            ..ph2d_material::OpenPbr::default()
        };
        let (duro, _) = quebra_na_banda(&doc, m, &cam, onde, luz, chao, false);
        let (mole, _) = quebra_na_banda(&doc, m, &cam, onde, luz, chao, true);
        println!("    {raio:>4.2}  ·  {duro:>10.2}  ·  {mole:>9.2}");
    }
    println!(
        "\n    ⭐ Se a coluna do DISPOSITIVO tambem descer, o botao lava o terminador pelo PERFIL\n \
         \x20     e nao pelo borrao — e o dono viu a lei, nao um defeito."
    );
}
