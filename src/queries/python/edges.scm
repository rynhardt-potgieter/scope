; Import statements (e.g. import os)
(import_statement
  name: (dotted_name) @imported_name) @import

; From-import statements (e.g. from os.path import join)
(import_from_statement
  module_name: (dotted_name) @source
  name: (dotted_name) @imported_name) @from_import

; Direct function calls (e.g. foo())
(call
  function: (identifier) @callee) @call

; Attribute calls / method calls (e.g. self.foo(), obj.bar())
(call
  function: (attribute
    object: (identifier) @object
    attribute: (identifier) @method)) @method_call

; Class inheritance — argument_list inside class_definition
(class_definition
  superclasses: (argument_list
    (identifier) @base_class)) @extends
