use crate::{source_map::CodespanSourceMap, ByteIndex, FileId};
use codespan_reporting as csr;
use codespan_reporting::diagnostic as csr_diag;
use codespan_reporting::term::termcolor;
use std::slice;

pub type Severity = libc::size_t;
pub const SEVERITY_HELP: Severity = 0;
pub const SEVERITY_NOTE: Severity = 1;
pub const SEVERITY_WARNING: Severity = 2;
pub const SEVERITY_ERROR: Severity = 3;
pub const SEVERITY_BUG: Severity = 4;

pub type DisplayStyle = libc::size_t;
pub const DISPLAY_STYLE_RICH: DisplayStyle = 0;
pub const DISPLAY_STYLE_MEDIUM: DisplayStyle = 1;
pub const DISPLAY_STYLE_SHORT: DisplayStyle = 2;

pub type CharStyle = libc::size_t;
pub const CHAR_STYLE_FANCY: CharStyle = 0;
pub const CHAR_STYLE_ASCII: CharStyle = 1;

pub type WriterCallback = unsafe extern "C" fn(
    user_data: *mut libc::c_void,
    utf8_output: *const u8,
    output_length: libc::size_t,
);

#[repr(C)]
pub struct CodespanDiagnostic {
    diagnostic: csr_diag::Diagnostic<FileId>,
    config: csr::term::Config,
    writer: WriterCallback,
}

impl CodespanDiagnostic {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_new_diagnostic(
        severity: Severity,
        message: *const u8,
        message_length: libc::size_t,
        writer: Option<WriterCallback>,
    ) -> *mut Self {
        let diagnostic = Box::new(Self {
            diagnostic: csr_diag::Diagnostic::<FileId>::new(match severity {
                SEVERITY_HELP => csr_diag::Severity::Help,
                SEVERITY_NOTE => csr_diag::Severity::Note,
                SEVERITY_WARNING => csr_diag::Severity::Warning,
                SEVERITY_ERROR => csr_diag::Severity::Error,
                SEVERITY_BUG => csr_diag::Severity::Bug,
                _ => csr_diag::Severity::Error,
            })
            .with_message(Self::utf8_to_string(message, message_length)),
            config: csr::term::Config::default(),
            writer: writer.unwrap(),
        });

        Box::into_raw(diagnostic)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_delete_diagnostic(diagnostic: *mut Self) {
        if !diagnostic.is_null() {
            unsafe { drop(Box::from_raw(diagnostic)) }
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_write_diagnostic(
        user_data: *mut libc::c_void,
        diagnostic: *const Self,
        source_map: *const CodespanSourceMap,
        color: u8,
    ) {
        if !diagnostic.is_null() && !source_map.is_null() {
            let diagnostic = unsafe { &*diagnostic };
            let source_map = unsafe { &*source_map };

            let mut buffer_stream = if color == 0 {
                termcolor::Buffer::no_color()
            } else {
                termcolor::Buffer::ansi()
            };

            csr::term::emit_to_write_style(
                &mut buffer_stream,
                &diagnostic.config,
                source_map,
                &diagnostic.diagnostic,
            )
            .unwrap();

            let utf8_output = buffer_stream.into_inner();
            unsafe { (diagnostic.writer)(user_data, utf8_output.as_ptr(), utf8_output.len()) };
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_diagnostic_set_code(
        diagnostic: *mut Self,
        code: *const u8,
        code_length: libc::size_t,
    ) {
        let diagnostic = unsafe { &mut (*diagnostic).diagnostic };
        diagnostic.code = Some(Self::utf8_to_string(code, code_length));
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_diagnostic_set_primary(
        diagnostic: *mut Self,
        file_id: FileId,
        start: ByteIndex,
        end: ByteIndex,
        message: *const u8,
        message_length: libc::size_t,
    ) {
        if !diagnostic.is_null() {
            let diagnostic = unsafe { &mut (*diagnostic).diagnostic };
            diagnostic.labels.push(
                csr_diag::Label::primary(file_id, start..end)
                    .with_message(Self::utf8_to_string(message, message_length)),
            );
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_diagnostic_add_secondary(
        diagnostic: *mut Self,
        file_id: FileId,
        start: ByteIndex,
        end: ByteIndex,
        message: *const u8,
        message_length: libc::size_t,
    ) {
        if !diagnostic.is_null() {
            let diagnostic = unsafe { &mut (*diagnostic).diagnostic };
            diagnostic.labels.push(
                csr_diag::Label::secondary(file_id, start..end)
                    .with_message(Self::utf8_to_string(message, message_length)),
            )
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_diagnostic_add_note(
        diagnostic: *mut Self,
        message: *const u8,
        message_length: libc::size_t,
    ) {
        if !diagnostic.is_null() {
            let diagnostic = unsafe { &mut (*diagnostic).diagnostic };
            diagnostic
                .notes
                .push(Self::utf8_to_string(message, message_length));
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_diagnostic_set_config(
        diagnostic: *mut Self,
        style: DisplayStyle,
        chars: CharStyle,
        tab_width: libc::size_t,
    ) {
        if !diagnostic.is_null() {
            let diagnostic = unsafe { &mut (*diagnostic) };
            diagnostic.config.display_style = match style {
                DISPLAY_STYLE_SHORT => csr::term::DisplayStyle::Short,
                DISPLAY_STYLE_MEDIUM => csr::term::DisplayStyle::Medium,
                DISPLAY_STYLE_RICH => csr::term::DisplayStyle::Rich,
                _ => csr::term::DisplayStyle::Rich,
            };
            diagnostic.config.chars = match chars {
                CHAR_STYLE_ASCII => csr::term::Chars::ascii(),
                CHAR_STYLE_FANCY => csr::term::Chars::default(),
                _ => csr::term::Chars::ascii(),
            };
            diagnostic.config.tab_width = tab_width;
        }
    }

    fn utf8_to_string(utf8_data: *const u8, data_length: libc::size_t) -> String {
        if utf8_data.is_null() {
            String::new()
        } else {
            str::from_utf8(unsafe { slice::from_raw_parts(utf8_data, data_length) })
                .unwrap()
                .to_owned()
        }
    }
}
