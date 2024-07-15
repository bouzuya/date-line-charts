// <https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/firestore_database>
resource "google_firestore_database" "default" {
  name            = "(default)"
  location_id     = "asia-northeast2"
  type            = "FIRESTORE_NATIVE"
  deletion_policy = "DELETE"
}

// <https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/firestore_index>
resource "google_firestore_index" "index1" {
  collection = "events"
  database   = google_firestore_database.default.name

  fields {
    field_path = "stream_id"
    order      = "ASCENDING"
  }
  fields {
    field_path = "version"
    order      = "ASCENDING"
  }
  fields {
    field_path = "__name__"
    order      = "ASCENDING"
  }
}
