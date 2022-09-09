//! An optional thread for writing logs.
//!
//! Goose can generate a number of log files during a load test, enabled through any combination of
//! the following run time options:
//!  - `--debug-log`, `--request-log`, `--transaction-log`, `--scenario-log`
//!
//! It's also possible to configure the format of any of thse logs to be `json`, `csv`, or `raw`
//! (the standard debug output of a Rust structure), using the following run time optios:
//!  - `--debug-format`, `--request-format`, `--transaction-format`, `--scenario-format`
//!
//! All of these loggers use a single shared logger thread, with
//! [`GooseUser`](../goose/struct.GooseUser.html)s sending log messages through the same shared
//! channel. The logger determines which log file to write the message to based on the message
//! data type. The logger thread uses Tokio's asynchronous
//! [`BufWriter`](https://docs.rs/tokio/*/tokio/io/struct.BufWriter.html). The logger thread only
//! starts if at least one logger is enabled.
//!
//! Note: there's also a `--goose-log` run time option which records any errors or messages
//! generated by Goose while running a load test. This functionality is not implemented in this
//! file.
//!
//! ## Request File logger
//! The Goose requests logger is enabled with the `--request-log` command-line option, or the
//! [`GooseDefault::RequestLog`](../config/enum.GooseDefault.html#variant.RequestLog) default
//! configuration option. The format of the log is configured with the `--request-format`
//! command-line option, or the
//! [`GooseDefault::RequestFormat`](../config/enum.GooseDefault.html#variant.RequestFormat) default
//! configuration option.
//!
//! Each [`GooseRequestMetric`] object generated by all [`GooseUser`](../goose/struct.GooseUser.html)
//! threads during a load test is written to this log file.
//!
//! ## Transaction File logger
//! The Goose transactions logger is enabled with the `--transaction-log` command-line option, or the
//! [`GooseDefault::TransactionLog`](../config/enum.GooseDefault.html#variant.TransactionLog) default
//! configuration option. The format of the log is configured with the `--transaction-format`
//! command-line option, or the
//! [`GooseDefault::TransactionFormat`](../config/enum.GooseDefault.html#variant.TransactionFormat) default
//! configuration option.
//!
//! Each [`TransactionMetric`] object generated by all [`GooseUser`](../goose/struct.GooseUser.html)
//! threads during a load test is written to this log file.
//!
//! ## Scenario File logger
//! The Goose scenarios logger is enabled with the `--scenario-log` command-line option, or the
//! [`GooseDefault::ScenarioLog`](../config/enum.GooseDefault.html#variant.ScenarioLog) default
//! configuration option. The format of the log is configured with the `--scenario-format`
//! command-line option, or the
//! [`GooseDefault::ScenarioFormat`](../config/enum.GooseDefault.html#variant.ScenarioFormat) default
//! configuration option.
//!
//! Each [`ScenarioMetric`] object generated by all [`GooseUser`](../goose/struct.GooseUser.html)
//! threads during a load test is written to this log file.
//!
//! ## Debug File logger
//! The Goose debug logger is enabled with the `--debug-log` command-line option, or the
//! [`GooseDefault::DebugLog`](../config/enum.GooseDefault.html#variant.DebugLog) default
//! configuration option.
//!
//! Each [`GooseDebug`] object generated by all [`GooseUser`](../goose/struct.GooseUser.html)
//! threads during a load test is written to this log file.
//!
//! ### Writing Debug Logs
//! Logs can be sent to the logger thread by invoking
//! [`log_debug`](../goose/struct.GooseUser.html#method.log_debug) from load test transaction functions.
//!
//! Calls to
//! [`set_failure`](../goose/struct.GooseUser.html#method.set_failure)
//! automatically invoke
//! [`log_debug`](../goose/struct.GooseUser.html#method.log_debug).
//!
//! Most of the included examples showing how to use the debug logger include a copy of the
//! request made, the response headers returned by the server, and the response body. It can
//! also be used to log arbitrary information, for example if you want to record everything you
//! sent via a POST to a form.
//!
//! ```rust
//! use goose::prelude::*;
//!
//! let mut transaction = transaction!(post_to_form);
//!
//! async fn post_to_form(user: &mut GooseUser) -> TransactionResult {
//!     let path = "/path/to/form";
//!     let params = [
//!      ("field_1", "foo"),
//!      ("field_2", "bar"),
//!      ("op", "Save"),
//!     ];
//!
//!     // Only log the form parameters we will post.
//!     user.log_debug(
//!         &format!("POSTing {:?} on {}", &params, path),
//!         None,
//!         None,
//!         None,
//!     )?;
//!
//!     let goose = user.post_form(path, &params).await?;
//!
//!     // Log the form parameters that were posted together with details about the entire
//!     // request that was sent to the server.
//!     user.log_debug(
//!         &format!("POSTing {:#?} on {}", &params, path),
//!         Some(&goose.request),
//!         None,
//!         None,
//!     )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! The first call to
//! [`log_debug`](../goose/struct.GooseUser.html#method.log_debug)
//! results in a debug log message similar to:
//! ```json
//! {"body":null,"header":null,"request":null,"tag":"POSTing [(\"field_1\", \"foo\"), (\"field_2\", \"bar\"), (\"op\", \"Save\")] on /path/to/form"}
//! ```
//!
//! The second call to
//! [`log_debug`](../goose/struct.GooseUser.html#method.log_debug)
//! results in a debug log message similar to:
//! ```json
//! {"body":null,"header":null,"request":{"elapsed":1,"final_url":"http://local.dev/path/to/form","method":"POST","name":"(Anon) post to form","redirected":false,"response_time":22,"status_code":404,"success":false,"update":false,"url":"http://local.dev/path/to/form","user":0},"tag":"POSTing [(\"field_1\", \"foo\"), (\"field_2\", \"bar\"), (\"op\", \"Save\")] on /path/to/form"}
//! ```
//!
//! For a more complex debug logging example, refer to the
//! [`log_debug`](../goose/struct.GooseUser.html#method.log_debug) documentation.
//!
//! ### Reducing File And Memory Usage
//!
//! The debug logger can result in a very large debug file, as by default it includes the
//! entire body of any pages returned that result in an error. This also requires allocating
//! a bigger [`BufWriter`](https://docs.rs/tokio/*/tokio/io/struct.BufWriter.html), and can
//! generate a lot of disk io.
//!
//! If you don't need to log response bodies, you can disable this functionality (and reduce
//! the amount of RAM required by the
//! [`BufWriter`](https://docs.rs/tokio/*/tokio/io/struct.BufWriter.html) by setting the
//! `--no-debug-body` command-line option, or the
//! [`GooseDefault::NoDebugBody`](../config/enum.GooseDefault.html#variant.NoDebugBody) default
//! configuration option. The debug logger will still record any custom messages, details
//! about the request (when available), and all server response headers (when available).

use regex::RegexSet;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::config::{GooseConfigure, GooseValue};
use crate::goose::GooseDebug;
use crate::metrics::{GooseErrorMetric, GooseRequestMetric, ScenarioMetric, TransactionMetric};
use crate::{GooseConfiguration, GooseDefaults, GooseError};

/// Optional unbounded receiver for logger thread, if debug logger is enabled.
pub(crate) type GooseLoggerJoinHandle =
    Option<tokio::task::JoinHandle<std::result::Result<(), GooseError>>>;
/// Optional unbounded sender from all GooseUsers to logger thread, if enabled.
pub(crate) type GooseLoggerTx = Option<flume::Sender<Option<GooseLog>>>;

/// Formats comma separated arguments into a csv row according to RFC 4180. Every argument has to be `Display`.
///
/// Specifically, this encloses all values with double quotes `"` which contain a comma, a quote or a new line.
/// Inner quotes are doubled according to RFC 4180 2.7.
/// The fields are joined by commas `,`, but *not* terminated with a line ending.
#[macro_export]
#[doc(hidden)]
macro_rules! format_csv_row {
    ($( $field:expr ),+ $(,)?) => {{
        [$( $field.to_string() ),*]
            .iter()
            .map(|s| {
                if s.contains('"') || s.contains(',') || s.contains('\n') {
                    // Enclose in quotes and escape inner quotes
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    // Because into_iter is not available in edition 2018
                    s.clone()
                }
            })
            .collect::<Vec<String>>()
            .join(",")
    }};
}
pub use format_csv_row;

/// If enabled, the logger thread can accept any of the following types of messages, and will
/// write them to the correct log file.
#[derive(Debug, Deserialize, Serialize)]
pub enum GooseLog {
    Debug(GooseDebug),
    Error(GooseErrorMetric),
    Request(GooseRequestMetric),
    Transaction(TransactionMetric),
    Scenario(ScenarioMetric),
}

/// Defines the formats logs can be written to file.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GooseLogFormat {
    Csv,
    Json,
    Raw,
    Pretty,
}
/// Allow setting log formats from the command line by impleenting [`FromStr`].
impl FromStr for GooseLogFormat {
    type Err = GooseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Use a [`RegexSet`] to match string representations of `GooseCoordinatedOmissionMitigation`,
        // returning the appropriate enum value. Also match a wide range of abbreviations and synonyms.
        let log_format = RegexSet::new(&[
            r"(?i)^csv$",
            r"(?i)^(json|jsn)$",
            r"(?i)^raw$",
            r"(?i)^pretty$",
        ])
        .expect("failed to compile log_format RegexSet");
        let matches = log_format.matches(s);
        if matches.matched(0) {
            Ok(GooseLogFormat::Csv)
        } else if matches.matched(1) {
            Ok(GooseLogFormat::Json)
        } else if matches.matched(2) {
            Ok(GooseLogFormat::Raw)
        } else if matches.matched(3) {
            Ok(GooseLogFormat::Pretty)
        } else {
            Err(GooseError::InvalidOption {
                option: format!("GooseLogFormat::{:?}", s),
                value: s.to_string(),
                detail: "Invalid log_format, expected: csv, json, or raw".to_string(),
            })
        }
    }
}

// @TODO this should be automatically derived from the structure.
fn debug_csv_header() -> String {
    format_csv_row!("tag", "request", "header", "body")
}

// @TODO this should be automatically derived from the structure.
fn error_csv_header() -> String {
    format_csv_row!(
        "elapsed",
        "raw",
        "name",
        "final_url",
        "redirected",
        "response_time",
        "status_code",
        "user",
        "error",
    )
}

// @TODO this should be automatically derived from the structure.
fn requests_csv_header() -> String {
    format_csv_row!(
        "elapsed",
        "raw",
        "name",
        "final_url",
        "redirected",
        "response_time",
        "status_code",
        "success",
        "update",
        "user",
        "error",
        "coordinated_omission_elapsed",
        "user_cadence",
    )
}

// @TODO this should be automatically derived from the structure.
fn transactions_csv_header() -> String {
    format_csv_row!(
        "elapsed",
        "scenario_index",
        "transaction_index",
        "name",
        "run_time",
        "success",
        "user",
    )
}

// @TODO this should be automatically derived from the structure.
fn scenarios_csv_header() -> String {
    format_csv_row!("elapsed", "name", "index", "run_time", "user",)
}

/// Two traits that must be implemented by all loggers provided through this thread.
pub(crate) trait GooseLogger<T> {
    /// Converts a rust structure to a formatted string.
    /// @TODO: rework with .to_string()
    fn format_message(&self, message: T) -> String;
}
/// Traits for GooseDebug logs.
impl GooseLogger<GooseDebug> for GooseConfiguration {
    /// Converts a GooseDebug structure to a formatted string.
    fn format_message(&self, message: GooseDebug) -> String {
        if let Some(debug_format) = self.debug_format.as_ref() {
            match debug_format {
                // Use serde_json to create JSON.
                GooseLogFormat::Json => json!(message).to_string(),
                // Raw format is Debug output for GooseRawRequest structure.
                GooseLogFormat::Raw => format!("{:?}", message),
                // Pretty format is Debug Pretty output for GooseRawRequest structure.
                GooseLogFormat::Pretty => format!("{:#?}", message),
                // Csv format with `,` separator and `"` quotes.
                GooseLogFormat::Csv => {
                    // @TODO: properly handle Option<>; flatten raw request in own columns
                    format_csv_row!(
                        message.tag,
                        format!("{:?}", message.request),
                        format!("{:?}", message.header),
                        format!("{:?}", message.body)
                    )
                }
            }
        } else {
            // A log format is required.
            unreachable!()
        }
    }
}
/// Traits for GooseErrorMetric logs.
impl GooseLogger<GooseErrorMetric> for GooseConfiguration {
    /// Converts a GooseErrorMetric structure to a formatted string.
    fn format_message(&self, message: GooseErrorMetric) -> String {
        if let Some(error_format) = self.error_format.as_ref() {
            match error_format {
                // Use serde_json to create JSON.
                GooseLogFormat::Json => json!(message).to_string(),
                // Raw format is Debug output for GooseErrorMetric structure.
                GooseLogFormat::Raw => format!("{:?}", message),
                // Pretty format is Debug Pretty output for GooseErrorMetric structure.
                GooseLogFormat::Pretty => format!("{:#?}", message),
                // Csv format with `,` separator and `"` quotes.
                GooseLogFormat::Csv => {
                    format_csv_row!(
                        message.elapsed,
                        format!("{:?}", message.raw),
                        message.name,
                        message.final_url,
                        message.redirected,
                        message.response_time,
                        message.status_code,
                        message.user,
                        message.error,
                    )
                }
            }
        } else {
            // A log format is required.
            unreachable!()
        }
    }
}
/// Traits for GooseRequestMetric logs.
impl GooseLogger<GooseRequestMetric> for GooseConfiguration {
    /// Converts a GooseRequestMetric structure to a formatted string.
    fn format_message(&self, message: GooseRequestMetric) -> String {
        if let Some(request_format) = self.request_format.as_ref() {
            match request_format {
                // Use serde_json to create JSON.
                GooseLogFormat::Json => json!(message).to_string(),
                // Raw format is Debug output for GooseRequestMetric structure.
                GooseLogFormat::Raw => format!("{:?}", message),
                // Pretty format is Debug Pretty output for GooseRequestMetric structure.
                GooseLogFormat::Pretty => format!("{:#?}", message),
                // Csv format with `,` separator and `"` quotes.
                GooseLogFormat::Csv => {
                    format_csv_row!(
                        message.elapsed,
                        format!("{:?}", message.raw),
                        message.name,
                        message.final_url,
                        message.redirected,
                        message.response_time,
                        message.status_code,
                        message.success,
                        message.update,
                        message.user,
                        message.error,
                        message.coordinated_omission_elapsed,
                        message.user_cadence,
                    )
                }
            }
        } else {
            // A log format is required.
            unreachable!()
        }
    }
}
/// Traits for TransactionMetric logs.
impl GooseLogger<TransactionMetric> for GooseConfiguration {
    /// Converts a TransactionMetric structure to a formatted string.
    fn format_message(&self, message: TransactionMetric) -> String {
        if let Some(transaction_format) = self.transaction_format.as_ref() {
            match transaction_format {
                // Use serde_json to create JSON.
                GooseLogFormat::Json => json!(message).to_string(),
                // Raw format is Debug output for TransactionMetric structure.
                GooseLogFormat::Raw => format!("{:?}", message),
                // Pretty format is Debug Pretty output for TransactionMetric structure.
                GooseLogFormat::Pretty => format!("{:#?}", message),
                // Csv format with `,` separator and `"` quotes.
                GooseLogFormat::Csv => {
                    format_csv_row!(
                        message.elapsed,
                        message.scenario_index,
                        message.transaction_index,
                        message.name,
                        message.run_time,
                        message.success,
                        message.user,
                    )
                }
            }
        } else {
            // A log format is required.
            unreachable!()
        }
    }
}

/// Traits for ScenarioMetric logs.
impl GooseLogger<ScenarioMetric> for GooseConfiguration {
    /// Converts a ScenarioMetric structure to a formatted string.
    fn format_message(&self, message: ScenarioMetric) -> String {
        if let Some(scenario_format) = self.scenario_format.as_ref() {
            match scenario_format {
                // Use serde_json to create JSON.
                GooseLogFormat::Json => json!(message).to_string(),
                // Raw format is Debug output for ScenarioMetric structure.
                GooseLogFormat::Raw => format!("{:?}", message),
                // Pretty format is Debug Pretty output for ScenarioMetric structure.
                GooseLogFormat::Pretty => format!("{:#?}", message),
                // Csv format with `,` separator and `"` quotes.
                GooseLogFormat::Csv => {
                    format_csv_row!(
                        message.elapsed,
                        message.name,
                        message.index,
                        message.run_time,
                        message.user,
                    )
                }
            }
        } else {
            // A log format is required.
            unreachable!()
        }
    }
}

/// Helpers to launch and control configured loggers.
impl GooseConfiguration {
    /// Makes sure the GooseConfiguration has any/all configured log files (loading from defaults
    /// if not configured through a run time option).
    pub(crate) fn configure_loggers(&mut self, defaults: &GooseDefaults) {
        // Configure debug_log path if enabled.
        self.debug_log = self
            .get_value(vec![
                // Use --debug-log if set.
                GooseValue {
                    value: Some(self.debug_log.to_string()),
                    filter: self.debug_log.is_empty(),
                    message: "",
                },
                // Otherwise use GooseDefault if set.
                GooseValue {
                    value: defaults.debug_log.clone(),
                    filter: defaults.debug_log.is_none(),
                    message: "",
                },
            ])
            .unwrap_or_else(|| "".to_string());

        // Set `debug_format`.
        self.debug_format = self.get_value(vec![
            // Use --debug-format if set.
            GooseValue {
                value: self.debug_format.clone(),
                filter: self.debug_format.is_none(),
                message: "",
            },
            // Otherwise use GooseDefault if set and not on Manager.
            GooseValue {
                value: defaults.debug_format.clone(),
                filter: defaults.debug_format.is_none() || self.manager,
                message: "",
            },
            // Otherwise default to GooseLogFormat::Json if not on Manager.
            GooseValue {
                value: Some(GooseLogFormat::Json),
                filter: self.manager,
                message: "",
            },
        ]);

        // Configure error_log path if enabled.
        self.error_log = self
            .get_value(vec![
                // Use --error-log if set.
                GooseValue {
                    value: Some(self.error_log.to_string()),
                    filter: self.error_log.is_empty(),
                    message: "",
                },
                // Otherwise use GooseDefault if set.
                GooseValue {
                    value: defaults.error_log.clone(),
                    filter: defaults.error_log.is_none(),
                    message: "",
                },
            ])
            .unwrap_or_else(|| "".to_string());

        // Set `error_format`.
        self.error_format = self.get_value(vec![
            // Use --error-format if set.
            GooseValue {
                value: self.error_format.clone(),
                filter: self.error_format.is_none(),
                message: "",
            },
            // Otherwise use GooseDefault if set and not on Manager.
            GooseValue {
                value: defaults.error_format.clone(),
                filter: defaults.error_format.is_none() || self.manager,
                message: "",
            },
            // Otherwise default to GooseLogFormat::Json if not on Manager.
            GooseValue {
                value: Some(GooseLogFormat::Json),
                filter: self.manager,
                message: "",
            },
        ]);

        // Configure request_log path if enabled.
        self.request_log = self
            .get_value(vec![
                // Use --request-log if set.
                GooseValue {
                    value: Some(self.request_log.to_string()),
                    filter: self.request_log.is_empty(),
                    message: "",
                },
                // Otherwise use GooseDefault if set.
                GooseValue {
                    value: defaults.request_log.clone(),
                    filter: defaults.request_log.is_none(),
                    message: "",
                },
            ])
            .unwrap_or_else(|| "".to_string());

        // Set `request_format`.
        self.request_format = self.get_value(vec![
            // Use --request-format if set.
            GooseValue {
                value: self.request_format.clone(),
                filter: self.request_format.is_none(),
                message: "",
            },
            // Otherwise use GooseDefault if set and not on Manager.
            GooseValue {
                value: defaults.request_format.clone(),
                filter: defaults.request_format.is_none() || self.manager,
                message: "",
            },
            // Otherwise default to GooseLogFormat::Json if not on Manager.
            GooseValue {
                value: Some(GooseLogFormat::Json),
                filter: self.manager,
                message: "",
            },
        ]);

        // Configure `request_body`.
        self.request_body = self
            .get_value(vec![
                // Use --request-body if set.
                GooseValue {
                    value: Some(self.request_body),
                    filter: !self.request_body,
                    message: "request_body",
                },
                // Otherwise use GooseDefault if set and not on Worker.
                GooseValue {
                    value: defaults.request_body,
                    filter: defaults.request_body.is_none() || self.manager,
                    message: "request_body",
                },
            ])
            .unwrap_or(false);

        // Configure transaction_log path if enabled.
        self.transaction_log = self
            .get_value(vec![
                // Use --transaction-log if set.
                GooseValue {
                    value: Some(self.transaction_log.to_string()),
                    filter: self.transaction_log.is_empty(),
                    message: "",
                },
                // Otherwise use GooseDefault if set.
                GooseValue {
                    value: defaults.transaction_log.clone(),
                    filter: defaults.transaction_log.is_none(),
                    message: "",
                },
            ])
            .unwrap_or_else(|| "".to_string());

        // Set `transaction_format`.
        self.transaction_format = self.get_value(vec![
            // Use --transaction-format if set.
            GooseValue {
                value: self.transaction_format.clone(),
                filter: self.transaction_format.is_none(),
                message: "",
            },
            // Otherwise use GooseDefault if set and not on Manager.
            GooseValue {
                value: defaults.transaction_format.clone(),
                filter: defaults.transaction_format.is_none() || self.manager,
                message: "",
            },
            // Otherwise default to GooseLogFormat::Json if not on Manager.
            GooseValue {
                value: Some(GooseLogFormat::Json),
                filter: self.manager,
                message: "",
            },
        ]);

        // Configure scenario_log path if enabled.
        self.scenario_log = self
            .get_value(vec![
                // Use --scenario-log if set.
                GooseValue {
                    value: Some(self.scenario_log.to_string()),
                    filter: self.scenario_log.is_empty(),
                    message: "",
                },
                // Otherwise use GooseDefault if set.
                GooseValue {
                    value: defaults.scenario_log.clone(),
                    filter: defaults.scenario_log.is_none(),
                    message: "",
                },
            ])
            .unwrap_or_else(|| "".to_string());

        // Set `scenario_format`.
        self.scenario_format = self.get_value(vec![
            // Use --scenario-format if set.
            GooseValue {
                value: self.scenario_format.clone(),
                filter: self.scenario_format.is_none(),
                message: "",
            },
            // Otherwise use GooseDefault if set and not on Manager.
            GooseValue {
                value: defaults.scenario_format.clone(),
                filter: defaults.scenario_format.is_none() || self.manager,
                message: "",
            },
            // Otherwise default to GooseLogFormat::Json if not on Manager.
            GooseValue {
                value: Some(GooseLogFormat::Json),
                filter: self.manager,
                message: "",
            },
        ]);
    }

    /// Spawns the logger thread if one or more loggers are enabled.
    pub(crate) async fn setup_loggers(
        &mut self,
        defaults: &GooseDefaults,
    ) -> Result<(GooseLoggerJoinHandle, GooseLoggerTx), GooseError> {
        // If running in Manager mode, no logger thread is started.
        if self.manager {
            return Ok((None, None));
        }

        // Update the logger configuration, loading defaults if necessasry.
        self.configure_loggers(defaults);

        // If no logger is enabled, return immediately without launching logger thread.
        if self.debug_log.is_empty()
            && self.request_log.is_empty()
            && self.transaction_log.is_empty()
            && self.scenario_log.is_empty()
            && self.error_log.is_empty()
        {
            return Ok((None, None));
        }

        // Create an unbounded channel allowing GooseUser threads to log errors.
        let (all_threads_logger_tx, logger_rx): (
            flume::Sender<Option<GooseLog>>,
            flume::Receiver<Option<GooseLog>>,
        ) = flume::unbounded();
        // Launch a new thread for logging.
        let configuration = self.clone();
        let logger_handle = tokio::spawn(async move { configuration.logger_main(logger_rx).await });
        Ok((Some(logger_handle), Some(all_threads_logger_tx)))
    }

    /// A helper used to open any/all log files, deleting any file that already exists.
    async fn open_log_file(
        &self,
        log_file_path: &str,
        log_file_type: &str,
        buffer_capacity: usize,
    ) -> std::option::Option<tokio::io::BufWriter<tokio::fs::File>> {
        if log_file_path.is_empty() {
            None
        } else {
            match File::create(log_file_path).await {
                Ok(f) => {
                    info!("writing {} to: {}", log_file_type, log_file_path);
                    Some(BufWriter::with_capacity(buffer_capacity, f))
                }
                Err(e) => {
                    error!(
                        "failed to create {} ({}): {}",
                        log_file_type, log_file_path, e
                    );
                    None
                }
            }
        }
    }

    /// Helper to write a line to the log file.
    async fn write_to_log_file(
        &self,
        log_file: &mut tokio::io::BufWriter<tokio::fs::File>,
        formatted_message: String,
    ) -> Result<(), ()> {
        match log_file
            .write(format!("{}\n", formatted_message).as_ref())
            .await
        {
            Ok(_) => (),
            Err(e) => {
                warn!("failed to write to {}: {}", &self.debug_log, e);
            }
        }

        Ok(())
    }

    /// Logger thread, opens a log file (if configured) and waits for messages from
    /// [`GooseUser`](../goose/struct.GooseUser.html) threads.
    pub(crate) async fn logger_main(
        self: GooseConfiguration,
        receiver: flume::Receiver<Option<GooseLog>>,
    ) -> Result<(), GooseError> {
        // If the debug_log is enabled, allocate a buffer and open the file.
        let mut debug_log = self
            .open_log_file(
                &self.debug_log,
                "debug file",
                if self.no_debug_body {
                    // Allocate a smaller 64K buffer if not logging response body.
                    64 * 1024
                } else {
                    // Allocate a larger 8M buffer if logging response body.
                    8 * 1024 * 1024
                },
            )
            .await;
        // If the debug_log is a CSV, write the header.
        if self.debug_format == Some(GooseLogFormat::Csv) {
            if let Some(log_file) = debug_log.as_mut() {
                // @TODO: error handling when writing to log fails.
                let _ = self.write_to_log_file(log_file, debug_csv_header()).await;
            }
        }

        // If the error_log is enabled, allocate a buffer and open the file.
        let mut error_log = self
            .open_log_file(&self.error_log, "error log", 64 * 1024)
            .await;
        // If the request_log is a CSV, write the header.
        if self.error_format == Some(GooseLogFormat::Csv) {
            if let Some(log_file) = error_log.as_mut() {
                // @TODO: error handling when writing to log fails.
                let _ = self.write_to_log_file(log_file, error_csv_header()).await;
            }
        }

        // If the request_log is enabled, allocate a buffer and open the file.
        let mut request_log = self
            .open_log_file(
                &self.request_log,
                "request log",
                if self.request_body {
                    // Allocate a larger 8M buffer if logging request body.
                    8 * 1024 * 1024
                } else {
                    // Allocate a smaller 64K buffer if not logging request body.
                    64 * 1024
                },
            )
            .await;
        // If the request_log is a CSV, write the header.
        if self.request_format == Some(GooseLogFormat::Csv) {
            if let Some(log_file) = request_log.as_mut() {
                // @TODO: error handling when writing to log fails.
                let _ = self
                    .write_to_log_file(log_file, requests_csv_header())
                    .await;
            }
        }

        // If the transaction_log is enabled, allocate a buffer and open the file.
        let mut transaction_log = self
            .open_log_file(&self.transaction_log, "transaction log", 64 * 1024)
            .await;
        // If the transaction_log is a CSV, write the header.
        if self.transaction_format == Some(GooseLogFormat::Csv) {
            if let Some(log_file) = transaction_log.as_mut() {
                // @TODO: error handling when writing to log fails.
                let _ = self
                    .write_to_log_file(log_file, transactions_csv_header())
                    .await;
            }
        }

        // If the scenario_log is enabled, allocate a buffer and open the file.
        let mut scenario_log = self
            .open_log_file(&self.scenario_log, "scenario log", 64 * 1024)
            .await;
        // If the scenario_log is a CSV, write the header.
        if self.scenario_format == Some(GooseLogFormat::Csv) {
            if let Some(log_file) = scenario_log.as_mut() {
                // @TODO: error handling when writing to log fails.
                let _ = self
                    .write_to_log_file(log_file, scenarios_csv_header())
                    .await;
            }
        }

        // Loop waiting for and writing error logs from GooseUser threads.
        while let Ok(received_message) = receiver.recv_async().await {
            if let Some(message) = received_message {
                let formatted_message;
                if let Some(log_file) = match message {
                    GooseLog::Debug(debug_message) => {
                        formatted_message = self.format_message(debug_message).to_string();
                        debug_log.as_mut()
                    }
                    GooseLog::Error(error_message) => {
                        formatted_message = self.format_message(error_message).to_string();
                        error_log.as_mut()
                    }
                    GooseLog::Request(request_message) => {
                        formatted_message = self.format_message(request_message).to_string();
                        request_log.as_mut()
                    }
                    GooseLog::Transaction(transaction_message) => {
                        formatted_message = self.format_message(transaction_message).to_string();
                        transaction_log.as_mut()
                    }
                    GooseLog::Scenario(scenario_message) => {
                        formatted_message = self.format_message(scenario_message).to_string();
                        scenario_log.as_mut()
                    }
                } {
                    // @TODO: error handling when writing to log fails.
                    let _ = self.write_to_log_file(log_file, formatted_message).await;
                }
            } else {
                // Empty message means it's time to exit.
                break;
            }
        }

        // Flush debug logs to disk if enabled.
        if let Some(debug_log_file) = debug_log.as_mut() {
            info!("flushing debug_log: {}", &self.debug_log);
            let _ = debug_log_file.flush().await;
        };

        // Flush requests log to disk if enabled.
        if let Some(requests_log_file) = request_log.as_mut() {
            info!("flushing request_log: {}", &self.request_log);
            let _ = requests_log_file.flush().await;
        }

        // Flush transaction log to disk if enabled.
        if let Some(transactions_log_file) = transaction_log.as_mut() {
            info!("flushing transaction_log: {}", &self.transaction_log);
            let _ = transactions_log_file.flush().await;
        }

        // Flush scenario log to disk if enabled.
        if let Some(scenarios_log_file) = scenario_log.as_mut() {
            info!("flushing scenario: {}", &self.scenario_log);
            let _ = scenarios_log_file.flush().await;
        }

        // Flush error logs to disk if enabled.
        if let Some(error_log_file) = error_log.as_mut() {
            info!("flushing error_log: {}", &self.error_log);
            let _ = error_log_file.flush().await;
        };

        Ok(())
    }
}
