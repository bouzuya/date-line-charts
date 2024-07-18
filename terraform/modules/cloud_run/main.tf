resource "google_project_service" "run" {
  service = "run.googleapis.com"
}

# <https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/cloud_run_v2_service>
resource "google_cloud_run_v2_service" "web" {
  name       = "cloudrun-service"
  location   = "asia-northeast2"
  ingress    = "INGRESS_TRAFFIC_ALL"
  depends_on = [google_project_service.run]

  template {
    containers {
      args  = ["server"]
      image = "asia-northeast2-docker.pkg.dev/bouzuya-terraform/date-line-charts/date-line-charts:${var.image_tag}"
    }
    # service_account
    scaling {
      max_instance_count = 1
      min_instance_count = 0
    }
  }
}

# <https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/cloud_run_v2_service_iam>
resource "google_cloud_run_v2_service_iam_member" "all_users" {
  name   = google_cloud_run_v2_service.web.name
  member = "allUsers"
  role   = "roles/run.invoker"
}
