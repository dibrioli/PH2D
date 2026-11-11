# Medição «depois» (B08c / B08.6) — NÃO MEDIDA nesta ronda: a placa esteve com outra linha

- **HEAD:** `1b4427f4b711` (`line/PainterWatercolor`), com **26 ficheiros `M` de outros agentes** na árvore (ph2d-tool-painter:
  `escrita_direta`, `warp/*`, `watercolor_secagem`, `paste_patch*`, testes; ph2d-app-painter: `composite_smoke.rs`) — o
  binário compilado leva-os.
- **Binário:** `CARGO_PROFILE_SMOKE_STRIP=none bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`
  (23:45, verde), copiado para `target/mede-depois/ph2d-host-desktop` (outro agente não o troca a meio).
- **Placa:** a mesma do ANTES (Radeon 610M, `VK_DRIVER_FILES` = só RADV, WSI sw). Nenhum lançamento chegou a medir.

## O que aconteceu (medido)

1. **1.ª corrida (23:45–23:55, 4K papel):** a `line/InputMap` tinha a vez da placa (`cargo run … --profile smoke`, pid
   2033164, desde 23:43). Os 5 primeiros lançamentos (omissão digital/water/impasto/wet, Round Mixing) esperaram a vez e
   sairam `win NENHUMA`; o 6.º (Wet Wash Mix) abriu às 23:51 (a vez chegou ao fim de 345 s); entre esse lançamento e o
   seguinte o porteiro **soltou a placa** e a InputMap reentrou (pid 2114981, 23:52:30) — Wet Liner `win NENHUMA`.
   Causa: o `mede_pen_up_4k.sh matriz` lança um app por pincel e cada app pede/solta a vez sozinho; entre dois
   lançamentos outra árvore entra.
2. **Cura do roteiro (este commit):** `mede_pen_up_4k.sh matriz` pede a vez com `scripts/vez-da-placa.sh` ANTES do Xvfb
   e segura-a até ao fim da matriz (o `$PPID` é o próprio roteiro — o modo «uma medição, do princípio ao fim» do
   `A_VEZ_DA_PLACA.md`, sem `PH2D_GPU=1`).
3. **2.ª corrida (23:54–00:13):** pedida a vez às 23:54:35; a InputMap segurou-a até ao fim do prazo deste agente
   (o app `2115007` vivo a 00:12 com 20 min). Nenhum lançamento. Corrida parada (processos só meus; máquina limpa).
   Não se forçou (A_VEZ_DA_PLACA §1: duas linhas na placa penduraram o driver em 14/09).

## Como correr (o próximo agente: ~25 min com a placa livre)

```
W=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
CARGO_PROFILE_SMOKE_STRIP=none bash $W/scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
rm -f $W/target/mede-depois/ph2d-host-desktop      # o roteiro copia o binário novo
timeout 1800 bash $W/scripts/ph2d-run.sh bash $W/docs/Painter/ferramentas/corre_depois_b08c.sh
python3 $W/docs/Painter/ferramentas/mede_pen_up_4k.py --matriz $W/target/mede-depois/matriz $W/docs/Painter/missao_4k/medicoes/depois > /tmp/depois.md
python3 $W/docs/Painter/ferramentas/compara_antes_depois.py $W/docs/Painter/missao_4k/medicoes/ANTES.md /tmp/depois.md
```

Grupos, por prioridade: 4K papel (15 pincéis, o ANTES inteiro) · 4K branco, 1080p papel, 4K blocos e 1024² blocos com
os 7 do núcleo (omissão ×4, Watercolor Basic, Filbert, Express Oil). Antes de lançar: `cat $XDG_RUNTIME_DIR/…` não
basta — veja no log do `ph2d-run.sh` se aparece «⏳ a placa está com outra árvore»; se aparecer, a espera é a da
InputMap (ou de outra linha), não do app.

O `compara_antes_depois.py` escolhe o «antes» de cada célula: a matriz principal (papel, 1024² blocos), o B08b
(4K/1080p branco) e a continuação do B08b para 4K blocos (load ~1; o B08b 4K blocos era load 11). Fases e alvos =
os do ANTES.md (traço p95 ≤ 16,6 · toque/soltar/+1 s/Ctrl+Z ≤ 33,3 · parado ≤ 5 % · Wet ≥ 10 img/s).

`ANTES_DEPOIS.md` não foi escrito: sem números «depois» seria uma tabela vazia.
