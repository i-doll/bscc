# OpenTofu fixture.

terraform {
  required_version = ">= 1.6.0"
}

variable "regions" {
  type    = list(string)
  default = ["us-east-1"]
}
