# CONTINUAÇÃO — `line/MiroClone` W12 «Votar e escrever em privado, ao vivo» (2026-10-11)

> Para a janela que ABRE a W12. Rota `/pd-linha-assumir`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. A linha foi integrada (W5…W11, smoke OK do dono, 10/10) e reposta sobre o `main` com
> `git reset --keep main` em 11/10 (`8f20c60de`): **não há commits da linha por integrar**. A retomada é
> `git cherry main HEAD` → `0` ⇒ `git reset --keep main` de novo se o `main` andou (nunca `rebase`).
> Último handoff de integração: [W5–W11](HANDOFF_INTEGRACAO_line_MiroClone_W11_2026-10-10.md) (§9 = o aberto).

## 1. A escolha do dono (11/10)

*«sim!»* à proposta: **as duas peças da Etapa 2 que faltam, juntas** ([plano §Etapa 2](../02_plano.md),
[`08_ao_vivo.md`](../08_ao_vivo.md)):

1. **Votação anónima entre pessoas** — cada um vota no SEU computador; ninguém vê os votos dos outros
   enquanto dura; o resultado só aparece no fim (o «End for all» de quem conduz, ou o relógio).
   Hoje só existe a votação de UM ecrã, por turnos (W7, [`05_facilitacao.md`](../05_facilitacao.md) §V3).
2. **Modo privado** — cada pessoa escreve as suas notas sem os outros as verem; quem conduz REVELA
   todas de uma vez.

A peça comum (a razão de irem juntas, CLAUDE.md §0.10): **algo que é de UMA pessoa e que só se mostra
a todos quando alguém manda** — um registo com dono e um evento de revelar. Desenhe-o UMA vez.

## 2. O que já existe (não reconstruir — confira o código antes de acreditar numa ausência)

- **Votar num ecrã** (W7): `ph2d-board-edit/src/vote.rs` (`VoteSession{setup, targets, totals, mine,
  voters}`, `VoteSetup`), o estado de sessão em `ph2d-editor-core/src/documents_fac.rs` (`parked`), o
  painel `screens/hero/board_facilitate.rs` (+`_paint`), a barra `board_bar_facilitate.rs`, o resultado
  no modelo `ph2d-board-model/src/organize.rs` (`Org::votes`) e o selo `facilitate.rs` (`vote_badge`,
  `vote_plus`). Gates: `board_bar_facilitate_tests.rs`, cena 9 (`PH2D_BOARD_SMOKE=9`).
- **Ao vivo** (W11): [ADR-0181](../../architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md)
  — a fusão por REGISTO (Lamport por campo) em `ph2d-board-sync` (pura: `replica.rs`, `team.rs`,
  `session.rs` com `Msg`/`Event`, presença em `presence.rs`), o transporte `ph2d-board-net` (porta,
  cifra, sinal de vida, `Host::close`), a ligação `ph2d-app-board/src/live.rs` (+`live_firewall.rs`) e o
  ecrã `ph2d-editor-core/src/documents_live.rs` + `screens/hero/board_live*.rs`. Cena 12.
- ⚠️ **O protocolo é versionado** (`PROTOCOL = 1`); `Msg` só CRESCE no fim da enumeração (o índice da
  variante é o byte que viaja — `Msg::End` = 8). Um voto/nota privada que NÃO pode chegar aos outros
  antes do tempo **não pode ir na réplica partilhada** (ela funde e mostra tudo): decida onde mora o
  que é privado (no anfitrião? cifrado? só o total?) e escreva-o no ADR (emenda à 0181 ou ADR novo —
  o número conta-se contra o `main` do dia).

## 3. Por ordem

0. **Oráculo** (CLAUDE.md §0.9) — a W7 registou que para o voto **nenhum corria** (Scrumlr precisava de
   servidor; Retrospected/Parabol GPL/AGPL; Miro/FigJam proprietários; tldraw/Excalidraw não têm). A W11
   já correu servidores de colaboração em 127.0.0.1 (excalidraw-room) ⇒ **refaça a triagem por
   artefacto** (Scrumlr: licença do que se instala + o servidor dele em local); se nenhum correr, a fonte
   é o Miro MEDIDO (`docs/MiroClone/ferramentas/facilitacao_miro_notas.md`, `capturas_miro/`) e diga-o.
   Casos mínimos: o voto de A não aparece a B antes do fim · quem entra a meio vota · quem sai a meio
   (o voto conta?) · o fim pelo relógio × pelo botão · o total por nota · o privado: B não vê o texto
   NEM o tamanho/posição? (meça o que o Miro esconde) · revelar · desfazer depois de revelar.
1. **Desenho + ADR** (onde mora o privado, o evento de revelar, o que viaja e quando).
2. **Construir** (modelo/sync puros com testes de convergência como os da W11; a ligação; a interface
   no painel da votação e na barra; i18n), **cena 13** (`PH2D_BOARD_SMOKE=13`: o roteador está em
   `crates/ph2d-app-board/src/smoke.rs`, `max_level` 12 → 13) com gate que a CONDUZ por duas janelas
   em duas threads (molde: `smoke_live_tests.rs`) e **fotos dos estados** pela placa (molde:
   `fotos_dos_estados_da_cena_12`).
3. **`/pd-linha-fechar`**: portão batched 1× (inclui `censos-da-arvore-combinada.sh`), auditoria de 2
   lentes (REDE: o que vaza antes do tempo; INTERFACE), mutação (agente `mutacao`), handoff de
   integração W12, §5.1, binário smoke quente. Não integra, não pusha.

## 4. Fora de escopo

A internet (o transporte virá atrás da mesma porta). As pontas de W11 §9 ficam como estão salvo se o
desenho as tocar.

## 5. Lembretes MEDIDOS nesta linha (W11, 10–11/10)

- O `cargo-test-narrow.sh` já está dentro do `ph2d-run.sh`; «0 passaram» = não compilou ou o filtro
  não casa (o nome do MÓDULO de teste pode não ser o do ficheiro — `board_live_tests.rs` é `live_board`).
- Teste que precisa de GANHAR uma porta a outro teste (reabrir na MESMA porta) perde numa máquina
  partilhada: torne a decisão pura e teste-a por caso (foi o `room_is_gone`).
- Um teste ignorado que FOTOGRAFA estados (duas threads, cada uma com a sua placa) achou 3 defeitos que
  nenhum gate via — faça-o antes do fecho, não depois.
- A prova de mutação achou 5 sobreviventes que a auditoria não viu; tape-as e re-mute.
- `git commit -- <paths>` NÃO apanha ficheiros por rastrear: `git add --` primeiro.
- Edite pela ferramenta `Edit` (esta linha usou `python3` demais — a régua do perfil leu 30 %).
- O primário pode ter `project-memory/` sujo de outra sessão: nunca `stash`; é do integrador.

## 6. Ao dono

PT-BR, curto, sem jargão; smoke em passos numerados (comando com `cd` · onde clicar · o que tem de
acontecer · como saber que deu errado); duas janelas no mesmo computador como na cena 12.
