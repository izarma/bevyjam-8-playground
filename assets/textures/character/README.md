# Monk Sprite Sheet

## Sheet Dimensions

| Property      | Value                 |
|---------------|-----------------------|
| File          | `monk-sheet.png`      |
| Total Size    | 416 × 114 px          |
| Grid          | 3 rows × 13 columns   |
| Cell Size     | 32 × 38 px            |
| Frame Padding | 0 px (no padding)     |

## Layout

One row per facing, left-to-right by animation:

| Row | Facing | Idle (4 frames) | Walk (4 frames) | Jump (5 frames) |
|-----|--------|-----------------|-----------------|-----------------|
| 0   | Front  | cols 0–3        | cols 4–7        | cols 8–12       |
| 1   | Back   | cols 0–3        | cols 4–7        | cols 8–12       |
| 2   | Side   | cols 0–3        | cols 4–7        | cols 8–12       |

`Side` is the screen-right

## Frame Index Formula

```text
column = animation_start + frame   (animation_start: Idle=0, Walk=4, Jump=8)
row    = facing                    (Front=0, Back=1, Side=2)

index  = row * 13 + column
```

Cell origin in pixels:

```text
x = column * 32
y = row    * 38
```
