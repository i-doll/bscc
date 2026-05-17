// Generic HCL — Vault listener config with conditional bits.

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = var.dev_mode && !var.require_tls ? 1 : 0
}

storage "consul" {
  path          = "vault/${var.cluster}"
  redirect_addr = var.use_redirect || var.fallback ? "http://primary:8200" : null
}
