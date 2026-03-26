; Top-level function declarations (e.g. func main() {})
(function_declaration
  name: (identifier) @name) @definition

; Method declarations with receiver (e.g. func (s *Server) Handle() {})
(method_declaration
  name: (field_identifier) @name) @definition

; Type declarations — struct, interface, and type aliases
; (e.g. type PaymentService struct { ... })
(type_declaration
  (type_spec
    name: (type_identifier) @name) @definition)

; Const declarations (e.g. const MaxRetries = 3)
(const_declaration
  (const_spec
    name: (identifier) @name) @definition)
