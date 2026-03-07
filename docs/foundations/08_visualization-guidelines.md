# 08 Visualization Guidelines

## Goal

Make simulation outputs understandable for non-specialists while preserving model rigor.

## Audience-First Principles

1. Start with behavior labels, not equations.
2. Show trends over time before showing parameter details.
3. Use plain language for interpretation and caveats.
4. Keep one chart = one question.

## Recommended Visual Set (MVP)

1. Trajectory chart:
   - `superorganism_index` and `mean_local_complexity` over time.
2. Mode composition chart:
   - counts of hunter-gatherer, sedentary, agriculture societies over time.
3. Scenario comparison table:
   - start/peak/end values with behavior class labels.

## Behavior Labels

Use consistent labels from model classification:

- Stabilizing complexity
- Overshoot and correction
- Fragile transition
- Stagnant low complexity

## Interpretation Rules

1. If peak is high but endpoint is much lower, call out correction/collapse risk.
2. If endpoint remains high with moderate correction, call out stability.
3. If complexity stays low across run, call out fragmentation.
4. Always mention that outputs are model-dependent and parameter-sensitive.
