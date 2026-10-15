//! Restrictions: combined so they only get tighter. A block that tries to loosen one is
//! reported, and the tighter value stays.
use super::{Ctx, paths};
use crate::duration;
use crate::model::*;

pub fn apply(acc: &mut Restrictions, new: &Restrictions, ctx: &mut Ctx) {
    if let Some(t) = &new.tools {
        tools(acc.tools.get_or_insert_with(Default::default), t, ctx);
    }
    if let Some(f) = &new.files {
        let into = acc.files.get_or_insert_with(Default::default);
        path_set(&mut into.read, &f.read, "restrictions.files.read", ctx);
        path_set(&mut into.write, &f.write, "restrictions.files.write", ctx);
    }
    if let Some(n) = &new.network {
        egress(&mut acc.network, &n.egress, ctx);
    }
    if let Some(b) = &new.budget {
        budget(acc.budget.get_or_insert_with(Default::default), b, ctx);
    }
    limits(acc, new, ctx);
}

/// Spawning, approvals and communication.
fn limits(acc: &mut Restrictions, new: &Restrictions, ctx: &mut Ctx) {
    if let Some(s) = &new.spawn {
        spawn(acc.spawn.get_or_insert_with(Default::default), s, ctx);
    }
    if let Some(a) = &new.approvals {
        let into = acc.approvals.get_or_insert_with(Default::default);
        union(&mut into.required_for, &a.required_for, "restrictions.approvals.required_for", ctx);
    }
    if let Some(c) = &new.communication {
        communication(acc.communication.get_or_insert_with(Default::default), c, ctx);
    }
}

fn tools(acc: &mut ToolRule, new: &ToolRule, ctx: &mut Ctx) {
    if let Some(allow) = &new.allow {
        let field = "restrictions.tools.allow";
        for t in allow {
            let outside = acc.allow.as_ref().is_some_and(|a| !a.contains(t));
            if outside || acc.deny.contains(t) {
                ctx.loosened(field, &format!("allows `{t}`, which an outer block does not allow"));
            }
        }
        let kept: Vec<String> = match &acc.allow {
            Some(a) => allow.iter().filter(|t| a.contains(t)).cloned().collect(),
            None => allow.clone(),
        };
        acc.allow = Some(kept);
        ctx.source(field);
    }
    union(&mut acc.deny, &new.deny, "restrictions.tools.deny", ctx);
}

fn union(acc: &mut Vec<String>, new: &[String], field: &str, ctx: &mut Ctx) {
    let before = acc.len();
    for item in new {
        if !acc.contains(item) {
            acc.push(item.clone());
        }
    }
    if acc.len() > before {
        ctx.source(field);
    }
}

fn path_set(acc: &mut Option<Vec<String>>, new: &Option<Vec<String>>, field: &str, ctx: &mut Ctx) {
    let Some(new) = new else { return };
    let Some(outer) = acc.clone() else {
        *acc = Some(new.clone());
        ctx.source(field);
        return;
    };
    for p in new {
        if !outer.iter().any(|q| paths::within(p, q)) {
            ctx.loosened(field, &format!("allows `{p}`, which is outside {outer:?}"));
        }
    }
    *acc = Some(paths::intersect(&outer, new));
    ctx.source(field);
}

fn rank(e: &Egress) -> u8 {
    match e {
        Egress::Rule(EgressRule::Deny) => 0,
        Egress::Hosts(_) => 1,
        Egress::Rule(EgressRule::Allow) => 2,
    }
}

fn egress(acc: &mut Option<Network>, new: &Egress, ctx: &mut Ctx) {
    let field = "restrictions.network.egress";
    let Some(current) = acc.as_ref().map(|n| n.egress.clone()) else {
        *acc = Some(Network { egress: new.clone() });
        ctx.source(field);
        return;
    };
    let combined = match (&current, new) {
        (Egress::Hosts(a), Egress::Hosts(b)) => {
            for h in b.iter().filter(|h| !a.contains(h)) {
                ctx.loosened(field, &format!("allows host `{h}`, which an outer block does not"));
            }
            Egress::Hosts(b.iter().filter(|h| a.contains(h)).cloned().collect())
        }
        _ if rank(new) > rank(&current) => {
            ctx.loosened(field, "allows more network access than an outer block");
            current
        }
        _ => new.clone(),
    };
    *acc = Some(Network { egress: combined });
    ctx.source(field);
}

fn budget(acc: &mut Budget, new: &Budget, ctx: &mut Ctx) {
    let f = "restrictions.budget";
    min_limit(&mut acc.tokens, &new.tokens, &format!("{f}.tokens"), ctx, |v| *v as f64);
    min(&mut acc.cost_usd, &new.cost_usd, &format!("{f}.cost_usd"), ctx, |v| *v);
    min_limit(&mut acc.turns, &new.turns, &format!("{f}.turns"), ctx, |v| *v as f64);
    let secs = |v: &String| duration::seconds(v).map_or(f64::INFINITY, |s| s as f64);
    min_limit(&mut acc.wall_clock, &new.wall_clock, &format!("{f}.wall_clock"), ctx, secs);
    min_limit(&mut acc.working_time, &new.working_time, &format!("{f}.working_time"), ctx, secs);
    let Some(scope) = new.scope else { return };
    if acc.scope == Some(BudgetScope::Shared) && scope == BudgetScope::Own {
        let why = "gives each agent its own budget, but an outer block shares one";
        ctx.loosened("restrictions.budget.scope", why);
    } else {
        acc.scope = Some(scope);
        ctx.source("restrictions.budget.scope");
    }
}

/// Like `min` on the hard values; the soft value kept is the smaller of the two blocks' soft values.
fn min_limit<T: Clone>(
    acc: &mut Option<Limit<T>>,
    new: &Option<Limit<T>>,
    field: &str,
    ctx: &mut Ctx,
    size: impl Fn(&T) -> f64,
) {
    let Some(n) = new else { return };
    if acc.as_ref().is_some_and(|a| size(n.hard()) > size(a.hard())) {
        return ctx.loosened(field, "is larger than an outer block's limit");
    }
    let soft = match (acc.as_ref().and_then(Limit::soft), n.soft()) {
        (Some(a), Some(b)) => Some(if size(a) < size(b) { a.clone() } else { b.clone() }),
        (a, b) => b.or(a).cloned(),
    };
    *acc = Some(match soft {
        Some(s) => Limit::SoftHard(SoftHard { soft: Some(s), hard: n.hard().clone() }),
        None => Limit::Hard(n.hard().clone()),
    });
    ctx.source(field);
}

/// Keep the smaller; report a larger value as loosening.
fn min<T: Clone>(acc: &mut Option<T>, new: &Option<T>, field: &str, ctx: &mut Ctx, size: impl Fn(&T) -> f64) {
    let Some(n) = new else { return };
    match acc {
        Some(a) if size(n) > size(a) => ctx.loosened(field, "is larger than an outer block's limit"),
        _ => {
            *acc = Some(n.clone());
            ctx.source(field);
        }
    }
}

fn spawn(acc: &mut Spawn, new: &Spawn, ctx: &mut Ctx) {
    if let Some(allowed) = new.allowed {
        let field = "restrictions.spawn.allowed";
        if acc.allowed == Some(false) && allowed {
            ctx.loosened(field, "allows spawning, but an outer block does not");
        } else {
            acc.allowed = Some(allowed);
            ctx.source(field);
        }
    }
    min(&mut acc.max_children, &new.max_children, "restrictions.spawn.max_children", ctx, |v| *v as f64);
    min(&mut acc.max_depth, &new.max_depth, "restrictions.spawn.max_depth", ctx, |v| *v as f64);
}

fn communication(acc: &mut Communication, new: &Communication, ctx: &mut Ctx) {
    if let Some(mode) = new.mode {
        let field = "restrictions.communication.mode";
        if acc.mode == Some(CommunicationMode::Closed) && mode == CommunicationMode::Open {
            ctx.loosened(field, "opens communication that an outer block closed");
        } else {
            acc.mode = Some(mode);
            ctx.source(field);
        }
    }
    let before = acc.deny.len();
    for rule in &new.deny {
        if !acc.deny.contains(rule) {
            acc.deny.push(rule.clone());
        }
    }
    if acc.deny.len() > before {
        ctx.source("restrictions.communication.deny");
    }
}
