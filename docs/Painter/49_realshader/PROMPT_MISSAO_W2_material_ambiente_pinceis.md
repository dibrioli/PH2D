# Prompt da janela nova — line/PainterWatercolor: MISSÃO W2 do Óleo Real (autónoma, do início ao fim)

```
Linha line/PainterWatercolor (Modo L) — MISSÃO W2 do Óleo Real no Wet Paint, AUTÓNOMA: o dono não está. Não faça perguntas, não peça permissões, não pare à espera dele. Faça tudo por AGENTES PARALELOS, por etapas, a janela principal só decide e integra — guarde o contexto dela.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, docs/Painter/49_realshader/PROMPT_MISSAO_W2_material_ambiente_pinceis.md e
docs/Painter/49_realshader/HANDOFF_CONTINUACAO_oleo_real_W1c_2026-10-27.md inteiros; docs/Painter/49_realshader/oraculo/README.md e
docs/Painter/Pinceis/REGRAS_DE_CUSTO.md. Depois execute as etapas E1→E4 deste ficheiro sem parar.
```

## Estado ao abrir

- W1c aprovada no smoke (27/10, «Realmente muito bom»). Backup: tag `backup/oleo-real-W1c-2026-10-27`. Árvore limpa
  (confirme `git log -3` e `git status`).
- O dono pediu (27/10), por esta ordem: **(1)** material realista como o Cycles com todos os ajustes; **(2)** contraste da
  sombra de fenda tipo AO; **(3)** HDRI como meia-esfera sobre o papel; e **quando 1–3 estiverem bons**, **(4)** presets
  parecidos com os do Rebelle: Metallic **Flat 3 · Round 3 · Splats**; Dabber **Dabber · Dabber 2 · Dabber 3 · Dabber 4 · Fan**;
  Knife **Smooth · Smooth 2 · Smooth 3 · Textured · Oily · Stain · Soft · Grainy · Dry · Dry 2**.

## Parâmetros novos: CRIE ou EXPONHA os que faltarem (dono 27/10)

O dono: *«talvez seja necessário criar ou expor novos parâmetros»*. Licença explícita: quando um preset da E4 ou um
material da E3 precisar de um controlo que o motor não tem (ou que existe mas não está no painel), **crie-o ou
exponha-o** — com linha de UI completa, neutro bit-idêntico e gate. Referência do que o artista do Rebelle consegue
MEXER (só o que se vê na tela dele, capturas do dono 27/10 — nunca ficheiros dele):

| Painel do Rebelle | Controlos que se veem |
|---|---|
| Pincel «Oils & Acrylics» (ex.: Knife Smooth 2) | **Size** 35 · **Loading** 100 · **Oiliness** 80 · **Pressure** 100; botões de modo: pintar, pintar com água/misturar, borrar, apagar; carregar/limpar pincel |
| Visual Settings → Rendering | **RealShader** · **NanoPixel** · **SoftShadows** (caixas) |
| → Oils & Acrylics | **Impasto Depth** 8 · **Gloss** 10 · **SoftShadows** 10 · **Shadow Altitude** 10 · **Environment** («Environment 2») |
| → Metallic Materials | **Reflectivity** 5 · **Reflection Map** (lista) · **Scale** 5 · **Strength** 5 · **Lock Reflectivity** (caixa) |
| → Canvas Visibility | **Paper Texture** 10 · **Paint Texture** 10 |

Leitura para a missão: **Oiliness** e **Loading** são por PINCEL (as facas diferem por eles — confira o que o nosso Oil
já tem e o que falta); **Reflectivity/Reflection Map/Scale/Strength** são o material metálico da E3 (Scale = escala do
mapa de reflexo; Lock Reflectivity = pintar sem mudar o metal já pousado — encaixa no material POR CÉLULA);
**Paint Texture** = quanto o relevo da tinta aparece (ponte com Depth). Nós podemos ter MAIS que o Rebelle (os ajustes
do Principled da E3); menos, não.

## Regras da missão autónoma (valem para a janela E para todo agente)

1. **Nada de perguntas nem de pedidos de permissão.** Decisão técnica = padrão-ouro, decidida e escrita no `ESTADO`.
   Item que não fica bom depois de **2 rondas** fica registado com a medição e a missão SEGUE para o próximo — nunca pára.
2. Confirme ANTES de lançar agentes: `grep -c aprova-sem-perguntar .claude/settings.local.json` ≥ 1. Todo briefing de agente
   leva as **formas proibidas**: `cd X && …`, `bash -c`/`sh -c`, `rm -rf` com variável ou glob, `kill` de PID alheio,
   escrever em `.git/`, `.claude/` ou rc da shell, `git stash`, `git push`. Caminhos ABSOLUTOS; o comando corre já na worktree.
3. **Contexto da janela principal:** ela não lê código grande nem saídas longas — delega. Agente devolve **≤ 20 linhas**
   (o que fez, commits, números, o que falhou). Leitura em muitos ficheiros → `explorador`; compilar/testes/portões →
   `verificador`; docs → `documentador`; provas de mutação → `mutacao`; implementação → `general-purpose` com briefing
   completo (o agente só sabe o que está no briefing).
4. **Custo (dono 21/10):** ≤ 3 agentes em paralelo; ≤ 2 rondas de cura por item; revisor/verificador com `model: "sonnet"`
   e resposta curta; ≤ 4 variantes por cura escolhidas pela medição; nada de grelhas grandes.
5. **Paralelismo sem colisão:** dois agentes ao mesmo tempo só em ficheiros disjuntos (diga a cada um quais são os dele).
   Mesmo ficheiro ⇒ em série. Cada agente comita SÓ os seus caminhos: `git add -- <paths>` +
   `git commit --no-verify -m "…" -- <paths>` (ficheiro novo: `git add` antes, senão o commit não o leva). Nunca `-A`/`.`.
   Um `CARGO_TARGET_DIR` só (o da worktree) — nunca partilhado com outra worktree.
6. **Estado em disco:** depois de cada etapa a janela escreve `docs/Painter/49_realshader/ESTADO_MISSAO_W2.md` (etapa,
   commits, números, decisões, o que ficou) — se o contexto acabar, a janela seguinte retoma DAÍ.
7. Engenharia: inner loop `bash scripts/cargo-check-narrow.sh <crate>` (para testes: `cargo check --tests` por
   `scripts/ph2d-run.sh`); testes POR NOME (`bash scripts/cargo-test-narrow.sh <crate> <nome>`); clippy SÓ no fecho
   (`PH2D_SUITE=fecho`); ⛔ suíte completa NÃO (é do integrador). Pesado por `bash scripts/ph2d-run.sh …`, `--profile smoke`.
   Variantes no MESMO processo (`PH2D_OLEO_AF="campo=valor"`). UI: zero hex/literal/string — tokens e `ph2d-i18n`.
8. **Oráculos (CLAUDE.md §0.9):** Blender/Cycles CORRIDO por script (receita em `oraculo/README.md`, `PH2D_GPU=1` +
   `LD_LIBRARY_PATH` do cudart); Rebelle CORRIDO (fotos já gravadas em `docs/Painter/Pinceis/fixtures/rebelle_bateria/`
   — gestos/tamanhos/sobre para TODOS os 18 presets da E4; fotos novas pela porta WebSocket, receita na memória
   `reference_rebelle_trial_installed_under_wine`). Nada do Rebelle se lê nem se copia (ficheiros de preset, texturas);
   do Blender nunca se lê o fonte. **Máscaras de régua vêm do ORÁCULO, nunca do predicado do produto.**
9. **Neutralidade:** cada ajuste novo tem um valor neutro que deixa a imagem de hoje IGUAL (gate por nome); a tinta lisa
   continua a sair com a cor dela em todo ambiente (`a_tinta_lisa_sai_com_a_cor_dela_em_todo_ambiente`).
10. **Olhe a FOLHA** (`medida_oleo_real_folha_png` / `medida_oleo_real_oraculo`, `magick` para PNG e zoom) antes de dar
    uma etapa por boa — número sozinho já aprovou régua cega nesta linha. ⛔ As tabelas `⛔ Recusas MEDIDAS` do
    `oraculo/README.md` não se reconstroem (sobretudo «difuso do HDRI × visibilidade média»).
11. Sem push, sem integração. A linha acaba no smoke + `ESTADO`; o handoff final só depois do veredito do dono.

## As etapas (série justificada: E2 muda a base que E3 mede; E4 usa o material da E3)

### E1 — Contraste da sombra de fenda (o «AO» do dono) · pequeno, 1 agente

- Dois ajustes no cartão Real Oil: **força** da sombra de fenda (0 = sem, 1 = a física de hoje, > 1 dramático) e
  **alcance** (até que distância uma pincelada escurece a vizinha — hoje o horizonte geométrico `[1..24]`).
- Neutro (1 e o alcance de hoje) = bit-idêntico ao de hoje (gate). Força alta nunca volta aos pontos pretos
  (gate: fenda vermelha sem pixel `max < 70`, a régua de `compara.py` §1). Linha de UI completa: id, `BrushSettings`,
  snapshot, decode/option_route, populate, i18n, macro JSON (campo ausente = mantém), costura `seam_wetpaint_oleo_real`
  — o mesmo caminho do seletor Environment (`c3365fb579`, veja o diff dele como molde).

### E2 — O HDRI como meia-esfera sobre o papel · o maior, 1–2 agentes em série

- O ambiente passa a iluminar o DIFUSO da tinta e do papel, não só o reflexo. Lei padrão-ouro: a visibilidade
  **por direcção** (os horizontes das fatias do GTAO já existem — cada fatia dá o sector de céu visível) integrada contra
  a irradiância do HDRI nesse sector (cone/«bent normal» por fatia ou integral por sector sobre o mapa pré-filtrado;
  escolha medindo). ⛔ Nunca o difuso × visibilidade MÉDIA (recusa medida: a janela entrava nas fendas, 1,09 × 0,55).
- Papel: recebe a luz do ambiente e a sombra suave da tinta vinda de todo o céu (hoje só céu uniforme + lâmpada).
- Ajuste de **força do ambiente** (mistura ambiente × lâmpada); Classic continua o de hoje, bit a bit.
- Régua contra o Cycles (`hdri_studio_cinza`, `studio_sol10_cinza`, `hdri_interior`, `hdri_courtyard`): difuso na tinta,
  na fenda (V Cycles < 0,55) e no papel ao pé das paredes; correlação; e a tinta lisa = a cor dela. Aproveite para medir
  o rebote da lâmpada nas paredes vizinhas (aberto: difuso 0,71 × 0,83).
- Custo: meça ns/célula (sonda `medida_oleo_real_custo`) e escreva a tabela; se passar de ~2× o de hoje, a decisão
  (pirâmide de máximos / passe de placa) vai para o `ESTADO` com o número — o teto é o do hardware (§0.0).

### E3 — Material como o Cycles (Principled) · 2–3 agentes, ficheiros disjuntos

- Ajustes de material, cada um com variante no oráculo Cycles (o Principled com o MESMO valor sobre a MESMA tinta) e
  gate por nome: **Metallic** (F0 = a cor da tinta, o ambiente pré-filtrado já existe), **Roughness** (relacione com o
  Gloss de hoje sem quebrar presets), **Specular/IOR**, **cor do reflexo (tint)**, **Clearcoat** (+ a rugosidade dele —
  o verniz), **Sheen**, **Anisotropia** (o brilho puxado ao longo da estria — a direcção vem do traço ou do gradiente
  do relevo; escolha medindo), **translucidez da tinta fina** (luz que atravessa a película fina e volta do papel).
- ⭐ Decisão de modelo (padrão-ouro, já tomada): o material de que um pincel metálico precisa é **da TINTA, por célula**
  (pintar metal ao lado de tinta comum e misturar no molhado) — canais por célula no grid do Wet Paint, append-only,
  misturados com o pigmento; o ajuste global do cartão fica como multiplicador/override. Se isto muda um formato
  GRAVADO, siga a regra do schema (contado, nunca escolhido: `python3 scripts/schema-recount.py`) e escreva no `ESTADO`.
- Régua: brilho p50/p99/p99,9 e área > 0,3 contra o Cycles por material; folha lado a lado.

### E4 — Os 18 presets parecidos com os do Rebelle · só quando E1–E3 estão bons

- Pastas da fábrica: `assets/brushes/WetPainter/Oil/{Metallic,Dabber,Knife}/` (molde: os presets que já existem lá;
  Dabber/Fan e Oil Bristle já são da fábrica — refaça só se a bancada mostrar que não parecem).
- Método da missão Pincéis (reuse, não reinvente): bancada e exame em `docs/Painter/Pinceis/` (`04_BANCADA.md`,
  `05_REVISAO_CEGA.md`, `ferramentas/exame.sh`, `exame_prancha.py`, `exame_regua.py`); oráculo = as fotos do Rebelle em
  `fixtures/rebelle_bateria/OIL_AND_ACRYLIC/` (gestos, tamanhos, sobre). Revisor cego `sonnet`, ≤ 2 rondas.
  Critério de qualidade: memória `feedback_a_brush_that_passes_the_ruler_can_still_be_useless`.
- Metallic usa o material da E3 (pincel que pinta tinta metálica). Knife precisa da **lâmina que gira** (CAP-01 / C38,
  `docs/Painter/Pinceis/01_CATALOGO.md`) e da lei do rebordo (`out/` e `ferramentas/regua_facas.py` desta pasta) —
  é UM item de motor: faça-o primeiro, uma vez, para as 10 facas; o resto são valores.
- Agrupe: um agente por família (Metallic · Dabber · Knife), ≤ 3 em paralelo, ficheiros de preset disjuntos; o motor da
  faca antes dos valores das facas.

## Fecho (a janela, no fim de tudo)

1. `PH2D_SUITE=fecho` clippy `--all-targets` das crates tocadas; testes por NOME de tudo o que nasceu; provas de mutação
   (agente `mutacao`) nas leis de neutralidade e de «fenda nunca preta». ⛔ Sem suíte completa.
2. Folha final `out/W2_folha.jpg` (hoje | W2 | Cycles) e prancha dos 18 presets × Rebelle.
3. `ESTADO_MISSAO_W2.md` completo + `oraculo/README.md` (números, recusas medidas novas) + linha no `plano.md`.
4. Build `cargo build -p ph2d-host-desktop --profile smoke` e a mensagem final ao dono: curta, PT-BR, sem jargão, o
   que ele consegue FAZER agora + **smoke em passos numerados** (1 o comando com o `cd`; 2 onde clicar, com os nomes da
   tela; 3 o que tem de acontecer; 4 como saber que deu errado) + o que ficou por fazer com o porquê.
5. Handoff de continuação e prompt seguinte **só depois do veredito do smoke**.
