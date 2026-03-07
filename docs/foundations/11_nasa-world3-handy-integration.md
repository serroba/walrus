# 11 NASA / World3 / HANDY Integration Notes

## Why integrate these models

- **World3** and **HANDY** provide macro-level sanity checks on long-run dynamics.
- Our actor model provides micro-level mechanism and heterogeneity.
- Combining both improves realism and falsifiability.

## Practical role in this project

1. **Calibration priors**
- Use World3/HANDY-style trend shapes as priors for plausible system trajectories.
- Avoid overfitting exact historical curves in early phases.

2. **Benchmark dimensions**
- population,
- resource depletion,
- per-capita output/surplus proxy,
- inequality/class split proxy,
- ecological stress.

3. **Validation mode**
- compare direction, turning windows, and collapse timing classes.
- compare robustness under parameter perturbations.

## Current status

- `calibration` module supports OWID/Maddison-compatible CSV ingestion and stylized calibration.
- `calibration` module now also supports HANDY-like CSV ingestion (`ingest_handy_csv`).
- confidence states: `exploratory`, `calibrated-stylized`, `calibrated-curve-fit`.
- ensemble validation reports uncertainty bands and robustness.

## Next integration step

1. Add adapter(s) for HANDY-like exports:
- commons/resources,
- elite/commoner or hierarchy proxy,
- inequality proxy.

2. Add side-by-side diagnostics:
- actor model vs HANDY baseline,
- actor model vs World3 trend envelope.

3. Add model discrepancy report:
- where actor microfoundations diverge from macro references,
- whether divergence is due to assumptions or implementation.

## Interpretation guideline

If actor simulation and macro references disagree, treat this as a learning signal:
- either micro rules are missing a key loop,
- or macro assumptions hide heterogeneity that matters.
