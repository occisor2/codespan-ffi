use crate::{ByteIndex, FileId};
use codespan_reporting as csr;
use libc;
use std::{ops, slice};

pub type LineIndex = libc::size_t;
pub type LineRange = ops::Range<libc::size_t>;

pub type FileNameCallback = unsafe extern "C" fn(
    user_data: *mut libc::c_void,
    file_id: FileId,
    file_name_length: *mut libc::size_t,
) -> *const u8;
pub type SourceCodeCallBack = unsafe extern "C" fn(
    user_data: *mut libc::c_void,
    file_id: FileId,
    source_code_length: *mut libc::size_t,
) -> *const u8;
pub type LineIndexCallback = unsafe extern "C" fn(
    user_data: *mut libc::c_void,
    file_id: FileId,
    byte_index: ByteIndex,
) -> LineIndex;
pub type LineRangeCallback = unsafe extern "C" fn(
    user_data: *mut libc::c_void,
    file_id: FileId,
    line_index: LineIndex,
    start: *mut libc::size_t,
    end: *mut libc::size_t,
);
pub type LineNumberCallback = unsafe extern "C" fn(
    user_data: *mut libc::c_void,
    file_id: FileId,
    line_index: LineIndex,
) -> libc::size_t;
pub type ColumnNumberCallback = unsafe extern "C" fn(
    user_data: *mut libc::c_void,
    file_id: FileId,
    line_index: LineIndex,
    byte_index: ByteIndex,
) -> libc::size_t;

#[repr(C)]
pub struct CodespanSourceMap {
    user_data: *mut libc::c_void,
    file_name: FileNameCallback,
    source_code: SourceCodeCallBack,
    line_index: LineIndexCallback,
    line_range: LineRangeCallback,
    line_number: Option<LineNumberCallback>,
    column_number: Option<ColumnNumberCallback>,
}

impl CodespanSourceMap {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_new_source_map(
        user_data: *mut libc::c_void,
        file_name: Option<FileNameCallback>,
        source_code: Option<SourceCodeCallBack>,
        line_index: Option<LineIndexCallback>,
        line_range: Option<LineRangeCallback>,
        line_number: Option<LineNumberCallback>,
        column_number: Option<ColumnNumberCallback>,
    ) -> *mut Self {
        Box::into_raw(Box::new(Self {
            user_data,
            file_name: file_name.unwrap(),
            source_code: source_code.unwrap(),
            line_index: line_index.unwrap(),
            line_range: line_range.unwrap(),
            line_number,
            column_number,
        }))
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn codespan_delete_source_map(source_map: *mut Self) {
        if !source_map.is_null() {
            unsafe {
                drop(Box::from_raw(source_map));
            }
        }
    }
}

impl<'a> csr::files::Files<'a> for CodespanSourceMap {
    type FileId = FileId;
    type Name = &'a str;
    type Source = &'a str;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, csr::files::Error> {
        Ok(str::from_utf8({
            let mut file_length = 0;
            let file_name_buffer =
                unsafe { (self.file_name)(self.user_data, id, &mut file_length) };

            if file_name_buffer.is_null() {
                panic!("file name buffer is NULL in CodespanSourceMap file_name");
            }

            unsafe { slice::from_raw_parts(file_name_buffer, file_length) }
        })
        .unwrap())
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, csr::files::Error> {
        Ok(str::from_utf8({
            let mut source_code_length = 0;
            let source_code_buffer =
                unsafe { (self.source_code)(self.user_data, id, &mut source_code_length) };

            if source_code_buffer.is_null() {
                panic!("source code buffer is NULL in CodespanSourceMap file_name");
            }

            unsafe { slice::from_raw_parts(source_code_buffer, source_code_length) }
        })
        .unwrap())
    }

    fn line_index(
        &'a self,
        id: Self::FileId,
        byte_index: ByteIndex,
    ) -> Result<usize, csr::files::Error> {
        Ok(unsafe { (self.line_index)(self.user_data, id, byte_index) })
    }

    fn line_range(
        &'a self,
        id: Self::FileId,
        line_index: LineIndex,
    ) -> Result<LineRange, csr::files::Error> {
        let (mut start, mut end) = (0, 0);
        unsafe { (self.line_range)(self.user_data, id, line_index, &mut start, &mut end) };
        Ok(start..end)
    }

    fn line_number(
        &'a self,
        id: Self::FileId,
        line_index: LineIndex,
    ) -> Result<usize, codespan_reporting::files::Error> {
        if let Some(callback) = self.line_number {
            Ok(unsafe { (callback)(self.user_data, id, line_index) })
        } else {
            Ok(line_index + 1)
        }
    }

    fn column_number(
        &'a self,
        id: Self::FileId,
        line_index: LineIndex,
        byte_index: ByteIndex,
    ) -> Result<usize, codespan_reporting::files::Error> {
        if let Some(callback) = self.column_number {
            Ok(unsafe { (callback)(self.user_data, id, line_index, byte_index) })
        } else {
            let source = self.source(id)?;
            let line_range = self.line_range(id, line_index)?;
            let column_index = csr::files::column_index(source.as_ref(), line_range, byte_index);

            Ok(column_index + 1)
        }
    }
}
