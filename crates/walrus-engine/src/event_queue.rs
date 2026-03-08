//! Discrete-event simulation core.
//!
//! Replaces the tick-based batch loop with a priority queue of stochastically
//! scheduled events.  Each event executes against the *current* world state
//! and applies its effects immediately, producing emergent non-deterministic
//! ordering even across runs with identical parameters.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

// ---------------------------------------------------------------------------
// Time
// ---------------------------------------------------------------------------

/// Simulation time as a continuous f64.  There is no global tick — agents act
/// on their own stochastic timelines.
pub type SimTime = f64;

// ---------------------------------------------------------------------------
// Event kinds
// ---------------------------------------------------------------------------

/// Action an individual agent can perform.
#[derive(Clone, Debug, PartialEq)]
pub enum AgentAction {
    /// Harvest energy from the local landscape cell.
    Forage,
    /// Initiate interaction (cooperate / conflict / trade) with a neighbor.
    Interact,
    /// Drift position toward kin centroid + random walk.
    Move,
    /// Attempt courtship and reproduction.
    Reproduce,
    /// Biological aging step.
    Age,
    /// Gain innovation / skill improvement.
    Learn,
    /// Adopt cultural traits from a neighbor (horizontal or oblique).
    Transmit,
}

/// Action performed at the kin-group level.
#[derive(Clone, Debug, PartialEq)]
pub enum GroupAction {
    /// Launch a raid against a target kin group.
    Raid { target_group: u32 },
    /// Evaluate members for migration to other groups.
    Migrate,
}

/// Global world-level events.
#[derive(Clone, Debug, PartialEq)]
pub enum WorldAction {
    /// Rebuild the spatial index so neighbor queries reflect current positions.
    RebuildSpatialIndex,
    /// Snapshot emergent state for observation / output.
    MeasureState,
    /// Deplete / regenerate energy landscape stocks.
    UpdateLandscape,
}

/// A simulation event — agent, group, or world level.
#[derive(Clone, Debug)]
pub enum EventKind {
    Agent { id: u64, action: AgentAction },
    Group { kin_group: u32, action: GroupAction },
    World { action: WorldAction },
}

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

/// A scheduled event with a continuous timestamp.
#[derive(Clone, Debug)]
pub struct Event {
    /// When this event fires (simulation time units).
    pub time: SimTime,
    /// What happens.
    pub kind: EventKind,
}

// BinaryHeap is a max-heap; we want earliest-first, so reverse the ordering.
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse: smaller time = higher priority.
        other
            .time
            .partial_cmp(&self.time)
            .unwrap_or(Ordering::Equal)
    }
}

// ---------------------------------------------------------------------------
// Event queue
// ---------------------------------------------------------------------------

/// Priority queue of simulation events, ordered by time (earliest first).
pub struct EventQueue {
    heap: BinaryHeap<Event>,
}

impl EventQueue {
    /// Create an empty queue.
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    /// Create a queue with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity),
        }
    }

    /// Schedule an event.
    pub fn push(&mut self, event: Event) {
        self.heap.push(event);
    }

    /// Pop the earliest event, or `None` if the queue is empty.
    pub fn pop(&mut self) -> Option<Event> {
        self.heap.pop()
    }

    /// Peek at the earliest event without removing it.
    pub fn peek(&self) -> Option<&Event> {
        self.heap.peek()
    }

    /// Number of pending events.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Stochastic scheduling
// ---------------------------------------------------------------------------

/// Simple xorshift64 PRNG — same algorithm used elsewhere in the codebase.
/// Returns a value in [0, 1).
pub fn rand_f64(state: &mut u64) -> f64 {
    let mut s = *state;
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    *state = s;
    (s as f64) / (u64::MAX as f64)
}

/// Sample from an exponential distribution with the given `rate` (events per
/// unit time).  Returns the delay until the next event.
///
/// Uses inverse-CDF: delay = -ln(U) / rate, where U ~ Uniform(0,1).
pub fn exponential_delay(rate: f64, rng: &mut u64) -> f64 {
    debug_assert!(rate > 0.0, "rate must be positive");
    let u = rand_f64(rng);
    // Clamp away from 0 to avoid ln(0) = -inf.
    let u = u.max(1e-15);
    -u.ln() / rate
}

/// Schedule the next occurrence of `action` for `agent_id`, starting from
/// `now`, with the given base `rate` (events per sim-time unit).
///
/// Returns an `Event` whose time is `now + exponential_delay(rate)`.
pub fn schedule_agent(
    now: SimTime,
    agent_id: u64,
    action: AgentAction,
    rate: f64,
    rng: &mut u64,
) -> Event {
    let delay = exponential_delay(rate, rng);
    Event {
        time: now + delay,
        kind: EventKind::Agent {
            id: agent_id,
            action,
        },
    }
}

/// Schedule a group-level event.
pub fn schedule_group(
    now: SimTime,
    kin_group: u32,
    action: GroupAction,
    rate: f64,
    rng: &mut u64,
) -> Event {
    let delay = exponential_delay(rate, rng);
    Event {
        time: now + delay,
        kind: EventKind::Group { kin_group, action },
    }
}

/// Schedule a world event at a fixed interval from `now`.
pub fn schedule_world(now: SimTime, action: WorldAction, interval: f64) -> Event {
    Event {
        time: now + interval,
        kind: EventKind::World { action },
    }
}

// ---------------------------------------------------------------------------
// Event loop
// ---------------------------------------------------------------------------

/// Result of dispatching a single event, returned by the caller's handler.
pub struct DispatchResult {
    /// New events to schedule as follow-ups.
    pub follow_ups: Vec<Event>,
}

/// Run the event loop until `end_time`, calling `handler` for each event.
///
/// The handler receives the current event and returns a `DispatchResult`
/// containing any follow-up events to schedule.  The loop automatically
/// enqueues them.
///
/// Returns the number of events processed.
///
/// Note: `simulate_event_driven` uses its own inline loop instead of this
/// function because it needs direct access to the queue for newborn agent
/// scheduling, dead-agent skipping, and population threshold checks.  This
/// helper is provided as a simpler building block for tests and custom loops.
pub fn run_event_loop<F>(queue: &mut EventQueue, end_time: SimTime, mut handler: F) -> u64
where
    F: FnMut(&Event) -> DispatchResult,
{
    let mut processed: u64 = 0;
    while let Some(event) = queue.pop() {
        if event.time > end_time {
            // Put it back — it's past our horizon.
            queue.push(event);
            break;
        }
        let result = handler(&event);
        for follow_up in result.follow_ups {
            queue.push(follow_up);
        }
        processed += 1;
    }
    processed
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_dispatch_in_time_order() {
        let mut q = EventQueue::new();
        q.push(Event {
            time: 3.0,
            kind: EventKind::World {
                action: WorldAction::MeasureState,
            },
        });
        q.push(Event {
            time: 1.0,
            kind: EventKind::World {
                action: WorldAction::RebuildSpatialIndex,
            },
        });
        q.push(Event {
            time: 2.0,
            kind: EventKind::World {
                action: WorldAction::UpdateLandscape,
            },
        });

        let e1 = q.pop().unwrap();
        let e2 = q.pop().unwrap();
        let e3 = q.pop().unwrap();
        assert!(e1.time <= e2.time);
        assert!(e2.time <= e3.time);
        assert!((e1.time - 1.0).abs() < 1e-9);
        assert!((e2.time - 2.0).abs() < 1e-9);
        assert!((e3.time - 3.0).abs() < 1e-9);
    }

    #[test]
    fn empty_queue_returns_none() {
        let mut q = EventQueue::new();
        assert!(q.pop().is_none());
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn scheduling_with_rate_produces_positive_delays() {
        let mut rng: u64 = 12345;
        for _ in 0..100 {
            let delay = exponential_delay(1.0, &mut rng);
            assert!(delay > 0.0, "delay should be positive, got {delay}");
            assert!(delay.is_finite(), "delay should be finite");
        }
    }

    #[test]
    fn exponential_mean_approximates_inverse_rate() {
        let mut rng: u64 = 98765;
        let rate = 2.0;
        let n = 10_000;
        let sum: f64 = (0..n).map(|_| exponential_delay(rate, &mut rng)).sum();
        let mean = sum / n as f64;
        // Expected mean = 1/rate = 0.5.  Allow 10% tolerance.
        assert!(
            (mean - 0.5).abs() < 0.1,
            "mean delay should be ~0.5, got {mean}"
        );
    }

    #[test]
    fn schedule_agent_creates_future_event() {
        let mut rng: u64 = 42;
        let now = 10.0;
        let event = schedule_agent(now, 7, AgentAction::Forage, 1.0, &mut rng);
        assert!(event.time > now);
        match &event.kind {
            EventKind::Agent { id, action } => {
                assert_eq!(*id, 7);
                assert_eq!(*action, AgentAction::Forage);
            }
            _ => panic!("expected agent event"),
        }
    }

    #[test]
    fn schedule_group_creates_future_event() {
        let mut rng: u64 = 42;
        let now = 5.0;
        let event = schedule_group(now, 3, GroupAction::Raid { target_group: 1 }, 0.5, &mut rng);
        assert!(event.time > now);
        match &event.kind {
            EventKind::Group { kin_group, action } => {
                assert_eq!(*kin_group, 3);
                assert_eq!(*action, GroupAction::Raid { target_group: 1 });
            }
            _ => panic!("expected group event"),
        }
    }

    #[test]
    fn schedule_world_uses_fixed_interval() {
        let event = schedule_world(2.0, WorldAction::MeasureState, 1.5);
        assert!((event.time - 3.5).abs() < 1e-9);
    }

    #[test]
    fn run_event_loop_respects_end_time() {
        let mut q = EventQueue::new();
        q.push(Event {
            time: 1.0,
            kind: EventKind::World {
                action: WorldAction::MeasureState,
            },
        });
        q.push(Event {
            time: 5.0,
            kind: EventKind::World {
                action: WorldAction::MeasureState,
            },
        });
        q.push(Event {
            time: 10.0,
            kind: EventKind::World {
                action: WorldAction::MeasureState,
            },
        });

        let processed = run_event_loop(&mut q, 7.0, |_| DispatchResult { follow_ups: vec![] });
        assert_eq!(processed, 2);
        // The t=10 event should remain in the queue.
        assert_eq!(q.len(), 1);
        assert!((q.peek().unwrap().time - 10.0).abs() < 1e-9);
    }

    #[test]
    fn run_event_loop_enqueues_follow_ups() {
        let mut q = EventQueue::new();
        q.push(Event {
            time: 1.0,
            kind: EventKind::World {
                action: WorldAction::MeasureState,
            },
        });

        let mut call_count = 0u32;
        let processed = run_event_loop(&mut q, 5.0, |event| {
            call_count += 1;
            if event.time < 3.0 {
                // Schedule a follow-up.
                DispatchResult {
                    follow_ups: vec![Event {
                        time: event.time + 1.0,
                        kind: EventKind::World {
                            action: WorldAction::MeasureState,
                        },
                    }],
                }
            } else {
                DispatchResult { follow_ups: vec![] }
            }
        });
        // t=1 → follow-up at t=2, t=2 → follow-up at t=3, t=3 → no follow-up.
        assert_eq!(processed, 3);
        assert_eq!(call_count, 3);
        assert!(q.is_empty());
    }

    #[test]
    fn dead_agent_events_can_be_skipped_by_handler() {
        let mut q = EventQueue::new();
        let dead_id = 99u64;
        q.push(Event {
            time: 1.0,
            kind: EventKind::Agent {
                id: dead_id,
                action: AgentAction::Forage,
            },
        });
        q.push(Event {
            time: 2.0,
            kind: EventKind::Agent {
                id: 1,
                action: AgentAction::Forage,
            },
        });

        let dead_set = std::collections::HashSet::from([dead_id]);
        let mut live_processed = 0u32;

        run_event_loop(&mut q, 10.0, |event| {
            if let EventKind::Agent { id, .. } = &event.kind {
                if !dead_set.contains(id) {
                    live_processed += 1;
                }
                // Dead agent event is consumed but produces no effects.
            }
            DispatchResult { follow_ups: vec![] }
        });

        assert_eq!(live_processed, 1);
    }

    #[test]
    fn high_rate_produces_short_delays() {
        let mut rng: u64 = 55555;
        let n = 1000;
        let high_rate_mean: f64 = (0..n)
            .map(|_| exponential_delay(100.0, &mut rng))
            .sum::<f64>()
            / n as f64;
        let low_rate_mean: f64 = (0..n)
            .map(|_| exponential_delay(0.1, &mut rng))
            .sum::<f64>()
            / n as f64;
        assert!(
            high_rate_mean < low_rate_mean,
            "high rate should produce shorter delays: high={high_rate_mean} low={low_rate_mean}"
        );
    }

    #[test]
    fn queue_handles_many_events() {
        let mut q = EventQueue::with_capacity(10_000);
        let mut rng: u64 = 1234;
        for _ in 0..10_000 {
            q.push(schedule_agent(0.0, 0, AgentAction::Move, 1.0, &mut rng));
        }
        assert_eq!(q.len(), 10_000);

        let mut prev_time = 0.0;
        while let Some(e) = q.pop() {
            assert!(
                e.time >= prev_time,
                "events should be in order: {prev_time} > {}",
                e.time
            );
            prev_time = e.time;
        }
    }

    #[test]
    fn rand_f64_produces_values_in_unit_range() {
        let mut rng: u64 = 77777;
        for _ in 0..1000 {
            let v = rand_f64(&mut rng);
            assert!((0.0..1.0).contains(&v), "value out of range: {v}");
        }
    }
}
