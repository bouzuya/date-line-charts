terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.37.0"
    }
  }
}

// <https://registry.terraform.io/providers/hashicorp/google/latest/docs/guides/provider_reference>
provider "google" {
  credentials = "../../credentials/dev.json"
  project     = local.project_id
  region      = "asia-northeast2"
}

terraform {
  // <https://developer.hashicorp.com/terraform/language/settings/backends/configuration>
  // <https://developer.hashicorp.com/terraform/language/settings/backends/gcs>
  backend "gcs" {
    bucket      = "bouzuya-terraform-state-dev"
    credentials = "../../credentials/dev.json"
  }
}
