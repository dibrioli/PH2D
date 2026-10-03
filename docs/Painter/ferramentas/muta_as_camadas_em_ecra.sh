#!/usr/bin/env bash
# Prova de mutação das CAMADAS EM TONS DE ECRÃ — ADR-0177 / doc Painter 45: o decode e o encode do
# compositor de referência, o des-premultiplicar da aquarela, o gémeo na placa (a tabela do decode,
# o encode final, o grafo de vizinhança) e — P3 — a fronteira DE CADA AJUSTE (os de luz convertem
# dentro deles, os de ecrã não convertem), a porta dos blurs, a mistura de volta (a cor muda, a
# cobertura não) e o corte do Threshold, na CPU e na placa — e P4: os desfoques em tons de ecrã
# pré-multiplicados (a porta dos blurs e os pontos da placa), o Bloom na sua porta de LUZ, e o
# compositor do Flip em luz (a tabela de 512, a constante `LIGHT_SPACE`, a porta `compositor_do_flip`).
#
# Corre-se pela fatia da linha e com a placa (a corrida `placa` pede-a; ~40 min ⇒ prazo maior):
#   PH2D_PRAZO=5400 PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/muta_as_camadas_em_ecra.sh
# Pré-voo das âncoras (zero testes): MUTA_SO_ANCORAS=1 bash <este ficheiro>
# Só algumas (regex sobre o nome, ex. 'G16|E3 '): MUTA_FILTRO='…' (os controlos correm na mesma)
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO (o idioma dos irmãos): a âncora casa EXACTAMENTE uma vez, a
# mutação COMPILA, e a corrida corre N > 0 testes (`passed + failed`). Cada corrida tem o seu
# CONTROLO verde antes.
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
FILTRO="${MUTA_FILTRO:-}"
SRCS=(crates/ph2d-tool-painter/src crates/ph2d-render/src crates/ph2d-painter-effects/src crates/ph2d-flip-render/src)
BK=$(mktemp -d)
for s in "${SRCS[@]}"; do mkdir -p "$BK/$s"; cp -r "$s/." "$BK/$s/"; done
restore() {
  for s in "${SRCS[@]}"; do rm -rf "$s"; mkdir -p "$s"; cp -r "$BK/$s/." "$s/"; find "$s" -type f -exec touch {} +; done
}
# ⚠️ Também no INT/TERM: a fatia da linha mata por prazo, e um arnês morto a meio deixaria o src MUTADO.
trap restore EXIT
trap 'exit 130' INT TERM

corre() {
  case "$1" in
    cpu) cargo test -p ph2d-tool-painter --lib -- compositor:: o_traco_numa_camada_nova watercolor_ground_is_the_real_backdrop 2>&1 ;;
    efeitos) ( cargo test -p ph2d-painter-effects --lib 2>&1; a=$?
               cargo test -p ph2d-tool-painter --lib -- compositor:: 2>&1; b=$?
               exit $((a | b)) ) ;;
    tabela) cargo test -p ph2d-render --lib -- decode_lut_is_the_cpu_decode 2>&1 ;;
    placa) cargo test -p ph2d-render --test it -- --ignored --test-threads=1 layer_compositor \
             --skip perf --skip 4k --skip 50_layers --skip too_many 2>&1 ;;
    flip) cargo test -p ph2d-flip-render --test it -- --ignored --test-threads=1 composite_ 2>&1 ;;
  esac
}
contados() { grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  for c in cpu efeitos tabela placa flip; do
    out=$(corre "$c"); rc=$?
    n=$(echo "$out" | contados)
    echo "CONTROLO [$c]: rc=$rc, $n testes"
    [ "$rc" -eq 0 ] && [ "$n" -gt 0 ] || { echo "ABORTO: o controlo [$c] não está verde"; exit 2; }
  done
fi

sangram=0; total=0
muta() { # ficheiro  agulha  substituto  nome  corrida
  local f="$1" agulha="$2" subst="$3" nome="$4" c="$5"
  if [ -n "$FILTRO" ] && ! echo "$nome" | grep -qE "$FILTRO"; then return; fi
  total=$((total+1))
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
  local out rc corridos
  out=$(corre "$c"); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutação não compila"
  else
    corridos=$(echo "$out" | contados)
    if [ "$corridos" -eq 0 ]; then echo "  ABORTO [$nome]: zero testes correram"
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

P=crates/ph2d-tool-painter/src
R=crates/ph2d-render/src

# ── CPU, o compositor de referência ──
muta $P/compositor/mod.rs '    b as f32 / 255.0' '    ph2d_color::srgb::srgb_to_linear_byte(b)' \
  'C1 o decode volta à luz' cpu
muta $P/compositor/mod.rs '(v.clamp(0.0, 1.0) * 255.0).round() as u8' '(v.clamp(0.0, 1.0) * 255.0) as u8' \
  'C2 o encode trunca' cpu
muta $P/compositor/compose.rs 'let blended = apply_blend(adj_mode, opaco(base), opaco(adjusted[i]));' 'let blended = apply_blend(adj_mode, base, [adjusted[i][0], adjusted[i][1], adjusted[i][2], base[3]]);' \
  'C3 a mistura de volta do ajuste volta ao over com o alfa da base' cpu
muta $P/tool/paint/watercolor_render.rs 'let l = (app - ground_enc[c] * (1.0 - out_a)) * inv_a;' 'let l = app;' \
  'C5 a aquarela numa camada não se des-premultiplica' cpu
muta $P/tool/paint/watercolor_render.rs 'let ground_enc = ground_enc.map(|g| g / 255.0);' 'let ground_enc = ground_enc.map(|g| g / 255.0 * 0.0 + ph2d_color::srgb::srgb_to_linear_unit(g / 255.0));' \
  'C6 a aquarela des-premultiplica sobre o chão em luz' cpu

# ── a placa ──
muta $R/layer_compositor/mod.rs 'CompositeSpace::Light if i < 256 =>' '_ if i < 256 =>' \
  'G1 a tabela do decode volta à luz (também no Painter)' tabela
muta $R/layer_compositor/mod.rs 'CompositeSpace::Light if i < 256 =>' 'CompositeSpace::Light =>' \
  'G1b a tabela em luz descodifica também o alfa' tabela
muta $R/shaders/layer_composite.wgsl 'decode_lut[256u + u32(raw.a * 255.0 + 0.5)],' 'decode_lut[u32(raw.a * 255.0 + 0.5)],' \
  'G1c o alfa lê a metade da cor (no Flip, a cobertura sai descodificada)' flip
muta $R/shaders/layer_composite.wgsl '    if LIGHT_SPACE {
        c = vec4<f32>(' '    if false {
        c = vec4<f32>(' \
  'G1d o encode final do Flip esquece a luz' flip
muta $R/layer_compositor/compositor/api.rs 'if self.space == CompositeSpace::Light && ops.iter().any(adjusts)' 'if false && ops.iter().any(adjusts)' \
  'G1e a porta em luz aceita um ajuste em silêncio' flip
muta crates/ph2d-flip-render/src/composite.rs 'LayerCompositor::with_space(gpu, ph2d_render::CompositeSpace::Light)' 'LayerCompositor::with_space(gpu, ph2d_render::CompositeSpace::DisplayTones)' \
  'F1 o Flip junta as camadas em tons de ecrã (a P1 em silêncio)' flip
muta $R/shaders/layer_composite.wgsl 'vec4<f32>(1.0)) * 255.0 + 0.5) / 255.0;' 'vec4<f32>(1.0)) * 255.0) / 255.0;' \
  'G2 o encode final trunca' placa
muta $R/shaders/layer_composite.wgsl '        return vec4<f32>(t.rgb * t.a, t.a);' '        let l = em_luz(t);
        return vec4<f32>(l.rgb * l.a, l.a);' \
  'G5 o 1.º passe do blur volta à luz (P4)' placa
muta $R/shaders/layer_composite.wgsl '        result = em_tons_de_ecra(unpremultiply(bloomed_pm));' '        result = unpremultiply(bloomed_pm);' \
  'G6 o combine do bloom devolve luz' placa
muta $R/shaders/layer_composite.wgsl '        let acc = em_luz(acc_enc); // Bloom is optical: its space is light' '        let acc = acc_enc;' \
  'G7 o bloom soma o brilho ao codificado' placa
muta $R/shaders/layer_composite.wgsl '                acc_enc + comb_g.amount * (acc_enc - blurred),' '                em_luz(acc_enc) + comb_g.amount * (em_luz(acc_enc) - blurred),' \
  'G7b o sharpen da placa parte da base em luz' placa
muta $R/shaders/layer_composite.wgsl '    let tr = textureLoad(chroma_src, sr, 0); // display tones, like the blurs (P4)' '    let tr = em_luz(textureLoad(chroma_src, sr, 0));' \
  'G8 o chroma junta o vermelho em luz (P4)' placa
muta $R/shaders/layer_composite.wgsl 'vec4<f32>(em_luz3(enc.rgb) * k, k)); // the glow is light' 'vec4<f32>(enc.rgb * k, k));' \
  'G9 o brilho do bloom é o codificado em vez da luz' placa
muta $R/shaders/layer_composite.wgsl 'vec4<f32>(display_luma(base.rgb), 0.0, 0.0, 0.0));' 'vec4<f32>(display_luma(em_luz(base).rgb), 0.0, 0.0, 0.0));' \
  'G10 a luma do S/H passa pela luz' placa
muta $R/shaders/layer_composite.wgsl '    var d = clamp(base_enc.rgb, vec3<f32>(0.0), vec3<f32>(1.0));' '    var d = em_luz3(base_enc.rgb);' \
  'G11 o S/H retoca a luz em vez dos tons de ecrã' placa

# ── P3: a fronteira de cada ajuste, na placa ──
muta $R/shaders/layer_composite.wgsl 'return em_tons_de_ecra3(adjust_hsb(em_luz3(rgb), ap.p0, ap.p1, ap.p2));' 'return adjust_hsb(rgb, ap.p0, ap.p1, ap.p2);' \
  'G12 o HSB da placa corre sobre o codificado' placa
muta $R/shaders/layer_composite.wgsl 'let v = max(em_luz3(rgb) * gain + vec3<f32>(ap.p1), vec3<f32>(0.0));' 'let v = max(rgb * gain + vec3<f32>(ap.p1), vec3<f32>(0.0));' \
  'G13 a Exposição da placa recebe o codificado como luz' placa
muta $R/shaders/layer_composite.wgsl 'return vec3<f32>(1.0) - clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));' 'return em_luz3(vec3<f32>(1.0) - em_tons_de_ecra3(rgb));' \
  'G14 o Invert da placa trata o codificado como luz' placa
muta $R/shaders/layer_composite.wgsl 'let ob = adj_luts[base + 2u * 256u + u32(s.b * 255.0 + 0.5)];
            return clamp(vec3<f32>(or_, og, ob), vec3<f32>(0.0), vec3<f32>(1.0));' 'let ob = adj_luts[base + 2u * 256u + u32(s.b * 255.0 + 0.5)];
            return em_luz3(vec3<f32>(or_, og, ob));' \
  'G15 as Curvas da placa devolvem luz' placa
muta $R/shaders/layer_composite.wgsl 'let v = select(0.0, 1.0, luma >= ap.p0 - 0.5 / 255.0);' 'let v = select(0.0, 1.0, luma >= ap.p0);' \
  'G16 o Threshold da placa corta em t/255' placa
muta $R/shaders/layer_composite.wgsl 'let blended = apply_blend(op.blend_mode, vec4<f32>(acc.rgb, 1.0), src_px);' 'let blended = apply_blend(op.blend_mode, acc, vec4<f32>(src_px.rgb, acc.a));' \
  'G17 a mistura de volta da placa volta ao over com o alfa da base' placa

# ── P3: a fronteira de cada ajuste, na CPU (ph2d-painter-effects) ──
E=crates/ph2d-painter-effects/src/adjustments
muta $E/compute/basic.rs '        let [r, g, b] = em_luz(px);
        let lab = OklabColor::from_linear(LinearRgba::new(r, g, b, 1.0));
        // Hue rotation' '        let [r, g, b] = [px[0], px[1], px[2]];
        let lab = OklabColor::from_linear(LinearRgba::new(r, g, b, 1.0));
        // Hue rotation' \
  'E1 o HSB recebe o codificado como luz' efeitos
muta $E/compute/basic.rs '        em_tons_de_ecra([r, g, bl], px);' '        px[0] = r;
        px[1] = g;
        px[2] = bl;' \
  'E2 o HSB devolve luz' efeitos
muta $E/compute/basic.rs '        let [r, g, b] = em_luz(px);
        let lab = OklabColor::from_linear(LinearRgba::new(r, g, b, 1.0));
        let chroma' '        let [r, g, b] = [px[0], px[1], px[2]];
        let lab = OklabColor::from_linear(LinearRgba::new(r, g, b, 1.0));
        let chroma' \
  'E3 a Vibrância recebe o codificado como luz' efeitos
muta $E/compute/basic.rs 'let lut = build_lut_em_luz(|v| (v * gain' 'let lut = build_lut(|v| (v * gain' \
  'E4 a Exposição corre sobre o codificado' efeitos
muta $E/compute/basic.rs 'let lut = build_lut_em_luz(|ch| {' 'let lut = build_lut(|ch| {' \
  'E5 Brilho/Contraste corre sobre o codificado (o pivô 0,214 deixa de ser o cinzento médio)' efeitos
muta $E/compute/basic.rs '        let mut l = em_luz(px);' '        let mut l = [px[0], px[1], px[2]];' \
  'E6 o Filtro de Foto recebe o codificado como luz' efeitos
muta $E/compute/basic.rs '        em_tons_de_ecra(l, px);' '        px[0] = l[0];
        px[1] = l[1];
        px[2] = l[2];' \
  'E7 o Filtro de Foto devolve luz' efeitos
muta $E/compute/curves.rs '*v = sample_display_lut(&luts[ch], *v).clamp(0.0, 1.0);' '*v = srgb_to_linear_f32(sample_display_lut(&luts[ch], linear_to_srgb_f32(*v)));' \
  'E8 as Curvas tratam o codificado como luz (a ida-e-volta antiga, dentro)' efeitos
muta $E/compute/levels.rs '*v = sample_display_lut(&lut, *v).clamp(0.0, 1.0);' '*v = srgb_to_linear_f32(sample_display_lut(&lut, linear_to_srgb_f32(*v)));' \
  'E9 os Níveis tratam o codificado como luz' efeitos
muta $E/compute/basic.rs '            *ch = 1.0 - ch.clamp(0.0, 1.0);' '            *ch = srgb_to_linear_f32(1.0 - linear_to_srgb_f32(*ch));' \
  'E10 o Invert trata o codificado como luz' efeitos
muta $E/compute/basic.rs '            *ch = (k as f32 / steps).min(1.0);' '            *ch = srgb_to_linear_f32((k as f32 / steps).min(1.0));' \
  'E11 o Posterize devolve a banda em luz' efeitos
muta $E/compute/basic.rs 'let cut = (p.threshold as f32 - 0.5) / 255.0;' 'let cut = p.threshold as f32 / 255.0;' \
  'E12 o Threshold corta em t/255' efeitos
muta $E/compute/channel_mixer.rs '            px[0] = mix(p.red_out, r, g, b);' '            px[0] = srgb_to_linear_f32(mix(p.red_out, r, g, b));' \
  'E13 o Misturador devolve o vermelho em luz' efeitos
muta $E/compute/color_balance.rs '            px[c] = ov.clamp(0.0, 1.0);' '            px[c] = srgb_to_linear_f32(*ov);' \
  'E14 o Equilíbrio de Cor devolve luz' efeitos
muta $E/compute/selective_color.rs '        px[0] = nr.clamp(0.0, 1.0);' '        px[0] = srgb_to_linear_f32(nr.clamp(0.0, 1.0));' \
  'E15 a Cor Seletiva devolve o vermelho em luz' efeitos
muta $E/compute/gradient_map.rs 'let lut = gradient_map_lut(p).map(|c| c.map(linear_to_srgb_f32));' 'let lut = gradient_map_lut(p);' \
  'E16 o Mapa de Gradiente devolve a cor da tabela em luz' efeitos
muta $E/spatial.rs '        *px = [px[0] * a, px[1] * a, px[2] * a, a];
    }
}

/// Inverse of [`premultiply`]' '        let l = em_luz(px);
        *px = [l[0] * a, l[1] * a, l[2] * a, a];
    }
}

/// Inverse of [`premultiply`]' \
  'E17 a porta dos blurs volta à luz (P4)' efeitos
muta $E/spatial.rs 'unpremultiply_with(buf, |v| v.clamp(0.0, 1.0));' 'unpremultiply_with(buf, linear_to_srgb_f32);' \
  'E18 a porta dos blurs devolve luz (P4)' efeitos
muta $E/spatial_tonal.rs '    premultiply_em_luz(acc);' '    premultiply(acc);' \
  'E23 o bloom soma o brilho à base codificada (CPU)' efeitos
muta $E/spatial_tonal.rs '    unpremultiply_em_luz(acc);' '    unpremultiply(acc);' \
  'E24 o bloom devolve a luz como se fosse codificado (CPU)' efeitos
muta $E/spatial.rs '                px[c] = (px[c] + p.amount * n * NOISE_SCALE).clamp(0.0, 1.0);' '                px[c] = super::compute::srgb_to_linear_f32((linear_to_srgb_f32(px[c]) + p.amount * n * NOISE_SCALE).clamp(0.0, 1.0));' \
  'E19 o Ruído trata o codificado como luz' placa
muta $E/lut.rs '            px[c] = d[c] + (clamp01(graded[c]) - d[c]) * amount;' '            px[c] = (d[c] + (clamp01(graded[c]) - d[c]) * amount).powf(2.2);' \
  'E20 o Color Lookup devolve luz' placa
muta $E/spatial_tonal.rs '                let l = em_luz(&base); // the glow is light' '                let l = [base[0], base[1], base[2]];' \
  'E21 o brilho do bloom é o codificado em vez da luz (CPU)' placa
muta $E/spatial_tonal.rs '            let mut d = [
                px[0].clamp(0.0, 1.0),' '            let mut d = [
                linear_to_srgb_f32(px[0]),' \
  'E22 o S/H trata o vermelho codificado como luz (CPU)' placa

echo "RESULTADO: $sangram de $total sangraram${SO_ANCORAS:+ (pré-voo: âncoras únicas)}"
