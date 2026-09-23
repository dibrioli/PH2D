#!/usr/bin/env bash
# Prova de mutacao da CERCA DO PLANO — a cura do panico do dono de 2026-09-21
# (`index out of bounds: the len is 196608 but the index is 196608`, que e'
# `4 x 49152`: a lista de faces que chegou ao registo tinha mais faces do que a
# topologia do plano).
#
# A rede cobre TRES camadas, porque a cura tem tres:
#   * a LEI    — `Topologia::descreve`, a pergunta pelas duas contagens;
#   * a PORTA  — `Topologia::payload`, que a responde face a face e RECUSA;
#   * o DEVICE — `upload_tinta_at`, que desarma em vez de estourar.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO nos mesmos QUATRO pontos do irmao
# `muta_a_metade_visivel.sh` (ancora unica · a mutacao compila · N > 0 testes
# correram · a corrida limpa esta' VERDE), e cada um deles ja' mentiu nesta casa.
#
# ⛔ A corrida do device precisa de ADAPTADOR: chame por
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_a_cerca_do_plano.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
# ⭐⭐⭐⭐ **PRE-VOO DAS ANCORAS (`MUTA_SO_ANCORAS=1`)** — confere que cada
# ancora casa EXACTAMENTE uma vez, sem correr um unico teste.
# ⛔⛔ Ele existe porque `cargo fmt` (ou um corte de ficheiro) reescreve a
# indentacao de uma ancora, ela passa a casar ZERO, e **isso le-se exactamente
# como uma mutacao que SOBREVIVEU** — com o custo de uma corrida inteira para
# descobrir. O pre-voo custa segundos e corre-se DEPOIS de todo `fmt`.
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
COL=crates/ph2d-mesh-colors/src
REN=crates/ph2d-mesh-render/src
BK=$(mktemp -d)
cp -r "$COL" "$BK/col"; cp -r "$REN" "$BK/ren"
restore() {
  rm -rf "$COL" "$REN"
  cp -r "$BK/col" "$COL"; cp -r "$BK/ren" "$REN"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO —
  # a licao esta' em `project-memory`.
  find "$COL" "$REN" -name '*.rs' -exec touch {} +
}
trap restore EXIT

# ⭐ Duas corridas porque as tres camadas nao cabem num pacote: as seis
# primeiras mutacoes sao puras (`ph2d-mesh-colors`) e a do device precisa do
# gate `#[ignore]` da `ph2d-mesh-render`. O rc e' a OU das duas.
corrida() {
  local a b ra rb
  a=$(cargo nextest run -p ph2d-mesh-colors --lib 2>&1); ra=$?
  b=$(cargo nextest run -p ph2d-mesh-render --test it --run-ignored all \
        -E 'test(tinta_no_device)' 2>&1); rb=$?
  printf '%s\n%s\n' "$a" "$b"
  [ "$ra" -eq 0 ] && [ "$rb" -eq 0 ]
}

# A populacao honesta e' a que o `nextest` diz ter CORRIDO — o `running N tests`
# do libtest CONTA os `#[ignore]`.
populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
limpa=$(corrida); rc_limpo=$?
verde=$(printf '%s' "$limpa" | populacao)
echo "VERDE antes: $verde testes correram"
[ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }
if [ "$rc_limpo" -ne 0 ]; then
  echo "ABORTO: a corrida limpa esta' VERMELHA -- um placar tirado daqui e' fabricado."
  printf '%s' "$limpa" | grep -E '^ *(FAIL|Summary)' | tail -8 | sed 's/^/      | /'
  exit 2
fi
fi

sangram=0; total=0
muta() { # ficheiro  ancora  substituto  nome
  local f="$1" agulha="$2" subst="$3" nome="$4"
  if [ -n "$FILTRO" ] && ! printf '%s' "$nome" | grep -Eq "$FILTRO"; then return; fi
  if [ -n "$SO_ANCORAS" ]; then
    total=$((total+1))
    local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
    if [ "$n" -ne 1 ]; then echo "  ✗ ANCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a ancora casou $n vezes (esperado 1)"; return; fi
  python3 -c '
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,"w").write(s.replace(a, b, 1))
' "$f" "$agulha" "$subst"
  touch "$f"
  local out rc corridos
  out=$(corrida); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutacao nao compila"
  else
    corridos=$(printf '%s' "$out" | populacao)
    if [ "$corridos" -eq 0 ]; then
      echo "  ABORTO [$nome]: zero testes correram — as ultimas linhas foram:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A PORTA: o payload recusa em vez de estourar ─────────────────────────
muta "$COL/topo.rs" \
  '            if f >= self.faces() {
                out.clear();
                return false;
            }
' '' \
  'N1 payload: a recusa das faces a MAIS desaparece — o dab que refina'

muta "$COL/topo.rs" \
  '        if vistas != self.faces() {
            out.clear();
            return false;
        }
' '' \
  'N2 payload: a recusa das faces a MENOS desaparece — a metade MUDA'

muta "$COL/topo.rs" \
  '            if n != self.cantos_de(f) {
                out.clear();
                return false;
            }
' '' \
  'N3 payload: a recusa dos CANTOS desaparece — a face que o descreve nao ve'

muta "$COL/topo.rs" \
  '            if f >= self.faces() {
                out.clear();
                return false;' \
  '            if f >= self.faces() {
                return false;' \
  'N4 payload: a recusa deixa o registo MEIO ESCRITO'

# ── A LEI: as duas contagens ─────────────────────────────────────────────
muta "$COL/topo.rs" \
  '        self.verts == verts && self.faces() == faces' \
  '        self.verts == verts' \
  'N5 descreve: a metade das FACES desaparece'

muta "$COL/topo.rs" \
  '        self.verts == verts && self.faces() == faces' \
  '        self.faces() == faces' \
  'N6 descreve: a metade dos VERTICES desaparece'

# ── O DEVICE: desarmar em vez de estourar ────────────────────────────────
muta "$REN/tinta_gpu.rs" \
  '        if !t
            .topologia()
            .descreve(mesh.vert_count(), mesh.faces().len())
            || !t.topologia().payload(faces(), &mut pay)
        {
            if slot.gpu.tinta.armado {
                queue.write_buffer(&slot.gpu.tinta.cfg, 0, bytemuck::cast_slice(&cfg_de(None)));
                self.slots[index].gpu.tinta.armado = false;
            }
            return;
        }' \
  '        let _ = t.topologia().payload(faces(), &mut pay);' \
  'N7 device: a porta ignora o veredito e sobe um registo de outra malha'

# ⭐⭐ **A metade dos VERTICES tem mutacao PROPRIA, e CORPUS proprio.**
# ⛔ Na 1.a corrida ela SOBREVIVEU e ficou NOMEADA: o payload sozinho ja'
# recusava toda a fixtura do gate, logo apagar a chamada a' lei nao mudava um
# bit. O que faltava era uma malha com as MESMAS faces e outra contagem de
# vertices — um vertice ORFAO —, que e' a unica forma de pôr as duas reguas a
# discordar. *Uma linha que a mutacao nao consegue matar e' comentario com
# sintaxe de codigo; construir o corpus vale mais do que nomear a ausencia.*
muta "$REN/tinta_gpu.rs" \
  '        if !t
            .topologia()
            .descreve(mesh.vert_count(), mesh.faces().len())
            || !t.topologia().payload(faces(), &mut pay)' \
  '        if !t.topologia().payload(faces(), &mut pay)' \
  'N9 device: apenas o payload decide -- a metade dos VERTICES desaparece'

# ── O CONTROLO ───────────────────────────────────────────────────────────
# ⚠️ Uma mutacao INERTE nao pode sangrar. Sem ela um arnes partido — um filtro
# que casa zero testes, uma arvore ja' vermelha — devolve um placar PERFEITO.
muta "$COL/topo.rs" \
  'pub const PAYLOAD_STRIDE: usize = 19;' \
  'pub const PAYLOAD_STRIDE: usize = 19;
' \
  'N8 CONTROLO: uma mutacao INERTE (uma linha em branco) nao pode sangrar'

echo
if [ -n "$SO_ANCORAS" ]; then
  # ⚠️ **O sumario tem de dizer o que ele MEDIU.** Aqui nenhum teste correu:
  # dizer «sangram» sobre uma corrida de ancoras seria um instrumento a
  # descrever-se mal, que e' o defeito que este arnes inteiro existe para nao ter.
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ] || exit 1
  exit 0
fi
if [ -n "$FILTRO" ]; then
  echo "PLACAR PARCIAL (filtro MUTA_FILTRO='$FILTRO'): $sangram de $total sangram"
else
  echo "PLACAR: $sangram de $total sangram (o N8 e' o CONTROLO e nao pode)"
fi
