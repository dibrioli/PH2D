# HANDOFF de continuação — line/UIUX, onda F38 (a família «memória da placa» fechada), 27/10

> Linha NÃO integrada, sem push. Base: `main` c3662cd8e4 (não andou). A F38 são os commits de
> `git log --oneline fea3340c24..HEAD`. **⚠️ Smoke do dono PENDENTE para F37 e F38 — ele adiou-os («smoke
> depois») e mandou seguir. Nenhuma das duas está aprovada até ele** (passos no §4).
> Mecanismo, tabelas, mutações: [`F38_plano.md`](../F38_plano.md). Antecessor:
> [`HANDOFF_CONTINUACAO_line_UIUX_2026-10-27_F37.md`](HANDOFF_CONTINUACAO_line_UIUX_2026-10-27_F37.md) (§8 dele
> continua a lista do que está aberto; a F38 fechou os itens 1, 2, 8 e 9).

## 1 — Commits

| commit | o quê |
|---|---|
| `60f006227f` | a textura recusada vive numa substituta 1×1 e não recebe trabalho de placa até renascer; `readback` lê a fonte; G28 |
| `edf89c69db` | plano F38 + rodada na Radeon + sonda `blocos_do_alocador` + `ocupa_a_placa` lê a VRAM de uma AMD |
| `a567ffa870` | fecho de onda: rótulos de placa do `fx_live_textura.rs` com cara de identificador (censo HR-15 da `ph2d-app-vec`) |
| `aa34bf4b07` | o smoke da placa cheia medido antes de entregar (F37 132 `[gpu-erro]` falsos, F38 0) |

## 2 — Números (plano §1–§3)

- Radeon 610M (2 GiB VRAM): cena 1 F36 819 → F37/F38 **366** MiB; cena 41 1 331 → **494** (NVIDIA 377/507).
  Nunca recusa: o `amdgpu` despeja no GTT (F36 com 546 livres: 1 020 MiB no GTT). A tela virtual tem de
  renderizar na mesma placa do app (kwin na NVIDIA ⇒ `Invalid surface`).
- Blocos: rabisco do Composite sem diferença nas duas placas (~20 ms); a sonda só mostra preço no recurso
  médio (4–16 MiB) criado em série ⇒ **fica `MemoryUsage`**.
- Mipgen: views e bind groups por chamada, nada em cache; o defeito era trabalho de placa sobre a recusada.

## 3 — Fecho de onda

- `cargo fmt --all`; clippy `-p ph2d-render -p ph2d-gpu --all-targets -D warnings`: limpo.
- G17, G18, G28 verdes (placa); M13–M24 sangraram todas no G28.
- A suíte (`nextest-impacted`, `PH2D_SUITE=onda`) correu inteira (22 421): 2 vermelhos — o censo HR-15 da
  `ph2d-app-vec` (desta linha, curado em `a567ffa870`, verde por nome) e
  `ph2d-host-code-scenes a_cena_3_conduzida_passo_a_passo` (CodeEditor, não tocado; carga ~47; verde sozinho
  nos perfis `smoke` e `dev`).
- Binário do smoke compilado (2.ª build «Finished … 0.45s», zero Compiling).

## 4 — Smoke do dono — PENDENTE (F37 + F38)

F37: 1) Terminal 1: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/examples/ocupa_a_placa 950 120` → «seguro 120 s».
2) Terminal 2: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=41 ./target/smoke/ph2d-host-desktop`.
3) Certo: a forma verde, «Bevel» na Hierarchy. 4) Errado: aviso de memória, «Path 0», forma ausente.
F38: 1) `… ocupa_a_placa 400 60` → «seguro 60 s». 2) o mesmo comando da cena 41. 3) Certo: a forma e
nenhuma linha `[gpu-erro]` no terminal; depois dos 60 s, igual. 4) Errado: `[gpu-erro] … recusou trabalho` a repetir.

## 5 — Aberto (por ordem)

0. Smokes F37 e F38 pendentes (§4).
1. F32 §6.2 — pose dos estados de UI alinha o MEIO da transição por ÍNDICE; F32 §6.3 — chave sobre
   `from`/`to` de uma seta guarda o NÚMERO do estado. Mesma família (referência por índice que envelhece).
2. F33 §6.2 — seleção pela cena 40 não acende o painel (`vec.pen.select_many`?).
3. F33 §6.3 — Zig Zag/Sketch no vale côncavo: decisão do dono, com o oráculo a correr.
4. F32 §6.4 — F31 §6.5 e F30 §6.5–6.9.
5. De outras famílias (não fazer aqui): tela 4K com rabisco automático e pool dos buffers do lote no
   `StampPass` (Painter); o `fetch` morto de `insert_atlas_sprite_with_regrow` (4 famílias).
6. `readback_mip` do átlas recusado lê a substituta (só testes); medir numa AMD dedicada.
7. Integrar e push só por ordem EXPLÍCITA do Enio; a suíte completa é do integrador.
