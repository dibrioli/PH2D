//! ⭐⭐⭐ **A ARTE DE UM PINCEL PODE SER UM GRUPO** — os gates do assunto (plano 36).
//!
//! Irmão do [`super::corner_tests`], e o corte é por RESPONSABILIDADE: aquele mede o que a QUINA de
//! uma guia faz às cópias; este mede o que um motivo com **vários membros** faz — a disposição, as
//! tintas, e o orçamento de cópias que passa a ser repartido.

/// ⭐⭐⭐ **UM GRUPO PODE SER A ARTE DE UM PINCEL, e a disposição sobrevive** (report do Enio,
/// 2026-08-30 — ele pediu-o para a estampa, e o pincel é a mesma metade noutra tinta).
///
/// # As duas metades, e nenhuma se mede sozinha
///
/// **A disposição.** Os membros têm de ser colocados no referencial do **conjunto**. Com o
/// referencial próprio — que era o único que a `pattern_along` sabia calcular — cada membro
/// centra-se na guia e eles **empilham-se uns sobre os outros**, perdendo o desenho do artista.
/// ⇒ a régua é a **separação** entre os dois membros dentro de uma cópia: com o grupo `A` em cima
/// de `B`, as cópias de `A` não podem coincidir com as de `B`.
///
/// **As cores.** ⛔ Fundir os membros num `VecPath` (que tem `subpaths`) seria a saída barata e
/// **destruiria a feature**: um `VecPath` tem UM `fill`, então o triângulo azul e o círculo laranja
/// sairiam de uma cor só. ⇒ cada cópia carrega a tinta do SEU motivo.
///
/// ⚠️ **E o CONTROLO é a forma sozinha:** o mesmo pincel com um membro só tem de continuar a dar o
/// que dava — senão isto não é *"um grupo passa a poder"*, é *"o pincel mudou"*.
#[test]
fn a_group_can_be_a_brush_art_and_it_keeps_both_the_layout_and_the_colours() {
    // ⚠️⚠️ **A GUIA É RECTA de propósito, e a 1.ª redacção usou um RECTÂNGULO e mediu ZERO.**
    //
    // A separação dos membros é **perpendicular à guia**. Num rectângulo isso é o `y` do mundo nas
    // arestas horizontais e o `x` nas verticais — e uma régua que olhasse só o `y` lia `0` nas
    // verticais e concluía *"colapsaram"* sobre produto correcto. *Uma régua ancorada no MUNDO mede
    // o gesto, não a forma.* ⇒ guia recta (a perpendicular é constante) **e** distância em 2D.
    let guia = crate::line([0.0, 0.0], [40.0, 0.0]);
    let mut s = crate::StrokeSpec::new(crate::Rgba8::new(0, 0, 0, 255), 2.0);
    s.paint = crate::StrokePaint::Brush(Box::new(crate::BrushStroke {
        art: Some(crate::VecPathId::default()),
        spacing: 1.0,
        ..crate::BrushStroke::default()
    }));
    let membro = |cy: f64, cor: crate::Rgba8| {
        let mut p = crate::rectangle([-0.5, cy - 0.5], [0.5, cy + 0.5]);
        p.fill = Some(crate::Paint::solid(cor));
        p
    };
    let azul = crate::Rgba8::new(0, 0, 255, 255);
    let laranja = crate::Rgba8::new(255, 128, 0, 255);
    // O grupo: dois membros EMPILHADOS, com cores diferentes — a fixtura tem os dois fenómenos.
    let grupo = [membro(1.5, azul), membro(-1.5, laranja)];

    let copias = crate::brush_along_path(&guia, &grupo, &s);
    assert!(!copias.is_empty(), "o grupo nao produziu copia nenhuma");

    // ⭐ AS CORES: as duas sobrevivem, e nenhuma cópia é de uma terceira cor.
    let tem = |cor: crate::Rgba8| {
        copias
            .iter()
            .any(|c| c.fill.as_ref().map(crate::Paint::primary_color) == Some(cor))
    };
    assert!(
        tem(azul) && tem(laranja),
        "as tintas dos membros nao sobreviveram - fundir o grupo num `VecPath` da' UMA cor"
    );
    assert!(
        copias.iter().all(
            |c| matches!(c.fill.as_ref().map(crate::Paint::primary_color),
                              Some(x) if x == azul || x == laranja)
        ),
        "apareceu uma copia de uma TERCEIRA cor - a fusao inventou tinta"
    );

    // ⭐⭐ A DISPOSIÇÃO: dentro de cada cópia os dois membros ficam SEPARADOS. A régua é o centro em
    // y de cada peça: se os membros se centrassem cada um na guia, os dois conjuntos coincidiriam.
    let centro = |p: &crate::VecPath| {
        let ps: Vec<[f64; 2]> = p.verts_all().map(|v| v.anchor).collect();
        #[allow(clippy::cast_precision_loss)]
        let n = ps.len() as f64;
        [
            ps.iter().map(|q| q[0]).sum::<f64>() / n,
            ps.iter().map(|q| q[1]).sum::<f64>() / n,
        ]
    };
    let de = |cor: crate::Rgba8| -> Vec<[f64; 2]> {
        copias
            .iter()
            .filter(|c| c.fill.as_ref().map(crate::Paint::primary_color) == Some(cor))
            .map(centro)
            .collect()
    };
    let (a, b) = (de(azul), de(laranja));
    assert_eq!(
        a.len(),
        b.len(),
        "os membros nao produziram o mesmo numero de copias"
    );
    let separacao: f64 = a
        .iter()
        .zip(&b)
        .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
        .fold(f64::MAX, f64::min);
    assert!(
        separacao > 0.5,
        "os membros do grupo COLAPSARAM uns sobre os outros (separacao minima {separacao:.4}) - \
         cada um centrou-se na guia em vez de usar o referencial do CONJUNTO"
    );

    // ⚠️ CONTROLO: um membro SOZINHO continua a dar o que dava — o grupo acrescenta, não muda.
    let so_um = crate::brush_along_path(&guia, std::slice::from_ref(&grupo[0]), &s);
    assert!(!so_um.is_empty() && so_um.len() < copias.len());
}

/// ⛔⛔ **O TECTO DE CÓPIAS É DO CONJUNTO, e ele passou a ser POR MEMBRO** — achado da auditoria de
/// 2026-08-30, e um defeito que a própria wave do grupo introduziu.
///
/// # O número, e porque ele estava escrito e mesmo assim se perdeu
///
/// O doc do `MAX_COPIES` mede o orçamento em cópias **EMITIDAS**: *"4096 cópias custam 7,53 ms
/// contra o kill de 8 ms do re-cook por-frame"*. O emit passou a ser membro a membro ⇒ o total
/// virou `N × 4096`. Medido em release: `8` membros davam **32 768** cópias e **14,46 ms** (1,8× o
/// kill); `16` davam **65 536** e **30,17 ms** (3,8×).
///
/// ⚠️ *Um número que guarda um orçamento deixa de o guardar no instante em que alguém multiplica o
/// que ele conta — e nenhuma linha do doc dele muda.* Nenhum gate media o caso de grupo.
///
/// # A régua é a CONTAGEM, não o relógio
///
/// Contar cópias é exacto e imune à carga da máquina; cronometrar seria um gate de razão, que é a
/// família de flakes que o `CLAUDE.md` §5.0 manda parar de contar uma a uma. ⇒ o que se afirma é
/// que o total emitido **não cresce com o número de membros**.
#[test]
fn the_group_shares_one_copy_budget_it_does_not_multiply_it() {
    // Uma guia longa com o espaçamento no piso: o tecto morde de certeza.
    let guia = crate::line([0.0, 0.0], [40000.0, 0.0]);
    let mut s = crate::StrokeSpec::new(crate::Rgba8::new(0, 0, 0, 255), 1.0);
    s.paint = crate::StrokePaint::Brush(Box::new(crate::BrushStroke {
        art: Some(crate::VecPathId::default()),
        spacing: 0.01,
        ..crate::BrushStroke::default()
    }));
    let membro = |cy: f64| crate::rectangle([-0.5, cy - 0.5], [0.5, cy + 0.5]);
    let conta = |n: usize| -> usize {
        let grupo: Vec<_> = (0..n).map(|i| membro(i as f64 * 2.0)).collect();
        crate::brush_along_path(&guia, &grupo, &s).len()
    };
    let um = conta(1);
    // ⚠️ CONTROLO da fixtura: com um membro o tecto TEM de morder — senão este gate mede um caso
    // que nunca lá chega, e um total constante seria constante por outra razão.
    assert_eq!(
        um,
        crate::pattern_path::MAX_COPIES,
        "a fixtura nao chega ao tecto ({um} copias) - este gate nao estaria a medir o tecto"
    );
    for n in [2usize, 4, 8, 16] {
        let total = conta(n);
        assert!(
            total <= um,
            "com {n} membros o pincel emitiu {total} copias contra o tecto de {um} - o orcamento \
             foi MULTIPLICADO por N, e a {n} membros isso ja' foi medido acima do kill de 8 ms"
        );
    }
}

/// ⚠️⚠️ **O QUE A AUDITORIA MEDIU E NENHUM GATE GUARDAVA** (2026-08-30).
///
/// Os dois gates acima medem a disposição, as tintas e o orçamento. A auditoria mediu mais **cinco**
/// composições com grupo — tracejado, quinas, rotação, espelho e opacidade —, achou-as todas
/// CERTAS, e nenhuma tinha gate. *Uma medição que ninguém encoda é uma nota que envelhece: a
/// próxima wave parte-a e a suíte não diz nada.*
///
/// ⚠️ **A régua é o EMPARELHAMENTO**: seja qual for o corte que a guia impõe (um traço, uma quina),
/// os membros do grupo têm de o atravessar **juntos**. Um número desemparelhado significa que um
/// membro caiu num corte em que o outro entrou — o grupo deixou de ser uma unidade.
#[test]
fn a_group_survives_dashes_corners_rotation_flip_and_opacity_as_one_unit() {
    let membro = |cy: f64, a: u8| {
        let mut p = crate::rectangle([-0.5, cy - 0.5], [0.5, cy + 0.5]);
        p.fill = Some(crate::Paint::solid(crate::Rgba8::new(10, 20, 30, a)));
        p
    };
    // ⚠️ Alfas AUTORADOS diferentes: é o que prova que cada membro escala o SEU, e não um comum.
    let grupo = [membro(1.5, 200), membro(-1.5, 100)];
    let pincel = |dash: bool, rot: f64, flip: bool, alfa: u8| {
        let mut s = crate::StrokeSpec::new(crate::Rgba8::new(0, 0, 0, 255), 2.0);
        s.paint = crate::StrokePaint::Brush(Box::new(crate::BrushStroke {
            art: Some(crate::VecPathId::default()),
            spacing: 1.0,
            rotation_deg: rot,
            flip,
            fallback: crate::Rgba8::new(0, 0, 0, alfa),
            ..crate::BrushStroke::default()
        }));
        if dash {
            s.dash = Some((4.0, 2.0));
        }
        s
    };
    let por_membro = |copias: &[crate::VecPath]| -> [usize; 2] {
        let n = |a: u8| {
            copias
                .iter()
                .filter(|c| {
                    c.fill
                        .as_ref()
                        .map(crate::Paint::primary_color)
                        .is_some_and(|x| x.a == a)
                })
                .count()
        };
        [n(200), n(100)]
    };

    // ⛔ As guias trazem o fenómeno: a recta não corta, o quadrado tem QUINAS, o círculo é fechado.
    for (nome, guia) in [
        ("recta", crate::line([0.0, 0.0], [60.0, 0.0])),
        ("quadrado", crate::rectangle([0.0, 0.0], [20.0, 20.0])),
        ("circulo", crate::ellipse([0.0, 0.0], 10.0, 10.0)),
    ] {
        for (caso, dash, rot, flip) in [
            ("liso", false, 0.0, false),
            ("tracejado", true, 0.0, false),
            ("rodado", false, 30.0, false),
            ("espelhado", false, 0.0, true),
        ] {
            let mut g = guia.clone();
            let s = pincel(dash, rot, flip, 255);
            g.stroke = Some(s.clone());
            let c = crate::brush_along_path(&g, &grupo, &s);
            let [a, b] = por_membro(&c);
            assert!(a > 0, "{nome}/{caso}: o grupo nao emitiu nada");
            assert_eq!(
                a, b,
                "{nome}/{caso}: os membros DESEMPARELHARAM ({a} contra {b}) - um caiu num corte em \
                 que o outro entrou, e o grupo deixou de ser uma unidade"
            );
        }
    }

    // ⭐ **A OPACIDADE é POR MEMBRO**: cada um escala o alfa que o artista lhe deu, e não um comum.
    // Medido pela auditoria: com o pincel a `128`, os autorados `200` e `100` saem `[100, 50]`.
    let mut g = crate::line([0.0, 0.0], [60.0, 0.0]);
    let s = pincel(false, 0.0, false, 128);
    g.stroke = Some(s.clone());
    let c = crate::brush_along_path(&g, &grupo, &s);
    let alfas: Vec<u8> = c
        .iter()
        .filter_map(|p| p.fill.as_ref().map(|f| crate::Paint::primary_color(f).a))
        .collect();
    assert!(
        alfas.contains(&100) && alfas.contains(&50),
        "a opacidade do pincel nao escalou o alfa de CADA membro: {:?} - um alfa comum apagaria a \
         transparencia que o artista autorou por forma",
        {
            let mut u = alfas.clone();
            u.sort_unstable();
            u.dedup();
            u
        }
    );

    // ⛔ **E UM MEMBRO SEM GEOMETRIA não emite caminhos vazios** — 100% de desperdicio entregue ao
    // renderer, medido pela auditoria.
    let com_vazio = [
        grupo[0].clone(),
        crate::VecPath::default(),
        grupo[1].clone(),
    ];
    let c2 = crate::brush_along_path(&g, &com_vazio, &s);
    assert!(
        c2.iter().all(|p| p.verts_all().next().is_some()),
        "o grupo emitiu copias SEM UM VERTICE - o membro degenerado passou pelo emissor"
    );
    assert_eq!(
        por_membro(&c2),
        por_membro(&c),
        "o membro vazio mudou a contagem dos que TEM geometria"
    );
}
