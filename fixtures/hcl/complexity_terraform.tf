# Terraform complexity fixture for bscc.
# Exercises: conditional, for_expr, dynamic, count/for_each, boolean ops.

variable "env" {
  type    = string
  default = "dev"
}

# TODO: lock down the CIDR
locals {
  upper_names  = [for n in var.names : upper(n) if length(n) > 0]
  bucket_count = var.env == "prod" ? 3 : 1
}

resource "aws_s3_bucket" "data" {
  count  = local.bucket_count
  bucket = "data-${var.env}-${count.index}"
}

resource "aws_security_group" "web" {
  for_each = var.rules

  dynamic "ingress" {
    for_each = each.value.ports
    content {
      from_port   = ingress.value
      to_port     = ingress.value
      cidr_blocks = var.env == "prod" && each.value.public ? ["0.0.0.0/0"] : ["10.0.0.0/8"]
    }
  }
}

output "primary" {
  value = local.is_prod || var.env == "staging" ? aws_s3_bucket.data[0].id : null
}
