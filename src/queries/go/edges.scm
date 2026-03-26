; Import specs (e.g. import "fmt" or import alias "fmt")
(import_spec
  path: (interpreted_string_literal) @source) @import

; Direct function calls (e.g. processPayment(...))
(call_expression
  function: (identifier) @callee) @call

; Selector expression calls — method and qualified calls
; (e.g. s.Handle(), fmt.Println())
(call_expression
  function: (selector_expression
    operand: (identifier) @object
    field: (field_identifier) @method)) @method_call

; Struct embedding — a field_declaration with a type but no field name
; (e.g. type Server struct { Logger })
(field_declaration
  type: (type_identifier) @base_type) @embedding
