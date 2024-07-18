module "database" {
  source = "../../modules/firestore"
}

module "registry" {
  source = "../../modules/artifact_registry"
}

variable "image_tag" {
  type     = string
  nullable = false
}

module "app" {
  source    = "../../modules/cloud_run"
  image_tag = var.image_tag
}

output "app_uri" {
  value = module.app.uri
}
