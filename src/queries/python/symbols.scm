; Function definitions (both decorated and undecorated — decorators are
; extracted in the Python plugin by walking up to a decorated_definition parent)
(function_definition
  name: (identifier) @name) @definition

; Class definitions (both decorated and undecorated)
(class_definition
  name: (identifier) @name) @definition
