# 06 Emergence Rules and Feedback Loops

## Objective

Define the minimal rules and feedback loops that can generate **new collective behavior** as group size increases, potentially yielding a superorganism dynamic.

## Core Premise

Emergence is detected when macro-level order parameters become stable and self-reinforcing even though no individual agent intends the macro outcome.

## State Variables for Emergence

1. `N`: effective group size.
2. `M`: subsistence mode (`HunterGatherer`, `Sedentary`, `Agriculture`).
3. `S`: surplus per capita.
4. `K`: network coupling (trade/info/logistics integration).
5. `P`: ecological pressure.
6. `A`: affinity vector (3D cultural identity, per agent).
7. `G`: governance state (policy type + stress channel: price pressure, legitimacy).

## Reinforcing Loops (R)

### R1: Surplus-Centralization Loop

`S ↑ -> storage dependence ↑ -> property lock-in ↑ -> institutional centralization ↑ -> extraction/throughput capacity ↑ -> S ↑`

### R2: Scale-Delegation Loop

`N ↑ -> coordination cost ↑ -> delegated authority ↑ -> hierarchy pressure ↑ -> policy lock-in ↑ -> large-scale persistence ↑`

### R3: Coupling-Synchronization Loop

`K ↑ -> synchronized expectations/incentives ↑ -> system-wide throughput pressure ↑ -> dependence on coordination infrastructure ↑ -> K ↑`

### R4: Oxytocin Bonding Loop (In-Group Reinforcement)

`cooperation ↑ -> affinity convergence ↑ -> in-group perception ↑ -> oxytocin bonding ↑ -> cooperation bias ↑`

Agents who cooperate drift toward each other in affinity space, reinforcing their in-group bond. This creates emergent tribal clusters.

### R5: Othering-Polarization Loop (Out-Group Escalation)

`conflict ↑ -> affinity divergence ↑ -> out-group perception ↑ -> othering bias ↑ -> conflict bias ↑`

Agents who conflict drift apart in affinity space, deepening cultural divides. This can produce persistent inter-group hostility.

### R6: War-Militarization Loop

`stress ↑ -> war probability ↑ -> military success rewards hierarchy + coercion -> extractive governance ↑ -> further stress ↑`

Societies under stress are more likely to initiate wars. War winners gain resources and population, reinforcing the institutional structures (hierarchy, coercion) that enabled military mobilization. At the agent level, raids drive affinity polarization — raiding parties converge internally while diverging from victims, producing self-reinforcing tribal factions.

### R7: Governance Extraction Spiral

`legitimacy ↓ -> extractive policy ↑ -> ecological degradation ↑ -> surplus ↓ -> price pressure ↑ -> legitimacy ↓`

Under stress, governance shifts toward extractive policies which further degrade the environment, creating a self-reinforcing collapse pathway.

### R8: Trust-Defection Spiral (Coordination Failure)

`trust ↓ -> cooperation tendency ↓ -> conflict ↑ -> observed cooperation ↓ -> trust ↓`

Agents with low trust_memory rationally choose conflict even when mutual cooperation yields higher surplus. The resulting defection further erodes trust across neighbors. This is the micro-foundation for the Moloch/multipolar-trap dynamic. The coordination failure index (CFI) measures the aggregate surplus gap between actual outcomes and cooperative optimum.

## Balancing Loops (B)

### B1: Ecological Constraint Loop

`throughput ↑ -> ecological pressure P ↑ -> productivity and stability constraints ↑ -> surplus growth capacity ↓`

This does not remove emergence; it can instead produce brittle, coercive stabilization.

### B2: Stress Channel — Resource → Price → Legitimacy

`resource scarcity ↑ -> price pressure ↑ -> legitimacy erosion ↑ -> policy adaptation -> redistribution ↑ -> stabilization`

Under moderate stress, governance adapts by redistributing surplus (Redistributive policy), partially stabilizing the system. This is a balancing loop until legitimacy drops below 0.35, at which point R6 (extraction spiral) dominates.

### B3: War Exhaustion

`war ↑ -> casualties + ecological damage ↑ -> reduced military capacity ↓ -> war probability ↓`

Wars are costly: both sides suffer population losses, ecological damage, and legitimacy erosion. This creates a natural ceiling on war frequency and prevents runaway militarization.

### B4: Trade-Driven Cultural Integration

`trade ↑ -> mild affinity convergence -> reduced othering -> more trade`

Trade produces slow cultural convergence between trading partners, gradually dissolving out-group boundaries. This counteracts the othering-polarization loop (R5) and promotes economic integration.

## Emergence Order Parameters

Track at each tick:

1. `throughput_pressure`
2. `coordination_centralization`
3. `policy_lock_in`
4. `autonomy_loss`
5. `superorganism_index`

A phase transition is suspected when these indicators rise together and remain high across shocks.

## Transition Hypothesis

1. Hunter-gatherer phase: low lock-in, high local autonomy, weak centralization.
2. Sedentary phase: rising storage dependence, property lock-in, and formal institutions.
3. Agricultural phase: stronger hierarchy, specialization, coercion capacity, and macro coordination.

## Experimental Protocol

1. Sweep `N` while holding other parameters fixed.
2. Sweep `M` transitions under matched `N`.
3. Sweep `K` and `P` to test reinforcing vs balancing dominance.
4. Record bifurcation points where order parameters cross predefined thresholds.
5. Sweep initial affinity diversity to test tribal clustering dynamics.
6. Stress-test governance adaptation under sustained ecological shocks.

## Falsifiability

The framework is wrong if increasing `N` and shifting to `Sedentary/Agriculture` does not produce persistent increases in at least two of:

- centralization,
- policy lock-in,
- autonomy loss,
- superorganism index.

The oxytocin model is wrong if:
- Cooperation does not produce detectable affinity clustering over time.
- Out-group conflict does not increase with affinity distance.
- Affinity-diverse populations do not show higher conflict rates than affinity-homogeneous populations.

The governance model is wrong if:
- Sustained resource scarcity does not erode legitimacy.
- Extractive governance does not worsen ecological outcomes relative to redistributive governance.
