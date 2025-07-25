use crate::workspace::{CheckFileSizeResult, DocumentFileSource};
use biome_analyze::RuleError;
use biome_configuration::diagnostics::{
    CantResolve, ConfigurationDiagnostic, EditorConfigDiagnostic,
};
use biome_configuration::{BiomeDiagnostic, CantLoadExtendFile};
use biome_console::fmt::Bytes;
use biome_console::markup;
use biome_css_parser::ParseDiagnostic;
use biome_diagnostics::{
    Advices, Category, Diagnostic, DiagnosticTags, Location, LogCategory, MessageAndDescription,
    Severity, Visit, category,
};
use biome_formatter::{FormatError, PrintError};
use biome_fs::{BiomePath, FileSystemDiagnostic};
use biome_grit_patterns::CompileError;
use biome_js_analyze::utils::rename::RenameError;
use biome_plugin_loader::PluginDiagnostic;
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::process::{ExitCode, Termination};

/// Generic errors thrown during biome operations
#[derive(Deserialize, Diagnostic, Serialize)]
pub enum WorkspaceError {
    /// Thrown when Biome can't read a generic file.
    CantReadFile(CantReadFile),

    /// Error thrown when validating the configuration. Once deserialized,
    /// further checks have to be done.
    Configuration(ConfigurationDiagnostic),

    /// Emitted when the file is ignored and should not be processed.
    FileIgnored(FileIgnored),

    /// Diagnostics emitted when querying the file system.
    FileSystem(FileSystemDiagnostic),

    /// The formatter encountered an error while formatting the file.
    FormatError(FormatError),

    /// The file could not be formatted since it has syntax errors and
    /// `format_with_errors` is disabled.
    FormatWithErrorsDisabled(FormatWithErrorsDisabled),

    /// An operation is attempted on the registered project, but there is no
    /// registered project.
    NoProject(NoProject),

    /// Thrown when the workspace attempts to register a nested project, but no
    /// working directory was provided.
    NoWorkspaceDirectory(NoWorkspaceDirectory),

    /// Path contained a non-UTF8 character.
    NonUtf8Path(NonUtf8Path),

    /// The file does not exist in the [crate::Workspace].
    NotFound(NotFound),

    /// One or more errors occurred during plugin loading.
    PluginErrors(PluginErrors),

    /// The formatter encountered an error while formatting the file.
    PrintError(PrintError),

    /// Diagnostic raised when a file is protected.
    ProtectedFile(ProtectedFile),

    /// Error thrown when Biome cannot rename a symbol.
    RenameError(RenameError),

    /// The file could not be analyzed because a rule caused an error.
    RuleError(RuleError),

    /// Error when searching for a pattern
    SearchError(SearchError),

    /// A file is not supported. Contains the language and path of the file.
    ///
    /// Use this error if Biome is trying to process a file that Biome can't
    /// understand.
    SourceFileNotSupported(SourceFileNotSupported),

    /// Error emitted by the underlying transport layer for a remote Workspace.
    TransportError(TransportError),

    /// Raised when there's an issue around the VCS integration.
    Vcs(VcsDiagnostic),

    /// Error in the workspace watcher.
    WatchError(WatchError),
}

impl WorkspaceError {
    pub fn format_with_errors_disabled() -> Self {
        Self::FormatWithErrorsDisabled(FormatWithErrorsDisabled)
    }

    pub fn cant_read_file(path: String) -> Self {
        Self::CantReadFile(CantReadFile { path })
    }

    pub fn invalid_pattern() -> Self {
        Self::SearchError(SearchError::InvalidPattern(InvalidPattern))
    }

    pub fn non_utf8_path(path: OsString) -> Self {
        Self::NonUtf8Path(NonUtf8Path {
            path: path.display().to_string(),
        })
    }

    pub fn not_found() -> Self {
        Self::NotFound(NotFound)
    }

    #[inline]
    pub fn no_project() -> Self {
        Self::NoProject(NoProject)
    }

    pub fn no_workspace_directory() -> Self {
        Self::NoWorkspaceDirectory(NoWorkspaceDirectory)
    }

    pub fn file_ignored(path: String) -> Self {
        Self::FileIgnored(FileIgnored { path })
    }

    pub fn source_file_not_supported(
        language: DocumentFileSource,
        path: String,
        extension: Option<String>,
    ) -> Self {
        Self::SourceFileNotSupported(SourceFileNotSupported {
            file_source: language,
            path,
            extension,
        })
    }

    pub fn plugin_errors(diagnostics: Vec<PluginDiagnostic>) -> Self {
        Self::PluginErrors(PluginErrors { diagnostics })
    }

    pub fn vcs_disabled() -> Self {
        Self::Vcs(VcsDiagnostic::DisabledVcs(DisabledVcs {}))
    }

    pub fn protected_file(file_path: impl Into<String>) -> Self {
        Self::ProtectedFile(ProtectedFile {
            file_path: file_path.into(),
            verbose_advice: ProtectedFileAdvice,
        })
    }

    pub fn is_editor_config_error(&self) -> bool {
        matches!(
            self,
            Self::Configuration(ConfigurationDiagnostic::EditorConfig(_))
        )
    }
}

impl Error for WorkspaceError {}

impl Debug for WorkspaceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl Display for WorkspaceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Diagnostic::description(self, f)
    }
}

impl Termination for WorkspaceError {
    fn report(self) -> ExitCode {
        ExitCode::FAILURE
    }
}

impl From<FormatError> for WorkspaceError {
    fn from(err: FormatError) -> Self {
        Self::FormatError(err)
    }
}

impl From<TransportError> for WorkspaceError {
    fn from(err: TransportError) -> Self {
        Self::TransportError(err)
    }
}

impl From<PrintError> for WorkspaceError {
    fn from(err: PrintError) -> Self {
        Self::PrintError(err)
    }
}

impl From<FileSystemDiagnostic> for WorkspaceError {
    fn from(err: FileSystemDiagnostic) -> Self {
        Self::FileSystem(err)
    }
}

impl From<BiomeDiagnostic> for WorkspaceError {
    fn from(value: BiomeDiagnostic) -> Self {
        Self::Configuration(value.into())
    }
}

impl From<EditorConfigDiagnostic> for WorkspaceError {
    fn from(value: EditorConfigDiagnostic) -> Self {
        Self::Configuration(value.into())
    }
}

impl From<CantLoadExtendFile> for WorkspaceError {
    fn from(value: CantLoadExtendFile) -> Self {
        Self::Configuration(BiomeDiagnostic::CantLoadExtendFile(value).into())
    }
}

impl From<CantResolve> for WorkspaceError {
    fn from(value: CantResolve) -> Self {
        Self::Configuration(BiomeDiagnostic::CantResolve(value).into())
    }
}

impl From<WorkspaceError> for biome_diagnostics::serde::Diagnostic {
    fn from(error: WorkspaceError) -> Self {
        Self::new(error)
    }
}

#[derive(Debug, Deserialize, Diagnostic, Serialize)]
#[diagnostic(
     category = "internalError/fs",
     severity = Error,
     message(
         description = "Biome does not support non-UTF8 characters in path: {path}",
         message("Biome does not support non-UTF8 characters in path: "<Emphasis>{self.path}</Emphasis>)
     ),
 )]
pub struct NonUtf8Path {
    #[location(resource)]
    path: String,
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    category = "internalError/fs",
    message = "The file does not exist in the workspace.",
    tags(INTERNAL)
)]
pub struct NotFound;

#[derive(Debug, Diagnostic, Deserialize, Serialize)]
#[diagnostic(category = "internalError/panic", severity = Fatal, tags(INTERNAL))]
pub struct Panic {
    #[message]
    #[description]
    message: MessageAndDescription,
}

impl From<Panic> for biome_diagnostics::serde::Diagnostic {
    fn from(error: Panic) -> Self {
        Self::new(error)
    }
}

impl Panic {
    pub fn with_file(path: &Utf8Path) -> Self {
        Self {
            message: MessageAndDescription::from(
                markup! {
                    "A task panicked while processing "<Emphasis>{path.to_string()}</Emphasis>
                }
                .to_owned(),
            ),
        }
    }

    pub fn with_file_and_message(path: &Utf8Path, message: impl Into<String>) -> Self {
        Self {
            message: MessageAndDescription::from(
                markup! {
                    "A task panicked while processing "<Emphasis>{path.to_string()}</Emphasis>": "{message.into()}
                }
                .to_owned(),
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    category = "format",
    message = "Code formatting aborted due to parsing errors. To format code with errors, enable the 'formatter.formatWithErrors' option."
)]
pub struct FormatWithErrorsDisabled;

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    category = "internalError/fs",
    message(
        message("Biome couldn't read the following file, maybe for permissions reasons or it doesn't exist: "{self.path}),
        description = "Biome couldn't read the following file, maybe for permissions reasons or it doesn't exist: {path}"
    )
)]
pub struct CantReadFile {
    #[location(resource)]
    path: String,
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    category = "internalError/fs",
    message(
        message("The file "{self.path}" was ignored."),
        description = "The file {path} was ignored."
    ),
    severity = Warning,
)]
pub struct FileIgnored {
    #[location(resource)]
    path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileTooLarge {
    pub size: usize,
    pub limit: usize,
}

impl From<CheckFileSizeResult> for FileTooLarge {
    fn from(value: CheckFileSizeResult) -> Self {
        Self {
            size: value.file_size,
            limit: value.limit,
        }
    }
}

impl Diagnostic for FileTooLarge {
    fn severity(&self) -> Severity {
        Severity::Information
    }

    fn message(&self, fmt: &mut biome_console::fmt::Formatter<'_>) -> std::io::Result<()> {
        fmt.write_markup(
            markup!{
                "The size of the file is "{Bytes(self.size)}", which exceeds the configured maximum of "{Bytes(self.limit)}" for this project.
Use the `files.maxSize` configuration to change the maximum size of files processed, or `files.includes` to ignore the file."
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    category = "project",
    message(
        message(
            "Biome attempted to perform an operation on the registered project, but no project was registered. This is a bug in Biome. If this problem persists, please report here: https://github.com/biomejs/biome/issues/"
        ),
        description = "Biome attempted to perform an operation on the registered project, but no project was registered. This is a bug in Biome. If this problem persists, please report here: https://github.com/biomejs/biome/issues/"
    )
)]
pub struct NoProject;

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(category = "project")]
pub struct NoWorkspaceDirectory;

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceFileNotSupported {
    file_source: DocumentFileSource,
    path: String,
    extension: Option<String>,
}

impl Diagnostic for SourceFileNotSupported {
    fn category(&self) -> Option<&'static Category> {
        Some(category!("files/missingHandler"))
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn location(&self) -> Location<'_> {
        Location::builder().resource(&self.path).build()
    }

    fn message(&self, fmt: &mut biome_console::fmt::Formatter<'_>) -> std::io::Result<()> {
        if self.file_source != DocumentFileSource::Unknown {
            fmt.write_markup(markup! {
                "Biome doesn't support this feature for the language "<Emphasis>{{&self.file_source}}</Emphasis>
            })
        } else if let Some(ext) = self.extension.as_ref() {
            fmt.write_markup(markup! {
                "Biome could not determine the language for the file extension "<Emphasis>{{ext}}</Emphasis>
            })
        } else {
            fmt.write_markup(
                markup!{
                    "Biome could not determine the language for the file"<Emphasis>{self.path}</Emphasis>" because it doesn't have a clear extension"
                }
            )
        }
    }

    fn description(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        if self.file_source != DocumentFileSource::Unknown {
            write!(
                fmt,
                "Biome doesn't support this feature for the language {}",
                &self.file_source
            )
        } else if let Some(ext) = self.extension.as_ref() {
            write!(
                fmt,
                "Biome could not determine the language for the file extension {ext}"
            )
        } else {
            write!(
                fmt,
                "Biome could not determine the language for the file {} because it doesn't have a clear extension",
                &self.path
            )
        }
    }

    fn verbose_advices(&self, visitor: &mut dyn Visit) -> std::io::Result<()> {
        visitor.record_log(
            LogCategory::Info,
            &markup! {
                "If you want to turn off this diagnostic, consider using "<Emphasis>"--files-ignore-unknown"</Emphasis>" from the CLI, or "<Emphasis>"files.ignoreUnknown"</Emphasis>" from the configuration file."
            }
        )
    }
}

#[derive(Debug, Deserialize, Diagnostic, Serialize)]
pub enum SearchError {
    /// An invalid pattern was given
    PatternCompilationError(PatternCompilationError),
    /// No pattern with the given ID
    InvalidPattern(InvalidPattern),
    /// Error while executing the search query.
    QueryError(QueryDiagnostic),
}

#[derive(Debug, Deserialize, Diagnostic, Serialize)]
pub struct PatternCompilationError {
    #[message]
    #[description]
    message: MessageAndDescription,
}

impl From<ParseDiagnostic> for PatternCompilationError {
    fn from(diagnostic: ParseDiagnostic) -> Self {
        Self {
            message: diagnostic.message,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    category = "search",
    message(
        message("Invalid pattern -- this is a bug in Biome."),
        description = "If this problem persists, please report here: https://github.com/biomejs/biome/issues/"
    )
)]
pub struct InvalidPattern;

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryDiagnostic(pub String);

impl Diagnostic for QueryDiagnostic {
    fn category(&self) -> Option<&'static Category> {
        Some(category!("search"))
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn message(&self, fmt: &mut biome_console::fmt::Formatter<'_>) -> std::io::Result<()> {
        fmt.write_str("Error executing the Grit query")
    }

    fn description(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str(&self.0)
    }

    fn verbose_advices(&self, visitor: &mut dyn Visit) -> std::io::Result<()> {
        visitor.record_log(
            LogCategory::Info,
            &markup! { "Please consult "<Hyperlink href="https://biomejs.dev/reference/gritql">"our GritQL reference"</Hyperlink>"." }
        )
    }
}

pub fn extension_error(path: &BiomePath) -> WorkspaceError {
    let file_source = DocumentFileSource::from_path(path);
    WorkspaceError::source_file_not_supported(
        file_source,
        path.clone().to_string(),
        path.clone().extension().map(|p| p.to_string()),
    )
}

#[derive(Debug, Serialize, Deserialize)]
/// Error emitted by the underlying transport layer for a remote Workspace
pub enum TransportError {
    /// Error emitted by the transport layer if the connection was lost due to an I/O error
    ChannelClosed,
    /// Error emitted by the transport layer if a request timed out
    Timeout,
    /// Error caused by a serialization or deserialization issue
    SerdeError(String),
    /// Generic error type for RPC errors that can't be deserialized into RomeError
    RPCError(String),
}

impl Display for TransportError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.description(fmt)
    }
}

impl Diagnostic for TransportError {
    fn category(&self) -> Option<&'static Category> {
        Some(category!("internalError/io"))
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn description(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerdeError(err) => write!(fmt, "serialization error: {err}"),
            Self::ChannelClosed => fmt.write_str(
                "a request to the remote workspace failed because the connection was interrupted",
            ),
            Self::Timeout => fmt.write_str("the request to the remote workspace timed out"),
            Self::RPCError(err) => fmt.write_str(err),
        }
    }

    fn message(&self, fmt: &mut biome_console::fmt::Formatter<'_>) -> std::io::Result<()> {
        match self {
            Self::SerdeError(err) => write!(fmt, "serialization error: {err}"),
            Self::ChannelClosed => fmt.write_str(
                "a request to the remote workspace failed because the connection was interrupted",
            ),
            Self::Timeout => fmt.write_str("the request to the remote workspace timed out"),
            Self::RPCError(err) => fmt.write_str(err),
        }
    }
    fn tags(&self) -> DiagnosticTags {
        DiagnosticTags::INTERNAL
    }
}

#[derive(Debug, Deserialize, Diagnostic, Serialize)]
pub enum VcsDiagnostic {
    /// When the VCS ignore file can't be found
    NoIgnoreFileFound(NoIgnoreFileFound),
    /// VCS is disabled
    DisabledVcs(DisabledVcs),
}

impl From<VcsDiagnostic> for WorkspaceError {
    fn from(value: VcsDiagnostic) -> Self {
        Self::Vcs(value)
    }
}

impl From<CompileError> for WorkspaceError {
    fn from(value: CompileError) -> Self {
        match value {
            CompileError::ParsePatternError(diagnostic) => Self::SearchError(
                SearchError::PatternCompilationError(PatternCompilationError::from(diagnostic)),
            ),
            // FIXME: This really needs proper diagnostics
            _ => Self::SearchError(SearchError::QueryError(QueryDiagnostic(format!(
                "{value:?}"
            )))),
        }
    }
}

#[derive(Debug, Diagnostic, Serialize, Deserialize)]
#[diagnostic(
    category = "internalError/fs",
    severity = Error,
    message(
        description = "Biome couldn't find an ignore file in the following folder: {path}",
        message("Biome couldn't find an ignore file in the following folder: "<Emphasis>{self.path}</Emphasis>),
    )
)]
pub struct NoIgnoreFileFound {
    #[location(resource)]
    pub path: String,
}

#[derive(Debug, Diagnostic, Serialize, Deserialize)]
#[diagnostic(
    category = "internalError/fs",
    severity = Warning,
    message = "Biome couldn't determine a folder for the VCS integration. VCS integration will be disabled."
)]
pub struct DisabledVcs {}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginErrors {
    diagnostics: Vec<PluginDiagnostic>,
}

impl Diagnostic for PluginErrors {
    fn category(&self) -> Option<&'static Category> {
        Some(category!("plugin"))
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn message(&self, fmt: &mut biome_console::fmt::Formatter<'_>) -> std::io::Result<()> {
        fmt.write_markup(markup!("Error(s) during loading of plugins:\n"))?;

        for diagnostic in &self.diagnostics {
            diagnostic.message(fmt)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    category = "project",
    severity = Information,
    message(
        message("The file "<Emphasis>{self.file_path}</Emphasis>" is protected because is handled by another tool. Biome won't process it."),
        description = "The file {file_path} is protected because is handled by another tool. Biome won't process it.",
    ),
    tags(VERBOSE)
)]
pub struct ProtectedFile {
    #[location(resource)]
    pub file_path: String,

    #[verbose_advice]
    pub verbose_advice: ProtectedFileAdvice,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtectedFileAdvice;

impl Advices for ProtectedFileAdvice {
    fn record(&self, visitor: &mut dyn Visit) -> std::io::Result<()> {
        visitor.record_log(LogCategory::Info, &markup! { "You can hide this diagnostic by using "<Emphasis>"--diagnostic-level=warn"</Emphasis>" to increase the diagnostic level shown by CLI." })
    }
}

#[derive(Debug, Deserialize, Diagnostic, Serialize)]
#[diagnostic(
    category = "project",
    severity = Error,
    message(
        message("Biome cannot watch files on disk: "<Info>{self.reason}</Info>),
        description = "Biome cannot watch files on disk: {reason}",
    ),
)]
pub struct WatchError {
    pub reason: String,
}

#[cfg(test)]
mod test {
    use crate::diagnostics::{CantReadFile, FileIgnored, NotFound, SourceFileNotSupported};
    use crate::file_handlers::DocumentFileSource;
    use crate::{TransportError, WorkspaceError};
    use biome_diagnostics::{DiagnosticExt, Error, print_diagnostic_to_string};
    use biome_formatter::FormatError;
    use biome_fs::BiomePath;
    use std::ffi::OsString;

    fn snap_diagnostic(test_name: &str, diagnostic: Error) {
        let content = print_diagnostic_to_string(&diagnostic);

        insta::with_settings!({
            prepend_module_to_snapshot => false,
        }, {
            insta::assert_snapshot!(test_name, content);

        });
    }

    #[test]
    fn diagnostic_size() {
        assert_eq!(std::mem::size_of::<WorkspaceError>(), 96)
    }

    #[test]
    fn file_ignored() {
        snap_diagnostic(
            "file_ignored",
            WorkspaceError::FileIgnored(FileIgnored {
                path: "example.js".to_string(),
            })
            .with_file_path("example.js"),
        )
    }

    #[test]
    fn cant_read_file() {
        snap_diagnostic(
            "cant_read_file",
            WorkspaceError::CantReadFile(CantReadFile {
                path: "example.js".to_string(),
            })
            .with_file_path("example.js"),
        )
    }

    #[test]
    fn not_found() {
        snap_diagnostic(
            "not_found",
            WorkspaceError::NotFound(NotFound).with_file_path("not_found.js"),
        )
    }

    #[test]
    fn source_file_not_supported() {
        let path = BiomePath::new("not_supported.toml");
        snap_diagnostic(
            "source_file_not_supported",
            WorkspaceError::SourceFileNotSupported(SourceFileNotSupported {
                file_source: DocumentFileSource::Unknown,
                path: path.to_string(),
                extension: path.extension().map(str::to_string),
            })
            .with_file_path("not_supported.toml"),
        )
    }

    #[test]
    fn transport_channel_closed() {
        snap_diagnostic(
            "transport_channel_closed",
            TransportError::ChannelClosed.into(),
        )
    }

    #[test]
    fn transport_timeout() {
        snap_diagnostic("transport_timeout", TransportError::Timeout.into())
    }

    #[test]
    fn transport_rpc_error() {
        snap_diagnostic(
            "transport_rpc_error",
            TransportError::RPCError("Some generic error".to_string()).into(),
        )
    }

    #[test]
    fn transport_serde_error() {
        snap_diagnostic(
            "transport_serde_error",
            TransportError::SerdeError("Some serialization/deserialization error".to_string())
                .into(),
        )
    }

    #[test]
    fn formatter_syntax_error() {
        snap_diagnostic(
            "formatter_syntax_error",
            WorkspaceError::FormatError(FormatError::SyntaxError).with_file_path("example.js"),
        )
    }

    #[test]
    fn non_utf8_path() {
        snap_diagnostic(
            "non_utf8_path",
            Error::from(WorkspaceError::non_utf8_path(OsString::from("path.js"))),
        )
    }
}
