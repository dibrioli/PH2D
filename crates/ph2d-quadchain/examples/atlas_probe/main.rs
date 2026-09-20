//! ⭐⭐⭐ **A PARAMETRIZAÇÃO VISTA COMO ATLAS** — as cinco perguntas da §9 da avaliação
//! `docs/3D/25_avaliacao_o_painter_na_malha.md`, medidas.
//!
//! ```text
//! cargo run --release -p ph2d-quadchain --example atlas_probe -- <peca.obj|esfera:48> [escala]
//! ```
//!
//! ⛔ **`--release`.** O solver contínuo gasta rondas de Gauss–Seidel aos milhares; em
//! `debug` isto é minutos por peça.
//!
//! # Por que esta sonda existe, e o que ela NÃO é
//!
//! A `ph2d-gridmap` foi construída para **extrair quads**, e a pergunta desta sonda é
//! outra: *ela serve de ATLAS para uma textura?* São grandezas diferentes e nenhuma
//! régua desta casa as media —
//!
//! | a extracção pergunta | um atlas pergunta |
//! |---|---|
//! | as isolinhas inteiras fecham? | o mapa **DOBRA** sobre si mesmo? |
//! | o `χ` sobrevive? | quantas **ILHAS**, e quanta **COSTURA**? |
//! | quantos quads saem? | que **RESOLUÇÃO** a peça pede, e quanto do atlas se desperdiça? |
//!
//! ⭐ **As linhas `ATLAS`, `corte`, `dobra` e `cruza` são a W1 e a W2** (doc 26 §8–§9): o
//! atlas construído, o corte que o torna injectivo, e as duas réguas da sobreposição — a
//! de TEXELS e a de ÁREA ATRIBUÍDA. ⛔ **Sobre `esfera:24` o defeito que a W2 ataca não
//! existe**, e isso está medido: ali `mesma-ilha` lê `0,00 %` (o corte age na mesma, pela
//! classe `dobra`). *Quem quiser ver a wave a trabalhar corre uma peça esculpida.*
//!
//! ⚠️⚠️ **E a pergunta que decide a arquitectura inteira é a primeira linha da tabela:**
//! a cadeia do botão REMALHA antes de parametrizar (F1), e uma textura tem de viver na
//! malha que o artista esculpiu. *Se a parametrização só funcionar depois do F1, o
//! caminho da textura destrói o trabalho dele* — por isso cada peça é medida **duas
//! vezes**, CRUA e remalhada, lado a lado.
//!
//! ⛔⛔ **Ela NÃO corre a [`ph2d_quadchain::quads_from_mesh_raw`]**, e a razão é que ela
//! mede um caminho que aquele não tem: um atlas precisa do mapa **CONTÍNUO** (G3) e o
//! botão precisa dele **INTEIRO** (G3+G5, que a `ChainTiming` soma numa coluna só).
//! *Arredondar a grade a inteiros é trabalho da extracção, nunca de uma textura.* As
//! fases partilhadas são as mesmas funções com os mesmos argumentos, e o número de
//! G3+G5 continua a ler-se no `chain_time`, que corre a porta do produto.

mod desenhos;
mod medidas;

use desenhos::{desenha, desenha_densidade, desenha_na_peca};
use medidas::{
    as_f64, cantos_duplicados, chain_len, densidade, fronteira_das_pecas, sobreposicao, uv_area2,
    vao_entre_ilhas,
};
use ph2d_gridmap::{CutMesh, GridMap};
use ph2d_mesh::Mesh;

/// Uma peça: um caminho de `.obj`, ou uma forma da casa.
fn load(name: &str) -> Mesh {
    if let Some(rest) = name.strip_prefix("esfera:") {
        let n: usize = rest.parse().unwrap_or(48);
        return ph2d_mesh::shapes::uv_sphere(n, n * 3 / 2, 1.0);
    }
    let text = std::fs::read_to_string(name).unwrap_or_else(|e| panic!("{name}: {e}"));
    ph2d_mesh::import_obj(&text)
        .unwrap_or_else(|e| panic!("{name} nao e' um OBJ deste leitor: {e:?}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{name} nao tem peca dentro"))
        .mesh
}

/// ⭐ O que a sonda mede de um par `(malha, mapa)`.
#[derive(Default)]
struct Atlas {
    /// Ilhas: um patch é uma carta.
    patches: usize,
    /// Quantas delas saíram discos (`χ = 1`) — o único resultado bom.
    discs: usize,
    /// Costuras interiores.
    seams: usize,
    /// O comprimento somado delas, em unidades de mundo.
    seam_len: f64,
    /// ⛔ Triângulos cuja área em `(u, v)` tem o sinal da MINORIA do patch: o mapa
    /// **dobrou** ali, e um atlas dobrado pinta dois sítios da peça no mesmo texel.
    folds: usize,
    /// Quantos triângulos entraram na conta.
    tris: usize,
    /// ⛔⛔ Quantos ficaram de FORA por o patch não ter `(u, v)` naquele local.
    ///
    /// ⚠️ **Sem esta coluna a sonda mente com a cara de um resultado perfeito:** a 1.ª
    /// corrida sobre a malha CRUA imprimiu `dobras 0/0 (0,00 %)`, que se lê como *«o
    /// mapa não dobra em lado nenhum»* e queria dizer *«nada foi medido»*. É a lei que
    /// esta casa já pagou nas duas réguas de valência do quad remesh — *um zero de «não
    /// medido» e um de «perfeito» são o mesmo byte*.
    sem_uv: usize,
    /// A área somada das cartas em `(u, v)`, em unidades de grade ao quadrado.
    uv_area: f64,
    /// A área somada das CAIXAS que envolvem cada carta — o que um empacotador de
    /// rectângulos teria de arrumar. ⚠️ É um **limite superior optimista** do
    /// aproveitamento: ele mede o desperdício DENTRO da caixa, e um empacotador
    /// desperdiça mais ainda entre elas.
    box_area: f64,
}

fn medir_atlas(mesh: &Mesh, cut: &CutMesh, map: &GridMap) -> Atlas {
    let pos = mesh.positions();
    let mut a = Atlas {
        patches: cut.origin.len(),
        seams: cut.seams.len(),
        seam_len: cut.seams.iter().map(|s| chain_len(pos, &s.chain)).sum(),
        ..Atlas::default()
    };
    for (p, tris) in cut.tris.iter().enumerate() {
        let Some(uv) = map.uv.get(p) else { continue };
        let (mut pos_n, mut neg_n) = (0usize, 0usize);
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        let mut soma = 0.0f64;
        for t in tris {
            // ⚠️ Um triângulo que aponte para um local que o patch não tem é um defeito
            // do CORTE e não do mapa — saltá-lo evita misturar dois achados.
            let (Some(&p0), Some(&p1), Some(&p2)) = (
                uv.get(t[0] as usize),
                uv.get(t[1] as usize),
                uv.get(t[2] as usize),
            ) else {
                a.sem_uv += 1;
                continue;
            };
            let s = uv_area2(p0, p1, p2);
            if s > 0.0 {
                pos_n += 1;
            } else if s < 0.0 {
                neg_n += 1;
            }
            soma += s.abs();
            for q in [p0, p1, p2] {
                lo[0] = lo[0].min(q[0]);
                lo[1] = lo[1].min(q[1]);
                hi[0] = hi[0].max(q[0]);
                hi[1] = hi[1].max(q[1]);
            }
        }
        a.tris += pos_n + neg_n;
        a.folds += pos_n.min(neg_n);
        a.uv_area += soma;
        if lo[0] <= hi[0] {
            a.box_area += f64::from(hi[0] - lo[0]) * f64::from(hi[1] - lo[1]);
        }
    }
    a
}

/// ⭐⭐⭐ **QUANTAS ILHAS UM ATLAS DE FACTO TERIA** — e não quantos patches o traçado fez.
///
/// ⛔⛔ *Um patch NÃO é uma ilha.* O mapa soldado acopla os dois lados de cada costura, e
/// onde o salto de período é `0 (mod 4)` **os dois lados leem a mesma função**: ali não há
/// corte nenhum, há uma linha que só existe porque o traçado partiu a peça em quads. Uma
/// ilha de textura só nasce onde a transição **RODA** — aí as duas cartas não cabem no
/// mesmo plano.
///
/// ⚠️ Devolve `(ilhas, coladas, rodadas, soltas, corte_len)` — e o último é **o
/// comprimento que um pintor SENTE**: só as costuras que rodam são cortes de verdade, e
/// somar todas conta linhas que não existem no atlas.
///
/// ⛔⛔ **O resíduo da costura NÃO se mede aqui, e a 1.ª redacção media-o — mal, duas
/// vezes.** A casa já tem [`ph2d_gridmap::seam_residual`], e a convenção dela é
/// `zb − turn2(za, jump) − shift`: eu rodei o lado ERRADO e depois esqueci a translação,
/// e li `2,17e1` e `3,84e1` células de grade sobre um solder que solda. *Uma terceira
/// cópia de uma lei é onde ela diverge* — hoje o número vem da porta que o produto usa.
fn ilhas_de_facto(
    pos: &[[f32; 3]],
    cut: &CutMesh,
    jumps: &[Option<i32>],
) -> (usize, usize, usize, usize, f64) {
    let n = cut.origin.len();
    let mut pai: Vec<usize> = (0..n).collect();
    fn raiz(pai: &mut [usize], mut x: usize) -> usize {
        while pai[x] != x {
            pai[x] = pai[pai[x]];
            x = pai[x];
        }
        x
    }
    let (mut coladas, mut rodadas, mut soltas) = (0usize, 0usize, 0usize);
    let mut corte_len = 0.0f64;
    for (s, seam) in cut.seams.iter().enumerate() {
        let Some(Some(j)) = jumps.get(s).copied() else {
            soltas += 1;
            corte_len += chain_len(pos, &seam.chain);
            continue;
        };
        let (pa, pb) = (seam.side[0].patch as usize, seam.side[1].patch as usize);
        if j.rem_euclid(4) == 0 {
            coladas += 1;
            let (ra, rb) = (raiz(&mut pai, pa), raiz(&mut pai, pb));
            if ra != rb {
                pai[ra] = rb;
            }
        } else {
            rodadas += 1;
            corte_len += chain_len(pos, &seam.chain);
        }
    }
    let ilhas = (0..n).filter(|&p| raiz(&mut pai, p) == p).count();
    (ilhas, coladas, rodadas, soltas, corte_len)
}

/// Corre F2 → G3 sobre uma malha e imprime o bloco dela.
fn corrida(rotulo: &str, mesh: &Mesh, alvo: f32, marca: &str) {
    let mut clock = std::time::Instant::now();
    let mut lap = || {
        let ms = clock.elapsed().as_secs_f64() * 1000.0;
        clock = std::time::Instant::now();
        ms
    };

    let dual = ph2d_crossfield::Dual::build(mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let ms_campo = lap();
    let layout = ph2d_trace::trace_patches(mesh, &dual, &field);
    let ms_trace = lap();
    let (cut, cutrep) = ph2d_gridmap::cut_along_patches(mesh, &layout);
    let (combed, _) = ph2d_gridmap::comb_patches(mesh, &layout, &cut);
    let ms_corte = lap();
    // ⛔⛔ **A PRIMEIRA REDACÇÃO DESTA SONDA CHAMOU `ph2d_gridmap::solve`, e era o motor
    // ERRADO.** Aquele é o G3 com a costura **PENALIZADA** (`SEAM_WEIGHT`), que a obra A
    // de 24/08 substituiu pela costura **ELIMINADA**; o produto corre o soldado desde
    // então. Medido na esfera de 24: o penalizado deu `10,76 %` de triângulos dobrados e
    // `5 784 ms`, o soldado dá o que a tabela abaixo imprime — e a diferença de relógio é
    // a das rondas (`160 000` contra `8 000`, `20×`, que o doc do `welded_rounds` já
    // escrevia). *Escolher a função pelo nome mais óbvio mede um programa que o produto
    // deixou de correr.*
    let rondas = ph2d_gridmap::RoundOptions::default().welded_rounds;
    let (map, wrel) = ph2d_gridmap::solve_welded(
        mesh,
        &cut,
        &combed,
        ph2d_gridmap::Step::uniform(alvo),
        rondas,
    );
    let rel = wrel.solve;
    let ms_mapa = lap();

    let mut a = medir_atlas(mesh, &cut, &map);
    a.discs = cutrep.discs;
    let area = f64::from(mesh.surface_area());
    let total = ms_campo + ms_trace + ms_corte + ms_mapa;

    println!(
        "\n-- {rotulo}: {} V | {} F | area {area:.4}",
        mesh.vert_count(),
        mesh.face_count()
    );
    println!(
        "   relogio  campo {ms_campo:8.1} | tracado {ms_trace:7.1} | corte {ms_corte:7.1} | \
         mapa-continuo {ms_mapa:9.1}   TOTAL {total:9.1} ms"
    );
    println!(
        "   ilhas    {} patches ({} discos) | {} costuras | fronteira {:.3} de mundo = {:.2}x sqrt(area)",
        a.patches,
        a.discs,
        a.seams,
        a.seam_len,
        a.seam_len / area.sqrt().max(1.0e-12)
    );
    let pct_fold = if a.tris == 0 {
        0.0
    } else {
        100.0 * as_f64(a.folds) / as_f64(a.tris)
    };
    let jumps = ph2d_gridmap::jumps_only(mesh, &cut, &combed);
    let (ilhas, coladas, rodadas, soltas, corte_len) =
        ilhas_de_facto(mesh.positions(), &cut, &jumps);
    // ⭐ O resíduo vem da PORTA da casa — ver a nota em [`ilhas_de_facto`].
    let (w, _) = ph2d_gridmap::weld(&cut, &combed);
    let sr = ph2d_gridmap::seam_residual(&w, &map);
    println!(
        "   ILHAS    {ilhas} de facto (de {} patches) | costuras {coladas} coladas + {rodadas} rodadas + {soltas} soltas | \
         CORTE que o pintor sente {corte_len:.3} = {:.2}x sqrt(area) ({:.0}% da fronteira)",
        a.patches,
        corte_len / area.sqrt().max(1.0e-12),
        100.0 * corte_len / a.seam_len.max(1.0e-12)
    );
    println!(
        "   costura  {} elos eliminados: p50 {:.2e} max {:.2e} | {} fechos: max {:.2e}",
        sr.links, sr.p50, sr.max, sr.closures, sr.closure_max
    );
    println!(
        "   solver   {} triangulos na energia | {} saltados | {} pares de costura | {} costuras soltas",
        rel.triangles, rel.skipped, rel.pairs, rel.loose_seams
    );
    // ⭐ DUAS medidas da MESMA grandeza, de propósito: a minha (sinal da área UV, por
    // patch) e a do solver (`folded_after`). *Quando uma página imprime duas medidas da
    // mesma coisa e elas discordam, isso É o achado* — a lei que a régua do `reach`
    // pagou em 31/08.
    println!(
        "   dobras   {}/{} triangulos com a area UV invertida ({pct_fold:.2}%) | {} SEM (u,v) | \
         o solver conta {} antes e {} depois",
        a.folds, a.tris, a.sem_uv, wrel.folded_before, wrel.folded_after
    );
    // ⛔ O piso de população: sem ele um `0/0` lê-se como um mapa impecável.
    assert!(
        a.tris > 0 || a.sem_uv > 0,
        "nem um triangulo chegou a ser medido em {rotulo} — a sonda nao mediu nada"
    );
    let fill = if a.box_area <= 0.0 {
        0.0
    } else {
        100.0 * a.uv_area / a.box_area
    };
    println!(
        "   atlas    caixas somadas {:.1} | cartas {:.1} => aproveitamento DENTRO da caixa {fill:.1}%",
        a.box_area, a.uv_area
    );
    // ⚠️ A área em `(u, v)` vem em unidades de GRADE, e uma unidade de grade vale `alvo`
    // de mundo — sem essa conversão os dois números não são comparáveis.
    let uv_em_mundo = a.uv_area * f64::from(alvo) * f64::from(alvo);
    println!(
        "   escala   area em UV {uv_em_mundo:.4} de mundo contra {area:.4} de superficie = {:.3}x",
        uv_em_mundo / area.max(1.0e-12)
    );
    // ⭐⭐⭐ **O ATLAS DE VERDADE** — as ilhas juntas, assentes e arrumadas em `[0,1]²`.
    // Tudo acima é a matéria-prima; isto é o que uma textura recebe.
    let relogio = std::time::Instant::now();
    // ⭐ A bissecção vive AQUI, na sonda, e não dentro da crate: uma env lida na
    // biblioteca alcançaria todo chamador e faria um gate medir a máquina.
    //
    // ⛔⛔ **E ela cai no valor de FÁBRICA quando ninguém pede.** A 1.ª redacção escrevia
    // `env(...) != Ok("0")`, que com a variável por definir devolve **`true`** — logo a
    // sonda armava a colagem por conta própria e **ignorava o valor de fábrica**. No dia
    // em que ele mudou, a sonda continuou a medir o programa antigo e a tabela saiu com
    // os números de antes. *Uma porta de bissecção que não lê o default mede outro
    // programa que o produto.*
    let fabrica = ph2d_uv_atlas::Opcoes::default();
    let porta = |chave: &str, padrao: bool| match std::env::var(chave).as_deref() {
        Ok("0") => false,
        Ok(_) => true,
        Err(_) => padrao,
    };
    let opcoes = ph2d_uv_atlas::Opcoes {
        orientar: porta("PH2D_ATLAS_ORIENTA", fabrica.orientar),
        colar: porta("PH2D_ATLAS_COLA", fabrica.colar),
        densidade_igual: porta("PH2D_ATLAS_DENSIDADE", fabrica.densidade_igual),
        ..fabrica
    };
    let atl = ph2d_uv_atlas::build_com(mesh, &cut, &map, &jumps, opcoes);
    let ms_atlas = relogio.elapsed().as_secs_f64() * 1000.0;
    let r = atl.relatorio;
    println!(
        "   ATLAS    {} ilhas | {} cantos, {} orfaos | {} cortes que o atlas obrigou ({}) | \
         aproveitamento {:.1}% | {ms_atlas:.1} ms",
        r.ilhas,
        r.cantos,
        r.orfaos,
        r.ciclos,
        // ⛔ Sem colagem estas duas colunas NAO FORAM MEDIDAS, e imprimi-las como `0,00`
        // lê-se como «assentou perfeito». O piso de população é a contagem de coladas.
        if r.coladas == 0 {
            "rasgo e cola: sem colagem, nao medidos".to_string()
        } else {
            format!(
                "rasgo {:.2e} | cola_max {:.2e}",
                r.holonomia_max, r.cola_max
            )
        },
        100.0 * f64::from(r.aproveitamento)
    );
    println!(
        "   corte    {} ilhas da superficie => {} pecas ({} de uma face so', {} sem vizinho) | \
         {} recusas, {} fusoes | peca p50 {} max {}",
        r.ilhas_antes_do_corte,
        r.ilhas,
        r.pecas_de_uma_face,
        r.faces_sem_vizinho,
        r.recusas_do_corte,
        r.fusoes_do_corte,
        r.peca_p50,
        r.peca_max
    );
    let (cobertos, dobrados) = sobreposicao(&atl, mesh, 1024);
    println!(
        "   dobra    {dobrados} de {cobertos} texels de 1024^2 pintados MAIS DE UMA VEZ ({:.2}%)",
        if cobertos == 0 {
            0.0
        } else {
            100.0 * as_f64(dobrados) / as_f64(cobertos)
        }
    );
    // ⛔⛔⛔ **A COLUNA QUE O OLHO LÊ, e que o `aproveitamento` NÃO era.**
    //
    // Até a W3 o relatório publicava `caixas / quadrado` — a fracção do quadrado que as
    // CAIXAS ocupam — e ela lia `67,7 %` numa peça em que a tinta ocupava `18,7 %`. *Uma
    // régua que mede o invólucro não mede o que está lá dentro.*
    //
    // ⭐ E as duas contas da TINTA vêm por caminhos diferentes de propósito: a do
    // relatório é a soma das ÁREAS dos triângulos e esta é a contagem de TEXELS pintados.
    // *Se elas discordassem, uma das duas estaria a medir outro atlas.*
    let tinta_texel = as_f64(cobertos) / (1024.0 * 1024.0);
    // ⛔⛔ **A coluna «% do desperdício DENTRO das caixas» SAIU, e a premissa dela morreu
    // na W3.** Ela dividia por `1 − tinta` supondo que as caixas ocupam uma fracção do
    // quadrado; com o empacotador por máscara **elas sobrepõem-se de propósito** e
    // `caixas/quadrado` passou a ler `107,2 %` numa peça, o que fazia a derivada imprimir
    // `110 %`. *Uma razão cuja premissa é «as partes não se cruzam» deixa de ser uma
    // fracção no dia em que elas se cruzam* — e o número que ela queria dar continua à
    // vista, que é a `tinta DENTRO da caixa`.
    println!(
        "   espaco   TINTA/quadrado {:.1}% (por texel {:.1}%) | caixas/quadrado {:.1}% | \
         tinta DENTRO da caixa {:.1}%",
        100.0 * f64::from(r.aproveitamento),
        100.0 * tinta_texel,
        100.0 * f64::from(r.caixas_no_quadrado),
        100.0 * f64::from(r.aproveitamento / r.caixas_no_quadrado.max(1.0e-12))
    );
    // ⭐⭐⭐ **A DENSIDADE, e a ATRIBUIÇÃO dela.** Ver [`densidade`]. Sem esta linha,
    // «o atlas está bom» é uma afirmação sobre o quadrado e não sobre a PEÇA.
    if let Some(d) = densidade(&atl, mesh) {
        println!(
            "   densidade {:.1} texels/mundo a 2048^2 | espalhamento {:.2}x..{:.2}x da mediana",
            d.p50 * 2048.0,
            d.global[0],
            d.global[1]
        );
        println!(
            "   atribui  ENTRE ilhas {:.2}x..{:.2}x | DENTRO das ilhas {:.2}x..{:.2}x",
            d.entre[0], d.entre[1], d.dentro[0], d.dentro[1]
        );
    }
    println!(
        "   escala   a igualacao teve de multiplicar as pecas por {:.2}x..{:.2}x",
        r.escala_das_pecas[0], r.escala_das_pecas[1]
    );
    // ⭐⭐ A costura contada em VÉRTICES. Ver [`cantos_duplicados`].
    let (distintos, verts) = cantos_duplicados(&atl, mesh);
    println!(
        "   placa    {distintos} vertices distintos (u,v) para {verts} da malha = +{:.1}% \
         para a placa",
        100.0 * (as_f64(distintos) / as_f64(verts.max(1)) - 1.0)
    );
    // ⭐⭐⭐ **O CONTRAPESO da tinta: a COSTURA que o atlas obriga, medida no MUNDO.**
    //
    // Toda fronteira entre duas peças é um sítio onde uma pincelada que a atravessa pode
    // mostrar um fio. ⛔ Sem esta coluna, «não colar dá mais tinta» é meia medição — *a
    // outra metade é quanto isso custa ao pintor*.
    println!(
        "   costura  as pecas fazem {:.3} de fronteira = {:.2}x sqrt(area) da peca",
        fronteira_das_pecas(&atl, mesh),
        fronteira_das_pecas(&atl, mesh) / area.sqrt().max(1.0e-12)
    );
    // ⭐⭐⭐ **A ATRIBUIÇÃO** — a linha de cima diz QUANTO e esta diz DE QUEM. Sem ela a
    // wave do corte começaria por adivinhar o mecanismo.
    let sob = ph2d_uv_atlas::sobreposicao::medir(mesh, &atl);
    assert!(
        sob.triangulos_com_area > 0,
        "nem um triangulo do atlas tem area em {rotulo} — a regua da sobreposicao nao mediu nada"
    );
    print!(
        "   cruza    {:.3}% da area pintada, {} de {} triangulos tocados |",
        100.0 * sob.fraccao(),
        sob.triangulos_tocados,
        sob.triangulos_com_area
    );
    for c in ph2d_uv_atlas::sobreposicao::Classe::ALL {
        print!(
            " {}={} ({:.2}%)",
            c.nome(),
            sob.pares_por_classe[c.indice()],
            100.0 * sob.area_por_classe[c.indice()] / sob.area_pintada.max(1.0e-12)
        );
    }
    println!();
    // ⭐⭐⭐ **O VÃO que a constante PROMETE, medido em texels.**
    //
    // ⛔ `VAO_EM_TEXELS` diz `8` a `2048²`, e até aqui ninguém tinha verificado que eles
    // lá estão: o empacotador garante UMA célula de `256`, e que isso dê `8` texels é
    // aritmética que ninguém correu sobre o atlas de verdade. *Uma constante que promete
    // um número e um atlas que ninguém mediu são duas coisas diferentes.*
    if let Some((vao, a, b)) = vao_entre_ilhas(&atl, mesh, 2048) {
        println!(
            "   vao      o menor vao entre duas ilhas mede {vao} texels de 2048^2 \
             (a constante pede {:.0}) — entre a {a} e a {b}",
            ph2d_uv_atlas::VAO_EM_TEXELS
        );
    }
    // ⭐ O PIOR par que sobra, com a área e a classe. ⚠️ Sem esta linha um `1` na coluna
    // de uma classe lê-se igual a um `1000`: *uma contagem sem magnitude não diz se o que
    // sobrou cabe num texel ou numa peça*.
    if let Some(pior) = sob.pares.first() {
        println!(
            "   pior     {} de area {:.3e} ({:.2e} da area pintada) entre os triangulos {} e {}",
            pior.classe.nome(),
            pior.area,
            pior.area / sob.area_pintada.max(1.0e-12),
            pior.a,
            pior.b
        );
    }
    // ⭐ O atlas SEM corte, pela mesma porta — é o CONTROLO das waves 2 e 3: sem ele «o
    // corte melhorou» é uma afirmação sobre uma imagem que ninguém pôs ao lado da outra.
    let cru = ph2d_uv_atlas::build_com(
        mesh,
        &cut,
        &map,
        &jumps,
        ph2d_uv_atlas::Opcoes {
            cortar: false,
            orientar: false,
            ..ph2d_uv_atlas::Opcoes::default()
        },
    );
    // ⭐⭐⭐ **A ATRIBUIÇÃO DA FORMA: quem deixa as ilhas esguias?**
    //
    // A coluna `tinta DENTRO da caixa` é uma propriedade das FORMAS e não da arrumação.
    // Lida nos dois lados ela responde à pergunta que decide a wave seguinte: *se a
    // parametrização já entrega ilhas esguias, a cura é a montante; se elas só ficam
    // esguias DEPOIS do corte, a cura é o corte.* ⛔ Sem as duas leituras, «as ilhas são
    // esguias» é uma observação sem culpado.
    let dentro = |r: &ph2d_uv_atlas::Relatorio| {
        100.0 * f64::from(r.aproveitamento / r.caixas_no_quadrado.max(1.0e-12))
    };
    println!(
        "   forma    tinta DENTRO da caixa: {:.1}% em {} ilhas ANTES do corte contra {:.1}% \
         em {} pecas DEPOIS",
        dentro(&cru.relatorio),
        cru.relatorio.ilhas,
        dentro(&r),
        r.ilhas
    );
    if let Ok(dir) = std::env::var("PH2D_ATLAS_DUMP") {
        desenha(&atl, mesh, &format!("{dir}/atlas_{marca}.ppm"), 1024);
        desenha(
            &cru,
            mesh,
            &format!("{dir}/atlas_{marca}_sem_corte.ppm"),
            1024,
        );
        // ⭐⭐⭐ O par que o dono julga: a MESMA peça com e sem a igualação de densidade.
        // ⛔ Sem o lado de fora, «a resolução é uniforme» é uma imagem cinzenta que
        // ninguém pôs ao lado de outra.
        desenha_densidade(&atl, mesh, &format!("{dir}/densidade_{marca}.ppm"), 1024);
        let sem = ph2d_uv_atlas::build_com(
            mesh,
            &cut,
            &map,
            &jumps,
            ph2d_uv_atlas::Opcoes {
                densidade_igual: false,
                ..opcoes
            },
        );
        desenha_densidade(
            &sem,
            mesh,
            &format!("{dir}/densidade_{marca}_sem_igualar.ppm"),
            1024,
        );
        // ⭐⭐⭐ E o par que DECIDE: a mesma coisa pesada pela SUPERFÍCIE. Ver
        // [`desenha_na_peca`] — é ela que responde *«onde na minha escultura?»*.
        desenha_na_peca(&atl, mesh, &format!("{dir}/peca_{marca}.ppm"), 1024);
        desenha_na_peca(
            &sem,
            mesh,
            &format!("{dir}/peca_{marca}_sem_igualar.ppm"),
            1024,
        );
    }
    // ⭐ A resolução que a peça pede: com o atlas aproveitado a `fill`, quanto mede um texel.
    for n in [1024u32, 2048, 4096] {
        let uteis = f64::from(n) * f64::from(n) * (fill / 100.0);
        let texel = (area / uteis.max(1.0)).sqrt();
        println!(
            "   {n}^2     um texel mede {texel:.5} de mundo = 1/{:.1} do quad pedido ({alvo:.5})",
            f64::from(alvo) / texel.max(1.0e-12)
        );
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| String::from("esfera:48"));
    let escala: f32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let mut crua = load(&name);
    crua.triangulate();
    let alvo = ph2d_remesh_iso::target_edge(&crua, ph2d_remesh_iso::ALPHA) * escala;
    println!("peca {name} | alvo de aresta {alvo:.5}");

    // ⭐⭐⭐ **A PERGUNTA QUE DECIDE A ARQUITECTURA:** a mesma medição nas duas entradas.
    corrida("CRUA (a malha do artista)", &crua, alvo, "crua");
    // ⛔⛔⛔ **O RÓTULO DESTA LINHA DIZIA «como o botao faz» E ERA FALSO** (report do dono,
    // 2026-09-20: *«ficou assim. com uma retopologia ruim»*). A [`ph2d_quadchain::phase_zero`]
    // é o remalhador ISOTRÓPICO de triângulos — a malha de TRABALHO que a cadeia mastiga
    // antes de parametrizar —, e o que o botão entrega é a saída da EXTRACÇÃO, uma malha de
    // QUADS. ⚠️ E as duas diferenças somam-se: o alvo daqui é o `ALPHA` da cadeia e o do
    // botão sai do SLIDER, e o botão corre **duas ou três** tentativas que uma medição
    // escolhe. *O doc do [`photo_button`] desta casa já escrevia esta lei — «duas ordens
    // diferentes com o mesmo nome dão dois números, e o que o artista vê é o da que ele
    // carrega» — e eu pu-la a mentir num rótulo.*
    //
    // ⭐ A malha do BOTÃO mede-se por este mesmo caminho, e sem uma linha nova: a sonda do
    // produto escreve-a (`PH2D_DUMP=<ficheiro>` em `the_artists_piece_through_the_button`) e
    // ela entra aqui como a peça de entrada.
    let f1 = ph2d_quadchain::phase_zero(&crua, alvo);
    corrida(
        "F1 (a malha de TRABALHO da cadeia — NAO e' o que o botao entrega)",
        &f1,
        alvo,
        "f1",
    );
}
