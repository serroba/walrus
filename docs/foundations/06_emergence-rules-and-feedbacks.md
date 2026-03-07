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

## Reinforcing Loops (R)

### R1: Surplus-Centralization Loop

`S ↑ -> storage dependence ↑ -> property lock-in ↑ -> institutional centralization ↑ -> extraction/throughput capacity ↑ -> S ↑`

### R2: Scale-Delegation Loop

`N ↑ -> coordination cost ↑ -> delegated authority ↑ -> hierarchy pressure ↑ -> policy lock-in ↑ -> large-scale persistence ↑`

### R3: Coupling-Synchronization Loop

`K ↑ -> synchronized expectations/incentives ↑ -> system-wide throughput pressure ↑ -> dependence on coordination infrastructure ↑ -> K ↑`

## Balancing Loop (B)

### B1: Ecological Constraint Loop

`throughput ↑ -> ecological pressure P ↑ -> productivity and stability constraints ↑ -> surplus growth capacity ↓`

This does not remove emergence; it can instead produce brittle, coercive stabilization.

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

## Falsifiability

The framework is wrong if increasing `N` and shifting to `Sedentary/Agriculture` does not produce persistent increases in at least two of:

- centralization,
- policy lock-in,
- autonomy loss,
- superorganism index.
