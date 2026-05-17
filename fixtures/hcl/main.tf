# Terraform fixture for bscc.

variable "region" {
  type    = string
  default = "us-east-1"
}

resource "aws_s3_bucket" "data" {
  bucket = "example-${var.region}"

  tags = {
    Environment = "test"
  }
}

output "bucket_name" {
  value = aws_s3_bucket.data.bucket
}
