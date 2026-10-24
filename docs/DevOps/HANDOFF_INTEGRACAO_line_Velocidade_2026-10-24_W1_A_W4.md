# Handoff de integração — `line/Velocidade`, W1–W4 (24/10)

> Worktree `Worktrees/line-Velocidade`, merge-base `972f411a6f` (main 23/10). Nada integrado, nada enviado.
> A linha FECHA aqui e PARA (DIRETRIZ §1.5.9). Continuação anterior:
> [HANDOFF_CONTINUACAO_line_Velocidade_2026-10-23_W1_A_W3.md](HANDOFF_CONTINUACAO_line_Velocidade_2026-10-23_W1_A_W3.md).

## 1. Commits

| onda | commit | o quê |
|---|---|---|
| W1 | `f95c707951` | a vez da placa é de quem a PEDIU: o porteiro larga quando o último pedinte acaba (antes: a árvore segurava até o comando acabar) |
| W2 | `9e7a70cd61` | segunda fila na placa integrada para binários de teste; a Luz 2D presa à principal (`PRESOS_A_PRINCIPAL`); o pedido num `write` só e o porteiro guarda o pedaço lido |
| W3 | `f0fcd978a3` | `impacto-preciso.py`: o leitor de fonte alheia (vigia) entra pelo FICHEIRO, não pelo pacote |
| W4 | (este fecho) | a rodada A × B ([VELOCIDADE_DOS_PORTOES.md §2.6](VELOCIDADE_DOS_PORTOES.md)), o roteiro dela ([`ferramentas/rodada_ab_do_portao.sh`](ferramentas/rodada_ab_do_portao.sh)), este handoff |

Desenho e recusas: [A_VEZ_DA_PLACA.md](A_VEZ_DA_PLACA.md) §6–§7 · [VELOCIDADE_DOS_PORTOES.md](VELOCIDADE_DOS_PORTOES.md) §2.5–§2.6.
Superfície: `crates/ph2d-gpu/{src/vez.rs,src/context.rs,src/lib.rs,examples/sonda_onboard.rs,tests/it/a_vez_da_placa.rs}`,
`scripts/{ph2d-run.sh,vez-da-placa.sh,vez-da-placa.prova.sh,impacto-preciso.py,nextest-impacted.sh}`, docs da DevOps.
Nenhum contrato §6, nenhum `Cargo.toml`, nenhum schema.

## 2. Os números (W4)

O diff do Quadro `d10a4ee617`, 3 rondas rodadas × 4 variantes, load 13–310, resumo = o MÍNIMO:

| variante | testes | mín. | principal segurada |
|---|---|---|---|
| antes (`PH2D_IMPACTO=pacote PH2D_PLACA=principal PH2D_VEZ_ARVORE=1`) | 12 463 | 880,6 s | 759–877 s (e uma corrida à espera de outra árvore, 77 timeouts) |
| só `PH2D_IMPACTO=pacote` | 12 463 | 1 012,9 s | 0 s (fila de testes 12–20 s) |
| só `PH2D_PLACA=principal` | 2 419 | 8,3 s | 0 s |
| depois | 2 419 | **8,3 s** | 0 s |

⇒ o relógio de quem corre: **~106×**, todo do seletor (W3). As filas (W1+W2) libertam a principal para as
OUTRAS árvores: ~13–15 min de placa por suíte de 12 k testes que deixam de ser espera alheia. Neste diff a
seleção nova não tem teste de placa — a separação das filas lê-se na coluna da placa, não no relógio.

## 3. Gate do fecho (`PH2D_SUITE=fecho`, load 21 → 8)

| portão | resultado |
|---|---|
| `nextest-impacted.sh` (BASE=main; `ph2d-gpu` mudou ⇒ 17 567 testes) | 17 562 verdes · **5 timeouts, nenhum desta linha** (abaixo) |
| `clippy -p ph2d-gpu --all-targets -D warnings` | verde |
| `vez-da-placa.prova.sh` | verde |
| `impacto-preciso.py --auto-teste` · `--confere` | verde · 11/11 filtros de vigia |

Os 5 timeouts, confirmados sozinhos com A × B das placas (4 corridas, `PH2D_PLACA=principal` × fila, load 8–33):
- `ph2d-light2d-gpu` `g4_custo_independe_das_luzes::*` (2): **4/4 verdes em 0,4–2,0 s** — o timeout da suíte é
  espera/carga (Luz 2D vai à principal).
- `ph2d-tool-painter` `presets_de_fabrica::a_largura_e_a_prometida` · `midias_de_fabrica::cada_preset_parece_a_sua_midia`
  e `ph2d-panel-painter-layers` `seam_estado_escondido::cada_interruptor_que_neutraliza_…` (3): verdes sozinhos,
  **265–274 · 217–234 · 189–222 s, IGUAIS nas duas placas** (são CPU). Em 11/10 eram 24,7 · 14,8 · ~71 s; a faixa
  do `.config/nextest.toml` é 6 × 60 s. ⇒ cresceram ~10× depois da missão Pincéis (fábrica 45, no main 23/10) e
  já estão a 1,35× do tecto sozinhos: **é da linha Painter** (a cura é a fixture, como em 11/10 — nunca subir o
  tecto). Não tocado aqui.

## 4. Para o integrador

1. **Compatível com as árvores por rebasear.** O porteiro velho e o novo trancam o MESMO `ph2d-gpu.lock`; a fila
   de testes (`ph2d-gpu-testes.lock`) só existe para quem já tem o `ph2d-run.sh` novo. Nada a coordenar.
2. **`PRESOS_A_PRINCIPAL`** (`crates/ph2d-gpu/src/vez.rs:53`) nomeia a Luz 2D (`ph2d-light2d-*`, `ph2d-app-light2d`):
   os seus 7 testes vermelhos na Radeon. Quando a linha Luz 2D os curar na integrada, sai da lista (e o
   `a_vez_da_placa` lê-a).
3. `ph2d-gpu` mudou ⇒ a suíte da integração puxa ~17,5 k testes (é dependência de quase todo o render). Os 3 do
   Painter acima podem TIMEOUT sob a suíte inteira: confirme-os sozinhos (§3) antes de culpar esta linha.
4. CLAUDE.md §5: a DevOps não tem entrada; nenhuma linha muda. O `docs/DevOps/README.md` já foi regenerado
   (`bash scripts/doc-index.sh`); se outra linha também o mexer, regenere-o de novo em vez de resolver à mão.

## 5. Aberto (não é da linha)

- O monitor secundário (AOC) está na integrada: testes nela podem tirar-lhe fluidez (nada pendurou em 22 min).
- A integrada é exclusiva entre árvores como a NVIDIA; abrir 2 vagas nela é medição por fazer.
- Seletor (W3): vigia sem `#[test]` puxa o binário inteiro; função de produção que lê fonte e é chamada por
  testes de outro pacote não é seguida (a mesma cegueira da regra antiga).
- **Segunda placa dedicada** (pergunta do dono, 23/10): com a fila de testes, a principal ficou 0 s com testes
  livres; o que nela forma espera é Luz 2D + medições + fotos. Não compensa até a fila da principal voltar a
  formar espera MEDIDA (o amostrador do roteiro W4 lê-a).
- Painter: os 3 gates de fábrica a 1,35× do tecto sozinhos (§3).
