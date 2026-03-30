packer {
  required_plugins {
    digitalocean = {
      version = ">= 1.1.0"
      source  = "github.com/digitalocean/digitalocean"
    }
  }
}

variable "do_token" {
  type      = string
  sensitive = true
  default   = env("DIGITALOCEAN_TOKEN")
}

variable "image_name" {
  type    = string
  default = "clasp-relay"
}

variable "relay_version" {
  type    = string
  default = "latest"
}

variable "region" {
  type    = string
  default = "nyc3"
}

source "digitalocean" "clasp" {
  api_token     = var.do_token
  image         = "ubuntu-22-04-x64"
  region        = var.region
  size          = "s-1vcpu-1gb"
  ssh_username  = "root"
  snapshot_name = "${var.image_name}-${formatdate("YYYYMMDD-hhmm", timestamp())}"
  snapshot_regions = [
    "nyc1", "nyc3", "sfo3", "ams3", "sgp1", "lon1", "fra1", "tor1", "blr1", "syd1"
  ]
  tags = ["clasp", "marketplace"]
}

build {
  sources = ["source.digitalocean.clasp"]

  # Create target directory and copy config files
  provisioner "shell" {
    inline = ["mkdir -p /tmp/clasp-files"]
  }

  provisioner "file" {
    source      = "files/clasp-setup"
    destination = "/tmp/clasp-files/clasp-setup"
  }

  provisioner "file" {
    source      = "files/docker-compose.yml.tpl"
    destination = "/tmp/clasp-files/docker-compose.yml.tpl"
  }

  provisioner "file" {
    source      = "files/Caddyfile.tpl"
    destination = "/tmp/clasp-files/Caddyfile.tpl"
  }

  provisioner "file" {
    source      = "files/application.info"
    destination = "/tmp/clasp-files/application.info"
  }

  provisioner "file" {
    source      = "files/99-one-click"
    destination = "/tmp/clasp-files/99-one-click"
  }

  # Install Docker
  provisioner "shell" {
    script = "scripts/01-docker.sh"
  }

  # Install CLASP relay and config
  provisioner "shell" {
    script = "scripts/02-clasp.sh"
    environment_vars = [
      "RELAY_VERSION=${var.relay_version}"
    ]
  }

  # Configure firewall
  provisioner "shell" {
    script = "scripts/03-ufw.sh"
  }

  # DigitalOcean image cleanup (required for marketplace)
  provisioner "shell" {
    script = "scripts/90-cleanup.sh"
  }
}
