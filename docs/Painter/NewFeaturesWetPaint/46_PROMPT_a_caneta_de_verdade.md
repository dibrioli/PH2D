# 46 — PROMPT: a CANETA de verdade — pressão, inclinação e clique da mesa digitalizadora no PH2D (17/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `bash scripts/btrfs-health.sh`. ⚠️ Este prompt e a pasta
[`../ferramentas/caneta/`](../ferramentas/caneta/README.md) chegaram **por rastrear** (escritos da janela principal):
o 1.º commit da sessão é `git add -- docs/Painter/NewFeaturesWetPaint/46_PROMPT_a_caneta_de_verdade.md docs/Painter/ferramentas/caneta`
(lembre: `git commit -- <paths>` não apanha ficheiro por rastrear). Se o P45 ainda estiver aberto, feche-o antes.
Leia SÓ: este prompt · o README de `ferramentas/caneta/` · o gate
`shells/desktop/tests/it/the_desktop_shell_has_no_pen_pressure.rs` (o levantamento de 31/07 e o que fazer quando
ficar vermelho) · `crates/ph2d-painter-brush/src/dynamics.rs` · o §C27/C28 do P45 (a pressão por pincel). Regras de
13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no fecho, edição provada) · R8 (sondas em `--profile smoke`).

## De onde vimos — MEDIDO hoje na máquina do dono, não suposto

O dono ligou uma **Wacom Intuos5 touch S** (PTH-450). Resultado, camada a camada:

| Camada | Resultado | Prova |
|---|---|---|
| kernel (`wacom`) | ✅ pressão 0..2047, ponta, 2 botões, tilt, distância | `evtest` |
| libinput / KWin | ✅ `TABLET_TOOL_TIP down/up`, pressão normalizada 0..1 | `medicoes_2026-10-17/libinput_…zst` |
| Qt com tablet-v2 (Dolphin) | ✅ clica | o dono |
| **winit 0.30.13 em Wayland nativo (= o PH2D hoje)** | ❌ **nem clica**: só entrar/sair, zero movimento, zero botão | `penprobe_wayland_nativo.log` |
| winit 0.30.13 em XWayland (`env -u WAYLAND_DISPLAY`) | ⚠️ clica, **sem pressão** | `penprobe_xwayland.log` + o dono pintou |

⇒ O KWin 6.7 **não emula ponteiro** para quem não liga `zwp_tablet_manager_v2`: o cursor anda (é do compositor),
mas a janela não recebe nada. Isto **refuta** a premissa de que a caneta já funciona como rato e só falta a pressão
(as notas em `vec_app_bridge.rs` · `pointer_dynamics` e em `painter_canvas_input.rs:14-25`). E a pressão morre num
sítio só: `input_dispatch.rs:211` e `:417` cravam `pressure: 1.0`, e o motor do Painter (`dynamics.rs`, e a pressão
por pincel do P44/P45) **nunca** recebeu outro número.

## Pedido — o produto final, não um MVP

1. **Medir e decidir o caminho primeiro (um lote só, §0.10), com ADR.** Os candidatos, cada um medido e não lido:
   (a) **`octotablet` 0.1.0** ao lado do winit 0.30 (o `RawDisplayHandle` do winit; triagem de licença por artefacto,
   §0.9) · (b) **ligar o `zwp_tablet_v2` à mão** com `wayland-client`/`wayland-protocols` sobre o mesmo `wl_display`
   · (c) **o winit 0.31** (hoje `0.31.0-beta.3`: beta, e é a fundação de janela de TODO módulo — cross-line, conte as
   linhas que tocam no winit antes de o escolher). Critérios: Wayland (a máquina do dono) primeiro; X11/XWayland e
   Windows pela mesma porta se o candidato os der; latência evento → dab; nenhuma amostra perdida entre quadros (a
   mesa reporta ~200 Hz; o traço quer TODAS, com o tempo de cada uma). A decisão técnica é sua (padrão-ouro); o ADR
   regista as recusadas e o preço.
2. **A caneta conduz a UI INTEIRA, não só a tela.** Depois de ligar o tablet-v2, o KWin manda a caneta SÓ por ele:
   menus, cartões, painéis, abas e o canvas de todos os módulos têm de responder à caneta como respondem ao rato
   (a ponta = botão esquerdo; os botões laterais → o que o dono escolher no KDE, por omissão direito/meio). A ponta
   de borracha (`eraser`) entra se a caneta a tiver. Hover (proximidade) move o cursor da shell.
3. **A pressão (e o tilt) chegam a quem já os lê**, por UMA porta: o `PointerEvent` da shell (`pressure`, e o que
   o tilt pedir), o `pointer_dynamics` do lápis do Vector, o Flip (`Min Width`/`Response`), e o Painter nos 5 meios
   (Digital · Watercolor · Impasto · Wet Paint · Composite) com a pressão por pincel do P44/P45. Rato continua `1.0`
   ao byte (identidade das 17 da fábrica, como no P42). A Macro grava o comando RESOLVIDO (ADR-0186): confira que
   a pressão viaja na gravação e o tocar dá o mesmo traço.
4. **O código vive numa crate** (`crates/ph2d-…`, drop-crate); a shell só compõe — a catraca `the_shell_only_shrinks`
   não sobe. O winit continua a dona da janela; a fonte nova de eventos entra na MESMA ordem de quadro que a do winit.
5. **Gates:** o conversor evento-de-mesa → `PointerEvent` com fixtures (proximidade, ponta, botões, borracha, tilt,
   pressão nos extremos); a sequência de pressão do `libinput_…zst` como ORÁCULO por evento (a mesma sequência
   entra e sai igual, por passo); prova de mutação nas leis que importam (a ponta que vira clique, o rato que fica
   `1.0`). O gate `the_desktop_shell_has_no_pen_pressure.rs` vai ficar vermelho — é a intenção dele: **re-calibre os
   sliders do Flip contra pressão real** como ele manda, e então apague-o. Varra as notas que afirmavam «sem pressão»
   (censo por `grep -rn 'pressure: 1.0\|sem pressão\|no pen pressure'`) — nota velha que tranquiliza mente.
6. **Smoke com o dono, numa sessão só:** ele tem a Intuos na mão. Planeie TODOS os traços de uma vez (leve × forte em
   cada meio do Painter, o lápis do Vector, o Flip, um menu e um cartão clicados só com a caneta, e a caneta no modo
   normal — sem o `env -u WAYLAND_DISPLAY`). Passos numerados (§0.8), e grave em paralelo a `libinput debug-events`
   (pede `sudo`; o dono corre o comando) para comparar a pressão que entrou com a que o PH2D usou.

## Fora deste prompt (vai para a lista 26)

Os botões e o anel da MESA (`Pad`: 7 botões + 1 anel), o toque com dedo (a Intuos5 é também touchpad) e a curva de
pressão própria do PH2D (o KDE já aplica a dele — meça se aplica antes de a duplicar).

## Fecho

Handoff de continuação (`/pd-onda-fechar` no fim da onda; a linha continua). Nada de integrar nem de `push`.
