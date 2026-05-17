# Packer fixture for bscc.

source "amazon-ebs" "ubuntu" {
  ami_name      = "bscc-test-${formatdate("YYYYMMDD-hhmmss", timestamp())}"
  instance_type = "t3.micro"
  region        = "us-east-1"
  source_ami    = "ami-0c55b159cbfafe1f0"
  ssh_username  = "ubuntu"
}

build {
  sources = ["source.amazon-ebs.ubuntu"]

  provisioner "shell" {
    inline = [
      "sudo apt-get update",
      "sudo apt-get install -y nginx",
    ]
  }
}
