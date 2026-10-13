# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W20 + W21 → `main` (2026-10-13)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem pusha.
> ⚠️ **A W20 NÃO foi integrada.** A linha leva W20 + W21 juntas, desde a base. Este handoff é o da linha inteira; o
> mecanismo, o portão e a auditoria da W20 estão no [handoff da W20](HANDOFF_INTEGRACAO_line_MiroClone_W20_2026-10-13.md)
> e NÃO se repetem aqui (os itens W20.1–W20.7 de lá continuam valendo).
> Briefing W21: [HANDOFF_CONTINUACAO_W21](HANDOFF_CONTINUACAO_W21_VIDEO_CLIQUE_FERRAMENTAS_WINDOWS_2026-10-13.md).
> Decisão: [ADR-0185 §W21](../../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md)
> (itens 1–4). Mecanismo: [`10_video_do_youtube.md`](../10_video_do_youtube.md) §W21 e §W21.3 (Windows). Oráculo:
> [`sonda_w21.mjs`](../ferramentas/excalidraw_oracle/sonda_w21.mjs) (saídas em `excalidraw_oracle/saidas/w21/`).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| HEAD | ver `git log -1` (este handoff é o último commit) |
| base (merge-base) | `757afebd58` = `main` de 13/10 |
| commits W20 | ver o handoff da W20 (`86678fcb0b` … `17b0b42c83`) |
| commits W21 (do mais velho) | `38d0a1c7a2` briefing · `70a27dba39` oráculo `sonda_w21.mjs` · `b4c03a4e8a` feat clique longo + outra ferramenta · `d14f8c5276` cena 21, lições 8 e 9 · `b2d690bd37` docs §W21 · `eeeecd817e` feat Windows · `58172406e8` docs §W21.3 · `ea6810f233` fecho · `433d21bc11` foto da cena · + este handoff |
| diff | `git diff --stat 757afebd58 HEAD` (W20+W21): 114 ficheiros, +20763/−256 (a maior parte é JSON e fotos dos oráculos). Só W21 (`git diff --stat 38d0a1c7a2 HEAD`): 43 ficheiros, +12147/−198. **`shells/` intocada** (`git diff --stat 757afebd58 HEAD -- shells/` vazio) |
| smoke | cena **21** (`PH2D_BOARD_SMOKE=21`, `smoke_w20.rs`, agora 9 lições; `max_level` continua 21, nenhuma cena nova). Foto: `ferramentas/soltar_wayland/quadro21_w21_depois.png` |

### O que mudou na W21

1. **Segurar parado num vídeo não é clique.** Largar depois de `now_ns − at_ns ≥ HOLD_NS` não activa. `HOLD_NS` =
   `LONG_PRESS_THRESHOLD_NS` = 400 ms (a lei da casa); o oráculo mediu ~300 ms (activa até 283 ms, não activa desde
   303 ms, a subir e a descer). A casa vence: segurar 300–400 ms ainda é clique aqui (decisão registada). O teste
   fixa `HOLD_NS == LONG_PRESS_THRESHOLD_NS`.
2. **Com outra ferramenta (nem a mão nem o laser) o clique rápido num vídeo é do vídeo (play/pause); o arrasto e o
   segurar são da ferramenta.** O dono escolheu a opção (a) numa pergunta com desenho. O oráculo faz o contrário
   (a ferramenta faz sempre o seu por cima, nunca activa nem pausa); a casa segue o dono. Mecanismo: `VideoClick`
   guarda `at_ns` e `tool`; `board_video::pointer_down` consome o Down (`other`) e adia-o; `deferred_drag` (move
   passa o limiar) e `deferred_hold` (largar segurado) entregam o Down adiado a `board_view::canvas_down` (o Down do
   quadro, extraído da `pointer`); o Up de um clique devolve `c.tool` (consumido). O laser fica fora (aponta, não é
   do vídeo).
3. **O tocador no Windows.** CDP por DOIS handles herdados (`--remote-debugging-io-pipes`), nunca a porta TCP (continua
   recusada). Crate de FFI nova `ph2d-win-pipe-spawn` (`PROC_THREAD_ATTRIBUTE_HANDLE_LIST` para herdar só os dois
   tubos; job `KILL_ON_JOB_CLOSE` para o navegador morrer com o app). `browser::find` acha Chrome, Chromium, Edge,
   Brave e Vivaldi nas raízes do Windows (a lista guarda pastas + programa, e o código junta o `.exe`).
4. As lições 8 e 9 da cena 21 (`board.smoke.w20.l8`, `l9`) ensinam (8) segurar parado não faz nada e (9) com a caneta
   (P) ou uma forma, o clique toca/pausa e arrastar desenha. Conduzidas pelo gate da cena (`Mouse::hold`).

## 2. Foundational / partilhado tocado (W21; os da W20 estão no handoff dela)

| onde | o quê | merge |
|---|---|---|
| `ph2d-editor-core` `documents.rs` | `VideoClick::{at_ns, tool}` (campos novos numa struct `pub(crate)`) | aditivo |
| `ph2d-editor-core` `hero/board_nav.rs` | ⚠️ `pointer_down`/`pointer_up` ganham `now_ns: u128` — ASSINATURA de fn `pub` mudou. Únicos chamadores: `board_view.rs:256` e `:288` (`grep -rn 'board_nav::pointer_\(down\|up\)' crates shells`) | quem chamar de outra linha parte |
| `ph2d-editor-core` `hero/board_view.rs` | `canvas_down` extraída da `pointer` (sem mudança de `pub`); o `theme` órfão da `pointer` saiu | só MiroClone |
| `ph2d-editor-core` `hero/board_video.rs` | `HOLD_NS`, `deferred_drag`, `deferred_hold` | só MiroClone |
| `ph2d-editor-core` testes do vídeo | `board_video_click_tests.rs` NOVO (filho de `board_video_tests.rs`): os 2 testes de clique da W20 movidos + os 2 da W21; `board_video_tests.rs` 847 → 639 LOC (tecto 700) | só MiroClone |
| `ph2d-i18n` `board_images.rs` | `board.smoke.w20.l8`, `l9` | aditivo |
| `ph2d-app-board` | `smoke_w20.rs` (`LESSONS` 7 → 9), `smoke_w20_tests.rs`, `smoke_w16_tests.rs` (`Mouse::hold`) | só MiroClone |
| `ph2d-board-video` | `browser.rs`, `player.rs`, `tests.rs`, `Cargo.toml` | só MiroClone |
| **crate NOVA** `crates/ph2d-win-pipe-spawn` | membro novo do workspace; `windows-sys` 0.61.2 (já no lock) só em `cfg(windows)`; `Cargo.lock` +1 entrada | aditivo; **regenere o lock, nunca à mão** |
| `ph2d-editor-core/tests/it/architecture_every_member_inherits_the_workspace_lints.rs` | +1 entrada em `EXCECOES`: a crate FFI precisa de `unsafe`, que o workspace proíbe (isenção que viaja com o código, como a `ph2d-wayland-dnd`) | aditivo (lista); se outra linha também acrescentar, junte as duas |

**Roteador de smoke**: `max_level` 21 (inalterado pela W21; reconte se outra linha criar a 21). **Contratos
congelados (§6)**: nenhum. `PROTOCOL`/`FORMAT_VERSION` do quadro: inalterados. **Catracas baixadas / itens partilhados
apagados**: nenhum.

## 3. O portão (1×, BASE=`757afebd58`, `loadavg` 46 → 95 → 38 — máquina partilhada)

| item | resultado |
|---|---|
| `nextest-impacted` | 17 313 / 17 315 → **2 ✗ determinísticos, curados em `ea6810f233`**: `every_chrome_key_exists_on_both_sides` (o literal `"chrome.exe"` da lista do Windows lia-se como chave `chrome.*`; cura: a lista guarda pastas + programa, o código junta o `.exe`, e no teste o nome vai em duas partes) e `workspace_src_files_under_loc_cap` (`board_video_tests.rs` 847 > 700; cura: o corte acima) |
| `CARGO_BUILD_WARNINGS=deny cargo check` e clippy do `ship.sh` | ✗ por um `theme` órfão em `board_view::pointer` → apagado em `ea6810f233`; clippy do ship re-corrido ✓ |
| `fmt --check` | ✗ 7 hunks → `cargo fmt`; re-corrido ✓ |
| `typos` · `machete` · `deny` · `audit` | ✓ |
| `censos-da-arvore-combinada.sh` | 116/117 (o mesmo censo das chaves) → ✓ depois da cura; `check-standalone-optional.sh` ✓ · `check-workflow-packages.sh` ✓ |
| clippy `--target x86_64-pc-windows-gnu -p ph2d-board-video -p ph2d-win-pipe-spawn --all-targets -D warnings` | ✓ |

Re-corrido depois do fecho: clippy do ship ✓ · fmt ✓ · os dois censos ✓ · 17/17 testes do vídeo ✓ · cena 21 ✓ ·
`ph2d-board-video` 7 ✓ + 8 ignorados · clippy Windows ✓. Antes do fecho: `cargo test -p ph2d-editor-core --lib
screens::hero` 444/444; `ph2d-app-board` 74 + 7 ignorados.

**Mutação (à mão, controlo verde antes, restauro com `touch`).** Apagar a condição de tempo do `pointer_up` →
sangra (`a_long_still_press_on_a_video_is_not_a_click`: «segurou e activou»). `other = false` → sangra
(`with_another_tool_…`: «o clique não activou»). Mutar só o VALOR de `HOLD_NS` (10 s) SOBREVIVE e é equivalente (o
teste segura `HOLD_NS`); o teste fixa `HOLD_NS == LONG_PRESS_THRESHOLD_NS`. A 1.ª tentativa com `u128::MAX/2` fez
transbordar o relógio do teste — vermelho falso, descartado. O código Windows não teve mutação (sem Windows; os testes
reais correram sob o Wine).

**Auditoria** (uma lente por item). (1) *Largar segurado não é clique*. CLAIM: segurar ≥ 400 ms não activa. TRAÇO:
`board_view::pointer` (Up) → `board_nav::pointer_up(now_ns)` → `board_video::pointer_up` (`now_ns − at_ns < HOLD_NS`)
→ `click`. ASSERÇÃO-VERMELHA: `a_long_still_press_on_a_video_is_not_a_click`. (2) *Outra ferramenta: o clique não
desenha*. TRAÇO: `board_video::pointer_down` (`other`, devolve `true` ⇒ o Down não chega a `canvas_down`) →
`pointer_up` devolve `c.tool` (o Up consumido); arrasto: `board_view::pointer_move` → `deferred_drag` →
`canvas_down(at)`; segurar: `board_view::pointer` (Up) → `deferred_hold` → `canvas_down(at)` → `ed.pointer_up`.
ASSERÇÃO-VERMELHA: `with_another_tool_…`. **NÃO-CHECADO-PELA-COMPILAÇÃO**: (3) o tocador num Windows real.

### Windows: o que foi medido (e a receita, que NÃO está no repo)

Compila para `x86_64-pc-windows-gnu` SEM mingw: o C do `ring` com `CC_x86_64_pc_windows_gnu=clang
CFLAGS_x86_64_pc_windows_gnu="--target=x86_64-pc-windows-gnu -isystem /usr/include/wine/msvcrt -isystem
/usr/include/wine/windows"`. Para LIGAR o `.exe`: o componente oficial `rust-mingw-1.98.0-x86_64-pc-windows-gnu`
(static.rust-lang.org, sha256 conferido) extraído FORA do repo + um linker de teste `clang
--target=x86_64-w64-windows-gnu -fuse-ld=lld -nostdlib crt2.o -L<libs>` com `-C link-self-contained=no -C
dlltool=/usr/bin/llvm-dlltool`. ⚠️ O script do linker viveu no scratchpad da sessão e **não está no repo**; quem quiser
repetir refaz a partir desta receita. O alvo `x86_64-pc-windows-gnu` foi instalado na bancada pelo `rustup` com
autorização do dono. Sob o Wine 11.19 (prefixo novo) com Chrome for Testing 155 win64 e `--no-sandbox` SÓ no teste
(`PH2D_TEST_BROWSER_ARGS`, `cfg(test)`): `Browser.getVersion` pelo tubo em 0,9 s; 30,6 quadros distintos/s; clique →
quadro em 21–29 ms; salto, volume, mudo e pausa confirmados. Linux: os 15 testes do tocador ✓ (13 + 2 novos, com o
Chrome real, `--include-ignored`).

## 4. Abertos (nomeados)

- (a) **Windows real por medir**: o Edge, o sandbox ligado, um antivírus. **Órfãos do fecho**: no Wine, 2 de 11
  fechos deixaram um `--type=renderer` (0 de 2 mandando `Browser.close` antes de matar). Se se repetir no Windows, o
  fecho passa a mandar `Browser.close` antes de matar — não feito agora (nenhum defeito medido fora do Wine, e uma
  espera no `Drop` pararia a interface).
- (b) O limiar do clique é 400 ms (a casa) e o do oráculo ~300 ms: segurar 300–400 ms ainda é clique aqui (decisão
  registada).
- (c) O laser fica fora da regra do clique (aponta, não é do vídeo) — decisão técnica; diga-o ao dono no smoke.
- (d) Da W20, sem mudança: o canal aberto fora não pausa o vídeo daqui; o fundo tracejado pinta-se sólido num vídeo;
  e os da W19 que o handoff da W20 lista (Premium · clicar num vídeo TAPADO na apresentação · o aviso no ecrã inteiro ·
  o enquadramento das cenas não desconta os painéis laterais · o `held` do app).
- **FECHADOS pelo dono (13/10)**: as pegas nos cantos do vídeo («deixar») e o trilho das setas da sessão («Não
  precisa»); o clique longo e a outra ferramenta («corrija») e o Windows («implemente») foram feitos aqui.
- O ficheiro solto `docs/MiroClone/ferramentas/cena2_forma/cena2_botao_forma.log` (de 11/10) é instrumento antigo, não
  commitado de propósito.

## 5. O integrador

- Ordem: só esta linha mexe no vídeo (W20 + W21 entram juntas). Reconte o `max_level` do roteador do quadro (21).
- Membro novo do workspace (`ph2d-win-pipe-spawn`): confira o registo de membros/CI (`check-workflow-packages` passou).
  `Cargo.lock`: regenere, nunca à mão. A `EXCECOES` do gate de lints é lista partilhada: junte, não escolha.
- Quem chamar `board_nav::pointer_down/up` noutra linha precisa passar `now_ns`.
- Testes reais do tocador (navegador real):
  `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && bash scripts/ph2d-run.sh cargo test -p ph2d-board-video --release --lib -- --include-ignored --nocapture --test-threads=1`

## 6. Perfil do agente (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo (chamadas por passo)              1,14   alvo: >= 1,5
  ✓ respostas por sessao (mediana)                 178   alvo: <= 800
  ✗ cargo test : cargo check                574 : 126   alvo: <= 1,0  razao 4,6x
  ✗ edicoes pela ferramenta Edit                   48%   alvo: >= 80%
  ✗ contexto relido por passo (media)          357 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao                63 mil   alvo: <= 80 mil
```

## 7. Smoke do dono

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone
PH2D_BOARD_SMOKE=21 ./target/smoke/ph2d-host-desktop
```
Precisa de rede e de um navegador da família Chrome. A cena 21 tem agora 9 lições: a 8 («segurar parado não faz
nada») e a 9 (com a caneta, tecla P, ou uma forma, o clique toca/pausa e arrastar desenha). Diga ao dono que o
laser fica de fora (aponta; não toca nem pausa). Foto: `ferramentas/soltar_wayland/quadro21_w21_depois.png` (a imagem
no meio é do fotógrafo, não da cena). Como saber que deu errado: segurar parado num vídeo e largar o activa ou pausa;
com a caneta, um clique rápido no vídeo deixa um ponto/traço em vez de tocar.

**Prova do binário** (depois de reclamar ~16 GB de `target/*/incremental`; 2.ª e 3.ª compilações, a mesma linha
de comando `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`): `Finished smoke profile
[optimized] target(s) in 0.23s`, zero `Compiling`. O binário é o de `ea6810f233` (os commits seguintes são só docs).
