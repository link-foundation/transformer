//! Conformance tests that run every shipped example program end to end and
//! assert on its canonical result (requirement R5). These double as executable
//! documentation: each example in `examples/` is exercised here.

use lino_transformer::{canonical, transform, Options, StopReason};

fn example(name: &str) -> String {
    let path = format!("{}/examples/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {}", path, e))
}

fn run(program_file: &str, input: &str, options: Options) -> (String, StopReason, u64) {
    let program = example(program_file);
    let result = transform(&program, input, options).expect("example runs");
    (canonical(&result.graph), result.stop_reason, result.steps)
}

#[test]
fn swap_reorders_once() {
    let (graph, stop, steps) = run("swap.lino", "(a before b)", Options::bounded());
    assert_eq!(graph, "(b before a)");
    assert_eq!(stop, StopReason::NormalForm);
    assert_eq!(steps, 1);
}

#[test]
fn peano_addition_computes_two_plus_one() {
    // add(2, 1) = 3, encoded in unary as nested successors.
    let (graph, stop, steps) = run(
        "peano-add.lino",
        "(add (s (s z)) (s z))",
        Options::bounded(),
    );
    assert_eq!(graph, "(s (s (s z)))");
    assert_eq!(stop, StopReason::NormalForm);
    assert_eq!(steps, 3);
}

#[test]
fn peano_multiplication_computes_two_times_three() {
    // mul(2, 3) = 6, reusing the addition rules.
    let (graph, stop, _) = run(
        "peano-mul.lino",
        "(mul (s (s z)) (s (s (s z))))",
        Options::bounded(),
    );
    assert_eq!(graph, "(s (s (s (s (s (s z))))))");
    assert_eq!(stop, StopReason::NormalForm);
}

#[test]
fn list_reverse_reverses_three_elements() {
    let (graph, stop, steps) = run(
        "list-reverse.lino",
        "(reverse (c a (c b (c c e))))",
        Options::bounded(),
    );
    assert_eq!(graph, "(c c (c b (c a e)))");
    assert_eq!(stop, StopReason::NormalForm);
    assert_eq!(steps, 5);
}

#[test]
fn busy_beaver_two_state_halts_with_four_ones() {
    // The 2-state busy beaver halts after exactly 6 steps, leaving four 1s on
    // the tape. This is the universality demonstration: a Turing machine
    // simulated purely by structural link rewriting (requirement R1).
    let (graph, stop, steps) = run("turing-bb2.lino", "(tm e A 0 e)", Options::bounded());
    assert_eq!(graph, "(tm (c 1 (c 1 e)) H 1 (c 1 e))");
    // The machine halts by reaching the H state, where no rule applies, so the
    // run ends in a normal form.
    assert_eq!(stop, StopReason::NormalForm);
    assert_eq!(steps, 6);
}

#[test]
fn runaway_machine_stops_at_the_step_limit() {
    // A machine with no halting state never terminates; bounded mode stops it
    // with a typed reason instead of looping forever (requirements R12, R23).
    let options = Options {
        max_steps: Some(5),
        ..Options::default()
    };
    let (_, stop, steps) = run("turing-runaway.lino", "(tm e R 0 e)", options);
    assert_eq!(stop, StopReason::StepLimit);
    assert_eq!(steps, 5);
}
