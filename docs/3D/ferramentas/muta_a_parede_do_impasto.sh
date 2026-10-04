#!/usr/bin/env bash
# Prova de mutação da PAREDE DO IMPASTO NA PEÇA contra o 2D (docs/3D/30 §19):
# o corpo pesa a LUZ e não a inclinação, a cor pintada entra descodificada, e o
# ambiente lê a peça sem o relevo.
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO nos QUATRO pontos dos irmãos (âncora única ·
# a mutação compila · N > 0 testes correram · a corrida limpa VERDE), e tem o
# PRÉ-VOO (`MUTA_SO_ANCORAS=1`).
#
# ⚠️ A POPULAÇÃO: os gates do shader sem placa (`ph2d-mesh-render`) e o gate da
# peça contra o 2D com placa (`ph2d-app-sculpt3d`, perfil `smoke`).
#
# ⛔ Chame-o sempre pela porta de recursos, com a placa:
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_a_parede_do_impasto.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
REN=crates/ph2d-mesh-render/src
BK=$(mktemp -d)
cp -r "$REN" "$BK/ren"
restore() {
  rm -rf "$REN"
  cp -r "$BK/ren" "$REN"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTAÇÃO.
  find "$REN" -type f -exec touch {} +
}
trap restore EXIT

corrida() {
  cargo test -p ph2d-mesh-render --lib -- wgsl_gate 2>&1
  cargo test -p ph2d-app-sculpt3d --profile smoke --lib -- --ignored --test-threads=1 \
    a_parede_do_impasto_na_peca_le_como_no_2d 2>&1
}
populacao() { grep -oP '\K[0-9]+(?= passed)' | awk '{s+=$1}END{print s+0}'; }
falhou() { grep -q 'test result: FAILED'; }

if [ -z "$SO_ANCORAS" ]; then
  limpa=$(corrida)
  verde=$(printf '%s' "$limpa" | populacao)
  echo "VERDE antes: $verde testes passaram"
  [ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa não correu teste nenhum"; exit 2; }
  if printf '%s' "$limpa" | falhou; then
    echo "ABORTO: a corrida limpa está VERMELHA -- um placar tirado daqui é fabricado."
    exit 2
  fi
fi

sangram=0; total=0
muta() { # ficheiro  âncora  substituto  nome
  local f="$1" agulha="$2" subst="$3" nome="$4"
  if [ -n "$FILTRO" ] && ! printf '%s' "$nome" | grep -Eq "$FILTRO"; then return; fi
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ -n "$SO_ANCORAS" ]; then
    if [ "$n" -ne 1 ]; then echo "  ✗ ÂNCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a âncora casou $n vezes (esperado 1)"; return; fi
  python3 -c '
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,"w").write(s.replace(a, b, 1))
' "$f" "$agulha" "$subst"
  touch "$f"
  local out corridos
  out=$(corrida)
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutação não compila"
  else
    corridos=$(printf '%s' "$out" | grep -oP '\K[0-9]+(?= (passed|failed))' | awk '{s+=$1}END{print s+0}')
    if [ "$corridos" -eq 0 ]; then
      echo "  ABORTO [$nome]: zero testes correram — as últimas linhas foram:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif printf '%s' "$out" | falhou; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── O CORPO PESA A LUZ ───────────────────────────────────────────────────
muta "$REN/fonte.rs" \
  '    return mix(liso, tinta, corpo);' \
  '    return tinta;' \
  'W1 o corpo deixa de pesar a luz: a parede com pouca tinta fica de lado (a orla)'

muta "$REN/fonte.rs" \
  '    let tinta = fs_core_n(in, t.c.xyz, tinta_relevo_n(in, t.g));
    return mix(liso, tinta, corpo);' \
  '    return fs_core_n(in, t.c.xyz, tinta_relevo_n(in, t.g * corpo));' \
  'W2 o corpo volta a pesar a INCLINAÇÃO (a lei velha exacta, a orla escura)'

# ── A COR ENTRA DESCODIFICADA ────────────────────────────────────────────
muta "$REN/shaders/mesh.wgsl" \
  '    let vcolor = cor_em_luz(codigo);' \
  '    let vcolor = codigo;' \
  'W3 o código sRGB multiplica a luz (a tinta cinzenta)'

muta "$REN/shaders/mesh.wgsl" \
  '    let alto = pow((max(c, vec3<f32>(0.04045)) + 0.055) / 1.055, vec3<f32>(2.4));' \
  '    let alto = pow((max(c, vec3<f32>(0.04045)) + 0.055) / 1.055, vec3<f32>(2.2));' \
  'W4 a curva de descodificação não é a do sRGB'

# ── O AMBIENTE LÊ A PEÇA SEM O RELEVO ────────────────────────────────────
muta "$REN/shaders/mesh.wgsl" \
  '        let cena = luz + mx_indirect(mt, na, PBR_VIEW) * cav_occ;' \
  '        let cena = luz + mx_indirect(mt, nc, PBR_VIEW) * cav_occ;' \
  'W5 o céu do PBR reflecte-se nas paredes da tinta (o véu)'

muta "$REN/shaders/mesh.wgsl" \
  '    let floor_e = ambient_floor(na);' \
  '    let floor_e = ambient_floor(nc);' \
  'W6 o piso ambiente com direcção lê o relevo'

# ── O CONTROLO ───────────────────────────────────────────────────────────
muta "$REN/fonte.rs" \
  '    // tinta com o relevo INTEIRO, misturados pela quantidade de tinta. Pesar o' \
  '    // tinta com o relevo INTEIRO; misturados pela quantidade de tinta. Pesar o' \
  'C1 CONTROLO: uma mutação INERTE (um comentário) não pode sangrar'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRÉ-VOO: $sangram de $total âncoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ] || exit 1
  exit 0
fi
if [ -n "$FILTRO" ]; then
  echo "PLACAR PARCIAL (filtro MUTA_FILTRO='$FILTRO'): $sangram de $total sangram"
else
  echo "PLACAR: $sangram de $total sangram (o C1 é o CONTROLO e não pode)"
fi
