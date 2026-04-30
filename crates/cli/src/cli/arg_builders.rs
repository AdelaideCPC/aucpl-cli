//! Shared clap argument builders for problem and competition args.

use clap::{Arg, ArgAction};

pub(crate) const PROBLEM_VALUE_NAME: &str = "PROBLEM";
pub(crate) const COMPETITION_VALUE_NAME: &str = "COMP";

const PROBLEM_HELP: &str = "Problem name (this is not the problem title)";
const COMPETITION_HELP: &str = "Competition name";

fn set_arg_metadata(arg: Arg, help: &'static str, value_name: &'static str) -> Arg {
    arg.help(help).value_name(value_name).action(ArgAction::Set)
}

pub(crate) fn configure_problem_arg(arg: Arg) -> Arg {
    set_arg_metadata(arg, PROBLEM_HELP, PROBLEM_VALUE_NAME)
}

pub(crate) fn configure_competition_arg(arg: Arg) -> Arg {
    set_arg_metadata(arg, COMPETITION_HELP, COMPETITION_VALUE_NAME)
}

pub(crate) fn problem_arg_optional() -> Arg {
    configure_problem_arg(Arg::new("problem"))
}

pub(crate) fn problem_option_arg_optional() -> Arg {
    problem_arg_optional().short('p').long("problem")
}

pub(crate) fn problem_option_arg_required() -> Arg {
    problem_option_arg_optional().required(true)
}

pub(crate) fn competition_arg_optional() -> Arg {
    configure_competition_arg(Arg::new("comp"))
}

pub(crate) fn competition_arg_required() -> Arg {
    competition_arg_optional().required(true)
}

pub(crate) fn competition_option_arg_optional() -> Arg {
    competition_arg_optional().short('c').long("comp")
}

pub(crate) fn competition_option_arg_required() -> Arg {
    competition_option_arg_optional().required(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problem_option_arg_helper_sets_expected_metadata() {
        let arg = problem_option_arg_required();

        assert_eq!(arg.get_id().as_str(), "problem");
        assert_eq!(arg.get_short(), Some('p'));
        assert_eq!(arg.get_long(), Some("problem"));
        assert_eq!(
            arg.get_value_names()
                .and_then(|names| names.first())
                .map(|name| name.as_str()),
            Some(PROBLEM_VALUE_NAME)
        );
        assert!(matches!(arg.get_action(), ArgAction::Set));
        assert!(arg.is_required_set());
    }

    #[test]
    fn competition_option_arg_helper_sets_expected_metadata() {
        let arg = competition_option_arg_optional();

        assert_eq!(arg.get_id().as_str(), "comp");
        assert_eq!(arg.get_short(), Some('c'));
        assert_eq!(arg.get_long(), Some("comp"));
        assert_eq!(
            arg.get_value_names()
                .and_then(|names| names.first())
                .map(|name| name.as_str()),
            Some(COMPETITION_VALUE_NAME)
        );
        assert!(matches!(arg.get_action(), ArgAction::Set));
        assert!(!arg.is_required_set());
    }
}
