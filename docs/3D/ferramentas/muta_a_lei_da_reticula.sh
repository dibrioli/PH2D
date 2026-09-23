#!/usr/bin/env bash
# Prova de mutação da ph2d-mesh-colors.
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO em três pontos, porque cada um deles já
# mentiu nesta casa:
#   (a) a âncora tem de casar EXACTAMENTE UMA vez  — casar zero lê-se como
#       «sobreviveu» e casar duas aplica a mutação no sítio errado;
#   (b) a mutação tem de COMPILAR — um erro de compilação lê-se como sangrar;
#   (c) a corrida tem de correr N > 0 testes, contados de `test result:`
#       (o `running N tests` CONTA os `#[ignore]`).
set -u
# ⭐ O PRE-VOO (`MUTA_SO_ANCORAS=1`) — acrescentado em 22/09. Uma ancora
#   que casa ZERO le-se num placar exactamente como uma mutacao que
#   SOBREVIVEU, e o `cargo fmt` (ou um corte de tecto de LOC) reescreve-a
#   sem avisar ninguem.
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
CRATE=crates/ph2d-mesh-colors
SRC=$CRATE/src
BK=$(mktemp -d)
cp -r "$SRC" "$BK/src"
restore() { rm -rf "$SRC"; cp -r "$BK/src" "$SRC"; find "$SRC" -name '*.rs' -exec touch {} +; }
trap restore EXIT

if [ -z "$SO_ANCORAS" ]; then
  verde=$(cargo test -p ph2d-mesh-colors 2>&1 | grep -oP 'test result: ok\. \K[0-9]+' | paste -sd+ - | tr '+' ' ' | awk '{s=0;for(i=1;i<=NF;i++)s+=$i;print s}')
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
  out=$(cargo test -p ph2d-mesh-colors 2>&1); rc=$?
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

# ── M1: o discriminante tri/quad ──────────────────────────────────────────
muta topo.rs \
  '4 if face[3] == TRI => 3,' \
  '4 if face[3] == TRI => 4,' \
  'M1 cantos: um triangulo marcado le-se como quad'

# ── M2: a contagem de interior de um TRIANGULO ────────────────────────────
muta topo.rs \
  'l.saturating_mul(l.saturating_sub(1)) / 2' \
  'l.saturating_mul(l.saturating_sub(1))' \
  'M2 interior_por_face: o triangulo conta o dobro'

# ── M3: a VIRADA de uma aresta partilhada ─────────────────────────────────
muta enderecos.rs \
  'let t = if virada { le - t } else { t };' \
  'let t = t;' \
  'M3 indice: a aresta percorrida ao contrario nao vira o t'

# ── M4: o canto NAO e o indice do vertice ─────────────────────────────────
muta enderecos.rs \
  'Sitio::Canto(c) => cantos[c],' \
  'Sitio::Canto(c) => c as u32,' \
  'M4 indice: o canto deixa de ser a numeracao da malha'

# ── M5: o bloco das arestas tem a largura errada ──────────────────────────
muta enderecos.rs \
  'topo.verts + topo.arestas_amostras() as usize' \
  'topo.verts + topo.arestas_amostras() as usize + 1' \
  'M5 total: o bloco das arestas conta uma amostra a mais'

# ── M6: o lado d->a do QUAD anda para a frente ────────────────────────────
muta enderecos.rs \
  'lado_da_face: 3,
            t: lado - j,' \
  'lado_da_face: 3,
            t: j,' \
  'M6 sitio_quad: o lado d->a deixa de andar para tras'

# ── M7: a semeadura do TRIANGULO troca dois eixos ─────────────────────────
muta lib.rs \
  'let idx = indice(&t.topo, fi, sitio_tri(l, i, j, k), &f[..n]) as usize;' \
  'let idx = indice(&t.topo, fi, sitio_tri(l, j, i, k), &f[..n]) as usize;' \
  'M7 semeada(tri): i e j trocados'

# ── M8: a semeadura do QUAD nao mistura ───────────────────────────────────
#    ⚠️ Esta SOBREVIVEU a primeira corrida: a fixtura irma e um TETRAEDRO,
#    so de triangulos, e o gate da bijeccao conta indices sem olhar valores.
muta lib.rs \
  'let idx = indice(&t.topo, fi, sitio_quad(l, i, j), &f[..n]) as usize;
                        t.amostras[idx] = mistura(&c, &w);' \
  'let idx = indice(&t.topo, fi, sitio_quad(l, i, j), &f[..n]) as usize;
                        t.amostras[idx] = c[0];' \
  'M8 semeada(quad): a bilinear vira o primeiro canto'

# ── M9: a mistura ignora os pesos ─────────────────────────────────────────
muta lib.rs \
  'o[e] += q[e] * k;' \
  'o[e] += q[e];' \
  'M9 mistura: os pesos sao ignorados'

# ── M10: o GEMEO — o lado c->d do quad anda para a frente ─────────────────
#    ⚠️ Ele so' e' observavel numa grelha 2x2: numa fita de dois quads o lado
#    `2` nunca e' partilhado, e inverter o `t` de uma aresta de BORDO e' so'
#    uma permutacao do proprio bloco dela.
muta enderecos.rs \
  'lado_da_face: 2,
            t: lado - i,' \
  'lado_da_face: 2,
            t: i,' \
  'M10 sitio_quad: o lado c->d deixa de andar para tras'

# ── M11: a bilinear perde o termo cruzado ─────────────────────────────────
muta amostragem.rs \
  '((i + 1, j + 1), fu * fv),' \
  '((i + 1, j + 1), fv),' \
  'M11 leitura_quad: o canto oposto perde o termo cruzado'

# ── M12: o piso deixa de ser cortado em L-1 ───────────────────────────────
#    ⚠️ Em `u = 1` exacto o `floor` da' `L` e a celula vira `[L, L+1]` — um
#    endereco FORA da face.
muta amostragem.rs \
  'let i = (su.floor() as u32).min(lado - 1);' \
  'let i = su.floor() as u32;' \
  'M12 leitura_quad: o piso de u nao e cortado na ultima celula'

# ── M13: o ponto nao entra no quadrado ────────────────────────────────────
muta amostragem.rs \
  'let v = uv[1].clamp(0.0, 1.0);' \
  'let v = uv[1];' \
  'M13 leitura_quad: um v fora nao e cortado'

# ── M14: o payload perde o bit da VIRADA ──────────────────────────────────
muta topo.rs \
  '                    self.lado_da_face[4 * f + s]' \
  '                    self.lado_da_face[4 * f + s] & !1' \
  'M14 payload: a virada da aresta nao viaja'

# ── M15: todas as faces partilham o inicio do interior ────────────────────
muta topo.rs \
  'out.push(self.off_interior[f]);' \
  'out.push(self.off_interior[0]);' \
  'M15 payload: o bloco de interior e o mesmo para todas as faces'

# ── M16: os cantos viajam ao contrario ────────────────────────────────────
# ⚠️⚠️ **Esta ancora MORREU e ninguem viu** (medido 22/09): a forma antiga
#   (`out.push(if s < n { cantos[s] } else { TRI });`) saiu do `topo.rs` no
#   `7d446fde6` — o gemeo em WGSL —, quando o laco foi reescrito para o idioma
#   que o clippy aceita. A partir dai ela casava ZERO vezes, e o arnes lia
#   `15 de 16` num log que ninguem re-le. *Foi o PRE-VOO que a achou, e e' por
#   isso que ele foi acrescentado aqui no mesmo dia.*
muta topo.rs \
  '            for c in cantos.iter().take(n) {' \
  '            for c in cantos.iter().take(n).rev() {' \
  'M16 payload: os cantos viajam invertidos'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
  exit $?
fi
echo "MUTACAO: $sangram de $total sangram"
[ "$sangram" -eq "$total" ]
