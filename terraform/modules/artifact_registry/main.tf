// <https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/google_project_service>
resource "google_project_service" "cloudresourcemanager" {
  service = "cloudresourcemanager.googleapis.com"
}

resource "google_project_service" "artifact_registry" {
  service    = "artifactregistry.googleapis.com"
  depends_on = [google_project_service.cloudresourcemanager]
}

// <https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/artifact_registry_repository>
resource "google_artifact_registry_repository" "date_line_charts" {
  location      = "asia-northeast2"
  repository_id = "date-line-charts"
  format        = "DOCKER"
  depends_on    = [google_project_service.artifact_registry]
}
