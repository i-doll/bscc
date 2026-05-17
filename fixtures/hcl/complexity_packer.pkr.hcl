# Packer complexity fixture.

source "amazon-ebs" "ubuntu" {
  instance_type = var.is_prod ? "t3.large" : "t3.micro"
  ami_name      = "img-${formatdate("YYYYMMDD", timestamp())}"
  tags          = { for k, v in var.tags : k => upper(v) }
}

# FIXME: parameterize regions
build {
  sources = ["source.amazon-ebs.ubuntu"]
  provisioner "shell" {
    inline = var.run_tests ? ["./test.sh"] : ["echo skip"]
  }
}
