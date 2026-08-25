#!/usr/bin/env bash
# ram-build.sh — põe o que CHURNA da compilação numa RAM disk que NÃO pode swapar.
#
# ============================================================================
# POR QUE ISTO EXISTE, E POR QUE NÃO É O `target-on-tmpfs.sh` RETIRADO
# ============================================================================
#
# O desenho de 22/08 foi retirado por um mecanismo MEDIDO: um `target/` de 33 GB
# em `/dev/shm` acabou como **30 GB de zram** (= RAM, 12,1 GiB comprimidos), com
# o swap a 100% e 61 GiB de RAM livre. Ver docs/DevOps/BTRFS_METADATA_E_SWAP.md §2.
#
# ⚠️ A causa tem NOME: `/dev/shm` é um tmpfs **swappável**. Página de tmpfs sob
# pressão vai para o swap; o swap desta máquina é zram; zram vive na RAM. O
# `vm.swappiness = 150` acelera isso. Não era «RAM a mais» — era RAM a fingir
# que era disco e a comer o próprio colchão de emergência.
#
# O QUE MUDOU: o kernel 6.4 trouxe a opção de montagem **`noswap`** para tmpfs, e
# esta máquina corre 6.18. Com ela a página NUNCA entra no swap: ou está na RAM,
# ou a escrita falha com ENOSPC. O modo de falha de 22/08 deixa de ser possível.
#
# PROVA, medida nesta máquina em 2026-08-24:
#   enchi um tmpfs `noswap` de 24 GiB até 100%  →  swap 12 GB antes, 12 GB depois.
#   Não se moveu um byte. O `dd` parou limpo em ENOSPC, nada mais foi tocado.
#
# ============================================================================
# O GANHO, MEDIDO — e note que é OUTRA MOEDA
# ============================================================================
#
# A nota de 22/08 diz «o preço acima está medido; o ganho nunca esteve» — e está
# CERTA, sobre VELOCIDADE. Ela nunca mediu ESCRITA, porque ninguém perguntava
# pelo desgaste do SSD. A/B nesta máquina (mesmo build, sccache quente,
# `cargo build --tests -p ph2d-anim -p ph2d-timeline`):
#
#   `cargo build --tests -p ph2d-anim -p ph2d-timeline -p ph2d-runtime`, quatro
#   configurações, sccache aquecido ANTES das quatro (senão o 1º arm paga por
#   popular o cache e a comparação mente — aconteceu na 1ª tentativa):
#
#   | configuração                    | escrito no SSD | tempo |
#   |---------------------------------|---------------:|------:|
#   | (a) tudo no disco               |       9,09 GB  |  28 s |
#   | (b) só `incremental/` na RAM    |       7,69 GB  |  27 s |
#   | (c) **target inteiro na RAM**   |   **2,42 GB**  |  24 s |
#   | (d) tudo na RAM, SEM sccache    |       4,39 GB  |  35 s |
#
#   → **(c) = 73% menos escrita.** A velocidade confirma a nota antiga: 28→24 s é ruído.
#
#   ⚠️ **(b) REFUTOU o primeiro desenho deste script.** A intuição dizia que
#   `incremental/` era o alvo — ele é 54% dos BYTES e é reescrito a cada edição.
#   Medido, ele vale **15%**. O que escreve é `deps/`: cada rebuild com fingerprint
#   novo escreve um `.rlib`/`.o` NOVO e o cargo nunca coleta o velho. *A pasta com
#   mais churn não é a pasta com mais ESCRITA.*
#
#   ⚠️ **(d) diz para MANTER o sccache**, e é contra-intuitivo: desligá-lo quase
#   DOBRA a escrita (2,42 → 4,39 GB) e ainda é mais lento. Um cache-hit não
#   compila, e o que não compila não escreve. Ele fica em disco DE PROPÓSITO —
#   é ele que serve os deps depois de um reboot, quando a RAM disk evaporou.
#
# ============================================================================
# O QUE VAI PARA A RAM — e por que NÃO é o `target/` inteiro
# ============================================================================
#
# Medido 2026-08-24, os `target/` vivos desta máquina: 42 G (primário) e
# 73/62/58/39/30 G nas worktrees — **304 GB somados**. Um target inteiro não
# cabe na RAM, e cinco não cabem nem de longe. Então a RAM leva só o que
# reescreve o tempo todo:
#
#   `<target>/debug/`         — 21 GB no primário. É o perfil do inner loop e de
#       todo `cargo build`/`check`/`test` do dia. É aqui que mora o 73%.
#   `<target>/rust-analyzer/` — 11 GB, reescrito em background pelo RA, 100%
#       descartável, e existe só na árvore aberta no VSCode.
#
# ⛔ `ci-test/` e `release/` FICAM NO DISCO, de propósito: correm uma ou duas
# vezes por jornada (gate batched, ship), então o churn deles é baixo — e são
# justamente os que não caberiam. A DIRETIVA_FIM_DE_DIA §2-bis Regra 1 já lhes
# tira o `incremental` com `CARGO_INCREMENTAL=0` (medido: 84 MB, contra 11 GB).
#
# ============================================================================
# QUANTAS LINHAS EM PARALELO — o número é DERIVADO, não escolhido
# ============================================================================
#
#   RAM total ............................ 123 GiB
#   base (desktop + VSCode + agentes) ....  24 GiB   (medido em repouso)
#   por linha ativa (build) ..............  12 GiB   (do pico de 116,7 GiB em
#       cinco linhas, descontando a base e o tmpfs de 33 GB que havia no pico)
#   teto de segurança .................... 105 GiB = 85%
#       (earlyoom dispara a 10% livre; `watermark_scale_factor=200` põe o kswapd
#        a trabalhar cedo. 18 GiB de folga é o colchão real.)
#
#   um `debug/` + RA por árvore ARMADA  ~24 GiB   (21 + 11, com folga de arredondamento)
#
#   24 + 12·N + 24·K  ≤  105        N = linhas a compilar · K = árvores ARMADAS
#
#   | N linhas | K armadas | total | % da RAM |                                  |
#   |---------:|----------:|------:|---------:|----------------------------------|
#   |        2 |         2 |  96 G |      78% | ✅ folgado                       |
#   |        3 |         2 | 108 G |      88% | ⚠️ no limite — só se as 3 não     |
#   |          |           |       |          |    compilarem ao mesmo tempo      |
#   |        3 |         1 |  84 G |      68% | ✅ **a receita**                 |
#   |        5 |         2 | 132 G |     107% | ❌ passa da RAM física            |
#
#   ⇒ **No máximo 2 árvores ARMADAS, e 3 linhas a trabalhar.** Uma 4ª linha é
#     segura — ela só não ganha a RAM disk (escreve mais no SSD, e pronto).
#
#   ⚠️ Isto não limita quantas worktrees EXISTEM (há 10 hoje). Limita quantas
#     estão ARMADAS ao mesmo tempo, que é outra coisa: uma worktree parada não
#     escreve nada.
#
# ⚠️ O tamanho do tmpfs é um TETO DURO, e é essa a rede: passou de 40 GiB, a
# escrita falha com ENOSPC e o cargo reclama — em vez de a máquina entrar em
# reclaim. Falhar alto é o desenho.
#
# ⚠️ Isto NÃO substitui o `ph2d-run.sh`. Ele limita a RAM de um COMANDO; este
# script limita a RAM dos ARQUIVOS. Um teste que aloca 90 GB continua a precisar
# do scope com `MemoryMax`.
#
# ============================================================================
# USO
# ============================================================================
#   bash scripts/ram-build.sh            # arma nesta árvore, MIGRANDO o build atual
#   bash scripts/ram-build.sh --cold     # arma DESCARTANDO o build atual (recompila)
#   bash scripts/ram-build.sh --off      # desarma, DEVOLVENDO o build ao disco
#   bash scripts/ram-build.sh --off --discard   # desarma jogando o build fora
#   bash scripts/ram-build.sh --status   # o que está armado, e quanto sobra
#   bash scripts/ram-build.sh --umount   # desarma TUDO e desmonta a RAM disk
#   bash scripts/ram-build.sh --install-boot   # 1x: RAM disk sobe sozinha no boot
#
#   PH2D_RAM_SIZE=24G bash scripts/ram-build.sh    # outro teto
# ============================================================================
set -euo pipefail

COLD=0
for a in "$@"; do [ "$a" = "--cold" ] && COLD=1; done

MNT="${PH2D_RAM_MNT:-/mnt/ramtarget}"
SIZE="${PH2D_RAM_SIZE:-48G}"
MAX_ARMED=2

die() { echo "[ram-build] ✗ $*" >&2; exit 1; }
say() { echo "[ram-build] $*"; }

[ "$(uname -s)" = "Linux" ] || { say "não é Linux — no-op."; exit 0; }

kver=$(uname -r | cut -d. -f1,2)
kmaj=${kver%%.*}; kmin=${kver##*.}
if [ "$kmaj" -lt 6 ] || { [ "$kmaj" -eq 6 ] && [ "$kmin" -lt 4 ]; }; then
  die "kernel $kver não tem tmpfs 'noswap' (precisa 6.4+). SEM ele este desenho repete o travamento de 22/08."
fi

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
tree="$(basename "$root")"

mounted() { grep -qE " ${MNT} tmpfs " /proc/mounts; }
has_noswap() { grep -E " ${MNT} tmpfs " /proc/mounts | grep -q noswap; }

# ---------------------------------------------------------------- status ----
if [ "${1:-}" = "--status" ]; then
  if ! mounted; then say "RAM disk NÃO montada ($MNT)."; exit 0; fi
  has_noswap || die "MONTADA SEM 'noswap' — isto é o desenho retirado. Desmonte: bash $0 --umount"
  echo
  df -h "$MNT" | sed 's/^/  /'
  echo
  echo "  árvores armadas:"
  found=0
  for d in "$MNT"/*/; do
    [ -d "$d" ] || continue
    printf '    %-28s %s\n' "$(basename "$d")" "$(du -sh "$d" 2>/dev/null | cut -f1)"
    found=$((found+1))
  done
  [ "$found" -eq 0 ] && echo "    (nenhuma)"
  echo
  if [ "$found" -gt "$MAX_ARMED" ]; then
    echo "  ⚠️  $found árvores armadas, o teto derivado é $MAX_ARMED."
    echo "      Desarme uma: cd <árvore> && bash scripts/ram-build.sh --off"
  fi
  free -g | awk 'NR==1{print "  "$0} NR==2{print "  "$0} NR==3{print "  "$0}'
  exit 0
fi

# --------------------------------------------------------- install-boot ----
# Um reboot apaga a RAM disk. Se ela não voltar no boot, os symlinks
# `target/debug -> /mnt/ramtarget/...` ficam PENDURADOS e o cargo falha com um
# erro que não nomeia a causa (`create_dir_all` devolve EEXIST num link morto).
#
# A cura é montar a RAM disk no boot. Ela nasce VAZIA — o build recompila, e o
# sccache (em disco, de propósito) serve os deps. Nenhum dado se perde: o que
# vive lá é, por definição, descartável.
#
# ⚠️ `nofail` é obrigatório: uma linha de fstab que falha bloqueia o boot num
# prompt de emergência. Com `nofail` o pior caso é a RAM disk não subir — e aí o
# script diz o que houve, em vez de a máquina não ligar.
if [ "${1:-}" = "--install-boot" ]; then
  line="tmpfs $MNT tmpfs rw,size=$SIZE,noswap,mode=0755,uid=$(id -u),gid=$(id -g),nofail 0 0"
  if grep -qF " $MNT " /etc/fstab 2>/dev/null; then
    say "já há uma linha para $MNT no /etc/fstab:"
    grep -F " $MNT " /etc/fstab | sed 's/^/    /'
    say "edite-a à mão se quiser mudar o tamanho."
    exit 0
  fi
  say "vou acrescentar ao /etc/fstab:"
  echo "    $line"
  printf '%s\n' "$line" | sudo tee -a /etc/fstab >/dev/null || die "não consegui escrever no /etc/fstab."
  if sudo findmnt --verify --fstab >/dev/null 2>&1; then
    say "✓ /etc/fstab válido. A RAM disk sobe sozinha no próximo boot."
  else
    say "⚠️ o findmnt reclamou do fstab — conferindo:"
    sudo findmnt --verify --fstab 2>&1 | sed 's/^/    /'
  fi
  say "⚠️ Depois de reiniciar, re-arme as árvores: bash scripts/ram-build.sh --cold"
  exit 0
fi

# --------------------------------------------------------------- doctor ----
# Symlink pendurado é o modo de falha nº 1 depois de um reboot. Curá-lo é trivial
# e o erro do cargo não o nomeia — então o script o detecta antes de qualquer coisa.
heal_dangling() {
  local healed=0 l
  for l in "$root/target/debug" "$root/target/rust-analyzer"; do
    if [ -L "$l" ] && [ ! -e "$l" ]; then rm -f "$l"; mkdir -p "$l"; healed=$((healed+1)); fi
  done
  if [ "$healed" -gt 0 ]; then
    say "⚠️ $healed symlink(s) pendurado(s) de um boot anterior — desfeitos (o build recompila)."
  fi
}

# ---------------------------------------------------------------- umount ----
if [ "${1:-}" = "--umount" ]; then
  mounted || { say "não estava montada."; exit 0; }
  say "desarmando todos os links que apontam para $MNT …"
  for d in "$MNT"/*/; do
    [ -d "$d" ] || continue
    t="$(cat "$d/.tree-path" 2>/dev/null || true)"
    [ -n "$t" ] && [ -d "$t" ] && (cd "$t" && bash "$0" --off >/dev/null 2>&1 || true)
  done
  sudo umount "$MNT" && say "✓ desmontada (as árvores armadas tiveram o build devolvido ao disco antes)."
  exit 0
fi

# ------------------------------------------------------------------- off ----
# --off devolve o build ao DISCO. --off --discard joga fora.
#
# ⚠️ A 1ª versão deste script descartava sempre, e isso é assimétrico com o arm
# (que migra): desarmar apagava 13 GB de build compilado sem avisar. Desarmar é
# uma operação de ARRUMAÇÃO — ela não pode custar um rebuild.
if [ "${1:-}" = "--off" ]; then
  DISCARD=0
  for a in "$@"; do [ "$a" = "--discard" ] && DISCARD=1; done
  n=0
  for l in "$root/target/debug" "$root/target/rust-analyzer"; do
    [ -L "$l" ] || continue
    src="$(readlink -f "$l" || true)"
    rm -f "$l"; mkdir -p "$l"
    if [ "$DISCARD" = "0" ] && [ -d "$src" ] && [ -n "$(ls -A "$src" 2>/dev/null)" ]; then
      sz="$(du -sh "$src" | cut -f1)"
      say "  … devolvendo $(basename "$l") ($sz) ao disco …"
      cp -a "$src/." "$l/" || die "cópia de volta falhou — o build ainda está em $src, NÃO desmonte a RAM disk."
    fi
    n=$((n+1))
  done
  rm -rf "${MNT:?}/${tree}" 2>/dev/null || true
  if [ "$DISCARD" = "1" ]; then
    say "✓ $tree desarmada ($n link(s)); os artefatos em RAM foram DESCARTADOS — o próximo build recompila."
  else
    say "✓ $tree desarmada ($n link(s)); o build voltou para o disco intacto."
  fi
  exit 0
fi

# ------------------------------------------------------------------- arm ----
if mounted; then
  has_noswap || die "$MNT montada SEM 'noswap'. Desmonte antes: sudo umount $MNT"
else
  say "montando RAM disk $MNT ($SIZE, noswap) …"
  sudo mkdir -p "$MNT"
  sudo mount -t tmpfs -o "size=$SIZE,noswap,mode=0755,uid=$(id -u),gid=$(id -g)" tmpfs "$MNT" \
    || die "mount falhou."
  has_noswap || { sudo umount "$MNT"; die "o kernel aceitou o mount mas SEM noswap — abortado."; }
  say "✓ montada com noswap (não pode swapar; teto duro em $SIZE)."
fi

heal_dangling

armed=$(find "$MNT" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | wc -l)
if [ ! -d "$MNT/$tree" ] && [ "$armed" -ge "$MAX_ARMED" ]; then
  echo
  echo "[ram-build] ⚠️  JÁ HÁ $armed ÁRVORES ARMADAS — o teto derivado é $MAX_ARMED."
  echo "            Acima disso, RAM(arquivos) + RAM(builds) passam de 85% e a máquina"
  echo "            entra em reclaim. A 4ª linha deve correr SEM RAM disk (é seguro:"
  echo "            ela só escreve mais no SSD). Detalhe: cabeçalho deste script."
  echo
  die "recuse-se a armar a $((armed+1))ª. Desarme uma antes, ou rode esta linha sem RAM."
fi

mkdir -p "$MNT/$tree"
echo "$root" > "$MNT/$tree/.tree-path"

# Espaço livre na RAM disk, em KiB.
ram_free_kb() { df -Pk "$MNT" | awk 'NR==2{print $4}'; }

# PRÉ-VOO: a soma do que vai migrar cabe na RAM disk?
#
# ⚠️ Sem isto o `cp` bate em ENOSPC no meio e deixa meia árvore copiada — que é
# pior que não ter armado, porque o cargo não sabe distinguir um `deps/` truncado
# de um incompleto. Falhar ANTES é o desenho; o ENOSPC do tmpfs é a rede de
# baixo, não a porta da frente.
preflight() {
  local need_kb=0 d
  for d in "$@"; do
    [ -d "$d" ] && [ ! -L "$d" ] && need_kb=$(( need_kb + $(du -sk "$d" | cut -f1) ))
  done
  local free_kb; free_kb=$(ram_free_kb)
  local need_g=$(( need_kb / 1048576 )) free_g=$(( free_kb / 1048576 ))
  if [ "$need_kb" -gt "$free_kb" ]; then
    echo
    echo "[ram-build] ✗ NÃO CABE: migrar precisa de ${need_g} GiB e a RAM disk tem ${free_g} GiB livres."
    echo "            Saídas, em ordem de preferência:"
    echo "              1. desarme outra árvore:  cd <outra> && bash scripts/ram-build.sh --off"
    echo "              2. descarte o build atual: bash scripts/ram-build.sh --cold   (recompila; o sccache serve os deps)"
    echo "              3. suba o teto:            PH2D_RAM_SIZE=64G bash scripts/ram-build.sh --umount && ... (MEÇA a RAM antes)"
    exit 1
  fi
  [ "$need_kb" -gt 0 ] && say "pré-voo: migrar ${need_g} GiB para a RAM disk (${free_g} GiB livres) — ok."
}

link_into_ram() {
  local real="$1" name="$2" ram="$MNT/$tree/$2"
  if [ -L "$real" ]; then say "  · $name já estava na RAM"; return 0; fi
  mkdir -p "$ram" "$(dirname "$real")"
  if [ -d "$real" ] && [ "$COLD" != "1" ]; then
    say "  … migrando $name ($(du -sh "$real" | cut -f1)) para a RAM …"
    cp -a "$real/." "$ram/" || die "cópia de $name falhou — nada foi trocado, o disco está intacto."
  fi
  # ⚠️ `rm -rf` FALHA com "diretório não vazio" quando um processo vivo está a
  # escrever ali — foi o que o rust-analyzer fez na 1ª tentativa. Sem tratar, o
  # script deixa a árvore PELA METADE: `debug` na RAM e `rust-analyzer` no disco,
  # sem dizer. Meio-armado é o pior dos três estados, porque parece armado.
  if ! rm -rf "$real" 2>/dev/null; then
    sleep 1
    if ! rm -rf "$real" 2>/dev/null; then
      rm -rf "$ram" 2>/dev/null || true
      say "  ⚠️ $name está EM USO (processo vivo a escrever) — fica no disco."
      say "     Para levá-lo à RAM: feche o VSCode (ou pare o rust-analyzer) e re-rode."
      return 0
    fi
  fi
  ln -s "$ram" "$real"
  say "  ✓ $name → RAM"
}

mkdir -p "$root/target"
[ "$COLD" = "1" ] || preflight "$root/target/debug" "$root/target/rust-analyzer"
link_into_ram "$root/target/debug"         "debug"
link_into_ram "$root/target/rust-analyzer" "rust-analyzer"

echo
say "✓ $tree armada. Nada mais a fazer — o cargo usa os links sozinho."
df -h "$MNT" | tail -1 | sed 's/^/  /'
echo
say "⚠️ Um reboot apaga a RAM disk e deixa os links pendurados. Re-rode este script"
say "   depois de reiniciar (ou 'bash scripts/ram-build.sh --off' para voltar ao disco)."
