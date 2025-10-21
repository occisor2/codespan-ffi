#pragma once

#include <stdint.h>
#include <stddef.h>

typedef size_t codespan_file_id;
typedef size_t codespan_byte_index;
typedef size_t codespan_line_index;

typedef const char *(*codespan_file_name_callback)(void *user_data,
                                                   codespan_file_id id,
                                                   size_t *name_len);
typedef const char *(*codespan_source_code_callback)(codespan_file_id id,
                                                     size_t *source_len);
typedef codespan_line_index (*codespan_line_index_callback)(codespan_file_id id,
                                                            codespan_byte_index index);
typedef void (*codespan_line_range_callback)(void *user_data,
                                             codespan_file_id id,
                                             codespan_line_index index,
                                             codespan_byte_index *start,
                                             codespan_byte_index *end);
typedef size_t (*codespan_line_number_callback)(void *user_data,
                                                codespan_file_id id,
                                                codespan_line_index index);
typedef size_t (*codespan_column_number_callback)(void *user_data,
                                                  codespan_file_id id,
                                                  codespan_line_index index);

typedef struct CodespanSourceMap CodespanSourceMap;

CodespanSourceMap *codespan_new_source_map(void *user_data,
                                           codespan_file_name_callback,
                                           codespan_source_code_callback,
                                           codespan_line_index_callback,
                                           codespan_line_range_callback,
                                           codespan_line_number_callback,
                                           codespan_column_number_callback);

void codespan_delete_source_map(CodespanSourceMap* source_map);

typedef size_t codespan_severity;
#define CODESPAN_SEVERITY_HELP 0
#define CODESPAN_SEVERITY_NOTE 1
#define CODESPAN_SEVERITY_WARNING 2
#define CODESPAN_SEVERITY_ERROR 3
#define CODESPAN_SEVERITY_BUG 4

typedef void(*codespan_writer_callback)(void *user_data,
                                        const uint8_t *utf8_output,
                                        size_t output_len);

typedef struct CodespanDiagnostic CodespanDiagnostic;

CodespanDiagnostic *codespan_new_diagnostic(codespan_severity severity,
                                            const uint8_t *msg,
                                            size_t msg_len,
                                            codespan_writer_callback);
void codespan_delete_diagnostic(CodespanDiagnostic *diagnostic);
void codespan_write_diagnostic(void* user_data,
                               const CodespanDiagnostic *diagnostic,
                               const CodespanSourceMap *src_map,
                               uint8_t color);
void codespan_diagnostic_set_code(const CodespanDiagnostic* diagnostic,
                                  const uint8_t *code,
                                  size_t code_len);
void codespan_diagnostic_set_primary(CodespanDiagnostic *diagnostic,
                                     codespan_file_id file_id,
                                     codespan_byte_index start,
                                     codespan_byte_index end,
                                     const uint8_t *msg,
                                     size_t msg_len);
void codespan_diagnostic_add_secondary(CodespanDiagnostic *diagnostic,
                                       codespan_file_id file_id,
                                       codespan_byte_index start,
                                       codespan_byte_index end,
                                       const uint8_t *msg,
                                       size_t msg_len);
void codespan_diagnostic_add_note(CodespanDiagnostic *diagnostic,
                                  const uint8_t *msg,
                                  size_t msg_len);

typedef size_t codespan_display_style;
#define CODESPAN_DISPLAY_STYLE_RICH 0;
#define CODESPAN_DISPLAY_STYLE_MEDIUM 1;
#define CODESPAN_DISPLAY_STYLE_SHORT 2;

typedef size_t codespan_char_style;
#define CODESPAN_CHAR_STYLE_FANCY 0;
#define CODESPAN_CHAR_STYLE_ASCII 1;

void codespan_diagnostic_set_config(CodespanDiagnostic *diagnostic,
                                    codespan_display_style display_style,
                                    codespan_char_style char_style,
                                    size_t tab_width);
