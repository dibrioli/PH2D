//! ⭐ **As dimensões de uma forma** — o que ela mede, e o que se pode escrever nela.
//!
//! # Por que isto existe
//!
//! Até aqui a única coisa editável de uma primitiva era o **raio do filete**. Um modelador em que
//! não se consegue dizer *"este cilindro tem 20 de raio e 50 de altura"* não é um modelador de
//! precisão — é um de escala uniforme, que é o gesto que sobra quando não há números.
//!
//! # A divisão: o documento dá a PAREDE, a vista dá o CONFORTO
//!
//! Cada grandeza diz o que **admite** ([`Dim::span`]) — o `round` de uma caixa não pode chegar à
//! meia-extensão dela, porque a fonte encolhida deixaria de existir. Isso é do documento e não se
//! negoceia.
//!
//! ⛔ **O teto de um slider NÃO é isso.** A largura de uma caixa não tem limite nenhum: escrever um
//! aqui seria inventar um número que a física não pede — o que o [`CLAUDE.md §0`] proíbe. Quem
//! escolhe até onde o **gesto** vai é a vista, e a resposta natural é *o que cabe no enquadramento*
//! — uma dimensão maior do que o quadro é uma cujo efeito não se vê. O campo numérico continua sem
//! teto, porque digitar 1000 é uma afirmação sobre a peça e não sobre a janela.
//!
//! # ⚠️ Uma faixa tem DUAS pontas, e o piso não é sempre zero
//!
//! Foi o que faltou à primeira versão: [`Dim`] só dizia o **teto**, e o painel punha o piso em zero
//! para todas as linhas. Numa largura isso está certo (o documento recusa ≤ 0); numa **posição** é um
//! defeito com sintoma mudo — digitar `-0,5` era reescrito para `0` pelo espelho do controle, e a
//! peça ia para a origem. O smoke não o apanhou porque o número experimentado foi positivo.
//!
//! Daí o [`Span`]: cada grandeza diz a **forma** da sua faixa e de que recurso vem cada ponta, e
//! quem fecha as pontas abertas é a vista, num sítio só.
//!
//! # Meias-extensões não aparecem
//!
//! O documento guarda **meias**-extensões (é a forma que a distância assinada quer). Ninguém diz que
//! uma caixa tem «meia-largura 5»: [`dims`] devolve a largura **inteira** e [`set_dim`] volta a
//! dividir. A conversão mora aqui, num sítio, e não em cada painel que a mostre.
//!
//! [`CLAUDE.md §0`]: ../../../CLAUDE.md

use crate::{FieldError, Primitive, round_limit};

/// ⭐ **Que número autorado de um nó** — a identidade de uma linha do painel.
///
/// ⚠️ Um `usize` cru serviria, com o painel a saber que «0..2 é a posição e o resto são dimensões».
/// Uma convenção implícita entre duas crates é o tipo de coisa que sobrevive até alguém acrescentar
/// uma linha no meio — e aí o controle passa a escrever noutro número, em silêncio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Param {
    /// A translação **local** do nó, por eixo (0 = X).
    ///
    /// ⚠️ **Local, e é a convenção da casa**: o Inspector dela mostra o `Transform.translation`, que
    /// é local, e o readout do gizmo 2D diz por extenso que o delta é local *"porque é isso que o
    /// Inspector mostra"*. Um painel que mostrasse mundo contradiria o número ao lado no dia em que
    /// alguém agrupasse.
    Pos(u8),
    /// Um dos três ângulos da rotação **local**, por eixo (0 = X), em **graus**.
    ///
    /// ⚠️ A pose guarda um **quaternion**; estes três são o nome canónico dele. Ver
    /// [`crate::xform::set_rotation_degree`], que é onde a lei (e o que ela recusou) está escrita.
    Rot(u8),
    /// A escala **uniforme** do nó. Ver a nota de [`crate::Xform::scale`].
    Scale,
    /// Uma dimensão da forma — a posição na lista de [`dims`].
    Dim(u16),
    /// Um número de um **modificador** — `slot` é a posição na pilha, `field` é qual dos números
    /// dele. Ver [`crate::mods`].
    ///
    /// ⚠️ **A posição, e não a natureza**: a pilha pode ter duas cascas, e uma chave por natureza
    /// não as distinguiria — escrever numa escreveria na outra.
    ///
    /// ⚠️ **E DOIS índices, não um**: uma matriz tem quantas cópias *e* que espaçamento. Um índice
    /// só obrigaria a achatar a pilha inteira numa lista de números, e aí inserir um modificador no
    /// meio renumeraria tudo o que vem depois — com um arrasto a meio a escrever noutro campo.
    Mod { slot: u16, field: u8 },
    /// ⭐⭐ **O NÍVEL DE RESOLUÇÃO de uma forma que ainda está ligada ao desenho** (W55).
    ///
    /// ⚠️ **Não é uma dimensão da FORMA, e é por isso que tem chave própria.** Um `Dim` diz o que a
    /// peça mede — largura, altura, filete —, e mexer nele muda a peça. Este número não muda a peça
    /// nenhuma: muda **com que finura o contorno desenhado é convertido** nela. As duas coisas vivem
    /// em sítios diferentes (a forma no nó, o vínculo ao lado dele) e sobrevivem a gestos
    /// diferentes — largar o vínculo apaga este número e deixa a forma intacta.
    ///
    /// O teto é [`crate::MAX_PROFILE_RESOLUTION`], e ele é medido.
    Resolution,
    /// ⭐⭐⭐ **O RAIO DA JUNÇÃO desta forma** (W98) — com que arredondamento ela se encontra com o
    /// resultado das anteriores.
    ///
    /// ⚠️ **Não é o [`Param::Dim`] do filete da forma, e a diferença é o SUJEITO.** O `Dim` do
    /// arredondamento é das arestas **dela própria** — as 12 de uma caixa, o aro de um cilindro — e
    /// existe mesmo numa peça de uma forma só. Este é do **encontro**, e só existe porque há alguma
    /// coisa antes. Uma caixa arredondada que corta com aresta viva precisa dos dois números ao
    /// mesmo tempo, e uma chave só não os saberia distinguir.
    ///
    /// ⭐ **Escrever aqui MATERIALIZA o verbo** quando a forma o estava a herdar: pedir um raio de
    /// junção próprio *é* pronunciar-se. O painel mostra isso na hora — o chip `Inherit` apaga-se e
    /// acende o verbo que ela agora tem por escrito.
    ///
    /// ⚠️ A **base** não tem esta chave: ela semeia o acumulado e não se junta a nada
    /// ([`crate::fold_verb`]).
    Joint,
}

/// ⭐ **O que uma grandeza admite** — a forma da faixa, e de que recurso vem cada ponta.
///
/// ⚠️ Nenhuma variante escolhe um número por conforto: ou a ponta é do **documento** (a peça deixa
/// de existir acima dela), ou é da **representação** (um ângulo canónico não passa de meia volta),
/// ou está **aberta** e quem a fecha é a vista — que é a única a saber o que cabe no quadro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Span {
    /// Positiva e sem parede: uma largura, um raio, uma escala. O documento recusa `≤ 0`; o teto é
    /// o alcance da **vista**.
    Positive,
    /// Positiva, com **parede** do documento: o filete. Acima de `wall` a fonte encolhida deixaria
    /// de existir e o campo deixaria de ser uma distância.
    Wall(f32),
    /// Simétrica e sem parede nenhuma: uma **posição**. As duas pontas são o alcance da vista, e a
    /// de baixo é negativa — a origem não é um canto do mundo.
    Free,
    /// **Periódica**: um ângulo. As pontas são `±half` e são a própria **representação** — nem o
    /// documento nem a vista têm voto, e um número além delas não é recusado, é renomeado.
    Turn(f32),
    /// ⭐ **Não há faixa nenhuma agora**: a grandeza existe, tem valor, e **não é editável neste
    /// estado**.
    ///
    /// ⚠️ É diferente de *"não aparece"*. O valor continua a ser um facto que o artista precisa de
    /// ler — e esconder a linha faria o painel saltar de tamanho a cada travessia. O que ela perde é
    /// o **controle**: quem a recebe pinta um facto, não um slider (*uma affordance que não pode ser
    /// honrada é pior do que nenhuma*).
    ///
    /// O caso de hoje é o terceiro ângulo na trava de cardan — ver
    /// [`crate::xform::rotation_axis_is_free`], que é a **mesma** porta que recusa a escrita.
    Locked,
    /// ⭐ **Simétrica, e fechada pelo DOCUMENTO**: `±max`, sem a vista ter voto.
    ///
    /// ⚠️ É a irmã da [`Span::Free`] com as pontas fechadas, e a diferença é de onde vem o número:
    /// numa posição não há limite nenhum e a vista escolhe o alcance; aqui o limite é um **facto**
    /// do documento — hoje, o custo de marcha que a inclinação paga
    /// ([`crate::mods::MAX_TAPER_SLOPE`]).
    Walls(f32),
    /// ⭐ **Uma CONTAGEM**: inteira, de `min` a `max`. Quantas cópias uma matriz tem, quantos lados
    /// um prisma tem.
    ///
    /// ⚠️ É uma faixa **própria** e não uma `Positive` disfarçada, porque três coisas mudam de uma
    /// vez: o passo do arrasto é **1** (e não um centésimo do curso), o número mostra-se **sem
    /// casas decimais** (não existe meia cópia), e o piso não é zero.
    ///
    /// ⚠️ **O `min` é um campo desde a W101**, e ele nasceu de um caso concreto: uma matriz começa
    /// em **1** (zero cópias é a peça a desaparecer, e apagar já tem botão) e um prisma começa em
    /// **3** (abaixo disso não há polígono). Com o piso fixo em `1`, o slider do prisma descia a 1,
    /// a escrita era recusada, e o controle **saltava para trás debaixo do dedo** — *uma recusa é
    /// informação, mas uma faixa que oferece o que a porta recusa é uma affordance que mente.*
    Count { min: u32, max: u32 },
    /// ⭐ **Positiva OU ZERO**, com o teto vindo da vista — a irmã da [`Span::Positive`] com o zero
    /// dentro.
    ///
    /// ⚠️ Ela existe por **uma** grandeza, e ela é a razão de ser da forma: o raio do TOPO de um
    /// [`crate::Primitive::Cone`], cujo zero **é o cone fechado**. Com `Positive` o documento recusa
    /// o zero e a forma que dá nome à primitiva fica indigitável; com `Free` o slider oferece
    /// negativo, que não quer dizer nada.
    FromZero,
}

/// Uma grandeza editável de um nó.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dim {
    /// A chave i18n do nome. ⚠️ Uma **chave**, nunca um rótulo pronto (HR-15).
    pub key: &'static str,
    /// O valor que o artista vê — já em unidades inteiras (ver o doc do módulo).
    pub value: f32,
    /// **O que ela admite**, e de onde vem cada ponta. Ver [`Span`].
    pub span: Span,
}

/// **O que esta forma mede.** Vazio quando ela não tem número nenhum a mexer.
///
/// ⚠️ A ordem é a que o painel mostra, e ela é parte da identidade: [`set_dim`] recebe o **índice**,
/// porque é o que um controle cunhado às cegas consegue guardar. Reordenar aqui reordena os
/// controles — e um arrasto a meio passaria a escrever noutro número.
#[must_use]
pub fn dims(p: &Primitive) -> Vec<Dim> {
    let round_dim = |value: f32| Dim {
        key: "field.dim.round",
        value,
        // ⚠️ `Positive` só sobra para uma forma que tenha filete e não tenha meia-extensão de onde
        // derivar a parede — não existe hoje, e a alternativa (um `expect`) transformaria uma
        // primitiva nova num pânico em vez de num slider sem teto.
        span: round_limit(p).map_or(Span::Positive, Span::Wall),
    };
    match p {
        Primitive::Box { half, round } => vec![
            Dim {
                key: "field.dim.width",
                value: half[0] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half[1] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.depth",
                value: half[2] * 2.0,
                span: Span::Positive,
            },
            round_dim(*round),
        ],
        Primitive::Sphere { radius } => vec![Dim {
            key: "field.dim.radius",
            value: *radius,
            span: Span::Positive,
        }],
        Primitive::Cylinder {
            radius,
            half_height,
            round,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            round_dim(*round),
        ],
        Primitive::Torus { major, minor } => vec![
            Dim {
                key: "field.dim.radius",
                value: *major,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.thickness",
                value: *minor,
                span: Span::Positive,
            },
        ],
        Primitive::Extrude {
            half_height, round, ..
        } => vec![
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            round_dim(*round),
        ],
        // ⚠️ Um torno **não tem dimensões próprias**: a forma dele é o contorno desenhado, e um
        // número aqui prometeria mexer numa coisa que só o editor vetorial autora. Ver
        // [`Primitive::Revolve`].
        Primitive::Revolve { .. } => Vec::new(),
        // ⭐⭐ **O topo é o único número deste arquivo cujo piso é ZERO** (W101) — `Span::Positive`
        // proíbe o zero, e o zero é o **cone fechado**. `Span::Walls` também não serve (aceitaria
        // negativo), então a faixa é `From { lo: 0 }`: *a forma que dá nome à primitiva não pode
        // ser indigitável*.
        Primitive::Cone {
            bottom,
            top,
            half_height,
            round,
        } => vec![
            Dim {
                key: "field.dim.radius_bottom",
                value: *bottom,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.radius_top",
                value: *top,
                span: Span::FromZero,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            round_dim(*round),
        ],
        Primitive::Capsule {
            radius,
            half_height,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            // ⚠️ **A altura é a do SEGMENTO, não a da peça** — a cápsula mede `2·(h + r)` de ponta a
            // ponta. Publicar o total faria o número saltar ao mexer no raio, que é a linha de
            // cima: *um controle que se mexe sozinho quando se mexe noutro é o que faz o artista
            // deixar de confiar no painel.*
            Dim {
                key: "field.dim.length",
                value: half_height * 2.0,
                span: Span::Positive,
            },
        ],
        Primitive::Prism {
            sides,
            radius,
            half_height,
            round,
        } => vec![
            Dim {
                key: "field.dim.sides",
                value: *sides as f32,
                span: Span::Count {
                    min: crate::MIN_PRISM_SIDES,
                    max: crate::MAX_PRISM_SIDES,
                },
            },
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            round_dim(*round),
        ],
    }
}

/// ⭐ **Escreve uma dimensão**, ou recusa.
///
/// # ⚠️ Encolher uma forma ENCOLHE o filete dela, e não é recusado
///
/// Um `round` que deixa de caber quando a caixa encolhe é a situação normal, não um erro: o artista
/// pediu o tamanho, e o filete é o que **decorre** dele. Recusar obrigaria a desfazer o filete
/// primeiro — dois gestos onde há um — e é o que todo CAD resolve limitando o filete em silêncio.
///
/// ⚠️ **Em silêncio, mas não invisível**: o número do filete é uma linha do mesmo painel, e ela
/// muda à vista. Um valor que muda sozinho **sem aparecer** seria outra coisa.
///
/// # Errors
/// [`FieldError::NonPositive`] para um valor não-finito ou ≤ 0, e para um índice que não é desta
/// forma. [`FieldError::RoundTooLarge`] quando é o próprio filete que não cabe.
pub fn set_dim(p: &mut Primitive, node: u32, index: usize, value: f32) -> Result<(), FieldError> {
    let bad = |what: &'static str| FieldError::NonPositive { node, what };
    // ⭐⭐ **QUEM DECIDE SE O ZERO PASSA É A FAIXA DECLARADA** (W101), e não uma excepção escrita
    // aqui.
    //
    // ⚠️ Esta guarda dizia `value <= 0.0` para tudo, e isso tornava o **cone fechado**
    // indigitável: o raio de topo zero é a forma que dá nome à primitiva. Uma excepção
    // `if é_cone && index == 1` curaria o caso e não a família — a próxima grandeza cujo zero
    // significa alguma coisa voltaria a bater na mesma linha. A [`Span`] já sabe a resposta
    // (`FromZero`), e ela vem do mesmo sítio que o painel lê.
    let zero_ok = matches!(dims(p).get(index).map(|d| d.span), Some(Span::FromZero));
    if !value.is_finite() || value < 0.0 || (value == 0.0 && !zero_ok) {
        return Err(bad("dim"));
    }
    let half = value * 0.5;
    match (p, index) {
        (Primitive::Box { half: h, .. }, i @ 0..=2) => h[i] = half,
        (Primitive::Sphere { radius }, 0) | (Primitive::Cylinder { radius, .. }, 0) => {
            *radius = value;
        }
        (Primitive::Cylinder { half_height, .. }, 1)
        | (Primitive::Extrude { half_height, .. }, 0) => *half_height = half,
        (Primitive::Torus { major, .. }, 0) => *major = value,
        (Primitive::Torus { minor, .. }, 1) => *minor = value,
        (Primitive::Cone { bottom, .. }, 0) => *bottom = value,
        // ⚠️ **O único destino de um zero neste arquivo** — ver a guarda acima.
        (Primitive::Cone { top, .. }, 1) => *top = value,
        (Primitive::Cone { half_height, .. }, 2)
        | (Primitive::Capsule { half_height, .. }, 1)
        | (Primitive::Prism { half_height, .. }, 2) => *half_height = half,
        (Primitive::Capsule { radius, .. }, 0) | (Primitive::Prism { radius, .. }, 1) => {
            *radius = value;
        }
        (Primitive::Prism { sides, .. }, 0) => {
            // ⚠️ **COAGE, não recusa** — a lei do `Unary::Taper`, e pela mesma razão: a faixa já
            // não oferece nada fora de `[MIN, MAX]`, então um valor de fora só chega por outra
            // porta (um ficheiro estragado), e recusar ali rejeitaria a peça inteira. É o
            // **documento** quem arredonda: um valor fracionário vindo de fora vira uma contagem,
            // não meio lado.
            *sides = (value.round() as u32).clamp(crate::MIN_PRISM_SIDES, crate::MAX_PRISM_SIDES);
        }
        // O filete é o último de cada forma que o tem — e ele passa pela lei do filete, que já
        // sabe recusar o que não cabe.
        (
            p @ (Primitive::Box { .. }
            | Primitive::Cylinder { .. }
            | Primitive::Extrude { .. }
            | Primitive::Cone { .. }
            | Primitive::Prism { .. }),
            i,
        ) if Some(i) == round_index(p) => {
            return set_round(p, node, value);
        }
        _ => return Err(bad("dim")),
    }
    Ok(())
}

/// Onde fica o filete na lista desta forma, se ela tiver um.
fn round_index(p: &Primitive) -> Option<usize> {
    dims(p).iter().position(|d| d.key == "field.dim.round")
}

fn set_round(p: &mut Primitive, node: u32, value: f32) -> Result<(), FieldError> {
    let limit = round_limit(p).ok_or(FieldError::NonPositive {
        node,
        what: "round",
    })?;
    if value >= limit {
        return Err(FieldError::RoundTooLarge {
            node,
            round: value,
            limit,
        });
    }
    // ⚠️ **EXAUSTIVO, e o `_ => {}` que estava aqui era uma armadilha** (W101): uma primitiva nova
    // COM filete caía no braço vazio, o `round_limit` dela respondia, o `set_round` dizia `Ok` — e
    // o número **nunca era escrito**. Um slider que se mexe e não faz nada é a falha mais cara de
    // diagnosticar, porque não deixa rasto. Com a lista fechada, a próxima é erro de compilação.
    match p {
        Primitive::Box { round, .. }
        | Primitive::Cylinder { round, .. }
        | Primitive::Extrude { round, .. }
        | Primitive::Cone { round, .. }
        | Primitive::Prism { round, .. } => *round = value,
        Primitive::Sphere { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. } => {}
    }
    Ok(())
}

/// ⭐ **Limita o filete ao que a forma agora comporta** — ver a nota de [`set_dim`].
///
/// Devolve `true` se teve de o mexer, para quem chamar poder dizê-lo.
pub fn clamp_round(p: &mut Primitive) -> bool {
    let Some(limit) = round_limit(p) else {
        return false;
    };
    // ⚠️ **Estritamente abaixo**, e não «até»: a validação recusa `round >= limit`, e um filete
    // exatamente no limite encolheria a fonte a zero. A margem é uma fração do próprio limite, e
    // não um épsilon absoluto — numa peça de 0,01 um épsilon fixo seria o limite inteiro.
    let ceiling = limit * (1.0 - ROUND_MARGIN);
    // ⚠️ Exaustivo pela razão do [`set_round`] — um braço `_` deixaria a primitiva nova com um
    // filete que a peça já não comporta, e a validação recusaria o documento inteiro no gesto
    // seguinte.
    match p {
        Primitive::Box { round, .. }
        | Primitive::Cylinder { round, .. }
        | Primitive::Extrude { round, .. }
        | Primitive::Cone { round, .. }
        | Primitive::Prism { round, .. } => {
            if *round > ceiling {
                *round = ceiling.max(0.0);
                return true;
            }
            false
        }
        Primitive::Sphere { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. } => false,
    }
}

/// A folga entre o filete máximo e a parede, em fração da parede. Ver [`clamp_round`].
const ROUND_MARGIN: f32 = 1.0e-3;

/// ⭐ **Escala uma primitiva multiplicando as DIMENSÕES dela**, e não a pose.
///
/// # Por que uma folha não usa `Xform::scale`
///
/// ⚠️ **Uma folha escalada teria DUAS verdades sobre o mesmo tamanho visível**: a largura que o
/// painel mostra e o fator da pose. Uma caixa de 1 de largura escalada 2× mede 2 na tela e continua
/// a dizer «1» — e o artista não tem como saber qual das duas o próximo gesto vai mexer.
///
/// Multiplicar as dimensões dá **exatamente a mesma forma** (a escala uniforme é isso, aplicada ao
/// campo) com **um** número a mudar — o que o painel já mostra.
///
/// ⚠️ Um grupo é o contrário: ele **não tem dimensões próprias**, então o fator da pose é a única
/// resposta, e ali ele não compete com nada.
///
/// Devolve `false` para um fator não-positivo ou não-finito, sem tocar na forma.
pub fn scale_primitive(p: &mut Primitive, factor: f32) -> bool {
    if !factor.is_finite() || factor <= 0.0 {
        return false;
    }
    match p {
        Primitive::Box { half, round } => {
            for h in half.iter_mut() {
                *h *= factor;
            }
            *round *= factor;
        }
        Primitive::Sphere { radius } => *radius *= factor,
        Primitive::Cylinder {
            radius,
            half_height,
            round,
        } => {
            *radius *= factor;
            *half_height *= factor;
            *round *= factor;
        }
        Primitive::Torus { major, minor } => {
            *major *= factor;
            *minor *= factor;
        }
        Primitive::Extrude {
            half_height, round, ..
        } => {
            // ⚠️ O **perfil** não é escalado: ele é o desenho, e o dono dele é o editor vetorial. O
            // que esta escala mexe é a altura da extrusão e o aro — as duas grandezas que este
            // módulo autora. Escalar um perfil aqui seria reescrever, em silêncio, um documento de
            // outro módulo.
            *half_height *= factor;
            *round *= factor;
        }
        // Um torno é só o perfil: não há nada aqui que este módulo possua.
        Primitive::Revolve { .. } => return false,
        Primitive::Cone {
            bottom,
            top,
            half_height,
            round,
        } => {
            *bottom *= factor;
            // ⚠️ **O topo escala como os outros, e o zero fica zero** — é o que mantém um cone
            // fechado fechado ao redimensionar. Uma escala que somasse seria a que o abriria.
            *top *= factor;
            *half_height *= factor;
            *round *= factor;
        }
        Primitive::Capsule {
            radius,
            half_height,
        } => {
            *radius *= factor;
            *half_height *= factor;
        }
        Primitive::Prism {
            sides: _,
            radius,
            half_height,
            round,
        } => {
            // ⚠️ **A contagem NÃO escala** — ela não é um comprimento. Multiplicá-la faria um
            // hexágono virar um dodecágono ao aumentar a peça, que é mudar a forma e não o tamanho.
            *radius *= factor;
            *half_height *= factor;
            *round *= factor;
        }
    }
    true
}
