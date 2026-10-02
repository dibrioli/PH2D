# HANDOFF DE CONTINUAÇÃO — `line/motion-value`, 2026-10-02 (AS LISTAS DAS CÉLULAS)

> **Para a próxima janela da MESMA linha** (não é um handoff de integração: a linha continua aberta).
> Assuma pelo [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
> (`cd` + `pwd` + `git branch --show-current` ANTES de ler). O mecanismo inteiro está no
> [doc 121 §9.8](../121_as_formas_na_placa.md); aqui só o estado e o que fica aberto.

## §0 — Identidade

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |
| ramo | `line/motion-value` |
| base | `main` @ `1ad60a1ce` (a linha foi alinhada por `git reset --keep main` — `cherry` dava `0`) |
| commits desta jornada | `5febba023` (as listas) · `22e7d069f` (gate da cena que muda + mutação 14/14) · `0d94bdfbc` (doc 121) · este handoff |
| ⚠️ o nome | o Enio chamou-a «linha `Nodes`»; não existe `line/Nodes` — a linha do Motion Nodes é esta |

## §1 — O que esta jornada fechou (itens 1 e 2 do handoff de 01/10)

- **Item 2 — as estrelas grandes no proxy:** cada célula de `32 × 1 px` guarda a LISTA das arestas que a
  cruzam (o ladrilho do Vello), montada por ARESTA em quatro despachos indirectos, fundos e somas em
  PONTO FIXO (`2¹⁶`) ⇒ a imagem não depende da ordem dos atómicos. Sonda calma, iGPU (placa · Vello):
  esticadas `2,04 → 1,74` · `1,05`, conformes `1,18 → 0,94` · `0,92` (empata), densas `2,13 → 1,71` ·
  `2,81`. RTX: esticadas passam à frente do Vello (`0,36` · `0,40`).
- **Item 1 — a W0/W5 como tabela única, máquina calma:** feita no app (doc 121 §9.8). A `=127` densa
  na iGPU: **`20,7 → 17,6 ms`** com a placa (era `35,2` no §9.3); o resto bate nos `60 Hz`.
- O protótipo «cobertura 8 px por fio» do §9.7 **não foi reconstruído** (não estava guardado e só
  dividia a leitura); o §9.7 aponta agora para o §9.8.

## §2 — Gates e provas (corridos nesta árvore)

- GPU RTX: `ph2d-shape-gpu` **7/7** (+ `as_fileiras_que_nao_cabem_nas_listas_desenham_o_mesmo` e
  `uma_cena_que_muda_nao_le_as_arestas_do_quadro_anterior`, novos) · `ph2d-app-motion` placa **4/4** +
  ponte **1/1** · `ph2d-gpu-cook` formas **2/2**.
- Mutação **14/14** ([arnês](../ferramentas/mutacao_as_listas_das_celulas_2026-10-02.py)); a M10
  sobreviveu aos seis gates de então e o gate da cena que muda foi escrito por ela.
- `cargo clippy -p ph2d-shape-gpu -p ph2d-app-motion --all-targets -D warnings` ✅ · `fmt` ✅ ·
  `doc-index --check` ✅. ⚠️ O gate batched de FECHO (`nextest-impacted` + censos) NÃO correu — é do
  fecho da linha.

## §3 — ⚠️ O que um leitor do diff pode entender ao contrário

1. **`QUADROS_DO_PRODUTO` 3 → 5 e `QUADROS` 4 → 6 não são folga:** as listas só se contam depois de
   as arestas existirem, logo a capacidade delas chega dois quadros depois da das arestas. Até lá a
   fileira é `SEM_LISTA` e desenha pelo caminho de sempre — a mesma imagem (gate).
2. **Os passes das células correm um fio por aresta RESERVADA**; a guarda `bl >= c0.y + c0.z + c0.w`
   em `aresta_de` é a que impede arestas velhas de um quadro anterior de entrarem (a M10).
3. **Na RTX a `=127` densa subiu `0,19 → 0,21 ms`** na sonda (quatro despachos a mais numa placa onde
   o cálculo era quase nada) — continua `2,5×` à frente do Vello. Nomeado, não curado.
4. O perfilador separa agora o cálculo em `render.contorno.conta` · `.escreve` · `.celulas` (era um
   relógio só, `render.contorno`).

## §4 — ⏳ O QUE FICA ABERTO, na ordem proposta

| item | o número / o endereço |
|---|---|
| **as grandes ESTICADAS no proxy** ainda perdem (`1,74` contra `1,05`) | decomposição iGPU: desenho `0,82` · células `0,43` (contar `0,16` · escrever `0,16` · lugar `0,08` · zerar `0,04`) · escrita das arestas `0,20` (um fio por CÓPIA) · contagem `0,06`. O desenho é o maior; o próximo degrau é o do Vello inteiro (rasterizar fino em cálculo, `4 px` por fio) e pede a ordem entre cópias — desenho próprio, não um ajuste |
| o **traço TRACEJADO** sob escala não-uniforme fica no Vello | pede o comprimento de arco por cópia no eixo (`eixo.rs`) |
| os **glifos** do `source.text` ficam no Vello | — |
| `M6`/`S6`/`S8` (§9.5/§9.4) e o `fx.glow` que lê o `pump` anterior | nomeados no handoff de 01/10, sem mudança |
| `fk.rs` duplicado em seis crates (bug #11) | wave própria |

## §5 — Instrumentos (versionados)

- [`mede_sonda_das_estrelas.sh`](../ferramentas/mede_sonda_das_estrelas.sh) — a sonda nos três arranjos,
  nas duas placas, só com `load < 4`, a placa pela porta da casa só durante a corrida. COPIE o binário
  antes (o próximo build do mesmo perfil sobrescreve o nome).
- `PH2D_FLUID_PROFILE=1` na `sonda_relogio_das_estrelas_grandes` ⇒ `250` quadros e o relógio da placa
  por passe.
- [`mede_formas_na_placa.sh`](../ferramentas/mede_formas_na_placa.sh) — a W5 no app (não edite `.rs`
  enquanto corre: o fotógrafo recompila e recusa código mais novo que o binário).
