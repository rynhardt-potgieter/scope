; Use declarations — simple path (e.g. use std::io)
(use_declaration
  argument: (scoped_identifier) @imported_name) @import

; Use declarations — use list (e.g. use std::{io, fs})
(use_declaration
  argument: (use_as_clause
    path: (scoped_identifier) @imported_name)) @aliased_import

; Direct call expressions (e.g. process_payment(...))
(call_expression
  function: (identifier) @callee) @call

; Scoped call expressions (e.g. PaymentService::new(...))
(call_expression
  function: (scoped_identifier) @callee) @scoped_call

; Method call expressions (e.g. self.client.charge(...))
(call_expression
  function: (field_expression
    field: (field_identifier) @method)) @method_call

; Macro invocations (e.g. println!(...), vec![...])
(macro_invocation
  macro: (identifier) @macro_name) @macro_call

; Scoped macro invocations (e.g. std::println!(...))
(macro_invocation
  macro: (scoped_identifier) @macro_name) @scoped_macro_call

; Type references in struct fields
(field_declaration
  type: (type_identifier) @type_ref) @field_type

; Type references in function parameters
(parameter
  type: (type_identifier) @type_ref) @param_type

; Type references in function return types
(function_item
  return_type: (type_identifier) @type_ref) @return_type_ref

; Match arm — struct pattern with scoped type (e.g. PaymentResult::Success { .. })
(match_arm
  pattern: (match_pattern
    (struct_pattern
      type: (scoped_type_identifier
        name: (type_identifier) @variant_ref)))) @match_struct_ref

; Match arm — tuple struct pattern (e.g. PaymentMethod::CreditCard(details))
(match_arm
  pattern: (match_pattern
    (tuple_struct_pattern
      type: (scoped_identifier) @variant_ref))) @match_tuple_ref
