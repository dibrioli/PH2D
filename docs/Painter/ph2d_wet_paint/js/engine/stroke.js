// Stroke engine (spec section 8): synthetic pressure + Catmull-Rom geometry.
//
// There is no pen pressure; SLOW = HEAVY. The recurrence
//   t' = 11 - min(8, (t + speedPrevFrame) / 2), slew-limited to +-0.5/frame
// starts at 0 on press (strokes fade in over ~0.25 s instead of landing as a
// blob), builds toward 7.33 while holding still, and drops toward 3 on a fast
// drag. Speed is read ONE FRAME LATE by design. The engine must be fed
// exactly one sample per 40 Hz sim step - feeding per pointer event would
// change the pressure model and over-deposit.
//
// Geometry: a uniform Catmull-Rom spline through the frame samples (first /
// last control points duplicated), walked in ~1 px micro-steps, emitting a
// dab every `spacing` px of arc length regardless of speed. Each dab's
// pressure lerps between the segment endpoints' t values.

export function createStrokeEngine(onSegment, onDab) {
  return {
    onSegment, // (chordLength) -> called BEFORE that segment's dabs
    onDab,     // (x, y, pressure, dirX, dirY)
    active: false,
    releasing: false,
    t: 0,
    prevSpeed: 0,
    lastX: 0, lastY: 0,
    pts: [],          // recorded control points {x, y, t}
    arcAccum: 0,
    dirX: 0, dirY: 0, // current stroke direction (unit); 0,0 = none yet
    firstDabDone: false,
    spacing: 2,
  };
}

export function strokeBegin(s, x, y, spacing) {
  s.active = true;
  s.releasing = false;
  s.t = 0;
  s.prevSpeed = 0;
  s.lastX = x; s.lastY = y;
  s.pts = [{ x, y, t: 0 }];
  s.arcAccum = 0;
  s.dirX = 0; s.dirY = 0;
  s.firstDabDone = false;
  s.spacing = Math.max(0.5, spacing || 2);
}

/**
 * One pointer sample per 40 Hz step while the pointer is down. Pressure ticks
 * every frame; geometry only advances when the frame's hop is >= 0.5 px.
 */
export function strokeTick(s, x, y) {
  if (!s.active || s.releasing) return;
  const dx = x - s.lastX, dy = y - s.lastY;
  const hop = Math.sqrt(dx * dx + dy * dy);
  tickPressure(s, hop);
  s.lastX = x; s.lastY = y;
  if (hop >= 0.5) pushPoint(s, x, y);
}

function tickPressure(s, speedThisFrame) {
  // The recurrence reads the PREVIOUS frame's speed.
  const target = 11 - Math.min(8, (s.t + s.prevSpeed) / 2);
  let d = target - s.t;
  if (d > 0.5) d = 0.5; else if (d < -0.5) d = -0.5;
  s.t += d;
  if (s.t < 0) s.t = 0;
  s.prevSpeed = speedThisFrame;
}

/** Pointer released: enter the fade-out tail. */
export function strokeRelease(s) {
  if (s.active) s.releasing = true;
}

/**
 * Tick the release tail at 40 Hz with the current (hover) pointer position:
 * pressure decays as t = (t - 0.5) * 0.6, and geometry keeps flowing while
 * the cursor still moves (hop >= 1 px) and t > 0.01 - the stroke lifts off
 * with a fading tail. Returns false once the stroke is fully finished
 * (spline flushed; the caller then drops the trail remainder).
 */
export function strokeReleaseTick(s, x, y) {
  if (!s.active) return false;
  s.t = (s.t - 0.5) * 0.6;
  if (s.t < 0) s.t = 0;
  const dx = x - s.lastX, dy = y - s.lastY;
  const hop = Math.sqrt(dx * dx + dy * dy);
  s.lastX = x; s.lastY = y;
  if (hop >= 1 && s.t > 0.01) {
    pushPoint(s, x, y);
    return true;
  }
  finishStroke(s);
  return false;
}

/** Flush the spline tail (duplicate the last point once) and deactivate. */
export function finishStroke(s) {
  if (!s.active) return;
  const n = s.pts.length;
  if (n >= 2) {
    const last = s.pts[n - 1];
    pushPoint(s, last.x, last.y, /*forceDuplicate*/ true);
  }
  s.active = false;
  s.releasing = false;
}

function pushPoint(s, x, y, forceDuplicate = false) {
  s.pts.push({ x, y, t: s.t });
  const m = s.pts.length;
  // Segment Pk -> Pk+1 becomes emittable once Pk+2 exists (p3). The first
  // segment duplicates P0 as its p0; forceDuplicate flushes the tail with
  // p3 = p2.
  if (m >= 3) {
    const p3 = s.pts[m - 1];
    const p2 = s.pts[m - 2];
    const p1 = s.pts[m - 3];
    const p0 = m >= 4 ? s.pts[m - 4] : p1; // duplicate the first point
    emitSegment(s, p0, p1, p2, forceDuplicate ? p2 : p3);
    // Wait - on a normal push the new segment is (m-3 -> m-2) with p3 = the
    // fresh point; on the duplicate flush the fresh point IS the duplicate,
    // so the segment is (m-3 -> m-2) with p3 = p2... which is exactly the
    // same call: p3 duplicates the last real point. Nothing special needed.
  }
  if (s.pts.length > 64) s.pts.splice(0, s.pts.length - 4); // keep the tail only
}

/** Uniform Catmull-Rom position at fraction u in [0,1] of segment p1->p2. */
function catmullRom(p0, p1, p2, p3, u, out) {
  const u2 = u * u, u3 = u2 * u;
  out.x = 0.5 * (2 * p1.x + (-p0.x + p2.x) * u
    + (2 * p0.x - 5 * p1.x + 4 * p2.x - p3.x) * u2
    + (-p0.x + 3 * p1.x - 3 * p2.x + p3.x) * u3);
  out.y = 0.5 * (2 * p1.y + (-p0.y + p2.y) * u
    + (2 * p0.y - 5 * p1.y + 4 * p2.y - p3.y) * u2
    + (-p0.y + 3 * p1.y - 3 * p2.y + p3.y) * u3);
}

const scratch = { x: 0, y: 0 };

function emitSegment(s, p0, p1, p2, p3) {
  const cdx = p2.x - p1.x, cdy = p2.y - p1.y;
  const chord = Math.sqrt(cdx * cdx + cdy * cdy);
  if (s.onSegment) s.onSegment(chord); // trail sizes its window BEFORE the dabs
  // The very first dab of a stroke lands exactly at the press point with its
  // recorded pressure (0) - guarantees the fade-in starts from nothing.
  if (!s.firstDabDone) {
    s.firstDabDone = true;
    if (s.onDab) s.onDab(p1.x, p1.y, p1.t, s.dirX, s.dirY);
  }
  const steps = Math.max(2, Math.ceil(chord));
  let px = p1.x, py = p1.y;
  for (let j = 1; j <= steps; j++) {
    const u = j / steps;
    catmullRom(p0, p1, p2, p3, u, scratch);
    const dx = scratch.x - px, dy = scratch.y - py;
    const d = Math.sqrt(dx * dx + dy * dy);
    if (d > 0) { s.dirX = dx / d; s.dirY = dy / d; }
    s.arcAccum += d;
    while (s.arcAccum >= s.spacing) {
      s.arcAccum -= s.spacing;
      const b = p1.t + (p2.t - p1.t) * u; // pressure lerped along the segment
      if (s.onDab) s.onDab(scratch.x, scratch.y, b, s.dirX, s.dirY);
    }
    px = scratch.x; py = scratch.y;
  }
}
