//! ⭐⭐⭐ **Architecture gate — o produto não cita o FONTE de um alvo restrito.**
//!
//! ## Porque isto existe
//!
//! O `SKILL_Cleanroom` §4.2 põe **nomes internos do alvo** (ficheiros, funções,
//! variáveis) na lista curta do que a lei protege, e o
//! [`ACHADO_proveniencia_por_nome_interno`](../../../docs/3D/cleanroom/ACHADO_proveniencia_por_nome_interno.md)
//! registou-o em 2026-08-24 sobre as **notas** do repo. ⚠️ **Em 2026-09-09
//! mediu-se que ele está vivo no CÓDIGO RASTREADO, não só em notas** — e a
//! diferença importa: uma nota fica no repo, mas uma citação dentro de uma
//! *string* **viaja no binário** e no log do CI.
//!
//! ⛔⛔ **Nada no `ship.sh` corria um censo destes** — a regra existia em prosa
//! desde a triagem e não tinha instrumento, que é a definição de nota que
//! envelhece.
//!
//! ## As duas regras, e porque são duas
//!
//! 1. **FORA de comentário: ZERO, sem excepção.** Uma citação numa mensagem de
//!    `assert!`, num `eprintln!` de cena de smoke ou num comentário de fim de
//!    linha de código entra na tabela de strings do binário. É a única
//!    sub-espécie que **sai do repositório sem passar pelo `git`**, e por isso
//!    não tem lista de tolerância nenhuma.
//! 2. **EM comentário: catraca por crate.** Ela está **VAZIA** — a dívida foi de
//!    `351` a `0` —, e por isso qualquer citação nova reprova. Uma entrada que
//!    chegue a zero é **obsoleta e tem de ser apagada** — *uma catraca sem censo
//!    de obsolescência não desce: ela vira licença* (`CLAUDE.md` §5.0).
//!
//! ## O detector, e as SETE maneiras de ele mentir
//!
//! ⛔⛔ **As sete foram achadas a MEDIR o que ele acusava, nunca a pensar** — e
//! quatro delas só apareceram depois de a dívida encolher, porque *uma população
//! grande esconde os erros do instrumento no meio do trabalho legítimo*.
//!
//! 1. **Pedir `:<linha>`** lia `64` ficheiros onde a verdade era muito maior:
//!    metade das citações é o nome **nu** entre crases.
//! 2. **Alargar sem olhar o CONTEXTO mente ao contrário:** `\w+\.h` casa
//!    `rect.h` — a ALTURA de um rectângulo — e acusa `152` sítios inocentes só
//!    na `ph2d-editor-core`. ⇒ o discriminador é a **linha ser um comentário**.
//! 3. **Um CAMPO entre crases numa expressão com espaços** (`` `a.w × a.h` ``).
//! 4. **Um campo nosso que «parecia» cabeçalho** por ter `_` (`list_rect.h`).
//! 5. **`ficheiro.cc::simbolo`** escapava às quatro outras formas — a citação
//!    **mais** específica de todas media zero, em seis sítios.
//! 6. **O HÍFEN cortava o nome** (`bezier-utils.cpp` → `utils.cpp`), logo um
//!    artefacto já triado continuava a contar.
//! 7. **A nossa própria bancada, se mora FORA do repo**, lê-se como alheia — o
//!    discriminador «é da nossa árvore?» só varre o que está dentro.
//!
//! ## ⛔ Uma entrada na catraca NÃO é uma acusação
//!
//! Ela diz *«ninguém classificou isto ainda»*. A triagem é por ARTEFACTO e é
//! papel de um revisor que vê os dois lados: uma citação a alvo **permissivo**
//! (MIT/BSD/…) é atribuição legítima e **fica**, com a licença **lida no
//! artefacto** ao lado. Uma citação a alvo **restrito** sai, e o FACTO que ela
//! carregava fica, re-dito em vocabulário do domínio.
//!
//! Dep-free (só `std`), como os outros gates de arquitectura.

use std::fs;
use std::path::{Path, PathBuf};

/// Extensões de ficheiro-fonte que um alvo restrito desta casa usa.
const EXT: &[&str] = &[
    "cc", "cpp", "cxx", "c", "h", "hh", "hpp", "py", "glsl", "osl", "inl",
];

/// ⭐ **Atribuição LEGÍTIMA a alvo PERMISSIVO** — classificada por um revisor que
/// leu os dois lados (2026-09-09). ⛔ Uma entrada nova aqui exige a licença
/// nomeada: sem isso ela é indistinguível de uma isenção de conveniência.
const ATRIBUICAO_PERMISSIVA: &[(&str, &str)] = &[
    ("ph2d-editor-core/src/paint.rs", "tema de editor MIT"),
    (
        "ph2d-editor-core/src/widget/list_rows/selection.rs",
        "tema de editor MIT",
    ),
    (
        "ph2d-editor-core/tests/it/a_list_is_not_a_form.rs",
        "tema de editor MIT",
    ),
    ("ph2d-tokens/src/spacing.rs", "tema de editor MIT"),
    ("ph2d-tokens/src/slider_style.rs", "tema de editor MIT"),
    ("ph2d-tokens/src/visuals.rs", "tema de editor MIT"),
    (
        "ph2d-quantize/src/refine.rs",
        "biblioteca de quantização MIT",
    ),
];

/// ⭐⭐ **ALVOS PERMISSIVOS, pelo NOME do ficheiro CITADO.**
///
/// ⚠️ **Isto é melhor que isentar um ficheiro NOSSO inteiro**, que é a forma das
/// sete entradas acima: aquela cega o ficheiro para todas as citações, incluindo
/// uma a alvo restrito que entre lá amanhã. Esta isenta exactamente o que foi
/// triado. *A unidade da triagem é o ARTEFACTO citado, não quem o cita.*
const ALVO_PERMISSIVO: &[(&str, &str)] = &[
    // Godot — MIT. A triagem desta casa autoriza LER e PORTAR, e o redesenho
    // plano da UI é derivado do tema «Modern» dele.
    ("theme_modern.cpp", "Godot, MIT"),
    ("editor_theme_manager.cpp", "Godot, MIT"),
    ("editor_dock.h", "Godot, MIT"),
    // Chromium — BSD-3. A resolução da bézier de temporização.
    ("cubic_bezier.cc", "Chromium, BSD-3"),
    // Instant Meshes — BSD-3. É o porte fiel que o `ph2d-quadflow` É, e a
    // atribuição é obrigação da licença, não dívida.
    //
    // ⚠️ **Quatro dos cinco declaram a licença NA PRÓPRIA linha que os cita**, e
    // o quinto (`optimizer.cpp`) é o mesmo artefacto instalado — nenhuma destas
    // foi adivinhada a partir do nome do projecto.
    ("field.cpp", "Instant Meshes, BSD-3"),
    ("extract.cpp", "Instant Meshes, BSD-3"),
    ("hierarchy.cpp", "Instant Meshes, BSD-3"),
    ("cleanup.cpp", "Instant Meshes, BSD-3"),
    ("adjacency.cpp", "Instant Meshes, BSD-3"),
    ("meshstats.cpp", "Instant Meshes, BSD-3"),
    ("optimizer.cpp", "Instant Meshes, BSD-3"),
    // ⭐⭐⭐ **MaterialX — Apache-2.0**, com a licença LIDA NO ARTEFACTO INSTALADO
    // (`pacman -Qo /usr/share/licenses/materialx/LICENSE` → `materialx 1.39.5-1.1`;
    // o ficheiro abre com *«Apache License, Version 2.0»*). ⛔ **Não foi adivinhada
    // pelo nome do projecto** — é a armadilha §0.9 que esta casa mede desde 09/09.
    //
    // A `ph2d-material` **É** o porte destes ficheiros (o OpenPBR Surface como lei
    // de referência em CPU, `docs/Render3d/05`), e a atribuição é **obrigação da
    // licença, não dívida**: a mesma leitura que o `ph2d-quadflow` tem sobre o
    // Instant Meshes, acima.
    //
    // ⚠️ Todos em `/usr/share/materialx/libraries/pbrlib/genglsl/`, e a proveniência
    // ficheiro a ficheiro vive no cabeçalho de `ph2d-material/src/bsdf.rs`.
    ("mx_microfacet.glsl", "MaterialX, Apache-2.0"),
    ("mx_microfacet_specular.glsl", "MaterialX, Apache-2.0"),
    ("mx_microfacet_diffuse.glsl", "MaterialX, Apache-2.0"),
    ("mx_dielectric_bsdf.glsl", "MaterialX, Apache-2.0"),
    ("mx_generalized_schlick_bsdf.glsl", "MaterialX, Apache-2.0"),
    ("mx_oren_nayar_diffuse_bsdf.glsl", "MaterialX, Apache-2.0"),
    ("mx_add_bsdf.glsl", "MaterialX, Apache-2.0"),
    ("mx_layer_bsdf.glsl", "MaterialX, Apache-2.0"),
    ("mx_multiply_bsdf_float.glsl", "MaterialX, Apache-2.0"),
    ("mx_multiply_bsdf_color3.glsl", "MaterialX, Apache-2.0"),
    ("mx_environment_prefilter.glsl", "MaterialX, Apache-2.0"),
    // ⭐ **Os TRÊS da SUBSUPERFÍCIE** (17/09, `docs/Render3d/10`) — a mesma triagem, o mesmo
    // artefacto instalado e o mesmo directório dos onze acima. ⚠️ A nodedef e as duas definições
    // do `stdlib`/`pbrlib` entram pela mesma porta: o `energy_compensation` desta wave é um valor
    // de OMISSÃO lido de um `.mtlx`, e um facto lido de um ficheiro nomeia-o.
    ("mx_translucent_bsdf.glsl", "MaterialX, Apache-2.0"),
    ("mx_subsurface_bsdf.glsl", "MaterialX, Apache-2.0"),
    ("mx_mix_bsdf.glsl", "MaterialX, Apache-2.0"),
    ("open_pbr_surface.mtlx", "MaterialX, Apache-2.0"),
    ("pbrlib_defs.mtlx", "MaterialX, Apache-2.0"),
    ("mx_math.glsl", "MaterialX, Apache-2.0"),
    // ⭐⭐⭐ **rive-runtime — MIT**, e é o BLUEPRINT declarado do módulo vectorial
    // ([ADR-0108](../../../../docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)
    // D2/D5: *«Rive é a fonte da verdade … portados com atribuição MIT»*). A
    // deformação por ossos desta casa é o LBS dele sobre pontos de controlo, e a
    // atribuição é **obrigação da licença, não dívida** — a mesma leitura que o
    // `ph2d-quadflow` tem sobre o Instant Meshes.
    //
    // ⚠️ **A licença foi LIDA NO ARTEFACTO** (o `LICENSE` na raiz de
    // `rive-app/rive-runtime`: *«MIT License · Copyright (c) 2020 Rive»*), e não
    // adivinhada pelo nome do projecto — que é a armadilha §0.9 que esta casa
    // mede desde 09/09. ⛔ **E o nível da prova é declarado:** estes quatro
    // ficheiros **não** têm cabeçalho SPDX próprio (abrem num `#include`), não há
    // `LICENSE` de sub-directório nem `NOTICE`, logo quem os cobre é o da raiz.
    // *Uma prova de raiz é mais fraca que uma de ficheiro, e dizê-lo é o que a
    // torna auditável.*
    //
    // ⚠️⚠️ **O EDITOR do Rive é FECHADO e nada dele entra aqui** — o que foi lido
    // é o *runtime*, que é um player. A distinção é a mesma do `libmypaint`: o
    // nome do projecto não é a unidade da triagem.
    ("weight.cpp", "rive-runtime, MIT"),
    ("skin.cpp", "rive-runtime, MIT"),
    ("vertex.cpp", "rive-runtime, MIT"),
    ("cubic_vertex.cpp", "rive-runtime, MIT"),
    // Graphics Gems — o ajuste de curva canónico, de uso livre.
    ("FitCurves.c", "Graphics Gems"),
    // lib2geom — dual `LGPL-2.1-only OR MPL-1.1`, triado 2026-09-09. ⚠️ O
    // empacotador desta máquina rotula o pacote como «GPL» e o ARTEFACTO
    // desmente-o: o cabeçalho companheiro instalado e a tag SPDX do próprio
    // ficheiro concedem a dupla. *A unidade da triagem é o artefacto instalado,
    // nunca o rótulo do pacote.*
    ("bezier-utils.cpp", "lib2geom, LGPL-2.1-only OR MPL-1.1"),
    // libSatsuma — MIT, com `SPDX-License-Identifier: MIT` na linha 2 de CADA um
    // dos dois. ⚠️⚠️ **É a armadilha do §0.9 ao contrário:** a APLICAÇÃO que os
    // linka (o oráculo de quantização) é GPL-3.0 e a BIBLIOTECA citada é MIT —
    // triar pelo nome do projecto restrito pagaria clean-room por um motor que
    // este repo pode simplesmente ligar.
    ("CostFunction.hh", "libSatsuma, MIT"),
    ("Highlevel.cc", "libSatsuma, MIT"),
    // ⭐⭐ **E estes dois são NOSSOS** — a bancada do PH2D, que vive FORA da
    // árvore do repo (`ph2d-quadbench/`, ao lado do oráculo). O discriminador
    // `nomes_da_nossa_arvore` só varre `crates`/`shells`/`docs`/`scripts`, logo
    // não os alcança: *a nossa própria ferramenta, se morar fora do repo, lê-se
    // como citação de alvo alheio.* A premissa da pergunta que os mandou triar
    // estava errada, e foi o revisor que a desmentiu.
    ("layout.py", "bancada do PH2D, obra própria"),
    ("metrics.py", "bancada do PH2D, obra própria"),
    // O arnês que corre a SciPy (BSD-3) sobre o NOSSO lattice — ver
    // `docs/Skeleton/ferramentas/oraculo_do_campo.py`, escrito nesta casa.
    ("oraculo_do_campo.py", "bancada do PH2D, obra própria"),
];

/// ⚠️ **A entrada que este ficheiro teve ERRADA, e o registo fica:** o
/// `layout.py` estava aqui listado como *«do oráculo, restrito»*, com a nota
/// *«vive fora da árvore»* lida como agravante. O revisor mediu-o: **o upstream
/// não tem ficheiro nenhum com esse nome**, e ele é a nossa própria bancada.
/// *Uma acusação de proveniência afirmada pelo vizinho de um ficheiro é um
/// palpite com cara de triagem* — a licença lê-se no artefacto.
const _: () = ();

/// A catraca: quantas citações **em comentário** cada crate ainda carrega.
///
/// ⭐⭐⭐ **ELA ESTÁ VAZIA, e isso é o fundo da escada.** A dívida foi de `351`
/// (censo de abertura) a `145`, a `103`, a **`0`** — parte curada, parte triada
/// como atribuição legítima a alvo permissivo. Com a lista vazia, **qualquer**
/// citação nova a ficheiro-fonte externo reprova o gate, e a cura é uma de duas:
/// dizer o FACTO em vocabulário do domínio, ou — se o artefacto for permissivo —
/// pô-lo em [`ALVO_PERMISSIVO`] **com a licença lida no próprio artefacto**.
///
/// ⚠️ **Os números só DESCEM.** Uma entrada que meça zero é obsoleta e tem de
/// sair; o gate exige-o, e foi essa metade que apagou as doze que aqui estavam.
const POR_CLASSIFICAR: &[(&str, usize)] = &[];

/// ⚠️⚠️ **O DETECTOR NÃO SE CONTA A SI PRÓPRIO.**
///
/// Este ficheiro carrega, no teste de controlo, exemplos das duas formas que ele
/// procura — é assim que se prova que ele conta o que deve. Sem esta linha ele
/// acusa-se, e a primeira corrida acusou-se mesmo (`3` fora de comentário, `24`
/// contra `15` na catraca).
///
/// ⛔ **É a terceira vez que um instrumento desta casa se casa a si próprio** (as
/// outras duas estão registadas no censo de atestados da `SPEC_cloth_brush`).
/// *Quem escreve um detector com exemplos dentro tem de o excluir da população.*
const O_PROPRIO_DETECTOR: &str = "architecture_no_restricted_source_citations.rs";

/// A raiz da workspace. `CARGO_MANIFEST_DIR` = `crates/ph2d-editor-core`.
fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("raiz da workspace")
        .to_path_buf()
}

/// A linha é um comentário de Rust?
fn e_comentario(l: &str) -> bool {
    let t = l.trim_start();
    t.starts_with("//") || t.starts_with('*')
}

/// Quantas citações de ficheiro-fonte a linha carrega.
///
/// Duas formas: o endereço com linha (`algo.cc:123`) e o nome nu entre crases
/// (`` `algo.cc` ``). ⚠️ A segunda só conta **entre crases** — é o que separa uma
/// citação de um `rect.h` que é a altura de um rectângulo.
fn citacoes_com(l: &str, nossos: &std::collections::BTreeSet<String>) -> usize {
    let mut n = 0;
    for e in EXT {
        let ponto = format!(".{e}");
        for (i, _) in l.match_indices(&ponto) {
            let depois = &l[i + ponto.len()..];
            // `algo.cc:123`
            let com_linha = depois.starts_with(':')
                && depois[1..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit());
            // ⚠️⚠️ **A QUINTA forma, e ela escapava a TODAS as outras quatro:**
            // `ficheiro.cc::simbolo` não é seguido de dígito nem de crase, então
            // nem `com_linha` nem `nu` a viam. Ela é a citação **mais** específica
            // das cinco — carrega o ficheiro *e* o nome interno —, e media zero.
            // ⛔ Ela existia em **seis** sítios, um deles numa crate que este
            // gate dava por curada: *um instrumento que não conhece uma forma
            // devolve zero sobre ela e lê-se como aprovado.*
            let com_simbolo = depois.starts_with("::");
            // `` `algo.cc` `` — a extensão fecha uma crase, e o que está entre as
            // duas é UM token só.
            //
            // ⚠️⚠️ **O «um token só» é o que separa uma citação de um CAMPO.**
            // Sem ele, `` `plane_region.w × plane_region.h` `` conta como citação
            // — e ali o `.h` é a ALTURA de um rectângulo NOSSO, dentro de uma
            // expressão com espaços. É a terceira forma de o detector mentir, e
            // as três foram achadas a MEDIR, nunca a pensar.
            let abre = l[..i].rfind('`');
            let nu = depois.starts_with('`') && abre.is_some_and(|k| !l[k + 1..i].contains(' '));
            // O token entre crases traz um separador de caminho?
            // (`makesdna/DNA_brush_types.h` traz; `list_rect.h` não.)
            let com_caminho = nu && abre.is_some_and(|k| l[k + 1..i].contains('/'));
            if !(com_linha || nu || com_simbolo) {
                continue;
            }
            // O que vem ANTES do ponto tem de parecer um nome de ficheiro.
            // ⚠️⚠️ **O HÍFEN faz parte do nome, e não fazia:** sem ele
            // `bezier-utils.cpp` extraía-se como `utils.cpp`, que não casa com
            // entrada nenhuma das duas tabelas ⇒ um artefacto **já triado como
            // permissivo** continuava a contar como dívida, e um restrito com
            // hífen no nome seria atribuído ao ficheiro errado. *Um extractor
            // que corta o nome a meio não erra só a contagem: erra a
            // IDENTIDADE, que é o que a triagem por artefacto precisa.*
            let antes = &l[..i];
            let ultimo = antes
                .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
                .map_or(antes, |k| &antes[k + 1..]);
            if ultimo.is_empty() || !ultimo.chars().next().is_some_and(char::is_alphabetic) {
                continue;
            }
            // ⚠️⚠️ **`.h` e `.c` COLIDEM com campos comuns** — `rect.h` é a altura
            // de um rectângulo e `region.h` a de uma região, os dois NOSSOS e os
            // dois escritos em crases sem espaço. ⇒ para estas duas extensões um
            // nome só conta se PARECER um cabeçalho: **maiúscula** no nome, ou um
            // **caminho** à frente dele.
            //
            // ⚠️⚠️ **A 1.ª redacção aceitava também o `_`, e isso era um FALSO
            // POSITIVO medido:** `` `list_rect.h` `` é a ALTURA de uma banda de
            // lista NOSSA (`ph2d-panel-asset-browser`), e o censo mandava alguém
            // «curar» uma linha viva. *O custo escrito ao lado desta heurística
            // dizia só a metade que falha para MENOS; ela também falhava para
            // MAIS, e só a população o disse.*
            //
            // ⛔ **O custo que FICA:** um cabeçalho de uma palavra, minúsculo,
            // sem caminho e citado sem número de linha passa despercebido. As
            // duas curas são do lado de quem cita — pôr a linha ou pôr o caminho
            // —, e **toda** citação real desta árvore já faz uma das duas.
            if matches!(*e, "h" | "c")
                && !com_linha
                && !com_simbolo
                && !com_caminho
                && !ultimo.chars().any(|c| c.is_ascii_uppercase())
            {
                continue;
            }
            // ⛔ Um alvo PERMISSIVO triado e' atribuicao legitima.
            if ALVO_PERMISSIVO
                .iter()
                .any(|(n, _)| *n == format!("{ultimo}{ponto}"))
            {
                continue;
            }
            // ⛔ Um ficheiro NOSSO nao e' uma citacao do alvo.
            if nossos.contains(&format!("{ultimo}{ponto}")) {
                continue;
            }
            n += 1;
        }
    }
    n
}

/// ⚠️⚠️ **UM FICHEIRO NOSSO NÃO É UMA CITAÇÃO DO ALVO.**
///
/// O repo tem arneses com extensão de outra linguagem — `blender_sculpt_oracle.py`,
/// `sculptgl_oracle.mjs`, `cook_matcaps.sh` — e apontá-los é **referência interna
/// legítima**, não proveniência de fonte alheio. ⛔ Sem esta metade o censo
/// inflaciona a dívida e manda alguém «curar» um ponteiro para a nossa própria
/// bancada. *Um censo que acusa o vivo manda a cura errada.*
///
/// O discriminador é exacto e não é heurística: o nome existe na nossa árvore?
fn nomes_da_nossa_arvore(raiz: &Path) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for topo in ["crates", "shells", "docs", "scripts"] {
        junta_nomes(&raiz.join(topo), &mut out);
    }
    out
}

fn junta_nomes(dir: &Path, out: &mut std::collections::BTreeSet<String>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if nome != "target" && nome != "oracle" && !nome.starts_with('.') {
                junta_nomes(&p, out);
            }
        } else if !nome.is_empty() {
            out.insert(nome.to_string());
        }
    }
}

/// Todos os `.rs` rastreados de `crates/` e `shells/`.
fn ficheiros(raiz: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for topo in ["crates", "shells"] {
        junta(&raiz.join(topo), &mut out);
    }
    out.sort();
    out
}

fn junta(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            // `target/` de uma worktree e o oráculo fora da árvore não entram.
            let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if nome != "target" && nome != "oracle" && !nome.starts_with('.') {
                junta(&p, out);
            }
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// A crate (ou shell) a que o caminho relativo pertence.
fn dono(rel: &str) -> String {
    rel.split('/').take(2).collect::<Vec<_>>().join("/")
}

fn censo() -> (Vec<String>, std::collections::BTreeMap<String, usize>) {
    let raiz = raiz();
    let isentos: std::collections::BTreeSet<&str> =
        ATRIBUICAO_PERMISSIVA.iter().map(|(p, _)| *p).collect();
    let nossos = nomes_da_nossa_arvore(&raiz);
    let mut fora = Vec::new();
    let mut em_comentario = std::collections::BTreeMap::new();
    for f in ficheiros(&raiz) {
        let rel = f
            .strip_prefix(&raiz)
            .unwrap_or(&f)
            .to_string_lossy()
            .replace('\\', "/");
        // A isenção é escrita relativa a `crates/`.
        if isentos.contains(rel.trim_start_matches("crates/")) {
            continue;
        }
        if rel.ends_with(O_PROPRIO_DETECTOR) {
            continue;
        }
        let Ok(texto) = fs::read_to_string(&f) else {
            continue;
        };
        for (i, l) in texto.lines().enumerate() {
            let n = citacoes_com(l, &nossos);
            if n == 0 {
                continue;
            }
            if e_comentario(l) {
                *em_comentario.entry(dono(&rel)).or_insert(0) += n;
            } else {
                fora.push(format!("{rel}:{}", i + 1));
            }
        }
    }
    (fora, em_comentario)
}

/// ⛔⛔ **REGRA 1 — uma citação FORA de comentário viaja no binário.**
///
/// Mensagem de `assert!`, texto de cena de smoke, comentário de fim de linha de
/// código: as três acabam na tabela de strings do executável, e a do smoke é
/// **impressa ao dono no terminal**. ⇒ zero, e sem lista de tolerância.
#[test]
fn nenhuma_citacao_de_fonte_restrito_viaja_no_binario() {
    let (fora, _) = censo();
    assert!(
        fora.is_empty(),
        "{} citacao(oes) de ficheiro-fonte FORA de comentario -- elas entram no \
         binario e no log do CI:\n  {}\n\nA cura e' dizer o FACTO em vocabulario do \
         dominio (o facto e' livre; a EXPRESSAO e' que nao e').",
        fora.len(),
        fora.join("\n  ")
    );
}

/// ⭐ **REGRA 2 — a catraca por crate, com o censo de obsolescência.**
#[test]
fn as_citacoes_em_comentario_so_descem() {
    let (_, agora) = censo();
    let congelado: std::collections::BTreeMap<&str, usize> =
        POR_CLASSIFICAR.iter().copied().collect();

    let mut cresceu = Vec::new();
    for (crate_, n) in &agora {
        let teto = congelado.get(crate_.as_str()).copied().unwrap_or(0);
        if *n > teto {
            cresceu.push(format!("{crate_}: {n} agora, {teto} congelado"));
        }
    }
    assert!(
        cresceu.is_empty(),
        "a divida de citacoes CRESCEU em {} crate(s):\n  {}\n\nUma citacao nova a um \
         alvo RESTRITO nao entra: escreva o facto em vocabulario do dominio. Se o \
         alvo for PERMISSIVO, a entrada vai para `ATRIBUICAO_PERMISSIVA` com a \
         licenca nomeada.",
        cresceu.len(),
        cresceu.join("\n  ")
    );

    // ⚠️ **A metade que impede a catraca de virar licenca**: uma linha que ja'
    // nao descreve nada tem de sair, senao a divida fica escrita para sempre
    // sobre trabalho ja' pago.
    let obsoletas: Vec<String> = POR_CLASSIFICAR
        .iter()
        .filter(|(c, _)| !agora.contains_key(*c))
        .map(|(c, n)| format!("{c} (congelada em {n}, mede 0 agora)"))
        .collect();
    assert!(
        obsoletas.is_empty(),
        "{} entrada(s) da catraca JA' NAO DESCREVEM NADA -- apague-as:\n  {}",
        obsoletas.len(),
        obsoletas.join("\n  ")
    );
}

/// ⚠️ **O CONTROLO DO DETECTOR** — sem ele, um detector partido devolve zero e
/// lê-se como aprovado.
///
/// ⛔ As duas maneiras de ele mentir, cada uma com um caso: `rect.h` é a ALTURA
/// de um rectângulo e **não** pode contar; `` `algo.cc` `` entre crases e
/// `algo.cc:12` **têm** de contar.
#[test]
fn o_detector_conta_o_que_deve_e_nada_mais() {
    let vazio = std::collections::BTreeSet::new();
    let citacoes = |l: &str| citacoes_com(l, &vazio);
    assert_eq!(citacoes("    let a = rect.h * 2.0;"), 0, "rect.h e' altura");
    assert_eq!(citacoes("    p.c = 1;"), 0, "p.c e' um campo");
    assert_eq!(citacoes("/// o `algo.cc` faz X"), 1, "nome nu entre crases");
    assert_eq!(citacoes("/// ver `algo.cc:1234`"), 1, "endereco com linha");
    assert_eq!(citacoes("// algo.cc:12 e outro.py:7"), 2, "duas na linha");
    assert_eq!(citacoes("/// nada aqui"), 0);
    // ⛔ E o CAMPO dentro de crases com espacos: `.h` ali e' uma ALTURA.
    assert_eq!(
        citacoes("/// a caixa e' `plane_region.w × plane_region.h` em pixels"),
        0,
        "um campo `.h` numa expressao com espacos nao e' um ficheiro"
    );
    assert_eq!(
        citacoes("/// ver `caminho/algo.cc`"),
        1,
        "e um caminho SEM espacos continua a contar -- o controlo do discriminador"
    );
    // ⛔ O QUARTO defeito, achado a medir a populacao: um CAMPO nosso em crases
    // que tem `_` e por isso «parecia» cabecalho.
    assert_eq!(
        citacoes("/// o `visible_h` era `body_h` num e `list_rect.h` no outro"),
        0,
        "`list_rect.h` e' a ALTURA de uma banda NOSSA, nao um cabecalho"
    );
    assert_eq!(
        citacoes("/// os campos de `DNA_brush_types.h`"),
        1,
        "e um cabecalho com MAIUSCULA conta -- o controlo do mesmo discriminador"
    );
    assert_eq!(
        citacoes("/// os campos de `makesdna/DNA_brush_types.h`"),
        1,
        "com caminho tambem"
    );
    assert_eq!(
        citacoes("/// ver `algum/cabecalho.h`"),
        1,
        "um cabecalho minusculo COM caminho conta -- a segunda metade da regra"
    );
    // ⛔ E o caso que so' a arvore responde: um arnes NOSSO nao conta.
    let nossos: std::collections::BTreeSet<String> = ["blender_sculpt_oracle.py".to_string()]
        .into_iter()
        .collect();
    assert_eq!(
        citacoes_com("/// ver `blender_sculpt_oracle.py`", &nossos),
        0,
        "um ficheiro da NOSSA arvore nao e' citacao do alvo"
    );
    assert_eq!(
        citacoes_com("/// ver `blender_sculpt_oracle.py`", &vazio),
        1,
        "e sem a arvore ele contaria -- o controlo do proprio discriminador"
    );
    // ⚠️⚠️ **O CONTROLO DE VACUIDADE TEVE DE MUDAR DE GRANDEZA no dia em que a
    // dívida chegou a zero.** Ele media *«o censo acha pelo menos 10 crates COM
    // citação»* — uma régua calibrada na dívida existir, que passa a reprovar
    // sobre a árvore limpa e cuja «cura» seria apagá-la. A pergunta que ele
    // sempre quis fazer é outra: *a TRAVESSIA anda?* ⇒ conta-se o que ela visita,
    // que é invariante à dívida.
    let visitados = ficheiros(&raiz()).len();
    assert!(
        visitados >= 1000,
        "a travessia visitou so' {visitados} ficheiros .rs -- ela partiu-se, e um \
         censo partido devolve zero e le-se como aprovado"
    );
}

/// ⭐ **A SONDA — onde estão as citações que a catraca ainda tolera.**
///
/// ⚠️ **Ela corre o MESMO detector do gate**, e é por isso que existe: uma lista
/// derivada por `grep` à parte seria uma segunda resposta à mesma pergunta, e a
/// que envelhece é sempre a que quem cura lê.
///
/// ```text
/// cargo test -p ph2d-editor-core --test architecture_no_restricted_source_citations \
///   -- --ignored --nocapture onde_estao
/// ```
#[test]
#[ignore = "sonda"]
fn onde_estao_as_citacoes_que_a_catraca_tolera() {
    let raiz = raiz();
    let isentos: std::collections::BTreeSet<&str> =
        ATRIBUICAO_PERMISSIVA.iter().map(|(p, _)| *p).collect();
    let nossos = nomes_da_nossa_arvore(&raiz);
    let mut por_crate: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for f in ficheiros(&raiz) {
        let rel = f
            .strip_prefix(&raiz)
            .unwrap_or(&f)
            .to_string_lossy()
            .replace('\\', "/");
        if isentos.contains(rel.trim_start_matches("crates/")) || rel.ends_with(O_PROPRIO_DETECTOR)
        {
            continue;
        }
        let Ok(texto) = fs::read_to_string(&f) else {
            continue;
        };
        for (i, l) in texto.lines().enumerate() {
            let n = citacoes_com(l, &nossos);
            if n == 0 {
                continue;
            }
            por_crate.entry(dono(&rel)).or_default().push(format!(
                "{rel}:{}  [{n}]  {}",
                i + 1,
                l.trim()
            ));
        }
    }
    let mut total = 0;
    for (c, linhas) in &por_crate {
        println!("\n== {c} — {} linha(s) ==", linhas.len());
        for l in linhas {
            println!("  {l}");
        }
        total += linhas.len();
    }
    println!("\n(total: {total} linha(s) em {} crates)", por_crate.len());
}
