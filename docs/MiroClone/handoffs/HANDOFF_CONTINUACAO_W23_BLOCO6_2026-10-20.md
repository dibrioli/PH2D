# CONTINUAÇÃO — `line/MiroClone` W23: bloco 6 «funcionalidades novas» construído (B12–B15) — 2026-10-20

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_BLOCO5](HANDOFF_CONTINUACAO_W23_BLOCO5_2026-10-20.md) ·
> lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) · plano e o feito:
> [17_bloco6_plano.md](../17_bloco6_plano.md) (§9 = o que foi feito, M2, recusas). Base = main `100d34c8e0`. Sem integração nem
> push: só por ordem do Enio.

## 1. Feito (decisões do dono 20/10 no cabeçalho do doc 17)

- **B15** `b345e77752` — «Copy to board…» / «Move to board…» no botão direito do quadro (o modal do send, sem o próprio quadro);
  mover = um passo em cada quadro + aviso «Undo» que desfaz os dois. ⭐ Defeito red-first curado: colar noutro quadro apontava o
  frame, o link e o pai do mapa de OUTRO elemento com o mesmo número (`BoardDoc::origin`).
- **B12** `221e5edc32` — o laço (botão no painel da caneta, Ctrl+Alt+arrastar): entra quem tem ≥ 90 % da ÁREA do contorno real
  dentro (comprimento para tinta/setas), exacto por faixas; a porta `select_region` é a do rectângulo também. M2 no doc 17.
- **B13** `401153313a` — o QuickShape do vetor na caneta do quadro (segurar parado, o MESMO `Hold`/`rest_index`); o objecto que
  desenha igual (≤ 1 px) ou a tinta limpa; Ctrl+Z devolve o traço; interruptor «QuickShape».
- **B14** `a32baf7331` `33dbabeee8` — menção como DADO (`Who`, formato 16, protocolo 10); o `@` com ↑/↓; o aviso com «Show»;
  a bolinha no Comments e na aba; «For you». Voto anónimo entre pessoas: já estava feito (W12/W13). ⭐ Defeito ANTIGO curado:
  fio esvaziado sumia sem lápide (`TeamState::parked`).
- **Cena 28** `38bc13c87a` `48d289258c` — fotos `ferramentas/w23_fotos/cena28_{1366,1930}.png` (a foto achou 3 defeitos que o
  gate não via: o Rui chamava «Me» — o nome do computador lê-se na 1.ª pintura —, lições a tapar títulos, frame 4 pálido).

## 2. O que um merge pode partir

- **Formato 16 e protocolo 10** (`board.rs`, `session.rs`): outra linha que mexa no `Board`/`Team`/`Message` reconta o degrau.
- `ph2d-board-edit` → `ph2d-vec-edit` (nova dep, `Cargo.lock`); `quickshape_hold::Hold` público e `rest_index` no lápis do vetor.
- `ph2d-editor-core`: `WidgetStore::arm_list_nav` + o braço das setas em `dispatch/key.rs`; `ContextMenuKind::SendSelection`;
  `board_alert.rs` (o aviso com botão) por cima do `notice`; `Tool::Lasso` no `enum Tool` do quadro.

## 3. Prova

Gates por nome verdes: `paste_elsewhere` (3), `tests::lasso` (5), `tests::smart` (9), `cover` (7), `board_bar::tests` (197 —
inclui `send` 5, `lasso`, `smart`, `team::mention` 6), `interaction` (286), `ph2d-board-model` inteira (135, motivo: formato),
`ph2d-board-sync` inteira (60, motivo: protocolo), `ph2d-vec-edit` quickshape/pencil, `ph2d-app-vec` quickshape, cena 28.
⚠️ NÃO corridos: clippy `--all-targets`, as suítes inteiras das outras crates, provas de mutação (fecho da linha).

## 4. ABERTO (nesta ordem)

1. Smoke do dono da cena 28 (§5).
2. O GATE de M4 (doc 17 §9 «Por fazer» e §10): o oráculo já correu e está guardado; falta o teste que exige o nosso laço com
   `cover = 1.0` igual ao «contain» deles (casos 1–9, 6b; o 10, grupos, é divergência escolhida).
3. Provas de mutação do bloco (cobertura, o zerar das referências alheias, `for_me_at`, `keep_mentions`, o `parked`).
4. O realce a azul das menções no texto do fio (doc 17 §9 «Por fazer»).
5. Os restantes da lista-mãe (`textAlignVertical`, as perguntas ao dono, os nomeados do bloco 5).

## 5. Smoke ao dono (curto)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build --profile smoke -p ph2d-host-desktop && PH2D_BOARD_SMOKE=28 ./target/smoke/ph2d-host-desktop`
2. Siga as quatro notas compridas por baixo dos frames 1 a 4 (laço · desenhar e segurar · a menção do Rui · mandar para o «Plan»).
3. Tem de acontecer: o que cada nota diz.
4. Deu errado se: o laço levar a «Half outside»; a caixa desenhada e segurada não virar rectângulo; não aparecer «Rui mentioned
   you» no topo ao abrir; «Move to board…» não levar as duas notas e a seta para o «Plan», ou o «Undo» não as devolver.
