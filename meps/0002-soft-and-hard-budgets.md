# MEP 0002: Soft and hard limits, and a budget for the whole run

- **Status:** Accepted
- **Issue:** none (proposed with the reference implementation)
- **Revision:** draft

## Problem

A budget in the first draft is one number per agent: `budget.tokens: 2000000`. Two things a person
running many agents needs are missing:

1. **A limit for the whole run.** "This system may use 30% of my plan's 5-hour window" is a
   property of the run, not of any one agent, and per-agent restrictions combine by tightening, so
   they cannot express one shared amount for everyone.
2. **A warning before the stop.** Stopping an agent mid-task loses work. Runtimes want a first
   threshold at which nothing new starts and agents wrap up with a summary, and a second at which
   they are stopped at once.

## Proposal

**Limits.** Wherever a limit takes a number or a duration, it may also take `{ soft, hard }`:

```yaml
tokens: 2000000                          # the hard value, as before
tokens: { soft: 1500000, hard: 2000000 } # wrap up at the soft value, stop at the hard one
wall_clock: { soft: 45m, hard: 1h }
```

`soft` is optional; without one a runtime may choose one (for example 85% of `hard`). At the soft
value a runtime SHOULD start no new work for the limited agents and ask them to finish with a
summary. At the hard value it MUST stop them and what they started. `soft` above `hard` is an error
(`budget-soft-above-hard`).

**Per-agent budgets** (`restrictions.budget`) gain:

| Member | Meaning |
|---|---|
| `turns` | Turns (a model's reply to a message, with its tool calls) in one run |
| `working_time` | Time spent in turns, as a duration |

`tokens`, `wall_clock`, `turns` and `working_time` take limits. Down the settings chain each keeps
the smaller hard value, and the smaller soft value from whichever block set one; a larger hard value
is `restriction-loosened`, as before.

**A run budget** on `system.budget` covers every agent and everything they spawn together:

```yaml
system:
  id: c172
  name: C172 model
  budget:
    plan:
      - { provider: claude, window: five_hour, soft: 25, hard: 30 }
    tokens: { hard: 50000000 }
    wall_clock: { soft: 45m, hard: 1h }
    max_working: 6
```

| Member | Meaning |
|---|---|
| `plan` | Shares of a provider's plan window, in percent above 0 and at most 100 (`budget-percent-out-of-range`). `provider` and `window` are the provider's names, such as `claude` and `five_hour`. A runtime measures the run's share as the change in the window since the run started, so use outside the run counts too. |
| `tokens`, `cost_usd`, `wall_clock` | Limits for the run |
| `max_working` | Most agents working at the same time; the rest wait for a free slot. At least 1 (`budget-no-room`). |

The run budget does not combine with per-agent budgets: both apply, and whichever is reached first
acts.

## Compatibility

Every document valid in the first draft stays valid and means the same: a bare number is the hard
value, and a document without `soft` gets the runtime's choice where it previously had none.

## Conformance

- `valid/run-budget.yaml`
- `invalid/budget-soft-above-hard.yaml`, `invalid/budget-percent-out-of-range.yaml`,
  `invalid/budget-no-room.yaml`, `invalid/loosen-turns.yaml`
- `resolve/limits.yaml` (soft and hard values combining down the chain)

## Alternatives considered

- **A run budget under `defaults.restrictions` with a new scope.** Defaults are per-agent settings
  that tighten down the chain; a system-wide amount there would collide with every agent's own
  budget and its scope rules.
- **Soft values as a percentage of hard only.** Simpler, but a person often thinks in absolute
  terms ("wrap up at 25%, stop at 30%").
