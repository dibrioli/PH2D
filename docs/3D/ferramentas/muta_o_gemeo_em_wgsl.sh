#!/usr/bin/env bash
# Prova de mutação da ph2d-mesh-render.
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO em três pontos, porque cada um deles já
# mentiu nesta casa:
#   (a) a âncora tem de casar EXACTAMENTE UMA vez  — casar zero lê-se como
#       «sobreviveu» e casar duas aplica a mutação no sítio errado;
#   (b) a mutação tem de COMPILAR — um erro de compilação lê-se como sangrar;
#   (c) a corrida tem de correr N > 0 testes, contados de `test result:`
#       (o `running N tests` CONTA os `#[ignore]`).
set -u
CRATE=crates/ph2d-mesh-render
SRC=$CRATE/src
BK=$(mktemp -d)
cp -r "$SRC" "$BK/src"
restore() { rm -rf "$SRC"; cp -r "$BK/src" "$SRC"; find "$SRC" -name '*.rs' -exec touch {} +; }
trap restore EXIT

verde=$(env PH2D_GPU=1 cargo test -p ph2d-mesh-render --test it -- --ignored a_lei_da_reticula 2>&1 | grep -oP 'test result: ok\. \K[0-9]+' | paste -sd+ - | tr '+' ' ' | awk '{s=0;for(i=1;i<=NF;i++)s+=$i;print s}')
echo "VERDE antes: $verde testes correram"
[ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa não correu teste nenhum"; exit 2; }

sangram=0; total=0
muta() { # ficheiro  agulha  substituto  nome
  local f="$SRC/$1" agulha="$2" subst="$3" nome="$4"
  total=$((total+1))
  # ⚠️ A contagem é em PYTHON e não `grep -cF`: o grep conta LINHAS, logo uma
  #    âncora de duas linhas casa "duas vezes" numa ocorrência só. Já mordeu.
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
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
  out=$(env PH2D_GPU=1 cargo test -p ph2d-mesh-render --test it -- --ignored a_lei_da_reticula 2>&1); rc=$?
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

muta shaders/tinta.wgsl \
  'if ((w & 1u) == 1u) { tt = l - t; }' \
  'if (false) { tt = l - t; }' \
  'W1 a VIRADA da aresta nao viaja no shader'

muta shaders/tinta.wgsl \
  'if (sub == 1u) { uv = vec2<f32>(bar.y, bar.y + bar.z); }' \
  'if (false) { uv = vec2<f32>(bar.y, bar.y + bar.z); }' \
  'W2 as duas metades de um quad leem o mesmo (u,v)'

muta shaders/tinta.wgsl \
  '        ijk0 = vec3<u32>(i, j + 1u, k + 1u);' \
  '        ijk0 = vec3<u32>(i + 1u, j, k);' \
  'W3 o sub-triangulo INVERTIDO le o canto do direito'

muta shaders/tinta.wgsl \
  '        let wa = dot(cross(pc - pb, p - pb), n) / dd;' \
  '        let wa = dot(cross(pc - pb, p - pb), n) / dd * 0.5;' \
  'W4 a baricentrica sai errada'

# ⛔⛔ **W5 SOBREVIVE DE PROPOSITO, e a medicao esta' no shader.** Apagar o
#    corte do piso em `L−1` nao muda a COR: em `u = 1` exacto o `fu` sai
#    exactamente ZERO, logo as duas amostras de fora da face entram com peso
#    `0`. O corte guarda o ENDERECO, e um endereco fora do intervalo e' seguro
#    em WGSL e um PANICO do lado da `ph2d-mesh-colors` — onde a mesma mutacao
#    (`M12`) sangra. *Uma mutacao que nenhuma regua de saida pode matar nao e'
#    uma regua em falta.*
muta shaders/tinta.wgsl \
  '    let i = min(u32(floor(c.x)), l - 1u);' \
  '    let i = u32(floor(c.x));' \
  'W5 o piso do quad nao e cortado (SOBREVIVE de proposito — ver acima)'

echo
echo "MUTACAO: $sangram de $total sangram (a W5 sobrevive de proposito)"
# ⚠️ O tecto e' `total - 1`: a W5 esta' NOMEADA acima com a medicao.
[ "$sangram" -eq $((total - 1)) ]
