# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W5 … W11 → `main` (2026-10-10)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem
> pusha. **Substitui** o [handoff W10](HANDOFF_INTEGRACAO_line_MiroClone_W10_2026-10-10.md). W5 … W10 (smoke do
> dono OK em todas) nunca foram integradas e a W11 vai por cima, na mesma branch — **integra-se tudo de uma
> vez**; para o detalhe de cada uma valem os handoffs delas (W9 e W10 têm a tabela do foundational e o portão
> de cada onda). Mecanismo da W11: [`../08_ao_vivo.md`](../08_ao_vivo.md) (oráculo, comparação caso a caso,
> medições) e [ADR-0181](../../architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md)
> — **não se repete aqui**. Briefings: [continuação W11](HANDOFF_CONTINUACAO_W11_2026-10-10.md) ·
> [W11 fecho](HANDOFF_CONTINUACAO_W11_FECHO_2026-10-10.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `782f90b69` (a linha foi **rebaseada** sobre o `main` no início desta janela: 109 commits reaplicados, 0 conflitos, nenhum «Solved» do Mergiraf) — **confira se o `main` andou** (`git merge-base HEAD main`); se não, `--ff-only` |
| commits | 119 sobre a base antes dos docs deste fecho (`git log --oneline 782f90b69..HEAD`); a W11 = `1f4876186` … `c4febafbe` |
| diff em `crates/` | `git diff --shortstat 782f90b69..HEAD -- crates/`: 261 ficheiros, +57 932 / −1 334 (inclui fixtures e a Snowball vendorizada) |
| smoke | W5 … W10 **OK do dono**; **W11 por fazer** (§8) — o painel Live e a cena 12 foram vistos só por fotografia (§7) |

O que a W11 trouxe (decisão do dono 10/10: **rede local primeiro, a internet depois sem refazer nada**):

1. **Quadro ao vivo na rede local:** «Live» na fila de cima; entra-se por código de 6 (SPAKE2 + ChaCha20-Poly1305, só LAN, porta 47320, mDNS); funde por registo, presença ≤ 30 Hz, seguir, «trazer todos», cores `Person0..7` — [`08_ao_vivo.md`](../08_ao_vivo.md), ADR-0181. O transporte está atrás de uma porta (a internet virá implementá-la).
2. **Botão da firewall** (Linux; ufw/firewalld por `pkexec`, só a sub-rede local; lembra em `~/.ph2d/live_firewall.txt`).
3. Fim de propósito (`Msg::End`) e sinal de vida (quadro vazio de 20 B a cada 2 s; 10 s de silêncio derruba); versões por ID; lista do painel acaba em «+n».
4. Cena 12 `PH2D_BOARD_SMOKE=12`; o botão «Rascunho ↔ Final» alcança as notas (decisão do dono 10/10).

## 2. Foundational / partilhado tocado — e porque é aditivo

Crates da linha (`ph2d-board-*`, `ph2d-app-board`): ver os handoffs W5…W10. **Novas na W11:** `ph2d-board-sync`
(pura, sem IO: réplica Lamport por registo, equipa, sessão Host/Guest, retrato compactado 193 B/elemento) e
`ph2d-board-net` (SPAKE2/AEAD, TCP, mDNS, `firewall`, `lan_subnets`, `Liveness`, `random_u64` do `OsRng`).
`ph2d-board-model`: `FORMAT_VERSION` 9 → **10** com leitor do 9 (fixture `crates/ph2d-board-model/fixtures/format_v9.bin`,
gravada pelo build ANTES) — o formato do **quadro** nada soma com o `PROJECT_SCHEMA`.

Fora das pastas do quadro — **tudo aditivo** (lista da LINHA INTEIRA, `git diff --name-only 782f90b69..HEAD`):

| onde | o quê | porque é aditivo |
|---|---|---|
| `ph2d-editor-core` `documents_live.rs` (novo) | `LiveRequest` (Host/Join/Discover/Leave/Summon + **FirewallAllow/FirewallRemove no FIM**), `LiveView`, `LiveFirewall`, `LiveView.firewall` | só a família do quadro lê/escreve |
| `ph2d-editor-core` `documents_team.rs` | `Confirm::Version` e `renaming_version` passam de `usize` a `VersionId` (mudança de **tipo**); `Documents::restore_version(id, VersionId)`; `told()` novo | `grep -rn 'Confirm::Version\|renaming_version\|restore_version' crates shells` = só `board_team*.rs` e `documents_team.rs` |
| `ph2d-editor-core` `documents.rs`, `documents_nav.rs`, `documents_fac.rs`, `ids/chrome/documents.rs`, `icons.rs`, `screens/hero/board_*` (39 ficheiros) | painéis e barras do quadro: Live, versões, equipa, facilitação | módulos/ids novos; ids de botão **posicionais acrescentados no fim** de `ALL` |
| `ph2d-editor-core/Cargo.toml` | +3 linhas (deps das crates novas do quadro) | — |
| `ph2d-i18n` | `board_live.rs` (novo: `board.live.*`, `board.live.fw_*` 12, `board.smoke.live.*` 17), `board_team.rs`, `board_facilitate.rs`, `board_fidelity.rs`, `board.rs`, `lib.rs` | chaves novas, só inglês (como todas) |
| `ph2d-tokens` | `ColorToken::Person0..7` + `person-ink` (9 cores de DADO, o mesmo valor nos doze temas; gate `person_colours_read_in_every_theme`); `docs/design/tokens.json` e `docs/design/icons/` (7) | tokens novos no fim |
| `ph2d-panel-registry-init/tests/it/quantas_entradas_tem_cada_painel.rs` | ⚠️ **catraca SUBIDA**: tecto da abertura do painel `tokens` 2 866 → **3 091** px (9 cores × 25 px; o painel é uma lista de `ColorToken::ALL`, nenhuma secção a nascer aberta) — o porquê está no teste | **o integrador confirma contra o `main` do dia** (outra linha que ponha tokens soma) |
| `ph2d-vector` (`record.rs` novo, `scene.rs`, `lib.rs`) | gravação opcional dos desenhos (`rec: Option<Vec<DrawOp>>`, ligada só pelo export do quadro, W8); o re-export passa de `{Glyph, Scene}` a `{Glyph, NormalizedCoord, Scene}` + `FontData` | superconjunto; `rec: None` = comportamento antigo; **não** é o contrato `VectorOp`/`Vertex` (§6) |
| `ph2d-label-fold` | `case_fold` novo (stemmers da W7) | função nova |
| `ph2d-app-audio/src/ui_voice.rs` | `play_chime` + `CHIME` (campainha do cronómetro, W7) | método novo |
| shell `shells/desktop` (`keyboard_board.rs`, `legacy_chrome.rs`, `fase_pointer_subjects.rs`) | delta da linha inteira **+22 −11**; nada na W11 | a catraca `the_shell_only_shrinks` passou |
| `Cargo.toml` (raiz) | `exclude` da Snowball vendorizada; `[profile.dev.package.chacha20/poly1305] opt-level = 2` (a opt-0 a cifra é ~100× mais lenta: 104 ms → ~20 s a entrar num quadro de 57 MiB) — não o desfaça | aditivo |
| `Cargo.lock` | pacotes novos — **colar a tabela da §3** | — |
| `.typos.toml` · `THIRD_PARTY_LICENSES.md` | `^FlateDecode$`, `^t_hach$`, `avatares`, e `saidas/**` dos oráculos / a Snowball excluídos · secção BSD-3 da Snowball | aditivo |
| `docs/architecture/decisions/0181-…md` | ADR novo da linha (§7 sinal de vida e §8 o botão, emendados) | **número PROVISÓRIO** — reconte |
| `project-memory/` | `MEMORY.md` + `reference_topic_measurement_discipline.md` (linhas das ondas anteriores) | uma linha cada |

**Contratos congelados (§6): nenhum tocado** — `node.rs` e `tool.rs` intocados (collision-surface), nem `VectorOp`,
`PROJECT_SCHEMA`, `NodeOp`, `Tool`. **Schemas** iguais à base: `PROJECT_SCHEMA` 187 (tripla 187/13/22), VEC 22,
FLIP 13, `DOC_VERSION` 20, registo de componentes 106/107/107 — **reconte contra o `main` do dia**
(`python3 scripts/schema-recount.py`); `FORMAT_VERSION` do quadro 10.

## 3. Símbolos que podem colidir — `collision-surface.sh` (REFERÊNCIA, nunca prova)

> Medido contra o `main` `782f90b69` em 2026-10-10. O `main` pode ter andado: o integrador **re-corre o
> script pelo caminho absoluto e lê o `main` no ficheiro** (a coluna `base:` é o merge-base).

```text

SUPERFÍCIE DE COLISÃO — line/MiroClone contra main
  merge-base 782f90b69   ·   116 commit(s)   ·   1008 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        187   (base: 187)
      └ tripla do gate               (187, 13, 22)   (base: (187, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 20   (base: 20)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              106   (base: 106)
    ph2d-render (espelho)                 107   (base: 107)
    ph2d-script (espelho)                 107   (base: 107)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0181   próximo livre: 0182
  ⚠ esta linha cria ADR: 0181   — reconte contra o main do dia

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 28 pacote(s) '+name' novo(s):
      "aead"
      "chacha20"
      "chacha20poly1305"
      "cipher"
      "curve25519-dalek"
      "curve25519-dalek-derive"
      "fiat-crypto"
      "flume"
      "hkdf"
      "hmac"
      "if-addrs"
      "inout"
      "mdns-sd"
      "opaque-debug"
      "pdf-writer"
      "ph2d-board-excalidraw"
      "ph2d-board-export"
      "ph2d-board-net"
      "ph2d-board-sync"
      "ph2d-board-words"
      "poly1305"
      "snowball"
      "socket-pktinfo"
      "socket2"
      "spake2"
      "subtle"
      "universal-hash"
      "zeroize"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
  ✗   898 / 700   crates/ph2d-board-words/vendor/snowball/src/snowball/algorithms/english_stemmer.rs
  ✗   731 / 700   crates/ph2d-board-words/vendor/snowball/src/snowball/algorithms/portuguese_stemmer.rs
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

Os dois ✗ são **Snowball 3.1.1 gerado e vendorizado** (da W7): fora do `workspace_file_loc_cap` (o portão passou
verde), excluídos do typos e do `Cargo.toml`. Licença: `THIRD_PARTY_LICENSES.md`. Dependências de rede
(primeira vez no workspace): `cargo deny` ✓.

## 4. Portão batched (1×, 2.ª passagem sobre `782f90b69..9aca24213`) — W11

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | ✓ **18 124/18 124** (9 251 saltados), 114,4 s de nextest, `loadavg` 23,95 no arranque |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ |
| clippy `--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings` | ✓ |
| fmt · typos · machete · `cargo deny` · standalone-optional · workflow-packages | ✓ |
| `censos-da-arvore-combinada.sh` | ✓ **114/114**, 12/12 censos |
| `file_loc_caps` · `the_shell_only_shrinks` · `arch_safe_clamp_only` · `workspace_file_loc_cap` · `architecture_claude_md_cabe_no_orcamento` | ✓ |

Depois do 20/n (só `live.rs` e testes): clippy `-p ph2d-app-board -p ph2d-editor-core --all-targets -D warnings` ✓, fmt ✓, typos ✓.
A 1.ª passagem tinha 1 vermelho (tecto do painel `tokens`) + `unused_mut`, 7 clippy (incl. `RandomState`, proibido pela HR-5 → `random_u64`), fmt, typos — curados (19/n).
**macOS (`--target aarch64-apple-darwin`) NÃO correu**: o `cc` do host é GCC e o `blake3` não compila para esse alvo (o único `target_os` novo é de execução).
História: o portão da W10 foi 17 567/17 567 (base `2a925859d`) — superado.

## 5. O que só o `ship.sh` apanha

O `ship.sh` corre o clippy de paridade com a CI e o `-D warnings` do `check --all-targets`: o laço da linha **não**
os vê (uma função só chamada por testes cai no `-D dead-code`; um `const { assert!() }` dentro de função só
avalia no build). Ambos foram corridos à mão no fecho (§4). **Não corridos:** o cross-check macOS (acima) e a CI
remota. O `nextest-impacted` não alcança `tests/it` das crates não impactadas — o integrador corre o `ship.sh`.

## 6. Auditoria de 2 lentes e prova de mutação da W11 (cada cura com o gate visto VERMELHO)

- REDE: o convidado a voltar com o código velho travava a sala NOVA e ficava 60 s em «reconnecting» (cura `room_is_gone`, pura — gate `a_vanished_host_ends_the_guest_soon_and_its_old_code_never_locks_the_next_room`); o Welcome listava quem já saiu; quem entrava no instante do fecho não recebia o adeus (`Host::close` fecha a entrada antes).
- INTERFACE: o cartão Live alto engolia o clique nos avatares (`a_tall_panel_never_swallows_the_avatar_click`); a frase da firewall afirmava um bloqueio não medido («may»).
- Fotos (16/n): sala fechada ficava PARA SEMPRE nos achados · recém-chegado não via o cursor de quem estava parado (`a_newcomer_sees_where_everyone_already_is`) · «Address (optional)» cortado.

**Mutação (agente `mutacao`): 26 mutações — 20 sangraram à 1.ª, 1 equivalente (G1: `position(id==ver)` = `n`),
5 sobreviventes (E1 fechar sem adeus · E3 código errado/travado ao voltar · F1/F2 o «+n» e a reserva da fila ·
G2 apagar a versão que se renomeia) tapadas e re-mutadas VERMELHAS (exit 1, não erro de compilação) = 25/26 + 1
equivalente** (`c4febafbe`). Lição: um teste de reabrir na MESMA porta perde a corrida numa máquina partilhada
(outros testes abrem 47320+) ⇒ a classificação ficou pura. História: a W10 foi 15/17 → 17/17.

## 7. Números, fixtures e instrumentos

- **Latência de arrasto na LAN real** (2/4/8 pessoas): mediana 0,47–0,64 ms, máx ≤ 1,19 ms; bytes a arrastar
  1/10/50 formas: 60 / 409 / 1 969 B por imagem; sinal de vida 20 B (16 tag + 4) a cada 2 s só calado.
  Instrumentos = testes `#[ignore]` à mão (ADR-0181 §Medições). Convergência: 40 sementes P2P, 25 em estrela com quedas.
- **Oráculo de colaboração:** [`ferramentas/colab_oracle/`](../ferramentas/colab_oracle/) (Excalidraw app `4c00f31` +
  excalidraw-room `03ff435`; tldraw sync 5.5.2): 23 casos, duas corridas iguais.
- **Fotos versionadas:** `docs/MiroClone/ferramentas/cena12_foto.png` (a janela real, `fotografa_cena.sh
  PH2D_BOARD_SMOKE=12 1930 1040`, ufw real detectado) e `cena12_estados/` (a0_abertura … b3_a_ana_fechou) por
  `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-board --lib fotos_dos_estados_da_cena_12 -- --ignored --nocapture`
  (9,1 s; também AFIRMA que a sala fechada sai da lista — mDNS real, por isso é à mão). ⚠️ estão **por rastrear**
  no `git status` desta janela: o commit dos docs tem de os levar (um `git commit -- <paths>` ignora ficheiro não rastreado).
- Das ondas anteriores (valem): oráculos `excalidraw_oracle` (W9/W10; as 3 entradas NOSSAS regravadas),
  `tldraw_oracle`, `facilitacao_oracle`, `mindmap_oracle`; `cena11_w10_foto.png`; fixture `format_v9.bin` (W11),
  a do formato 8 (W9).

## 8. Smokes — o comando exacto (cena 12: duas janelas no MESMO computador, sem firewall)

Cenas `PH2D_BOARD_SMOKE=<n>`: **1** abas · **2** formas · **3** setas · **4** notas · **5** caneta/rascunho · **6**
organizar · **7** guias rodadas · **8** planear (mapa mental) · **9** facilitar · **10** equipa e saída · **11**
fidelidade ao Excalidraw (lições 1–5) · **12** o quadro ao vivo (W11). Todas pelo mesmo binário.

Para o dono (cena 12):

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=12 ./target/smoke/ph2d-host-desktop`
   — abre a «Retro» já como anfitrião, com o painel «Live» aberto e o código à vista.
2. Noutro terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=1 ./target/smoke/ph2d-host-desktop`
   → «Live» (canto de baixo à direita) → clicar «Retro — <nome>» na lista → escrever o código da 1.ª janela → Join.
3. Tem de acontecer: a 2.ª janela abre a «Retro»; arrastar uma nota numa mexe-a na outra; o cursor e a cor do
   outro aparecem; clicar o avatar dele segue-o («Following» com ×); «trazer todos» leva a outra janela; fechar a
   sala na 1.ª avisa a 2.ª.
4. Deu errado se: a lista de quadros fica vazia, o código é recusado sem mensagem, ou o botão da firewall não aparece com o ufw ligado.
   Esse botão (só no Linux, 1.ª janela): pede a senha e diz «Computers on this network can now reach your live
   boards»; «Remove the permission» desfaz. **Só é preciso para um 2.º COMPUTADOR** (não medido: só há esta máquina).

W11 e a cena 12 também: ver as lições 1–6 na própria cena. Cena 11 (W10) inalterada.

## 9. O que fica ABERTO

- **Smoke do dono da W11** por fazer (a descoberta + firewall num 2.º computador só ele mede).
- (c) agrupar as escritas a 30 Hz — só na internet (o transporte virá atrás da mesma porta).
- Avatar «+n» não clicável (≥ ~16 pessoas); reabrir logo após fechar pode cair na 47321 por ≤ ~2,5 s (o `Drop` do
  anfitrião antigo corre numa thread; o endereço mostrado tem a porta e a descoberta acha a certa); uma regra
  apagada à mão na firewall continua a «parecer» liberada (só o root lê as regras).
- Programa fechado a meio (crash) NÃO manda o adeus — os convidados acabam em ~2 s (porta recusada ao voltar).
- macOS cross-check não correu (toolchain). **W12** (votação anónima, modo privado) fora de escopo.
- Das ondas anteriores, inalterados: os de [W10 §9](HANDOFF_INTEGRACAO_line_MiroClone_W10_2026-10-10.md) (foco na faixa, encaixe do meio do lado, vão do losango no 0.18.1) e a tabela do estudo §4.

## 10. Para o integrador

- Ordem: a linha é uma só (W5 … W11); `main` = merge-base ⇒ `--ff-only`; senão `git cherry main HEAD` e o rebase já feito é o ponto de partida.
- Conflitos prováveis (palpite por tipo de ficheiro, não medido): `Cargo.lock` (regenere; confira os pacotes da §3), `ph2d-i18n/src/lib.rs`, `ids/chrome/documents.rs`,
  `ColorToken::ALL` e o tecto do painel `tokens` (§2 — **soma entre linhas**), `.typos.toml`, `THIRD_PARTY_LICENSES.md`,
  `MEMORY.md`. O ADR 0181 reconta-se.
- Depois de integrar: `scripts/ship.sh` (§5). O `cargo-test-narrow.sh` já se embrulha no `ph2d-run.sh`.
- CLAUDE.md §5.1 (Quadro) aponta este handoff e `PH2D_BOARD_SMOKE=1..12`.

## 11. A máquina

Nenhum processo desta linha vivo no fecho. `rm -rf target/*/incremental`: reclamados 38 GB (`debug/incremental` 34 G + `smoke/incremental` 3,9 G; `ci-test` vazio).
`agent-loop-profile.sh` (20 sessões): ✗ paralelismo 1,11/passo · ✗ `test`:`check` 3,8× · ✗ edições pela `Edit` 30 % (esta janela editou por `python3` com `assert`) · ✗ contexto relido 425 mil · ✓ respostas/sessão 200 · ✓ contexto inicial 62 mil.

## 11-bis. O binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, duas vezes; a 2.ª:

```text
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
▸ linha line_miroclone · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.23s
```

## 12. Smoke do dono

W5 … W10: **OK** (W10, 10/10: *«smoke ok.»* — lição 5 e o realce sem o ponto no foco). **W11: OK** (10/10: *«smoke
OK. Vamos integrar ao main.»* — as duas janelas da cena 12 no mesmo computador; o botão da firewall num 2.º computador
continua por medir).

## 13. Integração (10/10, por ordem do dono)

- **Uma linha só** ⇒ a ordem não se mede. O `main` tinha andado (`782f90b69 → e77dfb059`, a UIUX F6–F8, 115 commits):
  rebase de 120 commits, `range-diff` contra o tip de antes (`6d24e83fd`): **109 iguais, 11 mudados**, todos pelos
  conflitos abaixo; zero marcadores introduzidos em qualquer commit. O `foundational-integrate.sh` foi seguido À MÃO
  (passo a passo): ele recusa o primário com `??` de OUTRA linha (`crates/ph2d-tool-painter/tests/sonda_tracos_por_meio.rs`,
  `docs/Painter/NewFeaturesWetPaint/`) — não tocados.
- **Conflitos, pelos estágios:** (1) `board_bar.rs` (W5) — a linha MOVEU `toolbar_rects`/`shapes_rects`/`style_rects`
  para `board_bar_layout.rs` (tecto de LOC) e a UIUX F8 reescreveu os corpos sobre o `float_bar` genérico ⇒ ficou o
  movimento com os corpos da UIUX (o `style_rects_of` da linha também passa pelo `float_bar::strip_rects`, a mesma
  conta: `ROW_MAX` 8, duas linhas); no W6 o `hit_index.register` duplicado sai (o `float_bar::paint_button` regista).
  (2) `icons.rs` (W7, W8) — a UIUX tirou os testes para `icons_tests.rs`: as 7 variantes da linha (`LayoutGrid`,
  `Timer`, `Volume2`, `VolumeX`, `Vote`, `MessageSquare`, `Presentation`) entram na lista de lá pelo slug.
  (3) `CLAUDE.md` (×8, os handoffs W5…W11) — o do `main` com a linha do Quadro do commit. (4)
  `reference_topic_measurement_discipline.md` (W9) — união (as duas entradas). **Número que soma, CONTADO:** a família
  «Como uma RÉGUA mente» tem **206** entradas na árvore combinada; nenhum lado dizia 206 (`main` 205, linha 205).
- **Portão da árvore combinada** (`c93b7367a`): registos (staleness) ✓; `CARGO_BUILD_WARNINGS=deny cargo check
  --workspace --all-targets` ✓; `nextest-impacted` BASE=main **18 189/18 189** (112 s de nextest, `loadavg` 22,6);
  `ph2d-editor-core` inteira **2 146** ✓; `ph2d-panel-registry-init --test it` **133** ✓ (o tecto `tokens` 3 091
  aguenta); `ph2d-host-desktop --test it` **796** ✓; clippy (flags do `ship.sh`) ✓; machete ✓; censos **114/114**;
  standalone-optional ✓; workflow-packages ✓. Dois ✗ curados (`b31d1f940`): o fmt da fusão à mão e o `nin` do
  roteiro do oráculo do QuickShape (latente no `main`, UIUX F7) na lista do `typos`.
- **O primário tinha `project-memory/MEMORY.md` por commitar (de outra sessão)** e a linha muda o mesmo ficheiro:
  o `--ff-only` recusaria. Guardou-se o diff dele (patch + cópia inteira no scratchpad), limpou-se só esse ficheiro,
  `merge --ff-only`, re-aplicou-se o patch e conferiu-se que o diff que ficou por commitar é o MESMO de antes — nada
  dele foi commitado, nada se perdeu. Sem `stash` (a pilha é partilhada entre sessões).
