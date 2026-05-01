//! Cursor analysis and clap-lex-backed token resolution for shell completion.

use clap::{Arg, ArgAction, ArgMatches, Command};
use clap_lex::RawArgs;

/// The kind of completion requested at the cursor position.
pub(super) enum CompletionRequest<'a> {
    /// Complete subcommand names for the current command.
    CommandName { cmd: &'a Command, current: String },
    /// Complete option names for the current command.
    OptionName { cmd: &'a Command, current: String },
    /// Complete values for a specific argument.
    ArgValue(ValueTarget<'a>),
    /// No completion candidates are meaningful at this cursor position.
    None,
}

/// A parsed option token resolved against a clap [`Arg`].
struct OptionMatch<'a> {
    /// The clap argument matched by the token.
    arg: &'a Arg,
    /// The inline value attached to the option token, if any.
    attached_value: Option<String>,
    /// Prefix to re-add when returning completions for inline values.
    replacement_prefix: String,
}

impl<'a> OptionMatch<'a> {
    /// Return whether this option expects a value.
    fn takes_value(&self) -> bool {
        arg_takes_value(self.arg)
    }

    /// Convert an inline `--opt=value` / `-ovalue` token into a value target.
    fn attached_value_target(&self) -> Option<ValueTarget<'a>> {
        if !self.takes_value() {
            return None;
        }

        Some(ValueTarget::inline(
            self.arg,
            self.attached_value.clone()?,
            self.replacement_prefix.clone(),
        ))
    }

    /// Convert a standalone option token into the following-word value target.
    fn following_value_target(&self, current: &str) -> Option<ValueTarget<'a>> {
        if self.attached_value.is_some() || !self.takes_value() {
            return None;
        }

        Some(ValueTarget::standalone(self.arg, current))
    }
}

/// A concrete argument value currently being completed.
pub(super) struct ValueTarget<'a> {
    /// The clap argument that owns the value being completed.
    pub(super) arg: &'a Arg,
    /// The current partial value typed by the user.
    pub(super) current_value: String,
    /// Prefix to prepend when replacing inline option values.
    pub(super) replacement_prefix: String,
}

impl<'a> ValueTarget<'a> {
    /// Build a target for a value typed as a separate shell word.
    fn standalone(arg: &'a Arg, current_value: &str) -> Self {
        Self {
            arg,
            current_value: current_value.to_owned(),
            replacement_prefix: String::new(),
        }
    }

    /// Build a target for a value attached inline to its option token.
    fn inline(arg: &'a Arg, current_value: String, replacement_prefix: String) -> Self {
        Self {
            arg,
            current_value,
            replacement_prefix,
        }
    }
}

/// Return whether a clap argument consumes one or more values.
fn arg_takes_value(arg: &Arg) -> bool {
    arg.get_num_args()
        .unwrap_or_else(|| match arg.get_action() {
            ArgAction::Set | ArgAction::Append => 1.into(),
            ArgAction::SetTrue
            | ArgAction::SetFalse
            | ArgAction::Count
            | ArgAction::Help
            | ArgAction::HelpShort
            | ArgAction::HelpLong
            | ArgAction::Version => 0.into(),
            _ => 1.into(),
        })
        .takes_values()
}

/// Shared clap-lex classification for a single shell token.
struct TokenClass {
    /// Whether the token is a literal `--` escape.
    is_escape: bool,
    /// Whether clap-lex recognizes the token as a negative number.
    is_negative_number: bool,
    /// Whether clap-lex recognizes the token as a short or long option.
    is_option: bool,
}

/// Classify a single shell token using clap-lex.
#[cfg(test)]
fn classify_token(token: &str) -> Option<TokenClass> {
    let raw = RawArgs::new([token]);
    let mut cursor = raw.cursor();
    let arg = raw.next(&mut cursor)?;

    Some(TokenClass {
        is_escape: arg.is_escape(),
        is_negative_number: arg.is_negative_number(),
        is_option: arg.is_long() || arg.is_short(),
    })
}

/// Analyze a shell token for a specific clap command.
struct CommandToken<'a, 't> {
    /// Original token text.
    token: &'t str,
    /// Shared clap-lex classification.
    class: TokenClass,
    /// Resolved clap option match, if this token names an option.
    option_match: Option<OptionMatch<'a>>,
}

impl<'a, 't> CommandToken<'a, 't> {
    /// Parse and analyze a token once for the given command.
    fn parse(cmd: &'a Command, token: &'t str) -> Option<Self> {
        let raw = RawArgs::new([token]);
        let mut cursor = raw.cursor();
        let arg = raw.next(&mut cursor)?;

        let class = TokenClass {
            is_escape: arg.is_escape(),
            is_negative_number: arg.is_negative_number(),
            is_option: arg.is_long() || arg.is_short(),
        };

        let option_match = if let Some((name, attached_value)) = arg.to_long() {
            let name = name.ok()?;
            let attached_value = match attached_value {
                Some(value) => Some(value.to_str()?.to_owned()),
                None => None,
            };

            cmd.get_arguments()
                .find(|arg| matches_long(arg, name))
                .map(|arg| OptionMatch {
                    arg,
                    attached_value,
                    replacement_prefix: format!("--{name}="),
                })
        } else if let Some(mut shorts) = arg.to_short() {
            let mut consumed = '-'.len_utf8();
            let mut last_match = None;

            while let Some(name) = shorts.next_flag() {
                let name = name.ok()?;
                consumed += name.len_utf8();

                let Some(arg) = cmd.get_arguments().find(|arg| matches_short(arg, name)) else {
                    last_match = None;
                    break;
                };
                let attached_value = if arg_takes_value(arg) {
                    match shorts.next_value_os() {
                        Some(value) => Some(value.to_str()?.to_owned()),
                        None => None,
                    }
                } else {
                    None
                };

                let option_match = OptionMatch {
                    arg,
                    attached_value,
                    replacement_prefix: token[..consumed].to_owned(),
                };

                if option_match.takes_value() {
                    last_match = Some(option_match);
                    break;
                }

                last_match = Some(option_match);
            }

            last_match
        } else {
            None
        };

        Some(Self {
            token,
            class,
            option_match,
        })
    }

    /// Return whether the token is a literal `--` escape.
    fn is_escape(&self) -> bool {
        self.class.is_escape
    }

    /// Return whether the token should be treated as a non-option word.
    fn is_non_option_word(&self) -> bool {
        !self.class.is_option
    }

    /// Return whether the token is a negative number.
    fn is_negative_number(&self) -> bool {
        self.class.is_negative_number
    }

    /// Return whether the token should trigger option-name completion.
    fn requests_option_name_completion(&self) -> bool {
        self.class.is_option && !self.class.is_negative_number
    }
}

/// Return whether a shell token is lexed by clap as a negative number.
#[cfg(test)]
fn token_is_negative_number(token: &str) -> bool {
    classify_token(token).is_some_and(|token| token.is_negative_number)
}

/// Return whether the token should trigger option-name completion.
#[cfg(test)]
fn requests_option_name_completion(token: &str) -> bool {
    classify_token(token).is_some_and(|token| token.is_option && !token.is_negative_number)
}

/// Cursor-local view of the current shell words.
struct CursorContext<'a> {
    current: String,
    prev: Option<&'a str>,
}

impl<'a> CursorContext<'a> {
    fn new(words: &'a [String], cword: usize) -> Self {
        Self {
            current: words.get(cword).cloned().unwrap_or_default(),
            prev: cword
                .checked_sub(1)
                .and_then(|idx| words.get(idx))
                .map(String::as_str),
        }
    }
}

/// Command/escape state derived from the already-typed prefix.
struct PrefixScan<'a> {
    /// The active clap command at the cursor position.
    cmd: &'a Command,
    /// Whether the cursor is after a literal `--` escape.
    after_escape: bool,
}

/// Full prefix analysis used during completion resolution.
struct PrefixAnalysis<'a> {
    /// Lightweight token scan used to track command context and `--` handling.
    scan: PrefixScan<'a>,
    /// Clap parsing of the already-typed prefix, if parsing succeeded.
    parsed_matches: Option<ArgMatches>,
}

impl<'a> PrefixAnalysis<'a> {
    /// Analyze the already-typed command prefix once for the current cursor position.
    fn new(root: &'a Command, words: &[String], cword: usize) -> Self {
        Self {
            scan: scan_prefix(root, words, cword),
            parsed_matches: parse_prefix_matches(root, words, cword),
        }
    }

    /// Return the command active at the cursor position.
    fn cmd(&self) -> &'a Command {
        self.scan.cmd
    }

    /// Return whether the cursor is after a literal `--` escape.
    fn after_escape(&self) -> bool {
        self.scan.after_escape
    }

    /// Resolve a positional value target, if the cursor is completing one.
    fn positional_value_target(
        &self,
        root: &'a Command,
        current: &str,
        current_token: Option<&CommandToken<'a, '_>>,
    ) -> Option<ValueTarget<'a>> {
        let prefix_matches = self.parsed_matches.as_ref()?;
        let (parsed_cmd, active_matches) = active_command_context(root, prefix_matches);

        if !std::ptr::eq(parsed_cmd, self.cmd()) {
            return None;
        }

        positional_value_target(
            self.cmd(),
            active_matches,
            current,
            current_token,
            self.after_escape(),
        )
    }

    /// Fall back to option-name, subcommand, or no completion.
    fn fallback_request(
        &self,
        current: String,
        current_token: Option<&CommandToken<'a, '_>>,
    ) -> CompletionRequest<'a> {
        if self.after_escape() {
            CompletionRequest::None
        } else if current_token.is_some_and(CommandToken::requests_option_name_completion) {
            CompletionRequest::OptionName {
                cmd: self.cmd(),
                current,
            }
        } else {
            CompletionRequest::CommandName {
                cmd: self.cmd(),
                current,
            }
        }
    }
}

/// Walk the already-typed prefix to find the active command and escape state.
fn scan_prefix<'a>(root: &'a Command, words: &[String], cword: usize) -> PrefixScan<'a> {
    let mut cmd = root;
    let mut escaped = false;
    let mut expecting_value = false;

    for token in words.iter().take(cword).skip(1).map(String::as_str) {
        if escaped {
            continue;
        }

        if expecting_value {
            expecting_value = false;
            continue;
        }

        let Some(token) = CommandToken::parse(cmd, token) else {
            continue;
        };

        if token.is_escape() {
            escaped = true;
            continue;
        }

        if token.is_non_option_word() {
            if let Some(subcommand) = cmd.find_subcommand(token.token) {
                cmd = subcommand;
            }
            continue;
        }

        if token.option_match.as_ref().is_some_and(|option_match| {
            option_match.takes_value() && option_match.attached_value.is_none()
        }) {
            expecting_value = true;
        }
    }

    PrefixScan {
        cmd,
        after_escape: escaped,
    }
}

/// Return whether parsing state at the cursor is after a literal `--` escape.
#[cfg(test)]
fn is_cursor_after_escape(root: &Command, words: &[String], cword: usize) -> bool {
    scan_prefix(root, words, cword).after_escape
}

/// Return whether the token matches a long option or visible long alias.
fn matches_long(arg: &Arg, name: &str) -> bool {
    arg.get_long_and_visible_aliases()
        .is_some_and(|longs| longs.into_iter().any(|candidate| candidate == name))
}

/// Return whether the token matches a short option or visible short alias.
fn matches_short(arg: &Arg, name: char) -> bool {
    arg.get_short_and_visible_aliases()
        .is_some_and(|shorts| shorts.into_iter().any(|candidate| candidate == name))
}

/// Match a raw shell token to a clap option on the given command.
#[cfg(test)]
fn match_option<'a>(cmd: &'a Command, token: &str) -> Option<OptionMatch<'a>> {
    CommandToken::parse(cmd, token)?.option_match
}

/// Walk parsed clap matches to determine the active command at the cursor.
fn active_command_context<'a, 'm>(
    root: &'a Command,
    matches: &'m ArgMatches,
) -> (&'a Command, &'m ArgMatches) {
    let mut cmd = root;
    let mut current_matches = matches;

    while let Some((name, sub_matches)) = current_matches.subcommand() {
        let Some(subcommand) = cmd.find_subcommand(name) else {
            break;
        };

        cmd = subcommand;
        current_matches = sub_matches;
    }

    (cmd, current_matches)
}

/// Count positional arguments already consumed for the active command.
fn count_consumed_positionals(cmd: &Command, matches: &ArgMatches) -> usize {
    cmd.get_positionals()
        .map(|arg| {
            matches
                .indices_of(arg.get_id().as_str())
                .map(|indices| indices.count())
                .unwrap_or(0)
        })
        .sum()
}

/// Return whether the current token may be treated as positional input.
fn allows_positional_completion(
    current: &str,
    current_token: Option<&CommandToken<'_, '_>>,
    after_escape: bool,
) -> bool {
    after_escape
        || !current.starts_with('-')
        || current_token.is_some_and(CommandToken::is_negative_number)
}

/// Resolve the positional argument value currently being completed, if any.
fn positional_value_target<'a>(
    cmd: &'a Command,
    matches: &ArgMatches,
    current: &str,
    current_token: Option<&CommandToken<'a, '_>>,
    after_escape: bool,
) -> Option<ValueTarget<'a>> {
    if !allows_positional_completion(current, current_token, after_escape) {
        return None;
    }

    let arg = cmd
        .get_positionals()
        .nth(count_consumed_positionals(cmd, matches))?;

    Some(ValueTarget::standalone(arg, current))
}

/// Resolve an inline current-token option value like `--problem=fo` or `-pfo`.
fn current_token_option_value_target<'a>(
    current_token: Option<&CommandToken<'a, '_>>,
    after_escape: bool,
) -> Option<ValueTarget<'a>> {
    if after_escape {
        return None;
    }

    current_token
        .and_then(|token| token.option_match.as_ref())
        .and_then(OptionMatch::attached_value_target)
}

/// Resolve a next-word option value like `-p fo`.
fn previous_token_option_value_target<'a>(
    prev_token: Option<&CommandToken<'a, '_>>,
    current: &str,
) -> Option<ValueTarget<'a>> {
    prev_token
        .and_then(|token| token.option_match.as_ref())
        .and_then(|option_match| option_match.following_value_target(current))
}

/// Resolve the option argument value currently being completed, if any.
fn option_value_target<'a>(
    current_token: Option<&CommandToken<'a, '_>>,
    prev_token: Option<&CommandToken<'a, '_>>,
    current: &str,
    after_escape: bool,
) -> Option<ValueTarget<'a>> {
    current_token_option_value_target(current_token, after_escape)
        .or_else(|| previous_token_option_value_target(prev_token, current))
}

/// Parse the already-typed prefix with clap, tolerating incomplete input.
fn parse_prefix_matches(root: &Command, words: &[String], cword: usize) -> Option<ArgMatches> {
    let prefix_words: Vec<String> = if words.is_empty() || cword == 0 {
        vec![crate::BIN_NAME.to_owned()]
    } else {
        words.iter().take(cword).cloned().collect()
    };

    let mut parser = root.clone().ignore_errors(true);
    parser.try_get_matches_from_mut(prefix_words).ok()
}

fn top_level_command_request<'a>(
    root: &'a Command,
    words: &[String],
    cword: usize,
    current: String,
) -> Option<CompletionRequest<'a>> {
    if cword == 1 || (words.get(1).map(String::as_str) == Some("help") && cword == 2) {
        Some(CompletionRequest::CommandName { cmd: root, current })
    } else {
        None
    }
}

/// Classify the cursor position into a single completion request.
pub(super) fn resolve_request<'a>(
    root: &'a Command,
    words: &[String],
    cword: usize,
) -> CompletionRequest<'a> {
    let cursor = CursorContext::new(words, cword);

    if let Some(request) = top_level_command_request(root, words, cword, cursor.current.clone()) {
        return request;
    }

    let prefix = PrefixAnalysis::new(root, words, cword);
    let fallback_current = cursor.current.clone();
    let current_token = CommandToken::parse(prefix.cmd(), &cursor.current);
    let prev_token = cursor
        .prev
        .and_then(|prev| CommandToken::parse(prefix.cmd(), prev));

    if let Some(target) = option_value_target(
        current_token.as_ref(),
        prev_token.as_ref(),
        &cursor.current,
        prefix.after_escape(),
    ) {
        return CompletionRequest::ArgValue(target);
    }

    if let Some(target) =
        prefix.positional_value_target(root, &cursor.current, current_token.as_ref())
    {
        return CompletionRequest::ArgValue(target);
    }

    prefix.fallback_request(fallback_current, current_token.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cmd() -> Command {
        Command::new("aucpl")
            .arg(Arg::new("alpha").short('a').action(ArgAction::SetTrue))
            .arg(Arg::new("beta").short('b').action(ArgAction::Set))
            .arg(Arg::new("gamma").short('g').action(ArgAction::SetTrue))
            .arg(Arg::new("problem").long("problem").action(ArgAction::Set))
    }

    #[test]
    fn matches_long_option_with_inline_value() {
        let cmd = test_cmd();
        let matched = match_option(&cmd, "--problem=foo").expect("long option should match");

        assert_eq!(matched.arg.get_id().as_str(), "problem");
        assert_eq!(matched.attached_value.as_deref(), Some("foo"));
        assert_eq!(matched.replacement_prefix, "--problem=");
    }

    #[test]
    fn matches_short_option_with_inline_value() {
        let cmd = test_cmd();
        let matched = match_option(&cmd, "-bfoo").expect("short option should match");

        assert_eq!(matched.arg.get_id().as_str(), "beta");
        assert_eq!(matched.attached_value.as_deref(), Some("foo"));
        assert_eq!(matched.replacement_prefix, "-b");
    }

    #[test]
    fn respects_short_flag_clusters() {
        let cmd = test_cmd();
        let matched = match_option(&cmd, "-abfoo").expect("short cluster should match");

        assert_eq!(matched.arg.get_id().as_str(), "beta");
        assert_eq!(matched.attached_value.as_deref(), Some("foo"));
        assert_eq!(matched.replacement_prefix, "-ab");
    }

    #[test]
    fn does_not_treat_flag_cluster_suffix_as_an_attached_value() {
        let cmd = Command::new("aucpl")
            .arg(Arg::new("alpha").short('a').action(ArgAction::SetTrue))
            .arg(Arg::new("beta").short('b').action(ArgAction::SetTrue))
            .arg(Arg::new("gamma").short('g').action(ArgAction::SetTrue));
        let matched = match_option(&cmd, "-abg").expect("short cluster should match");

        assert_eq!(matched.arg.get_id().as_str(), "gamma");
        assert_eq!(matched.attached_value, None);
    }

    #[test]
    fn detects_negative_number_tokens() {
        assert!(token_is_negative_number("-1"));
        assert!(token_is_negative_number("-3.14"));
        assert!(!token_is_negative_number("--problem"));
        assert!(!token_is_negative_number("-p"));
    }

    #[test]
    fn detects_tokens_that_request_option_name_completion() {
        assert!(requests_option_name_completion("-p"));
        assert!(requests_option_name_completion("--problem"));
        assert!(!requests_option_name_completion("-"));
        assert!(!requests_option_name_completion("--"));
        assert!(!requests_option_name_completion("-1"));
    }

    #[test]
    fn resolves_negative_number_current_token_as_positional_value() {
        let root = Command::new("aucpl").subcommand(
            Command::new("create")
                .arg(Arg::new("difficulty").action(ArgAction::Set).required(true))
                .arg(Arg::new("name").action(ArgAction::Set).required(true)),
        );
        let words = vec!["aucpl".to_owned(), "create".to_owned(), "-1".to_owned()];

        let request = resolve_request(&root, &words, 2);

        assert!(matches!(request, CompletionRequest::ArgValue(_)));
    }

    #[test]
    fn resolves_value_completion_for_short_problem_option() {
        let root = Command::new("aucpl").subcommand(Command::new("problem").subcommand(
            Command::new("test").arg(Arg::new("problem").short('p').action(ArgAction::Set)),
        ));
        let words = vec![
            "aucpl".to_owned(),
            "problem".to_owned(),
            "test".to_owned(),
            "-p".to_owned(),
            "fo".to_owned(),
        ];

        let request = resolve_request(&root, &words, 4);

        match request {
            CompletionRequest::ArgValue(target) => {
                assert_eq!(target.arg.get_id().as_str(), "problem");
                assert_eq!(target.current_value, "fo");
                assert!(target.replacement_prefix.is_empty());
            }
            _ => panic!("expected short option value completion"),
        }
    }

    #[test]
    fn resolves_value_completion_for_inline_problem_option() {
        let root = Command::new("aucpl").subcommand(Command::new("problem").subcommand(
            Command::new("test").arg(Arg::new("problem").long("problem").action(ArgAction::Set)),
        ));
        let words = vec![
            "aucpl".to_owned(),
            "problem".to_owned(),
            "test".to_owned(),
            "--problem=fo".to_owned(),
        ];

        let request = resolve_request(&root, &words, 3);

        match request {
            CompletionRequest::ArgValue(target) => {
                assert_eq!(target.arg.get_id().as_str(), "problem");
                assert_eq!(target.current_value, "fo");
                assert_eq!(target.replacement_prefix, "--problem=");
            }
            _ => panic!("expected inline option value completion"),
        }
    }

    #[test]
    fn resolves_value_completion_for_short_comp_option() {
        let root = Command::new("aucpl").subcommand(Command::new("comp").subcommand(
            Command::new("list").arg(Arg::new("comp").short('c').action(ArgAction::Set)),
        ));
        let words = vec![
            "aucpl".to_owned(),
            "comp".to_owned(),
            "list".to_owned(),
            "-c".to_owned(),
            "ac".to_owned(),
        ];

        let request = resolve_request(&root, &words, 4);

        match request {
            CompletionRequest::ArgValue(target) => {
                assert_eq!(target.arg.get_id().as_str(), "comp");
                assert_eq!(target.current_value, "ac");
            }
            _ => panic!("expected competition option value completion"),
        }
    }

    #[test]
    fn resolves_positional_problem_value_for_cd() {
        let root = Command::new("aucpl")
            .subcommand(Command::new("cd").arg(Arg::new("problem").action(ArgAction::Set)));
        let words = vec!["aucpl".to_owned(), "cd".to_owned(), "al".to_owned()];

        let request = resolve_request(&root, &words, 2);

        match request {
            CompletionRequest::ArgValue(target) => {
                assert_eq!(target.arg.get_id().as_str(), "problem");
                assert_eq!(target.current_value, "al");
            }
            _ => panic!("expected positional problem value completion"),
        }
    }

    #[test]
    fn resolves_unknown_option_prefix_as_option_name_completion() {
        let root = Command::new("aucpl").subcommand(Command::new("problem").subcommand(
            Command::new("test").arg(Arg::new("problem").long("problem").action(ArgAction::Set)),
        ));
        let words = vec![
            "aucpl".to_owned(),
            "problem".to_owned(),
            "test".to_owned(),
            "--pro".to_owned(),
        ];

        assert!(matches!(
            resolve_request(&root, &words, 3),
            CompletionRequest::OptionName { .. }
        ));
    }

    #[test]
    fn does_not_offer_option_name_completion_for_stdio_or_escape_tokens() {
        let root = Command::new("aucpl").subcommand(Command::new("create"));

        let dash = resolve_request(
            &root,
            &["aucpl".to_owned(), "create".to_owned(), "-".to_owned()],
            2,
        );
        let escape = resolve_request(
            &root,
            &["aucpl".to_owned(), "create".to_owned(), "--".to_owned()],
            2,
        );

        assert!(matches!(dash, CompletionRequest::CommandName { .. }));
        assert!(matches!(escape, CompletionRequest::CommandName { .. }));
    }

    #[test]
    fn enters_escape_mode_after_double_dash() {
        let root = Command::new("aucpl").subcommand(
            Command::new("create")
                .arg(Arg::new("name").action(ArgAction::Set).required(true))
                .arg(Arg::new("extra").action(ArgAction::Set)),
        );
        let words = vec![
            "aucpl".to_owned(),
            "create".to_owned(),
            "foo".to_owned(),
            "--".to_owned(),
            "--problem=bar".to_owned(),
        ];

        assert!(is_cursor_after_escape(&root, &words, 4));
        assert!(matches!(
            resolve_request(&root, &words, 4),
            CompletionRequest::ArgValue(_)
        ));
    }

    #[test]
    fn double_dash_used_as_option_value_does_not_enter_escape_mode() {
        let root = Command::new("aucpl").subcommand(
            Command::new("create")
                .arg(Arg::new("problem").short('p').action(ArgAction::Set))
                .arg(Arg::new("name").action(ArgAction::Set)),
        );
        let words = vec![
            "aucpl".to_owned(),
            "create".to_owned(),
            "-p".to_owned(),
            "--".to_owned(),
            "--problem".to_owned(),
        ];

        assert!(!is_cursor_after_escape(&root, &words, 4));
        assert!(matches!(
            resolve_request(&root, &words, 4),
            CompletionRequest::OptionName { .. }
        ));
    }

    #[test]
    fn produces_no_completions_after_escape_without_a_positional_target() {
        let root = Command::new("aucpl").subcommand(Command::new("create"));
        let words = vec![
            "aucpl".to_owned(),
            "create".to_owned(),
            "--".to_owned(),
            "--problem".to_owned(),
        ];

        assert!(matches!(
            resolve_request(&root, &words, 3),
            CompletionRequest::None
        ));
    }

    #[test]
    fn completes_option_values_even_when_prefix_parsing_stops_at_a_missing_value() {
        let root = Command::new("aucpl").subcommand(Command::new("problem").subcommand(
            Command::new("test").arg(Arg::new("problem").short('p').action(ArgAction::Set)),
        ));
        let words = vec![
            "aucpl".to_owned(),
            "problem".to_owned(),
            "test".to_owned(),
            "-p".to_owned(),
        ];

        let request = resolve_request(&root, &words, 4);

        match request {
            CompletionRequest::ArgValue(target) => {
                assert_eq!(target.arg.get_id().as_str(), "problem");
                assert!(target.current_value.is_empty());
            }
            _ => panic!("expected option value completion"),
        }
    }
}
