# Game SFX creation tools: state of the art, interaction patterns and open space for PH2D (as of 2026-11-05)

Evidence labels used in these notes:
- **[page]**: I opened the page.
- **[excerpt]**: seen only as a search-result excerpt, because the page blocked fetching or I did not open it.
- **[vendor]**: the author sells the tool or a competitor.
- **[old]**: older than 2024, or the date is unknown.

PH2D constraint: every tool below is a source of *ideas about behaviour*. Do not copy proprietary code, presets, recordings, UI artwork or product names.

## What do these tools do, what is their signature interaction, price, strengths and weaknesses?

### Takeaway
The market splits into five families:
1. Instant toy generators (sfxr family, ChipTone).
2. Domain-specific procedural synths (GameSynth, DSP series, SoundSeed).
3. Performed sample tools (Krotos).
4. General synths that designers borrow (Serum, Vital, Phase Plant, Synplant).
5. Text or video generative tools (ElevenLabs, Firefly, Woosh).

No tool I found combines fast ear-first exploration, deep procedural control and creation inside a running game.

### Cited Findings

#### Comparison table
Prices are the latest I could see and are not all current. Check them before quoting.

| Tool | What it is | Signature interaction | Price | Strengths | Weaknesses |
|---|---|---|---|---|---|
| **jsfxr** (sfxr port) | Browser retro SFX generator | One click on a category (Pickup, Laser, Explosion, Powerup, Hit, Jump, Click, Blip, Synth, Tone), plus Random and Mutate; a sound can be shared as a permalink or serialized text — [page](https://sfxr.me/) | Free; Pro adds cloud sync, MP3/OGG and zip packs; unrestricted commercial use — [page](https://sfxr.me/) | Seconds to first result; the sound is its parameters | Retro timbre only (my inference) |
| **Bfxr** | sfxr descendant, open source | R randomizes, M mutates, L locks or unlocks all; collection files (`.bcol`), catalogue generator, Footsteppr port — [page](https://www.bfxr.net/) | Free; licence not named on the page — [page](https://www.bfxr.net/) | Keyboard-driven loop; collections | Licence must be checked in the repository |
| **ChipTone** | SFX generator from SFBGames | Mouse-driven; a commenter says settings run left to right from simple to complex; randomize-all is a favourite that "sometimes... creates absolute Horror" — [itch page](https://sfbgames.itch.io/chiptone), [comments, excerpt](https://sfbgames.itch.io/chiptone/comments?after=16) | Free or pay-what-you-want; output is CC0 — [page](https://sfbgames.itch.io/chiptone) | 4.9/5 from 1,072 ratings; clear rights | Last devlog Jan 2021; sampler and sequencer unfinished — [page](https://sfbgames.itch.io/chiptone) |
| **GameSynth** (Tsugi) | Procedural SFX workstation | Sketch Pad: draw to control parameters, where "speed, pressure and trajectories are all taken into account"; Automatic Variations; 130+ module patcher — [page](https://tsugi-studio.com/web/en/products-gamesynth.html) | $390 regular, $273 intro; Windows only as of the 2022 report — [excerpts](https://www.sonicstate.com/news/2022/03/31/tsugi-releases-game-synth-2022/), [2023 sale](https://www.toolfarm.com/?p=103082) | Models for Whoosh, Impact, Retro, Particles, VoiceFX, Footsteps, Weather; "hundreds" of WAVs in one click; CLI; API for Unity, Reaper, Pd, Blender; curve import from Maya and Unity — [page](https://tsugi-studio.com/web/en/products-gamesynth.html) | Newest version shown is 2023.1.2 — [page](https://tsugi-studio.com/web/en/products-gamesynth.html); price complaints — [comments, 2021](https://synthtopia.com/?p=122160) |
| **DSP Motion / DSP Action** (Tsugi) | Small "draw a sound" apps | Mouse or stylus movement becomes sound; choose the driving motion property (such as rotation or scale) and a style (fire, water, electricity...); drawing is synced to video — [CG Channel](https://www.cgchannel.com/?p=118178) | $49, Windows only — [CG Channel, 2022](https://www.cgchannel.com/?p=118178) | Retiming without artefacts; variations; Motion exports to Unity with metafiles and C# playback scripts — [CG Channel](https://www.cgchannel.com/?p=118178) | Narrower than GameSynth |
| **DSP Anime** (Tsugi) | Anime and game SFX generator | Category models (magic, combat, emotion, events, pets, elements, impact); no evidence of drawn gestures — [CGWORLD, 2015](https://cgworld.jp/news/tool/1507-dsp-anime12tsugi.html) | ¥4,900 / $49 in 2015 — [same](https://cgworld.jp/news/tool/1507-dsp-anime12tsugi.html) | Direct export to Unity, Wwise, FMOD | [old] current status unverified |
| **Krotos Studio** | Performed sound effects from recordings | XY pads driven by dragging; click inside the left pad to trigger Foley — [Krotos, vendor, excerpt](https://krotos.studio/blog/foley-sound-effects-category-guide). Records in the background with no record button; drag the take out — [Krotos FAQ, excerpt](https://krotos.studio/faq) | Studio €9.99 (yearly) or €19.99 (monthly); Pro €23.33 or €49.99; Max €83.33, yearly only — [page](https://krotos.studio/pricing) | Immediacy; game rights start at Pro; Max includes the plugin suite — [page](https://krotos.studio/pricing) | Subscription only; base tier cannot use your own sounds — [FAQ, excerpt](https://krotos.studio/faq); monthly plugin render caps of 50, 100 or 200 — [page](https://krotos.studio/pricing) |
| **Krotos Igniter** | Vehicle designer | Perform vehicles via MIDI or DAW automation; Granular, Synth, One Shot and Loop modules — [page](https://krotos.studio/igniter) | $399; Full Tank $999 — [page](https://krotos.studio/igniter) | 24 vehicles, 630 samples, 250+ presets | The page shows no mouse or XY RPM control and no game export |
| **Krotos Reformer Pro** | Library becomes an instrument | A mic or any audio drives a sound library in real time; Dynamic Input scans a library by frequency or amplitude without audio input; your own folders can be analysed — [page](https://krotos.studio/reformer-pro). It has an XY selector — [SoundFlow forum, excerpt](https://forum.soundflow.org/-1916/how-to-control-the-x-y-selector-in-krotos-reformer-pro) | $399 — [page](https://krotos.studio/reformer-pro) | Foley in minutes; library blending | A user reports no OSC support — [same forum, excerpt](https://forum.soundflow.org/-1916/how-to-control-the-x-y-selector-in-krotos-reformer-pro) |
| **Krotos Dehumaniser 2** | Creature and voice processor | Reacts to the input signal; triggers by amplitude, pitch or descriptors; node-based signal flow; macro "drawers" — [page](https://krotos.studio/dehumaniser-2) | $399 on the page; another listing gives $478.80, cut to $79.80 on sale — [press](https://krotos.hummingbirdmedia.com/krotos-announces-halloween-sale-80-off-dehumaniser-2-and-simple-monsters-craft-monstrous-otherworldly-vocal-effects) | The voice is the controller | List price conflicts between sources |
| **Krotos Concept** | Soft synth | MusicRadar showed the synth in a video; I found no reliable description of its interaction — [MusicRadar](https://www.musicradar.com/news/watch-krotos-concept-soft-synth-explored) | Unverified | n/a | The old product URL now redirects to the Studio Max page (seen on 2026-11-05), and it is absent from Krotos's "iconic plugins" list — [about page](https://krotos.studio/about). It may be discontinued; unconfirmed |
| **Synplant 2** (Sonic Charge) | Ear-first genetic synth | Plant a seed; 12 branches grow outward; distance from the centre sets how much the sound changes, and dragging back undoes it; replant any branch as the new seed — [Sound On Sound](https://www.soundonsound.com/reviews/sonic-charge-synplant-2) | €149; 3-week trial — [Sound On Sound](https://www.soundonsound.com/reviews/sonic-charge-synplant-2) | Genopatch grows four strands of matches from up to 2 s of audio; "fun... exploring sound design in a more tangible way" — [MusicTech](https://musictech.com/reviews/plug-ins/sonic-charges-synplant-2-review/) | See complaints below |
| **Serum 2** (Xfer) | Flagship wavetable synth | Third oscillator with Wavetable, Multisample, Sample, Granular and Spectral modes; clip sequencer; chaos modulation from strange attractors — [CDM, Mar 2025](https://cdm.link/serum-2/) | $249; free for Serum 1 owners — [CDM](https://cdm.link/serum-2/) | Keeps the known workflow | No Linux; granular "doesn't look quite as deep" as rivals — [CDM](https://cdm.link/serum-2/) |
| **Vital** | Spectral-warping wavetable synth | Text to wavetable — [LANDR](https://blog.landr.com/vital-synth) | Free; Plus $25; Pro $80; $5/month; free tier gets 5 text-to-wavetable requests a day — [excerpt](https://www.kvraudio.com/product/vital-by-matt-tytel/reviews) | Full synth for free; KVR 4.79 from 24 reviews | Fewer presets and wavetables in the free tier |
| **Phase Plant** (Kilohearts) | Semi-modular synth | Drag generators and effect "snapins" into place; generators can modulate each other — [page](https://kilohearts.com/products/phase_plant) | $199; 10-day trial; $9.99/month subscription — [page](https://kilohearts.com/products/phase_plant) | Used for game variation patches — [Kilohearts, vendor](https://kilohearts.com/blog/game_audio_sound_design_with_phase_plant) | Premium effects sold separately |
| **Output Portal** | Granular effect | Circular XY linked to granular parameters; macros — [excerpts](https://www.pluginboutique.com/products/5458) | $149 at launch — [Synth Anatomy](https://synthanatomy.com/2019/06/output-portal-granular-effect-plugin.html) | "One of the greatest effects plugins ever made" — [MusicRadar](https://musicradar.com/reviews/output-portal) | A KVR user found granular "too chaotic" — [KVR](https://www.kvraudio.com/forum/viewtopic.php?p=7440207) |
| **Tonsturm** | Motion-sound tools plus libraries | Traveler: draw a path for a sound past a listener, with Duration, Length and Speed — [Noizefield, 2018](https://www.noizefield.com/?p=29782). Whoosh: granular, Doppler-based, v1.5.2 — [excerpt](https://tonsturm.com/product/whoosh) | Traveler $229 in 2018; Whoosh $99 at review time — [Audio Spotlight](https://theaudiospotlight.com/?p=54331) | Path as the interface; catalogue includes Spectral Warp (marked new) — [store](https://tonsturm.com/) | Sources disagree on whether Whoosh needs Reaktor: [AI-written blog](https://www.daviddumaisaudio.com/which-tonsturm-whoosh-plugin-should-you-get/) vs [KVR listing as VST/AU/AAX](https://www.kvraudio.com/product/whoosh-by-tonsturm) |
| **SoundMorph** | Libraries plus plugins | Wave Warper 2: 8-layer morphing granular and wavetable whoosh engine — [Equipboard](https://equipboard.com/items/soundmorph-wave-warper-2); Dust granular synth — [Plugin Boutique](https://www.pluginboutique.com/manufacturers/373-SoundMorph) | About $149 for each — same sources | Paired with content packs | No product matched "Whoosh Morph" or similar names |
| **Unreal MetaSounds** | Engine-native DSP graph | Each sound is its own rendering graph, sample accurate; a preset references a base graph and overrides inputs; base edits propagate; live input preview — [Epic docs](https://dev.epicgames.com/documentation/unreal-engine/metasounds-the-next-generation-sound-sources-in-unreal-engine?lang=en-US) | Included in Unreal (my reading of the source; licence terms not checked) | Runtime procedural sound with no separate tool; graphs compile to optimized C++ | Middleware still leads on advanced spatial and adaptive music, per a [2026 blog](https://www.strayspark.studio/blog/wwise-fmod-metasounds-audio-middleware-comparison) |
| **Wwise SoundSeed** | Runtime synthesis plugins | Grain: granular, cloud, pulsar, concatenative and wavetable — [Audiokinetic](https://www.audiokinetic.com/learning/videos/MfE6DqXSgXg). Impact: analyses resonances, then resynthesizes — [Audiokinetic](https://www.audiokinetic.com/ko/community/wwisetour/pq_pafng4qs) | Indie tier free under a $250,000 budget — [excerpt](https://audiokinetic.com/pricing/for-games); plugin prices priced separately — [blog](https://blog.audiokinetic.com/en/wwise-licensing-and-pricing-philosophy/) | Runs inside the middleware | [old] current status unverified |
| **FMOD** | Middleware | Praised as "fantastic and super easy" in Unreal by one indie — [r/gamedev mirror, excerpt](https://r.enesgenc.dev/r/gamedev/comments/1kajdgw/fmod_vs_wwise_for_small_indie_teamswhats_your_take) | Not retrieved | Implementation rather than creation | n/a |
| **Max** | Visual interactive-software patcher | Objects joined by patchcords; Gen can produce source code — [page](https://cycling74.com/products/max) | 30-day trial; price not shown — [page](https://cycling74.com/products/max) | Unlimited depth | Build-your-own |
| **Reaktor** | Modular environment | Community discussion concerns its future rather than its interaction — [KVR](https://www.kvraudio.com/forum/viewtopic.php?p=9196331) | Not retrieved | n/a | Native Instruments entered preliminary insolvency in Jan 2026 — [BPB](https://bedroomproducersblog.com/2026/01/28/native-instruments-insolvency/); the CEO says business continues — [Mixmag](https://mixmag.net/read/native-instruments-says-business-continues-usual-after-insolvency-proceedings-news) |

#### New in 2025-2026
- **Serum 2:** released 17 March 2025 — [CDM](https://cdm.link/serum-2/).
- **Krotos Video to Sound:** AI matches footage to *recorded* sounds rather than generating audio. It launched in the browser at IBC 2025 — [ProVideo Coalition](https://www.provideocoalition.com/ibc-2025-krotos-introduces-video-to-sound-ai-assisted-tool-for-automatic-audio-sync/). A Premiere Pro plugin and a Resolve plugin followed — [Production Expert](https://www.production-expert.com/production-expert-1/krotos-announces-video-to-sound-integration-with-adobe-premiere-pro), [press](https://news.hummingbirdmedia.com/krotos-launches-video-to-sound-plugin-for-davinci-resolve). Launch types are ambiences, whooshes, transitions, risers, impacts and cloth.
- **Sony AI Woosh (18 May 2026):** an SFX foundation model with text-to-audio and video-to-audio. Public weights are non-commercial. A DAW plugin is in development, with planned variations and inpainting; the roadmap adds time controls, morphing and perfect loops — [Sony AI](https://ai.sony/archive/introducing-woosh-sony-ais-sound-effect-foundation-model).
- **Adobe Firefly sound effects:** describe a sound, then act its timing and intensity into a microphone. Each effect costs 10 credits; plans run from $9.99/month — [Cinevva, vendor, Sep 2026](https://app.cinevva.com/guides/ai-sound-effect-generators).
- **ElevenLabs Sound Effects v2:** clips up to 30 s, seamless looping, 48 kHz — [The Decoder](https://the-decoder.com/elevenlabs-releases-version-2-of-its-ai-sound-effects-model-with-longer-clips-and-better-audio-quality/).
- **Ludo:** API, MCP server and CLI let a coding agent generate SFX during development — [Ludo, vendor](https://ludo.ai/tools/game-sound-effects-generator).
- **SFX Engine:** text-to-SFX with a batch API — [listing](https://agenticgamedevelopment.com/tool/sfx-engine/).
- **In-engine plugins for Godot:**
  - gdfxr (MIT): a bottom panel in the editor; `.sfxr` files of about 100 bytes act as normal AudioStreams; looping, bit depth and sample rate live in the Import dock — [GitHub](https://github.com/timothyqiu/gdfxr).
  - GodotSfxr: player nodes with generators in the inspector — [asset page](https://godotassetlibrary.com/asset/tQDKxA/godotsfxr-(for-godot-4.x)).
  - UnlockSFX: an AI prompt inside the editor, which needs an account, credits and internet — [store](https://store.godotengine.org/asset/unlocksfx/unlocksfx/).

### Inferences
- GameSynth is the closest existing analogue to a PH2D module. Its visible release cadence has stalled since 2023, which leaves room, but that is absence of evidence and not proof of abandonment.
- gdfxr proves the engine-native parametric asset works, but it inherits sfxr's timbral ceiling. A deeper version of that idea looks more original than a better standalone synth.
- Several flagship tools do not run natively on Linux according to these sources: Serum 2 and, as last reported, GameSynth and the DSP apps. ChipTone users also request Linux. PH2D is developed on Linux, so this is a small differentiator.

### Gaps
- **GameSynth:** I could not confirm the 2026 price, platforms or any release after 2023.1.2.
- **Krotos Concept:** current status is unconfirmed.
- **FMOD:** licensing figures were not retrievable.
- **Wwise:** the official pricing page blocked fetching, and SoundSeed's deprecation status is unknown.
- **Pure Data:** I found no usable source on Pd, libpd or Heavy for game SFX, so it has no table row.
- **Reaktor:** its price and maintenance status are unknown.
- **Licences to verify before any oracle use:** Bfxr and Vital. Licences must be read from the installed artifact, as CLAUDE.md §0.9 requires, and not from these notes.
- **ChipTone author:** the [Boombox review](https://boombox-post.squarespace.com/blog/2018/5/25/lunch-and-learn-chip-tone) was reported to me with the name "Ton Vain". My unverified recollection is Tom Vian. Verify before naming him.

## What do professional game sound designers say about their workflow?

### Takeaway
The recurring loop is: decompose, make small variation sets, try in the game, restart if wrong, then rely on naming and batch export. The strongest repeated theme is that a sound is judged only in context. I found little primary material on the fine detail, because the A Sound Effect articles blocked fetching and I saw them only as excerpts.

### Cited Findings

#### Layering and decomposition
- Richard Stevens splits a sound such as an explosion into components, keeps several versions of each, and lets the system combine them randomly — [A Sound Effect, excerpt](https://www.asoundeffect.com/?p=27166).
- Stacked presets add detail, such as several footstep layers for grass, and lowering "Strength" on a gunshot imitates a suppressor — [Krotos, vendor, 2023](https://krotos.studio/blog/krotos-studio-game-audio-ableton).

#### Variations and round robin
- *Stray* built physics sounds with multiple round-robin or velocity-mapped alternatives through a custom Unreal component, then added random pitch and volume to get more from the same files — [A Sound Effect, excerpt](https://www.asoundeffect.com/stray-game-audio/).
- *The Last of Us Part II* used loops for near-infinite variation with a small memory footprint — [A Sound Effect, excerpt](https://www.asoundeffect.com/the-last-of-us-part-ii-sound/).
- File size limits how many variations one event can have — [Game Audio Explained, excerpt](https://www.asoundeffect.com/gameaudioexplained/).
- Aaron Wolff: a patch that lets you "quickly generate infinite variations" is what helps in a one-shot workflow, in formats like "Tonal Bullet Ricochet Hit" or "Sword Swish" that users can tweak. Tools should give "a frictionless path to executing an idea" — [Kilohearts, vendor](https://kilohearts.com/blog/game_audio_sound_design_with_phase_plant).
- Vendor advice: perform small sets such as soft/hard, close/distant and small/large — [Krotos, vendor](https://krotos.studio/blog/audio-for-games).
- `nvk_VARIATIONS` ($99, REAPER) generates variations from selected items, and nothing is saved until "Apply" — [Gumroad](https://nvktools.gumroad.com/l/nvk_VARIATIONS).

#### Design versus implementation, and in-game audition
- The *Stray* team tried sounds in the game and restarted the process if unhappy — [A Sound Effect, excerpt](https://www.asoundeffect.com/stray-game-audio/). The same piece describes a month reworking systems late on, because arbitrary-offset playback was unreliable on some platforms.
- Junya Osada (Nintendo, GDC 2024): late audio work "sometimes limits the ways our team was able to implement ideas and have influence on the game". TotK objects have no one-off sounds; a wagon's sound emerges from wheels, frame and chains, modified by distance, interaction, size and material. It produced sounds no one had authored — [Game Developer](https://gamedeveloper.com/audio/tears-of-the-kingdom-s-sound-designers-unintentionally-made-a-physics-engine-for-sound-).
- The Jurassic World Evolution designer played a few quick variations back-to-back against reference sounds, then matched with EQ — [A Sound Effect, excerpt](https://www.asoundeffect.com/jurassic-world-evolution-sound/).
- Timing and mix problems show up only when the scene runs. A generated file still needs import, trim, gain, a player node and event wiring: "a .wav file is not a game sound" — [Summer Engine, vendor](https://www.summerengine.com/blog/ai-sound-effect-generator-for-games).
- GDC 2010, Fournel: procedural audio needs design choices, typical models, tool and engine requirements, and QA across production — [GDC Vault](https://www.gdcvault.com/play/1012645/ProceduralAudio-for-Video-Games) [old; full talk is members-only].

#### Naming and batch export
- REAPER's region render matrix selects which tracks and regions render together, with regions standing for variations or states. Wildcards such as `$track`, `$region` and `$regionnumber` drive naming across hundreds of WAVs — [HdM REAPER guide, excerpt](https://curdt.home.hdm-stuttgart.de/PDF/Reaper.pdf).
- ReaWwise reads the REAPER render settings to decide which assets to transfer and builds Wwise object paths from them — [Audiokinetic](https://www.audiokinetic.com/en/reawwise-connecting-reaper-and-wwise).
- One forum user names projects "as if they were wwise events". The excerpt was ambiguous between [this r/GameAudio mirror](https://lr.sudovanilla.org/r/GameAudio/comments/1feif65/whats_your_asset_export_process_from_your_daw), which returned 404, and [this Steinberg thread](https://forums.steinberg.net/t/more-intuitive-workflow-for-game-audio-assets/156861).
- GameSynth supports the Universal Category System (UCS) — [Synthtopia](https://www.synthtopia.com/content/2023/06/09/tsugi-releases-gamesynth-2023-a-major-update-to-the-procedural-sound-design-tool/). It exports to Wwise, FMOD Studio and Unity — [page](https://tsugi-studio.com/web/en/products-gamesynth.html).

#### Loudness and delivery
- Sony ASWG-R001 sets about -23 LUFS for PS3/PS4 and -18 LUFS for Vita, tolerance ±2 LU, -1 dBTP maximum, and 20 LU maximum loudness range — [Sony slides, excerpt](https://sambuz.com/doc/loudness-and-how-to-measure-it-ppt-presentation-886969) [old].
- ITU, 2018: typical game loudness is -24±2 LKFS on consoles and -18±2 on portables — [ITU-R BS.2434](https://www.itu.int/dms_pub/itu-r/opb/rep/R-REP-BS.2434-2018-PDF-E.pdf).
- These are *whole-game mix* programme targets, not per-asset targets (my reading).
- Per-asset guidance: an Audiokinetic-hosted guide lists RMS references for console assets, including SFX at -16 dB and dialogue at -12 dB — [Audiokinetic, excerpt](https://blog.audiokinetic.com/loudness-processing-best-practice-chapter-3-scalable-loudness-processing-for-games/).
- Delivery: convert one-shots to OGG or Opus, ideally under 50 KB — [Cinevva, vendor](https://app.cinevva.com/guides/ai-sound-effect-generators).

### Inferences
- The SFX module's deliverable should be an **event-ready set**: variations, playback rules, category, loudness state and a memory cost. A WAV alone falls short, because every downstream step above exists to rebuild that information.
- Osada's wagon suggests a further level for PH2D: sound derived from an object's parts and materials. PH2D already has physics and components (CLAUDE.md §5.1).
- Back-to-back comparison against a reference is a professional habit that few of the creation tools reviewed here advertise. It would be cheap to make first-class.

### Gaps
- I could not read the full text of the A Sound Effect interviews (403), and the Reddit mirrors failed (404 or TLS).
- GDC Vault talks beyond abstracts are members-only.
- The practitioner evidence is therefore thinner than the question deserves. These points are not a survey.
- I found no authoritative per-category SFX loudness standard, and no Tencent guidance.

## Which interaction patterns make sound design fun and immediate?

### Takeaway
Fun comes from a short loop in which the user judges sounds rather than parameters, with cheap, reversible surprise. The most valuable patterns are:
- Synplant's continuous distance-as-mutation.
- GameSynth's and DSP Motion's drawing.
- Krotos's always-recording performance.
- Reformer's and Firefly's use of the voice as a controller.

### Cited Findings

#### Randomise and mutate
- Bfxr binds R, M and L to single keys — [page](https://www.bfxr.net/).
- A reviewer suggests using ChipTone's randomize to build a whole library of varied beeps — [Boombox](https://boombox-post.squarespace.com/blog/2018/5/25/lunch-and-learn-chip-tone).

#### Evolution and breeding
- Synplant: the distance a branch is dragged scales the change, and dragging back reverses it. You can audition by dragging or from MIDI notes. Replanting makes a branch the new seed — [Sound On Sound](https://www.soundonsound.com/reviews/sonic-charge-synplant-2). The "Atonality" control moves results from pitched to experimental — same source.
- MusicTech recommends it for those intimidated by deep synth editing — [MusicTech](https://musictech.com/reviews/plug-ins/sonic-charges-synplant-2-review/).
- Johnson (1999): exploring parameter spaces demands experience or specialist knowledge, so let users judge sounds and let the GA manage parameters — [Kent](https://kar.kent.ac.uk/21844/).
- Yee-King (2016): modular users (430 respondents) and Evosynth users (14) both valued unexpected output and worked exploratively. Evosynth's server logged 3,552 breed events and 90 sounds saved as "fit" in a month — [Goldsmiths](https://research.gold.ac.uk/id/eprint/20164). That is about 2.5 saves per 100 breeds (my calculation).
- McDermott et al.: slow user evaluation causes small populations and fatigue. Their formal usability experiments showed a new interactive EC interface improves performance — [excerpt](https://unpaywall.org/10.1007%2F978-3-540-78761-7_54).
- SoundTraveller offers evolutionary and morphing modes, generates hundreds of timbres without per-parameter editing, and in its study decreased cognitive load and raised perceived creativity — [Oulu, excerpt only, page returned 403](https://oulurepo.oulu.fi/handle/10024/47381).
- Dahlstedt treats exploring a huge parameter space as an act of creation — [MutaSynth](https://www.dmu.ac.uk/documents/technology-documents/research/mtirc/nowalls/mww-dahlstedt.pdf).

#### Gesture and drawing
- GameSynth's sketch uses speed, pressure and trajectory — [page](https://tsugi-studio.com/web/en/products-gamesynth.html). It also added tablet support, SVG import — [excerpt](https://www.toolfarm.com/?p=103082) — and curve scripting — [Synthtopia, 2021](https://synthtopia.com/?p=122160).
- DSP Motion and GameSynth start and stop video with the drawing, so sound-to-picture is a single gesture — [Synthtopia](https://www.synthtopia.com/content/2023/06/09/tsugi-releases-gamesynth-2023-a-major-update-to-the-procedural-sound-design-tool/), [Sonic State](https://sonicstate.com/news/2022/11/10/-new-versions-of-sound-drawing-tools).
- Traveler's drawn path to a listener is another version — [Noizefield](https://www.noizefield.com/?p=29782).

#### Performance with no ceremony
- Krotos Studio records in the background, needs no record button, and the take is dragged out — [FAQ, excerpt](https://krotos.studio/faq). Pausing between movements keeps your position — [Krotos, vendor](https://krotos.studio/blog/using-ai-powered-sound-effects-presets).

#### Voice as control
- Reformer Pro uses a mic to drive a library — [page](https://krotos.studio/reformer-pro).
- Dehumaniser reacts to input dynamics — [page](https://krotos.studio/dehumaniser-2).
- Firefly follows a performed timing — [Cinevva](https://app.cinevva.com/guides/ai-sound-effect-generators).
- Sketch2Sound reduces control to three time-varying curves: **loudness, brightness and pitch**. Controls may be rough or detailed, and they add to a text prompt — [arXiv](https://arxiv.org/abs/2412.08550v1).
- "Sketching With Your Voice" tunes a vocal-tract model to match a sound's perceptual features — [MIT](https://dspace.mit.edu/handle/1721.1/158128).
- The AES AIMLA 2025 challenge benchmarked retrieving SFX from vocal imitation — [QVIM](https://qvim-aes.github.io).

#### Instant preview and sound-from-sound
- Genopatch shows match candidates as coloured dots that you can audition while they grow — [Sound On Sound](https://www.soundonsound.com/reviews/sonic-charge-synplant-2).
- MetaSounds previews all inputs live — [Epic docs](https://dev.epicgames.com/documentation/unreal-engine/metasounds-the-next-generation-sound-sources-in-unreal-engine?lang=en-US).
- Dumping a 100-byte sound as text or a URL makes a sound shareable — [jsfxr](https://sfxr.me/), [gdfxr](https://github.com/timothyqiu/gdfxr).

### Inferences
Ideas to learn from, as behaviours:
1. **Mutation as a continuous, reversible drag** beats a discrete Mutate button, because magnitude and undo are the same gesture.
2. **Small populations and strong triage** address the fatigue that McDermott describes. The low save rate in Evosynth suggests most breeds are discarded, so discard must be nearly free.
3. **Three curves as a universal vocabulary:** loudness, brightness and pitch could be the common editable form for mouse, pen, voice and animation input alike. Each input type records the same three lanes, which then stay editable. This could work with deterministic signal analysis and no generative model. I did not test or measure this.
4. **Always recording** removes the fear of losing a good take.
5. **Lock plus randomize** gives control over which part of a sound stays while the rest explores.

### Gaps
- I found no controlled study comparing drawing, XY, breeding and sliders for SFX creation speed or satisfaction.
- The reference to Krotos's mouse versus trackpad behaviour is unconfirmed. Sources speak of dragging on XY pads only.
- GameSynth's exact mapping from sketch to model parameters was not available in the sources I read.

## What do users complain about, and what space is unoccupied?

### Takeaway
Complaints cluster around:
- Price and subscription walls.
- Platform limits.
- Weak organisation of results.
- Fiddly deep editing.
- AI output that cannot be steered.
- Results that stop at the file instead of reaching the game.

The most open space is **creating and judging a sound while the actual game moment is playing, with the result staying a live, editable, deterministic recipe**. I did not find a tool doing this, though my search cannot prove none exists.

### Cited Findings

#### Complaints by tool
- **Synplant 2:** presets have no tagging; DNA editing "still fiddly"; Genopatch converts to mono, and accuracy varies — [MusicTech](https://musictech.com/reviews/plug-ins/sonic-charges-synplant-2-review/). It is CPU-heavy, and short sounds work best — [Sound On Sound](https://www.soundonsound.com/reviews/sonic-charge-synplant-2).
- **GameSynth:** "Looks cool AF, but expensive"; a user wishes they could afford it; no Mac version. Also praised: a user calls it cheaper than plugins with similar features, and another says the new modular GUI is much easier than the old one — [Synthtopia comments, 2021](https://synthtopia.com/?p=122160).
- **ChipTone:** users request Linux and the unfinished sampler and sequencer — [page](https://sfbgames.itch.io/chiptone). It needs WebGL acceleration, and some Mac users see a "damaged" warning — [comments, excerpt](https://sfbgames.itch.io/chiptone/comments?after=16).
- **gdfxr:** the editor can freeze on long sounds, because GDScript is slow; `.sfxr` files are not generated at runtime — [GitHub](https://github.com/timothyqiu/gdfxr).
- **Krotos Studio:** subscription-only; base tier locked to presets; Edit mode needs Pro — [FAQ, excerpt](https://krotos.studio/faq). Games need Pro — [page](https://krotos.studio/pricing).
- **Portal:** "$149 is a bit too much" — [Synth Anatomy](https://synthanatomy.com/2019/06/output-portal-granular-effect-plugin.html).
- **DopplerPRO:** some find the interface hard because not everything fits on one screen — [Dumais, AI-written page](https://www.daviddumaisaudio.com/which-tonsturm-whoosh-plugin-should-you-get/). Low confidence.

#### Complaints about generative AI
- Marc Ferras, on Sony AI's own page: sound designers want more and more intuitive control, and professionals will want specific controls rather than simple text-in, audio-out tools, described as a "gimmick" — [Sony AI](https://ai.sony/archive/introducing-woosh-sony-ais-sound-effect-foundation-model).
- ElevenLabs produces one clip per prompt and cannot shape or perform it afterwards. Across tools, transients can feel late, tails need trimming and loops need crossfades — [Cinevva, vendor and competitor](https://app.cinevva.com/guides/ai-sound-effect-generators).
- A forum poster says they won't sell fully AI-generated SFX, calling them "crap" — [gamedev.net](https://gamedev.net/forums/topic/720371-ai-or-not-ai/5476065/).
- A veteran's essay argues the quietest threat is homogenisation — [Edward Ray, excerpt](https://cliquegames.squarespace.com/post/why-i-refuse-to-use-ai-audio-insight-from-industry-veteran-edward-ray).
- Rights vary: ElevenLabs' free tier has no commercial licence, and Woosh's public weights are non-commercial — [Cinevva](https://app.cinevva.com/guides/ai-sound-effect-generators), [Sony AI](https://ai.sony/archive/introducing-woosh-sony-ais-sound-effect-foundation-model). Krotos markets the opposite stance: "No AI-generated slop" — [page](https://krotos.studio/pricing).

#### Integration gap
- One 2026 comparison notes standalone tools do not put the sound into your game — [Summer Engine, vendor](https://www.summerengine.com/blog/ai-sound-effect-generator-for-games). Summer Engine and UnlockSFX address this for generated audio only — [store](https://store.godotengine.org/asset/unlocksfx/unlocksfx/).
- GameSynth comes close through its API, and DSP Motion through generated C# scripts — [page](https://tsugi-studio.com/web/en/products-gamesynth.html), [CG Channel](https://www.cgchannel.com/?p=118178). Both still deliver into an external engine.

### Inferences
These are candidates for "never seen before", ranked by my judgement. Each is unproven and needs a design test.

1. **Paint the sound on the game moment.** Pause or loop a real gameplay moment, then draw or voice the sound while it plays and hear it in the actual mix. DSP Motion syncs to *video*; PH2D could sync to the *live scene*, with the same objects and camera. No external tool has access to that state.
2. **The sound's curves come from the object.** Bind the three control curves to velocity, contact force or animation from the Timeline, Physics or Flip. GameSynth imports curves from Maya and Unity as files; PH2D could read them live. This is the indie-scale version of Osada's wagon.
3. **A recipe, not a file.** A small parametric asset stored in the project, as with gdfxr, but deep and able to use procedural models. It would be editable to ship, with one-click baking when memory or CPU requires it. Recipe plus seed should produce identical bytes, in line with the "same stroke, same byte" owner decision in [project memory](../../project-memory/project_same_stroke_same_byte_on_any_machine_2026_11_04.md). That makes sounds diffable, undoable and shareable as text.
4. **Variation set as the first-class object.** One recipe, with ranges, playback rule (round robin, no-repeat, velocity) and a live memory budget meter. Preview by hammering the trigger, since repetition is exactly what designers fear.
5. **Breeding with a game-aware judge.** A Synplant-like radial exploration where each child is auditioned on the real event, plus a back-to-back reference slot and a "keep or discard" gesture that takes well under a second. Include tagging from the first day, because reviewers fault Synplant's preset system for lacking it.
6. **Voice sketch to an editable procedural recipe.** Extract three curves from the mic and fit a local procedural model. Unlike text-to-audio, the result stays editable, works offline and has clear rights. Unlike Genopatch, it targets gesture and not timbre alone. Risk: fitting quality is unknown. Check the [vocal-imitation search patent](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/9390695) before designing this, as its claims and status were not reviewed.
7. **Pre-flight on save.** Category-aware loudness, clipping, leading silence, tail, loop seam and size checks, with automatic naming from the object and event (UCS-compatible). Nothing cited above does all of this in one step during creation, based on what I read.

Licensing strategy, as a PH2D-specific reading:
- Output rights can be a selling point: ChipTone's CC0 is the clearest example in this research.
- PH2D could consider a plain statement that everything created with the module belongs to the creator.
- The owner must decide this.
- The sfxr family is a candidate for a runnable oracle under CLAUDE.md §0.9, only after artifact-level licence triage. jsfxr displays Unlicense and gdfxr MIT on their pages, while Bfxr names no licence on its page.
- Avoid names such as "Genopatch" and do not ingest any vendor presets or recordings.

### Gaps
- **Forum evidence:** the complaint evidence is thin and skewed to reviews and old comments. r/GameAudio and KVR threads could not be read in depth; the one KVR Synplant thread I opened only held launch-day reactions. A representative view would require manual reading or authenticated access.
- **Unverified quote:** a search summary attributed "might not be a fun experience to use it casually" to a Synthtopia piece on GameSynth. The page I opened (2021 release) did not contain it, so do not cite it.
- **MetaSounds:** I found no systematic source of designer complaints.
- **Survey:** the SMC 2019 survey on synth programming habits ([link](https://www.smc2019.uma.es/articles/S4/S4_03_SMC2019_paper.pdf)) was offline during research. It may be worth retrying.
- **Claim of absence:** "unoccupied" means not found across the tools and sources above. It is not a market census, and unreleased or in-house studio tools are invisible to this search.
