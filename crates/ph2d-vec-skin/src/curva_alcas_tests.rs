//! ⭐⭐⭐ **OS GATES DA CONCILIAÇÃO DAS ALÇAS** — o report do dono de 2026-09-19
//! (*«muitas irregularidades … certamente um mau tratamento das alças dos handles»*).
//!
//! ⛔ Ele vive ao lado do [`super::tests`] e não dentro dele por **tecto de LOC**: o irmão estava a
//! `882` linhas. *O ficheiro é uma unidade de manutenção; a lei é a mesma.*

use super::*;
use ph2d_skeleton::{SkinBone, Xform};
use ph2d_vec_scene::{ShapeKind, VecPath, cook};

use super::tests::{forma, pele};

/// O ângulo com que a tangente VIRA ao atravessar cada nó, em graus.
///
/// ⚠️⚠️ **Ele NÃO é a régua sozinho, e a 1.ª redacção deste ficheiro caiu nisso:** um rectângulo tem
/// quatro QUINAS autoradas, e ali este número vale `90°` em repouso. *Uma quina que o artista
/// desenhou não é uma quina que o ajuste cravou.* A régua é a [`mudanca_da_tangente`] — a diferença
/// contra a lei INGÉNUA, que preserva o que a fonte tinha por construção.
///
/// ⚠️ **Uma alça DEGENERADA devolve `None`** — ela não tem direcção, e o nó onde ela chega é um
/// CANTO. *Ali não há continuidade para conciliar*, e é por isso que a fixtura deste gate é a
/// ELIPSE: num rectângulo as quatro alças estão em cima das âncoras e esta régua lê `None` nas
/// quatro, o que a deixaria **verde por vácuo** (medido).
fn viragens(p: &VecPath) -> Vec<Option<f64>> {
    let c = p.cooked();
    let mut out = Vec::new();
    for k in 0..c.contour_count() {
        let Some((v, fechado)) = c.contour(k) else {
            continue;
        };
        if !fechado {
            continue;
        }
        for no in v {
            let ent = kurbo::Vec2::new(
                no.anchor[0] - no.in_handle[0],
                no.anchor[1] - no.in_handle[1],
            );
            let sai = kurbo::Vec2::new(
                no.out_handle[0] - no.anchor[0],
                no.out_handle[1] - no.anchor[1],
            );
            if ent.hypot() <= 0.0 || sai.hypot() <= 0.0 {
                // ⚠️ **Um buraco e não um salto** — saltá-lo desalinharia o emparelhamento com o
                // outro estado do mesmo caminho.
                out.push(None);
                continue;
            }
            let mut d = sai.atan2() - ent.atan2();
            while d > std::f64::consts::PI {
                d -= std::f64::consts::TAU;
            }
            while d < -std::f64::consts::PI {
                d += std::f64::consts::TAU;
            }
            out.push(Some(d.abs().to_degrees()));
        }
    }
    out
}

/// A mesma barra, **elíptica** — uma forma cujos nós têm alças a sério.
///
/// ⛔⛔ **A [`forma`] não serve a um gate de TANGENTE e isso está medido:** um rectângulo tem as
/// alças **em cima das âncoras**, logo a lei ingénua não deixa tangente nenhuma para comparar e a
/// régua lê `0,000°` dos dois lados — *verde por vácuo sobre o defeito*.
///
/// ⛔⛔ **E a `RoundRect` também não serve, por um motivo que engana:** ali o arredondamento é um
/// `corner_radius` **dentro do vértice** (Live Corners, ADR-0121), resolvido só no `cooked()` — a
/// FONTE, que é sobre quem esta lei corre, continua a ser um rectângulo de alças degeneradas.
/// *Uma forma que parece curva na tela pode ser recta na fonte.*
fn forma_curva() -> VecPath {
    cook(ShapeKind::Ellipse, [0.0, 0.0], [40.0, 10.0], &[])
}

/// A tabela do padrão-ouro DERIVADA da forma: o 1.º osso manda na metade esquerda, o 2.º na direita.
///
/// ⚠️ **Derivada e não escrita à mão**, porque a [`forma_curva`] tem mais nós do que o rectângulo e
/// uma tabela com a contagem errada é **ignorada em silêncio** (cai na lei derivada).
fn tabela_de(p: &VecPath) -> Vec<f64> {
    let mut out = Vec::new();
    for v in p.verts_all() {
        let linha = if v.anchor[0] < 20.0 {
            [1.0, 0.0]
        } else {
            [0.0, 1.0]
        };
        for _ in 0..3 {
            out.extend_from_slice(&linha);
        }
    }
    out
}

/// ⭐⭐⭐ **CONCILIAR A TANGENTE É O QUE CURA O REPORT — e o CONTROLO é a lei sem ela.**
///
/// O report do dono de 2026-09-19 (*«muitas irregularidades … certamente um mau tratamento das
/// alças dos handles»*) é esta medição: com cada segmento a resolver o seu ajuste sozinho, as duas
/// alças de um nó deixam de ser colineares e o nó LISO vira QUINA.
///
/// ⚠️⚠️ **O controlo é construído das peças PRIVADAS do produto** — a lei ingénua mais a
/// [`super::correccao_das_alcas`], sem o passe da [`super::reconcilia`] —, e é exactamente o
/// produto de ontem. *Sem ele este gate ficaria verde sobre uma fixtura que por acaso não dobra.*
///
/// (Mutações: apagar a chamada a `reconcilia` ⇒ RED na 2.ª · devolver `comum = desvio[i]` ⇒ RED na
/// 2.ª · reconciliar só uma das metades ⇒ RED na 2.ª.)
#[test]
fn conciliar_as_alcas_tira_a_quina_que_o_ajuste_livre_crava() {
    // ⛔ **Uma forma de arestas rectas NÃO entra aqui, e é medição e não descuido:** ali as quatro
    // alças da fonte estão em cima das âncoras, os quatro nós são CANTOS, e a conciliação é inerte
    // por desenho — ver [`super::SegmentoDaPele::direccao`].
    for (nome, fonte) in [("elipse", forma_curva())] {
        let k = pele(1.2);
        let t = tabela_de(&fonte);

        // ⭐ O CONTROLO: a lei de ontem, montada das peças do produto.
        let mut livre = fonte.clone();
        crate::aplica_corrigido(&k, &mut livre, &t, &[]);
        let n = fonte.verts.len();
        for seg in 0..n {
            let s = super::SegmentoDaPele {
                src: super::cubica(&fonte.verts, seg, n),
                pele: &k,
                ra: super::linha(&t, 2, seg),
                rb: super::linha(&t, 2, (seg + 1) % n),
                correcoes: &[],
                rigido: true,
            };
            let ja = super::cubica(&livre.verts, seg, n);
            let (d1, d2) = super::correccao_das_alcas(&s, &ja);
            let j = (seg + 1) % n;
            livre.verts[seg].out_handle = [
                livre.verts[seg].out_handle[0] + d1.x,
                livre.verts[seg].out_handle[1] + d1.y,
            ];
            livre.verts[j].in_handle = [
                livre.verts[j].in_handle[0] + d2.x,
                livre.verts[j].in_handle[1] + d2.y,
            ];
        }

        let mut curva = fonte.clone();
        aplica_pela_curva(&k, &mut curva, &t, &[]);

        // ⭐ A REFERÊNCIA é a lei ingénua: ela aplica UM afim às três metades de cada vértice, logo
        // preserva o que a fonte tinha (um nó liso fica liso, uma quina mantém a quina que o afim
        // lhe dá). O que se mede é quanto cada ajuste se AFASTA dela.
        let mut ingenua = fonte.clone();
        crate::aplica_corrigido(&k, &mut ingenua, &t, &[]);
        let sem = mudanca_da_tangente(&ingenua, &livre);
        let com = mudanca_da_tangente(&ingenua, &curva);
        // ⭐⭐ **E a conciliação não pode custar FIDELIDADE** — sem esta metade, uma escolha
        // qualquer do ângulo comum (o da 1.ª metade, por exemplo) fica trivialmente colinear e
        // passa. *Medido: ela é a única régua que separa a média pesada de um palpite.*
        let n2 = fonte.verts.len();
        let (mut cru, mut fino) = (0.0_f64, 0.0_f64);
        for seg in 0..n2 {
            let sd = super::SegmentoDaPele {
                src: super::cubica(&fonte.verts, seg, n2),
                pele: &k,
                ra: super::linha(&t, 2, seg),
                rb: super::linha(&t, 2, (seg + 1) % n2),
                correcoes: &[],
                rigido: true,
            };
            let (ing, agora) = (
                super::cubica(&ingenua.verts, seg, n2),
                super::cubica(&curva.verts, seg, n2),
            );
            for i in 1..64 {
                let u = f64::from(i) / 64.0;
                let verdade = sd.ponto(u);
                cru = cru.max((verdade - ing.eval(u)).hypot());
                fino = fino.max((verdade - agora.eval(u)).hypot());
            }
        }
        eprintln!(
            "[alcas] {nome}: sem conciliar {sem:.3}° · conciliada {com:.3e}° · erro ingenua \
             {cru:.4} -> conciliada {fino:.4}"
        );
        assert!(
            sem > 5.0,
            "em `{nome}` o ajuste livre virou a tangente so' {sem}° — a fixtura nao contem o \
             report, e a asserção abaixo passa a ser trivial"
        );
        // ⚠️ **A barra sai de DOIS pontos medidos, não de um número confortável:** a média pesada
        // pelo comprimento dá `2,1356` e a alternativa óbvia — o nó tomar o ângulo da 1.ª metade —
        // dá `2,3904`, sobre um erro ingénuo de `4,6619`. A barra fica no vale entre as duas.
        assert!(
            fino < cru / 2.0,
            "em `{nome}` conciliar custou FIDELIDADE ({cru} -> {fino}): o angulo comum tem de ser \
             a media pesada pelo COMPRIMENTO das duas alcas, senao o passe endireita o no' e \
             estraga o desenho"
        );
        assert!(
            com < 1e-9,
            "em `{nome}` o no' continua a ser uma QUINA ({com}°): as duas alças de um no' tem de \
             rodar JUNTAS, senão o `kind` que o artista desenhou sobrevive como byte e morre como \
             desenho"
        );
    }
}

/// ⭐⭐⭐ **NUMA FORMA DE ARESTAS RECTAS O PASSE É INERTE — e isso é desenho, não descuido.**
///
/// Uma alça em cima da âncora não carrega tangente nenhuma, e o nó onde ela chega é um **CANTO**:
/// ali não há continuidade para conciliar, e o ajuste livre é a resposta mais fiel. ⇒ o passe
/// salta-a, e numa forma cujas quatro alças estão nas âncoras ele não toca em **nada**.
///
/// ⚠️⚠️ **É esta a metade que guarda a F30 na arte que a caneta desenha sem arrastar.** Uma
/// alça degenerada que ENTRASSE na média envenena o ângulo comum com um `atan2(0, 0)` e faz a
/// outra metade do nó rodar para um sítio que ninguém pediu — e ela continua em cima da âncora,
/// porque rodar o vector nulo dá o vector nulo. *O sintoma seria a arte de polígono a torcer-se
/// nos cantos, sem uma linha de erro.*
///
/// (Mutação: tirar o `l <= 0.0` do salto ⇒ RED.)
#[test]
fn numa_forma_de_arestas_rectas_o_passe_nao_toca_em_nada() {
    let k = pele(1.2);
    let fonte = forma();
    let t = tabela_de(&fonte);

    let mut livre = fonte.clone();
    crate::aplica_corrigido(&k, &mut livre, &t, &[]);
    let n = fonte.verts.len();
    for seg in 0..n {
        let s = super::SegmentoDaPele {
            src: super::cubica(&fonte.verts, seg, n),
            pele: &k,
            ra: super::linha(&t, 2, seg),
            rb: super::linha(&t, 2, (seg + 1) % n),
            correcoes: &[],
            rigido: true,
        };
        let ja = super::cubica(&livre.verts, seg, n);
        let (d1, d2) = super::correccao_das_alcas(&s, &ja);
        let j = (seg + 1) % n;
        livre.verts[seg].out_handle = [
            livre.verts[seg].out_handle[0] + d1.x,
            livre.verts[seg].out_handle[1] + d1.y,
        ];
        livre.verts[j].in_handle = [
            livre.verts[j].in_handle[0] + d2.x,
            livre.verts[j].in_handle[1] + d2.y,
        ];
    }
    let mut curva = fonte.clone();
    aplica_pela_curva(&k, &mut curva, &t, &[]);

    // ⭐ O CONTROLO: a correcção livre MOVEU as alças — senão isto seria «nada mexeu em nada».
    let moveu = livre
        .verts
        .iter()
        .zip(&fonte.verts)
        .fold(0.0_f64, |m, (x, y)| {
            m.max((x.out_handle[0] - y.out_handle[0]).abs())
                .max((x.out_handle[1] - y.out_handle[1]).abs())
        });
    assert!(
        moveu > 1.0,
        "a correccao livre mexeu so' {moveu} — a fixtura nao contem o fenomeno"
    );
    for (a, b) in curva.verts.iter().zip(&livre.verts) {
        assert_eq!(
            (a.in_handle, a.out_handle),
            (b.in_handle, b.out_handle),
            "o passe da conciliacao tocou numa forma de arestas rectas: uma alca degenerada \
             entrou na media e envenenou o angulo comum do no'"
        );
    }
}

/// ⭐⭐⭐ **NUM NÓ MISTO, A METADE SEM EIXO FICA FORA DA MÉDIA.**
///
/// Um nó cuja alça de saída está em cima da âncora e cuja alça de entrada é a sério — a costura
/// entre um troço recto e um troço curvo, que toda arte mista tem. Ali há **uma** tangente, logo o
/// ângulo comum do nó **é** o desvio dela, e a alça de entrada não se mexe.
///
/// ⚠️⚠️ **Sem a cerca, a metade sem eixo entra com `atan2(0, 0) = 0` e um comprimento REAL** (a
/// correcção livre revive a alça), e o ângulo comum sai envenenado: a única tangente do nó roda
/// para um sítio que ninguém pediu. *A régua é o desenho, e o que se afirma é que ela NÃO se mexe.*
///
/// (Mutação: tirar o `es[i].hypot() <= 0.0` da média ⇒ RED.)
#[test]
fn num_no_misto_a_metade_sem_eixo_nao_entra_na_media() {
    let k = pele(1.2);
    let mut fonte = forma_curva();
    // ⭐ A COSTURA: o nó perde a alça de SAÍDA e mantém a de entrada. ⚠️ **Tem de ser um nó cujo
    // segmento seguinte ATRAVESSE a junta**, senão o mapa é afim ali, a correcção livre é zero e a
    // alça não revive — a fixtura fica sem o fenómeno (medido: `1,0e-14`).
    let n = fonte.verts.len();
    // ⚠️⚠️ **A tabela é BESPOKE e não a [`tabela_de`]: um nó SÓ fica com o 1.º osso.** É isso que
    // põe as DUAS arestas dele a atravessar a junta — a de entrada para a correcção livre RODAR a
    // alça de entrada (senão o ângulo comum é zero e a mutação da cerca não tem o que estragar) e
    // a de saída para ela REVIVER a alça degenerada. *Com a tabela partida por `x` nenhum nó da
    // elipse tem as duas, e o gate media o nada* (medido).
    let costura = (0..n)
        .min_by(|&a, &b| {
            fonte.verts[a].anchor[0]
                .partial_cmp(&fonte.verts[b].anchor[0])
                .expect("coordenadas finitas")
        })
        .expect("a elipse tem nos");
    fonte.verts[costura].out_handle = fonte.verts[costura].anchor;
    let mut t = Vec::new();
    for (i, _) in fonte.verts.iter().enumerate() {
        let linha = if i == costura { [1.0, 0.0] } else { [0.0, 1.0] };
        for _ in 0..3 {
            t.extend_from_slice(&linha);
        }
    }

    let mut livre = fonte.clone();
    crate::aplica_corrigido(&k, &mut livre, &t, &[]);
    for seg in 0..n {
        let s = super::SegmentoDaPele {
            src: super::cubica(&fonte.verts, seg, n),
            pele: &k,
            ra: super::linha(&t, 2, seg),
            rb: super::linha(&t, 2, (seg + 1) % n),
            correcoes: &[],
            rigido: true,
        };
        let ja = super::cubica(&livre.verts, seg, n);
        let (d1, d2) = super::correccao_das_alcas(&s, &ja);
        let j = (seg + 1) % n;
        livre.verts[seg].out_handle = [
            livre.verts[seg].out_handle[0] + d1.x,
            livre.verts[seg].out_handle[1] + d1.y,
        ];
        livre.verts[j].in_handle = [
            livre.verts[j].in_handle[0] + d2.x,
            livre.verts[j].in_handle[1] + d2.y,
        ];
    }
    let mut curva = fonte.clone();
    aplica_pela_curva(&k, &mut curva, &t, &[]);

    // ⭐ O CONTROLO: a correcção livre REVIVEU a alça sem eixo — é isso que dá à metade morta um
    // comprimento com que envenenar a média.
    let revivida = (livre.verts[costura].out_handle[0] - livre.verts[costura].anchor[0])
        .hypot(livre.verts[costura].out_handle[1] - livre.verts[costura].anchor[1]);
    assert!(
        revivida > 0.5,
        "a alca sem eixo ficou em cima da ancora ({revivida}) — a fixtura nao contem o fenomeno"
    );
    assert_eq!(
        curva.verts[costura].in_handle, livre.verts[costura].in_handle,
        "a unica tangente do no' misto rodou: a metade sem eixo entrou na media com um \
         `atan2(0, 0)` e envenenou o angulo comum"
    );
    // ⭐⭐ **E a metade SEM eixo também não se mexe** — ela não tem referência para onde rodar, e a
    // única resposta honesta é deixar o ajuste livre em paz. *Sem esta linha, a cerca do laço que
    // ESCREVE fica sem quem a mate.*
    assert_eq!(
        curva.verts[costura].out_handle, livre.verts[costura].out_handle,
        "a metade sem eixo foi rodada pelo angulo comum do no' — ela nao tem eixo, logo nao ha \
         angulo para o qual a rodar"
    );
}

/// ⭐⭐⭐ **UM OSSO COLAPSADO NÃO APAGA A FORMA** — a cerca do vector nulo, com quem a mata.
///
/// ⚠️ **Ela é alcançável pelo artista:** basta pôr a escala de um osso a `0` no `Transform`. Ali a
/// pose é singular, o mapa da pele é **constante**, e a direcção da tangente sai `R·û = 0` —
/// normalizá-la dá `NaN`, e um `NaN` numa alça faz a forma **desaparecer**, sem um erro.
///
/// ⛔ A resposta honesta é a forma colapsar num ponto, que é o que um osso de escala zero manda
/// fazer. O que ela não pode é virar `NaN`.
///
/// ⚠️⚠️ **Ele NÃO mata a mutação que tira a cerca da [`super::versor`], e está escrito lá porquê:**
/// nestas condições a alça colapsa junto com o mapa e a [`super::reconcilia`] salta o nó antes de
/// olhar para o eixo. *Este gate afirma a propriedade do DESENHO; a cerca fica declarada como
/// defensiva.* ⛔ Não escreva aqui uma promessa de mutação que ele não cumpre.
#[test]
fn um_osso_colapsado_nao_devolve_nan() {
    // ⭐⭐ **Um eixo colapsado, e o TENDÃO posto à mão** — sem o tendão os dois ossos partilham a
    // coluna `0` da tabela e a lei nunca chega ao osso morto.
    let mut vivo = SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        10.0,
        2.0,
        Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular");
    vivo.tendon = 0;
    let mut morto = SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        10.0,
        2.0,
        // ⚠️ O `y` colapsa e o `x` não — é essa a célula que mata a cerca. Com a pele INTEIRA
        // colapsada o mapa é constante, a alça cai em cima da âncora e a [`reconcilia`] salta o nó
        // **antes** de olhar para o eixo: o `NaN` é calculado e nunca chega ao desenho.
        Xform([1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular");
    morto.tendon = 1;
    let k = Skin::new(vec![vivo, morto]).expect("2 ossos");

    let mut p = cook(ShapeKind::Ellipse, [0.0, 0.0], [10.0, 5.0], &[]);
    // O nó da ESQUERDA (alças verticais) fica com o osso morto; os outros com o vivo.
    let mut t = Vec::new();
    for v in p.verts_all() {
        let linha = if v.anchor[0] < 1.0 {
            [0.0, 1.0]
        } else {
            [1.0, 0.0]
        };
        for _ in 0..3 {
            t.extend_from_slice(&linha);
        }
    }
    aplica_pela_curva(&k, &mut p, &t, &[]);
    for v in p.verts_all() {
        for q in [v.anchor, v.in_handle, v.out_handle] {
            assert!(
                q[0].is_finite() && q[1].is_finite(),
                "a forma virou NaN com um osso de escala zero num eixo — a cerca do vector nulo \
                 saiu, e uma alça revivida pela correcção livre encontrou um eixo `NaN`"
            );
        }
    }
}

/// O maior afastamento entre as VIRAGENS de dois estados do mesmo caminho, em graus — ver
/// [`viragens`].
fn mudanca_da_tangente(referencia: &VecPath, agora: &VecPath) -> f64 {
    let (r, a) = (viragens(referencia), viragens(agora));
    assert_eq!(r.len(), a.len(), "os dois estados tem de ter os mesmos nos");
    assert!(
        r.iter().filter(|x| x.is_some()).count() >= 4,
        "menos de quatro nos com tangente dos DOIS lados — a fixtura nao contem o fenomeno, e          esta regua leria zero por vacuo"
    );
    r.into_iter()
        .zip(a)
        .filter_map(|(x, y)| Some((x?, y?)))
        .fold(0.0_f64, |m, (x, y)| m.max((x - y).abs()))
}
