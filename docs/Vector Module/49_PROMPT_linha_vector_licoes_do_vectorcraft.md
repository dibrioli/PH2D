# 49 — PROMPT para o agente da linha Vector: o que o VectorCraft ensinou (2026-10-11)

> **Origem:** o estudo [`Estudos/ESTUDO_vectorcraft_comparado_2026-10-11.md`](Estudos/ESTUDO_vectorcraft_comparado_2026-10-11.md)
> — leitura dos dois códigos, o VectorCraft corrido por CLI e a prova lado a lado do Offset na estrela do
> gate. Cole este ficheiro inteiro numa janela nova (ou chame
> `/pd-linha-abrir Vector "<objectivo abaixo>" "<leituras abaixo>"`).

---

Abra a linha `line/Vector` em Modo L: execute o BLOCO de `docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md`
(FASE 1 inteira) e as regras A–K, que valem até ao fim da sessão. ⚠️ A branch `line/Vector` **já existe** e a
worktree `Worktrees/line-Vector` está hoje no ramo `integ/Vector-2026-10-07`, **já contido no `main`** — siga a
rota «linha reaberta» do passo 4 e a FASE 1 do `MODELO_TROCA_DE_AGENTE_NA_LINHA.md` (o `git cherry` antes do
rebase). ANTES de ler qualquer ficheiro: `cd` na worktree, `pwd`, `git branch --show-current`.

## Objectivo da linha

Fechar no motor `ph2d-vec-*` os defeitos e lacunas que o VectorCraft (storytold, **MIT OU Apache-2.0**, Rust,
kurbo + linesweeper 0.5 + usvg) expôs, **medidos e não supostos**:

1. o **Offset** (Expand) que faz a forma SUMIR em distâncias normais do slider;
2. o **texto sem kerning** (doc 42, prioridade 6);
3. o VectorCraft como **oráculo corrido por script** das nossas booleanas e do Offset (CLAUDE.md §0.9).

O VectorCraft está instalado: código em `~/Documentos/Recursos/Craft/vectorcraft` (commit `fe8af48f`), CLI em
`~/Apps/vectorcraft/squashfs-root/usr/bin/vectorcraft-cli` (`run --in F --cmd ID --params JSON --export F`,
`commands` lista os 732). A licença permite ler e portar **com atribuição** (cabeçalho com origem, commit e
licença em cada função portada) — mas o caminho primeiro é CORRÊ-LO e comparar, não copiá-lo. Onde ele erra
(Bevel para fora com `d` grande sai estilhaçado), a verdade é a analítica, não ele.

## Leia primeiro (nesta ordem; só isto, depois o código)

1. `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` + DIRETRIZ §1.5.
2. O estudo inteiro: `docs/Vector Module/Estudos/ESTUDO_vectorcraft_comparado_2026-10-11.md` (§3 é a tabela que
   esta linha tem de mudar) e os scripts (hoje em `ferramentas/vectorcraft_2026-10-11/`).
3. `crates/ph2d-vec-boolean/Cargo.toml` (o comentário da subida 0.3 → 0.4: a multiplicidade PAR em `Region::of`)
   e `crates/ph2d-vec-boolean/src/expand.rs:415-470` (`offset_path`, o `catch_unwind`, a tabela 0/83/118).
4. `docs/Vector Module/42_o_que_falta_ao_vetor.md` (linha 97 e a prioridade 6), `crates/ph2d-vec-text/src/glyph.rs:278-300`, `13_fora_de_escopo.md` §13.9,
   `BUGS_vector.md` (#12, #28, #30).

## O «antes» já medido (merge-base de 11/10, sonda release fora do workspace)

Estrela do gate `the_offset_never_takes_the_app_down.rs` (×100 ⇒ R = 140, r = 63), 100 distâncias `4..400` por
junção. Verdade para fora = `fill ∪ stroke(2d, mesma junção, miterlimit 4)` no resvg.

| Junção, lado | PH2D hoje (linesweeper 0.4) | PH2D com 0.5 (só o `Cargo.toml`) | VectorCraft |
|---|---|---|---|
| Round, fora | **87/100** (vazio em 76, 124–136, 160–200, 240, 244) | 100/100 | 100/100 |
| Bevel, fora | 35/100 | 40/100 (vazio de 164 em diante) | 100/100, estilhaçado ≥ 172 |
| Bevel, dentro | **vazio em 44–52** | igual | responde |
| Pânicos capturados | sim (`linesweeper-0.4.0/src/curve/mod.rs:364`) | **0** | — |

Miter (fora e dentro) e Round dentro já estão certos nos três; não os toque.

## A cerca (gate red-first, antes do 1.º commit de produto)

- `the_offset_sweep_matches_the_table`: a varredura acima versionada como gate (600 combinações, sem resvg no
  gate — compare contra a fixture rasterizada gravada com cabeçalho: origem, comando, data). Hoje ele tem de
  dar EXACTAMENTE a coluna «hoje», e cada onda move a coluna com o número na mensagem do commit.
- Um contador de pânicos (hook de `std::panic`) dentro do gate: o `catch_unwind` não pode voltar a esconder
  um pânico sem que o número suba.
- A sonda e os scripts saem de `Estudos/` para `docs/Vector Module/ferramentas/` (regra J4).

## As ondas (ordem = dependência). Cada uma: o que entrega · como se prova

**W0 — Bancada do oráculo.** `vectorcraft-cli` corrido por script sobre um corpus NOSSO (as formas do catálogo
`ShapeKind`, pares tangentes, coincidentes, aresta partilhada, auto-intersectante): booleanas
(`object.pathfinder.{unite,minusFront,intersect,exclude}`) e `object.path.offsetPath`. Saída gravada como
fixture com cabeçalho; comparação **por passo** com as nossas `apply`/`offset_path`. Prova: um teste
`#[ignore]` que imprime a tabela dos dois lados; um gate normal que lê a fixture gravada.

**W1 — `linesweeper` 0.4 → 0.5.** Já medido: compila sem mudar uma linha e zera os pânicos da fixture. O risco é
o da subida anterior: **mudar a representação** da saída (contagem de vértices, peças do Power Stroke). Corra
`nextest-impacted` inteiro, leia cada vermelho com o mecanismo do comentário do `Cargo.toml`, e recontar a tabela
de `expand.rs:436-440` com a versão nova. Decida, com a medição, o destino do `catch_unwind`: se fica, fica
como cinto CONTADO (o gate acima), nunca como silêncio. Prova: Round fora 100/100, zero pânicos, suíte verde.

**W2 — O Bevel que some.** Primeiro `/pd-bug-causa` (com 0.5 já não é pânico: é `Region::of` a devolver `None`
ou o teste de «a forma desapareceu»? meça qual). Dois defeitos distintos:
- **dentro, 44–52:** só nosso — a forma ainda existe analiticamente (o VC responde) e nós devolvemos vazio;
- **fora, ≥ 164:** os dois falham (nós vazio, o VC estilhaçado) — precisa de verdade analítica própria
  (o offset Bevel de um polígono é a união das arestas deslocadas e dos triângulos de chanfro; escreva-a como
  função de teste e meça contra ela, não contra o VC).

Candidatos de cura, a medir um a um: o pós-processamento das booleanas do VC (`crates/pathops/src/boolean.rs`
— `snap_horizontals` :160, `is_sliver` por 2·área/perímetro :325, epsilon proporcional à magnitude :123-136,
`Tidy` :233). Prova: Bevel dentro e fora 100/100 contra a verdade analítica, ≤ 0,5 % de pixels. Se uma
distância continuar sem resposta, o artista tem de VER a recusa (memória: *recusa que só o terminal vê é um
botão mudo*) — a forma não pode simplesmente sumir sob o slider.

**W3 — Kerning.** O cozedor de texto é advance-only por desenho (`crates/ph2d-vec-text/src/glyph.rs:287`, leia
o doc-comment de `wrapped_lines` inteiro), e o `parley` (com `harfrust`) **já está na árvore** sem moldar — não
é dependência nova. As duas leis daquele comentário mandam aqui: (a) o quebrador de linha e o desenhador medem
com a MESMA régua (`line_advance`) — moldar um sem o outro dá texto que quebra numa largura que não ocupa;
(b) moldar **muda a aparência de todo texto já autorado** — é aceitação do dono: mostre-lhe o antes/depois de um
projeto real antes de virar o padrão. Texto em caminho tem de continuar a funcionar. Oráculo: `vectorcraft-cli`
com `--outline-text` / `type.createOutlines` sobre a MESMA fonte e o mesmo texto (`AV`, `To`, `Wa`, `fi`),
comparado por avanço de cada glifo. Prova: os pares com kern na tabela `kern`/`GPOS` da fonte mudam o avanço
como o oráculo; a quebra de linha usa o avanço moldado; texto sem pares é byte-idêntico ao de hoje.

**W4 — (só se medido pior) o Build / Shape Builder.** O VC tem um mapa planar que conta CRUZAMENTOS em vez de
winding (path aberto não cancela) e varre num referencial cisalhado por potência de 2, exacto em float
(`crates/pathops/src/planar.rs:14-30`). Corra os casos de BUGS #12 e #28 e o corpus da W0 nos dois; só abra a
onda se o nosso perder. Senão, registe a medição e feche.

## Fora desta linha (não reabra)

- **Malha de gradiente:** recusa registada em `13_fora_de_escopo.md` §13.9.
- **PDF / AI / EPS:** decisão de produto do dono; nada aqui.
- O que o VC faz e NÃO se copia: render só na CPU sem dirty regions, hit-test linear sem índice espacial,
  coordenadas `f32`, defeito de entrada engolido sem aviso, booleanas sem prazo nem cancelamento.

## Smoke para o Enio (ao fechar)

Uma cena com uma **estrela** e o Offset (Expand) à vista: arrastar o slider de ponta a ponta com Round e com
Bevel, e a forma **nunca some** onde antes sumia. O número da cena conta-se no roteador da família; fotografe-a
antes de mandar. Passos numerados, sem jargão (CLAUDE.md §0.8).
