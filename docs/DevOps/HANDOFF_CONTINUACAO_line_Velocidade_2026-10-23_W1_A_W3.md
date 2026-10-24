# Continuação — `line/Velocidade`, W1–W3 feitas, falta a W4 (23/10)

> Para a próxima janela desta linha. Worktree `Worktrees/line-Velocidade`, base `972f411a6f` (main 23/10).
> Nada integrado, nada enviado. A linha fecha com handoff de integração (DIRETRIZ §1.5.9) e PARA.

## Commits

| onda | commit | o quê | prova |
|---|---|---|---|
| W1 | `f95c707951` | a vez da placa é de quem a PEDIU (o porteiro larga quando o último pedinte acaba) | `vez-da-placa.prova.sh` (2 casos vermelhos sobre o main) · `ph2d-gpu a_vez_da_placa` |
| W2 | `9e7a70cd61` | fila de testes na Radeon integrada; Luz 2D presa à principal; o pedido num único `write` + o porteiro guarda o pedaço | prova 23/23 (2 dos pedaços vermelhos sobre a W1) · 7/7 · ponta a ponta 547/547 em 2,2 s a load ~190, zero processos na NVIDIA |
| W3 | `f0fcd978a3` | o seletor de impacto escolhe os VIGIAS por ficheiro | `--auto-teste` 14/14 · `--confere` 271/271 · contagem em 5 diffs reais, 0 exceções |

Desenho, medições e recusas: [A_VEZ_DA_PLACA.md](A_VEZ_DA_PLACA.md) §6–§7 · [VELOCIDADE_DOS_PORTOES.md](VELOCIDADE_DOS_PORTOES.md) §2.5.

## Medição de partida (a que motivou a linha)

18–23/10, transcrições + saídas de tarefas de fundo: ≥ 36 esperas pela placa, ≥ 2 h 45 min, máx. 25 min;
quem segurava era sobretudo o `nextest-impacted.sh` inteiro. Radeon 610M: 5 573/5 580 testes de placa verdes,
os 7 vermelhos todos da Luz 2D (prompt entregue à linha dela pelo dono).

## W4 — o que falta (UMA rodada, intercalada — MEDIR_VELOCIDADE.md)

1. **Medição A × B do tempo de uma suíte**, o MESMO diff do Quadro (`d10a4ee617`, o da tabela da W3), com o
   `loadavg` ao lado de cada corrida. Variantes por interruptor de execução, sem recompilar, intercaladas em
   blocos rodados, resumo = o MÍNIMO:
   - **antes:** `PH2D_IMPACTO=pacote PH2D_PLACA=principal PH2D_VEZ_ARVORE=1`
   - **depois:** sem nenhum dos três
   - (opcional, para separar os ganhos: só `PH2D_IMPACTO=pacote`; só `PH2D_PLACA=principal`)
   - ⚠️ O selo do verde pula a corrida repetida: `PH2D_PORTAO_FORCAR=1` (VELOCIDADE_DOS_PORTOES §2.3).
   - ⚠️ As outras linhas estão a correr suítes (load 100–200 em 23/10 à noite): o relógio acima de `load ~5`
     vale pouco — intercalar é o que torna a comparação justa; registe o load de cada corrida.
   - A compilação `ci-test` da workspace já existe nesta worktree (a W3 a fez).
2. **Gate batched do fecho** (`PH2D_SUITE=fecho`): `nextest-impacted.sh` sobre o diff acumulado + clippy
   `--all-targets` do que mudou (`ph2d-gpu`) + `vez-da-placa.prova.sh` + `impacto-preciso.py --auto-teste`.
3. **Handoff de integração** (§1.5.9). Para o integrador:
   - as worktrees ganham as duas filas e a vez por pedinte só depois do rebase; até lá o porteiro velho e o
     novo trancam o MESMO `ph2d-gpu.lock` (compatíveis);
   - `PRESOS_A_PRINCIPAL` (`crates/ph2d-gpu/src/vez.rs`) nomeia pacotes da Luz 2D: se a linha Luz 2D curar os
     7 na integrada, ela sai da lista;
   - a linha CLAUDE.md §5 (DevOps/velocidade não tem entrada própria; ponha o link no handoff da DevOps).
4. **Ao dono, em 3 linhas sem jargão:** quanto tempo uma linha poupa por fecho, com o número da rodada.

## Aberto (não é da W4, fica no handoff)

- O monitor secundário (AOC) está na integrada: testes nela podem tirar-lhe fluidez (nada pendurou em 22 min).
- A integrada é exclusiva entre árvores como a NVIDIA; abrir vagas (2 árvores juntas) é medição por fazer.
- Seletor (W3): vigia sem `#[test]` puxa o binário inteiro; função de produção que lê fonte e é chamada por
  testes de outro pacote não é seguida (a mesma cegueira da regra antiga).
- Uma segunda placa dedicada (pergunta do dono, 23/10): com as duas filas, só compensa se a fila da principal
  (Luz 2D + medições + fotos) continuar a formar espera — a W4 diz.
