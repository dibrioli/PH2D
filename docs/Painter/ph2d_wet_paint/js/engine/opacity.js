// Saturating pigment-opacity table (spec section 3).
//
// Coverage is not linear in pigment mass: each unit of mass covers 0.2% of
// whatever paper is still showing, i.e. alpha(m) = 1 - 0.998^m. Precomputed
// once as a 3001-entry table; every consumer truncates its mass to an integer
// index and clamps into the table, so mass >= 3000 reads fully opaque.
// Half-opacity sits near mass ~346 - a light wash (100..400) lets the paper
// glow through, which is what makes deposits read as watercolor.

export const OPACITY_TABLE_MAX = 3000;

export function buildOpacityTable() {
  const t = new Float32Array(OPACITY_TABLE_MAX + 1);
  t[0] = 0;
  for (let m = 1; m <= OPACITY_TABLE_MAX; m++) {
    t[m] = 0.002 + 0.998 * t[m - 1];
  }
  t[OPACITY_TABLE_MAX] = 1; // force exact saturation at the top entry
  return t;
}

/** Shared singleton - the table is a pure constant. */
export const OPACITY = buildOpacityTable();

/** Opacity lookup: truncate mass to int, clamp into the table. */
export function alphaOfMass(m) {
  let i = m | 0;
  if (i <= 0) return 0;
  if (i >= OPACITY_TABLE_MAX) return 1;
  return OPACITY[i];
}
