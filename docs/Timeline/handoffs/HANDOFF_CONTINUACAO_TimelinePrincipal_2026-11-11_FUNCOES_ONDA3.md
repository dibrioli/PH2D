# Continuação — `line/TimelinePrincipal`, plano 17 (funções novas), onda 3 (itens 8 e 9) — código pronto, SEM smoke do dono

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · commits da onda sobre `596b2f5ae1`: `0ac53443da` (itens 8 + 9),
  `0ef726a36a` (acabamento pela foto + cena de smoke + teste do ramo FORA).
- Plano: [`17_plano_funcoes_novas.md`](../17_plano_funcoes_novas.md) — «Onda 3 — desenho medido» (maquetes [B5](../fotos/maquete_B5.png) · [B7](../fotos/maquete_B7.png)).
  Onda anterior: [`..._FUNCOES_ONDA2.md`](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-10_FUNCOES_ONDA2.md) (fechada, smoke OK 10/11).
- Estado: código e testes por nome verdes; fmt + clippy `-D warnings` verdes nos 8 pacotes tocados (`ph2d-preview-drive`, `ph2d-timeline-preview`, `ph2d-timeline`,
  `ph2d-panel-timeline`, `ph2d-signal-census`, `ph2d-signal-ends`, `ph2d-i18n`, `ph2d-host-desktop`); build de smoke ok. **Sem smoke do dono.**

## Item 8 (B5) — quem ouve um sinal

- `ph2d-signal-ends`: tipos novos `Quem`, `Listener`, `SignalListeners`.
- `ph2d-signal-census`: os leitores passam a entregar `(Entity, &str)`; porta nova `listeners(world, fora, name)` (`lib.rs`; `luau.rs::listeners` por dono, `script_may`).
  CAMPOS só contam `Listen`; FORA lê o `Doc`; dedup por (objeto, quem). **Census e fingerprint inalterados.**
- Painel (`ph2d-panel-timeline/src/signal_ouvintes.rs`): `signal_asked` / `set_signal_listeners` / `ouvintes(calcula)`. Clique simples num marcador COM sinal (cartões)
  leva a agulha até ele e abre o cartão «Signal at frame N · nome»: Name (texto só de leitura), linhas «✓ Heard by <objeto> · <componente>» numa caixa, «Nobody listens» (Warn),
  «A script may listen», «+N more» quando não cabe. Nunca acima da barra (`lugar_y`). Fecha com Esc, clique fora, marcador sumido, troca de aba, painel escondido.
  `marker_drag.rs` abre o cartão. A legenda sobe acima da faixa Markers (`faixa_marcadores::pintada` + `legenda`).
- Shell: 1 linha em `fase_snapshots_publish.rs` (só calcula com o cartão aberto).
- Testes: census `tests/ouvintes.rs` (3) e `tests/fora.rs::quem_ouve_de_fora_e_so_quem_ouve_aquele_nome`; painel `tests/it/o_sinal_e_quem_o_ouve.rs` (4, incl. `num_painel_baixo_o_cartao_nao_cobre_a_barra`).

## Item 9 (B7) — Animating · Done

- `ph2d-preview-drive`: `PreviewDrive::driven_held(entity, before, after)` (sem «outra mão»). `ph2d-timeline-preview`: `declare_timeline_writes_held(.., held)`
  (mudou → `driven_held`; igual → `still_driving`). A ponte continua com UMA declaração (gate `the_timeline_declares_what_it_writes` verde).
- Painel (`animando.rs`): ANIMAVEL (cartões + aba Keys), PARADO (Done), `sessao(keys, gesto) -> (a animar, parada)`, `take_ended()`. Pílula «● Animating <clip> [Done]» no início
  da 2.ª fileira da barra (na 1.ª tirava o dono da fileira e 24 px da folha — os testes `os_cartoes` provaram). Moldura azul da fita (`cor_da_moldura`) por cima do vermelho do Auto Key.
  `barra_clips`: os clipes descem de fileira se o 1.º chip não cabe.
- `TimelineState.animating` carimbado pela shell (`fase_timeline_drain`); a ponte parada não aplica o clipe; famílias usam `a_tocar(tocando || animando)`.
  `undo_app_macros.rs::devolve_a_cena` (`take_ended && !jogo` → `apply_project_with(base)` + `arma_o_restauro`).
- Testes: `ph2d-timeline-preview` `held_tests` (2); painel `animando_tests` (2) e `tests/it/animating_done_seam.rs` (3).

## Cena de smoke e fotos

- `PH2D_ANIMACAO_SMOKE` (todos os níveis) ganhou o marcador «land» em 1,0 s (sinal `land`) e Signal Actions no Inimigo, On «land», verbo Show (−7/+7 em `animacao_smoke.rs`).
- Fotos: [`31_animating`](../fotos/31_animating.png) · [`31_lado_a_lado_b7`](../fotos/31_lado_a_lado_b7.png) · [`32_heard_by`](../fotos/32_heard_by.png) · [`32_lado_a_lado_b5`](../fotos/32_lado_a_lado_b5.png)
  (+ `.log`). FOTO-hacks (`PH2D_FOTO_PULAR`, `PH2D_FOTO_PARADO`, `PH2D_FOTO_SINAL`) revertidos.

## Mutações

10/10 pedidas sangram: `held` false; `driven_held=driven`; sem `still_driving`; Click sem `done()`; publica sem `keys`; `sessao` sem ACABOU; `listeners` sem filtro `Listen`;
sem nome no CAMPOS; «Nobody» nunca pinta; `marker_drag` sem abrir o cartão. Mais 2 do ramo FORA que sobreviviam e agora sangram com o teste novo.

## Superfície de colisão (`git diff --stat 596b2f5ae1..HEAD`)

- Crates: `ph2d-panel-timeline` (`animando*`, `signal_ouvintes`, `barra_menus`, `barra_clips`, `faixa_marcadores`, `legenda`, `marker_drag`, `transport_cartoes`, `paint*`, `populate`, `interact`, `event`, `state`,
  `ids/{timeline_barra,timeline_cartao}`, `lib`, `tests/it/*`), `ph2d-preview-drive` (`lib`), `ph2d-timeline-preview` (`lib`, `held_tests`), `ph2d-timeline` (`state`),
  `ph2d-signal-census` (`lib`, `luau`), `ph2d-signal-ends` (`lib`), `ph2d-i18n` (`timeline.rs`).
- Cargo: `ph2d-panel-timeline/Cargo.toml` (+2), `ph2d-timeline-preview/Cargo.toml` (+3); `Cargo.lock` (+2).
- Shell: `render_loop/{fase_snapshots_publish,fase_timeline_drain,timeline_bridge,animacao_smoke}`, `undo_app.rs`, `undo_app_macros.rs`.
- Docs: plano 17, fotos (`maquete_B5/B7`, 31, 32).
- Contadores como DELTA: `PROJECT_SCHEMA` **sem mudança** nesta onda; nenhum canal/registo novo; shell +0 líquido — **no teto 189 040 / 189 041** (`the_shell_only_shrinks` verde):
  qualquer linha nova na shell precisa de corte.
- O que um merge pode partir: quem constrói `Listener`/`SignalListeners` ou lê o retorno dos leitores do census (agora `(Entity, &str)`); quem chama `declare_timeline_writes_held`/`driven_held`;
  quem escreve `TimelineState` (campo novo `animating`); testes `os_cartoes` de altura da barra; a cena `PH2D_ANIMACAO_SMOKE` (marcador «land» — cenas de outras linhas que contem marcadores).

## Diferenças que ficam contra as maquetes (não defeitos)

- Sem painel «Undo History» global (B7, direita). O Name do cartão do sinal não tem caixa.
- A pílula divide a 2.ª fileira com Auto Key/Record/Marker/clipes (o tocar fica no título).

## Abertos

- Depois do Done, o Auto Key ainda grava chave se o artista arrastar (a prévia está parada): **não decidido**.
- Custo do `apply_project_with` no Done: **não medido**.
- O laranja descartado no Done não é um passo de undo (desenho Construct).
- `PH2D_CARDS=0` sem sessão nem cartão (como hoje).

## Smoke (do dono, ainda por fazer)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Na Timeline (aba Keys) clique o clipe «pular». Na 2.ª fileira aparece «● Animating pular  Done» e a área das faixas ganha borda azul. Arraste a agulha na régua 3–4 vezes e solte:
   o Herói muda de pose. Clique Done: o Herói volta para onde estava, a pílula e a borda azul somem. Clique na régua: a pílula volta e o Herói segue a animação.
   Errado se: o Done não devolve o Herói; o Ctrl+Z anda pelas posições da agulha; a pílula aparece na aba Arrange.
3. Role a lista da Timeline até o fim: linha «Markers» com o marcador «land». Clique a alça dele: a agulha vai até lá e abre o cartão «Signal at frame 24 · land» com
   «Heard by Inimigo · Signal Actions». Esc fecha. Errado se: não abre, cobre a barra de cima, ou diz «Nobody listens».

## Próximo

Depois do smoke OK: onda 4 (item 10, container aberto no lugar, C2), sozinha. Onda 5 (itens 11–13) no fim da fila.
