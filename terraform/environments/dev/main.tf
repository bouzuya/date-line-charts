module "database" {
  source = "../../modules/firestore"
}

module "registry" {
  source = "../../modules/artifact_registry"
}

module "app" {
  source    = "../../modules/cloud_run"
  image_tag = var.image_tag
}
