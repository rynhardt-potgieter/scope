; Import statements
(import_statement
  (import_clause
    (named_imports
      (import_specifier
        name: (identifier) @imported_name)))
  source: (string) @source) @import

; Call expressions — direct calls
(call_expression
  function: (identifier) @callee) @call

; Call expressions — member access calls
(call_expression
  function: (member_expression
    object: (identifier) @object
    property: (property_identifier) @method)) @member_call

; New expressions (instantiation)
(new_expression
  constructor: (identifier) @class_name) @instantiation

; Extends clause
(class_heritage
  (extends_clause
    value: (identifier) @base_class))

; Implements clause
(class_heritage
  (implements_clause
    (type_identifier) @interface_name))

; Type annotations referencing other types
(type_annotation
  (type_identifier) @type_ref)
