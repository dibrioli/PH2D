#!/usr/bin/env bash
# ⭐⭐⭐ PROVA DE MUTAÇÃO — a lei da RAZÃO e o instrumento da recusa (2026-09-19).
#
# ⚠️ Os casos (1)–(5) e (9) mutam a MARCHA, que NÃO shipa — ela é o instrumento
# que mede a alternativa recusada, e uma recusa sem instrumento que a reproduza
# é uma opinião. Os casos (6)–(8) mutam a lei que o pincel de facto corre.
#
# Cada caso apaga UMA lei e exige que um gate nomeado fique VERMELHO. Um caso que
# fica verde é uma lei sem régua.
#
# ⚠️⚠️ As quatro armadilhas de arnês que este repo já pagou estão fechadas aqui:
#   * a agulha é contada em PYTHON (`grep -cF` conta LINHAS, não ocorrências) e
#     tem de casar EXACTAMENTE uma vez — se casar 0 ou 2, o caso ABORTA em vez de
#     reportar;
#   * a corrida é lida por `running N tests` e `N = 0` é DEFEITO DO ARNÊS, nunca
#     «sobreviveu»;
#   * «não compilou» é separado de «passou» pelo código de saída do cargo;
#   * o restauro faz `touch`, senão o cargo guarda o build DA MUTAÇÃO.
#
#   bash docs/3D/geodesica/mutacao_2026-09-19.sh
set -uo pipefail
cd "$(dirname "$0")/../../.."

vermelhos=0
verdes=0

muta() {
  local nome="$1" ficheiro="$2" de="$3" para="$4" filtro="$5" pacote="$6"
  python3 -P - "$ficheiro" "$de" "$para" <<'PY' || { echo "  ⛔ ARNÊS: a agulha não casou exactamente 1× em $nome"; return; }
import sys, pathlib
f, de, para = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
s = f.read_text()
n = s.count(de)
assert n == 1, f"a agulha casou {n} vezes"
f.with_suffix(f.suffix + ".bak").write_text(s)
f.write_text(s.replace(de, para, 1))
PY
  local saida rc
  saida="$(bash scripts/ph2d-run.sh cargo test -p "$pacote" "$filtro" 2>&1)"
  rc=$?
  mv "$ficheiro.bak" "$ficheiro"
  touch "$ficheiro"

  if echo "$saida" | grep -q 'error\[E\|could not compile'; then
    echo "  ⛔ ARNÊS: a mutação não COMPILA em $nome (lê-se como sangrar e não é)"
    return
  fi
  # ⚠️ **A soma é em `awk` e não em `bc`** — esta máquina não tem `bc`, e um
  # `|| echo 0` a engolir isso fazia o arnês reportar «zero testes» em TODOS os
  # casos. *Um guarda que falha para o lado seguro ainda esconde o resultado.*
  local correram
  correram="$(echo "$saida" | grep -oE 'running [0-9]+ tests?' \
    | grep -oE '[0-9]+' | awk '{s += $1} END {print s + 0}')"
  if [ "${correram:-0}" -eq 0 ]; then
    echo "  ⛔ ARNÊS: o filtro '$filtro' casou ZERO testes em $nome"
    return
  fi
  if [ $rc -ne 0 ]; then
    echo "  ✓ SANGRA  $nome  ($correram testes corridos)"
    vermelhos=$((vermelhos + 1))
  else
    echo "  ✗ SOBREVIVE  $nome  ($correram testes corridos)"
    verdes=$((verdes + 1))
  fi
}

echo "== A GEODÉSICA (ph2d-mesh) =="

# (1) Os cantos vêm da FACE e não da adjacência — a armadilha do quad.
muta "os cantos da face" \
  crates/ph2d-mesh/src/geodesica.rs \
  'for &fi in adj.vert_faces.neighbours(ui) {' \
  'for &fi in adj.vert_faces.neighbours(ui).iter().take(0) {' \
  geodesica ph2d-mesh

# (2) A aresta é o TECTO — tirar a travessia deixa Dijkstra.
muta "a travessia do triângulo" \
  crates/ph2d-mesh/src/geodesica.rs \
  '                        melhor = melhor.min(t);' \
  '                        melhor = melhor.max(t);' \
  geodesica ph2d-mesh

# (3) O tecto corta na ENTRADA — o defeito real que os oráculos apanharam.
muta "o tecto na entrada" \
  crates/ph2d-mesh/src/geodesica.rs \
  '                if melhor > tecto {
                    continue;
                }' \
  '                if melhor > tecto * 1e9 {
                    continue;
                }' \
  geodesica ph2d-mesh

# (4) O leque da semente entra pela corda.
muta "o leque da semente" \
  crates/ph2d-mesh/src/geodesica.rs \
  'if corda <= tecto && (self.visto[ui] != epoca || corda < self.d[ui]) {' \
  'if false && corda <= tecto && (self.visto[ui] != epoca || corda < self.d[ui]) {' \
  geodesica ph2d-mesh

# (5) A cerca da época — a volta a cada 4 bilhões.
muta "a volta da época" \
  crates/ph2d-mesh/src/geodesica.rs \
  '        if self.epoca == 0 {
            self.epoca = 1;
            self.visto.fill(0);
            self.fixo.fill(0);
        }' \
  '        if self.epoca == u32::MAX {
            self.epoca = 1;
            self.visto.fill(0);
            self.fixo.fill(0);
        }' \
  geodesica ph2d-mesh

echo
echo "== A MÁSCARA DE ALCANCE (ph2d-sculpt3d) =="

# (6) A RAZÃO — a lei desta wave.
muta "a razão máxima" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  'pub const RAZAO_MAXIMA: f32 = 3.5;' \
  'pub const RAZAO_MAXIMA: f32 = 1000.0;' \
  sonda_da_parede_fina ph2d-sculpt3d

# (7) O corte de quem a superfície não alcança de todo (a classe INALCANÇÁVEL).
muta "o inalcançável" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  '            if marca[v as usize] != epoca {
                return false;
            }' \
  '            if marca[v as usize] != epoca {
                return true;
            }' \
  dab_alcance ph2d-sculpt3d

# (7-bis) A cerca da época na máscara — a volta a cada 4 bilhões de dabs.
muta "a volta da época na máscara" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  '        if self.epoca == 0 {
            self.epoca = 1;
            self.marca.fill(0);
        }' \
  '        if self.epoca == u32::MAX {
            self.epoca = 1;
            self.marca.fill(0);
        }' \
  dab_alcance ph2d-sculpt3d

# (7-ter) O tecto absoluto continua a cortar — ele não saiu, ganhou companhia.
muta "o tecto absoluto" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  '                if nd > tecto {
                    continue;
                }' \
  '                if nd > tecto * 1e9 {
                    continue;
                }' \
  dab_alcance ph2d-sculpt3d

# (8) E a razão não pode comer a própria cratera — o gate da refutação.
muta "a razão apertada (o tecto absoluto de volta)" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  'pub const RAZAO_MAXIMA: f32 = 3.5;' \
  'pub const RAZAO_MAXIMA: f32 = 1.05;' \
  sonda_da_parede_fina::a_lei_da_razao ph2d-sculpt3d

echo
echo "== A CONCORDÂNCIA DAS DUAS MARCHAS =="

# (9) Mexer numa das duas cópias tem de acordar o gate.
muta "uma das duas atravessa" \
  crates/ph2d-mesh/src/geodesica.rs \
  '    let lin = 2.0 * lb * u * (la * cos_t - lb);' \
  '    let lin = 2.0 * lb * u * (la * cos_t - lb * 1.000001);' \
  as_duas_marchas ph2d-sculpt3d

echo
echo "-----------------------------------------"
echo "  sangram: $vermelhos   ·   sobrevivem: $verdes"
[ "$verdes" -eq 0 ]
