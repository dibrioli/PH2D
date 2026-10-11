# Handoff de continuação — Luz 2D, W5f: a onda de velocidade e o smoke do dono (cena 7, abertura) (2026-10-11)

## Regras da linha (PROMPT_LINHAS_RAPIDAS_2026-10-11 §2 — valem para TODA janela daqui em diante)

- **R1 — Janela curta.** Uma onda = uma janela. Passou de ~150 chamadas ou fechou a onda: commit, handoff ≤ 8 KB, PARE.
- **R2 — Edite pela ferramenta `Edit`.** Novo = `Write`; existente = `Edit`. Script só para a MESMA troca em N arquivos, com `assert`.
- **R3 — Edição e teste nunca no mesmo comando.** Edite; `bash scripts/cargo-check-narrow.sh <crate>`; só então teste.
- **R4 — Teste dirigido por NOME no laço** (`cargo-test-narrow.sh <crate> <filtro>`); suíte inteira e `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- **R5 — Nada de `until … sleep` à frente.** > 2 min: `run_in_background` e trabalho independente.
- **R6 — Chamadas independentes na MESMA mensagem.**
- **R7 — Doc > 40 KB nunca se lê inteiro:** `grep -n` + trecho.
- **R8 — Escreva menos.** Comentário: a lei em 1–3 linhas; o porquê no doc.
- **R9 — Tecto por teste: 10 s em `dev`.** Cura: (a) `opt-level = 2` na crate quente; (b) fixture menor que mantém a lei (com mutação); (c) só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`. ⛔ Lei afirmada nunca vai para `#[ignore]`.

> Linha `line/Luz2D`, base `main` `8f20c60de`; a W5e (`HANDOFF_CONTINUACAO_W5e_2026-10-11.md`) continua por
> integrar. Commits da W5f: `59533ec33` (a onda da cena 7) e o deste handoff (o `opt-level` da luz). Nada
> integrado; push e integração só por ordem do dono.

## 1. A onda de velocidade (§3.Luz2D)

`agent-loop-profile.sh 10` ANTES: paralelismo 1,15/passo (alvo ≥ 1,5) · `test:check` 3,7× (≤ 1) · `Edit` 49 %
(≥ 80) · contexto 369 k/passo (≤ 250 k) · respostas por sessão 153 ✓.

**A/B do `opt-level = 2`** (mínimo de 2 blocos intercalados, `--test-threads 1`, diretórios próprios):

| variante | `a_luz_num_ponto_nao_muda_com_a_vista` | `o_rebote_assenta_em_dez_quadros` |
|---|---|---|
| sem otimização | 58,7 s | 98,5 s |
| só `ph2d-light2d-ref` | 84,8 s (nada) | 102,1 s (nada) |
| `-ref` + `-ecs` + `-gpu` + `ph2d-app-light2d` | **20,9 s (2,8×)** | **17,3 s (5,7×)** |

Remontar após 1 edição real na `-gpu`: 1,7 s → 1,5–2,1 s. ⇒ o bloco das quatro crates no `Cargo.toml` da raiz
(fim da lista `[profile.dev.package.*]`, só acrescentar). ⚠️ O palpite do prompt («`-ref`?») estava errado: a
referência na CPU não é o custo.

**Suíte `ph2d-app-light2d` antes × depois:** 592 s (71 testes, `load` 20–30) → 445 s (72, `load` 58–70 —
o relógio só vale a ordem). Os mais lentos DEPOIS: `o_oclusor_fino_faz_a_sombra_do_gabarito` 140 s ·
`as_formas_redondas_nao_saem_em_facetas` 66 · `o_interior_do_opaco_nao_depende_da_posicao_na_grade` 61 ·
`a_sombra_nitida_nao_tem_escada` 42 · os dois de `a_luz_nao_muda_com_a_vista` 32 · `a_borda_da_sombra_e_reta_na_placa` 32.

- ⚠️ **ABERTO — R9 ainda não cumprido.** O `o_oclusor_fino` não desceu com as quatro crates: o custo dele
  mora noutra crate (suspeitos por medir: `ph2d-render`, `naga`/`wgpu-core` — a compilação de pipelines em
  `dev`). ⛔ Armadilha medida: `cargo nextest run … --config profile.dev.package.naga.opt-level=2` REPROVA no
  parser do nextest (exit 2, «use `-- --config`»); o mesmo formato com crates do workspace passou. Use
  `cargo --config … nextest run`. Depois, R9(b) — fixture menor com mutação — nos que ficarem acima de 10 s.
- O `o_oclusor_fino` que REPROVA no `main` `8f20c60de` **passa nesta linha** (antes e depois).

## 2. O smoke do dono — item 1 (cena 7, «não tem desenho Flip») — CURADO

- **Reproduzido em cinco variantes** na tela virtual (HOME limpo · `~/.ph2d` do dono: luz Baixa fixa, área
  Vector · só `prefs` · só `layout` · ultrawide 2560×1080 · área Flip): o traço do Flip aparecia SEMPRE.
  Perguntado, o dono confirmou: **viu as quatro bolas**. O defeito era a cena: os discos de tinta são gêmeos
  das imagens (a régua do espelho) e nada dizia qual era do Flip.
- **Cura** (`59533ec33`): `smoke::onda_a_mao` — um traço de caneta no MESMO objeto Flip, acima da lâmpada
  (48 pontos, largura 0,06 → 0,22 → 0,06 m, cor `COR_DA_ONDA`). Portão `a_onda_da_cena_7_nao_tapa_os_discos`
  (sem GPU): o leque da onda visto da lâmpada não cruza o de nenhum disco nem imagem gêmea, e a largura varia.
  Mutação: a onda de −2,6 a 2,6 m reprova; restaurada, verde.
- Fotos: [`fotos_W5f/cena7_antes_quatro_bolas.png`](fotos_W5f/cena7_antes_quatro_bolas.png) ·
  [`fotos_W5f/cena7_onda_do_flip.png`](fotos_W5f/cena7_onda_do_flip.png).

## 3. O smoke do dono — item 2 («app demorando de abrir») — MEDIDO, cura por fazer (1.ª tarefa da W5g)

**Régua** (`EXIT_AFTER_FRAMES=3`, lançar → 3 quadros → sair, tela virtual, 4 variantes × 3 blocos
intercalados, o mínimo; roteiro derivado do `fotografa_cena.sh` com `FOTO_HOME_DE`):

| | sem luz | cena 7, fábrica (Automático) | cena 7, prefs do dono (Baixa fixa) | cena 8, prefs do dono |
|---|---|---|---|---|
| abrir | **1,0 s** | 5,3 s | 5,8 s | 7,2 s |

**Onde vai** (sonda temporária na shell, retirada): o 1.º `gravar_com` custa **3,5 s** (os seguintes ~60 ms);
`codificar` e `ligar` < 50 ms. É o nascimento do solver (`Solver::novo_com` + o do entorno) a compilar as
pipelines no fio do quadro — e a janela fica presa. Não é a descida do Automático (Baixa fixa é PIOR).

**Cura desenhada (W5g):** montar o solver (e o entorno) FORA do fio do quadro — a shell liga
`LuzDoQuadro` em modo «em fundo»: o `preparar` dispara um fio com `Device`/`Queue`/`Fabrica` clonados que
chama `Solver::novo_com`; o `gravar_com` devolve sem luz até o canal entregar; os portões continuam
síncronos (modo de omissão). Conferir antes: os escopos de erro do `wgpu` são por fio na versão da casa;
dividir os 3,5 s entre `naga` e driver (o `~/.cache/nvidia` já guarda o driver) — se o driver pesar, a
`PipelineCache` persistida em disco encurta o tempo até a luz acender. Portão: com o modo em fundo o 1.º
quadro volta em < 100 ms e o solver chega em N quadros; mutação: o modo em fundo síncrono reprova.

## 4. Item 3 (cena 8, relevo) — sem mudança: F5, ÚLTIMO na fila por ordem do dono.

## Smoke do dono (o que já dá para ver)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. No meio da tela, a lâmpada; acima dela, uma **onda roxa desenhada à mão** — é o desenho do Flip (na lista
   da esquerda: «Traço do Flip»). As duas bolas de CIMA também são tinta do mesmo desenho; as de BAIXO são
   imagens iguais, para comparar.
3. Tem de acontecer: a onda faz uma sombra que sobe para o topo; a borda dela virada para a lâmpada fica
   clara. As bolas de cima e as de baixo têm a mesma luz e a mesma sombra.
4. Errado: a onda sem sombra, ou uma bola de cima diferente da gémea de baixo.
   ⚠️ O app ainda demora ~5 s a abrir com a luz (item 2, próxima janela).
