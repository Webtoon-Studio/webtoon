/// The dual of `assert!`.
///
/// Documents that `cond` can be true or false here, by design. Use it where a
/// condition looks like it should be an `assert!`, but actually can go either
/// way; `maybe!` plugs that hole, signaling the case was considered rather than
/// missed.
///
/// Expands to `debug_assert!(cond || !cond)`, so logically always holds.
///
/// Mirrors `TigerBeetle`'s `maybe()` from
/// ["It Takes Two to Contract"](https://tigerbeetle.com/blog/2023-12-27-it-takes-two-to-contract/).
macro_rules! maybe {
    ($cond:expr $(, $($arg:tt)+)?) => {
        debug_assert!($cond || !$cond $(, $($arg)+)?)
    };
}

pub(crate) use maybe;

/// Asserts the implication if `a` then `b`.
///
/// If `a` holds, `b` must also hold. Doesn't check anything when `a` is false.
///
/// Expands to `if a { debug_assert!(b) }`, a more readable form, rather than
/// the harder-to-parse `!a || b`. Takes an optional message like
/// `debug_assert!`.
///
/// See `TigerBeetle`'s ["Asserting Implications"](https://tigerbeetle.com/blog/2025-05-26-asserting-implications/).
#[macro_export]
macro_rules! implies {
    ($a:expr => $b:expr $(, $($arg:tt)+)?) => {
        if $a { debug_assert!($b $(, $($arg)+)?) }
    };
}

#[expect(unused)]
pub(crate) use implies;

#[cfg(test)]
mod tests {
    #[test]
    fn maybe_accepts_true_and_false() {
        maybe!(1 + 1 == 2);
        maybe!(1 + 1 == 3);
    }

    #[test]
    fn maybe_accepts_optional_message() {
        maybe!(true, "a creator can have zero webtoons");
        maybe!(false, "count was {}", 0);
    }

    #[test]
    fn maybe_still_evaluates_condition() {
        let mut calls = 0;
        maybe!({
            calls += 1;
            true
        });
        assert_eq!(calls, 1, "maybe! must still evaluate its condition");
    }

    #[test]
    fn implies_holds_when_antecedent_true_and_consequent_true() {
        implies!(true => true);
    }

    #[test]
    fn implies_skips_check_when_antecedent_false() {
        // Consequent is false, but antecedent is false too, so this must not panic.
        implies!(false => false);
    }

    #[test]
    #[should_panic(expected = "assertion failed: false")]
    fn implies_panics_when_antecedent_true_and_consequent_false() {
        implies!(true => false);
    }

    #[test]
    fn implies_accepts_optional_message() {
        implies!(true => true, "explanation {}", 1);
    }
}
