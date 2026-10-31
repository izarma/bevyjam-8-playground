# Character Sprite Sheet

Sprite sheet for the 8-direction player character.

## Sheet Dimensions

| Property        | Value                               |
|-----------------|-------------------------------------|
| File            | `16x32char-sheet-template.png`      |
| Total Size      | 1120 × 224 px                       |
| Grid            | 7 rows × 35 columns                 |
| Cell Size       | 32 × 32 px                          |
| Character Size  | ~16 × 32 px (centered in each cell) |

## Direction Convention

Each animation row contains five directional facings, ordered left to right:

| Order | Facing      | Notes                              |
|-------|-------------|------------------------------------|
| 1     | Down        | Facing toward camera               |
| 2     | Down_Side   | Facing down-right; flip X for left |
| 3     | Side        | Facing right; flip X for left      |
| 4     | Up_Side     | Facing up-right; flip X for left   |
| 5     | Up          | Facing away from camera            |

## Animation Layout

| Row | Animation Group | Facing     | Frames | Column Range | Pixel Range (x) |
|-----|-----------------|------------|--------|--------------|-----------------|
| 0 | Rotate | — | 8 | 0–7 | 0–255 |
| 1 | Idle | Down | 4 | 0–3 | 0–127 |
| 1 | Idle | Down_Side | 4 | 4–7 | 128–255 |
| 1 | Idle | Side | 4 | 8–11 | 256–383 |
| 1 | Idle | Up_Side | 4 | 12–15 | 384–511 |
| 1 | Idle | Up | 4 | 16–19 | 512–639 |
| 2 | Walk | Down | 4 | 0–3 | 0–127 |
| 2 | Walk | Down_Side | 4 | 4–7 | 128–255 |
| 2 | Walk | Side | 4 | 8–11 | 256–383 |
| 2 | Walk | Up_Side | 4 | 12–15 | 384–511 |
| 2 | Walk | Up | 4 | 16–19 | 512–639 |
| 3 | Run | Down | 6 | 0–5 | 0–191 |
| 3 | Run | Down_Side | 6 | 6–11 | 192–383 |
| 3 | Run | Side | 6 | 12–17 | 384–575 |
| 3 | Run | Up_Side | 6 | 18–23 | 576–767 |
| 3 | Run | Up | 6 | 24–29 | 768–959 |
| 4 | Interact | Down | 4 | 0–3 | 0–127 |
| 4 | Interact | Down_Side | 4 | 4–7 | 128–255 |
| 4 | Interact | Side | 4 | 8–11 | 256–383 |
| 4 | Interact | Up_Side | 4 | 12–15 | 384–511 |
| 4 | Interact | Up | 4 | 16–19 | 512–639 |
| 5 | Jump | Down | 5 | 0–4 | 0–159 |
| 5 | Jump | Down_Side | 5 | 5–9 | 160–319 |
| 5 | Jump | Side | 5 | 10–14 | 320–479 |
| 5 | Jump | Up_Side | 5 | 15–19 | 480–639 |
| 5 | Jump | Up | 5 | 20–24 | 640–799 |
| 6 | Punch | Down | 7 | 0–6 | 0–223 |
| 6 | Punch | Down_Side | 7 | 7–13 | 224–447 |
| 6 | Punch | Side | 7 | 14–20 | 448–671 |
| 6 | Punch | Up_Side | 7 | 21–27 | 672–895 |
| 6 | Punch | Up | 7 | 28–34 | 896–1119 |

## Frame Index Formula

When using a Bevy `TextureAtlasLayout::from_grid(UVec2::new(32, 32), 35, 7, None, None)`,
the atlas index for any frame is:

```
index = row * 35 + start_column + frame
```

Where `row` & `frame` are 0-based
