// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

const ARG_PREFIX: &str = "--";
const ARG_SEPARATOR: &str = "--";
const HELP_ARG: &str = "--help";
const VERSION_ARG: &str = "--version";

/// Errors associated with parsing and validating arguments.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The required argument was not provided.
    MissingArgument(String),
    /// A value for the argument was not provided.
    MissingValue(String),
    /// The provided argument was not expected.
    UnexpectedArgument(String),
    /// The argument was provided more than once.
    DuplicateArgument(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match *self {
            MissingArgument(ref arg) => write!(f, "Argument '{}' required, but not found.", arg),
            MissingValue(ref arg) => write!(
                f,
                "The argument '{}' requires a value, but none was supplied.",
                arg
            ),
            UnexpectedArgument(ref arg) => write!(
                f,
                "Found argument '{}' which wasn't expected, or isn't valid in this context.",
                arg
            ),
            DuplicateArgument(ref arg) => {
                write!(f, "The argument '{}' was provided more than once.", arg)
            }
        }
    }
}

/// Keep information about the argument parser.
#[derive(Clone, Default)]
pub struct ArgParser<'a> {
    arguments: Arguments<'a>,
}

impl<'a> ArgParser<'a> {
    /// Create a new ArgParser instance.
    pub fn new() -> Self {
        ArgParser::default()
    }

    /// Add an argument with its associated `Argument` in `arguments`.
    pub fn arg(mut self, argument: Argument<'a>) -> Self {
        self.arguments.insert_arg(argument);
        self
    }

    /// Parse the command line arguments.
    pub fn parse_from_cmdline(&mut self) -> Result<()> {
        self.arguments.parse_from_cmdline()
    }

    /// Concatenate the `help` information of every possible argument
    /// in a message that represents the correct command line usage
    /// for the application.
    pub fn formatted_help(&self) -> String {
        let mut help_builder = vec![];

        for arg in self.arguments.args.values() {
            help_builder.push(format!("{}\n", arg.format_help()));
        }

        help_builder.concat()
    }

    /// Return a reference to `arguments` field.
    pub fn arguments(&self) -> &Arguments {
        &self.arguments
    }
}

/// Stores the characteristics of the `name` command line argument.
#[derive(Clone, Debug, PartialEq)]
pub struct Argument<'a> {
    name: &'a str,
    required: bool,
    requires: Option<&'a str>,
    takes_value: bool,
    default_value: Option<Value>,
    help: Option<&'a str>,
    user_value: Option<Value>,
}

impl<'a> Argument<'a> {
    /// Create a new `Argument` that keeps the necessary information for an argument.
    pub fn new(name: &'a str) -> Argument<'a> {
        Argument {
            name,
            required: false,
            requires: None,
            takes_value: false,
            default_value: None,
            help: None,
            user_value: None,
        }
    }

    /// Set if the argument *must* be provided by user.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Add `other_arg` as a required parameter when `self` is specified.
    pub fn requires(mut self, other_arg: &'a str) -> Self {
        self.requires = Some(other_arg);
        self
    }

    /// If `takes_value` is true, then the user *must* provide a value for the
    /// argument, otherwise that argument is a flag.
    pub fn takes_value(mut self, takes_value: bool) -> Self {
        self.takes_value = takes_value;
        self
    }

    /// Keep a default value which will be used if the user didn't provide a value for
    /// the argument.
    pub fn default_value(mut self, default_value: &'a str) -> Self {
        self.default_value = Some(Value::String(String::from(default_value)));
        self
    }

    /// Set the information that will be displayed for the argument when user passes
    /// `--help` flag.
    pub fn help(mut self, help: &'a str) -> Self {
        self.help = Some(help);
        self
    }

    fn format_help(&self) -> String {
        let mut help_builder = vec![];

        help_builder.push(format!("--{}", self.name));

        if self.takes_value {
            help_builder.push(format!(" <{}>", self.name));
        }

        if let Some(help) = self.help {
            help_builder.push(format!(": {}", help));
        }
        help_builder.concat()
    }
}

/// Represents the value of an argument, which will be a `String` if
/// the argument takes a value, or `bool` if it's a flag.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    String(String),
}

impl Value {
    fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Stores the arguments of the parser.
#[derive(Clone, Default)]
pub struct Arguments<'a> {
    // A Hash Map in which the key is an argument and the value is its associated `Argument`.
    args: HashMap<&'a str, Argument<'a>>,
    // The arguments specified after `--` (i.e. end of command options).
    extra_args: Vec<String>,
}

impl<'a> Arguments<'a> {
    /// Add an argument with its associated `Argument` in `args`.
    fn insert_arg(&mut self, argument: Argument<'a>) {
        self.args.insert(argument.name, argument);
    }

    /// Get the value for the argument specified by `arg_name`.
    fn value_of(&self, arg_name: &'static str) -> Option<&Value> {
        self.args.get(arg_name).and_then(|argument| {
            argument
                .user_value
                .as_ref()
                .or_else(|| argument.default_value.as_ref())
        })
    }

    /// Return the value of an argument if the argument exists and has the type
    /// String. Otherwise return None.
    pub fn value_as_string(&self, arg_name: &'static str) -> Option<String> {
        self.value_of(arg_name)
            .and_then(|arg_value| arg_value.as_string())
    }

    /// Return the value of an argument if the argument exists and has the type
    /// bool. Otherwise return None.
    pub fn value_as_bool(&self, arg_name: &'static str) -> Option<bool> {
        self.value_of(arg_name)
            .and_then(|arg_value| arg_value.as_bool())
    }

    /// Get the extra arguments (all arguments after `--`).
    pub fn extra_args(&self) -> Vec<String> {
        self.extra_args.clone()
    }

    // Split `args` in two slices: one with the actual arguments of the process and the other with
    // the extra arguments, meaning all parameters specified after `--`.
    fn split_args(args: &[String]) -> (&[String], &[String]) {
        if let Some(index) = args.iter().position(|arg| arg == ARG_SEPARATOR) {
            return (&args[..index], &args[index + 1..]);
        }

        (args, &[])
    }

    /// Collect the command line arguments and the values provided for them.
    pub fn parse_from_cmdline(&mut self) -> Result<()> {
        let args: Vec<String> = env::args().collect();

        self.parse(&args)
    }

    /// Clear split between the actual arguments of the process, the extra arguments if any
    /// and the `--help` argument if present.
    pub fn parse(&mut self, args: &[String]) -> Result<()> {
        // Skipping the first element of `args` as it is the name of the binary.
        let (args, extra_args) = Arguments::split_args(&args[1..]);
        self.extra_args = extra_args.to_vec();

        // If `--help` is provided as a parameter, we artificially skip the parsing of other
        // command line arguments by adding just the help argument to the parsed list and
        // returning.
        if args.contains(&HELP_ARG.to_string()) {
            let mut help_arg = Argument::new("help").help("Show the help message.");
            help_arg.user_value = Some(Value::Bool(true));
            self.insert_arg(help_arg);
            return Ok(());
        }

        // If `--version` is provided as a parameter, we artificially skip the parsing of other
        // command line arguments by adding just the version argument to the parsed list and
        // returning.
        if args.contains(&VERSION_ARG.to_string()) {
            let mut version_arg = Argument::new("version").help("Print the binary version number.");
            version_arg.user_value = Some(Value::Bool(true));
            self.insert_arg(version_arg);
            return Ok(());
        }

        // Otherwise, we continue the parsing of the other arguments.
        self.populate_args(args)
    }

    // Check if `required` and `requires` field rules are indeed followed by every argument.
    fn validate_requirements(&self, args: &[String]) -> Result<()> {
        for argument in self.args.values() {
            // The arguments that are marked `required` must be provided by user.
            if argument.required && argument.user_value.is_none() {
                return Err(Error::MissingArgument(argument.name.to_string()));
            }
            // For the arguments that require a specific argument to be also present in the list
            // of arguments provided by user, search for that argument.
            if argument.user_value.is_some() {
                if let Some(arg_name) = argument.requires {
                    if !args.contains(&(format!("--{}", arg_name))) {
                        return Err(Error::MissingArgument(arg_name.to_string()));
                    }
                }
            }
        }
        Ok(())
    }

    // Does a general validation of `arg` command line argument.
    fn validate_arg(&self, arg: &str) -> Result<()> {
        if !arg.starts_with(ARG_PREFIX) {
            return Err(Error::UnexpectedArgument(arg.to_string()));
        }
        let arg_name = &arg[ARG_PREFIX.len()..];

        // Check if the argument is an expected one and, if yes, check that it was not
        // provided more than once.
        if self
            .args
            .get(arg_name)
            .ok_or_else(|| Error::UnexpectedArgument(arg_name.to_string()))?
            .user_value
            .is_some()
        {
            return Err(Error::DuplicateArgument(arg_name.to_string()));
        }
        Ok(())
    }

    /// Validate the arguments provided by user and their values. Insert those
    /// values in the `Argument` instances of the corresponding arguments.
    fn populate_args(&mut self, args: &[String]) -> Result<()> {
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            self.validate_arg(arg)?;

            // If the `arg` argument is indeed an expected one, set the value provided by user
            // if it's a valid one.
            let argument = self
                .args
                .get_mut(&arg[ARG_PREFIX.len()..])
                .ok_or_else(|| Error::UnexpectedArgument(arg[ARG_PREFIX.len()..].to_string()))?;

            let arg_val = if argument.takes_value {
                let val = iter
                    .next()
                    .filter(|v| !v.starts_with(ARG_PREFIX))
                    .ok_or_else(|| Error::MissingValue(argument.name.to_string()))?
                    .clone();
                Value::String(val)
            } else {
                Value::Bool(true)
            };

            argument.user_value = Some(arg_val);
        }

        // Check the constraints for the `required` and `requires` fields of all arguments.
        self.validate_requirements(args)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arg_parser::Value;

    fn build_arg_parser() -> ArgParser<'static> {
        ArgParser::new()
            .arg(
                Argument::new("exec-file")
                    .required(true)
                    .takes_value(true)
                    .help("'exec-file' info."),
            )
            .arg(
                Argument::new("no-api")
                    .requires("config-file")
                    .takes_value(false)
                    .help("'no-api' info."),
            )
            .arg(
                Argument::new("api-sock")
                    .takes_value(true)
                    .default_value("socket")
                    .help("'api-sock' info."),
            )
            .arg(
                Argument::new("id")
                    .takes_value(true)
                    .default_value("instance")
                    .help("'id' info."),
            )
            .arg(
                Argument::new("seccomp-level")
                    .takes_value(true)
                    .default_value("2")
                    .help("'seccomp-level' info."),
            )
            .arg(
                Argument::new("config-file")
                    .takes_value(true)
                    .help("'config-file' info."),
            )
    }

    #[test]
    fn test_arg_help() {
        // Checks help format for an argument.
        let mut argument = Argument::new("exec-file").required(true).takes_value(true);

        assert_eq!(argument.format_help(), "--exec-file <exec-file>");

        argument = Argument::new("exec-file")
            .required(true)
            .takes_value(true)
            .help("'exec-file' info.");

        assert_eq!(
            argument.format_help(),
            "--exec-file <exec-file>: 'exec-file' info."
        );

        argument = Argument::new("no-api")
            .requires("config-file")
            .takes_value(false);

        assert_eq!(argument.format_help(), "--no-api");

        argument = Argument::new("no-api")
            .requires("config-file")
            .takes_value(false)
            .help("'no-api' info.");

        assert_eq!(argument.format_help(), "--no-api: 'no-api' info.");
    }

    #[test]
    fn test_arg_parser_help() {
        // Checks help information when user passes `--help` flag.
        let arg_parser = ArgParser::new().arg(
            Argument::new("exec-file")
                .required(true)
                .takes_value(true)
                .help("'exec-file' info."),
        );
        assert_eq!(
            arg_parser.formatted_help(),
            "--exec-file <exec-file>: 'exec-file' info.\n"
        );
    }

    #[test]
    fn test_value() {
        //Test `as_string()` and `as_bool()` functions behaviour.
        let mut value = Value::Bool(true);
        assert!(Value::as_string(&value).is_none());
        value = Value::String("arg".to_string());
        assert_eq!(Value::as_string(&value).unwrap(), "arg".to_string());

        value = Value::String("arg".to_string());
        assert!(Value::as_bool(&value).is_none());
        value = Value::Bool(true);
        assert!(Value::as_bool(&value).unwrap());
    }

    #[test]
    fn test_parse() {
        let arg_parser = build_arg_parser();

        // Test different scenarios for the command line arguments provided by user.
        let mut arguments = arg_parser.arguments().clone();

        let args = vec!["binary-name", "--exec-file", "foo", "--help"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();

        assert!(arguments.parse(&args).is_ok());
        assert!(arguments.args.contains_key("help"));

        arguments = arg_parser.arguments().clone();

        let args = vec!["binary-name", "--exec-file", "foo", "--version"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();

        assert!(arguments.parse(&args).is_ok());
        assert!(arguments.args.contains_key("version"));

        arguments = arg_parser.arguments().clone();

        let args = vec![
            "binary-name",
            "--exec-file",
            "foo",
            "--api-sock",
            "--id",
            "bar",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

        assert_eq!(
            arguments.parse(&args),
            Err(Error::MissingValue("api-sock".to_string()))
        );

        arguments = arg_parser.arguments().clone();

        let args = vec![
            "binary-name",
            "--exec-file",
            "foo",
            "--api-sock",
            "bar",
            "--api-sock",
            "foobar",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

        assert_eq!(
            arguments.parse(&args),
            Err(Error::DuplicateArgument("api-sock".to_string()))
        );

        arguments = arg_parser.arguments().clone();

        let args = vec!["binary-name", "--api-sock", "foo"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();

        assert_eq!(
            arguments.parse(&args),
            Err(Error::MissingArgument("exec-file".to_string()))
        );

        arguments = arg_parser.arguments().clone();

        let args = vec![
            "binary-name",
            "--exec-file",
            "foo",
            "--api-sock",
            "bar",
            "--invalid-arg",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

        assert_eq!(
            arguments.parse(&args),
            Err(Error::UnexpectedArgument("invalid-arg".to_string()))
        );

        arguments = arg_parser.arguments().clone();

        let args = vec![
            "binary-name",
            "--exec-file",
            "foo",
            "--api-sock",
            "bar",
            "--id",
            "foobar",
            "--no-api",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

        assert_eq!(
            arguments.parse(&args),
            Err(Error::MissingArgument("config-file".to_string()))
        );

        arguments = arg_parser.arguments().clone();

        let args = vec![
            "binary-name",
            "--exec-file",
            "foo",
            "--api-sock",
            "bar",
            "--id",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

        assert_eq!(
            arguments.parse(&args),
            Err(Error::MissingValue("id".to_string()))
        );

        arguments = arg_parser.arguments().clone();

        let args = vec![
            "binary-name",
            "--exec-file",
            "foo",
            "--config-file",
            "bar",
            "--no-api",
            "foobar",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

        assert_eq!(
            arguments.parse(&args),
            Err(Error::UnexpectedArgument("foobar".to_string()))
        );

        arguments = arg_parser.arguments().clone();

        let args = vec![
            "binary-name",
            "--exec-file",
            "foo",
            "--api-sock",
            "bar",
            "foobar",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

        assert_eq!(
            arguments.parse(&args),
            Err(Error::UnexpectedArgument("foobar".to_string()))
        );

        arguments = arg_parser.arguments().clone();

        let args = vec!["binary-name", "foo"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();

        assert_eq!(
            arguments.parse(&args),
            Err(Error::UnexpectedArgument("foo".to_string()))
        );

        arguments = arg_parser.arguments().clone();

        let args = vec![
            "binary-name",
            "--exec-file",
            "foo",
            "--api-sock",
            "bar",
            "--id",
            "foobar",
            "--seccomp-level",
            "0",
            "--",
            "--extra-flag",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

        assert!(arguments.parse(&args).is_ok());
        assert!(arguments.extra_args.contains(&"--extra-flag".to_string()));
    }

    #[test]
    fn test_split() {
        let mut args = vec!["--exec-file", "foo", "--", "--extra-arg-1", "--extra-arg-2"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
        let (left, right) = Arguments::split_args(&args);
        assert_eq!(left.to_vec(), vec!["--exec-file", "foo"]);
        assert_eq!(right.to_vec(), vec!["--extra-arg-1", "--extra-arg-2"]);

        args = vec!["--exec-file", "foo", "--"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
        let (left, right) = Arguments::split_args(&args);
        assert_eq!(left.to_vec(), vec!["--exec-file", "foo"]);
        assert!(right.is_empty());

        args = vec!["--exec-file", "foo"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
        let (left, right) = Arguments::split_args(&args);
        assert_eq!(left.to_vec(), vec!["--exec-file", "foo"]);
        assert!(right.is_empty());
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            format!("{}", Error::MissingArgument("foo".to_string())),
            "Argument 'foo' required, but not found."
        );
        assert_eq!(
            format!("{}", Error::MissingValue("foo".to_string())),
            "The argument 'foo' requires a value, but none was supplied."
        );
        assert_eq!(
            format!("{}", Error::UnexpectedArgument("foo".to_string())),
            "Found argument 'foo' which wasn't expected, or isn't valid in this context."
        );
        assert_eq!(
            format!("{}", Error::DuplicateArgument("foo".to_string())),
            "The argument 'foo' was provided more than once."
        );
    }
}
