; Standalone function definitions
(function_item
  name: (identifier) @name) @definition

; Struct definitions
(struct_item
  name: (type_identifier) @name) @definition

; Enum definitions
(enum_item
  name: (type_identifier) @name) @definition

; Trait definitions
(trait_item
  name: (type_identifier) @name) @definition

; Type alias definitions
(type_item
  name: (type_identifier) @name) @definition

; Const definitions
(const_item
  name: (identifier) @name) @definition

; Static definitions
(static_item
  name: (identifier) @name) @definition
