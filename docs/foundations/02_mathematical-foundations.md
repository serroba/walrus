# 02 Mathematical Foundations

## State Space

At time `t`, world state is:

`X_t = {A_t, I_t, R_t, E_t, C_t}`

- `A_t`: agent-level states (needs, wealth, status, beliefs, trust, power).
- `I_t`: institutions/policies (rules, taxation, censorship, redistribution, property).
- `R_t`: resource stocks (fossil, renewable flows, minerals, biomass, ecological integrity).
- `E_t`: energy conversion and transport infrastructure.
- `C_t`: network topology (trade, influence, governance, conflict).

## Dynamics

General transition:

`X_{t+1} = F(X_t, theta, u_t, epsilon_t)`

- `theta`: parameter vector (behavioral, technical, ecological, institutional).
- `u_t`: interventions/policies.
- `epsilon_t`: stochastic shocks.

## Agent Decision Model (generic)

Agent `i` chooses action `a_i,t` maximizing bounded utility:

`U_i = w_n*N_i + w_s*S_i + w_sec*Sec_i - w_risk*Risk_i - w_norm*Penalty_i`

subject to budget, energy availability, and institutional constraints.

## Group Size and Regime Functions

Define:

- `N_t`: effective group size (band, settlement, polity, networked civilization).
- `M_t`: subsistence mode in `{HunterGatherer, Sedentary, Agriculture}`.

Behavioral coefficients are regime-conditioned functions of scale:

`theta_b,t = H(N_t, M_t, phi)`

where `phi` are baseline human traits.

Example mappings:

- coordination cost: `k_coord = alpha_M * log(1 + N_t)`
- hierarchy pressure: `k_hier = beta_M * log(1 + N_t)`
- coercion propensity: `k_coerce = gamma_M * max(0, N_t - N0_M) / N_t`
- norm cohesion: `k_cohesion = delta_M / (1 + eta_M * N_t)`

Expected ordering by mode for equal `N_t`:

- `k_hier(Agriculture) > k_hier(Sedentary) > k_hier(HunterGatherer)`
- `k_coerce(Agriculture) > k_coerce(Sedentary) > k_coerce(HunterGatherer)`

## Biophysical Constraints

- Throughput limited by extraction + regeneration rates.
- Energy return affects usable surplus.
- Ecological damage feeds back to productivity, health, and stability.

Example stock-flow form:

`R_{k,t+1} = R_{k,t} + Regen_k(R_t) - Extract_k(A_t,E_t,I_t)`

## Emergence Metrics (Superorganism)

Define macro “superorganism intensity” index `SO_t` from:

- global throughput growth,
- coordination centralization,
- local autonomy loss,
- policy lock-in,
- resilience fragility tradeoff.

`SO_t = g(throughput_t, centralization_t, lockin_t, fragility_t)`

Add group-size/regime component:

`SO_t = g(..., N_t, M_t, k_hier,t, k_coerce,t)`

## Validation Framing

- Generative validation: does model reproduce stylized facts?
- Structural validation: do mechanisms match empirical literature?
- Predictive validation: does out-of-sample trend direction hold?
