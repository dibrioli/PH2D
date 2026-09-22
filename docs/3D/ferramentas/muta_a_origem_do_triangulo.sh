#!/usr/bin/env bash
# Prova de mutação da ph2d-mesh.
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO em três pontos, porque cada um deles já
# mentiu nesta casa:
#   (a) a âncora tem de casar EXACTAMENTE UMA vez  — casar zero lê-se como
#       «sobreviveu» e casar duas aplica a mutação no sítio errado;
#   (b) a mutação tem de COMPILAR — um erro de compilação lê-se como sangrar;
#   (c) a corrida tem de correr N > 0 testes, contados de `test result:`
#       (o `running N tests` CONTA os `#[ignore]`).
set -u
# ⭐ O PRE-VOO (`MUTA_SO_ANCORAS=1`) — acrescentado em 22/09, e este arnes
#   e' a razao de ele existir: as TRES ancoras dele casavam ZERO vezes
#   porque um CORTE de tecto de LOC mudou a lei de `mesh.rs` para
#   `mesh_indices.rs`, e sem pre-voo isso le-se no log como `0 de 3`.
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
CRATE=crates/ph2d-mesh
SRC=$CRATE/src
BK=$(mktemp -d)
cp -r "$SRC" "$BK/src"
restore() { rm -rf "$SRC"; cp -r "$BK/src" "$SRC"; find "$SRC" -name '*.rs' -exec touch {} +; }
trap restore EXIT

if [ -z "$SO_ANCORAS" ]; then
  verde=$(cargo test -p ph2d-mesh 2>&1 | grep -oP 'test result: ok\. \K[0-9]+' | paste -sd+ - | tr '+' ' ' | awk '{s=0;for(i=1;i<=NF;i++)s+=$i;print s}')
  echo "VERDE antes: $verde testes correram"
  [ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa não correu teste nenhum"; exit 2; }
fi

sangram=0; total=0
muta() { # ficheiro  agulha  substituto  nome
  local f="$SRC/$1" agulha="$2" subst="$3" nome="$4"
  total=$((total+1))
  # ⚠️ A contagem é em PYTHON e não `grep -cF`: o grep conta LINHAS, logo uma
  #    âncora de duas linhas casa "duas vezes" numa ocorrência só. Já mordeu.
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ -n "$SO_ANCORAS" ]; then
    if [ "$n" -ne 1 ]; then echo "  ✗ ANCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a âncora casou $n vezes (esperado 1)"; return; fi
  python3 - "$f" "$agulha" "$subst" <<'PY'
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,'w').write(s.replace(a, b, 1))
PY
  touch "$f"
  local out rc
  out=$(cargo test -p ph2d-mesh 2>&1); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutação não compila"
  else
    local corridos
    # ⚠️⚠️ A populacao e' `passed + failed`, nunca so' `passed`. Com UM teste
    #    no filtro, uma mutacao que SANGRA deixa `0 passed; 1 failed` — e um
    #    contador que le so' o `passed` le' isso como *«zero testes correram»* e
    #    ABORTA a mutacao que estava a funcionar. Medido em 2026-09-20, sobre o
    #    gate de paridade da placa: `0 de 5` onde a verdade era `4 de 5`.
    corridos=$(echo "$out" | grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}')
    if [ "$corridos" -eq 0 ]; then echo "  ABORTO [$nome]: zero testes correram"
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

muta mesh_indices.rs \
  'o.push(((fi as u32) << 1) | sub as u32);' \
  'o.push((fi as u32) << 1);' \
  'A1 a origem esquece qual metade do quad'

muta mesh_indices.rs \
  'o.push(((fi as u32) << 1) | sub as u32);' \
  'o.push((fi as u32) | sub as u32);' \
  'A2 a origem nao desloca a face'

muta mesh_indices.rs \
  '        self.triangle_indices_com_origem(out, None);' \
  '        self.triangle_indices_com_origem(out, None);
        out.truncate(out.len().saturating_sub(1));' \
  'A3 a porta sem origem deixa de entregar a mesma lista'

echo
if [ -n "$SO_ANCORAS" ]; then
  # ⚠️ O sumario DIZ o que mediu: aqui nenhum teste correu.
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
  exit $?
fi
echo "MUTACAO: $sangram de $total sangram"
[ "$sangram" -eq "$total" ]
