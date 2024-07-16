module "database" {
  source = "../../modules/firestore"
}

module "registry" {
  source = "../../modules/artifact_registry"
}
