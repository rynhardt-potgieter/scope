; Using directives
(using_directive
  name: (identifier) @imported_name) @import

; Using directives with qualified names
(using_directive
  (qualified_name) @imported_name) @qualified_import

; Method invocations via member access (e.g. _logger.Info(...))
(invocation_expression
  function: (member_access_expression
    expression: (identifier) @object
    name: (identifier) @method)) @member_call

; Direct invocations (e.g. DoSomething(...))
(invocation_expression
  function: (identifier) @callee) @call

; Object creation (e.g. new PaymentService(...))
(object_creation_expression
  type: (identifier) @class_name) @instantiation

; Base types in class/interface/struct inheritance
(base_list
  (identifier) @base_type) @inheritance

; Type references in qualified base types
(base_list
  (qualified_name) @base_type) @qualified_inheritance
