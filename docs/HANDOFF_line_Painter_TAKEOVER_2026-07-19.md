# HANDOFF — TAKEOVER da `line/Painter` (2026-07-19)

> Você assume a `line/Painter` **depois de ela ter integrado ao main**. Este documento mostra **onde
> paramos** e **os planos a seguir**. Ele é o passo 5 da FASE 2 do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md) — leia o
> BLOCO daquele modelo ANTES deste, e execute a FASE 0 antes de ler qualquer código.

---

## 0. Reabra a linha (FASE 0 + FASE 1 do modelo — não pule)

A linha **já integrou**. Toda a rodada anterior (28 commits) está no `origin/main` com SHAs novos (rebase
na integração — os assuntos batem, os hashes não). Sua worktree ainda tem os commits antigos.

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && pwd && git branch --show-current
#   → TEM de terminar em /Worktrees/line-Painter  e  a branch TEM de ser line/Painter
#   → deu `main`? PARE. Você está na árvore errada (é o desenho da bancada, não desatenção).
git fetch origin main
git rebase origin/main        # obrigatório no início da jornada (DIRETRIZ §1.5.2.3)
cargo check -p ph2d-tool-painter
```

⚠️ **A cwd volta para o primário (`main`) entre turnos — aconteceu comigo DUAS vezes nesta jornada**, uma
delas mandou um commit pro `main` por engano (resgatado). **Abra TODO comando que muta com
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter &&`.** Não confie em lembrar; é o único erro
que se paga na integração.

Estado no fim da jornada anterior (antes do rebase): HEAD `2a898e11`, base `389676f9`, workspace **7651
verdes**, clippy 0, LOC verde.

---

## 1. O que esta linha É agora (impasto + sculpt + smear + deform)

O Painter tem uma pilha de **relevo** (impasto) completa. Os subsistemas vivos, com o arquivo-chave:

- **Depósito** (`ph2d-painter-brush/src/height.rs`): o relevo é um **envelope `max`** sobre o caminho
  varrido — idempotente, independente do espaçamento. Guarda um **VENCEDOR por carga**, não altura (a
  altura é `derive_height`, derivada depois — é isso que mantém Depth/Body/Source vivos após o traço).
- **Luz** (`impasto_light.rs` na CPU + `ph2d-render::ImpastoLightPass` na GPU): 4 lâmpadas, material
  per-pixel (`MaterialBytes = [u8;7]`), teto de vidro `soft_ceiling` (compressão C¹, mora na LUZ não no
  buffer). ⚠️ **`DEPTH_UNIT_PX = 16.0` é o número mais importante** — toda grandeza geométrica sobre `h`
  cruza essa unidade na entrada.
- **Sculpt** (`sculpt_blur.rs`/`sculpt_session.rs`/`sculpt.rs` + `sculpt_offset.rs`/`sculpt_close.rs`): 8
  verbos, `h = pre + k·Δ`. Inflate = a **bola limitada exata** + **closing morfológico**. Chisel = V em
  torno do eixo do traço.
- **Smear/Deform** (`warp/`): transporte por **mapa composto** (semi-Lagrangiano). Cor e corpo pela porta
  única `warp_render_relief`.

A doença recorrente desta linha, encontrada **quatro** vezes (cápsula do relevo · mordida do bow wave ·
campo do Smear · e a que o Accumulate teria trazido): **o relevo tem de ser função do CAMINHO, nunca de
quão fino o motor amostrou o caminho.** Toda vez que um kernel acumula sequencialmente lendo o resultado
anterior, o Spacing e a taxa de polling vazam para a obra. A cura é sempre um **envelope** ou uma
**integral de linha** (função pura da distância ao caminho), nunca uma soma sobre a lista de dabs.

Detalhe módulo-a-módulo: **CLAUDE.md §5**, entrada do Painter (é longa de propósito — cada gotcha custou um
smoke).

---

## 2. O que a jornada anterior FEZ (tudo smokado e aprovado, salvo nota)

| # | o quê | commit (pré-rebase) | smoke |
|---|---|---|---|
| 1 | **Smear virou um campo** — transporte por mapa composto | `14deb079` | ✅ aprovado |
| 2 | **Fork do plano canvas-inteiro paralelo** — engasgo de início de traço | `050c2d80` | (perf) |
| 3 | **Perf do memo do SMOOTH** paralelizada (10→5,5 ms montagem) | `aeedd9e5` | (perf) |
| 4 | **Perf do Warp gateada** — era o buraco do módulo; advecção é 60% do Deform | `db203634` | (perf) |
| 5 | **Conserve p/ Flatten/Fill** construído → **reprovado** → **removido de TUDO** | `b53e77a7`→`881432b4` | ✅ removido |
| 6 | **Eixo do Chisel vem do CAMINHO** (não da tangente suavizada) + oráculo de pixels | `c632b06e`+`10410f8e` | ✅ aprovado |
| 7 | **Inflate: defaults Sharper (Pow4) + ombro 8 px** | `ec29dd2a` | ✅ aprovado |
| 8 | **Accumulate no relevo** construído → **reprovado** → revertido; checkbox escondido no impasto | `1e22c1e0`→`09ffd339` | ✅ escondido |

**Lições que viraram memória nesta jornada** (leia se for mexer nos temas):
[[feedback_remeasure_a_documented_residual_before_curing_it]] (a causa E o número de uma nota podem estar
errados — aconteceu 2× no dia) · [[feedback_a_ratio_between_two_sick_channels_is_green_by_construction]]
(oráculo relativo é verde por construção; ancore no pincel/spec).

---

## 3. Onde PARAMOS — pendente de smoke (herdado, o integrador deve devolver)

Estas landaram e **não têm smoke ainda**. São a primeira coisa a mostrar ao Enio quando ele voltar:

1. **Luz do impasto na GPU** — [`HANDOFF_line_Painter_gpu_light_2026-07-18.md`](HANDOFF_line_Painter_gpu_light_2026-07-18.md).
   Paridade medida byte-a-byte com a CPU em 5 materiais. Só a ÓPTICA porta; o fold fica na CPU.
2. **Closing morfológico do Inflate** — [`HANDOFF_line_Painter_inflate_closing_2026-07-18.md`](HANDOFF_line_Painter_inflate_closing_2026-07-18.md).
   O footprint virou dilata+erode: enche a axila côncava, preserva a borda convexa.
3. **Filter Layer / Filter Stroke** (W5b do Sculpt) — o Filter Layer no Inflate foi aprovado, a borda
   serrilhada foi diagnosticada e a bola limitada/closing a curaram; **re-smoke junto do #2**.

Como ver: `cargo run --release -p ph2d-host-desktop`. Cenas montadas: `PH2D_IMPASTO_SMOKE=1`, e a sonda
`push_look_probe` (`PH2D_PUSH_LOOK_DIR=<dir> cargo test -p ph2d-host-desktop probe_push_render_and_look --
--ignored`).

---

## 4. Os PLANOS a seguir (fila, com recomendação)

Nenhum destes está começado. Todos precisam de **ordem explícita do Enio** para virar código — mas o
diagnóstico está pronto, para você não reconstruir.

### 4.1 Relevo do PAPEL — barreira aberta, mas o item MUDOU DE FORMA
[`docs/Painter/19_relevo_do_papel_investigacao.md`](Painter/19_relevo_do_papel_investigacao.md).
O Enio abriu a barreira do plano 16 §2, mas a investigação mostrou que **não é fiar `paper_height` na luz**
— é uma **extração de SUBSTRATO com um ADR na frente**. Três achados: (a) `paper_height` é só o *fallback*;
(b) o papel é do substrato, não da aquarela (mora em `watercolor_*` por acidente — a próxima mídia herdaria
o acoplamento); (c) a luz **não tem termo de substrato**, e a regra dela (*relevo sob cobertura zero não
acende*) **contradiz** um papel, que acende sem tinta. A pergunta do ADR: *onde mora o substrato, e o que a
luz sabe dele?* Parte cara = a **composição** (tinta espessa **preenche** o dente do papel, não soma:
`lerp(h_papel, h_tinta, f(carga))`, `f` quer medição). **Recomendo extrair; NÃO o atalho** de chamar
`watercolor_noise` do impasto. É uma wave, não uma fiação.

### 4.2 Accumulate na mesma pincelada — AVALIADO, e há um caminho vivo
[`docs/Painter/20_accumulate_na_mesma_pincelada.md`](Painter/20_accumulate_na_mesma_pincelada.md).
⚠️ **No IMPASTO foi construído e reprovado** (o Enio não gostou; o checkbox está escondido lá). Mas a
avaliação original era *"em todo o sistema"*, e a **integral de linha** (`∫ perfil·ds` sobre a CARGA, não
soma de dabs) é a lei correta: converge com o espaçamento e é idempotente sob re-stamp. O checkbox
`BrushSpec::accumulate` **já existe** (governa a cor). ⚠️ Se o Enio pedir de novo, a bifurcação do §6
(arco puro ⇒ parar o pincel deposita nada · por tempo ⇒ demora constrói, mas integra relógio de parede,
nunca ticks) tem de ser decidida ANTES. **Provavelmente morto para o impasto; talvez vivo para outro modo.**

### 4.3 Pen-up do Inflate — diagnosticado e RECUSADO por medição
Handoff da linha [§9.6](HANDOFF_line_Painter_smear_field_2026-07-18.md). `paint_end` renderiza **duas
vezes** (~3,7 ms/traço), mas isso **não atravessa um frame de 60 fps** — o artista não sente. Recusei
mexer no agendamento de render por ganho invisível. Só reabra se o Enio pedir.

### 4.4 A agulha do Smear — DECIDIDO que fica
Com o pincel padrão da faca (hardness 0.00) o Smear afunila numa agulha. **Não é a lei do transporte** —
é o Hardness, e *dureza É o controle de largura da esfregada*. Medido e aprovado assim. Não "conserte"
sem ordem.

---

## 5. Regras desta linha que NÃO são óbvias (paguei cada uma)

- **`git checkout` NUNCA para desfazer mutação** com trabalho não-commitado — apaga a feature e o gate
  "passa". Use `cp` do backup ([[feedback_mutation_undo_with_cp_never_git_checkout]]). Errei isto aqui.
- **`python str.replace` sem âncora única** casa a ocorrência errada — assert `count == 1` sempre
  ([[feedback_python_replace_silent_noop_after_fmt]]). Salvou-me 2×.
- **Backticks em msg de commit executam** — use `git commit -F <arquivo>`
  ([[feedback_backticks_in_commit_message_are_command_substitution]]).
- **Teto de LOC (700) mora na `editor-core`** e NÃO roda com `cargo test -p ph2d-tool-painter` — rode
  `cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap`. Estourou? **split**, nunca
  allowlist.
- **`HeightFields` é construído em 4 sítios** (3 testes + `impasto.rs`) — um campo novo toca os 4. E um
  plano novo arrasta o **ciclo de vida inteiro** (snapshot/restore/undo — §10.4, a cicatriz do `mats`).
- **Gates de perf são `#[ignore]`** e usam **RATIO**, não wall-clock (esta máquina deriva ~3×/sessão).
  Rode com `--release -- --ignored`.
- **Oráculo tem de medir a APARÊNCIA** (o que o artista vê), não o número intermediário que o tool
  computou. O gate do Chisel viveu verde sobre o bug reportado porque media *"os sulcos diferem"*, não
  *"o sulco é paralelo"*.

---

## 6. Reporte de entrada (passo 8 da FASE 2)

Depois da FASE 0/1, reporte ao Enio:

> *"Assumi line/Painter em Worktrees/line-Painter (HEAD `<sha>`, rebaseado sobre a main integrada). A
> rodada anterior fechou 8 itens (Smear/Chisel/Inflate/Conserve-removido/Accumulate); pendente de smoke:
> luz na GPU, closing do Inflate, Filter Layer. Aguardo a tarefa."*

E **PARE**. Não comece nada sem tarefa. Se o Enio mandar smoke primeiro, o §3 tem os comandos.
