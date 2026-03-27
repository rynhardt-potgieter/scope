; Import declarations
(import_declaration
  (scoped_identifier) @imported_name) @import

; Method invocations via member access (e.g. service.processPayment(...))
(method_invocation
  object: (identifier) @object
  name: (identifier) @method) @member_call

; Direct method invocations (e.g. processPayment(...))
(method_invocation
  name: (identifier) @callee) @call

; this.method() calls — `this` is a keyword node, not an identifier
(method_invocation
  object: (this)
  name: (identifier) @method) @this_call

; Object creation (e.g. new PaymentService(...))
(object_creation_expression
  type: (type_identifier) @class_name) @instantiation

; Superclass in class declaration (extends)
(class_declaration
  (superclass
    (type_identifier) @base_type)) @extends

; Super interfaces in class declaration (implements)
(class_declaration
  (super_interfaces
    (type_list
      (type_identifier) @base_type))) @class_implements

; Super interfaces in interface declaration (extends)
(interface_declaration
  (extends_interfaces
    (type_list
      (type_identifier) @base_type))) @interface_extends

; Type references in field declarations
(field_declaration
  type: (type_identifier) @type_ref) @field_type_ref

; Type references in method parameters
(formal_parameter
  type: (type_identifier) @type_ref) @param_type_ref

; super.method() calls — parent class method invocation
(method_invocation
  object: (super)
  name: (identifier) @method) @super_call

; Switch case label referencing an enum constant (e.g. case SUCCESS:)
(switch_label
  (identifier) @variant_ref) @switch_case_ref
