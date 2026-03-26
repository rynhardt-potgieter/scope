; Class declarations
(class_declaration
  name: (identifier) @name) @definition

; Interface declarations
(interface_declaration
  name: (identifier) @name) @definition

; Enum declarations
(enum_declaration
  name: (identifier) @name) @definition

; Record declarations (Java 16+)
(record_declaration
  name: (identifier) @name) @definition

; Method declarations
(method_declaration
  name: (identifier) @name) @definition

; Constructor declarations
(constructor_declaration
  name: (identifier) @name) @definition

; Field declarations
(field_declaration
  declarator: (variable_declarator
    name: (identifier) @name)) @definition

; Annotation type declarations
(annotation_type_declaration
  name: (identifier) @name) @definition
