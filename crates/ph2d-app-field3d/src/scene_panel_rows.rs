//! ⭐⭐⭐ **OS NÚMEROS DO OBJECTO ESCOLHIDO** — as linhas que o painel do modelador mostra.
//!
//! ⚠️ **Saíram do [`super::scene_panel`] em 2026-09-15 por TETO DE LINHAS**, e a fronteira é a que
//! o doc da [`param_rows`] já declarava: a Hierarquia mostra **o que existe** e isto mostra **os
//! números do escolhido**. O resto do painel são os CHIPS — as vistas, os modos, os verbos —, que
//! são outra pergunta.
//!
//! ⭐ **É aqui, num sítio só, que uma faixa aberta se fecha** — ver o doc da [`param_rows`].

use super::*;

/// ⭐ **Os números do objeto selecionado** — o painel é o inspetor da seleção.
///
/// ⚠️ **Mudou de forma na W10.** Antes era uma linha por nó com o raio dele — uma segunda vista da
/// estrutura, a competir com a Hierarquia e sem onde pôr largura, altura e profundidade. A divisão
/// passou a ser a da casa: a Hierarquia mostra **o que existe**, o painel mostra **os números do
/// escolhido**.
///
/// `view_span` é o alcance do **gesto** (ver [`ph2d_field::Span`]): uma posição e uma largura não
/// têm teto físico, e quem escolhe até onde o slider vai é a vista — o que cabe no enquadramento. O
/// documento só contribui as **paredes** (um filete que não cabe).
///
/// ⭐ **É AQUI, num sítio só, que uma faixa aberta se fecha.** O documento diz a *forma* do que cada
/// grandeza admite; esta função é a única que sabe o enquadramento, e é ela que escreve as duas
/// pontas. Espalhar isto por linha era o que fazia toda linha começar em zero.
pub fn param_rows(
    world: &bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
    view_span: f32,
) -> Vec<ph2d_panel_model3d::ParamRow> {
    let Some(&e) = selection.first() else {
        return Vec::new();
    };
    // ⚠️ O valor E as pontas vêm os DOIS do nó (`params_of`). Um painel que guardasse o seu próprio
    // valor teria duas verdades sobre o mesmo número, e a que aparece na tela seria a errada sempre
    // que algo o mudasse de outro lado — um desfazer, um arquivo aberto, o gizmo.
    // ⭐⭐⭐ **A que SECÇÃO cada linha pertence** (report do Enio, 2026-08-30: *«o modificador deveria
    // ter sua própria seção no painel»*). Um cabeçalho nasce quando a **natureza** da linha muda:
    // as dimensões da forma, e depois um por modificador, com o **nome dele**.
    //
    // ⚠️ **Derivado do `Param`, e não uma segunda lista** — a ordem já é a do `params_of` (a forma
    // primeiro, os modificadores por último, na ordem em que correm), e um segundo sítio a decidir
    // secções divergiria dela no dia em que ela mudasse.
    let mods = ph2d_field_ecs::mods_of(world, e);
    let mut anterior: Option<ph2d_field::Param> = None;
    let secao = move |p: ph2d_field::Param| -> Option<&'static str> {
        let mesma = match (anterior, p) {
            // Duas linhas do MESMO modificador continuam a secção dele.
            (
                Some(ph2d_field::Param::Mod { slot: a, .. }),
                ph2d_field::Param::Mod { slot: b, .. },
            ) => a == b,
            // Duas linhas do material continuam a secção dele.
            (Some(ph2d_field::Param::Material(_)), ph2d_field::Param::Material(_)) => true,
            // ⭐ E duas linhas da LUZ continuam a dela.
            (Some(ph2d_field::Param::Light(_)), ph2d_field::Param::Light(_)) => true,
            // Tudo o que não é modificador, material nem luz é a forma, e ela é uma secção só.
            (Some(a), b) => {
                let solta = |p: ph2d_field::Param| {
                    !matches!(
                        p,
                        ph2d_field::Param::Mod { .. }
                            | ph2d_field::Param::Material(_)
                            | ph2d_field::Param::Light(_)
                    )
                };
                solta(a) && solta(b)
            }
            (None, _) => false,
        };
        anterior = Some(p);
        if mesma {
            return None;
        }
        match p {
            ph2d_field::Param::Mod { slot, .. } => mods
                .get(slot as usize)
                .map(|m| m.key())
                // ⚠️ Um slot sem modificador não pode acontecer (as duas listas saem do mesmo nó),
                // e se acontecer o cabeçalho genérico é melhor do que nenhum.
                .or(Some("panel.model3d.section.modifier")),
            // ⭐⭐⭐ **O MATERIAL é uma secção própria** (`docs/Render3d/05`) — o que a forma mede
            // à LUZ não é o que ela mede à régua, e cinco números sem cabeçalho debaixo das
            // dimensões leem-se como mais dimensões.
            ph2d_field::Param::Material(_) => Some("panel.model3d.section.material"),
            // ⭐⭐⭐ **A LUZ é uma secção própria** (ordem do dono, 14/09) — e a de cima dela é a
            // POSE, que sai com o cabeçalho da forma. *Uma lâmpada também tem um «onde».*
            ph2d_field::Param::Light(_) => Some("panel.model3d.section.light"),
            _ => Some("panel.model3d.section.shape"),
        }
    };
    let mut secao = secao;
    let numeros = ph2d_field_ecs::params_of(world, e);
    // ⭐⭐⭐ **OS TRÊS CANAIS DA COR BASE SÃO UMA LINHA SÓ** (Enio, 2026-09-14: *«em vez de 3 sliders
    // de RGB, deveríamos ter uma caixa seletora de cor»*) — ver [`ph2d_panel_model3d::ParamRow::swatch`].
    //
    // ⚠️ **A dobra é da APRESENTAÇÃO, e o `params_of` não muda.** Ele responde *«que números tem
    // este nó?»*, e a resposta continua a ser cinco: a porta de escrita `Param::Material(0..2)`
    // continua a existir, continua gateada, e é por ela que esta cura escreve. *Colapsar na fonte
    // tornaria a cor inalcançável por toda a maquinaria que já a alcança.*
    //
    // ⚠️ **Os três valores saem da MESMA lista** que dá as outras linhas — não do componente. Ler o
    // `FieldMaterial` aqui seria a segunda fonte para os mesmos números, e a que diverge no dia em
    // que o `params_of` passar a derivar algum deles.
    // ⚠️ **Ele recebe o `Param`, e não um índice**, desde que a luz entrou: um `u8` obrigaria quem
    // chama a saber de que família o canal é, e essa é exactamente a informação que a âncora já tem.
    let canal = |p: ph2d_field::Param| {
        numeros
            .iter()
            .find(|(q, _)| *q == p)
            .map_or(0.0, |(_, d)| d.value)
    };
    // ⭐⭐ **As cores de um material são uma TABELA de âncoras** (`docs/Render3d/05` §20), e não um
    // caso especial repetido: a base em `0` e a da emissão em `6`, cada uma com o rótulo da **cor
    // inteira** — `field.dim.base_r` diz *«Base Color R»*, que nomeia um canal, e a linha deixou de
    // ser um canal.
    //
    // ⚠️ **Os seguidores derivam da âncora** (`+1` e `+2`) em vez de serem listados: uma segunda
    // lista teria de ser reescrita a cada cor nova, e o dia em que as duas discordassem a linha
    // pintaria uma cor e escreveria noutra.
    const CORES: [(u8, &str); 4] = [
        (1, "field.dim.base_color"),
        (7, "field.dim.specular_color"),
        (13, "field.dim.coat_color"),
        (20, "field.dim.emission_color"),
    ];
    // ⭐⭐⭐ **E a cor de uma LUZ entra pela MESMA máquina** (ordem do dono, 14/09): âncora no canal
    // `R`, seguidores `+1` e `+2`. *A caixa de cor de uma lâmpada não é um segundo widget — é a
    // mesma linha, sobre outro `Param`.*
    const COR_DA_LUZ: (u8, &str) = (1, "field.dim.light_color");
    // ⚠️ **Os três canais saem da PORTA** ([`ph2d_field::Param::colour_channels`]) e não de uma
    // conta aqui: o dreno que os escreve usa a mesma, e é isso que impede a linha de pintar uma cor
    // e escrever noutra.
    let ancora = |p: ph2d_field::Param| -> Option<(&'static str, [ph2d_field::Param; 3])> {
        let rotulo = match p {
            ph2d_field::Param::Material(k) => CORES.iter().find(|(a, _)| *a == k).map(|(_, r)| *r),
            ph2d_field::Param::Light(k) if k == COR_DA_LUZ.0 => Some(COR_DA_LUZ.1),
            _ => None,
        }?;
        Some((rotulo, p.colour_channels()?))
    };
    let seguidor = |p: &ph2d_field::Param| match p {
        ph2d_field::Param::Material(k) => CORES.iter().any(|(a, _)| k == &(a + 1) || k == &(a + 2)),
        ph2d_field::Param::Light(k) => *k == COR_DA_LUZ.0 + 1 || *k == COR_DA_LUZ.0 + 2,
        _ => false,
    };
    let mut linhas: Vec<ph2d_panel_model3d::ParamRow> = numeros
        .iter()
        .cloned()
        // ⛔ **Os canais seguidores saem ANTES do `secao`**, que é uma máquina de estados sobre a
        // sequência: chamá-lo para uma linha que não é publicada partiria a secção do material em
        // duas — a segunda com cabeçalho repetido — no dia em que a ordem mudasse.
        .filter(|(p, _)| !seguidor(p))
        .map(|(param, d)| {
            use ph2d_field::{Bound, Span};
            let (lo, bound) = match d.span {
                // Positiva: o documento recusa `≤ 0`, e o teto é o que cabe no quadro.
                Span::Positive => (0.0, Bound::Soft(view_span)),
                // ⭐⭐⭐ **O tecto vem do DOCUMENTO e não da vista** (report do Enio de 09/09) — ver
                // [`ph2d_field::Span::SoftFromZero`]. ⚠️ O piso protege contra uma peça degenerada
                // dar um slider de curso zero, que é um controlo morto com aparência de vivo.
                Span::SoftFromZero(top) => (0.0, Bound::Soft(top.max(1.0e-4))),
                // A única ponta que o documento **impõe**.
                Span::Wall(w) => (0.0, Bound::Hard(w)),
                // Simétrica em torno da origem: as duas pontas são da vista, e a de baixo é
                // negativa — sem isto uma posição negativa não se digita.
                Span::Free => (-view_span, Bound::Soft(view_span)),
                // ⭐⭐⭐ **A banda de um deformador é uma posição AO LONGO DO EIXO DELE**, e o alcance
                // dela nunca sai da peça — ver [`ph2d_field::Span::Along`] e o report de 2026-08-31.
                //
                // ⚠️ **É DERIVADO do alcance do gesto, e não um segundo número**: o `view_span` é a
                // oitava de `4×` o raio da peça (ver [`gesture_span`]), logo um quarto dele é a
                // oitava do próprio raio — que majora a meia-extensão da peça em **qualquer** eixo.
                // *Um segundo parâmetro seria uma segunda resposta à mesma pergunta, e divergiria no
                // dia em que a primeira mudasse.*
                Span::Along => (-view_span * 0.25, Bound::Soft(view_span * 0.25)),
                // Periódica: as pontas são da representação, e a vista não tem voto.
                Span::Turn(half) => (-half, Bound::Wrap(half)),
                // ⭐ **Sem faixa nenhuma**: a grandeza tem valor e não é editável neste estado. As
                // duas pontas colapsam no próprio valor — não há para onde arrastar — e a linha
                // segue marcada para o painel a pintar como facto.
                Span::Locked => (d.value, Bound::Wrap(d.value)),
                // ⭐ **Contagem**: as duas pontas são do DOCUMENTO — uma matriz começa em 1 (zero
                // cópias é a peça a desaparecer, e apagar já tem botão) e um prisma em 3 (abaixo
                // não há polígono).
                //
                // ⚠️ **O piso era o literal `1.0` aqui** (W101): com ele, o slider dos lados descia
                // a 1, a porta do documento coagia para 3, e o controle **saltava para trás
                // debaixo do dedo**. *Uma faixa que oferece o que a porta recusa é uma affordance
                // que mente* — e o piso é um facto do documento, não deste arquivo.
                Span::Count { min, max } => (min as f32, Bound::Hard(max as f32)),
                // Simétrica e fechada pelo documento: as duas pontas são paredes.
                Span::Walls(max) => (-max, Bound::Hard(max)),
                // ⭐ **Positiva OU zero** — o teto é da vista, como a `Positive`, e a diferença toda
                // está no piso: aqui o zero é uma resposta (o cone fechado), não uma recusa.
                Span::FromZero => (0.0, Bound::Soft(view_span)),
                // ⭐⭐ **Parede do documento E zero alcançável** — a faixa dos dois recuos de uma
                // aresta. ⚠️ O mapeamento é o mesmo da `Wall`; o que muda é do outro lado, na porta
                // de escrita, que agora aceita o zero que este slider sempre ofereceu.
                Span::WallFromZero(w) => (0.0, Bound::Hard(w)),
                // ⭐⭐ **PISO do documento, tecto da vista** — a imagem no espelho da `Wall`. Ver o
                // doc dela para o report de 06/09 que a obrigou: sem o piso, o slider oferecia
                // valores que a validação recusa, e um nó recusado apaga a CENA INTEIRA.
                // ⚠️ **O tecto tem de ficar ACIMA do piso** — o alcance da vista é a oitava de
                // `4×` o raio da peça, e o piso é uma medida da própria peça, então em quase toda
                // peça o primeiro já é maior. *Quase* não é uma lei: um slider com `lo ≥ hi` não
                // tem para onde arrastar, e é o mesmo defeito que a `Span::Count` pagou com o piso
                // do prisma. O `2×` é o que faz a faixa ter curso em vez de ser um ponto.
                Span::Floor(f) => (f, Bound::Soft(view_span.max(f * 2.0))),
                // ⭐⭐ **Adimensional: as duas pontas são do DOCUMENTO** e a vista não tem voto —
                // ver [`ph2d_field::Span::Range`]. `Hard`, logo digitar um número de fora clampa.
                Span::Range { min, max } => (min, Bound::Hard(max)),
                // ⭐ **Uma ESCOLHA**: as pontas são a lista, e são do DOCUMENTO — um índice fora
                // dela não é um gesto que a vista possa oferecer. `Hard`, logo digitar clampa.
                Span::Choice(nomes) => (0.0, Bound::Hard(nomes.len().saturating_sub(1) as f32)),
            };
            // ⭐⭐⭐ **O primeiro canal é a ÂNCORA da cor** — a linha que ele publica é a amostra.
            let cor = ancora(param);
            ph2d_panel_model3d::ParamRow {
                entity: e.to_bits(),
                param,
                key: cor.map_or(d.key, |(rotulo, _)| rotulo),
                value: d.value,
                lo,
                bound,
                live: d.span != Span::Locked,
                // ⚠️ **Uma escolha também é inteira** — meio eixo não existe, e sem isto o passo
                // do arrasto e as casas decimais seriam os de um número contínuo.
                integral: matches!(d.span, Span::Count { .. } | Span::Choice(_)),
                section: secao(param),
                choices: match d.span {
                    Span::Choice(nomes) => nomes,
                    _ => &[],
                },
                swatch: cor.map(|(_, canais)| crate::materials::colour_srgb8(canais.map(canal))),
                subject: None,
            }
        })
        .collect();
    material_for_the_selection(world, selection, view_span, &mut linhas);
    linhas
}
