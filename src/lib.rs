#![feature(bool_to_result, vec_into_raw_parts)]
pub mod diagnostic;
pub mod source_map;

pub type FileId = libc::size_t;
pub type ByteIndex = libc::size_t;

#[cfg(test)]
mod tests {
    use std::{ptr, slice};

    use super::*;
    use crate::diagnostic::{
        CodespanDiagnostic, CHAR_STYLE_FANCY, DISPLAY_STYLE_RICH, SEVERITY_ERROR,
    };
    use crate::source_map::{CodespanSourceMap, LineIndex};
    use codespan_reporting as csr;
    use codespan_reporting::diagnostic as csr_diag;
    use codespan_reporting::files::{Files, SimpleFiles};
    use unindent::unindent;

    type SimpleMap = SimpleFiles<String, String>;

    unsafe extern "C" fn file_name(
        user_data: *mut libc::c_void,
        file_id: FileId,
        file_name_length: *mut libc::size_t,
    ) -> *const u8 {
        let files = unsafe { &*(user_data as *const SimpleMap) };
        let file = files.get(file_id).unwrap();
        let name_bytes = file.name().as_bytes();
        unsafe {
            *file_name_length = name_bytes.len();
        }
        name_bytes.as_ptr()
    }

    unsafe extern "C" fn source_code(
        user_data: *mut libc::c_void,
        file_id: FileId,
        source_code_length: *mut libc::size_t,
    ) -> *const u8 {
        let files = unsafe { &*(user_data as *const SimpleMap) };
        let file = files.get(file_id).unwrap();
        let source_bytes = file.source().as_bytes();
        unsafe {
            *source_code_length = source_bytes.len();
        }
        source_bytes.as_ptr()
    }

    unsafe extern "C" fn line_index(
        user_data: *mut libc::c_void,
        file_id: FileId,
        byte_index: ByteIndex,
    ) -> LineIndex {
        let files = unsafe { &*(user_data as *const SimpleMap) };
        files.line_index(file_id, byte_index).unwrap()
    }

    unsafe extern "C" fn line_range(
        user_data: *mut libc::c_void,
        file_id: FileId,
        line_index: LineIndex,
        start: *mut libc::size_t,
        end: *mut libc::size_t,
    ) {
        let files = unsafe { &*(user_data as *const SimpleMap) };
        let range = files.line_range(file_id, line_index).unwrap();
        unsafe {
            *start = range.start;
            *end = range.end;
        }
    }

    #[test]
    fn simple_test() {
        unsafe {
            backtrace_on_stack_overflow::enable();
        }

        let mut simple_map = SimpleMap::new();

        let file_id = simple_map.add(
            "FizzBuzz.fun".to_owned(),
            unindent(
                r#"
            module FizzBuzz where

            fizz₁ : Nat → String
            fizz₁ num = case (mod num 5) (mod num 3) of
                0 0 => "FizzBuzz"
                0 _ => "Fizz"
                _ 0 => "Buzz"
                _ _ => num

            fizz₂ : Nat → String
            fizz₂ num =
                case (mod num 5) (mod num 3) of
                    0 0 => "FizzBuzz"
                    0 _ => "Fizz"
                    _ 0 => "Buzz"
                    _ _ => num
            "#,
            ),
        );

        let src_map = unsafe {
            let map_ptr = CodespanSourceMap::codespan_new_source_map(
                &mut simple_map as *mut _ as *mut libc::c_void,
                Some(file_name),
                Some(source_code),
                Some(line_index),
                Some(line_range),
                None,
                None,
            );
            if map_ptr.is_null() {
                panic!("new_source_map returned NULL pointer");
            }
            &mut *map_ptr
        };

        let diagnostic = csr_diag::Diagnostic::error()
            .with_message("`case` clauses have incompatible types")
            .with_code("E0308")
            .with_labels(vec![
                csr_diag::Label::primary(file_id, 328..331)
                    .with_message("expected `String`, found `Nat`"),
                csr_diag::Label::secondary(file_id, 211..331)
                    .with_message("`case` clauses have incompatible types"),
                csr_diag::Label::secondary(file_id, 258..268)
                    .with_message("this is found to be of type `String`"),
                csr_diag::Label::secondary(file_id, 284..290)
                    .with_message("this is found to be of type `String`"),
                csr_diag::Label::secondary(file_id, 306..312)
                    .with_message("this is found to be of type `String`"),
                csr_diag::Label::secondary(file_id, 186..192)
                    .with_message("expected type `String` found here"),
            ])
            .with_notes(vec![unindent::unindent(
                "
            expected type `String`
                found type `Nat`
            ",
            )]);

        let writer =
            csr::term::termcolor::StandardStream::stderr(csr::term::termcolor::ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();
        codespan_reporting::term::emit_to_write_style(
            &mut writer.lock(),
            &config,
            src_map,
            &diagnostic,
        )
        .unwrap();

        unsafe { CodespanSourceMap::codespan_delete_source_map(src_map) };
    }

    unsafe extern "C" fn write_callback(
        _user_data: *mut libc::c_void,
        utf8_output: *const u8,
        output_length: libc::size_t,
    ) {
        let output =
            str::from_utf8(unsafe { slice::from_raw_parts(utf8_output, output_length) }).unwrap();
        println!("{output}");
    }

    #[test]
    fn diagnostic_test() {
        unsafe {
            backtrace_on_stack_overflow::enable();
        }

        let mut simple_map = SimpleMap::new();

        let file_id = simple_map.add(
            "FizzBuzz.fun".to_owned(),
            unindent(
                r#"
            module FizzBuzz where

            fizz₁ : Nat → String
            fizz₁ num = case (mod num 5) (mod num 3) of
                0 0 => "FizzBuzz"
                0 _ => "Fizz"
                _ 0 => "Buzz"
                _ _ => num

            fizz₂ : Nat → String
            fizz₂ num =
                case (mod num 5) (mod num 3) of
                    0 0 => "FizzBuzz"
                    0 _ => "Fizz"
                    _ 0 => "Buzz"
                    _ _ => num
            "#,
            ),
        );

        let src_map = unsafe {
            let map_ptr = CodespanSourceMap::codespan_new_source_map(
                &mut simple_map as *mut _ as *mut libc::c_void,
                Some(file_name),
                Some(source_code),
                Some(line_index),
                Some(line_range),
                None,
                None,
            );
            if map_ptr.is_null() {
                panic!("new_source_map returned NULL pointer");
            }
            &mut *map_ptr
        };

        let header_msg = "`case` clauses have incompatible types".as_bytes();
        let primary_msg = "expected `String`, found `Nat`".as_bytes();
        let diagnostic = unsafe {
            CodespanDiagnostic::codespan_new_diagnostic(
                SEVERITY_ERROR,
                header_msg.as_ptr(),
                header_msg.len(),
                Some(write_callback),
            )
        };

        let code = "-Wthis_is_an_error".as_bytes();
        let secondary_msg = "`case` clauses have incompatible types".as_bytes();
        unsafe {
            CodespanDiagnostic::codespan_diagnostic_set_config(
                diagnostic,
                DISPLAY_STYLE_RICH,
                CHAR_STYLE_FANCY,
                4,
            );

            CodespanDiagnostic::codespan_diagnostic_set_code(diagnostic, code.as_ptr(), code.len());

            CodespanDiagnostic::codespan_diagnostic_set_primary(
                diagnostic,
                file_id,
                328,
                331,
                primary_msg.as_ptr(),
                primary_msg.len(),
            );

            CodespanDiagnostic::codespan_diagnostic_add_secondary(
                diagnostic,
                file_id,
                211,
                231,
                secondary_msg.as_ptr(),
                secondary_msg.len(),
            );

            CodespanDiagnostic::codespan_write_diagnostic(ptr::null_mut(), diagnostic, src_map, 1);
        }

        unsafe {
            CodespanDiagnostic::codespan_delete_diagnostic(diagnostic);
            CodespanSourceMap::codespan_delete_source_map(src_map)
        }
    }
}
