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
# ⛔⛔⛔ **A RAZÃO PERDEU O TRABALHO PARA A LEI DA NORMAL, e a mutação disse-o:**
# com a `RAZAO_MAXIMA` inerte (`1e9`) das `697` corridas das duas crates caía UMA,
# e era um gate que media a própria razão. ⇒ ela ficou com UM sítio onde ainda é
# a única coisa que separa as folhas — os verbos que RELAXAM, que não lêem o olho
# —, e é esse o gate que a mata agora.
echo "  ⚠️ NOMEADA  a razão máxima — subsumida pela lei da NORMAL na mesma tarde."
echo "     Medido: inerte (1e9), das 697 corridas das duas crates cai UMA (um gate"
echo "     que media a propria razao); e no regime construido para ela decidir"
echo "     (verbo que RELAXA, chapa de duas folhas, R=0,20, d=0,12, dentro da banda"
echo "     3,5t < 2R) ela le 7 de 64 a escapar COM ela e SEM ela — quem corta e' o"
echo "     ALCANCE_TECTO, e os 7 sao os LATERAIS, que ela nunca apanhou."
echo "     ⇒ nenhuma fixtura deste repo a distingue. A remocao e' wave propria."


# (7) O corte de quem a superfície não alcança de todo (a classe INALCANÇÁVEL).
muta "o inalcançável" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  '            if marca[vi] != epoca {
                return false;
            }' \
  '            if marca[vi] != epoca {
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
echo "== A CENA \`=50\` (ph2d-app-sculpt3d) =="

# ⚠️ Os três casos seguintes NÃO mutam a lei: mutam a CENA e a FIAÇÃO dela.
# *Uma cena que ensina o contrário do que acontece é pior que uma cena ausente*
# (CLAUDE.md §5.0), e quem a defende tem de sangrar quando ela deixa de conter o
# fenómeno — ou quando o app deixa de a usar.

# (10) A peça deixa de ser FINA ⇒ o fenómeno não existe e a cena não ensina nada.
muta "a barbatana engorda" \
  crates/ph2d-app-sculpt3d/src/scenes_parede_fina.rs \
  'const ESPESSURA: f32 = 0.06;' \
  'const ESPESSURA: f32 = 0.60;' \
  parede_fina ph2d-app-sculpt3d

# (11) O selector deixa de escolher a barbatana ⇒ a cena abre na peça de fábrica,
#      que é GROSSA. ⛔ O gate que chama `barbatana()` directamente fica VERDE
#      sobre isto — é por isso que existe a metade que lê o DESPACHO.
#      ⚠️⚠️ **E a 1.ª redacção dessa metade procurava os dois NOMES soltos, logo
#      esta mutação SOBREVIVEU:** prefixar `false &&` deixa os dois presentes e o
#      despacho morto. A agulha passou a ser o BRAÇO inteiro.
muta "o selector de malha" \
  crates/ph2d-app-sculpt3d/src/scenes_mesh.rs \
  '    if parede_fina::parede_fina_scene() {
        return parede_fina::barbatana();
    }' \
  '    if false && parede_fina::parede_fina_scene() {
        return parede_fina::barbatana();
    }' \
  parede_fina ph2d-app-sculpt3d

# (12) O roteiro existe e ninguém o imprime — o defeito que um `warning: never
#      used` apanhou uma vez e que deixa de ser visível assim que a função é
#      `pub`. *Um aviso do compilador mede VISIBILIDADE, nunca a lei.*
muta "o roteiro mudo" \
  crates/ph2d-app-sculpt3d/src/scripts.rs \
  '    crate::scenes::parede_fina::announce();' \
  '' \
  parede_fina ph2d-app-sculpt3d

# (13) E a LEI outra vez, agora medida PELA CENA: sem a razão as costas da
#      barbatana movem-se, e o passo (4) do roteiro passa a mentir.
#      ⛔⛔⛔ **Ela SOBREVIVEU à 1.ª redacção, e o defeito era a FIXTURA:** o gate
#      carimbava a meio caminho da beira (`d = 0,50`), onde a superfície mede
#      `1,06` contra o tecto ABSOLUTO de `0,80` ⇒ *a lei antiga já cortava ali*,
#      e o gate media uma cura que existia antes desta wave. A sonda
#      `diag_onde_a_razao_e_a_unica_que_cura` deu a banda (`0,10`–`0,35`), o
#      carimbo mudou-se para `0,20`, e o ROTEIRO mudou com ele — ele mandava o
#      dono carimbar exactamente no sítio onde não havia nada de novo para ver.
muta "a razão, medida na cena do dono" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  'pub const RAZAO_MAXIMA: f32 = 3.5;' \
  'pub const RAZAO_MAXIMA: f32 = 1000.0;' \
  a_cena_contem_o_defeito_e_a_cura ph2d-app-sculpt3d

echo
echo "== A FOLHA QUE O OLHO ESCOLHE (report de 19/09) =="

# (14) A lei nova. Inerte (`dot <= 1` sempre), as costas voltam a mover-se.
muta "a folha do olho" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  'pub const NORMAL_LIMIAR: f32 = 0.30;' \
  'pub const NORMAL_LIMIAR: f32 = 9.0;' \
  as_costas_ficam_quietas ph2d-app-sculpt3d

# (15) A barra apertada: a `0,0` ela come o labio de uma ruga funda.
muta "a folga da barra" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  'pub const NORMAL_LIMIAR: f32 = 0.30;' \
  'pub const NORMAL_LIMIAR: f32 = -0.90;' \
  a_folha_do_olho_nao_corta ph2d-sculpt3d

# (16) A cerca da pegada vazia — sem ela um pincel fica INERTE e mudo.
muta "a cerca da pegada vazia" \
  crates/ph2d-sculpt3d/src/dab_alcance.rs \
  '        let corta_normal = olho_bom
            && tem_normais
            && pegada.iter().any(|&v| {' \
  '        let corta_normal = olho_bom
            && tem_normais
            && !pegada.iter().any(|&v| {' \
  uma_pegada_toda_virada ph2d-sculpt3d

# (17) A cerca por VERBO — quem relaxa nao le o olho.
muta "a cerca por verbo" \
  crates/ph2d-sculpt3d/src/brush_verb_predicados.rs \
  '        !matches!(
            self,
            Self::Smooth | Self::SurfaceSmooth | Self::SlideRelax | Self::Sharpen
        )' \
  '        true' \
  quem_relaxa_nao_le_o_olho ph2d-sculpt3d

# (18) E o OLHO tem de chegar ao produto: o chamador que o zera para todos.
muta "o olho no chamador" \
  crates/ph2d-sculpt3d/src/stroke_dab_core.rs \
  '                if brush.verb.a_folha_do_olho_decide() {
                    dab.eye
                } else {
                    [0.0, 0.0, 0.0]
                },' \
  '                [0.0, 0.0, 0.0],' \
  as_costas_ficam_quietas ph2d-app-sculpt3d

echo
echo "-----------------------------------------"
echo "  sangram: $vermelhos   ·   sobrevivem: $verdes"
[ "$verdes" -eq 0 ]
