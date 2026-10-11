# CONTINUAÇÃO — `line/motion-value`: ZERAR as pendências do Motion (ordem do dono, 11/10)

> Ordem do Enio (11/10): *«vamos até zerar as pendências»*. As regras R1–R9 do topo do
> [`…_SO_O_ESTADO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_SO_O_ESTADO.md) valem iguais.
> Onda anterior: [`…_COLLIDE_NA_PLACA.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_COLLIDE_NA_PLACA.md).

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, base `8f20c60de` (= `main`),
  HEAD `414aed3c22` + o commit deste ficheiro. **Nada integrado nem enviado.**
- ✅ **Smokes do dono, os dois feitos (11/10):** o `Collide` na placa («smoke OK») e o da W4b — os
  filtros e a lente da camada («parece ok»). Não há smoke do dono pendente.

## Como esta fila se fecha

- **Zerar = cada item sai da lista por UM de três caminhos**, escrito no handoff com o porquê:
  (a) **feito**, com gate e mutação; (b) **fechado por medição** — mediu-se e o custo/defeito não
  existe ou não compensa (tabela ao lado, CLAUDE §5.0 «medido e rejeitado não se reconstrói»);
  (c) **decisão do dono** — só o que é de PRODUTO, perguntado ao Enio em linguagem dele (§0.8).
- **Por FAMÍLIA, num bloco** (CLAUDE §0.10): um plano, um lote de binários, UMA rodada de medição
  intercalada, um portão batched por família. A ordem abaixo é a sugerida, não lei.
- **R1:** passou de ~150 chamadas ou fechou uma família ⇒ commit, handoff de continuação ≤ 8 KB
  (a lista abaixo com o que saiu riscado), PARE. O dono abre a janela seguinte com o mesmo prompt.
- Smoke ao dono por família que mude o que se VÊ (FORMAS + SIMULAÇÃO, fotografado antes, §0.8).

## A fila (cada item com a morada onde está o detalhe)

**A — Defeitos**
1. [BUGS #19](../BUGS_motion_nodes.md): o ângulo do *streak* do `fx.glow` aparece só onde não faz
   nada — `PARAM_GATES` do `fx.glow` + `motion_bridge_params_visible.rs`. Mecanismo já medido.
2. BUGS #18 (no arquivo dos fechados, `archive/BUGS_motion_nodes/`): falta o gate de GESTO pelo
   caminho REAL do rato. Detalhe: [W4b, «ABERTOS NOMEADOS»](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_CICLO_20_W4b.md).

**B — A camada e os filtros** (doc 129 §7; [integração §8](HANDOFF_INTEGRACAO_line_motion_value_2026-10-11_CICLO_20.md))
3. O retângulo da camada usa o `scene_viewport` e não a área VISÍVEL (`cena_visivel`) — medir.
4. A lei de mistura do `fx.filter` sem ficha (oráculo a correr, §0.9) e a rampa com só 2 stops.
5. `Multiplica` dentro da camada pede `DUAL_SOURCE_BLENDING`; hoje `Everything`/`Copies` vão ao
   Vello (`+0,4–1,9 ms`). Primeiro: a feature existe nos adaptadores que importam?
6. Os filtros de TODOS os fios aplicam-se à camada inteira (`from_graph` concatena) — a cura (uma
   camada por fio) foi RECUSADA no item 4 do doc 129; nenhuma cena o tem. Candidato a (b)/(c).

**C — Desempenho (placa e CPU de reserva)**
7. A cadeia pequena com muitas varreduras no grupo único ainda custa `1,7–6×` a CPU — o `scan` por
   subgrupos ([doc 130 §3.1](../130_o_passe_na_placa.md)); e o passe recria grupos de ligação e
   uniforms por cozimento (não usa as `Ligacoes`).
8. O `Boundaries` (rota híbrida) — o sufixo pode precisar de cada tique da fronteira; e o
   `cook_taps_only` coze todas as tomadas ([`…_SO_O_ESTADO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_SO_O_ESTADO.md)).
9. `opt-level` da `ph2d-contact` — medir com a máquina livre (R9).
10. (W3b) a rota de placa do `SpriteRenderer` sensível na silhueta; `4 096` formas = 4 páginas;
    U2 (px do projecto) — [integração W3b](HANDOFF_INTEGRACAO_line_motion_value_2026-10-11_CICLO_20_W3b.md).
11. `Bezier`/`Tint` OKLCH caros na CPU de reserva; a fita acima de `32 MiB` (ciclo 19, W4b).
12. A troca de rota a meio sem engasgo pediria o estado do dispositivo na CPU (`e30104881`,
    integração §9) — medir se o engasgo de `0,115 ms`/tique se vê.

**D — Produto (perguntar ao dono o que for dele)**
13. Uma linha de COR inteira num gesto; vida e dano pedem dois objectos (ciclo 19, W4b).
14. O colisor escrito PELO NOME num text param (`RECUSA_COLISOR`) continua na CPU — levá-lo à placa
    é o mesmo kernel do doc 130 com a declaração por coluna.

**E — Documentação**
15. Docs `> 40 KB` por partir com `python3 scripts/doc-split.py` (o `115_…` tem `~170 KB`).

## Fecho de cada janela

DIRETRIZ §1.5.9 pelo `verificador`: `nextest-impacted` sobre o diff, clippy `-D warnings`, fmt,
censos, `machete`, tectos de LOC (⚠️ `ph2d-eval-motion/src/lib.rs` `698/700` e
`ph2d-gpu-cook/src/lib.rs` `695/700` — campo novo vai para um irmão), e o §10 do handoff de
integração ganha uma linha por família fechada. Sem integrar e sem push.
