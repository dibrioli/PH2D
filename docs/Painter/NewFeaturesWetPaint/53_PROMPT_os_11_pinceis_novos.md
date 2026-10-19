# 53 — PROMPT: os 11 pincéis novos do Wet Paint (de receita medida a preset de fábrica)

> Leitor: a próxima janela da `line/PainterWatercolor`. Escrito a 2026-10-19 no fim da sessão que mediu os
> pincéis por fazer ([52](52_medicao_os_pinceis_por_fazer.md)). Tudo aqui é técnico; o que vai ao Enio é
> curto, sem jargão, e em passos numerados (CLAUDE.md §0.8).

## §0 — Arranque (antes de abrir QUALQUER ficheiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
git log --oneline -5 && git status --short --ignored | grep -v target/
git cherry main HEAD | grep -c '^+'      # >0: os commits são da linha; 0: o integrador já os levou → git reset --keep main
git rebase main 2>&1 | tee target/rebase.log
```

- `pwd` tem de acabar em `/Worktrees/line-PainterWatercolor`; a branch tem de ser `line/PainterWatercolor`. Outra
  coisa ⇒ PARE e reporte ao Enio ([MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).
- HEAD esperado: o commit deste prompt, sobre `beb8a7e38b` (o 52) e `2fbe88879c` (os abertos do P51 adiados).
  Merge-base `100d34c8e0`. Nada integrado nem enviado.
- Leia, por esta ordem e SÓ isto: o [52](52_medicao_os_pinceis_por_fazer.md) inteiro (9 KB) · a [DIRETIVA_IMPLEMENTACAO](../../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)
  · `crates/ph2d-tool-painter/tests/it/receitas_de_fabrica.rs` (o `struct Receita`, a REGRA da fábrica e duas receitas
  vizinhas: Graphite 8B e Acrylic Round) · o portão `cada_preset_parece_a_sua_midia` em `tests/it/midias_de_fabrica.rs`.
  Para o resto, delegue ao `explorador` (a janela anterior chegou a 325 mil tokens por passo; o alvo é ≤ 250 mil).

## §1 — A ordem do dono e o estado

- 19/10: *«adie essas tarefas abertas e veja quantos pincéis faltam»* → *«vamos nisso»*. Os 4 abertos do P51 (relógio
  por parte · carimbo fora do orçamento do quadro · pó por comprimento · `Salto` em `Dab::capsula`) estão **ADIADOS**
  ([51 §8](51_medicao_spacing_continuous.md)): não os abra sem ordem dele.
- O 52 mediu os 19 pincéis do plano de arranque ([02 §6](02_presets_para_wet_painter.md)) contra 16 fotos novas do
  Rebelle: **11 saem só de valores** (esta tarefa), 4 já são um vizinho na régua, o Marker/Effect pede motor,
  Granulation/Spatter sem oráculo, Sumi-e fora desde o 18.
- **Esta janela:** transformar as 11 receitas vencedoras em presets de FÁBRICA (ficheiro em `assets/brushes/WetPainter/`,
  receita em `receitas_de_fabrica.rs`, portão por foto), provar que cada um é ele (e não o vizinho), e entregar o
  smoke ao Enio. Produto final, não MVP.

## §2 — O que JÁ existe (não reconstruir)

| coisa | onde |
|---|---|
| as 16 fotos do Rebelle (estado A, cabeçalho, README) | `fixtures/rebelle/{pencil2h,pencilhb,pencil2b,pencil4bcolor,scratch,squarehard,waxscrape,filbert,dabber,flatoily,knife,…}.png` · `cabecalhos.json` |
| as receitas EXACTAS das passadas (fonte da verdade dos valores) | `52_regua/receitas_19.json` · `receitas_19b.json` · `receitas_19bc.json` (por `nome`) |
| a impressão digital da cena SECA de cada vencedora | `52_regua/fnv_vencedoras.txt` (raio, `fnv`, lei raio→largura) |
| o veredito de cada passada e dos vizinhos de fábrica | `52_regua/passada{1,2,3}_*.txt` · `vizinhos_de_fabrica.txt` |
| a sonda (receita com `base` = preset de fábrica + overrides; QUALQUER knob do registo) | `tests/it/sonda_midias.rs` (`PH2D_SONDA_RECEITAS`, `PH2D_SONDA_SECA=1320`) |
| a barra (o vale do 18 §2, 3× o ruído do Rebelle, largura ± 2) | `ferramentas/avalia_19.py <lote> <docs/…/NewFeaturesWetPaint> [sufixo]` |
| as fotos por lote pela porta WebSocket | `ferramentas/fotos_lote.sh <lista> <saída>` (+ `lote_19.txt`) |
| o gerador dos presets de fábrica | `tests/it/presets_de_fabrica.rs` (`gera_os_presets_de_fabrica`, ignorado) |

## §3 — Os 11 (os valores: LEIA-OS do JSON pelo nome; esta tabela é o mapa, não a fonte)

| preset (nome proposto) | pasta `WetPainter/…` | foto | receita vencedora | base | o que muda (resumo do 52 §4) |
|---|---|---|---|---|---|
| Graphite 2H | `Pencil/Graphite` | `pencil2h` | `lapisb_027` | Graphite 8B | pigmento 40 · papel 0,3 · Noise 16 |
| Graphite HB | `Pencil/Graphite` | `pencilhb` | `lapisb_028` | Graphite 8B | pigmento 40 · papel 0,3 · Noise 32 |
| Graphite 2B | `Pencil/Graphite` | `pencil2b` | `lapisb_032` | Graphite 8B | pigmento 40 · papel 0,65 · Noise 64 |
| Colored 4B | `Pencil/Colored` | `pencil4bcolor` | `lapisb_041` | Graphite 8B | pigmento 80 · papel 0,65 · Noise 64 |
| Scratch | `Pencil/Scratch` | `scratch` | `lapisc_053` | Charcoal | pigmento 20 · papel 0,5 |
| Square Hard | `Pastel/Dry` | `squarehard` | `sq_007` | Soft Pastel | pigmento 60 · papel 1 · dureza 0,2 · ponta reta (`dab_quina` 1) · Noise 100 |
| Wax Scrape | `Pastel/Scrape` | `waxscrape` | `wx_007` | Soft Pastel | pigmento 150 · papel 1 · dureza 0,2 · Noise 100 |
| Filbert | `Oil/Flat` | `filbert` | `filbert_016` | Acrylic Round | cerdas 2 · pigmento 500 · dureza 0,5 |
| Dabber | `Oil/Dabber` | `dabber` | `dabberc_032` | Oil Bristle | cerdas 2 · relevo 1,2 · força 2 · jitter 0,5 · dureza 0,6 |
| Flat Oily | `Oil/Flat` | `flatoily` | `laminac_018` | Acrylic Round | sem cerdas (força 0, feltro 1) · pigmento 1000 · chato 0,82 a 0° · dureza 0,9 |
| Knife Smooth | `Oil/Knife` | `knife` | `laminac_012` | Acrylic Round | idem com pigmento 500 |

Os nomes e as pastas são PROPOSTA (o idioma dos 20 de hoje: o nome do Rebelle, a pasta pela ferramenta dele); o
dono confirma no smoke. ⚠️ Os nomes vão para o ficheiro do preset (dado), não para o i18n — confira como os 20 o fazem.

## §4 — Como (passo a passo, cada passo com a prova dele)

1. **A receita é a BASE inteira + os overrides.** Cada `Receita` nova copia TODOS os campos da `Receita` da base
   (`water`, `opacity`, `pigment`, `gate`, `extras` — incl. `MAIS_PERTO`, `capsuleMaxPx`, `pigmentSpacing`,
   `capsuleSpacing` —, `achatado`, `grao`, `pincel`) e aplica por cima os do JSON vencedor (`knobs` → campo próprio
   ou `extras`; `brush` → `hardness`/`grao`/`achatado`/`pincel`). O Dabber herda o `achatado (0,68, 90)` do Oil
   Bristle e o Flat Oily/Knife o `capsuleSpacing` da acrílica: herdar É a receita medida. ⛔ Não «limpe» o que veio da
   base sem medir.
2. **A regra da fábrica** (`SPACING_DA_FABRICA` 0,1 + Continuous, censo `a_regra_do_spacing_da_fabrica`) vale para os
   11 — as bases já a seguem, por isso a receita medida já a tem.
3. **O tamanho** (`size_px`): o do preset do Rebelle (o idioma dos 20: «o tamanho é o do preset dele»). O JSON dos 281
   não o tem numa chave óbvia — procure como o Graphite 8B (6,5) e o Charcoal (37) o acharam (comentários das receitas,
   prompt 35, `ferramentas/extrai_pinceis.py`); se não houver fonte, MEÇA na porta WebSocket (`SET_BRUSH` sem
   `size_px` e ler a largura) — nunca invente.
4. **`PRESETS_DE_FABRICA` 20 → 31** é CONTADO (`the_factory_presets_are_counted`), nunca escolhido.
5. **Gerar só os novos:** `PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="Graphite 2H,…" bash scripts/ph2d-run.sh cargo test -p
   ph2d-tool-painter --profile smoke --test it gera_os_presets_de_fabrica -- --ignored --nocapture` (os 20 de hoje
   ficam ao byte; confira `git status` em `assets/brushes/`). Se o gerador imprimir as linhas de `LARGURA_DO_WET_ROUND`,
   cole-as (`wetpaint/largura.rs`).
6. **⭐ Prova de identidade (o preset É a receita medida):** uma receita `{"nome":"id_<x>","base":"<preset novo>",
   "alvo":<alvo do fnv_vencedoras>}` na `sonda_midias` (SECA) tem de dar o MESMO `fnv` do `fnv_vencedoras.txt`. Fnv
   diferente = a cópia da base ou a tradução dos overrides está errada: ache a diferença antes de seguir.
7. **⭐ Prova de que não é o VIZINHO** (18 §4.3, «um nome, não uma mídia»): pelo `avalia_19.py`, cada preset novo
   passa a SUA foto, e o vizinho de fábrica não (o `vizinhos_de_fabrica.txt` já o mostra para a base). E entre IRMÃOS:
   o 52 achou que as melhores do **2H e do HB passam uma na outra** (o Rebelle difere pouco: cob 0,595/0,561), e
   **Flat Oily e Knife diferem só no pigmento** (1000/500; o cruzamento do Rebelle 0,723/0,513). Meça cada irmão contra
   a foto do outro; se passar nas duas, procure o eixo que os separa (o cruzamento, a cobertura) com uma grelha curta, ou
   reporte ao Enio que no Rebelle também são quase o mesmo pincel. ⛔ Não entregue dois presets que pintam igual.
8. **O portão por mídia:** cada um dos 11 entra no `cada_preset_parece_a_sua_midia` com a SUA foto, à lei de largura
   do PRÓPRIO preset (43 §4: o preset traz a sua lei; a sonda lia o traço MOLHADO e a régua o SECO com grão — 3–5 px de
   diferença nos lápis). Mutação: trocar a foto de um pelo vizinho tem de reprovar (red-first, `/pd-mutacao`).
9. **A curva e a ponta:** o Flat Oily do Rebelle é fino no recto e LARGO na curva (`flatoily.png`, a lâmina fixa na
   tela). A régua só mede o recto — olhe a curva na prancha antes/depois e diga o que viu.

## §5 — Medição num lote só (CLAUDE.md §0.10)

Um `cargo test --profile smoke` por rodada, e numa rodada: as 11 provas de identidade (fnv) + os 11 contra a própria
foto + os irmãos cruzados (2H↔HB, Flat Oily↔Knife, 2B↔4B Color) + os vizinhos. ~100 receitas ≈ 3 min (load ~5).
Série só quando um resultado muda o passo seguinte (ex.: a grelha que separa irmãos) — e diga-o.

## §6 — Armadilhas medidas (não repita)

- **A placa:** a tela virtual do Rebelle (kwin + wine) usa a GPU e NÃO pede a vez sozinha — `PH2D_GPU=1 bash
  scripts/ph2d-run.sh bash ferramentas/gotas_inicia.sh` (a janela anterior abriu-a sem a vez: falha registada).
  Fechar só com `gotas_fecha.sh` (⛔ nunca o `fecha.sh` do oráculo: mata as telas das outras linhas).
- **O teste do Rebelle expira ~06/11** (18 dias a 19/10). Foto nova é para agora.
- Um `SET_BRUSH` com preset inválido deixa **diálogos de erro em fila** que estragam as fotos seguintes:
  `bash ~/Apps/rebelle/oraculo/inj.sh "rect e"` antes de um lote.
- O `import` de `receitas_19.py` GRAVA (código no topo): copie a função, não a importe.
- O guarda R6 lê qualquer `.replace(` num script como edição de ficheiro: monte nomes com `split`/`join`.
- O `metricas()` parte (`IndexError`) num traço que não acha ponta; o `avalia_19.py` já o trata como «sem medida».

## §7 — FORA do escopo (não faça; está no 52 §6)

Os 4 «cobertos» (Rake, Fountain Square, Reed Pen, Marker Chisel: precisam da métrica da FORMA da ponta antes de
existirem) · o Marker/Effect (motor: o modo de mistura do pincel; há um campo `blend` em `CAMPOS_DO_PINCEL` — por medir
se o Wet Paint o lê) · Granulation/Spatter (sem oráculo sob o Wine) · Sumi-e · os 4 abertos do P51. Se o dono pedir um
deles, é outra onda.

## §8 — Fecho (DIRETRIZ §1.5.9; a linha NÃO integra nem envia)

1. Gate batched 1× sobre o diff acumulado: `PH2D_SUITE=fecho bash scripts/nextest-impacted.sh` + clippy `--all-targets`
   (só aqui) + `fmt`; delegue ao `verificador`. ⚠️ Flakes de carga conhecidos: `o_pigmento_por_comprimento::
   a_tinta_nao_muda_com_o_spacing` (TIMEOUT a load 21; promova a [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) se repetir).
2. Handoff de integração `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_<data>_P53.md` (≤ 8 192 bytes;
   o P51 é o modelo), a linha do §5 do CLAUDE.md, e o 26_ABERTOS.
3. O binário do smoke JÁ compilado na worktree (`--profile smoke`; 2.ª compilação colada no handoff), a máquina limpa
   (`ps -eo pid,comm` — nada nosso), `rm -rf target/*/incremental`, `bash scripts/agent-loop-profile.sh`.
4. **Smoke ao Enio**, curto e em passos, com uma foto da cena (os 11 pincéis lado a lado, um traço de cada, e o
   vizinho ao lado para ver a diferença). Rascunho:
   1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_WETPAINT_SMOKE=2 ./target/smoke/ph2d-host-desktop`
   2. No Painter, em Wet Paint, abra o navegador de pincéis: há 11 novos (Graphite 2H, HB, 2B, Colored 4B, Scratch,
      Square Hard, Wax Scrape, Filbert, Dabber, Flat Oily, Knife Smooth).
   3. Risque com cada um: o lápis mais duro sai mais claro e fino de grão; a espátula e o Flat Oily saem lisos e
      chapados; o Dabber sai manchado como uma esponja.
   4. Deu errado se dois deles pintam igual, se algum pincel antigo mudou, ou se algum sai em branco.
