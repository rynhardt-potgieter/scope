; Function declarations
(function_declaration
  name: (identifier) @name
  parameters: (formal_parameters) @params
  return_type: (type_annotation)? @return_type) @definition

; Arrow functions assigned to const/let
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: [(arrow_function) (function_expression)] @definition))

; Class declarations
(class_declaration
  name: (type_identifier) @name) @definition

; Method definitions inside classes
(method_definition
  name: (property_identifier) @name
  parameters: (formal_parameters) @params
  return_type: (type_annotation)? @return_type) @definition

; Interface declarations
(interface_declaration
  name: (type_identifier) @name) @definition

; Enum declarations
(enum_declaration
  name: (identifier) @name) @definition

; Type alias declarations
(type_alias_declaration
  name: (type_identifier) @name) @definition

; Property declarations in classes (fields)
(public_field_definition
  name: (property_identifier) @name) @definition

; Enum member definitions (with initializer)
(enum_assignment
  name: (property_identifier) @name) @definition

; Enum member definitions (without initializer — bare identifiers in enum body)
(enum_body
  name: (property_identifier) @name @definition)
