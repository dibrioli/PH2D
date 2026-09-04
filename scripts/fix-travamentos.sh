#!/usr/bin/env bash
# fix-travamentos.sh — aplica a cura MEDIDA dos travamentos de 2026-08-30.
#
# ============================================================================
# O QUE ISTO CURA, E COMO SE SABE
# ============================================================================
#
# A máquina travou 4× em 20 boots (19/08, 22/08 e DUAS vezes em 30/08). Os dois
# de 30/08 deixaram trace completo, e o mecanismo está medido:
#
#   `/mnt/ramtarget` estava montada com **`noswap`**. Essa opção marca as
#   páginas `unevictable`, e a compactação normal do kernel **não migra página
#   `unevictable`** (o `ISOLATE_UNEVICTABLE` só é ligado no caminho
#   `alloc_contig`/CMA). ⇒ elas ficam FINCADAS. Uma página fincada de 4 KB
#   estraga o bloco de 2 MB inteiro em que cai, e esta máquina tem 62 976 blocos.
#
#   · `Unevictable` == uso daquela tmpfs, ao kB (29 GB no travamento).
#   · zona Normal às 18:35:03 — `0*2048kB 0*4096kB`: **68 GB livres e ZERO
#     blocos de 2 MB**. Não foi falta de RAM.
#   · 11:09 → `kcompactd0` preso **495 s** num núcleo. Máquina morta.
#   · 18:35 → `nvidia_uvm` pediu 2 MB contíguos para registar a GPU, levou
#     -ENOMEM, não tratou, e derrubou o kernel 10 s depois.
#
# ⚠️ AUMENTAR o tamanho era a cura ao contrário: já a 4 GB são 17 páginas
# fincadas por bloco, e basta UMA para o estragar.
#
# A cura tem duas metades, e as duas são obrigatórias:
#   (1) a tmpfs volta a ser SWAPPÁVEL  ⇒ páginas movíveis, a compactação volta
#       a funcionar. É isto que cura o travamento.
#   (2) a máquina ganha um swap em DISCO, que nunca teve (só havia zram).
#       Sem ele, (1) sozinha repete o travamento de 22/08: o excedente
#       comprimiria para dentro da própria RAM e o zram encheria a 100%.
#
# Registo: project-memory/project_ramtarget_noswap_fragments_memory_and_freezes.md
#
# ============================================================================
# USO
# ============================================================================
#   sudo bash scripts/fix-travamentos.sh
#
# É idempotente: correr duas vezes não faz mal. Guarda uma cópia do /etc/fstab
# antes de lhe tocar. Recusa-se a correr com um build em curso.
# ============================================================================
set -euo pipefail

log() { echo "[fix] $*"; }
die() { echo "[fix] ✗ $*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "corra como root:  sudo bash $0"

U="${SUDO_USER:-enio}"
id "$U" >/dev/null 2>&1 || die "utilizador '$U' não existe (defina SUDO_USER)."
UIDN="$(id -u "$U")"; GIDN="$(id -g "$U")"

MNT="/mnt/ramtarget"
RAM_SIZE="${PH2D_RAM_SIZE:-64G}"     # pode subir: as páginas passam a ser movíveis
SWAP_SUBVOL="/swap"
SWAP_FILE="$SWAP_SUBVOL/swapfile"
SWAP_SIZE="${PH2D_SWAP_SIZE:-32G}"

mounted() { grep -qE "[[:space:]]$MNT[[:space:]]" /proc/mounts; }
has_disk_swap() { swapon --show=NAME --noheadings 2>/dev/null | grep -qv zram; }

# ---------------------------------------------------------------- 0. guarda --
# ⚠️ A 1ª versão recusava com `pgrep rustc|cargo`, e isso é o teste ERRADO por
# duas razões medidas em 30/08: (a) das 8 árvores só a PRIMÁRIA está armada, e um
# build numa worktree não toca nesta pasta; (b) com muitos agentes os builds
# entram e saem em segundos, então o teste e o `umount` correm numa CORRIDA que
# o deixa vermelho sem motivo — aconteceu 2×.
# O teste certo é o que de facto prediz o `umount`: **quem SEGURA o mount**.
holders() { fuser -m "$MNT" 2>/dev/null | tr -s ' ' '\n' | grep -E '^[0-9]+$' || true; }
if [ -n "$(holders)" ]; then
  log "· seguram $MNT agora:"
  fuser -vm "$MNT" 2>&1 | sed 's/^/    /'
  log "· vou esperar até 120 s por uma janela (o kill não é meu para dar)"
fi

# ------------------------------------------------------------------ 1. rede --
BAK="/etc/fstab.bak-$(date +%Y%m%d-%H%M%S)"
cp -a /etc/fstab "$BAK"
log "✓ /etc/fstab copiado para $BAK"

# -------------------------------------------------------- 2. swap em DISCO --
if has_disk_swap; then
  log "· já há swap em disco — nada a fazer"
else
  if [ ! -f "$SWAP_FILE" ]; then
    # ⚠️ Subvolume PRÓPRIO, de propósito: o `/` desta máquina TEM snapshots
    # (`/.snapshots`), e um swapfile dentro de um subvolume que é snapshotado
    # não pode ser activado. Um subvolume aninhado não entra no snapshot do pai.
    if ! btrfs subvolume show "$SWAP_SUBVOL" >/dev/null 2>&1; then
      btrfs subvolume create "$SWAP_SUBVOL" >/dev/null || die "não consegui criar o subvolume $SWAP_SUBVOL"
    fi
    chmod 700 "$SWAP_SUBVOL"
    log "· a criar $SWAP_FILE ($SWAP_SIZE) — demora um pouco"
    btrfs filesystem mkswapfile --size "$SWAP_SIZE" "$SWAP_FILE" >/dev/null \
      || die "mkswapfile falhou"
  fi
  # prioridade 10 = ATRÁS do zram (100). O zram é o 1º nível (rápido,
  # comprimido); o disco é a rede que impede o wedge de 22/08.
  swapon --priority 10 "$SWAP_FILE"
  grep -qF "$SWAP_FILE " /etc/fstab || \
    echo "$SWAP_FILE none swap defaults,pri=10 0 0" >> /etc/fstab
  log "✓ swap em disco de $SWAP_SIZE activo (prioridade 10, atrás do zram)"
fi

# -------------------------------------- 3. o disco em RAM deixa de fincar ----
if mounted; then
  # espera por uma janela em vez de desistir: com muitos agentes ela abre e
  # fecha em segundos. ⚠️ NADA de `umount -l` — o lazy deixa a tmpfs antiga viva
  # até à última referência, logo a RAM não se liberta e a nova monta por cima.
  ok=0
  for i in $(seq 1 60); do
    if umount "$MNT" 2>/dev/null; then ok=1; break; fi
    [ "$i" = 1 ] && log "· $MNT ocupada — à espera de uma janela (até 120 s)"
    sleep 2
  done
  [ "$ok" = 1 ] || {
    echo; fuser -vm "$MNT" 2>&1 | sed 's/^/    /'
    die "não consegui desmontar $MNT em 120 s. Os processos acima seguram-na —
  se for 'rust-analyzer-p', feche o VS Code e repita. NADA foi alterado no
  /etc/fstab (cópia intacta em $BAK)."
  }
  log "· $MNT desmontada"
fi
# ⚠️ apaga QUALQUER linha antiga deste ponto de montagem antes de escrever a nova
sed -i "\|[[:space:]]$MNT[[:space:]]|d" /etc/fstab
echo "tmpfs $MNT tmpfs rw,size=$RAM_SIZE,mode=0755,uid=$UIDN,gid=$GIDN,nofail 0 0" >> /etc/fstab
mkdir -p "$MNT"
mount "$MNT" || die "mount falhou — o /etc/fstab original está em $BAK"
grep -E "[[:space:]]$MNT[[:space:]]" /proc/mounts | grep -q noswap \
  && die "montou COM noswap — abortado. Reponha: cp $BAK /etc/fstab"
# os symlinks `target/debug -> $MNT/PH2D/debug` já existem nas árvores; a tmpfs
# nasce vazia (como depois de um reboot) e estas pastas fazem-nos voltar a valer.
install -d -o "$UIDN" -g "$GIDN" -m 755 "$MNT/PH2D" "$MNT/PH2D/debug" "$MNT/PH2D/rust-analyzer"
log "✓ $MNT montada SWAPPÁVEL, $RAM_SIZE (páginas movíveis ⇒ a compactação volta a funcionar)"

# --------------------------------- 4. o kernel arruma memória em background --
cat > /etc/sysctl.d/99-ph2d-fragmentacao.conf <<'EOF'
# Escrito por scripts/fix-travamentos.sh — cura medida em 2026-08-30.
# O default do CachyOS é 0: nada desfragmenta em segundo plano, e a conta chega
# toda de uma vez, sob pressão. Foi assim que o kcompactd0 ficou 495 s preso.
vm.compaction_proactiveness = 20
EOF
# THP `always` faz TODO rustc pedir blocos de 2 MB o tempo todo — é a procura que
# encontrava a memória estilhaçada. `madvise` entrega-os só a quem os pede.
echo 'w /sys/kernel/mm/transparent_hugepage/enabled - - - - madvise' \
  > /etc/tmpfiles.d/99-ph2d-thp.conf
sysctl --system >/dev/null
echo madvise > /sys/kernel/mm/transparent_hugepage/enabled
log "✓ compaction_proactiveness=20 e THP=madvise (agora e em todo boot)"

# -------------------------------------------------------------- 5. veredito --
echo
echo "======================== VEREDITO ========================"
printf '  %-34s %s\n' "disco em RAM:" "$(findmnt -no SIZE,OPTIONS "$MNT" 2>/dev/null | head -1)"
printf '  %-34s %s\n' "montada com noswap?" "$(grep -qE "[[:space:]]$MNT[[:space:]].*noswap" /proc/mounts && echo '⛔ SIM' || echo '✓ não')"
printf '  %-34s %s\n' "swap em disco?" "$(has_disk_swap && echo '✓ sim' || echo '⛔ NÃO')"
printf '  %-34s %s\n' "compaction_proactiveness:" "$(sysctl -n vm.compaction_proactiveness)"
printf '  %-34s %s\n' "THP:" "$(cat /sys/kernel/mm/transparent_hugepage/enabled)"
awk '/^Unevictable/{printf "  %-34s %s kB   (tem de ficar ~3 MB)\n", "Unevictable:", $2}' /proc/meminfo
awk '/Normal/{ g=($(NF-1)*2+$NF*4)/1024;
  printf "  %-34s %.1f GB   (2 MB=%s · 4 MB=%s)\n", "memória contígua ≥2 MB:", g, $(NF-1), $NF }' /proc/buddyinfo
swapon --show
echo "=========================================================="
echo
echo "  A vigiar daqui para a frente, com os agentes a trabalhar:"
echo "    bash scripts/ram-build.sh --status"
echo
echo "  O 'Unevictable' NÃO pode voltar a acompanhar o uso do disco em RAM."
echo "  Se acompanhar, o 'noswap' voltou — e o travamento volta com ele."
