# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor` no `main` (2026-10-17)

> Ordem do dono (17/10): 7.ª linha da rodada, depois de Motion, Vector, CodeEditor, Macros, Luz 2D e MiroClone.
> O que a linha fez: [P42 — a lei da tinta](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-16_P42_LEI_DA_TINTA.md)
> e [P43 — o pó fino](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P43_PO_FINO.md); antes delas, P35–P41
> (a família seca contra o Rebelle, rampas do Shape, Flat/Fan fora, Shape Color no Wet Paint), cada uma no seu handoff.
> Aberto: o §6 de cada um + [26](../NewFeaturesWetPaint/26_ABERTOS.md).

## §1 — O que entrou (71 commits, prompts 35–43)

- **P42**: o Glaze é do PINCEL (plano `vela` na grade; a camada que desce leva a sua água) e 5 pincéis de tinta de fábrica
  (Ink Bullet, Dip Pen, Ink Brush, Marker Bullet, Marker Liner) — fábrica 12 → 17.
- **P43**: o pó do pincel (`Knob::Dust`, `KNOB_COUNT` 56, só desta linha) e o Airbrush de fábrica — fábrica 18. O pó vai no mapa
  `engine` do preset; `SCHEMA` do preset fica 1.
- P41: a macro leva a Shape Color (`macro_estado.rs`); P39: `BrushShape::{Flat, Fan}` saem do motor; P38: o Alpha das rampas sai.

## §2 — A rodada

- Merge-base `0aab2c1cb6`; o `main` andou 419 commits. 22 ficheiros mexidos pelos dois lados, quase todos em
  `ph2d-tool-painter`/`ph2d-wet-paint` (a trilha e os instantâneos da Macros W5c–W7). Lista: `git diff --name-only` das duas pontas.
- Rebase de 71 commits, 2 conflitos, ambos pelos estágios: `stroke.rs` (a linha apaga `dir_x/dir_y`, o `main` põe
  `#[derive(Clone)]` — ficou a união) e `macro_estado.rs` (a linha acrescenta a Shape Color por cima do estado que a Macros
  reescreveu — as duas linhas ficaram). Marcadores varridos em cada commit (incluindo `|||||||`): zero. Comparação commit a
  commit com o original (multiconjunto `+/-` por ficheiro, controlo positivo 870): só 3 diferem, de propósito (§3).
- Nenhuma colisão de mesmo-símbolo; nenhum contrato congelado; sem ADR; `Cargo.lock` sem pacote externo novo.

## §3 — O que só a árvore combinada achou (curado NO commit rebaseado, o `bisect` não cruza vermelho)

Cada linha estava verde; juntas não compilavam. Achado por `git rebase main --exec 'cargo check …'` (a árvore de cada commit):

1. P38 — `macro_estado/tela.rs` (Macros W6c) gravava o `alfa` das rampas, que o P38 apagou. A chave `alfa` sai da escrita
   e da leitura; uma macro gravada antes a traz e o leitor ignora (campo por nome).
2. P39 — o `Clone` do `Engine` (`painter/instantaneo.rs`, Macros W5c) listava `shape`, que o P39 apagou.
3. P42 — o mesmo ficheiro não contava o plano novo `vela` em `Grid`/`GridSnapshot::bytes_retidos`. Sem `..` por desenho: entrou.

## §4 — Números (árvore combinada, contados)

`censos-da-arvore-combinada.sh` 121/121 (12 de 12 censos correram) · `foundational-integrate.sh`: `check --workspace
--all-targets` com avisos como erro ✓ · `nextest-impacted` **18 881** testes (18 879 verdes + 2 TIMEOUT; antes, a MiroClone fechou
17 754/17 754) · `PROJECT_SCHEMA` 190 (a linha não toca; tripla do `main` intacta) · registo de componentes sem mudança ·
`KNOB_COUNT` 56 · pincéis de fábrica 18.

⚠️ Os 2 TIMEOUT foram **carga**, não código: `load` 74 (outras janelas), `achatamento_tests::o_escorrimento_do_s_nao_se_corta_em_rectangulos`
(180 s) e `mistura_camadas_tests::n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto` (360 s). A sós, a `load` 16–26:
16,8 s e 50,1 s, verdes. O 2.º já mora na faixa pesada do nextest; o 1.º **não** — candidato à família
[FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) (duas escritas: a tabela e a faixa).

## §5 — Para o Enio smokar

Os smokes são os do P42 (§8 do handoff) e P43; o binário do `main` não foi recompilado depois da fusão.

## §6 — Aberto (verbatim dos handoffs, resumido)

- P43: a 20 px a textura do pó lê 28,7 contra 22,5 do Rebelle; a pressão ESMAECE no Rebelle e ESTREITA em nós (medir se o
  pincel tem pressão → opacidade); a gota é da CÉLULA (razão da grade > 1 ⇒ 2·razão px); o custo do hash por célula não
  medido em intercalado.
- P42: o azul sobe ao secar no cruzamento (escoamento do vizinho molhado); a faixa de cima do Ink Brush; o chão do marcador
  0,025–0,043; a foto `marker` do Rebelle; o plano `vela` não escoa com a água; custo do Glaze 1 em cruzamento (42 ms contra 15).
- B13/B14 (a Shape Color no pincel salvo — a mesma família do que o preset leva).
